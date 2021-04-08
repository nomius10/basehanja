use crate::repack::{uVar, RepackIterator};

pub struct Encoding {
    pub name: &'static str,
    pub long_name: &'static str,
    pub padding_char: char,
    char_space: CharSpace,
}

enum CharSpace {
    Concrete(&'static str),
    Intervals(&'static [(char, char)]),
}

static ENCODINGS: &[Encoding] = &[
    Encoding {
        name: "base64",
        long_name: "Base64",
        char_space: CharSpace::Intervals(&[
            ('A', 'Z'),
            ('a', 'z'),
            ('0', '9'),
            ('+', '+'),
            ('/', '/'),
        ]),
        padding_char: '=',
    },
    Encoding {
        name: "hiragana",
        long_name: "Hiragana (ひらがな)",
        char_space: CharSpace::Concrete(
            // ordering mostly follows https://www.youtube.com/watch?v=lrMkJAzbWQc
            "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみ\
            むめもやゆよらりるれろわをんがぎぐげござじずぜぞだぢづでどばびぶ",
        ),
        padding_char: 'ゐ',
    },
    Encoding {
        name: "katakana",
        long_name: "Katakana (かたかな)",
        char_space: CharSpace::Concrete(
            "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミ\
            ムメモヤユヨラリルレロワヲンガギグゲゴザジズゼゾダヂヅデドバビブ",
        ),
        padding_char: 'ヰ',
    },
    Encoding {
        name: "kanji",
        long_name: "Hanzi+Kanji+Hanja (漢字)",
        char_space: CharSpace::Intervals(&[
            ('\u{04e00}', '\u{09fff}'), // 20_992 chars; https://en.wikipedia.org/wiki/CJK_Unified_Ideographs_(Unicode_block)
            ('\u{03400}', '\u{03DB5}'), //  6_592 chars; https://en.wikipedia.org/wiki/CJK_Unified_Ideographs_Extension_A
            ('\u{20000}', '\u{2a6df}'), // 42_720 chars; https://en.wikipedia.org/wiki/CJK_Unified_Ideographs_Extension_B
        ]),
        padding_char: '々',
    },
];

macro_rules! cp_len {
    ($a:expr, $b:expr) => {
        ($b as usize) - ($a as usize) + 1
    };
}

use CharSpace::*;
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

    fn size(&self) -> usize {
        match self {
            Concrete(s) => s.chars().count(),
            Intervals(arr) => arr.iter().map(|(a, b)| cp_len!(*a, *b)).sum(),
        }
    }

    pub fn bitcount(&self) -> u8 {
        let l = self.size();
        let mut i = 0;
        // 00011010
        loop {
            if 1 << i + 1 > l {
                break i;
            }
            i += 1;
        }
    }
}

pub fn get_encodings<'a>() -> &'static [Encoding] {
    ENCODINGS
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
    pub fn bitcount(&self) -> u8 {
        self.char_space.bitcount()
    }

    pub fn encode(&self, bytes: &[u8]) -> String {
        let it = bytes.iter().map(|&x| x as uVar);
        let it = RepackIterator::new(it, 8, self.bitcount(), false);
        let mut s = it
            .map(|x| self.char_space.idx_to_char(x))
            .collect::<String>();
        // Append padding chars
        let pad_len = self.get_pad_len(bytes.len());
        s += &self.padding_char.to_string().repeat(pad_len);
        s
    }

    pub fn decode(&self, text: &str) -> Result<Vec<u8>, String> {
        let mut acc = vec![];
        for txt in self.deconcat(text) {
            acc.append(&mut self._decode(txt)?);
        }
        Ok(acc)
    }

    fn gcd(a: usize, b: usize) -> usize {
        let mut x = std::cmp::max(a, b);
        let mut y = std::cmp::min(a, b);
        loop {
            let r = x % y;
            if r == 0 {
                return y;
            }
            x = y;
            y = r;
        }
    }

    // for bitcounts larger than 8, the padding will determine the num of chars to drop
    fn get_pad_len(&self, l: usize) -> usize {
        let bc = self.bitcount() as usize;
        if bc <= 8 {
            if l == 0 {
                return 0;
            }
            let gcd = Self::gcd(bc, 8);
            return gcd - (l % gcd);
        }
        let mut i = l * 8;
        while i % bc != 0 {
            i += 1;
        }
        i = i - l * 8;
        i / 8
    }

    fn deconcat<'a>(&self, text: &'a str) -> Vec<&'a str> {
        let mut acc = vec![];
        let mut prev_i = 0;
        let mut met = false;
        for (i, c) in text.char_indices() {
            if met == false && c == self.padding_char {
                met = true;
            }
            if met == true && c != self.padding_char {
                acc.push(text.get(prev_i..i).unwrap());
                met = false;
                prev_i = i;
            }
        }
        acc.push(text.get(prev_i..).unwrap());
        acc
    }

    fn _decode(&self, text: &str) -> Result<Vec<u8>, String> {
        let unpadded = text.trim_end_matches(self.padding_char);
        let mut err = Ok(vec![0]);
        let it = unpadded
            .chars()
            .enumerate()
            .map(|(i, x)| (i, self.char_space.char_to_idx(x)))
            .scan(0, |_, (i, x)| match x {
                Ok(o) => Some(o),
                Err(e) => {
                    err = Err(format!("At chrar #{}: {}", i, e));
                    None
                }
            });
        let mut arr = RepackIterator::new(it, self.bitcount(), 8, true)
            .map(|x| x as u8)
            .collect::<Vec<u8>>();
        err?;

        if self.bitcount() > 8 {
            let drop_count = text.chars().count() - unpadded.chars().count();
            arr.resize(arr.len() - drop_count, 0);
        }
        Ok(arr)
    }
}

#[cfg(test)]
mod tests {
    use super::Encoding;

    static COUNTS: &[(&str, u8)] = &[
        ("base64", 6),
        ("katakana", 6),
        ("hiragana", 6),
        ("kanji", 16),
    ];

    #[test]
    fn test_bitcounts() {
        for (k, v) in COUNTS {
            let enc = k.parse::<&Encoding>().unwrap();
            assert_eq!(*v, enc.bitcount());
        }
    }
}
