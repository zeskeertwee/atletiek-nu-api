use log::trace;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

// Captures the age (just the digits) in the first capture group, and the club name in the second capture group
const REGEX_AGE_AND_CLUB: &'static str = r#"([\d]{1,3}) years \| ([\s\S]{1,})"#;
// Captures the ID in the first capture group
const REGEX_ATHLETE_ID: &'static str = r#"koppel_id=([\d]{1,})"#;

pub type AthleteList = Vec<AthleteListElement>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthleteListElement {
    pub id: u32,
    pub name: String,
    pub club_name: String,
    pub age: u8,
}

pub fn parse(html: Html) -> anyhow::Result<AthleteList> {
    let selector =
        Selector::parse("div.list-athletes > ul > li > a > div.item-inner > div.item-title")
            .unwrap();
    let re_age_and_club = Regex::new(REGEX_AGE_AND_CLUB).unwrap();
    let re_athlete_id = Regex::new(REGEX_ATHLETE_ID).unwrap();

    let mut res = Vec::new();
    for i in html.select(&selector) {
        let texts: Vec<&str> = i
            .text()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .collect();

        if texts.len() < 2 {
            trace!("skipping athlete row: expected >=2 text nodes, got {} ({:?})", texts.len(), texts);
            continue;
        }
        let name = texts[0].replace("  ", " ");
        let age_and_club = texts[1];

        let Some(captures) = re_age_and_club.captures_iter(&age_and_club).next() else {
            trace!("skipping athlete row: age/club regex did not match: {:?}", age_and_club);
            continue;
        };

        let onclick = i
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.value().as_element())
            .and_then(|e| e.attr("onclick"));

        let Some(onclick) = onclick else {
            trace!("skipping athlete row: missing onclick on grandparent element (name={:?})", name);
            continue;
        };

        let Some(id_cap) = re_athlete_id.captures_iter(onclick).next() else {
            trace!("skipping athlete row: athlete_id regex did not match onclick: {:?}", onclick);
            continue;
        };
        let Ok(athlete_id) = id_cap[1].parse::<u32>() else {
            trace!("skipping athlete row: could not parse athlete_id as u32: {:?}", &id_cap[1]);
            continue;
        };

        res.push(AthleteListElement {
            id: athlete_id,
            name: name.to_string(),
            club_name: captures[2].to_string(),
            age: captures[1].parse().unwrap_or(0),
        });
    }

    Ok(res)
}
