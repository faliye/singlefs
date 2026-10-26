//! m4-validate-code-r1 攻方腿在冻结副本拷贝上造的用例（只在副本里，不进仓）。
//! 每条测试断言的是「正确的那一形」，红了就是打中：
//! - 标签 0 配 size > 0 的第一个文件（读者三处都收的合法镜像，K5 / S1）可写挂载之后改 size：应交类型化结局，不该 panic。
//! - 变长到 1 TiB（K12 上限之内）之后：挂载态读者、可写重挂、冷走读与冷恢复各自的结局。
//! - RefFS 截到 1 TiB。
//! - 不带 incompat 位 2 的池（0x02）可写挂载、截断写字段之后各槽的位与重读。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_truncation;

use std::panic::{catch_unwind, AssertUnwindSafe};

use common::{build_pool, parameters};
use common_deletion_in_progress::{change_the_unit_and_propagate_its_checksum, pool_with_a_three_unit_file};
use common_pointer_head_corruption::{
    first_upper_leaf_entry, newest_root, node_bytes, the_slot_of, tree_table_entry_of_kind,
    unit_bytes_at,
};
use common_truncation::{change_the_size_of_the_first_file_and_advance, payload_capacity, FIRST_FILE};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::extent_tree::ExtentUpperLeafTarget;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::{mount_read_only, open_pool_for_read};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::records::{parse_mapping_entry, TREE_KIND_EXTENT};
use singlefs_core::recovery::{recover, walk_to_file, JournalPolicy};
use singlefs_core::system_configuration::{SystemConfiguration, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET};
use singlefs_core::transaction::FIRST_INODE_NUMBER;
use singlefs_core::unit::parse_index_node;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_reference_filesystem::{FileContent, Operation, OperationOutcome, ReferenceFileSystem};

const WRITE_TIME_SECONDS: u64 = 1_788_000_300;
const ONE_TEBIBYTE: u64 = 1 << 40;

fn devices_of(image: &MemoryPool) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device =
                SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect()
}

fn image_of(devices: &[(DeviceIdentity, SparseBlockDevice)], device_size_in_bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes,
    }
}

fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|text| (*text).to_string()))
        .unwrap_or_else(|| "（panic 载荷不是字符串）".to_string())
}

// ── 标签 0 的镜像：照 tests/whole_file_hole_of_upper_leaf_tag_zero_with_a_positive_size_read_by_every_reader.rs 的手造办法 ──

const EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET: usize = 8;
const EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET: usize = 24;
const INDEX_NODE_DECLARED_LENGTH_OFFSET: usize = 8;
const INDEX_NODE_LEVEL_OFFSET: usize = 50;
const INDEX_NODE_KEY_RANGE_OFFSET: usize = 52;

fn read_u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

fn rewrite_entries(
    image: &mut MemoryPool,
    slot: singlefs_core::address::SlotNumber,
    key_range_follows_the_entries: bool,
    change: impl FnOnce(&mut Vec<Vec<u8>>),
) {
    let parsed = parse_index_node(&unit_bytes_at(image, slot, node_bytes())).expect("节点解得开");
    let key_width = parsed.key_width;
    let entry_width = parsed.entry_width;
    let mut entries = parsed.entries;
    change(&mut entries);
    change_the_unit_and_propagate_its_checksum(image, slot, node_bytes(), |node| {
        let entries_start = 115 + 2 * key_width;
        node[entries_start..].fill(0);
        for (position, entry) in entries.iter().enumerate() {
            let start = entries_start + position * entry_width;
            node[start..start + entry_width].copy_from_slice(entry);
        }
        let count_offset = 82 + 2 * key_width;
        node[count_offset..count_offset + 2]
            .copy_from_slice(&u16::try_from(entries.len()).expect("条目数").to_le_bytes());
        node[INDEX_NODE_DECLARED_LENGTH_OFFSET..INDEX_NODE_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
            &u16::try_from(entries.len() * entry_width).expect("声明长度").to_le_bytes(),
        );
        if let (true, Some(first), Some(last)) =
            (key_range_follows_the_entries, entries.first(), entries.last())
        {
            let smallest = INDEX_NODE_KEY_RANGE_OFFSET;
            let largest = INDEX_NODE_KEY_RANGE_OFFSET + key_width;
            node[smallest..smallest + key_width].copy_from_slice(&first[..key_width]);
            node[largest..largest + key_width].copy_from_slice(&last[..key_width]);
        }
    });
}

