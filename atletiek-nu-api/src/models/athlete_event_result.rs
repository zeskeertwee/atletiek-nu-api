use std::collections::HashMap;
use anyhow::bail;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use log::{trace, warn};

const REGEX_EVENT: &'static str =
    r#"https://www.atletiek.nu/wedstrijd/uitslagenonderdeel/[\d]{0,}/([A-z\d-]{0,})/"#;
// Group 1: pos or neg sign, group 2: wind speed
const REGEX_WIND: &'static str = r#"([+-])([\d]{1,}.[\d]{1,})m/s"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthleteEventResults {
    pub results: Vec<EventResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResult {
    pub event_name: String,
    pub event_url: String,
    pub items: Vec<EventResultItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventResultItem {
    Position {
        position: u16,
    },
    Measurement {
        wind_speed: Option<f64>,
        result: f64,
    },
    Points {
        amount: u16,
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
    let re_event = Regex::new(&REGEX_EVENT).unwrap();
    let re_wind = Regex::new(&REGEX_WIND).unwrap();

    let mut results = Vec::new();

    let table = match html
        .select(&selector)
        .next() {
        Some(v) => v,
        None => anyhow::bail!("No results found! (yet?)")
    };

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
                    Some(v) => v.parse().unwrap(),
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
            if (idx + 1 == len && is_combined_event) {
                results.push(EventResult {
                    event_name: event_name.clone(),
                    event_url: href.to_string(),
                    items: vec![EventResultItem::Points {
                        amount: i.text().next().unwrap().parse().unwrap()
                    }],
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

            let mut data = data_element.value().attr("data").unwrap().parse()?;
            if data < 0.0 {
                // invalid
                warn!("Data is less than 0: {:.2} for event {}", data, event_name);
                continue;
            } else if data > 10000.0 {
                warn!("Data is more than 10000, assuming DNF/DNS: {:.2} for event {}", data, event_name);
                continue;
            }
            //dbg!(data);

            let wind_speed =
                if let Some(captures) = re_wind.captures_iter(&visible_element.html()).next() {
                    let sign = match &captures[1] {
                        "+" => 1.0,
                        "-" => -1.0,
                        other => bail!("Unexpected wind speed sign: {}", other),
                    };
                    Some(captures[2].parse::<f64>()? * sign)
                } else {
                    None
                };

            //dbg!(wind_speed);

            results.push(EventResult {
                event_name: event_name.clone(),
                event_url: href.to_string(),
                items: vec![EventResultItem::Measurement {
                    result: data,
                    wind_speed,
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


    Ok(AthleteEventResults { results: res })
}
