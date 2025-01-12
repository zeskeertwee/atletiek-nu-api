use std::ops::Sub;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Serializer, Deserialize, Deserializer};

pub fn serialize<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    let unix_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards?").as_secs();
    let elapsed_s = instant.elapsed().as_secs();
    let approx = unix_timestamp - elapsed_s;
    approx.serialize(serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Instant, D::Error>
    where
        D: Deserializer<'de>,
{
    let de = u64::deserialize(deserializer)?;
    let unix_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards?").as_secs();
    let age_s = unix_timestamp - de;
    let approx = Instant::now().sub(Duration::from_secs(age_s));
    Ok(approx)
}