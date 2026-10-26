//! C152（收窄后未发布号重发的扫描重建复核）：崩溃恢复落到较旧的根之后，被抛弃那一段里发过的 inode 号可以重发
//! （D8（核心索引结构） 已定项 6 射程：唯一性的射程是一条已发布历史内，崩溃恢复抛弃的那一段不在里面）。盘上同时留着被抛弃那一段写下的单元头
//! 与重发之后写的新对象；只走扫描重建（不读树，只读最新根指着的实例表当已发布谓词的输入）重建的三张表要与遍历方向逐项相同——
//! 号对应的只有新的那个对象。池级 checker 的 I-1.13（扫描重建与遍历方向的索引相同） 判这件事，用例看它真判成立（不是不适用）。
//!
//! 两格（欠账 C152 那一行「怎么拦」原文）：
//! - 第一个文件 A（实例 1，txg 3，inode 1）的单元与记录写下了、根没写成就崩：A 的根槽一直是零，A 那条 journal 记录点名的数据单元在那次可写挂载时
//!   读不出、不施加，择根落到暖机那条根（txg 2）；实例 2 写行、暖机，再发第一个文件——inode 1 重发。A 的数据单元头照样在盘上，五元组里 inode 号
//!   与新对象的相同、对象出生代不同。树 ID 不重发：写行那次发布的树 ID 水位取根环里的根与环里记录新根段的 max（`singlefs_core::mount` 的
//!   `tree_identifier_watermark_of_the_ring`），A 那条记录带着发过十三棵树之后的水位。
//! - A 的根落了盘、之后被同一种崩溃恢复抛弃（根槽暂时读不出、挂载之后原样写回：读得出、按实例 2 的实例表判是被抛弃的），实例 2 再发第一个文件：
//!   inode 1 重发，inode 1 的两个对象按 (号, 对象出生代) 分开。「回退」照崩溃恢复那条路造：管理员回退那次发布的水位取 max、不重发号
//!   （D8（核心索引结构） 已定项 6），造不出重发。
//!
//! 判别力：第一格把 A 的数据单元头里的写序抹成零（已发布谓词按实例表行认不出它是被抛弃那一段写的），扫描重建在 inode 1 偏移 0 上见到两个对象，
//! I-1.13 不再成立；两格各由 `crates/mutations.tsv` 里改坏已发布谓词的变异行钉着。第二格欠账行写的「把五元组里的对象出生代抹掉必须红」在今天的
//! 读法下不成立：被抛弃那一版的单元诞生代号晚于实例 1 那一行的已发布 txg，已发布谓词先把它挡在扫描重建的输入之外，对象出生代不参与（实现员
//! 2026-10-26 现跑：抹掉 A 的数据单元头里的对象出生代之后 I-1.13 照样成立，只多红 I-7.4），交主 agent。
//!
//! 搭法照 `tests/tree_identifier_watermark_after_the_first_file_root_is_abandoned.rs` 那一份（私有的，这里照它另写一份，多一个参数：
//! A 的根槽挂载之后写不写回）。
mod common;

