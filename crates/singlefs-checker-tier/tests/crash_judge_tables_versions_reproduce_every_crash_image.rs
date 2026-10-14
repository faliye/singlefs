//! checker 档模块：crash、crash_library、crash_amplification、crash_judge_tables
//! GPU 判器输入表的落点版本模型在新池新建文件那条流上的判别力自证：每个崩溃状态、每个落点，按持久掩码查到的版本的字节要与
//! 崩溃点重放的崩溃镜像在那个落点上读出的字节逐字节相同（崩溃镜像那一侧是 CPU 参照：基线、枚举写与每个状态的持久集合作一张给定状态表
//! 先写进对拍那一类的库、再读回，镜像按读回的那一份叠，用户 2026-10-13 定：CPU 参照也从库读）；逐扇区等不等的掩码要与「那次写还在盘上」逐扇区相同；
//! 两条根槽写的根身份、发布分组与记录核对器同一个分法。改坏叠写的次序（后写的不盖前写的）时第一条断言必红（写法见文件末尾的注释）。
#![allow(
    clippy::doc_markdown,
    reason = "测试文件头写的是中文说明，不是要加反引号的标识符"
)]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{build_pool, file_content, geometry};
use singlefs_checker_tier::crash::{
    crash_point_replay_plan_state_count, crash_state_enumeration_tables,
    CrashPointReplaySegmentExpansion,
};
use singlefs_checker_tier::crash_amplification::RecordedGivenStatesTable;
use singlefs_checker_tier::crash_judge_tables::{
    build_judge_tables, JudgeFlow, JudgeLocationKind, MAXIMUM_LOCATION_SECTORS,
};
use singlefs_checker_tier::crash_library::{
    repository_root_of_this_crate, two_stages, CrashLibraryClass, ItemManifest, ItemName,
    StageOutcome,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::recovery::PoolReader;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, CrashImage, PublishedVersion, SECTOR_BYTES,
};
use singlefs_harness::segments::StepKind;

const LARGEST_EXPANDED_SEGMENT_WRITES: usize = 6;

/// 崩溃镜像那一侧（CPU 参照）的输入在对拍那一类库里的项名，与清单里那张表那一样的名字。
const CPU_REFERENCE_ITEM: &str = "crash_judge_tables_versions/cpu-reference";
const CPU_REFERENCE_TABLE_IN_THE_MANIFEST: &str = "cpu_reference_given_states_table";

/// 崩溃镜像那一侧（CPU 参照）的输入先入库、再读回：录入段把这张给定状态表按摘要写进对拍那一类的库、按项名登清单；判段凭项名读清单、
/// 按摘要读回、核摘要，交回读回的那一份。只录那一趟交 `None`（用例当场返回）。
fn crash_images_read_back_from_the_pairing_library(
    table: &RecordedGivenStatesTable,
) -> Option<RecordedGivenStatesTable> {
    let bytes = table.to_bytes();
    let digest = singlefs_harness::sha256::sha256_digest(&bytes);
    let item = ItemName::new(CPU_REFERENCE_ITEM).expect("写死的项名合规矩");
    two_stages(
        CrashLibraryClass::JudgeTablesPairing,
        &repository_root_of_this_crate(),
        &item,
        |library| {
            library
                .lock()
                .store_given_states_table(&digest, &bytes)
                .map_err(|error| format!("崩溃镜像那一侧的表写不进库：{error:?}"))?;
            Ok(ItemManifest {
                kind: "cpu_reference_pairing".to_string(),
                digests: vec![(CPU_REFERENCE_TABLE_IN_THE_MANIFEST.to_string(), digest)],
            })
        },
        |library, manifest| {
            let recorded = manifest.digest_named(CPU_REFERENCE_TABLE_IN_THE_MANIFEST)?;
            let read_back = library
                .lock()
                .read_given_states_table(&recorded)
                .map_err(|error| format!("读崩溃镜像那一侧的表失败：{error:?}"))?
                .ok_or_else(|| "崩溃镜像那一侧的表不在库里：先跑录入段".to_string())?;
            if singlefs_harness::sha256::sha256_digest(&read_back) != recorded {
                return Err("崩溃镜像那一侧的表从库里读回的摘要对不上".to_string());
            }
            RecordedGivenStatesTable::from_bytes(&read_back)
                .map(StageOutcome::Judged)
                .ok_or_else(|| "崩溃镜像那一侧的表从库里读回的字节解不出".to_string())
        },
    )
    .unwrap_or_else(|refusal| panic!("崩溃镜像那一侧的表过库失败：{refusal}"))
    .judged()
}

fn expansion(_segment_index: usize, segment: &[usize]) -> CrashPointReplaySegmentExpansion {
    if segment.len() <= LARGEST_EXPANDED_SEGMENT_WRITES {
        CrashPointReplaySegmentExpansion::EveryProperSubset
    } else {
        CrashPointReplaySegmentExpansion::NotExpanded
    }
}

