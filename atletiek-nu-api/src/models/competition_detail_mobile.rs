//! Parser for the mobile competition detail endpoint:
//! `athleteapp.php?page=event&do=get&event_id=<id>`
//!
//! This endpoint is NOT protected by Cloudflare Turnstile.
//! It returns competition metadata (name, date, location, description)
//! and a categories table showing which age groups can participate
//! with their available events and pricing.

use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

const REGEX_COUNTRY_CODE: &str = r#"/([a-z]{2})\.png"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileCompetitionDetail {
    pub name: String,
    pub date: String,
    pub location: String,
    pub country_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub federation_recognized: bool,
    pub categories: Vec<CompetitionCategoryGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionCategoryGroup {
    /// Age group names, e.g. ["Senior Men", "U20 Men", "U18 Men"]
    pub age_groups: Vec<String>,
    /// Event groups available for these categories
    pub event_groups: Vec<CompetitionEventGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionEventGroup {
    /// e.g. "Sprint", "Mid. dist.", "Throw", "Jump", "Combined-events"
    pub label: String,
    /// Event names, e.g. ["100m", "200m"]
    pub events: Vec<String>,
    /// Price per event, e.g. "€5.00"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    /// Capacity info extracted from tooltips
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub capacity_info: Vec<String>,
}

pub fn parse(html: Html) -> anyhow::Result<MobileCompetitionDetail> {
    let name_sel = Selector::parse("h2.no-margin").unwrap();
    let header_sel = Selector::parse(".event-header-container").unwrap();
    let header_p_sel = Selector::parse("p").unwrap();
    let flag_sel = Selector::parse("img.flagicon").unwrap();
    let desc_sel = Selector::parse(".event-description").unwrap();
    let badge_sel = Selector::parse("i.atletiekunie").unwrap();
    let table_sel = Selector::parse("table.categorieenonderdelenpakket").unwrap();
    let tr_sel = Selector::parse("tr").unwrap();
    let td_sel = Selector::parse("td").unwrap();
    let tipped_sel = Selector::parse("span.tipped").unwrap();

    let re_country = Regex::new(REGEX_COUNTRY_CODE).unwrap();

    // Name
    let name = html
        .select(&name_sel)
        .next()
        .map(|el| {
            el.text()
                .filter(|t| !t.trim().is_empty())
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        })
        .unwrap_or_default();

    // Header: location and date
    let mut location = String::new();
    let mut date = String::new();
    let mut country_code = String::new();

    if let Some(header) = html.select(&header_sel).next() {
        let paragraphs: Vec<_> = header.select(&header_p_sel).collect();

        if let Some(loc_p) = paragraphs.first() {
            location = loc_p
                .text()
                .collect::<String>()
                .trim()
                .to_string();

            if let Some(img) = loc_p.select(&flag_sel).next() {
                if let Some(src) = img.value().attr("src") {
                    if let Some(cap) = re_country.captures(src) {
                        country_code = cap[1].to_string();
                    }
                }
            }
        }

        if let Some(date_p) = paragraphs.get(1) {
            date = date_p
                .text()
                .collect::<String>()
                .trim()
                // Clean up template markers like {{_ "u"}}
                .replace("{{_ \"u\"}}", "")
                .trim()
                .to_string();
        }
    }

    // Description
    let description = html
        .select(&desc_sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty());

    // Federation recognized
    let federation_recognized = html.select(&badge_sel).next().is_some();

    // Categories table
    let mut categories = Vec::new();

    if let Some(table) = html.select(&table_sel).next() {
        let rows: Vec<_> = table.select(&tr_sel).collect();
        let mut i = 0;

        // Skip header row
        if !rows.is_empty() {
            i = 1;
        }

        while i < rows.len() {
            let cells: Vec<_> = rows[i].select(&td_sel).collect();
            if cells.is_empty() {
                i += 1;
                continue;
            }

            // Check if first cell has rowspan (category group start)
            let first_cell = &cells[0];
            let has_categories = first_cell.select(&tipped_sel).next().is_some()
                && first_cell
                    .value()
                    .attr("rowspan")
                    .is_some();

            if has_categories || (cells.len() >= 4 && first_cell.select(&tipped_sel).next().is_some()) {
                let age_groups: Vec<String> = first_cell
                    .select(&tipped_sel)
                    .map(|s| {
                        s.value()
                            .attr("title")
                            .unwrap_or("")
                            .trim()
                            .to_string()
                    })
                    .filter(|s| !s.is_empty())
                    .collect();

                let rowspan: usize = first_cell
                    .value()
                    .attr("rowspan")
                    .and_then(|r| r.parse().ok())
                    .unwrap_or(1);

                let mut event_groups = Vec::new();

                // Parse this row's event (cells offset by 1 because of category cell)
                parse_event_row(&cells[1..], &tipped_sel, &mut event_groups);

                // Parse subsequent rows that belong to this rowspan
                for j in 1..rowspan {
                    if i + j < rows.len() {
                        let sub_cells: Vec<_> = rows[i + j].select(&td_sel).collect();
                        if !sub_cells.is_empty() {
                            parse_event_row(&sub_cells, &tipped_sel, &mut event_groups);
                        }
                    }
                }

                categories.push(CompetitionCategoryGroup {
                    age_groups,
                    event_groups,
                });

                i += rowspan;
            } else {
                i += 1;
            }
        }
    }

    Ok(MobileCompetitionDetail {
        name,
        date,
        location,
        country_code,
        description,
        federation_recognized,
        categories,
    })
}

fn parse_event_row(
    cells: &[scraper::ElementRef],
    tipped_sel: &Selector,
    event_groups: &mut Vec<CompetitionEventGroup>,
) {
    if cells.is_empty() {
        return;
    }

    // First cell: label (e.g. "Sprint:", "Mid. dist.:", "Combined-events:")
    let label = cells[0]
        .text()
        .collect::<String>()
        .trim()
        .trim_end_matches(':')
        .to_string();

    // Second cell (if exists): events + capacity tooltips
    let mut events = Vec::new();
    let mut capacity_info = Vec::new();

    if cells.len() > 1 {
        // Events from tipped spans (tooltip = event full name)
        let tipped: Vec<_> = cells[1].select(tipped_sel).collect();

        // Check if the tipped spans contain event names or capacity info
        for span in &tipped {
            let title = span
                .value()
                .attr("title")
                .unwrap_or("")
                .trim()
                .to_string();

            if title.contains("starting places") || title.contains("reserve") {
                capacity_info.push(title);
            } else if !title.is_empty() {
                events.push(title);
            }
        }

        // If no event names from tooltips, try the cell text
        if events.is_empty() {
            let text = cells[1].text().collect::<String>();
            let text = text.trim();
            if !text.is_empty()
                && !text.contains("starting places")
                && !text.contains("reserve")
            {
                // Split by comma for abbreviated event lists like "100m, 200m"
                for part in text.split(',') {
                    let part = part.trim();
                    if !part.is_empty() {
                        events.push(part.to_string());
                    }
                }
            }
        }

        // Also check cells[1] for capacity info if not found yet
        if capacity_info.is_empty() {
            for span in cells[1].select(tipped_sel) {
                let title = span
                    .value()
                    .attr("title")
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if title.contains("starting places") || title.contains("reserve") {
                    capacity_info.push(title);
                }
            }
        }
    }

    // Price from last cell
    let price = cells
        .last()
        .map(|c| c.text().collect::<String>().trim().to_string())
        .filter(|s| s.contains('€') || s.contains("free") || s.contains("gratis"));

    if !label.is_empty() || !events.is_empty() {
        event_groups.push(CompetitionEventGroup {
            label,
            events,
            price,
            capacity_info,
        });
    }
}