use common::{build_pool, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::{InvariantVerdict, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{choose_system_configuration, readable_roots};
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolWriter, TransactionOutput};
use singlefs_core::unit::seal_header_checksum;
use singlefs_format::DATA_UNIT_BYTES;
use singlefs_harness::memory_pool::MemoryPool;

/// 码 1 头里五元组各段与写序的偏移（D18（块里携带什么信息） 已定项 7 的字段表：类标签 42、树 43、对象 51、对象出生代 59、锚点偏移 67；
/// 诞生代号 75、fsid 83、写序 91（实例代号 4 + 事务号 6）、载荷 CRC 101、明文头末尾 105）。
const DATA_UNIT_TREE_OFFSET: usize = 43;
const DATA_UNIT_OBJECT_OFFSET: usize = 51;
const DATA_UNIT_OBJECT_BIRTH_OFFSET: usize = 59;
const DATA_UNIT_WRITE_ORDER_OFFSET: usize = 91;
const DATA_UNIT_WRITE_ORDER_BYTES: usize = 10;
const DATA_UNIT_PAYLOAD_CRC_OFFSET: usize = 101;
const DATA_UNIT_HEADER_BYTES: usize = 105;

/// 实例 2 再发的第一个文件写进记录三个时间秒的值比 A 晚多少秒。
const SECONDS_OF_THE_FIRST_FILE_PUBLISHED_AGAIN_AFTER_THE_FIRST_WRITE: u64 = 300;

/// 实例 2 再发的第一个文件的内容：与 A 的不同，扫描重建认错对象时数据单元集合与中央映射对不上。
const CONTENT_OF_THE_FIRST_FILE_PUBLISHED_AGAIN: &[u8] =
    b"the first file published again by the second instance after the crash recovery";

/// A 的根槽在那次可写挂载之后怎么样。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RootOfTheFirstFileAfterTheFallBack {
    /// 一直是零：A 的单元写下了、根没写成就崩。
    NeverWritten,
    /// 原样写回：A 的根落过盘，读得出、按实例 2 的实例表判是被抛弃的。
    WrittenBackAndAbandoned,
}

/// 搭出来的盘面：A 那一版、实例 2 再发的那一版、之后的镜像。
struct NumberReissuedAfterTheFallBack {
    abandoned_first_file: TransactionOutput,
    first_file_published_again: TransactionOutput,
    image: MemoryPool,
}

fn index_of_device(devices: &[(DeviceIdentity, Recorded)], identity: DeviceIdentity) -> usize {
    devices
        .iter()
        .position(|(candidate, _)| *candidate == identity)
        .expect("池里有这块盘")
}

fn data_unit_bytes() -> usize {
    usize::try_from(DATA_UNIT_BYTES).expect("32768")
}

