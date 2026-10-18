//! checker 档模块：crash_library、crash_amplification、gpu_unit_checks
//! 崩溃放量的 GPU 核对（`gpu_unit_checks`）在本机的卡上：新池新建文件那条流写出的每个单元，两道校验和 GPU 与 CPU 逐个相同、全部成立；
//! 翻载荷里一个字节报载荷 CRC 不对、翻头里一个字节报头校验和不对。GPU 与 CPU 的对拍只在这里单独做，流水线里一段只由一处判（用户 2026-10-11 定）：
//! GPU 一趟（`check_unit_checksums_on_gpu`）、CPU 参照一趟（`check_unit_checksums_on_the_cpu`）各判各的，两边的输入都是同一张表经对拍那一类的库
//! 写进去再读回来的那一份（`gpu_test_start::cpu_reference_table_read_back_from_the_pairing_library`，用户 2026-10-13 定：CPU 参照也从库读）。
//! 要 `gpu` 与 `verdict-store` 两个特性（配置 `ENABLE_GPU=1` 的跑法才开）；开了 `gpu` 而本机一张 NVIDIA 独显适配器都没有时打一行
//! `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功，有卡而用不上（一张都没起来，或 PCI 上有卡而 Vulkan 枚举不出）判红（`gpu_unit_checks::gpu_test_start`）。
//! 卡经这个进程的卡池取，知道要核几个单元之后按 `gpu_unit_checks::mebibytes_crc32c_of_ranges_needs` 借，等满了借不到报「本次未跑」。
#![cfg(all(feature = "gpu", feature = "verdict-store"))]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{build_pool, geometry};
use singlefs_checker::crc32_castagnoli_table;
use singlefs_checker_tier::crash_amplification::RecordedGivenStatesTable;
use singlefs_checker_tier::crash_library::{
    CrashLibrary, CrashLibraryClass, RecordingInputsDigest,
};
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
    cpu_reference_table_of_the_manifest, cpu_reference_table_read_back_from_the_pairing_library,
    gpu_card_pool_or_report_not_run, GpuCardLease, GpuCardPool,
};
use singlefs_checker_tier::gpu_unit_checks::{
    check_unit_checksums_on_gpu, check_unit_checksums_on_the_cpu, mebibytes_crc32c_of_ranges_needs,
    UnitChecksumVerdict,
};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::recovery::PoolReader;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, WrittenContents,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::sha256_digest;

/// 在卡池里借一张卡，够 `checks_at_once` 批各核 `units_per_check` 个单元的两道校验和同时在卡上（每个单元两段字节）；
/// 借到打一行借的是哪张卡，等满了借不到时借卡那一层打「本次未跑」那一行、这里交 `None`（用例当场返回）。
fn lease_for_unit_checks_or_report_not_run(
    cards: &'static GpuCardPool,
    test_name: &str,
    units_per_check: usize,
    checks_at_once: u64,
) -> Option<GpuCardLease> {
    let lease = cards.lease_card_or_report_not_run(
        test_name,
        checks_at_once * mebibytes_crc32c_of_ranges_needs(2 * units_per_check),
    )?;
    let card = lease.card();
    println!(
        "GPU_CARD name={:?} pci={} free_mebibytes={} lent_mebibytes={}",
        card.name,
        card.pci_bus_identifier,
        card.free_memory_mebibytes,
        lease.mebibytes()
    );
    Some(lease)
}

/// 新池新建文件那条流录成一张给定状态表：基线是 mkfs 之后的池，写表是之后的全部写，一个状态、全落。
fn recorded_table(tag: &str) -> RecordedGivenStatesTable {
    let pool = build_pool(tag);
    let operations = pool.retained_operations();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let write_count = writes.len();
    RecordedGivenStatesTable {
        table_name: tag.to_string(),
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        seams: Vec::new(),
        persisted_sets: vec![vec![true; write_count]],
    }
}

/// 从库里读回的那张表全落之后每个单元写的字节（写表里的单元写逐个，按写表次序）。
fn every_unit(read_back: &RecordedGivenStatesTable) -> Vec<Vec<u8>> {
    let mut image = read_back.base.clone();
    image.apply_writes(&read_back.writes);
    read_back
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .map(|write| {
            PoolReader::read(
                &image,
                write.device,
                write.offset,
                usize::try_from(write.length_in_bytes()).expect("装得进"),
            )
            .expect("读得到单元")
        })
        .collect()
}

#[test]
fn gpu_crc32c_matches_the_cpu_on_every_unit_and_both_checksums_of_every_healthy_unit_hold() {
    const TEST_NAME: &str =
        "gpu_crc32c_matches_the_cpu_on_every_unit_and_both_checksums_of_every_healthy_unit_hold";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(read_back) = cpu_reference_table_read_back_from_the_pairing_library(&recorded_table(
        "gpu-unit-checks-healthy",
    )) else {
        return;
    };
    let units = every_unit(&read_back);
    let Some(lease) = lease_for_unit_checks_or_report_not_run(cards, TEST_NAME, units.len(), 1)
    else {
        return;
    };
    let card = lease.card();
    assert!(
        units.len() >= 20,
        "新池新建文件至少写 20 个单元，实际 {}",
        units.len()
    );
    let borrowed: Vec<&[u8]> = units.iter().map(Vec::as_slice).collect();
    let gpu = card.crc32c_of_ranges(&borrowed);
    let cpu: Vec<u32> = borrowed
        .iter()
        .map(|unit| crc32_castagnoli_table(unit))
        .collect();
    assert_eq!(gpu, cpu, "GPU 与 CPU 的 CRC-32C 逐个相同");
    let gpu_verdicts = check_unit_checksums_on_gpu(card, &borrowed);
    drop(lease);
    let cpu_verdicts = check_unit_checksums_on_the_cpu(&borrowed);
    assert_eq!(
        gpu_verdicts, cpu_verdicts,
        "GPU 一趟与 CPU 参照一趟的核对结果逐个相同"
    );
    assert!(
        gpu_verdicts
            .iter()
            .all(|verdict| *verdict == UnitChecksumVerdict::BothHold),
        "健康单元两道校验和都成立：{gpu_verdicts:?}"
    );
    println!("GPU_UNIT_CHECKS units={} all_hold=true", units.len());
}

