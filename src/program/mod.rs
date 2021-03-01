use crate::program::downloader::SegmentDownloader;
use crate::program::encoder::Encoder;
use crate::program::m3u8_grabber::{M3U8Grabber, Scrubber};
use clap::{crate_version, App, Arg, ArgMatches};
use m3u8_grabber::Sender;
use mktemp::Temp;
use std::fs::create_dir_all;

mod data;
mod downloader;
mod encoder;
mod m3u8_grabber;

pub struct Program {
    sender: Sender,
}

impl Program {
    fn new() -> Self {
        Program {
            sender: Sender::new(),
        }
    }

    pub fn start() {
        // Start the program and get it initialized.
        let program = Program::new();
        let args = program.match_commands();

        // Get the input url and scrub the data for the m3u8 url;
        info!("Scrubbing url...");
        let url = args.value_of("input").unwrap();
        let scrubber = Scrubber::new(program.sender.clone());
        let script_request = scrubber.scrub(url);

        // Once the url data is scrubbed, we can pass it to grabber for the segments.
        info!("Grabbing m3u8 playlists...");
        let grabber = M3U8Grabber::new(program.sender.clone());
        let (url, segments) = grabber.grab_segments(script_request);

        // Now that the segments are grabbed, the downloader can now be used to download all the segments
        // for the encoder to later combine. Something else that needs to be done here is to make a temp
        // directory for the downloader to place the segments in. This temp directory will be deleted
        // at the end of the runtime with all the segments included.
        info!("Downloading segments...");
        let temp_dir = Temp::new_dir_in("./").unwrap();
        create_dir_all(&temp_dir).unwrap();
        let mut downloader = SegmentDownloader::new(url);
        downloader.download_segments(temp_dir.to_str().unwrap(), segments);

        // With everything set up, the encoder can now step in and encode all of the videos together.
        info!("Encoding video...");
        let encoder = Encoder::new(temp_dir.to_str().unwrap());
        encoder.generate_merge_list();
        encoder.encode_video(args.value_of("output").unwrap());
    }

    fn match_commands(&self) -> ArgMatches {
        App::new("Picarto Stream Downloader")
            .version(crate_version!())
            .author("McSib")
            .about("A simple downloader for Picarto vods.")
            .arg(
                Arg::with_name("input")
                    .long("input")
                    .short("i")
                    .value_name("URL")
                    .help("The input url.")
                    .required(true)
                    .takes_value(true)
                    .max_values(1),
            )
            .arg(
                Arg::with_name("output")
                    .long("output")
                    .short("o")
                    .value_name("FILE")
                    .help("What the output file should be called.")
                    .required(true)
                    .takes_value(true)
                    .max_values(1),
            )
            .get_matches()
    }
}
