//! Parser for the mobile endpoint: `athleteapp.php?page=athlete&do=records`
//!
//! This endpoint is NOT protected by Cloudflare Turnstile and returns richer
//! data than the desktop profile page, including per-event performance history
//! with wind speed, location, country, and delta progression.
//!
//! HTML structure:
//!   - First `div.list-athlete-results > ul` = Personal Records list
//!   - `div.tabs.graphTabs > div[id^=tab-eventgraph-]` = history tabs per event
//!   - Each history tab contains a `div.list-athlete-results > ul` with performances

use log::trace;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

const REGEX_WIND: &str = r#"([+-]?\d+[.,]\d+)\s*m/s"#;
const REGEX_COUNTRY_CODE: &str = r#"/([a-z]{2})\.png"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileAthleteProfile {
    pub name: String,
    pub personal_bests: Vec<MobilePersonalBest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobilePersonalBest {
    pub event: String,
    pub performance: String,
    pub performance_value: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wind_speed: Option<f32>,
    pub hand_measured: bool,
    pub location: String,
    pub country_code: String,
    pub date: String,
    pub not_important: bool,
    pub indoor: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<String>,
    pub history: Vec<PerformanceHistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHistoryEntry {
    pub performance: String,
    pub performance_value: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wind_speed: Option<f32>,
    pub hand_measured: bool,
    pub location: String,
    pub country_code: String,
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<String>,
}

/// Data extracted from a single `<li>` element (used for both PRs and history entries).
struct ParsedLi {
    event_name: String,
    performance: String,
    performance_value: f32,
    wind_speed: Option<f32>,
    hand_measured: bool,
    location: String,
    country_code: String,
    date: String,
    not_important: bool,
    indoor: bool,
    attribute: Option<String>,
}

fn parse_wind(text: &str) -> Option<f32> {
    let re = Regex::new(REGEX_WIND).unwrap();
    re.captures(text).and_then(|cap| {
        cap[1].replace(",", ".").parse::<f32>().ok()
    })
}

fn extract_country_code(html: &str) -> String {
    let re = Regex::new(REGEX_COUNTRY_CODE).unwrap();
    re.captures(html)
        .map(|cap| cap[1].to_string())
        .unwrap_or_default()
}

fn parse_performance_value(text: &str) -> f32 {
    let text = text.trim().trim_end_matches('h');
    if text.is_empty() || text == "DNF" || text == "DNS" || text == "DQ" || text == "NM" || text == "NH" {
        return 0.0;
    }

    let parts: Vec<&str> = text.split(':').collect();
    match parts.len() {
        3 => {
            let hours: f32 = parts[0].parse().unwrap_or(0.0);
            let minutes: f32 = parts[1].parse().unwrap_or(0.0);
            let seconds: f32 = parts[2].replace(",", ".").parse().unwrap_or(0.0);
            hours * 3600.0 + minutes * 60.0 + seconds
        }
        2 => {
            let minutes: f32 = parts[0].parse().unwrap_or(0.0);
            let seconds: f32 = parts[1].replace(",", ".").parse().unwrap_or(0.0);
            minutes * 60.0 + seconds
        }
        _ => text.replace(",", ".").parse().unwrap_or(0.0),
    }
}

fn parse_li(li: &scraper::ElementRef) -> Option<ParsedLi> {
    let onderdeel_sel = Selector::parse(".onderdeel").unwrap();
    let item_title_sel = Selector::parse(".item-title").unwrap();
    let item_footer_sel = Selector::parse(".item-footer").unwrap();
    let item_footer_span_sel = Selector::parse(".item-footer > span").unwrap();
    let result_footer_sel = Selector::parse(".result-footer-details").unwrap();
    let location_sel = Selector::parse(".location").unwrap();
    let badge_sel = Selector::parse(".badge").unwrap();
    let flagicon_sel = Selector::parse("img.flagicon").unwrap();

    // Event name
    let onderdeel = li.select(&onderdeel_sel).next()?;
    let event_name: String = onderdeel
        .text()
        .filter(|t| {
            let trimmed = t.trim();
            !trimmed.is_empty()
                && !trimmed.contains('\n')
                // skip badge text like "indoor"
                && trimmed.len() > 1
        })
        .next()
        .unwrap_or("")
        .trim()
        .to_string();

    // Indoor detection
    let indoor = onderdeel
        .select(&badge_sel)
        .next()
        .map(|b| b.text().any(|t| t.to_lowercase().contains("indoor")))
        .unwrap_or(false);

    // Attribute from footer
    let attribute = onderdeel
        .select(&item_footer_sel)
        .next()
        .and_then(|f| {
            let text = f.text().collect::<String>().trim().to_string();
            if text.is_empty() { None } else { Some(text) }
        });

    // Performance
    let perf_div = li.select(&item_title_sel).next()?;
    let perf_text: String = perf_div
        .text()
        .filter(|t| !t.trim().is_empty())
        .next()
        .unwrap_or("")
        .trim()
        .to_string();

    let hand_measured = perf_text.ends_with('h');
    let performance_value = parse_performance_value(&perf_text);

    // Wind speed: try `.item-footer > span` (history format) then `.result-footer-details` (PR format)
    let wind_speed = perf_div
        .select(&item_footer_span_sel)
        .next()
        .or_else(|| perf_div.select(&result_footer_sel).next())
        .or_else(|| perf_div.select(&item_footer_sel).next())
        .and_then(|el| {
            let text = el.text().collect::<String>();
            let text = text.trim();
            if text == "n/a m/s" || text.is_empty() {
                return None;
            }
            parse_wind(text)
        });

    // Date and location
    let mut date = String::new();
    let mut location = String::new();
    let mut country_code = String::new();

    if let Some(loc_div) = li.select(&location_sel).next() {
        date = loc_div
            .text()
            .filter(|t| !t.trim().is_empty())
            .next()
            .unwrap_or("")
            .trim()
            .to_string();

        if let Some(footer) = loc_div.select(&result_footer_sel).next() {
            location = footer
                .text()
                .collect::<String>()
                .trim()
                .to_string();

            if let Some(img) = footer.select(&flagicon_sel).next() {
                if let Some(src) = img.value().attr("src") {
                    country_code = extract_country_code(src);
                }
            }
        }
    }

    let not_important = li
        .value()
        .attr("class")
        .map(|c| c.contains("notThatImportant"))
        .unwrap_or(false);

    Some(ParsedLi {
        event_name,
        performance: perf_text,
        performance_value,
        wind_speed,
        hand_measured,
        location,
        country_code,
        date,
        not_important,
        indoor,
        attribute,
    })
}

fn parse_history_tab(tab: &scraper::ElementRef) -> Vec<PerformanceHistoryEntry> {
    let li_sel = Selector::parse("li").unwrap();
    let mut entries = Vec::new();

    for li in tab.select(&li_sel) {
        if li.value().attr("class").map(|c| c.contains("list-header")).unwrap_or(false) {
            continue;
        }
        if let Some(parsed) = parse_li(&li) {
            entries.push(PerformanceHistoryEntry {
                performance: parsed.performance,
                performance_value: parsed.performance_value,
                wind_speed: parsed.wind_speed,
                hand_measured: parsed.hand_measured,
                location: parsed.location,
                country_code: parsed.country_code,
                date: parsed.date,
                delta: if parsed.event_name.is_empty() { None } else { Some(parsed.event_name) },
            });
        }
    }

    entries
}

/// Parse the mobile athlete records page.
///
/// The `name` parameter should be fetched separately from the `do=get` endpoint.
pub fn parse(html: Html, name: String) -> anyhow::Result<MobileAthleteProfile> {
    let list_sel = Selector::parse("div.list-athlete-results").unwrap();
    let li_sel = Selector::parse("ul > li").unwrap();
    let tab_sel = Selector::parse("div.tabs.graphTabs > div[id^='tab-eventgraph-']").unwrap();

    // First div.list-athlete-results contains the PR list
    let pr_container = html
        .select(&list_sel)
        .next()
        .ok_or_else(|| anyhow::anyhow!("no div.list-athlete-results found"))?;

    let mut personal_bests = Vec::new();

    for li in pr_container.select(&li_sel) {
        if li.value().attr("class").map(|c| c.contains("list-header")).unwrap_or(false) {
            continue;
        }
        if let Some(parsed) = parse_li(&li) {
            personal_bests.push(MobilePersonalBest {
                event: parsed.event_name,
                performance: parsed.performance,
                performance_value: parsed.performance_value,
                wind_speed: parsed.wind_speed,
                hand_measured: parsed.hand_measured,
                location: parsed.location,
                country_code: parsed.country_code,
                date: parsed.date,
                not_important: parsed.not_important,
                indoor: parsed.indoor,
                attribute: parsed.attribute,
                history: Vec::new(),
            });
        }
    }

    // History tabs: div.tabs.graphTabs > div[id^=tab-eventgraph-]
    let tabs: Vec<_> = html.select(&tab_sel).collect();

    // Match each tab to its PR by checking if the PR's performance+date appear in the tab
    let mut tab_idx = 0;
    let mut assigned: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for pb in personal_bests.iter_mut() {
        if tab_idx >= tabs.len() {
            break;
        }

        let history = parse_history_tab(&tabs[tab_idx]);

        // Verify this tab matches this PR
        let matches = history.iter().any(|h| {
            h.performance == pb.performance && h.date == pb.date
        });

        if matches {
            pb.history = history;
            assigned.insert(tab_idx);
            tab_idx += 1;
        } else {
            // Look ahead up to 3 tabs for a match
            let mut found = false;
            for look in tab_idx..std::cmp::min(tab_idx + 3, tabs.len()) {
                if assigned.contains(&look) {
                    continue;
                }
                let look_history = parse_history_tab(&tabs[look]);
                if look_history.iter().any(|h| {
                    h.performance == pb.performance && h.date == pb.date
                }) {
                    pb.history = look_history;
                    assigned.insert(look);
                    if look == tab_idx {
                        tab_idx += 1;
                    }
                    found = true;
                    break;
                }
            }
            if !found {
                trace!("No matching history tab for PR: {} {}", pb.event, pb.performance);
            }
        }
    }

    Ok(MobileAthleteProfile {
        name,
        personal_bests,
    })
}

/// Parse the athlete name from the `do=get` mobile endpoint.
pub fn parse_name(html: Html) -> anyhow::Result<String> {
    let title_sel = Selector::parse("h3.no-margin-bottom").unwrap();
    let name = html
        .select(&title_sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().replace("  ", " "))
        .unwrap_or_default();

    if name.is_empty() {
        // Fallback: try .title
        let fallback_sel = Selector::parse(".title").unwrap();
        let name = html
            .select(&fallback_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .ok_or_else(|| anyhow::anyhow!("could not find athlete name in mobile profile"))?;
        Ok(name)
    } else {
        Ok(name)
    }
}
