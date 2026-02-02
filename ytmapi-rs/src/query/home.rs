use super::{PostMethod, PostQuery, Query};
use crate::auth::AuthToken;
use crate::parse::HomeSections;
use serde_json::json;
use std::borrow::Cow;

/// Query to get the home page feed from YouTube Music.
/// The home page is structured as titled rows (sections), returning music
/// suggestions at a time. Content varies and may contain artist, album, song,
/// playlist or video suggestions, sometimes mixed within the same row.
#[derive(Debug, Clone, Default)]
pub struct GetHomeQuery;

impl GetHomeQuery {
    pub fn new() -> Self {
        Self
    }
}

impl<A: AuthToken> Query<A> for GetHomeQuery {
    type Output = HomeSections;
    type Method = PostMethod;
}

impl PostQuery for GetHomeQuery {
    fn header(&self) -> serde_json::Map<String, serde_json::Value> {
        serde_json::Map::from_iter([("browseId".to_string(), json!("FEmusic_home"))])
    }

    fn params(&self) -> Vec<(&str, Cow<'_, str>)> {
        vec![]
    }

    fn path(&self) -> &str {
        "browse"
    }
}
