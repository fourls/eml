use crate::parse::util::next_crlf;

pub fn take_header(buffer: &[u8]) -> (&[u8], usize) {
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

pub fn take_headers(buffer: &[u8]) -> (Vec<&[u8]>, usize) {
    let mut index = 0;
    let mut headers = vec![];

    // Headers
    while index < buffer.len() {
        let (header_buf, size) = take_header(&buffer[index..]);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_email_header_null_line() {
        let text = b"\r\n";
        let (buf, size) = take_header(text);
        assert_eq!(buf, &b""[..]);
        assert_eq!(size, 0);
    }

    #[test]
    fn test_read_simple_email_header() {
        let text = b"Mime-Version: 1.0\r\n";
        let (buf, size) = take_header(text);
        assert_eq!(buf, &b"Mime-Version: 1.0"[..]);
        assert_eq!(size, 17);
    }

    #[test]
    fn test_read_email_header_no_crlf() {
        let text = b"Mime-Version: 1.0";
        let (buf, size) = take_header(text);
        assert_eq!(buf, &b"Mime-Version: 1.0"[..]);
        assert_eq!(size, 17);
    }

    #[test]
    fn test_read_email_header_one_of_many() {
        let text = b"Mime-Version: 1.0\r\nContent-Type: text/plain\r\n";
        let (buf, size) = take_header(text);
        assert_eq!(buf, &b"Mime-Version: 1.0"[..]);
        assert_eq!(size, 17);
    }

    #[test]
    fn test_read_multiline_email_header() {
        let text = b"Content-Type: text/plain;\r\n  charset=utf-8\r\n";
        let (buf, size) = take_header(text);
        assert_eq!(buf, &b"Content-Type: text/plain;\r\n  charset=utf-8"[..]);
        assert_eq!(size, 42);
    }

    #[test]
    fn test_read_email_headers() {
        let text =
            b"Mime-Version: 1.0\r\nSubject: What's The Deal With\r\n\tAirplane Food?\r\n\r\nContent-Type: text/plain\r\n";

        let (headers, size) = take_headers(text);

        assert_eq!(size, 69);
        assert_eq!(headers[0], &b"Mime-Version: 1.0"[..]);
        assert_eq!(
            headers[1],
            &b"Subject: What's The Deal With\r\n\tAirplane Food?"[..]
        );
        assert_eq!(headers.get(2), None);
    }
}
