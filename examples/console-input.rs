fn main() {
    let buf = std::fs::read_to_string("examples/input_hex.txt")
        .expect("read file");
    let mut ins = Vec::new();
    let mut iter = buf.chars()
        .filter_map(|ch| ch.to_digit(16).map(|d| d as u8))
        .peekable();
    while let Some(a) = iter.next() {
        if let Some(b) = iter.next() {
            ins.push((a << 4) + b)
        }
    }
    rvda::dump(std::io::Cursor::new(ins))
        .expect("");
}
