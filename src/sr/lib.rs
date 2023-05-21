use std::fmt::Display;

use chrono::DateTime;
use chrono::{NaiveDateTime, Utc};
use chrono_tz::Tz;
use lazy_static::lazy_static;
use regex::Regex;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Sr {
    pub copyright: String,
    pub pagination: Pagination,
    pub messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub size: u32,
    pub totalhits: u32,
    pub totalpages: u32,
}

impl Iterator for Pagination {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_last_page() {
            None
        } else {
            Some(self.page + 1)
        }
    }
}

impl Pagination {
    pub fn is_last_page(&self) -> bool {
        self.page == self.totalpages
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Category {
    Vagtrafik = 0,
    Kollektivtrafik = 1,
    PlaneradStorning = 2,
    Ovrigt = 3,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: i32,
    #[serde(deserialize_with = "date_from_str")]
    pub createddate: DateTime<Tz>,
    pub exactlocation: String,
    pub description: String,
    pub title: String,
    pub latitude: f32,
    pub longitude: f32,
    pub category: i32,
    pub subcategory: String,
}

impl Message {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "id: {}", self.id)?;
        writeln!(f, "Created Date: {}", self.createddate)?;
        writeln!(f, "Exact location: {}", self.exactlocation)?;
        writeln!(f, "Title: {}", self.title)?;
        writeln!(f, "Description: {}", self.description)?;
        writeln!(f, "Latitude: {}", self.latitude)?;
        writeln!(f, "Longitude: {}", self.longitude)?;
        writeln!(f, "category: {}", self.category)?;
        writeln!(f, "subcategory: {}", self.subcategory)?;
        Ok(())
    }
}

fn date_from_str<'de, D>(deserializer: D) -> Result<DateTime<Tz>, D::Error>
where
    D: Deserializer<'de>,
{
    lazy_static! {
        static ref RE: Regex = Regex::new(r"/Date\((\d+)([-+]\d+)\)/").unwrap();
    }

    let date_text = String::deserialize(deserializer)?;

    match RE.captures(&date_text) {
        Some(caps) => {
            let tz: Tz = chrono_tz::Europe::Stockholm;
            let millis = caps[1]
                .parse::<i64>()
                .map_err(|e| D::Error::custom(format!("Could not parse timestamp: {}", e)))?;

            match NaiveDateTime::from_timestamp_millis(millis) {
                Some(value) => Ok(value.and_local_timezone(Utc).unwrap().with_timezone(&tz)),
                None => Err(D::Error::custom("Invalid timestamp".to_string())),
            }
        }
        None => Err(D::Error::custom(format!(
            "Invalid date string: {}",
            date_text
        ))),
    }
}

#[cfg(test)]
mod tests {
    use serde::de::value::{Error as ValueError, StrDeserializer};
    use serde::de::IntoDeserializer;

    use super::date_from_str;

    #[test]
    fn deserialize_invalid_timestamp() {
        let des: StrDeserializer<ValueError> =
            "/Date(11111111682481845657+0200)/".into_deserializer();
        let date = date_from_str(des);
        assert!(&date.is_err());
        assert_eq!(
            &date.err().unwrap().to_string(),
            "Could not parse timestamp: number too large to fit in target type"
        );
    }

    #[test]
    fn deserialize_test() {
        let des: StrDeserializer<ValueError> = "/Date(1682481845657+0200)/".into_deserializer();
        let date = date_from_str(des).unwrap();

        assert_eq!(date.to_string(), "2023-04-26 06:04:05.657 CEST");
    }

    #[test]
    fn deserialize_crap() {
        let des: StrDeserializer<ValueError> = "Hestsnp".into_deserializer();
        let date = date_from_str(des);
        assert!(date.is_err())
    }
}
