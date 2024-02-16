use log::{error, warn};
use regex::Regex;
use crate::util::round_float_to_digits;

// Group 1: pos or neg sign, group 2: wind speed
const REGEX_WIND: &'static str = r#"([+-])([\d.]{0,})m/s"#;

pub fn parse(text: &str) -> Option<f32> {
    let re_wind = Regex::new(REGEX_WIND).unwrap();

    let text = text.replace(" ", "").replace(",", ".");
    let x = if let Some(captures) = re_wind.captures_iter(&text).next() {
        let sign = match &captures[1] {
            "+" => 1.0,
            "-" => -1.0,
            other => {
                error!("Unexpected wind speed sign: {}", other);
                return None;
            },
        };
        let value = captures[2].parse::<f32>().unwrap();
        Some(round_float_to_digits(value, 2) * sign)
    } else {
        warn!("No capture on {}", text);
        None
    };

    x
}

#[test]
fn test_wind_speed() {
    assert_eq!(parse("+2.1m/s"), Some(2.1));
    assert_eq!(parse("-1.2 m/s"), Some(-1.2));
}