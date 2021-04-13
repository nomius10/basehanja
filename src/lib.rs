use wasm_bindgen::prelude::*;

mod encoding;
mod repack;
use encoding::Encoding;

#[wasm_bindgen(catch)]
pub fn decode(text: &str, charset: &str) -> Result<Box<[u8]>, JsValue> {
    let dec = charset.parse::<&Encoding>()?.decode(text)?;
    Ok(dec.into_boxed_slice())
}

#[wasm_bindgen(catch)]
pub fn decode_utf8(text: &str, charset: &str) -> Result<String, JsValue> {
    let dec = charset.parse::<&Encoding>()?.decode(text)?;
    // or_else() is, comapred to or(), lazy. This permits calling this function from the examples, albeit
    // in a hacky manner
    String::from_utf8(dec).or_else(|_| Err(JsValue::from_str("Invalid UTF-8 encoding")))
}

#[wasm_bindgen(catch)]
pub fn encode(text: Box<[u8]>, charset: &str) -> Result<String, JsValue> {
    Ok(charset.parse::<&Encoding>()?.encode(&text))
}

#[wasm_bindgen(catch)]
pub fn encode_utf8(text: &str, charset: &str) -> Result<String, JsValue> {
    Ok(charset.parse::<&Encoding>()?.encode(text.as_bytes()))
}

/// Helper struct that will be returned to JS
#[derive(serde::Serialize)]
struct EncodingDescription {
    name: String,
    description: String,
    bitcount: u8,
}

/// wasm-side: Return an array of objects
#[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
#[wasm_bindgen]
pub fn get_encodings() -> Box<[JsValue]> {
    encoding::get_encodings()
        .iter()
        .map(|x| EncodingDescription {
            name: x.name.to_owned(),
            description: x.long_name.to_owned(),
            bitcount: x.bitcount(),
        })
        .map(|x| JsValue::from_serde(&x).unwrap())
        .collect::<Vec<JsValue>>()
        .into_boxed_slice()
}

/// PC side: Return just a vector of names
#[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
pub fn get_encodings() -> Vec<&'static str> {
    encoding::get_encodings()
        .iter()
        .map(|x| x.name)
        .collect::<Vec<&str>>()
}

#[cfg(test)]
mod tests {
    use crate::encoding::get_encodings as _get_enc;

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

        text.lines()
            .filter(|x| !(x.starts_with("#") || x.is_empty()))
            .map(String::from)
            .collect()
    }

    macro_rules! enc {
        ($s:expr, $e:expr) => {
            $e.encode($s.as_bytes())
        };
    }

    macro_rules! dec {
        ($s:expr, $e:expr) => {
            String::from_utf8($e.decode($s).unwrap()).unwrap()
        };
    }

    #[test]
    fn test_empty_inputs() {
        for c in _get_enc() {
            assert_eq!("", dec!("", c), "Failed with encoding {}", c.name);
            assert_eq!("", enc!("", c), "Failed with encoding {}", c.name);
        }
    }

    #[test]
    fn test_reflexivity_blns() {
        for c in _get_enc() {
            for l in get_blns() {
                let e = enc!(l, c);
                assert_eq!(l, dec!(&e, c), "Failed with encoding `{}`", c.name);
            }
        }
    }

    #[test]
    fn test_reflexivity_bytespace() {
        let ref_dec = (0..255).chain(255..=0).collect::<Vec<u8>>();
        for c in _get_enc() {
            let enc = c.encode(&ref_dec);
            let dec = c.decode(&enc).unwrap();
            assert_eq!(ref_dec, dec, "Failed with encoding `{}`", c.name);
        }
    }

    #[test]
    fn test_reflexivity_concatenated() {
        for i in 0..30 {
            let ref_dec = "a".repeat(i);
            for c in _get_enc() {
                let enc = enc!(ref_dec, c);
                let enc = enc.repeat(3);
                let dec = dec!(&enc, c);
                assert_eq!(ref_dec.repeat(3), dec, "Failed with encoding `{}`", c.name);
            }
        }
    }
}
