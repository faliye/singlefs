//! 池级 checker 的 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同，用户 2026-10-04 定新立）：三段合法历史上成立、C342（树 ID 水位在根读不出时
//! 退回去重发） 那一形上红。历史照攻方原型 `research/prompts/m3-sweep-and-tree-id-r1-opus-model/sweep-proto/src/bin/tree_id_histories.rs`
//! 的 `orphan_ff`、`applied_ff`、`rollback`、`c342`（`C342_ROLL_BACK_SYSCONF=1`）四臂另写一份（D13（验证路线） 已定项 5：用例不与原型共用代码），
//! 产物是同目录 `results/out/th-*.out`（判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第二节 S2）。
//!
//! 另有三形钉第四种判定（用户 2026-10-04 定「树表读不出算部分没判」、加 `InvariantVerdict::PartiallyJudged`；细则主 agent 定）：
//! 一条根的树表读不出、其余对得上 ⇒ 部分没判且列出那条根；一条根的树表读得出而条目比字段表短、其余对得上 ⇒ 同样部分没判且列出那条根；
//! 没有一棵树出现在两个不同的树表单元里 ⇒ 不适用；有一处对不上、另有一条根的树表读不出 ⇒ 照报违例。

mod common;
mod common_pointer_head_corruption;

use common_pointer_head_corruption::{
    node_bytes, replace_the_unit_and_propagate_its_checksum, unit_bytes_at,
};
use singlefs_checker::image::{
    InvariantVerdict, TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::seal_header_checksum;
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 这几份池的 fsid。
const FILESYSTEM_IDENTIFIER_OF_THE_BIRTH_POOLS: [u8; 16] = *b"BIRTH-EVERY-ROOT";
/// 每块盘 4 GiB 内存稀疏盘。
const DEVICE_BYTES_OF_THE_BIRTH_POOLS: u64 = 4 << 30;
/// 每区槽数 S = 4：环长 12。
const SLOTS_PER_REGION_OF_THE_BIRTH_POOLS: u64 = 4;
/// 写入时间的起点。
const WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS: u64 = 1_788_000_000;
/// 每版文件的长度：一个数据单元装得下。
const FILE_BYTES: usize = 3000;
/// extent 树的树 ID：从 mkfs 水位 65536 发起的第一个号（D8（核心索引结构） 已定项 8）。
const TREE_IDENTIFIER_OF_THE_FIRST_EXTENT_TREE: u64 = 65536;
/// 改坏一片树表时翻它载荷里的哪一个字节（头之后、条目区里：两份位置条目读出来整单元校验和都对不上）。
const BYTE_INSIDE_THE_PAYLOAD_OF_A_TREE_TABLE: u64 = 4000;
/// 第一个文件那一版的树 ID 水位：mkfs 水位 65536 之后发 13 个号（第 50 批加了退役树，此前 12 个、水位 65548）（`th-c342-sysconf.out` 里实例 1 那一版与实例 2 重发那一版都是 65548）。
const TREE_IDENTIFIER_WATERMARK_AFTER_THE_FIRST_FILE: u64 = 65549;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn parameters_of_the_birth_pools() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER_OF_THE_BIRTH_POOLS,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                SLOTS_PER_REGION_OF_THE_BIRTH_POOLS,
            )
            .expect("S = 4 在 4..16 里"),
        },
    }
}

fn fixed_geometry_of_the_birth_pools() -> FixedGeometry {
    let parameters = parameters_of_the_birth_pools();
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

fn fresh_devices() -> Devices {
    [0u32, 1]
        .into_iter()
        .map(|device| {
            (
                DeviceIdentity(device),
                SparseBlockDevice::new(
                    DEVICE_BYTES_OF_THE_BIRTH_POOLS,
                    PhysicalBlockSizeInBytes(512),
                ),
            )
        })
        .collect()
}

fn memory_pool_of(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES_OF_THE_BIRTH_POOLS,
    }
}

fn devices_of(image: &MemoryPool) -> Devices {
    image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                DEVICE_BYTES_OF_THE_BIRTH_POOLS,
                PhysicalBlockSizeInBytes(512),
            );
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect()
}

fn recorded_devices_of(
    image: &MemoryPool,
    stream: &SharedStream,
) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
    devices_of(image)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect()
}

