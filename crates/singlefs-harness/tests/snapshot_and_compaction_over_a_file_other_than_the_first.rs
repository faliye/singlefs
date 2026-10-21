//! 快照与整理搬迁对第一个文件之外的文件（代码轮判决 `research/prompts/m3-close-w5-code-r1-main-verification.md` 第三节第 2 条；
//! 正推腿 `m3-close-w5-code-r1-sonnet-output.md` W5 节的缺口 C、D）：
//! - 快照下覆盖写别的号：被活快照引用的那几个单元（它的数据单元与下段节点，生在建快照之前）进 deadlist、映射条目留存，不进这次的释放
//!   （D5（快照 / 空间记账机制） 已定项 19「birth ≤ p：进 deadlist」；`transaction::asks_the_snapshot_lifetime_judgement` 加的两个成员）。
//! - 整理的一批搬别的号的数据单元：内存里那一版的指针换成映射位置（`transaction::take_the_mapping_positions_over_the_location_hints` 新加的那一段，
//!   C417 第一版政策「随重写刷新」），源落点两盘都清零之后照样读得回，之后一次写第一个文件重写上段时装的是映射位置。
//!
//! 都在同一个进程里直调发布入口；期望值从被测之前那一版现算（映射 key 与它的落点）。
mod common;

use std::collections::BTreeSet;

use common::{build_pool, parameters, BuiltPool};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber,
    InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::Placement;
use singlefs_core::compaction_plan::PlannedRelocation;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::pointer::LocationEntry;
use singlefs_core::transaction::{
    central_mapping_locations_in_the_version, publish_create_snapshot, publish_new_inodes,
    publish_relocation_batch, publish_sequential_write_of_an_inode, MappingLookup, PoolWriter,
    TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::MemoryPool;

const INSTANCE: InstanceGeneration = InstanceGeneration(1);
const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
/// 建 inode 与按 inode 号写那几次的写入时间：晚于第一个文件。
const WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE: u64 = common::FIXED_WRITE_TIME_SECONDS + 600;
/// 被测的那个第一个文件之外的号：第一个文件之后建的第一个。
const THE_OTHER_INODE: u64 = 2;

/// 定长的伪随机字节流（xorshift64，种子取奇数）：`data_units` 个数据单元差 5 个字节写满。
fn content_of_data_units(data_units: usize, seed: u64) -> Vec<u8> {
    let mut state = seed | 1;
    (0..data_units * data_unit_payload_capacity() - 5)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            u8::try_from(state & 0xff).expect("低 8 位")
        })
        .collect()
}

/// 新池新建文件之后建一个 inode（号 2）、给它顺序写 `data_units` 个数据单元。
fn pool_with_another_file_of(tag: &str, data_units: usize, seed: u64) -> (BuiltPool, Vec<u8>) {
    let mut pool = build_pool(tag);
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    pool.output = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &previous,
            1,
            WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE,
            INSTANCE,
        )
        .expect("建 inode 2")
    };
    let content = content_of_data_units(data_units, seed);
    sequential_write_and_advance(&mut pool, THE_OTHER_INODE, &content);
    (pool, content)
}

/// 同一个进程里按 inode 号顺序写一次，池接到它那一版上。
fn sequential_write_and_advance(
    pool: &mut BuiltPool,
    inode: u64,
    content: &[u8],
) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let written = publish_sequential_write_of_an_inode(
        &mut writer,
        &mut pool.allocator,
        &previous,
        InodeNumber(inode),
        content,
        WRITE_TIME_SECONDS_AFTER_THE_FIRST_FILE,
        INSTANCE,
    )
    .unwrap_or_else(|refusal| panic!("按 inode 号写 inode {inode}：{refusal:?}"));
    pool.output = written.clone();
    written
}