#[test]
fn every_location_version_reproduces_the_crash_image_bytes_and_sector_equality_on_every_state() {
    let pool = build_pool("crash-judge-tables-versions");
    let operations = pool.retained_operations();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布有一条根槽写");
    let base = pool.memory_pool_after_mkfs();
    let versions = vec![PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
        content: file_content(),
    }];
    let flow = JudgeFlow {
        base: &base,
        writes: &writes,
        segments: &segments,
        judged_root_index,
        versions: &versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&flow).expect("新池新建文件那条流建得出表");
    let enumeration = crash_state_enumeration_tables(&base, &writes, &segments, &expansion);
    let enumerated = &enumeration.enumerated_writes;
    assert_eq!(
        tables.state_count,
        crash_point_replay_plan_state_count(&base, &writes, &segments, &expansion)
    );
    assert_eq!(
        usize::try_from(tables.enumerated_write_count).expect("装得进"),
        enumerated.len()
    );
    // 落点种类都在：每盘两个系统配置槽、根环槽、单元槽；这条流没有 journal 记录时也有记录写落下的记录槽
    for kind in [
        JudgeLocationKind::SystemConfiguration,
        JudgeLocationKind::RootSlot,
        JudgeLocationKind::JournalRecord,
        JudgeLocationKind::UnitSlot,
    ] {
        assert!(
            tables
                .locations
                .iter()
                .any(|location| location.kind == kind),
            "落点表里有 {kind:?}"
        );
    }
    // 每条根槽写的根身份就是它字节里偏移 24 / 28 的那两个数
    for (write_index, write) in enumerated.iter().enumerate() {
        let judge_write = &tables.writes[write_index];
        if write.kind == StepKind::RootRecordFua {
            let bytes = write.bytes().expect("根槽写带字节");
            let instance = u32::from_le_bytes(bytes[24..28].try_into().expect("4"));
            let txg = u64::from_le_bytes(bytes[28..36].try_into().expect("8"));
            assert_eq!(judge_write.root_identity, Some((instance, txg)));
            assert_eq!(judge_write.root_bytes, bytes);
        } else {
            assert_eq!(judge_write.root_identity, None);
            assert!(judge_write.root_bytes.is_empty());
        }
    }
    assert_eq!(
        tables.publishes.len(),
        enumerated
            .iter()
            .filter(|write| write.kind == StepKind::RootRecordFua)
            .count(),
        "每条根槽写一次发布"
    );
    let Some(crash_images) =
        crash_images_read_back_from_the_pairing_library(&RecordedGivenStatesTable {
            table_name: "crash-judge-tables-versions".to_string(),
            base: base.clone(),
            writes: enumerated.clone(),
            segments: Vec::new(),
            seams: Vec::new(),
            persisted_sets: (0..tables.state_count)
                .map(|ordinal| tables.persisted_writes_of_state(ordinal))
                .collect(),
        })
    else {
        return;
    };
    assert_eq!(
        u64::try_from(crash_images.persisted_sets.len()).expect("装得进 u64"),
        tables.state_count,
        "读回的表里每个状态一个持久集合"
    );
    let mut versions_checked = 0u64;
    let mut sectors_checked = 0u64;
    for (ordinal, persisted) in (0..tables.state_count).zip(&crash_images.persisted_sets) {
        let image = CrashImage {
            base: &crash_images.base,
            writes: &crash_images.writes,
            persisted: persisted.clone(),
        };
        for location in &tables.locations {
            let version = tables.version_of(location, persisted).unwrap_or_else(|| {
                panic!(
                    "状态 {ordinal} 在落点 (盘 {}, 偏移 {}) 上的组合没有版本",
                    location.device, location.offset
                )
            });
            let on_disk = PoolReader::read(
                &image,
                DeviceIdentity(location.device),
                DeviceOffsetInBytes(location.offset),
                usize::try_from(location.length).expect("长度"),
            )
            .expect("落点在盘内");
            assert!(
                version.bytes == on_disk,
                "状态 {ordinal} 落点 (盘 {}, 偏移 {}, {:?}) 的版本字节与崩溃镜像不同",
                location.device,
                location.offset,
                location.kind
            );
            versions_checked += 1;
            let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
            for (position, entry) in location.writes.iter().enumerate() {
                let write = &crash_images.writes[usize::try_from(*entry).expect("下标")];
                let mask = version.sector_equals[position];
                let sector_count = usize::try_from(location.length).expect("长度") / sector_bytes;
                assert!(sector_count <= MAXIMUM_LOCATION_SECTORS);
                for sector in 0..sector_count {
                    let sector_start =
                        location.offset + u64::try_from(sector).expect("扇区") * SECTOR_BYTES;
                    let inside_the_write = sector_start >= write.offset.0
                        && sector_start + SECTOR_BYTES <= write.offset.0 + write.length_in_bytes();
                    let expected = inside_the_write
                        && write.contents.range_still_on_disk(
                            usize::try_from(sector_start - write.offset.0).expect("偏移"),
                            &on_disk[sector * sector_bytes..(sector + 1) * sector_bytes],
                        );
                    assert_eq!(
                        (mask >> sector) & 1 == 1,
                        expected,
                        "状态 {ordinal} 落点 (盘 {}, 偏移 {}) 第 {sector} 扇区对写 {entry} 的等不等",
                        location.device,
                        location.offset
                    );
                    sectors_checked += 1;
                }
            }
        }
    }
    let words = tables.words();
    assert_eq!(
        words.locations.len(),
        tables.locations.len() * singlefs_checker_tier::crash_judge_tables::LOCATION_WORDS
    );
    assert_ne!(tables.digest(), [0u8; 32]);
    println!(
        "CRASH_JUDGE_TABLES states={} locations={} versions_checked={versions_checked} sectors_checked={sectors_checked} version_bytes_words={}",
        tables.state_count,
        tables.locations.len(),
        words.version_bytes.len()
    );
}

// 证红：把 crash_judge_tables.rs 的 `overlay` 改成只在 `bytes` 全 0 处才写（`if bytes[destination..].iter().all(|byte| *byte == 0)`），
// 后写的不再盖前写的，撕裂镜像叠上重放那一格的版本字节与镜像不同，第一条 assert 报「版本字节与崩溃镜像不同」。
