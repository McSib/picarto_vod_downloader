#[macro_use]
extern crate log;

use crate::program::Program;
use simplelog::{
    CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;

mod program;

fn main() {
    let mut config = ConfigBuilder::new();
    config.add_filter_allow_str("picarto_vod_downloader");
    CombinedLogger::init(vec![
        TermLogger::new(
            if cfg!(debug_assertions) {
                LevelFilter::max()
            } else {
                LevelFilter::Info
            },
            config.build(),
            TerminalMode::Mixed,
        ),
        WriteLogger::new(
            LevelFilter::max(),
            config.build(),
            File::create("picarto_vod_downloader.log").unwrap(),
        ),
    ])
    .unwrap();

    Program::start()
}