/// 进程退出之后，A（`pool.output`）的根槽与它的数据单元（两份）暂时读不出（暂存、清零），A 那次发布之后轮换写的系统配置槽
/// （每块盘一槽）写回成轮换之前的那一代（系统配置没见证到 A），重开可写挂载——择根落到暖机那条根，A 那条记录施加前验点名单元失败、不施加；
/// 实例 2 写行与暖机。之后把 A 的数据单元原样写回，A 的根槽按 `root_of_the_first_file` 写回或一直是零；实例 2 再发第一个文件。
fn first_file_published_again_after_a_recovery_that_lands_on_the_warm_up_root(
    tag: &str,
    root_of_the_first_file: RootOfTheFirstFileAfterTheFallBack,
) -> NumberReissuedAfterTheFallBack {
    let mut pool: BuiltPool = build_pool(tag);
    let abandoned_first_file = pool.output.clone();
    let publish_parameters = parameters();
    let target = singlefs_core::root_ring::target_for_publish(
        abandoned_first_file.root.checkpoint_txg,
        publish_parameters.geometry.root_ring_slots_per_region,
    );
    let root_slot = (
        publish_parameters.region_devices[usize::try_from(target.region).expect("区域号")],
        singlefs_core::root_ring::slot_offset(
            target,
            publish_parameters.geometry.fixed_structure_slot_spacing,
        ),
        usize::try_from(publish_parameters.geometry.physical_block_size).expect("根槽宽"),
    );
    // A 的根槽与它的数据单元（两份）在那次可写挂载时暂时读不出，A 那条记录施加前验点名单元失败、不施加（记录照样在环里：它点名的落点
    // 被那次挂载隔离，实例 2 不把 A 的单元槽再分出去，单元头留在盘上）。记录也没落盘的那种崩溃造不出这两格：实例 2 再发第一个文件把
    // A 的数据单元槽原样再分出去、整个盖掉（2026-10-26 实现员在这份用例上现跑：数据单元头读回来就是新对象的），盘上不留孤儿。
    let data_units: Vec<(DeviceIdentity, DeviceOffsetInBytes, usize)> = abandoned_first_file
        .data_pointers
        .iter()
        .flat_map(|pointer| pointer.locations)
        .map(|location| {
            (
                location.device,
                location.slot.to_device_offset(),
                data_unit_bytes(),
            )
        })
        .collect();
    let mut devices = pool.reopen_recorded();
    let mut saved_root_slot = Vec::new();
    let mut saved_data_units = Vec::new();
    for (place, is_the_root_slot) in
        std::iter::once((root_slot, true)).chain(data_units.iter().map(|place| (*place, false)))
    {
        let (identity, offset, length) = place;
        let index = index_of_device(&devices, identity);
        let mut bytes = vec![0u8; length];
        devices[index].1.read_at(offset, &mut bytes).expect("暂存");
        devices[index]
            .1
            .write_at(offset, &vec![0u8; length], WriteDurability::Plain)
            .expect("清零");
        if is_the_root_slot {
            saved_root_slot.push((identity, offset, bytes));
        } else {
            saved_data_units.push((identity, offset, bytes));
        }
    }
    // 系统配置没见证到 A：轮换写那一槽（每块盘世代号最大的那一槽）换成另一槽的内容、世代号改成 g − 2，两槽都自证得过、见证值停在 A 之前
    // （写法与理由同 `tests/common` 的 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`）。
    let slot_spacing_in_bytes = u64::from(publish_parameters.geometry.fixed_structure_slot_spacing);
    let rotation_not_written_of_each_device: Vec<(DeviceOffsetInBytes, Vec<u8>)> = devices
        .iter()
        .map(|(identity, _)| {
            let mut verified_slots = singlefs_core::recovery::verified_system_configuration_slots(
                devices.as_slice(),
                *identity,
                slot_spacing_in_bytes,
                &publish_parameters.filesystem_identifier,
            );
            verified_slots.sort_by_key(|slot| slot.quantities.slot_generation);
            let [older, newest_slot] =
                <[_; 2]>::try_from(verified_slots).expect("A 之前每块盘两槽都自证得过");
            let newest_generation = newest_slot.quantities.slot_generation;
            let mut rotation_not_written = older;
            rotation_not_written.quantities.slot_generation = newest_generation
                .checked_sub(2)
                .expect("A 之前至少还有两次轮换");
            (
                DeviceOffsetInBytes(
                    (newest_generation % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
                        * slot_spacing_in_bytes,
                ),
                rotation_not_written.to_slot(),
            )
        })
        .collect();
    for ((_, device), (newest_slot, rotation_not_written)) in
        devices.iter_mut().zip(rotation_not_written_of_each_device)
    {
        device
            .write_at(newest_slot, &rotation_not_written, WriteDurability::Plain)
            .expect("见证 A 的系统配置槽写回成轮换之前的那一代");
    }
    let mut mounted = mount_writable(&publish_parameters, &mut devices)
        .expect("A 的根与数据单元读不出：择根落到暖机那条根，照常可写挂载");
    let written_back = match root_of_the_first_file {
        RootOfTheFirstFileAfterTheFallBack::NeverWritten => saved_data_units,
        RootOfTheFirstFileAfterTheFallBack::WrittenBackAndAbandoned => saved_data_units
            .into_iter()
            .chain(saved_root_slot)
            .collect(),
    };
    for (identity, offset, bytes) in &written_back {
        let index = index_of_device(&devices, *identity);
        devices[index]
            .1
            .write_at(*offset, bytes, WriteDurability::Plain)
            .expect("原样写回");
    }
    let instance = mounted.output.instance;
    let first_file_published_again = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut mounted.allocator,
            mounted.current.root(),
            FirstFile {
                content: CONTENT_OF_THE_FIRST_FILE_PUBLISHED_AGAIN,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS
                    + SECONDS_OF_THE_FIRST_FILE_PUBLISHED_AGAIN_AFTER_THE_FIRST_WRITE,
            },
            instance,
            mounted.current.record_bytes(),
        )
        .unwrap_or_else(|error| panic!("实例 2 再发第一个文件：{error:?}"))
    };
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator.clone();
    pool.output = first_file_published_again.clone();
    NumberReissuedAfterTheFallBack {
        abandoned_first_file,
        first_file_published_again,
        image: pool.memory_pool(),
    }
}

