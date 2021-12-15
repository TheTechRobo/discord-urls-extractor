use regex::Regex;
use rusqlite::{params, Connection, Result};
use std::io::Write;
use std::fs;

#[derive(Debug)]

struct S {
    data: String
}

fn read_data() -> Vec<String> {
    eprintln!("Reading ignores. Please wait...");
    let status = fs::read_to_string("ignores.url");//.split("\n").collect();
    let status = match status {
        Ok(s) => s,
        Err(err) => { eprintln!("failed to read ignores.url, proceeding with default");"".to_string() },
    };
    let ej: Vec<&str> = status.split("\n").collect();
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
    let mut file = match file {
        Ok(fil) => {filefailed = false; fil},
        Err(fil) =>{filefailed = true; fs::OpenOptions::new().write(true).append(true).open("/dev/null").unwrap()},
    };
    if filefailed == false {
    for ignore in ignores {
        write!(file, "{}\n", ignore).expect("failed to write file");
    }
    }
    let mut file = fs::OpenOptions::new().create(true).write(true).open("urls.url").unwrap();
    for url in urls { write!(file, "{}\n", url).expect("failed to write URLs"); }
    //https://www.codegrepper.com/code-examples/rust/rust+how+to+append+to+a+file
}
fn main() {
        let args: Vec<String> = std::env::args().collect();
            if args.len() < 2 {
                        panic!("not enough arguments lol");
                            }

    let mut ignores = read_data();
    let mut urls = vec!();
    eprintln!("Connecting to SQL Database...");
    let conn = Connection::open(&args[1]).unwrap();
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
        //let regex = Regex::new(r"(\n| |(|)|<|>)").unwrap();
        let regex = Regex::new(r"[\n()<>]").expect("bad regex"); // split by newlines, brackets, and angle brackets
        let splitted = regex.split(&m);
        for i in splitted {
            if i.starts_with("http://") || i.starts_with("https://") { // check if its actually an HTTP/S url
                if ignores.contains(&i.to_string()) {
                    continue;
                }
                urls.push(i.to_string());
            }
        }
    }
    for url_to_ignore in &urls {
        ignores.push(url_to_ignore.clone());
    }
    write_data(ignores, urls);

}