fn mapping_locations_of(version: &TransactionOutput, mapping_key: &[u8]) -> [LocationEntry; 2] {
    match central_mapping_locations_in_the_version(version, mapping_key) {
        MappingLookup::Found(locations) => locations,
        MappingLookup::NodeMalformedOrNoEntryWithThisKey
        | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => {
            panic!("这把 key 在这一版的映射里")
        }
    }
}

/// 一版里 inode 2 的数据单元与下段节点各自的映射 key。
fn mapping_keys_of_the_other_inode(version: &TransactionOutput) -> Vec<(TransactionUnit, Vec<u8>)> {
    version
        .mapped_units
        .iter()
        .filter(|(role, _)| {
            matches!(
                role,
                TransactionUnit::DataOfAFileOtherThanTheFirst { inode, .. }
                | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { inode, .. }
                    if inode.0 == THE_OTHER_INODE
            )
        })
        .cloned()
        .collect()
}

/// 只读挂载这张镜像、打开 `inode`、从偏移 0 读整个文件。
fn read_back(image: &MemoryPool, inode: u64) -> Vec<u8> {
    let mounted = mount_read_only(image).expect("只读挂载");
    let file = mounted
        .mounted
        .open_file(image, InodeNumber(inode))
        .unwrap_or_else(|failure| panic!("打开 inode {inode}：{failure:?}"));
    let size = file.inode_record().size;
    file.read_at(image, FileOffsetInBytes(0), size)
        .unwrap_or_else(|failure| panic!("读 inode {inode}：{failure:?}"))
        .bytes
}

fn violations_of_the_pool_checker(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

/// 快照下覆盖写第一个文件之外的 inode 2（两个数据单元、一个下段叶，都生在建快照之前）：它们的映射 key 每一把都进 deadlist、映射条目留存，
/// 落点不在这次的释放清单里；inode 2 读回新写的那一份，池级 checker 一条违例都没有。
/// 判别力：别的号的两种角色不问快照生死（`crates/mutations.tsv` 以「快照 × 别的文件：」起头那一行）时它们照旧立即释放、deadlist 一条不写，这一条红。
#[test]
fn overwriting_another_file_under_a_live_snapshot_puts_its_replaced_units_on_the_deadlist() {
    let (mut pool, _) = pool_with_another_file_of("snapshot-over-another-file", 2, 0x2701);
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let created = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_create_snapshot(&mut writer, &mut pool.allocator, &previous, INSTANCE)
            .expect("建快照")
    };
    pool.output = created.clone();
    let referenced_by_the_snapshot = mapping_keys_of_the_other_inode(&created);
    assert_eq!(
        referenced_by_the_snapshot
            .iter()
            .map(|(role, _)| *role)
            .collect::<BTreeSet<_>>()
            .len(),
        3,
        "快照引用着 inode 2 的两个数据单元与一个下段叶"
    );
    let slots_referenced_by_the_snapshot: BTreeSet<SlotNumber> = referenced_by_the_snapshot
        .iter()
        .map(|(_, mapping_key)| mapping_locations_of(&created, mapping_key)[0].slot)
        .collect();
    let content = content_of_data_units(2, 0x2702);
    let overwritten = sequential_write_and_advance(&mut pool, THE_OTHER_INODE, &content);
    for (role, mapping_key) in &referenced_by_the_snapshot {
        assert!(
            overwritten
                .deadlist_entries
                .iter()
                .any(|entry| &entry.mapping_key == mapping_key),
            "{role:?} 进 deadlist"
        );
        assert!(
            overwritten
                .mapping_entries_kept_for_the_snapshots
                .iter()
                .any(|(kept_key, _)| kept_key == mapping_key),
            "{role:?} 的映射条目留存"
        );
    }
    assert!(
        overwritten
            .released
            .iter()
            .all(|placement| !slots_referenced_by_the_snapshot.contains(&placement.slot)),
        "被快照引用的落点不在这次的释放清单里：{:?}",
        overwritten.released
    );
    let image = pool.memory_pool();
    assert_eq!(
        read_back(&image, THE_OTHER_INODE),
        content,
        "inode 2 读回新写的"
    );
    assert_eq!(
        violations_of_the_pool_checker(&image),
        Vec::new(),
        "快照下覆盖写 inode 2 之后"
    );
}

