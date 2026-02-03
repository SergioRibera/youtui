use std::ops::{Deref, DerefMut};

use super::{ParsedSongAlbum, ParsedSongArtist, ProcessedResult};
use crate::common::*;
use crate::continuations::ParseFromContinuable;
use crate::nav_consts::*;
use crate::query::{GetContinuationsQuery, GetHomeQuery};
use crate::youtube_enums::YoutubeMusicVideoType;
use crate::Result;
use const_format::concatcp;
use json_crawler::{JsonCrawler, JsonCrawlerOwned};
use serde::{Deserialize, Serialize};

/// Path for section list continuation response
const SECTION_LIST_CONTINUATION: &str = "/continuationContents/sectionListContinuation";

/// A collection of home sections returned from the home feed query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HomeSections(Vec<HomeSection>);

impl HomeSections {
    /// Creates a new `HomeSections` from a vector of sections.
    pub fn new(sections: Vec<HomeSection>) -> Self {
        Self(sections)
    }

    /// Extends this collection with sections from another `HomeSections`.
    pub fn extend(&mut self, other: HomeSections) {
        self.0.extend(other.0);
    }

    /// Truncates the sections to the given length.
    pub fn truncate(&mut self, len: usize) {
        self.0.truncate(len);
    }
}

impl Deref for HomeSections {
    type Target = Vec<HomeSection>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HomeSections {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for HomeSections {
    type Item = HomeSection;
    type IntoIter = std::vec::IntoIter<HomeSection>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a HomeSections {
    type Item = &'a HomeSection;
    type IntoIter = std::slice::Iter<'a, HomeSection>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// A section on the home page containing a title and mixed content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HomeSection {
    pub title: String,
    pub contents: Vec<HomeContent>,
}

/// Content items that can appear in a home section.
/// The content type varies and can be albums, playlists, artists, songs, or videos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HomeContent {
    Album(HomeAlbum),
    Playlist(HomePlaylist),
    Artist(HomeArtist),
    Song(HomeSong),
    Video(HomeVideo),
    WatchPlaylist(HomeWatchPlaylist),
}

/// An album shown on the home page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HomeAlbum {
    pub title: String,
    pub album_id: AlbumID<'static>,
    pub thumbnails: Vec<Thumbnail>,
    pub year: Option<String>,
    pub artists: Vec<ParsedSongArtist>,
    pub explicit: Explicit,
    pub album_type: Option<String>,
}

/// A playlist shown on the home page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HomePlaylist {
    pub title: String,
    pub playlist_id: PlaylistID<'static>,
    pub thumbnails: Vec<Thumbnail>,
    pub description: Option<String>,
    pub count: Option<String>,
    pub author: Vec<ParsedSongArtist>,
}

/// An artist shown on the home page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HomeArtist {
    pub title: String,
    pub channel_id: ArtistChannelID<'static>,
    pub subscribers: Option<String>,
    pub thumbnails: Vec<Thumbnail>,
}

/// A song shown on the home page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HomeSong {
    pub title: String,
    pub video_id: VideoID<'static>,
    pub artists: Vec<ParsedSongArtist>,
    pub thumbnails: Vec<Thumbnail>,
    pub album: Option<ParsedSongAlbum>,
    pub explicit: Explicit,
    pub playlist_id: Option<PlaylistID<'static>>,
}

/// A video shown on the home page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HomeVideo {
    pub title: String,
    pub video_id: VideoID<'static>,
    pub artists: Vec<ParsedSongArtist>,
    pub thumbnails: Vec<Thumbnail>,
    pub views: Option<String>,
    pub playlist_id: Option<PlaylistID<'static>>,
}

/// A watch playlist shown on the home page (auto-generated mixes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HomeWatchPlaylist {
    pub title: String,
    pub playlist_id: PlaylistID<'static>,
    pub thumbnails: Vec<Thumbnail>,
}

