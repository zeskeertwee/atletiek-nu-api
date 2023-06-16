mod cache;
mod get_registrations;
mod search_competitions;
mod util;

#[macro_use]
extern crate rocket;

use crate::cache::Cache;
use log::trace;
use rocket::tokio;
use rocket::{Build, Rocket};
use std::sync::mpsc::sync_channel;
use std::time::Duration;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    tokio::spawn(async {
        let (req_tx, req_rx) = sync_channel::<(usize, atletiek_nu_api::Request)>(1000);
        let (sta_tx, sta_rx) = sync_channel::<(usize, atletiek_nu_api::StatusCode)>(1000);

        atletiek_nu_api::set_request_callback(req_tx, sta_tx);

        loop {
            tokio::time::sleep(Duration::from_millis(50)).await;

            for i in req_rx.try_iter() {
                println!("REQUEST {} --> {}", i.0, i.1.url().to_string());
            }

            for i in sta_rx.try_iter() {
                println!("REQUEST {} --> STATUS {}", i.0, i.1);
            }
        }
    });

    let cache = Cache::new();

    let cache_copy = cache.clone();
    tokio::spawn(async {
        let cache = cache_copy;
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;

            trace!("Starting cache clean");
            cache.clean();
            trace!("Finished cache clean");
        }
    });

    rocket::build()
        .mount(
            "/competitions/search",
            routes![search_competitions::search_competitions],
        )
        .mount(
            "/competitions/registrations",
            routes![get_registrations::get_registrations],
        )
        .manage(cache)
        .ignite()
        .await?
        .launch()
        .await?;

    Ok(())
}
