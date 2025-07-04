use std::collections::HashMap;
use anyhow::bail;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use log::{trace, warn};
use crate::models::competition_registrations_list::CompetitionRegistrationList;

const REGEX_EVENT: &'static str =
    r#"https://www.athletics.app/wedstrijd/uitslagenonderdeel/[\d]{0,}/([A-z\d-]{0,})/"#;
const REGEX_COMPETITION_ID: &'static str = r#"wedstrijd/main/([0-9]{0,})/"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthleteEventResults {
    pub name: String,
    pub competition_id: u32,
    pub results: Vec<EventResult>,
    pub timetable: Vec<TimetableEvent>,
    pub participated_in: CompetitionRegistrationList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResult {
    pub event_name: String,
    pub event_url: String,
    pub items: Vec<EventResultItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimetableEvent {
    pub time: chrono::DateTime<chrono::Utc>,
    pub startlist_url: String,
    pub event_name: String,
    pub event_short: String,
    pub start_group_name: String
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventResultItem {
    Position {
        position: u16,
    },
    Measurement {
        wind_speed: Option<f32>,
        result: f32,
        dnf: bool,
        // the reason that it was assumed to be DNF/DNS
        #[serde(skip_serializing_if = "Option::is_none")]
        dnf_reason: Option<DnfReason>,
    },
    Points {
        amount: u16,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DnfReason {
    DataBelowZero,
    DataAboveThreshold {
        threshold: f32
    }
}

impl AthleteEventResults {
    pub fn get_total_points(&self) -> Option<u16> {
        let mut points = 0;
        let mut no_points = true;

        for i in self.results.iter() {
            for item in i.items.iter() {
                match item {
                    EventResultItem::Points { amount } => {
                        no_points = false;
                        points += amount;
                    },
                    _ => (),
                }
            }
        }

        if no_points {
            None
        } else {
            Some(points)
        }
    }
}

/// Expects the DESKTOP site
pub fn parse(html: Html) -> anyhow::Result<AthleteEventResults> {
    let selector = Selector::parse("#uitslagentabel > tbody").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let row_element_selector = Selector::parse("td").unwrap();
    let a_selector = Selector::parse("a").unwrap();
    let data_span_selector = Selector::parse("span.sortData").unwrap();
    let visible_span_selector = Selector::parse("span.tipped").unwrap();
    let name_element_selector = Selector::parse("div.pageTitle").unwrap();
    let competition_element_selector = Selector::parse("div#menubottom > a.hidden-xs").unwrap();
    let visible_xs_inline_selector = Selector::parse("span.visible-xs-inline").unwrap();
    let hidden_xs_selector = Selector::parse("span.hidden-xs").unwrap();

    let timetable_selector = Selector::parse("table.chronoloogtabel > tbody > tr").unwrap();

    let re_event = Regex::new(&REGEX_EVENT).unwrap();
    let re_competition_id = Regex::new(&REGEX_COMPETITION_ID).unwrap();

    let name = html.select(&name_element_selector).next().unwrap().text().filter(|v| !v.trim().is_empty()).next().unwrap().trim().to_string().replace("  ", " ");
    let competition_url = html.select(&competition_element_selector).next().unwrap().value().attr("href").unwrap();
    let competition_id = re_competition_id.captures_iter(competition_url).next().unwrap()[1].parse().unwrap();

    let mut results = Vec::new();
    let participated_in = super::competition_registrations_list::parse(html.root_element())?;

    let table = match html
        .select(&selector)
        .next() {
        Some(v) => v,
        None => bail!("No results found! (yet?)")
    };

    if table.html().contains("Athletics Champs") {
        bail!("Athletics champs results not supported yet!");
    }

    for row in table.select(&row_selector) {
        let mut fields = row.select(&row_element_selector);
        //dbg!(row.html());

        let mut is_combined_event = false;
        // sometimes there is an extra column for "is combined-event"
        // so, if we cannot find the <a>, try the next one
        // example: https://www.atletiek.nu/atleet/main/1785082/
        let event = {
            let mut event_td = fields.next().unwrap();
            match event_td.select(&a_selector).next() {
                Some(v) => v,
                None => {
                    trace!("is combined-event, using second column for event_td");
                    is_combined_event = true;
                    event_td = fields.next().unwrap();
                    event_td.select(&a_selector).next().unwrap()
                }
            }
        };

        let href = event.value().attr("href").unwrap();
        let event_name = re_event.captures_iter(href).next().unwrap()[1].to_string();
        //dbg!(&event_name);

        let fields: Vec<(usize, scraper::ElementRef)> = fields.enumerate().collect();
        let len = fields.len();
        for (idx, i) in fields {
            if (idx + 1 == len && !is_combined_event) || (idx + 2 == len && is_combined_event) {
                // the last one is position, if this isn't a combined-event, otherwise the single-last one is position
                let position = match i.text().next() {
                    Some(v) => match v.parse() {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("Failed to parse position for event {}: {} ({})", event_name, e, v);
                            continue;
                        }
                    },
                    None => {
                        // we don't have a position, maybe DNS/DNF?
                        warn!("No position for event {}", event_name);
                        continue;
                    }
                };

                results.push(EventResult {
                    event_name: event_name.clone(),
                    event_url: href.to_string(),
                    items: vec![EventResultItem::Position {
                        position,
                    }],
                });

                continue;
            }

            // if combined-event AND the last one, this is points
            if idx + 1 == len && is_combined_event {
                let mut items = vec![];
                match i.text().next().unwrap().parse() {
                    Ok(v) => items.push(EventResultItem::Points {
                        amount: v
                    }),
                    Err(e) => warn!("Failed to parse combined-event points for event {}: {}", event_name, e)
                }

                results.push(EventResult {
                    event_name: event_name.clone(),
                    event_url: href.to_string(),
                    items,
                });
            }

            let data_element = match i.select(&data_span_selector).next() {
                None => {
                    continue;
                }
                Some(v) => v,
            };

            let visible_element = match i.select(&visible_span_selector).next() {
                None => {
                    continue;
                }
                Some(v) => v,
            };

            let data = match data_element.value().attr("data").unwrap().parse() {
                Ok(v) => v,
                Err(e) => {
                    trace!("Failed to parse data attr in sortData span: {} ({})", e, data_element.html());
                    continue;
                }
            };
            let mut dnf = None;
            if data < 0.0 {
                // invalid
                warn!("Data is less than 0: {:.2} for event {}, assuming DNF/DNS", data, event_name);
                dnf = Some(DnfReason::DataBelowZero);
            } else if data > 10000.0 {
                warn!("Data is more than 10000, assuming DNF/DNS: {:.2} for event {}", data, event_name);
                dnf = Some(DnfReason::DataAboveThreshold {
                    threshold: 10000.0
                });
            }
            //dbg!(data);

            let wind_speed = crate::components::wind_speed::parse(&visible_element.html());

            //dbg!(wind_speed);

            results.push(EventResult {
                event_name: event_name.clone(),
                event_url: href.to_string(),
                items: vec![EventResultItem::Measurement {
                    result: data,
                    wind_speed,
                    dnf: dnf.is_some(),
                    dnf_reason: dnf
                }],
            })
        }
    }

    let mut res_map: HashMap<String, Vec<EventResult>> = HashMap::new();

    for i in results {
        match res_map.contains_key(&i.event_url) {
            true => { res_map.get_mut(&i.event_url).unwrap().push(i); },
            false => { res_map.insert(i.event_url.clone(), vec![i]); },
        }
    }

    let mut res: Vec<EventResult> = Vec::new();
    for (_, results) in res_map.into_iter() {
        let name = results[0].event_name.clone();
        let url = results[0].event_url.clone();
        let mut items = Vec::new();
        for i in results {
            items.extend(i.items);
        }

        res.push(EventResult {
            event_name: name,
            event_url: url,
            items
        })
    }

    let mut timetable = Vec::new();
    for row in html.select(&timetable_selector) {
        let mut row_elements: Vec<_> = row.select(&row_element_selector).collect();

        let time = chrono::DateTime::from_timestamp(row_elements[0].select(&data_span_selector).next().unwrap().value().attr("data").unwrap().parse().unwrap(), 0).unwrap();
        let startlist_url = row_elements[0].select(&a_selector).next().unwrap().value().attr("href").unwrap().to_string();
        let start_group_name = row_elements[1].select(&a_selector).next().unwrap().select(&hidden_xs_selector).next().unwrap().text().next().unwrap().to_string();
        let event_a = row_elements[2].select(&a_selector).next().unwrap();
        let event_short = event_a.select(&visible_xs_inline_selector).next().unwrap().text().next().unwrap().to_string();
        let event_long = event_a.select(&hidden_xs_selector).next().unwrap().text().next().unwrap().to_string();

        timetable.push(TimetableEvent {
            time,
            startlist_url,
            start_group_name,
            event_short,
            event_name: event_long
        })
    }


    Ok(AthleteEventResults { name, competition_id, results: res, timetable, participated_in })
}
