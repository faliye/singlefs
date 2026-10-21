//! 欠账 C758（`.claude/kb/checks-owed.md`「无读故障时末条读得出根读不出的发布没量」）钉住的那一形：
//! F2 退回（D23（journal 的角色与格式） 已定项 14，`recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`）
//! 里「不跳过一次末条读得出、根读不出、不被抛弃的发布」那一句要有对象，得有一段 (见证到的那一版, 所选根) 里装着这样的发布。
//! 没有读故障时这一段装不下别的发布（每次发布在根槽之后轮换系统配置、过屏障，c_见证 最多落后所选根一次发布），
//! 这一形只在系统配置的写静默丢失时出现：第一个文件之后写到两块盘两个系统配置槽的写都报成功、不落盘，再覆盖写 30 次，
//! c_见证 停在第一个文件那次发布（计数器 3），根环 24 槽住着 txg 12..35，txg 4..11 那八次发布的根槽被盖掉、末条记录还在 journal 环里。
//! 所选根（txg 35）的树表两份各翻坏一个字节（坏镜像，不是读故障），三处入口都走 F2：退到区间里最新的可读根 txg 34，
//! 那八次发布都在 txg 34 之下，「不跳过」只看退到的那一条与所选根之间，挡不住，三处都落 txg 34。
//! 这几条用例钉的是今天的结局（用户 2026-10-20 定「再补一条用例钉住」）；该不该退、退到哪归 D23 已定项 14，条款改了这几条跟着改。
mod common_admission_pool;

use common_admission_pool::{
    device_bytes_of_the_384_slot_small_devices, parameters_of_the_384_slot_small_devices,
    plain_devices_on, DeviceOverASparseImage, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber, InstanceGeneration,
};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::journal::JournalRecordPlaceInPublish;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::{mount_read_only, MountedReadOnly};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, readable_roots, recover, scan_journal,
    verified_system_configuration_slots, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use std::collections::{BTreeMap, BTreeSet};

/// 打开静默丢写之后覆盖写的次数：30 次把根环 24 槽转过一圈多，txg 4..11 那八次发布的根槽被后来的根盖掉。
const OVERWRITES_WITH_THE_SYSTEM_CONFIGURATION_WRITES_LOST: u64 = 30;

/// 树表那一单元里翻坏的字节在单元里的偏移：落在节点头之后、单元校验和罩住的那一段里。
const FLIPPED_BYTE_INDEX_IN_THE_TREE_TABLE_UNIT: u64 = 200;

/// 一块内存盘：写到两个系统配置槽（偏移 0 与槽距处）的写报成功、不落盘；别的照转给里面那块稀疏盘。
struct DeviceLosingSystemConfigurationSlotWrites {
    inner: SparseBlockDevice,
    system_configuration_slot_spacing_in_bytes: u64,
}

impl DeviceLosingSystemConfigurationSlotWrites {
    fn is_a_system_configuration_slot(&self, offset: DeviceOffsetInBytes) -> bool {
        offset.0 == 0 || offset.0 == self.system_configuration_slot_spacing_in_bytes
    }
}

impl BlockDevice for DeviceLosingSystemConfigurationSlotWrites {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        if self.is_a_system_configuration_slot(offset) {
            return Ok(());
        }
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.inner.discard_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

impl DeviceOverASparseImage for DeviceLosingSystemConfigurationSlotWrites {
    fn sparse_image(&self) -> &SparseDevice {
        &self.inner.image
    }
}

/// 一版的键：(实例代号, checkpoint_txg)。
fn version_key(
    instance_generation: u32,
    checkpoint_txg: u64,
) -> (InstanceGeneration, CheckpointTxg) {
    (
        InstanceGeneration(instance_generation),
        CheckpointTxg(checkpoint_txg),
    )
}

fn key_of(root: &RootRecord) -> (InstanceGeneration, CheckpointTxg) {
    (root.instance, root.checkpoint_txg)
}

/// 静默丢写那一段写下的盘面（树表没坏），连同每一版的文件内容。
struct PoolWithTheSystemConfigurationWritesLost {
    image: MemoryPool,
    content_of_each_version: BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>,
}

/// 第一个文件（txg 3）之后打开静默丢写，再在同一个会话里覆盖写 30 次（txg 4..35）。
fn pool_with_the_system_configuration_writes_lost_after_the_first_file(
) -> PoolWithTheSystemConfigurationWritesLost {
    let parameters = parameters_of_the_384_slot_small_devices();
    let system_configuration_slot_spacing_in_bytes =
        u64::from(parameters.geometry.fixed_structure_slot_spacing);
    // 系统配置槽的写从第一个文件之后才丢：mkfs、取号、暖机与第一个文件那几次轮换都落了盘，c_见证 停在第一个文件那次发布。
    let pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters,
        device_bytes_of_the_384_slot_small_devices(),
        |_identity, inner| inner,
    );
    let devices_losing_the_writes: Vec<_> = pool
        .devices
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                DeviceLosingSystemConfigurationSlotWrites {
                    inner,
                    system_configuration_slot_spacing_in_bytes,
                },
            )
        })
        .collect();
    let mut pool_losing_the_writes = PoolUnderTest {
        parameters: pool.parameters,
        device_bytes: pool.device_bytes,
        devices: devices_losing_the_writes,
        session: pool.session,
        write_time_seconds: pool.write_time_seconds,
        content_of_the_current_version: pool.content_of_the_current_version,
        content_of_each_root: pool.content_of_each_root,
    };
    for _ in 0..OVERWRITES_WITH_THE_SYSTEM_CONFIGURATION_WRITES_LOST {
        pool_losing_the_writes
            .overwrite(OVERWRITE_BYTES)
            .expect("系统配置写丢了照样报成功，会话里的覆盖写做得成");
    }
    PoolWithTheSystemConfigurationWritesLost {
        image: pool_losing_the_writes.image(),
        content_of_each_version: pool_losing_the_writes.content_of_each_root,
    }
}

