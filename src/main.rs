use std::{fs::File, io::prelude::*};

fn next_crlf(buffer: &[u8]) -> Option<usize> {
    let mut last_char_cr = false;

    for (i, &c) in buffer.iter().enumerate() {
        if c == b'\r' {
            last_char_cr = true;
        } else if last_char_cr && c == b'\n' {
            return Some(i - 1);
        } else if last_char_cr {
            last_char_cr = false;
        }
    }

    None
}

fn read_email_header(buffer: &[u8]) -> (&[u8], usize) {
    let mut includes_next_line = true;
    let mut end: usize = 0;
    while includes_next_line {
        let buffer_rem = &buffer[end..];

        end = end + next_crlf(buffer_rem).unwrap_or(buffer_rem.len());
        includes_next_line = match buffer.get(end + 2) {
            Some(&c) => match c {
                b'\r' | b'\n' => false,
                c => c.is_ascii_whitespace(),
            },
            None => false,
        };

        if includes_next_line {
            end += 2; // Offset to include CRLF
        }
    }

    (&buffer[..end], end)
}

fn read_email_headers(buffer: &[u8]) -> (Vec<&[u8]>, usize) {
    let mut index = 0;
    let mut headers = vec![];

    // Headers
    while index < buffer.len() {
        let (header_buf, size) = read_email_header(&buffer[index..]);

        index = index + size + 2; // Skip CRLF

        if size == 0 {
            // Header and body are separated by null line
            break;
        } else {
            headers.push(header_buf);
        }
    }

    (headers, index)
}

struct Header {
    pub name: String,
    pub body: String,
}

#[derive(Debug)]
enum HeaderParseError {
    EncodingError(std::str::Utf8Error),
    Malformed,
}

impl Header {
    pub fn parse(buffer: &[u8]) -> Result<Header, HeaderParseError> {
        let text = match std::str::from_utf8(buffer) {
            Ok(s) => s,
            Err(e) => return Err(HeaderParseError::EncodingError(e)),
        };

        let mut split_text = text.split(':');
        let field_name = match split_text.next() {
            Some(s) => String::from(s),
            None => return Err(HeaderParseError::Malformed),
        };
        let field_body = match split_text.next() {
            Some(s) => s.replace("\r\n", ""),
            None => return Err(HeaderParseError::Malformed),
        };

        Ok(Header {
            name: field_name,
            body: field_body,
        })
    }
}

fn main() {
    let mut f = File::open("test.txt").unwrap();
    let mut buffer = vec![];
    f.read_to_end(&mut buffer).unwrap();

    let (header_bufs, size) = read_email_headers(&buffer);
    let body_buf = &buffer[size..];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_crlf() {
        assert_eq!(
            next_crlf(b"Mime-Version: 1.0\r\nContent-Type: text/plain\r\n"),
            Some(17)
        );
        assert_eq!(next_crlf(b"Content-Type: text/plain\r\n"), Some(24));
        assert_eq!(next_crlf(b"\r\n"), Some(0));
        assert_eq!(next_crlf(b"abcd\nef\r\n"), Some(7));
    }

    #[test]
    fn test_next_crlf_no_crlfs() {
        assert_eq!(next_crlf(b"abcd"), None);
        assert_eq!(next_crlf(b"abcd\nef\n\r"), None);
    }

    #[test]
    fn test_read_email_header_null_line() {
        let text = b"\r\n";
        let (buf, size) = read_email_header(text);
        assert_eq!(buf, &b""[..]);
        assert_eq!(size, 0);
    }

    #[test]
    fn test_read_simple_email_header() {
        let text = b"Mime-Version: 1.0\r\n";
        let (buf, size) = read_email_header(text);
        assert_eq!(buf, &b"Mime-Version: 1.0"[..]);
        assert_eq!(size, 17);
    }

    #[test]
    fn test_read_email_header_no_crlf() {
        let text = b"Mime-Version: 1.0";
        let (buf, size) = read_email_header(text);
        assert_eq!(buf, &b"Mime-Version: 1.0"[..]);
        assert_eq!(size, 17);
    }

    #[test]
    fn test_read_email_header_one_of_many() {
        let text = b"Mime-Version: 1.0\r\nContent-Type: text/plain\r\n";
        let (buf, size) = read_email_header(text);
        assert_eq!(buf, &b"Mime-Version: 1.0"[..]);
        assert_eq!(size, 17);
    }

    #[test]
    fn test_read_multiline_email_header() {
        let text = b"Content-Type: text/plain;\r\n  charset=utf-8\r\n";
        let (buf, size) = read_email_header(text);
        assert_eq!(buf, &b"Content-Type: text/plain;\r\n  charset=utf-8"[..]);
        assert_eq!(size, 42);
    }

    #[test]
    fn test_read_email_headers() {
        let text =
            b"Mime-Version: 1.0\r\nSubject: What's The Deal With\r\n\tAirplane Food?\r\n\r\nContent-Type: text/plain\r\n";

        let (headers, size) = read_email_headers(text);

        assert_eq!(size, 69);
        assert_eq!(headers[0], &b"Mime-Version: 1.0"[..]);
        assert_eq!(
            headers[1],
            &b"Subject: What's The Deal With\r\n\tAirplane Food?"[..]
        );
        assert_eq!(headers.get(2), None);
    }

    #[test]
    fn test_parse_header() {
        let header = Header::parse(b"Mime-Version: 1.0").expect("Error parsing header");
        assert_eq!(header.name, "Mime-Version");
        assert_eq!(header.body, " 1.0");
    }

    #[test]
    fn test_parse_multiline_header() {
        let header = Header::parse(b"Subject: What's The Deal With\r\n Airplane Food?")
            .expect("Error parsing header");
        assert_eq!(header.name, "Subject");
        assert_eq!(header.body, " What's The Deal With Airplane Food?");
    }
}
