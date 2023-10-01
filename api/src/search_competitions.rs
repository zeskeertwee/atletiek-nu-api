use leaky_bucket::RateLimiter;
use rocket::State;
use crate::cache::{CachedRequest, RequestCache};
use crate::util::{ApiResponse, RequestNaiveDate};

#[get("/?<start>&<end>&<query>")]
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