/// 同一份盘面，所选根的树表两份各翻坏一个字节：所选那一版走读失败。
fn with_the_tree_table_of_the_chosen_root_damaged(image: &MemoryPool) -> MemoryPool {
    let system_configuration = choose_system_configuration(image).expect("系统配置择得出");
    let chosen_root = choose_root(image, &system_configuration).expect("根环里有可读的根");
    let mut damaged = MemoryPool {
        devices: image.devices.clone(),
        device_size_in_bytes: image.device_size_in_bytes,
    };
    for location in chosen_root.tree_table.locations {
        damaged.flip_byte(
            location.device,
            DeviceOffsetInBytes(location.slot.0 * SLOT_BYTES),
            FLIPPED_BYTE_INDEX_IN_THE_TREE_TABLE_UNIT,
        );
    }
    damaged
}

/// 只读挂载读到的文件（inode 1）内容。
fn file_content_of_the_read_only_mount(image: &MemoryPool, mounted: &MountedReadOnly) -> Vec<u8> {
    let inode_record = mounted
        .mounted
        .inode_records()
        .iter()
        .find(|record| record.inode == 1)
        .copied()
        .expect("第一个文件那一版起每一版都有 inode 1");
    mounted
        .mounted
        .open_file(image, InodeNumber(1))
        .expect("挂上的那一版打得开 inode 1")
        .read_at(image, FileOffsetInBytes(0), inode_record.size)
        .expect("挂上的那一版读得出 inode 1 的全部内容")
        .bytes
}

