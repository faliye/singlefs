use super::journal_in_flight_record_limit as apparatus_journal_in_flight_record_limit;
use singlefs_core::system_configuration::journal_in_flight_record_limit as core_journal_in_flight_record_limit;
use std::collections::BTreeSet;

/// 多一条在飞记录要多多少环长：一条记录 4096（D23（journal 的角色与格式） 已定项 12）× F = 3（I-8.1（环几何够大））。
/// 抄条款的数，不取格式常量：钉绝对值那条要在共享标量被改歪时照样红。
const RING_BYTES_PER_IN_FLIGHT_RECORD: u64 = 4096 * 3;
const DENSELY_SCANNED_RING_BYTES_UPPER_END: u64 = 1 << 20;
const DEFAULT_RING_BYTES: u64 = 768 * 1024 * 1024;
const LARGEST_RING_BYTES_MAKE_FILESYSTEM_ACCEPTS: u64 = 1 << 60;
const RING_BYTES_WHERE_THE_LIMIT_FIRST_EXCEEDS_FOUR_BYTES: u64 =
    (1 << 32) * RING_BYTES_PER_IN_FLIGHT_RECORD;

/// 枚举点 ② 到 ⑤（① 那一段在测试里直接逐个走，不进这个集合）。
fn ring_sizes_at_every_magnitude_and_at_the_edges() -> BTreeSet<u64> {
    let mut ring_sizes = BTreeSet::new();
    for bit_position in 0..u64::BITS {
        let power_of_two = 1u64 << bit_position;
        let multiple_below =
            power_of_two / RING_BYTES_PER_IN_FLIGHT_RECORD * RING_BYTES_PER_IN_FLIGHT_RECORD;
        for anchor in [power_of_two, multiple_below] {
            for offset_below in [1u64, 0] {
                if let Some(ring_bytes) = anchor.checked_sub(offset_below) {
                    ring_sizes.insert(ring_bytes);
                }
            }
            for offset_above in [1u64, 4095, 4096, 8191, 8192] {
                if let Some(ring_bytes) = anchor.checked_add(offset_above) {
                    ring_sizes.insert(ring_bytes);
                }
            }
        }
    }
    let top_multiple = u64::MAX / RING_BYTES_PER_IN_FLIGHT_RECORD * RING_BYTES_PER_IN_FLIGHT_RECORD;
    for ring_bytes in [
        u64::MAX,
        u64::MAX - 1,
        top_multiple - 1,
        top_multiple,
        top_multiple + 1,
    ] {
        ring_sizes.insert(ring_bytes);
    }
    for ring_bytes in [
        RING_BYTES_WHERE_THE_LIMIT_FIRST_EXCEEDS_FOUR_BYTES - 1,
        RING_BYTES_WHERE_THE_LIMIT_FIRST_EXCEEDS_FOUR_BYTES,
        RING_BYTES_WHERE_THE_LIMIT_FIRST_EXCEEDS_FOUR_BYTES + 1,
        DEFAULT_RING_BYTES,
    ] {
        ring_sizes.insert(ring_bytes);
    }
    ring_sizes
}

#[test]
fn core_and_apparatus_compute_the_same_in_flight_record_limit_at_every_enumerated_ring_size() {
    let mut densely_compared = 0u64;
    for ring_bytes in 0..=DENSELY_SCANNED_RING_BYTES_UPPER_END {
        assert_eq!(
            core_journal_in_flight_record_limit(ring_bytes),
            apparatus_journal_in_flight_record_limit(ring_bytes),
            "环长 {ring_bytes} 字节：core 与装置的在飞上限不等"
        );
        densely_compared += 1;
    }
    assert_eq!(
        densely_compared,
        DENSELY_SCANNED_RING_BYTES_UPPER_END + 1,
        "① 那一段从 0 到 2^20 一个不落"
    );
    let edge_ring_sizes = ring_sizes_at_every_magnitude_and_at_the_edges();
    for ring_bytes in &edge_ring_sizes {
        assert_eq!(
            core_journal_in_flight_record_limit(*ring_bytes),
            apparatus_journal_in_flight_record_limit(*ring_bytes),
            "环长 {ring_bytes} 字节：core 与装置的在飞上限不等"
        );
    }
    for must_be_compared in [
        u64::MAX,
        LARGEST_RING_BYTES_MAKE_FILESYSTEM_ACCEPTS,
        RING_BYTES_WHERE_THE_LIMIT_FIRST_EXCEEDS_FOUR_BYTES,
        DEFAULT_RING_BYTES,
    ] {
        assert!(
            edge_ring_sizes.contains(&must_be_compared),
            "枚举点里要有 {must_be_compared}：取值域的这一端没比到"
        );
    }
}

/// 绝对值按条款手算（记录 4096、F = 3，两次取整数部分），不调任何一份式子。
#[test]
fn the_in_flight_record_limit_is_pinned_at_values_worked_out_from_the_clauses() {
    let pinned: [(u64, u64, &str); 7] = [
        (
            805_306_368,
            65_536,
            "默认环 768 MiB：196608 个记录槽 ÷ 3（D23 已定项 18 的定案原文）",
        ),
        (12_287, 0, "不到 3 条记录：mkfs 拒的那一格"),
        (12_288, 1, "恰好 3 条记录：mkfs 收得下的最短环"),
        (
            1_152_921_504_606_846_976,
            93_824_992_236_885,
            "2^60：按条款 mkfs 收得下的最长环，2^48 ÷ 3 的整数部分",
        ),
        (
            52_776_558_133_247,
            4_294_967_295,
            "在飞上限恰好还装得进 4 字节",
        ),
        (52_776_558_133_248, 4_294_967_296, "在飞上限刚越过 4 字节"),
        (u64::MAX, 1_501_199_875_790_165, "u64 顶端：(2^52 − 1) ÷ 3"),
    ];
    for (ring_bytes, expected_limit, why) in pinned {
        assert_eq!(
            core_journal_in_flight_record_limit(ring_bytes),
            expected_limit,
            "core：{why}"
        );
        assert_eq!(
            apparatus_journal_in_flight_record_limit(ring_bytes),
            expected_limit,
            "装置：{why}"
        );
    }
}
