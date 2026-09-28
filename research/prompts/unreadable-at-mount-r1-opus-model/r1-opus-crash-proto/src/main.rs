//! unreadable-at-mount-r1 云端攻方腿的原型（只在仓副本里跑，不是入库装置）：几条自己造的小流按层 0 的枚举域展开
//! （`singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state`，每段都展开、原地覆写取三态），
//! 每个崩溃状态在这份副本的产品代码上可写挂载，看拒不拒、丢不丢已确认的写。每个状态打一行 `name=crashproto …`。
#![allow(dead_code, reason = "common 各函数只用到一部分")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{
    build_pool, crash_state_devices, geometry, parameters, publish_overwrite_in_process,
    with_unreadable_ranges, FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange,
    UnreadableRangeReadBack, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker_tier::crash::{
    enumerate_layer0_selecting_versions_observing_each_state,
    layer0_state_count_with_torn_in_place_overwrites, Layer0SegmentExpansion,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, FileOffsetInBytes, InodeNumber,
    InstanceGeneration,
};
use singlefs_core::journal::record_offset;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
};
use singlefs_format::{JOURNAL_RECORD_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, CrashImage, MemoryPool, PublishedVersion,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;
use std::collections::BTreeMap;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn second_instance_content(index: usize) -> Vec<u8> {
    content_of(4100 - 100 * index, 3 + index)
}

fn d_content() -> Vec<u8> {
    content_of(3300, 17)
}

fn e_content() -> Vec<u8> {
    content_of(3737, 30)
}

fn slot_spacing() -> u64 {
    u64::from(parameters().geometry.fixed_structure_slot_spacing)
}

/// 标准历史：A（实例 1，txg 3）→ 重开，实例 2 写行 4、暖机 5、B 6、C 7，全部确认返回。交回盘上的样子与版本表。
fn standard_history() -> (
    MemoryPool,
    Vec<PublishedVersion>,
    TransactionOutput,
    singlefs_core::allocator::PoolAllocator,
) {
    let mut pool = build_pool("r1-opus-crash-proto");
    let a = pool.output.clone();
    let mut versions = vec![PublishedVersion {
        instance: a.root.instance,
        checkpoint_txg: a.root.checkpoint_txg,
        content: common::file_content(),
    }];
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("实例 2 可写挂载");
    for version in std::iter::once(&mounted.output.row_publish).chain(mounted.output.warm_up_publishes.iter()) {
        versions.push(PublishedVersion {
            instance: version.root().instance,
            checkpoint_txg: version.root().checkpoint_txg,
            content: common::file_content(),
        });
    }
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current.into_file_version().expect("带文件");
    for index in 0..2 {
        let previous = pool.output.clone();
        let next = publish_overwrite_in_process(
            &mut pool,
            &previous,
            &second_instance_content(index),
            FIXED_WRITE_TIME_SECONDS + 60 * (1 + u64::try_from(index).expect("小")),
            InstanceGeneration(2),
        )
        .expect("实例 2 覆盖写");
        versions.push(PublishedVersion {
            instance: next.root.instance,
            checkpoint_txg: next.root.checkpoint_txg,
            content: second_instance_content(index),
        });
        pool.output = next;
    }
    let c = pool.output.clone();
    (pool.memory_pool(), versions, c, pool.allocator.clone())
}

fn newest_slot_ranges(image: &MemoryPool, failing: FailingReadsOfARange) -> Vec<UnreadableRange> {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|device| {
            let newest = singlefs_core::recovery::verified_system_configuration_slots(
                image,
                device,
                slot_spacing(),
                &parameters().filesystem_identifier,
            )
            .iter()
            .map(|slot| slot.quantities.slot_generation)
            .max()
            .expect("有");
            UnreadableRange {
                device,
                offset_in_bytes: (newest % 2) * slot_spacing(),
                length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
                failing_reads: failing,
            }
        })
        .collect()
}

