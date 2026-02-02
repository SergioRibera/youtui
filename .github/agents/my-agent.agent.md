---
# Fill in the fields below to create a basic custom agent for your repository.
# The Copilot CLI can be used for local testing: https://gh.io/customagents/cli
# To make this agent available, merge this file into the default repository branch.
# For format details, see: https://gh.io/customagents/config

name: ytmusicapi expert
description: Implement get home api
---
Estás implementando un cliente para YouTube Music en Rust. Ya existen implementaciones similares en Python (referencia proporcionada) y en Rust para otros endpoints (library y upload). Necesitas implementar el método get_home() que obtiene el feed de inicio de YouTube Music.

Tengo estos tipos sugeridos para el home:
```rust
pub struct HomeSection { pub title: String, pub contents: Vec<HomeContent>, }
pub enum HomeContent { Album(AlbumContent), Playlist(PlaylistContent), Artist(ArtistContent), Song(SongContent), Video(VideoContent), }
pub struct AlbumContent { pub title: String, pub browse_id: Option<String>, pub thumbnails: Vec<Thumbnail>, pub year: Option<String>, pub artists: Vec<Artist>, pub is_explicit: bool, }
pub struct PlaylistContent { pub title: String, pub playlist_id: String, pub thumbnails: Vec<Thumbnail>, pub description: Option<String>, pub count: Option<String>, pub author: Vec<Artist>, }
pub struct ArtistContent { pub title: String, pub browse_id: String, pub subscribers: Option<String>, pub thumbnails: Vec<Thumbnail>, }
pub struct SongContent { pub title: String, pub video_id: String, pub artists: Vec<Artist>, pub thumbnails: Vec<Thumbnail>, pub album: Option<AlbumReference>, pub views: Option<String>, }
pub struct VideoContent { pub title: String, pub video_id: String, pub artists: Vec<Artist>, pub thumbnails: Vec<Thumbnail>, pub views: Option<String>, }
pub struct Artist { pub name: String, pub id: Option<String>, }
pub struct AlbumReference { pub name: String, pub id: String, }
```

Con estas referencias en python en las que quiero que te bases

mixins/browsing.py
```py
class BrowsingMixin(MixinProtocol):
    def get_home(self, limit: int = 3) -> JsonList:
        """
        Get the home page.
        The home page is structured as titled rows, returning 3 rows of music suggestions at a time.
        Content varies and may contain artist, album, song or playlist suggestions, sometimes mixed within the same row

        :param limit: Number of rows on the home page to return
        :return: List of dictionaries keyed with 'title' text and 'contents' list
        """
        endpoint = "browse"
        body = {"browseId": "FEmusic_home"}
        response = self._send_request(endpoint, body)
        results = nav(response, SINGLE_COLUMN_TAB + SECTION_LIST)
        home = parse_mixed_content(results)

        section_list = nav(response, [*SINGLE_COLUMN_TAB, "sectionListRenderer"])
        if "continuations" in section_list:
            request_func: RequestFuncType = lambda additionalParams: self._send_request(
                endpoint, body, additionalParams
            )

            home.extend(
                get_continuations(
                    section_list,
                    "sectionListContinuation",
                    limit - len(home),
                    request_func,
                    parse_mixed_content,
                )
            )

        return home
```

