//! 正推腿对 K3-1 的 CPU 两侧差分：`singlefs_core::instance_table::InstanceTableChainRecord::parse`（实现）对
//! checker 的 `walk::chain_record_view`（草稿副本里给它加了一个公开的垫片 `chain_record_view_code_for_the_forward_leg_probe`，不入库）。
//! 输入：88 字节的链指针记录，从一条合法的「无下一片」与一条合法的「有下一片」出发，逐字节改成几个代表值、再两字节组合，
//! 加长度不是 88 的、加带固定种子的伪随机（大半为 0）。GPU 两个读法（WGSL）这里跑不了，另按行对读，见报告。
use singlefs_checker::walk::chain_record_view_code_for_the_forward_leg_probe as checker_code;
use singlefs_core::instance_table::InstanceTableChainRecord;

fn core_code(record: &[u8]) -> u8 {
    match InstanceTableChainRecord::parse(record) {
        Some(InstanceTableChainRecord::LastPage) => 0,
        Some(InstanceTableChainRecord::NextPage(_)) => 1,
        None => 2,
    }
}

fn last_page_record() -> Vec<u8> {
    let mut record = vec![0u8; 88];
    record[0] = 1;
    record
}

/// 有下一片：标志 1、位置指针里写一条看得过去的（出生树 0、出生 txg 5、两条位置条目升序）。
fn next_page_record() -> Vec<u8> {
    let mut record = vec![0u8; 88];
    record[0] = 1;
    record[1] = 1;
    let pointer = 2;
    record[pointer + 42] = 5; // 出生 txg 低字节
    record[pointer + 50] = 1; // 位置条目 0 的设备
    record[pointer + 54] = 7; // 槽
    record[pointer + 64] = 2; // 位置条目 1 的设备
    record[pointer + 68] = 8;
    record
}

fn assert_same(record: &[u8], what: &str, cases: &mut u64, disagreements: &mut Vec<String>) {
    *cases += 1;
    let (core, checker) = (core_code(record), checker_code(record));
    if core != checker {
        disagreements.push(format!("{what}: core={core} checker={checker} bytes={record:?}"));
    }
}

#[test]
fn probe_the_core_and_the_checker_read_every_one_byte_and_two_byte_change_of_a_chain_record_the_same_way() {
    assert_eq!(core_code(&last_page_record()), 0);
    assert_eq!(core_code(&next_page_record()), 1);
    assert_eq!(checker_code(&last_page_record()), 0);
    assert_eq!(checker_code(&next_page_record()), 1);
    let values = [0u8, 1, 2, 0x7f, 0x80, 0xff];
    let mut cases = 0u64;
    let mut disagreements = Vec::new();
    for base in [last_page_record(), next_page_record()] {
        for position in 0..88 {
            for value in values {
                let mut record = base.clone();
                record[position] = value;
                assert_same(&record, &format!("one byte @{position}={value}"), &mut cases, &mut disagreements);
            }
        }
        // 两字节：头 36 字节（种类、标志、MAC/nonce、算法类型、压缩码、压后长度、extent 偏移）里任两处
        for first in 0..36 {
            for second in first + 1..36 {
                for value in [1u8, 0x80] {
                    let mut record = base.clone();
                    record[first] = value;
                    record[second] = value;
                    assert_same(&record, &format!("two bytes @{first},{second}={value}"), &mut cases, &mut disagreements);
                }
            }
        }
    }
    for length in [0usize, 1, 2, 87, 89, 176] {
        let mut record = next_page_record();
        record.resize(length, 0);
        if length >= 88 || length == 0 {
            // 长度不是 88：两边都是解不开（不去碰短于 2 字节会在读标志时越界的写法）
            assert_same(&record, &format!("length {length}"), &mut cases, &mut disagreements);
        }
    }
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    for _ in 0..200_000 {
        let mut record = vec![0u8; 88];
        for byte in &mut record {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            if state % 4 == 0 {
                *byte = (state >> 32) as u8;
            }
        }
        record[0] = if state % 8 == 0 { (state >> 40) as u8 } else { 1 };
        record[1] = [0u8, 1, 1, 2][(state >> 48) as usize % 4];
        assert_same(&record, "random", &mut cases, &mut disagreements);
    }
    println!("PROBE_K3_CPU cases={cases} disagreements={}", disagreements.len());
    for line in disagreements.iter().take(5) {
        println!("PROBE_K3_CPU_DISAGREEMENT {line}");
    }
    assert!(disagreements.is_empty());
}
