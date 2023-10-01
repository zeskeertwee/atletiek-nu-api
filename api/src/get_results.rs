use leaky_bucket::RateLimiter;
use rocket::State;
use crate::cache::{CachedRequest, RequestCache};
use crate::util::{ApiResponse, RequestNaiveDate};

#[get("/<id>")]
pub async fn get_results(id: u32, cache: RequestCache, ratelimiter: &State<RateLimiter>) -> ApiResponse {
    let req = CachedRequest::new_get_results(id);
    req.run(cache, ratelimiter).await
}
