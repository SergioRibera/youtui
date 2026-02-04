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

/// Header path for carousel shelf
const CAROUSEL_HEADER: &str = "/header/musicCarouselShelfBasicHeaderRenderer";

/// A mood/category chip shown at the top of the home feed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HomeMoodChip {
    pub title: String,
    pub params: MoodCategoryParams<'static>,
}

/// A collection of home sections returned from the home feed query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HomeSections {
    /// Mood/category chips shown at the top (e.g., "Energía", "Relax", "Fiesta")
    pub chips: Vec<HomeMoodChip>,
    /// The actual content sections
    sections: Vec<HomeSection>,
}

impl HomeSections {
    /// Creates a new `HomeSections` from chips and sections.
    pub fn new(chips: Vec<HomeMoodChip>, sections: Vec<HomeSection>) -> Self {
        Self { chips, sections }
    }

    /// Creates a new `HomeSections` with only sections (no chips).
    pub fn from_sections(sections: Vec<HomeSection>) -> Self {
        Self {
            chips: Vec::new(),
            sections,
        }
    }

    /// Extends this collection with sections from another `HomeSections`.
    /// Note: chips are only kept from the first response.
    pub fn extend(&mut self, other: HomeSections) {
        self.sections.extend(other.sections);
    }

    /// Truncates the sections to the given length.
    pub fn truncate(&mut self, len: usize) {
        self.sections.truncate(len);
    }

    /// Returns a reference to the sections.
    pub fn sections(&self) -> &[HomeSection] {
        &self.sections
    }
}

impl Deref for HomeSections {
    type Target = Vec<HomeSection>;

    fn deref(&self) -> &Self::Target {
        &self.sections
    }
}

impl DerefMut for HomeSections {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sections
    }
}

impl IntoIterator for HomeSections {
    type Item = HomeSection;
    type IntoIter = std::vec::IntoIter<HomeSection>;

    fn into_iter(self) -> Self::IntoIter {
        self.sections.into_iter()
    }
}

impl<'a> IntoIterator for &'a HomeSections {
    type Item = &'a HomeSection;
    type IntoIter = std::slice::Iter<'a, HomeSection>;

    fn into_iter(self) -> Self::IntoIter {
        self.sections.iter()
    }
}

/// A section on the home page containing a title and mixed content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HomeSection {
    /// The main title of the section (e.g., "Vuelve a escucharlo")
    pub title: String,
    /// Optional strapline text above the title (e.g., "SERGIO RIBERA")
    pub strapline: Option<String>,
    /// Optional thumbnail for the section header
    pub thumbnail: Option<Thumbnail>,
    /// The content items in this section
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
    /// Full subtitle text (localized, e.g., "Album • Artist • 2024")
    pub subtitle: Option<String>,
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
    /// Full subtitle text (localized, e.g., "Playlist • Author")
    pub subtitle: Option<String>,
}

/// An artist shown on the home page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HomeArtist {
    pub title: String,
    pub channel_id: ArtistChannelID<'static>,
    pub subscribers: Option<String>,
    pub thumbnails: Vec<Thumbnail>,
    /// Full subtitle text (localized, e.g., "1.5M de suscriptores")
    pub subtitle: Option<String>,
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
    /// Full subtitle text (localized, e.g., "Canción • Artist • Album")
    pub subtitle: Option<String>,
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
    /// Full subtitle text (localized, e.g., "Video • Artist • 1M de visualizaciones")
    pub subtitle: Option<String>,
}

/// A watch playlist shown on the home page (auto-generated mixes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HomeWatchPlaylist {
    pub title: String,
    pub playlist_id: PlaylistID<'static>,
    pub thumbnails: Vec<Thumbnail>,
    /// Full subtitle text (localized)
    pub subtitle: Option<String>,
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

        // Parse mood chips from header (chips are in sectionListRenderer/header/chipCloudRenderer/chips)
        let chips = parse_header_chips(&mut section_list)?;

        // Parse sections from contents
        let contents = section_list.navigate_pointer("/contents")?;
        let sections = parse_home_sections(contents)?;

        Ok((HomeSections::new(chips, sections), continuation_params))
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
        // Continuation responses don't have chips, only sections
        let sections = if let Ok(contents) = section_list.navigate_pointer("/contents") {
            parse_mixed_content(contents)?
        } else {
            Vec::new()
        };

        Ok((HomeSections::from_sections(sections), continuation_params))
    }
}

