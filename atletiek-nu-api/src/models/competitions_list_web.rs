use chrono::NaiveDate;
use log::warn;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

const REGEX_COMPETITION_ID: &'static str = r#"(\d{1,})"#;
// 1: day of month, 2: month (MAR, AUG, etc.), 3: year
const REGEX_DATE: &'static str = r#"\w{3} (\d{2}) (\w{3}) (\d{4})"#;
const REGEX_REGISTRATIONS: &'static str = r#"(\d{0,}) athletes"#;

pub type CompetitionsWebList = Vec<CompetitionsListWebElement>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionsListWebElement {
    pub date: NaiveDate,
    pub name: String,
    pub location: String,
    pub club: String,
    pub registrations: u32,
    pub results_availible: bool,
    pub club_members_only: bool,
    pub competition_id: u32,
}

pub fn parse(html: Html) -> anyhow::Result<CompetitionsWebList> {
    let row_selector = Selector::parse("tr[onclick]").unwrap();
    let date_selector = Selector::parse("td.datumCol > span.hidden-xs").unwrap();
    let name_selector = Selector::parse("td.eventnaam > a > span > span.eventnaam").unwrap();
    let location_selector =
        Selector::parse("td.eventnaam > a > span > span.verenigingnaam").unwrap();
    let registrations_selector =
        Selector::parse("td.eventnaam > a > span > span.aantaldeelnemers").unwrap();
    let status_selector = Selector::parse("td:last-child > span").unwrap();
    let id_re = Regex::new(REGEX_COMPETITION_ID).unwrap();
    let date_re = Regex::new(REGEX_DATE).unwrap();
    let registrations_re = Regex::new(REGEX_REGISTRATIONS).unwrap();

    let mut result = Vec::new();

    for i in html.select(&row_selector) {
        let onclick = i.value().attr("onclick").unwrap();
        let id: u32 = id_re.captures_iter(onclick).next().unwrap()[1]
            .to_string()
            .parse()?;

        let date = match i.select(&date_selector).next() {
            Some(element) => {
                let date_text = element.inner_html();
                let date_captures = date_re.captures_iter(&date_text).next().unwrap();

                let day = date_captures[1].parse()?;
                let month = match &date_captures[2] {
                    "JAN" => 1,
                    "FEB" => 2,
                    "MAR" => 3,
                    "APR" => 4,
                    "MAY" => 5,
                    "JUN" => 6,
                    "JUL" => 7,
                    "AUG" => 8,
                    "SEP" => 9,
                    "OCT" => 10,
                    "NOV" => 11,
                    "DEC" => 12,
                    _ => anyhow::bail!("Invalid month: {}", &date_captures[2]),
                };
                let year = date_captures[3].parse()?;
                NaiveDate::from_ymd_opt(year, month, day).unwrap()
            }
            // Assume that it's today as there is no date
            None => {
                warn!("No date found, assuming today");
                chrono::offset::Local::now().date_naive()
            },
        };

        let name_node = i.select(&name_selector).next().unwrap();
        let club_members_only = name_node.inner_html().contains("Club members only");
        let name = name_node
            .text()
            .next()
            .unwrap()
            .replace("\u{a0}", "")
            .trim()
            .to_string();

        let location = i.select(&location_selector).next().unwrap().inner_html();
        let mut ls = location.split(", ");
        let club = ls.next().unwrap().to_owned();
        let location = ls.next().unwrap().to_owned();

        let registrations = {
            let text = i
                .select(&registrations_selector)
                .next()
                .unwrap()
                .inner_html();
            registrations_re.captures_iter(&text).next().unwrap()[1].parse()?
        };
        let results_availible = {
            if let Some(text) = i.select(&status_selector).next() {
                if text.inner_html().as_str() == "Results" {
                    true
                } else {
                    false
                }
            } else {
                warn!("Status selector none, assuming no results");
                false
            }
        };

        result.push(CompetitionsListWebElement {
            name,
            date,
            registrations,
            location,
            club,
            competition_id: id,
            club_members_only,
            results_availible,
        })
    }

    Ok(result)
}
