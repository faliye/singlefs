//! checker 的已知坏镜像语料（C13（checker 判定失效））：第一版判的 36 条不变量，每条配一份「改坏了正好触发它」的镜像。
//! 每份都从一份干净镜像出发（写完新池新建文件的，或再覆盖写一次的），改一处；要让被改的那一条之前的判定不先挡住，
//! 被改的单元重封校验和，再把新的整单元校验和沿引用链一路补到根记录（父节点里的位置条目、中央映射条目、根槽的自证校验和）。

mod common;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
    copy_each_root_into_the_new_root_segment_of_its_journal_records, crash_state_devices,
    format_pool, geometry, memory_pool_of_sparse_devices, parameters, publish_overwrite_in_process,
    rewrite_journal_record_and_rechain_the_records_after_it, BuiltPool, FIXED_WRITE_TIME_SECONDS,
    IMAGE_BYTES,
};
use singlefs_checker::image::{
    InvariantVerdict, ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
    DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
    FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
    HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
    INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
    INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
    JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
    LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH, NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
    POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER, PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
    REFERENCE_INTERVALS_ARE_EXACT, ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES,
    ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
    SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
    SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
    SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
    SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
    TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
    UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocation_record_tree::AllocationRecordTreeNodePosition;
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::journal::{
    back_chain_of, record_offset, JournalRecord, JournalRecordOrdinalWithinPublish,
    JournalRecordPlaceInPublish,
};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
    INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, InstanceRow,
    MountError, RollbackTarget, ShadowLedger, Unmounted,
};
use singlefs_core::records::{TREE_KIND_ALLOCATION, TREE_KIND_DELETED_INODES, TREE_KIND_TOMBSTONE};
use singlefs_core::recovery::{
    instance_table_chain_of_root, recover, JournalPolicy, PoolReader, RecoveryFailure,
    RecoveryOutcome,
};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_instance_table_on_version_without_file,
    publish_overwrite, publish_sequential_write, publish_without_units, warm_up, FirstFile,
    InstanceTableOnlyPublishPlan, InstanceTableRewrite, PoolVersion, PoolWriter, TransactionOutput,
    TransactionUnit, ZeroUnitPublishPlan,
};
use singlefs_format::{
    ALLOCATION_RECORD_TREE_LEAF_SLOTS, JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES,
    JOURNAL_SAFETY_FACTOR, NODE_BYTES, TREE_IDENTIFIER_EXTENT,
};
use singlefs_harness::history::{allocation_statistic_mechanism, AllocationStatisticMechanism};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RetainedOperation, SharedStream};

const SLOT: u64 = 16384;
const DEVICES: [u32; 2] = [0, 1];
/// 写完新池新建文件之后单元区里的单元：(起点槽, 字节数)。分配记录树按位置寻址（D8（核心索引结构） 已定项 14）：4 GiB 两块盘上根在第 2 层，
/// A 写出它的五个节点——两块盘各自的叶 61（50245、50246）、两块盘各自的第 1 层节点 0（50247、50248）与根（50249），先叶后根。
const UNITS: [(u64, usize); 14] = [
    (50176, 32768),
    (50178, 16384),
    (50180, 32768),
    (50240, 16384),
    (50242, 32768),
    (50244, 16384),
    (50245, 16384),
    (50246, 16384),
    (50247, 16384),
    (50248, 16384),
    (50249, 16384),
    (50250, 16384),
    (50251, 16384),
    (50252, 16384),
];
/// 再覆盖写一次（发布 B，txg 4）之后单元区里的单元：A 的十四个槽仍在盘上（根环里 A 的根还引用着它们），加上 B 写出的十二个单元
/// （落点由 `overwrite_in_one_instance.rs` 的验收钉住）。
const UNITS_AFTER_OVERWRITE: [(u64, usize); 26] = [
    (50176, 32768),
    (50178, 16384),
    (50180, 32768),
    (50182, 32768),
    (50240, 16384),
    (50242, 32768),
    (50244, 16384),
    (50245, 16384),
    (50246, 16384),
    (50247, 16384),
    (50248, 16384),
    (50249, 16384),
    (50250, 16384),
    (50251, 16384),
    (50252, 16384),
    (50253, 16384),
    (50254, 32768),
    (50256, 16384),
    (50257, 16384),
    (50258, 16384),
    (50259, 16384),
    (50260, 16384),
    (50261, 16384),
    (50262, 16384),
    (50263, 16384),
    (50264, 16384),
];
/// 发布 B 写出的分配记录树两片叶（盘 0 的叶 61、盘 1 的叶 61；I-3.9、I-3.10、I-5.4 那几份坏镜像改的就是它们）与树表（I-9.14 那一份改它）。
/// 每片叶只装它那块盘上的记录：一个单元两盘各一份落点，两条记录分在两片叶里。
const ALLOCATION_LEAVES_AFTER_OVERWRITE: [u64; 2] = [50257, 50258];
const TREE_TABLE_AFTER_OVERWRITE: u64 = 50264;
const INSTANCE_TABLE: u64 = 50176;
/// mkfs 种下的第 0 版树表单元：第一个文件版本换下它之后只有第 0 代根还引用。
const GENESIS_TREE_TABLE: u64 = 50178;
const DATA_UNIT: u64 = 50180;
const EXTENT_ROOT: u64 = 50240;
const INODE_LEAF: u64 = 50242;
const INODE_ROOT: u64 = 50244;
/// 新池新建文件（A）写出的分配记录树两片叶（盘 0、盘 1 各自的叶 61）：发布 B 之后只有 A 的根还（经 A 的第 1 层节点）指着它们。
const ALLOCATION_LEAVES: [u64; 2] = [50245, 50246];
const ACCOUNTING_ROOT: u64 = 50250;
const TREE_TABLE: u64 = 50252;

fn read(pool: &MemoryPool, device: u32, offset: u64, length: usize) -> Vec<u8> {
    PoolReader::read(
        pool,
        DeviceIdentity(device),
        DeviceOffsetInBytes(offset),
        length,
    )
    .expect("读")
}

fn write(pool: &mut MemoryPool, device: u32, offset: u64, bytes: &[u8]) {
    pool.devices
        .get_mut(&DeviceIdentity(device))
        .expect("盘")
        .write(DeviceOffsetInBytes(offset), bytes);
}

/// 按类重封：先算载荷 CRC，再算头校验和（码 1：头 105、CRC 在 101；码 3：头 107、CRC 在 89；码 2：头 86 + 2k、CRC 在 76 + 2k）。
fn reseal_unit(bytes: &mut [u8]) {
    let (header_end, payload_crc_offset) = match bytes[6] {
        1 => (105usize, 101usize),
        3 => (107, 89),
        _ => (
            86 + 2 * usize::from(bytes[51]),
            76 + 2 * usize::from(bytes[51]),
        ),
    };
    let payload_crc = crc32_castagnoli(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    singlefs_core::unit::seal_header_checksum(bytes, header_end);
}

/// 根环全部槽：(盘, 偏移)。区域 r 在盘 [0, 1, 0][r] 上，起点 64 × 16384 + r × 3 MiB，槽距 4096。
fn root_slots() -> Vec<(u32, u64)> {
    (0..3u64)
        .flat_map(|region| {
            (0..8u64).map(move |slot| {
                (
                    [0u32, 1, 0][usize::try_from(region).expect("区域")],
                    64 * SLOT + region * 3 * (1 << 20) + slot * 4096,
                )
            })
        })
        .collect()
}

fn location_pattern(device: u32, slot: u64, checksum: u32) -> Vec<u8> {
    [
        device.to_le_bytes().to_vec(),
        slot.to_le_bytes()[..6].to_vec(),
        checksum.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn replace_all(haystack: &mut [u8], needle: &[u8], replacement: &[u8]) -> bool {
    let mut changed = false;
    let mut index = 0;
    while index + needle.len() <= haystack.len() {
        if haystack[index..index + needle.len()] == *needle {
            haystack[index..index + needle.len()].copy_from_slice(replacement);
            changed = true;
            index += needle.len();
        } else {
            index += 1;
        }
    }
    changed
}

/// 一个单元（两盘同槽）的整单元校验和从 old 变成 new：把引用它的位置条目逐个补上，被补的单元重封、再往上补，直到根槽。
/// `units` 是这份镜像单元区里的单元表：写完新池新建文件的是 `UNITS`，再覆盖写一次的是 `UNITS_AFTER_OVERWRITE`。
fn propagate(pool: &mut MemoryPool, units: &[(u64, usize)], mut changes: Vec<(u64, u32, u32)>) {
    while let Some((slot, old, new)) = changes.pop() {
        if old == new {
            continue;
        }
        for (container, length) in units.iter().copied() {
            if container == slot {
                continue;
            }
            let mut bytes = read(pool, 0, container * SLOT, length);
            let mut touched = false;
            for device in DEVICES {
                touched |= replace_all(
                    &mut bytes,
                    &location_pattern(device, slot, old),
                    &location_pattern(device, slot, new),
                );
            }
            if touched {
                let before = crc32_castagnoli(&read(pool, 0, container * SLOT, length));
                reseal_unit(&mut bytes);
                for device in DEVICES {
                    write(pool, device, container * SLOT, &bytes);
                }
                changes.push((container, before, crc32_castagnoli(&bytes)));
            }
        }
        for (device, offset) in root_slots() {
            let mut bytes = read(pool, device, offset, 512);
            let mut touched = false;
            for location_device in DEVICES {
                touched |= replace_all(
                    &mut bytes,
                    &location_pattern(location_device, slot, old),
                    &location_pattern(location_device, slot, new),
                );
            }
            if touched {
                let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
                bytes[138..170].copy_from_slice(&digest);
                write(pool, device, offset, &bytes);
            }
        }
    }
    copy_each_root_into_the_new_root_segment_of_its_journal_records(pool);
}

/// journal 记录头里 `checkpoint_txg`（8 字节）的偏移（D23（journal 的角色与格式） 已定项 4 / 已定项 17 的字段表）。
const JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET: usize = 26;

/// 只改一块盘上的那一份单元（另一块盘留着原样）：重封被改那一份的两道校验和，再把它的新整单元校验和补进引用它的
/// **那一块盘那一条**位置条目——位置条目逐盘各带一个整单元校验和（D4（校验和位置），指针 86 / 88 里位置条目 14 × 2），
/// 所以补完两份各自仍与自己那条位置条目对得上，I-2.1（校验和与内容匹配） 不跟着红；父单元两盘同内容，它自己的整单元校验和
/// 往上补那一段照旧走 `propagate`。
fn mutate_one_device_copy_of_unit(
    pool: &mut MemoryPool,
    units: &[(u64, usize)],
    device: u32,
    slot: u64,
    change: impl Fn(&mut Vec<u8>),
) {
    let length = units
        .iter()
        .find(|(start, _)| *start == slot)
        .expect("登记过的单元")
        .1;
    let before = read(pool, device, slot * SLOT, length);
    let mut bytes = before.clone();
    change(&mut bytes);
    reseal_unit(&mut bytes);
    write(pool, device, slot * SLOT, &bytes);
    let (old, new) = (crc32_castagnoli(&before), crc32_castagnoli(&bytes));
    let mut parents_changed = Vec::new();
    for (container, container_length) in units.iter().copied() {
        if container == slot {
            continue;
        }
        let container_before = read(pool, 0, container * SLOT, container_length);
        let mut container_bytes = container_before.clone();
        if replace_all(
            &mut container_bytes,
            &location_pattern(device, slot, old),
            &location_pattern(device, slot, new),
        ) {
            reseal_unit(&mut container_bytes);
            for each_device in DEVICES {
                write(pool, each_device, container * SLOT, &container_bytes);
            }
            parents_changed.push((
                container,
                crc32_castagnoli(&container_before),
                crc32_castagnoli(&container_bytes),
            ));
        }
    }
    for (root_device, offset) in root_slots() {
        let mut root_bytes = read(pool, root_device, offset, 512);
        if replace_all(
            &mut root_bytes,
            &location_pattern(device, slot, old),
            &location_pattern(device, slot, new),
        ) {
            let digest = wide_checksum_with_field_zeroed(&root_bytes, 512, 138);
            root_bytes[138..170].copy_from_slice(&digest);
            write(pool, root_device, offset, &root_bytes);
        }
    }
    propagate(pool, units, parents_changed);
}

/// 改一个单元（两盘一起改）：`reseal` 为真时重封它的两道校验和；新的整单元校验和沿引用链补上去。
fn mutate_unit_of(
    pool: &mut MemoryPool,
    units: &[(u64, usize)],
    slot: u64,
    change: impl Fn(&mut Vec<u8>),
    reseal: bool,
) {
    let length = units
        .iter()
        .find(|(start, _)| *start == slot)
        .expect("登记过的单元")
        .1;
    let before = read(pool, 0, slot * SLOT, length);
    let mut bytes = before.clone();
    change(&mut bytes);
    if reseal {
        reseal_unit(&mut bytes);
    }
    for device in DEVICES {
        write(pool, device, slot * SLOT, &bytes);
    }
    propagate(
        pool,
        units,
        vec![(slot, crc32_castagnoli(&before), crc32_castagnoli(&bytes))],
    );
}

/// 写完新池新建文件那份镜像上改一个单元。
fn mutate_unit(pool: &mut MemoryPool, slot: u64, change: impl Fn(&mut Vec<u8>), reseal: bool) {
    mutate_unit_of(pool, &UNITS, slot, change, reseal);
}

/// 分配记录树的两片叶（两块盘各一片）逐片改、各自重封，新的整单元校验和沿引用链补上去（叶 → 那块盘的第 1 层节点 → 根 → 树表 → 根槽）；
/// `change` 返回它在那一片叶里改了几条记录，这里交回两片合起来的条数。
fn mutate_allocation_leaves(
    pool: &mut MemoryPool,
    units: &[(u64, usize)],
    leaves: [u64; 2],
    change: impl Fn(&mut Vec<u8>) -> usize,
) -> usize {
    let changed_records = std::cell::Cell::new(0usize);
    for leaf in leaves {
        mutate_unit_of(
            pool,
            units,
            leaf,
            |bytes| changed_records.set(changed_records.get() + change(bytes)),
            true,
        );
    }
    changed_records.get()
}

fn verdict(pool: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(pool)
        .into_iter()
        .find(|(identifier, _)| *identifier == invariant)
        .expect("清单里有")
        .1
}

/// journal 记录头里这三个字段的偏移（D23（journal 的角色与格式） 已定项 17 的字段表；记录头自证校验和在 46、罩整条 4096）。
const JOURNAL_RECORD_INSTANCE_OFFSET: usize = 16;
const JOURNAL_RECORD_TRANSACTION_OFFSET: usize = 78;

/// 这几份镜像上环里的记录条数上界：读到这个计数器还没读完就说明镜像变了，断言会响（别让列表被悄悄截断）。
const RECORDS_READ_BACK_AT_MOST: u64 = 64;

/// 一块盘的 journal 环里从计数器 1 起连着的记录，读成 (计数器, 实例代号, 事务号)；读到没有 magic 的那一格就停。
/// 计数器与环槽一一对应（`record_offset`），所以这里按计数器取偏移，不整条环扫。
fn journal_records_on_device(pool: &MemoryPool, device: u32) -> Vec<(u64, u32, u64)> {
    let mut records = Vec::new();
    for counter in 1..=RECORDS_READ_BACK_AT_MOST {
        let offset = singlefs_core::journal::record_offset(
            counter,
            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        )
        .0;
        let bytes = read(pool, device, offset, 4096);
        if bytes[..4] != singlefs_core::journal::JOURNAL_MAGIC {
            break;
        }
        records.push((
            counter,
            u32::from_le_bytes(
                bytes[JOURNAL_RECORD_INSTANCE_OFFSET..JOURNAL_RECORD_INSTANCE_OFFSET + 4]
                    .try_into()
                    .expect("4 字节"),
            ),
            get_u64(&bytes, JOURNAL_RECORD_TRANSACTION_OFFSET),
        ));
    }
    assert!(
        u64::try_from(records.len()).expect("条数") < RECORDS_READ_BACK_AT_MOST,
        "环里的记录多到读满了上界 {RECORDS_READ_BACK_AT_MOST}：这几份镜像上不该有这么多"
    );
    records
}

/// 一条 journal 记录的事务号改成 `transaction`，头校验和重封（罩整条 4096，字段在 46）；两盘各一份都改——
/// 事务号重号是写者算错号，不是一块盘的字节坏了，两盘会写出同一条记录。
fn set_transaction_number_of_journal_record(
    pool: &mut MemoryPool,
    counter: u64,
    expected_transaction: u64,
    transaction: u64,
) {
    let offset =
        singlefs_core::journal::record_offset(counter, singlefs_format::JOURNAL_RING_DEFAULT_BYTES)
            .0;
    for device in DEVICES {
        let mut record = read(pool, device, offset, 4096);
        assert_eq!(
            get_u64(&record, JOURNAL_RECORD_TRANSACTION_OFFSET),
            expected_transaction,
            "盘 {device} 计数器 {counter} 那条记录的事务号"
        );
        set_u64(&mut record, JOURNAL_RECORD_TRANSACTION_OFFSET, transaction);
        let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
        record[46..78].copy_from_slice(&digest);
        write(pool, device, offset, &record);
    }
}

/// 系统配置字段表（`.claude/kb/layout/01-first-txn.md` 一）：加密类型在整槽校验和（155，宽 32）、自身 MAC（16）、nonce 水位（12）、
/// KDF 标识（4）之后，偏移 219；第一版只读 0（不加密）。
const SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET: usize = 219;
/// 登记过的加密算法，第一版读者不读加密的卷：带它的系统配置槽自证过也整池拒绝挂载。
const ENCRYPTION_TYPE_THE_FIRST_VERSION_DOES_NOT_READ: u8 = 1;
/// I-7.13（系统配置池级字段在读者收的范围里） 那份坏镜像改的那一槽：盘 1 的槽 0。
const DEVICE_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT: u32 = 1;
const OFFSET_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT: u64 = 0;
/// 系统配置字段表（`.claude/kb/layout/01-first-txn.md` 一）几何段里几个 16 KiB 槽号与环长的偏移：journal 环起点 325、环长 333、
/// 根环起点 371、单元区起始槽号 417。checker 与实现今天各自按这几个偏移读；这里照字段表各写一份。
const SYSTEM_CONFIGURATION_JOURNAL_RING_START_SLOT_OFFSET: usize = 325;
const SYSTEM_CONFIGURATION_JOURNAL_RING_BYTES_OFFSET: usize = 333;
const SYSTEM_CONFIGURATION_ROOT_RING_BASE_SLOT_OFFSET: usize = 371;
const SYSTEM_CONFIGURATION_UNIT_AREA_START_SLOT_OFFSET: usize = 417;
/// 系统配置自举头里的本盘设备号（4 字节）：fsid（偏移 102，宽 16）之后 20 字节、再 1 字节之后，偏移 139。
const SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_OFFSET: usize = 139;
/// 第一版 mkfs 写进这几格的值（字段表那一列）：journal 环从槽 1024 起、默认环长 768 MiB、根环从槽 64 起、单元区从槽 50176 起。
const JOURNAL_RING_START_SLOT_OF_THE_POOL: u64 = 1024;
const JOURNAL_RING_BYTES_OF_THE_POOL: u64 = 805_306_368;
const ROOT_RING_BASE_SLOT_OF_THE_POOL: u64 = 64;
const UNIT_AREA_START_SLOT_OF_THE_POOL: u64 = 50176;
/// 聚簇段 64 槽（D3（空间分配） 已定项 10 ①）：单元区起始槽号要落在它的整数倍上。
const CLUSTER_SEGMENT_SLOTS_OF_THE_POOL: u64 = 64;

/// 系统配置槽重封：自证校验和在 155，罩整槽 4096。
fn mutate_system_configuration_slot(
    pool: &mut MemoryPool,
    device: u32,
    slot_offset: u64,
    change: impl Fn(&mut Vec<u8>),
) {
    let mut bytes = read(pool, device, slot_offset, 4096);
    change(&mut bytes);
    let digest = wide_checksum_with_field_zeroed(&bytes, 4096, 155);
    bytes[155..187].copy_from_slice(&digest);
    write(pool, device, slot_offset, &bytes);
}

fn set_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
fn set_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
fn get_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4 字节"))
}

fn set_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

/// 两块盘对调盘体：每块盘两个系统配置槽里的本盘设备号改成另一块盘的身份（盘 0 的两槽写 1、盘 1 的两槽写 0），整槽重封，
/// 别的字节不动。与盘体真对调不同的只有这一格：根环区域、单元都还在原来的盘上，所以别的不变量照判成立。
fn swap_the_own_device_numbers_of_the_two_devices(image: &mut MemoryPool) {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    for (device, own_device_number_written) in [(0u32, 1u32), (1, 0)] {
        for slot_offset in [0, spacing] {
            mutate_system_configuration_slot(image, device, slot_offset, |bytes| {
                set_u32(
                    bytes,
                    SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_OFFSET,
                    own_device_number_written,
                );
            });
        }
    }
}

/// I-7.13（系统配置池级字段在读者收的范围里） 的几何那五格（实审 Y4 的 A、B、C、D、E，用户 2026-09-27 定「读字段，不等于常量就整池拒」）：
/// 盘 1 槽 0 的一个几何字段改掉、整槽重封，别的三槽照原样。每项带 checker 该报的那个 `Verdict` 成员名（违例说明里带着它）。
fn known_bad_images_of_the_system_configuration_geometry() -> Vec<(&'static str, Mutation)> {
    let on_the_refused_slot = |change: fn(&mut Vec<u8>)| -> Mutation {
        Box::new(move |image: &mut MemoryPool| {
            mutate_system_configuration_slot(
                image,
                DEVICE_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT,
                OFFSET_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT,
                change,
            );
        })
    };
    vec![
        // A：单元区起始槽号往后挪一段（50176 → 50240），环长不动：不是环长现算的那个槽。
        (
            "UnitAreaStartNotTheSlotAfterTheJournalRing",
            on_the_refused_slot(|bytes| {
                set_u64(
                    bytes,
                    SYSTEM_CONFIGURATION_UNIT_AREA_START_SLOT_OFFSET,
                    UNIT_AREA_START_SLOT_OF_THE_POOL + CLUSTER_SEGMENT_SLOTS_OF_THE_POOL,
                );
            }),
        ),
        // D：环长少一槽（768 MiB − 16 KiB），单元区起始槽号跟着改成它现算的 50175（两者自洽）：不在聚簇段边界上。
        (
            "UnitAreaStartOffTheClusterSegmentBoundary",
            on_the_refused_slot(|bytes| {
                set_u64(
                    bytes,
                    SYSTEM_CONFIGURATION_JOURNAL_RING_BYTES_OFFSET,
                    JOURNAL_RING_BYTES_OF_THE_POOL - SLOT,
                );
                set_u64(
                    bytes,
                    SYSTEM_CONFIGURATION_UNIT_AREA_START_SLOT_OFFSET,
                    UNIT_AREA_START_SLOT_OF_THE_POOL - 1,
                );
            }),
        ),
        // B：journal 环起点写 1023：实现按常量 1024 写记录、扫环，按字段扫的读者判的是错位 16 KiB 的另一个环。
        (
            "JournalRingStartSlotNotTheFirstVersionConstant",
            on_the_refused_slot(|bytes| {
                set_u64(
                    bytes,
                    SYSTEM_CONFIGURATION_JOURNAL_RING_START_SLOT_OFFSET,
                    JOURNAL_RING_START_SLOT_OF_THE_POOL - 1,
                );
            }),
        ),
        // C：根环起点写 65：实现按常量 64 写根，按字段读的读者读的是另一个根环。
        (
            "RootRingBaseSlotNotTheFirstVersionConstant",
            on_the_refused_slot(|bytes| {
                set_u64(
                    bytes,
                    SYSTEM_CONFIGURATION_ROOT_RING_BASE_SLOT_OFFSET,
                    ROOT_RING_BASE_SLOT_OF_THE_POOL + 1,
                );
            }),
        ),
        // E：根环起点写 0：先于槽距判，报起点、不报「槽 1 不在根环起点之前」。
        (
            "RootRingBaseSlotNotTheFirstVersionConstant",
            on_the_refused_slot(|bytes| {
                set_u64(bytes, SYSTEM_CONFIGURATION_ROOT_RING_BASE_SLOT_OFFSET, 0);
            }),
        ),
    ]
}

/// 树表单元里第 index 条条目（条目区从 115 + 2 × 8 = 131 起，每条 200）。
fn tree_table_entry_offset(index: usize) -> usize {
    131 + 200 * index
}

/// 一个码 2 节点的自描述三段（D18（块里携带什么信息） 已定项 7：key 宽 1 字节在 51、条目数 2 字节在 82 + 2k、
/// 条目宽 2 字节在 84 + 2k；声明长度 2 字节在 8，条目区从 86 + 2k + 29 起）改成「条目宽 = key 宽、只装一条条目」：
/// 条目数改 1、声明长度改 k、largest_key 贴到那一条条目的 key 上、条目区之后的字节清零。
///
/// 这么改是为了让**除 I-1.10（码 2 条目宽等于字段表宽） 之外的每一道都还成立**：声明长度 = 条目数 × 条目宽、
/// 条目宽 ≥ key 宽（`index_node_view` 判的那两道）、key 区间贴紧首末条目（I-1.1（块头自述逻辑地址））、
/// 声明长度之后的补齐为零且在载荷 CRC 内（I-2.3（补齐字节为零且参与校验和））。
/// 盘上留下的就是一个处处自洽、唯独条目宽与那棵树的字段表不符的节点——
/// I-1.10（码 2 条目宽等于字段表宽） 判的正是这一格，判不出来它，走读下一步就按字段表的固定偏移切这条 k 字节的条目、当场越界
/// （panic 面普查 R13）。
fn narrow_the_entry_width_to_the_key_width(bytes: &mut [u8]) {
    let key_width = usize::from(bytes[51]);
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

/// 码 2 节点头里出生序号那 4 字节的偏移**扣掉 2k**（D18（块里携带什么信息） 已定项 7 的字段表：
/// 诞生代号 52 + 2k、fsid 60 + 2k、写序 68 + 2k、出生序号 72 + 2k；k 是节点自述的 key 宽，在偏移 51）。
const INDEX_NODE_BIRTH_SEQUENCE_OFFSET_BEFORE_THE_KEYS: usize = 72;
/// 码 1 数据单元头里诞生代号那 8 字节的偏移（D18（块里携带什么信息） 已定项 7 的字段表）。
const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;
/// extent 树上段叶（key 宽 24）里第一条条目内联的那条数据指针中诞生代号那 8 字节的偏移：
/// 条目区从 115 + 2 × 24 = 163 起，条目 = key (0, inode 8, 0) 24 + 标签 1（2 = 单单元内联）+ 数据指针 88
/// （D8（核心索引结构） 已定项 14），指针里诞生代号在 42。
const EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET: usize = 163 + 24 + 1 + 42;
/// 一条树表条目里诞生 txg 那 8 字节的偏移：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8
/// （D8（核心索引结构） 已定项 8 的字段表）。
const TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8;
/// 分配记录树叶（key 宽 10）：条目数在 82 + 2 × 10、条目区从 115 + 2 × 10 起，每条 20 = key (设备 4, 槽号 6) +
/// value (跨度段 2，最高位是已释放标志, 分配代或释放代 8)。
const ALLOCATION_NODE_ENTRY_COUNT_OFFSET: usize = 102;
const ALLOCATION_NODE_ENTRY_START: usize = 135;
const ALLOCATION_RECORD_BYTES: usize = 20;
const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;

fn get_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

/// 中央映射树（根兼叶，key 宽 27、条目 55）里唯一那条码 1 条目：key 的出生 txg（key 偏移 9）从 `from` 改成 `to`。
/// 码 1 排在最前（类标签 1 最小），头里的最小 key 就是它的 key，跟着改；改完仍按 key 升序。
fn rewrite_the_data_unit_mapping_key_birth_txg(bytes: &mut [u8], from: u64, to: u64) {
    const MAPPING_KEY_BIRTH_TXG_OFFSET: usize = 9;
    const HEADER_SMALLEST_KEY_OFFSET: usize = 52;
    let key_width = usize::from(bytes[51]);
    assert_eq!(key_width, 27, "映射 key 宽 27");
    let entries_start = 115 + 2 * key_width;
    assert_eq!(bytes[entries_start], 1, "第一条是码 1 那一条");
    assert_eq!(
        bytes[entries_start..entries_start + key_width],
        bytes[HEADER_SMALLEST_KEY_OFFSET..HEADER_SMALLEST_KEY_OFFSET + key_width],
        "头里的最小 key 就是第一条的 key"
    );
    for key_start in [entries_start, HEADER_SMALLEST_KEY_OFFSET] {
        assert_eq!(
            get_u64(bytes, key_start + MAPPING_KEY_BIRTH_TXG_OFFSET),
            from,
            "码 1 映射 key 的出生 txg 本来是 {from}"
        );
        set_u64(bytes, key_start + MAPPING_KEY_BIRTH_TXG_OFFSET, to);
    }
}

/// 实审 B2 新立的「中央映射条目的 key 与它指的单元头相符」：I-1.11（映射 key 与单元头相符）。
const MAPPING_KEY_INVARIANT: &str = singlefs_checker::image::MAPPING_KEY_MATCHES_THE_UNIT_HEADER;

/// 中央映射树（根兼叶，key 宽 27、条目 55）里指着 `slot` 那个码 2 节点的那条条目：key 尾段的出生序号（key 偏移 21，4 字节）+1。
/// 头里的最小 / 最大 key 是它时跟着改；改完仍按 key 严格升序（码 2 的 key 按出生树分开，这一条与左右邻不同出生树）。
fn raise_the_mapping_key_tail_of_the_node_at(bytes: &mut [u8], slot: u64) {
    const MAPPING_KEY_TAIL_OFFSET: usize = 21;
    const MAPPING_ENTRY_BYTES: usize = 55;
    const MAPPING_ENTRY_FIRST_LOCATION_SLOT_OFFSET: usize = 27 + 4;
    const HEADER_SMALLEST_KEY_OFFSET: usize = 52;
    let key_width = usize::from(bytes[51]);
    assert_eq!(key_width, 27, "映射 key 宽 27");
    let header_largest_key_offset = HEADER_SMALLEST_KEY_OFFSET + key_width;
    let entry_count = usize::from(u16::from_le_bytes([
        bytes[82 + 2 * key_width],
        bytes[83 + 2 * key_width],
    ]));
    let entries_start = 115 + 2 * key_width;
    let entry_start = (0..entry_count)
        .map(|index| entries_start + index * MAPPING_ENTRY_BYTES)
        .find(|start| {
            bytes[*start] == 2
                && get_u64(bytes, start + MAPPING_ENTRY_FIRST_LOCATION_SLOT_OFFSET)
                    & 0xFFFF_FFFF_FFFF
                    == slot
        })
        .expect("映射里有指着这个码 2 节点的那一条");
    let old_key = bytes[entry_start..entry_start + key_width].to_vec();
    let tail = entry_start + MAPPING_KEY_TAIL_OFFSET;
    let birth_sequence = u32::from_le_bytes(bytes[tail..tail + 4].try_into().expect("4 字节"));
    bytes[tail..tail + 4].copy_from_slice(&(birth_sequence + 1).to_le_bytes());
    let new_key = bytes[entry_start..entry_start + key_width].to_vec();
    for header_key in [HEADER_SMALLEST_KEY_OFFSET, header_largest_key_offset] {
        if bytes[header_key..header_key + key_width] == old_key[..] {
            bytes[header_key..header_key + key_width].copy_from_slice(&new_key);
        }
    }
}

/// 分配记录树叶里，把代字段等于 `from` 的已释放记录改成 `to`；返回改了几条。
fn rewrite_released_generations(bytes: &mut [u8], from: u64, to: u64) -> usize {
    let count = usize::from(u16::from_le_bytes([
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
    ]));
    let mut rewritten = 0;
    for index in 0..count {
        let record = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * index;
        let span_field = u16::from_le_bytes([bytes[record + 10], bytes[record + 11]]);
        if span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0 && get_u64(bytes, record + 12) == from
        {
            set_u64(bytes, record + 12, to);
            rewritten += 1;
        }
    }
    rewritten
}

/// 分配记录树叶里槽号是 `slot` 的记录（两盘各一条）跨度改成 `span_slots`，已释放标志照留；返回改了几条。
fn set_allocation_record_span(bytes: &mut [u8], slot: u64, span_slots: u16) -> usize {
    let count = usize::from(u16::from_le_bytes([
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
    ]));
    let mut rewritten = 0;
    for index in 0..count {
        let record = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * index;
        let mut slot_bytes = [0u8; 8];
        slot_bytes[..6].copy_from_slice(&bytes[record + 4..record + 10]);
        if u64::from_le_bytes(slot_bytes) == slot {
            let span_field = u16::from_le_bytes([bytes[record + 10], bytes[record + 11]]);
            set_u16(
                bytes,
                record + 10,
                (span_field & ALLOCATION_RECORD_RELEASED_FLAG) | span_slots,
            );
            rewritten += 1;
        }
    }
    rewritten
}

/// 一份坏镜像：它要让哪一条判违例，以及怎么从干净镜像改出来。
type Mutation = Box<dyn Fn(&mut MemoryPool)>;

fn known_bad_images(clean: &MemoryPool) -> Vec<(&'static str, Mutation)> {
    let leaf_head_checksum = crc32_castagnoli(&read(clean, 0, INODE_LEAF * SLOT, 16384));
    let stray_node = {
        let mut stray = read(clean, 0, EXTENT_ROOT * SLOT, 16384);
        set_u64(&mut stray, 42, 65624);
        reseal_unit(&mut stray);
        stray
    };
    let orphan_with_instance_two = {
        let mut orphan = read(clean, 0, DATA_UNIT * SLOT, 32768);
        orphan[91..95].copy_from_slice(&2u32.to_le_bytes());
        reseal_unit(&mut orphan);
        orphan
    };
    let orphan_for_second_sentence = orphan_with_instance_two.clone();
    let record_with_instance_two = {
        let offset =
            singlefs_core::journal::record_offset(3, singlefs_format::JOURNAL_RING_DEFAULT_BYTES).0;
        let mut record = read(clean, 0, offset, 4096);
        record[16..20].copy_from_slice(&2u32.to_le_bytes());
        let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
        record[46..78].copy_from_slice(&digest);
        record
    };
    vec![
        // 第 0 代根（暖机 txg 1、2）引用的 mkfs 树表单元（50178，A 换下之后只有它们引用）内容改掉再重封：那条候选根指着的块被复用。
        (
            "I-7.4",
            Box::new(|image: &mut MemoryPool| {
                let mut unit = read(image, 0, GENESIS_TREE_TABLE * SLOT, 16384);
                set_u64(&mut unit, 42, 77);
                reseal_unit(&mut unit);
                for device in DEVICES {
                    write(image, device, GENESIS_TREE_TABLE * SLOT, &unit);
                }
            }),
        ),
        // 同一个单元的头抹成零、不重封：从第 0 代根出发的遍历读不到它。
        (
            "I-4.8",
            Box::new(|image: &mut MemoryPool| {
                let erased = vec![0u8; 16384];
                for device in DEVICES {
                    write(image, device, GENESIS_TREE_TABLE * SLOT, &erased);
                }
            }),
        ),
        // 数据单元五元组里的锚点偏移改成 32768（extent key 的偏移是 0）。
        (
            "I-1.1",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(image, DATA_UNIT, |bytes| set_u64(bytes, 67, 32768), true)
            }),
        ),
        // extent 树根头里的树 ID 改成 99。
        (
            "I-1.3",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(image, EXTENT_ROOT, |bytes| set_u64(bytes, 42, 99), true)
            }),
        ),
        // inode 叶容器头里的 fsid 改掉。
        (
            "I-1.4",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(image, INODE_LEAF, |bytes| set_u64(bytes, 81, 0xDEAD), true)
            }),
        ),
        // 盘 1 被择的那个系统配置槽里 fsid 改成别的池的（一块外来的旧盘）；单元头照旧。
        (
            "I-1.4",
            Box::new(|image: &mut MemoryPool| {
                mutate_system_configuration_slot(image, 1, 4096, |bytes| bytes[102] ^= 0xFF)
            }),
        ),
        // 码 3 容器类身份段的首字节改成 1。
        (
            "I-1.6",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(image, INODE_LEAF, |bytes| bytes[42] = 1, true)
            }),
        ),
        // 类型 2 容器的记录宽改成 139（登记的是 140）。
        (
            "I-1.7",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(image, INODE_LEAF, |bytes| set_u16(bytes, 71, 139), true)
            }),
        ),
        // 记账树根自述的条目宽缩到 key 宽 22（登记的字段表宽 34），别的自描述字段跟着调成自洽的一份。
        (
            "I-1.10",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    ACCOUNTING_ROOT,
                    |bytes| narrow_the_entry_width_to_the_key_width(bytes),
                    true,
                )
            }),
        ),
        // 数据单元盘 0 那一份翻一个载荷字节，不补任何校验和。
        (
            "I-2.1",
            Box::new(|image: &mut MemoryPool| {
                let mut bytes = read(image, 0, DATA_UNIT * SLOT, 32768);
                bytes[200] ^= 1;
                write(image, 0, DATA_UNIT * SLOT, &bytes);
            }),
        ),
        // 数据单元补齐区写一个非零字节，载荷 CRC 跟着重算（校验和对、补齐不为 0）。
        (
            "I-2.3",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(image, DATA_UNIT, |bytes| bytes[32000] = 1, true)
            }),
        ),
        // 数据单元头里翻一个字节，不重封头校验和（整单元校验和照样补上去，I-2.1 不先挡）。
        (
            "I-2.4",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(image, DATA_UNIT, |bytes| bytes[60] ^= 1, false)
            }),
        ),
        // 树表里 extent 树那条的根指针，两条位置条目对调（盘 1 在前）。
        (
            "I-2.5",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    TREE_TABLE,
                    |bytes| {
                        let pointer = tree_table_entry_offset(0) + 14;
                        let first: Vec<u8> = bytes[pointer + 50..pointer + 64].to_vec();
                        let second: Vec<u8> = bytes[pointer + 64..pointer + 78].to_vec();
                        bytes[pointer + 50..pointer + 64].copy_from_slice(&second);
                        bytes[pointer + 64..pointer + 78].copy_from_slice(&first);
                    },
                    true,
                );
            }),
        ),
        // 记账树里盘 0 的「已分配」多记 16384。
        (
            "I-3.1",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    ACCOUNTING_ROOT,
                    |bytes| adjust_accounting(bytes, 1, 0, 16384),
                    true,
                )
            }),
        ),
        // 树表里分配记录树那条的根指针，盘 0 那条位置条目改指槽 50243（压在 inode 叶容器的第二个槽上）。
        (
            "I-5.1",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    TREE_TABLE,
                    |bytes| {
                        let location = tree_table_entry_offset(2) + 14 + 50 + 4;
                        bytes[location..location + 6].copy_from_slice(&50243u64.to_le_bytes()[..6]);
                    },
                    true,
                );
            }),
        ),
        // 记账树里盘 0 的「空闲」多记 16384。
        (
            "I-5.2",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    ACCOUNTING_ROOT,
                    |bytes| adjust_accounting(bytes, 2, 0, 16384),
                    true,
                )
            }),
        ),
        // 根环全部槽抹成 0。
        (
            "I-7.1",
            Box::new(|image: &mut MemoryPool| {
                for (device, offset) in root_slots() {
                    write(image, device, offset, &[0u8; 512]);
                }
            }),
        ),
        // inode 叶容器两份都改坏一个字节（不补校验和）：最新的根走不完。
        (
            "I-7.2",
            Box::new(|image: &mut MemoryPool| {
                for device in DEVICES {
                    let mut bytes = read(image, device, INODE_LEAF * SLOT, 32768);
                    bytes[5000] ^= 1;
                    write(image, device, INODE_LEAF * SLOT, &bytes);
                }
            }),
        ),
        // 盘 0 被择的那个系统配置槽（槽 1，世代 5）里逐区域设备改成 [0, 0, 0]。
        (
            "I-7.6",
            Box::new(|image: &mut MemoryPool| {
                mutate_system_configuration_slot(image, 0, 4096, |bytes| {
                    bytes[379..391].copy_from_slice(&[0u8; 12])
                })
            }),
        ),
        // 盘 1 槽 0 的加密类型改成 1、重算整槽校验和：这一槽自证过而带这一版读者不收的值，实现整池拒绝挂载
        // （A3-checker-2 第四节第 1 条，别的三槽照原样）。
        (
            SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
            Box::new(|image: &mut MemoryPool| {
                mutate_system_configuration_slot(
                    image,
                    DEVICE_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT,
                    OFFSET_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT,
                    |bytes| {
                        bytes[SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET] =
                            ENCRYPTION_TYPE_THE_FIRST_VERSION_DOES_NOT_READ
                    },
                )
            }),
        ),
        // 两块盘四个系统配置槽里的本盘设备号互换、整槽重封（实审 Y2-b 越格线索：盘体对调之后的池可写挂载因本盘设备号拒）。
        (
            SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
            Box::new(swap_the_own_device_numbers_of_the_two_devices),
        ),
        // C322（取号那一步的屏障怎么放没有条款） 的撞号镜像：没人引用的槽 50302 上放一份写序实例代号 2 的数据单元（头与载荷
        // 校验和都过），两盘系统配置都还是实例 1 ⇒ ① 红（旧的「各盘相等且 ≥ 根环」在这里判成立）。
        (
            "I-7.7",
            Box::new(move |image: &mut MemoryPool| {
                write(image, 0, 50302 * SLOT, &orphan_with_instance_two)
            }),
        ),
        // journal 环里一个空槽放一条实例代号 2 的记录（头校验和重封）⇒ ① 红：扫描方向要读 journal 环。
        (
            "I-7.7",
            Box::new(move |image: &mut MemoryPool| {
                write(
                    image,
                    0,
                    singlefs_core::journal::record_offset(
                        9,
                        singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
                    )
                    .0,
                    &record_with_instance_two,
                )
            }),
        ),
        // 盘 0 较新那一槽的实例代号改成 2，再放那份实例代号 2 的孤儿 ⇒ 各盘最大号 [2, 1] 不等而较大者有东西带着，② 红（① 成立）。
        (
            "I-7.7",
            Box::new(move |image: &mut MemoryPool| {
                mutate_system_configuration_slot(image, 0, 4096, |bytes| {
                    bytes[477..481].copy_from_slice(&2u32.to_le_bytes())
                });
                write(image, 0, 50302 * SLOT, &orphan_for_second_sentence);
            }),
        ),
        // 在没人引用的槽 50300 上放一个树 ID 65624（高于水位 65548）的码 2 节点（头校验和过、诞生代号 3）。
        (
            "I-7.8",
            Box::new(move |image: &mut MemoryPool| write(image, 0, 50300 * SLOT, &stray_node)),
        ),
        // 树表里 inode 树那条的根指针改指 inode 叶容器（码 3），校验和取那个容器前 16 KiB 的 CRC。
        (
            "I-9.1",
            Box::new(move |image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    TREE_TABLE,
                    |bytes| {
                        let pointer = tree_table_entry_offset(1) + 14;
                        for location in [pointer + 50, pointer + 64] {
                            bytes[location + 4..location + 10]
                                .copy_from_slice(&INODE_LEAF.to_le_bytes()[..6]);
                            bytes[location + 10..location + 14]
                                .copy_from_slice(&leaf_head_checksum.to_le_bytes());
                        }
                    },
                    true,
                );
            }),
        ),
        // inode 树根那条内部条目的身份引用里容器号改成 2（子头里是 1）。
        (
            "I-9.2",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(image, INODE_ROOT, |bytes| set_u64(bytes, 131 + 18, 2), true)
            }),
        ),
        // 容器号改成 5，而容器里那条记录的 inode 是 1。
        (
            "I-9.4",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(image, INODE_LEAF, |bytes| set_u64(bytes, 53, 5), true)
            }),
        ),
        // inode 记录的预留段（记录偏移 139）写一个非零字节。
        (
            "I-9.7",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(image, INODE_LEAF, |bytes| bytes[136 + 139] = 1, true)
            }),
        ),
        // inode 记录的对象出生代改成 2（数据单元五元组里是 3）。
        (
            "I-9.10",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(image, INODE_LEAF, |bytes| set_u64(bytes, 136 + 8, 2), true)
            }),
        ),
        // inode 记录的 blocks（记录偏移 48）写回按分到一个 32 KiB 单元算的 64，而 size 3000 ⇒ ⌈3000 ÷ 512⌉ = 6。
        (
            "I-9.15",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    INODE_LEAF,
                    |bytes| set_u64(bytes, 136 + INODE_RECORD_BLOCKS_OFFSET, 64),
                    true,
                )
            }),
        ),
        // 树表里 livelist 与稀疏旁表两条互换种类：盘上仍按树 ID 升序（I-1.1 不红），按发号次序 livelist 的号 65542 大过稀疏旁表的号 65541。
        (
            "I-9.16",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    TREE_TABLE,
                    |bytes| swap_the_kinds_of_the_livelist_and_sparse_side_table_entries(bytes),
                    true,
                )
            }),
        ),
        // 实例表里多一行 (1, 3, 0)：行的实例代号等于挂载根的实例 1，不「低于」；链指针记录仍在末尾、行仍唯一，红的只有那一半。
        (
            "I-3.8",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    INSTANCE_TABLE,
                    |bytes| {
                        let chain_record: Vec<u8> = bytes[136..224].to_vec();
                        let mut row = vec![0u8; 88];
                        row[1..5].copy_from_slice(&1u32.to_le_bytes());
                        row[5..13].copy_from_slice(&3u64.to_le_bytes());
                        bytes[136..224].copy_from_slice(&row);
                        bytes[224..312].copy_from_slice(&chain_record);
                        set_u16(bytes, 69, 2);
                        set_u16(bytes, 8, 176);
                    },
                    true,
                );
            }),
        ),
        // 盘 0 环内偏移 4096 那条记录（w4，暖机第二条空发布）的反向链字段改成 0，头校验和重封：
        // 它不是本实例第一条（本实例第一条是 w1、计数器 1），链值必须等于 CRC32C(w1 的 487 字节头)。
        (
            "I-8.6",
            Box::new(|image: &mut MemoryPool| {
                let offset = singlefs_core::journal::record_offset(
                    2,
                    singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
                )
                .0;
                let mut record = read(image, 0, offset, 4096);
                let back_chain_field =
                    JOURNAL_RECORD_BACK_CHAIN_OFFSET..JOURNAL_RECORD_BACK_CHAIN_OFFSET + 4;
                assert_ne!(
                    u32::from_le_bytes(
                        record[back_chain_field.clone()].try_into().expect("4 字节")
                    ),
                    0,
                    "w4 的反向链本来是 CRC32C(w1 的头)，非 0"
                );
                record[back_chain_field].copy_from_slice(&0u32.to_le_bytes());
                let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
                record[46..78].copy_from_slice(&digest);
                write(image, 0, offset, &record);
            }),
        ),
        // inode 叶容器**盘 1 那一份**的载荷改一个字节、重封载荷校验和与头校验和，类身份段与写序一个字节不动：
        // 两份归并成同一组（归并键逐字节相同）而载荷不同。盘 1 那条位置条目跟着补上 ⇒ I-2.1 不先挡。
        (
            "I-1.8",
            Box::new(|image: &mut MemoryPool| {
                mutate_one_device_copy_of_unit(image, &UNITS, 1, INODE_LEAF, |bytes| {
                    assert_eq!(bytes[20000], 0, "记录区之后的补齐本来是 0");
                    bytes[20000] = 1;
                });
            }),
        ),
        // extent 树根（码 2，key 宽 24）头里的出生序号 +1，引用它的树表条目里那条指针一个字节不动：
        // 头里的出生身份与引用它的指针记的不符 ⇒ I-1.2（块头写序已发布） 红。
        // 出生序号不是已发布谓词的输入（谓词只读诞生代号与写序），所以 I-4.2（无被引用未提交块） 不跟着红。
        // 中央映射里 extent 根那条 key 的尾段就是它的出生序号（D19（块指针的结构与宽度预算） 已定项 6）：跟着 +1，
        // 不跟着改就是映射 key 与单元头不符、映射 key 那一条（I-1.11（映射 key 与单元头相符）） 一起红。
        (
            "I-1.2",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    EXTENT_ROOT,
                    |bytes| {
                        let offset = INDEX_NODE_BIRTH_SEQUENCE_OFFSET_BEFORE_THE_KEYS
                            + 2 * usize::from(bytes[51]);
                        let birth_sequence = u32::from_le_bytes(
                            bytes[offset..offset + 4].try_into().expect("4 字节"),
                        );
                        bytes[offset..offset + 4]
                            .copy_from_slice(&(birth_sequence + 1).to_le_bytes());
                    },
                    true,
                );
                mutate_unit(
                    image,
                    MAPPING_ROOT,
                    |bytes| raise_the_mapping_key_tail_of_the_node_at(bytes, EXTENT_ROOT),
                    true,
                );
            }),
        ),
        // 中央映射里数据单元那条 key 的出生 txg 从 3 改成 4，数据单元一个字节不动：映射 key 与它指的单元头里的诞生代号不符
        // ⇒ 映射 key 那一条（I-1.11（映射 key 与单元头相符）） 红。
        (
            MAPPING_KEY_INVARIANT,
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    MAPPING_ROOT,
                    |bytes| rewrite_the_data_unit_mapping_key_birth_txg(bytes, 3, 4),
                    true,
                );
            }),
        ),
        // 数据单元头里的诞生代号从 3 改成 4（最新的根就是 txg 3、实例 1），引用它的 extent 记录里那条数据指针的
        // 诞生代号跟着改成 4：指针与头仍然一致（I-1.2（块头写序已发布） 不跟着红），而这个**被引用**的块按
        // C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的已发布谓词判不成已发布（i == 挂载根实例 ⇒ 要 b ≤ 挂载根的 txg）
        // ⇒ I-4.2（无被引用未提交块） 红：盘上留下的正是「提交了的根指着一个还没提交的块」。
        // 分配记录树里那个落点的分配代也跟着改成 4（两盘各一条）：记录与单元同源，不跟着改就是 I-3.10（已分配记录的分配代等于
        // 它罩住的单元的诞生代号） 一起红，判别力算不到 I-4.2 头上。
        (
            "I-4.2",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    DATA_UNIT,
                    |bytes| {
                        assert_eq!(
                            get_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET),
                            3,
                            "数据单元的诞生代号本来是 3"
                        );
                        set_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET, 4);
                    },
                    true,
                );
                mutate_unit(
                    image,
                    EXTENT_ROOT,
                    |bytes| {
                        assert_eq!(
                            get_u64(bytes, EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET),
                            3,
                            "extent 记录里那条数据指针的诞生代号本来是 3"
                        );
                        set_u64(bytes, EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET, 4);
                    },
                    true,
                );
                // 中央映射里那个数据单元的 key 带着它的诞生代号（D19（块指针的结构与宽度预算） 已定项 6）：跟着改成 4，
                // 不跟着改就是映射 key 与单元头的出生身份不符、I-1.2 一起红（实审 B1 起 checker 拿映射 key 比单元头）。
                mutate_unit(
                    image,
                    MAPPING_ROOT,
                    |bytes| rewrite_the_data_unit_mapping_key_birth_txg(bytes, 3, 4),
                    true,
                );
                assert_eq!(
                    mutate_allocation_leaves(image, &UNITS, ALLOCATION_LEAVES, |bytes| {
                        rewrite_unreleased_allocation_generation(bytes, DATA_UNIT, 3, 4)
                    }),
                    2,
                    "数据单元两盘各一条未释放记录（两片叶各一条），分配代本来是 3"
                );
            }),
        ),
        // 记账里的 inode 号水位从 2 改成 1：树里最大的 key 就是 1，水位不再严格大于它（I-9.6）。
        // 别的行一个字节都不动 ⇒ I-3.1 / I-5.2 那两条照旧绿。
        (
            "I-9.6",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    ACCOUNTING_ROOT,
                    |bytes| set_accounting(bytes, 12, 0xFFFF_FFFF, 1),
                    true,
                );
            }),
        ),
        // inode 根那条条目的分隔 key 从 1 改成 2（连着把节点头的 key 区间也改成 [2, 2]，
        // 不然 I-1.1（key 区间罩住条目） 先红、这一格就成了搭车）：分隔 key 2 大于这个孩子的最小 key 1 ⇒ I-9.12 红。
        (
            "I-9.12",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    INODE_ROOT,
                    |bytes| {
                        set_u64(bytes, 131, 2);
                        set_u64(bytes, 52, 2);
                        set_u64(bytes, 60, 2);
                    },
                    true,
                );
            }),
        ),
        // 类型 2 容器的记录数与声明长度都改成 0。
        (
            "I-9.13",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    INODE_LEAF,
                    |bytes| {
                        set_u16(bytes, 69, 0);
                        set_u16(bytes, 8, 0);
                        bytes[136..276].fill(0);
                    },
                    true,
                );
            }),
        ),
        // I-3.5：A 的数据单元（50180）头里的诞生代号 3 → 2，重封、补到根：A 引用、暖机那一版（txg 2）不引用的单元诞生代号不大于 2，
        // 也不在整棵继承下来的用户可见树里（extent 树根指针出生 txg 3）。指针、映射 key 与分配代还记 3，I-1.2、I-1.11、I-3.10 跟着红，
        // 所以放在这一张表里（只判「它红」，不判「只红它」）；按各个组成部分的几份在
        // `checker_judges_reference_intervals_and_every_new_root_segment_field.rs`。
        (
            REFERENCE_INTERVALS_ARE_EXACT,
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    DATA_UNIT,
                    |bytes| {
                        assert_eq!(
                            get_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET),
                            3,
                            "A 的数据单元生在 txg 3"
                        );
                        set_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET, 2);
                    },
                    true,
                );
            }),
        ),
        // I-5.5（判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K2）：A 那一版是从 mkfs 水位第一次建树的那一版，
        // 它唯一的数据单元（50180 = 单元区起始槽号 50176 + 4）搬到再往后两槽，每块盘上落点最小的数据单元就不在 U + 4。
        // 分配记录与记账照旧记着 50180，这份镜像上不只红 I-5.5；这一格只要它判违例。
        (
            FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
            Box::new(move_the_data_unit_of_the_first_file_two_slots_further),
        ),
    ]
}

/// 新池新建文件写出的那个数据单元（`DATA_UNIT`，32 KiB）原样搬到往后两槽（写完新池新建文件那份镜像上空着，下一个 32 KiB 对齐的槽对）：
/// 两盘各写一份；`UNITS` 里引用它的单元（extent 树根里的数据指针、中央映射叶）把位置条目的槽号改成新的、重封，
/// 新的整单元校验和沿引用链补到根槽。50180 那两份原样留着：journal 记录的点名项仍指着它。
fn move_the_data_unit_of_the_first_file_two_slots_further(image: &mut MemoryPool) {
    const DATA_UNIT_BYTES: usize = 32768;
    let moved_to = DATA_UNIT + 2;
    assert!(
        UNITS.iter().all(|(start, _)| *start != moved_to),
        "槽 {moved_to} 在写完新池新建文件的镜像上空着"
    );
    let data_unit = read(image, 0, DATA_UNIT * SLOT, DATA_UNIT_BYTES);
    for device in DEVICES {
        write(image, device, moved_to * SLOT, &data_unit);
    }
    let checksum = crc32_castagnoli(&data_unit);
    let mut parents_changed = Vec::new();
    for (container, length) in UNITS.iter().copied() {
        if container == DATA_UNIT {
            continue;
        }
        let before = read(image, 0, container * SLOT, length);
        let mut bytes = before.clone();
        let mut touched = false;
        for device in DEVICES {
            touched |= replace_all(
                &mut bytes,
                &location_pattern(device, DATA_UNIT, checksum),
                &location_pattern(device, moved_to, checksum),
            );
        }
        if touched {
            reseal_unit(&mut bytes);
            for device in DEVICES {
                write(image, device, container * SLOT, &bytes);
            }
            parents_changed.push((
                container,
                crc32_castagnoli(&before),
                crc32_castagnoli(&bytes),
            ));
        }
    }
    assert!(
        !parents_changed.is_empty(),
        "写完新池新建文件的镜像上有单元指着那个数据单元"
    );
    propagate(image, &UNITS, parents_changed);
}

/// 发布 B（txg 4）之后的干净镜像：A 的十二个单元（十四个槽）在这次发布里改写成已释放、释放代 4，树表是 B 写的那一版，
/// A 那一版仍被根环里 A 的根指着。I-3.9（释放代落在停止引用它的那一格区间里）与 I-9.14（树表条目的诞生 txg 跨根不变）
/// 要的跨根比较到这里才有内容：写完新池新建文件的那份镜像上只有 A 的根有树表条目（mkfs 种的第 0 版树表是空的、
/// 两次暖机空发布不写树表），I-9.14 在那份镜像上报「不适用」。
fn image_after_the_overwrite(tag: &str) -> MemoryPool {
    let mut pool = build_pool(tag);
    let previous = pool.output.clone();
    // 第二次写的内容与第一次不同长、不同字节；具体哪些字节这里不承重（落点与释放代由步 1 的验收钉住）。
    let content: Vec<u8> = (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect();
    let second = publish_overwrite_in_process(
        &mut pool,
        &previous,
        &content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写");
    assert_eq!(
        second.root.checkpoint_txg,
        CheckpointTxg(4),
        "发布 B 是 txg 4"
    );
    pool.memory_pool()
}

/// C374（释放代与树表诞生 txg 只有验收断言盯着） 立的那两条各一份坏镜像，都从发布 B 之后的干净镜像出发改一处。
fn known_bad_images_after_the_overwrite() -> Vec<(&'static str, Mutation)> {
    vec![
        // 发布 B 那次的释放代写成上一次发布的 txg（4 → 3）：A 的十二个单元两盘各一条（两片叶各十二条），二十四条一起改。
        (
            "I-3.9",
            Box::new(|image: &mut MemoryPool| {
                assert_eq!(
                    mutate_allocation_leaves(
                        image,
                        &UNITS_AFTER_OVERWRITE,
                        ALLOCATION_LEAVES_AFTER_OVERWRITE,
                        |bytes| rewrite_released_generations(bytes, 4, 3),
                    ),
                    24,
                    "A 的十二个单元、两盘各一条"
                );
            }),
        ),
        // 树表里 extent 树（树 ID 65536）那条的诞生 txg 改成跟着这次发布的 txg（3 → 4）：A 写的那一版树表里仍是 3。
        (
            "I-9.14",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit_of(
                    image,
                    &UNITS_AFTER_OVERWRITE,
                    TREE_TABLE_AFTER_OVERWRITE,
                    |bytes| {
                        let birth_txg =
                            tree_table_entry_offset(0) + TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET;
                        assert_eq!(get_u64(bytes, birth_txg), 3, "extent 树的诞生 txg 本来是 3");
                        set_u64(bytes, birth_txg, 4);
                    },
                    true,
                );
            }),
        ),
        // I-4.9：A 的数据单元（50180，只有 A 的根 txg 3 还引用）诞生代号 3 → 4，引用它的四处一起改成 4（A 的 extent 树根里的数据指针、
        // A 的中央映射里那条码 1 key、A 的分配记录树两片叶里那两条未释放记录的分配代），各自重封、沿引用链补到 A 的根：
        // 指针、映射、分配代与单元头都对得上（I-1.2、I-1.11、I-3.10 不红），按已发布谓词判也是已发布的（4 ≤ 挂载根 txg 4），
        // 只剩「A 那条根引用了一个比它晚诞生的单元」。
        (
            UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
            Box::new(|image: &mut MemoryPool| {
                mutate_unit_of(
                    image,
                    &UNITS_AFTER_OVERWRITE,
                    DATA_UNIT,
                    |bytes| {
                        assert_eq!(
                            get_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET),
                            3,
                            "A 的数据单元诞生代号本来是 3"
                        );
                        set_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET, 4);
                    },
                    true,
                );
                mutate_unit_of(
                    image,
                    &UNITS_AFTER_OVERWRITE,
                    EXTENT_ROOT,
                    |bytes| {
                        assert_eq!(
                            get_u64(bytes, EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET),
                            3,
                            "A 的数据指针出生 txg 本来是 3"
                        );
                        set_u64(bytes, EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET, 4);
                    },
                    true,
                );
                mutate_unit_of(
                    image,
                    &UNITS_AFTER_OVERWRITE,
                    MAPPING_ROOT,
                    |bytes| rewrite_the_data_unit_mapping_key_birth_txg(bytes, 3, 4),
                    true,
                );
                assert_eq!(
                    mutate_allocation_leaves(
                        image,
                        &UNITS_AFTER_OVERWRITE,
                        ALLOCATION_LEAVES,
                        |bytes| {
                            rewrite_unreleased_allocation_generation(bytes, DATA_UNIT, 3, 4)
                        }
                    ),
                    2,
                    "A 的数据单元两盘各一条未释放记录"
                );
            }),
        ),
        // I-8.11：B 那条记录（计数器 4）里码 1 那个点名项（B 的数据单元 50182）的诞生代号 4 → 3，两盘各一份，载荷校验和与头校验和重封。
        // 位置条目没动、50182 的整单元校验和照旧相符 ⇒ 点名项与单元头比得上，而诞生代号不同。B 是环里最后一条，没有后继的反向链要补。
        (
            NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_birth_txg_of_the_data_unit_named_by_journal_record(
                    image,
                    LAST_COUNTER_AFTER_THE_OVERWRITE,
                    DATA_UNIT_AFTER_OVERWRITE,
                    4,
                    3,
                );
            }),
        ),
        // I-8.11 点名项另外四个字段各一份（`research/prompts/m3-batch4-code-r1-sonnet-output.md` 第五节盲区第 4 条），同一个点名项、
        // 同一种重封：类标签 1 → 3（码 3 与码 1 读同样长，位置条目的整单元校验和照旧相符）、出生树改成另一棵发过号的树、
        // key 尾段的写序实例代号 1 → 2、写序事务号加 1。
        (
            NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_named_entry_of_the_data_unit_in_journal_record(
                    image,
                    LAST_COUNTER_AFTER_THE_OVERWRITE,
                    DATA_UNIT_AFTER_OVERWRITE,
                    |entry| {
                        assert_eq!(entry[NAMED_ENTRY_UNIT_CLASS_OFFSET], 1, "点名的是码 1");
                        entry[NAMED_ENTRY_UNIT_CLASS_OFFSET] = 3;
                    },
                );
            }),
        ),
        (
            NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_named_entry_of_the_data_unit_in_journal_record(
                    image,
                    LAST_COUNTER_AFTER_THE_OVERWRITE,
                    DATA_UNIT_AFTER_OVERWRITE,
                    |entry| {
                        assert_eq!(
                            get_u64(entry, NAMED_ENTRY_BIRTH_TREE_OFFSET),
                            TREE_IDENTIFIER_EXTENT,
                            "数据单元的出生树本来是 extent 树"
                        );
                        set_u64(
                            entry,
                            NAMED_ENTRY_BIRTH_TREE_OFFSET,
                            ANOTHER_ISSUED_TREE_BELOW_THE_WATERMARK,
                        );
                    },
                );
            }),
        ),
        (
            NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_named_entry_of_the_data_unit_in_journal_record(
                    image,
                    LAST_COUNTER_AFTER_THE_OVERWRITE,
                    DATA_UNIT_AFTER_OVERWRITE,
                    |entry| {
                        assert_eq!(
                            get_u32(entry, NAMED_ENTRY_KEY_TAIL_INSTANCE_OFFSET),
                            1,
                            "写序实例代号本来是 1"
                        );
                        set_u32(entry, NAMED_ENTRY_KEY_TAIL_INSTANCE_OFFSET, 2);
                    },
                );
            }),
        ),
        (
            NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_named_entry_of_the_data_unit_in_journal_record(
                    image,
                    LAST_COUNTER_AFTER_THE_OVERWRITE,
                    DATA_UNIT_AFTER_OVERWRITE,
                    |entry| {
                        let transaction_byte = NAMED_ENTRY_KEY_TAIL_TRANSACTION_OFFSET;
                        entry[transaction_byte] = entry[transaction_byte].wrapping_add(1);
                    },
                );
            }),
        ),
        // I-8.12：B 那条记录（计数器 4）的 checkpoint_txg 4 → 0，两盘各一份、头校验和重封：同实例前一条（计数器 3）是 txg 3。
        // 实例 1 没有 txg 0 的根（mkfs 种的第 0 代根属实例 0）⇒ I-8.10 没有可比的根；B 是最后一条，没有后继的反向链要补。
        (
            JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE,
            Box::new(|image: &mut MemoryPool| {
                set_checkpoint_txg_of_journal_record(image, LAST_COUNTER_AFTER_THE_OVERWRITE, 4, 0);
            }),
        ),
        // I-7.8 最新那条根那一格（`research/prompts/m3-inv-a-r1-opus-output.md` 三·3 攻方第一份）：只把 B 的根（txg 4，最新）的水位压到
        // mkfs 时的水位，B 那条记录的新根段跟着压（同一份新根段，I-8.10）。A 的根水位照旧盖得住 A 发的十二棵树 ⇒ 取 max 那一格不红。
        (
            "I-7.8",
            Box::new(|image: &mut MemoryPool| {
                let (device, offset) = root_slot_of(CheckpointTxg(4));
                let mut bytes = read(image, device, offset, 512);
                assert_eq!(get_u64(&bytes, 28), 4, "那一槽是 B 的根");
                set_u64(
                    &mut bytes,
                    ROOT_RECORD_TREE_IDENTIFIER_WATERMARK_OFFSET,
                    singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS,
                );
                let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
                bytes[138..170].copy_from_slice(&digest);
                write(image, device, offset, &bytes);
                assert_eq!(
                    copy_each_root_into_the_new_root_segment_of_its_journal_records(image),
                    2,
                    "B 那条记录两盘各一份跟着压"
                );
            }),
        ),
    ]
}

/// 两个各自提交了的事务撞号、而又紧挨着：从**发布 B 之后**的干净镜像出发（实例 1 有 A 的计数器 3、事务号 1，
/// B 的计数器 4、事务号 2），把 B 那一条的事务号改回 1、两盘各一份都改、头校验和重封。号不减、同号相连，
/// I-8.7（实例内事务号不重号） 分不出它与「一事务两条」；分得出的是提交标记——事务号 1 那一组两条都带 ⇒
/// 红的是 I-8.8（前缀里的事务不被切开） 的 ③。
fn two_committed_transactions_sharing_one_number_side_by_side(image: &mut MemoryPool) {
    set_transaction_number_of_journal_record(image, 4, 2, 1);
}

/// 记录头里 fsid 那 8 字节的偏移：事务段 78 + 事务号 8 + 提交标记 1 + 本次发布内序号 4 + 反向链 4 + 载荷校验和 4 + 新根段 364
/// （D23（journal 的角色与格式） 已定项 4 / 已定项 15 的字段表）。
const JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET: usize = 463;

/// 记录头里反向链那 4 字节的偏移：事务段 78 + 事务号 8 + 提交标记 1 + 本次发布内序号 4（D23（journal 的角色与格式） 已定项 4 / 已定项 8）。
const JOURNAL_RECORD_BACK_CHAIN_OFFSET: usize = 91;

/// 记录头里提交标记那 1 字节的偏移：事务段 78 + 事务号 8（D23（journal 的角色与格式） 已定项 7）。
const JOURNAL_RECORD_COMMIT_MARKER_OFFSET: usize = 86;

/// 发布 B 之后那份镜像环里最后一条记录（B 那条，事务号 2）的计数器：I-8.7、I-8.8 的几份镜像都从它后面接着写。
const LAST_COUNTER_AFTER_THE_OVERWRITE: u64 = 4;

/// I-8.8（前缀里的事务不被切开） 报不适用时的理由（照 checker 原样）：一事务一条、每条都带提交标记时 ③ ④ 没有对象。
const COMMIT_MARKERS_NOT_APPLICABLE: &str = "环里没有哪一组（同一块盘上实例代号与事务号都相同的非 0 记录）落了两条以上、也没有哪一组一条提交标记都没有：③ ④ 没有判的对象，提交标记字节单独成立不算这一条成立";

/// 接在 B 那条记录之后的一条记录：它的事务号与提交标记那 1 字节（D23（journal 的角色与格式） 已定项 7 的两个事务边界字段）。
#[derive(Clone, Copy)]
struct AppendedRecord {
    transaction: u64,
    commit_marker_byte: u8,
}

const fn without_commit_marker(transaction: u64) -> AppendedRecord {
    AppendedRecord {
        transaction,
        commit_marker_byte: 0,
    }
}

const fn with_commit_marker(transaction: u64) -> AppendedRecord {
    AppendedRecord {
        transaction,
        commit_marker_byte: 1,
    }
}

/// 不承载事务的空发布记录：事务号 0、提交标记 1（D23（journal 的角色与格式） 已定项 19 ①），不点名任何单元。
const fn empty_publish() -> AppendedRecord {
    AppendedRecord {
        transaction: 0,
        commit_marker_byte: 1,
    }
}

/// 一事务跨两条记录的合法形态（D23（journal 的角色与格式） 已定项 7「一个事务可以跨多条记录」）：事务号 3 先落一条不带提交标记的，
/// 再落一条带的，接在 B 那条记录之后。I-8.7 与 I-8.8 共用的阳性对照；下面前四份坏镜像各自只在这个形态上改一处。
const ONE_TRANSACTION_OVER_TWO_RECORDS: [AppendedRecord; 2] =
    [without_commit_marker(3), with_commit_marker(3)];

/// I-8.7（实例内事务号不重号） 或 I-8.8（前缀里的事务不被切开） 的一份坏镜像：只该红 `violated_invariant` 这一条，
/// 违例说明里红的恰是 `registered_criteria` 这几条判据（次序照 `TRANSACTION_BOUNDARY_CRITERIA`）。
struct TransactionBoundaryBadImage {
    what_is_changed: &'static str,
    violated_invariant: &'static str,
    registered_criteria: &'static [&'static str],
    appended_records: &'static [AppendedRecord],
}

/// 两条不变量违例说明里的判据标签，次序照 checker：I-8.7 的 ① ②，I-8.8 的 ③、提交标记字节、④。
const TRANSACTION_BOUNDARY_CRITERIA: [&str; 5] =
    ["判据①", "判据②", "判据③", "提交标记字节", "判据④"];

/// I-8.7 与 I-8.8 的坏镜像。前两份是 C37（事务与记录的对应无定义） 点名的「交错」，第四份是它点名的「提交标记放错位置」。
const TRANSACTION_BOUNDARY_BAD_IMAGES: [TransactionBoundaryBadImage; 7] = [
    // 阳性对照那两条之间夹进事务 4 的一条。夹进来的是同一实例一个非 0 的号 ⇒ ① 一定跟着红：
    // 号不减又要把同号的两条隔开，夹进来的号只能等于它自己。
    TransactionBoundaryBadImage {
        what_is_changed: "交错：夹进另一个事务的记录",
        violated_invariant: "I-8.7",
        registered_criteria: &["判据①", "判据②"],
        appended_records: &[
            without_commit_marker(3),
            with_commit_marker(4),
            with_commit_marker(3),
        ],
    },
    // 阳性对照那两条之间夹进一次空发布（事务号 0）：号没有减，红的只有 ②。
    TransactionBoundaryBadImage {
        what_is_changed: "交错：夹进一条空发布",
        violated_invariant: "I-8.7",
        registered_criteria: &["判据②"],
        appended_records: &[
            without_commit_marker(3),
            empty_publish(),
            with_commit_marker(3),
        ],
    },
    // 两个各一条的事务，号先 4 后 3：没有同号的两条，红的只有 ①。
    TransactionBoundaryBadImage {
        what_is_changed: "事务号倒退",
        violated_invariant: "I-8.7",
        registered_criteria: &["判据①"],
        appended_records: &[with_commit_marker(4), with_commit_marker(3)],
    },
    // 阳性对照那两条的提交标记挪到前一条上：带提交标记的那条不是这一组计数器最大的。
    TransactionBoundaryBadImage {
        what_is_changed: "提交标记放错位置",
        violated_invariant: "I-8.8",
        registered_criteria: &["判据③"],
        appended_records: &[with_commit_marker(3), without_commit_marker(3)],
    },
    // 事务 3 一条提交标记都没落，同一实例却接着写了事务 4：失败之后这个实例还在写。
    TransactionBoundaryBadImage {
        what_is_changed: "没提交的事务后面还有号更大的记录",
        violated_invariant: "I-8.8",
        registered_criteria: &["判据④"],
        appended_records: &[without_commit_marker(3), with_commit_marker(4)],
    },
    // 事务 3 一条提交标记都没落，同一实例接着写了一条空发布：空发布也算「失败之后还在写」。
    // 号没有变大（事务号 0 不进 I-8.7 的序列），只看「号更大」的写法在这一份上判不出来。
    TransactionBoundaryBadImage {
        what_is_changed: "没提交的事务后面跟一条空发布",
        violated_invariant: "I-8.8",
        registered_criteria: &["判据④"],
        appended_records: &[without_commit_marker(3), empty_publish()],
    },
    // 事务 3 那一条的提交标记字节写成 2（头校验和照样重封）：不许静默读成「不带」。
    TransactionBoundaryBadImage {
        what_is_changed: "提交标记字节写成 2",
        violated_invariant: "I-8.8",
        registered_criteria: &["提交标记字节"],
        appended_records: &[AppendedRecord {
            transaction: 3,
            commit_marker_byte: 2,
        }],
    },
];

/// 在发布 B 那条记录之后接着写 `appended` 这几条记录，两盘各一份：每条照 B 那条记录改计数器（从 5 起）、代号（从 5 起，
/// 没有哪条不变量读记录头的代号）、事务号与提交标记（事务号 0 的空发布另把点名项清空），反向链接在前一条的头上，
/// 载荷校验和与头校验和由 `JournalRecord::to_bytes` 重封（它只写得出提交标记 0 或 1，别的值改完那一字节再重封头校验和）——
/// 记录自证与 I-8.6（反向链算法） 照旧成立，这几份镜像只在事务号与提交标记的排法上不同。
fn append_journal_records_after_the_overwrite(pool: &mut MemoryPool, appended: &[AppendedRecord]) {
    for device in DEVICES {
        let mut previous_bytes = read(
            pool,
            device,
            singlefs_core::journal::record_offset(
                LAST_COUNTER_AFTER_THE_OVERWRITE,
                singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
            )
            .0,
            4096,
        );
        let template = singlefs_core::journal::JournalRecord::parse(
            &previous_bytes,
            get_u64(&previous_bytes, JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET),
        )
        .expect("B 那条记录自证过");
        assert_eq!(
            (template.counter, template.transaction),
            (LAST_COUNTER_AFTER_THE_OVERWRITE, 2),
            "盘 {device} 环里最后一条是 B 那条"
        );
        for (position, appended_record) in (1u64..).zip(appended) {
            let mut record = template.clone();
            record.counter = LAST_COUNTER_AFTER_THE_OVERWRITE + position;
            record.checkpoint_txg = CheckpointTxg(template.checkpoint_txg.0 + position);
            record.transaction = appended_record.transaction;
            record.is_commit = appended_record.commit_marker_byte == 1;
            record.back_chain = singlefs_core::journal::back_chain_of(&previous_bytes);
            if appended_record.transaction == 0 {
                record.named = Vec::new();
            }
            let mut bytes = record.to_bytes();
            if bytes[JOURNAL_RECORD_COMMIT_MARKER_OFFSET] != appended_record.commit_marker_byte {
                bytes[JOURNAL_RECORD_COMMIT_MARKER_OFFSET] = appended_record.commit_marker_byte;
                let digest = wide_checksum_with_field_zeroed(&bytes, 4096, 46);
                bytes[46..78].copy_from_slice(&digest);
            }
            write(
                pool,
                device,
                singlefs_core::journal::record_offset(
                    record.counter,
                    singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
                )
                .0,
                &bytes,
            );
            previous_bytes = bytes;
        }
    }
}

/// I-8.7 与 I-8.8 的坏镜像，交给「每条不变量至少有一份坏镜像」那一处数清单；判定逐份在
/// `each_transaction_boundary_bad_image_reddens_only_its_own_invariant_on_its_registered_criteria` 里。
fn known_bad_images_of_the_transaction_boundaries() -> Vec<(&'static str, Mutation)> {
    TRANSACTION_BOUNDARY_BAD_IMAGES
        .into_iter()
        .map(|bad_image| -> (&'static str, Mutation) {
            (
                bad_image.violated_invariant,
                Box::new(move |image: &mut MemoryPool| {
                    append_journal_records_after_the_overwrite(image, bad_image.appended_records);
                }),
            )
        })
        .collect()
}

/// 一事务一条（第一版可达的形态）、别处都干净的镜像上每一条该有的判定：I-8.8（前缀里的事务不被切开） 的 ③ ④ 没有对象、
/// 报不适用，别的每一条都要真被评估过且成立。I-8.8 的阳性对照与坏镜像在
/// `one_transaction_over_two_records_with_the_commit_marker_on_the_last_holds_both_transaction_boundary_invariants` 与后一条用例里。
/// 用它的镜像都没抬过 F（发布 B 之后、回退到 A 之后），I-7.9 报不适用；抬过 F 的镜像在
/// `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant` 里。
fn expected_verdict_with_one_record_per_transaction(invariant: &str) -> InvariantVerdict {
    match invariant {
        "I-8.8" => InvariantVerdict::NotApplicable(COMMIT_MARKERS_NOT_APPLICABLE),
        "I-7.9" => InvariantVerdict::NotApplicable(NO_ROOT_RAISED_THE_FLOOR),
        // 最新根之上没有记录：重放之后那一版就是最新根。
        "I-4.1" => InvariantVerdict::NotApplicable(NOTHING_APPLIED_ABOVE_THE_CHOSEN_ROOT),
        _ => InvariantVerdict::Holds,
    }
}

/// 没抬过 F 的镜像上 I-7.9（回退下界 F 不高于抬 F 的上限） 报的不适用理由。
const NO_ROOT_RAISED_THE_FLOOR: &str = "根环里没有抬 F 的根：每条根带的 F 都不高于同一实例里它前一条根带的（回退与新实例的第一条根不算抬）";

/// 一份改过的镜像上 I-8.7（实例内事务号不重号） 与 I-8.8（前缀里的事务不被切开） 那一片判定，拿来与登记的整张表比。
#[derive(Debug, PartialEq, Eq)]
struct TransactionBoundaryJudgement {
    what_is_changed: &'static str,
    violated_invariants: Vec<&'static str>,
    /// 两条的违例说明里红了哪几条判据，次序照 `TRANSACTION_BOUNDARY_CRITERIA`。
    reddened_criteria: Vec<&'static str>,
    /// 除 I-8.7 与 I-8.8 之外，判定与干净那一份不同的不变量。
    other_verdicts_that_changed: Vec<&'static str>,
}

fn transaction_boundary_judgement(
    what_is_changed: &'static str,
    verdicts: &[(&'static str, InvariantVerdict)],
    verdicts_on_the_clean_image: &[(&'static str, InvariantVerdict)],
) -> TransactionBoundaryJudgement {
    let violation_details: Vec<&str> = verdicts
        .iter()
        .filter(|(name, _)| ["I-8.7", "I-8.8"].contains(name))
        .filter_map(|(_, found)| match found {
            InvariantVerdict::Violated(detail) => Some(detail.as_str()),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect();
    TransactionBoundaryJudgement {
        what_is_changed,
        violated_invariants: violated_invariants(verdicts),
        reddened_criteria: TRANSACTION_BOUNDARY_CRITERIA
            .into_iter()
            .filter(|criterion| {
                violation_details
                    .iter()
                    .any(|detail| detail.contains(criterion))
            })
            .collect(),
        other_verdicts_that_changed: verdicts_that_differ_outside(
            verdicts,
            verdicts_on_the_clean_image,
            &[
                "I-8.7",
                "I-8.8",
                // 接在 B 后面的记录在最新根之上：下一次挂载会重放它们，I-4.1（重放后树可遍历） 从「一次都不施加、不适用」变成真被评估过——
                // 判的是重放之后那一版，不是事务边界；它要是红了，上面 `violated_invariants` 那一格照样看得见。
                TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
            ],
        )
        .into_iter()
        .map(|(name, _, _)| name)
        .collect(),
    }
}

/// 一份镜像上判红的不变量，次序照 checker 的清单。
fn violated_invariants(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<&'static str> {
    verdicts
        .iter()
        .filter(|(_, found)| matches!(found, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect()
}

/// `changed` 之外每一条不变量在两份镜像上的判定：逐项不同的列出来（空 = 逐项相同）。
fn verdicts_that_differ_outside<'verdicts>(
    verdicts: &'verdicts [(&'static str, InvariantVerdict)],
    verdicts_on_the_clean_image: &'verdicts [(&'static str, InvariantVerdict)],
    changed: &[&str],
) -> Vec<(
    &'static str,
    &'verdicts InvariantVerdict,
    &'verdicts InvariantVerdict,
)> {
    verdicts
        .iter()
        .zip(verdicts_on_the_clean_image)
        .filter(|((name, found), (_, before))| !changed.contains(name) && found != before)
        .map(|((name, found), (_, before))| (*name, before, found))
        .collect()
}

/// 新池新建文件那条根（txg 3）的根槽：区域 `3 mod 3` = 0（盘 0）的槽 `(3 div 3) mod 8` = 1（D22（单元原子性怎么合成）
/// 已定项 2 / 已定项 16；区域 0 起点 64 × 16384，槽距 4096）。
const NEW_POOL_FILE_CREATION_ROOT_SLOT: (u32, u64) = (0, 64 * SLOT + 4096);

/// I-7.3（环健康性） 的坏镜像，从 **mkfs 刚写完**那份镜像出发：三个区域槽 0 上那三条第 0 代根的 `checkpoint_txg`
/// 一起改成 1、各自重封自证校验和——「第一次发布把三个区域全盖了一遍」（轮换键算错，每次发布盖满整个环）在盘上的样子。
/// S 里代号全是 1 ⇒ 除代号最大者外一条更早代号的记录都没有，而 S 又不全是第 0 代 ⇒ 例外那一支够不着 ⇒ 必须红。
/// **为什么不建在写完新池新建文件那份镜像上**：那一份上要让 S 只剩代号最大的一条，就得把 mkfs 那条第 0 代根也去掉，
/// 而 mkfs 种的树表单元（50178）只有它还引用着 ⇒ I-3.1（已分配统计对得上） 跟着红（遍历少算 16384），判别力算不到 I-7.3 头上。
/// 那一份的形态另由 `the_root_ring_health_invariant_reddens_...` 里第二段钉住，连它带的那一处一起钉。
fn known_bad_images_of_the_root_ring_health() -> Vec<(&'static str, Mutation)> {
    vec![(
        "I-7.3",
        Box::new(|image: &mut MemoryPool| {
            let mut bumped_root_slots = 0;
            for (device, offset) in root_slots() {
                let mut bytes = read(image, device, offset, 512);
                if &bytes[..4] != b"SFSR" {
                    continue;
                }
                assert_eq!(get_u64(&bytes, 28), 0, "mkfs 种下的是第 0 代");
                set_u64(&mut bytes, 28, 1);
                let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
                bytes[138..170].copy_from_slice(&digest);
                write(image, device, offset, &bytes);
                bumped_root_slots += 1;
            }
            assert_eq!(
                bumped_root_slots, 3,
                "mkfs 在三个区域的槽 0 各种一条第 0 代根"
            );
        }),
    )]
}

/// I-5.4（分配记录罩住的槽互不相交） 的坏镜像，都从发布 B 之后的干净镜像出发、把 A 的 inode 树根那个落点（槽 50244、跨 1）的记录
/// 跨度改成 2（已释放标志照留），两盘各一条（两片叶各一条）——它罩住的 50245 上另有一条记录（A 的分配记录树盘 0 那片叶的落点）。
/// 50244 与 50245 都在叶 61 罩的 [49532, 50344) 里，改完的记录末槽 50245 仍在它那片叶里，按位置寻址那一道（I-1.1） 不跟着红。
/// 这是代码三方第二轮 Z1-d 在盘上的形状（复用时跨度变了、新记录罩住的旧记录留着）；树里的引用一个没动，I-5.1（物理范围不重叠） 看不见它。
fn known_bad_images_of_overlapping_allocation_records() -> Vec<(&'static str, Mutation)> {
    vec![
        // 改在 B 写的那棵账里（最新根下面）：两条都是已释放、释放代 4 的记录。
        (
            "I-5.4",
            Box::new(|image: &mut MemoryPool| {
                assert_eq!(
                    mutate_allocation_leaves(
                        image,
                        &UNITS_AFTER_OVERWRITE,
                        ALLOCATION_LEAVES_AFTER_OVERWRITE,
                        |bytes| set_allocation_record_span(bytes, INODE_ROOT, 2),
                    ),
                    2,
                    "A 的 inode 树根那一个落点，两盘各一条"
                );
            }),
        ),
        // 改在 A 写的那棵账里（只有 A 的根指着它，B 之后 A 的根仍在候选集里）：两条都是仍分配、分配代 3 的记录。
        // 「候选集里每条有效根」那个量词靠它：只判最新根那棵账的 checker 在这份镜像上判成立。
        (
            "I-5.4",
            Box::new(|image: &mut MemoryPool| {
                assert_eq!(
                    mutate_allocation_leaves(
                        image,
                        &UNITS_AFTER_OVERWRITE,
                        ALLOCATION_LEAVES,
                        |bytes| set_allocation_record_span(bytes, INODE_ROOT, 2),
                    ),
                    2,
                    "A 的 inode 树根那一个落点，两盘各一条"
                );
            }),
        ),
    ]
}

/// I-5.4 那份坏镜像**只**红在 I-5.4 上：别的不变量跟着红，就说明镜像改宽了，判别力算不到这一条头上。
#[test]
fn an_allocation_record_whose_span_covers_the_next_record_reddens_only_the_disjointness_invariant()
{
    let clean = image_after_the_overwrite("known-bad-overlapping-records");
    assert_eq!(
        verdict(&clean, "I-5.4"),
        InvariantVerdict::Holds,
        "发布 B 之后的干净镜像上 I-5.4 要真被评估过且成立"
    );
    let images_and_where_they_are_changed =
        ["B 写的那棵账（最新根）", "A 写的那棵账（更早的候选根）"];
    let bad_images = known_bad_images_of_overlapping_allocation_records();
    assert_eq!(bad_images.len(), images_and_where_they_are_changed.len());
    for ((invariant, mutation), where_changed) in bad_images
        .into_iter()
        .zip(images_and_where_they_are_changed)
    {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        let violated: Vec<&str> = verdicts
            .iter()
            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(
            violated,
            [invariant],
            "改在{where_changed}：判红的该只有 {invariant}：{verdicts:?}"
        );
    }
}

/// 同一实例一个事务号用了两次：I-8.7（实例内事务号不重号） 与 I-8.8（前缀里的事务不被切开） 各看到哪一面，
/// 都建在发布 B 之后那份镜像上：
/// ① 干净那一份：(计数器, 实例, 事务号) 整张表钉下来——写行与两次暖机空发布写 0，A 是 1、B 是 2——I-8.7 真被评估过且成立；
/// ② **阳性对照**：B 之后接一次空发布（事务号 0）再接事务 3。**事务号 0 夹在同一实例两个不同的非 0 号中间是合法的**
///    （抬回退下界那两次空发布就是这个形态）：I-8.7 成立、一条红都没有。把 0 也算进序列的写法在这一份上就红（2 之后是 0），
///    这一格是「事务号 0 不进序列」那条射程的阳性对照；
/// ③ B 那条的事务号改回 1（与 A 紧挨着、各带提交标记）：号不减、同号相连，I-8.7 分不出它与「一事务两条」，
///    分得出的是提交标记——事务号 1 那一组两条都带 ⇒ 只红 I-8.8 的 ③；
/// ④ 2026-09-21 修掉的那个 bug 在盘上的样子：B 之后两次抬下界的空发布，下一次发布取「上一条记录的事务号 + 1」= 1
///    ⇒ I-8.7 红在 ①（2 之后是 1）与 ②（1 被隔开之后又出现），I-8.8 红在 ③（事务号 1 那一组两条都带提交标记）。
/// ③ ④ 两份的判定整张表比一次；每一份都要求除 I-8.7 与 I-8.8 之外每一条的判定与 ① 那一份逐项相同。
#[test]
fn repeated_transaction_number_in_one_instance_reddens_the_uncut_transaction_invariant_and_only_when_separated_the_transaction_number_invariant(
) {
    let clean = image_after_the_overwrite("known-bad-transaction-number");
    for device in DEVICES {
        assert_eq!(
            journal_records_on_device(&clean, device),
            vec![(1, 1, 0), (2, 1, 0), (3, 1, 1), (4, 1, 2)],
            "盘 {device} 环里的 (计数器, 实例, 事务号)：写行与两次暖机空发布写 0，A 是 1、B 是 2"
        );
    }
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    assert_eq!(
        verdict(&clean, "I-8.7"),
        InvariantVerdict::Holds,
        "环里有事务号 0 的空发布记录，I-8.7 要真被评估过且成立"
    );
    let mut empty_publish_between_two_numbers = clean.clone();
    append_journal_records_after_the_overwrite(
        &mut empty_publish_between_two_numbers,
        &[empty_publish(), with_commit_marker(3)],
    );
    assert_eq!(
        verdict(&empty_publish_between_two_numbers, "I-8.7"),
        InvariantVerdict::Holds,
        "事务号 0 夹在 2 与 3 中间：I-8.7 要真被评估过且成立"
    );
    let verdicts = check_pool_image(&empty_publish_between_two_numbers);
    assert_eq!(
        transaction_boundary_judgement(
            "空发布夹在 2 与 3 中间",
            &verdicts,
            &verdicts_on_the_clean_image
        ),
        TransactionBoundaryJudgement {
            what_is_changed: "空发布夹在 2 与 3 中间",
            violated_invariants: Vec::new(),
            reddened_criteria: Vec::new(),
            other_verdicts_that_changed: Vec::new(),
        },
        "一条红都不该有：{verdicts:?}"
    );
    let mut side_by_side = clean.clone();
    two_committed_transactions_sharing_one_number_side_by_side(&mut side_by_side);
    let mut after_raising_the_floor = clean.clone();
    append_journal_records_after_the_overwrite(
        &mut after_raising_the_floor,
        &[empty_publish(), empty_publish(), with_commit_marker(1)],
    );
    let judged: Vec<TransactionBoundaryJudgement> = [
        ("撞号而紧挨着", &side_by_side),
        ("撞号而中间隔着两次空发布", &after_raising_the_floor),
    ]
    .into_iter()
    .map(|(what_is_changed, image)| {
        transaction_boundary_judgement(
            what_is_changed,
            &check_pool_image(image),
            &verdicts_on_the_clean_image,
        )
    })
    .collect();
    assert_eq!(
        judged,
        vec![
            TransactionBoundaryJudgement {
                what_is_changed: "撞号而紧挨着",
                violated_invariants: vec!["I-8.8"],
                reddened_criteria: vec!["判据③"],
                other_verdicts_that_changed: Vec::new(),
            },
            TransactionBoundaryJudgement {
                what_is_changed: "撞号而中间隔着两次空发布",
                violated_invariants: vec!["I-8.7", "I-8.8"],
                reddened_criteria: vec!["判据①", "判据②", "判据③"],
                other_verdicts_that_changed: Vec::new(),
            },
        ],
        "两份撞号镜像各红在哪；违例说明：紧挨着那份 I-8.8 = {:?}，隔开那份 I-8.7 = {:?}、I-8.8 = {:?}",
        verdict(&side_by_side, "I-8.8"),
        verdict(&after_raising_the_floor, "I-8.7"),
        verdict(&after_raising_the_floor, "I-8.8")
    );
}

/// I-8.7（实例内事务号不重号） 与 I-8.8（前缀里的事务不被切开） 共用的阳性对照，建在发布 B 之后那份镜像上：
/// ① 干净那一份一事务一条、每条都带提交标记 ⇒ I-8.8 的 ③ ④ 没有对象，报「不适用」，不报成立；
/// ② B 之后接着写事务 3 的两条记录，提交标记在计数器大的那一条（D23（journal 的角色与格式） 已定项 7 允许的形态），
///    两盘各一份、反向链接好 ⇒ 两条都要真被评估过且成立、一条红都没有，除 I-8.7 与 I-8.8 之外每一条的判定与 ① 那一份逐项相同。
///    见到两条同号就红的实现（I-8.7 还按严格递增判）、和从来报不出成立的 I-8.8 在这一格上红；
///    从来不红的实现由后一条用例的坏镜像挡。
#[test]
fn one_transaction_over_two_records_with_the_commit_marker_on_the_last_holds_both_transaction_boundary_invariants(
) {
    let clean = image_after_the_overwrite("known-bad-transaction-boundaries-positive");
    for device in DEVICES {
        assert_eq!(
            journal_records_on_device(&clean, device),
            vec![(1, 1, 0), (2, 1, 0), (3, 1, 1), (4, 1, 2)],
            "盘 {device} 环里的 (计数器, 实例, 事务号)：写行与两次暖机空发布写 0，A 是 1、B 是 2"
        );
    }
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    assert_eq!(
        verdict(&clean, "I-8.8"),
        InvariantVerdict::NotApplicable(COMMIT_MARKERS_NOT_APPLICABLE),
        "一事务一条：I-8.8 的 ③ ④ 没有对象，报不适用"
    );
    let mut image = clean.clone();
    append_journal_records_after_the_overwrite(&mut image, &ONE_TRANSACTION_OVER_TWO_RECORDS);
    for device in DEVICES {
        assert_eq!(
            journal_records_on_device(&image, device),
            vec![
                (1, 1, 0),
                (2, 1, 0),
                (3, 1, 1),
                (4, 1, 2),
                (5, 1, 3),
                (6, 1, 3)
            ],
            "盘 {device}：B 之后接着落事务 3 的两条"
        );
    }
    let verdicts = check_pool_image(&image);
    assert_eq!(
        (verdict(&image, "I-8.7"), verdict(&image, "I-8.8")),
        (InvariantVerdict::Holds, InvariantVerdict::Holds),
        "事务 3 跨两条、提交标记在计数器大的那条：I-8.7 与 I-8.8 都要真被评估过且成立"
    );
    assert_eq!(
        transaction_boundary_judgement("一事务两条", &verdicts, &verdicts_on_the_clean_image),
        TransactionBoundaryJudgement {
            what_is_changed: "一事务两条",
            violated_invariants: Vec::new(),
            reddened_criteria: Vec::new(),
            other_verdicts_that_changed: Vec::new(),
        },
        "一条红都不该有，别的判定一条不变：{verdicts:?}"
    );
}

/// I-8.7（实例内事务号不重号） 与 I-8.8（前缀里的事务不被切开） 的七份坏镜像（`TRANSACTION_BOUNDARY_BAD_IMAGES`：
/// 前四份各自只在阳性对照那个形态上改一处）：每份**只红登记的那一条不变量**，违例说明里红的判据恰是登记的那几条
/// （判别力要算到那一条判据头上，不只算到这条不变量头上），除 I-8.7 与 I-8.8 之外每一条的判定与干净那一份逐项相同。
/// 七份的判定先全部收齐、再整张表比一次，一处改坏时红的消息里看得到每一份各红在哪。
#[test]
fn each_transaction_boundary_bad_image_reddens_only_its_own_invariant_on_its_registered_criteria() {
    let clean = image_after_the_overwrite("known-bad-transaction-boundaries");
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    assert_eq!(
        verdict(&clean, "I-8.8"),
        InvariantVerdict::NotApplicable(COMMIT_MARKERS_NOT_APPLICABLE),
        "一事务一条：I-8.8 的 ③ ④ 没有对象，报不适用"
    );
    let mut judged_by_image: Vec<TransactionBoundaryJudgement> = Vec::new();
    let mut registered_by_image: Vec<TransactionBoundaryJudgement> = Vec::new();
    let mut details_by_image: Vec<(&str, InvariantVerdict, InvariantVerdict)> = Vec::new();
    for TransactionBoundaryBadImage {
        what_is_changed,
        violated_invariant,
        registered_criteria,
        appended_records,
    } in TRANSACTION_BOUNDARY_BAD_IMAGES
    {
        let mut image = clean.clone();
        append_journal_records_after_the_overwrite(&mut image, appended_records);
        let verdicts = check_pool_image(&image);
        judged_by_image.push(transaction_boundary_judgement(
            what_is_changed,
            &verdicts,
            &verdicts_on_the_clean_image,
        ));
        registered_by_image.push(TransactionBoundaryJudgement {
            what_is_changed,
            violated_invariants: vec![violated_invariant],
            reddened_criteria: registered_criteria.to_vec(),
            other_verdicts_that_changed: Vec::new(),
        });
        details_by_image.push((
            what_is_changed,
            verdict(&image, "I-8.7"),
            verdict(&image, "I-8.8"),
        ));
    }
    assert_eq!(
        judged_by_image, registered_by_image,
        "每份坏镜像只红登记的那一条、红在登记的那几条判据上、别的判定一条不变；(镜像, I-8.7, I-8.8) 逐份是 {details_by_image:?}"
    );
}

/// 记录头里记录标志那 1 字节的偏移：magic 4 + 类型 2 + 算法类型 1（D23（journal 的角色与格式） 已定项 4，原「填充 1」那个字节）。
const JOURNAL_RECORD_FLAGS_OFFSET: usize = 7;

/// 接在 B 那条记录之后的一条记录，I-8.9（一次发布的记录序号连续且只有末条带标志） 那几份镜像用：它属于哪次发布
/// （checkpoint_txg 比 B 大几）、事务号与提交标记、本次发布内序号、记录标志那 1 字节（D23（journal 的角色与格式） 已定项 4 / 已定项 17）。
#[derive(Clone, Copy)]
struct AppendedPublishRecord {
    txg_after_the_overwrite: u64,
    transaction: u64,
    is_commit: bool,
    ordinal_within_publish: u32,
    record_flags_byte: u8,
}

const fn publish_record(
    txg_after_the_overwrite: u64,
    transaction: u64,
    is_commit: bool,
    ordinal_within_publish: u32,
    record_flags_byte: u8,
) -> AppendedPublishRecord {
    AppendedPublishRecord {
        txg_after_the_overwrite,
        transaction,
        is_commit,
        ordinal_within_publish,
        record_flags_byte,
    }
}

/// 一次发布两条记录的合法形态（D23（journal 的角色与格式） 已定项 17：最后一个事务装不下一条记录时末条再跨记录）：
/// 发布 txg = B + 1，事务 3 跨两条，序号 1、2，提交标记与末条标志都只在第二条。I-8.9 的阳性对照；
/// 下面几份坏镜像各自只在这个形态上改一处。
const ONE_PUBLISH_OVER_TWO_RECORDS: [AppendedPublishRecord; 2] = [
    publish_record(1, 3, false, 1, 0),
    publish_record(1, 3, true, 2, 1),
];

/// I-8.9 违例说明里的判据标签，次序照 checker（`judge_publish_ordinals_and_last_record_flags`）。
const PUBLISH_ORDINAL_CRITERIA: [&str; 7] = [
    "标志其余位",
    "序号 0",
    "跳号",
    "不从 1 起",
    "多于一条末条",
    "末条之后还有记录",
    "没有末条",
];

/// I-8.9 的一份坏镜像：只该红 I-8.9，违例说明里红的恰是 `registered_criteria` 这几条判据（次序照 `PUBLISH_ORDINAL_CRITERIA`）。
struct PublishOrdinalBadImage {
    what_is_changed: &'static str,
    registered_criteria: &'static [&'static str],
    appended_records: &'static [AppendedPublishRecord],
}

/// I-8.9 的坏镜像，每份都从阳性对照那个形态改一处。前五份是派发点名的那五种（跳号、序号 0、没有末条、两条末条、
/// 标志其余位非 0），后两份是 checker 另两条判据（不从 1 起、末条之后同一次发布还有记录）各自的判别力。
const PUBLISH_ORDINAL_BAD_IMAGES: [PublishOrdinalBadImage; 7] = [
    // 第二条的序号 2 → 3：jsn 连号而序号跳了一格。
    PublishOrdinalBadImage {
        what_is_changed: "一次发布之内跳号",
        registered_criteria: &["跳号"],
        appended_records: &[
            publish_record(1, 3, false, 1, 0),
            publish_record(1, 3, true, 3, 1),
        ],
    },
    // 第一条的序号 1 → 0。它的前一个计数器上坐着 B 那条（别的发布）⇒ 它是这次发布的第一条，「不从 1 起」跟着红；
    // 序号 0、1 与计数器同步，不算跳号。
    PublishOrdinalBadImage {
        what_is_changed: "序号 0",
        registered_criteria: &["序号 0", "不从 1 起"],
        appended_records: &[
            publish_record(1, 3, false, 0, 0),
            publish_record(1, 3, true, 1, 1),
        ],
    },
    // 第二条的末条标志摘掉，同一实例接着写了下一次发布（txg = B + 2）：这次发布写完了，却一条末条都没有。
    PublishOrdinalBadImage {
        what_is_changed: "没有末条",
        registered_criteria: &["没有末条"],
        appended_records: &[
            publish_record(1, 3, false, 1, 0),
            publish_record(1, 3, true, 2, 0),
            publish_record(2, 4, true, 1, 1),
        ],
    },
    // 第一条也带末条标志。
    PublishOrdinalBadImage {
        what_is_changed: "两条末条",
        registered_criteria: &["多于一条末条"],
        appended_records: &[
            publish_record(1, 3, false, 1, 1),
            publish_record(1, 3, true, 2, 1),
        ],
    },
    // 末条的记录标志写成 0b11：位 0 对，位 1 不许为 1。
    PublishOrdinalBadImage {
        what_is_changed: "记录标志其余位非 0",
        registered_criteria: &["标志其余位"],
        appended_records: &[
            publish_record(1, 3, false, 1, 0),
            publish_record(1, 3, true, 2, 0b0000_0011),
        ],
    },
    // 两条的序号 2、3：与 jsn 同步、不跳号，可这次发布的第一条不是 1。
    PublishOrdinalBadImage {
        what_is_changed: "不从 1 起",
        registered_criteria: &["不从 1 起"],
        appended_records: &[
            publish_record(1, 3, false, 2, 0),
            publish_record(1, 3, true, 3, 1),
        ],
    },
    // 末条标志挪到第一条：带标志的那条后面同一次发布还有一条。
    PublishOrdinalBadImage {
        what_is_changed: "末条标志不在最后一条",
        registered_criteria: &["末条之后还有记录"],
        appended_records: &[
            publish_record(1, 3, false, 1, 1),
            publish_record(1, 3, true, 2, 0),
        ],
    },
];

/// 在发布 B 那条记录之后接着写 `appended` 这几条记录，两盘各一份：照 B 那条记录改计数器（从 5 起）、checkpoint_txg、
/// 事务号与提交标记、本次发布内序号与记录标志，反向链接在前一条的头上；载荷校验和与头校验和由 `JournalRecord::to_bytes` 重封，
/// 它写不出来的记录标志（其余位非 0）改完那一字节再重封头校验和——记录自证与 I-8.6（反向链算法） 照旧成立。
fn append_publish_records_after_the_overwrite(
    pool: &mut MemoryPool,
    appended: &[AppendedPublishRecord],
) {
    for device in DEVICES {
        let mut previous_bytes = read(
            pool,
            device,
            singlefs_core::journal::record_offset(
                LAST_COUNTER_AFTER_THE_OVERWRITE,
                singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
            )
            .0,
            4096,
        );
        let template = singlefs_core::journal::JournalRecord::parse(
            &previous_bytes,
            get_u64(&previous_bytes, JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET),
        )
        .expect("B 那条记录自证过");
        assert_eq!(
            (template.counter, template.transaction),
            (LAST_COUNTER_AFTER_THE_OVERWRITE, 2),
            "盘 {device} 环里最后一条是 B 那条"
        );
        for (position, appended_record) in (1u64..).zip(appended) {
            let mut record = template.clone();
            record.counter = LAST_COUNTER_AFTER_THE_OVERWRITE + position;
            record.checkpoint_txg =
                CheckpointTxg(template.checkpoint_txg.0 + appended_record.txg_after_the_overwrite);
            record.transaction = appended_record.transaction;
            record.is_commit = appended_record.is_commit;
            record.ordinal_within_publish =
                singlefs_core::journal::JournalRecordOrdinalWithinPublish(
                    appended_record.ordinal_within_publish,
                );
            record.place_in_publish = if appended_record.record_flags_byte & 1 == 1 {
                singlefs_core::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
            } else {
                singlefs_core::journal::JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
            };
            record.back_chain = singlefs_core::journal::back_chain_of(&previous_bytes);
            let mut bytes = record.to_bytes();
            if bytes[JOURNAL_RECORD_FLAGS_OFFSET] != appended_record.record_flags_byte {
                bytes[JOURNAL_RECORD_FLAGS_OFFSET] = appended_record.record_flags_byte;
                let digest = wide_checksum_with_field_zeroed(&bytes, 4096, 46);
                bytes[46..78].copy_from_slice(&digest);
            }
            write(
                pool,
                device,
                singlefs_core::journal::record_offset(
                    record.counter,
                    singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
                )
                .0,
                &bytes,
            );
            previous_bytes = bytes;
        }
    }
}

/// I-8.9 的坏镜像，交给「每条不变量至少有一份坏镜像」那一处数清单；判定逐份在
/// `each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant_on_its_registered_criteria` 里。
fn known_bad_images_of_the_publish_ordinals() -> Vec<(&'static str, Mutation)> {
    PUBLISH_ORDINAL_BAD_IMAGES
        .into_iter()
        .map(|bad_image| -> (&'static str, Mutation) {
            (
                "I-8.9",
                Box::new(move |image: &mut MemoryPool| {
                    append_publish_records_after_the_overwrite(image, bad_image.appended_records);
                }),
            )
        })
        .collect()
}

/// 一份改过的镜像上 I-8.9 那一片判定，拿来与登记的整张表比。
#[derive(Debug, PartialEq, Eq)]
struct PublishOrdinalJudgement {
    what_is_changed: &'static str,
    violated_invariants: Vec<&'static str>,
    /// I-8.9 的违例说明里红了哪几条判据，次序照 `PUBLISH_ORDINAL_CRITERIA`。
    reddened_criteria: Vec<&'static str>,
    /// 除 I-8.9 之外，判定与阳性对照那一份不同的不变量。
    other_verdicts_that_changed: Vec<&'static str>,
}

fn publish_ordinal_judgement(
    what_is_changed: &'static str,
    verdicts: &[(&'static str, InvariantVerdict)],
    verdicts_on_the_positive_control: &[(&'static str, InvariantVerdict)],
) -> PublishOrdinalJudgement {
    let violation_detail = verdicts
        .iter()
        .find(|(name, _)| *name == "I-8.9")
        .and_then(|(_, found)| match found {
            InvariantVerdict::Violated(detail) => Some(detail.as_str()),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .unwrap_or("");
    PublishOrdinalJudgement {
        what_is_changed,
        violated_invariants: violated_invariants(verdicts),
        reddened_criteria: PUBLISH_ORDINAL_CRITERIA
            .into_iter()
            .filter(|criterion| violation_detail.contains(&format!("{criterion}：")))
            .collect(),
        other_verdicts_that_changed: verdicts_that_differ_outside(
            verdicts,
            verdicts_on_the_positive_control,
            // I-4.1 不比的理由同 `transaction_boundary_judgement`：接在 B 后面的记录在最新根之上，重放到哪一条随序号与末条标志变。
            &["I-8.9", TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE],
        )
        .into_iter()
        .map(|(name, _, _)| name)
        .collect(),
    }
}

/// I-8.9（一次发布的记录序号连续且只有末条带标志） 的阳性对照，建在发布 B 之后那份镜像上：
/// ① 干净那一份每次发布一条记录（写行、两次暖机、A、B，序号都是 1、都带末条标志）⇒ I-8.9 真被评估过且成立；
/// ② B 之后接着写一次两条记录的发布（事务 3 跨两条，序号 1、2，提交标记与末条标志都只在第二条，D23（journal 的角色与格式）
///    已定项 17 末条再跨记录的形态）⇒ I-8.9 仍成立、一条红都没有；除 I-8.8（前缀里的事务不被切开） 那一格
///    （一事务两条，它从不适用变成真被评估过且成立）之外每一条的判定与 ① 那一份逐项相同。
///    把多条记录的发布一律判红的实现（例如「每条记录都要带末条标志」）在这一格上红；从来不红的实现由后一条用例的坏镜像挡。
#[test]
fn one_publish_over_two_records_with_the_last_record_flag_on_the_second_holds_the_publish_ordinal_invariant(
) {
    let clean = image_after_the_overwrite("known-bad-publish-ordinals-positive");
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    assert_eq!(
        verdict(&clean, "I-8.9"),
        InvariantVerdict::Holds,
        "每次发布一条记录：I-8.9 真被评估过且成立"
    );
    let mut image = clean.clone();
    append_publish_records_after_the_overwrite(&mut image, &ONE_PUBLISH_OVER_TWO_RECORDS);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        (verdict(&image, "I-8.8"), verdict(&image, "I-8.9")),
        (InvariantVerdict::Holds, InvariantVerdict::Holds),
        "一次发布两条记录：I-8.8 与 I-8.9 都要真被评估过且成立"
    );
    assert_eq!(
        verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE),
        InvariantVerdict::Holds,
        "这次两条记录的发布在最新根之上、末条带标志与提交标记：下一次挂载整次重放它，重放之后那一版走得完"
    );
    assert_eq!(
        publish_ordinal_judgement("一次发布两条记录", &verdicts, &verdicts_on_the_clean_image),
        PublishOrdinalJudgement {
            what_is_changed: "一次发布两条记录",
            violated_invariants: Vec::new(),
            reddened_criteria: Vec::new(),
            other_verdicts_that_changed: vec!["I-8.8"],
        },
        "一条红都不该有，除 I-8.8 从不适用变成成立之外别的判定一条不变：{verdicts:?}"
    );
}

/// I-8.9（一次发布的记录序号连续且只有末条带标志） 的七份坏镜像（`PUBLISH_ORDINAL_BAD_IMAGES`，每份只在阳性对照那个形态上改一处）：
/// 每份**只红 I-8.9**，违例说明里红的判据恰是登记的那几条，除 I-8.9 之外每一条的判定与阳性对照那一份逐项相同。
/// 七份的判定先全部收齐、再整张表比一次，一处改坏时红的消息里看得到每一份各红在哪。
#[test]
fn each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant_on_its_registered_criteria(
) {
    let clean = image_after_the_overwrite("known-bad-publish-ordinals");
    let mut positive_control = clean.clone();
    append_publish_records_after_the_overwrite(
        &mut positive_control,
        &ONE_PUBLISH_OVER_TWO_RECORDS,
    );
    let verdicts_on_the_positive_control = check_pool_image(&positive_control);
    let mut judged_by_image: Vec<PublishOrdinalJudgement> = Vec::new();
    let mut registered_by_image: Vec<PublishOrdinalJudgement> = Vec::new();
    let mut details_by_image: Vec<(&str, InvariantVerdict)> = Vec::new();
    for PublishOrdinalBadImage {
        what_is_changed,
        registered_criteria,
        appended_records,
    } in PUBLISH_ORDINAL_BAD_IMAGES
    {
        let mut image = clean.clone();
        append_publish_records_after_the_overwrite(&mut image, appended_records);
        let verdicts = check_pool_image(&image);
        judged_by_image.push(publish_ordinal_judgement(
            what_is_changed,
            &verdicts,
            &verdicts_on_the_positive_control,
        ));
        registered_by_image.push(PublishOrdinalJudgement {
            what_is_changed,
            violated_invariants: vec!["I-8.9"],
            reddened_criteria: registered_criteria.to_vec(),
            other_verdicts_that_changed: Vec::new(),
        });
        details_by_image.push((what_is_changed, verdict(&image, "I-8.9")));
    }
    assert_eq!(
        judged_by_image, registered_by_image,
        "每份坏镜像只红 I-8.9、红在登记的那几条判据上、别的判定一条不变；(镜像, I-8.9) 逐份是 {details_by_image:?}"
    );
}

/// C374 那两条：发布 B 之后的干净镜像上每条不变量都成立，两份坏镜像各自**只**红在自己那一条上
/// （别的不变量跟着红就说明镜像改宽了，判别力算不到这一条头上）。
#[test]
fn each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite(
) {
    let clean = image_after_the_overwrite("known-bad-after-overwrite");
    for (invariant, found) in check_pool_image(&clean) {
        assert_eq!(
            found,
            if invariant == INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM {
                // 只有实例 1，实例表一行都没有（回退那几份有行，那里 I-3.12 成立）。
                InvariantVerdict::NotApplicable(INSTANCE_TABLE_ROWS_NOT_APPLICABLE)
            } else {
                expected_verdict_with_one_record_per_transaction(invariant)
            },
            "发布 B 之后的干净镜像上 {invariant} 的判定"
        );
    }
    for (invariant, mutation) in known_bad_images_after_the_overwrite() {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        let violated: Vec<&str> = verdicts
            .iter()
            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(
            violated,
            [invariant],
            "判红的该只有 {invariant}：{verdicts:?}"
        );
    }
}

/// I-7.3（环健康性） 的两支各跑一份，都建在 **mkfs 刚写完**那份镜像上——两份镜像只差三条根的 `checkpoint_txg`
/// （0 → 1），别的字节逐字相同：
/// ① 例外那一支的阳性对照：S 全部是第 0 代 ⇒ 判绿，而且要真被评估过（不是「不适用」）；
/// ② 代号一起改成 1 之后 S 里没有更早代号的记录、例外又够不着 ⇒ 判红，而且**除 I-7.3 之外每一条的判定与 ① 那一份逐项相同**。
/// 两份的唯一差别就是那个代号 ⇒ 这一条分得出差别，判别力算得到它头上（`test-discipline.md`「检查本身也可能是错的」）。
///
/// 第二段是写完新池新建文件那份镜像上的形态：S 只剩代号最大的那一条（除 txg 3 之外的根槽自证校验和都改坏）。
/// 它同时红 I-3.1（已分配统计对得上）——mkfs 种的树表单元（50178）只有那条第 0 代根还引用着，去掉它遍历就少算 16384，
/// 两条一起红是这份镜像的结构决定的，照实钉下来。
#[test]
fn the_root_ring_health_invariant_reddens_when_the_previous_generation_is_gone_but_not_right_after_mkfs(
) {
    let built = build_pool("known-bad-root-ring-health");
    let freshly_formatted = built.memory_pool_after_mkfs();
    let generations_right_after_mkfs: Vec<u64> = root_slots()
        .into_iter()
        .map(|(device, offset)| read(&freshly_formatted, device, offset, 512))
        .filter(|bytes| bytes[..4] == *b"SFSR")
        .map(|bytes| get_u64(&bytes, 28))
        .collect();
    assert_eq!(
        generations_right_after_mkfs,
        [0, 0, 0],
        "mkfs 刚写完：三个区域的槽 0 各一条第 0 代根，例外那一支罩的就是这个态"
    );
    let verdicts_right_after_mkfs = check_pool_image(&freshly_formatted);
    assert_eq!(
        verdict(&freshly_formatted, "I-7.3"),
        InvariantVerdict::Holds,
        "S 全部是第 0 代 ⇒ 例外那一支判绿，而且要真被评估过"
    );
    for (invariant, mutation) in known_bad_images_of_the_root_ring_health() {
        let mut image = freshly_formatted.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        let violated: Vec<&str> = verdicts
            .iter()
            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
            .map(|(name, _)| *name)
            .collect();
        // 三个区域槽 0 都坐着 txg 1 的根，而 txg 1 只许住区域 1 槽 0：区域 0、2 那两条照 I-7.17（根槽位置等于 txg 算出的位置）
        // 必然一起红，它是这份镜像造法自带的，不是 I-7.3 搭车；I-7.3 的判别力仍由「两份只差那个代号」钉住（I-7.17 之外逐项相同）。
        let red_with_it = [invariant, ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES];
        assert_eq!(
            violated, red_with_it,
            "判红的该只有 {invariant} 与 txg 1 住错区域的 I-7.17：{verdicts:?}"
        );
        let differences: Vec<(&str, &InvariantVerdict, &InvariantVerdict)> = verdicts
            .iter()
            .zip(&verdicts_right_after_mkfs)
            .filter(|((name, found), (_, before))| !red_with_it.contains(name) && found != before)
            .map(|((name, found), (_, before))| (*name, before, found))
            .collect();
        assert!(
            differences.is_empty(),
            "除 {invariant} 之外每一条的判定都该与 mkfs 刚写完那一份相同：{differences:?}"
        );
    }
    // 写完新池新建文件那份镜像上的同一个形态：S 只剩 txg 3 那一条。
    let after_the_new_pool_file_creation = built.memory_pool();
    assert_eq!(
        verdict(&after_the_new_pool_file_creation, "I-7.3"),
        InvariantVerdict::Holds,
        "写完新池新建文件的干净镜像上 I-7.3 要真被评估过且成立"
    );
    let mut only_the_newest_root_left = after_the_new_pool_file_creation.clone();
    let mut corrupted_root_slots = 0;
    for (device, offset) in root_slots() {
        if (device, offset) == NEW_POOL_FILE_CREATION_ROOT_SLOT {
            continue;
        }
        let mut bytes = read(&only_the_newest_root_left, device, offset, 512);
        if &bytes[..4] != b"SFSR" {
            continue;
        }
        bytes[138] ^= 0xFF;
        write(&mut only_the_newest_root_left, device, offset, &bytes);
        corrupted_root_slots += 1;
    }
    assert_eq!(
        corrupted_root_slots, 3,
        "根环里除 txg 3 那条之外自证过的根：mkfs 第 0 代 1 条 + 暖机 2 条"
    );
    let verdicts = check_pool_image(&only_the_newest_root_left);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(
        violated,
        ["I-3.1", "I-7.3"],
        "S 只剩 txg 3 那一条：I-7.3 红，I-3.1 跟着红（第 0 代根一走，mkfs 种的树表单元没人引用了）：{verdicts:?}"
    );
}

/// I-1.2（块头写序已发布） 与 I-4.2（无被引用未提交块） 那两份坏镜像**只**红在自己那一条上——两条判的是同一批
/// 被引用单元、同一份输入（头里的诞生代号、写序、出生序号），分得开它们的就是这两份镜像：
/// ① I-1.2 那一份只动头里的出生序号（不是谓词的输入）⇒ 指针对不上而「已发布」照旧成立；
/// ② I-4.2 那一份把头与指针的诞生代号一起抬到最新根的 txg 之上 ⇒ 指针仍然对得上而「已发布」不成立。
/// 一份镜像同时红两条，就说明两条里有一条是搭车判的，判别力算不到它头上。
#[test]
fn the_birth_identity_bad_images_redden_only_their_own_invariant() {
    let clean = build_pool("known-bad-birth-identity").memory_pool();
    let judged_here = ["I-1.2", "I-4.2"];
    let cases: Vec<(&'static str, Mutation)> = known_bad_images(&clean)
        .into_iter()
        .filter(|(invariant, _)| judged_here.contains(invariant))
        .collect();
    assert_eq!(cases.len(), judged_here.len(), "两条各有一份坏镜像");
    for (invariant, mutation) in cases {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        let violated: Vec<&str> = verdicts
            .iter()
            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(
            violated,
            [invariant],
            "判红的该只有 {invariant}：{verdicts:?}"
        );
    }
}

/// I-1.8（归并后版本全序） 与 I-8.6（反向链算法） 那两份坏镜像**只**红在自己那一条上：
/// 别的不变量跟着红就说明镜像改宽了，判别力算不到这一条头上。
/// I-1.8 那一份尤其要看 I-2.1（校验和与内容匹配）：盘 1 那条位置条目跟着补过，两份各自与自己那条位置条目对得上，
/// 分得开「两个副本载荷不同」的只有 I-1.8 这一条。
#[test]
fn the_merge_and_the_back_chain_bad_images_redden_only_their_own_invariant() {
    let clean = build_pool("known-bad-merge-and-back-chain").memory_pool();
    let judged_here = ["I-1.8", "I-8.6"];
    let cases: Vec<(&'static str, Mutation)> = known_bad_images(&clean)
        .into_iter()
        .filter(|(invariant, _)| judged_here.contains(invariant))
        .collect();
    assert_eq!(cases.len(), judged_here.len(), "两条各有一份坏镜像");
    for (invariant, mutation) in cases {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        let violated: Vec<&str> = verdicts
            .iter()
            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(
            violated,
            [invariant],
            "判红的该只有 {invariant}：{verdicts:?}"
        );
    }
}

/// I-7.13（系统配置池级字段在读者收的范围里） 那份坏镜像（盘 1 槽 0 加密类型 1）**只**红 I-7.13，红在盘 1 偏移 0 那一槽；
/// 别的每一条都报不适用（主 agent 2026-09-27 认的处置：实现整池拒绝挂载，checker 不拿别的三槽照判、不替这个池作保）。
#[test]
fn a_system_configuration_slot_whose_encryption_type_is_on_reddens_only_its_own_invariant_and_every_other_is_not_applicable(
) {
    let clean = build_pool("known-bad-refused-system-configuration-value").memory_pool();
    let cases: Vec<(&'static str, Mutation)> = known_bad_images(&clean)
        .into_iter()
        .filter(|(invariant, _)| {
            *invariant == SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS
        })
        .collect();
    assert_eq!(
        cases.len(),
        1,
        "写完新池新建文件那份镜像上 I-7.13 有一份坏镜像"
    );
    for (invariant, mutation) in cases {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert!(
            verdicts.iter().any(|(judged, _)| *judged == invariant),
            "{invariant} 在判定清单里：{verdicts:?}"
        );
        for (judged, found) in &verdicts {
            if *judged == invariant {
                match found {
                    InvariantVerdict::Violated(detail) => assert!(
                        detail.contains(&format!(
                            "盘 {DEVICE_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT} 偏移 {OFFSET_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT}"
                        )),
                        "违例要点名被改的那一槽：{detail}"
                    ),
                    other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
                        panic!("加密类型 1 的槽自证过：实现整池拒绝挂载，{invariant} 要判违例，得到 {other:?}")
                    }
                }
            } else {
                assert!(
                    matches!(found, InvariantVerdict::NotApplicable(_)),
                    "这个池挂不上，{judged} 要报不适用，得到 {found:?}"
                );
            }
        }
    }
}

/// I-7.13（系统配置池级字段在读者收的范围里） 的几何那五格（[`known_bad_images_of_the_system_configuration_geometry`]）各**只**红 I-7.13，
/// 红在盘 1 偏移 0 那一槽、说明里带该报的那个 `Verdict` 成员；别的每一条都报不适用（实现整池拒绝挂载，checker 不替这个池作保）。
/// 改之前 checker 不判 417 = 环长现算、不判段边界、按字段读 325 与 371：A、B、C、D 四格 I-7.13 不红（A、D 红的是 I-5.2，
/// C 红的是 I-7.1），E 格红的是槽距那个成员。
#[test]
fn each_system_configuration_geometry_field_the_reader_refuses_reddens_only_its_own_invariant_naming_the_refused_field(
) {
    let clean = build_pool("known-bad-refused-system-configuration-geometry").memory_pool();
    let cases = known_bad_images_of_the_system_configuration_geometry();
    assert_eq!(cases.len(), 5, "实审 Y4 的 A、B、C、D、E 五格");
    for (refused_member, mutation) in cases {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert!(
            verdicts.iter().any(|(judged, _)| *judged
                == SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS),
            "I-7.13 在判定清单里：{verdicts:?}"
        );
        for (judged, found) in &verdicts {
            if *judged == SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS {
                match found {
                    InvariantVerdict::Violated(detail) => {
                        assert!(
                            detail.contains(&format!(
                                "盘 {DEVICE_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT} 偏移 {OFFSET_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT}"
                            )),
                            "{refused_member}：违例要点名被改的那一槽：{detail}"
                        );
                        assert!(
                            detail.contains(refused_member),
                            "{refused_member}：违例说明里要是这个成员：{detail}"
                        );
                    }
                    other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
                        panic!("{refused_member}：这一槽自证过而带这一版读者不收的值，I-7.13 要判违例，得到 {other:?}")
                    }
                }
            } else {
                assert!(
                    matches!(found, InvariantVerdict::NotApplicable(_)),
                    "{refused_member}：这个池挂不上，{judged} 要报不适用，得到 {found:?}"
                );
            }
        }
    }
}

/// I-7.14（本盘设备号等于盘在池里的身份，暂立） 那份坏镜像（两块盘四个系统配置槽的本盘设备号互换）**只**红 I-7.14，
/// 说明里点名盘 0 上自述 1 的那一槽；别的每一条与干净镜像上的判定逐条相同。改之前 checker 读本盘设备号而不拿它比盘的身份，
/// 这份镜像上 0 条违例（实审 Y2-b 越格线索）。
#[test]
fn devices_whose_own_device_numbers_are_swapped_redden_only_the_own_device_number_invariant() {
    let clean = build_pool("known-bad-swapped-own-device-numbers").memory_pool();
    let clean_verdicts = check_pool_image(&clean);
    let mut image = clean.clone();
    swap_the_own_device_numbers_of_the_two_devices(&mut image);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        vec![SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY],
        "只红 I-7.14：{verdicts:?}"
    );
    let detail = match verdict(
        &image,
        SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
    ) {
        InvariantVerdict::Violated(detail) => detail,
        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
            panic!("本盘设备号互换，I-7.14 要判违例，得到 {other:?}")
        }
    };
    assert!(
        detail.contains("盘 0 上") && detail.contains("本盘设备号 1"),
        "第一处违例是盘 0 上自述 1 的那一槽：{detail}"
    );
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &clean_verdicts,
            &[SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY]
        ),
        Vec::new(),
        "除 I-7.14 之外每一条与干净镜像上的判定逐条相同"
    );
}

#[test]
fn the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target() {
    let clean = build_pool("known-bad").memory_pool();
    for (invariant, found) in check_pool_image(&clean) {
        let expected = match invariant {
            // 写完新池新建文件的镜像上只有 A 的根有树表条目：没有一棵树的条目出现在两个树表单元里，跨根比不出来。
            // 这一条的坏镜像在发布 B 之后那一份上（`each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`）。
            "I-9.14" => InvariantVerdict::NotApplicable(
                "没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来",
            ),
            // 这一份上实例 1 只有 A 一条非 0 事务号的记录（写行与两次暖机空发布都写 0）：一对可比的号凑不出来。
            // 这一条的坏镜像与阳性对照都在发布 B 之后那一份上
            // （`repeated_transaction_number_in_one_instance_reddens_the_uncut_transaction_invariant_and_only_when_separated_the_transaction_number_invariant` 与
            // `each_transaction_boundary_bad_image_reddens_only_its_own_invariant_on_its_registered_criteria`）。
            "I-8.7" => InvariantVerdict::NotApplicable(
                "环里没有哪个实例写出过两条非 0 事务号的记录：一对可比的号都凑不出来",
            ),
            // 一事务一条、每条都带提交标记：③ ④ 没有对象。写行与两次暖机那两条事务号 0 的记录各带提交标记 1，
            // 拿事务号 0 成组的写法在这一份上就红（③：一组里两条带提交标记）——这一格是「事务号 0 不成组」那条射程的阳性对照。
            // 阳性对照与坏镜像在发布 B 之后那一份上
            // （`each_transaction_boundary_bad_image_reddens_only_its_own_invariant_on_its_registered_criteria`）。
            "I-8.8" => InvariantVerdict::NotApplicable(COMMIT_MARKERS_NOT_APPLICABLE),
            // 这一份上没抬过 F：每条根带的 F 都是 0。阳性对照与坏镜像在步 5 那段历史上
            // （`raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`）。
            "I-7.9" => InvariantVerdict::NotApplicable(NO_ROOT_RAISED_THE_FLOOR),
            // 这一份上只有实例 1，实例表一行都没有。阳性对照与坏镜像在可写挂载施加过一条记录的那一份上
            // （`an_instance_table_row_whose_published_txg_reaches_the_root_referencing_it_reddens_only_the_instance_table_row_invariant`）。
            "I-3.12" => InvariantVerdict::NotApplicable(INSTANCE_TABLE_ROWS_NOT_APPLICABLE),
            // 这一份上最新根（txg 3）之上没有记录：重放之后那一版就是它。阳性对照与坏镜像在「发布 B 的记录落了、根槽没落」那一份崩溃镜像上
            // （`a_replayed_version_whose_tree_table_does_not_read_back_reddens_only_the_replayed_version_invariant`）。
            "I-4.1" => InvariantVerdict::NotApplicable(NOTHING_APPLIED_ABOVE_THE_CHOSEN_ROOT),
            _ => InvariantVerdict::Holds,
        };
        assert_eq!(found, expected, "干净镜像上 {invariant} 的判定");
    }
    let cases = known_bad_images(&clean);
    let targets: std::collections::BTreeSet<&str> = cases
        .iter()
        .chain(known_bad_images_after_the_overwrite().iter())
        .chain(known_bad_images_of_overlapping_allocation_records().iter())
        .chain(known_bad_images_of_the_root_ring_health().iter())
        .chain(known_bad_images_of_the_transaction_boundaries().iter())
        .chain(known_bad_images_of_the_publish_ordinals().iter())
        .chain(known_bad_images_of_the_allocation_generation().iter())
        .chain(known_bad_images_of_a_pure_leak().iter())
        .chain(known_bad_images_of_the_deferred_statistic().iter())
        .chain(known_bad_images_of_the_rollback_floor().iter())
        .chain(known_bad_images_of_the_unmount_floor().iter())
        .chain(known_bad_images_of_the_system_configuration_floor().iter())
        .chain(known_bad_images_of_pointer_journal_and_generation_fields().iter())
        .chain(known_bad_images_of_the_system_configuration_slot_generations().iter())
        .chain(known_bad_images_of_the_instance_table_rows().iter())
        .chain(known_bad_images_of_the_ring_geometry_and_the_root_slot_alignment().iter())
        .chain(known_bad_images_of_the_tail_slot_witnesses().iter())
        .chain(known_bad_images_of_the_replayed_version().iter())
        .map(|(invariant, _)| *invariant)
        .collect();
    let listed: std::collections::BTreeSet<&str> = singlefs_checker::image::IMPLEMENTED_INVARIANTS
        .iter()
        .copied()
        .collect();
    assert_eq!(
        targets, listed,
        "第一版每条不变量都至少有一份坏镜像，也没有清单之外的"
    );
    for (invariant, mutation) in cases {
        let mut image = clean.clone();
        mutation(&mut image);
        let found = verdict(&image, invariant);
        assert!(
            matches!(found, InvariantVerdict::Violated(_)),
            "{invariant} 那份坏镜像应当判违例，实际 {found:?}"
        );
    }
}

/// 分配记录树第 1 层、盘 0 那一格的节点（写完新池新建文件那份镜像上：`UNITS` 里 50247，它下面只有盘 0 的叶 61）。
const ALLOCATION_LEVEL_ONE_NODE_OF_DEVICE_ZERO: u64 = 50247;
/// 挂空叶用的槽：新池新建文件写完之后紧挨着树表（50252）的第一个空槽，与已分配的那一段相连——占掉它，空闲段数与全空聚簇段数都不变。
const EMPTY_LEAF_SLOT: u64 = 50253;
/// 空叶的出生序号：这棵树在 txg 3 里发过的号之外的一个。
const EMPTY_LEAF_BIRTH_SEQUENCE: u32 = 200;

/// 写完新池新建文件的镜像上，往分配记录树的根之下挂一个空节点、其余全部照账补齐：
/// 一、空节点那一槽（50253）在每块盘上各记一条已分配记录（分配代 3），记在罩着它的那片叶 61 里；记账的已分配各加一槽、空闲各减一槽；
/// 二、盘 0 的叶 62（按位置罩 [62 × 812, 63 × 812)，那一段里一条记录都没有）写成一个一条条目都没有的节点，放在 50253，
///     挂到第 1 层盘 0 那一格下面；改过的每个单元重封、新的整单元校验和沿引用链补到根槽（`mutate_unit` / `propagate`）。
fn hang_an_empty_leaf_below_the_root_of_the_allocation_record_tree(image: &mut MemoryPool) {
    use singlefs_core::address::SlotNumber;
    use singlefs_core::allocation_record_tree::{
        AllocationRecordTreeGeometry, AllocationRecordTreeNode,
    };
    use singlefs_core::allocator::AllocationRecord;
    use singlefs_core::pointer::{BirthSequence, LocationEntry, NodePointer, PointerHead};
    use singlefs_core::records::{STATISTIC_ALLOCATED_BYTES, STATISTIC_FREE_BYTES};
    use singlefs_core::unit::{build_index_node, parse_index_node};
    let geometry = AllocationRecordTreeGeometry::of_devices(
        &DEVICES.map(|device| (DeviceIdentity(device), image.device_size_in_bytes / SLOT)),
    );
    let filesystem_identifier = parameters().filesystem_identifier;
    for (device, leaf_slot) in DEVICES.into_iter().zip(ALLOCATION_LEAVES) {
        let leaf = parse_index_node(&read(image, 0, leaf_slot * SLOT, 16384)).expect("叶 61");
        let mut entries = leaf.entries.clone();
        entries.push(
            AllocationRecord {
                device: DeviceIdentity(device),
                slot: SlotNumber(EMPTY_LEAF_SLOT),
                span_slots: 1,
                generation: CheckpointTxg(3),
                is_released: false,
            }
            .to_bytes(),
        );
        entries.sort();
        let rebuilt = build_index_node(
            leaf.tree,
            leaf.level,
            leaf.key_width,
            &leaf.smallest_key,
            &leaf.largest_key,
            leaf.birth_txg,
            &filesystem_identifier,
            leaf.instance,
            leaf.birth_sequence,
            u16::try_from(leaf.entry_width).expect("20"),
            &entries,
        );
        mutate_unit(
            image,
            leaf_slot,
            |bytes| bytes.copy_from_slice(&rebuilt),
            false,
        );
    }
    mutate_unit(
        image,
        ACCOUNTING_ROOT,
        |bytes| {
            for device in DEVICES {
                adjust_accounting(bytes, STATISTIC_ALLOCATED_BYTES, device, SLOT);
                reduce_accounting(bytes, STATISTIC_FREE_BYTES, device, SLOT);
            }
        },
        true,
    );
    let level_one = parse_index_node(&read(
        image,
        0,
        ALLOCATION_LEVEL_ONE_NODE_OF_DEVICE_ZERO * SLOT,
        16384,
    ))
    .expect("第 1 层盘 0 那一格");
    let (smallest_key, largest_key) = geometry.key_range_of(
        AllocationRecordTreeNode::BelowTheRoot(AllocationRecordTreeNodePosition {
            level: 0,
            device: DeviceIdentity(0),
            index_in_device: 62,
        }),
    );
    let empty_leaf = build_index_node(
        level_one.tree,
        0,
        level_one.key_width,
        &smallest_key,
        &largest_key,
        level_one.birth_txg,
        &filesystem_identifier,
        level_one.instance,
        BirthSequence(EMPTY_LEAF_BIRTH_SEQUENCE),
        u16::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
        &[],
    );
    for device in DEVICES {
        write(image, device, EMPTY_LEAF_SLOT * SLOT, &empty_leaf);
    }
    let empty_leaf_checksum = crc32_castagnoli(&empty_leaf);
    let pointer = NodePointer {
        head: PointerHead {
            birth_tree: level_one.tree,
            birth_txg: level_one.birth_txg,
        },
        locations: DEVICES.map(|device| LocationEntry {
            device: DeviceIdentity(device),
            slot: SlotNumber(EMPTY_LEAF_SLOT),
            unit_checksum: empty_leaf_checksum,
        }),
        instance: level_one.instance,
        birth_sequence: BirthSequence(EMPTY_LEAF_BIRTH_SEQUENCE),
    };
    let mut pointer_writer = singlefs_core::bytes::ByteWriter::new(
        usize::try_from(singlefs_format::NODE_POINTER_BYTES).expect("86"),
    );
    pointer.write_to(&mut pointer_writer);
    let mut entry_for_the_empty_leaf = smallest_key.clone();
    entry_for_the_empty_leaf.extend_from_slice(&pointer_writer.into_bytes());
    let mut level_one_entries = level_one.entries.clone();
    level_one_entries.push(entry_for_the_empty_leaf);
    let rebuilt_level_one = build_index_node(
        level_one.tree,
        level_one.level,
        level_one.key_width,
        &level_one.smallest_key,
        &level_one.largest_key,
        level_one.birth_txg,
        &filesystem_identifier,
        level_one.instance,
        level_one.birth_sequence,
        u16::try_from(level_one.entry_width).expect("96"),
        &level_one_entries,
    );
    mutate_unit(
        image,
        ALLOCATION_LEVEL_ONE_NODE_OF_DEVICE_ZERO,
        |bytes| bytes.copy_from_slice(&rebuilt_level_one),
        false,
    );
}

/// 按位置寻址的树根之下挂一个空节点（实二一报告「停下交主 agent」第 5 条，主 agent 定放在 I-1.1 那一格）：分配记录树第 1 层盘 0 那一格下面
/// 挂一片一条记录都没有的叶 62，账与引用链都补齐（`hang_an_empty_leaf_below_the_root_of_the_allocation_record_tree`）——
/// D8（核心索引结构） 已定项 14「缺席即全空闲、没有记录的一段不写节点」，实现的读者拒收它；池级 checker **只**红 I-1.1（块头自述逻辑地址），
/// 红在读那个节点那一步：`check_internal_node_separators` 对按位置寻址的节点判「节点不空」（`walk::read_index_node` 的
/// `PrescribedByThePositionJudgedByTheCaller` 那一臂）。
/// ⚠️ 派发规格写「今天的 checker 上必须不红、改完红」：这份镜像在实二五开工那一刻的 checker 上就只红 I-1.1（草稿副本上现跑过），
/// 那一判已经在，`walk.rs` 没改；这条用例钉住它，变异表里另有一行拿掉那一判时它必须红。
#[test]
fn an_empty_leaf_hung_below_the_root_of_the_allocation_record_tree_reddens_only_the_block_identity_invariant(
) {
    let clean = build_pool("known-bad-empty-leaf-below-the-root").memory_pool();
    assert_eq!(
        violated_invariants(&check_pool_image(&clean)),
        Vec::<&str>::new(),
        "干净镜像上一条不红"
    );
    let mut image = clean;
    hang_an_empty_leaf_below_the_root_of_the_allocation_record_tree(&mut image);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        vec!["I-1.1"],
        "只红 I-1.1：账、校验和、引用链都补齐了"
    );
    let detail = verdicts
        .iter()
        .find_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) if *invariant == "I-1.1" => Some(detail.clone()),
            InvariantVerdict::Violated(_)
            | InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_) => None,
        })
        .expect("I-1.1 红");
    assert!(
        detail.contains("层级 0 盘 0 第 62 格的节点"),
        "红在挂上去的那片空叶上：{detail}"
    );
}

/// inode 记录里 `blocks` 的偏移（D8（核心索引结构） 已定项 6 的偏移表：size 在 40，blocks 在 48）。
const INODE_RECORD_BLOCKS_OFFSET: usize = 48;

/// I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）：新池新建文件那条记录 size 3000、blocks 6。三个取样值各改一份：
/// 64 是按分到一个 32 KiB 单元算的旧占位值，5 是向下取整，7 是多一块；每一份都**只**红 I-9.15，红在「blocks 与 ⌈3000 ÷ 512⌉ = 6 不等」。
/// 盘上原样的 6 就是干净镜像（`the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 判它成立）。
#[test]
fn inode_record_blocks_other_than_the_logical_length_in_512_byte_blocks_reddens_only_the_blocks_invariant(
) {
    let clean = build_pool("known-bad-inode-blocks").memory_pool();
    assert_eq!(
        verdict(&clean, "I-9.15"),
        InvariantVerdict::Holds,
        "干净镜像上 blocks = 6：I-9.15 真被评估过且成立"
    );
    for wrong_blocks in [64u64, 5, 7] {
        let mut image = clean.clone();
        mutate_unit(
            &mut image,
            INODE_LEAF,
            |bytes| set_u64(bytes, 136 + INODE_RECORD_BLOCKS_OFFSET, wrong_blocks),
            true,
        );
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            ["I-9.15"],
            "blocks 写成 {wrong_blocks}：判红的该只有 I-9.15：{verdicts:?}"
        );
        let InvariantVerdict::Violated(detail) = verdict(&image, "I-9.15") else {
            panic!("上面判过 I-9.15 红");
        };
        assert_eq!(
            detail,
            format!("inode 1 的记录 blocks {wrong_blocks} 不等于 ⌈size 3000 ÷ 512⌉ = 6"),
            "红在第一个文件那条记录上"
        );
    }
}

/// 树表条目里种类那 2 字节的偏移（树 ID 8 + 条目长度 2 之后，D8（核心索引结构） 已定项 8）。
const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
/// 新池新建文件那一版树表里十一条按发号次序排：livelist 是第 4 条（从 0 数，号 65541）、稀疏旁表第 5 条（号 65542）。
const LIVELIST_TREE_TABLE_ENTRY_INDEX: usize = 4;
const SPARSE_SIDE_TABLE_TREE_TABLE_ENTRY_INDEX: usize = 5;
const TREE_KIND_LIVELIST: u16 = 6;
const TREE_KIND_SPARSE_SIDE_TABLE: u16 = 7;

/// 树表单元里 livelist 与稀疏旁表两条互换种类（条目偏移 10 的 2 字节）；树 ID、根指针不动。
fn swap_the_kinds_of_the_livelist_and_sparse_side_table_entries(tree_table: &mut [u8]) {
    for (index, kind_before, kind_after) in [
        (
            LIVELIST_TREE_TABLE_ENTRY_INDEX,
            TREE_KIND_LIVELIST,
            TREE_KIND_SPARSE_SIDE_TABLE,
        ),
        (
            SPARSE_SIDE_TABLE_TREE_TABLE_ENTRY_INDEX,
            TREE_KIND_SPARSE_SIDE_TABLE,
            TREE_KIND_LIVELIST,
        ),
    ] {
        let kind_offset = tree_table_entry_offset(index) + TREE_TABLE_ENTRY_KIND_OFFSET;
        assert_eq!(
            u16::from_le_bytes([tree_table[kind_offset], tree_table[kind_offset + 1]]),
            kind_before,
            "树表第 {index} 条的种类"
        );
        set_u16(tree_table, kind_offset, kind_after);
    }
}

/// I-9.16（树表条目按树 ID 严格升序且合发号次序）：写完新池新建文件那一版树表里 livelist（号 65541）与稀疏旁表（号 65542）两条互换种类。
/// 盘上次序仍按树 ID 升序（① 成立，树表的 key 就是树 ID，I-1.1 也不红），按发号次序 livelist 的号 65542 大过稀疏旁表的号 65541（② 不成立）：
/// **只**红 I-9.16，第一处违例写明最新那条根（实例 1、txg 3）与发号次序上相邻的那两条。两条的根都是空根，改种类不牵动别的判定。
/// 两条整条互换位置（只违反 ①）的那一份连 I-1.1 一起红，钉在 `tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs`。
#[test]
fn a_tree_table_whose_two_entries_swapped_their_kinds_reddens_only_the_tree_table_ordering_invariant(
) {
    let clean = build_pool("known-bad-tree-table-ordering").memory_pool();
    assert_eq!(
        verdict(&clean, "I-9.16"),
        InvariantVerdict::Holds,
        "干净镜像上 I-9.16 真被评估过且成立"
    );
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| swap_the_kinds_of_the_livelist_and_sparse_side_table_entries(bytes),
        true,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-9.16"],
        "两条互换种类：判红的该只有 I-9.16：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-9.16"),
        InvariantVerdict::Violated(
            "根（实例 1、txg 3）的树表：种类 6 那条（第 5 条，树 ID 65542）与发号次序上的下一棵、种类 7 那条（第 4 条，树 ID 65541）不按树 ID 严格升序（② 发号次序）"
                .to_owned()
        ),
        "红在最新那条根的树表、livelist 与稀疏旁表那两条"
    );
}

/// 新池新建文件写出的中央映射树根（bump 次序：分配记录树五个节点、记账树、映射树、树表）。
const MAPPING_ROOT: u64 = 50251;
/// 根记录里中央映射树根指针的出生树：指针在根记录偏移 256（D22（单元原子性怎么合成） 已定项 7），出生树在指针偏移 34
/// （D19（块指针的结构与宽度预算） 已定项 11）。
const ROOT_RECORD_MAPPING_POINTER_BIRTH_TREE_OFFSET: usize = 256 + 34;
/// 这个池发过、又不是中央映射树的一个号（livelist 树 65541），低于水位 65548。
const ANOTHER_ISSUED_TREE_BELOW_THE_WATERMARK: u64 = 65541;

/// 中央映射树不进树表：I-1.3（块头树 ID 一致） 对它的根，「实际引用它的树」按根记录里那条根指针的出生树取
/// （回退到树表 0 条的一版之后再发第一个文件版本，十二棵树从水位重新发号，映射树不再恒是 65540——C511（回退到无文件那一版之后诞生代怎么接））。
/// 两份坏镜像各只改一边、都只有 I-1.3 红：映射树根头里的树 ID 改成 65541（指针仍说 65540），与根记录里那条指针的出生树改成 65541
/// （头仍说 65540，重封根槽的自证校验和）。后一份钉的是「按指针判」这个读法：checker 退回写死 65540 时它不红。
/// 取 65541 不取一个大数：65541 是这个池发过的号（livelist 树）、低于水位 65548，改成水位之上的号会让 I-7.8（根记录树 ID 水位不低于全池最大树 ID）
/// 跟着红（它扫码 2 单元头），判别力就算不到 I-1.3 一条头上。
#[test]
fn central_mapping_root_whose_header_tree_differs_from_its_root_pointer_birth_tree_reddens_only_the_tree_identifier_invariant(
) {
    let clean = build_pool("known-bad-central-mapping-tree").memory_pool();
    let violated_on_the_clean_image = violated_invariants(&check_pool_image(&clean));
    assert!(
        violated_on_the_clean_image.is_empty(),
        "干净镜像上一条违例都没有：{violated_on_the_clean_image:?}"
    );
    let header_says_another_tree: Mutation = Box::new(|image: &mut MemoryPool| {
        mutate_unit(
            image,
            MAPPING_ROOT,
            |bytes| set_u64(bytes, 42, ANOTHER_ISSUED_TREE_BELOW_THE_WATERMARK),
            true,
        );
    });
    let root_pointer_says_another_tree: Mutation = Box::new(|image: &mut MemoryPool| {
        let (device, offset) = NEW_POOL_FILE_CREATION_ROOT_SLOT;
        let mut root_slot = read(image, device, offset, 512);
        assert_eq!(
            get_u64(&root_slot, ROOT_RECORD_MAPPING_POINTER_BIRTH_TREE_OFFSET),
            65540,
            "新池新建文件那条根的映射树根指针说出生树 65540"
        );
        set_u64(
            &mut root_slot,
            ROOT_RECORD_MAPPING_POINTER_BIRTH_TREE_OFFSET,
            ANOTHER_ISSUED_TREE_BELOW_THE_WATERMARK,
        );
        let digest = wide_checksum_with_field_zeroed(&root_slot, 512, 138);
        root_slot[138..170].copy_from_slice(&digest);
        write(image, device, offset, &root_slot);
        copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
    });
    for (which_side, mutation) in [
        ("映射树根头里的树 ID", header_says_another_tree),
        (
            "根记录里映射树根指针的出生树",
            root_pointer_says_another_tree,
        ),
    ] {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            ["I-1.3"],
            "只改了{which_side}：判红的该只有 I-1.3：{verdicts:?}"
        );
    }
}

/// 记账树叶里 (统计量, 设备) 那一行的值改成 value（偏移口径同 `adjust_accounting`）。
fn set_accounting(bytes: &mut [u8], statistic: u16, device: u32, value: u64) {
    let count = usize::from(u16::from_le_bytes([bytes[82 + 44], bytes[83 + 44]]));
    for index in 0..count {
        let row = 159 + 34 * index;
        if u16::from_le_bytes([bytes[row], bytes[row + 1]]) == statistic
            && u32::from_le_bytes(bytes[row + 10..row + 14].try_into().expect("4")) == device
        {
            bytes[row + 22..row + 30].copy_from_slice(&value.to_le_bytes());
            return;
        }
    }
    panic!("记账树里没有 ({statistic}, {device}) 这一行");
}

/// 记账树叶里 (统计量, 设备) 那一行的值加 delta（条目区从 115 + 2 × 22 = 159 起，每条 34：统计量 2、树 8、设备 4、代 8、值 8、seq 4）。
fn adjust_accounting(bytes: &mut [u8], statistic: u16, device: u32, delta: u64) {
    let count = usize::from(u16::from_le_bytes([bytes[82 + 44], bytes[83 + 44]]));
    for index in 0..count {
        let row = 159 + 34 * index;
        if u16::from_le_bytes([bytes[row], bytes[row + 1]]) == statistic
            && u32::from_le_bytes(bytes[row + 10..row + 14].try_into().expect("4")) == device
        {
            let value = u64::from_le_bytes(bytes[row + 22..row + 30].try_into().expect("8"));
            bytes[row + 22..row + 30].copy_from_slice(&(value + delta).to_le_bytes());
            return;
        }
    }
    panic!("记账树里没有 ({statistic}, {device}) 这一行");
}

/// 只被最新根引用的单元被盖（A 的数据单元改内容重封，暖机根与种子根都不引用它）：I-7.4 在最新根那一格也要红，
/// 不能只在更早的候选根上判（步 6 checker 三方第一轮本地攻方腿：两条不变量都写「候选集里每一条根」，最新根也在候选集里；
/// 根环种子 24 条 txg 0 的根让「候选集只剩最新根」构造不出来，所以用这一格钉最新根那一次判定）。
#[test]
fn reused_unit_referenced_only_by_the_newest_root_makes_the_reuse_invariant_red() {
    let mut image = build_pool("known-bad-newest-only").memory_pool();
    // 直接改盘上的单元、不把新校验和传播进位置条目：这就是「块被别的对象盖了」在盘上的样子。
    let mut unit = read(&image, 0, DATA_UNIT * SLOT, 32768);
    set_u64(&mut unit, 100, 77);
    reseal_unit(&mut unit);
    for device in DEVICES {
        write(&mut image, device, DATA_UNIT * SLOT, &unit);
    }
    assert!(
        matches!(verdict(&image, "I-7.4"), InvariantVerdict::Violated(_)),
        "只被 A 引用的数据单元被盖，I-7.4 要在最新根上判红：{:?}",
        verdict(&image, "I-7.4")
    );
}

/// I-1.10（码 2 条目宽等于字段表宽） 那份坏镜像**只**红在 I-1.10 上：记账树根自述的条目宽缩到 key 宽 22
/// （字段表宽 34），别的自描述字段跟着调成自洽的一份（见 `narrow_the_entry_width_to_the_key_width`）。
///
/// 跟着变的只有四条**不适用**：走读在判红的那一步停下、不按字段表解那 15 行记账条目，于是
/// I-3.1（已分配统计对得上）、I-3.11（已分配减 defer 等于最新根走读）、I-5.2（空闲统计对得上）与 I-9.6（水位大于两处最大号） 都没有了判的对象。
/// 「停下不解」正是 I-1.10（码 2 条目宽等于字段表宽） 要的判定顺序（**先于**按字段表解条目，与
/// I-1.7（打包容器合法与判定顺序） 同一纪律），所以这四条转成不适用是这份镜像的结构决定的，照实钉下来。
#[test]
fn an_entry_width_that_is_not_the_field_table_width_reddens_only_the_entry_width_invariant() {
    let clean = build_pool("known-bad-entry-width").memory_pool();
    assert_eq!(
        verdict(&clean, "I-1.10"),
        InvariantVerdict::Holds,
        "干净镜像上 I-1.10 要真被评估过且成立（四棵树的根都比过条目宽）"
    );
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        ACCOUNTING_ROOT,
        |bytes| narrow_the_entry_width_to_the_key_width(bytes),
        true,
    );
    let verdicts = check_pool_image(&image);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(violated, ["I-1.10"], "判红的该只有 I-1.10：{verdicts:?}");
    let turned_not_applicable: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::NotApplicable(_)))
        .map(|(name, _)| *name)
        .filter(|name| !matches!(verdict(&clean, name), InvariantVerdict::NotApplicable(_)))
        .collect();
    assert_eq!(
        turned_not_applicable,
        ["I-3.1", "I-3.11", "I-5.2", "I-9.6", "I-3.13"],
        "跟着转成不适用的该只有这五条（走读在 I-1.10 那一步停下，记账条目不再解；I-3.13 判的就是记账行的代段）：{verdicts:?}"
    );
}

/// 系统配置自述的区域数 R 越过字段表给逐区域设备身份留的那三个字段（偏移 379 / 383 / 387）时，
/// `geometry_of` 交回 [`singlefs_checker::Verdict::RegionCountPastTheRegionDeviceFields`]，不夹成 3、也不 panic
/// （panic 面普查 R12：`root_slot_positions` 拿这个数下标一个长 3 的数组）。
///
/// 写死的字节：槽里偏移 361 那一字节（R）从 3 改成 4，整槽自证校验和照新内容重算 ⇒
/// `check_system_configuration_slot` 那一道照样过，**只有**新加的这一道拦得住它。
#[test]
fn region_count_past_the_region_device_fields_is_refused_by_the_geometry_reader() {
    let mut pool = build_pool("known-bad-region-count").memory_pool();
    let clean_slot = read(&pool, 0, 0, 4096);
    let clean_view =
        singlefs_checker::check_system_configuration_slot(&clean_slot).expect("干净槽自证过");
    assert_eq!(clean_slot[361], 3, "干净镜像上的区域数是 3");
    assert!(
        singlefs_checker::image::geometry_of(&clean_slot, &clean_view).is_ok(),
        "R = 3 正好用满那三个字段，要交得出几何"
    );
    mutate_system_configuration_slot(&mut pool, 0, 0, |bytes| bytes[361] = 4);
    let damaged_slot = read(&pool, 0, 0, 4096);
    let damaged_view = singlefs_checker::check_system_configuration_slot(&damaged_slot)
        .expect("改完重算过整槽校验和，自证那一道还过");
    assert_eq!(
        singlefs_checker::image::geometry_of(&damaged_slot, &damaged_view),
        Err(singlefs_checker::Verdict::RegionCountPastTheRegionDeviceFields),
        "R = 4 时第 4 个区域没有设备身份可读，这一槽不可择"
    );
}

/// A (1, 3)、B (1, 4) → 重开取号 2、写行、暖机两次、C (2, 8)，与 `rollback_by_a_forward_publish.rs` 的固定脚本同一段。
/// 交回池、A 与 C 那两版。
fn pool_through_the_third_version(tag: &str) -> (BuiltPool, TransactionOutput, TransactionOutput) {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let second_content: Vec<u8> = (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect();
    pool.output = publish_overwrite_in_process(
        &mut pool,
        &first,
        &second_content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写 B");
    let mut devices_reopened_for_the_second_instance = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices_reopened_for_the_second_instance)
        .expect("重开取号 2");
    pool.devices = Some(devices_reopened_for_the_second_instance);
    pool.allocator = mounted.allocator;
    let current = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    let third_content: Vec<u8> = (0..2500usize)
        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
        .collect();
    let third = publish_overwrite_in_process(
        &mut pool,
        &current,
        &third_content,
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(2),
    )
    .expect("覆盖写 C");
    pool.output = third.clone();
    (pool, first, third)
}

/// 挂着的时候回退到 A 之后的干净镜像（D23（journal 的角色与格式） 已定项 14：一次向前发布 D (2, 9)，新根的 inode 树、extent 树取 A 的，
/// 树表现算、另写一片）。A、B、C、D 同在一条时间线上：没有哪条根被实例表判出局。
struct ImageAfterTheRollbackToTheFirstRoot {
    image: MemoryPool,
    /// 回退目标 A 的 txg：它那一版树表里 extent 树那条的诞生 txg 就是它。
    rollback_target_txg: u64,
    /// D 那片树表单元的槽号（两盘同槽）与 D 那次发布的 txg。
    tree_table_slot_of_the_rollback_publish: u64,
    rollback_publish_txg: u64,
}

/// 两盘同槽（D2（RAID 条带策略） 已定项 10）：`mutate_unit_of` 读第一块盘那一份、两盘写同一个槽，靠的就是这一条。
fn slot_shared_by_both_copies(
    pointer_locations: [singlefs_core::pointer::LocationEntry; 2],
) -> u64 {
    assert_eq!(
        pointer_locations[0].slot, pointer_locations[1].slot,
        "两盘同槽：这份镜像的改法按一个槽号改两盘"
    );
    pointer_locations[0].slot.0
}

fn image_after_rolling_back_to_the_first_root(tag: &str) -> ImageAfterTheRollbackToTheFirstRoot {
    let (mut pool, first, mut third) = pool_through_the_third_version(tag);
    let rollback_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &rollback_parameters,
        devices,
        &mut pool.allocator,
        &mut third,
        RollbackTarget {
            instance: first.root.instance,
            checkpoint_txg: first.root.checkpoint_txg,
        },
    )
    .expect("挂着的时候回退到 A");
    let rollback_publish = third;
    ImageAfterTheRollbackToTheFirstRoot {
        image: pool.memory_pool(),
        rollback_target_txg: first.root.checkpoint_txg.0,
        tree_table_slot_of_the_rollback_publish: slot_shared_by_both_copies(
            rollback_publish.root.tree_table.locations,
        ),
        rollback_publish_txg: rollback_publish.root.checkpoint_txg.0,
    }
}

/// 崩溃恢复抛弃 C 之后的干净镜像（`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：
/// C 的根槽与数据单元暂时读不出，恢复落到 (2, 7)，实例 3 写行与暖机，再把 C 写回）：C (2, 8) 读得出、按实例 3 的实例表判出局。
/// 交回镜像、A 那一版记的诞生 txg、C 那片树表单元的槽号与 C 那次发布的 txg。
fn image_after_a_recovery_abandoned_the_third_root(tag: &str) -> (MemoryPool, u64, u64, u64) {
    let (mut pool, first, third) = pool_through_the_third_version(tag);
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &third);
    (
        pool.memory_pool(),
        first.root.checkpoint_txg.0,
        slot_shared_by_both_copies(third.root.tree_table.locations),
        third.root.checkpoint_txg.0,
    )
}

/// 一片树表单元里 extent 树（树 ID 65536，按树 ID 升序排第一条）那条的诞生 txg 从 `from` 改成 `to`，重封、沿引用链补到根槽。
fn rewrite_the_extent_tree_birth_txg_in_tree_table(
    image: &mut MemoryPool,
    tree_table_slot: u64,
    from: u64,
    to: u64,
) {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    mutate_unit_of(
        image,
        &[(tree_table_slot, node_bytes)],
        tree_table_slot,
        |bytes| {
            assert_eq!(
                get_u64(bytes, tree_table_entry_offset(0)),
                TREE_IDENTIFIER_EXTENT,
                "树表第一条是 extent 树"
            );
            let birth_txg = tree_table_entry_offset(0) + TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET;
            assert_eq!(
                get_u64(bytes, birth_txg),
                from,
                "槽 {tree_table_slot} 那片树表里 extent 树的诞生 txg 本来是 {from}"
            );
            set_u64(bytes, birth_txg, to);
        },
        true,
    );
}

/// 反向那一头（C511 收窄之后仍要红的那一格）：挂着的时候回退到 A 之后，D 那片树表把 extent 树的诞生 txg 记成 D 那次发布的 txg
/// （在现行线上重新建树的样子），而回退目标 A 那一版仍记 3。向前回退不切掉任何一条线，A 与 D 在**同一条时间线上**
/// ⇒ I-9.14（树表条目的诞生 txg 跨根不变） 必须红，而且只红它。把 I-9.14 的收窄做宽到把回退目标判出局的，这条红。
#[test]
fn birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_birth_invariant(
) {
    let rolled_back =
        image_after_rolling_back_to_the_first_root("known-bad-after-rollback-same-line");
    for (invariant, found) in check_pool_image(&rolled_back.image) {
        assert_eq!(
            found,
            expected_verdict_with_one_record_per_transaction(invariant),
            "回退到 A 之后的干净镜像上 {invariant} 的判定"
        );
    }
    let mut image = rolled_back.image.clone();
    rewrite_the_extent_tree_birth_txg_in_tree_table(
        &mut image,
        rolled_back.tree_table_slot_of_the_rollback_publish,
        rolled_back.rollback_target_txg,
        rolled_back.rollback_publish_txg,
    );
    let verdicts = check_pool_image(&image);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(
        violated,
        ["I-9.14"],
        "回退目标 A 与回退那次发布 D 同在现行线上，诞生 txg 不一致只该红 I-9.14：{verdicts:?}"
    );
}

/// 正向那一头（C511 的收窄本身）：只有被崩溃恢复切掉的 C 把 extent 树的诞生 txg 记成它自己那次发布的 txg，
/// 现行线上 A、B、实例 2 与实例 3 的根仍一致记 3 ⇒ I-9.14（树表条目的诞生 txg 跨根不变） 不拿 C 比，判成立；别的每一条也都成立。
/// 管理员回退改成向前发布之后，被切掉的线只由崩溃恢复造出（D23（journal 的角色与格式） 已定项 14）。
/// 把收窄去掉（I-9.14 拿根环里每一条根比，不按实例表判出局）这条就红。
#[test]
fn birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_is_not_compared_and_every_invariant_holds(
) {
    let (clean, birth_txg, tree_table_slot_of_the_abandoned_root, abandoned_publish_txg) =
        image_after_a_recovery_abandoned_the_third_root("known-good-after-recovery-abandoned-line");
    let mut image = clean;
    rewrite_the_extent_tree_birth_txg_in_tree_table(
        &mut image,
        tree_table_slot_of_the_abandoned_root,
        birth_txg,
        abandoned_publish_txg,
    );
    let verdicts = check_pool_image(&image);
    let birth_invariant = verdicts
        .iter()
        .find(|(name, _)| *name == "I-9.14")
        .expect("清单里有 I-9.14");
    assert_eq!(
        birth_invariant.1,
        InvariantVerdict::Holds,
        "被崩溃恢复切掉的 C 不与现行线比；现行线上的树表一致，I-9.14 要真被评估过且成立"
    );
    for (invariant, found) in &verdicts {
        assert_eq!(
            *found,
            expected_verdict_with_one_record_per_transaction(invariant),
            "只改了被抛弃根 C 那片树表里的一个诞生 txg，{invariant} 的判定"
        );
    }
}

/// 发布 B 写出的数据单元：它的分配记录在 B 那一版分配记录树的两片叶（`ALLOCATION_LEAVES_AFTER_OVERWRITE`）里，未释放、分配代 4，
/// 单元头里的诞生代号也是 4（落点由 `overwrite_in_one_instance.rs` 的验收钉住）。
const DATA_UNIT_AFTER_OVERWRITE: u64 = 50182;

/// 分配记录树叶里槽号是 `slot`、**未带**已释放标志、分配代等于 `from` 的记录（两盘各一条）改成 `to`；返回改了几条。
fn rewrite_unreleased_allocation_generation(
    bytes: &mut [u8],
    slot: u64,
    from: u64,
    to: u64,
) -> usize {
    let count = usize::from(u16::from_le_bytes([
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
    ]));
    let mut rewritten = 0;
    for index in 0..count {
        let record = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * index;
        let mut slot_bytes = [0u8; 8];
        slot_bytes[..6].copy_from_slice(&bytes[record + 4..record + 10]);
        let span_field = u16::from_le_bytes([bytes[record + 10], bytes[record + 11]]);
        if u64::from_le_bytes(slot_bytes) == slot
            && span_field & ALLOCATION_RECORD_RELEASED_FLAG == 0
            && get_u64(bytes, record + 12) == from
        {
            set_u64(bytes, record + 12, to);
            rewritten += 1;
        }
    }
    rewritten
}

/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 的坏镜像，从发布 B 之后的干净镜像出发：B 那一版分配记录树里
/// B 的数据单元那两条未释放记录，分配代从 4 改回上一次发布的 txg 3，单元一个字节不动。
/// 盘上留下的形状就是增补 3 第 1 件代码三方第一轮攻方腿那条变异（复用一个已回收的落点、改写那条分配记录时分配代没改）
/// 留下的：一条未释放记录的分配代停在上一次的代上，而它罩住的单元是这一次写的。
/// 释放代、跨度、key 一样没动 ⇒ I-3.9（释放代落在停止引用它的那一格区间里）、I-5.4（分配记录罩住的槽互不相交） 与引用集合都不受影响。
fn known_bad_images_of_the_allocation_generation() -> Vec<(&'static str, Mutation)> {
    vec![(
        "I-3.10",
        Box::new(|image: &mut MemoryPool| {
            assert_eq!(
                mutate_allocation_leaves(
                    image,
                    &UNITS_AFTER_OVERWRITE,
                    ALLOCATION_LEAVES_AFTER_OVERWRITE,
                    |bytes| {
                        rewrite_unreleased_allocation_generation(
                            bytes,
                            DATA_UNIT_AFTER_OVERWRITE,
                            4,
                            3,
                        )
                    },
                ),
                2,
                "B 的数据单元两盘各一条未释放记录（两片叶各一条），分配代本来是 4"
            );
        }),
    )]
}

/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）：发布 B 之后的干净镜像上它真被评估过且成立（每条未释放记录的起点槽上
/// 都读得出可用的单元头），坏镜像上**只**红它、别的判定一条不变。
#[test]
fn an_unreleased_allocation_record_that_kept_the_previous_generation_reddens_only_the_allocation_generation_invariant(
) {
    let clean = image_after_the_overwrite("known-bad-allocation-generation");
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    assert_eq!(
        verdict(&clean, "I-3.10"),
        InvariantVerdict::Holds,
        "发布 B 之后的干净镜像上 I-3.10 要真被评估过且成立"
    );
    for (invariant, mutation) in known_bad_images_of_the_allocation_generation() {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [invariant],
            "判红的该只有 {invariant}：{verdicts:?}"
        );
        assert_eq!(
            verdicts_that_differ_outside(&verdicts, &verdicts_on_the_clean_image, &[invariant]),
            [],
            "{invariant} 之外每一条的判定与干净镜像逐项相同"
        );
    }
}

/// 记账树叶里 (统计量, 设备) 那一行的值减 delta（偏移口径同 `adjust_accounting`）。
fn reduce_accounting(bytes: &mut [u8], statistic: u16, device: u32, delta: u64) {
    let count = usize::from(u16::from_le_bytes([bytes[82 + 44], bytes[83 + 44]]));
    for index in 0..count {
        let row = 159 + 34 * index;
        if u16::from_le_bytes([bytes[row], bytes[row + 1]]) == statistic
            && u32::from_le_bytes(bytes[row + 10..row + 14].try_into().expect("4")) == device
        {
            let value = u64::from_le_bytes(bytes[row + 22..row + 30].try_into().expect("8"));
            bytes[row + 22..row + 30].copy_from_slice(
                &value
                    .checked_sub(delta)
                    .expect("这一行的值不小于要减的量")
                    .to_le_bytes(),
            );
            return;
        }
    }
    panic!("记账树里没有 ({statistic}, {device}) 这一行");
}

/// 记账里「defer 队列待释放」那一行的统计量编号（D5（快照 / 空间记账机制） 已定项 4 第 5 项）。
const STATISTIC_DEFER_QUEUE_BYTES: u16 = 5;

/// 一整槽的可分配空间凭空消失、没有任何单元在用它：记账盘 0 的「已分配」+16384、「空闲」−16384（纯泄漏，分配器为一个没人引用的槽
/// 同时减了空闲、加了已分配）。「空闲 + 已分配 = 单元区」照旧成立 ⇒ I-5.2（空闲统计对得上） 管不住它，
/// 记账多于遍历这一侧全仓只有 I-3.1（已分配统计对得上） 一条。`with_inflated_defer_queue` 为真时 defer 队列那一行也抬 16384：
/// 松弛量由被测那一方自己供给（「记账 − 遍历 ≤ defer 行」那种改法当场绿）。
fn leak_one_slot_keeping_free_plus_allocated(bytes: &mut [u8], with_inflated_defer_queue: bool) {
    adjust_accounting(bytes, 1, 0, 16384);
    reduce_accounting(bytes, 2, 0, 16384);
    if with_inflated_defer_queue {
        adjust_accounting(bytes, STATISTIC_DEFER_QUEUE_BYTES, 0, 16384);
    }
}

/// 增补 2 收口表第 54 行那一轮三方（`research/prompts/m2-placement-falsepositive-r1-main-verification.md` Z3）攻方腿的
/// 两份纯泄漏坏镜像，建在写完新池新建文件那份干净镜像上：Z3-A（只泄漏）与 Z3-B（泄漏 + defer 行跟着抬高）。
/// 候选 c（记账 ≥ 遍历）在 Z3-A 上哑、候选 f（记账 − 遍历 ≤ defer 行）在两份上都哑，这一轮的判决就栽在它们上。
fn known_bad_images_of_a_pure_leak() -> Vec<(&'static str, Mutation)> {
    vec![
        (
            "I-3.1",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    ACCOUNTING_ROOT,
                    |bytes| leak_one_slot_keeping_free_plus_allocated(bytes, false),
                    true,
                );
            }),
        ),
        (
            "I-3.1",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    ACCOUNTING_ROOT,
                    |bytes| leak_one_slot_keeping_free_plus_allocated(bytes, true),
                    true,
                );
            }),
        ),
    ]
}

/// 纯泄漏那两份各自判红的不变量，次序照 `known_bad_images_of_a_pure_leak`：
/// Z3-A 只抬「已分配」、defer 行没动 ⇒ 已分配减 defer 比最新根走读多一槽，I-3.11（已分配减 defer 等于最新根走读） 跟着红——
/// 这一形本来就在它的射程里；Z3-B 的 defer 行跟着抬了同一槽 ⇒ 那个差不变，I-3.11 照旧成立，只红 I-3.1。
const INVARIANTS_EACH_PURE_LEAK_REDDENS: [&[&str]; 2] = [&["I-3.1", "I-3.11"], &["I-3.1"]];

/// I-3.1（已分配统计对得上） 判红时 checker 在说明文字里带的机理标识，每一项都读得回来
/// （`singlefs_harness::history::allocation_statistic_mechanism`）：「已知红」清单按它分辨机理，少写一项（比如回退下界 F 那一项）
/// 那一段就读不出来，清单那一形就归成新发现。原来读这段真文字的是 `reuse_after_raising_the_floor.rs` 里收口表第 43 行那一形的用例，
/// 它改钉成双故障形（C583（双故障形：每盘一槽系统配置坏加最新根一时读不出），抬 F 之前已经 I-7.4 红）之后这段文字由这一条钉。
/// 镜像：写完新池新建文件那份干净镜像上记账盘 0 纯泄漏一槽（Z3-A）；环里 mkfs、两次暖机与 A 四条根（txg 0–3）全走到，
/// 没抬过 F，F 是 0、没有根被 F 挡掉。
#[test]
fn the_allocated_statistic_violation_carries_every_mechanism_field_the_known_red_forms_read() {
    let mut image = build_pool("known-bad-allocated-statistic-mechanism").memory_pool();
    mutate_unit(
        &mut image,
        ACCOUNTING_ROOT,
        |bytes| leak_one_slot_keeping_free_plus_allocated(bytes, false),
        true,
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-3.1") else {
        panic!("纯泄漏一槽：I-3.1 要红");
    };
    assert_eq!(
        allocation_statistic_mechanism(&detail),
        Some(AllocationStatisticMechanism {
            root_ring_slot_count: 24,
            newest_root_txg: 3,
            readable_root_slot_count: 4,
            oldest_readable_root_txg: 0,
            walked_root_slot_count: 4,
            root_slots_dropped_as_abandoned: 0,
            rollback_floor: 0,
            root_slots_dropped_below_floor: 0,
        }),
        "I-3.1 的说明文字里机理标识每一项都读得回来：{detail}"
    );
}

/// 候选 b（回退候选集补上「由记录施加出来、根槽从没落盘的那一版」，2026-09-23 用户定）不放宽 I-3.1 的等式、只补等式右边：
/// 纯泄漏的两份坏镜像照样都红 I-3.1，别的判定除 `INVARIANTS_EACH_PURE_LEAK_REDDENS` 登记的之外与干净镜像逐项相同
/// （Z3-A 让 I-5.2 照旧绿，是这份镜像要的样子）。
#[test]
fn pure_leak_that_keeps_free_plus_allocated_equal_to_the_unit_area_reddens_only_the_allocated_statistic_invariant(
) {
    let clean = build_pool("known-bad-pure-leak").memory_pool();
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    let cases = known_bad_images_of_a_pure_leak();
    assert_eq!(
        cases.len(),
        INVARIANTS_EACH_PURE_LEAK_REDDENS.len(),
        "每份纯泄漏镜像都登记了它该红的那几条"
    );
    for ((invariant, mutation), expected_violations) in
        cases.into_iter().zip(INVARIANTS_EACH_PURE_LEAK_REDDENS)
    {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert!(
            expected_violations.contains(&invariant),
            "{invariant} 在登记的那几条里"
        );
        assert_eq!(
            violated_invariants(&verdicts),
            expected_violations,
            "判红的该只有 {expected_violations:?}：{verdicts:?}"
        );
        assert_eq!(
            verdicts_that_differ_outside(
                &verdicts,
                &verdicts_on_the_clean_image,
                expected_violations
            ),
            [],
            "{expected_violations:?} 之外每一条的判定与干净镜像逐项相同"
        );
    }
}

/// 发布 B 写出的记账树（bump 次序：分配记录树五个节点 50257–50261、记账树、映射树、树表 50264）：I-3.11 那几份镜像改的就是它。
const ACCOUNTING_ROOT_AFTER_OVERWRITE: u64 = 50262;
/// 发布 B 之后记账里每盘的「已分配」与「defer 待释放」（槽）：mkfs 3 + A 14 + B 14 = 31；A 的 14 槽 + A 换下的 mkfs 树表 1 槽 = 15
/// （`overwrite_in_one_instance.rs` 的验收钉着同一对数）。最新根（B）走读到的是 31 − 15 = 16 槽：B 的 14 槽 + mkfs 那片实例表 2 槽。
const ALLOCATED_SLOTS_AFTER_OVERWRITE: u64 = 31;
const DEFERRED_SLOTS_AFTER_OVERWRITE: u64 = 15;

/// 记账树叶里 (统计量, 设备) 那一行的值（偏移口径同 `adjust_accounting`）。
fn accounting_value(bytes: &[u8], statistic: u16, device: u32) -> u64 {
    let count = usize::from(u16::from_le_bytes([bytes[82 + 44], bytes[83 + 44]]));
    (0..count)
        .map(|index| 159 + 34 * index)
        .find(|row| {
            u16::from_le_bytes([bytes[*row], bytes[*row + 1]]) == statistic
                && u32::from_le_bytes(bytes[*row + 10..*row + 14].try_into().expect("4")) == device
        })
        .map(|row| get_u64(bytes, row + 22))
        .unwrap_or_else(|| panic!("记账树里没有 ({statistic}, {device}) 这一行"))
}

/// I-3.11（已分配减 defer 等于最新根走读） 的两份坏镜像，从发布 B 之后的干净镜像出发、只改 B 那棵记账树里盘 0 的「defer 待释放」那一行：
/// ① **活单元记成已释放**：defer +1 槽（写者把一个最新根还引用着的单元放进 defer）——已分配与空闲两行一个字节不动，
///    I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 照旧成立，只有已分配减 defer 比最新根走读少一槽；
/// ② **换下的单元没进 defer 账**：defer −1 槽——同样只动 defer 那一行，差比走读多一槽。
/// 两份各盯判据一个方向：把 `==` 放宽成 `≥` 的 checker 在 ① 上哑，放宽成 `≤` 的在 ② 上哑。
fn known_bad_images_of_the_deferred_statistic() -> Vec<(&'static str, Mutation)> {
    let change_deferred_on_device_zero = |grow: bool| -> Mutation {
        Box::new(move |image: &mut MemoryPool| {
            mutate_unit_of(
                image,
                &UNITS_AFTER_OVERWRITE,
                ACCOUNTING_ROOT_AFTER_OVERWRITE,
                |bytes| {
                    assert_eq!(
                        accounting_value(bytes, STATISTIC_DEFER_QUEUE_BYTES, 0),
                        DEFERRED_SLOTS_AFTER_OVERWRITE * SLOT,
                        "改之前盘 0 的 defer 待释放是 15 槽"
                    );
                    if grow {
                        adjust_accounting(bytes, STATISTIC_DEFER_QUEUE_BYTES, 0, SLOT);
                    } else {
                        reduce_accounting(bytes, STATISTIC_DEFER_QUEUE_BYTES, 0, SLOT);
                    }
                },
                true,
            );
        })
    };
    vec![
        ("I-3.11", change_deferred_on_device_zero(true)),
        ("I-3.11", change_deferred_on_device_zero(false)),
    ]
}

/// I-3.11（已分配减 defer 等于最新根走读）：发布 B 之后的干净镜像上它真被评估过且成立（记账的两行先钉绝对值：已分配 31 槽、defer 15 槽），
/// 两份坏镜像上**只**红它、别的判定一条不变——I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 照旧成立，
/// 这就是它补的那一种（写者把活单元记成已释放、放进 defer，两个和照样对得上）。
#[test]
fn live_unit_recorded_as_deferred_reddens_only_the_allocated_minus_deferred_invariant() {
    let clean = image_after_the_overwrite("known-bad-deferred-statistic");
    let accounting_root = read(&clean, 0, ACCOUNTING_ROOT_AFTER_OVERWRITE * SLOT, 16384);
    for device in DEVICES {
        assert_eq!(
            (
                accounting_value(&accounting_root, 1, device),
                accounting_value(&accounting_root, STATISTIC_DEFER_QUEUE_BYTES, device)
            ),
            (
                ALLOCATED_SLOTS_AFTER_OVERWRITE * SLOT,
                DEFERRED_SLOTS_AFTER_OVERWRITE * SLOT
            ),
            "发布 B 之后盘 {device} 的已分配与 defer 待释放"
        );
    }
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    assert_eq!(
        verdict(&clean, "I-3.11"),
        InvariantVerdict::Holds,
        "发布 B 之后的干净镜像上 I-3.11 要真被评估过且成立"
    );
    for (invariant, mutation) in known_bad_images_of_the_deferred_statistic() {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [invariant],
            "判红的该只有 {invariant}：{verdicts:?}"
        );
        assert_eq!(
            verdicts_that_differ_outside(&verdicts, &verdicts_on_the_clean_image, &[invariant]),
            [],
            "{invariant} 之外每一条的判定与干净镜像逐项相同"
        );
        for still_holding in ["I-3.1", "I-5.2"] {
            assert_eq!(
                verdict(&image, still_holding),
                InvariantVerdict::Holds,
                "defer 账对不上时 {still_holding} 照旧真被评估过且成立（判别力只在 I-3.11 上）"
            );
        }
    }
}

/// I-3.11（已分配减 defer 等于最新根走读） 判别力自证那一格（`.claude/kb/invariants.md` I-3.11 那一行的 ⚠️）：
/// 「不减第 5 项就转绿」在可达状态上造不出来——每次发布都把换下的单元放进 defer，第 5 项恒 ≥ 1——所以在**造出来的基底**上做
/// （E156（alloc-basis 四条岔路的代价数） 的 β_syn）：发布 B 之后的镜像，把两块盘的 defer 那一行都整个挪回去
/// （每盘已分配 −15 槽、空闲 +15 槽、defer 置 0）。这份基底上 I-3.11 与 I-5.2 成立、I-3.1 红（记账少于遍历，造它就是这样），
/// 它不是合法状态，只拿来做下面这一步：再往盘 0 的 defer 上加一槽，I-3.11 必须红。
/// 两块盘都挪：只挪一块的话，另一块上「已分配 == 最新根走读」不成立，去掉「减第 5 项」的 checker 在基底上就红，转色看不出来。
/// 这一格上两块盘都是「已分配 == 最新根走读」 ⇒ 判据里去掉「减第 5 项」的 checker 在基底上判绿、在加了一槽的那一份上也判绿，
/// 这条用例红在「defer 多记一槽」那一句上（由红转绿的那一格）。
#[test]
fn on_a_synthetic_base_with_an_empty_defer_queue_one_deferred_slot_reddens_the_allocated_minus_deferred_invariant(
) {
    let mut synthetic_base =
        image_after_the_overwrite("known-bad-deferred-statistic-synthetic-base");
    mutate_unit_of(
        &mut synthetic_base,
        &UNITS_AFTER_OVERWRITE,
        ACCOUNTING_ROOT_AFTER_OVERWRITE,
        |bytes| {
            for device in DEVICES {
                let deferred = accounting_value(bytes, STATISTIC_DEFER_QUEUE_BYTES, device);
                assert_eq!(
                    deferred,
                    DEFERRED_SLOTS_AFTER_OVERWRITE * SLOT,
                    "挪之前盘 {device} 的 defer 待释放是 15 槽"
                );
                reduce_accounting(bytes, 1, device, deferred);
                adjust_accounting(bytes, 2, device, deferred);
                set_accounting(bytes, STATISTIC_DEFER_QUEUE_BYTES, device, 0);
            }
        },
        true,
    );
    let accounting_root = read(
        &synthetic_base,
        0,
        ACCOUNTING_ROOT_AFTER_OVERWRITE * SLOT,
        16384,
    );
    for device in DEVICES {
        assert_eq!(
            (
                accounting_value(&accounting_root, 1, device),
                accounting_value(&accounting_root, STATISTIC_DEFER_QUEUE_BYTES, device)
            ),
            (
                (ALLOCATED_SLOTS_AFTER_OVERWRITE - DEFERRED_SLOTS_AFTER_OVERWRITE) * SLOT,
                0
            ),
            "造出来的基底：盘 {device} 已分配 16 槽（= 最新根走读）、defer 0"
        );
    }
    let verdicts_on_the_synthetic_base = check_pool_image(&synthetic_base);
    assert_eq!(
        violated_invariants(&verdicts_on_the_synthetic_base),
        ["I-3.1"],
        "造出来的基底只红 I-3.1（记账少于遍历）：{verdicts_on_the_synthetic_base:?}"
    );
    assert_eq!(
        verdict(&synthetic_base, "I-3.11"),
        InvariantVerdict::Holds,
        "造出来的基底上 I-3.11 真被评估过且成立（已分配 16 − defer 0 = 最新根走读 16）"
    );
    let mut one_deferred_slot = synthetic_base.clone();
    mutate_unit_of(
        &mut one_deferred_slot,
        &UNITS_AFTER_OVERWRITE,
        ACCOUNTING_ROOT_AFTER_OVERWRITE,
        |bytes| adjust_accounting(bytes, STATISTIC_DEFER_QUEUE_BYTES, 0, SLOT),
        true,
    );
    let verdicts = check_pool_image(&one_deferred_slot);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-3.1", "I-3.11"],
        "defer 多记一槽：I-3.11 红（16 − 1 ≠ 16），I-3.1 照旧是基底那一份红：{verdicts:?}"
    );
    assert_eq!(
        verdicts_that_differ_outside(&verdicts, &verdicts_on_the_synthetic_base, &["I-3.11"]),
        [],
        "I-3.11 之外每一条的判定与造出来的基底逐项相同"
    );
}

/// 这一格用到的内容：长度与字节都随 `seed` 变（具体哪些字节不承重）。
fn content_seeded(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

/// 在同一个进程里接着现行版本覆盖写一次，现行版本跟着换成这一版。
fn overwrite_and_advance(pool: &mut BuiltPool, content: &[u8], instance: InstanceGeneration) {
    let previous = pool.output.clone();
    pool.output = publish_overwrite_in_process(
        pool,
        &previous,
        content,
        FIXED_WRITE_TIME_SECONDS + 60,
        instance,
    )
    .expect("覆盖写");
}

/// 步 5 那段历史走到抬 F 之前：A、B、重开写行、C（实例 2）、挂着的时候回退到 (1, 3)（一次向前发布 D，txg 9，实例仍是 2）、
/// 实例 2 覆盖写五次（txg 10–14）。没有哪条根被实例表判出局；非空的不同状态按新到旧是 14、13、12、11、……（D 与 A 是同一个状态，
/// 只算一次），抬 F 的上限 = min(每块盘上最新的 13, 第 4 个不同状态 11) = 11
/// （`reuse_after_raising_the_floor.rs` 的 `raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused` 钉着同一个上限）。
fn pool_before_raising_the_floor(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_and_advance(&mut pool, &content_seeded(4100, 3), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    overwrite_and_advance(&mut pool, &content_seeded(2500, 11), InstanceGeneration(2));
    let rollback_parameters = parameters();
    let open_devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &rollback_parameters,
        open_devices,
        &mut pool.allocator,
        &mut pool.output,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
    )
    .expect("挂着的时候回退到 A");
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(9),
        "回退那次发布是 txg 9"
    );
    for seed in [17usize, 19, 23, 29, 31] {
        overwrite_and_advance(
            &mut pool,
            &content_seeded(3000 + seed, seed),
            InstanceGeneration(2),
        );
    }
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(14),
        "实例 2 覆盖写到 txg 14"
    );
    pool
}

/// 抬 F 到 `new_floor`，走产品入口 `mount::raise_rollback_floor`：回收、影子账重算、带新 F 的空发布直到每块盘上都有一条，都是真的。
/// `floor_the_entry_is_told_is_current` 是交给入口的「现行版本」里记的 F——入口按「txg ≥ 今天的 F」取有效根算上限，
/// 把它先改成要抬到的那个值，入口算出来的上限就恒不低于它，超过真上限的目标也放得过去。盘上留下的就是步 5 验收那条变异
/// （F 抬过上限）的样子：记账、回收、分配记录与一次合法的抬 F 一样自洽，只有 F 本身抬过了头；盘上 txg 14 那条根带的还是 0，
/// checker 按它认「抬之前的 F」。只改根记录里那 8 个字节造不出它：候选集跟着 F 缩小，txg 11 那条根独占的单元掉出遍历，I-3.1 跟着红。
fn raise_the_floor(
    pool: &mut BuiltPool,
    new_floor: CheckpointTxg,
    floor_the_entry_is_told_is_current: CheckpointTxg,
) {
    let mut current = pool.output.clone();
    current.root.rollback_floor = floor_the_entry_is_told_is_current;
    let devices = pool.devices.as_mut().expect("镜像还开着");
    raise_rollback_floor(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        new_floor,
        ShadowLedger::On,
    )
    .expect("抬 F");
    pool.output = current;
}

/// 抬 F 到 11 之后再发两版（txg 17、18）：E0 的数据单元落回 mkfs 种的实例表那两槽，E 的数据单元落回 mkfs 种的第 0 版树表那一槽
/// （都只被 txg 11 之下的根引用，抬 F 生效之后合法回收）——第 0 代根那几条的实例表与树表从此读不出。
fn overwrite_and_advance_onto_the_genesis_tree_table_slot(pool: &mut BuiltPool) {
    let data_slot = |current: &BuiltPool| {
        current
            .output
            .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
            .slot
            .0
    };
    overwrite_and_advance(pool, &content_seeded(1200, 3), InstanceGeneration(2));
    assert_eq!(
        data_slot(pool),
        INSTANCE_TABLE_SLOT.0,
        "E0 的数据单元落回 mkfs 种的实例表那两槽"
    );
    overwrite_and_advance(pool, &content_seeded(1234, 5), InstanceGeneration(2));
    assert_eq!(
        data_slot(pool),
        GENESIS_TREE_TABLE,
        "E 的数据单元落回 mkfs 种的第 0 版树表那一槽"
    );
}

/// I-7.9（回退下界 F 不高于抬 F 的上限） 的坏镜像：步 5 那段历史上把 F 抬到 12，而上限是 11。
/// 不是改一处字节改得出来的（见 `raise_the_floor`），所以这一格的「改法」是整份换成那段历史的镜像，入参那份不用。
fn known_bad_images_of_the_rollback_floor() -> Vec<(&'static str, Mutation)> {
    vec![(
        "I-7.9",
        Box::new(|image: &mut MemoryPool| {
            let mut pool = pool_before_raising_the_floor("known-bad-floor-above-its-ceiling");
            raise_the_floor(&mut pool, CheckpointTxg(12), CheckpointTxg(12));
            *image = pool.memory_pool();
        }),
    )]
}

/// I-7.9（回退下界 F 不高于抬 F 的上限）：同一段历史（步 5 的脚本，上限 11）上三份镜像——
/// ① **抬到上限 11**（推 txg 15、16 两条带 F 11 的空发布；抬 F 的根是 15，它同实例的前一条 14 带 F 0）：I-7.9 真被评估过且成立，别的一条不红；
/// ② **① 之后再发两版 E0、E**（`overwrite_and_advance_onto_the_genesis_tree_table_slot`）：E 的数据单元落回 mkfs 种的第 0 版树表那一槽
///    （抬 F 生效之后它被合法回收），第 0 代根那几条的树表从此读不出——
///    它们在「抬之前的 F」（0）之上，是算上限的有效根，空不空判不了；把它们都算非空，第 4 新的非空根还是 11，上限的两沿都是 11，
///    I-7.9 照样真被评估过且成立（「不在后来每张镜像上重算」那一半，读不出的根不按空或非空猜）；
/// ③ **抬到 12**（上限 11，入口被骗过，见 `raise_the_floor`）⇒ **只**红 I-7.9，红在「从 0 抬到 12、上限 11」那一格。
#[test]
fn raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant() {
    let mut at_the_ceiling = pool_before_raising_the_floor("known-good-floor-at-its-ceiling");
    let floor_before_the_raise = at_the_ceiling.output.root.rollback_floor;
    assert_eq!(floor_before_the_raise, CheckpointTxg(0), "抬之前 F 是 0");
    raise_the_floor(
        &mut at_the_ceiling,
        CheckpointTxg(11),
        floor_before_the_raise,
    );
    assert_eq!(
        at_the_ceiling.output.root.rollback_floor,
        CheckpointTxg(11),
        "最新根带 F = 11"
    );
    let image_at_the_ceiling = at_the_ceiling.memory_pool();
    let verdicts_at_the_ceiling = check_pool_image(&image_at_the_ceiling);
    assert_eq!(
        violated_invariants(&verdicts_at_the_ceiling),
        Vec::<&str>::new(),
        "抬到上限 11：一条都不许红：{verdicts_at_the_ceiling:?}"
    );
    assert_eq!(
        verdict(&image_at_the_ceiling, "I-7.9"),
        InvariantVerdict::Holds,
        "抬到上限 11：I-7.9 要真被评估过且成立"
    );
    overwrite_and_advance_onto_the_genesis_tree_table_slot(&mut at_the_ceiling);
    let image_after_reuse = at_the_ceiling.memory_pool();
    let verdicts_after_reuse = check_pool_image(&image_after_reuse);
    assert_eq!(
        violated_invariants(&verdicts_after_reuse),
        Vec::<&str>::new(),
        "抬到 11 之后再发 E：一条都不许红：{verdicts_after_reuse:?}"
    );
    assert_eq!(
        verdict(&image_after_reuse, "I-7.9"),
        InvariantVerdict::Holds,
        "抬之前的 F 之上有树表读不出的根，上限的两沿仍都是 11：I-7.9 要真被评估过且成立"
    );
    for (invariant, mutation) in known_bad_images_of_the_rollback_floor() {
        let mut image = image_at_the_ceiling.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [invariant],
            "F 抬到 12（上限 11）：判红的该只有 {invariant}：{verdicts:?}"
        );
        let InvariantVerdict::Violated(detail) = verdict(&image, invariant) else {
            panic!("上面判过 {invariant} 红");
        };
        assert!(
            detail.contains("实例 2 txg 15 那条根把回退下界 F 从 0 抬到 12，高于它之前的根算出的抬 F 上限 11")
                && detail.contains("非空有效根 [3, 4, 8, 9, 10, 11, 12, 13, 14]"),
            "红在「抬 F 的根 txg 15、从 0 抬到 12、非空有效根 3、4、8–14、上限 11」那一格：{detail}"
        );
    }
}

/// 系统配置槽里回退下界 F 那 8 字节的偏移：字段表最后一行（D22（单元原子性怎么合成） 已定项 9）。
fn system_configuration_rollback_floor_offset() -> usize {
    usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_BYTES - 8).expect("字段表里的偏移")
}

/// 把 `device` 那块盘两个系统配置槽里的 F 都改成 `rollback_floor`（整槽重封，别的字节不动）。
fn set_the_system_configuration_floor_on_one_device(
    image: &mut MemoryPool,
    device: u32,
    rollback_floor: u64,
) {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    for slot_offset in [0, spacing] {
        mutate_system_configuration_slot(image, device, slot_offset, |bytes| {
            set_u64(
                bytes,
                system_configuration_rollback_floor_offset(),
                rollback_floor,
            );
        });
    }
}

/// checkpoint_txg 为 `txg` 的那条根住的根槽：(盘, 偏移)（与写者同一条根环公式）。
fn root_slot_of(txg: CheckpointTxg) -> (u32, u64) {
    let target = singlefs_core::root_ring::target_for_publish(
        txg,
        parameters().geometry.root_ring_slots_per_region,
    );
    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let offset = singlefs_core::root_ring::slot_offset(
        target,
        parameters().geometry.fixed_structure_slot_spacing,
    );
    (device.0, offset.0)
}

/// 把 checkpoint_txg 为 `txg` 的那条根带的 F（根记录偏移 130）改成 `rollback_floor`，自证校验和重封。
fn set_the_root_floor(image: &mut MemoryPool, txg: CheckpointTxg, rollback_floor: u64) {
    let (device, offset) = root_slot_of(txg);
    let root_slot_bytes = usize::try_from(parameters().geometry.physical_block_size).expect("512");
    let mut bytes = read(image, device, offset, root_slot_bytes);
    set_u64(&mut bytes, 130, rollback_floor);
    let checksum_offset = singlefs_core::root_record::ROOT_CHECKSUM_OFFSET;
    let digest = wide_checksum_with_field_zeroed(&bytes, root_slot_bytes, checksum_offset);
    bytes[checksum_offset..checksum_offset + 32].copy_from_slice(&digest);
    write(image, device, offset, &bytes);
    copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
}

/// 把 checkpoint_txg 为 `txg` 的那条根所在的根槽改坏一个字节（自证校验和罩整槽）：那条根读不出。
fn make_the_root_unreadable(image: &mut MemoryPool, txg: CheckpointTxg) {
    let (device, offset) = root_slot_of(txg);
    let root_slot_bytes = usize::try_from(parameters().geometry.physical_block_size).expect("512");
    let mut bytes = read(image, device, offset, root_slot_bytes);
    bytes[100] ^= 0xff;
    write(image, device, offset, &bytes);
}

/// I-7.12（系统配置 F 不低于同盘根上的 F） 的坏镜像：步 5 那段历史上抬到上限 11（先把 11 写进两盘的系统配置，再推 txg 15 落盘 0、
/// 16 落盘 1），然后把盘 1 两个系统配置槽里的 F 都改回 0——就是「先写系统配置」挪到第一条根之后、崩在两者之间的那一格：
/// 盘 1 上 txg 16 那条根带 11，盘 1 的系统配置里读得出的最大 F 是 0。F 生效值照旧是 11（最新根与盘 0 的系统配置都带 11）。
/// 不是改一处字节就走得到抬 F 之后的历史，所以这一格的「改法」是整份换成那段历史的镜像再改，入参那份不用。
fn known_bad_images_of_the_system_configuration_floor() -> Vec<(&'static str, Mutation)> {
    vec![(
        "I-7.12",
        Box::new(|image: &mut MemoryPool| {
            let mut pool = pool_before_raising_the_floor("known-bad-system-configuration-floor");
            raise_the_floor(&mut pool, CheckpointTxg(11), CheckpointTxg(0));
            let mut raised = pool.memory_pool();
            set_the_system_configuration_floor_on_one_device(&mut raised, 1, 0);
            *image = raised;
        }),
    )]
}

/// I-7.12（系统配置 F 不低于同盘根上的 F）：步 5 那段历史上抬到上限 11 的镜像一条都不红、I-7.12 真被评估过且成立（每块盘的系统配置里
/// 都有先写进去的 11）；[`known_bad_images_of_the_system_configuration_floor`] 那份（盘 1 两槽的 F 改回 0）**只**红 I-7.12，红在盘 1、txg 16 那一格。
#[test]
fn one_device_whose_system_configuration_floor_is_below_a_root_on_it_reddens_only_the_system_configuration_floor_invariant(
) {
    let mut at_the_ceiling = pool_before_raising_the_floor("known-good-system-configuration-floor");
    raise_the_floor(&mut at_the_ceiling, CheckpointTxg(11), CheckpointTxg(0));
    let clean = at_the_ceiling.memory_pool();
    let clean_verdicts = check_pool_image(&clean);
    assert_eq!(
        violated_invariants(&clean_verdicts),
        Vec::<&str>::new(),
        "抬到上限 11：一条都不许红：{clean_verdicts:?}"
    );
    assert_eq!(
        verdict(&clean, "I-7.12"),
        InvariantVerdict::Holds,
        "抬到上限 11：I-7.12 要真被评估过且成立"
    );
    for (invariant, mutation) in known_bad_images_of_the_system_configuration_floor() {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [invariant],
            "盘 1 两槽的 F 改回 0：判红的该只有 {invariant}：{verdicts:?}"
        );
        let InvariantVerdict::Violated(detail) = verdict(&image, invariant) else {
            panic!("上面判过 {invariant} 红");
        };
        assert!(
            detail.contains("盘 1：两槽里自证过的系统配置带的 F 最大 0")
                && detail.contains("txg 16 那条根带的 F 11"),
            "红在盘 1、txg 16 那条带 11 的根那一格：{detail}"
        );
    }
}

/// 正常卸载那段历史：新池新建文件之后同一个实例里覆盖写三次（txg 4–6），卸载把 F 抬到 6，推 txg 7（落盘 1）、8（落盘 0）两条带卸载记号的根。
/// 这段历史上准入抬 F 的上限是 3（第 4 新的非空根是 A 的 3）。
fn image_after_the_normal_unmount(tag: &str) -> MemoryPool {
    let mut pool = build_pool(tag);
    for seed in [3usize, 5, 7] {
        overwrite_and_advance(
            &mut pool,
            &content_seeded(2600 + seed, seed),
            InstanceGeneration(1),
        );
    }
    let mut current = PoolVersion::WithFile(pool.output.clone());
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let Unmounted::FloorRaisedToTheCurrentVersion(unmounted) = unmount(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        ShadowLedger::On,
    )
    .expect("正常卸载") else {
        panic!("现行那一版带文件：卸载抬 F");
    };
    assert_eq!(unmounted.rollback_floor, CheckpointTxg(6));
    pool.memory_pool()
}

/// I-7.9（回退下界 F 不高于抬 F 的上限） 带卸载记号那一支的坏镜像：卸载之后把 txg 7 那条带记号的根（抬 F 的那一条，盘 1）带的 F 改成 7，
/// 高于根环里 txg 比它小的最新那条根的 txg 6；盘 1 两个系统配置槽的 F 一起改成 7，I-7.12 不先红。
fn known_bad_images_of_the_unmount_floor() -> Vec<(&'static str, Mutation)> {
    vec![(
        "I-7.9",
        Box::new(|image: &mut MemoryPool| {
            *image =
                image_after_the_normal_unmount("known-bad-unmount-floor-above-the-previous-root");
            set_the_root_floor(image, CheckpointTxg(7), 7);
            set_the_system_configuration_floor_on_one_device(image, 1, 7);
        }),
    )]
}

/// I-7.9（回退下界 F 不高于抬 F 的上限） 按卸载记号分两支（D22（单元原子性怎么合成） 已定项 7）：
/// ① 正常卸载之后的镜像（txg 7 那条带记号的根把 F 从 0 抬到 6）一条都不红，I-7.9 真被评估过且成立——按带记号那一支判 6 ≤ 6；
///    按准入那一支判它就红了（这段历史上准入上限是 3）；
/// ② [`known_bad_images_of_the_unmount_floor`] 那份（F 改成 7）**只**红 I-7.9，红在「带卸载记号的根从 0 抬到 7、前一条根 txg 6」那一格。
/// 不带记号那一支的坏镜像是 [`known_bad_images_of_the_rollback_floor`]（准入入口抬到 12、上限 11）。
#[test]
fn marked_root_raising_the_floor_above_the_previous_root_reddens_only_the_rollback_floor_invariant()
{
    let clean = image_after_the_normal_unmount("known-good-unmount-floor");
    let clean_verdicts = check_pool_image(&clean);
    assert_eq!(
        violated_invariants(&clean_verdicts),
        Vec::<&str>::new(),
        "正常卸载之后：一条都不许红：{clean_verdicts:?}"
    );
    assert_eq!(
        verdict(&clean, "I-7.9"),
        InvariantVerdict::Holds,
        "正常卸载之后：I-7.9 按带记号那一支真被评估过且成立"
    );
    for (invariant, mutation) in known_bad_images_of_the_unmount_floor() {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [invariant],
            "带记号的根把 F 抬到 7（前一条根 txg 6）：判红的该只有 {invariant}：{verdicts:?}"
        );
        let InvariantVerdict::Violated(detail) = verdict(&image, invariant) else {
            panic!("上面判过 {invariant} 红");
        };
        assert!(
            detail.contains(
                "实例 1 txg 7 那条带卸载记号的根把回退下界 F 从 0 抬到 7，高于根环里 txg 比它小的最新那条根的 txg 6"
            ),
            "红在带记号那一支：{detail}"
        );
    }
}

/// C556（checker 与层 0 不读系统配置里的 F）：步 5 那段历史上抬到 11、再发两版 E0、E（txg 17、18，E 的数据单元落回 mkfs 种的第 0 版树表那一槽），
/// 然后把四条带 F 11 的根（txg 15–18）都改坏——根环里读得出的根带的 F 全是 0，两块盘的系统配置里还是 11（抬 F 先写进去的）。
/// checker 的回退候选集下界取 F 生效值 max(最新根带的, 系统配置里的) = 11：被复用的那一槽只有 txg 11 之下的根引用，它们不在候选集里，
/// 一条都不红；I-3.1 报不适用（最新根 txg 14 带 F 0，它的记账按 0 记，与按 11 取的候选集并集不可比）。
/// 判别力自证：同一张镜像再把四个系统配置槽的 F 都改回 0（等于只读根上的 F），txg 11 之下的根回到候选集、走到被复用的那一槽，
/// 按回退候选集判的那几条红。
#[test]
fn every_root_carrying_the_raised_floor_unreadable_keeps_the_candidate_invariants_green_because_the_checker_reads_the_system_configuration_floor(
) {
    let mut pool = pool_before_raising_the_floor("c556-roots-carrying-the-floor-unreadable");
    raise_the_floor(&mut pool, CheckpointTxg(11), CheckpointTxg(0));
    overwrite_and_advance_onto_the_genesis_tree_table_slot(&mut pool);
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(18));
    let mut image = pool.memory_pool();
    for txg in [15, 16, 17, 18] {
        make_the_root_unreadable(&mut image, CheckpointTxg(txg));
    }
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "带 F 11 的根全读不出、系统配置里的 11 还在：一条都不许红：{verdicts:?}"
    );
    assert!(
        matches!(verdict(&image, "I-3.1"), InvariantVerdict::NotApplicable(_)),
        "最新根 txg 14 带 F 0、F 生效值 11：I-3.1 报不适用"
    );
    let mut floor_only_on_the_roots = image.clone();
    for device in DEVICES {
        set_the_system_configuration_floor_on_one_device(&mut floor_only_on_the_roots, device, 0);
    }
    let verdicts_reading_only_the_roots = check_pool_image(&floor_only_on_the_roots);
    assert_eq!(
        violated_invariants(&verdicts_reading_only_the_roots),
        ["I-2.1", "I-3.10", "I-4.8", "I-7.4"],
        "系统配置里的 F 也改回 0（等于只读根上的 F）：txg 11 之下的根回到候选集、走到被复用的那两处（实例表那两槽的分配记录分配代 0、\
         单元头诞生代号 17，I-3.10 也红）：{verdicts_reading_only_the_roots:?}"
    );
}

/// 残留记录那一版的内容：与 A、B 都不同长、不同字节（具体哪些字节不承重）。
fn content_applied_only_by_its_journal_record() -> Vec<u8> {
    (0..3700usize)
        .map(|index| u8::try_from((index * 7 + 5) % 253).expect("小于 256"))
        .collect()
}

/// 一次可写挂载接在「由记录施加出来、根槽从没落盘」的那一版之后的镜像（增补 2 收口表第 54 行那段历史，
/// 与 `crash_enumeration_fixed_script_stream.rs` 残留记录那条流里实例 2 的根已落盘的那 12 个状态同形）：
/// mkfs → 取号 → 暖机 → A → B，实例 1 再发一版（txg 5）而**只落了它的单元与 journal 记录、根槽与系统配置槽一个都没落**；
/// 可写挂载择 B 的根 (1, 4)、施加那条记录、给实例 1 写行 (1, 5, 3)、写行发布 txg 6 把 (1, 5) 那一版的固定点单元换下放进 defer、
/// 暖机一次 txg 7。交回挂载前后两份镜像、挂载之后接着发布要用的盘与分配器，以及几个单元的槽（两盘同槽）。
struct ImageAfterAVersionAppliedOnlyByItsJournalRecord {
    /// 挂载之前：那一版的单元与记录已在盘上，最新的根还是 B 的 (1, 4)，实例表里没有实例 1 的行。
    image_before_the_mount: MemoryPool,
    /// 挂载之后（最新的根是暖机 txg 7）。
    image: MemoryPool,
    devices: Vec<(
        DeviceIdentity,
        singlefs_harness::RecordingBlockDevice<singlefs_harness::memory_pool::SparseBlockDevice>,
    )>,
    allocator: singlefs_core::allocator::PoolAllocator,
    newest: singlefs_core::transaction::TransactionOutput,
    newest_accounting_root_slot: u64,
    /// 只由记录施加出来的那一版 (1, 5) 自己的树表单元：写行发布 txg 6 把它换下放进 defer，根环里没有一条根引用它。
    tree_table_slot_of_the_version_applied_only_by_its_record: u64,
}

fn image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
    tag: &str,
) -> ImageAfterAVersionAppliedOnlyByItsJournalRecord {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let second_content: Vec<u8> = (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect();
    let second = publish_overwrite_in_process(
        &mut pool,
        &first,
        &second_content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写 B");
    let operations_through_the_second_publish = pool.retained_operations().len();
    let applied_only_by_its_record = publish_overwrite_in_process(
        &mut pool,
        &second,
        &content_applied_only_by_its_journal_record(),
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(1),
    )
    .expect("实例 1 再发一版");
    assert_eq!(
        (
            applied_only_by_its_record.record.instance,
            applied_only_by_its_record.record.checkpoint_txg,
            applied_only_by_its_record.record.transaction
        ),
        (InstanceGeneration(1), CheckpointTxg(5), 3),
        "只由记录施加出来的那一版：实例 1、txg 5、事务号 3"
    );
    let operations = pool.retained_operations();
    let classifier = geometry();
    let units_and_record_of_the_version: Vec<RetainedOperation> = operations
        [operations_through_the_second_publish..]
        .iter()
        .filter(|retained| match classifier.classify(&retained.operation) {
            StepKind::UnitWrite | StepKind::JournalRecord => true,
            StepKind::ZeroFill
            | StepKind::RootRecordFua
            | StepKind::SystemConfigurationSlot
            | StepKind::Barrier => false,
        })
        .cloned()
        .collect();
    let mut image_before_the_mount = pool.memory_pool_after_mkfs();
    image_before_the_mount
        .apply(&operations[pool.mkfs_operation_count..operations_through_the_second_publish]);
    image_before_the_mount.apply(&units_and_record_of_the_version);
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&image_before_the_mount, &[], &[], &stream);
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    assert_eq!(
        (
            mounted.output.chosen_root.instance,
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.journal.prefix_applied
        ),
        (InstanceGeneration(1), CheckpointTxg(4), CheckpointTxg(5), 1),
        "择 B 的根 (1, 4)，施加那条记录，现行版是 (1, 5)"
    );
    assert_eq!(
        mounted.output.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(5),
            applied_transaction_high_water: 3,
        }],
        "写行 (1, 5, 3)"
    );
    assert_eq!(
        mounted.output.row_publish.root().checkpoint_txg,
        CheckpointTxg(6),
        "写行发布 txg 6"
    );
    let newest = mounted
        .current
        .into_file_version()
        .expect("施加了那条记录，现行那一版带文件");
    ImageAfterAVersionAppliedOnlyByItsJournalRecord {
        image_before_the_mount,
        image: memory_pool_of_sparse_devices(&devices),
        devices,
        allocator: mounted.allocator,
        newest_accounting_root_slot: newest.unit(TransactionUnit::AccountingTree).slot.0,
        newest,
        tree_table_slot_of_the_version_applied_only_by_its_record: applied_only_by_its_record
            .unit(TransactionUnit::TreeTable)
            .slot
            .0,
    }
}

/// 单元区起点（第一个单元落在这个槽上：mkfs 种的实例表）。
const UNIT_AREA_FIRST_SLOT: u64 = 50176;
/// `units_found_on_device_zero` 从单元区起点往后扫多少个槽：这几份镜像上用到的落点都在前几百个槽里。
const SLOTS_SCANNED_FOR_UNITS: u64 = 1024;

/// 单元区里盘 0 上此刻的单元：(起点槽, 字节数)，按头里的类标签认（码 2 一槽，码 1 / 码 3 两槽）。
/// 镜像是现造出来的、没有写死的单元表时，拿它给沿引用链补校验和（`mutate_unit_of`）当单元表：
/// 父单元（树表、中央映射树的根）里引用被改单元的位置条目都要补上，漏一个就是 I-2.1 跟着红。
fn units_found_on_device_zero(image: &MemoryPool) -> Vec<(u64, usize)> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let mut units = Vec::new();
    let mut slot = UNIT_AREA_FIRST_SLOT;
    while slot < UNIT_AREA_FIRST_SLOT + SLOTS_SCANNED_FOR_UNITS {
        let header = read(image, 0, slot * SLOT, 512);
        let slots_taken = if &header[..4] == b"SFSU" {
            match header[6] {
                2 => {
                    units.push((slot, node_bytes));
                    1
                }
                1 | 3 => {
                    units.push((slot, 2 * node_bytes));
                    2
                }
                _unregistered_unit_class => 1,
            }
        } else {
            1
        };
        slot += slots_taken;
    }
    units
}

/// 增补 2 收口表第 54 行（2026-09-23 用户定候选 b）的两半，建在同一份镜像上：
/// ① **误报消失**：可写挂载接在「由记录施加出来、根槽从没落盘」的那一版之后，那一版被换下的四个固定点单元在 defer 里、仍算已分配，
///    根环里没有一条根引用它们；checker 把那一版并进遍历之后 I-3.1（已分配统计对得上） 真被评估过且成立，别的也一条不红。
///    候选集不补这一版（`versions_applied_only_by_records` 交回空），这一段就在 I-3.1 上红、差 65 536 字节（每盘 4 槽）。
/// ② **改完仍会红**：同一份镜像上记账盘 0 再泄漏一槽（「已分配」+16384、「空闲」−16384），I-3.1 照样红
///    （I-3.11（已分配减 defer 等于最新根走读） 跟着红：defer 行没动），别的不红，
///    说明文字里的机理标识报出那一版确实并进了遍历——补上的只是等式右边缺的那一版，没把泄漏一起吞掉。
#[test]
fn version_applied_only_by_its_journal_record_is_walked_so_the_allocated_statistic_holds_and_the_leak_on_top_still_reddens_it(
) {
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-good-version-applied-only-by-its-record",
    );
    let verdicts_on_the_clean_image = check_pool_image(&built.image);
    assert_eq!(
        violated_invariants(&verdicts_on_the_clean_image),
        Vec::<&str>::new(),
        "接在只由记录施加出来的那一版之后的合法镜像上一条都不许红：{verdicts_on_the_clean_image:?}"
    );
    assert_eq!(
        verdict(&built.image, "I-3.1"),
        InvariantVerdict::Holds,
        "I-3.1 要真被评估过且成立"
    );
    let mut leaked = built.image.clone();
    mutate_unit_of(
        &mut leaked,
        &units_found_on_device_zero(&built.image),
        built.newest_accounting_root_slot,
        |bytes| leak_one_slot_keeping_free_plus_allocated(bytes, false),
        true,
    );
    let verdicts = check_pool_image(&leaked);
    // defer 行没跟着抬：已分配减 defer 比最新根走读多一槽，I-3.11（已分配减 defer 等于最新根走读） 跟着红（同 Z3-A）。
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-3.1", "I-3.11"],
        "再泄漏一槽只该红 I-3.1 与 I-3.11：{verdicts:?}"
    );
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &verdicts_on_the_clean_image,
            &["I-3.1", "I-3.11"]
        ),
        [],
        "I-3.1 与 I-3.11 之外每一条的判定与泄漏之前逐项相同"
    );
    let InvariantVerdict::Violated(detail) = verdict(&leaked, "I-3.1") else {
        panic!("上面判过 I-3.1 红");
    };
    assert!(
        detail.contains("并进遍历的由记录施加出来的版本 1 个"),
        "机理标识要报出那一版并进了遍历：{detail}"
    );
}

/// 候选 b 那一版的第 ① 条（「它被施加过」要看实例表里有没有这个实例的行）：同一段历史**挂载之前**那一刻，
/// 那一版的单元与记录已在盘上、最新的根还是 B 的 (1, 4)，实例表里没有实例 1 的行——施加要等下一次挂载，这一刻它不算一版。
/// 这一刻它的单元是孤儿、不在记账里；把它并进遍历就是遍历多算，I-3.1（已分配统计对得上） 红
/// （`crash_enumeration_fixed_script_stream.rs` 残留记录那条流里「链接得到残留记录」的 19 个状态就是这一格）。
#[test]
fn journal_record_that_no_mount_has_applied_yet_is_not_walked_and_every_invariant_holds() {
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-good-record-not-applied-yet",
    );
    let verdicts = check_pool_image(&built.image_before_the_mount);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "挂载之前那一刻一条都不许红：{verdicts:?}"
    );
    assert_eq!(
        verdict(&built.image_before_the_mount, "I-3.1"),
        InvariantVerdict::Holds,
        "I-3.1 要真被评估过且成立"
    );
}

/// 只由记录施加出来的那一版在候选集里：它引用的单元照候选根一样判，I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽）
/// 按这一版各判一格。把它自己的树表单元（只有它引用，在 defer 里）两份都抹成 0：从它出发的遍历读不到那个单元，
/// 这两条必须红——根环里的根一条都不引用它，按根判的那几格判不出来。
#[test]
fn erasing_a_unit_only_the_version_applied_by_its_journal_record_references_reddens_the_reuse_invariants(
) {
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-bad-version-applied-only-by-its-record-erased",
    );
    let mut image = built.image.clone();
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    for device in DEVICES {
        write(
            &mut image,
            device,
            built.tree_table_slot_of_the_version_applied_only_by_its_record * SLOT,
            &vec![0u8; node_bytes],
        );
    }
    let verdicts = check_pool_image(&image);
    let violated = violated_invariants(&verdicts);
    for invariant in ["I-7.4", "I-4.8"] {
        assert!(
            violated.contains(&invariant),
            "那一版的树表被抹掉，{invariant} 要红：{verdicts:?}"
        );
    }
    assert!(
        !violated.contains(&"I-7.2"),
        "最新的根不引用那个单元，I-7.2（最新根能完整走完） 不该跟着红：{verdicts:?}"
    );
}

/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 射程 ③：记录罩住的起点槽上读不出可用的单元头时这一格没有对象。
/// 发布 B 之后的干净镜像上把 B 的数据单元两份头里的诞生代号改成 99、**不重封**头校验和（盘上坏了的样子）：
/// 那条记录不判，I-3.10 照旧由别的记录判成成立；读不出本身由 I-2.1 / I-2.4 说话。
/// 把「头校验和过」那一道从「可用」里拿掉，这里就按坏头里的 99 判红。
#[test]
fn an_allocation_record_whose_start_slot_header_does_not_verify_is_not_judged_by_the_allocation_generation_invariant(
) {
    let mut image = image_after_the_overwrite("known-good-allocation-generation-unusable-header");
    let data_unit_bytes = 2 * usize::try_from(NODE_BYTES).expect("16384");
    for device in DEVICES {
        let mut unit = read(
            &image,
            device,
            DATA_UNIT_AFTER_OVERWRITE * SLOT,
            data_unit_bytes,
        );
        assert_eq!(
            get_u64(&unit, DATA_UNIT_BIRTH_TXG_OFFSET),
            4,
            "B 的数据单元诞生代号本来是 4"
        );
        set_u64(&mut unit, DATA_UNIT_BIRTH_TXG_OFFSET, 99);
        write(&mut image, device, DATA_UNIT_AFTER_OVERWRITE * SLOT, &unit);
    }
    let verdicts = check_pool_image(&image);
    assert_eq!(
        verdict(&image, "I-3.10"),
        InvariantVerdict::Holds,
        "起点槽的头校验和不过的那条记录不判，别的未释放记录照判且成立：{verdicts:?}"
    );
    assert!(
        violated_invariants(&verdicts).contains(&"I-2.1"),
        "单元坏了由 I-2.1 说话：{verdicts:?}"
    );
}

/// 抬 F 之前实例 2 接着发几版：发到 txg 10，「第 4 新的非空持久有效根」才是 txg 6，F 抬得到 6（上限的算法见
/// `crates/singlefs-core/src/mount.rs` 的 `raise_rollback_floor`）。
const OVERWRITES_BEFORE_RAISING_THE_FLOOR_PAST_THE_VERSION: u64 = 3;

/// 实例 2 在同一次挂载里接着覆盖写 `rounds` 次，交回最后那一版。
fn overwrite_in_the_second_instance(
    built: &mut ImageAfterAVersionAppliedOnlyByItsJournalRecord,
    rounds: u64,
    write_time_offset_seconds: u64,
) -> singlefs_core::transaction::TransactionOutput {
    let publish_parameters = parameters();
    let mut previous = built.newest.clone();
    for round in 0..rounds {
        let round_index = usize::try_from(round).expect("轮次");
        let content: Vec<u8> = (0..(2000 + round_index * 37))
            .map(|index| u8::try_from((index * 11 + round_index) % 251).expect("小于 256"))
            .collect();
        let mut writer = PoolWriter::new(&publish_parameters, built.devices.as_mut_slice());
        previous = publish_overwrite(
            &mut writer,
            &mut built.allocator,
            &previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + write_time_offset_seconds + round,
            },
            InstanceGeneration(2),
        )
        .expect("实例 2 接着覆盖写");
    }
    previous
}

/// 候选 b 那一版的第 ③ 条（「不低于回退下界」）：F 抬过 (1, 5) 之后它就不再并进遍历。
/// 同一段历史在实例 2 里接着发三版（txg 8–10），再把 F 抬到 6：抬 F 生效之后回收释放代 ≤ 6 的落点，
/// (1, 5) 那一版被写行发布 txg 6 换下的单元（释放代 6）就在其中——不再算已分配。把它照旧并进遍历就是遍历多算，I-3.1 红。
/// 环里比它老的有效根还在（txg 1–4），第 ④ 条拦不住这一格，拦它的只有第 ③ 条。
#[test]
fn version_applied_only_by_its_journal_record_leaves_the_walk_once_the_rollback_floor_is_raised_past_it(
) {
    let mut built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-good-version-applied-only-by-its-record-below-the-floor",
    );
    let mut current = overwrite_in_the_second_instance(
        &mut built,
        OVERWRITES_BEFORE_RAISING_THE_FLOOR_PAST_THE_VERSION,
        180,
    );
    assert_eq!(
        current.root.checkpoint_txg,
        CheckpointTxg(10),
        "实例 2 发到 txg 10"
    );
    let raised = singlefs_core::mount::raise_rollback_floor(
        &parameters(),
        &mut built.devices,
        &mut built.allocator,
        &mut current,
        CheckpointTxg(6),
        ShadowLedger::On,
    )
    .expect("抬 F 到 6");
    assert!(
        raised.reclaimed.iter().any(|placement| placement.slot.0
            == built.tree_table_slot_of_the_version_applied_only_by_its_record),
        "(1, 5) 那一版的树表单元（释放代 6）在抬 F 生效之后被回收：{:?}",
        raised.reclaimed
    );
    let image = memory_pool_of_sparse_devices(&built.devices);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "F 抬过 (1, 5) 之后一条都不许红：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-3.1"),
        InvariantVerdict::Holds,
        "I-3.1 要真被评估过且成立"
    );
}

/// 由记录施加出来的那一版照它施加在其上那一版的根走实例表（D23（journal 的角色与格式） 已定项 15「实例表单元指针照所选根」，
/// 主 agent 2026-09-27 定、零轮）：同一段历史在实例 2 里接着发三版（txg 8–10），再把 F 抬到 5——它施加在其上的 B (1, 4) 掉到 F 之下、
/// 不在回退候选集里，(1, 5) 自己还在（txg 5 ≥ F）。实例 1 那张实例表（mkfs 写的第 0 片，槽 50176）只有实例 1 的根与 (1, 5) 这一版引用，
/// 写行发布 txg 6 换下它、释放代 6 > max(F 5, 环里最旧有效根 1)，抬 F 之后仍算已分配。checker 走 (1, 5) 时照 B 的根记录走它的实例表，
/// I-3.1（已分配统计对得上） 对得上、一条都不红。改之前只走记录新根段里的树表与映射根，那一片 32768 字节没人数，I-3.1 红
/// （崩溃注入快档种子 7463871032432355136 第 91 段那两条同形，`research/prompts/m2-investigate-crash-injection-reds-report.md` 第三节）。
#[test]
fn version_applied_only_by_its_journal_record_walks_the_instance_table_of_the_root_it_was_applied_on_even_below_the_floor(
) {
    let mut built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-good-version-applied-only-by-its-record-base-below-the-floor",
    );
    let mut current = overwrite_in_the_second_instance(
        &mut built,
        OVERWRITES_BEFORE_RAISING_THE_FLOOR_PAST_THE_VERSION,
        180,
    );
    assert_eq!(
        current.root.checkpoint_txg,
        CheckpointTxg(10),
        "实例 2 发到 txg 10"
    );
    singlefs_core::mount::raise_rollback_floor(
        &parameters(),
        &mut built.devices,
        &mut built.allocator,
        &mut current,
        CheckpointTxg(5),
        ShadowLedger::On,
    )
    .expect("抬 F 到 5");
    let image = memory_pool_of_sparse_devices(&built.devices);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "B 掉到 F 之下、(1, 5) 还在：一条都不许红：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-3.1"),
        InvariantVerdict::Holds,
        "I-3.1 要真被评估过且成立"
    );
}

/// journal 环里计数器 `counter` 那条记录（两块盘各一份）的反向链字段改掉（最低位翻过来）、重封头校验和：
/// 写者写错了链，两盘写出同一份错的记录。它的载荷（点名项）一个字节不动。
fn break_the_back_chain_of_journal_record(pool: &mut MemoryPool, counter: u64) {
    let offset =
        singlefs_core::journal::record_offset(counter, singlefs_format::JOURNAL_RING_DEFAULT_BYTES)
            .0;
    for device in DEVICES {
        let mut record = read(pool, device, offset, 4096);
        record[JOURNAL_RECORD_BACK_CHAIN_OFFSET] ^= 1;
        let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
        record[46..78].copy_from_slice(&digest);
        write(pool, device, offset, &record);
    }
}

/// I-8.6（反向链算法）「不等的记录不进重放前缀」，checker 的前缀口径跟着恢复对齐（代码审阅 6c 第 25 条，用户 2026-09-27 定恢复补反向链检查；
/// 实审 A2a 报告「checker 要跟着改什么」）：可写挂载施加过 (1, 5) 那条记录、写了行的那份镜像上，把那条记录两盘各一份的反向链改坏。
/// 恢复照 I-8.6 不施加它，checker 也不再把它当「被恢复施加过的那一版」并进遍历：它换下的固定点单元仍记在已分配里、没人走到，
/// I-3.1（已分配统计对得上） 红，机理标识报并进遍历的版本 0 个；I-8.6 在那条记录上红，别的不红。改之前 checker 认版本时不看链，照旧并进，I-3.1 成立。
#[test]
fn a_journal_record_whose_back_chain_does_not_match_is_not_walked_as_a_version_a_recovery_applied()
{
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-bad-version-applied-only-by-a-record-with-a-broken-back-chain",
    );
    let mut image = built.image.clone();
    let (counter, _, _) = journal_records_on_device(&image, 0)
        .into_iter()
        .find(|(_, instance, transaction)| *instance == 1 && *transaction == 3)
        .expect("环里有实例 1、事务号 3 的那条记录（只由记录施加出来的 (1, 5)）");
    break_the_back_chain_of_journal_record(&mut image, counter);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-3.1", "I-8.6"],
        "只该红 I-3.1 与 I-8.6：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-3.1") else {
        panic!("上面判过 I-3.1 红");
    };
    assert!(
        detail.contains("并进遍历的由记录施加出来的版本 0 个"),
        "机理标识要报出那一版没并进遍历：{detail}"
    );
}

/// 同一个前缀口径的另一处（实审 A2a 报告那一节第 3 条）：「下一次挂载会先施加的那一版」。发布 B 的记录已落、根槽没落的崩溃镜像上，
/// B 那棵分配记录树里 B 的数据单元的分配代改成诞生代号 + 1（这一步单红 I-3.10，见上一条用例），再把 B 那条记录的反向链改坏：
/// 下一次挂载的恢复照 I-8.6 不施加它，checker 不再读它那一版的分配记录——只红 I-8.6，I-3.10 由候选集里 A 那一版判成立。
/// 改之前 checker 取那一版时不看链，I-3.10 照红。
#[test]
fn a_record_whose_back_chain_does_not_match_is_not_the_version_the_next_mount_applies_first() {
    let mut image = crash_image_with_the_overwrite_record_but_not_its_root(
        "known-bad-next-mount-version-with-a-broken-back-chain",
    );
    raise_the_allocation_generation_of_the_overwrite_data_unit_past_its_birth(&mut image);
    let (counter, _, _) = journal_records_on_device(&image, 0)
        .into_iter()
        .find(|(_, instance, transaction)| *instance == 1 && *transaction == 2)
        .expect("环里有 B 那条记录（实例 1、事务号 2）");
    break_the_back_chain_of_journal_record(&mut image, counter);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-8.6"],
        "B 那条记录进不了前缀：只该红 I-8.6：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-3.10"),
        InvariantVerdict::Holds,
        "I-3.10 由候选集里 A 那一版判成立"
    );
}

/// 环转过 (1, 5) 那一版之前实例 2 在同一次挂载里接着发几版：从 txg 8 起到 txg 29 为止，环里每个槽都被 txg ≥ 6 的根盖过
/// （根环 24 槽：三个区域各 8 槽，槽位按 txg 轮换）。
const OVERWRITES_TURNING_THE_ROOT_RING_PAST_THE_VERSION: u64 = 22;

/// 候选 b 那一版的第 ④ 条（「它换下的单元还没到回收的时候」）：环转过去之后它就不再并进遍历。
/// 同一段历史在实例 2 里接着发 22 版（txg 8–29），环里最旧的有效根变成比 (1, 5) 新；再重开可写挂载（实例 3），
/// 重建分配器按「释放代 ≤ max(F_生效, 环里最旧有效根)」（D3（空间分配） 已定项 7）把那一版换下的单元（释放代 6）收回去，
/// 再发一版。这时那一版的单元不再算已分配、可能已被复用；把它照旧并进遍历就是拿已经不归它的槽去比。
///
/// 实例 2 在同一次挂载里转过根环时就按谓词回收（C518（一次挂载之内环转过一圈之后不回收） 修好之后），每一版的记账行按回收之后的数写，
/// 这一份镜像上一条都不红：那一版的单元已回收、不再算已分配，第 ④ 条让它不并进遍历，记账与遍历对得上。
/// 把第 ④ 条拿掉，那一版照旧并进遍历，遍历多出它那几个已回收的槽，I-3.1（已分配统计对得上） 红。
#[test]
fn version_applied_only_by_its_journal_record_leaves_the_walk_once_the_root_ring_has_turned_past_it_and_the_mount_reclaimed_its_units(
) {
    let mut built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-good-version-applied-only-by-its-record-after-the-ring-turned",
    );
    let publish_parameters = parameters();
    let previous = overwrite_in_the_second_instance(
        &mut built,
        OVERWRITES_TURNING_THE_ROOT_RING_PAST_THE_VERSION,
        180,
    );
    assert_eq!(
        previous.root.checkpoint_txg,
        CheckpointTxg(29),
        "实例 2 发到 txg 29，环里每个槽都被 txg ≥ 6 的根盖过"
    );
    let remounted = mount_writable(&publish_parameters, &mut built.devices).expect("重开可写挂载");
    let mut allocator = remounted.allocator;
    let remounted_current = remounted
        .current
        .into_file_version()
        .expect("重开之后现行那一版带文件");
    let mut writer = PoolWriter::new(&publish_parameters, built.devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        &mut allocator,
        &remounted_current,
        FirstFile {
            content: &content_applied_only_by_its_journal_record(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 600,
        },
        remounted.output.instance,
    )
    .expect("重开之后再发一版");
    let image = memory_pool_of_sparse_devices(&built.devices);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "环转过 (1, 5) 之后那一版不再并进遍历，一条都不许红：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-3.1"),
        InvariantVerdict::Holds,
        "I-3.1 要真被评估过且成立"
    );
}

/// 发布 B（txg 4）的记录已落盘、根槽没落盘的崩溃镜像：B 的十二个单元、两块盘上各一份的记录与记录之后那道屏障都在，
/// 崩在根槽 FUA 之前（Y4-a 那三个前缀里的最后一个，`research/prompts/m2-wave3-code-r1-main-verification.md` 第三节 Y4）。
/// 最新的根还是 A 的 (1, 3)，实例表里没有实例 1 的行；B 那一版只在它那条记录的新根段里，下一次挂载的恢复由记录重建它
/// （D23（journal 的角色与格式） 已定项 15）。第二次写的内容与 `image_after_the_overwrite` 同一份，B 的落点就是 `UNITS_AFTER_OVERWRITE`。
fn crash_image_with_the_overwrite_record_but_not_its_root(tag: &str) -> MemoryPool {
    let mut pool = build_pool(tag);
    let operations_through_the_first_publish = pool.retained_operations().len();
    let previous = pool.output.clone();
    let content: Vec<u8> = (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect();
    let second = publish_overwrite_in_process(
        &mut pool,
        &previous,
        &content,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("覆盖写 B");
    assert_eq!(
        second.root.checkpoint_txg,
        CheckpointTxg(4),
        "发布 B 是 txg 4"
    );
    let operations = pool.retained_operations();
    let classifier = geometry();
    let kinds_of_the_overwrite: Vec<StepKind> = operations[operations_through_the_first_publish..]
        .iter()
        .map(|retained| classifier.classify(&retained.operation))
        .collect();
    let root_record_step = kinds_of_the_overwrite
        .iter()
        .position(|kind| *kind == StepKind::RootRecordFua)
        .expect("发布 B 有一步根槽 FUA");
    assert_eq!(
        (
            kinds_of_the_overwrite[..root_record_step]
                .iter()
                .filter(|kind| **kind == StepKind::JournalRecord)
                .count(),
            kinds_of_the_overwrite[root_record_step - 1]
        ),
        (2, StepKind::Barrier),
        "根槽 FUA 之前：B 那条记录两块盘各写了一份，紧挨着根槽的是记录之后那道屏障"
    );
    let mut image = pool.memory_pool_after_mkfs();
    image.apply(
        &operations
            [pool.mkfs_operation_count..operations_through_the_first_publish + root_record_step],
    );
    image
}

/// 每个单元（两盘同内容，取盘 0 那一份）此刻的整单元校验和：沿引用链补过哪几个单元，前后一比就知道。
fn unit_checksums(pool: &MemoryPool, units: &[(u64, usize)]) -> Vec<(u64, u32)> {
    units
        .iter()
        .map(|(slot, length)| {
            (
                *slot,
                crc32_castagnoli(&read(pool, 0, slot * SLOT, *length)),
            )
        })
        .collect()
}

/// 记录头里点名项数（4 字节）与载荷校验和（4 字节）的偏移、记录头宽、一条点名项的宽（D23（journal 的角色与格式） 已定项 4 /
/// 已定项 17 的字段表：magic 4 + 类型 2 + 算法 1 + 记录标志 1 + 记录长 4 之后是点名项数；事务段 78 + 事务号 8 + 提交标记 1 +
/// 本次发布内序号 4 + 反向链 4 之后是载荷校验和，罩记录头之后的点名项）。
const JOURNAL_RECORD_NAMED_COUNT_OFFSET: usize = 12;
const JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET: usize = 95;
const JOURNAL_RECORD_HEADER_BYTES: usize = 487;
const JOURNAL_NAMED_ENTRY_BYTES: usize = 56;

/// 几个单元的整单元校验和变了（`changes`：(起点槽, 旧, 新)）：journal 环里引用它们的位置条目——记录新根段里的树表指针与映射根指针、
/// 点名项——逐条补上，补过的记录先重算载荷校验和、再重封记录头校验和（罩整条 4096，字段在 46）。返回补了几条记录（按盘各算一条）。
/// 根槽没落盘的那一版只在记录里，`propagate` 补到根槽为止、够不着它。
fn propagate_into_journal_records(pool: &mut MemoryPool, changes: &[(u64, u32, u32)]) -> usize {
    let mut records_touched = 0;
    for device in DEVICES {
        for (counter, _, _) in journal_records_on_device(pool, device) {
            let offset = singlefs_core::journal::record_offset(
                counter,
                singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
            )
            .0;
            let mut record = read(pool, device, offset, 4096);
            let mut touched = false;
            for (slot, old, new) in changes {
                for location_device in DEVICES {
                    touched |= replace_all(
                        &mut record,
                        &location_pattern(location_device, *slot, *old),
                        &location_pattern(location_device, *slot, *new),
                    );
                }
            }
            if !touched {
                continue;
            }
            let named_count = usize::try_from(u32::from_le_bytes(
                record[JOURNAL_RECORD_NAMED_COUNT_OFFSET..JOURNAL_RECORD_NAMED_COUNT_OFFSET + 4]
                    .try_into()
                    .expect("4 字节"),
            ))
            .expect("点名项数");
            let payload_end = JOURNAL_RECORD_HEADER_BYTES + JOURNAL_NAMED_ENTRY_BYTES * named_count;
            let payload_checksum =
                crc32_castagnoli(&record[JOURNAL_RECORD_HEADER_BYTES..payload_end]);
            record[JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET
                ..JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET + 4]
                .copy_from_slice(&payload_checksum.to_le_bytes());
            let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
            record[46..78].copy_from_slice(&digest);
            write(pool, device, offset, &record);
            records_touched += 1;
        }
    }
    records_touched
}

/// Y4-a 的坏法：B 那棵分配记录树里 B 的数据单元那两条未释放记录（两盘各一条），分配代从 4 改成诞生代号 + 1 = 5，
/// 单元一个字节不动；新的整单元校验和沿引用链补到 B 的树表、中央映射，再补进 B 那条记录（两块盘各一份）。
/// 盘上留下的就是攻方那条分配器变异（只在发布 B 那一次把新记的分配代写成 txg + 1）崩在根槽之前的样子。
fn raise_the_allocation_generation_of_the_overwrite_data_unit_past_its_birth(
    image: &mut MemoryPool,
) {
    let checksums_before = unit_checksums(image, &UNITS_AFTER_OVERWRITE);
    assert_eq!(
        mutate_allocation_leaves(
            image,
            &UNITS_AFTER_OVERWRITE,
            ALLOCATION_LEAVES_AFTER_OVERWRITE,
            |bytes| rewrite_unreleased_allocation_generation(
                bytes,
                DATA_UNIT_AFTER_OVERWRITE,
                4,
                5
            ),
        ),
        2,
        "B 的数据单元两盘各一条未释放记录（两片叶各一条），分配代本来是 4"
    );
    let checksums_after = unit_checksums(image, &UNITS_AFTER_OVERWRITE);
    let changes: Vec<(u64, u32, u32)> = checksums_before
        .iter()
        .zip(&checksums_after)
        .filter(|((_, before), (_, after))| before != after)
        .map(|((slot, before), (_, after))| (*slot, *before, *after))
        .collect();
    assert_eq!(
        propagate_into_journal_records(image, &changes),
        2,
        "B 那条记录两块盘各一份，新根段里的树表指针都要补"
    );
}

/// 改法 E（`research/prompts/m2-wave3-code-r1-main-verification.md` 第三节 Y4、第四节第 2 条，被攻过零轮）：
/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 的读集再加「下一次挂载会先施加的那一版」——与最新根同实例、
/// txg = 最新根 + 1、带提交标记、根环里没有它的根的那条记录，新根段里树表指着的分配记录树。
///
/// 发布 B 的记录已落、根槽没落的崩溃镜像上，候选集（A 的根及更早）与「由记录施加出来的那几版」（实例表里还没有实例 1 的行）
/// 都读不到 B 那棵分配记录树，只有 E 那一步读得到：
/// ① 干净的崩溃镜像上一条都不红，I-3.10 真被评估过且成立（E 不在合法镜像上误报）；
/// ② B 的数据单元那两条未释放记录的分配代改成诞生代号 + 1，崩溃镜像上**只**红 I-3.10、别的判定与干净镜像逐项相同——
///    E 之前的读法在这张镜像上判成立；
/// ③ 这张坏镜像可写挂载在任何写之前被拒：挂载重建上一版时判分配记录的代与跨度（实审 A3a，`rebuild_version` 调
///    `allocation_record_generations_and_spans_are_judged`），分配代 5 晚于那一版的根，报「E142 走读同款」那一句；
///    两块盘逐字节不变、录制流一步没多。checker 在挂载之前红的，正是下一次挂载会拒掉的那一版。
#[test]
fn an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount(
) {
    let clean = crash_image_with_the_overwrite_record_but_not_its_root(
        "known-bad-allocation-generation-of-the-version-the-next-mount-applies",
    );
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    assert_eq!(
        violated_invariants(&verdicts_on_the_clean_image),
        Vec::<&str>::new(),
        "B 的记录已落、根槽没落的干净崩溃镜像上一条都不许红：{verdicts_on_the_clean_image:?}"
    );
    assert_eq!(
        verdict(&clean, "I-3.10"),
        InvariantVerdict::Holds,
        "干净的崩溃镜像上 I-3.10 要真被评估过且成立"
    );

    let mut image = clean.clone();
    raise_the_allocation_generation_of_the_overwrite_data_unit_past_its_birth(&mut image);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-3.10"],
        "B 那一版只在记录里时，它的分配代写错要在崩溃镜像上就红、只红 I-3.10：{verdicts:?}"
    );
    assert_eq!(
        verdicts_that_differ_outside(&verdicts, &verdicts_on_the_clean_image, &["I-3.10"]),
        [],
        "I-3.10 之外每一条的判定与干净的崩溃镜像逐项相同"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-3.10") else {
        panic!("上面判过 I-3.10 红");
    };
    assert!(
        detail.contains(&format!("槽 {DATA_UNIT_AFTER_OVERWRITE} "))
            && detail.contains("分配代 5")
            && detail.contains("诞生代号是 4"),
        "红在 B 的数据单元那条记录上：分配代 5、单元头里的诞生代号 4：{detail}"
    );

    let stream = SharedStream::new();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let image_before_the_mount = memory_pool_of_sparse_devices(&devices);
    let recorded_operations_before_the_mount = stream.operation_count();
    let refusal = match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => panic!(
            "坏镜像被可写挂载收下了：择根 txg {:?}、现行版 txg {:?}、施加记录前缀 {} 条",
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.journal.prefix_applied
        ),
        Err(refusal) => refusal,
    };
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "E142 走读同款",
                detail: "分配记录跨度为 0，或分配代 / 释放代晚于根",
            })
        ),
        "重建上一版时分配代 5 晚于那一版的根，可写挂载要拒在分配记录的代 / 跨度这一条上：{refusal:?}"
    );
    assert_eq!(
        memory_pool_of_sparse_devices(&devices),
        image_before_the_mount,
        "拒在任何写之前：两块盘逐字节不变（{refusal:?}）"
    );
    assert_eq!(
        stream.operation_count(),
        recorded_operations_before_the_mount,
        "拒在任何写之前：录制流一步没多（一个写、一道屏障都没发）"
    );
}

/// 根记录里树 ID 水位那 8 字节的偏移（D22（单元原子性怎么合成） 已定项 7 的偏移表：树 ID 水位 122、自证校验和 138）。
const ROOT_RECORD_TREE_IDENTIFIER_WATERMARK_OFFSET: usize = 122;

/// 根环里每一个带根 magic 的槽：树 ID 水位改成 `watermark`、重封自证校验和（罩整槽 512，字段在 138）。返回改了几个槽。
/// 盘上留下的就是「每次发布都没把水位抬过这一代发出的号」的样子（I-7.8 那一行判别力 ① 那一类）。
fn set_the_tree_identifier_watermark_of_every_root(
    image: &mut MemoryPool,
    watermark: u64,
) -> usize {
    let mut rewritten = 0;
    for (device, offset) in root_slots() {
        let mut bytes = read(image, device, offset, 512);
        if bytes[..4] != singlefs_core::root_record::ROOT_MAGIC {
            continue;
        }
        set_u64(
            &mut bytes,
            ROOT_RECORD_TREE_IDENTIFIER_WATERMARK_OFFSET,
            watermark,
        );
        let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
        bytes[138..170].copy_from_slice(&digest);
        write(image, device, offset, &bytes);
        rewritten += 1;
    }
    copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
    rewritten
}

/// 一份干净镜像上 I-7.8 真被评估过且成立；把根环里每一条根的水位压到 `lowered_watermark` 之后**只**红 I-7.8、
/// 红在「盘上出现过的最大树 ID」= `highest_tree_identifier_seen_on_disk` 上，别的判定与干净镜像逐项相同。
fn assert_lowering_the_watermark_reddens_only_the_watermark_invariant(
    clean: &MemoryPool,
    lowered_watermark: u64,
    highest_tree_identifier_seen_on_disk: u64,
    step: &str,
) {
    let verdicts_on_the_clean_image = check_pool_image(clean);
    assert_eq!(
        violated_invariants(&verdicts_on_the_clean_image),
        Vec::<&str>::new(),
        "{step}：干净镜像上一条都不许红：{verdicts_on_the_clean_image:?}"
    );
    assert_eq!(
        verdict(clean, "I-7.8"),
        InvariantVerdict::Holds,
        "{step}：干净镜像上 I-7.8 要真被评估过且成立"
    );
    let mut image = clean.clone();
    assert!(
        set_the_tree_identifier_watermark_of_every_root(&mut image, lowered_watermark) > 0,
        "{step}：根环里至少改了一条根"
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-7.8"],
        "{step}：水位压到 {lowered_watermark} 只红 I-7.8：{verdicts:?}"
    );
    assert_eq!(
        verdicts_that_differ_outside(&verdicts, &verdicts_on_the_clean_image, &["I-7.8"]),
        [],
        "{step}：I-7.8 之外每一条的判定与干净镜像逐项相同"
    );
    assert_eq!(
        verdict(&image, "I-7.8"),
        InvariantVerdict::Violated(format!(
            "根环水位最大 {lowered_watermark}，盘上出现过的最大树 ID {highest_tree_identifier_seen_on_disk}"
        )),
        "{step}：红在盘上出现过的最大树 ID {highest_tree_identifier_seen_on_disk} 上"
    );
}

/// 改法 D（I-7.8（根记录树 ID 水位不低于全池最大树 ID） 扫描方向排除「最新根的实例表判得出没发布过」的码 2 节点，
/// `research/prompts/m2-wave3-code-r1-main-verification.md` 第三节 Y1、第四节第 1 条，被攻过零轮）的「T = 0 的行不排除」那一道
/// 配一份只红 I-7.8 的坏镜像：只做过 mkfs 的池可写挂载（实例 1，暖机），再挂一次（实例 2）发第一个文件版本；
/// 进程退出之后实例 2 的每一条根的根槽都清零，接着在实例 1 那一版上建实例 3：取号、写行 (1, T, 0) 与中间实例行 (2, 0, 0)、暖机——
/// 经事务层的入口直接写（`acquire_instance`、`publish_instance_table_on_version_without_file` 交这里拼好的行、零单元发布），
/// 不经可写挂载：系统配置见证过实例 2 的发布，C554 乙之后可写挂载重读一次仍读不出实例 2 的根就拒，走不到写中间实例行。
/// 实例 2 写出的码 2 节点只由 (2, 0, 0) 那一行说到，行里说不出它发布到哪一代，照旧算「出现过」（D8（核心索引结构） 已定项 8 ②）。
/// 根环里的水位全压到实例 2 发的第一个号，I-7.8 要红在盘上还留着的实例 2 节点里最大的那个号上。干净镜像上一条都不红
/// （实例 3 的水位照可写挂载的取法按环里的记录取，盖得住实例 2 发过的号）。
/// 回退行那一道（① 回退行不排除）随回退行删掉（D18（块里携带什么信息） 已定项 11），没有对象。
#[test]
fn published_nodes_behind_an_intermediate_row_still_count_against_the_tree_identifier_watermark() {
    let mut pool = format_pool("known-bad-watermark-behind-an-intermediate-row");
    let mut devices = pool.reopen_recorded();
    let first_instance =
        mount_writable(&parameters(), &mut devices).expect("只做过 mkfs 的池可写挂载");
    assert_eq!(first_instance.output.instance, InstanceGeneration(1));
    let PoolVersion::WithoutFile(version_of_the_first_instance) = &first_instance.current else {
        panic!("只做过 mkfs 的池上实例 1 那一版树表 0 条")
    };
    let newest_root_of_the_first_instance = version_of_the_first_instance.root;
    drop(first_instance);
    let second_instance = mount_writable(&parameters(), &mut devices).expect("再挂一次");
    assert_eq!(second_instance.output.instance, InstanceGeneration(2));
    let PoolVersion::WithoutFile(version) = &second_instance.current else {
        panic!("只做过 mkfs 的池上现行那一版树表 0 条")
    };
    let mut allocator = second_instance.allocator.clone();
    let content: Vec<u8> = (0..3300usize)
        .map(|index| u8::try_from((index * 7 + 17) % 253).expect("小于 256"))
        .collect();
    let first_file_of_the_second_instance = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            &version.root,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
            },
            second_instance.output.instance,
            &version.record_bytes,
        )
        .expect("实例 2 发第一个文件版本")
    };
    let first_identifier_of_the_second_instance = version.root.tree_identifier_watermark;
    let mut roots_of_the_second_instance: Vec<CheckpointTxg> =
        vec![second_instance.output.row_publish.root().checkpoint_txg];
    roots_of_the_second_instance.extend(
        second_instance
            .output
            .warm_up_publishes
            .iter()
            .map(|warm_up| warm_up.root().checkpoint_txg),
    );
    roots_of_the_second_instance.push(first_file_of_the_second_instance.root.checkpoint_txg);
    drop(devices);
    let mut reopened = pool.reopen_recorded();
    let root_slot_bytes = usize::try_from(parameters().geometry.physical_block_size).expect("512");
    for txg in &roots_of_the_second_instance {
        let (device, offset) = root_slot_of(*txg);
        let (_, recorded) = reopened
            .iter_mut()
            .find(|(identity, _)| identity.0 == device)
            .expect("根槽所在的盘");
        recorded
            .write_at(
                DeviceOffsetInBytes(offset),
                &vec![0u8; root_slot_bytes],
                singlefs_core::block_device::WriteDurability::Plain,
            )
            .expect("清零实例 2 的根槽");
    }
    let rows_of_the_third_instance = vec![
        InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: newest_root_of_the_first_instance.checkpoint_txg,
            applied_transaction_high_water: 0,
        },
        InstanceRow {
            instance: InstanceGeneration(2),
            selected_root_txg: CheckpointTxg(0),
            applied_transaction_high_water: 0,
        },
    ];
    let newest_root_of_the_third_instance = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, reopened.as_mut_slice());
        let third_instance = acquire_instance(&mut writer).expect("取号 3");
        assert_eq!(third_instance, InstanceGeneration(3));
        // 账照可写挂载在实例 1 那一版上重建的样子：那一版分配记录树根指针全零，只有 mkfs 写在单元区里的两个单元；实例 2 的根清零了，
        // 影子账看不见它的单元。
        let mut allocator_rebuilt_on_the_first_instance = PoolAllocator::new(
            DEVICES
                .iter()
                .map(|device| DeviceFreeMap::new(DeviceIdentity(*device), common::IMAGE_BYTES))
                .collect(),
        );
        allocator_rebuilt_on_the_first_instance.mark_format_time_units(
            Placement {
                slot: INSTANCE_TABLE_SLOT,
                span: 2,
            },
            Placement {
                slot: TREE_TABLE_GENESIS_SLOT,
                span: 1,
            },
        );
        let rewrite = InstanceTableRewrite {
            rows: rows_of_the_third_instance,
            replaced_chain: vec![newest_root_of_the_first_instance.instance_table],
        };
        // 写行那次的 txg 与 jsn 照可写挂载接在环里全部根与全部记录的最大值后面（D23（journal 的角色与格式） 已定项 14 第 3 条）：
        // 实例 2 的根清零了、记录还在，最大的是它第一个文件那一版；水位照可写挂载取环里的 max（D8（核心索引结构） 已定项 8 ②）。
        let row_publish = publish_instance_table_on_version_without_file(
            &mut writer,
            &mut allocator_rebuilt_on_the_first_instance,
            &newest_root_of_the_first_instance,
            InstanceTableOnlyPublishPlan {
                txg: CheckpointTxg(first_file_of_the_second_instance.root.checkpoint_txg.0 + 1),
                counter: first_file_of_the_second_instance.record.counter + 1,
                instance: third_instance,
                back_chain: 0,
                rollback_floor: CheckpointTxg(0),
                instance_table: &rewrite,
                tree_identifier_watermark: first_file_of_the_second_instance
                    .root
                    .tree_identifier_watermark,
            },
        )
        .expect("实例 3 在实例 1 那一版上写行");
        // 暖机：本实例的根覆盖两块盘之前连推零单元发布（D16（发布语义） 已定项 8 戊，至多根环区域数那么多次）。
        let mut devices_holding_a_root_of_the_third_instance =
            std::collections::BTreeSet::from([root_slot_of(row_publish.root.checkpoint_txg).0]);
        let mut newest = row_publish;
        for _ in 0..parameters().region_devices.len() {
            if DEVICES
                .iter()
                .all(|device| devices_holding_a_root_of_the_third_instance.contains(device))
            {
                break;
            }
            let warm_up = publish_without_units(
                &mut writer,
                &newest.root,
                ZeroUnitPublishPlan {
                    txg: CheckpointTxg(newest.root.checkpoint_txg.0 + 1),
                    counter: newest.record.counter + 1,
                    instance: third_instance,
                    back_chain: back_chain_of(&newest.record_bytes),
                    rollback_floor: CheckpointTxg(0),
                    tree_identifier_watermark: newest.root.tree_identifier_watermark,
                },
            )
            .expect("实例 3 暖机");
            devices_holding_a_root_of_the_third_instance
                .insert(root_slot_of(warm_up.root.checkpoint_txg).0);
            newest = warm_up;
        }
        newest.root
    };
    pool.devices = Some(reopened);
    assert_eq!(
        instance_table_chain_of_root(&pool.memory_pool(), &newest_root_of_the_third_instance)
            .expect("实例 3 最新那条根的实例表读得出")
            .records
            .rows,
        vec![
            InstanceRow {
                instance: InstanceGeneration(1),
                selected_root_txg: newest_root_of_the_first_instance.checkpoint_txg,
                applied_transaction_high_water: 0,
            },
            InstanceRow {
                instance: InstanceGeneration(2),
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
            }
        ],
        "盘上实例 3 最新那条根的表里是实例 1 那一行与中间实例行 (2, 0, 0)"
    );
    assert_eq!(
        (
            first_identifier_of_the_second_instance,
            first_file_of_the_second_instance
                .tree_identifiers
                .highest()
                .0
        ),
        (65536, 65547),
        "实例 2 发 65536..65547，其中 65541..65547 只有树表条目、没有单元 ⇒ 盘上扫得出来最大的是映射树 65540"
    );
    assert_lowering_the_watermark_reddens_only_the_watermark_invariant(
        &pool.memory_pool(),
        65536,
        65540,
        "中间实例行后面的实例 2",
    );
}

/// 两块 4 GiB 内存盘上新池新建文件之后连着 369 次取号之后崩溃，可写挂载一次（取号 371、写 370 行：实例表两片），再挂一次
/// （取号 372、写 1 行）：第二次写行整条链重写、两片旧链逐片释放（释放代 = 那次写行的 txg）。交回镜像、现行那一版、
/// 旧链第 1 片的槽、第二次写行的 txg，与两次挂载发出的每一版的单元表（起点槽, 字节数）去重合在一起——分配记录树按位置寻址之后
/// （D8（核心索引结构） 已定项 14）一片叶可以被好几版照抄，改它要把每一版里指着它的父节点都补上，不只现行那一版的。
fn image_after_rewriting_a_two_page_instance_table_chain() -> (
    MemoryPool,
    singlefs_core::transaction::TransactionOutput,
    u64,
    u64,
    Vec<(u64, usize)>,
) {
    type Devices = Vec<(
        DeviceIdentity,
        singlefs_harness::RecordingBlockDevice<singlefs_harness::memory_pool::SparseBlockDevice>,
    )>;
    let stream = SharedStream::new();
    let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
        .iter()
        .map(|identity| {
            (
                *identity,
                singlefs_harness::RecordingBlockDevice::with_shared_stream(
                    *identity,
                    singlefs_harness::memory_pool::SparseBlockDevice::new(
                        common::IMAGE_BYTES,
                        singlefs_core::block_device::PhysicalBlockSizeInBytes(512),
                    ),
                    stream.clone(),
                ),
            )
        })
        .collect();
    singlefs_harness::scenario::run_new_pool_file_creation(
        &parameters(),
        &mut devices,
        &stream,
        |_, _| {},
    )
    .expect("新池新建文件");
    {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        for _ in 0..369 {
            singlefs_core::transaction::acquire_instance(&mut writer).expect("取号");
        }
    }
    let first = mount_writable(&parameters(), &mut devices).expect("写 370 行：两片");
    let old_first_page = first.output.row_publish.root().instance_table;
    let old_first_page_bytes = PoolReader::read(
        devices.as_slice(),
        old_first_page.locations[0].device,
        old_first_page.locations[0].slot.to_device_offset(),
        32768,
    )
    .expect("第 0 片读得到");
    let singlefs_core::instance_table::InstanceTableChainRecord::NextPage(old_second_page) =
        singlefs_core::instance_table::InstanceTablePage::parse(
            &old_first_page_bytes,
            singlefs_core::instance_table::InstanceTablePageIndex::FIRST,
        )
        .expect("第 0 片解得开")
        .chain
    else {
        panic!("第一次写行写的是两片")
    };
    let second = mount_writable(&parameters(), &mut devices).expect("再挂一次：整条链重写");
    let release_txg = second.output.row_publish.root().checkpoint_txg.0;
    let units_of_every_version: Vec<(u64, usize)> = [&first, &second]
        .into_iter()
        .flat_map(|mounted| {
            std::iter::once(&mounted.output.row_publish)
                .chain(&mounted.output.warm_up_publishes)
                .chain(std::iter::once(&mounted.current))
        })
        .flat_map(|version| {
            version
                .file_version()
                .expect("新池新建文件之后每一版都带文件")
                .units
                .iter()
                .map(|unit| {
                    (
                        unit.slot.0,
                        usize::try_from(unit.identity.span_slots() * SLOT).expect("单元字节数"),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<std::collections::BTreeSet<(u64, usize)>>()
        .into_iter()
        .collect();
    let current = second
        .current
        .into_file_version()
        .expect("新池新建文件之后的现行版本带文件");
    (
        memory_pool_of_sparse_devices(&devices),
        current,
        old_second_page.locations[0].slot.0,
        release_txg,
        units_of_every_version,
    )
}

/// 分配记录树叶里槽号是 `slot` 的已释放记录（两盘各一条）释放代改成 `generation`；返回改了几条。
fn set_released_generation_of_slot(bytes: &mut [u8], slot: u64, generation: u64) -> usize {
    let count = usize::from(u16::from_le_bytes([
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
    ]));
    let mut rewritten = 0;
    for index in 0..count {
        let record = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * index;
        let mut slot_bytes = [0u8; 8];
        slot_bytes[..6].copy_from_slice(&bytes[record + 4..record + 10]);
        let span_field = u16::from_le_bytes([bytes[record + 10], bytes[record + 11]]);
        if u64::from_le_bytes(slot_bytes) == slot
            && span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0
        {
            set_u64(bytes, record + 12, generation);
            rewritten += 1;
        }
    }
    rewritten
}

/// 实四报告「顺带发现」O3：I-3.9（释放代落在停止引用它的那一格区间里）的引用集合要认实例表链第 1 片起的落点
/// （D18（块里携带什么信息） 已定项 11：第 k 片末尾的链指针记录指着第 k + 1 片）。整条两片链被重写之后，最新那棵账里旧链第 1 片那两条
/// 已释放记录的释放代写成那次写行的 txg − 1：引用过它的最新一条有效根就是 txg − 1 那一条，释放代不在 (txg − 1, txg] 里 ⇒ I-3.9 红。
/// 引用集合不认第 1 片时这两条被当成「见证它释放的根已不在候选集里」跳过，I-3.9 照样判成立。干净镜像上 I-3.9 判成立（不是不适用）。
#[test]
fn release_generation_outside_its_interval_on_the_second_page_of_the_replaced_instance_table_chain_reddens_the_release_generation_invariant(
) {
    let (clean, current, old_second_page_slot, release_txg, units) =
        image_after_rewriting_a_two_page_instance_table_chain();
    assert_eq!(verdict(&clean, "I-3.9"), InvariantVerdict::Holds);
    // 分配记录树按位置寻址（D8（核心索引结构） 已定项 14）：旧链第 1 片两盘各一条记录，分在两块盘各自装着那一槽的叶里。
    let leaf_holding_the_old_second_page = |device: u32| {
        current
            .unit(TransactionUnit::AllocationTreeNodeBelowTheRoot(
                AllocationRecordTreeNodePosition {
                    level: 0,
                    device: DeviceIdentity(device),
                    index_in_device: old_second_page_slot / ALLOCATION_RECORD_TREE_LEAF_SLOTS,
                },
            ))
            .slot
            .0
    };
    let mut image = clean.clone();
    assert_eq!(
        mutate_allocation_leaves(
            &mut image,
            &units,
            [
                leaf_holding_the_old_second_page(0),
                leaf_holding_the_old_second_page(1)
            ],
            |bytes| set_released_generation_of_slot(bytes, old_second_page_slot, release_txg - 1),
        ),
        2,
        "旧链第 1 片两盘各一条已释放记录"
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        vec!["I-3.9"],
        "只有 I-3.9 红：{verdicts:?}"
    );
}

/// 只有一个实例的镜像上 I-3.12（实例表行 T 低于引用它的根的 txg） 报的不适用理由（池级 checker 的原文）。
const INSTANCE_TABLE_ROWS_NOT_APPLICABLE: &str = "根环里没有哪条根指着一张读得出、有行的实例表";

/// 一条 journal 记录（两盘各一份）里码 1 那个点着 `data_unit_slot` 的点名项，诞生代号从 `from` 改成 `to`；
/// 载荷校验和（罩点名项）与头校验和（罩整条 4096，字段在 46）重封。
fn rewrite_the_birth_txg_of_the_data_unit_named_by_journal_record(
    image: &mut MemoryPool,
    counter: u64,
    data_unit_slot: u64,
    from: u64,
    to: u64,
) {
    rewrite_the_named_entry_of_the_data_unit_in_journal_record(
        image,
        counter,
        data_unit_slot,
        |entry| {
            assert_eq!(
                get_u64(entry, NAMED_ENTRY_BIRTH_TXG_OFFSET),
                from,
                "计数器 {counter} 那条记录点名的诞生代号本来是 {from}"
            );
            set_u64(entry, NAMED_ENTRY_BIRTH_TXG_OFFSET, to);
        },
    );
}

/// 点名项（56 字节，D23（journal 的角色与格式） 已定项 17）里的字段偏移：第一条位置条目的槽号（盘 4 之后）、类标签 28、出生树 29、
/// 诞生代号 37、key 尾段 45（码 1：写序实例代号 4 + 事务号 6）。
const NAMED_ENTRY_FIRST_LOCATION_SLOT_OFFSET: usize = 4;
const NAMED_ENTRY_UNIT_CLASS_OFFSET: usize = 28;
const NAMED_ENTRY_BIRTH_TREE_OFFSET: usize = 29;
const NAMED_ENTRY_BIRTH_TXG_OFFSET: usize = 37;
const NAMED_ENTRY_KEY_TAIL_INSTANCE_OFFSET: usize = 45;
const NAMED_ENTRY_KEY_TAIL_TRANSACTION_OFFSET: usize = 49;

/// 一条 journal 记录（两盘各一份）里码 1 那个点着 `data_unit_slot` 的点名项交给 `change` 改（拿到的是那 56 字节）；
/// 载荷校验和（罩点名项）与头校验和（罩整条 4096，字段在 46）重封。
fn rewrite_the_named_entry_of_the_data_unit_in_journal_record(
    image: &mut MemoryPool,
    counter: u64,
    data_unit_slot: u64,
    change: impl Fn(&mut [u8]),
) {
    let offset =
        singlefs_core::journal::record_offset(counter, singlefs_format::JOURNAL_RING_DEFAULT_BYTES)
            .0;
    for device in DEVICES {
        let mut record = read(image, device, offset, 4096);
        let named_count = usize::try_from(u32::from_le_bytes(
            record[JOURNAL_RECORD_NAMED_COUNT_OFFSET..JOURNAL_RECORD_NAMED_COUNT_OFFSET + 4]
                .try_into()
                .expect("4 字节"),
        ))
        .expect("点名项数");
        let entry_start = (0..named_count)
            .map(|index| JOURNAL_RECORD_HEADER_BYTES + JOURNAL_NAMED_ENTRY_BYTES * index)
            .find(|entry_start| {
                let mut slot_bytes = [0u8; 8];
                slot_bytes[..6].copy_from_slice(
                    &record[entry_start + NAMED_ENTRY_FIRST_LOCATION_SLOT_OFFSET
                        ..entry_start + NAMED_ENTRY_FIRST_LOCATION_SLOT_OFFSET + 6],
                );
                record[entry_start + NAMED_ENTRY_UNIT_CLASS_OFFSET] == 1
                    && u64::from_le_bytes(slot_bytes) == data_unit_slot
            })
            .expect("那条记录点名了这个数据单元");
        change(&mut record[entry_start..entry_start + JOURNAL_NAMED_ENTRY_BYTES]);
        let payload_end = JOURNAL_RECORD_HEADER_BYTES + JOURNAL_NAMED_ENTRY_BYTES * named_count;
        let payload_checksum = crc32_castagnoli(&record[JOURNAL_RECORD_HEADER_BYTES..payload_end]);
        record[JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET..JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET + 4]
            .copy_from_slice(&payload_checksum.to_le_bytes());
        let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
        record[46..78].copy_from_slice(&digest);
        write(image, device, offset, &record);
    }
}

/// 一条 journal 记录（两盘各一份）的 `checkpoint_txg` 从 `from` 改成 `to`，头校验和重封。
fn set_checkpoint_txg_of_journal_record(image: &mut MemoryPool, counter: u64, from: u64, to: u64) {
    let offset =
        singlefs_core::journal::record_offset(counter, singlefs_format::JOURNAL_RING_DEFAULT_BYTES)
            .0;
    for device in DEVICES {
        let mut record = read(image, device, offset, 4096);
        assert_eq!(
            get_u64(&record, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET),
            from,
            "盘 {device} 计数器 {counter} 那条记录的 txg 本来是 {from}"
        );
        set_u64(&mut record, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET, to);
        let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
        record[46..78].copy_from_slice(&digest);
        write(image, device, offset, &record);
    }
}

/// 系统配置槽里世代号（8 字节）、设备数（4 字节）、槽距（4 字节，槽 1 在这个偏移上）的偏移（`.claude/kb/layout/01-first-txn.md` 一）。
const SYSTEM_CONFIGURATION_SLOT_GENERATION_OFFSET: usize = 147;
const SYSTEM_CONFIGURATION_DEVICE_COUNT_OFFSET: usize = 143;
const SYSTEM_CONFIGURATION_SLOT_SPACING_OFFSET: usize = 429;

/// 一块盘上系统配置槽 1 的偏移：槽 0 记的槽距。
fn system_configuration_slot_one_offset(image: &MemoryPool, device: u32) -> u64 {
    let slot_zero = read(image, device, 0, 4096);
    u64::from(u32::from_le_bytes(
        slot_zero[SYSTEM_CONFIGURATION_SLOT_SPACING_OFFSET
            ..SYSTEM_CONFIGURATION_SLOT_SPACING_OFFSET + 4]
            .try_into()
            .expect("4 字节"),
    ))
}

/// 记账树根兼叶（50250）里条目区的起点与每条宽（`set_accounting` 同一个读法：115 + 2 × 22 起，每条 34）、代段在行里的偏移
/// （统计量 2 + 树 8 + 设备 4 之后）。
const ACCOUNTING_LEAF_ENTRY_START: usize = 159;
const ACCOUNTING_LEAF_ENTRY_BYTES: usize = 34;
const ACCOUNTING_ROW_GENERATION_OFFSET: usize = 14;

/// inode 叶容器第 0 条记录的起点（码 3 记录区从 136 起）与记录里改动计数的偏移（D8（核心索引结构） 已定项 6：偏移 88）。
const INODE_LEAF_FIRST_RECORD_START: usize = 136;
const INODE_RECORD_CHANGE_COUNT_OFFSET: usize = 88;

/// 根记录里实例表指针的起点（86 字节，D22（单元原子性怎么合成） 已定项 7）与指针头部 extent 偏移（D19（块指针的结构与宽度预算） 已定项 11：
/// MAC 16 + nonce 12 + 算法类型 1 + 压缩算法 1 + 压后长度 2 之后的 2 字节）。
const ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET: usize = 170;
const POINTER_EXTENT_OFFSET_FIELD_OFFSET: usize = 32;

/// 指针（节点指针 86、数据指针 88 同一个前缀）里两条位置条目的起点与每条宽（头部 50 之后，位置条目 = 盘 4 + 槽 6 + 整单元校验和 4，
/// D19（块指针的结构与宽度预算） 已定项 11）。
const POINTER_FIRST_LOCATION_OFFSET: usize = 50;
const POINTER_LOCATION_BYTES: usize = 14;

/// 写完新池新建文件那份镜像上，`parent` 单元里指着 `child` 的那条指针（两条位置条目依次是盘 0、盘 1 那一份）的 extent 偏移 0 → 1，
/// 重封、补到根。指针按两条位置条目的字节找，`parent` 里恰好一处。
fn set_the_extent_offset_of_the_pointer_from_parent_to_child(
    image: &mut MemoryPool,
    parent: u64,
    child: u64,
) {
    let child_length = UNITS
        .iter()
        .find(|(start, _)| *start == child)
        .expect("登记过的单元")
        .1;
    let locations: Vec<u8> = DEVICES
        .iter()
        .flat_map(|device| {
            let child_checksum =
                crc32_castagnoli(&read(image, *device, child * SLOT, child_length));
            location_pattern(*device, child, child_checksum)
        })
        .collect();
    assert_eq!(locations.len(), 2 * POINTER_LOCATION_BYTES);
    mutate_unit(
        image,
        parent,
        |bytes| {
            let found: Vec<usize> = bytes
                .windows(locations.len())
                .enumerate()
                .filter(|(_, window)| *window == locations.as_slice())
                .map(|(position, _)| position)
                .collect();
            assert_eq!(
                found.len(),
                1,
                "单元 {parent} 里指着 {child} 的指针恰好一条：{found:?}"
            );
            let field =
                found[0] - POINTER_FIRST_LOCATION_OFFSET + POINTER_EXTENT_OFFSET_FIELD_OFFSET;
            assert_eq!(
                bytes[field..field + 2],
                [0, 0],
                "指着 {child} 的那条指针 extent 偏移本来是 0"
            );
            bytes[field] = 1;
        },
        true,
    );
}

/// 树表条目里 `previous_snapshot_txg` 那 8 字节的偏移：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86
/// （D8（核心索引结构） 已定项 8 的字段表）。
const TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 86;
/// 树的种类码里 inode 树那一个（`.claude/kb/invariants.md` I-9.25 那一行「种类码 ≠ 2」）。
const TREE_KIND_INODE: u16 = 2;
/// 新池新建文件那一版树表的条目数（十一条，见 `LIVELIST_TREE_TABLE_ENTRY_INDEX` 那一行）。
const TREE_TABLE_ENTRIES_AFTER_THE_FIRST_FILE: usize = 11;

/// 新池新建文件那一版树表里 inode 树那一条（种类 2）的下标：恰好一条。
fn inode_tree_entry_index_in(tree_table: &[u8]) -> usize {
    let indexes: Vec<usize> = (0..TREE_TABLE_ENTRIES_AFTER_THE_FIRST_FILE)
        .filter(|index| {
            let kind_offset = tree_table_entry_offset(*index) + TREE_TABLE_ENTRY_KIND_OFFSET;
            u16::from_le_bytes([tree_table[kind_offset], tree_table[kind_offset + 1]])
                == TREE_KIND_INODE
        })
        .collect();
    assert_eq!(indexes.len(), 1, "树表里 inode 树恰好一条：{indexes:?}");
    indexes[0]
}

/// inode 记录里对象出生代那 8 字节的偏移（D8（核心索引结构） 已定项 6：偏移 8）。
const INODE_RECORD_OBJECT_BIRTH_OFFSET: usize = 8;

/// 记录新根段（记录头 99 起，D23（journal 的角色与格式） 已定项 4）里 I-8.10 逐字节比的另外三段，各取一个改了不牵动别的判定的字段。
#[derive(Clone, Copy)]
enum NewRootSegmentField {
    /// 树表指针（新根段 [0..86)）第一条位置条目的整单元校验和（4 字节）。
    TreeTablePointerFirstChecksum,
    /// 中央映射树根指针（新根段 [86..172)）第一条位置条目的整单元校验和（4 字节）。
    MappingRootPointerFirstChecksum,
    /// 树 ID 水位（新根段 [172..180)，8 字节）。
    TreeIdentifierWatermark,
}

const JOURNAL_RECORD_NEW_ROOT_SEGMENT_OFFSET: usize = 99;
const NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER_OFFSET: usize = 86;
const NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK_OFFSET: usize = 172;
/// 位置条目里整单元校验和的偏移（盘 4 + 槽 6 之后）。
const LOCATION_CHECKSUM_OFFSET: usize = 10;

/// A 那条记录（计数器 3，新池新建文件那次发布的末条）新根段里 `field` 加 1，两盘各一份、头校验和重封；A 的根不动。
fn add_one_to_a_new_root_segment_field_of_the_first_file_record(
    image: &mut MemoryPool,
    field: NewRootSegmentField,
) {
    let offset =
        singlefs_core::journal::record_offset(3, singlefs_format::JOURNAL_RING_DEFAULT_BYTES).0;
    for device in DEVICES {
        let mut record = read(image, device, offset, 4096);
        assert_eq!(
            get_u64(&record, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET),
            3,
            "计数器 3 是 A 那条记录"
        );
        match field {
            NewRootSegmentField::TreeTablePointerFirstChecksum => {
                let checksum_offset = JOURNAL_RECORD_NEW_ROOT_SEGMENT_OFFSET
                    + POINTER_FIRST_LOCATION_OFFSET
                    + LOCATION_CHECKSUM_OFFSET;
                let checksum = get_u32(&record, checksum_offset);
                set_u32(&mut record, checksum_offset, checksum.wrapping_add(1));
            }
            NewRootSegmentField::MappingRootPointerFirstChecksum => {
                let checksum_offset = JOURNAL_RECORD_NEW_ROOT_SEGMENT_OFFSET
                    + NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER_OFFSET
                    + POINTER_FIRST_LOCATION_OFFSET
                    + LOCATION_CHECKSUM_OFFSET;
                let checksum = get_u32(&record, checksum_offset);
                set_u32(&mut record, checksum_offset, checksum.wrapping_add(1));
            }
            NewRootSegmentField::TreeIdentifierWatermark => {
                let watermark_offset = JOURNAL_RECORD_NEW_ROOT_SEGMENT_OFFSET
                    + NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK_OFFSET;
                let watermark = get_u64(&record, watermark_offset);
                set_u64(&mut record, watermark_offset, watermark + 1);
            }
        }
        let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
        record[46..78].copy_from_slice(&digest);
        write(image, device, offset, &record);
    }
}

/// 二·1 那一批新立的几条里，坏镜像建在**写完新池新建文件**那份干净镜像上的：每份改一处、只红它自己那一条
/// （`research/prompts/m3-inv-b-r1-main-verification.md` 第二节、`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 7、11 行）。
fn known_bad_images_of_pointer_journal_and_generation_fields() -> Vec<(&'static str, Mutation)> {
    vec![
        // I-1.12：A 的 extent 树根里那条数据指针的出生树（指针偏移 34）改成另一棵发过号的树，重封、补到根：单元头里的树照旧是 extent 树。
        (
            POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
            Box::new(|image: &mut MemoryPool| {
                let data_pointer_birth_tree = EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET - 8;
                mutate_unit(
                    image,
                    EXTENT_ROOT,
                    |bytes| {
                        assert_eq!(
                            get_u64(bytes, data_pointer_birth_tree),
                            TREE_IDENTIFIER_EXTENT,
                            "数据指针的出生树本来是 extent 树"
                        );
                        set_u64(
                            bytes,
                            data_pointer_birth_tree,
                            ANOTHER_ISSUED_TREE_BELOW_THE_WATERMARK,
                        );
                    },
                    true,
                );
            }),
        ),
        // I-8.10：A 那条记录（计数器 3，环里最后一条、带末条标志）的新根段里 F（偏移 99 + 180）0 → 1，两盘各一份、头校验和重封；
        // A 的根（txg 3）照旧带 F 0。
        (
            LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
            Box::new(|image: &mut MemoryPool| {
                const NEW_ROOT_SEGMENT_ROLLBACK_FLOOR_OFFSET: usize = 99 + 180;
                let offset = singlefs_core::journal::record_offset(
                    3,
                    singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
                )
                .0;
                for device in DEVICES {
                    let mut record = read(image, device, offset, 4096);
                    assert_eq!(
                        get_u64(&record, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET),
                        3,
                        "计数器 3 是 A 那条记录"
                    );
                    assert_eq!(
                        get_u64(&record, NEW_ROOT_SEGMENT_ROLLBACK_FLOOR_OFFSET),
                        0,
                        "A 那条记录的新根段 F 本来是 0"
                    );
                    set_u64(&mut record, NEW_ROOT_SEGMENT_ROLLBACK_FLOOR_OFFSET, 1);
                    let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
                    record[46..78].copy_from_slice(&digest);
                    write(image, device, offset, &record);
                }
            }),
        ),
        // I-9.25：A 的树表（50252）第 0 条（extent 树，不是 inode 树）的 previous_snapshot_txg（条目偏移 100）0 → 3，重封、补到根。
        (
            PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    TREE_TABLE,
                    |bytes| {
                        let field = tree_table_entry_offset(0)
                            + TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET;
                        assert_eq!(
                            get_u64(bytes, field),
                            0,
                            "extent 树那条的 previous_snapshot_txg 本来是 0"
                        );
                        set_u64(bytes, field, 3);
                    },
                    true,
                );
            }),
        ),
        // I-3.13：A 的记账树（50250，根兼叶）第 1 条（不是首末条，头里的 key 区间不跟着变）的代段 3 → 2，重封、补到根。
        (
            ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    ACCOUNTING_ROOT,
                    |bytes| {
                        let count =
                            usize::from(u16::from_le_bytes([bytes[82 + 44], bytes[83 + 44]]));
                        assert!(count >= 3, "记账树至少三行，第 1 条不是首末条");
                        let generation = ACCOUNTING_LEAF_ENTRY_START
                            + ACCOUNTING_LEAF_ENTRY_BYTES
                            + ACCOUNTING_ROW_GENERATION_OFFSET;
                        assert_eq!(get_u64(bytes, generation), 3, "A 那一版记账行的代段是 3");
                        set_u64(bytes, generation, 2);
                    },
                    true,
                );
            }),
        ),
        // I-9.26：A 的 inode 叶容器（50242）第 0 条记录的改动计数 3 → 4（大于容器诞生代号 3），重封、补到根。
        (
            INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    INODE_LEAF,
                    |bytes| {
                        let field =
                            INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_CHANGE_COUNT_OFFSET;
                        assert_eq!(get_u64(bytes, field), 3, "改动计数本来是 3");
                        set_u64(bytes, field, 4);
                    },
                    true,
                );
            }),
        ),
        // I-9.25 inode 树那一半（`research/prompts/m3-batch4-code-r1-sonnet-output.md` 第五节盲区第 2 条）：A 的树表（50252）里
        // inode 树那一条（种类 2）的 previous_snapshot_txg 0 → 4，比引用这张树表的根（txg 3）大，重封、补到根。
        (
            PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    TREE_TABLE,
                    |bytes| {
                        let field = tree_table_entry_offset(inode_tree_entry_index_in(bytes))
                            + TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET;
                        assert_eq!(
                            get_u64(bytes, field),
                            0,
                            "inode 树那条的 previous_snapshot_txg 本来是 0"
                        );
                        set_u64(bytes, field, 4);
                    },
                    true,
                );
            }),
        ),
        // I-9.26 下界（盲区第 3 条）：A 的 inode 叶容器（50242）第 0 条记录的改动计数改成对象出生代减 1（上界照旧成立），重封、补到根。
        (
            INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    INODE_LEAF,
                    |bytes| {
                        let object_birth = get_u64(
                            bytes,
                            INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_OBJECT_BIRTH_OFFSET,
                        );
                        assert!(object_birth >= 1, "对象出生代 {object_birth} 至少是 1");
                        set_u64(
                            bytes,
                            INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_CHANGE_COUNT_OFFSET,
                            object_birth - 1,
                        );
                    },
                    true,
                );
            }),
        ),
        // I-7.17 只错槽那一格（盲区第 6 条）：txg 1 那条根（区域 1 槽 0，盘 1）整槽原样拷到同一区域的槽 3（那一槽空着）：
        // 区域对、槽不对。
        (
            ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES,
            Box::new(|image: &mut MemoryPool| {
                let region_one_start = 64 * SLOT + 3 * (1 << 20);
                let root_of_txg_one = read(image, 1, region_one_start, 512);
                assert_eq!(
                    get_u64(&root_of_txg_one, 28),
                    1,
                    "区域 1 槽 0 坐着 txg 1 的根"
                );
                let target = region_one_start + 3 * 4096;
                assert_ne!(
                    &read(image, 1, target, 512)[..4],
                    b"SFSR",
                    "区域 1 槽 3 空着"
                );
                write(image, 1, target, &root_of_txg_one);
            }),
        ),
        // I-8.10 新根段另外三段各一份（盲区第 5 条）：A 那条记录（计数器 3）新根段里树表指针第一条位置条目的校验和、映射根指针第一条
        // 位置条目的校验和、树 ID 水位各加 1，两盘各一份、头校验和重封；A 的根照旧。
        (
            LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
            Box::new(|image: &mut MemoryPool| {
                add_one_to_a_new_root_segment_field_of_the_first_file_record(
                    image,
                    NewRootSegmentField::TreeTablePointerFirstChecksum,
                );
            }),
        ),
        (
            LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
            Box::new(|image: &mut MemoryPool| {
                add_one_to_a_new_root_segment_field_of_the_first_file_record(
                    image,
                    NewRootSegmentField::MappingRootPointerFirstChecksum,
                );
            }),
        ),
        (
            LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
            Box::new(|image: &mut MemoryPool| {
                add_one_to_a_new_root_segment_field_of_the_first_file_record(
                    image,
                    NewRootSegmentField::TreeIdentifierWatermark,
                );
            }),
        ),
        // I-2.4 指针 extent 偏移在走读的另外三个入口上各一份（盲区第 1 条）：码 2 节点指针（树表里 extent 树那条的根指针，指着 50240）、
        // inode 内部条目的子指针（inode 树根 50244 里指着叶容器 50242 的那条）、extent 记录的数据指针（extent 树根里指着数据单元 50180 的那条），
        // extent 偏移 0 → 1，重封、补到根。
        (
            "I-2.4",
            Box::new(|image: &mut MemoryPool| {
                set_the_extent_offset_of_the_pointer_from_parent_to_child(
                    image,
                    TREE_TABLE,
                    EXTENT_ROOT,
                );
            }),
        ),
        (
            "I-2.4",
            Box::new(|image: &mut MemoryPool| {
                set_the_extent_offset_of_the_pointer_from_parent_to_child(
                    image, INODE_ROOT, INODE_LEAF,
                );
            }),
        ),
        (
            "I-2.4",
            Box::new(|image: &mut MemoryPool| {
                set_the_extent_offset_of_the_pointer_from_parent_to_child(
                    image,
                    EXTENT_ROOT,
                    DATA_UNIT,
                );
            }),
        ),
        // I-7.17：txg 1 那条根（区域 1 槽 0，盘 1）整槽原样拷到区域 2 槽 3（盘 0，那一槽空着）：按 txg 算它只该住区域 1 槽 0。
        (
            ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES,
            Box::new(|image: &mut MemoryPool| {
                let root_of_txg_one = read(image, 1, 64 * SLOT + 3 * (1 << 20), 512);
                assert_eq!(
                    get_u64(&root_of_txg_one, 28),
                    1,
                    "区域 1 槽 0 坐着 txg 1 的根"
                );
                let target = 64 * SLOT + 2 * 3 * (1 << 20) + 3 * 4096;
                assert_ne!(
                    &read(image, 0, target, 512)[..4],
                    b"SFSR",
                    "区域 2 槽 3 空着"
                );
                write(image, 0, target, &root_of_txg_one);
            }),
        ),
        // I-7.15：盘 1 两个系统配置槽的设备数 2 → 3，整槽校验和重封：盘 0 择到的说 2、盘 1 择到的说 3。
        (
            DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
            Box::new(|image: &mut MemoryPool| {
                for slot_offset in [0, system_configuration_slot_one_offset(image, 1)] {
                    mutate_system_configuration_slot(image, 1, slot_offset, |bytes| {
                        assert_eq!(
                            get_u32(bytes, SYSTEM_CONFIGURATION_DEVICE_COUNT_OFFSET),
                            2,
                            "两块盘的池"
                        );
                        set_u32(bytes, SYSTEM_CONFIGURATION_DEVICE_COUNT_OFFSET, 3);
                    });
                }
            }),
        ),
        // I-2.4 指针 extent 偏移那一格（`research/prompts/m3-inv-a-r1-opus-output.md` A6 的造法）：每条自证过的根里实例表指针的 extent 偏移 0 → 1，
        // 根槽自证校验和重封；实例表指针从第 8 批起也在 I-8.10 比的新根段七段里，同 (实例, txg) 的记录跟着改、反向链接着重算
        // （`set_the_extent_offset_of_the_instance_table_pointer_in_the_records_of_these_roots`）。
        (
            "I-2.4",
            Box::new(|image: &mut MemoryPool| {
                let mut rewritten_roots = 0;
                for (device, offset) in root_slots() {
                    let mut bytes = read(image, device, offset, 512);
                    if bytes[..4] != singlefs_core::root_record::ROOT_MAGIC {
                        continue;
                    }
                    let field = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
                        + POINTER_EXTENT_OFFSET_FIELD_OFFSET;
                    assert_eq!(
                        bytes[field..field + 2],
                        [0, 0],
                        "实例表指针的 extent 偏移本来是 0"
                    );
                    bytes[field] = 1;
                    let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
                    bytes[138..170].copy_from_slice(&digest);
                    write(image, device, offset, &bytes);
                    rewritten_roots += 1;
                }
                assert!(rewritten_roots > 0, "根环里至少改了一条根");
                set_the_extent_offset_of_the_instance_table_pointer_in_the_records_of_these_roots(
                    image,
                );
            }),
        ),
        // I-9.27（判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K1）：A 的树表（50252）第 0 条（extent 树，种类 1）的
        // 头 ID（条目偏移 116）从同一张表里 inode 树条目的树 ID 改成 0，重封、补到根。头 ID 别的判定都不读，只红 I-9.27。
        (
            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    TREE_TABLE,
                    |bytes| {
                        let entry = tree_table_entry_offset(0);
                        assert_eq!(
                            u16::from_le_bytes([
                                bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET],
                                bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET + 1]
                            ]),
                            TREE_KIND_EXTENT,
                            "A 的树表第 0 条是 extent 树"
                        );
                        let inode_tree_entry =
                            tree_table_entry_offset(inode_tree_entry_index_in(bytes));
                        let field = entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET;
                        assert_eq!(
                            get_u64(bytes, field),
                            get_u64(bytes, inode_tree_entry),
                            "extent 树那条的头 ID 本来等于 inode 树条目的树 ID"
                        );
                        set_u64(bytes, field, 0);
                    },
                    true,
                );
            }),
        ),
        // I-9.27 种类 2 那一支（判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第二节 K1 证据缺口「种类 2 自等」）：
        // A 的树表里 inode 树条目（种类 2）的头 ID 从它自己的树 ID 改成树 ID + 1，重封、补到根。种类 1、9、10 比的是 inode 树条目的树 ID、
        // 不读它的头 ID，只红 I-9.27。
        (
            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
                    image,
                    TREE_KIND_INODE,
                    |own_tree_identifier, _inode_tree_identifier| own_tree_identifier,
                    |own_tree_identifier, _inode_tree_identifier| own_tree_identifier + 1,
                );
            }),
        ),
        // I-9.27 写 0 那一支（同一节证据缺口「写 0 的七种」）：A 的树表里分配记录树条目（种类 3）的头 ID 从 0 改成 inode 树条目的树 ID。
        (
            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
                    image,
                    TREE_KIND_ALLOCATION,
                    |_own_tree_identifier, _inode_tree_identifier| 0,
                    |_own_tree_identifier, inode_tree_identifier| inode_tree_identifier,
                );
            }),
        ),
        // I-9.27 种类 9 那一格（同一节证据缺口「种类 9 / 10」）：A 的树表里 `deleted_inodes` 条目的头 ID 从 inode 树条目的树 ID 改成 0。
        (
            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
                    image,
                    TREE_KIND_DELETED_INODES,
                    |_own_tree_identifier, inode_tree_identifier| inode_tree_identifier,
                    |_own_tree_identifier, _inode_tree_identifier| 0,
                );
            }),
        ),
        // I-9.27 种类 10 那一格：A 的树表里墓碑树条目的头 ID 从 inode 树条目的树 ID 改成 0。
        (
            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
            Box::new(|image: &mut MemoryPool| {
                rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
                    image,
                    TREE_KIND_TOMBSTONE,
                    |_own_tree_identifier, inode_tree_identifier| inode_tree_identifier,
                    |_own_tree_identifier, _inode_tree_identifier| 0,
                );
            }),
        ),
    ]
}

/// A 的树表（50252）里种类是 `kind` 的那一条（恰好一条）：头 ID（条目偏移 116）先核等于 `head_identifier_before(那条的树 ID, inode 树条目的树 ID)`，
/// 再改成 `head_identifier_after(…)`，重封、补到根。别的字段一个都不动。
fn rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
    image: &mut MemoryPool,
    kind: u16,
    head_identifier_before: impl Fn(u64, u64) -> u64,
    head_identifier_after: impl Fn(u64, u64) -> u64,
) {
    mutate_unit(
        image,
        TREE_TABLE,
        |bytes| {
            let entries_of_the_kind: Vec<usize> = (0..TREE_TABLE_ENTRIES_AFTER_THE_FIRST_FILE)
                .map(tree_table_entry_offset)
                .filter(|entry| {
                    u16::from_le_bytes([
                        bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET],
                        bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET + 1],
                    ]) == kind
                })
                .collect();
            let [entry] = entries_of_the_kind.as_slice() else {
                panic!("A 的树表里种类 {kind} 的条目恰好一条：{entries_of_the_kind:?}")
            };
            let own_tree_identifier = get_u64(bytes, *entry);
            let inode_tree_identifier = get_u64(
                bytes,
                tree_table_entry_offset(inode_tree_entry_index_in(bytes)),
            );
            let field = entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET;
            assert_eq!(
                get_u64(bytes, field),
                head_identifier_before(own_tree_identifier, inode_tree_identifier),
                "种类 {kind} 那条的头 ID 本来的取值"
            );
            set_u64(
                bytes,
                field,
                head_identifier_after(own_tree_identifier, inode_tree_identifier),
            );
        },
        true,
    );
}

/// 记录新根段里实例表指针的起点：新根段 99 起，树表指针 86、映射根指针 86、树 ID 水位 8、F 8 之后（D23（journal 的角色与格式） 已定项 15）。
const JOURNAL_RECORD_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET: usize = 99 + 188;

/// 环里从计数器 1 起读到第一格没有 magic 的那条为止：同 (实例, txg) 的根读得出（根槽 magic 在、实例表指针 extent 偏移已改成 1）的记录，
/// 新根段里实例表指针的 extent 偏移也 0 → 1；每一条的反向链按前一条（改后的）记录头重算，头校验和重封，两盘各一份。
/// 记录不经 `JournalRecord::parse` 改：实现的读者拒 extent 偏移非 0 的指针，解不出这样的记录。
/// 这份镜像上只有实例 1 写过记录（反向链在实例内逐条接），迭代上界是 [`RECORDS_READ_BACK_AT_MOST`]。
fn set_the_extent_offset_of_the_instance_table_pointer_in_the_records_of_these_roots(
    image: &mut MemoryPool,
) {
    let roots_with_the_extent_offset_set: std::collections::BTreeSet<(u32, u64)> = root_slots()
        .into_iter()
        .map(|(device, offset)| read(image, device, offset, 512))
        .filter(|bytes| bytes[..4] == singlefs_core::root_record::ROOT_MAGIC)
        .map(|bytes| (get_u32(&bytes, 24), get_u64(&bytes, 28)))
        .collect();
    for device in DEVICES {
        let mut previous_record: Option<Vec<u8>> = None;
        for counter in 1..=RECORDS_READ_BACK_AT_MOST {
            let offset = record_offset(counter, JOURNAL_RING_DEFAULT_BYTES).0;
            let mut bytes = read(image, device, offset, 4096);
            if bytes[..4] != singlefs_core::journal::JOURNAL_MAGIC {
                break;
            }
            let identity = (
                get_u32(&bytes, JOURNAL_RECORD_INSTANCE_OFFSET),
                get_u64(&bytes, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET),
            );
            if roots_with_the_extent_offset_set.contains(&identity) {
                let field = JOURNAL_RECORD_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
                    + POINTER_EXTENT_OFFSET_FIELD_OFFSET;
                bytes[field] = 1;
            }
            if let Some(previous) = &previous_record {
                set_u32(
                    &mut bytes,
                    JOURNAL_RECORD_BACK_CHAIN_OFFSET,
                    back_chain_of(previous),
                );
            }
            let digest = wide_checksum_with_field_zeroed(&bytes, 4096, 46);
            bytes[46..78].copy_from_slice(&digest);
            write(image, device, offset, &bytes);
            previous_record = Some(bytes);
        }
    }
}

/// 树表条目里头 ID 那 8 字节的偏移：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8 + 诞生 txg 8
/// （D8（核心索引结构） 已定项 8 的字段表）。
const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8 + 8;
/// 树的种类码里 extent 树那一个（D8（核心索引结构） 已定项 9 的登记表）。
const TREE_KIND_EXTENT: u16 = 1;

/// 上面那批坏镜像各自**只**红在自己那一条上；干净镜像上那几条要真被评估过且成立（不是「不适用」）。
#[test]
fn each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant() {
    let clean = build_pool("known-bad-pointer-journal-and-generation").memory_pool();
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    for (invariant, _) in known_bad_images_of_pointer_journal_and_generation_fields() {
        assert_eq!(
            verdict(&clean, invariant),
            InvariantVerdict::Holds,
            "干净镜像上 {invariant} 要真被评估过且成立：{verdicts_on_the_clean_image:?}"
        );
    }
    for (invariant, mutation) in known_bad_images_of_pointer_journal_and_generation_fields() {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [invariant],
            "判红的该只有 {invariant}：{verdicts:?}"
        );
    }
}

/// 盘 0 两个系统配置槽：(偏移, 世代号)，被择的那一槽（世代号大的）在前。
fn system_configuration_slots_of_device_zero_chosen_first(image: &MemoryPool) -> [(u64, u64); 2] {
    let slots = [0, system_configuration_slot_one_offset(image, 0)].map(|slot_offset| {
        (
            slot_offset,
            get_u64(
                &read(image, 0, slot_offset, 4096),
                SYSTEM_CONFIGURATION_SLOT_GENERATION_OFFSET,
            ),
        )
    });
    if slots[1].1 > slots[0].1 {
        [slots[1], slots[0]]
    } else {
        slots
    }
}

/// I-7.16 只破差值那一格（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 13 行）：盘 0 没被择的那一槽世代号
/// 改成被择那个减 3（仍比它小、择哪一槽不变，与被择那个奇偶相反——住的槽对），整槽校验和重封：两槽相差 3。
fn lower_the_generation_of_the_unchosen_slot_of_device_zero_by_three(image: &mut MemoryPool) {
    let [(_, chosen_generation), (unchosen_offset, unchosen_generation)] =
        system_configuration_slots_of_device_zero_chosen_first(image);
    assert_eq!(
        chosen_generation,
        unchosen_generation + 1,
        "干净镜像上盘 0 两槽世代号相差 1"
    );
    assert!(
        chosen_generation >= 3,
        "盘 0 被择那个槽的世代号 {chosen_generation} ≥ 3，减 3 不越过 0"
    );
    mutate_system_configuration_slot(image, 0, unchosen_offset, |bytes| {
        set_u64(
            bytes,
            SYSTEM_CONFIGURATION_SLOT_GENERATION_OFFSET,
            chosen_generation - 3,
        );
    });
}

/// I-7.16 只破奇偶那一格：盘 0 被择那一槽世代号加 1、没被择那一槽改成原先被择那个世代号（择哪一槽不变，两槽仍相差 1），
/// 整槽校验和各自重封：两槽的世代号都 ≥ 2 而都住在对方的槽里。
fn swap_the_parities_of_the_two_system_configuration_slots_of_device_zero(image: &mut MemoryPool) {
    let [(chosen_offset, chosen_generation), (unchosen_offset, unchosen_generation)] =
        system_configuration_slots_of_device_zero_chosen_first(image);
    assert_eq!(
        chosen_generation,
        unchosen_generation + 1,
        "干净镜像上盘 0 两槽世代号相差 1"
    );
    assert!(
        unchosen_generation >= 1,
        "没被择那个槽改成 {chosen_generation} 之后也 ≥ 2，奇偶那一格判得到"
    );
    mutate_system_configuration_slot(image, 0, chosen_offset, |bytes| {
        set_u64(
            bytes,
            SYSTEM_CONFIGURATION_SLOT_GENERATION_OFFSET,
            chosen_generation + 1,
        );
    });
    mutate_system_configuration_slot(image, 0, unchosen_offset, |bytes| {
        set_u64(
            bytes,
            SYSTEM_CONFIGURATION_SLOT_GENERATION_OFFSET,
            chosen_generation,
        );
    });
}

/// I-7.16 的两份坏镜像，交给「每条不变量至少有一份坏镜像」那一处数清单；判定各在一条用例里。
fn known_bad_images_of_the_system_configuration_slot_generations() -> Vec<(&'static str, Mutation)>
{
    vec![
        (
            SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
            Box::new(lower_the_generation_of_the_unchosen_slot_of_device_zero_by_three),
        ),
        (
            SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
            Box::new(swap_the_parities_of_the_two_system_configuration_slots_of_device_zero),
        ),
    ]
}

/// 一份 I-7.16 坏镜像只红 I-7.16，第一处违例落在 `cell_in_the_detail` 说的那一格上；干净镜像上 I-7.16 真被评估过且成立。
fn assert_only_the_rotation_invariant_reddens_on_the_cell(
    tag: &str,
    mutation: impl Fn(&mut MemoryPool),
    cell_in_the_detail: &str,
) {
    let clean = build_pool(tag).memory_pool();
    assert_eq!(
        verdict(
            &clean,
            SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION
        ),
        InvariantVerdict::Holds,
        "干净镜像上 I-7.16 真被评估过且成立"
    );
    let mut image = clean.clone();
    mutation(&mut image);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION],
        "判红的该只有 I-7.16：{verdicts:?}"
    );
    match verdict(
        &image,
        SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
    ) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains(cell_in_the_detail),
            "I-7.16 红在「{cell_in_the_detail}」那一格：{detail}"
        ),
        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
            panic!("I-7.16 该判违例，实际 {other:?}")
        }
    }
}

/// 两槽相差 3、各自住的槽都对：只红 I-7.16 的差值那一格（奇偶那一格照旧成立）。
#[test]
fn system_configuration_slot_generations_three_apart_in_their_own_slots_redden_only_the_rotation_invariant_on_the_difference(
) {
    assert_only_the_rotation_invariant_reddens_on_the_cell(
        "known-bad-system-configuration-generation-difference",
        lower_the_generation_of_the_unchosen_slot_of_device_zero_by_three,
        "相差不是 1",
    );
}

/// 两槽相差 1、都住在对方的槽里：只红 I-7.16 的奇偶那一格（差值那一格照旧成立）。
#[test]
fn system_configuration_slot_generations_one_apart_in_each_others_slot_redden_only_the_rotation_invariant_on_the_parity(
) {
    assert_only_the_rotation_invariant_reddens_on_the_cell(
        "known-bad-system-configuration-generation-parity",
        swap_the_parities_of_the_two_system_configuration_slots_of_device_zero,
        "该住槽",
    );
}

/// I-3.12（实例表行 T 低于引用它的根的 txg） 的坏镜像，建在「可写挂载施加了一条记录」那一份上：实例表里实例 1 那一行 (1, 5, 3)
/// 的 T 5 → 6（= 写行发布那条根的 txg），重封、补到引用它的根槽。
fn known_bad_images_of_the_instance_table_rows() -> Vec<(&'static str, Mutation)> {
    vec![(
        INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
        Box::new(|image: &mut MemoryPool| {
            const INSTANCE_TABLE_ROW_INSTANCE_OFFSET: usize = 1;
            const INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET: usize = 5;
            const INSTANCE_TABLE_ROW_BYTES: usize = 88;
            let (device, offset) = root_slot_of(CheckpointTxg(7));
            let newest_root = read(image, device, offset, 512);
            assert_eq!(get_u64(&newest_root, 28), 7, "最新的根是暖机 txg 7");
            let location = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + 50;
            let mut slot_bytes = [0u8; 8];
            slot_bytes[..6].copy_from_slice(&newest_root[location + 4..location + 10]);
            let instance_table_slot = u64::from_le_bytes(slot_bytes);
            mutate_unit_of(
                image,
                &[(instance_table_slot, 32768)],
                instance_table_slot,
                |bytes| {
                    let row_count = usize::from(u16::from_le_bytes([bytes[69], bytes[70]]));
                    let row = (0..row_count)
                        .map(|index| {
                            INSTANCE_TABLE_FIRST_ROW_START + INSTANCE_TABLE_ROW_BYTES * index
                        })
                        .find(|row| {
                            bytes[*row] == 0
                                && get_u32(bytes, row + INSTANCE_TABLE_ROW_INSTANCE_OFFSET) == 1
                        })
                        .expect("实例表里有实例 1 那一行");
                    assert_eq!(
                        get_u64(bytes, row + INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET),
                        5,
                        "那一行是 (1, 5, 3)"
                    );
                    set_u64(bytes, row + INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET, 6);
                },
                true,
            );
        }),
    )]
}

/// 实例表单元（码 3）的记录区起点。
const INSTANCE_TABLE_FIRST_ROW_START: usize = 136;

/// 可写挂载施加过一条记录那一份上：I-3.12 真被评估过且成立；行 (1, 5, 3) 的 T 抬到 6 之后**只**红 I-3.12。
#[test]
fn an_instance_table_row_whose_published_txg_reaches_the_root_referencing_it_reddens_only_the_instance_table_row_invariant(
) {
    let clean = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-bad-instance-table-rows",
    )
    .image;
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    assert_eq!(
        violated_invariants(&verdicts_on_the_clean_image),
        Vec::<&str>::new(),
        "干净镜像上一条都不红：{verdicts_on_the_clean_image:?}"
    );
    assert_eq!(
        verdict(
            &clean,
            INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM
        ),
        InvariantVerdict::Holds,
        "干净镜像上 I-3.12 要真被评估过且成立"
    );
    for (invariant, mutation) in known_bad_images_of_the_instance_table_rows() {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [invariant],
            "判红的该只有 {invariant}：{verdicts:?}"
        );
    }
}

/// 系统配置字段表里在飞记录数上限（4 字节）、最坏占用（8 字节）、安全系数 F（4 字节）、io_min（4 字节）的偏移
/// （`.claude/kb/layout/01-first-txn.md` 一：环长 333 + 8 → 记录尺寸 341 + 4 → 在飞上限 345 → 最坏占用 349 → F 357 → R 361；io_min 在 425）。
const SYSTEM_CONFIGURATION_IN_FLIGHT_RECORD_LIMIT_OFFSET: usize = 345;
const SYSTEM_CONFIGURATION_WORST_CASE_OCCUPANCY_OFFSET: usize = 349;
const SYSTEM_CONFIGURATION_SAFETY_FACTOR_OFFSET: usize = 357;
const SYSTEM_CONFIGURATION_MINIMUM_INPUT_OUTPUT_OFFSET: usize = 425;

/// 根槽距 4096 的两倍：io_min 声明成它，判定宽度跟着成 8192，槽 1（区域起点 + 4096）不是它的整数倍。
const MINIMUM_INPUT_OUTPUT_TWICE_THE_ROOT_SLOT_SPACING: u32 = 8192;

/// 全池四个系统配置槽（两块盘各两槽）：(盘, 偏移)。槽 1 的偏移按改之前槽 0 记的槽距取。
fn every_system_configuration_slot(image: &MemoryPool) -> Vec<(u32, u64)> {
    DEVICES
        .iter()
        .flat_map(|device| {
            [
                (*device, 0),
                (
                    *device,
                    system_configuration_slot_one_offset(image, *device),
                ),
            ]
        })
        .collect()
}

/// I-8.1（环几何够大）与 I-7.5（根槽按判定宽度对齐）的坏镜像，建在**写完新池新建文件**那份干净镜像上：每份改系统配置的字段、重封整槽校验和，
/// 只红它自己那一条（`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 4、5 行）。
fn known_bad_images_of_the_ring_geometry_and_the_root_slot_alignment(
) -> Vec<(&'static str, Mutation)> {
    vec![
        // I-8.1 三个字段等式各一份，都只改盘 0 槽 0、重封，另外两个等式照样成立：
        // ① 在飞上限加 1、最坏占用跟着改成新上限 × 4096——只有「在飞上限 = ⌊环长 ÷ 4096 ÷ F⌋」不成立；
        (
            JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
            Box::new(|image: &mut MemoryPool| {
                mutate_system_configuration_slot(image, 0, 0, |bytes| {
                    let limit =
                        get_u32(bytes, SYSTEM_CONFIGURATION_IN_FLIGHT_RECORD_LIMIT_OFFSET) + 1;
                    set_u32(
                        bytes,
                        SYSTEM_CONFIGURATION_IN_FLIGHT_RECORD_LIMIT_OFFSET,
                        limit,
                    );
                    set_u64(
                        bytes,
                        SYSTEM_CONFIGURATION_WORST_CASE_OCCUPANCY_OFFSET,
                        u64::from(limit) * 4096,
                    );
                });
            }),
        ),
        // ② 最坏占用多 4096——只有「最坏占用 = 在飞上限 × 4096」不成立；
        (
            JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
            Box::new(|image: &mut MemoryPool| {
                mutate_system_configuration_slot(image, 0, 0, |bytes| {
                    let worst = get_u64(bytes, SYSTEM_CONFIGURATION_WORST_CASE_OCCUPANCY_OFFSET);
                    set_u64(
                        bytes,
                        SYSTEM_CONFIGURATION_WORST_CASE_OCCUPANCY_OFFSET,
                        worst + 4096,
                    );
                });
            }),
        ),
        // ③ F 写 1、在飞上限与最坏占用按 F = 1 算——两个等式成立，只有「F = 格式常量」那一格不成立。
        (
            JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
            Box::new(|image: &mut MemoryPool| {
                mutate_system_configuration_slot(image, 0, 0, |bytes| {
                    set_the_safety_factor_and_the_two_fields_computed_from_it(bytes, 1);
                });
            }),
        ),
        // ④ ⑤ 四个槽的 F 都写 2（读者下界那一格成立）、都写 4，在飞上限与最坏占用按槽里的 F 算——两个等式成立，只有「F = 格式常量」
        // 那一格不成立（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 3 行，攻方 C2-1 那两份镜像）。
        (
            JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
            Box::new(|image: &mut MemoryPool| {
                for (device, offset) in every_system_configuration_slot(image) {
                    mutate_system_configuration_slot(image, device, offset, |bytes| {
                        set_the_safety_factor_and_the_two_fields_computed_from_it(bytes, 2);
                    });
                }
            }),
        ),
        (
            JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
            Box::new(|image: &mut MemoryPool| {
                for (device, offset) in every_system_configuration_slot(image) {
                    mutate_system_configuration_slot(image, device, offset, |bytes| {
                        set_the_safety_factor_and_the_two_fields_computed_from_it(bytes, 4);
                    });
                }
            }),
        ),
        // ⑥ 只改盘 1 没被择的那一槽：F 写 2，另两个字段按它算。择到的槽照旧是 F = 3 那一份，那一槽照样整池拒。
        (
            JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
            Box::new(|image: &mut MemoryPool| {
                let slots = [0, system_configuration_slot_one_offset(image, 1)].map(|offset| {
                    (
                        offset,
                        get_u64(
                            &read(image, 1, offset, 4096),
                            SYSTEM_CONFIGURATION_SLOT_GENERATION_OFFSET,
                        ),
                    )
                });
                let unchosen_offset = if slots[1].1 > slots[0].1 {
                    slots[0].0
                } else {
                    slots[1].0
                };
                mutate_system_configuration_slot(image, 1, unchosen_offset, |bytes| {
                    set_the_safety_factor_and_the_two_fields_computed_from_it(bytes, 2);
                });
            }),
        ),
        // I-7.5：四个槽的 io_min 都声明成 8192，判定宽度成 8192，根环里每个奇数号槽（区域起点 + 4096 × 奇数）不对齐。
        (
            ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
            Box::new(|image: &mut MemoryPool| {
                for (device, offset) in every_system_configuration_slot(image) {
                    mutate_system_configuration_slot(image, device, offset, |bytes| {
                        set_u32(
                            bytes,
                            SYSTEM_CONFIGURATION_MINIMUM_INPUT_OUTPUT_OFFSET,
                            MINIMUM_INPUT_OUTPUT_TWICE_THE_ROOT_SLOT_SPACING,
                        );
                    });
                }
            }),
        ),
        // I-7.5 异构池那一格（`research/prompts/m3-batch4-code-r1-sonnet-output.md` 第五节盲区第 8 条）：只有盘 1 两槽的 io_min 声明成 8192，
        // 盘 0 仍是 512——判定宽度取各盘里最大的那个（8192），奇数号槽照样不对齐。
        (
            ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
            Box::new(|image: &mut MemoryPool| {
                for offset in [0, system_configuration_slot_one_offset(image, 1)] {
                    mutate_system_configuration_slot(image, 1, offset, |bytes| {
                        set_u32(
                            bytes,
                            SYSTEM_CONFIGURATION_MINIMUM_INPUT_OUTPUT_OFFSET,
                            MINIMUM_INPUT_OUTPUT_TWICE_THE_ROOT_SLOT_SPACING,
                        );
                    });
                }
            }),
        ),
    ]
}

/// 系统配置槽里 F 写成 `safety_factor`，在飞上限与最坏占用按它算（⌊环长 ÷ 4096 ÷ F⌋、上限 × 4096）：I-8.1 两个等式照样成立。
fn set_the_safety_factor_and_the_two_fields_computed_from_it(bytes: &mut [u8], safety_factor: u32) {
    let ring_records = JOURNAL_RING_BYTES_OF_THE_POOL / 4096;
    let in_flight_record_limit = ring_records / u64::from(safety_factor);
    set_u32(
        bytes,
        SYSTEM_CONFIGURATION_SAFETY_FACTOR_OFFSET,
        safety_factor,
    );
    set_u32(
        bytes,
        SYSTEM_CONFIGURATION_IN_FLIGHT_RECORD_LIMIT_OFFSET,
        u32::try_from(in_flight_record_limit).expect("环里 196 608 条，装得进 4 字节"),
    );
    set_u64(
        bytes,
        SYSTEM_CONFIGURATION_WORST_CASE_OCCUPANCY_OFFSET,
        in_flight_record_limit * 4096,
    );
}

/// I-8.1 三个字段等式有一格不等的镜像，实现整池拒绝挂载（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 3、4 行）：
/// 上面那批里 I-8.1 的六份（在飞上限、最坏占用、F = 1 / 2 / 4、只坏盘 1 没被择的那一槽）每一份都只红 I-8.1，别的每一条
/// （连同 I-7.13）都报不适用——与 I-7.13 那一支同口径，checker 不替挂不上的池作保。
#[test]
fn journal_ring_geometry_fields_that_disagree_redden_only_the_ring_geometry_invariant_and_every_other_is_not_applicable(
) {
    let clean =
        build_pool("journal-ring-geometry-disagreement-is-a-whole-pool-refusal").memory_pool();
    let mut images_judged = 0;
    for (invariant, mutation) in known_bad_images_of_the_ring_geometry_and_the_root_slot_alignment()
    {
        if invariant != JOURNAL_RING_GEOMETRY_FIELDS_AGREE {
            continue;
        }
        images_judged += 1;
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [JOURNAL_RING_GEOMETRY_FIELDS_AGREE],
            "判红的该只有 I-8.1：{verdicts:?}"
        );
        let judged_other_than_not_applicable: Vec<&(&str, InvariantVerdict)> = verdicts
            .iter()
            .filter(|(name, found)| {
                *name != JOURNAL_RING_GEOMETRY_FIELDS_AGREE
                    && !matches!(found, InvariantVerdict::NotApplicable(_))
            })
            .collect();
        assert!(
            judged_other_than_not_applicable.is_empty(),
            "I-8.1 三格有一格不等时别的每一条都该报不适用：{judged_other_than_not_applicable:?}"
        );
    }
    assert_eq!(images_judged, 6, "I-8.1 三个字段等式那几份坏镜像六份");
}

/// 上面那批坏镜像各自**只**红在自己那一条上；干净镜像上 I-8.1 与 I-7.5 真被评估过且成立。
#[test]
fn each_ring_geometry_and_root_slot_alignment_bad_image_reddens_only_its_own_invariant() {
    let clean = build_pool("known-bad-ring-geometry-and-root-slot-alignment").memory_pool();
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    for invariant in [
        JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
        ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
    ] {
        assert_eq!(
            verdict(&clean, invariant),
            InvariantVerdict::Holds,
            "干净镜像上 {invariant} 要真被评估过且成立：{verdicts_on_the_clean_image:?}"
        );
    }
    for (invariant, mutation) in known_bad_images_of_the_ring_geometry_and_the_root_slot_alignment()
    {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [invariant],
            "判红的该只有 {invariant}：{verdicts:?}"
        );
    }
}

/// 翻一个系统配置槽里的一个字节、不重封：magic 照旧，整槽校验和对不上，这一槽不自证。
fn break_the_system_configuration_slot_without_resealing(
    image: &mut MemoryPool,
    device: u32,
    offset: u64,
) {
    const BYTE_INSIDE_THE_CHECKSUMMED_SLOT: usize = 300;
    let mut bytes = read(image, device, offset, 4096);
    bytes[BYTE_INSIDE_THE_CHECKSUMMED_SLOT] ^= 0xff;
    write(image, device, offset, &bytes);
}

/// I-8.5（tail 槽有效集非空，V-池）的坏镜像：全池四个系统配置槽各翻一个字节、不重封——一个自证过的槽都不剩。
fn known_bad_images_of_the_tail_slot_witnesses() -> Vec<(&'static str, Mutation)> {
    vec![(
        SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
        Box::new(|image: &mut MemoryPool| {
            for (device, offset) in every_system_configuration_slot(image) {
                break_the_system_configuration_slot_without_resealing(image, device, offset);
            }
        }),
    )]
}

/// 全池皆废：I-8.5 判红，别的每一条都报不适用（这不是一个挂得上的镜像）；干净镜像上 I-8.5 真被评估过且成立。
#[test]
fn no_self_verified_system_configuration_slot_on_any_device_reddens_the_tail_slot_invariant_and_every_other_is_not_applicable(
) {
    let clean = build_pool("known-bad-tail-slot-witnesses").memory_pool();
    assert_eq!(
        verdict(
            &clean,
            SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
        ),
        InvariantVerdict::Holds,
        "干净镜像上 I-8.5 要真被评估过且成立"
    );
    for (invariant, mutation) in known_bad_images_of_the_tail_slot_witnesses() {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [invariant],
            "判红的该只有 {invariant}：{verdicts:?}"
        );
        let judged_other_than_not_applicable: Vec<&(&str, InvariantVerdict)> = verdicts
            .iter()
            .filter(|(name, found)| {
                *name != invariant && !matches!(found, InvariantVerdict::NotApplicable(_))
            })
            .collect();
        assert!(
            judged_other_than_not_applicable.is_empty(),
            "全池皆废时别的每一条都该报不适用：{judged_other_than_not_applicable:?}"
        );
    }
}

/// 单盘全废不进违例通道：盘 1 两槽都翻坏，I-8.5 照样成立（盘 0 有自证过的槽），旁路报告点名盘 1 两槽都废、可写挂载会拒。
/// 只坏盘 0 的槽 0 时盘 0 少一槽自证过，旁路报告只报它可写挂载会拒；干净镜像上旁路报告两样都空。
#[test]
fn a_device_short_of_self_verified_system_configuration_slots_is_reported_aside_while_the_tail_slot_invariant_holds(
) {
    use singlefs_checker::image::{
        system_configuration_redundancy_report, SystemConfigurationRedundancyReport,
    };
    let clean = build_pool("tail-slot-witnesses-reported-aside").memory_pool();
    assert_eq!(
        system_configuration_redundancy_report(&clean),
        SystemConfigurationRedundancyReport {
            devices_without_a_self_verified_slot: Vec::new(),
            devices_whose_writable_mount_would_be_refused: Vec::new(),
        },
        "干净镜像上每块盘两槽都自证过"
    );
    let mut device_one_lost = clean.clone();
    let slot_one_offset_of_device_one = system_configuration_slot_one_offset(&clean, 1);
    for offset in [0, slot_one_offset_of_device_one] {
        break_the_system_configuration_slot_without_resealing(&mut device_one_lost, 1, offset);
    }
    assert_eq!(
        verdict(
            &device_one_lost,
            SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
        ),
        InvariantVerdict::Holds,
        "盘 0 两槽都还自证过：单盘全废不判违例"
    );
    assert_eq!(
        system_configuration_redundancy_report(&device_one_lost),
        SystemConfigurationRedundancyReport {
            devices_without_a_self_verified_slot: vec![1],
            devices_whose_writable_mount_would_be_refused: vec![1],
        }
    );
    let mut one_slot_of_device_zero_lost = clean.clone();
    break_the_system_configuration_slot_without_resealing(&mut one_slot_of_device_zero_lost, 0, 0);
    assert_eq!(
        system_configuration_redundancy_report(&one_slot_of_device_zero_lost),
        SystemConfigurationRedundancyReport {
            devices_without_a_self_verified_slot: Vec::new(),
            devices_whose_writable_mount_would_be_refused: vec![0],
        }
    );
}

/// 干净镜像（最新根之上没有记录）上 I-4.1 报的不适用理由。
const NOTHING_APPLIED_ABOVE_THE_CHOSEN_ROOT: &str =
    "所选根之上一次发布都不施加：重放之后那一版就是所选根，它可不可遍历归 I-7.2";

/// 记录头里新根段（99 起）的树表指针（新根段 [0..86)）两条位置条目的整单元校验和（指针 50 + 10、64 + 10）。
const JOURNAL_RECORD_NEW_TREE_TABLE_POINTER_CHECKSUMS: [usize; 2] = [99 + 50 + 10, 99 + 64 + 10];

/// 发布 B 那条记录的计数器（新池新建文件写到 3，B 是第 4 条）。
const COUNTER_OF_THE_OVERWRITE_RECORD: u64 = 4;

/// I-4.1（重放后树可遍历）的坏镜像，建在「发布 B 的记录两盘都落了、根槽没落」那份崩溃镜像上：B 那条记录新根段里树表指针的两条位置条目的
/// 校验和各加 1、重封记录头（新根段在头里，载荷校验和不动）。点名项一个字节不动 ⇒ 点名单元照样两份都验得过、记录照样施加，
/// 施加出来那一版的树表两份都读不对。
fn known_bad_images_of_the_replayed_version() -> Vec<(&'static str, Mutation)> {
    vec![(
        TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
        Box::new(|image: &mut MemoryPool| {
            for device in DEVICES {
                break_the_new_tree_table_pointer_of_the_overwrite_record_on(image, device);
            }
        }),
    )]
}

/// 崩在发布 B 的记录落了、根槽没落：下一次挂载施加 B 那条记录，重放之后那一版走得完 ⇒ I-4.1 真被评估过且成立、一条都不红；
/// B 的新根段里树表指针读不对 ⇒ **只**红 I-4.1。
#[test]
fn a_replayed_version_whose_tree_table_does_not_read_back_reddens_only_the_replayed_version_invariant(
) {
    let clean =
        crash_image_with_the_overwrite_record_but_not_its_root("known-bad-replayed-version");
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    assert_eq!(
        violated_invariants(&verdicts_on_the_clean_image),
        Vec::<&str>::new(),
        "干净的崩溃镜像上一条都不红：{verdicts_on_the_clean_image:?}"
    );
    assert_eq!(
        verdict(&clean, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE),
        InvariantVerdict::Holds,
        "B 那条记录被重放、施加出来的那一版走得完：I-4.1 真被评估过且成立"
    );
    for (invariant, mutation) in known_bad_images_of_the_replayed_version() {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            [invariant],
            "判红的该只有 {invariant}：{verdicts:?}"
        );
    }
}

/// 起点含介质损坏不在 I-4.1 判的合法镜像里（`research/prompts/m3-inv-a-r1-main-verification.md` 第四节用户定案第 1 条）：
/// B 那条记录第一个点名单元在盘 1 上那一份翻一个字节，前缀停在 B 那条（两份都验过才施加）⇒ I-4.1 报不适用，不报成立也不报违例。
#[test]
fn a_replay_stopped_by_a_named_unit_that_does_not_verify_leaves_the_replayed_version_invariant_not_applicable(
) {
    let mut image =
        crash_image_with_the_overwrite_record_but_not_its_root("replay-stopped-by-a-named-unit");
    let offset = singlefs_core::journal::record_offset(
        COUNTER_OF_THE_OVERWRITE_RECORD,
        singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
    )
    .0;
    let record = read(&image, 0, offset, 4096);
    assert!(
        get_u32(&record, JOURNAL_RECORD_NAMED_COUNT_OFFSET) > 0,
        "B 那条记录有点名项"
    );
    let second_location = JOURNAL_RECORD_HEADER_BYTES + 14;
    let named_device = get_u32(&record, second_location);
    let mut named_slot_bytes = [0u8; 8];
    named_slot_bytes[..6].copy_from_slice(&record[second_location + 4..second_location + 10]);
    let named_slot = u64::from_le_bytes(named_slot_bytes);
    let mut unit = read(&image, named_device, named_slot * SLOT, 16384);
    unit[1000] ^= 0xff;
    write(&mut image, named_device, named_slot * SLOT, &unit);
    match verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE) {
        InvariantVerdict::NotApplicable(reason) => assert!(
            reason.contains("介质损坏"),
            "报不适用的理由要说是起点含介质损坏：{reason}"
        ),
        other @ (InvariantVerdict::Holds | InvariantVerdict::Violated(_)) => {
            panic!("点名单元一份验不过时 I-4.1 该报不适用，实际 {other:?}")
        }
    }
}

/// 「发布 B 的记录两盘都落了、根槽没落」那份崩溃镜像上 B 那条记录照恢复的读法施加、I-4.1 真被评估过且成立；`change` 改过之后，
/// D23（journal 的角色与格式） 已定项 14 的前缀规则里有一条该把 B 那条挡在前缀之外 ⇒ 一次发布都不施加，I-4.1 报不适用
/// （`research/prompts/m3-batch4-code-r1-sonnet-output.md` 第五节盲区第 10 条：重放规则各配一份只靠它挡住的镜像）。
fn assert_the_replay_stops_before_the_overwrite_record(
    tag: &str,
    change: impl Fn(&mut MemoryPool),
) {
    let mut image = crash_image_with_the_overwrite_record_but_not_its_root(tag);
    assert_eq!(
        verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE),
        InvariantVerdict::Holds,
        "改之前 B 那条记录被重放、施加出来的那一版走得完"
    );
    change(&mut image);
    assert_eq!(
        verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE),
        InvariantVerdict::NotApplicable(NOTHING_APPLIED_ABOVE_THE_CHOSEN_ROOT),
        "B 那条记录该被挡在重放前缀之外、一次发布都不施加"
    );
}

/// 带末条标志的 B 那条之后，同一次发布（实例 1、txg 4）还有一条读得出的记录（计数器 5，序号 2，反向链接在 B 上）：
/// 读者规则「带末条标志的那条之后还有记录」断在 B 那条，B 不施加。
#[test]
fn a_readable_record_of_the_same_publish_after_the_last_record_flag_stops_the_replay_before_it() {
    assert_the_replay_stops_before_the_overwrite_record(
        "replay-stops-at-a-record-after-the-last-record-flag",
        |image| {
            let offset_of_the_overwrite_record =
                record_offset(COUNTER_OF_THE_OVERWRITE_RECORD, JOURNAL_RING_DEFAULT_BYTES).0;
            let offset_after_it = record_offset(
                COUNTER_OF_THE_OVERWRITE_RECORD + 1,
                JOURNAL_RING_DEFAULT_BYTES,
            )
            .0;
            for device in DEVICES {
                let overwrite_bytes = read(image, device, offset_of_the_overwrite_record, 4096);
                let overwrite = JournalRecord::parse(
                    &overwrite_bytes,
                    get_u64(
                        &overwrite_bytes,
                        JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET,
                    ),
                )
                .expect("B 那条记录自证过");
                let mut follower = overwrite.clone();
                follower.counter = COUNTER_OF_THE_OVERWRITE_RECORD + 1;
                follower.ordinal_within_publish = JournalRecordOrdinalWithinPublish(2);
                follower.place_in_publish =
                    JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow;
                follower.is_commit = false;
                follower.named = Vec::new();
                follower.back_chain = back_chain_of(&overwrite_bytes);
                write(image, device, offset_after_it, &follower.to_bytes());
            }
        },
    );
}

/// B 那条（带末条标志）摘掉提交标记：已定项 14 第三条「末条要带提交标记」不成，B 不施加。
#[test]
fn a_last_record_without_its_commit_marker_stops_the_replay_before_it() {
    assert_the_replay_stops_before_the_overwrite_record(
        "replay-stops-at-a-last-record-without-commit-marker",
        |image| {
            rewrite_journal_record_and_rechain_the_records_after_it(
                image,
                COUNTER_OF_THE_OVERWRITE_RECORD,
                |record| record.is_commit = false,
            );
        },
    );
}

/// B 那条（一次发布的第一条）本次发布内序号 1 → 2：已定项 4 读者规则「一次发布的首条序号不是 1」断在这一条，B 不施加。
#[test]
fn a_first_record_of_a_publish_whose_ordinal_is_not_one_stops_the_replay_before_it() {
    assert_the_replay_stops_before_the_overwrite_record(
        "replay-stops-at-a-publish-starting-past-ordinal-one",
        |image| {
            rewrite_journal_record_and_rechain_the_records_after_it(
                image,
                COUNTER_OF_THE_OVERWRITE_RECORD,
                |record| record.ordinal_within_publish = JournalRecordOrdinalWithinPublish(2),
            );
        },
    );
}

/// B 那条的反向链翻一位：同实例逻辑前一条（计数器 3，A 那条）在盘上、链值对不上（I-8.6「不等的记录不进重放前缀」），B 不施加。
#[test]
fn a_record_whose_back_chain_does_not_match_its_predecessor_stops_the_replay_before_it() {
    assert_the_replay_stops_before_the_overwrite_record(
        "replay-stops-at-a-broken-back-chain",
        |image| {
            rewrite_journal_record_and_rechain_the_records_after_it(
                image,
                COUNTER_OF_THE_OVERWRITE_RECORD,
                |record| record.back_chain ^= 1,
            );
        },
    );
}

/// B 那条挪到计数器 5（环里那一格），计数器 4 那一格两盘都清零：链首锚在 A 那条（计数器 3）之后要接 4，断号即止，B 不施加。
#[test]
fn a_record_one_counter_past_the_anchor_successor_stops_the_replay_before_it() {
    assert_the_replay_stops_before_the_overwrite_record("replay-stops-at-a-counter-gap", |image| {
        let offset_of_the_overwrite_record =
            record_offset(COUNTER_OF_THE_OVERWRITE_RECORD, JOURNAL_RING_DEFAULT_BYTES).0;
        let offset_after_it = record_offset(
            COUNTER_OF_THE_OVERWRITE_RECORD + 1,
            JOURNAL_RING_DEFAULT_BYTES,
        )
        .0;
        for device in DEVICES {
            let overwrite_bytes = read(image, device, offset_of_the_overwrite_record, 4096);
            let mut moved = JournalRecord::parse(
                &overwrite_bytes,
                get_u64(
                    &overwrite_bytes,
                    JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET,
                ),
            )
            .expect("B 那条记录自证过");
            moved.counter = COUNTER_OF_THE_OVERWRITE_RECORD + 1;
            write(image, device, offset_after_it, &moved.to_bytes());
            write(image, device, offset_of_the_overwrite_record, &[0u8; 4096]);
        }
    });
}

/// B 那条记录新根段里树表指针两条位置条目的校验和各加 1、重封记录头（只改 `device` 那一份）。
fn break_the_new_tree_table_pointer_of_the_overwrite_record_on(
    image: &mut MemoryPool,
    device: u32,
) {
    let offset = record_offset(COUNTER_OF_THE_OVERWRITE_RECORD, JOURNAL_RING_DEFAULT_BYTES).0;
    let mut record = read(image, device, offset, 4096);
    assert_eq!(
        get_u64(&record, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET),
        4,
        "盘 {device} 计数器 4 那条记录是发布 B（txg 4）"
    );
    for checksum_offset in JOURNAL_RECORD_NEW_TREE_TABLE_POINTER_CHECKSUMS {
        let checksum = get_u32(&record, checksum_offset);
        set_u32(&mut record, checksum_offset, checksum.wrapping_add(1));
    }
    let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
    record[46..78].copy_from_slice(&digest);
    write(image, device, offset, &record);
}

/// 攻方 C2-2 的手造镜像（`research/prompts/m3-batch4-code-r1-opus-output.md` C2-2）：「发布 B 的记录两盘都落了、根槽没落」那份
/// 崩溃镜像上先取号到实例 2（两盘系统配置跟着轮换，I-7.7 不先红），再把盘 0 计数器 4 那一格换成实例 2、txg 5 的一条记录
/// （照 B 那条改实例、txg、反向链 0、事务号 1、带提交标记与末条标志、点名项清空），盘 1 那一格仍是 B。
/// 第一版两盘池上推为不可达（同一计数器要被两个实例各写一次），判决第二节第 8 行要 checker 与实现同样按 (实例, 计数器) 读环。
fn image_with_another_instance_on_device_zero_at_the_counter_of_the_overwrite_record(
    tag: &str,
) -> MemoryPool {
    let crash = crash_image_with_the_overwrite_record_but_not_its_root(tag);
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&crash, &[], &[], &stream);
    let parameters_of_the_pool = parameters();
    {
        let mut writer = PoolWriter::new(&parameters_of_the_pool, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        assert_eq!(instance, InstanceGeneration(2), "崩溃镜像上下一个实例是 2");
    }
    let mut forged = memory_pool_of_sparse_devices(&devices);
    let offset = record_offset(COUNTER_OF_THE_OVERWRITE_RECORD, JOURNAL_RING_DEFAULT_BYTES).0;
    let overwrite_bytes = read(&forged, 1, offset, 4096);
    let mut another_instance = JournalRecord::parse(
        &overwrite_bytes,
        get_u64(
            &overwrite_bytes,
            JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET,
        ),
    )
    .expect("盘 1 计数器 4 上是 B");
    another_instance.instance = InstanceGeneration(2);
    another_instance.checkpoint_txg = CheckpointTxg(5);
    another_instance.back_chain = 0;
    another_instance.transaction = 1;
    another_instance.is_commit = true;
    another_instance.place_in_publish = JournalRecordPlaceInPublish::LastRecordOfThePublish;
    another_instance.named = Vec::new();
    write(&mut forged, 0, offset, &another_instance.to_bytes());
    forged
}

/// 两块盘同一计数器坐着不同实例的记录：实现扫环按 (实例, 计数器) 两条都收、从实例 1 的 A 那条接到 B、施加 B（落在 (1, 4)），
/// checker 的重放同样施加 B（I-4.1 真被评估过且成立），不拿盘 0 那条实例 2 的记录挤掉 B（判决第二节第 8 行）。
#[test]
fn a_record_of_another_instance_at_the_same_counter_on_the_first_device_does_not_hide_the_overwrite_record_from_the_replay(
) {
    let forged = image_with_another_instance_on_device_zero_at_the_counter_of_the_overwrite_record(
        "replay-reads-the-ring-by-instance-and-counter",
    );
    assert_eq!(
        recover(&forged, JournalPolicy::Consult).effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(4))),
        "实现的恢复施加 B、落在 (1, 4)"
    );
    assert_eq!(
        verdict(&forged, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE),
        InvariantVerdict::Holds,
        "checker 的重放同样施加 B、施加出来那一版走得完"
    );
}

/// 同一张镜像再把盘 1 那份 B 的新根段里树表指针改坏：实现施加 B 之后读不出树表、恢复失败；checker 的重放同样施加 B（末条计数器 4），
/// 施加出来那一版走不完 ⇒ 红 I-4.1。
#[test]
fn a_record_of_another_instance_at_the_same_counter_does_not_hide_a_broken_overwrite_record_from_the_replayed_version_invariant(
) {
    let mut forged =
        image_with_another_instance_on_device_zero_at_the_counter_of_the_overwrite_record(
            "replay-by-instance-and-counter-reaches-a-broken-record",
        );
    break_the_new_tree_table_pointer_of_the_overwrite_record_on(&mut forged, 1);
    assert!(
        matches!(
            recover(&forged, JournalPolicy::Consult).outcome,
            RecoveryOutcome::Failed { .. }
        ),
        "实现施加 B 之后读不出它的树表、恢复失败"
    );
    match verdict(&forged, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains("（末条计数器 4）"),
            "checker 施加的是 B 那条（计数器 4）：{detail}"
        ),
        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
            panic!("checker 施加 B 之后那一版走不完，I-4.1 该判违例，实际 {other:?}")
        }
    }
}

/// 1 MiB 的 journal 环：64 槽，环末尾的下一个槽 1088 = 17 × 64 落在聚簇段边界上，mkfs 收它；256 条记录，在飞上限 ⌊256 ÷ 3⌋ = 85 条
/// （`core_review_unit_area_start_and_publish_limits.rs` 用的同一个环）。
const JOURNAL_RING_OF_ONE_MEBIBYTE: u64 = 1 << 20;
const RECORD_LIMIT_ON_THE_ONE_MEBIBYTE_RING: u64 =
    JOURNAL_RING_OF_ONE_MEBIBYTE / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR;

/// 两块内存盘、1 MiB 环的池：mkfs → 取号 → 暖机 → 新池新建文件（txg 3）→ 从文件偏移 0 顺序写 85 个数据单元（txg 4）：
/// 那次发布切成恰好在飞上限那么多条记录，末条带末条标志。交回镜像与那次发布末条的计数器。
fn image_with_a_publish_of_as_many_records_as_the_in_flight_limit_on_a_one_mebibyte_ring(
) -> (MemoryPool, u64) {
    let default_parameters = parameters();
    let parameters_on_the_one_mebibyte_ring = MakeFilesystemParameters {
        filesystem_identifier: default_parameters.filesystem_identifier,
        region_devices: default_parameters.region_devices,
        geometry: SystemImmutableSizes {
            physical_block_size: default_parameters.geometry.physical_block_size,
            minimum_input_output_bytes: default_parameters.geometry.minimum_input_output_bytes,
            fixed_structure_slot_spacing: default_parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: JOURNAL_RING_OF_ONE_MEBIBYTE,
            root_ring_slots_per_region: default_parameters.geometry.root_ring_slots_per_region,
        },
    };
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                )
            })
            .collect();
    let genesis = make_filesystem(&parameters_on_the_one_mebibyte_ring, &mut devices)
        .expect("1 MiB 的环上 mkfs");
    let mut allocator =
        allocator_after_make_filesystem(&parameters_on_the_one_mebibyte_ring, &devices, &genesis);
    let data_unit_payload_bytes = u64::try_from(singlefs_core::unit::data_unit_payload_capacity())
        .expect("一个数据单元的净荷装得进 u64");
    let sequential_content: Vec<u8> =
        (0..(RECORD_LIMIT_ON_THE_ONE_MEBIBYTE_RING - 1) * data_unit_payload_bytes + 1)
            .map(|index| u8::try_from(index % 251).expect("小于 256"))
            .collect();
    let sequential_write = {
        let mut writer =
            PoolWriter::new(&parameters_on_the_one_mebibyte_ring, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let content = common::file_content();
        let first_file = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
        publish_sequential_write(
            &mut writer,
            &mut allocator,
            &first_file,
            FirstFile {
                content: &sequential_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            instance,
        )
        .expect("85 条记录不多于在飞上限 85 条")
    };
    assert_eq!(
        u64::try_from(sequential_write.earlier_records_of_this_publish.len()).expect("条数") + 1,
        RECORD_LIMIT_ON_THE_ONE_MEBIBYTE_RING,
        "顺序写那次发布切成恰好 85 条记录"
    );
    let image = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: IMAGE_BYTES,
    };
    let last_counter = (1..=JOURNAL_RING_OF_ONE_MEBIBYTE / JOURNAL_RECORD_BYTES)
        .take_while(|counter| {
            let bytes = read(
                &image,
                0,
                record_offset(*counter, JOURNAL_RING_OF_ONE_MEBIBYTE).0,
                4096,
            );
            JournalRecord::parse(
                &bytes,
                get_u64(&bytes, JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET),
            )
            .is_some()
        })
        .last()
        .expect("环里有记录");
    (image, last_counter)
}

/// 上面那份镜像上把顺序写那次发布接长一条：原先的末条（计数器 `last_counter`）摘掉末条标志，后面接一条同实例、同 txg 的记录
/// （计数器加 1、序号 86、带末条标志与提交标记、事务号接着往上、点名项清空、反向链接在改过的原末条上），两盘各一份：
/// 那次发布 86 条记录，多于在飞上限 85 条，别的读者规则都还成立。
fn lengthen_the_publish_on_the_one_mebibyte_ring_past_the_in_flight_limit(
    image: &mut MemoryPool,
    last_counter: u64,
) {
    for device in DEVICES {
        let last_offset = record_offset(last_counter, JOURNAL_RING_OF_ONE_MEBIBYTE).0;
        let last_bytes = read(image, device, last_offset, 4096);
        let mut last = JournalRecord::parse(
            &last_bytes,
            get_u64(&last_bytes, JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET),
        )
        .expect("顺序写那次发布的末条自证过");
        assert_eq!(
            (
                last.ordinal_within_publish,
                last.place_in_publish,
                last.is_commit
            ),
            (
                JournalRecordOrdinalWithinPublish(85),
                JournalRecordPlaceInPublish::LastRecordOfThePublish,
                true
            ),
            "盘 {device} 计数器 {last_counter} 是那次发布的第 85 条、末条"
        );
        last.place_in_publish = JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow;
        let rewritten_last_bytes = last.to_bytes();
        write(image, device, last_offset, &rewritten_last_bytes);
        let mut appended = last.clone();
        appended.counter = last_counter + 1;
        appended.ordinal_within_publish = JournalRecordOrdinalWithinPublish(86);
        appended.place_in_publish = JournalRecordPlaceInPublish::LastRecordOfThePublish;
        appended.transaction = last.transaction + 1;
        appended.is_commit = true;
        appended.named = Vec::new();
        appended.back_chain = back_chain_of(&rewritten_last_bytes);
        write(
            image,
            device,
            record_offset(appended.counter, JOURNAL_RING_OF_ONE_MEBIBYTE).0,
            &appended.to_bytes(),
        );
    }
}

/// I-8.1「同一次发布的记录数不多于在飞上限」那一格的边界（`research/prompts/m3-batch4-code-r1-sonnet-output.md` 第五节盲区第 7 条）：
/// 一次发布恰好 85 条记录、在飞上限 85 条 ⇒ I-8.1 真被评估过且成立，一条都不红。
#[test]
fn a_publish_of_exactly_the_in_flight_limit_of_records_holds_the_ring_geometry_invariant() {
    let (image, _) =
        image_with_a_publish_of_as_many_records_as_the_in_flight_limit_on_a_one_mebibyte_ring();
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "一次发布 85 条记录、在飞上限 85 条：一条都不红：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, JOURNAL_RING_GEOMETRY_FIELDS_AGREE),
        InvariantVerdict::Holds,
        "I-8.1 真被评估过且成立"
    );
}

/// 同一次发布接长到 86 条（多于在飞上限 85 条），系统配置三个字段照样成立 ⇒ **只**红 I-8.1 记录条数那一格。
#[test]
fn a_publish_of_one_record_more_than_the_in_flight_limit_reddens_only_the_ring_geometry_invariant()
{
    let (mut image, last_counter) =
        image_with_a_publish_of_as_many_records_as_the_in_flight_limit_on_a_one_mebibyte_ring();
    lengthen_the_publish_on_the_one_mebibyte_ring_past_the_in_flight_limit(
        &mut image,
        last_counter,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [JOURNAL_RING_GEOMETRY_FIELDS_AGREE],
        "判红的该只有 I-8.1：{verdicts:?}"
    );
}

/// 86 条那一份再把那次发布的根（txg 4）改坏：所选根退到 txg 3，它之上那次发布 86 条，重放数到在飞上限 85 条就停（已定项 18），
/// 那次发布的末条没到、整体不施加 ⇒ I-4.1 报不适用（`research/prompts/m3-batch4-code-r1-sonnet-output.md` 第五节盲区第 10 条 L1）。
#[test]
fn a_replay_stops_at_the_in_flight_limit_and_a_publish_longer_than_it_is_not_applied() {
    let (mut image, last_counter) =
        image_with_a_publish_of_as_many_records_as_the_in_flight_limit_on_a_one_mebibyte_ring();
    lengthen_the_publish_on_the_one_mebibyte_ring_past_the_in_flight_limit(
        &mut image,
        last_counter,
    );
    make_the_root_unreadable(&mut image, CheckpointTxg(4));
    assert_eq!(
        verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE),
        InvariantVerdict::NotApplicable(NOTHING_APPLIED_ABOVE_THE_CHOSEN_ROOT),
        "86 条的那次发布超过在飞上限，重放一次发布都不施加"
    );
}

/// 系统配置槽里区域数 R 那 1 字节的偏移（`.claude/kb/layout/01-first-txn.md` 一：F 357 + 4 → R 361）。
const SYSTEM_CONFIGURATION_REGION_COUNT_OFFSET: usize = 361;
/// 比系统配置字段表留的三个逐区域设备字段多一个区域：这一槽自证过，而这一版读者不可择（`RegionCountPastTheRegionDeviceFields`）。
const REGION_COUNT_PAST_THE_REGION_DEVICE_FIELDS: u8 = 4;

/// I-8.5（tail 槽有效集非空）数的是自证过的槽，「自证过而不可择」的也算（`research/prompts/m3-batch4-code-r1-sonnet-output.md`
/// 第五节盲区第 9 条）：全池四个槽的 R 都写 4、整槽校验和重封 ⇒ 一槽都择不出、每一槽都自证过 ⇒ I-8.5 真被评估过且成立，
/// 别的每一条报不适用，一条都不红。
#[test]
fn self_verified_system_configuration_slots_no_reader_can_choose_still_witness_the_tail() {
    let clean = build_pool("tail-slot-witnesses-self-verified-but-not-choosable").memory_pool();
    let mut image = clean.clone();
    for (device, offset) in every_system_configuration_slot(&clean) {
        mutate_system_configuration_slot(&mut image, device, offset, |bytes| {
            assert_eq!(
                bytes[SYSTEM_CONFIGURATION_REGION_COUNT_OFFSET], 3,
                "三个区域"
            );
            bytes[SYSTEM_CONFIGURATION_REGION_COUNT_OFFSET] =
                REGION_COUNT_PAST_THE_REGION_DEVICE_FIELDS;
        });
    }
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "每一槽都自证过：一条都不红：{verdicts:?}"
    );
    assert_eq!(
        verdict(
            &image,
            SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
        ),
        InvariantVerdict::Holds,
        "自证过而不可择的槽也是 tail 的见证者：I-8.5 真被评估过且成立"
    );
}

/// 根记录里实例表指针（170 起）头部出生 txg 的偏移（指针偏移 42）。
const ROOT_RECORD_INSTANCE_TABLE_POINTER_BIRTH_TXG_OFFSET: usize =
    ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + 42;

/// I-1.2（块头写序已发布） 状态收窄（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 3 行）：实例表单元按例外只免谓词那一半，
/// 指着它的指针与它头里的出生身份仍要相等。新池新建文件那条根（txg 3）的实例表指针出生 txg 加 1、重封根槽：实例表本身一个字节不动 ⇒
/// **只**红 I-1.2。
#[test]
fn an_instance_table_pointer_whose_birth_txg_differs_from_the_instance_table_header_reddens_only_the_birth_identity_invariant(
) {
    let clean = build_pool("known-bad-instance-table-pointer-birth").memory_pool();
    assert_eq!(
        verdict(&clean, "I-1.2"),
        InvariantVerdict::Holds,
        "干净镜像上 I-1.2 真被评估过且成立"
    );
    let mut image = clean.clone();
    let (device, offset) = NEW_POOL_FILE_CREATION_ROOT_SLOT;
    let mut root = read(&image, device, offset, 512);
    assert_eq!(get_u64(&root, 28), 3, "这一槽是新池新建文件那条根（txg 3）");
    let birth_txg = get_u64(&root, ROOT_RECORD_INSTANCE_TABLE_POINTER_BIRTH_TXG_OFFSET);
    set_u64(
        &mut root,
        ROOT_RECORD_INSTANCE_TABLE_POINTER_BIRTH_TXG_OFFSET,
        birth_txg + 1,
    );
    let digest = wide_checksum_with_field_zeroed(&root, 512, 138);
    root[138..170].copy_from_slice(&digest);
    write(&mut image, device, offset, &root);
    // 记录新根段里那份实例表指针跟着改（I-8.10 七段里有它）：盘上留下的是「写者从一开始就写出了这样的根」的样子。
    copy_each_root_into_the_new_root_segment_of_its_journal_records(&mut image);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-1.2"],
        "判红的该只有 I-1.2：{verdicts:?}"
    );
}

/// 翻一个单元两份各一个载荷字节，不重封、不补引用链：位置条目校验和对不上，只读不判那一遍读不出它。
fn break_both_copies_of_the_unit(image: &mut MemoryPool, slot: u64, length: usize) {
    for device in DEVICES {
        let mut bytes = read(image, device, slot * SLOT, length);
        bytes[length - 100] ^= 0xff;
        write(image, device, slot * SLOT, &bytes);
    }
}

/// I-3.9（释放代落在停止引用它的那一格区间里） 守卫收窄的正例（攻方 `research/prompts/m3-inv-b-r1-opus-model/proto.diff` 的组合坏镜像）：
/// 发布 B 之后那份镜像上 B 的释放代 4 改成 3，再把 A 那棵账的一片叶两份都翻坏——走不完的是 A（txg 3），它不比还引用那几个落点的最后一条根
/// （L = A 自己）晚，那几条记录照判 ⇒ I-3.9 判红。整条按「任一候选根走不完就不适用」的守卫在这里放过释放代。
#[test]
fn a_wrong_release_generation_is_still_reddened_when_only_a_root_not_later_than_the_last_one_referencing_it_is_incomplete(
) {
    let mut image =
        image_after_the_overwrite("release-generation-behind-an-earlier-incomplete-root");
    assert_eq!(
        mutate_allocation_leaves(
            &mut image,
            &UNITS_AFTER_OVERWRITE,
            ALLOCATION_LEAVES_AFTER_OVERWRITE,
            |bytes| rewrite_released_generations(bytes, 4, 3),
        ),
        24,
        "B 的账里释放代 4 的记录两片叶合起来 24 条"
    );
    break_both_copies_of_the_unit(&mut image, ALLOCATION_LEAVES[0], 16384);
    match verdict(&image, "I-3.9") {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains("释放代 3"),
            "红在释放代 3 那几条上：{detail}"
        ),
        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
            panic!("A 走不完而它不比 L 晚：I-3.9 照判、该红，实际 {other:?}")
        }
    }
}

/// I-3.9 守卫收窄的对照格（攻方造出来的那一份）：步 5 那段历史抬 F 到 11（空发布 txg 15、16 不换用户可见树），再覆盖写一次（txg 17）
/// 释放 txg 14 那一版的用户可见单元（释放代 17，真 L = 16）；把 txg 16 那条根自己的树表两份都翻坏，那几个落点藏在读不出的树表下面，
/// 算出来的 L = 15、T = 16。比 L 晚的 txg 16 走不完 ⇒ 那几条跳过，I-3.9 报不适用、不红；不设防的判法在这里误红。
#[test]
fn release_generations_hidden_behind_a_later_incomplete_root_are_skipped_and_the_release_generation_invariant_is_not_applicable(
) {
    let mut pool =
        pool_before_raising_the_floor("release-generation-behind-a-later-incomplete-root");
    raise_the_floor(&mut pool, CheckpointTxg(11), CheckpointTxg(0));
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(16),
        "抬 F 那串空发布最后一次是 txg 16"
    );
    let tree_table_of_the_last_raise = pool.output.unit(TransactionUnit::TreeTable).slot.0;
    overwrite_and_advance(&mut pool, &content_seeded(1200, 3), InstanceGeneration(2));
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(17),
        "抬 F 之后那次覆盖写是 txg 17"
    );
    let mut image = pool.memory_pool();
    assert!(
        !matches!(verdict(&image, "I-3.9"), InvariantVerdict::Violated(_)),
        "翻坏之前 I-3.9 不红"
    );
    break_both_copies_of_the_unit(&mut image, tree_table_of_the_last_raise, 16384);
    match verdict(&image, "I-3.9") {
        InvariantVerdict::NotApplicable(reason) => assert!(
            reason.contains("走不完"),
            "报不适用的理由要说比 L 晚的候选根走不完：{reason}"
        ),
        other @ (InvariantVerdict::Holds | InvariantVerdict::Violated(_)) => {
            panic!("落点藏在比 L 晚、走不完的根后面：I-3.9 该报不适用，实际 {other:?}")
        }
    }
}

/// 同步助手 `common::copy_each_root_into_the_new_root_segment_of_its_journal_records` 里「前一条被改过、又与这一条同实例，
/// 这一条的反向链按前一条改后的记录头重算」那一支（I-8.6（反向链算法）只链本实例内逻辑前一条）。发布 B 之后的干净镜像上实例 1 有
/// A 的记录（计数器 3，txg 3）与 B 的记录（计数器 4，txg 4）；改的是较旧那条根——A 的根（txg 3）树 ID 水位压到 mkfs 时的水位、重封自证校验和。
/// 同步之后计数器 3 两盘各一份的新根段跟着压（I-8.10（记录新根段与同代根相同）），计数器 4 两盘各一份的新根段不动、反向链按改后的计数器 3 重算。
/// B 的根是最新那条、水位照旧盖得住全池出现过的树 ID（I-7.8 取根环全部根的 max，另判最新那条根），所以判决与干净镜像逐项相同；
/// 那一支不重算，计数器 4 的反向链接不上改后的计数器 3，红 I-8.6。
#[test]
fn rewriting_an_older_root_rechains_the_later_record_of_its_instance_and_every_verdict_stays_as_on_the_clean_image(
) {
    let clean = image_after_the_overwrite("older-root-rewritten-later-record-rechained");
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    let mut image = clean.clone();
    let (device, offset) = root_slot_of(CheckpointTxg(3));
    let mut bytes = read(&image, device, offset, 512);
    assert_eq!(get_u64(&bytes, 28), 3, "那一槽是 A 的根（txg 3）");
    assert!(
        get_u64(&bytes, ROOT_RECORD_TREE_IDENTIFIER_WATERMARK_OFFSET)
            > singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS,
        "A 发过树，它的水位高于 mkfs 时的水位：压下去才是真改了"
    );
    set_u64(
        &mut bytes,
        ROOT_RECORD_TREE_IDENTIFIER_WATERMARK_OFFSET,
        singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS,
    );
    let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
    bytes[138..170].copy_from_slice(&digest);
    write(&mut image, device, offset, &bytes);
    let records_rewritten =
        copy_each_root_into_the_new_root_segment_of_its_journal_records(&mut image);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "改了较旧那条根、同步之后一条都不红（不重算后一条的反向链就红 I-8.6）：{verdicts:?}"
    );
    assert_eq!(
        verdicts_that_differ_outside(&verdicts, &verdicts_on_the_clean_image, &[]),
        [],
        "每一条的判定与干净镜像逐项相同"
    );
    assert_eq!(
        records_rewritten, 4,
        "计数器 3 两盘各一份改新根段，计数器 4 两盘各一份重算反向链"
    );
}
