use super::{CharSpace, Encoding, PadType};

use PadType::*;

pub static ENCODINGS: &[Encoding] = &[
    Encoding {
        name: "binary",
        long_name: "Binary",
        char_space: CharSpace::Concrete("01"),
        pad_char: BlockPad('?'), // it's not going to be used...
    },
    Encoding {
        name: "hex",
        long_name: "Hexadecimal",
        char_space: CharSpace::Intervals(&[('0', '9'), ('A', 'F')]),
        pad_char: BlockPad('?'), // it's not going to be used...
    },
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
        pad_char: BlockPad('='),
    },
    Encoding {
        name: "hiragana",
        long_name: "Hiragana (ひらがな)",
        char_space: CharSpace::Concrete(
            // ordering mostly follows https://www.youtube.com/watch?v=lrMkJAzbWQc
            "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみ\
            むめもやゆよらりるれろわをんがぎぐげござじずぜぞだぢづでどばびぶ",
        ),
        pad_char: BlockPad('ゐ'),
    },
    Encoding {
        name: "katakana",
        long_name: "Katakana (かたかな)",
        char_space: CharSpace::Concrete(
            "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミ\
            ムメモヤユヨラリルレロワヲンガギグゲゴザジズゼゾダヂヅデドバビブ",
        ),
        pad_char: BlockPad('ヰ'),
    },
    Encoding {
        name: "hangul",
        long_name: "Hangul (한글) (13-bit)",
        char_space: CharSpace::Intervals(&[
            ('\u{AC00}', '\u{D74f}'), // 11_088 chars
        ]),
        pad_char: DropPad('흐'),
    },
    Encoding {
        name: "kanji",
        long_name: "Hanzi+Kanji+Hanja (漢字)",
        char_space: CharSpace::Intervals(&[
            ('\u{04e00}', '\u{09fff}'), // 20_992 chars; https://en.wikipedia.org/wiki/CJK_Unified_Ideographs_(Unicode_block)
            ('\u{03400}', '\u{03DB5}'), //  6_592 chars; https://en.wikipedia.org/wiki/CJK_Unified_Ideographs_Extension_A
            ('\u{20000}', '\u{2a6df}'), // 42_720 chars; https://en.wikipedia.org/wiki/CJK_Unified_Ideographs_Extension_B
        ]),
        pad_char: DropPad('々'),
    },
];
