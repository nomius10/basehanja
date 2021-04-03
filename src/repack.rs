/// Internal "container" type for variable-length uint
#[allow(non_camel_case_types)]
type uVar = u16;

/// Iterator that repacks bits from ux to uy. u16 is used to represent input and output values.
///
/// E.g: from u8->u6 (or vice versa)
///
///     11111111 00000000 11111111
///
///     111111 110000 000011 111111
pub struct RepackIterator<T: Iterator> {
    iband: std::iter::Peekable<T>,
    cbits: u8,
    isize: u8,
    osize: u8,
    discard: bool,
}

impl<T: Iterator<Item = uVar>> RepackIterator<T> {
    pub fn new(iband: T, isize: u8, osize: u8, discard: bool) -> RepackIterator<T> {
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
