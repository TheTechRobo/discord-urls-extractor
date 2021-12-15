use rusqlite::{params, Connection, Result};
use std::io::Write;
use std::fs;

#[derive(Debug)]

struct S {
    data: String
}

fn read_data() -> Vec<String> {
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
        write!(file, "{}\n", ignore);
    }
    }
    let mut file = fs::OpenOptions::new().create(true).write(true).open("urls.url").unwrap();
    for url in urls { write!(file, "{}\n", url); }
    //https://www.codegrepper.com/code-examples/rust/rust+how+to+append+to+a+file
}
fn main() {
        let args: Vec<String> = std::env::args().collect();
            if args.len() < 2 {
                        panic!("not enough arguments lol");
                            }

    let mut ignores = read_data();
    let mut urls = vec!();
    eprintln!("Please wait...");
    let conn = Connection::open(&args[1]).unwrap();
    let mut stmt = conn.prepare("SELECT * FROM attachments").unwrap();
    let person_iter = stmt.query_map([], |row| {
                Ok(S {
                    data: row.get(4).unwrap(),
                    })
                    }).unwrap();
    for hi in person_iter {
        ignores.push(hi.unwrap().data);
        //urls.push(hi.unwrap().data.clone());
    }
    let mut stmt = conn.prepare("SELECT * FROM attachments").unwrap();
    let position_iter = stmt.query_map([], |row| {
        Ok(S { data:row.get(4).unwrap(),})}).unwrap();

    for hi2 in position_iter {
        urls.push(hi2.unwrap().data);
    }
    write_data(ignores, urls);

}
