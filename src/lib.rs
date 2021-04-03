use wasm_bindgen::prelude::*;

#[macro_use]
extern crate lazy_static;

mod encoding;
use encoding::Encoding;

mod repack;
use repack::RepackIterator;

#[wasm_bindgen]
pub fn decode(text: &str, charset: &str) -> String {
    let charset = match Encoding::parse(charset) {
        Some(c) => c,
        _ => return "Error: bad charset".to_owned(),
    };

    match _decode(text, charset) {
        Ok(res) => std::str::from_utf8(&res[..])
            .unwrap_or("Invalid UTF-8 string")
            .to_string(),
        Err(e) => e.to_owned(),
    }
}

#[wasm_bindgen]
pub fn encode(text: &str, charset: &str) -> String {
    let charset = match Encoding::parse(charset) {
        Some(c) => c,
        _ => return "Error: bad charset".to_owned(),
    };

    _encode(text.as_bytes(), charset)
}

fn _encode(bytes: &[u8], encoding: &Encoding) -> String {
    let it = bytes.iter().map(|&x| x as u16);
    let it = RepackIterator::new(it, 8, encoding.bitcount(), false);
    it.map(encoding.enc_fn()).collect()
}

fn _decode(text: &str, encoding: &Encoding) -> Result<Vec<u8>, String> {
    let text = text.trim_end_matches(encoding.escape_char);

    if let Err((idx, c)) = encoding.validate(text) {
        return Err(format!(
            "Error decoding: unknown character '{}' at position {}",
            c, idx
        ));
    }

    let it = text.chars().map(encoding.dec_fn());
    let it = RepackIterator::new(it, encoding.bitcount(), 8, true);

    Ok(it.map(|x| x as u8).collect())
}

#[cfg(test)]
mod tests {
    use crate::{_decode, _encode, decode, encode};

    #[test]
    fn test_base64_empty_inputs() {
        assert_eq!("", encode("", "base64"));
        assert_eq!("", decode("", "base64"));
    }

    #[test]
    fn test_base64_sample_text() {
        let ref_dec = "Hello world";
        let ref_enc = "SGVsbG8gd29ybGQ";

        assert_eq!(ref_enc, encode(ref_dec, "base64"));
        assert_eq!(ref_dec, decode(ref_enc, "base64"));
    }

    #[test]
    fn test_base64_padding() {
        let ref_dec = "Hello world";
        let ref_enc = "SGVsbG8gd29ybGQ==";
        assert_eq!(ref_dec, decode(ref_enc, "base64"));
    }

    #[test]
    fn test_base64_bytespace() {
        let ref_dec = (0..255).chain(255..=0).collect::<Vec<u8>>();
        let codec = crate::encoding::Encoding::parse("base64").unwrap();

        let enc = _encode(&ref_dec, codec);
        let dec = _decode(&enc, codec).unwrap();
        assert_eq!(ref_dec, dec);
    }

    #[test]
    fn test_utf8_error() {
        let reff = "㞻㧝㫮㭷㞻㧝㫮㭷㞻㧝㫮㭷㞻㧝㫮㭷㞻㧝㫮㭷㞻㧝㨀㨀㨀㨀";
        let dec = decode(&reff, "kanji");
        assert!(dec.contains("Invalid UTF-8"));
    }
}