/// 一版的数据单元（指针里的第一个落点）在镜像上的整单元字节。
fn first_data_unit_of(image: &MemoryPool, version: &TransactionOutput) -> Vec<u8> {
    let location = version
        .data_pointers
        .first()
        .expect("第一个文件有一个数据单元")
        .locations[0];
    image
        .devices
        .get(&location.device)
        .expect("池里有那块盘")
        .read(location.slot.to_device_offset(), data_unit_bytes())
}

fn header_field(unit: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        unit[offset..offset + 8]
            .try_into()
            .expect("头里这一段 8 字节"),
    )
}

/// 五元组里的 (树, 对象, 对象出生代)。
fn tree_object_and_object_birth(unit: &[u8]) -> (u64, u64, u64) {
    (
        header_field(unit, DATA_UNIT_TREE_OFFSET),
        header_field(unit, DATA_UNIT_OBJECT_OFFSET),
        header_field(unit, DATA_UNIT_OBJECT_BIRTH_OFFSET),
    )
}

/// 一版的每个数据单元的两份副本都按 `change` 改头、按码 1 重封（先载荷 CRC，再头校验和）。被改的是盘上没被任何根引用的单元
/// （被抛弃那一段写下的），不用往上补校验和。
fn change_the_data_units_of(
    image: &mut MemoryPool,
    version: &TransactionOutput,
    change: impl Fn(&mut [u8]),
) {
    for location in version
        .data_pointers
        .iter()
        .flat_map(|pointer| pointer.locations)
    {
        let device = image
            .devices
            .get_mut(&location.device)
            .expect("池里有那块盘");
        let mut unit = device.read(location.slot.to_device_offset(), data_unit_bytes());
        change(&mut unit);
        let payload_crc = crc32_castagnoli(&unit[DATA_UNIT_HEADER_BYTES..]);
        unit[DATA_UNIT_PAYLOAD_CRC_OFFSET..DATA_UNIT_PAYLOAD_CRC_OFFSET + 4]
            .copy_from_slice(&payload_crc.to_le_bytes());
        seal_header_checksum(&mut unit, DATA_UNIT_HEADER_BYTES);
        device.write(location.slot.to_device_offset(), &unit);
    }
}

fn verdict_of_the_scan_rebuild(image: &MemoryPool) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(invariant, _)| *invariant == SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX)
        .expect("I-1.13 在池级 checker 的清单里")
        .1
}

/// 搭建自证：A 的数据单元头照样在盘上（被抛弃那一段写下的），与实例 2 再发的那一版 inode 号相同、对象出生代不同。
fn assert_the_number_is_reissued(reissued: &NumberReissuedAfterTheFallBack) {
    let abandoned_unit = first_data_unit_of(&reissued.image, &reissued.abandoned_first_file);
    let published_again_unit =
        first_data_unit_of(&reissued.image, &reissued.first_file_published_again);
    assert_eq!(
        &abandoned_unit[..4],
        b"SFSU",
        "A 的数据单元照样在盘上（实例 2 没把它的槽再分出去）"
    );
    let abandoned = tree_object_and_object_birth(&abandoned_unit);
    let published_again = tree_object_and_object_birth(&published_again_unit);
    assert!(
        abandoned.1 == published_again.1 && abandoned.2 != published_again.2,
        "inode 号重发、对象出生代不同：A 的 (树, 对象, 对象出生代) {abandoned:?}，实例 2 再发的 {published_again:?}"
    );
}