/// 第 `version` 版文件的内容：每版字节各不相同。
fn content_of_version(version: u64) -> Vec<u8> {
    let salt = usize::try_from(version % 251).expect("小于 251");
    (0..FILE_BYTES)
        .map(|index| u8::try_from((index + salt * 7 + 3) % 251).expect("小于 251"))
        .collect()
}

fn overwrite(
    devices: &mut Devices,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    version: u64,
    instance: InstanceGeneration,
) -> TransactionOutput {
    let parameters = parameters_of_the_birth_pools();
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        allocator,
        previous,
        FirstFile {
            content: &content_of_version(version),
            write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS + version,
        },
        instance,
    )
    .expect("覆盖写")
}

fn birth_txg_verdict(image: &MemoryPool) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(invariant, _)| {
            *invariant == TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT
        })
        .expect("池级 checker 的清单里有 I-9.28")
        .1
}

/// 把 `version` 那条根指着的树表两份位置条目各翻一个字节（不重封、不往上补）：那条根照样自证合法，它的树表读不出。
fn make_the_tree_table_of_the_version_unreadable(
    image: &mut MemoryPool,
    version: &TransactionOutput,
) {
    for location in &version.root.tree_table.locations {
        image.flip_byte(
            location.device,
            DeviceOffsetInBytes(location.slot.0 * SLOT_BYTES),
            BYTE_INSIDE_THE_PAYLOAD_OF_A_TREE_TABLE,
        );
    }
}

/// 码 2 节点头里 key 宽那 1 字节的偏移（D18（块里携带什么信息） 已定项 7 的字段表）；k 是它的值，下面几个偏移都按它推。
const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;

fn set_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

/// 一个码 2 节点的自描述三段（声明长度 2 字节在 8、条目数 2 字节在 82 + 2k、条目宽 2 字节在 84 + 2k，条目区从 86 + 2k + 29 起）
/// 改成「条目宽 = key 宽、只装一条条目」：条目数改 1、声明长度改 k、largest_key（52 + k 起 k 字节）贴到那一条条目的 key 上、
/// 条目区之后的字节清零。除了条目宽不等于字段表宽，节点处处自洽（与 `checker_known_bad_images.rs` 里 I-1.10 那份坏镜像同一种改法）。
fn narrow_the_entry_width_to_the_key_width(bytes: &mut [u8]) {
    let key_width = usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
    let key_width_field = u16::try_from(key_width).expect("key 宽装得进 u16");
    let entries_start = 86 + 2 * key_width + 29;
    let first_key: Vec<u8> = bytes[entries_start..entries_start + key_width].to_vec();
    set_u16(bytes, 8, key_width_field);
    set_u16(bytes, 82 + 2 * key_width, 1);
    set_u16(bytes, 84 + 2 * key_width, key_width_field);
    bytes[52 + key_width..52 + 2 * key_width].copy_from_slice(&first_key);
    for byte in &mut bytes[entries_start + key_width..] {
        *byte = 0;
    }
}

/// 码 2 节点重封：载荷 CRC 罩 [86 + 2k, 节点末)、住 76 + 2k，再封头校验和（头 [0, 86 + 2k)）。
fn reseal_index_node(bytes: &mut [u8]) {
    let key_width = usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
    let header_end = 86 + 2 * key_width;
    let payload_checksum_offset = 76 + 2 * key_width;
    let payload_checksum = crc32_castagnoli(&bytes[header_end..]);
    bytes[payload_checksum_offset..payload_checksum_offset + 4]
        .copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(bytes, header_end);
}

/// mkfs → 可写挂载实例 1 → 第一个文件 A → 覆盖写两次（B、C）：三片带条目的树表，同一批树 ID、诞生 txg 都相同。交回镜像与 B 那一版。
fn first_file_and_two_overwrites() -> (MemoryPool, TransactionOutput) {
    let parameters = parameters_of_the_birth_pools();
    let mut devices = fresh_devices();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut mounted = mount_writable(&parameters, &mut devices).expect("挂载实例 1");
    let instance = mounted.output.instance;
    let first = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut mounted.allocator,
            mounted.current.root(),
            FirstFile {
                content: &content_of_version(0),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS,
            },
            instance,
            mounted.current.record_bytes(),
        )
        .expect("第一个文件 A")
    };
    let second = overwrite(&mut devices, &mut mounted.allocator, &first, 1, instance);
    overwrite(&mut devices, &mut mounted.allocator, &second, 2, instance);
    (memory_pool_of(&devices), second)
}

