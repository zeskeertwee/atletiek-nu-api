use std::collections::HashSet;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use log::{trace, warn};

const REGEX_EVENT: &'static str =
    r#"https://www.athletics.app/wedstrijd/startlijst/([\d]+)/([\d]+)/"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionEvent {
    id1: u32,
    id2: u32
}

pub fn parse(html: Html) -> anyhow::Result<Vec<CompetitionEvent>> {
    let a_selector = Selector::parse("tbody > tr > td > span + a").unwrap();
    let event_id_re = Regex::new(REGEX_EVENT).unwrap();
    let mut set: HashSet<(u32,u32)> = HashSet::new();

    for url in html.select(&a_selector).flat_map(|el| el.value().attr("href")) {
        for (_,[id1,id2]) in event_id_re.captures_iter(&url).map(|c| c.extract()) {
            let _ = set.insert((id1.parse()?, id2.parse()?));
        }
    }
    let events: Vec<CompetitionEvent> = set.iter().map(|(id1,id2)| CompetitionEvent {id1: *id1,id2: *id2}).collect();
    Ok(events)
}
