pub fn encode(input: &[u8]) -> String {
    let mut input = input.to_vec();
    let l = input.len() % 3;
    while input.len() % 3 > 0 {
        input.push(0);
    }

    let mut base64 = input
        .chunks(3)
        .flat_map(encode_chunk)
        .map(encode_char)
        .collect::<String>();

    if l > 0 {
        base64.pop();
        if l == 1 {
            base64.pop();
        }
    }

    base64
}

pub fn decode(input: &str) -> Option<Vec<u8>> {
    let mut vec = input.chars().map(decode_char).collect::<Option<Vec<_>>>()?;
    while vec.len() % 4 > 0 {
        vec.push(0);
    }

    let str = vec
        .chunks(4)
        .flat_map(decode_chunk)
        .take_while(|&c| c != 0)
        .collect::<Vec<_>>();

    Some(str)
}

fn encode_chunk(chunk: &[u8]) -> [u8; 4] {
    let i0 = chunk[0];
    let i1 = chunk[1];
    let i2 = chunk[2];

    let o0 = i0 >> 2;
    let o1 = (i0 & 0b11) << 4 | (i1 >> 4);
    let o2 = (i1 & 0b1111) << 2 | (i2 >> 6);
    let o3 = i2 & 0b111111;

    [o0, o1, o2, o3]
}

fn decode_chunk(chunk: &[u8]) -> [u8; 3] {
    let i0 = chunk[0];
    let i1 = chunk[1];
    let i2 = chunk[2];
    let i3 = chunk[3];

    let o0 = i0 << 2 | i1 >> 4;
    let o1 = (i1 & 0b1111) << 4 | i2 >> 2;
    let o2 = (i2 & 0b11) << 6 | i3;

    [o0, o1, o2]
}

fn encode_char(char: u8) -> char {
    match char {
        0..=25 => (b'A' + char) as char,
        26..=51 => (b'a' + char - 26) as char,
        52..=61 => (b'0' + char - 52) as char,
        62 => '-',
        63 => '_',
        _ => unreachable!(),
    }
}

fn decode_char(char: char) -> Option<u8> {
    match char {
        'A'..='Z' => Some(char as u8 - b'A'),
        'a'..='z' => Some(char as u8 - b'a' + 26),
        '0'..='9' => Some(char as u8 - b'0' + 26 + 26),
        '-' => Some(62),
        '_' => Some(63),
        '=' => Some(0),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_encode_a_char() {
        assert_eq!('A', encode_char(0));
        assert_eq!('Z', encode_char(25));
        assert_eq!('a', encode_char(26));
        assert_eq!('z', encode_char(51));
        assert_eq!('0', encode_char(52));
        assert_eq!('9', encode_char(61));
        assert_eq!('-', encode_char(62));
        assert_eq!('_', encode_char(63));
    }

    #[test]
    fn it_should_decode_a_char() {
        assert_eq!(Some(0), decode_char('A'));
        assert_eq!(Some(25), decode_char('Z'));
        assert_eq!(Some(26), decode_char('a'));
        assert_eq!(Some(51), decode_char('z'));
        assert_eq!(Some(52), decode_char('0'));
        assert_eq!(Some(61), decode_char('9'));
        assert_eq!(Some(62), decode_char('-'));
        assert_eq!(Some(63), decode_char('_'));
        assert_eq!(Some(0), decode_char('='));
    }

    #[test]
    fn it_should_return_none_for_bad_char() {
        assert_eq!(None, decode_char('?'));
    }

    #[test]
    fn it_should_encode_a_string() {
        assert_eq!("".to_string(), encode(b""));
        assert_eq!("YWJj".to_string(), encode(b"abc"));
        assert_eq!("SGVsbG8gV29ybGQ".to_string(), encode(b"Hello World"));
        assert_eq!(
            "TG9yZW0gaXBzdW0sIGRvbG9yIHNpdCBhbWV0Lg".to_string(),
            encode(b"Lorem ipsum, dolor sit amet.")
        );
    }

    #[test]
    fn it_should_decode_a_string() {
        assert_eq!(Some("".into()), decode(""));
        assert_eq!(Some("abc".into()), decode("YWJj"));
        assert_eq!(Some("Hello World".into()), decode("SGVsbG8gV29ybGQ="));
        assert_eq!(
            Some("Lorem ipsum, dolor sit amet.".into()),
            decode("TG9yZW0gaXBzdW0sIGRvbG9yIHNpdCBhbWV0Lg==")
        );
    }
}
