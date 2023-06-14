use regex::Regex;
use scraper::{Html, Selector};

// Captures the ID in the first capture group
const REGEX_PARTICIPANT_ID: &'static str = r#"deelnemer_id=([\d]{1,})"#;
const REGEX_CATEGORY_AND_CLUB: &'static str = r#"([\s\S]{1,}) \| ([\s\S]{1,})"#;
const REGEX_CATEGORY_CLUB_AND_TEAM: &'static str =
    r#"([\s\S]{1,}) \| ([\s\S]{1,}) \| ([\s\S]{1,})"#;

pub type RegistrationsList = Vec<RegistrationsListElement>;

#[derive(Debug, Clone)]
pub struct RegistrationsListElement {
    pub participant_id: u32,
    pub name: String,
    pub category: String,
    pub club_name: String,
    pub team_name: Option<String>,
    pub events: Vec<String>,
}

pub fn parse(html: Html) -> anyhow::Result<RegistrationsList> {
    let script_selector = Selector::parse("script.list-content-registrations").unwrap();
    let selector = Selector::parse("li > a").unwrap();
    let info_selector = Selector::parse("div.item-inner > div.item-title").unwrap();
    let event_selector = Selector::parse("div.item-inner > div.item-after").unwrap();
    let re_participant_id = Regex::new(REGEX_PARTICIPANT_ID).unwrap();
    let re_cat_and_club = Regex::new(REGEX_CATEGORY_AND_CLUB).unwrap();
    let re_cat_club_and_team = Regex::new(REGEX_CATEGORY_CLUB_AND_TEAM).unwrap();

    let script = html
        .select(&script_selector)
        .next()
        .unwrap()
        .inner_html()
        .replace("&lt;", "<")
        .replace("&gt;", ">");

    let mut res = Vec::new();
    for i in Html::parse_fragment(&script).select(&selector) {
        let info_element = i.select(&info_selector).next().unwrap();
        let event_element = i.select(&event_selector).next().unwrap();

        let href = i.value().attr("href").unwrap();
        let participant_id: u32 =
            re_participant_id.captures_iter(href).next().unwrap()[1].parse()?;

        let info_texts: Vec<&str> = info_element
            .text()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .collect();

        let name = info_texts[0].trim().replace("  ", " ");
        let cat_and_club = info_texts[info_texts.len() - 1];

        let (category, club_name, team_name) =
            match re_cat_club_and_team.captures_iter(cat_and_club).next() {
                Some(x) => (
                    x[1].trim().replace("  ", " ").to_string(),
                    x[2].trim().replace("  ", " ").to_string(),
                    Some(x[3].trim().replace("  ", " ").to_string()),
                ),
                None => {
                    let captures = re_cat_and_club.captures_iter(cat_and_club).next().unwrap();
                    (
                        captures[1].trim().replace("  ", " ").to_string(),
                        captures[2].trim().replace("  ", " ").to_string(),
                        None,
                    )
                }
            };

        let events: Vec<String> = event_element
            .text()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string())
            .collect();

        res.push(RegistrationsListElement {
            participant_id,
            name: name.to_string(),
            category,
            club_name,
            team_name,
            events,
        })
    }

    Ok(res)
}
