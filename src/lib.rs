pub mod sr {
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

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Message {
        pub id: i32,
        #[serde(deserialize_with = "date_from_str")]
        pub createddate: DateTime<Tz>,
        pub exactlocation: String,
        pub description: String,
        pub latitude: f32,
        pub longitude: f32,
        pub category: i32,
        pub subcategory: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Category {
        Vagtrafik = 0,
        Kollektivtrafik = 1,
        PlaneradStorning = 2,
        Ovrigt = 3,
    }

    fn from_json(json: &str) -> Result<Sr, SrError> {
        serde_json::from_str(json).map_err(|_| SrError::ParseError)
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

    pub struct SrRequest {
        pub format: String,
        pub indent: bool,
        pub page: u32,
    }

    impl SrRequest {
        pub fn new(format: String, indent: bool, page: u32) -> Self {
            Self {
                format,
                indent,
                page,
            }
        }
    }

    pub async fn fetch_messages(req: SrRequest) -> Result<String, SrError> {
        println!("Fetching page: {}", req.page);
        let url = format!(
            "http://api.sr.se/api/v2/traffic/messages?format={}&indent={}",
            req.format, req.indent
        );
        let client = reqwest::Client::new();

        match client.get(url).send().await {
            Ok(body) => body.text().await.map_err(|e| SrError::Unknwon {
                message: format!("Unknown error: {:?}", e),
            }),
            Err(e) => Err(SrError::CommunicationError {
                message: format!("Http error: {:?}", e),
            }),
        }
    }

    #[derive(Debug)]
    pub enum SrError {
        Unknwon { message: String },
        ParseError,
        CommunicationError { message: String },
    }

    async fn fetch_message_to_struct(page_num: u32) -> Result<Sr, SrError> {
        let req = SrRequest {
            format: String::from("json"),
            indent: false,
            page: page_num,
        };

        let json = fetch_messages(req).await?;
        from_json(&json)
    }

    pub async fn load_all_messages() -> Result<Vec<Message>, SrError> {
        let page_one = fetch_message_to_struct(1).await?;

        let mut messages = Vec::new();

        for page_num in 1..=page_one.pagination.totalpages {
            let mut sr = fetch_message_to_struct(page_num).await?;
            messages.append(&mut sr.messages);
        }

        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDateTime, Utc};
    use chrono_tz::Tz;
    use regex::Regex;

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
            None => println!("Could not parse: {}", test_date),
        };
    }
}
