use chrono::NaiveDate;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use scraper::node::Element;
use serde::{Deserialize, Serialize};

const REGEX_PARTICIPANT_ID: &'static str = r#"https://www.athletics.app/atleet/main/([\d]{0,})/"#;
const REGEX_LOCATION: &'static str = r#"([A-z]{0,})<br><span class="subtext">([A-z]{0,})</span>"#;

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
            let link = row.select(&competition_list_link_selector).next().unwrap();
            //dbg!(row.html());
            let date: String = row.select(&competition_list_sortdata_selector).next()
                .unwrap()
                .value().attr("data").unwrap()
                // we want to drop the location from the string so keep only digits
                .chars().filter(|v| v.is_ascii_digit()).collect();

            let date = NaiveDate::parse_from_str(&date, "%Y%m%d").unwrap();

            let text = link.text().filter(|v| {
                !v.trim().is_empty()
            }).next().unwrap().trim().to_string();
            let participant_id = if let Some(v) = link.select(&a_selector).next() {
                // if there is a child, it will be <a> with a link to the competition
               let s = v.value().attr("href").unwrap();
                re_participant.captures_iter(s).next().unwrap()[1].parse().unwrap()
            } else { 0 };

            let location_element = row.select(&competition_list_location_selector).next().unwrap();
            let place = location_element.text().next().unwrap();
            let location_img = location_element.select(&img_selector).next().unwrap();
            let flag_img_src = location_img.value().attr("src").unwrap();

            let captures = re_location.captures_iter(location_img.value().attr("title").unwrap()).next().unwrap();

            participated_in.push(CompetitionRegistration {
                participant_id,
                name: text,
                date,
                location: CompetitionLocation {
                    country: captures[1].trim().to_string(),
                    continent: captures[2].trim().to_string(),
                    flag_img_url: flag_img_src.to_string(),
                    place: place.trim().to_string()
                }
            });
        }
    }

    Ok(participated_in)
}