/// mkfs → 可写挂载实例 1 → 第一个文件那次发布录下来，切在第一次 `cut_before` 那一类写之前：交回崩溃镜像。
fn first_file_crashed_before_the_first(cut_before: StepKind) -> MemoryPool {
    let parameters = parameters_of_the_birth_pools();
    let mut devices = fresh_devices();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let first_mount = mount_writable(&parameters, &mut devices).expect("挂载实例 1");
    let before = memory_pool_of(&devices);
    let stream = SharedStream::retaining_contents();
    let mut recorded = recorded_devices_of(&before, &stream);
    {
        let mut writer = PoolWriter::new(&parameters, recorded.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut first_mount.allocator.clone(),
            first_mount.current.root(),
            FirstFile {
                content: &content_of_version(0),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS,
            },
            first_mount.output.instance,
            first_mount.current.record_bytes(),
        )
        .expect("第一个文件");
    }
    let operations = stream.retained_operations();
    let geometry = fixed_geometry_of_the_birth_pools();
    let cut = operations
        .iter()
        .position(|operation| geometry.classify(&operation.operation) == cut_before)
        .expect("第一个文件那次发布写了这一类");
    let mut crashed = before;
    crashed.apply(&operations[..cut]);
    crashed
}

/// 攻方 `orphan_ff` 臂：第一个文件那次发布切在第一条 journal 记录之前（它写的单元全是孤儿），实例 2 挂载落回暖机那一版、重发第一个文件
/// （新线上从根环水位 65536 发号）、再覆盖写一次。被抛弃的那条线一条根都没落盘，全部可读根的树表里同一树 ID 只有实例 2 记的那一个诞生 txg：
/// I-9.28 真被评估过且成立。
#[test]
fn a_first_file_cut_before_its_first_journal_record_and_reissued_by_the_next_instance_holds_the_every_readable_root_birth_invariant(
) {
    let parameters = parameters_of_the_birth_pools();
    let crashed = first_file_crashed_before_the_first(StepKind::JournalRecord);
    let mut devices = devices_of(&crashed);
    let mut second_mount = mount_writable(&parameters, &mut devices).expect("挂载实例 2");
    assert!(
        second_mount.current.file_version().is_none(),
        "第一个文件那次发布一条记录都没落：实例 2 落回没有文件的那一版"
    );
    let instance = second_mount.output.instance;
    let reissued = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut second_mount.allocator,
            second_mount.current.root(),
            FirstFile {
                content: &content_of_version(1),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS + 1,
            },
            instance,
            second_mount.current.record_bytes(),
        )
        .expect("实例 2 重发第一个文件")
    };
    overwrite(
        &mut devices,
        &mut second_mount.allocator,
        &reissued,
        2,
        instance,
    );
    assert_eq!(
        birth_txg_verdict(&memory_pool_of(&devices)),
        InvariantVerdict::Holds,
        "重发第一个文件之后再覆盖写一次：两片树表记的诞生 txg 相同"
    );
}

/// 攻方 `applied_ff` 臂：第一个文件那次发布切在根槽写之前（记录已落），实例 2 挂载由记录施加出那一版、写行、暖机：I-9.28 不许红；
/// 之后再覆盖写一次（攻方那一臂没有这一步，加上它根环里一定有两片带条目的树表可比），两片树表里同一树 ID 的诞生 txg 相同：
/// I-9.28 真被评估过且成立。
#[test]
fn a_first_file_whose_record_landed_without_its_root_applied_by_the_next_instance_and_overwritten_holds_the_every_readable_root_birth_invariant(
) {
    let parameters = parameters_of_the_birth_pools();
    let crashed = first_file_crashed_before_the_first(StepKind::RootRecordFua);
    let mut devices = devices_of(&crashed);
    let mut second_mount = mount_writable(&parameters, &mut devices).expect("挂载实例 2");
    let instance = second_mount.output.instance;
    let verdict_after_the_mount = birth_txg_verdict(&memory_pool_of(&devices));
    assert!(
        !matches!(verdict_after_the_mount, InvariantVerdict::Violated(_)),
        "实例 2 挂载之后：{verdict_after_the_mount:?}"
    );
    let applied = second_mount
        .current
        .into_file_version()
        .expect("由记录施加出来的那一版带文件");
    overwrite(
        &mut devices,
        &mut second_mount.allocator,
        &applied,
        1,
        instance,
    );
    assert_eq!(
        birth_txg_verdict(&memory_pool_of(&devices)),
        InvariantVerdict::Holds,
        "覆盖写之后两片树表记的诞生 txg 相同"
    );
}