fn root_and_record_ranges(txg: u64, counter: u64) -> Vec<UnreadableRange> {
    let mut ranges = vec![common::unreadable_root_slot_of(
        CheckpointTxg(txg),
        FailingReadsOfARange::Every,
    )];
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        ranges.push(UnreadableRange {
            device,
            offset_in_bytes: record_offset(counter, parameters().geometry.journal_ring_bytes).0,
            length_in_bytes: JOURNAL_RECORD_BYTES,
            failing_reads: FailingReadsOfARange::Every,
        });
    }
    ranges
}

type FaultyDevices = Vec<(
    DeviceIdentity,
    common::DeviceWithUnreadableRanges<
        singlefs_harness::RecordingBlockDevice<singlefs_harness::memory_pool::SparseBlockDevice>,
    >,
)>;

fn snapshot_of(devices: &FaultyDevices) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner_device().wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: common::IMAGE_BYTES,
    }
}

/// 一个盘面上可写挂载一次（`hidden` 在挂载时读不出，挂载交回就撤），做成了就写 E、确认返回、此刻崩溃，冷重开只读看 E 在不在。
fn mount_and_judge(image: &MemoryPool, hidden: Vec<UnreadableRange>, newest_acknowledged_txg: u64) -> String {
    let unreadable = SharedUnreadableRanges::new(hidden, UnreadableRangeReadBack::DeviceError);
    let stream = SharedStream::new();
    let mut devices: FaultyDevices =
        with_unreadable_ranges(crash_state_devices(image, &[], &[], &stream), &unreadable);
    let mounted = mount_writable(&parameters(), &mut devices);
    let mut mounted = match mounted {
        Err(error) => {
            let text: String = format!("{error:?}").chars().take(80).collect();
            return format!("refused[{text}]");
        }
        Ok(mounted) => mounted,
    };
    unreadable.lift();
    let effective = (
        mounted.output.effective_root.instance.0,
        mounted.output.effective_root.checkpoint_txg.0,
    );
    let previous = mounted.current.file_version().expect("带文件").clone();
    let params = parameters();
    let published = {
        let mut writer = PoolWriter::new(&params, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut mounted.allocator,
            &previous,
            FirstFile {
                content: &e_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 900,
            },
            mounted.output.instance,
        )
    };
    let e = match published {
        Ok(output) => output,
        Err(error) => return format!("writable_effective=({},{}) publish_err[{error:?}]", effective.0, effective.1),
    };
    mounted.current = PoolVersion::WithFile(e.clone());
    let after = snapshot_of(&devices);
    let view = mount_read_only(&after).ok().map(|read_only| {
        let bytes = read_only
            .mounted
            .open_file(&after, InodeNumber(FIRST_INODE_NUMBER))
            .ok()
            .and_then(|file| {
                file.read_at(&after, FileOffsetInBytes(0), u64::try_from(e_content().len()).expect("长度"))
                    .ok()
            })
            .map(|out| out.bytes)
            .unwrap_or_default();
        (read_only.effective_root.instance.0, read_only.effective_root.checkpoint_txg.0, bytes == e_content())
    });
    let older_lost = effective.1 < newest_acknowledged_txg;
    let e_kept = matches!(view, Some((instance, txg, true)) if (instance, txg) == (e.root.instance.0, e.root.checkpoint_txg.0));
    let verdict = match (older_lost, e_kept) {
        (false, true) => "writable_no_loss",
        (true, true) => "writable_LOST_older_acknowledged",
        (false, false) => "writable_LOST_new_instance",
        (true, false) => "writable_LOST_both",
    };
    format!("{verdict} effective=({},{}) e=({},{}) final={view:?}", effective.0, effective.1, e.root.instance.0, e.root.checkpoint_txg.0)
}

fn verdict_word(line: &str) -> String {
    line.split(|character: char| character == ' ' || character == '[')
        .next()
        .unwrap_or("")
        .to_string()
}

/// 录一条流：在 `base` 上起内存盘（外面按 `faults` 包读故障），跑 `action`，交回这条流的写表与段。
fn record_stream(
    base: &MemoryPool,
    faults: Vec<UnreadableRange>,
    action: &mut dyn FnMut(&mut FaultyDevices) -> String,
) -> (String, Vec<singlefs_harness::memory_pool::RetainedWrite>, Vec<Vec<usize>>) {
    let stream = SharedStream::retaining_contents();
    let unreadable = SharedUnreadableRanges::new(faults, UnreadableRangeReadBack::DeviceError);
    let mut devices: FaultyDevices =
        with_unreadable_ranges(crash_state_devices(base, &[], &[], &stream), &unreadable);
    let outcome = action(&mut devices);
    let operations = stream.retained_operations();
    let (writes, segments) = writes_and_segments(&operations, &geometry());
    (outcome, writes, segments)
}

