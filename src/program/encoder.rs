use mktemp::Temp;
use std::fs::{read_dir, write};
use std::path::Path;
use std::process::{Command, Stdio};

pub struct Encoder {
    dir: String,
    merge_file_name: Temp,
}

impl Encoder {
    pub fn new(dir: &str) -> Self {
        Encoder {
            dir: dir.to_string(),
            merge_file_name: Temp::new_file_in("./").unwrap(),
        }
    }

    pub fn generate_merge_list(&self) {
        fn get_files(dir: &Path) -> Vec<String> {
            let mut temp = Vec::new();
            if dir.is_dir() {
                for entry in read_dir(dir).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_file() {
                        temp.push(format!("file \'{}\'\r\n", path.as_path().to_str().unwrap()));
                    }
                }
            }
            temp
        }

        let mut merge_text = String::new();
        for lines in get_files(&Path::new(&self.dir)) {
            merge_text.push_str(&lines);
        }

        write(self.merge_file_name.as_path(), &merge_text).unwrap();
    }

    pub fn encode_video(&self, output_file: &str) {
        let ffmpeg = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-hide_banner",
                "-f",
                "concat",
                "-safe",
                "0",
                "-i",
                self.merge_file_name.to_str().unwrap(),
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
