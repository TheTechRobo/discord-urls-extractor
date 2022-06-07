use regex::Regex;
use rusqlite::Connection;
use std::io::Write;
use std::fs;
use std::fs::File;
use std::io::{Error, BufReader, BufRead};

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
    let usage = format!("Usage: {} <file> <type: dht|plaintext>", &args[0]);
    if args.len() < 3 {
        panic!("{}", usage);
    }
    let mut ignores = read_data();
    let mut urls = vec!();
    let s: RetVal = match args[2].as_str() {
        "dht" => sql(&args[1], ignores.clone(), urls.clone(), regex),
        "plaintext" => plain_text(&args[1], ignores.clone(), urls.clone(), regex),
        _ => panic!("{}", usage)
    };
    urls = s.urls;
    ignores = s.ignores;
    for url_to_ignore in &urls {
        ignores.push(url_to_ignore.clone());
    }
    write_data(ignores, urls);

}
