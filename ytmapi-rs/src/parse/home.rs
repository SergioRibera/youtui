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

        // Parse mood chips and sections from contents
        let contents = section_list.navigate_pointer("/contents")?;
        let (chips, sections) = parse_home_contents(contents)?;

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

/// Parse home feed contents, extracting mood chips and sections.
fn parse_home_contents(
    mut contents: JsonCrawlerOwned,
) -> Result<(Vec<HomeMoodChip>, Vec<HomeSection>)> {
    let mut chips = Vec::new();
    let mut sections = Vec::new();

    for row in contents.try_iter_mut()? {
        // Try to parse mood chips from chip cloud renderer
        if row.path_exists("/musicImmersiveHeaderRenderer/chipCloud/chipCloudRenderer/chips") {
            if let Ok(mut chip_cloud) = row.navigate_pointer("/musicImmersiveHeaderRenderer/chipCloud/chipCloudRenderer/chips") {
                for mut chip in chip_cloud.try_iter_mut()? {
                    if let Ok(chip) = parse_mood_chip(&mut chip) {
                        chips.push(chip);
                    }
                }
            }
            continue;
        }

        // Try to get carousel shelf
        if row.path_exists(CAROUSEL) {
            if let Ok(carousel) = row.navigate_pointer(CAROUSEL) {
                if let Some(section) = parse_carousel_section(carousel)? {
                    sections.push(section);
                }
            }
        }
    }

    Ok((chips, sections))
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

/// Get the full subtitle text from subtitle runs, joined together.
fn get_full_subtitle(data: &mut impl JsonCrawler) -> Option<String> {
    if let Ok(runs) = data.borrow_pointer(SUBTITLE_RUNS) {
        let parts: Vec<String> = runs
            .try_into_iter()
            .ok()?
            .filter_map(|mut r| r.take_value_pointer::<String>("/text").ok())
            .collect();
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(""))
        }
    } else {
        None
    }
}

/// Parse a home album from musicTwoRowItemRenderer.
fn parse_home_album(mut data: impl JsonCrawler) -> Result<HomeAlbum> {
    let title: String = data.take_value_pointer(TITLE_TEXT)?;
    let album_id: AlbumID<'static> = data.take_value_pointer(concatcp!(TITLE, NAVIGATION_BROWSE_ID))?;
    let thumbnails: Vec<Thumbnail> = data.take_value_pointer(THUMBNAIL_RENDERER)?;

    // Get full subtitle for localized text
    let subtitle = get_full_subtitle(&mut data);

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
        subtitle,
    })
}

/// Parse a home artist from musicTwoRowItemRenderer.
fn parse_home_artist(mut data: impl JsonCrawler) -> Result<HomeArtist> {
    let title: String = data.take_value_pointer(TITLE_TEXT)?;
    let channel_id: ArtistChannelID<'static> =
        data.take_value_pointer(concatcp!(TITLE, NAVIGATION_BROWSE_ID))?;
    let thumbnails: Vec<Thumbnail> = data.take_value_pointer(THUMBNAIL_RENDERER)?;

    // Get full subtitle for localized text (e.g., "1.5M de suscriptores")
    let subtitle = get_full_subtitle(&mut data);

    // Subscribers from subtitle, extract just the number
    let subscribers: Option<String> = subtitle
        .as_ref()
        .map(|s| s.split(' ').next().unwrap_or(s).to_string());

    Ok(HomeArtist {
        title,
        channel_id,
        subscribers,
        thumbnails,
        subtitle,
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

    // Get full subtitle for localized text
    let subtitle = get_full_subtitle(&mut data);

    // Use subtitle as description (they're the same in this context)
    let description = subtitle.clone();

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
        subtitle,
    })
}

/// Parse a home song from musicTwoRowItemRenderer.
fn parse_home_song(mut data: impl JsonCrawler) -> Result<HomeSong> {
    let title: String = data.take_value_pointer(TITLE_TEXT)?;
    let video_id: VideoID<'static> = data.take_value_pointer(NAVIGATION_VIDEO_ID)?;
    let thumbnails: Vec<Thumbnail> = data.take_value_pointer(THUMBNAIL_RENDERER)?;

    // Get full subtitle for localized text
    let subtitle = get_full_subtitle(&mut data);

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
        subtitle,
    })
}

/// Parse a home video from musicTwoRowItemRenderer.
fn parse_home_video(mut data: impl JsonCrawler) -> Result<HomeVideo> {
    let title: String = data.take_value_pointer(TITLE_TEXT)?;
    let video_id: VideoID<'static> = data.take_value_pointer(NAVIGATION_VIDEO_ID)?;
    let thumbnails: Vec<Thumbnail> = data.take_value_pointer(THUMBNAIL_RENDERER)?;

    // Get full subtitle for localized text
    let subtitle = get_full_subtitle(&mut data);

    // Parse artists from subtitle runs
    let artists = parse_song_artists_from_runs(&mut data)?;

    // Get views from subtitle (usually the last part)
    let views: Option<String> = subtitle
        .as_ref()
        .and_then(|s| {
            // Find the last part that contains numbers or "views"
            s.split(" • ")
                .last()
                .filter(|v| v.contains("views") || v.contains("visualizaciones") || v.chars().any(|c| c.is_ascii_digit()))
                .map(|s| s.to_string())
        });

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
        subtitle,
    })
}

/// Parse a home watch playlist (mix/radio) from musicTwoRowItemRenderer.
fn parse_home_watch_playlist(mut data: impl JsonCrawler) -> Result<HomeWatchPlaylist> {
    let title: String = data.take_value_pointer(TITLE_TEXT)?;
    let playlist_id: PlaylistID<'static> = data.take_value_pointer(NAVIGATION_WATCH_PLAYLIST_ID)?;
    let thumbnails: Vec<Thumbnail> = data.take_value_pointer(THUMBNAIL_RENDERER)?;

    // Get full subtitle for localized text
    let subtitle = get_full_subtitle(&mut data);

    Ok(HomeWatchPlaylist {
        title,
        playlist_id,
        thumbnails,
        subtitle,
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

            // Skip if text is null or missing
            let Some(text) = run.take_value_pointer::<String>("/text").ok() else {
                continue;
            };

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

            // Skip if text is null or missing
            let Some(text) = run.take_value_pointer::<String>("/text").ok() else {
                continue;
            };

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
                    // Skip if text is null or missing
                    let Some(name) = run.take_value_pointer::<String>("/text").ok() else {
                        continue;
                    };
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
