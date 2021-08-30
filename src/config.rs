use ron::from_str;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use structopt::StructOpt;

///Flags for Lonely Tribes
#[derive(StructOpt, Debug)]
pub struct Flags {
    ///Enable an FPS counter in the console
    #[structopt(short, long)]
    pub fps: bool,

    ///Enable the console
    #[structopt(short, long)]
    pub console: bool,

    ///Enable debug options (disables high scores)
    ///Similar to Valve svcheats
    #[structopt(short, long)]
    pub debug: bool,

    ///Starting level, requires debug mode
    #[structopt(short, long)]
    pub level: Option<usize>,

    ///Option to enable legacy movement
    #[structopt(short, long)]
    pub timed_movement: Option<f32>,

    ///Option to use the debug level, requires debug mode
    #[cfg(debug_assertions)]
    #[structopt(long)]
    pub debug_level: bool,

    #[cfg(not(debug_assertions))]
    #[structopt(skip = false)]
    pub debug_level: bool,
}

pub struct LTConfig {
    pub flags: Flags,
    pub conf: ParsedConfig,
}
#[derive(Serialize, Deserialize)]
struct ReadInConfig {
    pub screen_dimensions: Option<(u32, u32)>,
    pub maximised: bool,
}
pub struct ParsedConfig {
    pub screen_dimensions: (u32, u32),
    pub maximised: bool,
}

impl ReadInConfig {
    fn new() -> ParsedConfig {
        let path = "config/conf.ron".to_string();
        let contents = read_to_string(path.clone()).unwrap_or_default();
        match from_str(contents.as_str()) {
            Ok(w) => {
                let w: ReadInConfig = w;
                let sd = w.screen_dimensions.unwrap_or_else(|| {
                    //TODO: grab screen res
                    (1920, 1080)
                });
                ParsedConfig {
                    screen_dimensions: sd,
                    maximised: w.maximised,
                }
            }
            Err(e) => {
                log::warn!(
                    "Unable to parse conf: {}, contents: {}, path: {}",
                    e,
                    contents,
                    path
                );
                ParsedConfig::default()
            }
        }
    }
}
impl Default for ParsedConfig {
    fn default() -> Self {
        ParsedConfig {
            screen_dimensions: (1600, 900),
            maximised: false,
        }
    }
}
impl ParsedConfig {
    pub fn new() -> Self {
        ReadInConfig::new()
    }
}
impl LTConfig {
    pub fn new() -> Self {
        Self {
            flags: Flags::from_args(),
            conf: ParsedConfig::new(),
        }
    }
}
