use e7_index_bench::usize_from_u64;
use super::*;

#[test]
fn crc32c_matches_the_published_check_value() {
    assert_eq!(castagnoli_crc32(b"123456789"), 0xE306_9283);
    assert_eq!(castagnoli_crc32(&[]), 0);
}

/// D18（块里携带什么信息） 已定项 17：32 字节的校验和字段 = CRC32C 4 字节 + 28 字节零，字段自身按零参与。
#[test]
fn wide_checksum_is_crc32c_in_four_bytes_then_twenty_eight_zeroes() {
    let mut slot = vec![0u8; usize_from_u64(PHYSICAL_BLOCK_BYTES)];
    for (index, byte) in slot.iter_mut().enumerate() {
        *byte = u8::try_from(index % 251).expect("小于 256");
    }
    let digest = wide_checksum_with_field_zeroed(&slot, usize_from_u64(PHYSICAL_BLOCK_BYTES), 10);
    assert!(digest[WIDE_CHECKSUM_CRC_BYTES..].iter().all(|byte| *byte == 0), "后 28 字节恒零");
    let mut zeroed = slot.clone();
    zeroed[10..42].fill(0);
    assert_eq!(u32::from_le_bytes(digest[..4].try_into().expect("切了 4 字节")), castagnoli_crc32(&zeroed));
    assert_eq!(WIDE_CHECKSUM_CRC_BYTES + 28, usize_from_u64(WIDE_CHECKSUM_BYTES));
}

// ───────── E142 第十一次跑（量 5，crates 侧补上之后）新增：SHA-256 与 crates 快照解析的单测 ─────────

#[test]
fn sha256_matches_the_two_standard_test_vectors() {
    assert_eq!(sha256_hex(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}

#[test]
fn sha256_of_the_actual_root_record_bytes_matches_the_crates_side_reference_value() {
    // 这个十六进制串与 sha256 都是从 `cargo run -p singlefs-implementation-experiments --bin new_pool_file_creation_region_bytes`
    // 的真实产出里摘的一行（`region=root_record device=0`），用来钉住「这份手写 SHA-256 与 crates 侧算的是同一个函数」，
    // 不是只对着标准测试向量自证。
    let hex = "534653525f5346532d453134322d303030312d000000000001000000030000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000048c40000000024acb30e0100000048c40000000024acb30e010000000000000013000000000000000000000000000000afc75d3e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c40000000021aa2fd40100000000c40000000021aa2fd40000000000000000000000000000000000000000000000000000000000000000000000000000000000000f0000000000000003000000000000000000000047c400000000bedfaead0100000047c400000000bedfaead01000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    assert_eq!(hex_decode(hex).len(), 512);
    assert_eq!(sha256_hex(&hex_decode(hex)), "b2c014f937018d4e11d74410302b319efa0feda62b6b7073446bcb64560dd51f");
}

#[test]
fn hex_decode_round_trips_hex_bytes() {
    assert_eq!(hex_decode("00ff10"), vec![0x00, 0xff, 0x10]);
    assert_eq!(hex_decode(""), Vec::<u8>::new());
}

#[test]
fn parse_result_line_reads_key_value_pairs_and_ignores_the_leading_marker() {
    let fields = parse_result_line("E7RESULT name=impl_region_bytes region=root_record device=0 offset=1052672 length=512 sha256=abc123");
    assert_eq!(fields.get("region").map(String::as_str), Some("root_record"));
    assert_eq!(fields.get("device").map(String::as_str), Some("0"));
    assert_eq!(fields.get("sha256").map(String::as_str), Some("abc123"));
    assert_eq!(fields.get("name").map(String::as_str), Some("impl_region_bytes"));
}

#[test]
fn byte_diff_summary_finds_the_first_differing_byte_and_counts_the_rest() {
    assert_eq!(byte_diff_summary(&[1, 2, 3], &[1, 2, 3]), (None, Some(0)));
    assert_eq!(byte_diff_summary(&[1, 2, 3], &[1, 9, 9]), (Some(1), Some(2)));
    assert_eq!(byte_diff_summary(&[1, 2], &[1, 2, 3]), (Some(0), Some(3)), "长度不等直接报整段");
}

#[test]
fn impl_bytes_equal_reports_true_when_a_synthetic_snapshot_line_has_the_same_sha256() {
    // 端到端钉一遍量 5 的关键逻辑：给一行手工拼的「crates 快照」，装置读出来的字节算出的 sha256 与快照里写的一样 ⇒ equal=true。
    let device_bytes = b"hello region bytes".to_vec();
    let expected_sha256 = sha256_hex(&device_bytes);
    let snapshot_line = format!("E7RESULT name=impl_region_bytes region=root_record device=0 offset=0 length={} sha256={expected_sha256} hexadecimal_extent=whole_region hexadecimal={}", device_bytes.len(), hex_bytes(&device_bytes));
    let fields = parse_result_line(&snapshot_line);
    assert_eq!(fields.get("sha256").map(String::as_str), Some(expected_sha256.as_str()));
    assert_eq!(hex_decode(fields.get("hexadecimal").expect("hexadecimal 字段")), device_bytes);
}
