use clap::{App, Arg};
use indicatif::{ProgressBar, ProgressStyle};
use m3u8_rs::parse_playlist;
use m3u8_rs::playlist::Playlist;
use mktemp::Temp;
use rayon::prelude::*;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::fs::write;
use std::io::Write;
use std::process::{Command, Stdio};
use url::Url;
// use url::Url;

#[derive(Serialize, Deserialize)]
pub struct ScriptRequest {
    vod: String,
    ima: String,
    product: i64,
    channel: String,
    #[serde(rename = "vodThumb")]
    vod_thumb: String,
}

fn main() {
    let app = App::new("Picarto Stream Downloader")
        .version("1.0")
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
        .get_matches();

    let url = app.value_of("input").unwrap();
    let output_file = app.value_of("output").unwrap();

    println!("Grabbing {}", url);

    // this is checking if the url is a direct video popout of the vod.
    if url.contains("videopopout") {
        let client = Client::new();
        let response = client.get(url).send().expect("Url is invalid!");
        let text = response.text().unwrap();
        let html = Html::parse_document(&text);

        // now after loading the html and parsing it, we can look into finding the hls_m3u8.
        let body = html
            .select(&Selector::parse("body").unwrap())
            .next()
            .unwrap();
        let script = body
            .select(&Selector::parse("script").unwrap())
            .next()
            .unwrap();
        // This grabs and trims the code inside the script tag, leaving only the json that was passed.
        let script_text = script
            .inner_html()
            .trim()
            .trim_end_matches(')')
            .replace("riot.mount(\"#vod-player\", ", "")
            .replace("\\/", "/");
        println!("{}", script_text);
        let script_request = from_str::<ScriptRequest>(&script_text).unwrap();

        println!("{}", script_request.vod);
        // After preparing the important bits, the next request can be sent for the m3u8.
        let m3u8_request = client
            .get(&script_request.vod)
            .send()
            .expect("Could not grab m3u8!")
            .bytes()
            .unwrap();

        let m3u8 = parse_playlist(m3u8_request.as_ref());
        let m3u8_second_uri = match m3u8 {
            Ok((_, Playlist::MasterPlaylist(pl))) => {
                let variant = pl.variants.first().unwrap();
                variant.uri.clone()
            }
            Ok((_, Playlist::MediaPlaylist(_))) => {
                panic!("This shouldn't be a media type!")
            }
            Err(_) => {
                panic!("Unable to parse m3u8!");
            }
        };

        let temp_dir = Temp::new_dir_in("./").unwrap();
        std::fs::create_dir_all(&temp_dir).unwrap();
        let mut input_url = Url::parse(&script_request.vod).unwrap();
        println!("{}", input_url);
        input_url.path_segments_mut().unwrap().pop();
        input_url.path_segments_mut().unwrap().push("/");
        println!("{}", input_url);
        let m3u8_second_request = input_url.join(&m3u8_second_uri).unwrap();
        println!("{}", m3u8_second_uri);
        println!("{}", m3u8_second_request);
        let second_m3u8_request = client
            .get(m3u8_second_request.as_str())
            .send()
            .expect("Could not grab m3u8!")
            .bytes()
            .unwrap();
        let media_m3u8 = parse_playlist(second_m3u8_request.as_ref());
        match media_m3u8 {
            Ok((_, Playlist::MasterPlaylist(_))) => {}
            Ok((_, Playlist::MediaPlaylist(pl))) => {
                let sty = ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                    .progress_chars("->=");
                let progress = ProgressBar::new(pl.segments.len() as u64);
                progress.set_style(sty);
                pl.segments
                    .par_iter()
                    .enumerate()
                    .for_each(|(filename, segment)| {
                        let mut request = m3u8_second_request.clone();
                        request.set_query(None);
                        request.path_segments_mut().unwrap().pop();
                        request.path_segments_mut().unwrap().push(&segment.uri);

                        let file = client
                            .get(request.as_str())
                            .send()
                            .expect("Could not download file!")
                            .bytes()
                            .unwrap();
                        write(
                            format!("{}/{}.ts", temp_dir.to_str().unwrap(), filename),
                            file.as_ref(),
                        )
                        .unwrap();

                        progress.inc(1);
                    });

                progress.finish();
            }
            Err(_) => panic!("Parsing error!"),
        }

        let temp_merge_file = Temp::new_file_in("./").unwrap();
        let mut merge_gen = Command::new("PowerShell")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut merge_in = merge_gen.stdin.take().unwrap();
        merge_in
            .write_all(
                format!(
                    "$text = foreach ($i in Get-ChildItem ./{}/*.ts) {{ echo \"file \'$i\'\" }}\r\n",
                    temp_dir.to_str().unwrap()
                )
                .as_bytes(),
            )
            .unwrap();
        merge_in
            .write_all("$utf8 = New-Object System.Text.UTF8Encoding $False\r\n".as_bytes())
            .unwrap();
        merge_in
            .write_all(
                format!(
                    "[System.IO.File]::WriteAllLines(\"{}\", $text, $utf8)\r\n",
                    temp_merge_file.to_str().unwrap()
                )
                .as_bytes(),
            )
            .unwrap();
        merge_in.write_all("exit\r\n".as_bytes()).unwrap();
        merge_gen.wait_with_output().unwrap();

        let ffmpeg = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-hide_banner",
                "-f",
                "concat",
                "-safe",
                "0",
                "-i",
                temp_merge_file.to_str().unwrap(),
                "-c:v",
                "ffv1",
                "-level",
                "3",
                "-context",
                "1",
                "-c:a",
                "pcm_s16le",
                &format!("{}.avi", output_file),
            ])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        ffmpeg.wait_with_output().unwrap();
    }
}
