use clap::Parser;

use std::path::PathBuf;

const AUTHOR: &str = "TheTechRobo";
const VERSION: &str = "v.1.2";
const ABOUT: &str = "Extracts URLs from Discord scrapes";

#[derive(Parser, Debug)]
#[clap(author=AUTHOR, version=VERSION, about=ABOUT)]
pub struct Args {
    /// Specifies server ID. Required by --parse-websockets
    #[clap(long="guild-id")]
    pub guild_id: Option<String>,

    /// Go through websockets to get server icon, server emojis, and more
    #[clap(long="parse-websockets")]
    pub use_websockets: bool,

    #[clap(value_name="FILE")]
    pub file: PathBuf,

    #[clap(value_name="TYPE", value_parser=["dht", "discard2", "plaintext", "dce"])]
    pub file_type: String
}
