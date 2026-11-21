use super::*;

#[test]
fn every_anchor_matches_including_the_block_file_length_and_the_value_byte_totals() {
    let lines = network_anchor_lines();
    for (identifier, computed, registered, matches) in &lines {
        assert!(matches, "锚点 {identifier} 算出 {computed}，登记是 {registered}");
    }
    assert_eq!(lines.len(), 20, "送块那边 18 项，加 A11_file、A10");
}
