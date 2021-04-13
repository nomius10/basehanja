use crate::repack::{uVar, RepackIterator};

mod config;
use config::ENCODINGS;

pub struct Encoding {
    pub name: &'static str,
    pub long_name: &'static str,
    char_space: CharSpace,
    pad_char: PadType,
}

enum CharSpace {
    Concrete(&'static str),
    Intervals(&'static [(char, char)]),
}

enum PadType {
    /// "padding" is the same as in base64
    BlockPad(char),
    /// "padding" signifies how many chars to drop when decoding
    DropPad(char),
}

use CharSpace::*;
use PadType::*;

// taboo AFAIK; I'm doing it just because I can
impl std::ops::Deref for PadType {
    type Target = char;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::BlockPad(c) => c,
            Self::DropPad(c) => c,
        }
    }
}

macro_rules! cp_len {
    ($a:expr, $b:expr) => {
        ($b as usize) - ($a as usize) + 1
    };
}

macro_rules! div_ceil {
    ($div:expr, $by:expr) => {{
        let mut a = $div;
        while a % $by != 0 {
            a += 1;
        }
        a / $by
    }};
}

impl CharSpace {
    fn idx_to_char(&self, u: uVar) -> char {
        match self {
            Concrete(s) => s.chars().nth(u as usize).unwrap(),
            Intervals(arr) => {
                let mut u = u as usize;
                for (a, b) in *arr {
                    let crt_len = cp_len!(*a, *b);
                    if u < crt_len {
                        return (*a..=*b).nth(u).expect(&format!("is {}", u));
                    } else {
                        u -= crt_len;
                    }
                }
                panic!();
            }
        }
    }

    fn char_to_idx(&self, c: char) -> Result<uVar, String> {
        match self {
            Concrete(s) => s
                .chars()
                .position(|x| x == c)
                .map(|x| x as uVar)
                .ok_or(format!("Invalid char {}", c)),
            Intervals(arr) => {
                let mut crt_idx = 0;
                for (a, b) in *arr {
                    if (*a..=*b).contains(&c) {
                        return Ok((crt_idx + (*a..=*b).position(|x| x == c).unwrap()) as uVar);
                    } else {
                        crt_idx += cp_len!(*a, *b);
                    }
                }
                Err(format!("Invalid char {}", c))
            }
        }
    }

    fn num_chars(&self) -> usize {
        match self {
            Concrete(s) => s.chars().count(),
            Intervals(arr) => arr.iter().map(|(a, b)| cp_len!(*a, *b)).sum(),
        }
    }
}

impl std::str::FromStr for &Encoding {
    type Err = String;

    fn from_str(key: &str) -> Result<Self, Self::Err> {
        let key = key.trim().to_lowercase().to_owned();
        ENCODINGS
            .iter()
            .find(|x| x.name == key)
            .ok_or(format!("invalid encoding: {}", key))
    }
}

impl Encoding {
    pub fn encode(&self, bytes: &[u8]) -> String {
        let it = bytes.iter().map(|&x| x as uVar);
        let it = RepackIterator::new(it, 8, self.bitcount());
        let mut s = it
            .map(|x| self.char_space.idx_to_char(x))
            .collect::<String>();
        // Append padding chars
        let pad_len = self.get_pad_len(bytes);
        s += &self.pad_char.to_string().repeat(pad_len);
        s
    }

    pub fn decode(&self, text: &str) -> Result<Vec<u8>, String> {
        let mut acc = vec![];
        for txt in self.deconcat(text) {
            acc.append(&mut self.decode_single(txt)?);
        }
        Ok(acc)
    }

    pub fn bitcount(&self) -> u8 {
        let l = self.char_space.num_chars();
        let mut i = 0;
        // 00011010
        loop {
            if 1 << i + 1 > l {
                break i;
            }
            i += 1;
        }
    }

    /// How many padding chars should be added to the encoding?
    fn get_pad_len(&self, arr: &[u8]) -> usize {
        let ebc = self.bitcount() as usize;
        let nbytes = arr.len();
        if nbytes == 0 {
            return 0;
        }
        match self.pad_char {
            BlockPad(_) => {
                let block_len = lcm(ebc, 8);
                let last_bits = (nbytes * 8) % block_len;
                if last_bits == 0 {
                    return 0;
                }
                (block_len - last_bits) / ebc
            }
            DropPad(_) => {
                let inp_bitlen = nbytes * 8;
                let out_bitlen = div_ceil!(inp_bitlen, ebc) * ebc;
                let extra = out_bitlen - inp_bitlen;
                div_ceil!(extra, 8)
            }
        }
    }

