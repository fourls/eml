use std::{fs::File, io::prelude::*};

mod parse;

fn main() {
    let mut f = File::open("test.txt").unwrap();
    let mut buffer = vec![];
    f.read_to_end(&mut buffer).unwrap();

    let (headers, body_start) = parse::header::parse_email_headers(&buffer).unwrap();
}