navigation.py
```py
CONTENT = ["contents", 0]
RUN_TEXT = ["runs", 0, "text"]
TAB_CONTENT = ["tabs", 0, "tabRenderer", "content"]
TAB_1_CONTENT = ["tabs", 1, "tabRenderer", "content"]
TAB_2_CONTENT = ["tabs", 2, "tabRenderer", "content"]
TWO_COLUMN_RENDERER = ["contents", "twoColumnBrowseResultsRenderer"]
SINGLE_COLUMN = ["contents", "singleColumnBrowseResultsRenderer"]
SINGLE_COLUMN_TAB = SINGLE_COLUMN + TAB_CONTENT
SECTION = ["sectionListRenderer"]
SECTION_LIST = [*SECTION, "contents"]
SECTION_LIST_ITEM = SECTION + CONTENT
RESPONSIVE_HEADER = ["musicResponsiveHeaderRenderer"]
ITEM_SECTION = ["itemSectionRenderer", *CONTENT]
MUSIC_SHELF = ["musicShelfRenderer"]
GRID = ["gridRenderer"]
GRID_ITEMS = [*GRID, "items"]
MENU = ["menu", "menuRenderer"]
MENU_ITEMS = [*MENU, "items"]
MENU_LIKE_STATUS = [*MENU, "topLevelButtons", 0, "likeButtonRenderer", "likeStatus"]
MENU_SERVICE = ["menuServiceItemRenderer", "serviceEndpoint"]
TOGGLE_MENU = "toggleMenuServiceItemRenderer"
OVERLAY_RENDERER = ["musicItemThumbnailOverlayRenderer", "content", "musicPlayButtonRenderer"]
PLAY_BUTTON = ["overlay", *OVERLAY_RENDERER]
NAVIGATION_BROWSE = ["navigationEndpoint", "browseEndpoint"]
NAVIGATION_BROWSE_ID = [*NAVIGATION_BROWSE, "browseId"]
PAGE_TYPE = ["browseEndpointContextSupportedConfigs", "browseEndpointContextMusicConfig", "pageType"]
WATCH_VIDEO_ID = ["watchEndpoint", "videoId"]
PLAYLIST_ID = ["playlistId"]
WATCH_PLAYLIST_ID = ["watchEndpoint", *PLAYLIST_ID]
NAVIGATION_VIDEO_ID = ["navigationEndpoint", *WATCH_VIDEO_ID]
QUEUE_VIDEO_ID = ["queueAddEndpoint", "queueTarget", "videoId"]
NAVIGATION_PLAYLIST_ID = ["navigationEndpoint", *WATCH_PLAYLIST_ID]
WATCH_PID = ["watchPlaylistEndpoint", *PLAYLIST_ID]
NAVIGATION_WATCH_PLAYLIST_ID = ["navigationEndpoint", *WATCH_PID]
NAVIGATION_VIDEO_TYPE = [
    "watchEndpoint",
    "watchEndpointMusicSupportedConfigs",
    "watchEndpointMusicConfig",
    "musicVideoType",
]
ICON_TYPE = ["icon", "iconType"]
TOGGLED_BUTTON = ["toggleButtonRenderer", "isToggled"]
TITLE = ["title", "runs", 0]
TITLE_TEXT = ["title", *RUN_TEXT]
TEXT_RUNS = ["text", "runs"]
TEXT_RUN = [*TEXT_RUNS, 0]
TEXT_RUN_TEXT = [*TEXT_RUN, "text"]
SUBTITLE = ["subtitle", *RUN_TEXT]
SUBTITLE_RUNS = ["subtitle", "runs"]
SUBTITLE_RUN = [*SUBTITLE_RUNS, 0]
SUBTITLE2 = [*SUBTITLE_RUNS, 2, "text"]
SUBTITLE3 = [*SUBTITLE_RUNS, 4, "text"]
THUMBNAIL = ["thumbnail", "thumbnails"]
THUMBNAILS = ["thumbnail", "musicThumbnailRenderer", *THUMBNAIL]
THUMBNAIL_RENDERER = ["thumbnailRenderer", "musicThumbnailRenderer", *THUMBNAIL]
THUMBNAIL_OVERLAY_NAVIGATION = ["thumbnailOverlay", *OVERLAY_RENDERER, "playNavigationEndpoint"]
THUMBNAIL_OVERLAY = [*THUMBNAIL_OVERLAY_NAVIGATION, *WATCH_PID]
THUMBNAIL_CROPPED = ["thumbnail", "croppedSquareThumbnailRenderer", *THUMBNAIL]
BADGE_PATH = [0, "musicInlineBadgeRenderer", "accessibilityData", "accessibilityData", "label"]
BADGE_LABEL = ["badges", *BADGE_PATH]
SUBTITLE_BADGE_LABEL = ["subtitleBadges", *BADGE_PATH]
CATEGORY_TITLE = ["musicNavigationButtonRenderer", "buttonText", *RUN_TEXT]
CATEGORY_PARAMS = ["musicNavigationButtonRenderer", "clickCommand", "browseEndpoint", "params"]
MMRIR = "musicMultiRowListItemRenderer"
MRLIR = "musicResponsiveListItemRenderer"
MTRIR = "musicTwoRowItemRenderer"
MNIR = "menuNavigationItemRenderer"
TASTE_PROFILE_ITEMS = ["contents", "tastebuilderRenderer", "contents"]
TASTE_PROFILE_ARTIST = ["title", "runs"]
SECTION_LIST_CONTINUATION = ["continuationContents", "sectionListContinuation"]
MENU_PLAYLIST_ID = [*MENU_ITEMS, 0, MNIR, *NAVIGATION_WATCH_PLAYLIST_ID]
MULTI_SELECT = ["musicMultiSelectMenuItemRenderer"]
HEADER = ["header"]
HEADER_DETAIL = [*HEADER, "musicDetailHeaderRenderer"]
EDITABLE_PLAYLIST_DETAIL_HEADER = ["musicEditablePlaylistDetailHeaderRenderer"]
HEADER_EDITABLE_DETAIL = [*HEADER, *EDITABLE_PLAYLIST_DETAIL_HEADER]
HEADER_SIDE = [*HEADER, "musicSideAlignedItemRenderer"]
HEADER_MUSIC_VISUAL = [*HEADER, "musicVisualHeaderRenderer"]
DESCRIPTION_SHELF = ["musicDescriptionShelfRenderer"]
DESCRIPTION = ["description", *RUN_TEXT]
CAROUSEL = ["musicCarouselShelfRenderer"]
IMMERSIVE_CAROUSEL = ["musicImmersiveCarouselShelfRenderer"]
CAROUSEL_CONTENTS = [*CAROUSEL, "contents"]
CAROUSEL_TITLE = [*HEADER, "musicCarouselShelfBasicHeaderRenderer", *TITLE]
CARD_SHELF_TITLE = [*HEADER, "musicCardShelfHeaderBasicRenderer", *TITLE_TEXT]


@overload
def nav(root: JsonDict | None, items: list[Any], none_if_absent: Literal[False] = False) -> Any:
    """overload for mypy only"""


@overload
def nav(root: JsonDict | None, items: list[Any], none_if_absent: Literal[True] = True) -> Any | None:
    """overload for mypy only"""


def nav(root: JsonDict | None, items: list[Any], none_if_absent: bool = False) -> Any | None:
    """Access a nested object in root by item sequence."""
    if root is None:
        return None
    try:
        for k in items:
            root = root[k]
    except (KeyError, IndexError) as e:
        if none_if_absent:
            return None
        raise type(e)(f"Unable to find '{k}' using path {items!r} on {root!r}, exception: {e}")
    return root


def find_object_by_key(
    object_list: JsonList, key: str, nested: str | None = None, is_key: bool = False
) -> JsonDict | None:
    for item in object_list:
        if nested:
            item = item[nested]
        if key in item:
            return item[key] if is_key else item

    return None


def find_objects_by_key(object_list: JsonList, key: str, nested: str | None = None) -> JsonList:
    objects = []
    for item in object_list:
        if nested:
            item = item[nested]
        if key in item:
            objects.append(item)
    return objects
```

