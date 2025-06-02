use std::collections::HashMap;
use chrono::NaiveDate;
use log::{error, trace, warn};
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use crate::models::competition_registrations_list::{self, CompetitionRegistrationList};
use crate::util::round_float_to_digits;

const REGEX_PB_SORT_DATA: &'static str = r#"([0-9]{4})([0-9]{2})([0-9]{2})([\w\s-]{0,}) \(([\w]{0,})\)"#;
const REGEX_PERFORMANCE: &'static str = r#"([0-9]{0,}):([0-9]{0,})[\.,]([0-9]{0,})([h]{0,})"#;
const REGEX_ATTRIBUTE: &'static str = r#"([\d.]{0,})(cm|kg|gr)"#;
const REGEX_GRAPH_INFO: &'static str = r#"title: \{text: '([\w\d\- ]+)'\},subtitle: \{text: '(\d+) results'\}"#;
const REGEX_GRAPH_POINTS: &'static str = r#"\[Date.UTC\((\d{0,}), (\d{0,}), (\d{0,})\),([\d.]{0,})\]"#;
const REGEX_GRAPH_EVENT_ID: &'static str = r#"tab-pane#([\d-]+)"#;
const REGEX_GRAPH_SPECIFICATION: &'static str = r#"history-scores_([\w\d-]+)_1"#;
const REGEX_DIV_SPECIFICATION: &'static str = r#"specification-container-([\d-]+)-([al\d-]+)"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthleteProfile {
    pub name: String,
    pub personal_bests: Vec<PersonalBestItem>,
    pub graphs: Vec<EventGraph>,
    pub competitions: CompetitionRegistrationList
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalBestItem {
    pub event: String,
    pub performance: f32,
    pub wind_speed: Option<f32>,
    pub display_performance: String,
    pub hand_measured: bool,
    pub location: String,
    pub country: String,
    pub date: NaiveDate,
    pub not_important: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<EventAttribute>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventAttribute {
    Height(f32),
    UnknownHeight,
    Weight(f32),
    All
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventGraph {
    pub specification: EventAttribute,
    pub event: String,
    pub event_id: i32,
    pub points: Vec<(NaiveDate, f32)>
}

pub fn parse(html: Html) -> anyhow::Result<AthleteProfile> {
    let pb_table_row_selector = Selector::parse("div#records > table#persoonlijkerecords > tbody > tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let sort_span_selector = Selector::parse("span.sortData").unwrap();
    let subtext_span_selector = Selector::parse("span.subtext").unwrap();
    let span_selector = Selector::parse("span").unwrap();
    let graph_selector = Selector::parse("div#graphTabs > div").unwrap();
    let script_selector = Selector::parse("script").unwrap();
    let spec_div_selector = Selector::parse("div > a.specification-selector").unwrap();
    let page_title_selector = Selector::parse("div.pageTitle").unwrap();
    let re_pb_data = Regex::new(REGEX_PB_SORT_DATA).unwrap();
    let re_performance = Regex::new(REGEX_PERFORMANCE).unwrap();
    let re_graph_info = Regex::new(REGEX_GRAPH_INFO).unwrap();
    let re_graph_points = Regex::new(REGEX_GRAPH_POINTS).unwrap();
    let re_graph_event_id = Regex::new(REGEX_GRAPH_EVENT_ID).unwrap();
    let re_graph_spec = Regex::new(REGEX_GRAPH_SPECIFICATION).unwrap();
    let re_div_spec = Regex::new(REGEX_DIV_SPECIFICATION).unwrap();

    let mut name = String::new();
    let mut page_title_texts = html.select(&page_title_selector).next().unwrap().text();
    while name.is_empty() {
        name = page_title_texts.next().unwrap().trim().replace("  ", " ");
    }

    let competitions = competition_registrations_list::parse(html.root_element())?;
    let mut personal_bests = Vec::new();

    for row in html.select(&pb_table_row_selector) {
        let mut cells = row.select(&td_selector);
        let mut item = PersonalBestItem {
            event: String::new(),
            performance: 0.0,
            wind_speed: None,
            display_performance: String::new(),
            hand_measured: false,
            location: String::new(),
            country: String::new(),
            date: NaiveDate::from_ymd_opt(0, 1, 1).unwrap(),
            not_important: false,
            attribute: None,
        };

        item.not_important = row.value().attr("class").unwrap().contains("notThatImportant");

        item.event = {
            let element = cells.next().unwrap();

            for i in element.select(&subtext_span_selector) {
                let text = i.text().next().unwrap().trim().to_string();
                if text.is_empty() {
                    continue;
                }
                trace!("Got attribute {}", text);
                match text.to_lowercase().as_str() {
                    "manual" => {
                        item.hand_measured = true
                    },
                    "unknown height" => {
                        item.attribute = Some(EventAttribute::UnknownHeight)
                    },
                    x => {
                        if !(x.contains("cm") || x.contains("gr") || x.contains("kg")) {
                            warn!("Unexpected attribute: {}", x);
                            continue;
                        }
                        item.attribute = parse_attribute(x);
                    }
                }
            }

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
            padded_performance_text.push_str(",0");
            trace!("Got performance text {} -> {}", performance_text, padded_performance_text);
            let captures = re_performance.captures_iter(&padded_performance_text).next().unwrap();

            let minutes = captures[1].parse::<u32>().unwrap();
            let seconds = captures[2].parse::<u32>().unwrap();
            let milliseconds = captures[3].parse::<u32>().unwrap();
            let ms_accuracy = captures[3].len();

            // contains 'h' if hand measured
            item.hand_measured = !captures[4].is_empty();

            item.performance = round_float_to_digits(minutes as f32 * 60.0 + seconds as f32 + milliseconds as f32 / f32::powi(10.0, ms_accuracy as _), 3);
            trace!("Parsed to {:.2} hand measured {}", item.performance, item.hand_measured);

            if let Some(Some(span)) = element.select(&subtext_span_selector).next().map(|v| v.select(&span_selector).next()) {
                // wind speed
                let text = span.text().next().unwrap();
                item.wind_speed = crate::components::wind_speed::parse(text);
                trace!("Got wind speed text {} -> {:?}", text, item.wind_speed);
            }
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

        personal_bests.push(item);
    }

    let mut graphs = Vec::new();

    let mut specs: HashMap<String, EventAttribute> = HashMap::new();

    for graph in html.select(&graph_selector) {
        for div in graph.select(&spec_div_selector) {
            let target = div.value().attr("data-target").unwrap();
            let (event_id, spec) = {
                let captures = re_div_spec.captures_iter(target).next().unwrap();
                (captures[1].to_string(), captures[2].to_string())
            };
            let text = div.text().next().unwrap();
            let attr = match text {
                "Everything" => EventAttribute::All,
                _ => parse_attribute(text).unwrap(),
            };
            trace!("Found spec {} for event {} -> {:?}", spec, event_id, attr);
            if specs.insert(spec.clone(), attr).is_some() {
                warn!("spec id collision on {}", spec);
            }
        }
    }
    // we want to get all the specs first

    for graph in html.select(&graph_selector) {
        for script in graph.select(&script_selector) {
            let html = script.inner_html();

            let text = html.trim().replace("\t", "").replace("\n", "");

            let (event_name, point_count) = {
                let captures = re_graph_info.captures_iter(&text).next().unwrap();
                (captures[1].to_string(), captures[2].parse::<usize>().unwrap())
            };
            let event_id = re_graph_event_id.captures_iter(&text).next().unwrap()[1].to_string();
            let specification = re_graph_spec.captures_iter(&text).next().unwrap()[1].to_string();
            trace!("Found graph for {}, {} points (event id {}, spec {})", event_name, point_count, event_id, specification);

            let mut points = Vec::new();

            for point in re_graph_points.captures_iter(&text) {
                trace!("Point capture {} {} {} {}", point[1].to_string(), point[2].to_string(), point[3].to_string(), point[4].to_string());
                let date = NaiveDate::from_ymd_opt(
                    point[1].parse().unwrap(),
                    point[2].parse::<u32>().unwrap() + 1, // in javascript months are 0-based
                    point[3].parse().unwrap()
                ).unwrap();
                let performance = point[4].parse().unwrap();

                points.push((date, performance));
            }

            graphs.push(EventGraph {
                specification: {
                    if specification == event_id {
                        EventAttribute::All
                    } else {
                        specs.get(&specification).unwrap().to_owned()
                    }
                },
                event_id: event_id.parse().unwrap(),
                event: event_name,
                points
            })
        }
    }

    Ok(AthleteProfile {
        name, personal_bests, competitions, graphs
    })
}

fn parse_attribute(text: &str) -> Option<EventAttribute> {
    let re_attribute = Regex::new(REGEX_ATTRIBUTE).unwrap();

    let captures = re_attribute.captures_iter(&text).next().unwrap();
    let value: f32 = captures[1].parse().unwrap();
    match captures[2].to_string().as_str() {
        "cm" => Some(EventAttribute::Height(value / 100.0)),
        "gr" => Some(EventAttribute::Weight(value / 1000.0)),
        "kg" => Some(EventAttribute::Weight(value)),
        x => { error!("Unexpected value in capture group: {}", x); None },
    }
}
