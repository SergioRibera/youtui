use super::{ParseFrom, ProcessedResult};
use crate::Result;
use crate::common::Thumbnail;
use crate::nav_consts::*;
use crate::query::GetHomeQuery;
use const_format::concatcp;
use json_crawler::{JsonCrawler, JsonCrawlerOwned};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeSection {
    pub title: String,
    pub contents: Vec<HomeContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HomeContent {
    Album(AlbumContent),
    Playlist(PlaylistContent),
    Artist(ArtistContent),
    Song(SongContent),
    Video(VideoContent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumContent {
    pub title: String,
    pub browse_id: Option<String>,
    pub thumbnails: Vec<Thumbnail>,
    pub year: Option<String>,
    pub artists: Vec<Artist>,
    pub is_explicit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistContent {
    pub title: String,
    pub playlist_id: String,
    pub thumbnails: Vec<Thumbnail>,
    pub description: Option<String>,
    pub count: Option<String>,
    pub author: Vec<Artist>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistContent {
    pub title: String,
    pub browse_id: String,
    pub subscribers: Option<String>,
    pub thumbnails: Vec<Thumbnail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongContent {
    pub title: String,
    pub video_id: String,
    pub artists: Vec<Artist>,
    pub thumbnails: Vec<Thumbnail>,
    pub album: Option<AlbumReference>,
    pub views: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoContent {
    pub title: String,
    pub video_id: String,
    pub artists: Vec<Artist>,
    pub thumbnails: Vec<Thumbnail>,
    pub views: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub name: String,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumReference {
    pub name: String,
    pub id: String,
}

impl ParseFrom<GetHomeQuery> for Vec<HomeSection> {
    fn parse_from(p: ProcessedResult<GetHomeQuery>) -> Result<Self> {
        let json_crawler: JsonCrawlerOwned = p.into();
        let results = json_crawler.navigate_pointer(concatcp!(SINGLE_COLUMN_TAB, SECTION_LIST))?;

        parse_mixed_content(results)
    }
}

fn parse_mixed_content(results: JsonCrawlerOwned) -> Result<Vec<HomeSection>> {
    results.try_into_iter()?.map(parse_home_section).collect()
}

fn parse_home_section(mut section: JsonCrawlerOwned) -> Result<HomeSection> {
    // Try different section types
    let array = [
        // Try description shelf
        |s: &mut JsonCrawlerOwned| -> std::result::Result<HomeSection, json_crawler::CrawlerError> {
            let mut description_shelf = s.borrow_pointer(DESCRIPTION_SHELF)?;
            let title = description_shelf.take_value_pointer(concatcp!("/header", RUN_TEXT))?;
            Ok(HomeSection {
                title,
                contents: vec![],
            })
        },
        // Try carousel
        |s: &mut JsonCrawlerOwned| -> std::result::Result<HomeSection, json_crawler::CrawlerError> {
            let mut carousel = s.borrow_pointer(CAROUSEL)?;
            let title = carousel.take_value_pointer(concatcp!(CAROUSEL_TITLE, "/text"))?;
            let mut contents_crawler = carousel.borrow_pointer("/contents")?;
            let contents = contents_crawler
                .try_iter_mut()?
                .filter_map(|mut item| {
                    // Try to parse as MTRIR
                    if let Ok(mut data) = item.borrow_pointer(MTRIR) {
                        parse_home_content_from_data(&mut data).ok()
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            Ok(HomeSection { title, contents })
        },
    ];

    section.try_functions(array).map_err(Into::into)
}

fn parse_home_content_from_data(data: &mut impl JsonCrawler) -> Result<HomeContent> {
    // Check page type to determine content type
    let page_type_str = if let Ok(mut page_type) =
        data.borrow_pointer(concatcp!(TITLE, NAVIGATION_BROWSE, PAGE_TYPE))
    {
        Some(page_type.take_value_pointer::<String>("/")?)
    } else {
        None
    };

    if let Some(page_type_str) = page_type_str {
        match page_type_str.as_str() {
            "MUSIC_PAGE_TYPE_ALBUM" | "MUSIC_PAGE_TYPE_AUDIOBOOK" => {
                Ok(HomeContent::Album(parse_album(data)?))
            }
            "MUSIC_PAGE_TYPE_ARTIST" | "MUSIC_PAGE_TYPE_USER_CHANNEL" => {
                Ok(HomeContent::Artist(parse_artist(data)?))
            }
            "MUSIC_PAGE_TYPE_PLAYLIST" => Ok(HomeContent::Playlist(parse_playlist(data)?)),
            _ => Err(crate::error::Error::response("Unknown page type")),
        }
    } else {
        // No page type - could be song or video
        // Check if it has watchPlaylistId
        if data.borrow_pointer(NAVIGATION_WATCH_PLAYLIST_ID).is_ok() {
            // It's a playlist (watch_playlist)
            Ok(HomeContent::Playlist(parse_watch_playlist(data)?))
        } else {
            // It's a song or video
            parse_song_or_video(data)
        }
    }
}

fn parse_album(data: &mut impl JsonCrawler) -> Result<AlbumContent> {
    let title = data.take_value_pointer(TITLE_TEXT)?;
    let browse_id = data.take_value_pointer(NAVIGATION_BROWSE_ID).ok();
    let thumbnails = data
        .take_value_pointer(THUMBNAIL_RENDERER)
        .or_else(|_| data.take_value_pointer(THUMBNAILS))
        .unwrap_or_default();
    let is_explicit = data.borrow_pointer(SUBTITLE_BADGE_LABEL).is_ok();

    // Parse subtitle runs for artists and year
    let mut artists = Vec::new();
    let mut year = None;

    if let Ok(subtitle_runs) = data.borrow_pointer(SUBTITLE_RUNS) {
        if let Ok(iter) = subtitle_runs.try_into_iter() {
            for mut run in iter {
                if let Ok(browse_id) = run.take_value_pointer(NAVIGATION_BROWSE_ID) {
                    if let Ok(name) = run.take_value_pointer("/text") {
                        artists.push(Artist {
                            name,
                            id: Some(browse_id),
                        });
                    }
                } else if let Ok(text) = run.take_value_pointer::<String>("/text") {
                    let trimmed = text.trim();
                    // Check if it's a year (4 digits)
                    if trimmed.len() == 4 && trimmed.chars().all(|c| c.is_numeric()) {
                        year = Some(text);
                    }
                }
            }
        }
    }

    Ok(AlbumContent {
        title,
        browse_id,
        thumbnails,
        year,
        artists,
        is_explicit,
    })
}

fn parse_artist(data: &mut impl JsonCrawler) -> Result<ArtistContent> {
    let title = data.take_value_pointer(TITLE_TEXT)?;
    let browse_id = data.take_value_pointer(NAVIGATION_BROWSE_ID)?;
    let thumbnails = data
        .take_value_pointer(THUMBNAIL_RENDERER)
        .or_else(|_| data.take_value_pointer(THUMBNAILS))
        .unwrap_or_default();

    let subscribers = data
        .take_value_pointer::<String>(SUBTITLE)
        .ok()
        .and_then(|s| s.split_whitespace().next().map(String::from));

    Ok(ArtistContent {
        title,
        browse_id,
        subscribers,
        thumbnails,
    })
}

fn parse_playlist(data: &mut impl JsonCrawler) -> Result<PlaylistContent> {
    let title = data.take_value_pointer(TITLE_TEXT).ok();
    let browse_id: String = data.take_value_pointer(NAVIGATION_BROWSE_ID)?;
    // Remove "VL" prefix from playlist ID
    let playlist_id = if browse_id.starts_with("VL") {
        browse_id[2..].to_string()
    } else {
        browse_id
    };
    let thumbnails = data
        .take_value_pointer(THUMBNAIL_RENDERER)
        .or_else(|_| data.take_value_pointer(THUMBNAILS))
        .unwrap_or_default();

    let mut description = None;
    let mut count = None;
    let mut author = Vec::new();

    if let Ok(mut subtitle) = data.borrow_pointer("/subtitle") {
        if let Ok(mut runs) = subtitle.borrow_pointer("/runs") {
            let mut full_desc = String::new();
            if let Ok(iter) = runs.try_iter_mut() {
                for mut run in iter {
                    if let Ok(text) = run.take_value_pointer::<String>("/text") {
                        full_desc.push_str(&text);

                        // Try to parse author
                        if let Ok(browse_id) = run.take_value_pointer(NAVIGATION_BROWSE_ID) {
                            author.push(Artist {
                                name: text.clone(),
                                id: Some(browse_id),
                            });
                        }
                    }
                }
            }
            description = Some(full_desc);

            // Try to extract count from description
            if let Some(parts) = description.as_ref().and_then(|d| d.split(" • ").nth(1)) {
                if let Some(num) = parts.split_whitespace().next() {
                    count = Some(num.to_string());
                }
            }
        }
    }

    Ok(PlaylistContent {
        title: title.unwrap_or_default(),
        playlist_id,
        thumbnails,
        description,
        count,
        author,
    })
}

fn parse_watch_playlist(data: &mut impl JsonCrawler) -> Result<PlaylistContent> {
    let title = data.take_value_pointer(TITLE_TEXT)?;
    let playlist_id = data.take_value_pointer(NAVIGATION_WATCH_PLAYLIST_ID)?;
    let thumbnails = data
        .take_value_pointer(THUMBNAIL_RENDERER)
        .or_else(|_| data.take_value_pointer(THUMBNAILS))
        .unwrap_or_default();

    Ok(PlaylistContent {
        title,
        playlist_id,
        thumbnails,
        description: None,
        count: None,
        author: Vec::new(),
    })
}

fn parse_song_or_video(data: &mut impl JsonCrawler) -> Result<HomeContent> {
    let title: String = data.take_value_pointer(TITLE_TEXT)?;
    let video_id: String = data.take_value_pointer(NAVIGATION_VIDEO_ID)?;
    let thumbnails: Vec<Thumbnail> = data
        .take_value_pointer(THUMBNAIL_RENDERER)
        .or_else(|_| data.take_value_pointer(THUMBNAILS))
        .unwrap_or_default();

    // Parse subtitle runs for artists and other info
    let mut artists = Vec::new();
    let mut album = None;
    let mut views = None;

    // Collect all runs first
    if let Ok(mut subtitle_runs) = data.borrow_pointer(SUBTITLE_RUNS) {
        let mut all_runs: Vec<(String, Option<String>)> = Vec::new();
        if let Ok(iter) = subtitle_runs.try_iter_mut() {
            for mut run in iter {
                if let Ok(text) = run.take_value_pointer::<String>("/text") {
                    let browse_id: Option<String> =
                        run.take_value_pointer(NAVIGATION_BROWSE_ID).ok();
                    all_runs.push((text, browse_id));
                }
            }
        }

        // Find dot separator to determine where artists end
        let mut dot_idx = all_runs.len();
        for (i, (text, _)) in all_runs.iter().enumerate() {
            if text.trim() == "•" {
                dot_idx = i;
                break;
            }
        }

        // Parse artists (before dot)
        for (text, browse_id) in all_runs.iter().take(dot_idx) {
            if let Some(id) = browse_id {
                artists.push(Artist {
                    name: text.clone(),
                    id: Some(id.clone()),
                });
            }
        }

        // Last item after dot is usually views
        if let Some((text, _)) = all_runs.last() {
            if text.contains(" views") || text.contains("M") || text.contains("K") {
                views = Some(text.split_whitespace().next().unwrap_or(text).to_string());
            }
        }

        // Check if there's an album (usually between artists and views)
        if dot_idx + 2 < all_runs.len() {
            let (name, browse_id) = &all_runs[dot_idx + 1];
            if let Some(id) = browse_id {
                album = Some(AlbumReference {
                    name: name.clone(),
                    id: id.clone(),
                });
            }
        }
    }

    // Determine if it's a song or video based on presence of album
    if album.is_some() {
        Ok(HomeContent::Song(SongContent {
            title,
            video_id,
            artists,
            thumbnails,
            album,
            views,
        }))
    } else {
        Ok(HomeContent::Video(VideoContent {
            title,
            video_id,
            artists,
            thumbnails,
            views,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::BrowserToken;
    use crate::query::GetHomeQuery;

    #[tokio::test]
    async fn test_get_home() {
        // This would need actual test JSON data
        // parse_test!(
        //     "./test_json/get_home_20240723.json",
        //     "./test_json/get_home_20240723_output.txt",
        //     GetHomeQuery::new(),
        //     BrowserToken
        // );
    }
}
