use crate::util::clean_html;
use regex::Regex;
use scraper::{Html, Selector};

// Captures the amount of registrations in the first capture group
const REGEX_REGISTRATIONS: &'static str = "([0-9]{1,}) registrations";

pub type CompetitionsList = Vec<CompetitionsListElement>;

#[derive(Debug, Clone)]
pub struct CompetitionsListElement {
    // TODO: Country & date and maybe WA-label?
    pub id: u32,
    pub name: String,
    pub location: String,
    pub registrations: u16,
    pub club_only: bool,
    pub world_athletics_recognized: bool,
}

pub fn parse(html: Html) -> anyhow::Result<CompetitionsList> {
    let selector = Selector::parse("div.competitions-list").unwrap();

    let mut res = Vec::new();

    for i in html.select(&selector) {
        let html = Html::parse_fragment(&i.inner_html());
        res.extend(parse_element_list(html)?);
    }

    Ok(res)
}

pub fn parse_element_list(html: Html) -> anyhow::Result<Vec<CompetitionsListElement>> {
    let mut res = Vec::new();

    let selector = Selector::parse("li").unwrap();
    let a_selector = Selector::parse("a").unwrap();
    let info_selector = Selector::parse("div.item-inner > div.item-title").unwrap();
    let info_title_selector = Selector::parse("h6").unwrap();
    let info_location_selector = Selector::parse("div.subtitle").unwrap();
    let info_registrations_selector = Selector::parse("div.item-footer").unwrap();
    let club_only_selector = Selector::parse("span.clubmembersonly").unwrap();
    let wa_recognized_selector = Selector::parse("img.WA-label").unwrap();

    for i in html.select(&selector) {
        let id = i
            .select(&a_selector)
            .next()
            .unwrap()
            .value()
            .attr("href")
            .and_then(|v| Some(v.replace("/event&do=get&event_id=", "")))
            .unwrap_or("".to_string());

        let info = i.select(&info_selector).next().unwrap();
        let title_element = &info.select(&info_title_selector).next().unwrap();
        let title = title_element.text().find(|v| !v.trim().is_empty()).unwrap();
        let world_athletics_recognized = title_element
            .select(&wa_recognized_selector)
            .next()
            .is_some();

        let location_element = info.select(&info_location_selector).next().unwrap();
        let location = clean_html(&location_element.inner_html());
        let club_only = location_element
            .select(&club_only_selector)
            .next()
            .is_some();

        let registrations = info
            .select(&info_registrations_selector)
            .next()
            .unwrap()
            .inner_html();

        let registrations_re = Regex::new(REGEX_REGISTRATIONS).unwrap();
        let registrations = registrations_re
            .captures_iter(&registrations)
            .next()
            .unwrap()[1]
            .to_string();

        res.push(CompetitionsListElement {
            club_only,
            world_athletics_recognized,
            id: id.parse()?,
            name: title.trim().to_string(),
            location: location.trim().to_string(),
            registrations: registrations.parse()?,
        });
    }

    Ok(res)
}
