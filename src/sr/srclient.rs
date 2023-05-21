pub mod client {
    use std::{fmt::Display, writeln};

    use crate::sr::{Message, Sr};

    #[derive(Debug)]
    pub struct SrRequest {
        pub format: String,
        pub indent: bool,
        pub page: u32,
    }

    #[derive(Debug)]
    pub enum SrError {
        Unknwon { message: String },
        ParseError,
        CommunicationError { message: String },
    }

    impl Display for SrError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                SrError::Unknwon { message } => writeln!(f, "Uknown error: {}", message),
                SrError::ParseError => writeln!(f, "Parse error"),
                SrError::CommunicationError { message } => {
                    writeln!(f, "Communication Error: {}", message)
                }
            }
        }
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

    fn from_json(json: &str) -> Result<Sr, SrError> {
        serde_json::from_str(json).map_err(|_| SrError::ParseError)
    }

    async fn fetch_messages(req: SrRequest) -> Result<String, SrError> {
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

    pub async fn fetch_page(page_num: u32) -> Result<Sr, SrError> {
        let req = SrRequest::new(String::from("json"), false, page_num);

        let json = fetch_messages(req).await?;
        from_json(&json)
    }

    #[allow(dead_code)]
    pub async fn load_all_messages() -> Result<Vec<Message>, SrError> {
        let page_one = fetch_page(1).await?;

        let mut messages = Vec::new();

        for page_num in 1..=page_one.pagination.totalpages {
            let mut sr = fetch_page(page_num).await?;
            messages.append(&mut sr.messages);
        }

        Ok(messages)
    }
}
