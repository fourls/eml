mod bufread;

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    pub name: String,
    pub body: String,
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.body)
    }
}

#[derive(Debug)]
pub enum HeaderParseError {
    EncodingError(std::str::Utf8Error),
    Malformed,
}

fn parse_header(buffer: &[u8]) -> Result<Header, HeaderParseError> {
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

#[derive(Debug)]
pub enum EmailParseError {
    InvalidHeaders,
}

pub fn parse_email_headers(buffer: &[u8]) -> Result<(Vec<Header>, usize), EmailParseError> {
    let (header_bufs, size) = bufread::take_headers(buffer);
    let header_results: Vec<_> = header_bufs.into_iter().map(|b| parse_header(b)).collect();

    if header_results.iter().any(|r| r.is_err()) {
        return Err(EmailParseError::InvalidHeaders);
    }

    Ok((
        header_results.into_iter().map(|r| r.unwrap()).collect(),
        size,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let header = parse_header(b"Mime-Version: 1.0").expect("Error parsing header");
        assert_eq!(header.name, "Mime-Version");
        assert_eq!(header.body, " 1.0");
    }

    #[test]
    fn test_parse_multiline_header() {
        let header = parse_header(b"Subject: What's The Deal With\r\n Airplane Food?")
            .expect("Error parsing header");
        assert_eq!(header.name, "Subject");
        assert_eq!(header.body, " What's The Deal With Airplane Food?");
    }

    #[test]
    fn test_parse_invalid_header() {
        let res = parse_header(b"Mime-Version = 1.0");
        assert!(match res {
            Err(e) => match e {
                HeaderParseError::Malformed => true,
                _ => false,
            },
            _ => false,
        })
    }

    #[test]
    fn test_parse_miscoded_header() {
        let res = parse_header(&[240, 30, 22, 168]);
        assert!(match res {
            Err(e) => match e {
                HeaderParseError::EncodingError(_) => true,
                _ => false,
            },
            _ => false,
        })
    }

    #[test]
    fn test_parse_email_headers() {
        let text = b"Subject: Foo Bar\r\n Baz\r\nMime-Version: 1.0\r\n\r\nHello world,\r\nThis is a foo message\r\n";

        let res = parse_email_headers(text).expect("Email parsing error");

        assert_eq!(
            res.0.get(0),
            Some(&Header {
                name: "Subject".into(),
                body: " Foo Bar Baz".into()
            })
        );
        assert_eq!(
            res.0.get(1),
            Some(&Header {
                name: "Mime-Version".into(),
                body: " 1.0".into()
            })
        );
        assert_eq!(res.0.get(2), None);
        assert_eq!(res.1, 45);
    }
}
