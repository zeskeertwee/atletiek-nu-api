use crate::cache::{CachedRequest, RequestCache};
use crate::util::ApiResponse;

#[get("/<id>")]
pub async fn get_registrations(id: u32, cache: RequestCache) -> ApiResponse {
    let req = CachedRequest::new_get_registrations(id);
    req.run(cache).await
}
