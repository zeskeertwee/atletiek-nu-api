use crate::util::ApiResponse;
use atletiek_nu_api::chrono::NaiveDate;
use dashmap::DashMap;
use log::trace;
use rocket::http::ext::IntoCollection;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};
use std::ops::Deref;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::Serialize;

const HOUR_IN_S: u64 = 60 * 60;

#[derive(Eq, PartialEq, Hash)]
pub enum CachedRequest {
    SearchCompetitions {
        start: NaiveDate,
        end: NaiveDate,
        query: String,
    },
    GetCompetitionRegistrations {
        id: u32,
    },
    GetCompetitionResults {
        id: u32,
    },
}

#[derive(Serialize)]
struct NotFoundError {
    error: String,
}

impl CachedRequest {
    pub fn new_search_competitions(
        start: NaiveDate,
        end: NaiveDate,
        query: Option<String>,
    ) -> Self {
        let query = match query {
            Some(v) => {
                if v.is_empty() {
                    "".to_string()
                } else {
                    v.to_lowercase()
                }
            }
            None => "".to_string(),
        };

        Self::SearchCompetitions { start, end, query }
    }

    pub fn new_get_registrations(id: u32) -> Self {
        Self::GetCompetitionRegistrations { id }
    }

    pub fn new_get_results(id: u32) -> Self {
        Self::GetCompetitionResults { id }
    }

    fn cache_duration(&self) -> Duration {
        match self {
            Self::SearchCompetitions { .. } => Duration::from_secs(HOUR_IN_S * 12),
            Self::GetCompetitionRegistrations { .. } => Duration::from_secs(HOUR_IN_S * 12),
            Self::GetCompetitionResults { .. } => Duration::from_secs(HOUR_IN_S * 24),
        }
    }

    pub async fn run(self, cache: RequestCache) -> ApiResponse {
        if let Some((age, v)) = cache.lookup(&self) {
            log::info!("Found in cache");
            return ApiResponse::new_ok_from_string(v).cached(age);
        }

        match match &self {
            Self::SearchCompetitions { start, end, query } => {
                atletiek_nu_api::search_competitions_for_time_period(
                    start.to_owned(),
                    end.to_owned(),
                    &query,
                )
                .await
                .map(|v| rocket::serde::json::to_string(&v).unwrap())
            }
            Self::GetCompetitionRegistrations { id } => {
                atletiek_nu_api::get_competition_registrations(id)
                    .await
                    .map(|v| rocket::serde::json::to_string(&v).unwrap())
            }
            Self::GetCompetitionResults { id } => match atletiek_nu_api::get_athlete_event_result(*id)
                .await {
                Ok(v) => Ok(rocket::serde::json::to_string(&v).unwrap()),
                Err(e) => return ApiResponse::new_not_found(rocket::serde::json::to_string(&NotFoundError {
                    error: e.to_string()
                }).unwrap()),
            }
        } {
            Ok(v) => {
                cache.insert(self, v.clone());
                ApiResponse::new_ok_from_string(v).nocache()
            }
            Err(e) => ApiResponse::new_internal_error(e.to_string()),
        }
    }
}

// TODO: persistent cache on disk?
pub struct Cache {
    cached: Arc<DashMap<CachedRequest, (Instant, String)>>,
}

impl Clone for Cache {
    fn clone(&self) -> Self {
        Self {
            cached: Arc::clone(&self.cached),
        }
    }
}

impl Cache {
    pub fn new() -> Self {
        Self {
            cached: Arc::new(DashMap::new()),
        }
    }

    pub fn clean(&self) {
        for i in self.cached.iter() {
            let valid_for = i.key().cache_duration();
            let valid = i.value().0.elapsed() < valid_for;

            if !valid {
                trace!(
                    "Removing item from cache (age {}s, max {}s)",
                    i.value().0.elapsed().as_secs(),
                    valid_for.as_secs()
                );
                self.cached.remove(i.key());
            }
        }
    }

    pub fn lookup(&self, query: &CachedRequest) -> Option<(Instant, String)> {
        match self.cached.get(&query) {
            Some(v) => Some(v.value().to_owned()),
            None => None,
        }
    }

    pub fn insert(&self, query: CachedRequest, value: String) {
        trace!("Inserted new entry into cache");
        self.cached.insert(query, (Instant::now(), value));
    }
}

pub struct RequestCache {
    cache: Cache,
}

#[async_trait]
impl<'r> FromRequest<'r> for RequestCache {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(Self {
            cache: request
                .guard::<&State<Cache>>()
                .await
                .unwrap()
                .deref()
                .deref()
                .clone(),
        })
    }
}

impl Deref for RequestCache {
    type Target = Cache;

    fn deref(&self) -> &Self::Target {
        &self.cache
    }
}
