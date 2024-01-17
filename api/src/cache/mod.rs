mod serialize_instant;

use std::fs::File;
use std::io::{Read, Write};
use crate::util::ApiResponse;
use atletiek_nu_api::chrono::NaiveDate;
use dashmap::DashMap;
use log::trace;
use rocket::http::ext::IntoCollection;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use leaky_bucket::RateLimiter;
use serde::{Serialize, Deserialize};
use anyhow::Result;

const HOUR_IN_S: u64 = 60 * 60;

#[derive(Eq, PartialEq, Hash, Serialize, Deserialize)]
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

    pub async fn run(self, cache: RequestCache, ratelimiter: &State<RateLimiter>) -> ApiResponse {
        if let Some((age, v)) = cache.lookup(&self) {
            log::info!("Found in cache");
            return ApiResponse::new_ok_from_string(v).cached(age);
        }

        ratelimiter.acquire_one().await;

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
#[derive(Serialize, Deserialize)]
pub struct Cache {
    cached: Arc<DashMap<CachedRequest, CacheEntry>>,
}

#[derive(Serialize, Deserialize)]
pub struct CacheEntry {
    #[serde(with = "serialize_instant")]
    pub timestamp: Instant,
    pub value: String,
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

    fn get_path_on_disk() -> Result<PathBuf> {
        let mut path = std::env::current_exe()?;
        path.pop();
        path.push("cache.json");

        Ok(path)
    }

    pub fn save_to_disk(&self) -> Result<()> {
        let path = Self::get_path_on_disk()?;
        log::info!("Saving cache to {}", path.to_string_lossy());
        let mut file = File::create(path)?;
        file.write_all(rocket::serde::json::to_string(&self)?.as_bytes())?;

        Ok(())
    }

    pub fn load_from_disk() -> Result<Self> {
        let path = Self::get_path_on_disk()?;
        log::info!("Loading cache from {}", path.to_string_lossy());

        let mut file = File::open(path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        let s = rocket::serde::json::from_str(&buf)?;
        Ok(s)
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
