use super::{PostMethod, PostQuery, Query};
use crate::auth::AuthToken;
use crate::parse::HomeSection;
use serde_json::json;
use std::borrow::Cow;

#[derive(Clone)]
pub struct GetHomeQuery {
    limit: usize,
}

impl GetHomeQuery {
    pub fn new() -> Self {
        Self { limit: 3 }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
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
        serde_json::Map::from_iter([("browseId".to_string(), json!("FEmusic_home"))])
    }

    fn params(&self) -> Vec<(&str, Cow<'_, str>)> {
        vec![]
    }

    fn path(&self) -> &str {
        "browse"
    }
}
