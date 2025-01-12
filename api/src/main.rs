mod cache;
mod route;
mod util;

#[macro_use]
extern crate rocket;

use crate::cache::Cache;
use log::trace;
use rocket::tokio;
use std::sync::mpsc::sync_channel;
use std::time::Duration;
use leaky_bucket::RateLimiter;
use self_update::cargo_crate_version;
use rpr;

const RATELIMIT_REFIL_AMOUNT: u16 = 1;
const RATELIMIT_REFIL_INTERVAL: Duration = Duration::from_millis(1000);
const RPR_SHARED_KEY: Option<&'static str> = option_env!("RPR_SHARED_KEY");

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    if let Some(key) = RPR_SHARED_KEY {
        rpr::initialize([41, 54, 52, 41, 50, 49], &key);
    } else {
        println!("Missing shared key during compilation, crash reporting disabled.");
    }

    println!("Checking for updates...");
    let status = rocket::tokio::task::spawn_blocking(|| self_update::backends::github::Update::configure()
        .repo_owner("zeskeertwee")
        .repo_name("atletiek-nu-api")
        .bin_name("api")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build().unwrap()
        .update().unwrap()).await.unwrap();


    if status.updated() {
        println!("Binary was updated to a new version: {}, please re-start the application", status.version());
        println!("Press enter to exit the application");
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        std::process::exit(0);
    } else {
        println!("No update is available.")
    }

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

    let cache = match Cache::load_from_disk() {
        Ok(c) => {
            println!("Cache loaded from disk OK, cleaning");
            c.clean();
            println!("Finished cleaning cache");
            c
        },
        Err(e) => {
            println!("Failed to load cache from disk: {}", e);
            println!("Creating empty cache");
            Cache::new()
        }
    };

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

    let ratelimiter = RateLimiter::builder()
        .initial(0)
        .refill(RATELIMIT_REFIL_AMOUNT as _)
        .interval(RATELIMIT_REFIL_INTERVAL)
        .max(2)
        .build();

    let cache_managed = cache.clone();
    rocket::build()
        .mount(
            "/",
            routes![
                route::search_competitions,
                route::get_registrations,
                route::get_results,
                route::search_athletes,
                route::get_athlete_profile,
            ],
        )
        .manage(cache_managed)
        .manage(ratelimiter)
        .ignite()
        .await?
        .launch()
        .await?;

    match cache.save_to_disk() {
        Ok(()) => println!("Cache saved to disk OK"),
        Err(e) => println!("Cache failed to save to disk: {}", e),
    }

    Ok(())
}