impl ParseFromContinuable<GetHomeQuery> for HomeSections {
    fn parse_from_continuable(
        p: ProcessedResult<GetHomeQuery>,
    ) -> Result<(Self, Option<ContinuationParams<'static>>)> {
        let json_crawler: JsonCrawlerOwned = p.into();

        // Navigate to section list renderer (same pattern as other parsers)
        let mut section_list =
            json_crawler.navigate_pointer(concatcp!(SINGLE_COLUMN_TAB, "/sectionListRenderer"))?;

        // Get continuation params if present (must be done before navigating to contents)
        let continuation_params: Option<ContinuationParams<'static>> =
            section_list.take_value_pointer(CONTINUATION_PARAMS).ok();

        // Parse the sections from contents
        let sections = parse_mixed_content(section_list.navigate_pointer("/contents")?)?;

        Ok((HomeSections(sections), continuation_params))
    }

    fn parse_continuation(
        p: ProcessedResult<GetContinuationsQuery<'_, GetHomeQuery>>,
    ) -> Result<(Self, Option<ContinuationParams<'static>>)> {
        let json_crawler: JsonCrawlerOwned = p.into();

        // Try to navigate to section list continuation
        // If this path doesn't exist, return empty results (end of stream)
        // This matches Python's behavior: `if "continuationContents" in response`
        let Ok(mut section_list) = json_crawler.navigate_pointer(SECTION_LIST_CONTINUATION) else {
            return Ok((HomeSections::default(), None));
        };

        // Get continuation params if present (for next page)
        let continuation_params: Option<ContinuationParams<'static>> =
            section_list.take_value_pointer(CONTINUATION_PARAMS).ok();

        // Parse the sections from continuation contents
        // Python uses get_continuation_contents which looks for "contents" or "items"
        let sections = if let Ok(contents) = section_list.navigate_pointer("/contents") {
            parse_mixed_content(contents)?
        } else {
            Vec::new()
        };

        Ok((HomeSections(sections), continuation_params))
    }
}

/// Parse mixed content sections from the home feed.
/// This corresponds to the Python parse_mixed_content function.
fn parse_mixed_content(mut sections: JsonCrawlerOwned) -> Result<Vec<HomeSection>> {
    let mut items = Vec::new();

    for row in sections.try_iter_mut()? {
        // Try to get carousel shelf
        if let Ok(mut carousel) = row.navigate_pointer(CAROUSEL) {
            // Skip if no contents
            if !carousel.path_exists("/contents") {
                continue;
            }

            // Get title
            let title: String = carousel.take_value_pointer(concatcp!(
                "/header/musicCarouselShelfBasicHeaderRenderer",
                TITLE,
                "/text"
            ))?;

            let mut contents = Vec::new();

            // Parse each item in the carousel
            for result in carousel.navigate_pointer("/contents")?.try_iter_mut()? {
                if let Some(content) = parse_home_item(result)? {
                    contents.push(content);
                }
            }

            if !contents.is_empty() {
                items.push(HomeSection { title, contents });
            }
        }
    }

    Ok(items)
}

