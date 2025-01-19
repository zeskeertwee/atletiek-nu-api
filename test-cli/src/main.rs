use atletiek_nu_api::{
    chrono::NaiveDate, get_competition_registrations, get_competitions_for_time_period, scraper,
    search_athletes, search_competitions_for_time_period, get_athlete_event_result, get_athlete_profile
};
use std::sync::mpsc::{sync_channel, Receiver};
use tokio;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let (req_tx, req_rx) = sync_channel::<(usize, atletiek_nu_api::Request)>(1000);
    let (sta_tx, sta_rx) = sync_channel::<(usize, atletiek_nu_api::StatusCode)>(1000);

    atletiek_nu_api::set_request_callback(req_tx, sta_tx);

    //let a = search_competitions_for_time_period(
    //    NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
    //    NaiveDate::from_ymd_opt(2023, 12, 1).unwrap(),
    //    "scopias",
    //)
    //.unwrap();
    //dbg!(a);

    //let a = get_athlete_event_result(1592145).await.unwrap();
    //dbg!(a);
    //let a = search_athletes("femke bol").unwrap();
    //dbg!(a);
    //let c = search_competitions("scopias").unwrap();
    //dbg!(c);

    //let a = atletiek_nu_api::get_competition_registrations_web(&40350).await.unwrap();
    //dbg!(&a);
    //let b = atletiek_nu_api::get_competition_registrations_web(&38774).await.unwrap();
    //dbg!(b);

    let a = get_athlete_profile(872863).await.unwrap();
    dbg!(a);

    print_channel_updates(req_rx, sta_rx);
}

fn print_channel_updates(
    req_rx: Receiver<(usize, atletiek_nu_api::Request)>,
    sta_rx: Receiver<(usize, atletiek_nu_api::StatusCode)>,
) {
    for i in req_rx.try_iter() {
        println!("REQUEST {} --> {}", i.0, i.1.url().to_string());
    }

    for i in sta_rx.try_iter() {
        println!("REQUEST {} --> STATUS {}", i.0, i.1);
    }
}
