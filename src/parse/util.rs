pub fn next_crlf(buffer: &[u8]) -> Option<usize> {
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
}
