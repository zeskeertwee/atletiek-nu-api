use log::trace;
use regex::Regex;
use scraper::{Html, Selector};
use super::registrations_list::{RegistrationsList, RegistrationsListElement};

const REGEX_PARTICIPANT_ID: &'static str = "deelnemer_id=([0-9]{0,})";

pub struct RegistrationsWebList {
    pub participant_id: u32,
    pub name: String,
    pub category: String,
    pub club_name: String,
    pub team_name: Option<String>,
    pub events: Vec<String>,
    pub out_of_competition: bool,
    pub bib_number: Option<u32>
}

pub fn parse(html: Html) -> anyhow::Result<RegistrationsList> {
    let table_selector = Selector::parse("table.deelnemerstabel").unwrap();
    let th_selector = Selector::parse("thead > tr > th").unwrap();
    let tr_selector = Selector::parse("tbody > tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let span_selector = Selector::parse("span").unwrap();
    let re_participant = Regex::new(REGEX_PARTICIPANT_ID).unwrap();

    let table = html.select(&table_selector).next().unwrap();
    let mut table_headers: Vec<String> = Vec::new();

    for item in table.select(&th_selector) {
        let text = item.text().next().unwrap().to_string();
        trace!("Got table header {}: {}", table_headers.len() + 1, &text);
        table_headers.push(text.to_lowercase());
    }

    for row in table.select(&tr_selector) {
        let participant_id_str = row.value().attr("id").expect("id attribute on row");
        let participant_id = re_participant.captures_iter(participant_id_str).next().expect("participant id in id attribute")[0].parse()?;

        let mut item = RegistrationsWebList {
            participant_id,
            name: String::new(),
            category: String::new(),
            club_name: String::new(),
            team_name: None,
            events: Vec::new(),
            out_of_competition: false,
            bib_number: None,
        };

        for (i, element) in row.select(&td_selector).enumerate() {
            match table_headers[i].as_str() {
                "bib" => match element.select(&span_selector).next().unwrap().text().next().unwrap().parse() {
                    Ok(v) => item.bib_number = Some(v),
                    Err(e) => {
                        trace!("Failed to parse bib '{}': {}", element.select(&span_selector).next().unwrap().text().next().unwrap(), e)
                    },
                },
                v => trace!("Unexpected table header '{}'", v)
            }
        }
    }

    unimplemented!()
}