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
pub use scraper;

pub mod models;
mod traits;
mod util;

use crate::models::competitions_list_web::CompetitionsWebList;
pub use reqwest::{Request, StatusCode};

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

pub async fn send_request(url: &str) -> anyhow::Result<String> {
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
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
        sender.send((id, req.try_clone().unwrap())).unwrap();
    }

    let res = client.execute(req).await?;

    if let Some(sender) = STATUS_SENDER.load().deref() {
        sender.send((id, res.status())).unwrap();
    }

    Ok(res.text().await?)
}

#[deprecated]
pub async fn search_competitions(q: &str) -> anyhow::Result<CompetitionsList> {
    let url = format!("https://www.atletiek.nu/athleteapp.php?page=events&do=searchresults&country_iso2=NL&search={}&predefinedSearchTemplate=0&startDate=-30610225172&endDate=-30610225172&language=en_GB&version=1.16&improvePerformance=0", urlencoding::encode(q));
    let body = send_request(&url).await?;
    models::competitions_list::parse(Html::parse_fragment(&body))
}

pub async fn search_athletes(q: &str) -> anyhow::Result<AthleteList> {
    let url = format!("https://www.atletiek.nu/athleteapp.php?page=athletes&do=searchresults&name={}&language=en_GB&version=1.16&improvePerformance=0", urlencoding::encode(q));
    let body = send_request(&url).await?;
    models::athlete_list::parse(Html::parse_fragment(&body))
}

pub async fn get_competition_registrations<C: CompetitionID>(
    competition_id: &C,
) -> anyhow::Result<RegistrationsList> {
    let url = format!("https://www.atletiek.nu/athleteapp.php?page=event&do=registrations&event_id={}&version=1.16&language=en_GB&improvePerformance=0", competition_id.competition_id());
    let body = send_request(&url).await?;
    //println!("{}", body);
    models::registrations_list::parse(Html::parse_fragment(&body))
}

// curl 'https://www.atletiek.nu/atleet/main/1398565/' --compressed -H 'User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/114.0' -H 'Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8' -H 'Accept-Language: en-US,en;q=0.5' -H 'Accept-Encoding: gzip, deflate, br' -H 'DNT: 1' -H 'Connection: keep-alive' -H 'Cookie: atletieknu_Session=av73k220pflv57f599g4r0u5c2; __cmpcc=1; __cmpconsentx66181=CPtS9XAPtS9XAAfC1BENDICgAAAAAAAAAAigAAAS0gHAA4AKcAZ8BHgCVwFYAMEAdiA7YB3IEKQJEASjAloAAA; __cmpcccx66181=aBPtVahoAAAAAAA' -H 'Upgrade-Insecure-Requests: 1' -H 'Sec-Fetch-Dest: document' -H 'Sec-Fetch-Mode: navigate' -H 'Sec-Fetch-Site: none' -H 'Sec-Fetch-User: ?1' > grep.html

pub async fn get_athlete_event_result(participant_id: u32) -> anyhow::Result<AthleteEventResults> {
    let url = format!("https://www.atletiek.nu/atleet/main/{}/", participant_id);
    let body = send_request(&url).await?;
    //std::fs::write("dump.html", &body).unwrap();
    //panic!("done");
    //dbg!(&body);
    models::athlete_event_result::parse(Html::parse_document(&body))
}

pub async fn get_competitions_for_time_period(
    start: NaiveDate,
    end: NaiveDate,
) -> anyhow::Result<CompetitionsWebList> {
    search_competitions_for_time_period(start, end, "").await
}

pub async fn search_competitions_for_time_period(
    start: NaiveDate,
    end: NaiveDate,
    q: &str,
) -> anyhow::Result<CompetitionsWebList> {
    let start = NaiveDateTime::new(start, NaiveTime::from_hms_opt(0, 0, 0).unwrap()).timestamp();
    let end = NaiveDateTime::new(end, NaiveTime::from_hms_opt(0, 0, 0).unwrap()).timestamp();
    let url = format!("https://www.atletiek.nu/feeder.php?page=search&do=events&country=NL&event_soort[]=in&event_soort[]=out&search={}&startDate={}&endDate={}", urlencoding::encode(q), start, end);
    let body = send_request(&url).await?;
    models::competitions_list_web::parse(Html::parse_document(&body))
}