fn image_whose_first_file_is_a_whole_file_hole(tag: &str) -> MemoryPool {
    let pool = build_pool(tag);
    let mut image = pool.memory_pool();
    let ExtentUpperLeafTarget::InlineDataUnit(data_unit) = first_upper_leaf_entry(&image).target else {
        panic!("前提：新池新建文件那一版第一个文件是标签 2")
    };
    let mapping_root_slot = the_slot_of(&newest_root(&image).mapping_root);
    assert_eq!(unit_bytes_at(&image, mapping_root_slot, node_bytes())[INDEX_NODE_LEVEL_OFFSET], 0);
    rewrite_entries(&mut image, mapping_root_slot, true, |entries| {
        entries.retain(|entry| {
            let (_key, locations) = parse_mapping_entry(entry).expect("映射条目");
            locations[0].slot != data_unit.locations[0].slot
        });
    });
    let extent_root_slot = the_slot_of(&tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root);
    rewrite_entries(&mut image, extent_root_slot, false, |entries| {
        let first_file_entry = entries
            .iter_mut()
            .find(|entry| read_u64_at(entry, EXTENT_UPPER_LEAF_ENTRY_INODE_OFFSET) == FIRST_INODE_NUMBER)
            .expect("第一个文件那一条");
        first_file_entry[EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET] = 0;
        first_file_entry[EXTENT_UPPER_LEAF_ENTRY_TAG_OFFSET + 1..].fill(0);
    });
    assert_eq!(first_upper_leaf_entry(&image).target, ExtentUpperLeafTarget::NoDataUnit);
    image
}

/// 标签 0、size 3000 的第一个文件可写挂载、会话里一次改 size：交回结局的文字（panic 也收成文字）。
fn session_size_change_on_a_whole_file_hole(tag: &str, new_size: u64) -> String {
    let image = image_whose_first_file_is_a_whole_file_hole(tag);
    let mut devices = devices_of(&image);
    let mounted = match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => mounted,
        Err(refusal) => return format!("可写挂载拒：{refusal:?}"),
    };
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        session.publish_user_change(
            &mut devices,
            UserChange::Truncate {
                inode: FIRST_FILE,
                new_size,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
        )
    }));
    match outcome {
        Err(payload) => format!("panic：{}", panic_text(payload.as_ref())),
        Ok(Ok(published)) => format!("做成：{} 版", published.versions_published.iter().count()),
        Ok(Err(refused)) => format!("拒：{}", format!("{refused:?}").chars().take(300).collect::<String>()),
    }
}

fn assert_no_panic(outcome: &str, what: &str) {
    eprintln!("ATTACK-OUTCOME {what}: {outcome}");
    assert!(!outcome.starts_with("panic"), "{what}：{outcome}");
}

#[test]
fn a_whole_file_hole_of_tag_zero_shrunk_by_the_session_does_not_panic() {
    let outcome = session_size_change_on_a_whole_file_hole("attack-tag-zero-shrink", 100);
    assert_no_panic(&outcome, "标签 0 size 3000 截到 100");
}

#[test]
fn a_whole_file_hole_of_tag_zero_kept_at_its_size_by_the_session_does_not_panic() {
    let outcome = session_size_change_on_a_whole_file_hole("attack-tag-zero-same", 3000);
    assert_no_panic(&outcome, "标签 0 size 3000 改成 3000");
}

#[test]
fn a_whole_file_hole_of_tag_zero_extended_by_the_session_does_not_panic() {
    let outcome = session_size_change_on_a_whole_file_hole("attack-tag-zero-extend", 3 * payload_capacity());
    assert_no_panic(&outcome, "标签 0 size 3000 变长到三个净荷");
}

