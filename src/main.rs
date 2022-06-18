use regex::Regex;
use rusqlite::Connection;
use std::io::Write;
use std::fs;
use std::fs::File;
use std::io::{Error, BufReader, BufRead};
use std::collections::HashMap;

use tinyjson::JsonValue;

#[derive(Debug)]
struct S {
    data: String
}

struct RetVal {
    urls: Vec<String>,
    ignores: Vec<String>
}

fn read_data() -> Vec<String> {
    eprintln!("Reading ignores. Please wait...");
    let status = fs::read_to_string("ignores.url");//.split("\n").collect();
    let status = match status {
        Ok(s) => s,
        Err(_) => { eprintln!("failed to read ignores.url, proceeding with default");"".to_string() },
    };
    let ej: Vec<&str> = status.split('\n').collect();
    let mut fj = vec!();
    for i in ej {
        fj.push(i.to_string());
    }
    fj
}

fn write_data(ignores: Vec<String>, urls: Vec<String>) {
    eprintln!("Now writing data.");
    let file = fs::OpenOptions::new()
              .write(true)
                    .append(true)
                    .create(true)
                          .open("ignores.url");
    let mut filefailed = false;
    let mut error: Error = Error::new(std::io::ErrorKind::Other, "bye");
    let mut file = match file {
        Ok(fil) => {filefailed = false; fil},
        Err(e) =>{error = e;filefailed = true; fs::OpenOptions::new().write(true).append(true).open("/dev/null").unwrap()},
    };
    if !filefailed {
        for ignore in ignores {
            writeln!(file, "{}", ignore).expect("failed to write file");
        }
    }
    else {
        eprintln!("Failed to write ignores: {}", error);
    }
    let mut file = fs::OpenOptions::new().create(true).write(true).open("urls.url").unwrap();
    for url in urls { writeln!(file, "{}", url).expect("failed to write URLs"); }
    //https://www.codegrepper.com/code-examples/rust/rust+how+to+append+to+a+file
}

fn sql(filename: &str, ignores: Vec<String>, mut urls: Vec<String>, regex: Regex) -> RetVal {
    eprintln!("Connecting to SQL database...");
    let conn = Connection::open(filename).unwrap();
    eprintln!("Attachments...");
    let mut stmt = conn.prepare("SELECT * FROM attachments").unwrap();
    let person_iter = stmt.query_map([], |row| {
                Ok(S {
                    data: row.get(4).unwrap(), //attachment URL is on 5th column of each row.
                    })
                    }).unwrap();
    for attachment_url in person_iter {
        let att = attachment_url.unwrap().data;
        if ignores.contains(&att) {
            continue;
        }
        urls.push(att);
    }
    eprintln!("Finished attachments. Now extracting messages...\nThis may take a while. Go get a coffee.");
    let mut stmt = conn.prepare("SELECT * FROM messages").unwrap();
    let person_iter = stmt.query_map([], |row| {
        Ok(S {
            data: row.get(3).unwrap(), // message data is on 4th column of each row.
        })}).unwrap();
    for message in person_iter {
        let m = message.unwrap().data;
        for mat in regex.find_iter(&m) {
            let i = mat.as_str();
            if !ignores.contains(&i.to_string()) {
                urls.push(i.to_string());
            }
        }
    }
    RetVal { urls, ignores }
}

// Some of the following code logic is taken from
// https://github.com/Sanqui/discard2/blob/master/src/reader/reader.ts
fn messages_from_json(json: JsonValue) -> JsonValue {
    let status_code: f64 = json["response"]["status_code"].clone().try_into().unwrap();
    let endpoint: String = json["request"]["path"].clone().try_into().unwrap();
    let method: String = json["request"]["method"].clone().try_into().unwrap();
    let request_type: String = json["type"].clone().try_into().unwrap();
    let regex = Regex::new(r#"^/api/v9/channels/\d+/messages"#).unwrap();
    if request_type == "http" && method == "GET" && status_code == 200.0
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
        if embed.contains_key("description") {
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
                let icon_url: String = embed["footer"]["icon_url"]
                    .clone().try_into().unwrap();
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
            }
            else if embed.contains_key("proxy_url") { // better than nothing
                let url: String = embed["video"]["proxy_url"].clone().try_into().unwrap();
                this_code_sucks_lol.push(url);
            }
            else {
                eprintln!("EMBED WARNING: Video does not have a url or proxy_url.")
            }
        }
        if embed.contains_key("author") {
            let author: HashMap<String, JsonValue> = embed["author"].clone().try_into().unwrap();
            if author.contains_key("url") {
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
            let provider: HashMap<String, JsonValue> = embed["provider"].clone().try_into().unwrap();
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
        if embed.contains_key("url") {
            let url: String = embed["url"].clone().try_into().unwrap();
            this_code_sucks_lol.push(url.to_string());
        }
    }
    this_code_sucks_lol
        // todo reduce duplication
}
fn discard2_jsonl(filename: &str, mut ignores: Vec<String>, mut urls: Vec<String>, regex: Regex) -> RetVal {
    let file = File::open(filename).expect("Failed to open file");
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let json_line: JsonValue = line.unwrap().parse().expect("Failed to parse JSONL");
        let messages: Vec<JsonValue> = messages_from_json(json_line).try_into().unwrap();
        for message in messages {
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
                let avatar_link = format!("https://cdn.discordapp.com/avatars/{}/{}.webp?size=4096", user_id, avatar_id);
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
            //println!("{:?}", message);
        }
    }
    RetVal { urls, ignores }
}
fn plain_text(filename: &str, ignores: Vec<String>, mut urls: Vec<String>, regex: Regex) -> RetVal {
    let file = File::open(filename).expect("Failed to open file");
    let reader = BufReader::new(file);
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
    let regex = Regex::new(r#"(https?://[^\s<]+[^?~*|<>.,:;"'`)\]\s])"#).unwrap();
    let args: Vec<String> = std::env::args().collect();
    let usage = format!("Usage: {} <file> <type: dht|plaintext|discard2>", &args[0]);
    if args.len() < 3 {
        panic!("{}", usage);
    }
    let mut ignores = read_data();
    let mut urls = vec!();
    let s: RetVal = match args[2].as_str() {
        "dht" => sql(&args[1], ignores.clone(), urls.clone(), regex),
        "plaintext" => plain_text(&args[1], ignores.clone(), urls.clone(), regex),
        "discard2" => discard2_jsonl(&args[1], ignores.clone(), urls.clone(), regex),
        _ => panic!("{}", usage)
    };
    urls = s.urls;
    ignores = s.ignores;
    for url_to_ignore in &urls {
        ignores.push(url_to_ignore.clone());
    }
    write_data(ignores, urls);

}
