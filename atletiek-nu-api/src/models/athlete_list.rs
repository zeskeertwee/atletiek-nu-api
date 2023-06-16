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

        let name = texts[0].replace("  ", " ");
        let age_and_club = texts[1];

        let captures = re_age_and_club.captures_iter(&age_and_club).next().unwrap();

        let onclick = i
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .value()
            .as_element()
            .unwrap()
            .attr("onclick")
            .unwrap();

        let athlete_id: u32 = re_athlete_id.captures_iter(onclick).next().unwrap()[1].parse()?;

        res.push(AthleteListElement {
            id: athlete_id,
            name: name.to_string(),
            club_name: captures[2].to_string(),
            age: captures[1].parse()?,
        });
    }

    Ok(res)
}