#[test]
fn a_whole_file_hole_of_tag_zero_overwritten_by_the_session_does_not_panic() {
    let image = image_whose_first_file_is_a_whole_file_hole("attack-tag-zero-overwrite");
    let mut devices = devices_of(&image);
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let content = vec![7u8; 100];
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        session.publish_user_change(
            &mut devices,
            UserChange::OverwriteOfAnInode {
                inode: FIRST_FILE,
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
        )
    }));
    let text = match outcome {
        Err(payload) => format!("panic：{}", panic_text(payload.as_ref())),
        Ok(Ok(published)) => format!("做成：{} 版", published.versions_published.iter().count()),
        Ok(Err(refused)) => format!("拒：{}", format!("{refused:?}").chars().take(300).collect::<String>()),
    };
    assert_no_panic(&text, "标签 0 size 3000 覆盖写 100 字节");
}

// ── 变长到 1 TiB（K12 上限之内，洞不写单元） ──

fn pool_extended_to_one_tebibyte(tag: &str) -> common::BuiltPool {
    let mut pool = pool_with_a_three_unit_file(tag);
    let extended = change_the_size_of_the_first_file_and_advance(&mut pool, ONE_TEBIBYTE);
    assert_eq!(extended.inode_record.expect("记录").size, ONE_TEBIBYTE);
    assert_eq!(extended.data_pointers.len(), 3, "变长不加单元：之后到 size 是洞");
    pool
}

#[test]
fn the_mounted_reader_reads_the_tail_of_a_one_tebibyte_hole_as_zeros() {
    let pool = pool_extended_to_one_tebibyte("attack-huge-mounted");
    let image = pool.memory_pool();
    let root = newest_root(&image);
    let mounted = open_pool_for_read(&image, &root).expect("打开挂载态");
    let file = mounted.open_file(&image, FIRST_FILE).expect("打开第一个文件");
    let read = file
        .read_at(&image, FileOffsetInBytes(ONE_TEBIBYTE - 10), 10)
        .expect("读洞的末尾");
    assert_eq!(read.bytes, vec![0u8; 10]);
    eprintln!("ATTACK-OUTCOME 挂载态读者读 1 TiB 文件末尾 10 字节：全 0，读盘 {} 个单元", read.observation.data_units_read);
}

