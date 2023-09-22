use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

pub struct Articut<'a, 'b> {
    username: &'a str,
    api_key: &'b str,
    client: Option<reqwest::Client>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Specified version does not exist")]
    InvalidVersion,
    #[error("Specified level does not exist")]
    InvalidLevel,
    #[error("Authentication failed")]
    BadAuth,
    #[error("Invalid Articut key")]
    InvalidAPIKey,
    #[error("Your input text is too long")]
    InputTextTooLong,
    #[error("Insufficient word count balance")]
    NotEnoughQuota,
    #[error("Internal server error")]
    InternalServerError,
    #[error("Invalid content type")]
    InvalidContentType,
    #[error("Invalid arguments")]
    InvalidArguments,
    #[error("User defined dictionary parse error")]
    UserDictionaryParseError,
    #[error("User defined dictionary file size exceeded")]
    UserDictionarySizeExceed,
    #[error("Requests per minute exceeded")]
    RateLimited,
    #[error(transparent)]
    NetworkError(#[from] reqwest::Error),
}

impl FromStr for Error {
    type Err = ();

    fn from_str(msg: &str) -> Result<Self, Self::Err> {
        match msg {
            s if s.contains("Specified version does not exist") => Ok(Self::InvalidVersion),
            s if s.contains("Specified level does not exist") => Ok(Self::InvalidLevel),
            s if s.contains("Authtication failed") => Ok(Self::BadAuth),
            s if s.contains("Invalid Articut key") => Ok(Self::InvalidAPIKey),
            s if s.contains("Your input_str is too long") => Ok(Self::InputTextTooLong),
            s if s.contains("Insufficient word count balance") => Ok(Self::NotEnoughQuota),
            s if s.contains("Internal server error") => Ok(Self::InternalServerError),
            s if s.contains("Invalid content_type") => Ok(Self::InvalidContentType),
            s if s.contains("Invalid arguments") => Ok(Self::InvalidArguments),
            s if s.contains("UserDefinedDICT Parsing") => Ok(Self::UserDictionaryParseError),
            s if s.contains("Maximum UserDefinedDICT file size") => {
                Ok(Self::UserDictionarySizeExceed)
            }
            s if s.contains("Requests per minute exceeded") => Ok(Self::RateLimited),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default)]
pub enum Level {
    #[serde(rename = "lv1")]
    Lv1,
    #[serde(rename = "lv2")]
    #[default]
    Lv2,
    #[serde(rename = "lv3")]
    Lv3,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PosTag {
    pos: String, // TODO: Make this a enum
    text: String,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub level: Level,
    msg: String,
    #[serde(default)]
    pub exec_time: f32,
    #[serde(default)]
    pub result_pos: Vec<String>,
    #[serde(default)]
    pub result_obj: Vec<Vec<PosTag>>,
    #[serde(default)]
    pub result_segmentation: String,
    #[serde(default)]
    pub word_count_balance: i32,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Pinyin {
    HANYU,
    BOPOMOFO,
}

#[derive(Clone, Serialize, Debug)]
pub struct RequestOptions {
    pub version: &'static str,
    pub level: Level,
    #[serde(rename = "user_defined_dict_file")]
    pub user_dict: HashMap<String, String>,
    pub opendata_place: bool,
    pub wikidata: bool,
    pub chemical: bool,
    pub emoji: bool,
    pub time_ref: String,
    pub pinyin: Pinyin,
}

lazy_static! {
    /// This is an example for using doc comment attributes
    static ref DEFAULT_OPTION: RequestOptions = {
        RequestOptions {
            version: "latest",
            level: Level::Lv2,
            user_dict: HashMap::new(),
            opendata_place: false,
            wikidata: false,
            chemical: false,
            emoji: false,
            time_ref: "".to_string(),
            pinyin: Pinyin::BOPOMOFO,
        }
    };
}

impl<'a, 'b> Articut<'a, 'b> {
    pub fn new(username: &'a str, api_key: &'b str) -> Articut<'a, 'b> {
        Articut {
            username,
            api_key,
            client: None,
        }
    }

    pub async fn parse(self, text: &str) -> Result<Response, Error> {
        self.parse_full_options(text, DEFAULT_OPTION.clone()).await
    }

    pub async fn parse_full_options(
        mut self,
        text: &str,
        option: RequestOptions,
    ) -> Result<Response, Error> {
        #[derive(Debug, Serialize)]
        struct Payload<'a, 'b> {
            #[serde(flatten)]
            opt: RequestOptions,
            input_str: String,
            username: &'a str,
            api_key: &'b str,
        }

        let client = self.client.get_or_insert_with(|| reqwest::Client::new());
        let payload = &Payload {
            opt: option,
            input_str: text.to_string(),
            username: self.username,
            api_key: self.api_key,
        };
        client
            .post("https://api.droidtown.co/Articut/API/")
            .json(&payload)
            .send()
            .await?
            .json::<Response>()
            .await
            .map_err(Into::into)
            .and_then(|res| Error::from_str(&res.msg).map_or(Ok(res), |e| Err(e.into())))
    }
}
