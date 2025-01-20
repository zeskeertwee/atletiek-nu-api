use leaky_bucket::RateLimiter;
use rocket::State;
use crate::cache::{CachedRequest, RequestCache};
use crate::util::{ApiResponse, RequestNaiveDate};
use atletiek_nu_api::Country;

#[get("/competitions/search?<start>&<end>&<query>&<country>")]
pub async fn search_competitions(
    start: RequestNaiveDate,
    end: RequestNaiveDate,
    query: Option<String>,
    country: Option<String>,
    cache: RequestCache,
    ratelimiter: &State<RateLimiter>
) -> ApiResponse {
    let country = match Country::parse(country.unwrap_or_default().as_str()) {
        Ok(country) => country,
        Err(e) => return ApiResponse::new_not_found(e),
    };
    let req = CachedRequest::new_search_competitions(start.0, end.0, query.clone(), country);
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