/// 一条流按层 0 的枚举域全量展开，每个崩溃状态跑 `observations` 里的每一样观察，逐状态打一行，最后按观察汇总。
fn enumerate_and_observe(
    stream_name: &str,
    base: &MemoryPool,
    writes: &[singlefs_harness::memory_pool::RetainedWrite],
    segments: &[Vec<usize>],
    versions: &[PublishedVersion],
    observations: &[(&str, &dyn Fn(&MemoryPool) -> String)],
    states_budget: &mut u64,
) {
    let two_state_count = closed_form_state_count(segments);
    let full_expansion = |_segment_index: usize, _segment: &[usize]| Layer0SegmentExpansion::EveryProperSubset;
    let layer0_count = layer0_state_count_with_torn_in_place_overwrites(base, writes, segments, &full_expansion);
    println!(
        "name=crashproto-plan stream={stream_name} writes={} segments={} closed_form_state_count={two_state_count} layer0_state_count_with_torn={layer0_count}",
        writes.len(),
        segments.len()
    );
    let tearable = singlefs_checker_tier::crash::TearableInPlaceOverwrites::of(base, writes);
    for (segment_index, segment) in segments.iter().enumerate() {
        let kinds: Vec<String> = segment
            .iter()
            .map(|write_index| {
                let write = &writes[*write_index];
                format!(
                    "{}:{:?}@d{}+{}{}",
                    write_index,
                    write.kind,
                    write.device.0,
                    write.offset.0,
                    if tearable.contains(*write_index) { ":tearable" } else { "" }
                )
            })
            .collect();
        println!(
            "name=crashproto-segment stream={stream_name} segment={segment_index} writes={} kinds={}",
            segment.len(),
            if segment.len() <= 6 { kinds.join(",") } else { format!("{} writes ({} tearable)", segment.len(), segment.iter().filter(|index| tearable.contains(**index)).count()) }
        );
    }
    if layer0_count > 1_000_000 {
        println!(
            "name=crashproto-skip stream={stream_name} reason=layer0_state_count_over_10^6 layer0_state_count_with_torn={layer0_count}"
        );
        return;
    }
    *states_budget += layer0_count;
    let Some(judged_root_index) = writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua) else {
        println!("name=crashproto-skip stream={stream_name} reason=no_root_write_in_the_stream");
        return;
    };
    let mut tallies: BTreeMap<(String, String), u64> = BTreeMap::new();
    let mut state_index = 0u64;
    let mut observe = |image: &CrashImage<'_>, _report: &singlefs_core::recovery::RecoveryReport| {
        let stream = SharedStream::new();
        let devices = crash_state_devices(image.base, image.writes, &image.persisted, &stream);
        let state_image = MemoryPool {
            devices: devices
                .iter()
                .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
                .collect(),
            device_size_in_bytes: common::IMAGE_BYTES,
        };
        let persisted: String = image
            .persisted
            .iter()
            .map(|is_persisted| if *is_persisted { '1' } else { '0' })
            .collect();
        for (observation_name, observation) in observations {
            let line = observation(&state_image);
            *tallies
                .entry(((*observation_name).to_string(), verdict_word(&line)))
                .or_insert(0) += 1;
            println!(
                "name=crashproto stream={stream_name} state={state_index} persisted={persisted} observation={observation_name} {line}"
            );
        }
        state_index += 1;
    };
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        &|_segment_index, _segment| true,
        &mut observe,
    );
    println!(
        "name=crashproto-summary stream={stream_name} states_enumerated={} states_observed={state_index} tallies={tallies:?}",
        tally.states
    );
}

