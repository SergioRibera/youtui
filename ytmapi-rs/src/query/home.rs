use super::{PostMethod, PostQuery, Query};
use crate::auth::AuthToken;
use crate::parse::HomeSections;
use std::borrow::Cow;

/// Query to get the home page feed from YouTube Music.
/// The home page is structured as titled rows (sections), returning music
/// suggestions at a time. Content varies and may contain artist, album, song,
/// playlist or video suggestions, sometimes mixed within the same row.
#[derive(Debug, Clone)]
pub struct GetHomeQuery {
    limit: usize,
}

impl Default for GetHomeQuery {
    fn default() -> Self {
        Self {
            limit: 3,
        }
    }
}

impl GetHomeQuery {
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

impl<A: AuthToken> Query<A> for GetHomeQuery {
    type Output = HomeSections;
    type Method = PostMethod;
}

impl PostQuery for GetHomeQuery {
    fn header(&self) -> serde_json::Map<String, serde_json::Value> {
        serde_json::Map::from_iter([(
            "browseId".to_string(),
            serde_json::Value::String("FEmusic_home".into()),
        )])
    }

    fn params(&self) -> Vec<(&str, Cow<'_, str>)> {
        vec![]
    }

    fn path(&self) -> &str {
        "browse"
    }
}