/// 攻方 `rollback` 臂：同一实例第一个文件 A、覆盖写三次、挂着时回退到 A（一次向前发布）、再覆盖写一次。管理员回退不切时间线、树表条目照抄：
/// I-9.28 真被评估过且成立。
#[test]
fn a_rollback_by_a_forward_publish_and_an_overwrite_after_it_hold_the_every_readable_root_birth_invariant(
) {
    let parameters = parameters_of_the_birth_pools();
    let mut devices = fresh_devices();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut mounted = mount_writable(&parameters, &mut devices).expect("挂载实例 1");
    let instance = mounted.output.instance;
    let first = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut mounted.allocator,
            mounted.current.root(),
            FirstFile {
                content: &content_of_version(0),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS,
            },
            instance,
            mounted.current.record_bytes(),
        )
        .expect("第一个文件 A")
    };
    let target = RollbackTarget {
        instance: first.root.instance,
        checkpoint_txg: first.root.checkpoint_txg,
    };
    let mut current = first;
    for version in 1..=3 {
        current = overwrite(
            &mut devices,
            &mut mounted.allocator,
            &current,
            version,
            instance,
        );
    }
    roll_back_by_a_forward_publish(
        &parameters,
        &mut devices,
        &mut mounted.allocator,
        &mut current,
        target,
    )
    .expect("挂着时回退到 A");
    overwrite(&mut devices, &mut mounted.allocator, &current, 9, instance);
    assert_eq!(
        birth_txg_verdict(&memory_pool_of(&devices)),
        InvariantVerdict::Holds,
        "回退之前、回退那次与之后的树表记的诞生 txg 相同"
    );
}

