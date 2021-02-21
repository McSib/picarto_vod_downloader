use mktemp::Temp;
use std::io::Write;
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
					self.dir
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
                    self.merge_file_name.to_str().unwrap()
                )
                .as_bytes(),
            )
            .unwrap();
        merge_in.write_all("exit\r\n".as_bytes()).unwrap();
        merge_gen.wait_with_output().unwrap();
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
