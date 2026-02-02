use super::{PostMethod, PostQuery, Query};
use crate::auth::AuthToken;
use crate::parse::HomeSection;
use serde_json::json;
use std::borrow::Cow;

#[derive(Clone)]
pub struct GetHomeQuery {
    limit: Option<usize>,
}

impl GetHomeQuery {
    pub fn new() -> Self {
        Self { limit: None }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl Default for GetHomeQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: AuthToken> Query<A> for GetHomeQuery {
    type Output = Vec<HomeSection>;
    type Method = PostMethod;
}

impl PostQuery for GetHomeQuery {
    fn header(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::from_iter([("browseId".to_string(), json!("FEmusic_home"))]);

        // Add limit parameter if specified (used by continuation system)
        if let Some(_limit) = self.limit {
            map.insert(
                "browseEndpointContextSupportedConfigs".to_string(),
                json!({
                    "browseEndpointContextMusicConfig": {
                        "pageType": "MUSIC_PAGE_TYPE_HOME"
                    }
                }),
            );
        }

        map
    }

    fn params(&self) -> Vec<(&str, Cow<'_, str>)> {
        vec![]
    }

    fn path(&self) -> &str {
        "browse"
    }
}
