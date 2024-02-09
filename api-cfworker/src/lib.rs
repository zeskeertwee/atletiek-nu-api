use std::collections::HashMap;
use worker::*;
use atletiek_nu_api::chrono::{NaiveDate, ParseError};

fn parse_naive_date(s: &str) -> std::result::Result<NaiveDate, ParseError> {
    NaiveDate::parse_from_str(&s, "%Y-%m-%d")
}

#[event(fetch)]
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
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

                let query = pairs.get("query").map(|v| v.to_owned()).unwrap_or_default();

                match atletiek_nu_api::search_competitions_for_time_period(start_date, end_date, &query).await {
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
        .run(req, env)
        .await
}
