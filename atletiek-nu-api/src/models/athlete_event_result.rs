use anyhow::bail;
use regex::Regex;
use scraper::{Html, Selector};

const REGEX_EVENT: &'static str =
    r#"https://www.atletiek.nu/wedstrijd/uitslagenonderdeel/[\d]{0,}/([A-z\d]{0,})/"#;
// Group 1: pos or neg sign, group 2: wind speed
const REGEX_WIND: &'static str = r#"([+-])([\d]{1,}.[\d]{1,})m/s"#;

#[derive(Debug, Clone)]
pub struct AthleteEventResults {
    pub results: Vec<EventResult>,
}

#[derive(Debug, Clone)]
pub struct EventResult {
    pub event_name: String,
    pub result: f64,
    pub event_url: String,
    pub wind_speed: Option<f64>,
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

    let table = html
        .select(&selector)
        .next()
        .expect("No results found (yet?)");

    for row in table.select(&row_selector) {
        let mut fields = row.select(&row_element_selector);
        dbg!(row.html());

        let event_td = fields.next().unwrap();
        let event = event_td.select(&a_selector).next().unwrap();
        let href = event.value().attr("href").unwrap();
        let event_name = re_event.captures_iter(href).next().unwrap()[1].to_string();
        dbg!(&event_name);

        for i in fields {
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
                data = f64::NAN;
            }
            dbg!(data);

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

            dbg!(wind_speed);

            results.push(EventResult {
                result: data,
                event_name: event_name.clone(),
                event_url: href.to_string(),
                wind_speed,
            })
        }
    }

    Ok(AthleteEventResults { results })
}