parsers/browing.py
```py
def parse_mixed_content(
    rows: JsonList,
) -> JsonList:
    items = []
    for row in rows:
        if DESCRIPTION_SHELF[0] in row:
            results = nav(row, DESCRIPTION_SHELF)
            title = nav(results, ["header", *RUN_TEXT])
            contents = nav(results, DESCRIPTION)
        else:
            results = next(iter(row.values()))
            if "contents" not in results:
                continue
            title = nav(results, [*CAROUSEL_TITLE, "text"])
            contents = []
            for result in results["contents"]:
                data = nav(result, [MTRIR], True)
                content = None
                if data:
                    page_type = nav(data, TITLE + NAVIGATION_BROWSE + PAGE_TYPE, True)
                    if page_type is None:  # song or watch_playlist
                        if nav(data, NAVIGATION_WATCH_PLAYLIST_ID, True) is not None:
                            content = parse_watch_playlist(data)
                        else:
                            content = parse_song(data)
                    elif page_type in ["MUSIC_PAGE_TYPE_ALBUM", "MUSIC_PAGE_TYPE_AUDIOBOOK"]:
                        content = parse_album(data)
                    elif page_type in ["MUSIC_PAGE_TYPE_ARTIST", "MUSIC_PAGE_TYPE_USER_CHANNEL"]:
                        content = parse_related_artist(data)
                    elif page_type == "MUSIC_PAGE_TYPE_PLAYLIST":
                        content = parse_playlist(data)
                    elif page_type == "MUSIC_PAGE_TYPE_PODCAST_SHOW_DETAIL_PAGE":
                        content = parse_podcast(data)
                elif data := nav(result, [MRLIR], True):
                    content = parse_song_flat(data)
                elif data := nav(result, [MMRIR], True):
                    content = parse_episode(data)
                else:
                    continue

                contents.append(content)

        items.append({"title": title, "contents": contents})
    return items


def parse_content_list(results: JsonList, parse_func: ParseFuncDictType, key: str = MTRIR) -> JsonList:
    contents = []
    for result in results:
        contents.append(parse_func(result[key]))

    return contents


def parse_album(result: JsonDict) -> JsonDict:
    album = {
        "title": nav(result, TITLE_TEXT),
        "type": nav(result, SUBTITLE),
        "artists": [parse_id_name(x) for x in nav(result, ["subtitle", "runs"]) if "navigationEndpoint" in x],
        "browseId": nav(result, TITLE + NAVIGATION_BROWSE_ID),
        "audioPlaylistId": parse_album_playlistid_if_exists(nav(result, THUMBNAIL_OVERLAY_NAVIGATION, True)),
        "thumbnails": nav(result, THUMBNAIL_RENDERER),
        "isExplicit": nav(result, SUBTITLE_BADGE_LABEL, True) is not None,
    }

    if (year := nav(result, SUBTITLE2, True)) and year.isnumeric():
        album["year"] = year

    return album


def parse_single(result: JsonDict) -> JsonDict:
    return {
        "title": nav(result, TITLE_TEXT),
        "year": nav(result, SUBTITLE, True),
        "browseId": nav(result, TITLE + NAVIGATION_BROWSE_ID),
        "thumbnails": nav(result, THUMBNAIL_RENDERER),
    }


def parse_song(result: JsonDict) -> JsonDict:
    song = {
        "title": nav(result, TITLE_TEXT),
        "videoId": nav(result, NAVIGATION_VIDEO_ID),
        "playlistId": nav(result, NAVIGATION_PLAYLIST_ID, True),
        "thumbnails": nav(result, THUMBNAIL_RENDERER),
    }
    song.update(parse_song_runs(nav(result, SUBTITLE_RUNS), skip_type_spec=True))
    return song


def parse_song_flat(data: JsonDict, with_playlist_id: bool = False) -> JsonDict:
    columns = [get_flex_column_item(data, i) for i in range(0, len(data["flexColumns"]))]
    song = {
        "title": nav(columns[0], TEXT_RUN_TEXT),
        "videoId": nav(columns[0], TEXT_RUN + NAVIGATION_VIDEO_ID, True),
        "videoType": nav(data, [*PLAY_BUTTON, "playNavigationEndpoint", *NAVIGATION_VIDEO_TYPE], True),
        "thumbnails": nav(data, THUMBNAILS),
        "isExplicit": nav(data, BADGE_LABEL, True) is not None,
    }

    if with_playlist_id:
        song["playlistId"] = nav(data, [*PLAY_BUTTON, "playNavigationEndpoint", *WATCH_PLAYLIST_ID])

    runs = nav(columns[1], TEXT_RUNS)
    song.update(parse_song_runs(runs, skip_type_spec=True))

    if len(columns) > 2 and columns[2] is not None and "navigationEndpoint" in nav(columns[2], TEXT_RUN):
        song["album"] = {
            "name": nav(columns[2], TEXT_RUN_TEXT),
            "id": nav(columns[2], TEXT_RUN + NAVIGATION_BROWSE_ID),
        }

    return song


def parse_video(result: JsonDict) -> JsonDict:
    runs = nav(result, SUBTITLE_RUNS)
    artists_len = get_dot_separator_index(runs)
    videoId = nav(result, NAVIGATION_VIDEO_ID, True)
    if not videoId:
        videoId = next(
            video_id
            for entry in nav(result, MENU_ITEMS)
            if (video_id := nav(entry, MENU_SERVICE + QUEUE_VIDEO_ID, True))
        )
    return {
        "title": nav(result, TITLE_TEXT),
        "videoId": videoId,
        "artists": parse_artists_runs(runs[:artists_len]),
        "playlistId": nav(result, NAVIGATION_PLAYLIST_ID, True),
        "thumbnails": nav(result, THUMBNAIL_RENDERER, True),
        "views": runs[-1]["text"].split(" ")[0],
    }


def parse_playlist(data: JsonDict) -> JsonDict:
    playlist = {
        "title": nav(
            data,
            TITLE_TEXT,
            none_if_absent=True,  # rare but possible for playlist title to be missing
        ),
        "playlistId": nav(data, TITLE + NAVIGATION_BROWSE_ID)[2:],
        "thumbnails": nav(data, THUMBNAIL_RENDERER),
    }
    subtitle = data["subtitle"]
    if "runs" in subtitle:
        playlist["description"] = "".join([run["text"] for run in subtitle["runs"]])
        if len(subtitle["runs"]) == 3 and re.search(r"\d+ ", nav(data, SUBTITLE2)):
            playlist["count"] = nav(data, SUBTITLE2).split(" ")[0]
            playlist["author"] = parse_artists_runs(subtitle["runs"][:1])

    return playlist


def parse_related_artist(data: JsonDict) -> JsonDict:
    subscribers = nav(data, SUBTITLE, True)
    if subscribers:
        subscribers = subscribers.split(" ")[0]
    return {
        "title": nav(data, TITLE_TEXT),
        "browseId": nav(data, TITLE + NAVIGATION_BROWSE_ID),
        "subscribers": subscribers,
        "thumbnails": nav(data, THUMBNAIL_RENDERER),
    }


def parse_watch_playlist(data: JsonDict) -> JsonDict:
    return {
        "title": nav(data, TITLE_TEXT),
        "playlistId": nav(data, NAVIGATION_WATCH_PLAYLIST_ID),
        "thumbnails": nav(data, THUMBNAIL_RENDERER),
    }
```

