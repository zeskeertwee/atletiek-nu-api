use atletiek_nu_api::chrono::{NaiveDate, ParseError};
use rocket::form::{FromFormField, ValueField};
use rocket::http::Status;
use rocket::request::FromParam;
use rocket::response::Responder;
use rocket::serde::Serialize;
use rocket::{Request, Response};
use std::ops::Deref;
use std::time::Instant;

pub struct RequestNaiveDate(pub NaiveDate);

impl<'a> FromParam<'a> for RequestNaiveDate {
    type Error = ParseError;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        match NaiveDate::parse_from_str(&param, "%Y-%m-%d") {
            Ok(date) => Ok(RequestNaiveDate(date)),
            Err(e) => Err(e),
        }
    }
}

impl<'a> FromFormField<'a> for RequestNaiveDate {
    fn from_value(field: ValueField<'a>) -> rocket::form::Result<'a, Self> {
        match NaiveDate::parse_from_str(&field.value, "%Y-%m-%d") {
            Ok(date) => Ok(RequestNaiveDate(date)),
            Err(e) => Err(rocket::form::Error::validation(e.to_string()))?,
        }
    }
}

impl Deref for RequestNaiveDate {
    type Target = NaiveDate;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub enum ApiResponse {
    Ok {
        body: String,
        headers: Vec<(String, String)>,
    },
    InternalError(String),
    NotFound(String)
}

impl ApiResponse {
    pub fn new_ok_from_string(data: String) -> Self {
        Self::Ok {
            body: data,
            headers: vec![],
        }
    }

    pub fn new_ok<T>(data: &T) -> Self
    where
        T: Serialize,
    {
        let json = rocket::serde::json::to_string(data).unwrap();

        Self::new_ok_from_string(json)
    }

    pub fn add_header<T>(mut self, k: T, v: T) -> Self
    where
        T: ToString,
    {
        match &mut self {
            Self::Ok { headers, .. } => headers.push((k.to_string(), v.to_string())),
            _ => log::warn!("Cannot set headers on ApiResponse!"),
        }

        self
    }

    pub fn new_internal_error<T>(error: T) -> Self
    where
        T: ToString,
    {
        Self::InternalError(error.to_string())
    }

    pub fn new_not_found<T>(error: T) -> Self
    where
        T: ToString,
    {
        Self::NotFound(error.to_string())
    }

    pub fn nocache(self) -> Self {
        self.add_header("X-Cached", "false")
    }

    pub fn cached(self, age: Instant) -> Self {
        self.add_header("X-Cached", "true")
            .add_header("X-Cached-Age", &age.elapsed().as_secs().to_string())
    }
}

impl<'r> Responder<'r, 'static> for ApiResponse {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (body, code) = match &self {
            Self::Ok { body, .. } => (body, Status::Ok),
            Self::InternalError(body) => (body, Status::InternalServerError),
            Self::NotFound(body) => (body, Status::NotFound),
        };

        let mut resp = Response::build_from(body.to_owned().respond_to(request)?);
        resp.status(code);

        match self {
            Self::Ok { headers, .. } => {
                for (k, v) in headers {
                    resp.raw_header(k, v);
                }
                resp.raw_header("Content-Type", "Application/json");
            }
            _ => (),
        }

        resp.ok()
    }
}