/// Parse a single home item from the carousel contents.
/// Based on the page_type, determines what kind of content it is.
fn parse_home_item(item: impl JsonCrawler) -> Result<Option<HomeContent>> {
    // Try to get musicTwoRowItemRenderer
    if let Ok(mut data) = item.navigate_pointer(MTRIR) {
        // Try to determine the page type
        let page_type: Option<String> =
            data.take_value_pointer(concatcp!(TITLE, NAVIGATION_BROWSE, PAGE_TYPE))
                .ok();

        match page_type.as_deref() {
            Some("MUSIC_PAGE_TYPE_ALBUM") | Some("MUSIC_PAGE_TYPE_AUDIOBOOK") => {
                Ok(Some(HomeContent::Album(parse_home_album(data)?)))
            }
            Some("MUSIC_PAGE_TYPE_ARTIST") | Some("MUSIC_PAGE_TYPE_USER_CHANNEL") => {
                Ok(Some(HomeContent::Artist(parse_home_artist(data)?)))
            }
            Some("MUSIC_PAGE_TYPE_PLAYLIST") => {
                Ok(Some(HomeContent::Playlist(parse_home_playlist(data)?)))
            }
            None => {
                // Could be a song, video, or watch playlist
                // Check if it has a watch playlist endpoint
                if data.path_exists(NAVIGATION_WATCH_PLAYLIST_ID) {
                    // Check if it's a song or watch playlist by looking for video type
                    let video_type: Option<YoutubeMusicVideoType> = data
                        .take_value_pointer(concatcp!(
                            "/navigationEndpoint/watchEndpoint",
                            "/watchEndpointMusicSupportedConfigs/watchEndpointMusicConfig/musicVideoType"
                        ))
                        .ok();

                    match video_type {
                        Some(YoutubeMusicVideoType::Ugc)
                        | Some(YoutubeMusicVideoType::Omv)
                        | Some(YoutubeMusicVideoType::Shoulder) => {
                            // It's a video
                            Ok(Some(HomeContent::Video(parse_home_video(data)?)))
                        }
                        Some(_) => {
                            // It's a song (Atv, OfficialSourceMusic, etc)
                            Ok(Some(HomeContent::Song(parse_home_song(data)?)))
                        }
                        None => {
                            // Watch playlist (radio, mix, etc)
                            Ok(Some(HomeContent::WatchPlaylist(parse_home_watch_playlist(
                                data,
                            )?)))
                        }
                    }
                } else if data.path_exists(NAVIGATION_VIDEO_ID) {
                    // It has a video ID, determine if song or video
                    let video_type: Option<YoutubeMusicVideoType> = data
                        .take_value_pointer(concatcp!(
                            "/navigationEndpoint/watchEndpoint",
                            "/watchEndpointMusicSupportedConfigs/watchEndpointMusicConfig/musicVideoType"
                        ))
                        .ok();

                    match video_type {
                        Some(YoutubeMusicVideoType::Ugc)
                        | Some(YoutubeMusicVideoType::Omv)
                        | Some(YoutubeMusicVideoType::Shoulder) => {
                            Ok(Some(HomeContent::Video(parse_home_video(data)?)))
                        }
                        _ => Ok(Some(HomeContent::Song(parse_home_song(data)?))),
                    }
                } else {
                    // Unknown type, skip
                    Ok(None)
                }
            }
            _ => {
                // Unknown page type, skip
                Ok(None)
            }
        }
    } else {
        // Not a musicTwoRowItemRenderer, skip
        Ok(None)
    }
}

/// Parse a home album from musicTwoRowItemRenderer.
fn parse_home_album(mut data: impl JsonCrawler) -> Result<HomeAlbum> {
    let title: String = data.take_value_pointer(TITLE_TEXT)?;
    let album_id: AlbumID<'static> = data.take_value_pointer(concatcp!(TITLE, NAVIGATION_BROWSE_ID))?;
    let thumbnails: Vec<Thumbnail> = data.take_value_pointer(THUMBNAIL_RENDERER)?;

    // Parse artists from subtitle runs
    let artists = parse_artists_from_subtitle_runs(&mut data)?;

    // Try to get year from subtitle (usually at position 2 or 4)
    let year: Option<String> = data
        .take_value_pointer(SUBTITLE2)
        .ok()
        .filter(|y: &String| y.chars().all(|c| c.is_ascii_digit()) && y.len() == 4);

    // Check if explicit
    let explicit = if data.path_exists(SUBTITLE_BADGE_LABEL) {
        Explicit::IsExplicit
    } else {
        Explicit::NotExplicit
    };

    // Album type from subtitle
    let album_type: Option<String> = data.take_value_pointer(SUBTITLE).ok();

    Ok(HomeAlbum {
        title,
        album_id,
        thumbnails,
        year,
        artists,
        explicit,
        album_type,
    })
}

/// Parse a home artist from musicTwoRowItemRenderer.
fn parse_home_artist(mut data: impl JsonCrawler) -> Result<HomeArtist> {
    let title: String = data.take_value_pointer(TITLE_TEXT)?;
    let channel_id: ArtistChannelID<'static> =
        data.take_value_pointer(concatcp!(TITLE, NAVIGATION_BROWSE_ID))?;
    let thumbnails: Vec<Thumbnail> = data.take_value_pointer(THUMBNAIL_RENDERER)?;

    // Subscribers from subtitle, extract just the number
    let subscribers: Option<String> = data
        .take_value_pointer::<String>(SUBTITLE)
        .ok()
        .map(|s| s.split(' ').next().unwrap_or(&s).to_string());

    Ok(HomeArtist {
        title,
        channel_id,
        subscribers,
        thumbnails,
    })
}