/// 攻方 `c342` 臂带 `C342_ROLL_BACK_SYSCONF=1`（`th-c342-sysconf.out`）：实例 1 第一个文件（extent 树 65536 生在 txg 3）、覆盖写 7 次到 txg 10；
/// 这 8 次发布的根槽写与记录写全抹成 0（读不出），系统配置也退回实例 1 发第一个文件之前那一份（整卷回滚，D18（块里携带什么信息） 已定项 11
/// 宣布不防；只抹根与记录时可写挂载在取号之前被拒）。实例 2 落回暖机那一版、写行、暖机、重发第一个文件（从根环水位 65536 再发一次号，
/// extent 树 65536 生在 txg 5）；之后把没被实例 2 盖掉的根槽与记录放回去（故障消失）。根环里实例 1 那几条老根记 65536 生在 3、实例 2 的记 5。
/// 交回放回之后的镜像，与实例 1 最后那一版（txg 10：它的根槽实例 2 没盖到，放回来了）。
fn image_after_a_whole_volume_rollback_with_the_hidden_roots_put_back(
) -> (MemoryPool, TransactionOutput) {
    let parameters = parameters_of_the_birth_pools();
    let geometry = fixed_geometry_of_the_birth_pools();
    let mut devices = fresh_devices();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut first_mount = mount_writable(&parameters, &mut devices).expect("挂载实例 1");
    let first_instance = first_mount.output.instance;
    let before = memory_pool_of(&devices);
    let stream = SharedStream::retaining_contents();
    let mut recorded = recorded_devices_of(&before, &stream);
    let last_version_of_the_first_instance = {
        let mut current = {
            let mut writer = PoolWriter::new(&parameters, recorded.as_mut_slice());
            publish_first_file(
                &mut writer,
                &mut first_mount.allocator,
                first_mount.current.root(),
                FirstFile {
                    content: &content_of_version(0),
                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS,
                },
                first_instance,
                first_mount.current.record_bytes(),
            )
            .expect("实例 1 第一个文件")
        };
        for version in 1..=7u64 {
            let mut writer = PoolWriter::new(&parameters, recorded.as_mut_slice());
            current = publish_overwrite(
                &mut writer,
                &mut first_mount.allocator,
                &current,
                FirstFile {
                    content: &content_of_version(version),
                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS + version,
                },
                first_instance,
            )
            .expect("实例 1 覆盖写");
        }
        current
    };
    let operations = stream.retained_operations();
    let mut image = before.clone();
    image.apply(&operations);
    let hidden: Vec<(DeviceIdentity, u64, Vec<u8>)> = operations
        .iter()
        .filter(|operation| {
            matches!(
                geometry.classify(&operation.operation),
                StepKind::RootRecordFua | StepKind::JournalRecord
            )
        })
        .filter_map(|operation| {
            operation.contents.clone().map(|contents| {
                (
                    operation.operation.device,
                    operation.operation.offset.0,
                    contents,
                )
            })
        })
        .collect();
    assert!(!hidden.is_empty(), "实例 1 那 8 次发布写过根槽与记录");
    for (device, offset, contents) in &hidden {
        image
            .devices
            .get_mut(device)
            .expect("两块盘")
            .write(DeviceOffsetInBytes(*offset), &vec![0u8; contents.len()]);
    }
    for operation in operations.iter().filter(|operation| {
        geometry.classify(&operation.operation) == StepKind::SystemConfigurationSlot
    }) {
        let length = usize::try_from(operation.operation.length).expect("一个系统配置槽");
        let earlier =
            before.devices[&operation.operation.device].read(operation.operation.offset, length);
        image
            .devices
            .get_mut(&operation.operation.device)
            .expect("两块盘")
            .write(operation.operation.offset, &earlier);
    }
    let mut devices_after_the_whole_volume_rollback = devices_of(&image);
    let mut second_mount =
        mount_writable(&parameters, &mut devices_after_the_whole_volume_rollback)
            .expect("整卷回滚之后实例 2 可写挂载");
    assert!(
        second_mount.current.file_version().is_none(),
        "实例 1 的根与记录都读不出：实例 2 落回没有文件的那一版"
    );
    let second_instance = second_mount.output.instance;
    let reissued = {
        let mut writer = PoolWriter::new(
            &parameters,
            devices_after_the_whole_volume_rollback.as_mut_slice(),
        );
        publish_first_file(
            &mut writer,
            &mut second_mount.allocator,
            second_mount.current.root(),
            FirstFile {
                content: &content_of_version(40),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS + 40,
            },
            second_instance,
            second_mount.current.record_bytes(),
        )
        .expect("实例 2 重发第一个文件")
    };
    assert_eq!(
        reissued.root.tree_identifier_watermark, TREE_IDENTIFIER_WATERMARK_AFTER_THE_FIRST_FILE,
        "实例 2 从根环水位 65536 再发一次同一批树 ID"
    );
    let mut image_with_the_hidden_writes_put_back =
        memory_pool_of(&devices_after_the_whole_volume_rollback);
    for (device, offset, contents) in &hidden {
        let now = image_with_the_hidden_writes_put_back.devices[device]
            .read(DeviceOffsetInBytes(*offset), contents.len());
        if now.iter().all(|byte| *byte == 0) {
            image_with_the_hidden_writes_put_back
                .devices
                .get_mut(device)
                .expect("两块盘")
                .write(DeviceOffsetInBytes(*offset), contents);
        }
    }
    (
        image_with_the_hidden_writes_put_back,
        last_version_of_the_first_instance,
    )
}

/// 放回之后根环里实例 1 那几条老根记 65536 生在 3、实例 2 的记 5：I-9.28 红，红在树 65536 上。
#[test]
fn tree_identifiers_reissued_after_a_whole_volume_rollback_redden_the_every_readable_root_birth_invariant_once_the_hidden_roots_come_back(
) {
    let (image_with_the_hidden_writes_put_back, _) =
        image_after_a_whole_volume_rollback_with_the_hidden_roots_put_back();
    match birth_txg_verdict(&image_with_the_hidden_writes_put_back) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains(&format!("树 {TREE_IDENTIFIER_OF_THE_FIRST_EXTENT_TREE} ")),
            "红在 extent 树 65536 上：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("老线与新线上同一树 ID 65536 诞生 txg 不同，I-9.28 要红，得到 {other:?}")
        }
    }
}

/// 同一张镜像上再把实例 1 最后那一版（txg 10）的树表两份都翻坏：那条根列进「没判到」，而老线与新线上别的根照样对不上——
/// 判出了违例就报违例，不报部分没判。
#[test]
fn a_birth_txg_disagreement_is_reported_as_violated_even_when_another_roots_tree_table_is_unreadable(
) {
    let (mut image, last_version_of_the_first_instance) =
        image_after_a_whole_volume_rollback_with_the_hidden_roots_put_back();
    make_the_tree_table_of_the_version_unreadable(&mut image, &last_version_of_the_first_instance);
    match birth_txg_verdict(&image) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains(&format!("树 {TREE_IDENTIFIER_OF_THE_FIRST_EXTENT_TREE} ")),
            "红在 extent 树 65536 上：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => panic!(
            "老线与新线上同一树 ID 65536 诞生 txg 不同、另有一条根的树表读不出：照报违例，得到 {other:?}"
        ),
    }
}