parsers/albums.py
```py
def parse_album_header(response: JsonDict) -> JsonDict:
    header = nav(response, HEADER_DETAIL)
    album = {
        "title": nav(header, TITLE_TEXT),
        "type": nav(header, SUBTITLE),
        "thumbnails": nav(header, THUMBNAIL_CROPPED),
        "isExplicit": nav(header, SUBTITLE_BADGE_LABEL, True) is not None,
    }

    if "description" in header:
        album["description"] = header["description"]["runs"][0]["text"]

    album_info = parse_song_runs(header["subtitle"]["runs"][2:])
    album.update(album_info)

    if len(header["secondSubtitle"]["runs"]) > 1:
        album["trackCount"] = to_int(header["secondSubtitle"]["runs"][0]["text"])
        album["duration"] = header["secondSubtitle"]["runs"][2]["text"]
    else:
        album["duration"] = header["secondSubtitle"]["runs"][0]["text"]

    # add to library/uploaded
    menu = nav(header, MENU)
    toplevel = menu["topLevelButtons"]
    album["audioPlaylistId"] = nav(toplevel, [0, "buttonRenderer", *NAVIGATION_WATCH_PLAYLIST_ID], True)
    if not album["audioPlaylistId"]:
        album["audioPlaylistId"] = nav(toplevel, [0, "buttonRenderer", *NAVIGATION_PLAYLIST_ID], True)
    service = nav(toplevel, [1, "buttonRenderer", "defaultServiceEndpoint"], True)
    if service:
        album["likeStatus"] = parse_like_status(service)

    return album


def parse_album_header_2024(response: JsonDict) -> JsonDict:
    header = nav(response, [*TWO_COLUMN_RENDERER, *TAB_CONTENT, *SECTION_LIST_ITEM, *RESPONSIVE_HEADER])
    album = {
        "title": nav(header, TITLE_TEXT),
        "type": nav(header, SUBTITLE),
        "thumbnails": nav(header, THUMBNAILS),
        "isExplicit": nav(header, SUBTITLE_BADGE_LABEL, True) is not None,
    }

    album["description"] = nav(header, ["description", *DESCRIPTION_SHELF, *DESCRIPTION], True)

    album_info = parse_song_runs(header["subtitle"]["runs"][2:])
    strapline_runs = nav(header, ["straplineTextOne", "runs"], True)
    album_info["artists"] = parse_artists_runs(strapline_runs) if strapline_runs else None
    album.update(album_info)

    if len(header["secondSubtitle"]["runs"]) > 1:
        album["trackCount"] = to_int(header["secondSubtitle"]["runs"][0]["text"])
        album["duration"] = header["secondSubtitle"]["runs"][2]["text"]
    else:
        album["duration"] = header["secondSubtitle"]["runs"][0]["text"]

    # add to library/uploaded
    buttons = header["buttons"]
    album["audioPlaylistId"] = nav(
        find_object_by_key(buttons, "musicPlayButtonRenderer"),
        ["musicPlayButtonRenderer", "playNavigationEndpoint", *WATCH_PID],
        True,
    )
    # remove this once A/B testing is finished and it is no longer covered
    if album["audioPlaylistId"] is None:
        album["audioPlaylistId"] = nav(
            find_object_by_key(buttons, "musicPlayButtonRenderer"),
            ["musicPlayButtonRenderer", "playNavigationEndpoint", *WATCH_PLAYLIST_ID],
            True,
        )
    service = nav(
        find_object_by_key(buttons, "toggleButtonRenderer"),
        ["toggleButtonRenderer", "defaultServiceEndpoint"],
        True,
    )
    album["likeStatus"] = "INDIFFERENT"
    if service:
        album["likeStatus"] = parse_like_status(service)

    return album


def parse_album_playlistid_if_exists(data: JsonDict | None) -> str | None:
    """the content of the data changes based on whether the user is authenticated or not"""
    return nav(data, WATCH_PID, True) or nav(data, WATCH_PLAYLIST_ID, True) if data else None
```

