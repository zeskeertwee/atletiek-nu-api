use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use scraper::node::Element;
use serde::{Deserialize, Serialize};

const REGEX_PARTICIPANT_ID: &'static str = r#"https://www.athletics.app/atleet/main/([\d]{0,})/"#;

pub type CompetitionRegistrationList = Vec<CompetitionRegistration>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionRegistration {
    pub participant_id: u32,
    pub name: String,
}

pub fn parse(element: ElementRef) -> anyhow::Result<CompetitionRegistrationList> {
    let competition_list_selector = Selector::parse("div#wedstrijden > table#persoonlijkerecords").unwrap();
    let competition_list_link_selector = Selector::parse("tbody > tr > td > a").unwrap();
    let re_participant = Regex::new(REGEX_PARTICIPANT_ID).unwrap();

    let mut participated_in = Vec::new();

    if let Some(competitions_table) = element.select(&competition_list_selector).next() {
        for i in competitions_table.select(&competition_list_link_selector) {
            let text = i.text().next().unwrap().to_string();
            let href = i.value().attr("href").unwrap();
            dbg!(href);
            participated_in.push(CompetitionRegistration {
                participant_id: re_participant.captures_iter(href).next().unwrap()[1].parse().unwrap(),
                name: text,
            });
        }
    }

    Ok(participated_in)
}