/// 第一个文件 A、覆盖写 B、C，再把 B 那条根指着的树表两份都翻坏（B 的根照样自证合法）：A 与 C 两片树表比得上、诞生 txg 相同，
/// B 那一片没进比较——报部分没判，说明里列出 B 那条根（实例 1、它的 txg）与「树表读不出」。
#[test]
fn a_readable_root_whose_tree_table_is_unreadable_while_the_others_agree_is_reported_as_partially_judged_naming_that_root(
) {
    let (mut image, second) = first_file_and_two_overwrites();
    assert_eq!(
        birth_txg_verdict(&image),
        InvariantVerdict::Holds,
        "翻坏之前：三片树表比得上、诞生 txg 相同"
    );
    make_the_tree_table_of_the_version_unreadable(&mut image, &second);
    match birth_txg_verdict(&image) {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.contains(&format!(
                "（实例 {}、txg {}）：树表读不出",
                second.root.instance.0, second.root.checkpoint_txg.0
            )),
            "列出 B 那条根与为什么没判：{unjudged}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::Violated(_)) => {
            panic!("B 的树表读不出、A 与 C 对得上：I-9.28 报部分没判，得到 {other:?}")
        }
    }
}

/// 第一个文件 A、覆盖写 B、C，再把 B 那条根指着的树表改成「条目宽 = key 宽、只装一条条目」（重封，新的整单元校验和补进 B 的根槽，
/// B 的根照样自证合法）：B 的树表读得出、条目比字段表短、那一条不解字段；A 与 C 两片树表比得上、诞生 txg 相同——报部分没判，
/// 说明里列出 B 那条根与「树表有条目比字段表短」。
#[test]
fn a_readable_root_whose_tree_table_entries_are_shorter_than_the_field_table_while_the_others_agree_is_reported_as_partially_judged_naming_that_root(
) {
    let (mut image, second) = first_file_and_two_overwrites();
    assert_eq!(
        birth_txg_verdict(&image),
        InvariantVerdict::Holds,
        "改窄之前：三片树表比得上、诞生 txg 相同"
    );
    let tree_table_slot = second.root.tree_table.locations[0].slot;
    let mut tree_table = unit_bytes_at(&image, tree_table_slot, node_bytes());
    narrow_the_entry_width_to_the_key_width(&mut tree_table);
    reseal_index_node(&mut tree_table);
    replace_the_unit_and_propagate_its_checksum(&mut image, tree_table_slot, &tree_table);
    match birth_txg_verdict(&image) {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.contains(&format!(
                "（实例 {}、txg {}）：树表有条目比字段表短",
                second.root.instance.0, second.root.checkpoint_txg.0
            )),
            "列出 B 那条根与为什么没判：{unjudged}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::Violated(_)) => {
            panic!("B 的树表条目比字段表短、A 与 C 对得上：I-9.28 报部分没判，得到 {other:?}")
        }
    }
}

/// mkfs → 可写挂载实例 1 → 第一个文件：带条目的树表只有第一个文件那一片（mkfs 与暖机那几条根指着的树表 0 条），
/// 没有一棵树的条目出现在两个不同的树表单元里——报不适用，不报成立。
#[test]
fn no_tree_seen_in_two_different_tree_table_units_is_reported_as_not_applicable() {
    let parameters = parameters_of_the_birth_pools();
    let mut devices = fresh_devices();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut mounted = mount_writable(&parameters, &mut devices).expect("挂载实例 1");
    let instance = mounted.output.instance;
    {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut mounted.allocator,
            mounted.current.root(),
            FirstFile {
                content: &content_of_version(0),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS,
            },
            instance,
            mounted.current.record_bytes(),
        )
        .expect("第一个文件");
    }
    match birth_txg_verdict(&memory_pool_of(&devices)) {
        InvariantVerdict::NotApplicable(reason) => assert!(
            reason.contains("两个不同的树表单元"),
            "理由要说没有一棵树出现在两个不同的树表单元里：{reason}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("只有一片带条目的树表：I-9.28 报不适用，得到 {other:?}")
        }
    }
}
