use std::collections::HashMap;
use std::ops::Add;
use std::time::Duration;
use worker::*;
use atletiek_nu_api::chrono::{NaiveDate, offset, NaiveDateTime, ParseError, DateTime, Utc};
use atletiek_nu_api::Country;
use urlencoding;
use console_error_panic_hook;

const GIT_VERSION: &str = git_version::git_version!();

const HOUR_S: u64 = 60 * 60;

fn parse_naive_date(s: &str) -> std::result::Result<NaiveDate, ParseError> {
    NaiveDate::parse_from_str(&s, "%Y-%m-%d")
}

fn format_http_timestamp(time: DateTime<Utc>) -> String {
    time.format("%a, %d %b %Y %H:%M:%S %Z").to_string()
}

#[event(fetch)]
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let poke_cache = if let Ok(Some(header)) = req.headers().get("X-poke-cache") {
        header == "true"
    } else {
        false
    };

    let cache = Cache::default();

    if poke_cache {
        console_log!("Ignoring cache");
    } else {
        match cache.get(&req, false).await {
            Ok(Some(mut r)) => {
                console_log!("Response from cache");
                return Ok(r);
            },
            Err(e) => console_error!("Failed to get from cache: {}", e),
            Ok(None) => (),
        }
    }


    let router = Router::new();

    let mut cache_validity = Duration::from_secs(HOUR_S);

    let mut response = router
        .get_async("/competitions/results/:id", |_req, ctx| async move {
            if let Some(id) = ctx.param("id") {
                if let Ok(id) = id.parse() {
                    let result = atletiek_nu_api::get_athlete_event_result(id).await;
                    match result {
                        Ok(r) => Response::from_json(&r),
                        Err(e) => {
                            console_error!("Error fetching results: {}", e);
                            Response::error("Internal error", 500)
                        }
                    }
                } else {
                    Response::error("Unable to parse ID", 400)
                }
            } else {
                Response::error("Missing ID", 400)
            }
        })
        .get_async("/athletes/search/:query", |_req, ctx| async move {
            console_log!("Query {:?}", ctx.param("query"));
            if let Some(query) = ctx.param("query") {
                let query = match urlencoding::decode(query) {
                    Ok(q) => q,
                    Err(e) => {
                        console_error!("Error decoding query: {}", e);
                        return Response::error("Query decode error", 400)
                    },
                };
                match atletiek_nu_api::search_athletes(&query).await {
                    Ok(r) => Response::from_json(&r),
                    Err(e) =>  {
                        console_error!("Error fetching results: {}", e);
                        Response::error("Internal error", 500)
                    }
                }
            } else {
                Response::error("Missing query", 400)
            }
        })
        .get_async("/athletes/profile/:id", |_req, ctx| async move {
            if let Some(id) = ctx.param("id") {
                if let Ok(id) = id.parse() {
                    let result = atletiek_nu_api::get_athlete_profile(id).await;
                    match result {
                        Ok(r) => Response::from_json(&r),
                        Err(e) => {
                            console_error!("Error fetching results: {}", e);
                            Response::error("Internal error", 500)
                        }
                    }
                } else {
                    Response::error("Unable to parse ID", 400)
                }
            } else {
                Response::error("Missing ID", 400)
            }
        })
        .get_async("/v1/competitions/registrations/:id", |_req, ctx| async move {
            if let Some(id) = ctx.param("id") {
                if let Ok(id) = id.parse::<u32>() {
                    match atletiek_nu_api::get_competition_registrations(&id).await {
                        Ok(r) => Response::from_json(&r),
                        Err(e) => {
                            console_error!("Error fetching results: {}", e);
                            Response::error("Internal error", 500)
                        }
                    }
                } else {
                    Response::error("Unable to parse ID", 400)
                }
            } else {
                Response::error("Missing ID", 400)
            }
        })
        .get_async("/competitions/registrations/:id", |_req, ctx| async move {
            if let Some(id) = ctx.param("id") {
                if let Ok(id) = id.parse::<u32>() {
                    match atletiek_nu_api::get_competition_registrations_web(&id).await {
                        Ok(r) => Response::from_json(&r),
                        Err(e) => {
                            console_error!("Error fetching results: {}", e);
                            Response::error("Internal error", 500)
                        }
                    }
                } else {
                    Response::error("Unable to parse ID", 400)
                }
            } else {
                Response::error("Missing ID", 400)
            }
        })
        .get_async("/competitions/search", |req, ctx| async move {
            if let Ok(url) = req.url() {
                let pairs: HashMap<String, String> = url
                    .query_pairs()
                    .into_iter()
                    .map(|v| (v.0.to_string(), v.1.to_string()))
                    .fold(HashMap::new(), |mut acc, v| { acc.insert(v.0, v.1); acc });

                let start_date = if let Some(v) = pairs.get("start") {
                    if let Ok(v) = parse_naive_date(v) {
                        v
                    } else {
                        return Response::error("Malformed date", 400);
                    }
                } else {
                    return Response::error("Missing start date", 400);
                };

                let end_date = if let Some(v) = pairs.get("end") {
                    if let Ok(v) = parse_naive_date(v) {
                        v
                    } else {
                        return Response::error("Malformed date", 400);
                    }
                } else {
                    return Response::error("Missing end date", 400);
                };

                if end_date < start_date {
                    return Response::error("End date is before start date", 400);
                }

                let country = pairs.get("country").map(|v| v.to_owned()).unwrap_or_default();
                let country = match Country::parse(country.as_str()) {
                    Ok(country) => country,
                    Err(e) => return Response::error(e.to_string(), 400),
                };

                let query = pairs.get("query").map(|v| v.to_owned()).unwrap_or_default();

                match atletiek_nu_api::search_competitions_for_time_period(start_date, end_date, country, &query).await {
                    Ok(r) => Response::from_json(&r),
                    Err(e) => {
                        console_error!("Error searching competitions: {}", e);
                        Response::error("Internal error", 500)
                    }
                }
            } else {
                Response::error("Internal error", 500)
            }
        })
        .run(req.clone().unwrap(), env)
        .await?;

    response.headers_mut().set("X-ATNAPI-Version", GIT_VERSION);

    let mut cache_resp = response.cloned().unwrap();
    let now = offset::Utc::now();
    let expires_in = now + cache_validity;
    let timestamp = format_http_timestamp(expires_in);
    console_log!("HTTP timestamp {}", timestamp);
    cache_resp.headers_mut().set("Expires", &timestamp);
    cache.put(&req, cache_resp).await;

    Ok(response)
}
