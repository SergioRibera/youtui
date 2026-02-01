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
    results
        .try_into_iter()?
        .map(parse_home_section)
        .collect()
}

fn parse_home_section(section: JsonCrawlerOwned) -> Result<HomeSection> {
    // Check if it's a description shelf or carousel
    if let Ok(mut description_shelf) = section.clone().navigate_pointer(DESCRIPTION_SHELF) {
        let title = description_shelf.take_value_pointer(concatcp!("/header", RUN_TEXT))?;
        Ok(HomeSection {
            title,
            contents: vec![],
        })
    } else {
        // It's a carousel
        let mut carousel = section.navigate_pointer(CAROUSEL)?;
        let title = carousel.take_value_pointer(concatcp!(CAROUSEL_TITLE, "/text"))?;
        let contents = carousel
            .navigate_pointer("/contents")?
            .try_into_iter()?
            .filter_map(|item| parse_home_content(item).transpose())
            .collect::<Result<Vec<_>>>()?;
        
        Ok(HomeSection { title, contents })
    }
}

fn parse_home_content(content: JsonCrawlerOwned) -> Result<Option<HomeContent>> {
    // Try to parse as MTRIR (musicTwoRowItemRenderer)
    if let Ok(mut data) = content.navigate_pointer(MTRIR) {
        // Check page type to determine content type
        if let Ok(mut page_type) = data.borrow_pointer(concatcp!(TITLE, NAVIGATION_BROWSE, PAGE_TYPE)) {
            let page_type_str: String = page_type.take_value_pointer("/")?;
            
            match page_type_str.as_str() {
                "MUSIC_PAGE_TYPE_ALBUM" | "MUSIC_PAGE_TYPE_AUDIOBOOK" => {
                    Ok(Some(HomeContent::Album(parse_album(&mut data)?)))
                }
                "MUSIC_PAGE_TYPE_ARTIST" | "MUSIC_PAGE_TYPE_USER_CHANNEL" => {
                    Ok(Some(HomeContent::Artist(parse_artist(&mut data)?)))
                }
                "MUSIC_PAGE_TYPE_PLAYLIST" => {
                    Ok(Some(HomeContent::Playlist(parse_playlist(&mut data)?)))
                }
                _ => Ok(None),
            }
        } else {
            // No page type - could be song or video
            // Check if it has watchPlaylistId
            if data.borrow_pointer(NAVIGATION_WATCH_PLAYLIST_ID).is_ok() {
                // It's a playlist (watch_playlist)
                Ok(Some(HomeContent::Playlist(parse_watch_playlist(&mut data)?)))
            } else {
                // It's a song or video
                Ok(Some(parse_song_or_video(&mut data)?))
            }
        }
    } else {
        // Could be MRLIR (musicResponsiveListItemRenderer) - flat song
        Ok(None)
    }
}

fn parse_album(data: &mut impl JsonCrawler) -> Result<AlbumContent> {
    let title = data.take_value_pointer(TITLE_TEXT)?;
    let browse_id = data.take_value_pointer(NAVIGATION_BROWSE_ID).ok();
    let thumbnails = data.take_value_pointer(THUMBNAIL_RENDERER)?;
    let is_explicit = data.borrow_pointer(SUBTITLE_BADGE_LABEL).is_ok();
    
    // Parse subtitle runs for artists and year
    let subtitle_runs = data.borrow_pointer(SUBTITLE_RUNS)?;
    let mut artists = Vec::new();
    let mut year = None;
    
    for mut run in subtitle_runs.try_into_iter()? {
        if let Ok(browse_id) = run.take_value_pointer(NAVIGATION_BROWSE_ID) {
            let name = run.take_value_pointer("/text")?;
            artists.push(Artist {
                name,
                id: Some(browse_id),
            });
        } else if let Ok(text) = run.take_value_pointer::<String>("/text") {
            if text.chars().all(|c| c.is_numeric()) {
                year = Some(text);
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
    let thumbnails = data.take_value_pointer(THUMBNAIL_RENDERER)?;
    
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
    let thumbnails = data.take_value_pointer(THUMBNAIL_RENDERER)?;
    
    let mut description = None;
    let mut count = None;
    let mut author = Vec::new();
    
    if let Ok(mut subtitle) = data.borrow_pointer("/subtitle") {
        if let Ok(mut runs) = subtitle.borrow_pointer("/runs") {
            let mut full_desc = String::new();
            for mut run in runs.try_iter_mut()? {
                let text: String = run.take_value_pointer("/text")?;
                full_desc.push_str(&text);
                
                // Try to parse author
                if let Ok(browse_id) = run.take_value_pointer(NAVIGATION_BROWSE_ID) {
                    author.push(Artist {
                        name: text.clone(),
                        id: Some(browse_id),
                    });
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
    let thumbnails = data.take_value_pointer(THUMBNAIL_RENDERER)?;
    
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
    let thumbnails: Vec<Thumbnail> = data.take_value_pointer(THUMBNAIL_RENDERER)?;
    
    // Parse subtitle runs for artists and other info
    let mut subtitle_runs = data.borrow_pointer(SUBTITLE_RUNS)?;
    let mut artists = Vec::new();
    let mut album = None;
    let mut views = None;
    
    // Collect all runs first
    let mut all_runs: Vec<(String, Option<String>)> = Vec::new();
    for mut run in subtitle_runs.try_iter_mut()? {
        let text: String = run.take_value_pointer("/text")?;
        let browse_id: Option<String> = run.take_value_pointer(NAVIGATION_BROWSE_ID).ok();
        all_runs.push((text, browse_id));
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