#[test]
fn a_flipped_payload_byte_and_a_flipped_header_byte_are_each_caught_on_the_gpu() {
    const TEST_NAME: &str =
        "a_flipped_payload_byte_and_a_flipped_header_byte_are_each_caught_on_the_gpu";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    // 翻在写表里（录进库之前）：第一个单元写载荷的末字节、最后一个单元写头里第 8 字节
    let mut table = recorded_table("gpu-unit-checks-flipped");
    let unit_writes: Vec<usize> = table
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::UnitWrite)
        .map(|(write_index, _)| write_index)
        .collect();
    let (first_unit_write, last_unit_write) = (
        *unit_writes.first().expect("流里有单元写"),
        *unit_writes.last().expect("流里有单元写"),
    );
    for (write_index, byte_to_flip) in [(first_unit_write, None), (last_unit_write, Some(8))] {
        let WrittenContents::Bytes(bytes) = &mut table.writes[write_index].contents else {
            panic!("单元写带字节");
        };
        let byte_index = byte_to_flip.unwrap_or(bytes.len() - 1);
        bytes[byte_index] ^= 0x01;
    }
    let Some(read_back) = cpu_reference_table_read_back_from_the_pairing_library(&table) else {
        return;
    };
    let units = every_unit(&read_back);
    let Some(lease) = lease_for_unit_checks_or_report_not_run(cards, TEST_NAME, units.len(), 1)
    else {
        return;
    };
    let last = units.len() - 1;
    for (unit, write_index) in [(0, first_unit_write), (last, last_unit_write)] {
        assert_eq!(
            Some(units[unit].as_slice()),
            read_back.writes[write_index].bytes(),
            "读回的盘上那个单元就是翻过的那一份（之后没有写盖住它）"
        );
    }
    let borrowed: Vec<&[u8]> = units.iter().map(Vec::as_slice).collect();
    let gpu_verdicts = check_unit_checksums_on_gpu(lease.card(), &borrowed);
    drop(lease);
    let cpu_verdicts = check_unit_checksums_on_the_cpu(&borrowed);
    assert_eq!(
        gpu_verdicts, cpu_verdicts,
        "GPU 一趟与 CPU 参照一趟的核对结果逐个相同"
    );
    assert_eq!(
        gpu_verdicts[0],
        UnitChecksumVerdict::PayloadCrcMismatch,
        "载荷末字节翻了"
    );
    assert_eq!(
        gpu_verdicts[last],
        UnitChecksumVerdict::HeaderChecksumMismatch,
        "头第 8 字节翻了"
    );
    assert!(gpu_verdicts[1..last]
        .iter()
        .all(|verdict| *verdict == UnitChecksumVerdict::BothHold));
}

/// CPU 参照的输入真的过了库（不要卡）：一张这一趟才有的小表（名字带进程号与此刻的纳秒，摘要别的趟都没录过）交给
/// `cpu_reference_table_read_back_from_the_pairing_library`，交回的与给的逐字段相同；对拍那一类的库里凭它摘要起的项名读得到清单，
/// 清单指到的那张表从库里读回来也与给的逐字段相同。
/// 判别力：CPU 参照那一侧不经库（帮手直接交回给的那一份），库里没有这一项的清单，这里红。
#[test]
fn the_cpu_reference_inputs_are_read_back_from_the_pairing_library() {
    let started_at_nanoseconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("此刻在 1970 年之后")
        .as_nanos();
    let table = RecordedGivenStatesTable {
        table_name: format!(
            "cpu-reference-round-trip-{}-{started_at_nanoseconds}",
            std::process::id()
        ),
        base: MemoryPool::with_devices(&[DeviceIdentity(0)], 1 << 20),
        writes: Vec::new(),
        segments: Vec::new(),
        seams: Vec::new(),
        persisted_sets: vec![Vec::new()],
    };
    let Some(read_back) = cpu_reference_table_read_back_from_the_pairing_library(&table) else {
        return;
    };
    assert_eq!(read_back, table, "交回的是读回的那一份，与给的逐字段相同");
    let library =
        CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("对拍那一类的库开得了");
    let manifest = library
        .manifest_of_the_inputs(&RecordingInputsDigest(sha256_digest(&table.to_bytes())))
        .expect("库里有这一项的清单：CPU 参照那一侧经过了库");
    assert_eq!(
        cpu_reference_table_of_the_manifest(&library, &manifest),
        Ok(table),
        "清单指到的那张表从库里读回来与给的逐字段相同"
    );
}
