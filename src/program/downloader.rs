use indicatif::{ProgressBar, ProgressStyle};
use m3u8_rs::playlist::MediaSegment;
use rayon::prelude::*;
use reqwest::blocking::Client;
use std::fs::write;
use url::Url;

/// The segment downloader downloads all the segments of the live stream, numbering the files and
/// ensuring that they are ready for the encoder when it goes to combine all of them into one file.
pub struct SegmentDownloader {
    url: Url,
}

impl SegmentDownloader {
    pub fn new(url: Url) -> Self {
        SegmentDownloader { url }
    }

    pub fn download_segments(&mut self, dir: &str, segments: Vec<MediaSegment>) {
        // I have to make another client so that way the threads can move it freely without restraint.
        let client = Client::new();
        let sty = ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .progress_chars("->=");
        let progress = ProgressBar::new(segments.len() as u64);
        progress.set_style(sty);

        segments
            .par_iter()
            .enumerate()
            .for_each(|(filename, segment)| {
                let mut url = self.url.clone();
                url.set_query(None);
                url.path_segments_mut().unwrap().push(&segment.uri);

                trace!("Segment url chunk: {}", url);

                let file = client
                    .get(url.as_str())
                    .send()
                    .expect("Unable to process request!")
                    .bytes()
                    .unwrap();
                debug!("Writing file to \"{}/{}.ts\"", dir, filename);
                write(format!("{}/{:0>10}.ts", dir, filename), file.as_ref()).unwrap();

                url.path_segments_mut().unwrap().pop();
                debug!("Popped url segment chunk: {}", url);
                progress.inc(1);
            });

        progress.finish_with_message("All segments downloaded...");
    }
}