/// Parse a home playlist from musicTwoRowItemRenderer.
fn parse_home_playlist(mut data: impl JsonCrawler) -> Result<HomePlaylist> {
    let title: String = data.take_value_pointer(TITLE_TEXT)?;

    // Playlist ID - remove "VL" prefix if present
    let browse_id: String = data.take_value_pointer(concatcp!(TITLE, NAVIGATION_BROWSE_ID))?;
    let playlist_id = PlaylistID::from_raw(if browse_id.starts_with("VL") {
        browse_id[2..].to_string()
    } else {
        browse_id
    });

    let thumbnails: Vec<Thumbnail> = data.take_value_pointer(THUMBNAIL_RENDERER)?;

    // Parse description from subtitle runs
    let description: Option<String> = if let Ok(mut subtitle) = data.borrow_pointer("/subtitle") {
        if let Ok(runs) = subtitle.borrow_pointer("/runs") {
            let desc = runs
                .try_into_iter()?
                .filter_map(|mut r| r.take_value_pointer::<String>("/text").ok())
                .collect::<String>();
            if desc.is_empty() {
                None
            } else {
                Some(desc)
            }
        } else {
            None
        }
    } else {
        None
    };

    // Try to extract count from subtitle (e.g., "100 songs")
    let count: Option<String> = data
        .take_value_pointer::<String>(SUBTITLE2)
        .ok()
        .and_then(|s| {
            if s.chars().any(|c| c.is_ascii_digit()) {
                Some(s.split(' ').next().unwrap_or(&s).to_string())
            } else {
                None
            }
        });

    // Parse author from subtitle runs
    let author = parse_artists_from_subtitle_runs(&mut data).unwrap_or_default();

    Ok(HomePlaylist {
        title,
        playlist_id,
        thumbnails,
        description,
        count,
        author,
    })
}

/// Parse a home song from musicTwoRowItemRenderer.
fn parse_home_song(mut data: impl JsonCrawler) -> Result<HomeSong> {
    let title: String = data.take_value_pointer(TITLE_TEXT)?;
    let video_id: VideoID<'static> = data.take_value_pointer(NAVIGATION_VIDEO_ID)?;
    let thumbnails: Vec<Thumbnail> = data.take_value_pointer(THUMBNAIL_RENDERER)?;

    // Parse artists from subtitle runs
    let artists = parse_song_artists_from_runs(&mut data)?;

    // Try to get album from subtitle runs
    let album: Option<ParsedSongAlbum> = parse_album_from_subtitle_runs(&mut data)?;

    // Check if explicit
    let explicit = if data.path_exists(SUBTITLE_BADGE_LABEL) {
        Explicit::IsExplicit
    } else {
        Explicit::NotExplicit
    };

    // Playlist ID if present
    let playlist_id: Option<PlaylistID<'static>> =
        data.take_value_pointer(NAVIGATION_PLAYLIST_ID).ok();

    Ok(HomeSong {
        title,
        video_id,
        artists,
        thumbnails,
        album,
        explicit,
        playlist_id,
    })
}

/// Parse a home video from musicTwoRowItemRenderer.
fn parse_home_video(mut data: impl JsonCrawler) -> Result<HomeVideo> {
    let title: String = data.take_value_pointer(TITLE_TEXT)?;
    let video_id: VideoID<'static> = data.take_value_pointer(NAVIGATION_VIDEO_ID)?;
    let thumbnails: Vec<Thumbnail> = data.take_value_pointer(THUMBNAIL_RENDERER)?;

    // Parse artists from subtitle runs
    let artists = parse_song_artists_from_runs(&mut data)?;

    // Get views from subtitle runs (usually the last item)
    let views: Option<String> = data
        .borrow_pointer(SUBTITLE_RUNS)
        .ok()
        .and_then(|mut runs| {
            let items: Vec<String> = runs
                .try_iter_mut()
                .ok()?
                .filter_map(|mut r| r.take_value_pointer::<String>("/text").ok())
                .collect();
            items.last().cloned()
        })
        .filter(|v| v.contains("views") || v.chars().any(|c| c.is_ascii_digit()));

    // Playlist ID if present
    let playlist_id: Option<PlaylistID<'static>> =
        data.take_value_pointer(NAVIGATION_PLAYLIST_ID).ok();

    Ok(HomeVideo {
        title,
        video_id,
        artists,
        thumbnails,
        views,
        playlist_id,
    })
}