/// 恢复：所选根 txg 35 的树表坏了，F2 退到 (c_见证 那一版 txg 3, 所选根) 里最新的可读根 txg 34，读出 txg 34 那一版的文件；
/// 区间里 txg 4..11 那八次发布末条读得出、根读不出，都在 txg 34 之下，「不跳过」不挡。树表没坏时恢复读的是所选那一版本身（F2 不走）。
#[test]
fn recovery_of_a_damaged_chosen_version_falls_back_to_the_newest_readable_root_below_it_past_publishes_whose_roots_were_overwritten(
) {
    let pool = pool_with_the_system_configuration_writes_lost_after_the_first_file();
    let image = &pool.image;
    let system_configuration = choose_system_configuration(image).expect("系统配置择得出");
    let chosen_root = choose_root(image, &system_configuration).expect("根环里有可读的根");
    assert_eq!(
        key_of(&chosen_root),
        version_key(1, 35),
        "所选根是第 30 次覆盖写那一版"
    );
    let witnessed_journal_counter = image
        .devices
        .keys()
        .flat_map(|device| {
            verified_system_configuration_slots(
                image,
                *device,
                u64::from(
                    system_configuration
                        .immutable
                        .sizes
                        .fixed_structure_slot_spacing,
                ),
                &system_configuration.immutable.filesystem_identifier,
            )
        })
        .map(|slot| slot.quantities.journal_tail)
        .max()
        .expect("每块盘的系统配置槽自证得过");
    assert_eq!(
        witnessed_journal_counter, 3,
        "系统配置写从第一个文件之后都丢了：c_见证 停在第一个文件那次发布的末条计数器"
    );
    let roots_in_the_ring: BTreeSet<(InstanceGeneration, CheckpointTxg)> = readable_roots(
        image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .iter()
    .map(key_of)
    .collect();
    assert_eq!(
        roots_in_the_ring,
        (12..=35).map(|txg| version_key(1, txg)).collect(),
        "根环 24 槽住着最近 24 条根 txg 12..35，更早的根槽被盖掉"
    );
    let publishes_whose_last_record_reads_while_the_root_does_not: BTreeSet<(
        InstanceGeneration,
        CheckpointTxg,
    )> = scan_journal(image, &system_configuration)
        .values()
        .filter(|record| {
            record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .map(|record| (record.instance, record.checkpoint_txg))
        .filter(|version| {
            CheckpointTxg(3) < version.1
                && version.1 < CheckpointTxg(35)
                && !roots_in_the_ring.contains(version)
        })
        .collect();
    assert_eq!(
        publishes_whose_last_record_reads_while_the_root_does_not,
        (4..=11).map(|txg| version_key(1, txg)).collect(),
        "(c_见证 那一版 txg 3, 所选根 txg 35) 里末条记录读得出、根读不出的发布是 txg 4..11 那八次：「不跳过」那一句有了对象"
    );
    match recover(image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { root, content } => {
            assert_eq!(
                root,
                version_key(1, 35),
                "树表没坏时恢复读的是所选那一版本身，F2 不走"
            );
            assert_eq!(
                Some(&content),
                pool.content_of_each_version.get(&version_key(1, 35)),
                "树表没坏时读回第 30 次覆盖写的内容"
            );
        }
        other @ (RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. }) => {
            panic!("树表没坏时恢复应当读出所选那一版，实际 {other:?}")
        }
    }
    let damaged = with_the_tree_table_of_the_chosen_root_damaged(image);
    match recover(&damaged, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { root, content } => {
            assert_eq!(
                root,
                version_key(1, 34),
                "所选那一版走读失败、没被见证：F2 退到区间里最新的可读根 txg 34，txg 4..11 那八次都在它之下，「不跳过」不挡"
            );
            assert_eq!(
                Some(&content),
                pool.content_of_each_version.get(&version_key(1, 34)),
                "退到 txg 34 读回的是第 29 次覆盖写的内容"
            );
        }
        other @ (RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. }) => {
            panic!("所选根的树表坏了，恢复应当退到 txg 34 读出文件，实际 {other:?}")
        }
    }
}

/// 只读挂载：同一份坏了树表的盘面，所选根与施加之后的根都报退到的 txg 34，读出 txg 34 那一版的文件。
#[test]
fn a_read_only_mount_of_a_damaged_chosen_version_falls_back_to_the_newest_readable_root_below_it_past_publishes_whose_roots_were_overwritten(
) {
    let pool = pool_with_the_system_configuration_writes_lost_after_the_first_file();
    let damaged = with_the_tree_table_of_the_chosen_root_damaged(&pool.image);
    let mounted = mount_read_only(&damaged)
        .expect("所选那一版走读失败、F2 退到 txg 34 打得开：只读挂载做得成");
    assert_eq!(
        key_of(&mounted.chosen_root),
        version_key(1, 34),
        "退过去之后所选根报退到的那一条"
    );
    assert_eq!(
        key_of(&mounted.effective_root),
        version_key(1, 34),
        "退过去不重放：施加之后的根也是 txg 34"
    );
    assert_eq!(
        Some(&file_content_of_the_read_only_mount(&damaged, &mounted)),
        pool.content_of_each_version.get(&version_key(1, 34)),
        "只读挂载读到的是第 29 次覆盖写的内容"
    );
}

/// 可写挂载：同一份坏了树表的盘面，挂载做得成，所选根与施加之后的根都是 txg 34，扫环报告里施加的那一截清零。
#[test]
fn a_writable_mount_of_a_damaged_chosen_version_falls_back_to_the_newest_readable_root_below_it_past_publishes_whose_roots_were_overwritten(
) {
    let pool = pool_with_the_system_configuration_writes_lost_after_the_first_file();
    let damaged = with_the_tree_table_of_the_chosen_root_damaged(&pool.image);
    let mut devices = plain_devices_on(&damaged);
    let mounted = mount_writable(&parameters_of_the_384_slot_small_devices(), &mut devices)
        .expect("所选那一版建不出来、F2 退到 txg 34 建得出来：可写挂载做得成");
    assert_eq!(
        key_of(&mounted.output.chosen_root),
        version_key(1, 34),
        "退过去之后所选根换成退到的那一条"
    );
    assert_eq!(
        key_of(&mounted.output.effective_root),
        version_key(1, 34),
        "退过去不重放：所选那一版也换成 txg 34"
    );
    assert_eq!(
        mounted.output.journal.prefix_applied, 0,
        "退过去之后扫环报告里施加的那一截清零"
    );
}
