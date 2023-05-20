use std::fmt::Display;

use chrono::DateTime;
use chrono::{NaiveDateTime, Utc};
use chrono_tz::Tz;
use lazy_static::lazy_static;
use regex::Regex;
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

#[derive(Debug, Serialize, Deserialize)]
pub enum Category {
    Vagtrafik = 0,
    Kollektivtrafik = 1,
    PlaneradStorning = 2,
    Ovrigt = 3,
}

fn date_from_str<'de, D>(deserializer: D) -> Result<DateTime<Tz>, D::Error>
where
    D: Deserializer<'de>,
{
    lazy_static! {
        static ref RE: Regex = Regex::new(r"/Date\((\d+)([-+]\d+)\)/").unwrap();
    }
    let date_text = String::deserialize(deserializer)?;

    let caps = RE.captures(&date_text).unwrap();

    let tz: Tz = chrono_tz::Europe::Stockholm;
    let r = NaiveDateTime::from_timestamp_millis(caps[1].parse::<i64>().unwrap().to_owned())
        .unwrap()
        .and_local_timezone(Utc)
        .unwrap();
    Ok(r.with_timezone(&tz))
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDateTime, Utc};
    use chrono_tz::Tz;
    use regex::Regex;
    use tracing::{event, Level};

    #[test]
    fn parse_date() {
        let test_date = "/Date(1682481845657+0200)/";
        let re = Regex::new(r"/Date\((\d+)([-+]\d+)\)/").unwrap();

        match re.captures(test_date) {
            Some(caps) => {
                let tz: Tz = chrono_tz::Europe::Stockholm;
                let r = NaiveDateTime::from_timestamp_millis(
                    caps[1].parse::<i64>().unwrap().to_owned(),
                )
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap();

                let _ = r.with_timezone(&tz);
            }
            None => event!(Level::ERROR, "Could not parse: {}", test_date),
        };
    }
}