/// 第一格：A 的根没写成就崩，实例 2 重发 inode 1。只走扫描重建，号对应的只有新的那个对象：I-1.13 真判成立。
/// 判别力：已发布谓词不看实例表行（`crates/mutations.tsv`「C152 已发布谓词不看实例表行」）时 A 的单元被当成已发布，这一条红。
#[test]
fn the_scan_rebuild_maps_a_number_reissued_after_a_root_that_never_landed_to_the_new_object() {
    let reissued = first_file_published_again_after_a_recovery_that_lands_on_the_warm_up_root(
        "c152-reissued-after-a-root-that-never-landed",
        RootOfTheFirstFileAfterTheFallBack::NeverWritten,
    );
    assert_the_number_is_reissued(&reissued);
    assert_eq!(
        verdict_of_the_scan_rebuild(&reissued.image),
        InvariantVerdict::Holds,
        "扫描重建只认实例 2 再发的那个对象：三张表与遍历方向逐项相同"
    );
}

/// 第一格的判别力：把 A 的数据单元（两份）头里的写序抹成零、重封。实例代号 0 在实例表里没有行，已发布谓词把它当成更早的实例写下、已发布，
/// 扫描重建在 inode 1 偏移 0 上见到 A 与新对象两个对象身份（对象出生代不同），停下、I-1.13 报不适用——写序是分开孤儿与活版的那一段。
#[test]
fn erasing_the_write_order_of_the_units_left_by_a_root_that_never_landed_stops_the_scan_rebuild_on_two_objects(
) {
    let mut reissued = first_file_published_again_after_a_recovery_that_lands_on_the_warm_up_root(
        "c152-reissued-after-a-root-that-never-landed-write-order-erased",
        RootOfTheFirstFileAfterTheFallBack::NeverWritten,
    );
    assert_the_number_is_reissued(&reissued);
    assert_eq!(
        verdict_of_the_scan_rebuild(&reissued.image),
        InvariantVerdict::Holds,
        "对照：抹写序之前 I-1.13 成立"
    );
    let abandoned_first_file = reissued.abandoned_first_file.clone();
    change_the_data_units_of(&mut reissued.image, &abandoned_first_file, |unit| {
        unit[DATA_UNIT_WRITE_ORDER_OFFSET
            ..DATA_UNIT_WRITE_ORDER_OFFSET + DATA_UNIT_WRITE_ORDER_BYTES]
            .fill(0);
    });
    assert_eq!(
        verdict_of_the_scan_rebuild(&reissued.image),
        InvariantVerdict::NotApplicable(
            "扫描重建：两个不同对象身份的当前版本数据单元落在同一个 (inode, 偏移) 上，按哪一个认没有条款"
        ),
        "写序抹掉之后 A 的单元也算已发布：inode 1 偏移 0 上两个对象，扫描重建停下"
    );
}

/// 第二格：A 的根落了盘、被崩溃恢复抛弃，实例 2 重发 inode 1。只走扫描重建，inode 1 的两个对象按 (号, 对象出生代) 分开、现行的是新的那个：
/// I-1.13 真判成立。判别力同第一格那一行变异。
#[test]
fn the_scan_rebuild_keeps_a_number_reissued_after_a_recovery_abandoned_its_root_on_the_new_object()
{
    let reissued = first_file_published_again_after_a_recovery_that_lands_on_the_warm_up_root(
        "c152-reissued-after-a-recovery-abandoned-the-root",
        RootOfTheFirstFileAfterTheFallBack::WrittenBackAndAbandoned,
    );
    let system_configuration = choose_system_configuration(&reissued.image).expect("系统配置");
    let roots_in_the_ring = readable_roots(
        &reissued.image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    assert!(
        roots_in_the_ring.iter().any(|root| root.checkpoint_txg
            == reissued.abandoned_first_file.root.checkpoint_txg
            && root.instance == reissued.abandoned_first_file.root.instance),
        "搭建自证：A 的根写回之后在根环里读得出"
    );
    assert_eq!(
        reissued.abandoned_first_file.root.checkpoint_txg,
        CheckpointTxg(3),
        "搭建自证：A 是新池新建文件那一版"
    );
    assert_the_number_is_reissued(&reissued);
    assert_eq!(
        verdict_of_the_scan_rebuild(&reissued.image),
        InvariantVerdict::Holds,
        "扫描重建只认实例 2 再发的那个对象：三张表与遍历方向逐项相同"
    );
}
