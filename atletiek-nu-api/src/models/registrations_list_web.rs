use log::{error, trace, warn};
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use super::registrations_list::{RegistrationsList, RegistrationsListElement};

const REGEX_PARTICIPANT_ID: &'static str = r#"deelnemer_id=([0-9]{0,})"#;
const REGEX_CATEGORY_AND_CLUB: &'static str = r#"([\s\S]{1,}) - ([\s\S]{1,})"#;
const REGEX_RELAY_PARTICIPANT_ID: &'static str = r#"https://www.atletiek.nu/estafetteteam/main/(\d{1,})/"#;

pub type RegistrationsWebList = Vec<RegistrationsWebListElement>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationsWebListElement {
    pub participant_id: u32,
    pub name: String,
    pub category: String,
    pub short_club_name: String,
    pub club_name: String,
    pub team_name: Option<String>,
    pub relay_teams: Vec<RelayTeam>,
    pub events: Vec<(String, EventStatus)>,
    pub out_of_competition: bool,
    pub bib_number: Option<u32>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayTeam {
    pub participant_id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventStatus {
    Accepted,
    Cancelled,
    Rejected,
    Reserve,
    Unverified,
    CheckedIn,

    // Used when the scraper can't determine the status (so it's probably not used by the competition)
    Unknown,
}

pub fn parse(html: Html) -> anyhow::Result<RegistrationsWebList> {
    let table_selector = Selector::parse("table.deelnemerstabel").unwrap();
    let th_selector = Selector::parse("thead > tr > th").unwrap();
    let tr_selector = Selector::parse("tbody > tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let span_selector = Selector::parse("span").unwrap();
    let a_selector = Selector::parse("a").unwrap();
    let span_detail_selector = Selector::parse("span.deelnemer-smalldetail").unwrap();
    let span_tipped_selector = Selector::parse("span.tipped").unwrap();
    let re_participant = Regex::new(REGEX_PARTICIPANT_ID).unwrap();
    let re_cat_club = Regex::new(REGEX_CATEGORY_AND_CLUB).unwrap();
    let re_relay_participant = Regex::new(REGEX_RELAY_PARTICIPANT_ID).unwrap();

    let table = html.select(&table_selector).next().unwrap();
    let mut table_headers: Vec<String> = Vec::new();

    for item in table.select(&th_selector) {
        let text = item.text().next().unwrap().to_string();
        trace!("Got table header {}: {}", table_headers.len() + 1, &text);
        table_headers.push(text.to_lowercase());
    }

    let mut res = Vec::new();

    for row in table.select(&tr_selector) {
        let participant_id_str = row.value().attr("id").expect("id attribute on row");
        let captured_participant_id = re_participant.captures_iter(participant_id_str).next().expect("participant id in id attribute");
        trace!("Captured participant id: {}", &captured_participant_id[1]);
        let participant_id = captured_participant_id[1].parse()?;

        let mut item = RegistrationsWebListElement {
            participant_id,
            name: String::new(),
            category: String::new(),
            short_club_name: String::new(),
            club_name: String::new(),
            team_name: None,
            relay_teams: Vec::new(),
            events: Vec::new(),
            out_of_competition: false,
            bib_number: None,
        };

        for (i, element) in row.select(&td_selector).enumerate() {
            match table_headers[i].as_str() {
                "bib" => {
                    let text = match element.select(&span_selector).next() {
                        Some(span) => span.text().next().unwrap(),
                        None => {
                            warn!("Bib tr contained no span!");
                            continue;
                        },
                    };
                    trace!("table header bib has value {}", text);
                    match text.parse() {
                        Ok(v) => item.bib_number = Some(v),
                        Err(e) => {
                            trace!("Failed to parse bib '{}': {}", element.select(&span_selector).next().unwrap().text().next().unwrap(), e)
                        }
                    }
                },
                "name" => {
                    let a = element.select(&a_selector).next().unwrap();
                    let a_text = a.text().next().unwrap().trim().replace("  ", " ");
                    trace!("Got text (item.name) from name > a: {}", a_text);
                    item.name = a_text.to_string();

                    let span_text = a.select(&span_detail_selector).next().unwrap().text().next().unwrap().trim();
                    trace!("Got text from name > a > span: {}", span_text);
                    let captures = re_cat_club.captures_iter(span_text).next().unwrap();

                    item.category = captures[1].to_string();
                    item.short_club_name = captures[2].to_string().replace(" -", "");

                    match a.select(&span_tipped_selector).next() {
                        Some(span) => {
                            let text = span.text().next().unwrap();
                            if text == "(OoC)" {
                                trace!("Found (OoC)");
                                item.out_of_competition = true;
                            } else {
                                warn!("Got tipped span with text other than (OoC), got {}", text);
                            }
                        },
                        None => (),
                    }
                },
                "club" | "team" => {
                    let mut a_text_iter = match element.select(&a_selector).next() {
                        Some(element) => element.text(),
                        None => {
                            warn!("club/team element contained no a!");
                            continue;
                        },
                    };
                    let mut a_text = String::new();

                    while a_text.is_empty() || a_text.contains("...") {
                        a_text = a_text_iter.next().unwrap().trim().to_string();
                        trace!("New a_text: {}", a_text);
                    }

                    trace!("table {} has value {}", table_headers[i], &a_text);

                    match table_headers[i].as_str() {
                        "club" => item.club_name = a_text,
                        "team" => item.team_name = Some(a_text),
                        _ => error!("Should be unreachable!"),
                    }
                },
                "events" => {
                    let mut events = Vec::new();

                    let mut tipped_spans = element.select(&span_tipped_selector);
                    while let Some(tipped_span) = tipped_spans.next() {
                        trace!("Scraping tipped span for events");
                        // we have tipped spans instead of normal text
                        let kind = tipped_span.value().attr("title").unwrap().trim().to_lowercase();
                        let event_status = match kind.as_str() {
                            "unverified" => EventStatus::Unverified,
                            "cancelled" => EventStatus::Cancelled,
                            "accepted" => EventStatus::Accepted,
                            "rejected" => EventStatus::Rejected,
                            "reserve" => EventStatus::Reserve,
                            "checked-in" => EventStatus::CheckedIn,
                            x => {
                                error!("Unexpected event status: {}", x);
                                EventStatus::Unknown
                            }
                        };
                        let event_text = tipped_span.text().next().unwrap().trim();
                        events.push((event_text.to_string(), event_status));
                    }

                    // if the tipped spans didn't work, try without
                    if events.is_empty() {
                        trace!("No tipped span for events, using text");
                        let mut text = element.text();
                        while let Some(t) = text.next() {
                            let text = t.trim().to_string();
                            if !text.is_empty() {
                                events.push((text, EventStatus::Unknown));
                            }
                        }
                        trace!("table events has values {:?}", &events);
                    }

                    item.events = events;
                },
                "relay team" => {
                    let mut a = element.select(&a_selector);
                    while let Some(a) = a.next() {
                        let href = a.value().attr("href").unwrap();
                        let id = re_relay_participant.captures_iter(href).next().unwrap()[1].parse().unwrap();
                        let text = a.text().next().unwrap().trim().to_string();

                        item.relay_teams.push(RelayTeam {
                            participant_id: id,
                            name: text,
                        });
                    }
                },
                v => trace!("Unexpected table header '{}'", v)
            }
        }

        res.push(item);
    }

    Ok(res)
}