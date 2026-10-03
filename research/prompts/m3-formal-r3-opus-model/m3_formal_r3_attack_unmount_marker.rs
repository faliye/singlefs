//! m3-formal-r3 攻方腿的原型（只在仓副本里跑，不进库）：正常卸载那一串带卸载记号的空发布里，
//! 第一次的根槽已落、第二次的记录已落而根槽没落——D13（验证路线） 已定项 4 意义上的一个不带故障的崩溃状态。
//! 量两件：所选根的卸载记号、`replay_journal` 施加出来的根的卸载记号、第二次发布真正写下的根的卸载记号。
//! 盘用内存稀疏盘（`SparseBlockDevice`），不建镜像文件。

mod common;

use common::{
    crash_state_devices, file_content, geometry, memory_pool_of_sparse_devices, parameters,
    FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, unmount, ShadowLedger, Unmounted};
use singlefs_core::recovery::{choose_root, choose_system_configuration, replay_journal, scan_journal};
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes_and_entries, MemoryPool, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordedEntrySpan, RecordedPublishEntry, RecordingBlockDevice, SharedStream};

const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

#[test]
fn a_crash_between_the_second_unmount_publish_record_and_its_root_slot() {
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = BOTH_DEVICES
        .iter()
        .map(|identity| {
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(
                    *identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let publish_parameters = parameters();
    make_filesystem(&publish_parameters, &mut devices).expect("mkfs");
    let mounted = mount_writable(&publish_parameters, &mut devices).expect("可写挂载");
    let mut allocator = mounted.allocator;
    let content = file_content();
    let first_file = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            mounted.current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            mounted.output.instance,
            mounted.current.record_bytes(),
        )
        .expect("第一个文件版本")
    };
    let operations_before_the_unmount = stream.operations().len();
    let mut current = PoolVersion::WithFile(first_file);
    let unmounted = stream
        .record_entry(RecordedPublishEntry::Unmount, || {
            unmount(
                &publish_parameters,
                &mut devices,
                &mut allocator,
                &mut current,
                ShadowLedger::On,
            )
        })
        .expect("正常卸载");
    let Unmounted::FloorRaisedToTheCurrentVersion(raised) = unmounted else {
        panic!("现行那一版带文件：卸载推一串带卸载记号的空发布")
    };
    println!(
        "ATTACK unmount_publishes={} markers={:?} txgs={:?} counters={:?}",
        raised.publishes.len(),
        raised.publishes.iter().map(|publish| publish.root.unmount_marker).collect::<Vec<_>>(),
        raised.publishes.iter().map(|publish| publish.root.checkpoint_txg.0).collect::<Vec<_>>(),
        raised.publishes.iter().map(|publish| publish.record.counter).collect::<Vec<_>>(),
    );
    let operations = stream.retained_operations();
    let mut base = MemoryPool::with_devices(&BOTH_DEVICES, IMAGE_BYTES);
    base.apply(&operations[..operations_before_the_unmount]);
    let entry_spans: Vec<RecordedEntrySpan> = stream
        .entry_spans()
        .into_iter()
        .map(|span| RecordedEntrySpan {
            entry: span.entry,
            operations: span.operations.start - operations_before_the_unmount
                ..span.operations.end - operations_before_the_unmount,
        })
        .collect();
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &operations[operations_before_the_unmount..],
        &entry_spans,
        &geometry(),
    );
    let kinds: Vec<StepKind> = writes.iter().map(|write| write.kind).collect();
    println!("ATTACK unmount_write_kinds={kinds:?}");
    let root_writes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert!(root_writes.len() >= 2, "卸载那一串至少两次发布：{root_writes:?}");
    let second_root_write = root_writes[1];
    assert!(
        writes[root_writes[0]..second_root_write]
            .iter()
            .any(|write| write.kind == StepKind::JournalRecord),
        "第二次空发布的记录写在第一次的根槽之后、第二次的根槽之前"
    );
    // 崩溃状态：第二次空发布的根槽写之前的写全部持久，根槽与之后的一个都没有。
    let persisted: Vec<bool> = (0..writes.len())
        .map(|write_index| write_index < second_root_write)
        .collect();
    let crash_stream = SharedStream::retaining_contents();
    let crash_devices = crash_state_devices(&base, &writes, &persisted, &crash_stream);
    let image = memory_pool_of_sparse_devices(&crash_devices);
    let system_configuration = choose_system_configuration(&image).expect("系统配置择得出");
    let chosen_root = choose_root(&image, &system_configuration).expect("择得出根");
    let records = scan_journal(&image, &system_configuration);
    let (report, rebuilt_root) = replay_journal(
        &image,
        &chosen_root,
        system_configuration.immutable.sizes.journal_ring_bytes,
        &records,
        true,
    )
    .expect("重放");
    let second = &raised.publishes[1].root;
    println!(
        "ATTACK chosen=({},{}) chosen_marker={:?} rebuilt=({},{}) rebuilt_marker={:?} prefix_applied={} second_publish=({},{}) second_publish_marker={:?}",
        chosen_root.instance.0,
        chosen_root.checkpoint_txg.0,
        chosen_root.unmount_marker,
        rebuilt_root.instance.0,
        rebuilt_root.checkpoint_txg.0,
        rebuilt_root.unmount_marker,
        report.prefix_applied,
        second.instance.0,
        second.checkpoint_txg.0,
        second.unmount_marker,
    );
    println!(
        "ATTACK rebuilt_equals_second_publish_root={} other_fields_equal_chosen=({},{},{},{})",
        rebuilt_root == *second,
        rebuilt_root.filesystem_identifier == chosen_root.filesystem_identifier,
        rebuilt_root.unmount_marker == chosen_root.unmount_marker,
        rebuilt_root.instance_table == chosen_root.instance_table,
        rebuilt_root.allocation_record_tree_root == chosen_root.allocation_record_tree_root,
    );
}