/// 整理的一批搬第一个文件之外的 inode 2 那一个数据单元（内联在上段叶条目里）：这一批写出的那一版里 inode 2 的数据指针就是映射里的目的地，
/// 源落点两盘都清零之后只读挂载照样读回 inode 2；之后写一次第一个文件（上段第 0 片叶重写、inode 2 的条目照内存里的指针重装），那一版里 inode 2 的
/// 指针仍是目的地。池级 checker 在清零之前判一遍，一条违例都没有。
/// 判别力：映射位置盖过提示那一步不管别的号（`crates/mutations.tsv` 以「整理搬迁 × 别的文件：」起头那一行）时内存里那一版的指针还是源落点，这一条红。
#[test]
fn a_relocation_batch_moving_the_data_unit_of_another_file_takes_its_mapping_position_over_the_hint(
) {
    let (mut pool, content) = pool_with_another_file_of("relocation-of-another-file", 1, 0x2801);
    let role = TransactionUnit::DataOfAFileOtherThanTheFirst {
        inode: InodeNumber(THE_OTHER_INODE),
        unit: DataUnitIndexInFile::FIRST,
    };
    let (_, mapping_key) = pool
        .output
        .mapped_units
        .iter()
        .find(|(mapped_role, _)| *mapped_role == role)
        .cloned()
        .expect("inode 2 的数据单元进映射");
    let source = mapping_locations_of(&pool.output, &mapping_key);
    let planned = PlannedRelocation {
        mapping_key: mapping_key.clone(),
        source: Placement {
            slot: source[0].slot,
            span: role.span_slots(),
        },
    };
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let batch = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_relocation_batch(
            &mut writer,
            &mut pool.allocator,
            &previous,
            std::slice::from_ref(&planned),
            INSTANCE,
        )
        .expect("搬 inode 2 的数据单元那一批")
    };
    pool.output = batch.clone();
    let destination = mapping_locations_of(&batch, &mapping_key);
    assert_ne!(destination[0].slot, source[0].slot, "目的地不是源落点");
    let pointer_of_the_other_inode = |version: &TransactionOutput| {
        version
            .file_other_than_the_first(InodeNumber(THE_OTHER_INODE))
            .expect("inode 2 有数据单元")
            .data_pointers[0]
            .locations
    };
    assert_eq!(
        pointer_of_the_other_inode(&batch),
        destination,
        "这一批写出的那一版里 inode 2 的数据指针是映射位置"
    );
    assert_eq!(
        violations_of_the_pool_checker(&pool.memory_pool()),
        Vec::new(),
        "搬完那一批"
    );
    let first_file_content = content_of_data_units(1, 0x1801);
    let rewritten =
        sequential_write_and_advance(&mut pool, FIRST_INODE_NUMBER, &first_file_content);
    assert_eq!(
        pointer_of_the_other_inode(&rewritten),
        destination,
        "写第一个文件那一版里 inode 2 的指针仍是映射位置（上段叶照它重装）"
    );
    let mut image = pool.memory_pool();
    for device in BOTH_DEVICES {
        image.devices.get_mut(&device).expect("两块盘").zero_fill(
            DeviceOffsetInBytes(source[0].slot.0 * SLOT_BYTES),
            role.span_slots() * SLOT_BYTES,
        );
    }
    assert_eq!(
        (
            read_back(&image, THE_OTHER_INODE),
            read_back(&image, FIRST_INODE_NUMBER),
        ),
        (content, first_file_content),
        "源落点清零之后 inode 2 照样从目的地读回，第一个文件读回新写的"
    );
}