/// Parse mood chips from the sectionListRenderer header.
fn parse_header_chips(section_list: &mut impl JsonCrawler) -> Result<Vec<HomeMoodChip>> {
    let mut chips = Vec::new();

    // Chips are in header/chipCloudRenderer/chips
    if let Ok(mut chip_cloud) = section_list.borrow_pointer("/header/chipCloudRenderer/chips") {
        for mut chip in chip_cloud.try_iter_mut()? {
            if let Ok(parsed_chip) = parse_mood_chip(&mut chip) {
                chips.push(parsed_chip);
            }
        }
    }

    Ok(chips)
}

/// Parse home feed sections from contents.
fn parse_home_sections(mut contents: JsonCrawlerOwned) -> Result<Vec<HomeSection>> {
    let mut sections = Vec::new();

    for row in contents.try_iter_mut()? {
        // Try to get carousel shelf
        if row.path_exists(CAROUSEL) {
            if let Ok(carousel) = row.navigate_pointer(CAROUSEL) {
                if let Some(section) = parse_carousel_section(carousel)? {
                    sections.push(section);
                }
            }
        }
    }

    Ok(sections)
}

/// Parse a mood chip from the chip cloud.
fn parse_mood_chip(chip: &mut impl JsonCrawler) -> Result<HomeMoodChip> {
    let mut renderer = chip.borrow_pointer("/chipCloudChipRenderer")?;
    let title: String = renderer.take_value_pointer("/text/runs/0/text")?;
    let params: MoodCategoryParams<'static> = renderer.take_value_pointer(
        "/navigationEndpoint/browseEndpoint/params",
    )?;
    Ok(HomeMoodChip { title, params })
}

/// Parse a carousel section with header info.
fn parse_carousel_section(mut carousel: impl JsonCrawler) -> Result<Option<HomeSection>> {
    // Skip if no contents
    if !carousel.path_exists("/contents") {
        return Ok(None);
    }

    // Try to get header info
    let mut header = carousel.borrow_pointer(CAROUSEL_HEADER)?;

    // Get title (required)
    let title: String = header.take_value_pointer(concatcp!(TITLE, "/text"))?;

    // Get optional strapline (text above title, e.g., "SERGIO RIBERA")
    let strapline: Option<String> = header
        .take_value_pointer("/strapline/runs/0/text")
        .ok();

    // Get optional thumbnail
    let thumbnail: Option<Thumbnail> = header
        .take_value_pointer("/thumbnail/musicThumbnailRenderer/thumbnail/thumbnails/0")
        .ok();

    // Drop header borrow before navigating to contents
    drop(header);

    let mut contents = Vec::new();

    // Parse each item in the carousel
    for result in carousel.navigate_pointer("/contents")?.try_iter_mut()? {
        if let Some(content) = parse_home_item(result)? {
            contents.push(content);
        }
    }

    if contents.is_empty() {
        return Ok(None);
    }

    Ok(Some(HomeSection {
        title,
        strapline,
        thumbnail,
        contents,
    }))
}

