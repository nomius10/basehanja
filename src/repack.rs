/// Internal "container" type for variable-length uint
#[allow(non_camel_case_types)]
pub type uVar = u32;

/// Iterator that repacks bits from ux to uy. u16 is used to represent input and output values.
///
/// E.g: from u8->u6 (or vice versa)
///
/// `11111100 00001111` -> `111111 000000 111100`
pub struct RepackIterator<T: Iterator> {
    iband: std::iter::Peekable<T>,
    cbits: u8,
    isize: u8,
    osize: u8,
}

impl<T: Iterator<Item = uVar>> RepackIterator<T> {
    pub fn new<F>(iband: F, isize: u8, osize: u8) -> RepackIterator<T>
    where
        F: IntoIterator<Item = uVar, IntoIter = T>,
    {
        RepackIterator {
            iband: iband.into_iter().peekable(),
            cbits: 0,
            isize: isize,
            osize: osize,
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
                return Some(acc << (self.osize - aln));
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
