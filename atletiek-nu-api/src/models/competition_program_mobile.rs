//! Parser for the mobile competition program endpoint:
//! `athleteapp.php?page=event&do=program&event_id=<id>`
//!
//! This endpoint is NOT protected by Cloudflare Turnstile.
//! It returns the full program: gender groups → age categories → event groups → events.
//! More detailed than the `do=get` categories table.

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileCompetitionProgram {
    pub gender_groups: Vec<GenderGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenderGroup {
    /// "Men" or "Women"
    pub gender: String,
    pub categories: Vec<ProgramCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramCategory {
    /// e.g. "U18 Men", "Senior Women", "U10 Boys"
    pub name: String,
    pub event_groups: Vec<ProgramEventGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramEventGroup {
    /// e.g. "Track Events", "Field Events", "Combined-events"
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<String>,
    pub events: Vec<String>,
}

pub fn parse(html: Html) -> anyhow::Result<MobileCompetitionProgram> {
    let page_sel = Selector::parse(".page-content.page-event-program").unwrap();
    let group_title_sel = Selector::parse(":scope > ul > li.list-group-title").unwrap();
    let accordion_sel = Selector::parse(":scope > ul > li.accordion-item").unwrap();
    let item_title_sel = Selector::parse(".item-title").unwrap();
    let content_sel = Selector::parse(".accordion-item-content").unwrap();
    let inner_group_sel = Selector::parse(".list-group").unwrap();
    let inner_title_sel = Selector::parse("li.list-group-title").unwrap();
    let inner_item_sel = Selector::parse("li:not(.list-group-title)").unwrap();
    let icon_mars_sel = Selector::parse("i.fa-mars").unwrap();
    let icon_venus_sel = Selector::parse("i.fa-venus").unwrap();
    let price_sel = Selector::parse(".meerkamp-kosten-label").unwrap();

    // The page has a specific structure - try parsing from page-content first
    // If that fails, parse the whole document
    let root = html
        .select(&page_sel)
        .next();

    // Collect all top-level list-groups from the HTML
    // The structure is: div.list > div.list-group (with gender title) > ul > li.accordion-item
    let all_groups_sel = Selector::parse("div.list:not(.inset) > .list-group").unwrap();

    let groups: Vec<_> = if let Some(root_el) = root {
        root_el.select(&all_groups_sel).collect()
    } else {
        html.select(&all_groups_sel).collect()
    };

    let mut gender_groups = Vec::new();

    for group in groups {
        // Check if this is a gender group (has mars/venus icon in title)
        let title_el = match group.select(&group_title_sel).next() {
            Some(t) => t,
            None => continue,
        };

        let has_mars = title_el.select(&icon_mars_sel).next().is_some();
        let has_venus = title_el.select(&icon_venus_sel).next().is_some();

        if !has_mars && !has_venus {
            // This is a sub-group (Track Events, Field Events, etc.), not a gender group
            continue;
        }

        let gender = if has_mars {
            "Men".to_string()
        } else {
            "Women".to_string()
        };

        let mut categories = Vec::new();

        for accordion in group.select(&accordion_sel) {
            let cat_name = match accordion.select(&item_title_sel).next() {
                Some(t) => t.text().collect::<String>().trim().to_string(),
                None => continue,
            };

            let mut event_groups = Vec::new();

            if let Some(content) = accordion.select(&content_sel).next() {
                for inner_group in content.select(&inner_group_sel) {
                    let label_el = match inner_group.select(&inner_title_sel).next() {
                        Some(t) => t,
                        None => continue,
                    };

                    // Extract label (clean up price and capacity text)
                    let label_text = label_el
                        .text()
                        .collect::<String>();

                    // Price from dedicated span
                    let price = label_el
                        .select(&price_sel)
                        .next()
                        .map(|p| p.text().collect::<String>().trim().to_string());

                    // Clean label: remove price text and capacity info
                    let label = label_text
                        .trim()
                        .to_string();

                    // Extract just the event group name (before price/capacity)
                    let clean_label = if let Some(price_str) = &price {
                        label
                            .split(price_str.as_str())
                            .next()
                            .unwrap_or(&label)
                            .trim()
                            .to_string()
                    } else {
                        // Try to split at common capacity indicators
                        let label_clean = label
                            .split(|c: char| c.is_ascii_digit())
                            .next()
                            .unwrap_or(&label)
                            .trim()
                            .to_string();
                        if label_clean.is_empty() {
                            label.clone()
                        } else {
                            label_clean
                        }
                    };

                    // Capacity info (text after the label)
                    let capacity = {
                        let full = label_text.trim().to_string();
                        let after_label = if let Some(price_str) = &price {
                            full.split(price_str.as_str())
                                .nth(1)
                                .unwrap_or("")
                                .trim()
                                .to_string()
                        } else {
                            String::new()
                        };
                        if after_label.is_empty() {
                            None
                        } else {
                            Some(after_label)
                        }
                    };

                    // Events
                    let events: Vec<String> = inner_group
                        .select(&inner_item_sel)
                        .filter_map(|li| {
                            li.select(&item_title_sel)
                                .next()
                                .map(|t| t.text().collect::<String>().trim().to_string())
                        })
                        .filter(|s| !s.is_empty())
                        .collect();

                    if !events.is_empty() || !clean_label.is_empty() {
                        event_groups.push(ProgramEventGroup {
                            label: clean_label,
                            price,
                            capacity,
                            events,
                        });
                    }
                }
            }

            if !cat_name.is_empty() {
                categories.push(ProgramCategory {
                    name: cat_name,
                    event_groups,
                });
            }
        }

        if !categories.is_empty() {
            gender_groups.push(GenderGroup {
                gender,
                categories,
            });
        }
    }

    Ok(MobileCompetitionProgram { gender_groups })
}