/// Parse mixed content sections from continuation response.
fn parse_mixed_content(mut sections: JsonCrawlerOwned) -> Result<Vec<HomeSection>> {
    let mut items = Vec::new();

    for row in sections.try_iter_mut()? {
        // Try to get carousel shelf
        if row.path_exists(CAROUSEL) {
            if let Ok(carousel) = row.navigate_pointer(CAROUSEL) {
                if let Some(section) = parse_carousel_section(carousel)? {
                    items.push(section);
                }
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

/// Parsed subtitle information containing all extracted data.
struct ParsedSubtitle {
    /// Full subtitle text joined together
    full_text: Option<String>,
    /// Individual text parts (even indices only, excluding separators)
    parts: Vec<SubtitlePart>,
}

/// A single part from the subtitle runs.
struct SubtitlePart {
    text: String,
    browse_id: Option<String>,
}

/// Parse all subtitle information in one pass.
/// This extracts the full text and structured parts before the data is consumed.
fn parse_subtitle_data(data: &mut impl JsonCrawler) -> ParsedSubtitle {
    let mut full_parts: Vec<String> = Vec::new();
    let mut structured_parts: Vec<SubtitlePart> = Vec::new();

    if let Ok(mut runs) = data.borrow_pointer(SUBTITLE_RUNS) {
        if let Ok(iter) = runs.try_iter_mut() {
            for (i, mut run) in iter.enumerate() {
                // Get text (might be null)
                let text: Option<String> = run.take_value_pointer("/text").ok();

                if let Some(ref t) = text {
                    full_parts.push(t.clone());

                    // Only collect even indices (skip separators like " • ")
                    if i % 2 == 0 {
                        let browse_id: Option<String> =
                            run.take_value_pointer(NAVIGATION_BROWSE_ID).ok();
                        structured_parts.push(SubtitlePart {
                            text: t.clone(),
                            browse_id,
                        });
                    }
                }
            }
        }
    }

    ParsedSubtitle {
        full_text: if full_parts.is_empty() {
            None
        } else {
            Some(full_parts.join(""))
        },
        parts: structured_parts,
    }
}

/// Parse a home album from musicTwoRowItemRenderer.
fn parse_home_album(mut data: impl JsonCrawler) -> Result<HomeAlbum> {
    let title: String = data.take_value_pointer(TITLE_TEXT)?;
    let album_id: AlbumID<'static> = data.take_value_pointer(concatcp!(TITLE, NAVIGATION_BROWSE_ID))?;
    let thumbnails: Vec<Thumbnail> = data.take_value_pointer(THUMBNAIL_RENDERER)?;

    // Check if explicit
    let explicit = if data.path_exists(SUBTITLE_BADGE_LABEL) {
        Explicit::IsExplicit
    } else {
        Explicit::NotExplicit
    };

    // Parse all subtitle data in one pass
    let subtitle_data = parse_subtitle_data(&mut data);

    // Extract album type (first part, e.g., "Album", "Single", "EP")
    let album_type = subtitle_data.parts.first().map(|p| p.text.clone());

    // Extract year (4-digit number)
    let year = subtitle_data
        .parts
        .iter()
        .find(|p| p.text.len() == 4 && p.text.chars().all(|c| c.is_ascii_digit()))
        .map(|p| p.text.clone());

    // Extract artists (parts with browse IDs that are artist channels, not albums)
    let artists: Vec<ParsedSongArtist> = subtitle_data
        .parts
        .iter()
        .filter(|p| {
            // Skip type specifiers and years
            !matches!(p.text.as_str(), "Album" | "Single" | "EP" | "Song" | "Video")
                && !(p.text.len() == 4 && p.text.chars().all(|c| c.is_ascii_digit()))
                // Skip album references (browse ID starts with "MPRE")
                && !p.browse_id.as_ref().map(|id| id.starts_with("MPRE")).unwrap_or(false)
        })
        .map(|p| ParsedSongArtist {
            name: p.text.clone(),
            id: p.browse_id.as_ref().map(|id| ArtistChannelID::from_raw(id.clone())),
        })
        .collect();

    Ok(HomeAlbum {
        title,
        album_id,
        thumbnails,
        year,
        artists,
        explicit,
        album_type,
        subtitle: subtitle_data.full_text,
    })
}

/// Parse a home artist from musicTwoRowItemRenderer.
fn parse_home_artist(mut data: impl JsonCrawler) -> Result<HomeArtist> {
    let title: String = data.take_value_pointer(TITLE_TEXT)?;
    let channel_id: ArtistChannelID<'static> =
        data.take_value_pointer(concatcp!(TITLE, NAVIGATION_BROWSE_ID))?;
    let thumbnails: Vec<Thumbnail> = data.take_value_pointer(THUMBNAIL_RENDERER)?;

    // Parse all subtitle data in one pass
    let subtitle_data = parse_subtitle_data(&mut data);

    // Subscribers from subtitle, extract just the number
    let subscribers: Option<String> = subtitle_data
        .full_text
        .as_ref()
        .map(|s| s.split(' ').next().unwrap_or(s).to_string());

    Ok(HomeArtist {
        title,
        channel_id,
        subscribers,
        thumbnails,
        subtitle: subtitle_data.full_text,
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

    // Parse all subtitle data in one pass
    let subtitle_data = parse_subtitle_data(&mut data);

    // Try to extract count from subtitle parts (e.g., "100 songs")
    let count: Option<String> = subtitle_data
        .parts
        .iter()
        .find(|p| p.text.chars().any(|c| c.is_ascii_digit()))
        .map(|p| p.text.split(' ').next().unwrap_or(&p.text).to_string());

    // Extract authors (parts with browse IDs, skipping first part which is usually "Playlist")
    let author: Vec<ParsedSongArtist> = subtitle_data
        .parts
        .iter()
        .skip(1) // Skip "Playlist" type
        .filter(|p| !p.text.chars().any(|c| c.is_ascii_digit())) // Skip counts
        .map(|p| ParsedSongArtist {
            name: p.text.clone(),
            id: p.browse_id.as_ref().map(|id| ArtistChannelID::from_raw(id.clone())),
        })
        .collect();

    // Use full subtitle as description
    let description = subtitle_data.full_text.clone();

    Ok(HomePlaylist {
        title,
        playlist_id,
        thumbnails,
        description,
        count,
        author,
        subtitle: subtitle_data.full_text,
    })
}

/// Parse a home song from musicTwoRowItemRenderer.
fn parse_home_song(mut data: impl JsonCrawler) -> Result<HomeSong> {
    let title: String = data.take_value_pointer(TITLE_TEXT)?;
    let video_id: VideoID<'static> = data.take_value_pointer(NAVIGATION_VIDEO_ID)?;
    let thumbnails: Vec<Thumbnail> = data.take_value_pointer(THUMBNAIL_RENDERER)?;

    // Check if explicit
    let explicit = if data.path_exists(SUBTITLE_BADGE_LABEL) {
        Explicit::IsExplicit
    } else {
        Explicit::NotExplicit
    };

    // Playlist ID if present
    let playlist_id: Option<PlaylistID<'static>> =
        data.take_value_pointer(NAVIGATION_PLAYLIST_ID).ok();

    // Parse all subtitle data in one pass
    let subtitle_data = parse_subtitle_data(&mut data);

    // Extract artists (parts with artist channel IDs, not albums)
    let artists: Vec<ParsedSongArtist> = subtitle_data
        .parts
        .iter()
        .filter(|p| {
            // Skip type specifiers
            !matches!(p.text.as_str(), "Song" | "Video")
                // Skip views count
                && !p.text.contains("views") && !p.text.contains("visualizaciones")
                // Skip album references (browse ID starts with "MPRE")
                && !p.browse_id.as_ref().map(|id| id.starts_with("MPRE")).unwrap_or(false)
        })
        .map(|p| ParsedSongArtist {
            name: p.text.clone(),
            id: p.browse_id.as_ref().map(|id| ArtistChannelID::from_raw(id.clone())),
        })
        .collect();

    // Extract album (part with browse ID starting with "MPRE")
    let album: Option<ParsedSongAlbum> = subtitle_data
        .parts
        .iter()
        .find(|p| p.browse_id.as_ref().map(|id| id.starts_with("MPRE")).unwrap_or(false))
        .map(|p| ParsedSongAlbum {
            name: p.text.clone(),
            id: AlbumID::from_raw(p.browse_id.clone().unwrap()),
        });

    Ok(HomeSong {
        title,
        video_id,
        artists,
        thumbnails,
        album,
        explicit,
        playlist_id,
        subtitle: subtitle_data.full_text,
    })
}

/// Parse a home video from musicTwoRowItemRenderer.
fn parse_home_video(mut data: impl JsonCrawler) -> Result<HomeVideo> {
    let title: String = data.take_value_pointer(TITLE_TEXT)?;
    let video_id: VideoID<'static> = data.take_value_pointer(NAVIGATION_VIDEO_ID)?;
    let thumbnails: Vec<Thumbnail> = data.take_value_pointer(THUMBNAIL_RENDERER)?;

    // Playlist ID if present
    let playlist_id: Option<PlaylistID<'static>> =
        data.take_value_pointer(NAVIGATION_PLAYLIST_ID).ok();

    // Parse all subtitle data in one pass
    let subtitle_data = parse_subtitle_data(&mut data);

    // Extract artists (parts that are not type specifiers or view counts)
    let artists: Vec<ParsedSongArtist> = subtitle_data
        .parts
        .iter()
        .filter(|p| {
            // Skip type specifiers
            !matches!(p.text.as_str(), "Song" | "Video")
                // Skip views count
                && !p.text.contains("views") && !p.text.contains("visualizaciones")
                && !p.text.chars().any(|c| c.is_ascii_digit())
        })
        .map(|p| ParsedSongArtist {
            name: p.text.clone(),
            id: p.browse_id.as_ref().map(|id| ArtistChannelID::from_raw(id.clone())),
        })
        .collect();

    // Get views from subtitle parts (contains numbers or "views")
    let views: Option<String> = subtitle_data
        .parts
        .iter()
        .find(|p| p.text.contains("views") || p.text.contains("visualizaciones") || p.text.chars().any(|c| c.is_ascii_digit()))
        .map(|p| p.text.clone());

    Ok(HomeVideo {
        title,
        video_id,
        artists,
        thumbnails,
        views,
        playlist_id,
        subtitle: subtitle_data.full_text,
    })
}

/// Parse a home watch playlist (mix/radio) from musicTwoRowItemRenderer.
fn parse_home_watch_playlist(mut data: impl JsonCrawler) -> Result<HomeWatchPlaylist> {
    let title: String = data.take_value_pointer(TITLE_TEXT)?;
    let playlist_id: PlaylistID<'static> = data.take_value_pointer(NAVIGATION_WATCH_PLAYLIST_ID)?;
    let thumbnails: Vec<Thumbnail> = data.take_value_pointer(THUMBNAIL_RENDERER)?;

    // Parse all subtitle data in one pass
    let subtitle_data = parse_subtitle_data(&mut data);

    Ok(HomeWatchPlaylist {
        title,
        playlist_id,
        thumbnails,
        subtitle: subtitle_data.full_text,
    })
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_get_home_dummy_json() {
        // TODO: Add test JSON file when available
    }
}
