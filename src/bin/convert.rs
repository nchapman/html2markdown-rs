use std::io::{self, Read};

fn main() {
    let mut html = String::new();
    io::stdin().read_to_string(&mut html).expect("read stdin");
    let md = html_to_markdown::convert(&html);
    print!("{md}");
}
