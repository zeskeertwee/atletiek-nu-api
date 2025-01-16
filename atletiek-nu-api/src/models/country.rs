use serde::{Serialize, Deserialize};

#[derive(Clone,Deserialize, Serialize, PartialEq, Hash, Eq)]
pub enum Country {
    BE, BQ, CW, FR, DE, IL, NL, SX, ZA, CH, GB, US
}
impl Country {
    pub fn parse(country: &str) -> anyhow::Result<Self> {
        match country.to_uppercase().as_str() {
            "BE" => Ok(Country::BE),
            "BQ" => Ok(Country::BQ),
            "CW" => Ok(Country::CW),
            "FR" => Ok(Country::FR),
            "DE" => Ok(Country::DE),
            "IL" => Ok(Country::IL),
            "NL" => Ok(Country::NL),
            "SX" => Ok(Country::SX),
            "ZA" => Ok(Country::ZA),
            "CH" => Ok(Country::CH),
            "GB" => Ok(Country::GB),
            "US" => Ok(Country::US),
            "" => Ok(Country::NL),
            _ => anyhow::bail!("country abbriviation ({}): Not in available countries", country)
        }
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            Country::BE => "BE",
            Country::BQ => "BQ",
            Country::CW => "CW",
            Country::FR => "FR",
            Country::DE => "DE",
            Country::IL => "IL",
            Country::NL => "NL",
            Country::SX => "SX",
            Country::ZA => "ZA",
            Country::CH => "CH",
            Country::GB => "GB",
            Country::US => "US",
        }
    }
}
