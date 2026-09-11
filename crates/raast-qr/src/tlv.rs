pub struct RawTlv<'a> {
    pub tag: &'a str,
    pub length: usize,
    pub value: &'a str,
}
