use regex::Regex;
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader, Error};

use clap::Parser;

use tinyjson::JsonValue;

mod gateway;
mod cli;

#[derive(Debug)]
struct S {
    data: String, // i actually don't remember why I had to do it this way
}

#[derive(Debug)]
struct User {
    id: u64,
    avatar_id: Option<String>,
}

struct RetVal { // so we can return both urls and ignores from a function
    urls: Vec<String>,
    ignores: Vec<String>,
}

fn read_data() -> Vec<String> { // should really be named `read_ignores()'
    eprintln!("Reading ignores. Please wait...");
    let status = fs::read_to_string("ignores.url");
    let data = match status {
        Ok(s) => s,
        Err(_) => {
            eprintln!("failed to read ignores.url, proceeding with default");
            "".to_string()
        }
    };
    let urls: Vec<&str> = data.split('\n').collect();
    let mut ignores = Vec::new();
    for url in urls {
        if !url.is_empty() {
            ignores.push(url.to_string());
        }
    }
    ignores
}

fn write_data(ignores: Vec<String>, urls: Vec<String>) {
    eprintln!("Now writing data.");
    let file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("ignores.url");
    #[allow(unused_assignments)]
    let mut filefailed = false; // this brings up a warning but I'm not sure how to fix it
    let mut error: Error = Error::new(std::io::ErrorKind::Other, "bye");
    let mut file = match file {
        Ok(fil) => {
            filefailed = false;
            fil
        }
        Err(e) => {
            error = e;
            filefailed = true;
            fs::OpenOptions::new()
                .write(true)
                .open("/dev/null")
                .unwrap()
        }
    };
    if !filefailed {
        for ignore in ignores {
            writeln!(file, "{}", ignore).expect("failed to write file");
        }
    } else {
        eprintln!("Failed to write ignores: {}", error);
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open("urls.url")
        .expect("couldn't open urls.url");
    for url in urls {
        writeln!(file, "{}", url).expect("failed to write URLs");
    }
    //https://www.codegrepper.com/code-examples/rust/rust+how+to+append+to+a+file
}

fn dce(filename: &str, ignores: Vec<String>, mut urls: Vec<String>, regex: Regex, arguments: cli::Args) -> RetVal {
    if arguments.use_websockets {
        panic!("--parse-websockets can only be used with discard2 JSONL");
    }
    let jj: JsonValue = {
        // doing this as a block makes sure we drop this string afterwards
        // the compiler might optimise that but I'm not sure
        eprintln!("Reading JSON... This may take up an obscene amount of RAM.");
        let contents = fs::read_to_string(filename).expect("Couldnt read file");
        contents.parse().unwrap()
    };
    eprintln!("Now extracting URLs. This may take awhile!");
    let icon_url: String = jj["guild"]["iconUrl"].clone().try_into().unwrap();
    if !ignores.contains(&icon_url) {
        urls.push(icon_url);
    }
    let messages: Vec<_> = jj["messages"].clone().try_into().unwrap();
    for message in messages {
        let avatar_url: String = message["author"]["avatarUrl"].clone().try_into().unwrap();
        if !ignores.contains(&avatar_url) {
            urls.push(avatar_url.clone());
        }
        let content: String = message["content"].clone().try_into().unwrap();
        for mat in regex.find_iter(&content) {
            let i = mat.as_str();
            if !ignores.contains(&i.to_string()) {
                urls.push(i.to_string());
            }
        }
        let attachments: Vec<_> = message["attachments"].clone().try_into().unwrap();
        for attachment in attachments {
            let url: String = attachment["url"].clone().try_into().unwrap();
            if !ignores.contains(&url) {
                urls.push(url);
            }
        }
        let reactions: Vec<_> = message["reactions"].clone().try_into().unwrap();
        for reaction in reactions {
            let emoji: HashMap<_,_> = reaction["emoji"].clone().try_into().unwrap();
            let url: String = emoji["imageUrl"].clone().try_into().unwrap();
            if !ignores.contains(&url) {
                urls.push(url);
            }
        }
        for embed_url in get_embed_urls(message, regex.clone()) {
            if !ignores.contains(&embed_url) {
                urls.push(embed_url);
            }
        }
    }
    RetVal { urls: urls, ignores: ignores }
}

fn sql(filename: &str, ignores: Vec<String>, mut urls: Vec<String>, regex: Regex, arguments: cli::Args) -> RetVal {
    if arguments.use_websockets {
        panic!("--parse-websockets can only be used with discard2 JSONL");
    }
    eprintln!("Connecting to SQL database...");
    let conn = Connection::open(filename).unwrap();
    eprintln!("Attachments...");
    let mut stmt = conn.prepare("SELECT * FROM attachments").unwrap();
    let person_iter = stmt
        .query_map([], |row| {
            Ok(S {
                data: row.get(4).unwrap(), //attachment URL is on 5th column of each row.
            })
        })
        .unwrap();
    for attachment_url in person_iter {
        let att = attachment_url.unwrap().data;
        if ignores.contains(&att) {
            continue;
        }
        urls.push(att);
    }
    eprintln!("Avatars...");
    let mut stmt = conn.prepare("SELECT * FROM users").unwrap();
    let person_iter = stmt
        .query_map([], |row| {
            Ok(User {
                id: row.get(0).unwrap(), // The user ID
                avatar_id: row.get(2).unwrap(),
            })
        })
        .unwrap();
    for avatar_url in person_iter {
        let avatar_url = avatar_url.unwrap();
        if let Some(avatar_id) = avatar_url.avatar_id.clone() {
            let att = format!("https://cdn.discordapp.com/avatars/{}/{}.webp?size=4096", avatar_url.id, avatar_id);
            if ignores.contains(&att) {
                continue;
            }
            urls.push(att);
        }
    }
    eprintln!(
        "Finished avatars. Now extracting messages...\nThis may take a while. Go get a coffee."
    );
    let mut stmt = conn.prepare("SELECT * FROM messages").unwrap();
    let person_iter = stmt
        .query_map([], |row| {
            Ok(S {
                data: row.get(3).unwrap(), // message data is on 4th column of each row.
            })
        })
        .unwrap();
    for message in person_iter {
        let m = message.unwrap().data;
        for mat in regex.find_iter(&m) {
            let i = mat.as_str();
            if !ignores.contains(&i.to_string()) {
                urls.push(i.to_string());
            }
        }
    }
    eprintln!("Finally, extracting embeds...");
    let mut stmt = conn.prepare("SELECT * FROM embeds").unwrap();
    let person_iter = stmt
        .query_map([], |row| {
            Ok(S {
                data: row.get(1).unwrap(),
            })
        })
        .unwrap();
    for embed in person_iter {
        let embed = embed.unwrap().data.parse().expect("bad JSON");
        let mut hm = HashMap::new();
        hm.insert("embeds".to_string(), JsonValue::Array(vec![embed]));
        let embeds = JsonValue::Object(hm);
        for url in get_embed_urls(embeds, regex.clone()) {
            if !ignores.contains(&url) {
                urls.push(url);
            }
        }
    }
    RetVal { urls, ignores }
}

// Some of the following code logic is taken from
// https://github.com/Sanqui/discard2/blob/master/src/reader/reader.ts
fn messages_from_json(json: JsonValue) -> JsonValue {
    let regex = Regex::new(r#"^/api/v9/channels/\d+/messages"#).unwrap(); // the api endpoint currently used by discord
    let request_type: String = json["type"].clone().try_into().unwrap();
    if request_type != "http" {
        return JsonValue::Array(Vec::new());
    }
    let status_code: f64 = json["response"]["status_code"].clone().try_into().unwrap();
    let endpoint: String = json["request"]["path"].clone().try_into().unwrap();
    let method: String = json["request"]["method"].clone().try_into().unwrap();
    if request_type == "http"
            && method == "GET"
            && status_code == 200.0
            && regex.is_match(&endpoint) {
        return json["response"]["data"].clone();
    }
    JsonValue::Array(Vec::new())
}
fn get_embed_urls(json: JsonValue, regex: Regex) -> Vec<String> {
    let embeds: Vec<JsonValue> = json["embeds"].clone().try_into().unwrap();
    let mut this_code_sucks_lol = Vec::new();
    for embed in embeds {
        let embed: HashMap<String, JsonValue> = embed.clone().try_into().unwrap();
        let mut description: String = "".to_string();
        if embed.contains_key("description") { // We really need to wrap these into a function
            description = embed["description"].clone().try_into().unwrap();
        }
        if embed.contains_key("title") {
            let title: String = embed["title"].clone().try_into().unwrap();
            description = format!("{}\n{}", title, description); // to reduce code duplication (since we already operate on the description)
        }
        if embed.contains_key("footer") {
            let footer: HashMap<String, JsonValue> = embed["footer"].clone().try_into().unwrap();
            let footer_text: String = embed["footer"]["text"].clone().try_into().unwrap();
            description = format!("{}\n{}", description, footer_text);
            if footer.contains_key("icon_url") {
                let icon_url: String = embed["footer"]["icon_url"].clone().try_into().unwrap();
                this_code_sucks_lol.push(icon_url);
            }
        }
        if embed.contains_key("thumbnail") {
            let thumbnail: String = embed["thumbnail"]["url"].clone().try_into().unwrap();
            this_code_sucks_lol.push(thumbnail);
        }
        if embed.contains_key("video") {
            if embed.contains_key("url") {
                let url: String = embed["video"]["url"].clone().try_into().unwrap();
                this_code_sucks_lol.push(url);
            } else if embed.contains_key("proxy_url") {
                // better than nothing
                let url: String = embed["video"]["proxy_url"].clone().try_into().unwrap();
                this_code_sucks_lol.push(url);
            } else {
                eprintln!("EMBED WARNING: Video does not have a url or proxy_url.")
            }
        }
        if embed.contains_key("author") {
            let author: HashMap<String, JsonValue> = embed["author"].clone().try_into().unwrap();
            if author.contains_key("url") && !author["url"].is_null() {
                let url: String = author["url"].clone().try_into().unwrap();
                this_code_sucks_lol.push(url);
            }
            if author.contains_key("icon_url") {
                let url: String = author["icon_url"].clone().try_into().unwrap();
                this_code_sucks_lol.push(url);
            }
            if author.contains_key("proxy_icon_url") {
                let url: String = author["proxy_icon_url"].clone().try_into().unwrap();
                this_code_sucks_lol.push(url);
            }
        }
        if embed.contains_key("provider") {
            let provider: HashMap<String, JsonValue> =
                embed["provider"].clone().try_into().unwrap();
            if provider.contains_key("url") {
                let url: String = provider["url"].clone().try_into().unwrap();
                this_code_sucks_lol.push(url);
            }
        }
        // not sure searching for urls in the fields is worth it since it's so small
        let m = description;
        for mat in regex.find_iter(&m) {
            let i = mat.as_str();
            this_code_sucks_lol.push(i.to_string());
        }
        if embed.contains_key("image") {
            let image: String = embed["image"]["url"].clone().try_into().unwrap();
            this_code_sucks_lol.push(image);
        }
        if embed.contains_key("url") && !embed["url"].is_null() {
            let url: String = embed["url"].clone().try_into().unwrap();
            this_code_sucks_lol.push(url.to_string());
        }
    }
    this_code_sucks_lol
    // todo reduce duplication
}

fn get_component_urls(json: JsonValue) -> Vec<String> {
    let mut urls = Vec::new();
    let components: Vec<JsonValue> = json["components"].clone().try_into().unwrap();
    for raw_component_lis in components {
        let component_lis: Vec<JsonValue> = raw_component_lis["components"].clone().try_into().unwrap();
        for component_ra in component_lis {
            let component: HashMap<String, JsonValue> = component_ra.clone().try_into().unwrap();
            //println!("{:?}", component);
            if component.contains_key("url") {
                let url: String = component["url"].clone().try_into().unwrap();
                urls.push(url);
            }
        }
    }
    urls
}

fn discard2_jsonl(
    filename: &str,
    mut ignores: Vec<String>,
    mut urls: Vec<String>,
    regex: Regex,
    arguments: cli::Args
) -> RetVal {
    let guild_id: String = match arguments.guild_id {
        Some(id) => id,
        None => "a".to_string() // one that will never be used
    };
    if guild_id == "a" && arguments.use_websockets {
        panic!("You have to provide the server ID to use --parse-websockets. See --help for more information.");
    }
    let file = File::open(filename).expect("Failed to open file");
    let reader = BufReader::new(file);
    eprintln!("Searching through messages, this may take some time...");
    for line in reader.lines() {
        let json_line: JsonValue = line.unwrap().parse().expect("Failed to parse JSONL");
        let request_type: String = json_line["type"].clone().try_into().unwrap();
        if request_type == "ws" && arguments.use_websockets {
            let direction: String = json_line["direction"].clone().try_into().unwrap();
            if direction == "recv" {
                for url in gateway::gateway_parse(json_line["data"].clone(), guild_id.clone()) {
                    if !ignores.contains(&url) {
                        urls.push(url.clone());
                        ignores.push(url);
                    }
                }
            }
        }
        let messages: Vec<JsonValue> = messages_from_json(json_line).try_into().unwrap();
        for message in messages { // try to match with the url regex
            let m: String = message["content"].clone().try_into().unwrap();
            for mat in regex.find_iter(&m) {
                let i = mat.as_str();
                if !ignores.contains(&i.to_string()) {
                    ignores.push(i.to_string());
                    urls.push(i.to_string());
                }
            }
            // Avatar
            if !message["author"]["avatar"].is_null() {
                let user_id: String = message["author"]["id"].clone().try_into().unwrap();
                let avatar_id: String = message["author"]["avatar"].clone().try_into().unwrap();
                let avatar_link = format!(
                    "https://cdn.discordapp.com/avatars/{}/{}.webp?size=4096", // get the highest quality avatar available
                    user_id, avatar_id
                );
                if !ignores.contains(&avatar_link.to_string()) {
                    urls.push(avatar_link.clone());
                    ignores.push(avatar_link);
                }
            }
            // Attachments
            let attachments: Vec<JsonValue> = message["attachments"].clone().try_into().unwrap();
            for attachment in attachments {
                let i: String = attachment["url"].clone().try_into().unwrap();
                if !ignores.contains(&i.to_string()) {
                    urls.push(i.clone());
                    ignores.push(i);
                }
            }
            // Embeds (this is a big one!)
            for embed_url in get_embed_urls(message.clone(), regex.clone()) {
                if !ignores.contains(&embed_url.to_string()) {
                    urls.push(embed_url.clone());
                    ignores.push(embed_url);
                }
            }
            for component_url in get_component_urls(message.clone()) {
                if !ignores.contains(&component_url.to_string()) {
                    urls.push(component_url.clone());
                    ignores.push(component_url);
                }
            }
            //println!("{:?}", message);
        }
    }
    RetVal { urls, ignores }
}
fn plain_text(filename: &str, ignores: Vec<String>, mut urls: Vec<String>, regex: Regex, arguments: cli::Args) -> RetVal {
    if arguments.use_websockets {
        panic!("--parse-websockets can only be used with discard2 JSONL");
    }
    let file = File::open(filename).expect("Failed to open file");
    let reader = BufReader::new(file);
    eprintln!("Searching through lines of plain-text file...");
    for line in reader.lines() {
        let m = line.unwrap();
        for mat in regex.find_iter(&m) {
            let i = mat.as_str();
            if !ignores.contains(&i.to_string()) {
                urls.push(i.to_string());
            }
        }
    }
    RetVal { urls, ignores }
}

fn main() {
    if std::path::Path::new("urls.url").exists() {
        panic!("cowardly refusing to overwrite urls.url");
    }
    let regex = Regex::new(r#"(https?://[^\s<]+[^?~*|<>.,:;"'`)\]\s])"#).unwrap();
    let arguments = cli::Args::parse();
    let mut ignores = read_data();
    let mut urls = vec![];
    let filename: &str = &arguments.file.clone().into_os_string().into_string().unwrap();
    let s: RetVal = match arguments.file_type.as_str() {
        "dht" => sql(filename, ignores.clone(), urls, regex, arguments),
        "plaintext" => plain_text(filename, ignores.clone(), urls, regex, arguments),
        "discard2" => discard2_jsonl(filename, ignores.clone(), urls, regex, arguments),
        "dce" => dce(filename, ignores.clone(), urls, regex, arguments),
        _ => panic!("...?")
    };
    urls = s.urls;
    ignores = s.ignores;
    for url_to_ignore in &urls {
        ignores.push(url_to_ignore.clone());
    }
    write_data(ignores, urls);
}
