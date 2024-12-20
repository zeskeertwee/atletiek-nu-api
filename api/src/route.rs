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
    println!("lol");
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

// TODO: add /athletes/search/[query] endpoint
// TODO: add /athletes/profile/[id] endpoint
