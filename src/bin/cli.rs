/*
use basehanja::CharSet;

fn main() {
    println!("Hi {:?}", basehanja::CharSet::Kanji)
}


struct XetsIterator<'a> {
    rstr: &'a [u8],
    rbits: u8,
    osize: u8,
}

impl Iterator for XetsIterator<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        fn take(u: u8, from: u8, to: u8) -> u8 {
            // 01234567
            // __XXXXX_ -> ___XXXXX
            // 76543210
            let from = 8 - from;
            let to = 8 - to;
            let mask = ((1 << from) - 1) - ((1 << to) - 1);
            (u & mask) >> to
        }

        if self.rstr.is_empty() && self.rbits == 0 {
            return None
        }

        let mut acc: u16 = 0;
        let mut r = self.osize;
        let mut munch = 0;
        // 01234567 01234567 01234567
        // _____XXX XXXXXXXX XXXX____
        acc <<= self.rbits;
        acc |= take(self.rstr[0], 8 - self.rbits, 8) as u16;
        r -= self.rbits; self.rbits = 0;

        while r > 0 {
            self.rstr = &self.rstr[1..];

            munch = std::cmp::min(8, r);
            acc <<= munch;
            acc |= take(self.rstr[0], 0, munch) as u16;
            r -= munch;
        }

        self.rbits = 8 - munch;

        Some(acc)
    }
}
*/
fn main() {}