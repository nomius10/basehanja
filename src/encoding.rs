use crate::repack::{uVar, RepackIterator};

pub struct Encoding {
    pub name: &'static str,
    pub long_name: &'static str,
    pub padding_char: char,
    char_space: CharSpace,
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

enum CharSpace {
    Concrete(&'static str),
    Intervals(&'static [(char, char)]),
}

macro_rules! cp_len {
    ($a:expr, $b:expr) => {
        ($b as usize) - ($a as usize) + 1
    };
}

use CharSpace::*;
impl CharSpace {
    // this is taxing
    /*
    fn char_table(self) -> Vec<char> {
        match self {
            Concrete(s) => s.chars().collect(),
            Intervals(arr) => arr.iter().fold(vec![], |mut acc, (a, b)| {
                acc.extend_from_slice((*a..=*b).collect::<Vec<char>>().as_slice());
                acc
            }),
        }
    }*/

    fn idx_to_char(&self, u: uVar) -> char {
        match self {
            Concrete(s) => s.chars().nth(u as usize).expect(&format!("is {} {}", u, self.size())),
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
        let l = self.char_space.size();
        let mut i = 0;
        // 00011010
        while 1 << (i + 1) <= l {
            i += 1;
        }
        i
    }

    pub fn encode(&self, bytes: &[u8]) -> String {
        let it = bytes.iter().map(|&x| x as uVar);
        let it = RepackIterator::new(it, 8, self.bitcount(), false);
        it.map(|x| self.char_space.idx_to_char(x)).collect()
    }

    pub fn decode(&self, text: &str) -> Result<Vec<u8>, String> {
        let mut err = Ok(vec![0]);
        let mut idx = 0;
        let it = text
            .trim_end_matches(self.padding_char)
            .chars()
            .map(|x| self.char_space.char_to_idx(x))
            .scan(0, |_, x| match x {
                Ok(o) => {
                    idx += 1;
                    Some(o)
                }
                Err(e) => {
                    err = Err(format!("At idx {}: {}", idx, e));
                    None
                }
            });
        let it = RepackIterator::new(it, self.bitcount(), 8, true)
            .map(|x| x as u8)
            .collect();
        err?;
        Ok(it)
    }
}

#[cfg(test)]
mod tests {
    use super::Encoding;

    static COUNTS : &[(&str, u8)] = &[
        ("base64", 6),
        ("katakana", 6),
        ("hiragana", 6),
        ("kanji", 16)
    ];

    #[test]
    fn test_bitcounts() {
        for (k, v) in COUNTS {
            let enc = k.parse::<&Encoding>().unwrap();
            assert_eq!(enc.bitcount(), *v);
        }
    }
}
