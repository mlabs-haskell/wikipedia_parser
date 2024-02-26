// We operate on u8 instead of chars to avoid the overhead of decoding UTF8.
// This works because UTF8 guarantees that the multibyte UTF8 sequences won't contain any ASCII
// characters. See the Backwards Compatibility section here:
// https://en.wikipedia.org/wiki/UTF-8#Comparison_with_other_encodings
pub fn extract_links(input: &[u8]) -> String {
    let mut buffer = Vec::new();

    let mut chunks = input.windows(2);
    'outer: loop {
        let chunk = match chunks.next() {
            Some(x) => x,
            None => break 'outer,
        };

        if chunk[0] == ('[' as u8) && chunk[1] == ('[' as u8) {
            loop {
                let chunk = match chunks.next() {
                    Some(x) => x,
                    None => break 'outer,
                };
                buffer.push(chunk[0]);
                if chunk[0] == (']' as u8) && chunk[1] == (']' as u8) {
                    break;
                }
            }
        }
    }

    String::from_utf8_lossy(&buffer).to_string()
}
