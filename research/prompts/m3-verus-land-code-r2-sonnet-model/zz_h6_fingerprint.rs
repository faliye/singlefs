use singlefs_harness::sha256::sha256_hexadecimal;
fn input_fingerprint_of(domain_text: &str, size_text: &str, state_row: &str) -> Option<String> {
    let (_row_before_the_state, state_text) = state_row.rsplit_once(" state=")?;
    if state_text.is_empty() || state_text.chars().any(char::is_whitespace) {
        return None;
    }
    let state_before_the_outcome = state_text
        .split_once(";first=")
        .map_or(state_text, |(before_the_outcome, _outcome)| {
            before_the_outcome
        });
    Some(sha256_hexadecimal(
        format!("{domain_text}|{size_text}|{state_before_the_outcome}").as_bytes(),
    ))
}

#[test]
fn print_fingerprints() {
    let input = std::fs::read_to_string("/tmp/claude-1000/m3-verus-land-code-r2-sonnet/h6/rows.tsv").expect("rows");
    let mut out = String::new();
    for (index, line) in input.split('\n').enumerate() {
        if line.is_empty() { continue; }
        let mut parts = line.splitn(3, '\t');
        let (domain, size, row) = (parts.next().unwrap(), parts.next().unwrap(), parts.next().unwrap_or(""));
        let row = row.replace("\\t", "\t").replace("\\r", "\r");
        out.push_str(&format!("{index}\t{}\n", input_fingerprint_of(domain, size, &row).unwrap_or_else(|| "None".to_string())));
    }
    std::fs::write("/tmp/claude-1000/m3-verus-land-code-r2-sonnet/h6/rust_out.tsv", out).expect("write");
}