parsers/artists.py
```py
from ytmusicapi.navigation import *
from ytmusicapi.type_alias import JsonList


def parse_artists_runs(runs: JsonList) -> JsonList:
    """Returns artist names and IDs. Skips every other run to avoid separators."""
    artists = []
    for j in range(int(len(runs) / 2) + 1):
        artists.append({"name": runs[j * 2]["text"], "id": nav(runs[j * 2], NAVIGATION_BROWSE_ID, True)})
    return artists
```

parsers/podcasts.py
```py
PROGRESS_RENDERER = ["musicPlaybackProgressRenderer"]
DURATION_TEXT = ["durationText", "runs", 1, "text"]


@dataclass
class DescriptionElement:
    text: str

    def __str__(self) -> str:
        return self.text


@dataclass
class Link(DescriptionElement):
    url: str


@dataclass
class Timestamp(DescriptionElement):
    seconds: int


@dataclass
class Description(list[DescriptionElement]):
    def __init__(self, *args: Any, **kwargs: Any):
        super().__init__(args[0])

    @property
    def text(self) -> str:
        return "".join(str(element) for element in self)

    @classmethod
    def from_runs(cls, description_runs: JsonList) -> "Description":
        elements: list[DescriptionElement] = []
        for run in description_runs:
            navigationEndpoint = nav(run, ["navigationEndpoint"], True)
            if navigationEndpoint:
                element = DescriptionElement("")
                if "urlEndpoint" in navigationEndpoint:
                    element = Link(text=run["text"], url=navigationEndpoint["urlEndpoint"]["url"])
                elif "watchEndpoint" in navigationEndpoint:
                    element = Timestamp(
                        text=run["text"],
                        seconds=nav(navigationEndpoint, ["watchEndpoint", "startTimeSeconds"]),
                    )
            else:
                element = DescriptionElement(text=nav(run, ["text"], True))  # type: ignore

            elements.append(element)

        return cls(elements)


def parse_base_header(header: JsonDict) -> JsonDict:
    strapline = nav(header, ["straplineTextOne"])

    author = {
        "name": nav(strapline, [*RUN_TEXT], True),
        "id": nav(strapline, ["runs", 0, *NAVIGATION_BROWSE_ID], True),
    }

    return {
        "author": author if author["name"] else None,
        "title": nav(header, TITLE_TEXT),
        "thumbnails": nav(header, THUMBNAILS),
    }


def parse_podcast_header(header: JsonDict) -> JsonDict:
    metadata = parse_base_header(header)
    metadata["description"] = nav(header, ["description", *DESCRIPTION_SHELF, *DESCRIPTION], True)
    metadata["saved"] = nav(header, ["buttons", 1, *TOGGLED_BUTTON])

    return metadata


def parse_episode_header(header: JsonDict) -> JsonDict:
    metadata = parse_base_header(header)
    metadata["date"] = nav(header, [*SUBTITLE])
    progress_renderer = nav(header, ["progress", *PROGRESS_RENDERER])
    metadata["duration"] = nav(progress_renderer, DURATION_TEXT, True)
    metadata["progressPercentage"] = nav(progress_renderer, ["playbackProgressPercentage"])
    metadata["saved"] = nav(header, ["buttons", 0, *TOGGLED_BUTTON], True) or False

    metadata["playlistId"] = None
    menu_buttons = nav(header, ["buttons", -1, "menuRenderer", "items"])
    for button in menu_buttons:
        if nav(button, [MNIR, *ICON_TYPE], True) == "BROADCAST":
            metadata["playlistId"] = nav(button, [MNIR, *NAVIGATION_BROWSE_ID])

    return metadata


def parse_episode(data: JsonDict) -> JsonDict:
    """Parses a single episode under "Episodes" on a channel page or on a podcast page"""
    thumbnails = nav(data, THUMBNAILS)
    date = nav(data, SUBTITLE, True)
    duration = nav(data, ["playbackProgress", *PROGRESS_RENDERER, *DURATION_TEXT], True)
    title = nav(data, TITLE_TEXT)
    description = nav(data, DESCRIPTION, True)
    videoId = nav(data, ["onTap", *WATCH_VIDEO_ID], True)
    browseId = nav(data, [*TITLE, *NAVIGATION_BROWSE_ID], True)
    videoType = nav(data, ["onTap", *NAVIGATION_VIDEO_TYPE], True)
    index = nav(data, ["onTap", "watchEndpoint", "index"], True)
    return {
        "index": index,
        "title": title,
        "description": description,
        "duration": duration,
        "videoId": videoId,
        "browseId": browseId,
        "videoType": videoType,
        "date": date,
        "thumbnails": thumbnails,
    }


def parse_episode_flat(data: JsonDict) -> JsonDict:
    return {
        "title": nav(get_flex_column_item(data, 0), TEXT_RUN_TEXT),
        "podcast": parse_id_name(nav(get_flex_column_item(data, 1), TEXT_RUN)),
        "videoId": nav(data, ["playlistItemData", "videoId"]),
        "browseId": nav(get_flex_column_item(data, 0), [*TEXT_RUN, *NAVIGATION_BROWSE_ID]),
        "playlistId": nav(data, [*PLAY_BUTTON, "playNavigationEndpoint", *WATCH_PLAYLIST_ID]),
        "videoType": nav(data, [*PLAY_BUTTON, "playNavigationEndpoint", *NAVIGATION_VIDEO_TYPE]),
        "date": nav(get_flex_column_item(data, 2), TEXT_RUN_TEXT),
        "thumbnails": nav(data, THUMBNAILS),
    }


def parse_podcast(data: JsonDict) -> JsonDict:
    """Parses a single podcast under "Podcasts" on a channel page"""
    return {
        "title": nav(data, TITLE_TEXT),
        "channel": parse_id_name(nav(data, [*SUBTITLE_RUNS, 0], True)),
        "browseId": nav(data, TITLE + NAVIGATION_BROWSE_ID),
        "podcastId": nav(data, THUMBNAIL_OVERLAY, True),
        "thumbnails": nav(data, THUMBNAIL_RENDERER),
    }
```

