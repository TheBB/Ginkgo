use std::str::Chars;
use std::char::from_u32;

#[allow(dead_code)]
pub fn unescape(input: &str) -> Option<String> {
    let mut chars = input.chars();
    let mut output = String::new();
    loop {
        match chars.next() {
            Some('\\') => {
                if let Some(c) = unescape_single(&mut chars) {
                    output.push(c);
                } else {
                    return None;
                }
            }
            Some(c) => output.push(c),
            _ => break,
        }
    }

    Some(output)
}

pub fn escape(input: &str) -> String {
    let mut output = String::new();
    for c in input.chars() {
        match c {
            '\0' => output.push_str("\\0"),
            '\x07' => output.push_str("\\a"),
            '\x08' => output.push_str("\\b"),
            '\x09' => output.push_str("\\t"),
            '\x0a' => output.push_str("\\n"),
            '\x0b' => output.push_str("\\v"),
            '\x0c' => output.push_str("\\f"),
            '\x0d' => output.push_str("\\r"),
            '\x1b' => output.push_str("\\e"),
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\x7f' => output.push_str("\\^?"),
            c @ '\x01'..='\x1a' => {
                output.push_str("\\^");
                output.push(from_u32((c as u32) + ('a' as u32) - 1).unwrap());
            }
            c @ '\x01'..='\x1f' => {
                output.push_str("\\^");
                output.push(from_u32((c as u32) + ('A' as u32) - 1).unwrap());
            }
            c => output.push(c),
        }
    }

    output
}

fn unescape_single(input: &mut Chars) -> Option<char> {
    match input.next() {
        Some('0') => Some('\x00'),
        Some('a') => Some('\x07'),
        Some('b') => Some('\x08'),
        Some('t') => Some('\x09'),
        Some('n') => Some('\x0a'),
        Some('v') => Some('\x0b'),
        Some('f') => Some('\x0c'),
        Some('r') => Some('\x0d'),
        Some('e') => Some('\x1b'),
        Some('"') => Some('\x22'),
        Some('\\') => Some('\x5c'),
        Some('^') => {
            match input.next() {
                Some('?') => Some('\x7f'),
                Some(c @ 'a'..='z') => from_u32(1 + (c as u32) - ('a' as u32)),
                Some(c @ '@'..='_') => from_u32(1 + (c as u32) - ('A' as u32)),
                _ => None,
            }
        }
        Some('u') => unescape_unicode(input, 4),
        Some('U') => unescape_unicode(input, 8),
        _ => None,
    }
}

fn unescape_unicode(input: &mut Chars, nchars: usize) -> Option<char> {
    let mut code: u32 = 0;
    for _ in 0..nchars {
        match input.next() {
            Some(c) if ('0'..='9').contains(&c) ||
                       ('a'..='f').contains(&c) ||
                       ('A'..='F').contains(&c) =>
                code = code * 16 + c.to_digit(16).unwrap(),
            _ => return None,
        }
    }

    from_u32(code)
}
