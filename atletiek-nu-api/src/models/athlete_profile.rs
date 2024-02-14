use std::collections::HashMap;
use chrono::NaiveDate;
use log::trace;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use crate::models::competition_registrations_list::{self, CompetitionRegistrationList};

const REGEX_PB_SORT_DATA: &'static str = r#"([0-9]{4})([0-9]{2})([0-9]{2})([\w\s]{0,}) \(([\w]{0,})\)"#;
const REGEX_PERFORMANCE: &'static str = r#"([0-9]{0,}):([0-9]{0,}),([0-9]{0,})([h]{0,})"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthleteProfile {
    pub personal_bests: HashMap<String, PersonalBestItem>,
    pub competitions: CompetitionRegistrationList
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalBestItem {
    pub performance: f32,
    pub display_performance: String,
    pub hand_measured: bool,
    pub location: String,
    pub country: String,
    pub date: NaiveDate,
}

pub fn parse(html: Html) -> anyhow::Result<AthleteProfile> {
    let pb_table_row_selector = Selector::parse("div#records > table#persoonlijkerecords > tbody > tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let sort_span_selector = Selector::parse("span.sortData").unwrap();
    let re_pb_data = Regex::new(REGEX_PB_SORT_DATA).unwrap();
    let re_performance = Regex::new(REGEX_PERFORMANCE).unwrap();

    let competitions = competition_registrations_list::parse(html.root_element())?;
    let mut personal_bests = HashMap::new();

    for row in html.select(&pb_table_row_selector) {
        let mut cells = row.select(&td_selector);
        let mut item = PersonalBestItem {
            performance: 0.0,
            display_performance: String::new(),
            hand_measured: false,
            location: String::new(),
            country: String::new(),
            date: NaiveDate::from_ymd_opt(0, 1, 1).unwrap()
        };

        let event_name = {
            let element = cells.next().unwrap();
            element.text().next().unwrap().trim().to_string()
        };

        {
            let element = cells.next().unwrap();
            let performance_text = element.text().next().unwrap().trim();
            item.display_performance = performance_text.to_string();

            // prepend 0: for a little bit of regex hacking so the first group always captures
            // even if there is no : in the text (e.g. when it's a distance)
            let mut padded_performance_text = String::from("0:");
            padded_performance_text.push_str(performance_text);
            trace!("Got performance text {} -> {}", performance_text, padded_performance_text);
            let captures = re_performance.captures_iter(&padded_performance_text).next().unwrap();

            let minutes = captures[1].parse::<u32>().unwrap();
            let seconds = captures[2].parse::<u32>().unwrap();
            let milliseconds = captures[3].parse::<u32>().unwrap();
            let ms_accuracy = captures[3].len();

            // contains 'h' if hand measured
            item.hand_measured = !captures[4].is_empty();

            item.performance = minutes as f32 * 60.0 + seconds as f32 + milliseconds as f32 / f32::powi(10.0, ms_accuracy as _);
            trace!("Parsed to {:.2} hand measured {}", item.performance, item.hand_measured);
        }

        {
            let element = cells.next().unwrap();
            let span = element.select(&sort_span_selector).next().unwrap();
            let data_text = span.value().attr("data").unwrap().to_string();

            trace!("Got pb sort text {}", data_text);

            let captures = re_pb_data.captures_iter(&data_text).next().unwrap();
            item.date = NaiveDate::from_ymd_opt(
                captures[1].parse().unwrap(),
                captures[2].parse().unwrap(),
                captures[3].parse().unwrap()
            ).unwrap();
            item.location = captures[4].to_string();
            item.country = captures[5].to_string();
        }

        personal_bests.insert(event_name, item);
    }

    Ok(AthleteProfile {
        personal_bests, competitions
    })
}