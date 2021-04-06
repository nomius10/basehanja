use crate::repack::uVar;

static HIRAGANA: &'static str = "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんがぎぐげござじずぜぞだぢづでどばびぶべぼ";
static KATAKANA: &'static str = "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲンガギグゲゴザジズゼゾダヂヅデドバビブベボ";

pub struct Encoding<'a> {
    pub id: &'a str,
    pub description: &'a str,
    pub char_table: Vec<char>,
    pub escape_char: char,
}

lazy_static! {
    static ref ENCODINGS: Vec<Encoding<'static>> = vec![
        ("base64",   '=', "Base64", ('A'..='Z').chain('a'..='z').chain('0'..='9').chain("+/".chars()).collect()),
        ("kanji",    '々', "Kanji (漢字)", ('㐀'..'㿿').collect()), // seiai.ed.jp/sys/text/java/utf8table.html
        ("hiragana", 'ゐ', "Hiragana (ひらがな)" , HIRAGANA.chars().collect()),
        ("katakana", 'ヰ', "Katakana (かたかな)" , KATAKANA.chars().collect()),
    ]
    .into_iter()
    .map(|x| (Encoding { id: x.0, escape_char:x.1, description: x.2, char_table:x.3 }))
    .collect();
}

pub fn get_encodings<'a>() -> Vec<&'a Encoding<'static>> {
    ENCODINGS.iter().collect()
}

impl std::str::FromStr for &Encoding<'_> {
    type Err = String;

    fn from_str(key: &str) -> Result<Self, Self::Err> {
        let key = key.trim().to_lowercase().to_owned();
        ENCODINGS
            .iter()
            .find(|x| x.id == key)
            .ok_or(format!("invalid encoding: {}", key))
    }
}

impl Encoding<'_> {
    pub fn bitcount(&self) -> u8 {
        let l = self.char_table.len();
        let mut i = 0;
        // 00011010
        while 1 << (i + 1) <= l {
            i += 1;
        }
        i
    }

    pub fn enc_fn<'a>(&'a self) -> impl Fn(uVar) -> char + 'a {
        move |x| self.char_table[x as usize]
    }

    pub fn dec_fn(&self) -> impl Fn(char) -> uVar + '_ {
        move |c| self.char_table.iter().position(|&x| x == c).unwrap() as uVar
    }

    pub fn validate(&self, s: &str) -> Result<(), (usize, char)> {
        match s
            .chars()
            .enumerate()
            .find(|(_, x)| !self.char_table.contains(x))
        {
            Some(tup) => Err(tup),
            None => Ok(()),
        }
    }
}