fn main() {
    let (standard, versions, c, allocator_after_c) = standard_history();
    let c_txg = c.root.checkpoint_txg.0;
    let c_counter = c.record.counter;
    let mut states_budget = 0u64;
    // S1 / S2：C 之后实例 2 在同一个进程里接着发 D（txg 8）：读得出时，与两块盘最新那一槽系统配置在 D 发布期间读不出时
    // （D 的轮换按「读得出的最大世代号 + 1」落回见证 C 的那一槽）。
    for (stream_name, faults) in [
        ("d_publish", Vec::new()),
        (
            "d_publish_while_the_newest_slots_are_unreadable",
            newest_slot_ranges(&standard, FailingReadsOfARange::Every),
        ),
    ] {
        let mut d_root: Option<(u64, u64)> = None;
        let (outcome, writes, segments) = record_stream(&standard, faults, &mut |devices| {
            let params = parameters();
            let mut allocator = allocator_after_c.clone();
            let mut writer = PoolWriter::new(&params, devices.as_mut_slice());
            match publish_overwrite(
                &mut writer,
                &mut allocator,
                &c,
                FirstFile {
                    content: &d_content(),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 180,
                },
                InstanceGeneration(2),
            ) {
                Ok(d) => {
                    d_root = Some((d.root.checkpoint_txg.0, d.record.counter));
                    format!("D=({},{}) jsn {}", d.root.instance.0, d.root.checkpoint_txg.0, d.record.counter)
                }
                Err(error) => format!("D err {error:?}"),
            }
        });
        println!("name=crashproto-stream stream={stream_name} outcome=[{outcome}]");
        let (d_txg, d_counter) = d_root.expect("D 发完");
        let mut stream_versions = versions.clone();
        stream_versions.push(PublishedVersion {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(d_txg),
            content: d_content(),
        });
        let clean = |image: &MemoryPool| mount_and_judge(image, Vec::new(), c_txg);
        let hidden = |image: &MemoryPool| {
            let mut ranges = root_and_record_ranges(c_txg, c_counter);
            ranges.extend(root_and_record_ranges(d_txg, d_counter));
            mount_and_judge(image, ranges, c_txg)
        };
        enumerate_and_observe(
            stream_name,
            &standard,
            &writes,
            &segments,
            &stream_versions,
            &[("clean", &clean), ("hidden_c_and_d", &hidden)],
            &mut states_budget,
        );
    }
    // S3：V4 的第 k 次挂载——两块盘最新那一槽系统配置按 `timing` 读不出，整次挂载（取号、写行、暖机）照录；
    // 它的每个崩溃状态上做第 k + 1 次挂载，C 的根与记录读不出。
    for (stream_name, timing) in [
        ("v4_mount_k_newest_slots_every_read", FailingReadsOfARange::Every),
        ("v4_mount_k_newest_slots_from_the_5th_read", FailingReadsOfARange::FromTheNthOnward(5)),
    ] {
        let mut mount_versions = versions.clone();
        let (outcome, writes, segments) = record_stream(
            &standard,
            newest_slot_ranges(&standard, timing),
            &mut |devices| match mount_writable(&parameters(), devices) {
                Ok(mounted) => {
                    for version in std::iter::once(&mounted.output.row_publish)
                        .chain(mounted.output.warm_up_publishes.iter())
                    {
                        mount_versions.push(PublishedVersion {
                            instance: version.root().instance,
                            checkpoint_txg: version.root().checkpoint_txg,
                            content: second_instance_content(1),
                        });
                    }
                    format!(
                        "Ok instance {} effective ({},{})",
                        mounted.output.instance.0,
                        mounted.output.effective_root.instance.0,
                        mounted.output.effective_root.checkpoint_txg.0
                    )
                }
                Err(error) => format!("Err {}", format!("{error:?}").chars().take(100).collect::<String>()),
            },
        );
        println!("name=crashproto-stream stream={stream_name} outcome=[{outcome}] writes={}", writes.len());
        let hidden_c = |image: &MemoryPool| mount_and_judge(image, root_and_record_ranges(c_txg, c_counter), c_txg);
        enumerate_and_observe(
            stream_name,
            &standard,
            &writes,
            &segments,
            &mount_versions,
            &[("hidden_c", &hidden_c)],
            &mut states_budget,
        );
    }
    println!("name=crashproto-total states_budget={states_budget}");
}
