use wasm_bindgen::prelude::*;

mod encoding;
mod repack;
use encoding::Encoding;

#[wasm_bindgen]
#[derive(serde::Serialize)]
pub struct EncodingDescription {
    name: String,
    description: String,
}

#[wasm_bindgen]
pub fn get_encodings() -> Box<[JsValue]> {
    encoding::get_encodings()
        .iter()
        .map(|x| EncodingDescription {
            name: x.name.to_owned(),
            description: x.long_name.to_owned(),
        })
        .map(|x| JsValue::from_serde(&x).unwrap())
        .collect::<Vec<JsValue>>()
        .into_boxed_slice()
}

#[wasm_bindgen(catch)]
pub fn decode(text: &str, charset: &str) -> Result<Box<[u8]>, JsValue> {
    let charset = charset.parse()?;
    Ok(_decode(text, charset)?.into_boxed_slice())
}

#[wasm_bindgen(catch)]
pub fn decode_utf8(text: &str, charset: &str) -> Result<String, JsValue> {
    let charset = charset.parse()?;
    let dec = _decode(text, charset)?;
    String::from_utf8(dec).or(Err(JsValue::from_str("Invalid UTF-8 encoding")))
}

#[wasm_bindgen(catch)]
pub fn encode(text: &str, charset: &str) -> Result<String, JsValue> {
    let charset = charset.parse()?;
    Ok(_encode(text.as_bytes(), charset))
}

// Having JsValues makes the above functions panic for x86_64 targets at run-time,
// hence these sub-functions

fn _encode(bytes: &[u8], encoding: &Encoding) -> String {
    encoding.encode(bytes)
}

fn _decode(text: &str, encoding: &Encoding) -> Result<Vec<u8>, String> {
    encoding.decode(text)
}

#[cfg(test)]
mod tests {
    use crate::encoding::get_encodings as _get_enc;
    use crate::{_decode, _encode};

    const BLNS_URL: &str =
        "https://raw.githubusercontent.com/minimaxir/big-list-of-naughty-strings/master/blns.txt";

    // Indeed janky, but I don't want to pull the entire tokio crate for a simple test.
    fn get_blns() -> Vec<String> {
        let output = std::process::Command::new("curl")
            .arg("-s")
            .arg(BLNS_URL)
            .output()
            .expect("failed to curl");
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.len() > 700); // ballpark check

        text.split("\n")
            .filter(|x| !(x.starts_with("#") || x.is_empty()))
            .map(String::from)
            .collect()
    }

    macro_rules! enc {
        ($s:expr, $e:expr) => {
            _encode($s.as_bytes(), $e.parse().unwrap())
        };
    }

    macro_rules! dec {
        ($s:expr, $e:expr) => {
            String::from_utf8(_decode($s, $e.parse().unwrap()).unwrap()).unwrap()
        };
    }

    #[test]
    fn test_empty_inputs() {
        for c in _get_enc() {
            let c = c.name;
            assert_eq!("", dec!("", c), "Failed with encoding {}", c);
            assert_eq!("", enc!("", c), "Failed with encoding {}", c);
        }
    }

    #[test]
    fn test_base64_padding() {
        let ref_dec = "Hello world";
        let ref_enc = "SGVsbG8gd29ybGQ==";
        assert_eq!(ref_dec, dec!(ref_enc, "base64"));
    }

    #[test]
    fn test_reflexivity_blns() {
        for c in _get_enc() {
            let c = c.name;
            for l in get_blns() {
                let e = enc!(l, c);
                assert_eq!(l, dec!(&e, c), "Failed with encoding `{}`", c);
            }
        }
    }

    #[test]
    fn test_reflexivity_bytespace() {
        let ref_dec = (0..255).chain(255..=0).collect::<Vec<u8>>();
        for c in _get_enc() {
            let enc = _encode(&ref_dec, c);
            let dec = _decode(&enc, c).unwrap();
            assert_eq!(ref_dec, dec, "Failed with encoding `{}`", c.name);
        }
    }
}
