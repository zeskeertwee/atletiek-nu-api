use leaky_bucket::RateLimiter;
use rocket::State;
use crate::cache::{CachedRequest, RequestCache};
use crate::util::{ApiResponse, RequestNaiveDate};

#[get("/competitions/search?<start>&<end>&<query>")]
pub async fn search_competitions(
    start: RequestNaiveDate,
    end: RequestNaiveDate,
    query: Option<String>,
    cache: RequestCache,
    ratelimiter: &State<RateLimiter>
) -> ApiResponse {
    let req = CachedRequest::new_search_competitions(start.0, end.0, query.clone());
    req.run(cache, ratelimiter).await
}

#[get("/competitions/registrations/<id>")]
pub async fn get_registrations(id: u32, cache: RequestCache, ratelimiter: &State<RateLimiter>) -> ApiResponse {
    let req = CachedRequest::new_get_registrations(id);
    req.run(cache, ratelimiter).await
}

#[get("/competitions/results/<id>")]
pub async fn get_results(id: u32, cache: RequestCache, ratelimiter: &State<RateLimiter>) -> ApiResponse {
    let req = CachedRequest::new_get_results(id);
    req.run(cache, ratelimiter).await
}

#[get("/athletes/search/<query>")]
pub async fn search_athletes(
    query: String,
    cache: RequestCache,
    ratelimiter: &State<RateLimiter>,
) -> ApiResponse {
    let req = CachedRequest::new_search_athletes(query.clone());
    req.run(cache, ratelimiter).await
}

#[get("/athletes/profile/<id>")]
pub async fn get_athlete_profile(id: u32, cache: RequestCache, ratelimiter: &State<RateLimiter>) -> ApiResponse {
    let req = CachedRequest::new_get_athlete_profile(id);
    req.run(cache, ratelimiter).await
}

/// Mobile endpoint: athlete profile with personal bests and per-event performance history.
/// Not affected by Cloudflare Turnstile anti-bot protection.
#[get("/mobile/athletes/profile/<id>")]
pub async fn get_athlete_profile_mobile(id: u32, cache: RequestCache, ratelimiter: &State<RateLimiter>) -> ApiResponse {
    let req = CachedRequest::new_get_athlete_profile_mobile(id);
    req.run(cache, ratelimiter).await
}

/// Mobile endpoint: search competitions by country and date range.
/// Not affected by Cloudflare Turnstile anti-bot protection.
#[get("/mobile/competitions/search?<country>&<start>&<end>&<query>")]
pub async fn search_competitions_mobile(
    country: String,
    start: RequestNaiveDate,
    end: RequestNaiveDate,
    query: Option<String>,
    cache: RequestCache,
    ratelimiter: &State<RateLimiter>,
) -> ApiResponse {
    let req = CachedRequest::new_search_competitions_mobile(country, start.0, end.0, query);
    req.run(cache, ratelimiter).await
}

/// Mobile endpoint: competition detail with categories, events, and pricing.
/// Not affected by Cloudflare Turnstile anti-bot protection.
#[get("/mobile/competitions/detail/<id>")]
pub async fn get_competition_detail_mobile(id: u32, cache: RequestCache, ratelimiter: &State<RateLimiter>) -> ApiResponse {
    let req = CachedRequest::new_get_competition_detail_mobile(id);
    req.run(cache, ratelimiter).await
}

/// Mobile endpoint: full competition program (gender → category → event groups → events).
/// Not affected by Cloudflare Turnstile anti-bot protection.
#[get("/mobile/competitions/program/<id>")]
pub async fn get_competition_program_mobile(id: u32, cache: RequestCache, ratelimiter: &State<RateLimiter>) -> ApiResponse {
    let req = CachedRequest::new_get_competition_program_mobile(id);
    req.run(cache, ratelimiter).await
}
