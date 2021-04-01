use wasm_bindgen::prelude::*;

static HIRAGANA: &'static str = "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんがぎぐげござじずぜぞだぢづでどばびぶべぼ";
static KATAKANA: &'static str = "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲンガギグゲゴザジズゼゾダヂヅデドバビブベボ";

#[wasm_bindgen]
pub fn decode(text: &str, charset: &str) -> String {
    let charset = match CharSet::parse(charset) {
        Some(c) => c,
        _ => return "Error: bad charset".to_owned(),
    };

    match _decode(text, charset) {
        Ok(res) => std::str::from_utf8(&res[..]).unwrap().to_string(),
        Err(e) => e.to_owned(),
    }
}

#[wasm_bindgen]
pub fn encode(text: &str, charset: &str) -> String {
    let charset = match CharSet::parse(charset) {
        Some(c) => c,
        _ => return "Error: bad charset".to_owned(),
    };

    _encode(text.as_bytes(), charset)
}

#[derive(Copy, Clone)]
enum CharSet {
    Base64,
    Kanji,
    Hiragana,
    Katakana,
}

impl CharSet {
    fn char_table(self) -> Vec<char> {
        match self {
            Self::Base64 => ('A'..='Z')
                .chain('a'..='z')
                .chain('0'..='9')
                .chain("+/".chars())
                .collect(),
            Self::Kanji => ('㐀'..'㿿').collect(), // seiai.ed.jp/sys/text/java/utf8table.html
            Self::Hiragana => HIRAGANA.chars().collect(),
            Self::Katakana => KATAKANA.chars().collect(),
        }
    }

    fn bitcount(self) -> u8 {
        let l = self.char_table().len();
        let mut i = 0;
        // 00011010
        while 1 << (i + 1) <= l {
            i += 1;
        }
        i
    }

    fn parse(s: &str) -> Option<CharSet> {
        match s.trim().to_lowercase().as_str() {
            "hanja" | "hanzi" | "kanji" => Some(Self::Kanji),
            "hiragana" => Some(Self::Hiragana),
            "katakana" => Some(Self::Katakana),
            "base64" => Some(Self::Base64),
            _ => None,
        }
    }
}

/// "Container" type for variable-length uint
#[allow(non_camel_case_types)]
type uVar = u16;

/// Iterator that repacks bits from ux to uy. u16 is used to represent input and output values.
///
/// E.g: from u8->u6 (or vice versa)
///
///     11111111 00000000 11111111
///
///     111111 110000 000011 111111
struct RepackIterator<T: Iterator> {
    iband: std::iter::Peekable<T>,
    cbits: u8,
    isize: u8,
    osize: u8,
    discard: bool,
}

impl<T: Iterator<Item = uVar>> RepackIterator<T> {
    fn new(iband: T, isize: u8, osize: u8, discard: bool) -> RepackIterator<T> {
        RepackIterator {
            iband: iband.peekable(),
            cbits: 0,
            isize: isize,
            osize: osize,
            discard: discard,
        }
    }
}

fn take_bits(n: uVar, nsize: u8, count: u8, skip: u8) -> uVar {
    // 01234567
    // __XXXXX_ -> ___XXXXX
    // 76543210
    let from = nsize - skip;
    let to = nsize - skip - count;
    let mask = ((1 << from) - 1) - ((1 << to) - 1);
    (n & mask) >> to
}

impl<T: Iterator<Item = uVar>> Iterator for RepackIterator<T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<uVar> {
        if let None = self.iband.peek() {
            return None;
        }

        let mut acc: uVar = 0;
        let mut aln: u8 = 0;

        // 01234567 01234567 01234567
        // _____XXX XXXXXXXX XXXX____
        while aln < self.osize {
            if self.iband.peek().is_none() {
                return match self.discard {
                    true => None,
                    false => Some(acc << (self.osize - aln)),
                };
            }
            let crt_n = *self.iband.peek().unwrap();

            let munch = std::cmp::min(self.osize - aln, self.isize - self.cbits);
            acc <<= munch;
            acc |= take_bits(crt_n, self.isize, munch, self.cbits);
            self.cbits += munch;
            aln += munch;

            if self.cbits == self.isize {
                self.iband.next();
                self.cbits = 0;
            }
        }

        Some(acc)
    }
}

fn _encode(bytes: &[u8], encoding: CharSet) -> String {
    let chars_table = encoding.char_table();

    let it = bytes.iter().map(|&x| x as uVar);
    let it = RepackIterator::new(it, 8, encoding.bitcount(), false);

    it.map(|x| chars_table[x as usize]).collect()
}

fn _decode(text: &str, encoding: CharSet) -> Result<Vec<u8>, String> {
    let text = match encoding {
        CharSet::Base64 => text.trim_end_matches('='),
        _ => text
    };

    let chars_table = encoding.char_table();
    if let Some((i, c)) = text.chars().enumerate().find(|(_, x)| !chars_table.contains(x)) {
        return Err(format!("Error decoding: unknown character '{}' at position {}", c, i));
    }

    let rlookup = |c| chars_table.iter().position(|&x| x == c).unwrap();
    let it = text.chars().map(rlookup).map(|x| x as uVar);
    let it = RepackIterator::new(it, encoding.bitcount(), 8, true);

    Ok(it.map(|x| x as u8).collect())
}

#[cfg(test)]
mod tests {
    use crate::{CharSet, _decode, _encode};

    #[test]
    fn test_base64_empty_inputs() {
        let enc = _encode(&[], CharSet::Base64);
        assert_eq!("", enc);

        let dec = _decode("", CharSet::Base64).unwrap();
        assert!(dec.is_empty());
    }

    #[test]
    fn test_base64_sample_text() {
        let ref_dec = "Hello world";
        let ref_enc = "SGVsbG8gd29ybGQ";

        let enc = _encode(ref_dec.as_bytes(), CharSet::Base64);
        assert_eq!(ref_enc, enc);

        let dec = _decode(ref_enc, CharSet::Base64).unwrap();
        let dec = std::str::from_utf8(&dec[..]).unwrap();
        assert_eq!(ref_dec, dec);
    }

    #[test]
    fn test_base64_padding() {
        let ref_dec = "Hello world";
        let ref_enc = "SGVsbG8gd29ybGQ==";
        let dec = _decode(ref_enc, CharSet::Base64).unwrap();
        let dec = std::str::from_utf8(&dec[..]).unwrap();
        assert_eq!(ref_dec, dec);
    }

    #[test]
    fn test_base64_bytespace() {
        let ref_dec = (0..255).chain(255..=0).collect::<Vec<u8>>();

        let enc = _encode(&ref_dec, CharSet::Base64);
        let dec = _decode(&enc, CharSet::Base64).unwrap();
        assert_eq!(ref_dec, dec);
    }
}