#[test]
fn a_one_tebibyte_hole_remounts_writable_and_shrinks_back() {
    let pool = pool_extended_to_one_tebibyte("attack-huge-remount");
    let image = pool.memory_pool();
    let mut devices = devices_of(&image);
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写重挂");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let published = session
        .publish_user_change(
            &mut devices,
            UserChange::Truncate {
                inode: FIRST_FILE,
                new_size: 100,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
        )
        .expect("截回 100");
    eprintln!("ATTACK-OUTCOME 1 TiB 文件可写重挂截回 100：做成 {} 版", published.versions_published.iter().count());
}

/// 冷走读把整份内容交成一个 `Vec<u8>`，洞补到 size（`recovery::content_padded_with_the_hole_up_to_the_size`）：1 TiB 要分 1 TiB。
#[test]
fn the_cold_walk_of_a_one_tebibyte_hole_returns() {
    let pool = pool_extended_to_one_tebibyte("attack-huge-cold-walk");
    let image = pool.memory_pool();
    let root = newest_root(&image);
    eprintln!("ATTACK-OUTCOME 冷走读开始");
    let mut mapping_fallbacks = 0;
    let content = walk_to_file(&image, &root, &mut mapping_fallbacks).expect("冷走读");
    eprintln!("ATTACK-OUTCOME 冷走读交回 {:?} 字节", content.map(|bytes| bytes.len()));
}

#[test]
fn the_cold_recovery_of_a_one_tebibyte_hole_returns() {
    let pool = pool_extended_to_one_tebibyte("attack-huge-recover");
    let image = pool.memory_pool();
    eprintln!("ATTACK-OUTCOME 冷恢复开始");
    let report = recover(&image, JournalPolicy::Consult);
    eprintln!("ATTACK-OUTCOME 冷恢复交回 {}", format!("{:?}", report.outcome).chars().take(120).collect::<String>());
}

#[test]
fn the_reference_filesystem_truncates_to_one_tebibyte() {
    let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
    model
        .submit(Operation::CreateTheFirstFile(FileContent(b"ab".to_vec())), OperationOutcome::Succeeded)
        .expect("建第一个文件");
    eprintln!("ATTACK-OUTCOME RefFS 截到 1 TiB 开始");
    model
        .submit(Operation::Truncate(ONE_TEBIBYTE), OperationOutcome::Succeeded)
        .expect("截到 1 TiB");
    eprintln!("ATTACK-OUTCOME RefFS 截到 1 TiB 交回");
}

// ── 不带 incompat 位 2 的池（0x02）：可写挂载、截断写字段之后 ──

const FIRST_INCOMPAT_BYTE_OFFSET: usize = 6;

fn system_configuration_slots(image: &MemoryPool) -> Vec<(DeviceIdentity, u64, Vec<u8>)> {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let mut slots = Vec::new();
    for (identity, device) in &image.devices {
        for offset in [0, spacing] {
            slots.push((*identity, offset, device.read(DeviceOffsetInBytes(offset), slot_bytes)));
        }
    }
    slots
}

fn describe_slots(image: &MemoryPool) -> String {
    system_configuration_slots(image)
        .iter()
        .map(|(identity, offset, bytes)| {
            let generation = SystemConfiguration::parse_slot(bytes)
                .map(|configuration| configuration.quantities.slot_generation.to_string())
                .unwrap_or_else(|refusal| format!("{refusal:?}"));
            format!("盘 {} 槽 {offset}: incompat0=0x{:02x} 世代 {generation}", identity.0, bytes[FIRST_INCOMPAT_BYTE_OFFSET])
        })
        .collect::<Vec<_>>()
        .join("；")
}

fn set_the_first_incompat_byte_of_every_slot(image: &mut MemoryPool, value: u8) {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    for device in image.devices.values_mut() {
        for offset in [0, spacing] {
            let mut slot = device.read(DeviceOffsetInBytes(offset), slot_bytes);
            slot[FIRST_INCOMPAT_BYTE_OFFSET] = value;
            let digest = wide_checksum_with_field_zeroed(&slot, slot_bytes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
            slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            device.write(DeviceOffsetInBytes(offset), &slot);
        }
    }
}

#[test]
fn a_pool_without_incompat_bit_two_takes_a_truncation_and_still_mounts() {
    let pool = pool_with_a_three_unit_file("attack-bit-two");
    let mut image = pool.memory_pool();
    set_the_first_incompat_byte_of_every_slot(&mut image, 0x02);
    eprintln!("ATTACK-OUTCOME 改成 0x02 之后：{}", describe_slots(&image));
    mount_read_only(&image).expect("0x02 的池只读挂得上");
    let mut devices = devices_of(&image);
    let mounted = mount_writable(&parameters(), &mut devices).expect("0x02 的池可写挂载");
    let after_the_mount = image_of(&devices, image.device_size_in_bytes);
    eprintln!("ATTACK-OUTCOME 可写挂载之后：{}", describe_slots(&after_the_mount));
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let published = session
        .publish_user_change(
            &mut devices,
            UserChange::Truncate {
                inode: FIRST_FILE,
                new_size: 100,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
        )
        .expect("截到 100");
    let intent = published.versions_published.iter().next().expect("意图那一版");
    let field = intent.inode_record.expect("记录").truncation_field;
    eprintln!(
        "ATTACK-OUTCOME 截到 100：意图那一版字段 T = {}、N_w = {}",
        field.truncation_txg().0,
        field.truncated_size_in_bytes()
    );
    let after_the_truncation = image_of(&devices, image.device_size_in_bytes);
    eprintln!("ATTACK-OUTCOME 截断之后：{}", describe_slots(&after_the_truncation));
    let read_only = mount_read_only(&after_the_truncation).map(|_| ()).map_err(|refusal| format!("{refusal:?}"));
    eprintln!("ATTACK-OUTCOME 截断之后只读挂载：{read_only:?}");
    let mut mapping_fallbacks = 0;
    let cold = walk_to_file(&after_the_truncation, &newest_root(&after_the_truncation), &mut mapping_fallbacks)
        .map(|content| content.map(|bytes| bytes.len()))
        .map_err(|refusal| format!("{refusal:?}"));
    eprintln!("ATTACK-OUTCOME 截断之后冷走读：{cold:?}");
    let mut devices_again = devices_of(&after_the_truncation);
    let writable_again = mount_writable(&parameters(), &mut devices_again).map(|_| ()).map_err(|refusal| format!("{refusal:?}").chars().take(200).collect::<String>());
    eprintln!("ATTACK-OUTCOME 截断之后可写重挂：{writable_again:?}");
    assert!(read_only.is_ok() && cold.is_ok() && writable_again.is_ok());
}

// ── 标签 0 的镜像上别的改动（对照：只有改 size 那一条 panic） ──

fn session_change_on_a_whole_file_hole(tag: &str, change: impl FnOnce() -> UserChange<'static>) -> String {
    let image = image_whose_first_file_is_a_whole_file_hole(tag);
    let mut devices = devices_of(&image);
    let mounted = match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => mounted,
        Err(refusal) => return format!("可写挂载拒：{refusal:?}"),
    };
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let outcome = catch_unwind(AssertUnwindSafe(|| session.publish_user_change(&mut devices, change())));
    match outcome {
        Err(payload) => format!("panic：{}", panic_text(payload.as_ref())),
        Ok(Ok(published)) => format!("做成：{} 版", published.versions_published.iter().count()),
        Ok(Err(refused)) => format!("拒：{}", format!("{refused:?}").chars().take(300).collect::<String>()),
    }
}

#[test]
fn a_whole_file_hole_of_tag_zero_unlinked_by_the_session_does_not_panic() {
    let outcome = session_change_on_a_whole_file_hole("attack-tag-zero-unlink", || UserChange::Unlink {
        inode: FIRST_FILE,
        write_time_seconds: WRITE_TIME_SECONDS,
    });
    assert_no_panic(&outcome, "标签 0 size 3000 unlink");
}

#[test]
fn a_whole_file_hole_of_tag_zero_snapshotted_by_the_session_does_not_panic() {
    let outcome = session_change_on_a_whole_file_hole("attack-tag-zero-snapshot", || UserChange::CreateSnapshot);
    assert_no_panic(&outcome, "标签 0 size 3000 建快照");
}

#[test]
fn a_whole_file_hole_of_tag_zero_with_new_inodes_by_the_session_does_not_panic() {
    let outcome = session_change_on_a_whole_file_hole("attack-tag-zero-new-inodes", || UserChange::NewInodes {
        count: 1,
        write_time_seconds: WRITE_TIME_SECONDS,
    });
    assert_no_panic(&outcome, "标签 0 size 3000 建一个 inode");
}

// ── K10：截断意图那次每盘新占槽数在几种几何上（分配记录树之外，口径同 truncation_intent_batches_and_final_batch.rs 那条单测） ──

fn intent_slots_outside_the_allocation_record_tree(tag: &str, data_units: u64, target: u64) -> u64 {
    let mut pool = build_pool(tag);
    let content: Vec<u8> = (0..data_units * payload_capacity() - payload_capacity() / 2)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect();
    let written = common::sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    assert_eq!(u64::try_from(written.data_pointers.len()).expect("单元数"), data_units);
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, target);
    intent
        .rewritten
        .iter()
        .filter(|role| {
            !matches!(
                role,
                singlefs_core::transaction::TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
                    | singlefs_core::transaction::TransactionUnit::AllocationTree
            )
        })
        .map(|role| role.span_slots())
        .sum()
}

#[test]
fn the_truncation_intent_slots_outside_the_allocation_record_tree_across_geometries() {
    let payload = payload_capacity();
    for (data_units, target, what) in [
        (3, 100, "3 单元截到单元 0 中间"),
        (3, payload + 7, "3 单元截到单元 1 中间"),
        (150, 143 * payload + 21, "150 单元截到单元 143 中间（叶边界）"),
        (300, 1, "300 单元截到单元 0 中间"),
        (300, 287 * payload + 5, "300 单元截到单元 287 中间（叶边界）"),
        (300, 150 * payload, "300 单元截到单元边界 150 P（不重写）"),
    ] {
        let slots = intent_slots_outside_the_allocation_record_tree(
            &format!("attack-intent-slots-{data_units}-{target}"),
            data_units,
            target,
        );
        eprintln!("ATTACK-OUTCOME 意图那次分配记录树之外每盘新占 {slots} 槽：{what}");
        assert!(slots <= singlefs_core::admission::DELETION_PUBLISH_SLOTS_OUTSIDE_THE_ALLOCATION_RECORD_TREE_AT_MOST);
    }
}

// ── 回退之后变长：R_old 的截断字段 (T1, P) 与回退前那一时间线写下、回退放掉而还在盘上的单元（出生 txg 晚于 T1） ──

fn violations_of(image: &MemoryPool) -> Vec<(&'static str, String)> {
    singlefs_checker::walk::check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            singlefs_checker::image::InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            singlefs_checker::image::InvariantVerdict::Holds
            | singlefs_checker::image::InvariantVerdict::NotApplicable(_)
            | singlefs_checker::image::InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

fn run_history_and_check_every_step(history: &singlefs_harness::history::GeneratedHistory, what: &str) -> Vec<String> {
    use singlefs_harness::history::{execute_history_handing_out_every_step_image, HistoryExecution, StepPosition};
    let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
    let fault_plan = singlefs_harness::fault_injection::SharedFaultPlan::unarmed(execution.device_width.fixed_geometry());
    let mut findings = Vec::new();
    let run = execute_history_handing_out_every_step_image(
        history,
        execution,
        &singlefs_harness::SharedStream::new(),
        &fault_plan,
        &mut |_| {},
        &mut |step| {
            let violations = violations_of(step.image);
            let position = match step.position {
                StepPosition::StartingPoint => "起点".to_string(),
                StepPosition::Operation(index) => format!("第 {index} 步"),
            };
            let verdicts: Vec<String> = singlefs_checker::walk::check_pool_image(step.image)
                .into_iter()
                .filter(|(invariant, _)| ["I-1.13", "I-9.39", "I-9.7"].contains(invariant))
                .map(|(invariant, verdict)| format!("{invariant}={}", format!("{verdict:?}").chars().take(80).collect::<String>()))
                .collect();
            eprintln!("ATTACK-VERDICT {what}：{position} {verdicts:?}");
            for (invariant, detail) in violations {
                findings.push(format!("{position} {invariant}：{}", detail.chars().take(400).collect::<String>()));
            }
        },
    );
    eprintln!("ATTACK-OUTCOME {what}：历史结局 {:?}", run.ending);
    for (step, outcome) in run.outcomes.iter().enumerate() {
        eprintln!("ATTACK-OUTCOME {what}：第 {step} 步 {}", format!("{outcome:?}").chars().take(160).collect::<String>());
    }
    for finding in &findings {
        eprintln!("ATTACK-OUTCOME {what}：checker 红 {finding}");
    }
    findings
}

fn rollback_then_extend_history(rollback_index_from_newest: u64, growing_selector: u64) -> singlefs_harness::history::GeneratedHistory {
    use singlefs_harness::history::{GeneratedHistory, HistoryOperation, HistorySeed, HistoryStartingPoint, RollbackTargetChoice, TruncationTarget};
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::PublishOverwrite(common_truncation::content_of_data_units_in_a_history(3, 0)),
            common_truncation::truncate_in_a_history(TruncationTarget::ToADataUnitBoundary, 0),
            HistoryOperation::PublishOverwrite(common_truncation::content_of_data_units_in_a_history(3, 5)),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: rollback_index_from_newest,
            }),
            common_truncation::truncate_in_a_history(TruncationTarget::GrowingAcrossDataUnits, growing_selector),
        ],
    }
}

#[test]
fn rolling_back_to_a_truncated_version_and_extending_keeps_every_step_image_clean() {
    let mut all_findings = Vec::new();
    for rollback_index in [1u64, 2] {
        for growing_selector in [0u64, 1] {
            let what = format!("回退到第 {rollback_index} 新的根、变长选择子 {growing_selector}");
            all_findings.extend(run_history_and_check_every_step(
                &rollback_then_extend_history(rollback_index, growing_selector),
                &what,
            ));
        }
    }
    assert!(all_findings.is_empty(), "{all_findings:?}");
}
