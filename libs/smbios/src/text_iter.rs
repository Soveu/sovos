/// Iterator over SMBIOS text strings.
/// Returns a byte slice without null terminator.
///
/// Note that this iterator is not fused, meaning that if it is used
/// after hitting double-null terminator and returning None, it can
/// return "garbage" slice
pub struct TextIterator<'a> {
    pub slice: &'a [u8],
}

impl<'a> Iterator for TextIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let n = self
            .slice
            .iter()
            .position(|&x| x == 0)
            .expect("smbios::TextIterator - can't find null terminator");

        /* SAFETY: n must be in range [0, slice.len()) */
        let (text, rest) = unsafe {
            (
                self.slice.get_unchecked(..n),
                self.slice.get_unchecked(n + 1..),
            )
        };

        if text.len() == 0 {
            return None;
        }

        self.slice = rest;
        return Some(text);
    }
}
