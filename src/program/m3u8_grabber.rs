use crate::program::data::ScriptRequest;
use m3u8_rs::parse_playlist;
use m3u8_rs::playlist::{MediaSegment, Playlist};
use reqwest::blocking::{Client, Response};
use scraper::{Html, Selector};
use serde_json::from_str;
use std::rc::Rc;
use url::Url;

/// This class should only be initialized once and then cloned after from there on out.
/// Creating another copy of this class introduces unneeded memory allocation and an entirely new client
/// for no valid reasons.
#[derive(Clone)]
pub struct Sender {
    client: Rc<Client>, // This is Rc encase I need to clone it at all in the future.
}

impl Sender {
    pub fn new() -> Self {
        Sender {
            client: Rc::new(Client::new()),
        }
    }

    pub fn get(&self, url: &str) -> Response {
        debug!("Sending url: {}", url);
        self.client
            .get(url)
            .send()
            .expect("Sent call through client failed!")
    }
}

/// This class combs through the html of the input url and finds the m3u8 link, which it will then
/// format and return for the m3u8 grabber to use.
pub struct Scrubber {
    sender: Sender,
}

impl Scrubber {
    pub fn new(sender: Sender) -> Self {
        Scrubber { sender }
    }

    /// Scrubs through the html and returns the url for the m3u8 file.
    pub fn scrub(&self, url: &str) -> ScriptRequest {
        // Have to check if this url is actually a video popout.
        if url.contains("videopopout") {
            let text = self.sender.get(url).text().unwrap();
            let html = Html::parse_document(&text);

            // This is grabbing the div that's holding the `<script>` I need to get the link.
            let div = html
                .select(&Selector::parse("div#player_holder").unwrap())
                .next()
                .unwrap();

            // The last child node should always be the `<script>` because of how the html is generated.
            // However, this may cause errors in the future if `Picarto` changes anything.
            debug!("Located the script tag...");
            let script = div
                .select(&Selector::parse("script").unwrap())
                .next()
                .unwrap();

            // We now need to trim and remove unneeded text within the inner html, making sure to contain
            // only the json. The function call in the script only has two parameters and what we want is the
            // last. This actually makes it a lot easier.
            let mut inner = script.inner_html().trim().to_string();

            // This is removing javascript that isn't needed for what we want.
            // The code that gets removed is `riot.mount(\"#vod-player\", `.
            debug!("Removing code section `riot.mount(\"#vod-player\", `...");
            for _ in 0..25 {
                inner.remove(0);
            }

            // After removing the unneeded code, we can remove the last character at the end of the string.
            // This character is the closing `)` of the method call.
            debug!("Removing code section `)` from end of string....");
            inner.remove(inner.len() - 1);

            // Now that all of the trimming is down, we have to do one more thing. Because of how the code
            // needed to be formatted, any `/` was converted to `\\/`. Because of this, we need to revert
            // this change by replacing any `\\/` with just `/` for our future request.
            debug!("Replacing `\\/` with `/`...");
            let script_request = inner.replace("\\/", "/");

            // With all of the trims and replacing complete, we can now send out this data by deserializing
            // it as if it was an average json file.
            debug!("{}", script_request);
            from_str(&script_request).unwrap()
        } else {
            error!("This url is not a video popout!");
            panic!("Unable to scub url for m3u8!");
        }
    }
}

/// This is the grabber used to grab all the needed m3u8 files, trying to locate the segments of the video.
pub struct M3U8Grabber {
    sender: Sender,
}

impl M3U8Grabber {
    pub fn new(sender: Sender) -> Self {
        M3U8Grabber { sender }
    }

    /// This is a function that handles most of the manual calls needed, finding the master playlist
    /// and finding the media playlist through the script request before returning the grabbed segments.
    pub fn grab_segments(&self, script_request: ScriptRequest) -> (Url, Vec<MediaSegment>) {
        let mut url = Url::parse(&script_request.vod).expect("Unable to parse url!");
        loop {
            let bytes = self.sender.get(url.as_str()).bytes().unwrap();
            let m3u8 = parse_playlist(bytes.as_ref());
            // debug!("Chosen url: {}", url);
            // trace!("Data: {:#?}", bytes.as_ref());

            match m3u8 {
                Ok((_, Playlist::MasterPlaylist(pl))) => {
                    // This is only grabbing the first variant since there is no quality setting for
                    // Picarto.
                    let variant = pl.variants.first().unwrap();
                    url.path_segments_mut().unwrap().pop().push("/");
                    url = url.join(&variant.uri).unwrap();

                    debug!("Variant url: {}", url);
                }
                Ok((_, Playlist::MediaPlaylist(pl))) => {
                    // Assuming that the second request is the media playlist, we can grab the segments
                    // and then return it with the url in a tuple.
                    url.path_segments_mut().unwrap().pop();
                    debug!("Media url: {}", url);
                    return (url, pl.segments);
                }
                Err(_) => {
                    panic!("Unable to parse m3u8!");
                }
            };
        }
    }
}
