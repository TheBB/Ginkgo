use crate::string::*;

#[test]
fn unescape_() {
    assert_eq!(unescape("abc"), Some("abc".to_string()));
    assert_eq!(unescape("\\0"), Some("\0".to_string()));
    assert_eq!(unescape("\\a"), Some("\x07".to_string()));
    assert_eq!(unescape("\\b"), Some("\x08".to_string()));
    assert_eq!(unescape("\\t"), Some("\t".to_string()));
    assert_eq!(unescape("\\n"), Some("\n".to_string()));
    assert_eq!(unescape("\\v"), Some("\x0b".to_string()));
    assert_eq!(unescape("\\f"), Some("\x0c".to_string()));
    assert_eq!(unescape("\\r"), Some("\r".to_string()));
    assert_eq!(unescape("\\e"), Some("\x1b".to_string()));
    assert_eq!(unescape("\\\""), Some("\"".to_string()));
    assert_eq!(unescape("\\\\"), Some("\\".to_string()));
    
    assert_eq!(unescape("\\^@"), Some("\0".to_string()));
    assert_eq!(unescape("\\^A"), Some("\x01".to_string()));
    assert_eq!(unescape("\\^a"), Some("\x01".to_string()));
    assert_eq!(unescape("\\^Z"), Some("\x1a".to_string()));
    assert_eq!(unescape("\\^z"), Some("\x1a".to_string()));
    assert_eq!(unescape("\\^["), Some("\x1b".to_string()));
    assert_eq!(unescape("\\^_"), Some("\x1f".to_string()));
    assert_eq!(unescape("\\^?"), Some("\x7f".to_string()));
    
    assert_eq!(
        unescape("\\a\\b\\t\\n\\v\\f\\r\\^T\\^_\\\"\\\\\\^?\\e"),
        Some("\x07\x08\x09\x0a\x0b\x0c\x0d\x14\x1f\x22\x5c\x7f\x1b".to_string())
    );
    
    assert_eq!(unescape("\\U000000f8"), Some("\u{f8}".to_string()));
    assert_eq!(unescape("\\u00f8"), Some("\u{f8}".to_string()));
}

#[test]
fn escape_() {
    assert_eq!(escape("abc"), "abc".to_string());
    assert_eq!(escape("\0"), "\\0".to_string());
    assert_eq!(escape("\x07"), "\\a".to_string());
    assert_eq!(escape("\x08"), "\\b".to_string());
    assert_eq!(escape("\t"), "\\t".to_string());
    assert_eq!(escape("\n"), "\\n".to_string());
    assert_eq!(escape("\x0b"), "\\v".to_string());
    assert_eq!(escape("\x0c"), "\\f".to_string());
    assert_eq!(escape("\r"), "\\r".to_string());
    assert_eq!(escape("\x1b"), "\\e".to_string());
    assert_eq!(escape("\""), "\\\"".to_string());
    assert_eq!(escape("\\"), "\\\\".to_string());
    
    assert_eq!(escape("\x01"), "\\^a".to_string());
    assert_eq!(escape("\x1a"), "\\^z".to_string());
    assert_eq!(escape("\x1f"), "\\^_".to_string());
    assert_eq!(escape("\x7f"), "\\^?".to_string());
    
    assert_eq!(
        escape("\x07\x08\x09\x0a\x0b\x0c\x0d\x14\x1f\x22\x5c\x7f\x1b"),
        "\\a\\b\\t\\n\\v\\f\\r\\^t\\^_\\\"\\\\\\^?\\e",
    );
}
