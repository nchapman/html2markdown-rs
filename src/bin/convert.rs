use std::io::{self, Read};

fn main() {
    let mut html = String::new();
    io::stdin().read_to_string(&mut html).expect("read stdin");
    match html_to_markdown::convert(&html) {
        Ok(md) => print!("{md}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
