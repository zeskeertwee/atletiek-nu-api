//! Parser for the mobile competition search endpoint:
//! `athleteapp.php?page=events&do=searchresults`
//!
//! This endpoint is NOT protected by Cloudflare Turnstile, unlike `feeder.php`.
//! It returns a list of competitions with id, name, date, location, registrations,
//! club-only status, and WA recognition.
//!
//! HTML structure: `li > a[onclick] > div.item-inner > div.item-title > h6 (name)`

use log::trace;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

const REGEX_EVENT_ID: &str = r#"event_id=(\d+)"#;
const REGEX_REGISTRATIONS: &str = r#"(\d+)\s*(?:registrations|athletes|deelnemers)"#;

pub type MobileCompetitionsList = Vec<MobileCompetitionsListElement>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileCompetitionsListElement {
    pub id: u32,
    pub name: String,
    pub location: String,
    pub date_display: String,
    pub registrations: u32,
    pub club_only: bool,
    pub world_athletics_recognized: bool,
    pub cancelled: bool,
}

pub fn parse(html: Html) -> anyhow::Result<MobileCompetitionsList> {
    let li_sel = Selector::parse("li").unwrap();
    let a_sel = Selector::parse("a").unwrap();
    let title_sel = Selector::parse("div.item-title > h6").unwrap();
    let subtitle_sel = Selector::parse("div.subtitle, div.item-footer").unwrap();
    let club_only_sel = Selector::parse("span.clubmembersonly").unwrap();
    let wa_sel = Selector::parse("img.WA-label").unwrap();
    let calendar_header_sel = Selector::parse(".mini-calendar-header").unwrap();
    let calendar_footer_sel = Selector::parse(".mini-calendar-footer").unwrap();

    let re_event_id = Regex::new(REGEX_EVENT_ID).unwrap();
    let re_registrations = Regex::new(REGEX_REGISTRATIONS).unwrap();

    let mut result = Vec::new();

    for li in html.select(&li_sel) {
        // Skip group headers (month dividers)
        let header_attr = li.value().attr("data-header-name");

        let link = match li.select(&a_sel).next() {
            Some(a) => a,
            None => continue,
        };

        // Extract event_id from onclick
        let onclick = link.value().attr("onclick").unwrap_or("");
        let event_id = match re_event_id.captures(onclick) {
            Some(cap) => match cap[1].parse::<u32>() {
                Ok(id) => id,
                Err(_) => continue,
            },
            None => continue,
        };

        // Cancelled?
        let cancelled = link
            .value()
            .attr("class")
            .map(|c| c.contains("cancelled"))
            .unwrap_or(false);

        // Name
        let name = link
            .select(&title_sel)
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

        if name.is_empty() {
            trace!("Skipping competition with empty name (event_id={})", event_id);
            continue;
        }

        // WA recognition
        let world_athletics_recognized = link.select(&wa_sel).next().is_some();

        // Club only
        let club_only = link.select(&club_only_sel).next().is_some();

        // Location & registrations from subtitle/footer
        let mut location = String::new();
        let mut registrations: u32 = 0;

        for sub in link.select(&subtitle_sel) {
            let text = sub.text().collect::<String>();
            let text = text.trim().to_string();

            if let Some(cap) = re_registrations.captures(&text) {
                registrations = cap[1].parse().unwrap_or(0);
            }

            // Location is typically the subtitle text without registration count
            if location.is_empty() && !text.is_empty() {
                location = re_registrations.replace(&text, "").trim().to_string();
                // Clean up trailing separators
                location = location.trim_end_matches('|').trim().to_string();
            }
        }

        // Date display from mini-calendar
        let day_of_week = link
            .select(&calendar_header_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let day_number = link
            .select(&calendar_footer_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let date_display = if !day_of_week.is_empty() && !day_number.is_empty() {
            let month = header_attr.unwrap_or("");
            format!("{} {} {}", day_of_week, day_number, month)
        } else {
            String::new()
        };

        result.push(MobileCompetitionsListElement {
            id: event_id,
            name,
            location,
            date_display,
            registrations,
            club_only,
            world_athletics_recognized,
            cancelled,
        });
    }

    Ok(result)
}
