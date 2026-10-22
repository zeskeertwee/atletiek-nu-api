use chrono::NaiveDate;
use log::trace;
use regex::Regex;
use scraper::{ElementRef, Selector};
use serde::{Deserialize, Serialize};

const REGEX_PARTICIPANT_ID: &'static str = r#"https://www.athletics.app/atleet/main/([\d]{0,})/"#;
const REGEX_LOCATION: &'static str = r#"([\w ]{0,})<br><span class="subtext">([\w ]{0,})</span>"#;

pub type CompetitionRegistrationList = Vec<CompetitionRegistration>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionRegistration {
    pub participant_id: u32,
    pub name: String,
    pub location: CompetitionLocation,
    pub date: NaiveDate
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionLocation {
    pub country: String,
    pub continent: String,
    pub place: String,
    pub flag_img_url: String
}

pub fn parse(element: ElementRef) -> anyhow::Result<CompetitionRegistrationList> {
    let competition_list_selector = Selector::parse("div#wedstrijden > table#persoonlijkerecords").unwrap();
    let competition_list_row_selector = Selector::parse("tbody > tr").unwrap();
    let competition_list_link_selector = Selector::parse("td").unwrap();
    let competition_list_sortdata_selector = Selector::parse("td > span.sortData").unwrap();
    let competition_list_location_selector = Selector::parse("td > span.subtext > span.hidden-xs").unwrap();
    let img_selector = Selector::parse("img").unwrap();
    let a_selector = Selector::parse("a").unwrap();
    let re_participant = Regex::new(REGEX_PARTICIPANT_ID).unwrap();
    let re_location = Regex::new(REGEX_LOCATION).unwrap();

    let mut participated_in = Vec::new();

    if let Some(competitions_table) = element.select(&competition_list_selector).next() {
        for row in competitions_table.select(&competition_list_row_selector) {
            let Some(link) = row.select(&competition_list_link_selector).next() else {
                trace!("skipping registration row: no link cell (td) found");
                continue;
            };

            let date_str: String = match row.select(&competition_list_sortdata_selector).next()
                .and_then(|el| el.value().attr("data")) {
                Some(d) => d.chars().filter(|v| v.is_ascii_digit()).collect(),
                None => {
                    trace!("skipping registration row: missing sortData span or data attribute");
                    continue;
                }
            };

            let Ok(date) = NaiveDate::parse_from_str(&date_str, "%Y%m%d") else {
                trace!("skipping registration row: could not parse date from {:?}", date_str);
                continue;
            };

            let text = match link.text().filter(|v| !v.trim().is_empty()).next() {
                Some(t) => t.trim().to_string(),
                None => {
                    trace!("skipping registration row: link cell has no non-empty text");
                    continue;
                }
            };
            let participant_id = if let Some(v) = link.select(&a_selector).next() {
                v.value().attr("href")
                    .and_then(|s| re_participant.captures_iter(s).next())
                    .and_then(|c| c[1].parse().ok())
                    .unwrap_or(0)
            } else { 0 };

            let Some(location_element) = row.select(&competition_list_location_selector).next() else {
                trace!("skipping registration row: missing location element (name={:?})", text);
                continue;
            };
            let place = location_element.text().next().unwrap_or_default();

            let (flag_img_url, country, continent) = if let Some(location_img) = location_element.select(&img_selector).next() {
                let flag_img_src = location_img.value().attr("src").unwrap_or_default();

                let title = location_img.value().attr("title").unwrap_or_default();
                if let Some(captures) = re_location.captures_iter(title).next() {
                    (flag_img_src.to_string(), captures[1].trim().to_string(), captures[2].trim().to_string())
                } else {
                    (flag_img_src.to_string(), String::new(), String::new())
                }
            } else {
                (String::new(), String::new(), String::new())
            };

            participated_in.push(CompetitionRegistration {
                participant_id,
                name: text,
                date,
                location: CompetitionLocation {
                    country,
                    continent,
                    flag_img_url,
                    place: place.trim().to_string()
                }
            });
        }
    }

    Ok(participated_in)
}