    /// Separate a concatenated encoding into its individual parts
    fn deconcat<'a>(&self, text: &'a str) -> Vec<&'a str> {
        let mut acc = vec![];
        let mut prev_i = 0;
        let mut met = false;
        for (i, c) in text.char_indices() {
            if met == false && c == *self.pad_char {
                met = true;
            }
            if met == true && c != *self.pad_char {
                acc.push(text.get(prev_i..i).unwrap());
                met = false;
                prev_i = i;
            }
        }
        acc.push(text.get(prev_i..).unwrap());
        acc
    }

    /// Decode a non-concatenated string
    fn decode_single(&self, text: &str) -> Result<Vec<u8>, String> {
        let unpadded = text.trim_end_matches(*self.pad_char);

        // decode to array of bytes
        let mut err = Ok(vec![0]);
        let it = unpadded
            .chars()
            .enumerate()
            .map(|(i, x)| (i, self.char_space.char_to_idx(x)))
            .scan(0, |_, (i, x)| match x {
                Ok(o) => Some(o),
                Err(e) => {
                    err = Err(format!("Error: At char #{}: {}", i, e));
                    None
                }
            });
        let mut arr = RepackIterator::new(it, self.bitcount(), 8)
            .map(|x| x as u8)
            .collect::<Vec<u8>>();
        err?;

        // drop extra bytes resulted from the decoding, if any
        let drop_count = match self.pad_char {
            BlockPad(_) => {
                let block_char_size = lcm(8, self.bitcount() as usize) / self.bitcount() as usize;
                std::cmp::min(1, unpadded.chars().count() % block_char_size)
            }
            DropPad(_) => text.chars().count() - unpadded.chars().count(),
        };
        arr.resize(arr.len() - drop_count, 0);

        Ok(arr)
    }
}

/// smallest common multiple
fn lcm(a: usize, b: usize) -> usize {
    let mut crt = std::cmp::max(a, b);
    loop {
        if crt % a == 0 && crt % b == 0 {
            break crt;
        }
        crt += 1;
    }
}

pub fn get_encodings<'a>() -> &'static [Encoding] {
    ENCODINGS
}

#[cfg(test)]
mod tests {
    use super::Encoding;

    #[test]
    fn test_bitcounts() {
        let counts = &[
            ("base64", 6),
            ("hiragana", 6),
            ("kanji", 16),
            ("binary", 1),
            ("hex", 4),
        ];

        for (k, v) in counts {
            let enc = k.parse::<&Encoding>().unwrap();
            assert_eq!(*v, enc.bitcount());
        }
    }

    #[test]
    fn test_block_pad() {
        let pairs = &[
            ("a" /*        */, "YQ=="),
            ("aa" /*       */, "YWE="),
            ("aaa" /*      */, "YWFh"),
            ("aaaa" /*     */, "YWFhYQ=="),
            ("aaaaa" /*    */, "YWFhYWE="),
            ("aaaaaa" /*   */, "YWFhYWFh"),
            ("aaaaaaa" /*  */, "YWFhYWFhYQ=="),
            ("aaaaaaaa" /* */, "YWFhYWFhYWE="),
            ("aaaaaaaaa" /**/, "YWFhYWFhYWFh"),
        ];
        let codec = "base64".parse::<&Encoding>().unwrap();
        for (dec, enc) in pairs {
            assert_eq!(*enc, codec.encode(dec.as_bytes()), "Encoding mismatch");
            let res = codec.decode(enc).unwrap();
            assert_eq!(dec.as_bytes(), res, "Decoding mismatch");
        }
    }

    #[test]
    fn test_drop_pad() {
        let pairs = &[
            ("a" /*     */, "𠕊々"),
            ("aa" /*    */, "𠖫"),
            ("aaa" /*   */, "𠖫𠕊々"),
            ("aaaa" /*  */, "𠖫𠖫"),
            ("aaaaa" /* */, "𠖫𠖫𠕊々"),
            ("aaaaaa" /**/, "𠖫𠖫𠖫"),
        ];
        let codec = "kanji".parse::<&Encoding>().unwrap();
        for (dec, enc) in pairs {
            assert_eq!(*enc, codec.encode(dec.as_bytes()), "Encoding mismatch");
            let res = codec.decode(enc).unwrap();
            assert_eq!(dec.as_bytes(), res, "Decoding mismatch");
        }
    }
}
