//! checker 档模块：crash、crash_judge_tables
//! GPU 判器输入表的落点版本模型在新池新建文件那条流上的判别力自证：每个崩溃状态、每个落点，按持久掩码查到的版本的字节要与
//! 层 0 崩溃镜像在那个落点上读出的字节逐字节相同；逐扇区等不等的掩码要与「那次写还在盘上」逐扇区相同；
//! 两条根槽写的根身份、发布分组与记录核对器同一个分法。改坏叠写的次序（后写的不盖前写的）时第一条断言必红（写法见文件末尾的注释）。
#![allow(
    clippy::doc_markdown,
    reason = "测试文件头写的是中文说明，不是要加反引号的标识符"
)]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{build_pool, file_content, geometry};
use singlefs_checker_tier::crash::{
    layer0_enumeration_tables, layer0_plan_state_count, Layer0SegmentExpansion,
};
use singlefs_checker_tier::crash_judge_tables::{
    build_judge_tables, JudgeFlow, JudgeLocationKind, MAXIMUM_LOCATION_SECTORS,
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

fn expansion(_segment_index: usize, segment: &[usize]) -> Layer0SegmentExpansion {
    if segment.len() <= LARGEST_EXPANDED_SEGMENT_WRITES {
        Layer0SegmentExpansion::EveryProperSubset
    } else {
        Layer0SegmentExpansion::NotExpanded
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
    let enumeration = layer0_enumeration_tables(&base, &writes, &segments, &expansion);
    let enumerated = &enumeration.enumerated_writes;
    assert_eq!(
        tables.state_count,
        layer0_plan_state_count(&base, &writes, &segments, &expansion)
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
    let mut versions_checked = 0u64;
    let mut sectors_checked = 0u64;
    for ordinal in 0..tables.state_count {
        let persisted = tables.persisted_writes_of_state(ordinal);
        let image = CrashImage {
            base: &base,
            writes: enumerated,
            persisted: persisted.clone(),
        };
        for location in &tables.locations {
            let version = tables.version_of(location, &persisted).unwrap_or_else(|| {
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
                let write = &enumerated[usize::try_from(*entry).expect("下标")];
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
