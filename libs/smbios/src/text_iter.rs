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

        let (text, rest) = self.slice.split_at(n);
        if text.is_empty() {
            return None;
        }

        self.slice = rest;
        return Some(text);
    }
}
