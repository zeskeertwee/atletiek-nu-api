use std::fs::File;
use std::io::Write;
use crate::models::competitions_list::CompetitionsList;
use arc_swap;
use arc_swap::ArcSwapOption;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use reqwest::ClientBuilder;
use scraper::Html;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use urlencoding;

pub use crate::models::athlete_event_result::AthleteEventResults;
use crate::models::athlete_list::AthleteList;
use crate::models::registrations_list::RegistrationsList;
use crate::traits::CompetitionID;
pub use chrono;
use log::info;
use rand::RngCore;
pub use scraper;

pub const GIT_VERSION: &str = git_version::git_version!();

#[cfg(test)]
mod tests;

pub mod models;
mod traits;
mod util;
mod components;

use crate::models::competitions_list_web::CompetitionsWebList;
pub use crate::components::country::Country;
pub use reqwest::{Request, StatusCode};
use crate::models::athlete_profile::AthleteProfile;
use crate::models::registrations_list_web::RegistrationsWebList;

static REQUEST_SENDER: ArcSwapOption<SyncSender<(usize, Request)>> = ArcSwapOption::const_empty();
static STATUS_SENDER: ArcSwapOption<SyncSender<(usize, StatusCode)>> = ArcSwapOption::const_empty();
static REQUEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn set_request_callback(
    request_sender: SyncSender<(usize, Request)>,
    status_sender: SyncSender<(usize, StatusCode)>,
) {
    REQUEST_SENDER.store(Some(Arc::new(request_sender)));
    STATUS_SENDER.store(Some(Arc::new(status_sender)));
}

pub(crate) async fn send_request(url: &str) -> anyhow::Result<String> {
    let client = ClientBuilder::new()
        //.danger_accept_invalid_certs(true)
        .build()?;
    let req = client
        .get(url)
        // Without this header the sortData spans are gone so it needs to be here
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/114.0",
        )
        .build()?;

    let id = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);

    if let Some(sender) = REQUEST_SENDER.load().deref() {
        sender.send((id, req.try_clone().unwrap()))?;
    }

    let res = client.execute(req).await?;

    if let Some(sender) = STATUS_SENDER.load().deref() {
        sender.send((id, res.status()))?;
    }

    let text = res.text().await?;

    if std::env::var("ATN_DUMP_REQ").is_ok() {
        let n = rand::thread_rng().next_u64();

        let mut file = File::create(format!("/tmp/atn_req_{}.html", n))?;
        file.write_all(text.as_bytes())?;
        info!("Dumped request to /tmp/atn_req_{}.html", n);
    }

    Ok(text)
}

#[deprecated]
pub async fn search_competitions(q: &str) -> anyhow::Result<CompetitionsList> {
    let url = format!("https://www.athletics.app/athleteapp.php?page=events&do=searchresults&country_iso2=NL&search={}&predefinedSearchTemplate=0&startDate=-30610225172&endDate=-30610225172&language=en_GB&version=1.16&improvePerformance=0", urlencoding::encode(q));
    let body = send_request(&url).await?;
    models::competitions_list::parse(Html::parse_fragment(&body))
}

pub async fn search_athletes(q: &str) -> anyhow::Result<AthleteList> {
    let url = format!("https://www.athletics.app/athleteapp.php?page=athletes&do=searchresults&name={}&language=en_GB&version=1.16&improvePerformance=0", urlencoding::encode(q));
    let body = send_request(&url).await?;
    models::athlete_list::parse(Html::parse_fragment(&body))
}

#[deprecated(note = "Please use get_competition_registrations_web instead")]
pub async fn get_competition_registrations<C: CompetitionID>(
    competition_id: &C,
) -> anyhow::Result<RegistrationsList> {
    let url = format!("https://www.athletics.app/athleteapp.php?page=event&do=registrations&event_id={}&version=1.16&language=en_GB&improvePerformance=0", competition_id.competition_id());
    let body = send_request(&url).await?;
    //println!("{}", body);
    models::registrations_list::parse(Html::parse_fragment(&body))
}

pub async fn get_competition_registrations_web<C: CompetitionID>(
    competition_id: &C,
) -> anyhow::Result<RegistrationsWebList> {
    let url = format!("https://www.athletics.app/wedstrijd/atleten/{}/", competition_id.competition_id());
    let body = send_request(&url).await?;
    models::registrations_list_web::parse(Html::parse_document(&body))
}

pub async fn get_athlete_event_result(participant_id: u32) -> anyhow::Result<AthleteEventResults> {
    let url = format!("https://www.athletics.app/atleet/main/{}/", participant_id);
    let body = send_request(&url).await?;
    //std::fs::write("dump.html", &body).unwrap();
    //panic!("done");
    //dbg!(&body);
    models::athlete_event_result::parse(Html::parse_document(&body))
}

pub async fn get_athlete_profile(athlete_id: u32) -> anyhow::Result<AthleteProfile> {
    let url = format!("https://www.athletics.app/atleet/profiel/{}", athlete_id);
    let body = send_request(&url).await?;
    models::athlete_profile::parse(Html::parse_document(&body))
}

pub async fn get_competitions_for_time_period(
    start: NaiveDate,
    end: NaiveDate,
    country: Country,
) -> anyhow::Result<CompetitionsWebList> {
    search_competitions_for_time_period(start, end, country, "").await
}

pub async fn search_competitions_for_time_period(
    start: NaiveDate,
    end: NaiveDate,
    country: Country,
    q: &str,
) -> anyhow::Result<CompetitionsWebList> {
    let start = NaiveDateTime::new(start, NaiveTime::from_hms_opt(0, 0, 0).unwrap()).and_utc().timestamp();
    let end = NaiveDateTime::new(end, NaiveTime::from_hms_opt(0, 0, 0).unwrap()).and_utc().timestamp();
    let url = format!("https://www.athletics.app/feeder.php?page=search&do=events&country={}&event_soort[]=in&event_soort[]=out&search={}&startDate={}&endDate={}", 
        urlencoding::encode(country.to_str()), urlencoding::encode(q), start, end);
    let body = send_request(&url).await?;
    models::competitions_list_web::parse(Html::parse_document(&body))
}