/// Parse a home watch playlist (mix/radio) from musicTwoRowItemRenderer.
fn parse_home_watch_playlist(mut data: impl JsonCrawler) -> Result<HomeWatchPlaylist> {
    let title: String = data.take_value_pointer(TITLE_TEXT)?;
    let playlist_id: PlaylistID<'static> = data.take_value_pointer(NAVIGATION_WATCH_PLAYLIST_ID)?;
    let thumbnails: Vec<Thumbnail> = data.take_value_pointer(THUMBNAIL_RENDERER)?;

    Ok(HomeWatchPlaylist {
        title,
        playlist_id,
        thumbnails,
    })
}

/// Parse artists from subtitle runs, skipping type specifiers and separators.
fn parse_artists_from_subtitle_runs(data: &mut impl JsonCrawler) -> Result<Vec<ParsedSongArtist>> {
    let mut artists = Vec::new();

    if let Ok(mut runs) = data.borrow_pointer(SUBTITLE_RUNS) {
        for (i, mut run) in runs.try_iter_mut()?.enumerate() {
            // Skip separators (odd indices)
            if i % 2 != 0 {
                continue;
            }

            let text: String = run.take_value_pointer("/text")?;

            // Skip type specifiers like "Album", "Single", "Song", etc.
            if text == "Album"
                || text == "Single"
                || text == "EP"
                || text == "Song"
                || text == "Video"
            {
                continue;
            }

            // Skip if it looks like a year
            if text.len() == 4 && text.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            let id: Option<ArtistChannelID<'static>> =
                run.take_value_pointer(NAVIGATION_BROWSE_ID).ok();

            artists.push(ParsedSongArtist { name: text, id });
        }
    }

    Ok(artists)
}

/// Parse song artists from subtitle runs.
fn parse_song_artists_from_runs(data: &mut impl JsonCrawler) -> Result<Vec<ParsedSongArtist>> {
    let mut artists = Vec::new();

    if let Ok(mut runs) = data.borrow_pointer(SUBTITLE_RUNS) {
        let items: Vec<_> = runs.try_iter_mut()?.collect();

        for (i, mut run) in items.into_iter().enumerate() {
            // Skip separators (odd indices)
            if i % 2 != 0 {
                continue;
            }

            let text: String = run.take_value_pointer("/text")?;

            // Skip type specifiers
            if text == "Song" || text == "Video" {
                continue;
            }

            // Skip views count (usually last item)
            if text.contains("views") {
                continue;
            }

            // Check if this is an artist (has navigation endpoint or is not the last element)
            let id: Option<ArtistChannelID<'static>> =
                run.take_value_pointer(NAVIGATION_BROWSE_ID).ok();

            // If it's an album reference (has browse ID starting with "MPRE"), skip as artist
            if let Some(ref browse_id) = id {
                if browse_id.get_raw().starts_with("MPRE") {
                    continue;
                }
            }

            artists.push(ParsedSongArtist { name: text, id });
        }
    }

    Ok(artists)
}

/// Parse album reference from subtitle runs.
/// Returns None if no album is found.
fn parse_album_from_subtitle_runs(data: &mut impl JsonCrawler) -> Result<Option<ParsedSongAlbum>> {
    if let Ok(mut runs) = data.borrow_pointer(SUBTITLE_RUNS) {
        for mut run in runs.try_iter_mut()? {
            let browse_id: Option<String> = run.take_value_pointer(NAVIGATION_BROWSE_ID).ok();

            if let Some(id) = browse_id {
                // Check if this is an album (browse ID starts with "MPRE")
                if id.starts_with("MPRE") {
                    let name: String = run.take_value_pointer("/text")?;
                    return Ok(Some(ParsedSongAlbum {
                        name,
                        id: AlbumID::from_raw(id),
                    }));
                }
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_get_home_dummy_json() {
        // TODO: Add test JSON file when available
    }
}
