use atletiek_nu_api::{
    chrono::NaiveDate, get_competition_registrations, get_competitions_for_time_period, scraper,
    search_athletes, search_competitions_for_time_period,
};
use std::sync::mpsc::{sync_channel, Receiver};

fn main() {
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

    //let a = get_athlete_event_result(1612624).unwrap();
    //dbg!(a);
    //let a = search_athletes("femke bol").unwrap();
    //dbg!(a);
    //let c = search_competitions("scopias").unwrap();
    //dbg!(c);

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
