//! checker 档模块：crash、layer0_progress、crash_identity、crash_amplification、verdict_store、gpu_unit_checks
//! 崩溃放量的 GPU 核对（`gpu_unit_checks`）在本机的卡上：新池新建文件那条流写出的每个单元，两道校验和 GPU 与 CPU 逐个相同、全部成立；
//! 翻载荷里一个字节报载荷 CRC 不对、翻头里一个字节报头校验和不对；整条流水线开着 GPU 核一遍 0 红、0 对不上。
//! 要 `gpu` 与 `verdict-store` 两个特性（配置 `ENABLE_GPU=1` 的跑法才开）；开了 `gpu` 却一张能用的卡都没有就红，不悄悄跳过。
#![cfg(all(feature = "gpu", feature = "verdict-store"))]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use std::path::PathBuf;
use std::sync::Arc;

use common::{build_pool, file_content, geometry};
use singlefs_checker::crc32_castagnoli_table;
use singlefs_checker_tier::crash::Layer0SegmentExpansion;
use singlefs_checker_tier::crash_amplification::{
    check_recorded_blocks, plan_crash_points, record_crash_points, CrashFlow, CrashPointSpan,
    JudgingCodes, UnitChecksumsOn, WeightedShares,
};
use singlefs_checker_tier::crash_identity::CoverageReport;
use singlefs_checker_tier::gpu_unit_checks::{
    check_unit_checksums_on_gpu, usable_gpu_cards, GpuCard, UnitChecksumVerdict,
};
use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;
use singlefs_checker_tier::verdict_store::VerdictStore;
use singlefs_core::recovery::PoolReader;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs";

fn one_usable_card() -> GpuCard {
    let mut cards = usable_gpu_cards();
    assert!(!cards.is_empty(), "gpu 特性开着，本机却没有一张空闲显存够的 NVIDIA 独显（nvidia-smi 的 memory.free 不低于 2048 MiB）");
    let card = cards.remove(0);
    println!(
        "GPU_CARD name={:?} pci={} free_mebibytes={}",
        card.name, card.pci_bus_identifier, card.free_memory_mebibytes
    );
    card
}

struct Recorded {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    first_publish_write: usize,
}

fn record(tag: &str) -> Recorded {
    let pool = build_pool(tag);
    let operations = pool.retained_operations();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let publish_boundary = pool.warm_up_operation_count - pool.mkfs_operation_count;
    let first_publish_write = stream_indexes
        .iter()
        .position(|stream_index| *stream_index >= publish_boundary)
        .expect("发布有写");
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    let versions = vec![PublishedVersion {
        instance: pool.output.root.instance,
        checkpoint_txg: pool.output.root.checkpoint_txg,
        content: file_content(),
    }];
    Recorded {
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        judged_root_index,
        versions,
        first_publish_write,
    }
}

/// 整条流全落之后每个单元写的字节。
fn every_unit(recorded: &Recorded) -> Vec<Vec<u8>> {
    let mut image = recorded.base.clone();
    image.apply_writes(&recorded.writes);
    recorded
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
    let card = one_usable_card();
    let recorded = record("gpu-unit-checks-healthy");
    let units = every_unit(&recorded);
    assert!(
        units.len() >= 20,
        "新池新建文件至少写 20 个单元，实际 {}",
        units.len()
    );
    let borrowed: Vec<&[u8]> = units.iter().map(Vec::as_slice).collect();
    let gpu = card.crc32c_of_ranges(&borrowed);
    for (unit, crc) in units.iter().zip(&gpu) {
        assert_eq!(
            *crc,
            crc32_castagnoli_table(unit),
            "GPU 与 CPU 的 CRC-32C 逐个相同"
        );
    }
    let verdicts = check_unit_checksums_on_gpu(&card, &borrowed);
    assert!(
        verdicts
            .iter()
            .all(|verdict| *verdict == UnitChecksumVerdict::BothHold),
        "健康单元两道校验和都成立：{verdicts:?}"
    );
    println!("GPU_UNIT_CHECKS units={} all_hold=true", units.len());
}

#[test]
fn a_flipped_payload_byte_and_a_flipped_header_byte_are_each_caught_on_the_gpu() {
    let card = one_usable_card();
    let recorded = record("gpu-unit-checks-flipped");
    let mut units = every_unit(&recorded);
    let last = units.len() - 1;
    let last_byte_of_first_unit = units[0].len() - 1;
    units[0][last_byte_of_first_unit] ^= 0x01;
    units[last][8] ^= 0x01;
    let borrowed: Vec<&[u8]> = units.iter().map(Vec::as_slice).collect();
    let verdicts = check_unit_checksums_on_gpu(&card, &borrowed);
    assert_eq!(
        verdicts[0],
        UnitChecksumVerdict::PayloadCrcMismatch,
        "载荷末字节翻了"
    );
    assert_eq!(
        verdicts[last],
        UnitChecksumVerdict::HeaderChecksumMismatch,
        "头第 8 字节翻了"
    );
    assert!(verdicts[1..last]
        .iter()
        .all(|verdict| *verdict == UnitChecksumVerdict::BothHold));
}

#[test]
fn the_pipeline_with_gpu_unit_checks_reports_no_red_and_no_disagreement_on_the_healthy_flow() {
    let card = Arc::new(one_usable_card());
    let recorded = record("gpu-pipeline");
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= 6 {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let flow = CrashFlow {
        base: &recorded.base,
        writes: &recorded.writes,
        segments: &recorded.segments,
        judged_root_index: recorded.judged_root_index,
        versions: &recorded.versions,
        expansion: &expansion,
        crash_points: vec![
            CrashPointSpan {
                name: "acquire_instance_and_warm_up".to_string(),
                code_file_in_repository: THIS_FILE.to_string(),
                writes: 0..recorded.first_publish_write,
                ignored: false,
            },
            CrashPointSpan {
                name: "publish_first_file".to_string(),
                code_file_in_repository: THIS_FILE.to_string(),
                writes: recorded.first_publish_write..recorded.writes.len(),
                ignored: false,
            },
        ],
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("仓根");
    let judging = JudgingCodes::of_files(
        &root,
        &[THIS_FILE],
        &Layer0ToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .expect("判法摘要");
    let plan = plan_crash_points(&flow, &judging);
    let directory =
        std::env::temp_dir().join(format!("singlefs-gpu-pipeline-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    let mut store = VerdictStore::create_empty(&directory).expect("建得了库");
    record_crash_points(&mut store, &plan, &mut CoverageReport::default()).expect("录得进");
    let tally = check_recorded_blocks(
        &mut store,
        &flow,
        &plan,
        &WeightedShares::new(vec![1]),
        0,
        &UnitChecksumsOn::Gpu(card),
    )
    .expect("核得完");
    drop(store);
    std::fs::remove_dir_all(&directory).expect("删得掉库");
    assert!(tally.states_checked > 0);
    assert_eq!(
        tally.gpu_cpu_disagreements, 0,
        "GPU 与 CPU 一处都不许对不上"
    );
    assert_eq!(
        tally.red_states, 0,
        "健康的流开着 GPU 核也是 0 红：{:?}",
        tally.red_texts
    );
    println!(
        "GPU_PIPELINE states={} red={} disagreements={}",
        tally.states_checked, tally.red_states, tally.gpu_cpu_disagreements
    );
}