parsers/songs.py
```py
def parse_song_artists(data: JsonDict, index: int) -> JsonList:
    flex_item = get_flex_column_item(data, index)
    if not flex_item:
        return []
    else:
        runs = flex_item["text"]["runs"]
        return parse_artists_runs(runs)


def parse_song_run(run: JsonDict) -> JsonDict:
    text = run["text"]

    if "navigationEndpoint" in run:  # artist or album
        item = {"name": text, "id": nav(run, NAVIGATION_BROWSE_ID, True)}

        if item["id"] and (item["id"].startswith("MPRE") or "release_detail" in item["id"]):  # album
            return {"type": "album", "data": item}
        else:  # artist
            return {"type": "artist", "data": item}
    else:
        # note: YT uses non-breaking space \xa0 to separate number and magnitude
        if re.match(r"^\d([^ ])* [^ ]*$", text):
            return {"type": "views", "data": text.split(" ")[0]}

        elif re.match(r"^(\d+:)*\d+:\d+$", text):
            return {"type": "duration", "data": text}

        elif re.match(r"^\d{4}$", text):
            return {"type": "year", "data": text}

        else:  # artist without id
            return {"type": "artist", "data": {"name": text, "id": None}}


def parse_song_runs(runs: JsonList, skip_type_spec: bool = False) -> JsonDict:
    """
    :param skip_type_spec: if true, skip the type specifier (like "Song", "Single", or "Album") that may appear before artists ("Song • Eminem"). Otherwise, that text item is parsed as an artist with no ID.
    """

    parsed: JsonDict = {}

    # prevent type specifier from being parsed as an artist
    # it's the first run, separated from the actual artists by " • "
    if (
        skip_type_spec
        and len(runs) > 2
        and parse_song_run(runs[0])["type"] == "artist"
        and runs[1] == DOT_SEPARATOR_RUN
        and parse_song_run(runs[2])["type"] == "artist"
    ):
        runs = runs[2:]

    for i, run in list(enumerate(runs)):
        if i % 2:  # uneven items are always separators
            continue

        parsed_run = parse_song_run(run)
        data = parsed_run["data"]
        match parsed_run["type"]:
            case "album":
                parsed["album"] = data
            case "artist":
                parsed["artists"] = parsed.get("artists", [])
                parsed["artists"].append(data)
            case "views":
                parsed["views"] = data
            case "duration":
                parsed["duration"] = data
                parsed["duration_seconds"] = parse_duration(data)
            case "year":
                parsed["year"] = data

    return parsed


def parse_song_album(data: JsonDict, index: int) -> JsonDict | None:
    flex_item = get_flex_column_item(data, index)
    browse_id = nav(flex_item, TEXT_RUN + NAVIGATION_BROWSE_ID, True)
    return None if not flex_item else {"name": get_item_text(data, index), "id": browse_id}


def parse_song_menu_data(data: JsonDict) -> JsonDict:
    """
    :return: Dictionary with data from the provided song's context menu.
    """

    if "menu" not in data:
        return {}

    song_data: JsonDict = {}
    for item in nav(data, MENU_ITEMS):
        menu_item = nav(item, [TOGGLE_MENU], True) or nav(item, ["menuServiceItemRenderer"], True)
        if menu_item is None:
            continue

        song_data["inLibrary"] = song_data.get("inLibrary", False)
        song_data["pinnedToListenAgain"] = song_data.get("pinnedToListenAgain", False)

        current_icon_type = nav(menu_item, ["defaultIcon", "iconType"], True) or nav(
            menu_item, ["icon", "iconType"], True
        )
        feedback_token: Callable[[str], str | None] = lambda endpoint_type: nav(
            menu_item, [endpoint_type, *FEEDBACK_TOKEN], True
        )

        match current_icon_type:
            case "KEEP":  # pin to listen again
                song_data["listenAgainFeedbackTokens"] = {
                    "pin": feedback_token("defaultServiceEndpoint"),
                    "unpin": feedback_token("toggledServiceEndpoint"),
                }
            case "KEEP_OFF":  # unpin from listen again
                song_data["pinnedToListenAgain"] = True
                song_data["listenAgainFeedbackTokens"] = {
                    "pin": feedback_token("toggledServiceEndpoint"),
                    "unpin": feedback_token("defaultServiceEndpoint"),
                }
            case "BOOKMARK_BORDER":  # add to library
                song_data["feedbackTokens"] = {
                    "add": feedback_token("defaultServiceEndpoint"),
                    "remove": feedback_token("toggledServiceEndpoint"),
                }
            case "BOOKMARK":  # remove from library
                song_data["inLibrary"] = True
                song_data["feedbackTokens"] = {
                    "add": feedback_token("toggledServiceEndpoint"),
                    "remove": feedback_token("defaultServiceEndpoint"),
                }
            case "REMOVE_FROM_HISTORY":
                song_data["feedbackToken"] = feedback_token("serviceEndpoint")

    return song_data


def parse_like_status(service: JsonDict) -> str:
    status = ["LIKE", "INDIFFERENT"]
    return status[status.index(service["likeEndpoint"]["status"]) - 1]
```
