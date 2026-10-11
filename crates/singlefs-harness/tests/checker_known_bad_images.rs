//! checker 的已知坏镜像语料（C13（checker 判定失效））：第一版判的 36 条不变量，每条配一份「改坏了正好触发它」的镜像。
//! 每份都从一份干净镜像出发（写完新池新建文件的，或再覆盖写一次的），改一处；要让被改的那一条之前的判定不先挡住，
//! 被改的单元重封校验和，再把新的整单元校验和沿引用链一路补到根记录（父节点里的位置条目、中央映射条目、根槽的自证校验和）。

mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
    copy_each_root_into_the_new_root_segment_of_its_journal_records, crash_state_devices,
    format_pool, geometry, memory_pool_of_sparse_devices,
    overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
    publish_overwrite_in_process, rewrite_journal_record_and_rechain_the_records_after_it,
    tear_the_root_slot_written_for_txg, BuiltPool, CONTENT_BYTES_OF_PUBLISH_B,
    CONTENT_BYTES_OF_PUBLISH_C, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_checker::image::{
    InvariantVerdict, ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY, DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
    EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
    FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
    HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
    INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
    INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
    INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
    INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
    INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,
    JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
    LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
    LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
    LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER, NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
    NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
    NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
    ONE_OBJECT_BIRTH_PER_INODE_NUMBER, POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
    PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
    REFERENCE_INTERVALS_ARE_EXACT,
    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
    RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER, ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES,
    ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
    SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
    SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
    SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
    SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
    TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
    TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
    TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
    UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
};
use singlefs_checker::walk::{check_pool_image, InstanceTableOfRootRecord};
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
use singlefs_core::records::{
    RetirementRecord, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST,
    TREE_KIND_DELETED_INODES, TREE_KIND_LINEAGE, TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_RETIREMENT,
    TREE_KIND_TOMBSTONE,
};
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
        // 用它的镜像上都没有删除：没有处于删除中的号、没有 nlink = 0 的记录；I-4.4 第一版报不适用。
        "I-4.7" => InvariantVerdict::NotApplicable(NO_INODE_BEING_DELETED_IN_ANY_WALKED_VERSION),
        "I-4.6" => InvariantVerdict::NotApplicable(NO_RECORD_OF_NLINK_ZERO_IN_ANY_WALKED_VERSION),
        "I-4.4" => InvariantVerdict::NotApplicable(INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION),
        other_invariant => {
            expected_verdict_of_the_tombstone_and_lineage_invariants_without_deletion(
                other_invariant,
            )
        }
    }
}

/// 没有删除、没有回退的镜像上墓碑与谱系那九条（里程碑三·3，判决 `research/prompts/m3-delete-checker-r2-main-verification.md`）的判定：
/// 墓碑树、谱系树都是零根，I-9.17–I-9.23 没有对象、报不适用带理由（I-9.22 是旧入口当没装旋钮）；I-9.24（队列空的一版报成立）与
/// I-9.29（inode 记录各号只一个出生代）真被评估过且成立。别的不变量交回成立。
fn expected_verdict_of_the_tombstone_and_lineage_invariants_without_deletion(
    invariant: &str,
) -> InvariantVerdict {
    match invariant {
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER => {
            InvariantVerdict::NotApplicable(NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION)
        }
        QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE => {
            InvariantVerdict::NotApplicable(NO_QUEUED_INODE_IN_ANY_WALKED_VERSION)
        }
        TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT => {
            InvariantVerdict::NotApplicable(NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD)
        }
        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED => {
            InvariantVerdict::NotApplicable(NO_TOMBSTONE_IN_ANY_WALKED_VERSION)
        }
        NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE => {
            InvariantVerdict::NotApplicable(NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER)
        }
        TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE => {
            InvariantVerdict::NotApplicable(NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY)
        }
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER => {
            InvariantVerdict::NotApplicable(NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION)
        }
        // 退役树那几条（第 50 批）：没有叶容器身份消失，退役树零根；I-9.35 在相邻两版上真被评估过且成立（叶身份集没变、没有退役记录）。
        RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER => {
            InvariantVerdict::NotApplicable(NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION)
        }
        EVERY_RETIREMENT_RECORD_IS_WELL_FORMED => {
            InvariantVerdict::NotApplicable(NO_RETIREMENT_RECORD_IN_ANY_WALKED_VERSION)
        }
        RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT
        | RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF => InvariantVerdict::NotApplicable(
            NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,
        ),
        // 谱系树与退役树的容器容量那两条（第 53 批）：两棵都是零根，没有非最右的容器；旧入口没有谱系与退役的旋钮值。
        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE => {
            InvariantVerdict::NotApplicable(NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER)
        }
        LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE => {
            InvariantVerdict::NotApplicable(NO_LINEAGE_OR_RETIREMENT_CONTAINER_CAPACITY_KNOB)
        }
        _ => InvariantVerdict::Holds,
    }
}

/// 谱系树与退役树的容器容量那两条没有对象时报的理由（`walk` 的 `judge_tombstones_and_the_lineage_of_the_versions` 末尾）。
const NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER: &str =
    "走读到的每一版里没装旋钮的谱系树与退役树都是零根或只一片容器：没有非最右的容器";
const NO_LINEAGE_OR_RETIREMENT_CONTAINER_CAPACITY_KNOB: &str =
    "池级 checker 没拿到只供测试的谱系与退役容量旋钮（旧入口，或新入口没给这两个旋钮值）：旋钮那一条不判";

/// 退役树那几条没有对象时报的理由（`walk` 的 `judge_tombstones_and_the_lineage_of_the_versions` 与 `check_pool_image` 末尾）。
const NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版退役树都是零根：没有退役树内部条目可判";
const NO_RETIREMENT_RECORD_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版退役树都是零根：没有退役记录可判";
const NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版里没有一条退役记录在谱系上：没有对象可判";

/// 墓碑与谱系那几条没有对象时报的理由（`walk` 的 `judge_tombstones_and_the_lineage_of_the_versions` 与 `check_pool_image` 末尾）。
const NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版谱系树都是零根：没有谱系树内部条目可判";
const NO_QUEUED_INODE_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版 deleted_inodes 里都没有号：没有队列里的号可判";
const NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD: &str =
    "走读到的每一版里都没有在谱系上、而 inode 树里还有同号同对象出生代记录的墓碑";
const NO_TOMBSTONE_IN_ANY_WALKED_VERSION: &str = "走读到的每一版墓碑树都是零根：没有墓碑记录可判";
const NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER: &str =
    "走读到的每一版墓碑树都是零根或只一片容器：没有非最右的容器";
const NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY: &str =
    "池级 checker 的旧入口，当没装只供测试的墓碑容量旋钮：旋钮那一条不判";
const NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版墓碑树都是零根：没有墓碑树内部条目可判";

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
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
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
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
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
        // 同一条线上两条根记得不同，必然也是全部可读根里记得不同：I-9.14 的坏镜像连带 I-9.28 红，别的一条不红。
        let expected: Vec<&str> = if invariant == "I-9.14" {
            vec![
                "I-9.14",
                TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
            ]
        } else {
            vec![invariant]
        };
        assert_eq!(
            violated, expected,
            "判红的该只有 {expected:?}：{verdicts:?}"
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
                    other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_)) => {
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
                    other @ (InvariantVerdict::Holds
                    | InvariantVerdict::NotApplicable(_)
                    | InvariantVerdict::PartiallyJudged(_)) => {
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
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
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

/// 墓碑与谱系那九条（第 22 批）的坏镜像各要一份自己的基底（意图之后、根环转过一圈之后、回退之后、旋钮压小写出的），不进上面那张
/// 只改干净镜像一处的表；这里登记每一条的坏镜像在哪个用例里（那几个用例各自断言只红它自己）。
const TOMBSTONE_AND_LINEAGE_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE: [(&str, &str); 9] = [
    (
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "checker_known_bad_images.rs: a_lineage_tree_entry_whose_identity_names_another_container_reddens_only_the_lineage_entry_invariant",
    ),
    (
        QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
        "pool_checker_judges_the_deletion_state_of_every_walked_version.rs: a_queued_inode_whose_tombstone_died_on_a_branch_a_rollback_left_reddens_only_the_queued_tombstone_invariant",
    ),
    (
        TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
        "pool_checker_judges_the_deletion_state_of_every_walked_version.rs: a_tombstone_on_the_lineage_naming_a_live_inode_reddens_only_the_live_object_invariant",
    ),
    (
        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
        "pool_checker_judges_the_deletion_state_of_every_walked_version.rs: a_tombstone_that_does_not_cover_the_whole_object_reddens_only_the_tombstone_record_invariant",
    ),
    (
        NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        "container_trees_grow_a_level_when_the_root_is_full.rs: a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant",
    ),
    (
        TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
        "container_trees_grow_a_level_when_the_root_is_full.rs: a_tombstone_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant",
    ),
    (
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "checker_known_bad_images.rs: a_tombstone_tree_entry_whose_identity_names_another_container_reddens_only_the_tombstone_entry_invariant",
    ),
    (
        DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
        "pool_checker_judges_the_deletion_state_of_every_walked_version.rs: a_deletion_queue_of_two_inodes_being_deleted_with_their_tombstones_reddens_only_the_at_most_one_key_invariant",
    ),
    (
        ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
        "inode_number_watermark_pressed_back_reissues_a_number.rs: pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants",
    ),
];

/// 退役树那五条（I-9.31–I-9.35，第 50 批，判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条）的坏镜像各建在
/// 「三个单元的文件删到底」那一份上（唯一那片叶容器身份消失、退役树一条记录），或它再建 31 次 inode、删的那几版转出回退候选集的那一份上，
/// 不进只改干净镜像一处的表。
const RETIREMENT_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE: [(&str, &str); 5] = [
    (
        RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "checker_known_bad_images.rs: a_retirement_tree_entry_whose_identity_names_another_container_reddens_only_the_retirement_entry_invariant",
    ),
    (
        EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
        "checker_known_bad_images.rs: a_retirement_record_naming_a_container_of_record_type_one_reddens_only_the_well_formed_retirement_record_invariant",
    ),
    (
        RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
        "checker_known_bad_images.rs: a_second_copy_of_the_retirement_record_reddens_only_the_distinct_retirement_records_invariant",
    ),
    (
        RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
        "checker_known_bad_images.rs: a_retirement_record_naming_the_live_leaf_reddens_only_the_no_live_leaf_invariant",
    ),
    (
        RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
        "checker_known_bad_images.rs: a_retirement_record_moved_off_the_txg_the_leaf_died_at_reddens_only_the_retirement_completeness_invariant",
    ),
];

/// 谱系树与退役树的容器容量那两条（I-9.36、I-9.37，第 53 批，判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2）的坏镜像
/// 各要一份旋钮压小写出的基底（攻方 `opus_probe_container_closed_early` 那一形：两次叶身份消失、两次回退），不进只改干净镜像一处的表。
const LINEAGE_AND_RETIREMENT_CONTAINER_CAPACITY_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE: [(
    &str,
    &str,
); 3] = [
    (
        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        "checker_known_bad_images.rs: two_leaves_retired_one_record_per_retirement_container_judged_as_a_product_image_reddens_only_the_full_container_invariant",
    ),
    (
        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        "checker_known_bad_images.rs: two_rollbacks_one_record_per_lineage_container_judged_as_a_product_image_reddens_only_the_full_container_invariant",
    ),
    (
        LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
        "checker_known_bad_images.rs: a_lineage_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant",
    ),
];

/// I-3.14（实例表每片记录形态合法） 的三张坏镜像各建在攻方 N5 那一份基底上（崩溃恢复抛弃 C、C 与现行线同号不同出生代，
/// [`image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed`]），不进只改干净镜像一处的表；
/// 这里登记它们在哪几个用例里（各自断言判红的是 I-3.14 与这一片解不开带出来的那几条）。最后一条不是合法的链指针记录那一格的坏镜像在
/// `instance_table_chain_read.rs` 的 `contradictory_chain_record_reddens_only_the_instance_table_invariant`。
const INSTANCE_TABLE_RECORD_FORM_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE: [(&str, &str); 3] = [
    (
        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
        "checker_known_bad_images.rs: a_record_of_an_unregistered_kind_before_the_chain_record_of_the_newest_instance_table_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set",
    ),
    (
        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
        "checker_known_bad_images.rs: a_second_chain_record_in_the_middle_of_the_newest_instance_table_page_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set",
    ),
    (
        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
        "checker_known_bad_images.rs: a_row_whose_flags_are_not_zero_in_the_newest_instance_table_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set",
    ),
];

/// I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上，第 49 批） 的坏镜像各要一份自己的基底（崩溃恢复抛弃建过 inode 的根之后再建、再回退；
/// 建过 inode 之后覆盖写），不进只改干净镜像一处的表；这里登记它们在哪几个用例里（各自断言只红它）。② 那一判（记录晚于台阶出生）的坏镜像是
/// 压回水位再建的那几份（`inode_number_watermark_pressed_back_reissues_a_number.rs`）。
const INODE_NUMBER_WATERMARK_STEP_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE: [(&str, &str); 4] = [
    (
        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
        "checker_known_bad_images.rs: a_lineage_record_whose_watermark_rolled_back_from_is_below_the_last_step_reddens_only_the_watermark_step_invariant",
    ),
    (
        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
        "checker_known_bad_images.rs: a_lineage_record_whose_watermark_rolled_back_from_is_zero_reddens_only_the_watermark_step_invariant",
    ),
    (
        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
        "checker_known_bad_images.rs: an_inode_number_watermark_step_dropped_from_the_accounting_tree_reddens_only_the_watermark_step_invariant",
    ),
    (
        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
        "checker_known_bad_images.rs: an_inode_number_watermark_step_raised_above_the_watermark_reddens_only_the_watermark_step_invariant",
    ),
];

/// 清单里不要求坏镜像的几条：I-4.4（意图只描述目标态） 第一版报不适用（用户 2026-10-05 定，`.claude/kb/milestone/03-third-txn.md` 三·3 那一行
/// 「I-4.4 不要求坏镜像与变异行」）。
const INVARIANTS_WITHOUT_A_BAD_IMAGE: [&str; 1] = [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE];
/// 干净镜像上 I-4.7（删除中 inode 有账） 与 I-4.6（中间态其余不变量成立） 没有对象时报的理由（`walk` 的删除态扫描）。
const NO_INODE_BEING_DELETED_IN_ANY_WALKED_VERSION: &str = "走读到的每一版里都没有处于删除中的 inode（nlink = 0 的记录、有数据单元而没有 nlink ≥ 1 记录的号）";
const NO_RECORD_OF_NLINK_ZERO_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版 inode 树里都没有 nlink = 0 的记录";

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
            // 同一个理由，比的是根环里全部读得出的根（暖机那几代都指着 mkfs 种的第 0 版树表，空的）。这一条的坏镜像在「崩溃恢复抛弃了第三条根 C」
            // 那一份上（`birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_holds_the_same_line_invariant_and_reddens_only_the_every_readable_root_invariant`）。
            "I-9.28" => InvariantVerdict::NotApplicable(
                "根环里读得出的根中，没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来",
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
            // 这一份上没有删除：inode 1 的记录 nlink 1、队列空，没有处于删除中的号。阳性对照与坏镜像在删除中间那几版上
            // （`pool_checker_judges_the_deletion_state_of_every_walked_version.rs`，坏镜像登记在下面那一串 `.chain` 里）。
            "I-4.7" => {
                InvariantVerdict::NotApplicable(NO_INODE_BEING_DELETED_IN_ANY_WALKED_VERSION)
            }
            "I-4.6" => {
                InvariantVerdict::NotApplicable(NO_RECORD_OF_NLINK_ZERO_IN_ANY_WALKED_VERSION)
            }
            // 第一版报不适用并写理由（用户 2026-10-05 定），不要求坏镜像（`INVARIANTS_WITHOUT_A_BAD_IMAGE`）。
            "I-4.4" => {
                InvariantVerdict::NotApplicable(INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION)
            }
            // 墓碑与谱系那九条：这一份上没有删除、没有回退，墓碑树与谱系树都是零根。坏镜像见
            // `TOMBSTONE_AND_LINEAGE_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE`。
            other_invariant => {
                expected_verdict_of_the_tombstone_and_lineage_invariants_without_deletion(
                    other_invariant,
                )
            }
        };
        assert_eq!(found, expected, "干净镜像上 {invariant} 的判定");
    }
    let cases = known_bad_images(&clean);
    let (_, birth_txg, tree_table_slot_of_the_abandoned_root, abandoned_publish_txg) =
        image_after_a_recovery_abandoned_the_third_root(
            "known-bad-every-readable-root-birth-listed",
        );
    let targets: std::collections::BTreeSet<&str> = cases
        .iter()
        .chain(known_bad_images_after_the_overwrite().iter())
        .chain(
            known_bad_images_of_the_birth_txg_across_every_readable_root(
                tree_table_slot_of_the_abandoned_root,
                birth_txg,
                abandoned_publish_txg,
            )
            .iter(),
        )
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
        .chain(common_deletion_in_progress::known_bad_images_after_the_intent_of_a_three_unit_file().iter())
        .chain(common_deletion_in_progress::known_bad_images_after_the_intent_of_an_empty_inode().iter())
        .chain(
            common_deletion_in_progress::known_bad_images_after_the_first_batch_of_a_three_unit_file()
                .iter(),
        )
        .chain(common_deletion_in_progress::known_bad_images_after_one_new_inode().iter())
        .chain(common_deletion_in_progress::known_bad_images_of_the_pending_deletion_occupancy().iter())
        .map(|(invariant, _)| *invariant)
        .chain(
            TOMBSTONE_AND_LINEAGE_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .chain(
            INSTANCE_TABLE_RECORD_FORM_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .chain(
            INODE_NUMBER_WATERMARK_STEP_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .chain(
            RETIREMENT_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .chain(
            LINEAGE_AND_RETIREMENT_CONTAINER_CAPACITY_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE
                .iter()
                .map(|(invariant, _)| *invariant),
        )
        .collect();
    let listed: std::collections::BTreeSet<&str> = singlefs_checker::image::IMPLEMENTED_INVARIANTS
        .iter()
        .copied()
        .filter(|invariant| !INVARIANTS_WITHOUT_A_BAD_IMAGE.contains(invariant))
        .collect();
    assert_eq!(
        targets, listed,
        "第一版每条不变量（`INVARIANTS_WITHOUT_A_BAD_IMAGE` 那几条除外）都至少有一份坏镜像，也没有清单之外的"
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
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
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
/// I-3.1（已分配统计对得上）、I-3.11（已分配减 defer 等于最新根走读）、I-5.2（空闲统计对得上）与 I-9.6（水位大于两处最大号） 都没有了判的对象
/// （另有 I-3.13 与 I-3.4（可用空间扣待删占用） 两条读的也是记账行：删除态扫描里记账树条目宽不到 34 那一版报射程里判不了；
/// I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上） 的台阶行取自主走读走全了的记账树，走不全就没有行可判）。
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
        ["I-3.1", "I-3.11", "I-5.2", "I-9.6", "I-3.13", "I-3.4", "I-9.30"],
        "跟着转成不适用的该只有这七条（走读在 I-1.10 那一步停下，记账条目不再解；I-3.13 判的就是记账行的代段；\
         I-3.4 的删除态扫描读记账树也按条目宽 ≥ 34 才解，读不全那一版报射程里判不了；I-9.30 的台阶行取自走全了的记账树）：{verdicts:?}"
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
/// ⇒ I-9.14（树表条目的诞生 txg 跨根不变） 必须红；同一条线上记得不同必然也是全部可读根里记得不同，I-9.28（全部可读根里同一树 ID 的
/// 诞生 txg 相同） 跟着红，别的一条不红。把 I-9.14 的收窄做宽到把回退目标判出局的，这条红。
#[test]
fn birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_same_line_and_the_every_readable_root_birth_invariants(
) {
    let rolled_back =
        image_after_rolling_back_to_the_first_root("known-bad-after-rollback-same-line");
    for (invariant, found) in check_pool_image(&rolled_back.image) {
        assert_eq!(
            found,
            if invariant == LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER {
                // 回退那次发布写出谱系树（一片容器、一条谱系记录）：它的根条目真被评估过且成立。
                InvariantVerdict::Holds
            } else {
                expected_verdict_with_one_record_per_transaction(invariant)
            },
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
        [
            "I-9.14",
            TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT
        ],
        "回退目标 A 与回退那次发布 D 同在现行线上，诞生 txg 不一致只该红 I-9.14 与 I-9.28：{verdicts:?}"
    );
}

/// I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 的坏镜像，从「崩溃恢复抛弃了第三条根 C」那一份干净镜像出发：只改被抛弃的 C 那片树表里
/// extent 树（树 65536）的诞生 txg（`birth_txg` → C 那次发布的 txg `abandoned_publish_txg`），重封、沿引用链补到 C 的根槽。
/// C 不在回退候选集里（I-9.14 不拿它比）；I-9.28 不分时间线、比根环里全部读得出的根。
fn known_bad_images_of_the_birth_txg_across_every_readable_root(
    tree_table_slot_of_the_abandoned_root: u64,
    birth_txg: u64,
    abandoned_publish_txg: u64,
) -> Vec<(&'static str, Mutation)> {
    vec![(
        TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
        Box::new(move |image: &mut MemoryPool| {
            rewrite_the_extent_tree_birth_txg_in_tree_table(
                image,
                tree_table_slot_of_the_abandoned_root,
                birth_txg,
                abandoned_publish_txg,
            );
        }),
    )]
}

/// C511 的收窄那一头加 I-9.28 那一头：只有被崩溃恢复切掉的 C 把 extent 树的诞生 txg 记成它自己那次发布的 txg，
/// 现行线上 A、B、实例 2 与实例 3 的根仍一致记 3 ⇒ I-9.14（树表条目的诞生 txg 跨根不变） 不拿 C 比，判成立；
/// I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 比根环里全部读得出的根，C 也在里面，判红；别的每一条与干净镜像上相同。
/// 管理员回退改成向前发布之后，被切掉的线只由崩溃恢复造出（D23（journal 的角色与格式） 已定项 14）。
/// 这张镜像原先要求每一条都成立（I-9.14 判别力第 ③ 条的正向用例），用户 2026-10-04 定新立 I-9.28、删第 ③ 条之后改成这样。
/// 把收窄去掉（I-9.14 拿根环里每一条根比，不按实例表判出局）这条红在 I-9.14 上；把 I-9.28 收成只比回退候选集，这条红在 I-9.28 上。
#[test]
fn birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_holds_the_same_line_invariant_and_reddens_only_the_every_readable_root_invariant(
) {
    let (clean, birth_txg, tree_table_slot_of_the_abandoned_root, abandoned_publish_txg) =
        image_after_a_recovery_abandoned_the_third_root("known-good-after-recovery-abandoned-line");
    for (invariant, found) in check_pool_image(&clean) {
        assert_eq!(
            found,
            expected_verdict_with_one_record_per_transaction(invariant),
            "崩溃恢复抛弃了 C 之后的干净镜像上 {invariant} 的判定（I-9.28 在这里真被评估过且成立）"
        );
    }
    for (invariant, mutation) in known_bad_images_of_the_birth_txg_across_every_readable_root(
        tree_table_slot_of_the_abandoned_root,
        birth_txg,
        abandoned_publish_txg,
    ) {
        let mut image = clean.clone();
        mutation(&mut image);
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
        for (name, found) in &verdicts {
            if *name == invariant {
                assert!(
                    matches!(found, InvariantVerdict::Violated(detail)
                        if detail.contains("树 65536") && detail.contains(&format!("txg {abandoned_publish_txg}）记 {abandoned_publish_txg}"))),
                    "C 那片树表把树 65536 的诞生 txg 记成 {abandoned_publish_txg}，{invariant} 要红在它上面：{found:?}"
                );
            } else {
                assert_eq!(
                    *found,
                    expected_verdict_with_one_record_per_transaction(name),
                    "只改了被抛弃根 C 那片树表里的一个诞生 txg，{name} 的判定"
                );
            }
        }
    }
}

/// 一条位置条目（盘 4 + 槽 6 + 整单元校验和 4，D19（块指针的结构与宽度预算） 已定项 11）里槽号那 6 字节的偏移。
const LOCATION_ENTRY_SLOT_OFFSET: usize = 4;
/// 实例表单元（码 3）占两槽的字节数。
const INSTANCE_TABLE_UNIT_BYTES: usize = 32768;
/// 根记录里实例代号（4 字节）与 checkpoint_txg（8 字节）的偏移（D22（单元原子性怎么合成） 已定项 7）。
const ROOT_RECORD_INSTANCE_OFFSET: usize = 24;
const ROOT_RECORD_CHECKPOINT_TXG_OFFSET: usize = 28;

/// 根环里 (checkpoint_txg, 实例) 最大的那条根（择根序，D22（单元原子性怎么合成） 已定项 7）指着的实例表第 0 片的槽号（第一条位置条目，两盘同槽）。
fn instance_table_slot_of_the_newest_root(image: &MemoryPool) -> u64 {
    let newest_root = root_slots()
        .into_iter()
        .map(|(device, offset)| read(image, device, offset, 512))
        .filter(|root| &root[..4] == b"SFSR")
        .max_by_key(|root| {
            (
                get_u64(root, ROOT_RECORD_CHECKPOINT_TXG_OFFSET),
                get_u32(root, ROOT_RECORD_INSTANCE_OFFSET),
            )
        })
        .expect("根环里有根");
    let slot_start = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
        + POINTER_FIRST_LOCATION_OFFSET
        + LOCATION_ENTRY_SLOT_OFFSET;
    let mut slot_bytes = [0u8; 8];
    slot_bytes[..6].copy_from_slice(&newest_root[slot_start..slot_start + 6]);
    u64::from_le_bytes(slot_bytes)
}

/// 崩溃恢复抛弃 C 之后的干净镜像（[`image_after_a_recovery_abandoned_the_third_root`] 那一份）上，只改被抛弃的 C 那片 inode 叶容器（两盘同槽）里
/// 第一条记录的对象出生代（加一），重封、沿引用链补到 C 的根槽：C 那一版里这个号对着另一个对象出生代，现行线上 A、B 与实例 3 那几版仍是原来的。
fn image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth(
    tag: &str,
) -> MemoryPool {
    let (mut pool, _, third) = pool_through_the_third_version(tag);
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &third);
    let mut image = pool.memory_pool();
    let inode_leaf_of_the_abandoned_root = third
        .unit(TransactionUnit::InodeLeafContainer(
            singlefs_core::inode_tree::InodeLeafContainerIndexInTree::LEFTMOST,
        ))
        .slot
        .0;
    let units = units_found_on_device_zero(&image);
    mutate_unit_of(
        &mut image,
        &units,
        inode_leaf_of_the_abandoned_root,
        |bytes| {
            assert_eq!(
                bytes[6], 3,
                "槽 {inode_leaf_of_the_abandoned_root} 上还是 C 写下的 inode 叶容器（码 3）"
            );
            let object_birth = INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_OBJECT_BIRTH_OFFSET;
            let another_object_birth = get_u64(bytes, object_birth) + 1;
            set_u64(bytes, object_birth, another_object_birth);
        },
        true,
    );
    image
}

/// I-9.29（同号只许一个出生代） 判不了那一形（`.claude/kb/invariants.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」；
/// 第 22 批 C 实现员报告「交主 agent 的设计判断」第 5 条）。镜像见
/// [`image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth`]：被抛弃的 C 与现行线上的 A、B 同号不同出生代。
/// ① 最新根的实例表读得出：C 按实例 3 的实例表判出局、不进候选集，I-9.29 真被评估过且成立，判红的只是干净镜像上那几条（一条都没有）。
/// ② 再把最新根指着的实例表单元两份各翻一个载荷字节（不重封）：候选集滤不掉 C，I-9.29 报部分没判，不报违例
/// （读不出的那一片由 I-2.1、I-7.2 这几条判红，这里不钉）。
/// 判别力：实例表读不出时照旧跨版本比，C 那条记录的出生代与 A、B 的不同，② 红在 I-9.29 上。
#[test]
fn an_abandoned_root_naming_another_object_birth_leaves_the_one_birth_invariant_partially_judged_once_the_newest_instance_table_does_not_read_back(
) {
    let filtered = image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth(
        "known-bad-one-birth-abandoned-root-unfiltered",
    );
    let verdicts_while_filtered = check_pool_image(&filtered);
    assert_eq!(
        violated_invariants(&verdicts_while_filtered),
        Vec::<&str>::new(),
        "被抛弃的 C 按实例表滤掉了，改它那片 inode 叶不红任何一条：{verdicts_while_filtered:?}"
    );
    assert_eq!(
        verdict(&filtered, ONE_OBJECT_BIRTH_PER_INODE_NUMBER),
        InvariantVerdict::Holds,
        "C 不在候选集里，现行线上各版同号同出生代：I-9.29 真被评估过且成立"
    );
    let mut unfiltered = filtered.clone();
    let instance_table_slot = instance_table_slot_of_the_newest_root(&unfiltered);
    break_both_copies_of_the_unit(
        &mut unfiltered,
        instance_table_slot,
        INSTANCE_TABLE_UNIT_BYTES,
    );
    match verdict(&unfiltered, ONE_OBJECT_BIRTH_PER_INODE_NUMBER) {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.starts_with("最新根的实例表读不出"),
            "部分没判的说明点名最新根的实例表读不出：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => panic!(
            "最新根的实例表（槽 {instance_table_slot}）读不出、候选集滤不掉被抛弃的 C：I-9.29 要报部分没判，实际 {other_verdict:?}"
        ),
    }
}

/// 码 3 打包单元头里声明长度（2 字节）与记录数（2 字节）的偏移，载荷 CRC（4 字节）的偏移与头校验和的端点（`reseal_unit` 码 3 那一支的两个数；
/// D18（块里携带什么信息） 已定项 11）。
const PACKED_UNIT_DECLARED_LENGTH_OFFSET: usize = 8;
const PACKED_UNIT_RECORD_COUNT_OFFSET: usize = 69;
const PACKED_UNIT_PAYLOAD_CRC_OFFSET: usize = 89;
const PACKED_UNIT_HEADER_END: usize = 107;
/// 实例表一条记录的字节数；第 0 字节是种类（0 行、1 链指针记录，别的没登记），链指针记录第 1 字节是「有无下一片」（0 或 1）。
const INSTANCE_TABLE_RECORD_BYTES: usize = 88;
const INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD: u8 = 1;
const INSTANCE_TABLE_RECORD_KIND_UNREGISTERED: u8 = 2;
const CHAIN_RECORD_HAS_NEXT_PAGE_OFFSET: usize = 1;

/// [`image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth`] 那一份（被抛弃的 C 与现行线同号不同出生代）上
/// 再改最新根（实例 3）指着的实例表（两盘同槽，`change` 两份一起改），`reseal` 为真时重封它的两道校验和；新的整单元校验和补到引用它的根槽。
/// 交回改过的镜像与那张实例表的槽号。
fn image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
    tag: &str,
    change: impl Fn(&mut Vec<u8>),
    reseal: bool,
) -> (MemoryPool, u64) {
    let mut image =
        image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth(
            tag,
        );
    let instance_table_slot = instance_table_slot_of_the_newest_root(&image);
    mutate_unit_of(
        &mut image,
        &[(instance_table_slot, INSTANCE_TABLE_UNIT_BYTES)],
        instance_table_slot,
        |bytes| {
            let record_count = usize::from(u16::from_le_bytes([
                bytes[PACKED_UNIT_RECORD_COUNT_OFFSET],
                bytes[PACKED_UNIT_RECORD_COUNT_OFFSET + 1],
            ]));
            assert!(
                record_count >= 2,
                "实例 3 的实例表一片里有行、末尾一条链指针记录：{record_count} 条"
            );
            let chain_record_start =
                INSTANCE_TABLE_FIRST_ROW_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
            assert_eq!(
                (
                    bytes[chain_record_start],
                    bytes[chain_record_start + CHAIN_RECORD_HAS_NEXT_PAGE_OFFSET]
                ),
                (INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD, 0),
                "实例 3 的实例表只有一片：最后一条是链指针记录、没有下一片"
            );
            change(bytes);
        },
        reseal,
    );
    (image, instance_table_slot)
}

/// 攻方 N5-a（`research/prompts/m3-batch23-code-r1-opus-output.md`「一、N5 打中」；判决 `research/prompts/m3-batch23-code-r1-main-verification.md`
/// 第三节第 1 条）：最新根的实例表在链指针记录之前多插一条没登记种类（kind 2）的记录，重封、补到根槽。用户 2026-10-06 给 D18（块里携带什么信息）
/// 已定项 11 补「kind 不是 0 或 1 ⇒ 这一片解不开（表不可读）」（`records/2026-09-30-里程碑三开工第一轮.md`「用户 2026-10-06 弹窗定实例表 kind 那两问」）：
/// 主走读不收这一片的行、走不到末片，候选集滤不掉被抛弃的 C，I-9.29（同号只许一个出生代） 报部分没判。第 10k 批之前这里钉的是「照收照滤、I-9.29 成立」。
/// 判别力：这一片照收、照走到末片（`crates/mutations.tsv` 里点名这条的行）时 C 被滤掉，I-9.29 报成立。
#[test]
fn a_record_of_an_unregistered_kind_before_the_chain_record_of_the_newest_instance_table_leaves_the_one_birth_invariant_partially_judged(
) {
    let (image, instance_table_slot) =
        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
            "known-bad-one-birth-unregistered-record-kind-before-the-chain-record",
            |bytes| insert_a_record_of_an_unregistered_kind_before_the_chain_record(bytes),
            true,
        );
    match verdict(&image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER) {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.starts_with("最新根的实例表读不出"),
            "部分没判的说明点名最新根的实例表读不出：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => panic!(
            "最新根的实例表（槽 {instance_table_slot}）链指针记录之前多一条 kind 2：这一片解不开，候选集滤不掉被抛弃的 C，I-9.29 要报部分没判，实际 {other_verdict:?}"
        ),
    }
}

/// 实例表一片（整个单元的字节，两道校验和由调用方重封）在末尾的链指针记录之前插一条 `record`：链指针记录往后挪一格，记录数与声明长度各加一条。
fn insert_a_record_before_the_chain_record_of_the_instance_table_page(
    bytes: &mut [u8],
    record: &[u8],
) {
    let record_count = usize::from(u16::from_le_bytes([
        bytes[PACKED_UNIT_RECORD_COUNT_OFFSET],
        bytes[PACKED_UNIT_RECORD_COUNT_OFFSET + 1],
    ]));
    let chain_record_start =
        INSTANCE_TABLE_FIRST_ROW_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
    let chain_record =
        bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES].to_vec();
    bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES]
        .copy_from_slice(record);
    bytes[chain_record_start + INSTANCE_TABLE_RECORD_BYTES
        ..chain_record_start + 2 * INSTANCE_TABLE_RECORD_BYTES]
        .copy_from_slice(&chain_record);
    let new_record_count = record_count + 1;
    set_u16(
        bytes,
        PACKED_UNIT_RECORD_COUNT_OFFSET,
        u16::try_from(new_record_count).expect("一片装得下"),
    );
    set_u16(
        bytes,
        PACKED_UNIT_DECLARED_LENGTH_OFFSET,
        u16::try_from(new_record_count * INSTANCE_TABLE_RECORD_BYTES).expect("一片装得下"),
    );
}

/// 攻方 N5-a 的造法：链指针记录之前插一条全零、只有首字节是没登记的种类（kind 2）的记录。
fn insert_a_record_of_an_unregistered_kind_before_the_chain_record(bytes: &mut [u8]) {
    let mut record_of_an_unregistered_kind = vec![0u8; INSTANCE_TABLE_RECORD_BYTES];
    record_of_an_unregistered_kind[0] = INSTANCE_TABLE_RECORD_KIND_UNREGISTERED;
    insert_a_record_before_the_chain_record_of_the_instance_table_page(
        bytes,
        &record_of_an_unregistered_kind,
    );
}

/// 实例表行记录（kind 0）里 flags 字节的偏移（kind 1 + 实例代号 4 + T 8 + W 8，D18（块里携带什么信息） 已定项 11），
/// 与退役的 bit0（「曾标回退行，以后不给它另赋含义」）。
const INSTANCE_TABLE_ROW_FLAGS_OFFSET: usize = 21;
const INSTANCE_TABLE_ROW_FLAGS_RETIRED_BIT: u8 = 0b1;

/// 攻方 N5 那一份基底上最新根的实例表那一片解不开（I-3.14 不成立，第 10k 批）时判红的几条，按 `IMPLEMENTED_INVARIANTS` 的次序：
/// I-3.14 本身；走不到末片、那一片的行不收，被抛弃的 C 进了回退候选集，它那一版引用的单元算进遍历得到的已分配（I-3.1 与记账对不上）、
/// I-9.10 连带红。I-3.14 四形一律如此：引用集合那一遍（`singlefs_checker::walk` 的 `note_instance_table_pages_after_the_first`）
/// 第 10n 批起四形都记数不全，第 10k 批时另三形照记全、多红 I-3.9。量过：第 10n 批副本上这四张镜像 `violated_invariants` 的原样。
const VIOLATED_ONCE_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE: [&str; 3] = [
    "I-3.1",
    "I-9.10",
    INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
];

/// 最新根的实例表第 0 片就解不开时，I-3.8（实例表行唯一且低于挂载根）一行都没收、报不适用的理由（`singlefs_checker::walk` 的
/// `walk_instance_table_chain`，第 10n 批；之前落到缺省理由「这个镜像上没有它判的对象」）。
const INSTANCE_TABLE_ROWS_NOT_JUDGED_AS_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE: &str =
    "最新根的实例表那一片解不开（I-3.14 不成立）、行没收全：没有行可判";

/// 最新根的实例表那一片解不开的四张镜像共用的判法：判红的恰好是 [`VIOLATED_ONCE_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE`]，
/// I-3.8 报不适用、理由是 [`INSTANCE_TABLE_ROWS_NOT_JUDGED_AS_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE`]。
fn assert_the_newest_instance_table_page_does_not_decode(image: &MemoryPool, what: &str) {
    let verdicts = check_pool_image(image);
    assert_eq!(
        violated_invariants(&verdicts),
        VIOLATED_ONCE_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE,
        "{what}：判红的该是 I-3.14 与 C 进候选集带出来的两条：{verdicts:?}"
    );
    assert_eq!(
        verdict(image, "I-3.8"),
        InvariantVerdict::NotApplicable(
            INSTANCE_TABLE_ROWS_NOT_JUDGED_AS_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE
        ),
        "{what}：最新根的实例表第 0 片解不开、一行都没收，I-3.8 报不适用并说明为什么"
    );
}

/// I-3.14（实例表每片记录形态合法，`records/2026-09-30-里程碑三开工第一轮.md`「用户 2026-10-06 弹窗定实例表 kind 那两问」） 的坏镜像，造法照攻方 N5-a：
/// 最新根的实例表在链指针记录之前多插一条 kind 2，重封、补到根槽。实现读到它整张表解不开（恢复 `Failed`、可写挂载拒）；
/// 池级 checker 同一个读法（第 10k 批）：I-3.14 红（种类那一格），这一片的行不收、走不到末片，被抛弃的 C 滤不掉、进了回退候选集，
/// 它那一版的单元连带红 I-3.1、I-9.10（I-9.29 报部分没判，上面那一条用例钉着）；引用集合那一遍同记数不全（第 10n 批，之前多红 I-3.9）；
/// I-3.8 一行都没收、报不适用并给理由（第 10n 批）。第 10j 批时这一片照收、只红 I-3.14。
/// 判别力（`crates/mutations.tsv` 里点名这条的行）：种类那一格恒判成立时只红那两条；这一片照收照走时只红 I-3.14；
/// 引用集合那一遍照记全时多红 I-3.9；I-3.8 落回缺省理由时不适用的理由对不上。
#[test]
fn a_record_of_an_unregistered_kind_before_the_chain_record_of_the_newest_instance_table_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set(
) {
    let (image, instance_table_slot) =
        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
            "known-bad-instance-table-record-form-unregistered-kind",
            |bytes| insert_a_record_of_an_unregistered_kind_before_the_chain_record(bytes),
            true,
        );
    assert_the_newest_instance_table_page_does_not_decode(
        &image,
        &format!("最新根的实例表（槽 {instance_table_slot}）链指针记录之前多一条 kind 2"),
    );
}

/// I-3.14 的坏镜像（定案原话里的「片中间多一条链指针」）：最新根的实例表在链指针记录之前多插一条一模一样的链指针记录（kind 1、无下一片），重封、补到根槽。
/// 最后一条仍是合法的链指针记录，I-3.14 红的是「链指针记录只许在一片末尾」那一格；这一片解不开，判红的另两条与 I-3.8 的理由同上一条用例（第 10k 批、第 10n 批）。
/// 判别力：那一格恒判成立时只红那两条；这一片照收照走时只红 I-3.14（改之前的 checker 两臂里，中间那条 kind 1 被后面那条盖掉）。
#[test]
fn a_second_chain_record_in_the_middle_of_the_newest_instance_table_page_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set(
) {
    let (image, instance_table_slot) =
        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
            "known-bad-instance-table-record-form-chain-record-in-the-middle",
            |bytes| {
                let record_count = usize::from(u16::from_le_bytes([
                    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET],
                    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET + 1],
                ]));
                let chain_record_start = INSTANCE_TABLE_FIRST_ROW_START
                    + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
                let chain_record = bytes
                    [chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES]
                    .to_vec();
                insert_a_record_before_the_chain_record_of_the_instance_table_page(
                    bytes,
                    &chain_record,
                );
            },
            true,
        );
    assert_the_newest_instance_table_page_does_not_decode(
        &image,
        &format!("最新根的实例表（槽 {instance_table_slot}）片中间多一条链指针记录"),
    );
}

/// I-3.14 的坏镜像（定案原话里的「kind 0 行 flags 非 0」）：最新根的实例表第一行的 flags 写成退役的 bit0，重封、补到根槽。
/// 实现读行时拒收（`InstanceRow::parse` 判 flags 非 0）；池级 checker 红 I-3.14 的 flags 那一格，这一片解不开，判红的另两条与 I-3.8 的理由同上（第 10k 批、第 10n 批）。
/// 判别力：那一格恒判成立时只红那两条；这一片照收照走时只红 I-3.14（改之前 checker 哪里都不读 flags）。
#[test]
fn a_row_whose_flags_are_not_zero_in_the_newest_instance_table_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set(
) {
    let (image, instance_table_slot) =
        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
            "known-bad-instance-table-record-form-row-flags",
            |bytes| {
                let first_row_flags =
                    INSTANCE_TABLE_FIRST_ROW_START + INSTANCE_TABLE_ROW_FLAGS_OFFSET;
                assert_eq!(
                    (bytes[INSTANCE_TABLE_FIRST_ROW_START], bytes[first_row_flags]),
                    (0, 0),
                    "实例 3 的实例表第一条是行、flags 是 0"
                );
                bytes[first_row_flags] = INSTANCE_TABLE_ROW_FLAGS_RETIRED_BIT;
            },
            true,
        );
    assert_the_newest_instance_table_page_does_not_decode(
        &image,
        &format!("最新根的实例表（槽 {instance_table_slot}）第一行 flags 写 1"),
    );
}

/// 攻方 N5-c（出处同上）：最新根的实例表载荷 CRC 那 4 字节翻一位、只重封头校验和，整单元校验和补到根槽。主走读头判得过，载荷 CRC 只红 I-2.3、
/// 记录照用、沿链走到末片，候选集滤掉了被抛弃的 C：只红 I-2.3，I-9.29 真被评估过且成立。
/// 判别力：判不了的门另用只读不判的读法（它连载荷 CRC 一起判，对不上就交回 None）时，这里报部分没判。
#[test]
fn a_payload_crc_mismatch_on_the_newest_instance_table_reddens_only_the_payload_crc_and_still_filters_the_abandoned_root_so_the_one_birth_invariant_holds(
) {
    let (image, instance_table_slot) =
        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
            "known-bad-one-birth-newest-instance-table-payload-crc",
            |bytes| {
                bytes[PACKED_UNIT_PAYLOAD_CRC_OFFSET] ^= 1;
                singlefs_core::unit::seal_header_checksum(bytes, PACKED_UNIT_HEADER_END);
            },
            false,
        );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.3"],
        "最新根的实例表（槽 {instance_table_slot}）载荷 CRC 对不上、头与整单元校验和都对：判红的该只有 I-2.3：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER),
        InvariantVerdict::Holds,
        "主走读照用这一片的行、走到了末片，被抛弃的 C 滤掉了：I-9.29 要真被评估过且成立"
    );
}

/// 攻方 N5-b（出处同上）：最新根的实例表链指针记录「有无下一片」写 2，重封、补到根槽。I-3.14 红，这一片解不开（第 10k 批起行不收，
/// 第 10j 批时照收这一片的行），不知道后面还有没有片，走不到末片：I-9.29 报部分没判，不报成立（这一片其实就是整张表，可镜像上分不出「只有一片」与
/// 「后面还有片」）；C 进了候选集，判红的与另三形同是 [`VIOLATED_ONCE_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE`]，I-3.8 的理由同上
/// （第 10k 批时另三形多红 I-3.9：引用集合那一遍只在这一形记数不全；第 10n 批四形同记）。
/// 判别力：把「链指针记录不合法」当成走到了末片时 I-9.29 不报部分没判；照收这一片的行时 C 被滤掉、只红 I-3.14。
#[test]
fn a_malformed_chain_record_on_the_newest_instance_table_leaves_the_one_birth_invariant_partially_judged(
) {
    let (image, instance_table_slot) =
        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
            "known-bad-one-birth-newest-instance-table-malformed-chain-record",
            |bytes| {
                let record_count = usize::from(u16::from_le_bytes([
                    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET],
                    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET + 1],
                ]));
                let chain_record_start = INSTANCE_TABLE_FIRST_ROW_START
                    + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
                bytes[chain_record_start + CHAIN_RECORD_HAS_NEXT_PAGE_OFFSET] = 2;
            },
            true,
        );
    assert_the_newest_instance_table_page_does_not_decode(
        &image,
        &format!("最新根的实例表（槽 {instance_table_slot}）最后一条不是合法的链指针记录"),
    );
    match verdict(&image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER) {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.starts_with("最新根的实例表读不出"),
            "部分没判的说明点名最新根的实例表读不出：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => panic!(
            "主走读没沿最新根的实例表链走到末片（槽 {instance_table_slot} 链指针记录不合法）：I-9.29 要报部分没判，实际 {other_verdict:?}"
        ),
    }
}

/// 指针头部 MAC 16 字节的头一个字节在指针里的偏移（D19（块指针的结构与宽度预算） 已定项 11 的偏移表：MAC 从偏移 0 起）。
const POINTER_MAC_FIRST_BYTE_OFFSET: usize = 0;

/// I-3.14 第五形的造法（用户 2026-10-06 定 K2-1 甲：「有下一片而位置指针头部有一段不是第一版的取值」算这一片解不开，
/// `research/prompts/m3-batch10o-code-r1-forks.md` K2-1 那一行；攻方的形在 `research/prompts/m3-batch10o-code-r1-opus-output.md`「K2-1 打中」）：
/// 唯一那一片末尾的链指针记录改成「有下一片」，位置指针只把偏移 `offset_in_the_pointer` 那一字节写成 1、其余照旧全零。
/// 指针因此不全零（「有下一片时不全零」那一句过得去），头部那一段不是第一版的取值。重封归调用方。
fn make_the_chain_record_name_a_next_page_through_a_pointer_whose_head_byte_is_not_zero(
    bytes: &mut [u8],
    offset_in_the_pointer: usize,
) {
    let record_count = usize::from(u16::from_le_bytes([
        bytes[PACKED_UNIT_RECORD_COUNT_OFFSET],
        bytes[PACKED_UNIT_RECORD_COUNT_OFFSET + 1],
    ]));
    let pointer_start = INSTANCE_TABLE_FIRST_ROW_START
        + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES
        + CHAIN_RECORD_POINTER_OFFSET;
    assert!(
        bytes[pointer_start..pointer_start + NODE_POINTER_BYTES]
            .iter()
            .all(|byte| *byte == 0),
        "末片的链指针记录位置指针清零占位"
    );
    bytes[pointer_start - CHAIN_RECORD_POINTER_OFFSET + CHAIN_RECORD_HAS_NEXT_PAGE_OFFSET] =
        CHAIN_RECORD_HAS_NEXT_PAGE;
    bytes[pointer_start + offset_in_the_pointer] = 1;
}

/// I-3.14 第五形的坏镜像（MAC 那一段）：最新根的实例表唯一那一片末尾的链指针记录改成有下一片、位置指针 MAC 头一个字节写 1，重封、补到根槽。
/// 实现读到它这一片解不开（`InstanceTableChainRecord::parse` 经 `NodePointer::read_with_head_fields_judgement_from` 判 MAC 非 0，交 None）；
/// 池级 checker 同一个读法（第 10u 批）：I-3.14 红（最后一条是合法的链指针记录那一格），这一片的行不收、链不往下走、走不到末片，
/// 判红的另两条与 I-3.8 的理由同另四形（[`assert_the_newest_instance_table_page_does_not_decode`]：I-3.8 一行都没收、报不适用），
/// I-9.29 报部分没判。那条指针不跟随，它的头部 I-2.4 不判、不红。
/// 判别力（`crates/mutations.tsv` 里点名这条的行）：「有下一片」不看头部（或不看 MAC 那一段）时主走读沿这条指针往下走——I-2.4 红、
/// 指到的落点读不出记走读失败、I-3.14 不红，判红的集合对不上。
#[test]
fn a_chain_record_naming_a_next_page_through_a_pointer_whose_mac_is_not_zero_on_the_newest_instance_table_does_not_decode(
) {
    let (image, instance_table_slot) =
        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
            "known-bad-instance-table-record-form-chain-pointer-mac-not-zero",
            |bytes| {
                make_the_chain_record_name_a_next_page_through_a_pointer_whose_head_byte_is_not_zero(
                    bytes,
                    POINTER_MAC_FIRST_BYTE_OFFSET,
                );
            },
            true,
        );
    assert_the_newest_instance_table_page_does_not_decode(
        &image,
        &format!(
            "最新根的实例表（槽 {instance_table_slot}）链指针记录有下一片、位置指针 MAC 不全零"
        ),
    );
    match verdict(&image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER) {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.starts_with("最新根的实例表读不出"),
            "部分没判的说明点名最新根的实例表读不出：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => panic!(
            "主走读没沿最新根的实例表链走到末片（槽 {instance_table_slot} 链指针 MAC 不全零）：I-9.29 要报部分没判，实际 {other_verdict:?}"
        ),
    }
}

/// I-3.14 第五形的坏镜像（extent 偏移那一段）：同上一条，位置指针写 1 的是头部 extent 偏移的低字节（偏移 32）。判红的与 I-3.8 的理由同另四形。
/// 判别力：「有下一片」不看 extent 偏移那一段时主走读沿这条指针往下走，判红的集合对不上。
#[test]
fn a_chain_record_naming_a_next_page_through_a_pointer_whose_extent_offset_is_not_zero_on_the_newest_instance_table_does_not_decode(
) {
    let (image, instance_table_slot) =
        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
            "known-bad-instance-table-record-form-chain-pointer-extent-offset-not-zero",
            |bytes| {
                make_the_chain_record_name_a_next_page_through_a_pointer_whose_head_byte_is_not_zero(
                    bytes,
                    POINTER_EXTENT_OFFSET_FIELD_OFFSET,
                );
            },
            true,
        );
    assert_the_newest_instance_table_page_does_not_decode(
        &image,
        &format!(
            "最新根的实例表（槽 {instance_table_slot}）链指针记录有下一片、位置指针 extent 偏移非 0"
        ),
    );
}

/// 根环里 (checkpoint_txg, 实例) 最大的那条根（择根序）的根记录 512 字节。
fn newest_root_record_bytes(image: &MemoryPool) -> Vec<u8> {
    root_slots()
        .into_iter()
        .map(|(device, offset)| read(image, device, offset, 512))
        .filter(|root| &root[..4] == b"SFSR")
        .max_by_key(|root| {
            (
                get_u64(root, ROOT_RECORD_CHECKPOINT_TXG_OFFSET),
                get_u32(root, ROOT_RECORD_INSTANCE_OFFSET),
            )
        })
        .expect("根环里有根")
}

/// 实例表一片（整个单元的字节，重封归调用方）的载荷换成 `records`、容器号写 `page_index`：记录数与声明长度跟着改，记录之后清零。
fn rewrite_the_records_of_an_instance_table_page(
    bytes: &mut [u8],
    records: &[Vec<u8>],
    page_index: u64,
) {
    bytes[INSTANCE_TABLE_FIRST_ROW_START..].fill(0);
    for (position, record) in records.iter().enumerate() {
        let start = INSTANCE_TABLE_FIRST_ROW_START + position * INSTANCE_TABLE_RECORD_BYTES;
        bytes[start..start + INSTANCE_TABLE_RECORD_BYTES].copy_from_slice(record);
    }
    set_u16(
        bytes,
        PACKED_UNIT_RECORD_COUNT_OFFSET,
        u16::try_from(records.len()).expect("一片装得下"),
    );
    set_u16(
        bytes,
        PACKED_UNIT_DECLARED_LENGTH_OFFSET,
        u16::try_from(records.len() * INSTANCE_TABLE_RECORD_BYTES).expect("一片装得下"),
    );
    set_u64(bytes, PACKED_UNIT_CONTAINER_NUMBER_OFFSET, page_index);
}

/// 实例表行（kind 0）里实例代号（4 字节）与 T（8 字节）的偏移（D18（块里携带什么信息） 已定项 11：kind 1 + 实例代号 4 + T 8 + W 8 + flags 1）。
const INSTANCE_TABLE_ROW_INSTANCE_OFFSET: usize = 1;
const INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET: usize = 5;
/// 被崩溃恢复抛弃的 C 那条根的实例代号（[`image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth`]：
/// A、B 是实例 1，重开取号 2 之后写 C，恢复落到 C 之前那条根之后是实例 3）。
const INSTANCE_OF_THE_ABANDONED_ROOT: u32 = 2;
/// 第 1 片写在哪个槽（两盘同槽）：4 GiB 的盘上远在这段历史写到的单元之后，没人用。
const SLOT_OF_THE_SECOND_INSTANCE_TABLE_PAGE: u64 = 200_000;
/// 一条位置条目里整单元校验和（4 字节）的偏移（盘 4 + 槽 6）。
const LOCATION_ENTRY_CHECKSUM_OFFSET: usize = 10;
/// 链指针记录里位置指针的偏移（kind 1 + 有无下一片 1）与「有下一片」的值。
const CHAIN_RECORD_POINTER_OFFSET: usize = 2;
const CHAIN_RECORD_HAS_NEXT_PAGE: u8 = 1;

/// 第 1 片是哪一形：两份完好、指着它的指针盘 0 那条位置条目指回第 0 片（攻方 H-b），或盘 0 那一份坏（攻方 P2「第 1 片盘 0 那份坏」，H-b 的对照）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SecondInstanceTablePageShape {
    FirstLocationOfItsPointerNamesTheFirstPage,
    CopyOnDeviceZeroBroken,
}

/// 攻方 H-b 与它的对照的底子（`research/prompts/m3-batch25-code-r1-opus-output.md` 第 1.3 节；那一轮的 P2）：
/// [`image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth`] 那一份上，最新根（实例 3）的实例表改成两片——
/// 第 0 片留下别的行，被抛弃的 C 那个实例（实例 2）的行挪到第 1 片。第 1 片照第 0 片的头造（容器号 1、重封），两盘写在
/// [`SLOT_OF_THE_SECOND_INSTANCE_TABLE_PAGE`]；第 0 片末尾的链指针记录指着它（指针头照根记录里的实例表指针，两条位置条目是第 1 片的槽与整单元校验和），
/// 再照 `shape` 改；第 0 片重封、整单元校验和补到根槽。第 1 片的槽不在任何一版的分配记录里（I-3.1、I-3.11 那几条照红，这里不钉）。
fn image_with_the_newest_instance_table_in_two_pages_and_the_abandoned_instance_row_on_the_second(
    tag: &str,
    shape: SecondInstanceTablePageShape,
) -> MemoryPool {
    let mut image =
        image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth(
            tag,
        );
    let first_page_slot = instance_table_slot_of_the_newest_root(&image);
    let first_page = read(&image, 0, first_page_slot * SLOT, INSTANCE_TABLE_UNIT_BYTES);
    let record_count = usize::from(u16::from_le_bytes([
        first_page[PACKED_UNIT_RECORD_COUNT_OFFSET],
        first_page[PACKED_UNIT_RECORD_COUNT_OFFSET + 1],
    ]));
    let records: Vec<Vec<u8>> = (0..record_count)
        .map(|position| {
            let start = INSTANCE_TABLE_FIRST_ROW_START + position * INSTANCE_TABLE_RECORD_BYTES;
            first_page[start..start + INSTANCE_TABLE_RECORD_BYTES].to_vec()
        })
        .collect();
    let (chain_record, rows) = records.split_last().expect("实例表一片里至少有链指针记录");
    assert_eq!(
        (
            chain_record[0],
            chain_record[CHAIN_RECORD_HAS_NEXT_PAGE_OFFSET]
        ),
        (INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD, 0),
        "实例 3 的实例表只有一片"
    );
    let (rows_of_the_abandoned_instance, other_rows): (Vec<Vec<u8>>, Vec<Vec<u8>>) =
        rows.iter().cloned().partition(|row| {
            get_u32(row, INSTANCE_TABLE_ROW_INSTANCE_OFFSET) == INSTANCE_OF_THE_ABANDONED_ROOT
        });
    assert_eq!(
        rows_of_the_abandoned_instance.len(),
        1,
        "实例 3 的实例表里被抛弃的 C 那个实例恰好一行"
    );
    let mut second_page = first_page.clone();
    rewrite_the_records_of_an_instance_table_page(
        &mut second_page,
        &[
            rows_of_the_abandoned_instance[0].clone(),
            chain_record.clone(),
        ],
        1,
    );
    reseal_unit(&mut second_page);
    let second_page_checksum = crc32_castagnoli(&second_page);
    for device in DEVICES {
        let mut copy = second_page.clone();
        if device == 0 && shape == SecondInstanceTablePageShape::CopyOnDeviceZeroBroken {
            copy[INSTANCE_TABLE_UNIT_BYTES - 100] ^= 0xff;
        }
        write(
            &mut image,
            device,
            SLOT_OF_THE_SECOND_INSTANCE_TABLE_PAGE * SLOT,
            &copy,
        );
    }
    let root_record = newest_root_record_bytes(&image);
    let mut pointer = root_record[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
        ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
        .to_vec();
    for (location_index, device) in DEVICES.iter().enumerate() {
        let location = POINTER_FIRST_LOCATION_OFFSET + location_index * POINTER_LOCATION_BYTES;
        let slot = match (shape, *device) {
            (SecondInstanceTablePageShape::FirstLocationOfItsPointerNamesTheFirstPage, 0) => {
                first_page_slot
            }
            (SecondInstanceTablePageShape::FirstLocationOfItsPointerNamesTheFirstPage, _)
            | (SecondInstanceTablePageShape::CopyOnDeviceZeroBroken, _) => {
                SLOT_OF_THE_SECOND_INSTANCE_TABLE_PAGE
            }
        };
        pointer[location..location + 4].copy_from_slice(&device.to_le_bytes());
        pointer[location + LOCATION_ENTRY_SLOT_OFFSET..location + LOCATION_ENTRY_SLOT_OFFSET + 6]
            .copy_from_slice(&slot.to_le_bytes()[..6]);
        pointer[location + LOCATION_ENTRY_CHECKSUM_OFFSET
            ..location + LOCATION_ENTRY_CHECKSUM_OFFSET + 4]
            .copy_from_slice(&second_page_checksum.to_le_bytes());
    }
    let mut chain_record_to_the_second_page = vec![0u8; INSTANCE_TABLE_RECORD_BYTES];
    chain_record_to_the_second_page[0] = INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD;
    chain_record_to_the_second_page[CHAIN_RECORD_HAS_NEXT_PAGE_OFFSET] = CHAIN_RECORD_HAS_NEXT_PAGE;
    chain_record_to_the_second_page[CHAIN_RECORD_POINTER_OFFSET..].copy_from_slice(&pointer);
    let mut first_page_records = other_rows;
    first_page_records.push(chain_record_to_the_second_page);
    mutate_unit_of(
        &mut image,
        &[(first_page_slot, INSTANCE_TABLE_UNIT_BYTES)],
        first_page_slot,
        |bytes| rewrite_the_records_of_an_instance_table_page(bytes, &first_page_records, 0),
        true,
    );
    image
}

/// 攻方 H-b（`research/prompts/m3-batch25-code-r1-opus-output.md` 第 1.3 节）：最新根的实例表两片、被抛弃的 C 那个实例的行在第 1 片，
/// 指着第 1 片的指针盘 0 那条位置条目指回第 0 片（校验和仍是第 1 片的）、盘 1 那条完好（底子见
/// [`image_with_the_newest_instance_table_in_two_pages_and_the_abandoned_instance_row_on_the_second`]）。core 读得出整张表；主走读读第 1 片时
/// 盘 0 那份对不上（I-2.1）、盘 1 那份读回、自证过，「走过」按读回的那一份的落点认（第 10k 批）：第 1 片照走、行收全、走到末片，C 滤掉，
/// I-9.29 真被评估过且成立。判红的与对照（同一份底子、第 1 片盘 0 那一份坏）相同，只多 I-5.1：盘 0 那条位置条目与根记录里的实例表指针
/// 占着同一个落点，是两处引用同一段物理范围。修前（按第 0 条位置条目认走过）第 1 片被当成走过、它的行丢了：I-9.29 部分没判，另多红 I-3.9。
/// 判别力：按第 0 条位置条目认走过（`crates/mutations.tsv` 里点名这条的行）时 I-9.29 报部分没判。
#[test]
fn a_second_instance_table_page_whose_pointer_names_the_first_page_on_device_zero_is_walked_from_the_copy_read_back(
) {
    let control = image_with_the_newest_instance_table_in_two_pages_and_the_abandoned_instance_row_on_the_second(
        "known-bad-second-instance-table-page-copy-on-device-zero-broken",
        SecondInstanceTablePageShape::CopyOnDeviceZeroBroken,
    );
    let control_verdicts = check_pool_image(&control);
    let image = image_with_the_newest_instance_table_in_two_pages_and_the_abandoned_instance_row_on_the_second(
        "known-bad-second-instance-table-page-pointer-names-the-first-page-on-device-zero",
        SecondInstanceTablePageShape::FirstLocationOfItsPointerNamesTheFirstPage,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        verdict(&image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER),
        InvariantVerdict::Holds,
        "第 1 片按读回的盘 1 那一份走：行收全、走到末片，C 滤掉，I-9.29 真被评估过且成立：{verdicts:?}"
    );
    let mut expected = violated_invariants(&control_verdicts);
    expected.push("I-5.1");
    expected.sort_unstable();
    let mut violated = violated_invariants(&verdicts);
    violated.sort_unstable();
    assert_eq!(
        violated,
        expected,
        "判红的与对照（第 1 片盘 0 那一份坏：{:?}）相同、只多 I-5.1：{verdicts:?}",
        violated_invariants(&control_verdicts)
    );
}

/// 只读不判的读法（`singlefs_checker::walk::InstanceTableOfRootRecord::read`，I-3.12、I-7.9 与记录核对器读一条根自己的实例表）与主走读、
/// 与实现同一个读法：行的 flags 不是 0 的一片解不开（D18（块里携带什么信息） 已定项 11「第一版恒 0、非 0 拒收」，第 10k 批）。
/// 镜像：最新根的实例表第一行 flags 写成退役的 bit0、它的 T 抬到不低于最新根的 txg，重封、补到根槽。最新根那张表这条读法读不出，
/// I-3.12（实例表行 T 低于引用它的根的 txg）不判它、不红；第 10k 批之前这条读法照收这一行，I-3.12 红。
/// 判别力：这条读法不看 flags（`crates/mutations.tsv` 里点名这条的行）时读得出、I-3.12 红。
#[test]
fn a_row_whose_flags_are_not_zero_leaves_the_newest_instance_table_unread_by_the_reader_without_judging(
) {
    let clean =
        image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth(
            "known-bad-instance-table-row-flags-unread-by-the-reader-without-judging-clean",
        );
    assert!(
        InstanceTableOfRootRecord::read(&clean, &newest_root_record_bytes(&clean)).is_some(),
        "前提：改之前最新根的实例表这条读法读得出"
    );
    let newest_root_txg = get_u64(
        &newest_root_record_bytes(&clean),
        ROOT_RECORD_CHECKPOINT_TXG_OFFSET,
    );
    let (image, instance_table_slot) =
        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
            "known-bad-instance-table-row-flags-unread-by-the-reader-without-judging",
            |bytes| {
                let first_row = INSTANCE_TABLE_FIRST_ROW_START;
                assert_eq!(
                    (bytes[first_row], bytes[first_row + INSTANCE_TABLE_ROW_FLAGS_OFFSET]),
                    (0, 0),
                    "实例 3 的实例表第一条是行、flags 是 0"
                );
                bytes[first_row + INSTANCE_TABLE_ROW_FLAGS_OFFSET] =
                    INSTANCE_TABLE_ROW_FLAGS_RETIRED_BIT;
                set_u64(
                    bytes,
                    first_row + INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET,
                    newest_root_txg,
                );
            },
            true,
        );
    assert_eq!(
        InstanceTableOfRootRecord::read(&image, &newest_root_record_bytes(&image)),
        None,
        "最新根的实例表（槽 {instance_table_slot}）第一行 flags 写 1：只读不判的读法读不出"
    );
    let found = verdict(
        &image,
        INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
    );
    assert!(
        !matches!(found, InvariantVerdict::Violated(_)),
        "那一行读不出、I-3.12 不判它（T {newest_root_txg} 不低于最新根的 txg 也不红），实际 {found:?}"
    );
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
    /// 只由记录施加出来的那一版 (1, 5) 那次发布的输出（它写了哪些单元、重写了哪几样、那条 journal 记录）。
    version_applied_only_by_its_record: TransactionOutput,
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
        version_applied_only_by_its_record: applied_only_by_its_record,
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
/// ① **误报消失**：可写挂载接在「由记录施加出来、根槽从没落盘」的那一版之后，那一版被换下的固定点 8 个单元（分配记录树 5 个节点 + 记账 + 映射 + 树表）在 defer 里、仍算已分配，
///    根环里没有一条根引用它们；checker 把那一版并进遍历之后 I-3.1（已分配统计对得上） 真被评估过且成立，别的也一条不红。
///    候选集不补这一版（`versions_applied_only_by_records` 交回空），这一段就在 I-3.1 上红、盘 0 上差 131 072 字节（8 槽；2026-10-05 在 `crates/mutations.tsv` 里 `take(0)` 那条变异下量的）。
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
/// 准入抬 F 越得过 5：txg 5 那一槽的根从没落盘、读回全零，盘上全零的根槽另算「从没写过」，不并进抬 F 的上限（用户 2026-10-04 定，
/// 条文还没写回，定案在 `records/2026-09-30-里程碑三开工第一轮.md`）。照 Q3 把它当自证不过的槽并进来时上限是 5、抬到 6 被拒。
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

/// 前后两份整单元校验和（[`unit_checksums`]）里变了的那几个单元：(起点槽, 旧, 新)，交给 [`propagate_into_journal_records`]。
fn units_whose_checksum_changed(
    before: &[(u64, u32)],
    after: &[(u64, u32)],
) -> Vec<(u64, u32, u32)> {
    before
        .iter()
        .zip(after)
        .filter(|((_, old), (_, new))| old != new)
        .map(|((slot, old), (_, new))| (*slot, *old, *new))
        .collect()
}

/// 改法 RV（判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2「由记录施加出来的版本与下一次挂载先施加的那一版」
/// 那一行，攻方报告 `research/prompts/m3-formal-code-b33-r1-opus-output.md` P2-a 的 b3 形）：I-5.4 按版本那一层也判下一次挂载先施加的那一版。
/// 发布 B 的记录已落、根槽没落的崩溃镜像上，B 那棵分配记录树里 A 的 inode 树根那个落点（槽 50244）的记录跨度改成 2
/// （与 [`known_bad_images_of_overlapping_allocation_records`] 第一份同一种改法，两盘各一条），新的整单元校验和沿引用链补到 B 的树表，
/// 再补进 B 那条记录（两块盘各一份）：
/// ① 只红 I-5.4、别的判定与干净的崩溃镜像逐项相同，违例点名「下一次挂载先施加的那一版（实例 1、txg 4）」——改之前这一版整版不进
///    I-5.4，这张镜像一条都不红；
/// ② 可写挂载重建分配器时拒在「同一块盘上两条分配记录罩住同一个槽」上，两块盘逐字节不变：I-5.4 立条时要拦的
///    「下一次可写挂载重建分配器时两条记录罩住同一个槽」就是这一版。
#[test]
fn two_allocation_records_covering_one_slot_in_the_version_the_next_mount_applies_first_redden_only_the_disjointness_invariant(
) {
    let clean = crash_image_with_the_overwrite_record_but_not_its_root(
        "known-bad-overlapping-records-of-the-version-the-next-mount-applies",
    );
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    assert_eq!(
        violated_invariants(&verdicts_on_the_clean_image),
        Vec::<&str>::new(),
        "B 的记录已落、根槽没落的干净崩溃镜像上一条都不许红：{verdicts_on_the_clean_image:?}"
    );
    assert_eq!(
        verdict(&clean, "I-5.4"),
        InvariantVerdict::Holds,
        "干净的崩溃镜像上 I-5.4 要真被评估过且成立"
    );
    let mut image = clean.clone();
    let checksums_before = unit_checksums(&image, &UNITS_AFTER_OVERWRITE);
    assert_eq!(
        mutate_allocation_leaves(
            &mut image,
            &UNITS_AFTER_OVERWRITE,
            ALLOCATION_LEAVES_AFTER_OVERWRITE,
            |bytes| set_allocation_record_span(bytes, INODE_ROOT, 2),
        ),
        2,
        "A 的 inode 树根那一个落点，B 那棵账里两盘各一条"
    );
    let changes = units_whose_checksum_changed(
        &checksums_before,
        &unit_checksums(&image, &UNITS_AFTER_OVERWRITE),
    );
    assert_eq!(
        propagate_into_journal_records(&mut image, &changes),
        2,
        "B 那条记录两块盘各一份，新根段里的树表指针都要补"
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-5.4"],
        "B 那一版只在记录里时，它的两条记录罩住同一个槽要在崩溃镜像上就红、只红 I-5.4：{verdicts:?}"
    );
    assert_eq!(
        verdicts_that_differ_outside(&verdicts, &verdicts_on_the_clean_image, &["I-5.4"]),
        [],
        "I-5.4 之外每一条的判定与干净的崩溃镜像逐项相同"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-5.4") else {
        panic!("上面判过 I-5.4 红");
    };
    assert!(
        detail.starts_with("下一次挂载先施加的那一版（实例 1、txg 4）那棵分配记录树")
            && detail.contains(&format!("槽 {INODE_ROOT} 跨 2 的记录")),
        "红在 B 那一版（下一次挂载先施加的那一版）的分配记录树上、槽 {INODE_ROOT} 跨 2 那一条：{detail}"
    );

    let stream = SharedStream::new();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let image_before_the_mount = memory_pool_of_sparse_devices(&devices);
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
            MountError::Recovery(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
                what: "同一块盘上两条分配记录罩住同一个槽",
            })
        ),
        "重建 B 那一版的分配器时两条记录罩住同一个槽，可写挂载要拒在这一条上：{refusal:?}"
    );
    assert_eq!(
        memory_pool_of_sparse_devices(&devices),
        image_before_the_mount,
        "拒在任何写之前：两块盘逐字节不变（{refusal:?}）"
    );
}

/// 一片分配记录叶（码 2、层级 0、key 宽 10）里第一对「同一块盘、前一条正好接到后一条」的记录，前一条跨度加一（已释放标志照留）：
/// 两条罩住同一个槽（I-5.4 那一行的 Z1-d 形态）。返回改了几条（0 或 1）。
fn widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(
    bytes: &mut [u8],
) -> usize {
    if bytes[INDEX_NODE_LEVEL_OFFSET] != 0 || bytes[INDEX_NODE_KEY_WIDTH_OFFSET] != 10 {
        return 0;
    }
    let record_count = usize::from(u16::from_le_bytes([
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
    ]));
    let record_at = |leaf_bytes: &[u8], record_index: usize| {
        let start = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * record_index;
        let device = u32::from_le_bytes(leaf_bytes[start..start + 4].try_into().expect("4 字节"));
        let mut slot_bytes = [0u8; 8];
        slot_bytes[..6].copy_from_slice(&leaf_bytes[start + 4..start + 10]);
        let span_field = u16::from_le_bytes([leaf_bytes[start + 10], leaf_bytes[start + 11]]);
        (start, device, u64::from_le_bytes(slot_bytes), span_field)
    };
    for record_index in 0..record_count.saturating_sub(1) {
        let (start, device, slot, span_field) = record_at(bytes, record_index);
        let (_, next_device, next_slot, _) = record_at(bytes, record_index + 1);
        let span_slots = span_field & !ALLOCATION_RECORD_RELEASED_FLAG;
        if device == next_device && slot + u64::from(span_slots) == next_slot {
            set_u16(
                bytes,
                start + 10,
                (span_field & ALLOCATION_RECORD_RELEASED_FLAG) | (span_slots + 1),
            );
            return 1;
        }
    }
    0
}

/// 改 `version` 这次发布自己重写的第一片能改的分配记录叶（[`widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts`]），
/// 叶重封、新的整单元校验和沿引用链补上去（叶 → 内部节点 → 根 → 树表），再补进那次发布的记录：`record_copies_of_the_publish` 是
/// 那次发布的记录条数乘两块盘（一条记录的发布是 2，三条记录的是 6），只有它们引用这一版改过的单元。交回改的是哪一片叶。
/// 例：一次可写挂载接在「由记录施加出来、根槽从没落盘」的那一版 (1, 5) 之后的镜像上
/// （[`image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record`]）改 (1, 5) 那一版。
fn overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(
    image: &mut MemoryPool,
    version: &TransactionOutput,
    record_copies_of_the_publish: usize,
) -> u64 {
    let units = units_found_on_device_zero(image);
    let checksums_before = unit_checksums(image, &units);
    let leaf = version
        .units
        .iter()
        .filter(|unit| {
            version.rewritten.contains(&unit.identity)
                && matches!(
                    unit.identity,
                    TransactionUnit::AllocationTree
                        | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
                )
        })
        .map(|unit| unit.slot.0)
        .find(|slot| {
            let mut candidate_leaf = read(image, 0, slot * SLOT, 16384);
            widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(
                &mut candidate_leaf,
            ) == 1
        })
        .expect("这次发布重写的分配记录叶里有一对同盘、首尾相接的记录");
    mutate_unit_of(
        image,
        &units,
        leaf,
        |bytes| {
            assert_eq!(
                widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(bytes),
                1,
                "上面挑叶时这一片改得动"
            );
        },
        true,
    );
    let changes = units_whose_checksum_changed(&checksums_before, &unit_checksums(image, &units));
    assert_eq!(
        propagate_into_journal_records(image, &changes),
        record_copies_of_the_publish,
        "只有这次发布的记录（两块盘各一份）引用这一版改过的单元"
    );
    leaf
}

/// 改法 RV（判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2，攻方报告
/// `research/prompts/m3-formal-code-b33-r1-opus-output.md` P2-a 的 a3 形）：I-5.4 按版本那一层也判由记录施加出来的那几版。
/// 挂载之后 (1, 5) 只由记录施加出来（根环里没有它的根），它自己重写的一片分配记录叶里两条记录罩住同一个槽：只红 I-5.4，
/// 违例点名「由记录施加出来的那一版（实例 1、txg 5）」。改之前这一版整版不进 I-5.4，这张镜像一条都不红。
#[test]
fn two_allocation_records_covering_one_slot_in_a_version_applied_only_by_its_journal_record_redden_only_the_disjointness_invariant(
) {
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-bad-overlapping-records-of-a-version-applied-only-by-its-record",
    );
    let verdicts_on_the_clean_image = check_pool_image(&built.image);
    assert_eq!(
        violated_invariants(&verdicts_on_the_clean_image),
        Vec::<&str>::new(),
        "挂载之后的干净镜像上一条都不许红：{verdicts_on_the_clean_image:?}"
    );
    let mut image = built.image.clone();
    let leaf = overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(
        &mut image,
        &built.version_applied_only_by_its_record,
        2,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-5.4"],
        "(1, 5) 的叶 {leaf} 里两条记录罩住同一个槽：只红 I-5.4：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-5.4") else {
        panic!("上面判过 I-5.4 红");
    };
    assert!(
        detail.starts_with("由记录施加出来的那一版（实例 1、txg 5）那棵分配记录树"),
        "红在由记录施加出来的那一版 (1, 5) 的分配记录树上：{detail}"
    );
}

/// 改法 RV 读不出的两形（攻方报告 `research/prompts/m3-formal-code-b33-r1-opus-output.md` P2-a 的 a1、a2 形，F1、33-c 的同形）：
/// 挂载之后把只由记录施加出来的那一版 (1, 5) 的一个单元两份清零，判红的照改之前那几条（`violated`），I-5.4 不再报成立、
/// 报部分没判并点名「由记录施加出来的那一版（实例 1、txg 5）」（`partially_judged_detail`）。
fn assert_a_unit_of_the_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
    tag: &str,
    slot_of: impl Fn(&ImageAfterAVersionAppliedOnlyByItsJournalRecord) -> u64,
    violated: &[&str],
    partially_judged_detail: &str,
) {
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(tag);
    assert_eq!(
        verdict(&built.image, "I-5.4"),
        InvariantVerdict::Holds,
        "挂载之后的干净镜像上 I-5.4 成立"
    );
    let mut image = built.image.clone();
    zero_both_copies_of_the_unit(
        &mut image,
        slot_of(&built),
        usize::try_from(NODE_BYTES).expect("16384"),
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        violated,
        "判红的照改之前那几条：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-5.4"),
        InvariantVerdict::PartiallyJudged(partially_judged_detail.to_owned()),
        "(1, 5) 那一版读不出的那一格：别的版本的树走到了叶不替它计成立，I-5.4 报部分没判、点名它是由记录施加出来的那一版"
    );
}

/// a1 形：(1, 5) 的树表两份清零。
#[test]
fn the_tree_table_of_a_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
) {
    assert_a_unit_of_the_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
        "known-bad-disjointness-tree-table-of-a-version-applied-only-by-its-record-zeroed",
        |built| built.tree_table_slot_of_the_version_applied_only_by_its_record,
        &["I-2.1", "I-3.1", "I-4.8", "I-7.4"],
        "树表指针非零而树表读不出的版本，认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：由记录施加出来的那一版（实例 1、txg 5）",
    );
}

/// a2 形：(1, 5) 的分配记录树根两份清零。
#[test]
fn the_allocation_record_tree_of_a_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
) {
    assert_a_unit_of_the_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
        "known-bad-disjointness-allocation-root-of-a-version-applied-only-by-its-record-zeroed",
        |built| {
            assert!(
                built
                    .version_applied_only_by_its_record
                    .rewritten
                    .contains(&TransactionUnit::AllocationTree),
                "(1, 5) 重写了自己的分配记录树根，不与别的版本共用"
            );
            built
                .version_applied_only_by_its_record
                .unit(TransactionUnit::AllocationTree)
                .slot
                .0
        },
        &["I-2.1", "I-4.8", "I-7.4"],
        "带树一版根记录里分配记录树根全零那一格成立而没计（那一版自己的分配记录树读不出、走不到叶，别的版本的树走到了叶不替它作保）：由记录施加出来的那一版（实例 1、txg 5）",
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
        // 记根要分配器装着根环表（没装就断言，C551）。这张表只记实例 1 最新那一条根，F 0，下一条根就是写行那次：够这次发布记根用，
        // 不等于可写挂载在实例 1 那一版上读到的环——挂载读到的环里还有 mkfs 写的第 0 代根与实例 1 那次挂载更早发的根
        // （实例 2 的根槽清零了，那几条读不到），
        // 这张表比它少几条根，回收门槛因此偏高（环里最旧有效根取成实例 1 最新那一条）。门槛偏高也不多回收，靠的是这个前提：
        // 这个分配器上没有可回收的已释放记录——它只记着 mkfs 写在单元区里的两个单元、都未释放，写行那次释放的旧链释放代是写行那次的 txg，
        // 比门槛新。所以这一次一个都不回收，账与没装根环表时一样；往这个分配器上加已释放记录之前，先把表换成挂载读到的环。
        let row_publish_txg =
            CheckpointTxg(first_file_of_the_second_instance.root.checkpoint_txg.0 + 1);
        allocator_rebuilt_on_the_first_instance.install_root_ring_occupancy(
            singlefs_core::allocator::RootRingOccupancy::read_from_the_ring(
                parameters().geometry.root_ring_slots_per_region,
                vec![(
                    singlefs_core::root_ring::target_for_publish(
                        newest_root_of_the_first_instance.checkpoint_txg,
                        parameters().geometry.root_ring_slots_per_region,
                    ),
                    singlefs_core::allocator::RootRingOccupant::ValidRoot {
                        checkpoint_txg: newest_root_of_the_first_instance.checkpoint_txg,
                    },
                )],
                CheckpointTxg(0),
                CheckpointTxg(row_publish_txg.0 - 1),
            ),
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
                txg: row_publish_txg,
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
        (65536, 65548),
        "实例 2 发 65536..65548，其中 65541..65548 只有树表条目、没有单元 ⇒ 盘上扫得出来最大的是映射树 65540"
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
    let mut images: Vec<(&'static str, Mutation)> = vec![
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
    ];
    // I-9.27 写 0 那一支的其余六种（判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md` 第二节 K3-3）：照种类 3 那张，
    // A 的树表里那一种条目的头 ID 从 0 改成 inode 树条目的树 ID，每种一张，接在上面那批后面。
    images.extend(
        TREE_KINDS_WITH_A_ZERO_HEAD_IDENTIFIER_BESIDES_THE_ALLOCATION_RECORD_TREE.map(
            |kind| -> (&'static str, Mutation) {
                (
                    HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
                    Box::new(move |image: &mut MemoryPool| {
                        rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
                            image,
                            kind,
                            |_own_tree_identifier, _inode_tree_identifier| 0,
                            |_own_tree_identifier, inode_tree_identifier| inode_tree_identifier,
                        );
                    }),
                )
            },
        ),
    );
    images
}

/// I-9.27 头 ID 写 0 的七种（种类 3、4、6、7、8、11、12）里种类 3 之外的六种；A 的树表里每种恰好一条。
const TREE_KINDS_WITH_A_ZERO_HEAD_IDENTIFIER_BESIDES_THE_ALLOCATION_RECORD_TREE: [u16; 6] = [
    TREE_KIND_ACCOUNTING,
    TREE_KIND_LIVELIST,
    TREE_KIND_SPARSE_SIDE_TABLE,
    TREE_KIND_DEADLIST,
    TREE_KIND_LINEAGE,
    TREE_KIND_LOGGED_OPERATIONS,
];

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
    // 逐张判完再断言：哪几张（下标从 0 数，按清单次序）判红的不是只有它自己那一条，一次全列出来。
    let images_not_reddening_only_their_own_invariant: Vec<String> =
        known_bad_images_of_pointer_journal_and_generation_fields()
            .into_iter()
            .enumerate()
            .filter_map(|(image_index, (invariant, mutation))| {
                let mut image = clean.clone();
                mutation(&mut image);
                let verdicts = check_pool_image(&image);
                let violated = violated_invariants(&verdicts);
                (violated != [invariant]).then(|| {
                    format!(
                        "第 {image_index} 张坏镜像判红的该只有 {invariant}，实际 {violated:?}：{verdicts:?}"
                    )
                })
            })
            .collect();
    assert_eq!(
        images_not_reddening_only_their_own_invariant,
        Vec::<String>::new(),
        "每张坏镜像判红的只有它自己那一条"
    );
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
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
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
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
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
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
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
/// 第五节盲区第 9 条）：全池四个槽的 R 都写 4、整槽校验和重封 ⇒ 一槽都择不出、每一槽都自证过 ⇒ 全池皆废那一格不成立、I-8.5 不判红；
/// 而一块盘都交不出可择的系统配置，这不是一个挂得上的镜像：I-8.5 与别的每一条一样报不适用、带理由，不判成立
/// （`InvariantVerdict`「不适用必须带理由，不许静默算成立」；改之前 I-8.5 在 `chosen.is_empty()` 早退之前判成成立）。一条都不红。
#[test]
fn self_verified_system_configuration_slots_no_reader_can_choose_leave_every_invariant_not_applicable(
) {
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
    let judged: Vec<&(&str, InvariantVerdict)> = verdicts
        .iter()
        .filter(|(_, verdict)| !matches!(verdict, InvariantVerdict::NotApplicable(_)))
        .collect();
    assert_eq!(
        judged,
        Vec::<&(&str, InvariantVerdict)>::new(),
        "一槽都择不出：每一条都报不适用，I-8.5 也不判成立"
    );
    assert!(
        matches!(
            verdict(
                &image,
                SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
            ),
            InvariantVerdict::NotApplicable(_)
        ),
        "自证过而不可择的槽在：I-8.5 不判红，报不适用"
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
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
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
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
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

// ───────── I-9.16 ③ ④、I-5.5 部分没判、I-2.4 根槽留位、带树一版的分配记录树根（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md`）─────────

/// 新池新建文件那一版树表里十二条按发号次序排（从 0 数）：分配记录树第 2 条、deadlist 第 6 条、logged_ops 第 10 条、退役树第 11 条（最末）。
const ALLOCATION_TREE_TABLE_ENTRY_INDEX: usize = 2;
const DEADLIST_TREE_TABLE_ENTRY_INDEX: usize = 6;
const LOGGED_OPERATIONS_TREE_TABLE_ENTRY_INDEX: usize = 10;
const RETIREMENT_TREE_TABLE_ENTRY_INDEX: usize = 11;
/// 种类码登记表（D8（核心索引结构） 已定项 9）里没有的一个码：分配记录那一条改成它，树表里就一条种类 3 都没有。
/// 13 第 50 批登记给了退役树，改用 14。
const TREE_KIND_NOT_REGISTERED: u16 = 14;
/// 新池新建文件那一版树表里 extent 树那条的树 ID（mkfs 的水位 65536）与 inode 树那条的树 ID（65537）：
/// 改成种类 1 的那一条头 ID 照 extent 条目写成 inode 树的号，I-9.27 不先红。
const INODE_TREE_IDENTIFIER_OF_THE_FIRST_FILE_VERSION: u64 = 65537;

/// 树表里第 `index` 条改种类：先核那一条原来的种类，再改（树 ID、根指针不动）。
fn set_the_kind_of_tree_table_entry(
    tree_table: &mut [u8],
    index: usize,
    kind_before: u16,
    kind_after: u16,
) {
    let kind_offset = tree_table_entry_offset(index) + TREE_TABLE_ENTRY_KIND_OFFSET;
    assert_eq!(
        u16::from_le_bytes([tree_table[kind_offset], tree_table[kind_offset + 1]]),
        kind_before,
        "树表第 {index} 条原来的种类"
    );
    set_u16(tree_table, kind_offset, kind_after);
}

/// 树表里第 `index` 条改成种类 1（extent 树），头 ID 写成 inode 树的号（种类 1 的头 ID 该等于同表种类 2 条目的树 ID，I-9.27）。
fn turn_tree_table_entry_into_a_second_extent_entry(
    tree_table: &mut [u8],
    index: usize,
    kind_before: u16,
) {
    set_the_kind_of_tree_table_entry(tree_table, index, kind_before, TREE_KIND_EXTENT);
    set_u64(
        tree_table,
        tree_table_entry_offset(index) + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,
        INODE_TREE_IDENTIFIER_OF_THE_FIRST_FILE_VERSION,
    );
}

/// 树表里去掉第 `index` 条：它后面的条目往前挪 200、末尾 200 字节清零、条目数与声明长度各减一条
/// （码 2 头：key 宽 1 字节在 51、条目数 2 字节在 82 + 2k、声明长度 2 字节在 8；攻方 `harness-probe-tests.rs` 的 e4 造法）。
fn remove_tree_table_entry(tree_table: &mut [u8], index: usize) {
    let key_width = usize::from(tree_table[51]);
    let count_offset = 82 + 2 * key_width;
    let count = usize::from(u16::from_le_bytes([
        tree_table[count_offset],
        tree_table[count_offset + 1],
    ]));
    let start = tree_table_entry_offset(index);
    let end = tree_table_entry_offset(count);
    tree_table.copy_within(start + 200..end, start);
    tree_table[end - 200..end].fill(0);
    set_u16(
        tree_table,
        count_offset,
        u16::try_from(count - 1).expect("条目数装得进 u16"),
    );
    let declared = u16::from_le_bytes([tree_table[8], tree_table[9]]);
    set_u16(tree_table, 8, declared - 200);
}

/// 一份坏镜像上判红的只有 I-9.16、第一处违例是 `expected_violation`；干净镜像上 I-9.16 真被评估过且成立。
fn assert_only_the_tree_table_ordering_invariant_is_violated(
    clean: &MemoryPool,
    image: &MemoryPool,
    expected_violation: &str,
) {
    assert_eq!(
        verdict(clean, "I-9.16"),
        InvariantVerdict::Holds,
        "干净镜像上 I-9.16 真被评估过且成立"
    );
    let verdicts = check_pool_image(image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-9.16"],
        "判红的该只有 I-9.16：{verdicts:?}"
    );
    assert_eq!(
        verdict(image, "I-9.16"),
        InvariantVerdict::Violated(expected_violation.to_owned()),
        "红在 ③ ④ 那一道"
    );
}

/// I-9.16 ③（攻方 b2）：livelist 那一条改成种类 1、头 ID 照 extent 条目写成 inode 树的号——树表里种类 1 两条。
/// 树 ID 次序、发号次序（取先出现的那条）、头 ID 都对得上，改之前整份 checker 一条不红（core 读这一版报「树表里同一种树有两条」）。
#[test]
fn a_tree_table_whose_livelist_entry_became_a_second_extent_entry_reddens_only_the_tree_table_ordering_invariant(
) {
    let clean =
        build_pool("known-bad-tree-table-second-extent-entry-from-the-livelist").memory_pool();
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            turn_tree_table_entry_into_a_second_extent_entry(
                bytes,
                LIVELIST_TREE_TABLE_ENTRY_INDEX,
                TREE_KIND_LIVELIST,
            );
        },
        true,
    );
    assert_only_the_tree_table_ordering_invariant_is_violated(
        &clean,
        &image,
        "根（实例 1、txg 3）的树表：种类 1 有 2 条（③ 每种至多一条）",
    );
}

/// I-9.16 ③（攻方 b3）：logged_ops 那一条（树表最末一条）改成种类 1、头 ID 写成 inode 树的号。
#[test]
fn a_tree_table_whose_logged_operations_entry_became_a_second_extent_entry_reddens_only_the_tree_table_ordering_invariant(
) {
    let clean = build_pool("known-bad-tree-table-second-extent-entry-from-the-logged-operations")
        .memory_pool();
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            turn_tree_table_entry_into_a_second_extent_entry(
                bytes,
                LOGGED_OPERATIONS_TREE_TABLE_ENTRY_INDEX,
                TREE_KIND_LOGGED_OPERATIONS,
            );
        },
        true,
    );
    assert_only_the_tree_table_ordering_invariant_is_violated(
        &clean,
        &image,
        "根（实例 1、txg 3）的树表：种类 1 有 2 条（③ 每种至多一条）",
    );
}

/// I-9.16 ④（正推 e1）：分配记录那一条的种类改成没登记的码——树表里一条种类 3 都没有。
#[test]
fn a_tree_table_whose_allocation_entry_carries_an_unregistered_kind_reddens_only_the_tree_table_ordering_invariant(
) {
    let clean =
        build_pool("known-bad-tree-table-allocation-entry-of-an-unregistered-kind").memory_pool();
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            set_the_kind_of_tree_table_entry(
                bytes,
                ALLOCATION_TREE_TABLE_ENTRY_INDEX,
                TREE_KIND_ALLOCATION,
                TREE_KIND_NOT_REGISTERED,
            );
        },
        true,
    );
    assert_only_the_tree_table_ordering_invariant_is_violated(
        &clean,
        &image,
        "根（实例 1、txg 3）的树表有 12 条，种类 3 一条都没有（④ 树表非空时进树表的十二种各恰好一条）",
    );
}

/// I-9.16 ③（攻方 e2）：deadlist 那一条改成种类 3——树表里种类 3 两条（deadlist 的根是空根，改种类不牵动别的判定）。
#[test]
fn a_tree_table_whose_deadlist_entry_became_a_second_allocation_entry_reddens_only_the_tree_table_ordering_invariant(
) {
    let clean =
        build_pool("known-bad-tree-table-second-allocation-entry-from-the-deadlist").memory_pool();
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            set_the_kind_of_tree_table_entry(
                bytes,
                DEADLIST_TREE_TABLE_ENTRY_INDEX,
                TREE_KIND_DEADLIST,
                TREE_KIND_ALLOCATION,
            );
        },
        true,
    );
    assert_only_the_tree_table_ordering_invariant_is_violated(
        &clean,
        &image,
        "根（实例 1、txg 3）的树表：种类 3 有 2 条（③ 每种至多一条）",
    );
}

/// I-9.16 ④（攻方 e4）：整条去掉分配记录那一条——树表 10 条、没有种类 3。
#[test]
fn a_tree_table_without_its_allocation_entry_reddens_only_the_tree_table_ordering_invariant() {
    let clean = build_pool("known-bad-tree-table-without-the-allocation-entry").memory_pool();
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            let kind_offset = tree_table_entry_offset(ALLOCATION_TREE_TABLE_ENTRY_INDEX)
                + TREE_TABLE_ENTRY_KIND_OFFSET;
            assert_eq!(
                u16::from_le_bytes([bytes[kind_offset], bytes[kind_offset + 1]]),
                TREE_KIND_ALLOCATION,
                "去掉的是分配记录那一条"
            );
            remove_tree_table_entry(bytes, ALLOCATION_TREE_TABLE_ENTRY_INDEX);
        },
        true,
    );
    assert_only_the_tree_table_ordering_invariant_is_violated(
        &clean,
        &image,
        "根（实例 1、txg 3）的树表有 11 条，种类 3 一条都没有（④ 树表非空时进树表的十二种各恰好一条）",
    );
}

/// 树表里去掉第 `index` 条，先核那一条原来的种类是 `kind`。
fn remove_tree_table_entry_of_kind(tree_table: &mut [u8], index: usize, kind: u16) {
    let kind_offset = tree_table_entry_offset(index) + TREE_TABLE_ENTRY_KIND_OFFSET;
    assert_eq!(
        u16::from_le_bytes([tree_table[kind_offset], tree_table[kind_offset + 1]]),
        kind,
        "树表第 {index} 条原来的种类"
    );
    remove_tree_table_entry(tree_table, index);
}

/// I-9.16 ④ 十二种（正推 C1，判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-9.16 ④ 放过缺种类的树表（C1）」
/// 那一行）：整条去掉 deadlist 那一条——树表 11 条、没有种类 8。改之前只要种类 1–4 时整份 checker 一条不红，core 的可写挂载重建这一版时
/// 逐种取号报「树表里没有那棵树」（正推探针 `research/prompts/m3-formal-code-b26-r1-sonnet-model/probe/known_bad_probe.rs.append`）。
#[test]
fn a_tree_table_without_its_deadlist_entry_reddens_only_the_tree_table_ordering_invariant() {
    let clean = build_pool("known-bad-tree-table-without-the-deadlist-entry").memory_pool();
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            remove_tree_table_entry_of_kind(
                bytes,
                DEADLIST_TREE_TABLE_ENTRY_INDEX,
                TREE_KIND_DEADLIST,
            );
        },
        true,
    );
    assert_only_the_tree_table_ordering_invariant_is_violated(
        &clean,
        &image,
        "根（实例 1、txg 3）的树表有 11 条，种类 8 一条都没有（④ 树表非空时进树表的十二种各恰好一条）",
    );
}

/// I-9.16 ④ 十二种（同上判决与探针）：整条去掉退役树那一条（树表最末一条，第 50 批之前最末一条是 logged_ops）——树表 11 条、没有种类 13。
/// 走读另红 I-1.1（树表头里的 key 区间末端还是退役树的号），照实钉住
/// （正推探针改之前量到 `violated=["I-1.1"]`、I-9.16 成立：`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/probe-i916-missing-kind.txt`）。
#[test]
fn a_tree_table_without_its_retirement_entry_reddens_the_tree_table_ordering_invariant_besides_the_walk(
) {
    let clean = build_pool("known-bad-tree-table-without-the-retirement-entry").memory_pool();
    assert_eq!(
        verdict(&clean, "I-9.16"),
        InvariantVerdict::Holds,
        "干净镜像上 I-9.16 真被评估过且成立"
    );
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            remove_tree_table_entry_of_kind(
                bytes,
                RETIREMENT_TREE_TABLE_ENTRY_INDEX,
                singlefs_core::records::TREE_KIND_RETIREMENT,
            );
        },
        true,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-1.1", "I-9.16"],
        "判红的是 I-9.16 加走读带出来的 I-1.1：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-9.16"),
        InvariantVerdict::Violated(
            "根（实例 1、txg 3）的树表有 11 条，种类 13 一条都没有（④ 树表非空时进树表的十二种各恰好一条）"
                .to_owned()
        ),
        "红在 ④ 那一道"
    );
}

/// I-9.16 ④ 十二种：整条去掉 logged_ops 那一条（第 50 批起它不是最末一条，树表头里的 key 区间不变）——树表 11 条、没有种类 12，只红 I-9.16。
#[test]
fn a_tree_table_without_its_logged_operations_entry_reddens_only_the_tree_table_ordering_invariant()
{
    let clean =
        build_pool("known-bad-tree-table-without-the-logged-operations-entry").memory_pool();
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            remove_tree_table_entry_of_kind(
                bytes,
                LOGGED_OPERATIONS_TREE_TABLE_ENTRY_INDEX,
                TREE_KIND_LOGGED_OPERATIONS,
            );
        },
        true,
    );
    assert_only_the_tree_table_ordering_invariant_is_violated(
        &clean,
        &image,
        "根（实例 1、txg 3）的树表有 11 条，种类 12 一条都没有（④ 树表非空时进树表的十二种各恰好一条）",
    );
}

/// I-9.16 ③（攻方 f2）：覆盖写之后那一版（txg 4，不是从 mkfs 水位建树的那一版，I-5.5 不认它）的树表里 deadlist 改成种类 3。
/// I-9.16 按每张树表第一次被走到时判，这一张是 B 自己的。
#[test]
fn the_tree_table_after_the_overwrite_whose_deadlist_entry_became_a_second_allocation_entry_reddens_only_the_tree_table_ordering_invariant(
) {
    let clean = image_after_the_overwrite(
        "known-bad-tree-table-after-the-overwrite-second-allocation-entry",
    );
    let mut image = clean.clone();
    mutate_unit_of(
        &mut image,
        &UNITS_AFTER_OVERWRITE,
        TREE_TABLE_AFTER_OVERWRITE,
        |bytes| {
            set_the_kind_of_tree_table_entry(
                bytes,
                DEADLIST_TREE_TABLE_ENTRY_INDEX,
                TREE_KIND_DEADLIST,
                TREE_KIND_ALLOCATION,
            );
        },
        true,
    );
    assert_only_the_tree_table_ordering_invariant_is_violated(
        &clean,
        &image,
        "根（实例 1、txg 4）的树表：种类 3 有 2 条（③ 每种至多一条）",
    );
}

/// 两块盘上同一个槽那一份单元整个写 0（不重封、不补引用链）。
fn zero_both_copies_of_the_unit(image: &mut MemoryPool, slot: u64, length: usize) {
    for device in DEVICES {
        write(image, device, slot * SLOT, &vec![0u8; length]);
    }
}

/// 新池新建文件写出的分配记录树根（`UNITS` 里 50249，先叶后根）。
const ALLOCATION_RECORD_TREE_ROOT: u64 = 50249;

/// 一份坏镜像上 I-5.5 报部分没判、说明里点名最新那条根（实例 1、txg 3）与 `why_not_judged`，判红的照旧是 `violated`。
fn assert_the_first_data_unit_invariant_is_partially_judged(
    image: &MemoryPool,
    why_not_judged: &str,
    violated: &[&str],
) {
    let verdicts = check_pool_image(image);
    assert_eq!(
        violated_invariants(&verdicts),
        violated,
        "判红的照改之前那几条：{verdicts:?}"
    );
    let expected_unjudged =
        format!("判到的那几版落点都对得上；这几版没判：根（实例 1、txg 3）{why_not_judged}");
    assert_eq!(
        verdict(
            image,
            FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK
        ),
        InvariantVerdict::PartiallyJudged(expected_unjudged),
        "I-5.5 报部分没判，逐版列出没判的那一版与为什么"
    );
}

/// 读不出的单元（两份全零、不补引用链）由 I-2.1、I-4.8、I-7.2、I-7.4 判红。
const VIOLATED_WHEN_A_UNIT_OF_THE_NEWEST_ROOT_IS_ZEROED: [&str; 4] =
    ["I-2.1", "I-4.8", "I-7.2", "I-7.4"];

/// I-5.5（攻方 a1）：最新那条根的树表单元两份全零——认不出那一版是不是从 mkfs 水位建树的那一版，部分没判。
#[test]
fn a_tree_table_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged() {
    let mut image = build_pool("known-bad-first-data-unit-tree-table-zeroed").memory_pool();
    zero_both_copies_of_the_unit(&mut image, TREE_TABLE, 16384);
    assert_the_first_data_unit_invariant_is_partially_judged(
        &image,
        "：树表读不出，认不出它是不是从 mkfs 水位第一次建树的那一版",
        &VIOLATED_WHEN_A_UNIT_OF_THE_NEWEST_ROOT_IS_ZEROED,
    );
}

/// I-5.5（攻方 c1）：extent 树根两份全零——那一版引用的数据单元数不全，部分没判。
#[test]
fn an_extent_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged() {
    let mut image = build_pool("known-bad-first-data-unit-extent-root-zeroed").memory_pool();
    zero_both_copies_of_the_unit(&mut image, EXTENT_ROOT, 16384);
    assert_the_first_data_unit_invariant_is_partially_judged(
        &image,
        "（从 mkfs 水位第一次建树的那一版）：extent 树读不全，它引用的数据单元数不全",
        &VIOLATED_WHEN_A_UNIT_OF_THE_NEWEST_ROOT_IS_ZEROED,
    );
}

/// I-5.5（攻方 e3）：分配记录树根两份全零——落点算不出，部分没判。
#[test]
fn an_allocation_record_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged(
) {
    let mut image = build_pool("known-bad-first-data-unit-allocation-root-zeroed").memory_pool();
    zero_both_copies_of_the_unit(&mut image, ALLOCATION_RECORD_TREE_ROOT, 16384);
    assert_the_first_data_unit_invariant_is_partially_judged(
        &image,
        "（从 mkfs 水位第一次建树的那一版）：分配记录树读不全，落点算不出",
        &VIOLATED_WHEN_A_UNIT_OF_THE_NEWEST_ROOT_IS_ZEROED,
    );
}

/// I-5.4 挂法（正推 C2，判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-5.4 挂法（C2）」那一行）：
/// 分配记录树根两份全零——候选集里一棵分配记录树都走不到叶，带树一版「根记录里分配记录树根全零」那一格成立而不计进评估，I-5.4 报部分没判、
/// 点名没计的那一版（改之前那一格成立一次就压过不适用，报成立：正推探针 `research/prompts/m3-formal-code-b26-r1-sonnet-model/out/probe-i54-new.txt`）。
/// 判红的照旧是读不出的单元那几条。
#[test]
fn an_allocation_record_tree_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
) {
    let mut image = build_pool("known-bad-disjointness-allocation-root-zeroed").memory_pool();
    zero_both_copies_of_the_unit(&mut image, ALLOCATION_RECORD_TREE_ROOT, 16384);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        VIOLATED_WHEN_A_UNIT_OF_THE_NEWEST_ROOT_IS_ZEROED,
        "判红的照改之前那几条：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-5.4"),
        InvariantVerdict::PartiallyJudged(
            "候选集里没有一条根的分配记录树走得到叶，记录之间互不相交一格都没判；带树一版根记录里分配记录树根全零那一格成立而没计：根（实例 1、txg 3）"
                .to_owned()
        ),
        "一棵分配记录树都没走到叶：I-5.4 不报成立，报部分没判"
    );
}

/// I-5.4 按版本（判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2「I-5.4『成立要至少一棵树走到叶』」那一行的零轮收严，
/// 正推探针 `research/prompts/m3-formal-code-b29-r1-sonnet-model/probes/b29_l4_append.rs` 的 `b29_l2_i54_after_zeroing_each_unit_of_the_second_version`）：
/// 覆盖写之后（发布 B）把 B 那一版的分配记录树根两份清零——A 那一版的分配记录树照样走到叶，B 那一版自己的走不到；
/// B 带树、根记录里分配记录树根全零那一格成立而不计进评估，I-5.4 报部分没判、点名 B（改之前 A 的树走到了叶就替 B 计成立，报成立）。
#[test]
fn an_allocation_record_tree_of_the_newest_version_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged_while_the_previous_version_reads_back(
) {
    let clean = image_after_the_overwrite("known-bad-disjointness-newest-allocation-root-zeroed");
    assert_eq!(
        verdict(&clean, "I-5.4"),
        InvariantVerdict::Holds,
        "覆盖写之后的干净镜像上 A、B 两版的分配记录树都走到叶，I-5.4 成立"
    );
    let allocation_root_of_the_newest_version = common_pointer_head_corruption::the_slot_of(
        &common_pointer_head_corruption::tree_table_entry_of_kind(&clean, TREE_KIND_ALLOCATION)
            .root,
    );
    assert_ne!(
        allocation_root_of_the_newest_version.0, ALLOCATION_RECORD_TREE_ROOT,
        "B 写出了自己的分配记录树根，不与 A 共用"
    );
    let mut image = clean.clone();
    zero_both_copies_of_the_unit(
        &mut image,
        allocation_root_of_the_newest_version.0,
        usize::try_from(NODE_BYTES).expect("16384"),
    );
    assert_eq!(
        verdict(&image, "I-5.4"),
        InvariantVerdict::PartiallyJudged(
            "带树一版根记录里分配记录树根全零那一格成立而没计（那一版自己的分配记录树读不出、走不到叶，别的版本的树走到了叶不替它作保）：根（实例 1、txg 4）"
                .to_owned()
        ),
        "B 那一版的分配记录树走不到叶：A 的树走到了叶不替 B 计成立，I-5.4 报部分没判"
    );
}

/// 覆盖写之后的镜像上把 `slot` 那张树表两份清零：I-5.4 报部分没判，点名树表读不出的那几版（`versions_named`），判红的照旧是读不出的单元那几条
/// （`violated`）。改之前这几版整版不进评估，别的版本的树走到了叶就报成立（第 30 批攻方 `B30UNIT slot=… I-5.4=Holds` 那三行，
/// 判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2「I-5.4 按版本：坏盘」那一行的改法 F1）。
fn assert_a_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
    tag: &str,
    slot: u64,
    violated: &[&str],
    versions_named: &str,
) {
    let clean = image_after_the_overwrite(tag);
    assert_eq!(
        verdict(&clean, "I-5.4"),
        InvariantVerdict::Holds,
        "覆盖写之后的干净镜像上 I-5.4 成立"
    );
    let mut image = clean.clone();
    zero_both_copies_of_the_unit(
        &mut image,
        slot,
        usize::try_from(NODE_BYTES).expect("16384"),
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        violated,
        "判红的照改之前那几条：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-5.4"),
        InvariantVerdict::PartiallyJudged(format!(
            "树表指针非零而树表读不出的版本，认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：{versions_named}"
        )),
        "树表读不出的那几版：别的版本的树走到了叶不替它们计成立，I-5.4 报部分没判、逐版点名"
    );
}

/// I-5.4 按版本，树表读不出（改法 F1）：覆盖写之后把 B（实例 1、txg 4）的树表两份清零。
#[test]
fn the_tree_table_of_the_newest_version_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
) {
    assert_a_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
        "known-bad-disjointness-newest-tree-table-zeroed",
        TREE_TABLE_AFTER_OVERWRITE,
        &["I-2.1", "I-4.8", "I-7.2", "I-7.4"],
        "根（实例 1、txg 4）",
    );
}

/// I-5.4 按版本，树表读不出（改法 F1）：覆盖写之后把 A（实例 1、txg 3，新池新建文件那一版）的树表两份清零。
#[test]
fn the_tree_table_of_the_previous_version_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
) {
    assert_a_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
        "known-bad-disjointness-previous-tree-table-zeroed",
        TREE_TABLE,
        &["I-2.1", "I-3.1", "I-4.8", "I-7.4"],
        "根（实例 1、txg 3）",
    );
}

/// I-5.4 按版本，树表读不出（改法 F1）：覆盖写之后把第 0 代树表（mkfs 写的、树表 0 条，第 0 代根与暖机那两条根都指着它）两份清零。
#[test]
fn the_genesis_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
) {
    assert_a_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
        "known-bad-disjointness-genesis-tree-table-zeroed",
        GENESIS_TREE_TABLE,
        &["I-2.1", "I-4.8", "I-7.4"],
        "根（实例 0、txg 0）、根（实例 1、txg 1）、根（实例 1、txg 2）",
    );
}

/// 只做过 mkfs 的池可写挂载两次（第二次写实例 1 那一行）、再在同一个进程里发第一个文件版本：写行那次发布（实例 2、txg 3）与暖机
/// （实例 2、txg 4）是树表 0 条（照抄 mkfs 那片）、根记录直接持有分配记录树的那一版（C512（树表 0 条的一版上被换下的单元记在哪）），
/// 第一个文件版本（实例 2、txg 5）的分配记录树住它自己树表种类 3 那一条。返回镜像与根记录那一项指着的分配记录树根所在的槽（两盘同槽）。
fn image_after_the_first_file_on_the_version_whose_rows_were_written(
    tag: &str,
) -> (MemoryPool, u64) {
    let mut pool = format_pool(tag);
    let mut devices = pool.reopen_recorded();
    let first_instance =
        mount_writable(&parameters(), &mut devices).expect("只做过 mkfs 的池可写挂载：不写行");
    drop(first_instance);
    let row_written = mount_writable(&parameters(), &mut devices).expect("再挂一次：写行");
    let PoolVersion::WithoutFile(version) = &row_written.current else {
        panic!("写行之后现行那一版树表 0 条")
    };
    assert_eq!(
        version.root.tree_table, pool.genesis.root.tree_table,
        "写行那一版的树表照抄 mkfs 那片 0 条的"
    );
    assert_ne!(
        version.root.allocation_record_tree_root,
        singlefs_core::pointer::NodePointer::empty_root(),
        "写行那一版的分配记录树根住根记录"
    );
    let root_record_allocation_tree_root_slot =
        slot_shared_by_both_copies(version.root.allocation_record_tree_root.locations);
    let mut allocator = row_written.allocator.clone();
    {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        let first_file = publish_first_file(
            &mut writer,
            &mut allocator,
            &version.root,
            FirstFile {
                content: &common::file_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            row_written.output.instance,
            &version.record_bytes,
        )
        .expect("写行那一版上发第一个文件版本");
        assert_eq!(
            first_file.root.checkpoint_txg,
            CheckpointTxg(5),
            "第一个文件版本是 txg 5"
        );
    }
    drop(devices);
    (pool.memory_pool(), root_record_allocation_tree_root_slot)
}

/// I-5.4 按版本，树表 0 条、根记录直接持有的分配记录树读不出（改法 F1 的同形，`research/prompts/m3-formal-code-b30-r1-opus-output.md` 第 201 行记下、没造盘面）：
/// 写行那一版之上发了第一个文件版本的镜像上，把写行那一版根记录那一项指着的分配记录树根两份清零。判红的是读不出的单元那几条，
/// 与改之前相同；改之前写行那两版（txg 3、4）整版不进评估，第一个文件版本的树走到了叶，I-5.4 就报成立。
#[test]
fn the_allocation_record_tree_held_by_the_root_record_of_a_version_without_trees_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
) {
    let (clean, root_record_allocation_tree_root_slot) =
        image_after_the_first_file_on_the_version_whose_rows_were_written(
            "known-bad-disjointness-root-record-allocation-tree-zeroed",
        );
    assert_eq!(
        violated_invariants(&check_pool_image(&clean)),
        Vec::<&str>::new(),
        "写行那一版之上发了第一个文件版本的干净镜像上一条都不红"
    );
    assert_eq!(
        verdict(&clean, "I-5.4"),
        InvariantVerdict::Holds,
        "干净镜像上 I-5.4 真被评估过且成立"
    );
    let mut image = clean.clone();
    zero_both_copies_of_the_unit(
        &mut image,
        root_record_allocation_tree_root_slot,
        usize::try_from(NODE_BYTES).expect("16384"),
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.1", "I-3.1", "I-4.8", "I-7.4"],
        "判红的照改之前那几条：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-5.4"),
        InvariantVerdict::PartiallyJudged(
            "树表 0 条、分配记录树由根记录直接持有的版本，那棵树读不出、走不到叶，它自己的分配记录一条都没判（别的版本的树走到了叶不替它作保）：根（实例 2、txg 3）、根（实例 2、txg 4）"
                .to_owned()
        ),
        "写行那两版：第一个文件版本的树走到了叶不替它们计成立，I-5.4 报部分没判、逐版点名"
    );
}

/// 写行那一版之上发了第一个文件版本的镜像（[`image_after_the_first_file_on_the_version_whose_rows_were_written`]）上，
/// 把 checkpoint_txg 为 `txg` 的那条根的树表指针整条写零、重封根槽（每条根照抄进它那几条记录的新根段，I-8.10 照旧成立）。
/// 第一版没有一处写者写得出这种根（core 的根记录只从 mkfs 的树表、上一版照抄或这次发布写出的树表取这条指针）。
/// 返回镜像与根记录那一项指着的分配记录树根所在的槽。
fn image_with_the_tree_table_pointer_of_a_root_written_all_zero(
    tag: &str,
    txg: CheckpointTxg,
) -> (MemoryPool, u64) {
    let (mut image, root_record_allocation_tree_root_slot) =
        image_after_the_first_file_on_the_version_whose_rows_were_written(tag);
    rewrite_the_root_slot_of(&mut image, txg, |bytes| {
        bytes[ROOT_RECORD_TREE_TABLE_POINTER_RANGE].fill(0);
    });
    (image, root_record_allocation_tree_root_slot)
}

/// 根记录里树表指针那 86 字节（D22（单元原子性怎么合成） 已定项 7 的偏移表：树表指针 36 起、树 ID 水位 122）。
const ROOT_RECORD_TREE_TABLE_POINTER_RANGE: std::ops::Range<usize> = 36..122;

/// 树表指针全零的根（用户 2026-10-05 定甲，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第五节第 1 条，
/// 攻方报告 `research/prompts/m3-formal-code-b33-r1-opus-output.md` P2-b 的 y4 形）：最新那条根（第一个文件版本，实例 2、txg 5）的
/// 树表指针整条写零。core 的根记录读者照它读盘 0 槽 0、读不出，可写挂载拒成 `Recovery(UnitUnreadable { slot: SlotNumber(0) })`；
/// 改之前池级 checker 把它读成「这条根没有树表」，一条都不红、I-5.4 报成立。改之后照 core 跟过去：盘 0 槽 0 那两份对不上（I-2.1），
/// 最新的根走不完（I-7.2），从它出发的遍历读不出（I-4.8、I-7.4），I-7.2 的说明写明是树表指针全零。
#[test]
fn an_all_zero_tree_table_pointer_in_the_newest_root_is_followed_like_the_core_and_the_newest_root_does_not_walk_through(
) {
    let (image, _) = image_with_the_tree_table_pointer_of_a_root_written_all_zero(
        "known-bad-all-zero-tree-table-pointer-in-the-newest-root",
        CheckpointTxg(5),
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.1", "I-4.8", "I-7.2", "I-7.4"],
        "最新那条根的树表指针全零：照 core 跟过去读不出，不再一条都不红：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-7.2") else {
        panic!("上面判过 I-7.2 红");
    };
    assert!(
        detail.contains("根（实例 2、txg 5）的树表（根记录里树表指针全零"),
        "I-7.2 写明走不完的是最新那条根、因为树表指针全零：{detail}"
    );
}

/// 同上（攻方报告 P2-b 的 y1 形）：写行之后暖机那条根（实例 2、txg 4，树表 0 条、根记录直接持有分配记录树）的树表指针整条写零。
/// 改之前池级 checker 把它读成「这条根没有树表」、一条都不红；改之后照 core 跟过去：
/// ① 从这条候选根出发的遍历读不出（I-2.1、I-4.8、I-7.4 红），最新的根照样走得完（I-7.2 不红）；
/// ② 只读的几处判法把它与树表读不出同一处置、点名这一版：I-5.4、I-5.5 与 I-9.28 报部分没判，删除态与墓碑谱系两遍报射程里判不了。
#[test]
fn an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged(
) {
    let (image, _) = image_with_the_tree_table_pointer_of_a_root_written_all_zero(
        "known-bad-all-zero-tree-table-pointer-in-an-older-candidate-root",
        CheckpointTxg(4),
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.1", "I-4.8", "I-7.4"],
        "暖机那条根的树表指针全零：从它出发的遍历读不出，最新的根照样走得完：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-5.4"),
        InvariantVerdict::PartiallyJudged(
            "树表指针全零的版本（core 照它读盘 0 槽 0、读不出），认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：根（实例 2、txg 4）"
                .to_owned()
        ),
        "I-5.4 不再当这一版没有树表：与树表读不出同一处置、点名它"
    );
    let InvariantVerdict::PartiallyJudged(first_data_unit_unjudged) = verdict(
        &image,
        FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
    ) else {
        panic!("I-5.5 报部分没判：{verdicts:?}");
    };
    assert!(
        first_data_unit_unjudged.contains(
            "根（实例 2、txg 4）：树表指针全零（core 照它读盘 0 槽 0、读不出），认不出它是不是从 mkfs 水位第一次建树的那一版"
        ),
        "I-5.5 点名树表指针全零的那一版：{first_data_unit_unjudged}"
    );
    let InvariantVerdict::PartiallyJudged(tree_identifier_unjudged) = verdict(
        &image,
        TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
    ) else {
        panic!("I-9.28 报部分没判：{verdicts:?}");
    };
    assert!(
        tree_identifier_unjudged.contains(
            "（实例 2、txg 4）：树表指针全零（core 的根记录读者照它读盘 0 槽 0、读不出）"
        ),
        "I-9.28 点名树表指针全零的那条根：{tree_identifier_unjudged}"
    );
    // 单子里有树表指针全零的根，前缀不再说「树表指针非零」（判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md`
    // 第二节 P2-e：两种都说得通的说法）。
    assert!(
        tree_identifier_unjudged.contains(
            "这几条读得出、自证过的根树表条目没读全（树表指针全零、树表读不出或有条目比字段表短），没进比较："
        ),
        "I-9.28 部分没判的前缀把树表指针全零的根也说得通：{tree_identifier_unjudged}"
    );
    let deletion_scan_gap = InvariantVerdict::NotApplicable(
        "走读到的某一版树表指针全零（core 照它读盘 0 槽 0、读不出）：那一版的删除态射程里判不了",
    );
    assert_eq!(
        verdict(&image, DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY),
        deletion_scan_gap,
        "删除态扫描报射程里判不了，不再当这一版没有队列"
    );
    assert_eq!(
        verdict(&image, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE),
        deletion_scan_gap,
        "墓碑与谱系那一遍报射程里判不了，不再当这一版没有这几棵树"
    );
}

/// 同上（攻方报告 P2-b 的 y2 形）：y1 那张镜像上再把写行那两版（txg 3、4）根记录那一项共用的分配记录树根两份清零。
/// 改之前 I-5.4 的部分没判只点名 txg 3、不点名 txg 4（txg 4 被读成没有树表、不进「树表 0 条」那一支）；
/// 改之后两版都点名：txg 4 在「树表指针全零」那一句里，txg 3 在「根记录直接持有的那棵树读不出」那一句里。
#[test]
fn an_all_zero_tree_table_pointer_beside_an_unreadable_root_record_allocation_tree_names_both_versions_in_the_disjointness_invariant(
) {
    let (mut image, root_record_allocation_tree_root_slot) =
        image_with_the_tree_table_pointer_of_a_root_written_all_zero(
            "known-bad-all-zero-tree-table-pointer-beside-an-unreadable-root-record-allocation-tree",
            CheckpointTxg(4),
        );
    zero_both_copies_of_the_unit(
        &mut image,
        root_record_allocation_tree_root_slot,
        usize::try_from(NODE_BYTES).expect("16384"),
    );
    assert_eq!(
        verdict(&image, "I-5.4"),
        InvariantVerdict::PartiallyJudged(
            "树表指针全零的版本（core 照它读盘 0 槽 0、读不出），认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：根（实例 2、txg 4）；树表 0 条、分配记录树由根记录直接持有的版本，那棵树读不出、走不到叶，它自己的分配记录一条都没判（别的版本的树走到了叶不替它作保）：根（实例 2、txg 3）"
                .to_owned()
        ),
        "写行那两版都点名：txg 4 树表指针全零，txg 3 根记录直接持有的那棵树读不出"
    );
}

/// 新池新建文件那一版里一条走读跟随的指针头部 `change` 那一段改成非第一版的取值、重封、校验和沿引用链补到根槽：池级 checker 只红 I-2.4，
/// 违例那一句是 `expected_violation`（点名是哪条指针、哪一段）。core 读这条指针时拒（判这条指针所在的结构损坏）；改之前池级 checker 只判
/// MAC / nonce 与 extent 偏移，这三段改了一条都不红（第 30 批攻方 `B30ALG … checker_violated=[]` 那几行，判决
/// `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2「池级 checker 不判指针头的算法类型、压缩码、压后长度」那一行的改法 F2）。
fn assert_a_pointer_head_field_outside_the_first_version_reddens_only_the_header_coverage_invariant(
    image: &MemoryPool,
    expected_violation: &str,
) {
    let verdicts = check_pool_image(image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.4"],
        "只红 I-2.4：{verdicts:?}"
    );
    assert_eq!(
        verdict(image, "I-2.4"),
        InvariantVerdict::Violated(expected_violation.to_owned()),
        "I-2.4 的违例那一句写清是哪条指针、哪一段"
    );
}

/// 新池新建文件那一版的 extent 上段叶第一条条目内联的数据指针：头部 `change` 那一段改掉。
fn pool_whose_inline_data_pointer_head_carries(
    tag: &str,
    change: common_pointer_head_corruption::PointerHeadFieldChange,
) -> MemoryPool {
    let mut image = build_pool(tag).memory_pool();
    let singlefs_core::extent_tree::ExtentUpperLeafTarget::InlineDataUnit(data_pointer) =
        common_pointer_head_corruption::first_upper_leaf_entry(&image).target
    else {
        panic!("新池新建文件那一版只有一个数据单元，上段叶第一条内联它的数据指针");
    };
    let extent_root =
        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root;
    common_pointer_head_corruption::change_a_pointer_head_field_inside_the_unit(
        &mut image,
        common_pointer_head_corruption::the_slot_of(&extent_root),
        common_pointer_head_corruption::node_bytes(),
        &common_pointer_head_corruption::data_pointer_bytes(&data_pointer),
        change,
    );
    image
}

/// I-2.4 指针头部算法类型（改法 F2）：extent 上段叶内联的数据指针算法类型 0 → 1（登记过的加密算法，第一版不加密，D9（加密） 已定项 10）。
#[test]
fn an_inline_data_pointer_naming_an_encryption_algorithm_reddens_only_the_header_coverage_invariant(
) {
    let image = pool_whose_inline_data_pointer_head_carries(
        "known-bad-pointer-head-algorithm-type",
        common_pointer_head_corruption::PointerHeadFieldChange::AlgorithmTypeToOne,
    );
    assert_a_pointer_head_field_outside_the_first_version_reddens_only_the_header_coverage_invariant(
        &image,
        "extent 记录的数据指针：指针头部算法类型是 1，第一版不加密、这 1 字节恒 0",
    );
}

/// I-2.4 指针头部压缩算法码（改法 F2）：树表里 inode 树那条条目的根指针压缩算法码 0 → 1（登记表第一版只有码 0，D9（加密） 已定项 14）。
#[test]
fn a_tree_table_root_pointer_naming_an_unregistered_compression_algorithm_reddens_only_the_header_coverage_invariant(
) {
    let mut image = build_pool("known-bad-pointer-head-compression-algorithm-code").memory_pool();
    let inode_root =
        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
    let tree_table = common_pointer_head_corruption::newest_root(&image).tree_table;
    common_pointer_head_corruption::change_a_pointer_head_field_inside_the_unit(
        &mut image,
        common_pointer_head_corruption::the_slot_of(&tree_table),
        common_pointer_head_corruption::node_bytes(),
        &common_pointer_head_corruption::node_pointer_bytes(&inode_root),
        common_pointer_head_corruption::PointerHeadFieldChange::CompressionAlgorithmCodeToOne,
    );
    assert_a_pointer_head_field_outside_the_first_version_reddens_only_the_header_coverage_invariant(
        &image,
        "树 65537（种类 2）的根：指针头部压缩算法码是 1，第一版登记表只有码 0（不压缩）",
    );
}

/// I-2.4 指针头部压后长度（改法 F2）：extent 上段叶内联的数据指针压缩算法码仍 0、压后长度 0 → 1（码 0 时恒 0，D9（加密） 已定项 14）。
#[test]
fn an_inline_data_pointer_carrying_a_compressed_length_without_compression_reddens_only_the_header_coverage_invariant(
) {
    let image = pool_whose_inline_data_pointer_head_carries(
        "known-bad-pointer-head-compressed-length",
        common_pointer_head_corruption::PointerHeadFieldChange::CompressedLengthToOneWithoutCompression,
    );
    assert_a_pointer_head_field_outside_the_first_version_reddens_only_the_header_coverage_invariant(
        &image,
        "extent 记录的数据指针：指针头部压后长度是 1，压缩算法码 0 时这 2 字节恒 0",
    );
}

/// 树表里种类 `kind` 那棵容器树（墓碑树、谱系树）的码 2 根第 0 条条目里指向容器的子指针（条目里分隔 key 8 + 身份引用 26 之后那 86 字节）：
/// 头部 extent 偏移 0 → 1，根重封、新的整单元校验和沿引用链补到根槽
/// （攻方 `research/prompts/m3-formal-code-b29-r1-opus-model/tests/b29_opus_tombstone_container_pointer_head.rs` 的造法）。
fn set_the_extent_offset_of_the_first_container_pointer_in_the_container_tree_root_of_kind(
    image: &mut MemoryPool,
    kind: u16,
) {
    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
    let root_slot = common_pointer_head_corruption::the_slot_of(&root_pointer);
    let node_length = common_pointer_head_corruption::node_bytes();
    let root = singlefs_core::unit::parse_index_node(
        &common_pointer_head_corruption::unit_bytes_at(image, root_slot, node_length),
    )
    .expect("容器树根解得开");
    assert_eq!(root.level, 1, "种类 {kind} 那棵容器树的根直接管容器");
    let first_entry = root.entries.first().expect("根里至少一条条目");
    let child_pointer = first_entry[CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        ..CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET + CONTAINER_TREE_ENTRY_CHILD_POINTER_BYTES]
        .to_vec();
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        root_slot,
        node_length,
        |unit| {
            let occurrences: Vec<usize> = (0..=unit.len() - child_pointer.len())
                .filter(|start| unit[*start..*start + child_pointer.len()] == child_pointer[..])
                .collect();
            assert_eq!(occurrences.len(), 1, "根里这条子指针恰好出现一次");
            let field = occurrences[0] + POINTER_EXTENT_OFFSET_FIELD_OFFSET;
            assert_eq!(
                unit[field..field + 2],
                [0, 0],
                "第一版指针的 extent 偏移恒 0"
            );
            unit[field..field + 2].copy_from_slice(&1u16.to_le_bytes());
        },
    );
}

/// 墓碑树、谱系树码 2 节点条目里子指针的偏移与宽（分隔 key 8 + 身份引用 26 之后的 86 字节码 2 / 码 3 指针）。
const CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET: usize = 34;
const CONTAINER_TREE_ENTRY_CHILD_POINTER_BYTES: usize = 86;

/// E1（判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L4 第一行）：unlink 意图之后，墓碑树码 2 根里指向墓碑容器的
/// 子指针头部 extent 偏移改 1。走读跟随这条指针时按 I-2.4 判 extent 偏移（`.claude/kb/invariants.md` I-2.4 那一行），只红 I-2.4
/// （改之前 `walk_container_tree_root` 只判 MAC / nonce，I-2.4 报成立、一条都不红）。
#[test]
fn an_extent_offset_on_the_container_pointer_of_the_tombstone_tree_root_reddens_only_the_pointer_head_invariant(
) {
    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-container-pointer-extent-offset",
    );
    assert_eq!(
        verdict(&clean, "I-2.4"),
        InvariantVerdict::Holds,
        "unlink 意图之后的干净镜像上 I-2.4 成立"
    );
    let mut image = clean.clone();
    set_the_extent_offset_of_the_first_container_pointer_in_the_container_tree_root_of_kind(
        &mut image,
        TREE_KIND_TOMBSTONE,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.4"],
        "墓碑树根里指向容器的子指针 extent 偏移非 0：只红 I-2.4：{verdicts:?}"
    );
}

/// E1 谱系树那一半：回退到 A 之后（回退那次发布追加一条谱系记录、写出谱系树根与一片谱系容器），谱系树码 2 根里指向谱系容器的子指针
/// 头部 extent 偏移改 1，只红 I-2.4（墓碑树、谱系树走同一个 `walk_container_tree_root`）。
#[test]
fn an_extent_offset_on_the_container_pointer_of_the_lineage_tree_root_reddens_only_the_pointer_head_invariant(
) {
    let rolled_back = image_after_rolling_back_to_the_first_root(
        "known-bad-lineage-container-pointer-extent-offset",
    );
    assert_eq!(
        verdict(&rolled_back.image, "I-2.4"),
        InvariantVerdict::Holds,
        "回退到 A 之后的干净镜像上 I-2.4 成立"
    );
    let mut image = rolled_back.image.clone();
    set_the_extent_offset_of_the_first_container_pointer_in_the_container_tree_root_of_kind(
        &mut image,
        TREE_KIND_LINEAGE,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.4"],
        "谱系树根里指向容器的子指针 extent 偏移非 0：只红 I-2.4：{verdicts:?}"
    );
}

/// I-5.5（攻方 d1）：第一个文件的数据指针第二条位置条目从盘 1 改到盘 0——盘 1 上一个数据单元都没有，该盘跳过、不记，
/// 盘 0 上照判、成立；位置条目不按设备升序，I-2.5 照旧红。
#[test]
fn a_data_pointer_with_no_location_on_one_device_skips_that_device_and_the_first_data_unit_invariant_holds(
) {
    let clean = build_pool("known-bad-first-data-unit-no-location-on-device-one").memory_pool();
    let data_unit_checksum = crc32_castagnoli(&read(&clean, 0, DATA_UNIT * SLOT, 32768));
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        EXTENT_ROOT,
        |bytes| {
            assert!(
                replace_all(
                    bytes,
                    &location_pattern(1, DATA_UNIT, data_unit_checksum),
                    &location_pattern(0, DATA_UNIT, data_unit_checksum),
                ),
                "extent 根里有第一个文件的数据指针"
            );
        },
        true,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.5"],
        "判红的只有位置条目次序那一条：{verdicts:?}"
    );
    assert_eq!(
        verdict(
            &image,
            FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK
        ),
        InvariantVerdict::Holds,
        "盘 1 上没有数据单元：该盘跳过、不记，盘 0 上判到且成立"
    );
}

/// 改 checkpoint_txg 为 `txg` 的那条根的根槽、重算整槽自证校验和，再把每条根抄进它那几条记录的新根段（I-8.10 照旧成立）。
fn rewrite_the_root_slot_of(
    image: &mut MemoryPool,
    txg: CheckpointTxg,
    change: impl Fn(&mut [u8]),
) {
    let (device, offset) = root_slot_of(txg);
    let mut bytes = read(image, device, offset, 512);
    assert_eq!(get_u64(&bytes, 28), txg.0, "那一槽是 txg {} 的根", txg.0);
    change(&mut bytes);
    let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
    bytes[138..170].copy_from_slice(&digest);
    write(image, device, offset, &bytes);
    copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
}

/// 根记录里算法类型（428）、nonce（429 起 12）、MAC（441 起 16）的偏移（D22（单元原子性怎么合成） 已定项 7）。
const ROOT_RECORD_ALGORITHM_OFFSET: usize = 428;
const ROOT_RECORD_NONCE_OFFSET: usize = 429;
const ROOT_RECORD_MAC_OFFSET: usize = 441;

/// 发布 B（txg 4）那条根的根槽留位里改一个字节：只红 I-2.4，第一处违例写明是哪一字节。
fn assert_a_root_slot_reserved_byte_reddens_only_the_header_checksum_coverage_invariant(
    tag: &str,
    offset_in_the_root_record: usize,
    value: u8,
) {
    let clean = image_after_the_overwrite(tag);
    assert_eq!(
        verdict(&clean, "I-2.4"),
        InvariantVerdict::Holds,
        "干净镜像上 I-2.4 真被评估过且成立"
    );
    let mut image = clean.clone();
    rewrite_the_root_slot_of(&mut image, CheckpointTxg(4), |bytes| {
        bytes[offset_in_the_root_record] = value;
    });
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.4"],
        "判红的只有 I-2.4：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-2.4") else {
        panic!("I-2.4 要判红");
    };
    assert!(
        detail.contains("txg 4")
            && detail.contains(&format!(
                "根槽偏移 {offset_in_the_root_record} 的字节是 {value:#04x}"
            )),
        "第一处违例写明是哪条根、哪一字节：{detail}"
    );
}

/// I-2.4 根槽留位（攻方 m4）：算法类型写成 1。
#[test]
fn a_root_slot_whose_algorithm_type_is_one_reddens_only_the_header_checksum_coverage_invariant() {
    assert_a_root_slot_reserved_byte_reddens_only_the_header_checksum_coverage_invariant(
        "known-bad-root-slot-algorithm-type",
        ROOT_RECORD_ALGORITHM_OFFSET,
        1,
    );
}

/// I-2.4 根槽留位（攻方 m5）：nonce 第二个字节写成 0x5a。
#[test]
fn a_root_slot_whose_nonce_has_a_non_zero_byte_reddens_only_the_header_checksum_coverage_invariant()
{
    assert_a_root_slot_reserved_byte_reddens_only_the_header_checksum_coverage_invariant(
        "known-bad-root-slot-nonce",
        ROOT_RECORD_NONCE_OFFSET + 1,
        0x5a,
    );
}

/// I-2.4 根槽留位（攻方 m6）：MAC 第十个字节写成 0x5a。
#[test]
fn a_root_slot_whose_mac_has_a_non_zero_byte_reddens_only_the_header_checksum_coverage_invariant() {
    assert_a_root_slot_reserved_byte_reddens_only_the_header_checksum_coverage_invariant(
        "known-bad-root-slot-mac",
        ROOT_RECORD_MAC_OFFSET + 9,
        0x5a,
    );
}

/// 根记录里「树表 0 条那一版的分配记录树根」86 字节的偏移（D22（单元原子性怎么合成） 已定项 7）。
const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET: usize = 342;
/// 树表条目里根指针 86 字节的偏移（树 ID 8 + 条目长度 2 + 种类 2 + flags 2 之后）。
const TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET: usize = 14;
const NODE_POINTER_BYTES: usize = 86;

/// 带树的一版根记录里分配记录树根写成非零指针（判决第二节 Y1 `allocation_record_tree_root` 那一行；两腿都没造这份）：新池新建文件那条根（txg 3）
/// 那 86 字节抄成同一版树表里分配记录那一条的根指针——指着一份合法的分配记录节点（50249），这一版就有了两棵分配记录树。
/// 判在 I-5.4 下（这一格归属交主 agent 定，见 `walk::judge_allocation_records_disjoint`）。
/// 走读另有两条红，照实钉住：走读把根记录这一项当「树表 0 条那一版的分配记录树」（树 0）走下去，50249 头里是树 65538，红 I-1.3；
/// 同一版里 50249 被根记录这一项与树表种类 3 那一条各引用一次，红 I-5.1。这两条在改之前就红；改之后 I-5.4 多红一条、点名这一版有两棵分配记录树。
#[test]
fn a_file_version_whose_root_record_carries_a_non_zero_allocation_record_tree_root_reddens_the_disjointness_invariant_besides_the_walk(
) {
    let clean =
        build_pool("known-bad-file-version-with-an-allocation-record-tree-root").memory_pool();
    assert_eq!(
        verdict(&clean, "I-5.4"),
        InvariantVerdict::Holds,
        "干净镜像上 I-5.4 真被评估过且成立"
    );
    let tree_table = read(&clean, 0, TREE_TABLE * SLOT, 16384);
    let allocation_entry = tree_table_entry_offset(ALLOCATION_TREE_TABLE_ENTRY_INDEX);
    let allocation_root_pointer =
        tree_table[allocation_entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET
            ..allocation_entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET + NODE_POINTER_BYTES]
            .to_vec();
    let mut image = clean.clone();
    rewrite_the_root_slot_of(&mut image, CheckpointTxg(3), |bytes| {
        let field = &mut bytes[ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET
            ..ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET + NODE_POINTER_BYTES];
        assert!(
            field.iter().all(|byte| *byte == 0),
            "带文件的一版这一项本来全零"
        );
        field.copy_from_slice(&allocation_root_pointer);
    });
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-1.3", "I-5.1", "I-5.4"],
        "判红的是 I-5.4 加走读带出来的 I-1.3、I-5.1：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-5.4"),
        InvariantVerdict::Violated(
            "根（实例 1、txg 3）的树表有 12 条（带树的一版），根记录里树表 0 条那一版的分配记录树根却不是全零：这一版有两棵分配记录树"
                .to_owned()
        ),
        "红在最新那条根"
    );
}

/// 码 2 节点头里的层级（偏移 50）与 key 宽（偏移 51），条目区从 115 + 2 × key 宽 起（D18（块里携带什么信息） 已定项 18）；
/// 墓碑树、谱系树的根条目里类型段在条目偏移 16、身份引用的容器号在 18（分隔 key 8 + 出生树 8 + 类型段 2 之后，D8（核心索引结构） 已定项 6 的条目形态）。
const INDEX_NODE_LEVEL_OFFSET: usize = 50;
const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
const CONTAINER_TREE_ENTRY_TYPE_OFFSET: usize = 16;
const CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET: usize = 18;
/// 墓碑树、谱系树根条目类型段里条款之外的一个值（I-9.23 登记 {0, 1}、I-9.17 登记 {0, 5}）。
const CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED: u16 = 7;

/// 最新的根那一版树表里种类 `kind` 那棵容器树（墓碑树、谱系树）的码 2 根第 0 条条目改一处（`change` 拿到那 120 字节），
/// 根重封、新的整单元校验和沿引用链补到根槽。这几份镜像上这棵树的根是层级 1、直接管容器。
fn change_the_first_entry_of_the_container_tree_root_of_kind(
    image: &mut MemoryPool,
    kind: u16,
    change: impl FnOnce(&mut [u8]),
) {
    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
    let root_slot = common_pointer_head_corruption::the_slot_of(&root_pointer);
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        root_slot,
        common_pointer_head_corruption::node_bytes(),
        |root| {
            assert_eq!(
                root[INDEX_NODE_LEVEL_OFFSET], 1,
                "种类 {kind} 那棵容器树的根直接管容器"
            );
            let entries_start = 115 + 2 * usize::from(root[INDEX_NODE_KEY_WIDTH_OFFSET]);
            change(
                &mut root[entries_start
                    ..entries_start
                        + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
                        + CONTAINER_TREE_ENTRY_CHILD_POINTER_BYTES],
            );
        },
    );
}

/// 一条容器树根条目里身份引用的容器号加一（容器头不动）。
fn add_one_to_the_identity_container_number(entry: &mut [u8]) {
    let field = CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET;
    let container = get_u64(entry, field);
    set_u64(entry, field, container + 1);
}

/// I-9.23（墓碑树条目身份与子头相符） 只红自己的那一形（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T7 取 a）：
/// 三个单元的文件发完意图之后，墓碑树根第一条条目身份引用里的容器号加一、容器头不动。身份不符单独判定、不再记走读失败：
/// 改之前红的是 I-4.8、I-7.2、I-7.4（最新根走不完），现在只红 I-9.23。
#[test]
fn a_tombstone_tree_entry_whose_identity_names_another_container_reddens_only_the_tombstone_entry_invariant(
) {
    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-entry-identity",
    );
    assert_eq!(
        verdict(&clean, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER),
        InvariantVerdict::Holds,
        "意图之后的干净镜像上 I-9.23 真被评估过且成立"
    );
    let mut image = clean.clone();
    change_the_first_entry_of_the_container_tree_root_of_kind(
        &mut image,
        TREE_KIND_TOMBSTONE,
        add_one_to_the_identity_container_number,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "墓碑树根条目的身份与容器头不符：只红 I-9.23，不连带 I-4.8、I-7.2、I-7.4：{verdicts:?}"
    );
}

/// I-9.23 类型段不认得那一支：墓碑树根第一条条目的类型段 1 → 7。孩子先登记引用再判红（判决第二节 T7「类型段不认得时先登记引用再判红」）：
/// 不登记的话那片容器占的槽不进引用并集，I-3.1（已分配统计对得上） 连带红。只红 I-9.23。
#[test]
fn a_tombstone_tree_entry_with_an_unregistered_type_reddens_only_the_tombstone_entry_invariant() {
    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-entry-unregistered-type",
    );
    let mut image = clean.clone();
    change_the_first_entry_of_the_container_tree_root_of_kind(
        &mut image,
        TREE_KIND_TOMBSTONE,
        |entry| {
            assert_eq!(
                u16::from_le_bytes([
                    entry[CONTAINER_TREE_ENTRY_TYPE_OFFSET],
                    entry[CONTAINER_TREE_ENTRY_TYPE_OFFSET + 1]
                ]),
                1,
                "墓碑树根条目的类型段是 1"
            );
            set_u16(
                entry,
                CONTAINER_TREE_ENTRY_TYPE_OFFSET,
                CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
            );
        },
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "墓碑树根条目的类型段不认得：只红 I-9.23，不连带 I-3.1：{verdicts:?}"
    );
}

/// 退役树那五条（I-9.31–I-9.35）。
const RETIREMENT_INVARIANTS: [&str; 5] = [
    RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
];

/// 一份判定里 `invariant` 那一条。
fn verdict_among(
    verdicts: &[(&'static str, InvariantVerdict)],
    invariant: &str,
) -> InvariantVerdict {
    verdicts
        .iter()
        .find(|(identifier, _)| *identifier == invariant)
        .expect("清单里有")
        .1
        .clone()
}

/// 三个单元的文件按批大小 1 删到底（意图、三批）：最后一批删掉 inode 1 的记录，inode 树唯一那片叶容器空了、身份消失，
/// 退役树开第一片容器、写一条 (那片叶的身份, 最后一批的 txg)（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 3 条）。
/// 交回池、删之前那片叶的身份与最后一批那次发布。
fn pool_with_the_three_unit_file_unlinked_to_the_end(
    tag: &str,
) -> (
    BuiltPool,
    singlefs_core::unit::PackedIdentity,
    TransactionOutput,
) {
    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(tag);
    let intent = common_deletion_in_progress::publish_the_intent(&mut pool);
    let [leaf] = intent.inode_leaf_containers.as_slice() else {
        panic!("意图那一版 inode 树一片叶容器")
    };
    let leaf_identity = leaf.contents.identity;
    // 迭代上界：三个数据单元按批大小 1 删，三批删到底。
    for _ in 0..2 {
        common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool);
    }
    let final_batch =
        common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool);
    assert!(final_batch.deleted_inodes.is_empty(), "第三批是最后一批");
    (pool, leaf_identity, final_batch)
}

/// 这一版退役树按叶序的全部退役记录。
fn retirement_records_of(output: &TransactionOutput) -> Vec<RetirementRecord> {
    output
        .retirement_containers
        .iter()
        .flat_map(|container| container.contents.records.iter().copied())
        .collect()
}

/// 写入那一侧与 checker 那一侧对着同一份干净镜像：唯一那片叶容器在最后一批身份消失，那次发布往退役树写恰好一条
/// (那片叶的身份, 最后一批的 txg)；之前各版退役树零根。checker 上退役树那五条都真被评估过且成立，别的条也不红
/// （判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 3、8 条）。
#[test]
fn the_final_batch_that_empties_the_only_leaf_writes_one_retirement_record_and_every_retirement_invariant_holds(
) {
    let (pool, leaf_identity, final_batch) =
        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-clean");
    assert!(
        final_batch.inode_leaf_containers.is_empty(),
        "最后一批之后 inode 树零条目根"
    );
    assert_eq!(
        retirement_records_of(&final_batch),
        [RetirementRecord {
            retired: leaf_identity,
            death_txg: final_batch.root.checkpoint_txg,
        }],
        "身份消失的那次发布写一条退役记录，死亡 txg = 这次发布的 txg"
    );
    let verdicts = check_pool_image(&pool.memory_pool());
    let retirement_verdicts: Vec<(&str, InvariantVerdict)> = RETIREMENT_INVARIANTS
        .iter()
        .map(|invariant| (*invariant, verdict_among(&verdicts, invariant)))
        .collect();
    assert_eq!(
        retirement_verdicts,
        RETIREMENT_INVARIANTS
            .iter()
            .map(|invariant| (*invariant, InvariantVerdict::Holds))
            .collect::<Vec<_>>(),
        "退役树那五条都真被评估过且成立"
    );
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "干净镜像上一条都不红：{verdicts:?}"
    );
}

/// I-9.31（退役树条目身份与子头相符） 只红自己的那一形：删到底那一份上，退役树根第一条条目身份引用里的容器号加一、容器头不动。
#[test]
fn a_retirement_tree_entry_whose_identity_names_another_container_reddens_only_the_retirement_entry_invariant(
) {
    let (pool, _, _) =
        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-entry-identity");
    let mut image = pool.memory_pool();
    change_the_first_entry_of_the_container_tree_root_of_kind(
        &mut image,
        TREE_KIND_RETIREMENT,
        add_one_to_the_identity_container_number,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "退役树根条目的身份与容器头不符：只红 I-9.31：{verdicts:?}"
    );
}

/// 删到底之后建 1 个空 inode（新开一片叶容器，身份出生代 = 那次发布的 txg），再建 [`common_deletion_in_progress::PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER`]
/// 次：删的那几版与建新叶那几版都转出回退候选集（根环一圈 24 槽）。退役树那片容器在删到底那一版之后再没重写过。
/// 交回池、新叶的身份、建新叶那次发布的 txg。
fn pool_with_the_file_unlinked_to_the_end_a_new_leaf_and_the_root_ring_turned_over(
    tag: &str,
) -> (
    BuiltPool,
    singlefs_core::unit::PackedIdentity,
    CheckpointTxg,
) {
    let (mut pool, _, _) = pool_with_the_three_unit_file_unlinked_to_the_end(tag);
    let new_leaf_version =
        common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let [new_leaf] = new_leaf_version.inode_leaf_containers.as_slice() else {
        panic!("建 1 个空 inode 之后 inode 树一片叶容器")
    };
    let new_leaf_identity = new_leaf.contents.identity;
    common_deletion_in_progress::turn_the_root_ring_over_by_creating_inodes(&mut pool);
    (
        pool,
        new_leaf_identity,
        new_leaf_version.root.checkpoint_txg,
    )
}

/// 根环一圈 3 区 × 8 槽：最新那一版往前数这么多版之外的都不在回退候选集里。
const ROOTS_IN_THE_RING: u64 = 24;

/// I-9.32（退役记录合法） 只红自己的那一形：删的那几版转出候选集之后（I-9.35 在记录死的那一版上不再比），那条退役记录被退役身份的
/// 打包记录类型 2 → 1。I-9.33、I-9.34 按身份比，类型 1 的身份既不重也不是活叶，照旧成立。
#[test]
fn a_retirement_record_naming_a_container_of_record_type_one_reddens_only_the_well_formed_retirement_record_invariant(
) {
    let (pool, _, _) =
        pool_with_the_file_unlinked_to_the_end_a_new_leaf_and_the_root_ring_turned_over(
            "known-bad-retirement-record-type",
        );
    let clean = pool.memory_pool();
    let clean_verdicts = check_pool_image(&clean);
    assert_eq!(
        verdict_among(&clean_verdicts, EVERY_RETIREMENT_RECORD_IS_WELL_FORMED),
        InvariantVerdict::Holds,
        "改之前 I-9.32 真被评估过且成立"
    );
    assert_eq!(violated_invariants(&clean_verdicts), Vec::<&str>::new());
    let mut image = clean.clone();
    common_deletion_in_progress::change_the_first_retirement_record_of_the_newest_root(
        &mut image,
        |record| common_deletion_in_progress::set_the_retired_record_type(record, 1),
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [EVERY_RETIREMENT_RECORD_IS_WELL_FORMED],
        "被退役身份的类型不是 2：只红 I-9.32：{verdicts:?}"
    );
}

/// I-9.33（谱系上的退役记录按身份不重复） 只红自己的那一形：删到底那一份上，退役容器里那条记录原样再追加一条。
/// I-9.35 按身份集比，两条同身份收成一个，照旧成立。
#[test]
fn a_second_copy_of_the_retirement_record_reddens_only_the_distinct_retirement_records_invariant() {
    let (pool, _, _) =
        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-record-copy");
    let mut image = pool.memory_pool();
    common_deletion_in_progress::append_a_retirement_record_to_the_first_container_of_the_newest_root(
        &mut image,
        <[u8]>::to_vec,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT],
        "谱系上两条退役记录指同一个身份：只红 I-9.33：{verdicts:?}"
    );
}

/// I-9.34（谱系上的退役记录不指本版活叶） 只红自己的那一形：删的那几版与建新叶那几版转出候选集之后，那条退役记录改成指新叶的身份、
/// 死亡 txg 改成建新叶那次发布的 txg + 1（在每一版父链上、大于新叶的容器出生代、不是候选集里哪一版的 txg）。I-9.32 的几道照旧成立，
/// I-9.35 在死亡 txg 那一版与它的子版上都比不了（父版不在候选集里），只红 I-9.34。
#[test]
fn a_retirement_record_naming_the_live_leaf_reddens_only_the_no_live_leaf_invariant() {
    let (pool, new_leaf_identity, new_leaf_txg) =
        pool_with_the_file_unlinked_to_the_end_a_new_leaf_and_the_root_ring_turned_over(
            "known-bad-retirement-record-live-leaf",
        );
    let death_txg = new_leaf_txg.0 + 1;
    assert!(
        death_txg + ROOTS_IN_THE_RING <= pool.output.root.checkpoint_txg.0,
        "死亡 txg 那一版与它的子版都已转出候选集"
    );
    let new_leaf_identity_bytes = new_leaf_identity_in_record_order(new_leaf_identity);
    let mut image = pool.memory_pool();
    assert_eq!(
        common_deletion_in_progress::identity_of_the_first_inode_leaf_of_the_newest_root(&image),
        new_leaf_identity_bytes,
        "最新那一版 inode 树唯一那片叶就是建新叶那次开的"
    );
    common_deletion_in_progress::change_the_first_retirement_record_of_the_newest_root(
        &mut image,
        |record| {
            record[..common_deletion_in_progress::RETIREMENT_RECORD_IDENTITY_BYTES]
                .copy_from_slice(&new_leaf_identity_bytes);
            common_deletion_in_progress::set_retirement_record_field(
                record,
                common_deletion_in_progress::RETIREMENT_RECORD_DEATH_TXG_OFFSET,
                death_txg,
            );
        },
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF],
        "谱系上的退役记录指这一版的活叶：只红 I-9.34：{verdicts:?}"
    );
}

/// 一个打包记录容器身份按退役记录里被退役身份那 26 字节排（出生树 8、类型 2、容器号 8、容器出生代 8，小端）。
fn new_leaf_identity_in_record_order(identity: singlefs_core::unit::PackedIdentity) -> Vec<u8> {
    RetirementRecord {
        retired: identity,
        death_txg: CheckpointTxg(0),
    }
    .to_bytes()[..common_deletion_in_progress::RETIREMENT_RECORD_IDENTITY_BYTES]
        .to_vec()
}

/// I-9.35（退役完备）漏写那一形，只红自己：删到底那一份上，那条退役记录的死亡 txg 往前挪一（最后一批 → 第二批的 txg）。
/// 最后一批那一版比父版少了那片叶、死于这一版的记录一条都没有；第二批那一版看不到这片容器（退役树那时零根）；I-9.32 的几道照旧成立。
#[test]
fn a_retirement_record_moved_off_the_txg_the_leaf_died_at_reddens_only_the_retirement_completeness_invariant(
) {
    let (pool, _, final_batch) =
        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-record-missing");
    let mut image = pool.memory_pool();
    common_deletion_in_progress::change_the_first_retirement_record_of_the_newest_root(
        &mut image,
        |record| {
            common_deletion_in_progress::set_retirement_record_field(
                record,
                common_deletion_in_progress::RETIREMENT_RECORD_DEATH_TXG_OFFSET,
                final_batch.root.checkpoint_txg.0 - 1,
            );
        },
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION],
        "身份消失的那一版没记退役：只红 I-9.35：{verdicts:?}"
    );
}

/// I-9.35（退役完备）多写那一形，只红自己：删到底那一份上，退役容器追加一条与那条同死亡 txg、容器号加一（这一版父版里没有的身份）。
/// I-9.33 按身份比不重、I-9.34 这一版没有活叶，照旧成立。
#[test]
fn a_retirement_record_naming_a_leaf_the_parent_version_did_not_have_reddens_only_the_retirement_completeness_invariant(
) {
    let (pool, _, _) =
        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-record-extra");
    let mut image = pool.memory_pool();
    common_deletion_in_progress::append_a_retirement_record_to_the_first_container_of_the_newest_root(
        &mut image,
        |first| {
            let mut extra = first.to_vec();
            let container = common_deletion_in_progress::retirement_record_field(
                &extra,
                common_deletion_in_progress::RETIREMENT_RECORD_CONTAINER_OFFSET,
            );
            common_deletion_in_progress::set_retirement_record_field(
                &mut extra,
                common_deletion_in_progress::RETIREMENT_RECORD_CONTAINER_OFFSET,
                container + 1,
            );
            extra
        },
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION],
        "死于这一版的退役记录指父版里没有的身份：只红 I-9.35：{verdicts:?}"
    );
}

/// 退役记录里被退役身份的容器出生代那 8 字节的偏移（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 2 条：出生树 0、
/// 打包记录类型 8、容器号 10、容器出生代 18、死亡 txg 26）。
const RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET: usize = 18;

/// I-9.32（退役记录合法）「容器出生代 < 死亡 txg」那一句只红自己（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F4，
/// 攻方第二次抽样 K7 那条变异）：删的那几版转出候选集之后（I-9.35 在记录死的那一版上不再比），那条退役记录被退役身份的容器出生代改成它自己的
/// 死亡 txg。改了出生代的身份既不重也不是活叶，I-9.33、I-9.34 照旧成立。
#[test]
fn a_retirement_record_whose_container_birth_is_not_below_its_death_txg_reddens_only_the_well_formed_retirement_record_invariant(
) {
    let (pool, _, _) =
        pool_with_the_file_unlinked_to_the_end_a_new_leaf_and_the_root_ring_turned_over(
            "known-bad-retirement-record-born-at-its-death",
        );
    let mut image = pool.memory_pool();
    common_deletion_in_progress::change_the_first_retirement_record_of_the_newest_root(
        &mut image,
        |record| {
            let death_txg = common_deletion_in_progress::retirement_record_field(
                record,
                common_deletion_in_progress::RETIREMENT_RECORD_DEATH_TXG_OFFSET,
            );
            common_deletion_in_progress::set_retirement_record_field(
                record,
                RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET,
                death_txg,
            );
        },
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [EVERY_RETIREMENT_RECORD_IS_WELL_FORMED],
        "被退役身份的容器出生代不小于死亡 txg：只红 I-9.32：{verdicts:?}"
    );
}

/// I-9.32「死亡 txg ≤ 这一版的 txg」那一句只红自己（同上）：同一份基底上那条退役记录的死亡 txg 改成最新那一版的 txg + 1。
/// 每一版里它都晚于那一版、不在谱系上，I-9.33、I-9.34 不判它；没有一版的 txg 等于它，I-9.35 照旧成立。
#[test]
fn a_retirement_record_dying_after_the_newest_version_reddens_only_the_well_formed_retirement_record_invariant(
) {
    let (pool, _, _) =
        pool_with_the_file_unlinked_to_the_end_a_new_leaf_and_the_root_ring_turned_over(
            "known-bad-retirement-record-dies-after-the-newest-version",
        );
    let death_txg_after_the_newest_version = pool.output.root.checkpoint_txg.0 + 1;
    let mut image = pool.memory_pool();
    common_deletion_in_progress::change_the_first_retirement_record_of_the_newest_root(
        &mut image,
        |record| {
            common_deletion_in_progress::set_retirement_record_field(
                record,
                common_deletion_in_progress::RETIREMENT_RECORD_DEATH_TXG_OFFSET,
                death_txg_after_the_newest_version,
            );
        },
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [EVERY_RETIREMENT_RECORD_IS_WELL_FORMED],
        "死亡 txg 晚于每一版：只红 I-9.32：{verdicts:?}"
    );
}

/// I-1.7（打包容器合法与判定顺序） 记录宽那一步对类型 3（退役，登记 34；判决第三节 F4，正推 K6.3 的 C-retwidth）：三个单元的文件删到底之后，
/// 退役容器头里的记录宽 34 → 33。墓碑与谱系那一遍读退役容器时遇到记录宽不是登记的那一个，退役那几条在那一版判不了、不判红：只红 I-1.7。
#[test]
fn a_retirement_container_whose_record_width_is_not_thirty_four_reddens_only_the_packed_container_invariant(
) {
    let (pool, _, _) =
        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-record-width");
    let mut image = pool.memory_pool();
    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
        &mut image,
        TREE_KIND_RETIREMENT,
        u16::try_from(singlefs_format::RETIREMENT_RECORD_BYTES - 1).expect("33"),
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-1.7"],
        "退役容器的记录宽不是 34：只红 I-1.7：{verdicts:?}"
    );
}

/// 写入口装 `capacities`（只供测试的容器树旋钮），把空 inode `inode` unlink 到底：意图那一次，再走空（空 inode 没有数据单元，直接是最后一批）。
fn unlink_an_empty_inode_to_the_end_under_the_capacities(
    pool: &mut BuiltPool,
    inode: u64,
    capacities: singlefs_core::container_tree::ContainerTreeCapacities,
) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_container_tree_capacities(capacities);
    let mut current = singlefs_core::transaction::publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        singlefs_core::address::InodeNumber(inode),
        common_deletion_in_progress::WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        common_deletion_in_progress::INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
    singlefs_core::transaction::publish_unlink_steps_until_the_queue_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
        common_deletion_in_progress::INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的各批：{refusal:?}"));
    pool.output = current;
}

/// 写这几份退役树时退役容器压到的每片记录数（谱系、墓碑与节点照产品）。
const RETIREMENT_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;

/// 新池新建文件之外再建 233 个空 inode（叶 [1..=233]、[234]），写入口装退役容器一片 1 条的旋钮：unlink 234 到底（最右那片删空，退役一条），
/// 再建 1 个（235，最左那片满了，开新叶）、unlink 235 到底（又退役一条）。退役树两片容器各 1 条——攻方 `opus_probe_container_closed_early`
/// 退役那一形（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第二节 K3-4）。旋钮下的合法镜像。
fn pool_with_two_leaves_retired_one_record_per_retirement_container(tag: &str) -> BuiltPool {
    let capacities = singlefs_core::container_tree::ContainerTreeCapacities {
        tombstone_container:
            singlefs_core::tombstone_tree::TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container:
            singlefs_core::lineage_tree::LineageContainerCapacity::FromTheContainerFormat,
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
                singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(
                    RETIREMENT_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
                )
                .expect("1..=959"),
            ),
        node_entries: singlefs_core::container_tree::ContainerTreeNodeEntries::FromTheNodeFormat,
    };
    let mut pool = common_deletion_in_progress::pool_with_new_inodes(tag, 233);
    unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, 234, capacities);
    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, 235, capacities);
    pool
}

/// I-9.36（产品镜像里非最右谱系与退役容器装满，第 53 批）退役树那一形：旋钮压小写出的两片各 1 条的退役树，交给池级 checker 的产品入口
/// （当没装旋钮），不是最右的那片只装 1 条、不是 959：只红 I-9.36（墓碑树同形红 I-9.21）；交给带旋钮值的入口（退役一片 1 条）一条都不红、
/// I-9.37 真被评估过且成立。
#[test]
fn two_leaves_retired_one_record_per_retirement_container_judged_as_a_product_image_reddens_only_the_full_container_invariant(
) {
    let pool = pool_with_two_leaves_retired_one_record_per_retirement_container(
        "known-bad-retirement-container-closed-early",
    );
    assert_eq!(
        pool.output
            .retirement_containers
            .iter()
            .map(|container| container.contents.records.len())
            .collect::<Vec<_>>(),
        [1, 1],
        "退役树两片容器各 1 条"
    );
    let image = pool.memory_pool();
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE],
        "产品入口：不是最右的退役容器只装 1 条：只红 I-9.36：{verdicts:?}"
    );
    let verdicts_given_the_knob =
        singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
            &image,
            singlefs_checker::walk::ContainerCapacityKnobs {
                tombstone: None,
                lineage: None,
                retirement: Some(
                    singlefs_checker::walk::RetirementContainerCapacityKnob::new(
                        RETIREMENT_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
                    )
                    .expect("1..=959"),
                ),
            },
        );
    assert_eq!(
        violated_invariants(&verdicts_given_the_knob),
        Vec::<&str>::new(),
        "带旋钮值的入口：旋钮下的合法镜像一条都不红：{verdicts_given_the_knob:?}"
    );
    assert_eq!(
        verdict_among(
            &verdicts_given_the_knob,
            LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE
        ),
        InvariantVerdict::Holds,
        "I-9.37 在退役树上真被评估过且成立"
    );
}

/// 新池新建文件之后建 1 个空 inode（txg 4），分配器装上谱系容器一片 `lineage_records_per_container` 条的旋钮（回退的发布路径照分配器上的开关取容量），
/// 挂着回退到第一个文件那一版两次：谱系树两条记录。攻方 `opus_probe_container_closed_early` 谱系那一形。
fn pool_rolled_back_twice_under_the_lineage_knob(
    tag: &str,
    lineage_records_per_container: u64,
) -> BuiltPool {
    let mut pool = build_pool(tag);
    let first_file_version = pool.output.clone();
    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    pool.allocator
        .set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
            batch_size: singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
            tombstone_container_capacity:
                singlefs_core::tombstone_tree::TombstoneContainerCapacity::FromTheContainerFormat,
            lineage_container_capacity:
                singlefs_core::lineage_tree::LineageContainerCapacity::CappedForTests(
                    singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(
                        lineage_records_per_container,
                    )
                    .expect("1..=1359"),
                ),
            retirement_container_capacity:
                singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
            container_tree_node_entries:
                singlefs_core::container_tree::ContainerTreeNodeEntries::FromTheNodeFormat,
        });
    common_deletion_in_progress::roll_back_to(&mut pool, &first_file_version);
    common_deletion_in_progress::roll_back_to(&mut pool, &first_file_version);
    pool
}

/// 谱系容器一片 1 条时写这几份谱系树。
const LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;

fn lineage_record_counts_of_each_container(pool: &BuiltPool) -> Vec<usize> {
    pool.output
        .lineage_containers
        .iter()
        .map(|container| container.contents.records.len())
        .collect()
}

fn check_pool_image_given_the_lineage_knob(
    image: &MemoryPool,
    lineage_records_per_container: u64,
) -> Vec<(&'static str, InvariantVerdict)> {
    singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
        image,
        singlefs_checker::walk::ContainerCapacityKnobs {
            tombstone: None,
            lineage: Some(
                singlefs_checker::walk::LineageContainerCapacityKnob::new(
                    lineage_records_per_container,
                )
                .expect("1..=1359"),
            ),
            retirement: None,
        },
    )
}

/// I-9.36 谱系树那一形：旋钮压小写出的两片各 1 条的谱系树，产品入口判不是最右的那片只装 1 条、不是 1359：只红 I-9.36；
/// 带旋钮值的入口（谱系一片 1 条）一条都不红、I-9.37 真被评估过且成立。
#[test]
fn two_rollbacks_one_record_per_lineage_container_judged_as_a_product_image_reddens_only_the_full_container_invariant(
) {
    let pool = pool_rolled_back_twice_under_the_lineage_knob(
        "known-bad-lineage-container-closed-early",
        LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
    );
    assert_eq!(
        lineage_record_counts_of_each_container(&pool),
        [1, 1],
        "谱系树两片容器各 1 条"
    );
    let image = pool.memory_pool();
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE],
        "产品入口：不是最右的谱系容器只装 1 条：只红 I-9.36：{verdicts:?}"
    );
    let verdicts_given_the_knob = check_pool_image_given_the_lineage_knob(
        &image,
        LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
    );
    assert_eq!(
        violated_invariants(&verdicts_given_the_knob),
        Vec::<&str>::new(),
        "带旋钮值的入口：旋钮下的合法镜像一条都不红：{verdicts_given_the_knob:?}"
    );
    assert_eq!(
        verdict_among(
            &verdicts_given_the_knob,
            LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE
        ),
        InvariantVerdict::Holds,
        "I-9.37 在谱系树上真被评估过且成立"
    );
}

/// I-9.37（测试镜像里谱系与退役容器不超旋钮值） 只红自己：谱系容器一片 2 条写出的谱系树（一片 2 条），告诉带旋钮值的入口旋钮是 1：
/// 那片装了 2 条、多于旋钮值，只红 I-9.37（照 I-9.22 那一条的坏镜像）。产品入口上只一片容器，I-9.36 没有对象，一条都不红。
#[test]
fn a_lineage_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant()
{
    let pool = pool_rolled_back_twice_under_the_lineage_knob(
        "known-bad-lineage-container-past-the-knob",
        2,
    );
    assert_eq!(
        lineage_record_counts_of_each_container(&pool),
        [2],
        "谱系树一片容器 2 条"
    );
    let image = pool.memory_pool();
    assert_eq!(
        violated_invariants(&check_pool_image(&image)),
        Vec::<&str>::new(),
        "产品入口：只一片谱系容器，一条都不红"
    );
    let verdicts = check_pool_image_given_the_lineage_knob(
        &image,
        LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
    );
    assert_eq!(
        violated_invariants(&verdicts),
        [LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE],
        "谱系容器装了 2 条、旋钮值 1：只红 I-9.37：{verdicts:?}"
    );
}

/// 合法的回退不红（判决第三节第 5 条「回退照抄现行那一版的退役树」、第 8 条 I-9.33 只比谱系上的）：删到底（退役一条 (叶, F)）→
/// 回退到意图那一版（那片叶又活了，退役树照抄、仍是那一条）→ 再删到底（同一片叶再死一次，追加 (叶, F′)）。
/// 两条同身份，但 F 不在 F′ 那一版的父链上；每一版 checker 都一条不红，退役树那五条都成立。
#[test]
fn unlinking_rolling_back_and_unlinking_again_writes_a_second_record_of_the_same_leaf_and_no_invariant_reddens(
) {
    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(
        "known-bad-retirement-rollback-and-again",
    );
    let intent = common_deletion_in_progress::publish_the_intent(&mut pool);
    let [leaf] = intent.inode_leaf_containers.as_slice() else {
        panic!("意图那一版 inode 树一片叶容器")
    };
    let leaf_identity = leaf.contents.identity;
    // 迭代上界：三批删到底。
    let mut first_final_batch = intent.clone();
    for _ in 0..3 {
        first_final_batch =
            common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool);
    }
    let rollback = common_deletion_in_progress::roll_back_to(&mut pool, &intent);
    assert_eq!(
        retirement_records_of(&rollback),
        retirement_records_of(&first_final_batch),
        "回退那一版退役树照抄现行那一版"
    );
    assert_eq!(
        rollback
            .inode_leaf_containers
            .iter()
            .map(|container| container.contents.identity)
            .collect::<Vec<_>>(),
        [leaf_identity],
        "回退之后那片叶又在 inode 树里"
    );
    assert_eq!(
        violated_invariants(&check_pool_image(&pool.memory_pool())),
        Vec::<&str>::new(),
        "回退那一版一条都不红"
    );
    // 迭代上界：三批删到底。
    let mut second_final_batch = rollback.clone();
    for _ in 0..3 {
        second_final_batch =
            common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool);
    }
    assert!(second_final_batch.deleted_inodes.is_empty(), "再删也是三批");
    assert_eq!(
        retirement_records_of(&second_final_batch),
        [
            RetirementRecord {
                retired: leaf_identity,
                death_txg: first_final_batch.root.checkpoint_txg,
            },
            RetirementRecord {
                retired: leaf_identity,
                death_txg: second_final_batch.root.checkpoint_txg,
            },
        ],
        "同一片叶死第二次再追加一条，第一条不动"
    );
    let verdicts = check_pool_image(&pool.memory_pool());
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "再删到底那一版一条都不红：{verdicts:?}"
    );
    for invariant in RETIREMENT_INVARIANTS {
        assert_eq!(
            verdict_among(&verdicts, invariant),
            InvariantVerdict::Holds,
            "{invariant} 真被评估过且成立"
        );
    }
}

/// I-9.35 的父版取谱系（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条、正推 K5.1：这一版谱系树里有 T_rb 等于
/// 这一版 txg 的记录时父版是那条的 T_old，否则 txg − 1）：删到底（inode 树零条目根）之后建 1 个空 inode（开一片新叶），再回退到删到底那一版，
/// 回退那一版没有那片新叶。父版取 T_old（删到底那一版，同样没有叶），没有叶身份消失、也没有退役记录；按 txg − 1 取父版会拿建新叶那一版比、
/// 判它漏记一片。回退那一版一条都不红，I-9.35 真被评估过且成立。
#[test]
fn rolling_back_past_the_birth_of_a_leaf_compares_with_the_rollback_target_and_no_invariant_reddens(
) {
    let (mut pool, _, final_batch) = pool_with_the_three_unit_file_unlinked_to_the_end(
        "known-bad-retirement-rollback-past-a-leaf-birth",
    );
    let new_leaf_version =
        common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    assert_eq!(
        new_leaf_version.inode_leaf_containers.len(),
        1,
        "建 1 个空 inode 开一片新叶"
    );
    let rollback = common_deletion_in_progress::roll_back_to(&mut pool, &final_batch);
    assert!(
        rollback.inode_leaf_containers.is_empty(),
        "回退那一版 inode 树零条目根"
    );
    assert_eq!(
        retirement_records_of(&rollback),
        retirement_records_of(&final_batch),
        "回退照抄现行那一版的退役树，不写退役记录"
    );
    let verdicts = check_pool_image(&pool.memory_pool());
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "回退到新叶出生之前那一版一条都不红：{verdicts:?}"
    );
    assert_eq!(
        verdict_among(
            &verdicts,
            RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION
        ),
        InvariantVerdict::Holds,
        "I-9.35 真被评估过且成立"
    );
}

/// I-9.27 种类 13（退役树，第 50 批；头 ID 写 inode 树的树 ID，同墓碑树）：删到底那一份上，最新那一版树表里退役树那一条的头 ID 从 inode 树的
/// 树 ID 改成 0、配不上任何一条 inode 树条目。只红 I-9.27；退役记录对叶容器的那两判（I-9.34、I-9.35）在那一版判不了、理由写头 ID 配不上，
/// 不按「这棵 inode 树没有退役记录」判漏记。
#[test]
fn a_retirement_tree_entry_whose_head_identifier_names_no_inode_tree_reddens_only_the_head_identifier_invariant(
) {
    let (pool, _, _) =
        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-head-zero");
    let mut image = pool.memory_pool();
    let tree_table_slot = common_deletion_in_progress::tree_table_slot_of_the_newest_root(&image);
    common_deletion_in_progress::set_the_head_identifier_of_the_tree_table_entry_of_kind_at(
        &mut image,
        tree_table_slot,
        TREE_KIND_RETIREMENT,
        0,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES],
        "退役树条目的头 ID 配不上 inode 树：只红 I-9.27：{verdicts:?}"
    );
    for invariant in [
        RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
        RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
    ] {
        let found = verdict_among(&verdicts, invariant);
        assert!(
            matches!(found, InvariantVerdict::NotApplicable(reason) if reason.contains("头 ID 配不上")),
            "{invariant} 在头 ID 配不上的那一版判不了：{found:?}"
        );
    }
}

/// I-9.17（谱系树条目身份与子头相符） 只红自己的那一形：挂着的时候回退到 A 之后（回退那次发布写出谱系树），谱系树根第一条条目身份引用里的
/// 容器号加一、容器头不动。
#[test]
fn a_lineage_tree_entry_whose_identity_names_another_container_reddens_only_the_lineage_entry_invariant(
) {
    let rolled_back =
        image_after_rolling_back_to_the_first_root("known-bad-lineage-entry-identity");
    let mut image = rolled_back.image.clone();
    change_the_first_entry_of_the_container_tree_root_of_kind(
        &mut image,
        TREE_KIND_LINEAGE,
        add_one_to_the_identity_container_number,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "谱系树根条目的身份与容器头不符：只红 I-9.17：{verdicts:?}"
    );
}

/// I-1.7（打包容器合法与判定顺序） 记录宽那一步对类型 1（墓碑，登记 40）：三个单元的文件发完意图之后，墓碑容器头里的记录宽 40 → 39。
/// 墓碑与谱系那一遍、I-9.6 墓碑那一半读容器时遇到记录宽不是登记的那一个，那一版判不了、不判红：只红 I-1.7。
#[test]
fn a_tombstone_container_whose_record_width_is_not_forty_reddens_only_the_packed_container_invariant(
) {
    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-record-width",
    );
    let mut image = clean.clone();
    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
        &mut image,
        TREE_KIND_TOMBSTONE,
        39,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-1.7"],
        "墓碑容器的记录宽不是 40：只红 I-1.7：{verdicts:?}"
    );
}

/// I-1.7 记录宽那一步对类型 5（谱系，登记 24）：回退到 A 之后，谱系容器头里的记录宽 24 → 15。只红 I-1.7（I-9.30 读不出谱系，那一判报射程里判不了）。
#[test]
fn a_lineage_container_whose_record_width_is_not_the_registered_one_reddens_only_the_packed_container_invariant(
) {
    let rolled_back = image_after_rolling_back_to_the_first_root("known-bad-lineage-record-width");
    let mut image = rolled_back.image.clone();
    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
        &mut image,
        TREE_KIND_LINEAGE,
        15,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-1.7"],
        "谱系容器的记录宽不是 24：只红 I-1.7：{verdicts:?}"
    );
}

/// I-9.4（容器号不超最小 key） 第四句（同一片打包记录类型 2 容器里记录号沿记录序严格递增，判决第二节 T11 取甲）只红自己的那一形
/// （S4a：`research/prompts/m3-batch20-code-r1-main-verification.md` 第二节）：第一个文件之后建一个空 inode 的那一版，叶 [1, 2] 改成
/// [1 nlink 0, 1 nlink 1]、队列空。改之前一条都不红。
#[test]
fn a_leaf_holding_the_same_inode_number_twice_reddens_only_the_container_number_invariant() {
    let clean =
        common_deletion_in_progress::pool_with_new_inodes("known-bad-leaf-same-inode-twice", 1)
            .memory_pool();
    let mut image = clean.clone();
    common_deletion_in_progress::turn_the_leaf_into_two_records_of_the_first_file(&mut image);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-9.4"],
        "同一片叶里 1 号两条：只红 I-9.4：{verdicts:?}"
    );
    assert!(
        matches!(verdict(&image, "I-9.4"), InvariantVerdict::Violated(detail) if detail.contains("inode 1 之后是 inode 1")),
        "违例说明写出不严格递增的那一对"
    );
}

/// 容器树与 inode 树的码 2 节点条目宽（分隔 key 8 + 身份引用 26 + 子指针 86），身份引用里出生树在条目偏移 8、容器号与出生代在 18 起那 16 字节。
const INTERNAL_ENTRY_BYTES: usize = 120;
const INTERNAL_ENTRY_IDENTITY_BIRTH_TREE: std::ops::Range<usize> = 8..16;
const INTERNAL_ENTRY_IDENTITY_CONTAINER_AND_BIRTH: std::ops::Range<usize> = 18..34;
/// 墓碑树、谱系树根长高之后根的条目类型段（指下一层码 2 节点）；墓碑容器、谱系容器的打包记录类型（D18（块里携带什么信息） 已定项 24）。
const CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE: u16 = 0;
const PACKED_RECORD_TYPE_TOMBSTONE: u16 = 1;
const PACKED_RECORD_TYPE_LINEAGE: u16 = 5;
const PACKED_RECORD_TYPE_INODE: u16 = 2;
/// 码 3 单元头里容器号的偏移（类身份段：出生树 43、打包记录类型 51、容器号 53、容器出生代 61）。
const PACKED_UNIT_CONTAINER_NUMBER_OFFSET: usize = 53;

/// 最新的根那一版树表里种类 `kind` 那棵树（墓碑树、谱系树、inode 树）的码 2 根第 `entry_index` 条条目改一处（`change` 拿到那 120 字节），
/// 根重封、新的整单元校验和沿引用链补到根槽；交回根的层级。
fn change_an_entry_of_the_root_of_kind(
    image: &mut MemoryPool,
    kind: u16,
    entry_index: usize,
    change: impl FnOnce(&mut [u8]),
) -> u8 {
    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
    let root_slot = common_pointer_head_corruption::the_slot_of(&root_pointer);
    let mut level_of_the_root = 0;
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        root_slot,
        common_pointer_head_corruption::node_bytes(),
        |root| {
            level_of_the_root = root[INDEX_NODE_LEVEL_OFFSET];
            let start = 115
                + 2 * usize::from(root[INDEX_NODE_KEY_WIDTH_OFFSET])
                + entry_index * INTERNAL_ENTRY_BYTES;
            change(&mut root[start..start + INTERNAL_ENTRY_BYTES]);
        },
    );
    level_of_the_root
}

fn entry_type_of(entry: &[u8]) -> u16 {
    u16::from_le_bytes([
        entry[CONTAINER_TREE_ENTRY_TYPE_OFFSET],
        entry[CONTAINER_TREE_ENTRY_TYPE_OFFSET + 1],
    ])
}

/// 条目类型段从 `from` 改成 `to`（改之前先认它是 `from`）。
fn change_the_entry_type(entry: &mut [u8], from: u16, to: u16) {
    assert_eq!(entry_type_of(entry), from, "改之前条目类型段是 {from}");
    set_u16(entry, CONTAINER_TREE_ENTRY_TYPE_OFFSET, to);
}

/// 条目身份引用的出生树、容器号、出生代三段清零（类型段 0 的条目该有的样子），子指针不动。
fn zero_the_three_identity_fields(entry: &mut [u8]) {
    entry[INTERNAL_ENTRY_IDENTITY_BIRTH_TREE].fill(0);
    entry[INTERNAL_ENTRY_IDENTITY_CONTAINER_AND_BIRTH].fill(0);
}

/// 判红的说明里点名孩子头里的类与类型段说的那一类不符（`walk` 按孩子头里的类标签定读多长）。
fn assert_the_violation_names_the_child_class(
    image: &MemoryPool,
    invariant: &str,
    entry_type: u16,
    child_class_in_the_header: &str,
) {
    assert!(
        matches!(
            verdict(image, invariant),
            InvariantVerdict::Violated(detail)
                if detail.contains(&format!("类型段 {entry_type} 说孩子是"))
                    && detail.contains(&format!("孩子头里的类标签是 {child_class_in_the_header}"))
        ),
        "{invariant} 的违例说明写出类型段 {entry_type} 与孩子头里的类 {child_class_in_the_header}：{:?}",
        verdict(image, invariant)
    );
}

/// N4-a（判决 `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K2 N4 那一行）：三个单元的文件发完意图之后，墓碑树根（层级 1）
/// 第一条条目的类型段 1 → 0，孩子还是那片码 3 墓碑容器。I-9.23 字面「0 ⇒ 子单元是码 2」不成立；孩子读多长按它头里的类标签定，
/// 不按类型段：只红 I-9.23，不连带 I-4.8、I-7.2、I-7.4（改之前按 16 KiB 读码 3 孩子读不出，这一小句放过、又记走读失败）。
#[test]
fn a_tombstone_tree_entry_of_type_zero_over_a_container_reddens_only_the_tombstone_entry_invariant()
{
    let mut image = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-entry-type-zero-over-a-container",
    );
    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_TOMBSTONE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
    });
    assert_eq!(level, 1, "墓碑树根直接管容器");
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "类型段 0 的条目指着码 3 容器：只红 I-9.23，不连带 I-4.8、I-7.2、I-7.4：{verdicts:?}"
    );
    assert_the_violation_names_the_child_class(
        &image,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        "PackedRecordUnit",
    );
}

/// N4-i：同 N4-a，再把身份引用的出生树、容器号、出生代三段清零——这条条目除了孩子的类之外都是一条合法的类型段 0 条目。
/// 改之前孩子读不出、「子单元是码 2」那一小句放过，I-9.23 报成立（漏判）；现在只红 I-9.23。
#[test]
fn a_tombstone_tree_entry_of_type_zero_with_a_zero_identity_over_a_container_reddens_only_the_tombstone_entry_invariant(
) {
    let mut image = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-entry-type-zero-zero-identity-over-a-container",
    );
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_TOMBSTONE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        zero_the_three_identity_fields(entry);
    });
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "身份三段为 0 的类型段 0 条目指着码 3 容器：只红 I-9.23：{verdicts:?}"
    );
}

/// N4-c（谱系树那一侧，I-9.17 与 I-9.23 同形）：回退到 A 之后，谱系树根第一条条目的类型段 5 → 0，孩子还是那片码 3 谱系容器。只红 I-9.17。
#[test]
fn a_lineage_tree_entry_of_type_zero_over_a_container_reddens_only_the_lineage_entry_invariant() {
    let rolled_back =
        image_after_rolling_back_to_the_first_root("known-bad-lineage-entry-type-zero");
    let mut image = rolled_back.image.clone();
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_LINEAGE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_LINEAGE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
    });
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "谱系树类型段 0 的条目指着码 3 容器：只红 I-9.17：{verdicts:?}"
    );
    assert_the_violation_names_the_child_class(
        &image,
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        "PackedRecordUnit",
    );
}

/// I-9.17 类型段不认得那一支（M1 谱系那一侧，判决 `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K1 覆盖缺口那一行）：
/// 回退到 A 之后，谱系树根第一条条目的类型段 5 → 7。孩子先登记引用再判红：只红 I-9.17，不连带 I-3.1。
#[test]
fn a_lineage_tree_entry_with_an_unregistered_type_reddens_only_the_lineage_entry_invariant() {
    let rolled_back =
        image_after_rolling_back_to_the_first_root("known-bad-lineage-entry-unregistered-type");
    let mut image = rolled_back.image.clone();
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_LINEAGE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_LINEAGE,
            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
        );
    });
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "谱系树根条目的类型段不认得：只红 I-9.17，不连带 I-3.1：{verdicts:?}"
    );
    assert!(
        matches!(
            verdict(&image, LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER),
            InvariantVerdict::Violated(detail) if detail.contains("类型段 7 不在 {0, 5} 里")
        ),
        "违例说明写出不认得的类型段"
    );
}

/// I-9.2（inode 树条目身份与子头相符，与 I-9.17 / I-9.23 同形）：新池新建文件那一版，inode 树根（层级 1）第一条条目的类型段 2 → 0，
/// 孩子还是那片码 3 叶容器。I-9.2 字面「类型段 0 ⇒ 子单元偏移 6 = 2」不成立；孩子读多长按它头里的类标签定：只红 I-9.2，
/// 不连带 I-2.1（改之前按 16 KiB 读 32 KiB 的叶容器，校验和对不上，I-2.1 红、记走读失败，I-9.2 一次都没判到）。
#[test]
fn an_inode_tree_entry_of_type_zero_over_a_leaf_container_reddens_only_the_inode_entry_invariant() {
    let mut image = build_pool("known-bad-inode-entry-type-zero-over-a-leaf").memory_pool();
    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
    });
    assert_eq!(level, 1, "inode 树根直接管叶容器");
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-9.2"],
        "inode 树类型段 0 的条目指着码 3 叶容器：只红 I-9.2：{verdicts:?}"
    );
    assert_the_violation_names_the_child_class(
        &image,
        "I-9.2",
        CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        "PackedRecordUnit",
    );
}

/// 同上，再把身份引用三段清零：除了孩子的类，这是一条合法的类型段 0 条目。只红 I-9.2。
#[test]
fn an_inode_tree_entry_of_type_zero_with_a_zero_identity_over_a_leaf_container_reddens_only_the_inode_entry_invariant(
) {
    let mut image =
        build_pool("known-bad-inode-entry-type-zero-zero-identity-over-a-leaf").memory_pool();
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        zero_the_three_identity_fields(entry);
    });
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-9.2"],
        "身份三段为 0 的 inode 树类型段 0 条目指着码 3 叶容器：只红 I-9.2：{verdicts:?}"
    );
}

/// 写这几份长高的墓碑树时容器与节点压到的大小（照 `container_trees_grow_a_level_when_the_root_is_full.rs`）：一片 1 条、一个码 2 节点 2 条，
/// 第 3 片容器根满长一层。池级 checker 走带旋钮值的入口，旋钮值就是一片 1 条。
const TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;

fn capacities_of_one_record_and_two_entries(
) -> singlefs_core::container_tree::ContainerTreeCapacities {
    singlefs_core::container_tree::ContainerTreeCapacities {
        tombstone_container:
            singlefs_core::tombstone_tree::TombstoneContainerCapacity::CappedForTests(
                singlefs_core::tombstone_tree::TombstoneRecordsPerContainerCappedForTests::new(
                    TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
                )
                .expect("1..=815"),
            ),
        lineage_container: singlefs_core::lineage_tree::LineageContainerCapacity::CappedForTests(
            singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(1)
                .expect("1..=1359"),
        ),
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        node_entries: singlefs_core::container_tree::ContainerTreeNodeEntries::CappedForTests(
            singlefs_core::container_tree::ContainerTreeNodeEntriesCappedForTests::new(2)
                .expect("2..=135"),
        ),
    }
}

/// unlink 一个空 inode 到底（意图，再走空），写入口装一片 1 条、一个节点 2 条的旋钮。
fn unlink_an_empty_inode_with_one_record_per_tombstone_container(pool: &mut BuiltPool, inode: u64) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_container_tree_capacities(capacities_of_one_record_and_two_entries());
    let mut current = singlefs_core::transaction::publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        singlefs_core::address::InodeNumber(inode),
        common_deletion_in_progress::WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        common_deletion_in_progress::INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
    singlefs_core::transaction::publish_unlink_steps_until_the_queue_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
        common_deletion_in_progress::INSTANCE_OF_THE_PUBLISHES,
    )
    .expect("空 inode 一批就走空");
    pool.output = current;
}

/// 建 5 个空 inode（2..=6 号），在旋钮下 unlink `inodes` 里那几个：两个时墓碑树根层级 1、两片容器；三个时根满长到层级 2。
fn pool_with_empty_inodes_unlinked_under_the_tombstone_knob(
    tag: &str,
    inodes: &[u64],
) -> BuiltPool {
    let mut pool = common_deletion_in_progress::pool_with_new_inodes(tag, 5);
    for inode in inodes {
        unlink_an_empty_inode_with_one_record_per_tombstone_container(&mut pool, *inode);
    }
    pool
}

/// 池级 checker 带旋钮值（一片 1 条）的入口。
fn check_pool_image_under_the_tombstone_knob(
    image: &MemoryPool,
) -> Vec<(&'static str, InvariantVerdict)> {
    singlefs_checker::walk::check_pool_image_given_the_tombstone_container_capacity_knob(
        image,
        Some(
            singlefs_checker::walk::TombstoneContainerCapacityKnob::new(
                TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
            )
            .expect("旋钮值在 1..=815 里"),
        ),
    )
}

/// 带旋钮值的入口判出来的这一条的判定。
fn verdict_under_the_tombstone_knob(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image_under_the_tombstone_knob(image)
        .into_iter()
        .find(|(identifier, _)| *identifier == invariant)
        .expect("清单里有")
        .1
}

/// 墓碑树长到层级 2 的干净镜像（2、3、4 号在旋钮下删完）；带旋钮值的入口上一条都不红。
fn image_with_a_tombstone_tree_of_level_two(tag: &str) -> MemoryPool {
    let image =
        pool_with_empty_inodes_unlinked_under_the_tombstone_knob(tag, &[2, 3, 4]).memory_pool();
    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "长到层级 2 的干净镜像（带旋钮值）一条都不红：{verdicts:?}"
    );
    image
}

/// N4-f：墓碑树长到层级 2 之后，根第一条条目（类型段 0，孩子是层级 1 的码 2 节点）类型段 0 → 1。I-9.23 字面「1 ⇒ 子单元是码 3」不成立；
/// 孩子读多长按它头里的类标签定（改之前按 32 KiB 读 16 KiB 的节点，I-2.1 红、记走读失败，身份那一判跑不到、I-9.23 报成立）。
/// 孩子按码 2 一槽登记引用、不往下走：只红 I-9.23。
#[test]
fn a_tombstone_tree_entry_of_the_container_type_over_a_node_reddens_only_the_tombstone_entry_invariant(
) {
    let mut image = image_with_a_tombstone_tree_of_level_two(
        "known-bad-tombstone-entry-container-type-over-a-node",
    );
    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        change_the_entry_type(
            entry,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            PACKED_RECORD_TYPE_TOMBSTONE,
        );
    });
    assert_eq!(level, 2, "墓碑树根长到层级 2");
    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "类型段 1 的条目指着码 2 节点：只红 I-9.23，不连带 I-2.1、I-3.1、I-3.11、I-4.8、I-5.1、I-7.2、I-7.4：{verdicts:?}"
    );
    assert!(
        matches!(
            verdict_under_the_tombstone_knob(&image, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER),
            InvariantVerdict::Violated(detail)
                if detail.contains("类型段 1 说孩子是") && detail.contains("孩子头里的类标签是 IndexNode")
        ),
        "违例说明写出类型段 1 与孩子头里的码 2"
    );
}

/// N4-j：同 N4-f，墓碑树长到层级 2 之后再建 30 次（根环转过一圈），根之下那几个节点只剩现行这棵墓碑树引用、别的候选版不再替它们登记引用。只红 I-9.23。
#[test]
fn a_tombstone_tree_entry_of_the_container_type_over_a_node_after_the_root_ring_turned_over_reddens_only_the_tombstone_entry_invariant(
) {
    let mut pool = pool_with_empty_inodes_unlinked_under_the_tombstone_knob(
        "known-bad-tombstone-entry-container-type-over-a-node-ring-turned-over",
        &[2, 3, 4],
    );
    common_deletion_in_progress::turn_the_root_ring_over_by_creating_inodes(&mut pool);
    let clean = pool.memory_pool();
    assert_eq!(
        violated_invariants(&check_pool_image_under_the_tombstone_knob(&clean)),
        Vec::<&str>::new(),
        "根环转过一圈之后的干净镜像（带旋钮值）一条都不红"
    );
    let mut image = clean.clone();
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        change_the_entry_type(
            entry,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            PACKED_RECORD_TYPE_TOMBSTONE,
        );
    });
    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "根环转过一圈之后类型段 1 的条目指着码 2 节点：只红 I-9.23：{verdicts:?}"
    );
}

/// M1（判决第二节 K1 覆盖缺口那一行：类型段 0 那一支的身份判定没有用例钉）：墓碑树长到层级 2 之后，根第一条条目（类型段 0、孩子是码 2 节点）
/// 身份引用的容器号写成 1——类型段 0 要身份三段为 0。只红 I-9.23。
#[test]
fn a_tombstone_tree_entry_of_type_zero_with_a_container_number_reddens_only_the_tombstone_entry_invariant(
) {
    let mut image = image_with_a_tombstone_tree_of_level_two(
        "known-bad-tombstone-entry-type-zero-container-number",
    );
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        assert_eq!(
            entry_type_of(entry),
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            "根长高之后第一条条目类型段 0"
        );
        assert_eq!(
            get_u64(entry, CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET),
            0,
            "类型段 0 的条目容器号是 0"
        );
        set_u64(entry, CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET, 1);
    });
    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "类型段 0 的条目身份容器号不为 0：只红 I-9.23：{verdicts:?}"
    );
    assert!(
        matches!(
            verdict_under_the_tombstone_knob(&image, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER),
            InvariantVerdict::Violated(detail) if detail.contains("0 / 1 / 0（要三段为 0）")
        ),
        "违例说明写出身份三段"
    );
}

/// M5（判决第二节 K1 覆盖缺口那一行：I-9.21 / I-9.22「容器号沿叶序严格递增」没有用例钉）：旋钮下 2、3 号删完，墓碑树根层级 1、两片容器；
/// 第二片容器头里的容器号与根里指它那条条目的身份容器号一起改成第一片的号（身份照旧相符，分隔 key 不动）。带旋钮值的入口判 I-9.22：只红它。
#[test]
fn two_tombstone_containers_with_one_container_number_redden_only_the_capacity_knob_invariant() {
    let pool = pool_with_empty_inodes_unlinked_under_the_tombstone_knob(
        "known-bad-tombstone-containers-share-a-number",
        &[2, 3],
    );
    let mut image = pool.memory_pool();
    assert_eq!(
        verdict_under_the_tombstone_knob(
            &image,
            TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE
        ),
        InvariantVerdict::Holds,
        "两片容器的干净镜像上 I-9.22 真被评估过且成立"
    );
    let root_pointer =
        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_TOMBSTONE).root;
    let root =
        singlefs_core::unit::parse_index_node(&common_pointer_head_corruption::unit_bytes_at(
            &image,
            common_pointer_head_corruption::the_slot_of(&root_pointer),
            common_pointer_head_corruption::node_bytes(),
        ))
        .expect("墓碑树根解得开");
    assert_eq!(
        (root.level, root.entries.len()),
        (1, 2),
        "墓碑树根层级 1、两片容器"
    );
    let first_container_number = get_u64(
        &root.entries[0],
        CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET,
    );
    let second_container_pointer =
        singlefs_core::pointer::NodePointer::read_from(&mut singlefs_core::bytes::ByteReader::at(
            &root.entries[1],
            CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
        ));
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        &mut image,
        common_pointer_head_corruption::the_slot_of(&second_container_pointer),
        common_pointer_head_corruption::data_unit_bytes(),
        |container| {
            assert!(
                get_u64(container, PACKED_UNIT_CONTAINER_NUMBER_OFFSET) > first_container_number,
                "第二片的容器号大于第一片的"
            );
            set_u64(
                container,
                PACKED_UNIT_CONTAINER_NUMBER_OFFSET,
                first_container_number,
            );
        },
    );
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 1, |entry| {
        set_u64(
            entry,
            CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET,
            first_container_number,
        );
    });
    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE],
        "两片墓碑容器同号：只红 I-9.22：{verdicts:?}"
    );
}

/// 孩子头里的类标签只在「按那一类的长度读回、整单元校验和对得上」时才算孩子的类（`walk` 认孩子类的读法）：新池新建文件那一版，
/// inode 叶容器（32 KiB）头里的类标签 3 → 2，不重封，整单元校验和沿引用链补上（位置条目对得上这 32 KiB）。类标签说码 2，
/// 按 16 KiB 读回对不上校验和，不当成「另一类的孩子」：照类型段 2 那一支读 32 KiB，I-9.2 判类标签不是 3（违例说明写「（类 2）」），
/// 孩子照两槽登记引用，I-3.1 不连带红。
#[test]
fn an_inode_leaf_whose_class_tag_alone_says_index_node_is_read_at_the_length_of_its_entry_type() {
    let mut image = build_pool("known-bad-inode-leaf-class-tag-alone").memory_pool();
    mutate_unit(&mut image, INODE_LEAF, |bytes| bytes[6] = 2, false);
    assert!(
        matches!(
            verdict(&image, "I-9.2"),
            InvariantVerdict::Violated(detail) if detail.contains("（类 2）不符")
        ),
        "I-9.2 照类型段 2 那一支判出孩子头的类不是 3：{:?}",
        verdict(&image, "I-9.2")
    );
    assert!(
        !matches!(verdict(&image, "I-3.1"), InvariantVerdict::Violated(_)),
        "叶容器照两槽登记引用：I-3.1 不红：{:?}",
        verdict(&image, "I-3.1")
    );
}

/// 码 2 节点与码 3 打包记录单元头里的类标签（单元头偏移 6）。
const UNIT_CLASS_INDEX_NODE: u8 = 2;
const UNIT_CLASS_PACKED_RECORD_UNIT: u8 = 3;
/// 单元头里类标签的偏移。
const UNIT_HEADER_CLASS_TAG_OFFSET: usize = 6;

/// 最新的根那一版树表里种类 `kind` 那棵树的码 2 根第 `entry_index` 条条目的子指针。
fn child_pointer_of_an_entry_of_the_root_of_kind(
    image: &MemoryPool,
    kind: u16,
    entry_index: usize,
) -> singlefs_core::pointer::NodePointer {
    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
    let root =
        singlefs_core::unit::parse_index_node(&common_pointer_head_corruption::unit_bytes_at(
            image,
            common_pointer_head_corruption::the_slot_of(&root_pointer),
            common_pointer_head_corruption::node_bytes(),
        ))
        .expect("根解得开");
    singlefs_core::pointer::NodePointer::read_from(&mut singlefs_core::bytes::ByteReader::at(
        &root.entries[entry_index],
        CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
    ))
}

/// 最新的根那一版树表里种类 `kind` 那棵树的码 2 根（16 KiB 原样字节）：拿来当别处一个码 2 单元的陈旧副本。
fn root_node_bytes_of_the_tree_of_kind(image: &MemoryPool, kind: u16) -> Vec<u8> {
    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
    common_pointer_head_corruption::unit_bytes_at(
        image,
        common_pointer_head_corruption::the_slot_of(&root_pointer),
        common_pointer_head_corruption::node_bytes(),
    )
}

/// 一份副本陈旧：盘 `device` 上孩子的槽起写进 `stale_bytes`（别处一个单元的原样字节，不补任何校验和），另一块盘那一份不动。
/// 父条目位置条目里的整单元校验和还是孩子原来那一份的：陈旧那一份按哪一类的长度都自证不过。
fn make_the_copy_on_one_device_stale(
    image: &mut MemoryPool,
    device: u32,
    child: &singlefs_core::pointer::NodePointer,
    stale_bytes: &[u8],
) {
    let slot = common_pointer_head_corruption::the_slot_of(child);
    write(image, device, slot.0 * SLOT, stale_bytes);
}

/// 违例的那几条，按名字排成集合（比两份镜像红的是不是同一批，不管清单次序）。
fn violated_invariant_set(
    verdicts: &[(&'static str, InvariantVerdict)],
) -> std::collections::BTreeSet<&'static str> {
    violated_invariants(verdicts).into_iter().collect()
}

/// 攻方 H1（判决 `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K2 N1 两故障 H1 那一行）的判法：
/// 条目改坏（类型段与孩子的类不符）之外，孩子在一块盘上那一份陈旧、陈旧那一份头里的类恰好是类型段说的那一类，另一块盘那一份完好。
/// 两种盘序各判一次：红的恰好是「只有这份陈旧副本」那张镜像红的那几条再加 `entry_invariant`——陈旧副本自己连带的照红，
/// 条目那一条不随陈旧副本在哪块盘翻面，也不连带 I-7.2（改之前陈旧的在盘 0 时只认第一份的头、不验、不看第二份，
/// `entry_invariant` 报成立，另多红 I-7.2，层级 2 与 inode 那两形再多 I-3.1、I-3.11、I-5.1）。
fn assert_only_the_entry_invariant_adds_to_the_stale_copy_on_either_device(
    clean: &MemoryPool,
    image_with_the_entry_changed: &MemoryPool,
    child: &singlefs_core::pointer::NodePointer,
    stale_bytes: &[u8],
    entry_invariant: &'static str,
    check: impl Fn(&MemoryPool) -> Vec<(&'static str, InvariantVerdict)>,
) {
    let mut violated_on_each_device = Vec::new();
    for device in DEVICES {
        let mut stale_copy_only = clean.clone();
        make_the_copy_on_one_device_stale(&mut stale_copy_only, device, child, stale_bytes);
        let violated_by_the_stale_copy_only = violated_invariant_set(&check(&stale_copy_only));
        assert!(
            !violated_by_the_stale_copy_only.contains(entry_invariant),
            "只有盘 {device} 上那份陈旧副本、条目不改：{entry_invariant} 不红（{violated_by_the_stale_copy_only:?}）"
        );
        let mut image = image_with_the_entry_changed.clone();
        make_the_copy_on_one_device_stale(&mut image, device, child, stale_bytes);
        let verdicts = check(&image);
        let violated = violated_invariant_set(&verdicts);
        let mut expected = violated_by_the_stale_copy_only.clone();
        expected.insert(entry_invariant);
        assert_eq!(
            violated, expected,
            "条目改坏、孩子在盘 {device} 上那份陈旧：红的是只有陈旧副本时那几条再加 {entry_invariant}：{verdicts:?}"
        );
        assert!(
            !violated.contains("I-7.2"),
            "条目改坏、孩子在盘 {device} 上那份陈旧：不连带 I-7.2：{verdicts:?}"
        );
        violated_on_each_device.push(violated);
    }
    assert_eq!(
        violated_on_each_device[0], violated_on_each_device[1],
        "同一张镜像只差陈旧副本在哪块盘：红的同一批"
    );
}

/// H1 的 N4-i 那一形（攻方 `research/prompts/m3-batch24-code-r1-opus-model/expected-summary.txt` 第 391 / 392 行那一对）：三个单元的文件发完意图之后，
/// 墓碑树根（层级 1）第一条条目类型段 1 → 0、身份三段清零，孩子还是那片码 3 墓碑容器；孩子在一块盘上那一份换成墓碑树根（码 2）的原样字节。
#[test]
fn a_tombstone_tree_entry_of_type_zero_over_a_container_with_a_stale_copy_of_the_child_reddens_the_tombstone_entry_invariant_on_either_device(
) {
    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-entry-type-zero-over-a-container-stale-copy",
    );
    let child = child_pointer_of_an_entry_of_the_root_of_kind(&clean, TREE_KIND_TOMBSTONE, 0);
    let stale_bytes = root_node_bytes_of_the_tree_of_kind(&clean, TREE_KIND_TOMBSTONE);
    assert_eq!(
        stale_bytes[UNIT_HEADER_CLASS_TAG_OFFSET], UNIT_CLASS_INDEX_NODE,
        "陈旧副本头里的类就是类型段 0 说的码 2"
    );
    let mut image_with_the_entry_changed = clean.clone();
    change_an_entry_of_the_root_of_kind(
        &mut image_with_the_entry_changed,
        TREE_KIND_TOMBSTONE,
        0,
        |entry| {
            change_the_entry_type(
                entry,
                PACKED_RECORD_TYPE_TOMBSTONE,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            );
            zero_the_three_identity_fields(entry);
        },
    );
    assert_only_the_entry_invariant_adds_to_the_stale_copy_on_either_device(
        &clean,
        &image_with_the_entry_changed,
        &child,
        &stale_bytes,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        |image| check_pool_image(image),
    );
}

/// H1 的 N4-f 那一形（攻方 T2）：墓碑树长到层级 2 之后，根第一条条目类型段 0 → 1，孩子还是层级 1 的码 2 节点；
/// 孩子在一块盘上那一份换成一片墓碑容器的前 16 KiB（头里类标签 3）。带旋钮值的入口判。
#[test]
fn a_tombstone_tree_entry_of_the_container_type_over_a_node_with_a_stale_copy_of_the_child_reddens_the_tombstone_entry_invariant_on_either_device(
) {
    let clean = image_with_a_tombstone_tree_of_level_two(
        "known-bad-tombstone-entry-container-type-over-a-node-stale-copy",
    );
    let child = child_pointer_of_an_entry_of_the_root_of_kind(&clean, TREE_KIND_TOMBSTONE, 0);
    let first_entry_of_the_child =
        common_pointer_head_corruption::first_entry_of_the_node_at(&clean, &child);
    let container_below_the_child =
        singlefs_core::pointer::NodePointer::read_from(&mut singlefs_core::bytes::ByteReader::at(
            &first_entry_of_the_child,
            CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
        ));
    let stale_bytes = common_pointer_head_corruption::unit_bytes_at(
        &clean,
        common_pointer_head_corruption::the_slot_of(&container_below_the_child),
        common_pointer_head_corruption::node_bytes(),
    );
    assert_eq!(
        stale_bytes[UNIT_HEADER_CLASS_TAG_OFFSET], UNIT_CLASS_PACKED_RECORD_UNIT,
        "陈旧副本头里的类就是类型段 1 说的码 3"
    );
    let mut image_with_the_entry_changed = clean.clone();
    change_an_entry_of_the_root_of_kind(
        &mut image_with_the_entry_changed,
        TREE_KIND_TOMBSTONE,
        0,
        |entry| {
            change_the_entry_type(
                entry,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
                PACKED_RECORD_TYPE_TOMBSTONE,
            );
        },
    );
    assert_only_the_entry_invariant_adds_to_the_stale_copy_on_either_device(
        &clean,
        &image_with_the_entry_changed,
        &child,
        &stale_bytes,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        check_pool_image_under_the_tombstone_knob,
    );
}

/// H1 谱系树那一侧（I-9.17 与 I-9.23 同形）：回退到 A 之后，谱系树根第一条条目类型段 5 → 0、身份三段清零，孩子还是那片码 3 谱系容器；
/// 孩子在一块盘上那一份换成谱系树根（码 2）的原样字节。
#[test]
fn a_lineage_tree_entry_of_type_zero_over_a_container_with_a_stale_copy_of_the_child_reddens_the_lineage_entry_invariant_on_either_device(
) {
    let clean = image_after_rolling_back_to_the_first_root(
        "known-bad-lineage-entry-type-zero-over-a-container-stale-copy",
    )
    .image;
    let child = child_pointer_of_an_entry_of_the_root_of_kind(&clean, TREE_KIND_LINEAGE, 0);
    let stale_bytes = root_node_bytes_of_the_tree_of_kind(&clean, TREE_KIND_LINEAGE);
    assert_eq!(
        stale_bytes[UNIT_HEADER_CLASS_TAG_OFFSET], UNIT_CLASS_INDEX_NODE,
        "陈旧副本头里的类就是类型段 0 说的码 2"
    );
    let mut image_with_the_entry_changed = clean.clone();
    change_an_entry_of_the_root_of_kind(
        &mut image_with_the_entry_changed,
        TREE_KIND_LINEAGE,
        0,
        |entry| {
            change_the_entry_type(
                entry,
                PACKED_RECORD_TYPE_LINEAGE,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            );
            zero_the_three_identity_fields(entry);
        },
    );
    assert_only_the_entry_invariant_adds_to_the_stale_copy_on_either_device(
        &clean,
        &image_with_the_entry_changed,
        &child,
        &stale_bytes,
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        |image| check_pool_image(image),
    );
}

/// H1 的 inode 那一形（攻方 I0 type0-zero-identity）：新池新建文件那一版，inode 树根（层级 1）第一条条目类型段 2 → 0、身份三段清零，
/// 孩子还是那片码 3 叶容器；孩子在一块盘上那一份换成 inode 树根（码 2）的原样字节。
#[test]
fn an_inode_tree_entry_of_type_zero_over_a_leaf_container_with_a_stale_copy_of_the_child_reddens_the_inode_entry_invariant_on_either_device(
) {
    let clean = build_pool("known-bad-inode-entry-type-zero-over-a-leaf-stale-copy").memory_pool();
    let child = child_pointer_of_an_entry_of_the_root_of_kind(&clean, TREE_KIND_INODE, 0);
    let stale_bytes = root_node_bytes_of_the_tree_of_kind(&clean, TREE_KIND_INODE);
    assert_eq!(
        stale_bytes[UNIT_HEADER_CLASS_TAG_OFFSET], UNIT_CLASS_INDEX_NODE,
        "陈旧副本头里的类就是类型段 0 说的码 2"
    );
    let mut image_with_the_entry_changed = clean.clone();
    change_an_entry_of_the_root_of_kind(
        &mut image_with_the_entry_changed,
        TREE_KIND_INODE,
        0,
        |entry| {
            change_the_entry_type(
                entry,
                PACKED_RECORD_TYPE_INODE,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            );
            zero_the_three_identity_fields(entry);
        },
    );
    assert_only_the_entry_invariant_adds_to_the_stale_copy_on_either_device(
        &clean,
        &image_with_the_entry_changed,
        &child,
        &stale_bytes,
        "I-9.2",
        |image| check_pool_image(image),
    );
}

/// 不在登记表里的类标签：孩子头里写成它之后，按哪一类的长度都自证不过，两份都读不出。
const UNIT_CLASS_NOT_REGISTERED: u8 = 9;

/// I-1.11：映射条目 key 里的类标签与它指的单元头里的类标签相等（叶容器头里的类标签改成 [`UNIT_CLASS_NOT_REGISTERED`] 自己就带它红）。
const MAPPING_KEY_CLASS_TAG_MATCHES_THE_UNIT_HEADER: &str = "I-1.11";

/// 新池新建文件那一版（`image`）：inode 叶容器头里的类标签 3 → 9（不重封，整单元校验和沿引用链补上），再照 `change` 改 inode 树根第一条条目、重封到根。
/// 孩子按自己的类标签认不出、按类型段的长度读不出：类型段认得的，读不出归 I-2.1 与走读失败。
fn image_whose_inode_root_entry_names_a_child_that_does_not_read_back(
    mut image: MemoryPool,
    change: impl FnOnce(&mut [u8]),
) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(&image, TREE_KIND_INODE, 0);
    assert_eq!(
        common_pointer_head_corruption::the_slot_of(&child).0,
        INODE_LEAF,
        "inode 树根第一条条目指着那片叶容器"
    );
    mutate_unit(
        &mut image,
        INODE_LEAF,
        |bytes| bytes[UNIT_HEADER_CLASS_TAG_OFFSET] = UNIT_CLASS_NOT_REGISTERED,
        false,
    );
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, change);
    image
}

/// inode 树读不出孩子时 I-9.2 的判法与墓碑树、谱系树对齐（判决 `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K1 N1 那一行，
/// 正推 144 格矩阵里 inode 树不判的那 9 格）：inode 树根第一条条目类型段 2 → 0、身份三段不动（容器号、出生代不为 0），孩子读不出。
/// 「子单元是码 2」那一小句孩子读不出不判，身份三段不为 0 照判：I-9.2 红（改之前读不出就记走读失败、跳过这条条目，I-9.2 一判都没有）。
#[test]
fn an_inode_tree_entry_of_type_zero_with_an_identity_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
) {
    let image = image_whose_inode_root_entry_names_a_child_that_does_not_read_back(
        build_pool("known-bad-inode-entry-type-zero-identity-unreadable-child").memory_pool(),
        |entry| {
            change_the_entry_type(
                entry,
                PACKED_RECORD_TYPE_INODE,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            );
        },
    );
    assert!(
        matches!(
            verdict(&image, "I-9.2"),
            InvariantVerdict::Violated(detail)
                if detail.contains("孩子两份都读不出") && detail.contains("（要三段为 0）")
        ),
        "类型段 0 的条目身份三段不为 0、孩子读不出：I-9.2 照判身份红：{:?}",
        verdict(&image, "I-9.2")
    );
}

/// 同上一条，条目的类型段 2 → 7（不在 {0, 2} 里）、孩子读不出：判该条目损坏，与墓碑树、谱系树类型段不认得那一支同判
/// （`records/2026-10-01-删除与并发事务.md` 第 27 批，第 26 批 H 报告「交主 agent」第 1 条）——不按 16 KiB 去读孩子，孩子按层级 1 下面的叶容器两槽登记引用。
/// 对照：同一张新池镜像只把这条条目的类型段改成 7、孩子完好（读得出、自证出码 3，按它自己那一类两槽登记引用、判 I-9.2 红、不往下走）。
/// 孩子读不出的这一张红的不变量 = 对照所红（I-9.2）∪ 叶容器类标签改成 9 那一改自己带的 I-1.11（中央映射条目 key 里的类标签 3 与单元头不符，
/// 与 inode 树怎么走无关）：不连带 I-2.1（改之前按 16 KiB 读孩子、校验和对不上）、不记走读失败（改之前连带 I-4.8、I-7.2、I-7.4），
/// 也不连带 I-3.1（孩子占的两槽都进引用并集）。
/// 「不连带 I-2.1」只对这一张镜像成立（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行）：第 29 批起这一支
/// 每一份副本照判 I-2.1，头里的类标签 9 不登记、16 KiB 与 32 KiB 两种长度都比，这张镜像的叶容器类标签改坏、整单元校验和沿引用链补齐了，
/// 按 32 KiB 对得上；位置条目里的校验和本身坏了的那一形见
/// [`an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants`]。
#[test]
fn an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read_back_reddens_only_the_inode_entry_invariant(
) {
    let clean =
        build_pool("known-bad-inode-entry-unregistered-type-unreadable-child").memory_pool();
    let change_to_an_unregistered_type = |entry: &mut [u8]| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
        );
    };
    let mut image_with_a_readable_child = clean.clone();
    change_an_entry_of_the_root_of_kind(
        &mut image_with_a_readable_child,
        TREE_KIND_INODE,
        0,
        change_to_an_unregistered_type,
    );
    let verdicts_with_a_readable_child = check_pool_image(&image_with_a_readable_child);
    assert_eq!(
        violated_invariants(&verdicts_with_a_readable_child),
        ["I-9.2"],
        "对照：类型段 7 的条目指着完好的叶容器，只红 I-9.2：{verdicts_with_a_readable_child:?}"
    );
    let image = image_whose_inode_root_entry_names_a_child_that_does_not_read_back(
        clean,
        change_to_an_unregistered_type,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [
            MAPPING_KEY_CLASS_TAG_MATCHES_THE_UNIT_HEADER,
            violated_invariants(&verdicts_with_a_readable_child)[0]
        ],
        "类型段不认得、孩子读不出：红的 = 孩子完好的对照所红（I-9.2）∪ 类标签那一改自己带的 I-1.11，不连带 I-2.1、走读失败那几条与 I-3.1：{verdicts:?}"
    );
    assert!(
        matches!(
            verdict(&image, MAPPING_KEY_CLASS_TAG_MATCHES_THE_UNIT_HEADER),
            InvariantVerdict::Violated(detail)
                if detail.contains("它指的单元头里的类标签是 9")
        ),
        "I-1.11 红在映射条目 key 里的类标签与改成 9 的叶容器头不符（类标签那一改自己带的，不是走 inode 树带出来的）：{:?}",
        verdict(&image, MAPPING_KEY_CLASS_TAG_MATCHES_THE_UNIT_HEADER)
    );
    assert!(
        matches!(
            verdict(&image, "I-9.2"),
            InvariantVerdict::Violated(detail)
                if detail.contains("类型段 7 不在 {0, 2} 里（孩子没有一份按它头里那一类的长度自证过，不往下走）")
        ),
        "违例说明写出不认得的类型段、孩子不往下走：{:?}",
        verdict(&image, "I-9.2")
    );
}

/// 内部条目（120 字节）子指针里第 `location_index` 条位置条目的整单元校验和第 0 字节翻一下（位置条目 = 盘 4 + 槽 6 + 整单元校验和 4，
/// 子指针从条目偏移 [`CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET`] 起）：孩子本身不动，那一份按哪一种长度都与位置条目里的校验和对不上。
fn flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(
    entry: &mut [u8],
    location_index: usize,
) {
    let checksum_offset = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        + POINTER_FIRST_LOCATION_OFFSET
        + location_index * POINTER_LOCATION_BYTES
        + LOCATION_CHECKSUM_OFFSET;
    entry[checksum_offset] ^= 0xff;
}

/// 走读不往下读的孩子每一份副本照判 I-2.1（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a、H-J3b 两行与第三节第 1 条，
/// `.claude/kb/invariants.md` I-2.1「任一被引用的块，其校验和与内容匹配」）：`image` 红的恰好是 `expected_violations`，
/// I-2.1 的违例说明点名「走读不往下读的孩子」（红在新判的那一处，不是别的单元）；`expected_violations` 里有 I-7.2 的，
/// 它的违例说明点名两份都对不上（走读失败是这一处记的）。
fn assert_the_child_the_walk_does_not_go_into_reddens_exactly(
    image: &MemoryPool,
    expected_violations: &[&'static str],
    scene: &str,
) {
    let verdicts = check_pool_image(image);
    let verdict_of = |invariant: &str| {
        verdicts
            .iter()
            .find(|(identifier, _)| *identifier == invariant)
            .expect("清单里有")
            .1
            .clone()
    };
    assert_eq!(
        violated_invariant_set(&verdicts),
        expected_violations
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<&'static str>>(),
        "{scene}：{verdicts:?}"
    );
    assert!(
        matches!(
            verdict_of("I-2.1"),
            InvariantVerdict::Violated(detail) if detail.contains("走读不往下读的孩子")
        ),
        "{scene}：I-2.1 红在走读不往下读的那个孩子上：{:?}",
        verdict_of("I-2.1")
    );
    if expected_violations.contains(&"I-7.2") {
        assert!(
            matches!(
                verdict_of("I-7.2"),
                InvariantVerdict::Violated(detail)
                    if detail.contains("（走读不往下读的孩子）两份都与位置条目里的校验和对不上")
            ),
            "{scene}：I-7.2 红在走读不往下读的孩子两份都对不上：{:?}",
            verdict_of("I-7.2")
        );
    }
}

/// 攻方 B1 那一形（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行，`research/prompts/m3-batch27-code-r1-opus-output.md`
/// 第一节）：新池新建文件那一版，inode 树根（层级 1）第一条条目的类型段 2 → 7、子指针两条位置条目的整单元校验和各翻一字节，重封到根；
/// 叶容器不动。孩子不往下走，两份副本照判 I-2.1、都对不上、记走读失败：红 I-9.2、I-2.1、I-4.8、I-7.2、I-7.4
/// （第 27 批这一支两样都不记，只红 I-9.2）。
#[test]
fn an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
) {
    let mut image =
        build_pool("known-bad-inode-entry-unregistered-type-both-location-checksums-wrong")
            .memory_pool();
    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
        );
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 1);
    });
    assert_eq!(level, 1, "inode 树根直接管叶容器");
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &["I-9.2", "I-2.1", "I-4.8", "I-7.2", "I-7.4"],
        "inode 树根条目类型段 7、子指针两条位置条目的校验和都坏",
    );
}

/// 同上，墓碑树那一侧（I-9.23；第 27 批对齐的目标那一支，同一缺口在那里早就有）：三个单元的文件发完意图之后，墓碑树根（层级 1）
/// 第一条条目的类型段 1 → 7、子指针两条位置条目的整单元校验和各翻一字节，重封到根；墓碑容器不动。
#[test]
fn a_tombstone_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
) {
    let mut image = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-entry-unregistered-type-both-location-checksums-wrong",
    );
    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_TOMBSTONE,
            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
        );
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 1);
    });
    assert_eq!(level, 1, "墓碑树根直接管容器");
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &[
            TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
            "I-2.1",
            "I-4.8",
            "I-7.2",
            "I-7.4",
        ],
        "墓碑树根条目类型段 7、子指针两条位置条目的校验和都坏",
    );
}

/// 同上，谱系树那一侧（I-9.17）：回退到 A 之后，谱系树根（层级 1）第一条条目的类型段 5 → 7、子指针两条位置条目的整单元校验和各翻一字节，
/// 重封到根；谱系容器不动。
#[test]
fn a_lineage_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
) {
    let mut image = image_after_rolling_back_to_the_first_root(
        "known-bad-lineage-entry-unregistered-type-both-location-checksums-wrong",
    )
    .image;
    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_LINEAGE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_LINEAGE,
            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
        );
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 1);
    });
    assert_eq!(level, 1, "谱系树根直接管容器");
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &[
            LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
            "I-2.1",
            "I-4.8",
            "I-7.2",
            "I-7.4",
        ],
        "谱系树根条目类型段 7、子指针两条位置条目的校验和都坏",
    );
}

/// H-J3b（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3b 那一行，第 27 批之前就有）：新池新建文件那一版，
/// inode 树根（层级 1）第一条条目的类型段 2 → 0、子指针第一条位置条目的整单元校验和翻一字节，重封到根；叶容器不动。
/// 第二份自证成码 3（类型段 0 说的是码 2），走「孩子是另一类」那一支：I-9.2 红、不往下走；第一份按哪一种长度都对不上，照判 I-2.1 红，
/// I-4.8、I-7.4 跟着；第二份对得上，不记走读失败、I-7.2 不红（改之前这一支不判 I-2.1，映射树那一遍按映射条目的校验和读、也判不到）。
#[test]
fn an_inode_tree_entry_of_type_zero_over_a_leaf_container_whose_first_location_checksum_is_wrong_reddens_the_checksum_invariant(
) {
    let mut image =
        build_pool("known-bad-inode-entry-type-zero-over-a-leaf-first-location-checksum-wrong")
            .memory_pool();
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
    });
    assert_the_violation_names_the_child_class(
        &image,
        "I-9.2",
        CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        "PackedRecordUnit",
    );
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &["I-9.2", "I-2.1", "I-4.8", "I-7.4"],
        "inode 树根条目类型段 0 指着叶容器、子指针第一条位置条目的校验和坏",
    );
}

/// H-J3b 墓碑树那一侧（`Walk::judge_a_container_tree_entry_whose_child_is_of_another_class`，墓碑树与谱系树共用）：三个单元的文件发完意图之后，
/// 墓碑树根（层级 1）第一条条目的类型段 1 → 0、子指针第一条位置条目的整单元校验和翻一字节，重封到根；墓碑容器不动。
#[test]
fn a_tombstone_tree_entry_of_type_zero_over_a_container_whose_first_location_checksum_is_wrong_reddens_the_checksum_invariant(
) {
    let mut image = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-entry-type-zero-over-a-container-first-location-checksum-wrong",
    );
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_TOMBSTONE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
    });
    assert_the_violation_names_the_child_class(
        &image,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        "PackedRecordUnit",
    );
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &[
            TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
            "I-2.1",
            "I-4.8",
            "I-7.4",
        ],
        "墓碑树根条目类型段 0 指着容器、子指针第一条位置条目的校验和坏",
    );
}

/// 第 28 批正推线索②（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 3 条「读不出孩子那两条坏镜像补『只改身份一段』三形」）：
/// inode 树根第一条条目类型段 2 → 0、身份引用三段先清零再只把 `field` 那一段写成 1，孩子读不出（同
/// [`an_inode_tree_entry_of_type_zero_with_an_identity_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant`]）。
/// 三段里只有一段不为 0，I-9.2 照判身份红，违例说明写出的三段就是 `expected_three_fields`（「出生树 / 容器号 / 出生代」）。
fn assert_one_non_zero_identity_field_of_a_type_zero_inode_entry_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
    tag: &str,
    field: std::ops::Range<usize>,
    expected_three_fields: &str,
) {
    let image = image_whose_inode_root_entry_names_a_child_that_does_not_read_back(
        build_pool(tag).memory_pool(),
        |entry| {
            change_the_entry_type(
                entry,
                PACKED_RECORD_TYPE_INODE,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            );
            zero_the_three_identity_fields(entry);
            entry[field].copy_from_slice(&1u64.to_le_bytes());
        },
    );
    assert!(
        matches!(
            verdict(&image, "I-9.2"),
            InvariantVerdict::Violated(detail)
                if detail.contains("孩子两份都读不出")
                    && detail.contains(&format!("{expected_three_fields}（要三段为 0）"))
        ),
        "类型段 0 的条目身份只有一段不为 0（{expected_three_fields}）、孩子读不出：I-9.2 照判身份红：{:?}",
        verdict(&image, "I-9.2")
    );
}

/// 条目身份引用里容器出生代那 8 字节（容器号 18..26 之后）。
const INTERNAL_ENTRY_IDENTITY_CONTAINER_BIRTH: std::ops::Range<usize> = 26..34;

#[test]
fn an_inode_tree_entry_of_type_zero_whose_identity_carries_only_a_birth_tree_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
) {
    assert_one_non_zero_identity_field_of_a_type_zero_inode_entry_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
        "known-bad-inode-entry-type-zero-only-birth-tree-unreadable-child",
        INTERNAL_ENTRY_IDENTITY_BIRTH_TREE,
        "1 / 0 / 0",
    );
}

#[test]
fn an_inode_tree_entry_of_type_zero_whose_identity_carries_only_a_container_number_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
) {
    assert_one_non_zero_identity_field_of_a_type_zero_inode_entry_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
        "known-bad-inode-entry-type-zero-only-container-number-unreadable-child",
        CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET..INTERNAL_ENTRY_IDENTITY_CONTAINER_BIRTH.start,
        "0 / 1 / 0",
    );
}

#[test]
fn an_inode_tree_entry_of_type_zero_whose_identity_carries_only_a_container_birth_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
) {
    assert_one_non_zero_identity_field_of_a_type_zero_inode_entry_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
        "known-bad-inode-entry-type-zero-only-container-birth-unreadable-child",
        INTERNAL_ENTRY_IDENTITY_CONTAINER_BIRTH,
        "0 / 0 / 1",
    );
}

/// 条目里子指针头部出生树那 8 字节：子指针在条目偏移 34 起，出生树在指针头部偏移 34（D19（块指针的结构与宽度预算） 已定项 11 的偏移表）。
const INTERNAL_ENTRY_CHILD_POINTER_BIRTH_TREE: std::ops::Range<usize> =
    CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET + 34..CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET + 42;

/// 条目里 `field` 那 8 字节（出生树那两段之一）加 1。
fn add_one_to_a_birth_tree_field_of_the_entry(entry: &mut [u8], field: std::ops::Range<usize>) {
    let birth_tree = get_u64(entry, field.start);
    entry[field].copy_from_slice(&(birth_tree + 1).to_le_bytes());
}

/// 种类 `kind` 那棵树的根（层级 1）第一条条目指着的码 3 容器两份各翻一个载荷字节（不重封、不补引用链：两份都读不出），条目不改。
fn image_whose_first_container_of_the_root_of_kind_does_not_read_back(
    clean: &MemoryPool,
    kind: u16,
) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, kind, 0);
    let mut image = clean.clone();
    break_both_copies_of_the_unit(
        &mut image,
        common_pointer_head_corruption::the_slot_of(&child).0,
        common_pointer_head_corruption::data_unit_bytes(),
    );
    image
}

/// H-B（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第二节 K2 H3 H-B 那一行、第三节第 1 条；攻方
/// `research/prompts/m3-batch26-code-r1-opus-output.md` 第 2.2 节 U 段 BT / IDB 两种改法）的判法：种类 `kind` 那棵树的根（层级 1）第一条条目
/// 类型段是容器那一类（`container_entry_type`），它指的容器两份都读不出。对照（条目不改）：`entry_invariant` 不红——条目其余都对，
/// 「子指针头部的出生树 == 身份引用的出生树」成立，比孩子头的那几小句读不出不判（正推线索③：不过判）。
/// 再把条目里 `field`（子指针头部出生树或身份引用出生树）加 1：那一小句只比父条目里的两段、不看孩子，`entry_invariant` 红，
/// 违例说明写出两段出生树（改之前读不出孩子时整支不判，`entry_invariant` 报成立）。
fn assert_a_birth_tree_field_one_off_reddens_the_entry_invariant_over_a_container_that_does_not_read_back(
    clean: &MemoryPool,
    kind: u16,
    container_entry_type: u16,
    field: std::ops::Range<usize>,
    entry_invariant: &'static str,
) {
    let unreadable_child_only =
        image_whose_first_container_of_the_root_of_kind_does_not_read_back(clean, kind);
    assert!(
        !matches!(
            verdict(&unreadable_child_only, entry_invariant),
            InvariantVerdict::Violated(_)
        ),
        "对照：条目不改、容器两份都读不出，{entry_invariant} 不红：{:?}",
        verdict(&unreadable_child_only, entry_invariant)
    );
    let mut image = unreadable_child_only;
    let mut birth_trees_after_the_change = (0, 0);
    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
        assert_eq!(
            entry_type_of(entry),
            container_entry_type,
            "根第一条条目指容器"
        );
        add_one_to_a_birth_tree_field_of_the_entry(entry, field);
        birth_trees_after_the_change = (
            get_u64(entry, INTERNAL_ENTRY_CHILD_POINTER_BIRTH_TREE.start),
            get_u64(entry, INTERNAL_ENTRY_IDENTITY_BIRTH_TREE.start),
        );
    });
    assert_eq!(level, 1, "根层级 1、直接管容器");
    let (pointer_birth_tree, identity_birth_tree) = birth_trees_after_the_change;
    assert!(
        matches!(
            verdict(&image, entry_invariant),
            InvariantVerdict::Violated(detail)
                if detail.contains("两份都读不出")
                    && detail.contains(&format!(
                        "子指针头部的出生树 {pointer_birth_tree} 不等于身份引用的出生树 {identity_birth_tree}"
                    ))
        ),
        "出生树那两段不等、容器两份都读不出：{entry_invariant} 照判那一小句、红：{:?}",
        verdict(&image, entry_invariant)
    );
}

#[test]
fn an_inode_tree_entry_whose_identity_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back_reddens_the_inode_entry_invariant(
) {
    assert_a_birth_tree_field_one_off_reddens_the_entry_invariant_over_a_container_that_does_not_read_back(
        &build_pool("known-bad-inode-entry-identity-birth-tree-unreadable-leaf").memory_pool(),
        TREE_KIND_INODE,
        PACKED_RECORD_TYPE_INODE,
        INTERNAL_ENTRY_IDENTITY_BIRTH_TREE,
        "I-9.2",
    );
}

#[test]
fn a_tombstone_tree_entry_whose_child_pointer_birth_tree_is_one_off_over_a_container_that_does_not_read_back_reddens_the_tombstone_entry_invariant(
) {
    assert_a_birth_tree_field_one_off_reddens_the_entry_invariant_over_a_container_that_does_not_read_back(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-entry-pointer-birth-tree-unreadable-container",
        ),
        TREE_KIND_TOMBSTONE,
        PACKED_RECORD_TYPE_TOMBSTONE,
        INTERNAL_ENTRY_CHILD_POINTER_BIRTH_TREE,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    );
}

#[test]
fn a_lineage_tree_entry_whose_identity_birth_tree_is_one_off_over_a_container_that_does_not_read_back_reddens_the_lineage_entry_invariant(
) {
    assert_a_birth_tree_field_one_off_reddens_the_entry_invariant_over_a_container_that_does_not_read_back(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-entry-identity-birth-tree-unreadable-container",
        )
        .image,
        TREE_KIND_LINEAGE,
        PACKED_RECORD_TYPE_LINEAGE,
        INTERNAL_ENTRY_IDENTITY_BIRTH_TREE,
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    );
}

/// 正推线索③（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 3 条「补一条类型段 2 读不出孩子、I-9.2 不判的坏镜像（钉住不过判）」）：
/// 新池新建文件那一版，inode 树根第一条条目不改（类型段 2、两段出生树相等），它指的叶容器两份都读不出。
/// 比孩子头的那几小句读不出不判，「子指针头部的出生树 == 身份引用的出生树」成立：I-9.2 不红（报成立或不判都行）。
#[test]
fn an_inode_tree_entry_of_the_leaf_container_type_with_agreeing_birth_trees_over_a_leaf_that_does_not_read_back_leaves_the_inode_entry_invariant_unviolated(
) {
    let image = image_whose_first_container_of_the_root_of_kind_does_not_read_back(
        &build_pool("known-bad-inode-entry-agreeing-birth-trees-unreadable-leaf").memory_pool(),
        TREE_KIND_INODE,
    );
    assert!(
        matches!(verdict(&image, "I-2.1"), InvariantVerdict::Violated(_)),
        "叶容器两份都读不出：I-2.1 红（钉住这张镜像真把孩子弄成读不出）：{:?}",
        verdict(&image, "I-2.1")
    );
    assert!(
        !matches!(verdict(&image, "I-9.2"), InvariantVerdict::Violated(_)),
        "类型段 2、两段出生树相等、孩子读不出：I-9.2 不过判：{:?}",
        verdict(&image, "I-9.2")
    );
}

/// H-V（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第二节 K2 H1 H-V 那一行、第三节第 2 条；攻方
/// `research/prompts/m3-batch26-code-r1-opus-output.md` 第 1.3 节）的造法：种类 `kind` 那棵树的根（层级 1）第一条条目先照 `change_the_entry` 改，
/// 再把它子指针第 `location_index` 条位置条目的整单元校验和换成这棵树根（码 2）原样 16 KiB 的 CRC、重封到根，那块盘上孩子的槽起写进这 16 KiB；
/// 另一块盘那一份不动。两份按各自的位置条目都自证得过：一份码 2，一份是原来那片码 3 容器。
fn image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
    clean: &MemoryPool,
    kind: u16,
    location_index: usize,
    change_the_entry: &dyn Fn(&mut [u8]),
) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, kind, 0);
    let root_bytes = root_node_bytes_of_the_tree_of_kind(clean, kind);
    assert_eq!(
        root_bytes[UNIT_HEADER_CLASS_TAG_OFFSET], UNIT_CLASS_INDEX_NODE,
        "写进孩子槽的那 16 KiB 是码 2 节点"
    );
    let checksum_of_the_root_bytes = crc32_castagnoli(&root_bytes);
    let mut image = clean.clone();
    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
        change_the_entry(entry);
        let pointer_field = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
            ..CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET + CONTAINER_TREE_ENTRY_CHILD_POINTER_BYTES;
        let mut pointer = singlefs_core::pointer::NodePointer::read_from(
            &mut singlefs_core::bytes::ByteReader::at(entry, pointer_field.start),
        );
        pointer.locations[location_index].unit_checksum = checksum_of_the_root_bytes;
        entry[pointer_field].copy_from_slice(&common_pointer_head_corruption::node_pointer_bytes(
            &pointer,
        ));
    });
    assert_eq!(level, 1, "根层级 1、直接管容器");
    write(
        &mut image,
        child.locations[location_index].device.0,
        common_pointer_head_corruption::the_slot_of(&child).0 * SLOT,
        &root_bytes,
    );
    image
}

/// H-V 的判法：两种盘序（码 2 那一份在位置条目 0 / 1）各判一次，`entry_invariant` 都红、违例说明点名孩子头里的
/// `class_named_in_the_violation`，两种盘序红的同一批（改之前只看第一份自证过的副本：条目不改时码 2 那份在盘 1 上漏判，
/// 条目改成类型段 0 时码 2 那份在盘 0 上漏判、另连带 I-7.2 等）。
fn assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
    clean: &MemoryPool,
    kind: u16,
    change_the_entry: &dyn Fn(&mut [u8]),
    entry_invariant: &'static str,
    class_named_in_the_violation: &str,
) {
    let mut violated_on_each_device = Vec::new();
    for location_index in 0..2 {
        let image = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
            clean,
            kind,
            location_index,
            change_the_entry,
        );
        let verdicts = check_pool_image(&image);
        let entry_verdict = verdicts
            .iter()
            .find(|(identifier, _)| *identifier == entry_invariant)
            .map(|(_, entry_verdict)| entry_verdict)
            .expect("清单里有");
        assert!(
            matches!(
                entry_verdict,
                InvariantVerdict::Violated(detail)
                    if detail.contains(&format!("孩子头里的类标签是 {class_named_in_the_violation}"))
            ),
            "码 2 那一份在位置条目 {location_index}：{entry_invariant} 红、说明点名孩子头里的 {class_named_in_the_violation}：{verdicts:?}"
        );
        violated_on_each_device.push(violated_invariant_set(&verdicts));
    }
    assert_eq!(
        violated_on_each_device[0], violated_on_each_device[1],
        "同一张镜像只差码 2 那一份在哪块盘：红的同一批"
    );
}

/// 条目不改那一形：类型段说码 3，一份自证成码 2。
fn leave_the_entry_as_it_is(_entry: &mut [u8]) {}

#[test]
fn an_inode_tree_entry_whose_leaf_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_inode_entry_invariant_alike_on_either_device(
) {
    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
        &build_pool("known-bad-inode-entry-leaf-copy-verifying-as-a-node").memory_pool(),
        TREE_KIND_INODE,
        &leave_the_entry_as_it_is,
        "I-9.2",
        "IndexNode",
    );
}

#[test]
fn an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_inode_entry_invariant_alike_on_either_device(
) {
    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
        &build_pool("known-bad-inode-entry-type-zero-leaf-copy-verifying-as-a-node").memory_pool(),
        TREE_KIND_INODE,
        &|entry| {
            change_the_entry_type(
                entry,
                PACKED_RECORD_TYPE_INODE,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            );
            zero_the_three_identity_fields(entry);
        },
        "I-9.2",
        "PackedRecordUnit",
    );
}

#[test]
fn a_tombstone_tree_entry_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_tombstone_entry_invariant_alike_on_either_device(
) {
    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-entry-container-copy-verifying-as-a-node",
        ),
        TREE_KIND_TOMBSTONE,
        &leave_the_entry_as_it_is,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "IndexNode",
    );
}

#[test]
fn a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_tombstone_entry_invariant_alike_on_either_device(
) {
    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-entry-type-zero-container-copy-verifying-as-a-node",
        ),
        TREE_KIND_TOMBSTONE,
        &|entry| {
            change_the_entry_type(
                entry,
                PACKED_RECORD_TYPE_TOMBSTONE,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            );
            zero_the_three_identity_fields(entry);
        },
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "PackedRecordUnit",
    );
}

#[test]
fn a_lineage_tree_entry_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_lineage_entry_invariant_alike_on_either_device(
) {
    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-entry-container-copy-verifying-as-a-node",
        )
        .image,
        TREE_KIND_LINEAGE,
        &leave_the_entry_as_it_is,
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "IndexNode",
    );
}

#[test]
fn a_lineage_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_lineage_entry_invariant_alike_on_either_device(
) {
    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-entry-type-zero-container-copy-verifying-as-a-node",
        )
        .image,
        TREE_KIND_LINEAGE,
        &|entry| {
            change_the_entry_type(
                entry,
                PACKED_RECORD_TYPE_LINEAGE,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            );
            zero_the_three_identity_fields(entry);
        },
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "PackedRecordUnit",
    );
}

// ─── 第 30 批（删除线）：H-V 子树（F-W）、inode 读不出孩子时子指针出生树大一、认孩子的类时第一份坏而第二份自证成别类 ───

/// F-W 的判法（判决 `research/prompts/m3-batch28-code-r1-main-verification.md` 第二节「K2 H-V 两份都看之后判错」那一行、第三节第 1 条；
/// 攻方 `research/prompts/m3-batch28-code-r1-opus-output.md` 第 1.1 节那几形）：`with_the_violation` 是 `clean` 上把种类 `kind` 那棵树根
/// 第一条条目指的那片码 3 容器两份一起改坏（重封、整单元校验和沿引用链补到根）的镜像，只它就红 `subtree_invariant`。
/// 再做 H-V 那一步（[`image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device`]，条目不改）：一份换成这棵树根的码 2 字节、
/// 按改过的位置条目自证得过，另一份就是那片带违例、按自己的位置条目自证得过的码 3 容器（类型段说的那一类，读者用的就是它）。
/// 两种盘序（码 2 那一份在位置条目 0 / 1）各判一次：`subtree_invariant` 红（子树照类型段那一支走、照判）、`entry_invariant` 红；
/// 槽数按类型段那一类登记，不连带 I-3.1、I-3.11、I-5.1；两种盘序红的同一批。
/// 对照：同一形只做 H-V、不放违例，`subtree_invariant` 不红（红的是那片容器里的违例，不是 H-V 那一步带出来的）。
/// 改之前（第 28、29 批）「孩子是另一类」那一支见别类就不往下走：`subtree_invariant` 两种盘序都报不适用或成立。
fn assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
    clean: &MemoryPool,
    with_the_violation: &MemoryPool,
    kind: u16,
    entry_invariant: &'static str,
    subtree_invariant: &'static str,
) {
    let violated_by_the_violation_only =
        violated_invariant_set(&check_pool_image(with_the_violation));
    assert!(
        violated_by_the_violation_only.contains(subtree_invariant),
        "对照：只放违例、不做 H-V，{subtree_invariant} 红（{violated_by_the_violation_only:?}）"
    );
    let mut violated_on_each_device = Vec::new();
    for location_index in 0..2 {
        let without_the_violation =
            image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
                clean,
                kind,
                location_index,
                &leave_the_entry_as_it_is,
            );
        let violated_without_the_violation =
            violated_invariant_set(&check_pool_image(&without_the_violation));
        assert!(
            !violated_without_the_violation.contains(subtree_invariant),
            "对照：码 2 那一份在位置条目 {location_index}、不放违例，{subtree_invariant} 不红（{violated_without_the_violation:?}）"
        );
        let image = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
            with_the_violation,
            kind,
            location_index,
            &leave_the_entry_as_it_is,
        );
        let verdicts = check_pool_image(&image);
        let violated = violated_invariant_set(&verdicts);
        assert!(
            violated.contains(subtree_invariant),
            "码 2 那一份在位置条目 {location_index}：类型段那一类那一份的子树照走，里面的违例 {subtree_invariant} 红：{verdicts:?}"
        );
        assert!(
            violated.contains(entry_invariant),
            "码 2 那一份在位置条目 {location_index}：条目那一条 {entry_invariant} 照判红：{verdicts:?}"
        );
        for invariant_a_wrong_slot_count_drags_in in ["I-3.1", "I-3.11", "I-5.1"] {
            assert!(
                !violated.contains(invariant_a_wrong_slot_count_drags_in),
                "码 2 那一份在位置条目 {location_index}：槽数按类型段那一类登记，不连带 {invariant_a_wrong_slot_count_drags_in}：{verdicts:?}"
            );
        }
        // G4（判决 `research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M3 缺口 G4 那一行、第三节第 4 条）：码 2 那一份按类型段那一类
        // （码 3）的 32 KiB 读回比校验和，对不上，I-2.1 红，I-4.8、I-7.4 跟着（`.claude/kb/invariants.md` I-9.2 / I-9.17 / I-9.23「另一份按同一长度读回比校验和，
        // 对不上照红 I-2.1」）。中央映射条目读那片容器时同一份也对不上，这三条不只由 F-W 这一读红。
        for invariant_of_the_checksum_family in ["I-2.1", "I-4.8", "I-7.4"] {
            assert!(
                violated.contains(invariant_of_the_checksum_family),
                "码 2 那一份在位置条目 {location_index}：按类型段那一类的长度读回对不上，{invariant_of_the_checksum_family} 红：{verdicts:?}"
            );
        }
        violated_on_each_device.push(violated);
    }
    assert_eq!(
        violated_on_each_device[0], violated_on_each_device[1],
        "同一张镜像只差码 2 那一份在哪块盘：红的同一批"
    );
}

/// inode 记录预留段（记录偏移 120 起 20 字节，I-9.7 那三段的最后一段）的最后一个字节。
const INODE_RECORD_LAST_RESERVED_BYTE_OFFSET: usize = 139;

/// 攻方 1.1 inode 那一形（I-9.7）：新池新建文件那一版，inode 叶容器第一条记录的预留段写一个非零字节（两份、重封、补到根）。
#[test]
fn an_inode_leaf_with_a_non_zero_reserved_byte_beside_a_copy_verifying_as_a_node_reddens_the_reserved_bytes_invariant_on_either_device(
) {
    let clean = build_pool("known-bad-inode-leaf-reserved-byte-beside-a-copy-verifying-as-a-node")
        .memory_pool();
    let mut with_the_violation = clean.clone();
    mutate_unit(
        &mut with_the_violation,
        INODE_LEAF,
        |bytes| bytes[INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_LAST_RESERVED_BYTE_OFFSET] = 1,
        true,
    );
    assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
        &clean,
        &with_the_violation,
        TREE_KIND_INODE,
        "I-9.2",
        "I-9.7",
    );
}

/// 攻方 1.1 inode 那一形（I-9.15）：新池新建文件那一版，inode 叶容器第一条记录的 blocks 加 1（size 3000 ⇒ 要 6，写成 7）。
#[test]
fn an_inode_leaf_with_blocks_off_the_logical_length_beside_a_copy_verifying_as_a_node_reddens_the_blocks_invariant_on_either_device(
) {
    let clean =
        build_pool("known-bad-inode-leaf-blocks-beside-a-copy-verifying-as-a-node").memory_pool();
    let mut with_the_violation = clean.clone();
    mutate_unit(
        &mut with_the_violation,
        INODE_LEAF,
        |bytes| {
            let blocks_offset = INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_BLOCKS_OFFSET;
            let blocks = get_u64(bytes, blocks_offset);
            set_u64(bytes, blocks_offset, blocks + 1);
        },
        true,
    );
    assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
        &clean,
        &with_the_violation,
        TREE_KIND_INODE,
        "I-9.2",
        "I-9.15",
    );
}

/// 打包记录类型 `record_bytes`（登记的记录宽）少 1：I-1.7 记录宽那一步判红。
fn record_width_one_below(record_bytes: u64) -> u16 {
    u16::try_from(record_bytes - 1).expect("登记的记录宽装得进 u16")
}

/// 攻方 1.1 inode 那一形（I-1.7）：新池新建文件那一版，inode 叶容器头里的记录宽 140 → 139。
#[test]
fn an_inode_leaf_with_a_wrong_record_width_beside_a_copy_verifying_as_a_node_reddens_the_packed_container_invariant_on_either_device(
) {
    let clean = build_pool("known-bad-inode-leaf-record-width-beside-a-copy-verifying-as-a-node")
        .memory_pool();
    let mut with_the_violation = clean.clone();
    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
        &mut with_the_violation,
        TREE_KIND_INODE,
        record_width_one_below(singlefs_format::INODE_RECORD_BYTES),
    );
    assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
        &clean,
        &with_the_violation,
        TREE_KIND_INODE,
        "I-9.2",
        "I-1.7",
    );
}

/// 攻方 1.1 墓碑那一形（I-1.7）：三个单元的文件发完意图之后，墓碑容器头里的记录宽 40 → 39。
#[test]
fn a_tombstone_container_with_a_wrong_record_width_beside_a_copy_verifying_as_a_node_reddens_the_packed_container_invariant_on_either_device(
) {
    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-record-width-beside-a-copy-verifying-as-a-node",
    );
    let mut with_the_violation = clean.clone();
    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
        &mut with_the_violation,
        TREE_KIND_TOMBSTONE,
        record_width_one_below(singlefs_format::TOMBSTONE_RECORD_BYTES),
    );
    assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
        &clean,
        &with_the_violation,
        TREE_KIND_TOMBSTONE,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "I-1.7",
    );
}

/// 攻方 1.1 谱系那一形（I-1.7）：回退到 A 之后，谱系容器头里的记录宽 24 → 23。
#[test]
fn a_lineage_container_with_a_wrong_record_width_beside_a_copy_verifying_as_a_node_reddens_the_packed_container_invariant_on_either_device(
) {
    let clean = image_after_rolling_back_to_the_first_root(
        "known-bad-lineage-record-width-beside-a-copy-verifying-as-a-node",
    )
    .image;
    let mut with_the_violation = clean.clone();
    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
        &mut with_the_violation,
        TREE_KIND_LINEAGE,
        record_width_one_below(singlefs_format::LINEAGE_RECORD_BYTES),
    );
    assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
        &clean,
        &with_the_violation,
        TREE_KIND_LINEAGE,
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "I-1.7",
    );
}

/// 第 28 批代码轮正推腿 X2 / X11（判决 `research/prompts/m3-batch28-code-r1-main-verification.md` 第二节「K1 测试」那一行、第三节第 3 条；
/// 正推 `research/prompts/m3-batch28-code-r1-sonnet-output.md` 第 3.4 节第 1 条）：inode 那一形只把身份引用出生树加 1，子指针出生树比它小，
/// 读不出孩子时那一小句的 `==` 改成 `>=` 照样红、钉不住。这一形把子指针头部出生树加 1（比身份引用出生树大）：I-9.2 照判、红。
#[test]
fn an_inode_tree_entry_whose_child_pointer_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back_reddens_the_inode_entry_invariant(
) {
    assert_a_birth_tree_field_one_off_reddens_the_entry_invariant_over_a_container_that_does_not_read_back(
        &build_pool("known-bad-inode-entry-pointer-birth-tree-unreadable-leaf").memory_pool(),
        TREE_KIND_INODE,
        PACKED_RECORD_TYPE_INODE,
        INTERNAL_ENTRY_CHILD_POINTER_BIRTH_TREE,
        "I-9.2",
    );
}

/// 正推腿 X13 / X14（同上判决那一行、第三节第 3 条；正推第 3.4 节第 2 条）的判法：新池新建文件那一版，inode 树根第一条条目不改（类型段 2），
/// 叶容器盘序大那一份换成 inode 树根的码 2 字节、按改过的位置条目自证得过（[`image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device`]），
/// 再照 `break_the_first_copy` 把位置条目 0 那一份弄坏（自证不过）。认孩子的类时第一份那一处 `continue` 要往下看第二份：
/// 认出码 2，I-9.2 红、违例说明点名 IndexNode（改成 `break` 的话认不出别类，照类型段 2 读、两份都读不出，两段出生树相等，I-9.2 报成立）。
/// 两份里没有一份按码 3 自证得过：不走 F-W 那一支。
fn assert_a_second_copy_verifying_as_a_node_behind_a_broken_first_copy_reddens_the_inode_entry_invariant(
    tag: &str,
    break_the_first_copy: &dyn Fn(&mut MemoryPool),
) {
    let clean = build_pool(tag).memory_pool();
    let mut image = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
        &clean,
        TREE_KIND_INODE,
        1,
        &leave_the_entry_as_it_is,
    );
    break_the_first_copy(&mut image);
    assert_the_violation_names_the_child_class(
        &image,
        "I-9.2",
        PACKED_RECORD_TYPE_INODE,
        "IndexNode",
    );
}

/// X13：位置条目 0 那一份头里的类标签 3 → 9（不在登记表里，也就按哪一类的长度都自证不过）。
#[test]
fn an_inode_leaf_whose_first_copy_carries_an_unregistered_class_tag_and_whose_second_copy_verifies_as_a_node_reddens_the_inode_entry_invariant(
) {
    assert_a_second_copy_verifying_as_a_node_behind_a_broken_first_copy_reddens_the_inode_entry_invariant(
        "known-bad-inode-leaf-first-copy-unregistered-class-second-copy-node",
        &|image| {
            let first = child_pointer_of_an_entry_of_the_root_of_kind(image, TREE_KIND_INODE, 0)
                .locations[0];
            let offset = first.slot.0 * SLOT;
            let mut header =
                read(image, first.device.0, offset, common_pointer_head_corruption::node_bytes());
            header[UNIT_HEADER_CLASS_TAG_OFFSET] = UNIT_CLASS_NOT_REGISTERED;
            write(image, first.device.0, offset, &header);
        },
    );
}

/// X14：位置条目 0 那一份读不出（那条位置条目的槽写成盘尾之后的第一槽，读者按盘的契约拒读）。
#[test]
fn an_inode_leaf_whose_first_copy_does_not_read_back_and_whose_second_copy_verifies_as_a_node_reddens_the_inode_entry_invariant(
) {
    assert_a_second_copy_verifying_as_a_node_behind_a_broken_first_copy_reddens_the_inode_entry_invariant(
        "known-bad-inode-leaf-first-copy-unreadable-second-copy-node",
        &|image| {
            let first_slot_past_the_device_end =
                singlefs_core::address::SlotNumber(image.device_size_in_bytes / SLOT);
            change_an_entry_of_the_root_of_kind(image, TREE_KIND_INODE, 0, |entry| {
                let pointer_field = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
                    ..CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
                        + CONTAINER_TREE_ENTRY_CHILD_POINTER_BYTES;
                let mut pointer = singlefs_core::pointer::NodePointer::read_from(
                    &mut singlefs_core::bytes::ByteReader::at(entry, pointer_field.start),
                );
                pointer.locations[0].slot = first_slot_past_the_device_end;
                entry[pointer_field].copy_from_slice(
                    &common_pointer_head_corruption::node_pointer_bytes(&pointer),
                );
            });
            let first = child_pointer_of_an_entry_of_the_root_of_kind(image, TREE_KIND_INODE, 0)
                .locations[0];
            assert!(
                PoolReader::read(
                    image,
                    first.device,
                    DeviceOffsetInBytes(first.slot.0 * SLOT),
                    common_pointer_head_corruption::node_bytes()
                )
                .is_none(),
                "位置条目 0 那一份的头读不出"
            );
        },
    );
}

// ─── 第 35 批：由记录造出来的两类版本取哪条记录的新根段（FL）、树表指针全零（Q1、被抛弃的根）、I-5.5 认出来而没有数据单元（Q4） ───

/// 恰好要 `data_units` 个数据单元的内容（最后一个单元装一半）：顺序写一次发布切成 `data_units` 条记录（一个数据单元一条）。
fn content_needing_data_units(data_units: usize) -> Vec<u8> {
    let payload_capacity = singlefs_core::unit::data_unit_payload_capacity();
    (0..(data_units - 1) * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 13 + 8) % 251).expect("小于 256"))
        .collect()
}

/// 新池新建文件 A (1, 3) 之后顺序写三个数据单元的发布 B (1, 4)：B 切成三条记录（计数器 4、5、6，每条带提交标记、只有 6 带末条标志），
/// 两块盘各一份；崩在 B 的根槽 FUA 之前（攻方报告 `research/prompts/m3-formal-code-b34-r1-opus-output.md` P1-b 那张 crafted 镜像的同形）。
/// 最新的根还是 A 的，实例表里没有实例 1 的行：B 是下一次挂载先施加的那一版。交回镜像与 A、B 两次发布的输出。
struct CrashBeforeTheRootOfAThreeRecordPublish {
    image: MemoryPool,
    previous: TransactionOutput,
    publish: TransactionOutput,
}

fn crash_image_with_a_three_record_publish_but_not_its_root(
    tag: &str,
) -> CrashBeforeTheRootOfAThreeRecordPublish {
    let mut pool = build_pool(tag);
    let operations_through_the_first_publish = pool.retained_operations().len();
    let previous = pool.output.clone();
    let content = content_needing_data_units(3);
    let publish = {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_sequential_write(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            InstanceGeneration(1),
        )
        .expect("顺序写三个数据单元")
    };
    let counters: Vec<u64> = publish
        .earlier_records_of_this_publish
        .iter()
        .map(|written| written.record.counter)
        .chain([publish.record.counter])
        .collect();
    assert_eq!(
        (counters, publish.root.checkpoint_txg),
        (vec![4, 5, 6], CheckpointTxg(4)),
        "B 是 txg 4、切成计数器 4、5、6 三条记录"
    );
    let operations = pool.retained_operations();
    let classifier = geometry();
    let kinds_of_the_publish: Vec<StepKind> = operations[operations_through_the_first_publish..]
        .iter()
        .map(|retained| classifier.classify(&retained.operation))
        .collect();
    let root_record_step = kinds_of_the_publish
        .iter()
        .position(|kind| *kind == StepKind::RootRecordFua)
        .expect("发布 B 有一步根槽 FUA");
    assert_eq!(
        (
            kinds_of_the_publish[..root_record_step]
                .iter()
                .filter(|kind| **kind == StepKind::JournalRecord)
                .count(),
            kinds_of_the_publish[root_record_step - 1]
        ),
        (6, StepKind::Barrier),
        "根槽 FUA 之前：B 的三条记录两块盘各写了一份，紧挨着根槽的是记录之后那道屏障"
    );
    let mut image = pool.memory_pool_after_mkfs();
    image.apply(
        &operations
            [pool.mkfs_operation_count..operations_through_the_first_publish + root_record_step],
    );
    CrashBeforeTheRootOfAThreeRecordPublish {
        image,
        previous,
        publish,
    }
}

/// 把 B 的第一条记录（计数器 4，两块盘各一份）的新根段七项换成上一版 A 那条根的（写侧违反 D23（journal 的角色与格式） 已定项 15
/// 「每条记录都带同一份新根段」的样子），重封、后面两条重链（攻方报告 P1-b 的 s1 改法）。
fn write_the_root_of_the_previous_version_into_the_first_record_of_the_publish(
    image: &mut MemoryPool,
    crash: &CrashBeforeTheRootOfAThreeRecordPublish,
) {
    let first_counter = crash
        .publish
        .earlier_records_of_this_publish
        .first()
        .expect("B 有三条记录")
        .record
        .counter;
    let stale = &crash.previous.root;
    rewrite_journal_record_and_rechain_the_records_after_it(image, first_counter, |record| {
        record.new_tree_table = stale.tree_table;
        record.new_mapping_root = stale.mapping_root;
        record.new_tree_identifier_watermark = stale.tree_identifier_watermark;
        record.new_rollback_floor = stale.rollback_floor;
        record.new_instance_table = stale.instance_table;
        record.new_allocation_record_tree_root = stale.allocation_record_tree_root;
        record.new_unmount_marker = stale.unmount_marker;
    });
}

/// 改法 FL（判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P1-b，攻方报告 P1-b 的 s3 形）：
/// 下一次挂载先施加的那一版 B 切成三条记录，第一条的新根段写成上一版 A 的，B 自己重写的那片分配记录叶里两条记录罩住同一个槽
/// （新的整单元校验和沿引用链补到 B 的树表，再补进 B 的三条记录）。core 恢复走到带末条标志的那一条才整次施加、施加的是它的新根段，
/// 重建 B 的分配器时拒在「同一块盘上两条分配记录罩住同一个槽」上。checker 认这一版也取末条那一条：只红 I-5.4，点名下一次挂载先施加的那一版；
/// 取先到的那一条时走的是 A 的树，这张镜像一条都不红。
#[test]
fn a_stale_first_record_of_the_publish_the_next_mount_applies_first_does_not_hide_two_allocation_records_covering_one_slot(
) {
    let crash = crash_image_with_a_three_record_publish_but_not_its_root(
        "known-bad-stale-first-record-over-overlapping-records-of-the-next-mount-version",
    );
    let verdicts_on_the_clean_image = check_pool_image(&crash.image);
    assert_eq!(
        violated_invariants(&verdicts_on_the_clean_image),
        Vec::<&str>::new(),
        "B 的三条记录已落、根槽没落的干净崩溃镜像上一条都不许红：{verdicts_on_the_clean_image:?}"
    );
    let mut image = crash.image.clone();
    overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(&mut image, &crash.publish, 6);
    write_the_root_of_the_previous_version_into_the_first_record_of_the_publish(&mut image, &crash);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-5.4"],
        "第一条记录写旧了也不许把 B 的两条记录罩住同一个槽藏起来：只红 I-5.4：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-5.4") else {
        panic!("上面判过 I-5.4 红");
    };
    assert!(
        detail.starts_with("下一次挂载先施加的那一版（实例 1、txg 4）那棵分配记录树"),
        "红在下一次挂载先施加的那一版 B 的分配记录树上：{detail}"
    );

    let stream = SharedStream::new();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let image_before_the_mount = memory_pool_of_sparse_devices(&devices);
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
            MountError::Recovery(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
                what: "同一块盘上两条分配记录罩住同一个槽",
            })
        ),
        "core 施加的是末条那一份新根段（B 的树），重建分配器时拒在两条记录罩住同一个槽上：{refusal:?}"
    );
    assert_eq!(
        memory_pool_of_sparse_devices(&devices),
        image_before_the_mount,
        "拒在任何写之前：两块盘逐字节不变（{refusal:?}）"
    );
}

/// 改法 FL 由记录施加出来的那一版那一支（攻方报告 P1-b 的 s4 形）：B 的第一条记录写成 A 的新根段（只这一处，不叠别的坏法），
/// 可写挂载照常收下（core 施加末条那一份，现行版是 B (1, 4)），写行发布与暖机之后实例 2 再覆盖写一次：B 成了由记录施加出来、
/// 根槽从没落盘的那一版。checker 取末条那一条走 B 自己的树，一条都不红；取先到的那一条时走的是 A 的树、B 自己的单元没走到，
/// I-3.1 与 I-3.5 红。
#[test]
fn a_version_applied_only_by_its_records_whose_first_record_carries_the_previous_root_is_walked_from_its_last_record(
) {
    let crash = crash_image_with_a_three_record_publish_but_not_its_root(
        "known-good-stale-first-record-of-a-version-applied-only-by-its-records",
    );
    let mut image = crash.image.clone();
    write_the_root_of_the_previous_version_into_the_first_record_of_the_publish(&mut image, &crash);
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let mut mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    assert_eq!(
        (
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.journal.prefix_applied
        ),
        (CheckpointTxg(3), CheckpointTxg(4), 3),
        "择 A 的根 (1, 3)，施加 B 的三条记录，现行版是 (1, 4)"
    );
    let current = mounted
        .current
        .into_file_version()
        .expect("施加了 B，现行那一版带文件");
    let content: Vec<u8> = (0..2900usize)
        .map(|index| u8::try_from((index * 11 + 2) % 251).expect("小于 256"))
        .collect();
    {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut mounted.allocator,
            &current,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 180,
            },
            InstanceGeneration(2),
        )
        .expect("实例 2 再覆盖写一次");
    }
    let image_after_the_overwrite = memory_pool_of_sparse_devices(&devices);
    let verdicts = check_pool_image(&image_after_the_overwrite);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "B 只由记录施加出来、第一条记录写旧了：checker 与 core 同取末条，走的是 B 自己的树，一条都不红：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image_after_the_overwrite, "I-3.1"),
        InvariantVerdict::Holds,
        "I-3.1 要真被评估过且成立（B 自己的单元并进了遍历）"
    );
}

/// 这次发布写的第一个数据单元，在这次发布重写的分配记录叶里它那两条未释放记录（两块盘各一条，分在两片叶里）的分配代从这次发布的
/// txg 改成 txg + 1，叶重封、新的整单元校验和沿引用链补上去，再补进引用改过的单元的那几条记录；数据单元一个字节不动。
/// 盘上留下的是 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 的违例。交回那个数据单元的起点槽。
fn raise_the_allocation_generation_of_a_data_unit_the_publish_wrote_past_its_birth(
    image: &mut MemoryPool,
    version: &TransactionOutput,
    record_copies_referencing_the_changed_units: usize,
) -> u64 {
    let units = units_found_on_device_zero(image);
    let checksums_before = unit_checksums(image, &units);
    let data_unit_slot = version
        .units
        .iter()
        .find(|unit| {
            matches!(unit.identity, TransactionUnit::Data(_))
                && version.rewritten.contains(&unit.identity)
        })
        .expect("这次发布写了数据单元")
        .slot
        .0;
    let birth_txg = version.root.checkpoint_txg.0;
    let leaves: Vec<u64> = version
        .units
        .iter()
        .filter(|unit| {
            version.rewritten.contains(&unit.identity)
                && matches!(
                    unit.identity,
                    TransactionUnit::AllocationTree
                        | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
                )
        })
        .map(|unit| unit.slot.0)
        .filter(|slot| {
            let mut candidate_leaf = read(image, 0, slot * SLOT, 16384);
            candidate_leaf[INDEX_NODE_LEVEL_OFFSET] == 0
                && rewrite_unreleased_allocation_generation(
                    &mut candidate_leaf,
                    data_unit_slot,
                    birth_txg,
                    birth_txg + 1,
                ) == 1
        })
        .collect();
    assert_eq!(
        leaves.len(),
        2,
        "数据单元槽 {data_unit_slot} 两块盘各一份落点，两条未释放记录分在两块盘各自那片叶里"
    );
    for leaf in leaves {
        mutate_unit_of(
            image,
            &units,
            leaf,
            |bytes| {
                assert_eq!(
                    rewrite_unreleased_allocation_generation(
                        bytes,
                        data_unit_slot,
                        birth_txg,
                        birth_txg + 1
                    ),
                    1,
                    "上面挑叶时这一片改得动"
                );
            },
            true,
        );
    }
    let changes = units_whose_checksum_changed(&checksums_before, &unit_checksums(image, &units));
    assert_eq!(
        propagate_into_journal_records(image, &changes),
        record_copies_referencing_the_changed_units,
        "只有这次发布落了的那几条记录（两块盘各一份）引用这一版改过的单元"
    );
    data_unit_slot
}

/// 改法 R1（判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P1 与第三节第 1 条，攻方报告
/// `research/prompts/m3-formal-code-b35-r1-opus-output.md` P1 第 2 条 `cut=records-2..2-of-2` 那一形）：发布 B (1, 4) 切成三条记录，
/// 首条（计数器 4，两块盘各一份）没落、后两条落了，崩在根槽 FUA 之前——断号。core 从 A 的锚点起要计数器 4、读到的是 5，前缀断在这里，
/// B 整次不施加。B 重写的分配记录叶里放两处坏法：两条记录罩住同一个槽（I-5.4 的形），B 的一个数据单元那两条未释放记录分配代改成 txg + 1
/// （I-3.10 的形）；新的整单元校验和补进 B 的三条记录、后两条重链。
/// 阳性对照：三条都落了时 B 是下一次挂载先施加的那一版，I-3.10 与 I-5.4 都红在 B 上。
/// 首条没落时 checker 照 I-4.1 那一份重放认这一版：B 不施加就不认，I-5.4 与 I-3.10 都不读 B 的分配记录树，这张镜像一条都不红
/// （只看带末条标志的那一条、即计数器 6 时认 B，两条照红）；可写挂载照常收下：择 A 的根，一条记录都不施加，现行版仍是 A。
#[test]
fn a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first() {
    let crash = crash_image_with_a_three_record_publish_but_not_its_root(
        "known-good-publish-with-a-gap-before-its-last-record",
    );
    let mut image = crash.image.clone();
    overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(&mut image, &crash.publish, 6);
    raise_the_allocation_generation_of_a_data_unit_the_publish_wrote_past_its_birth(
        &mut image,
        &crash.publish,
        6,
    );
    let first_counter = crash
        .publish
        .earlier_records_of_this_publish
        .first()
        .expect("B 有三条记录")
        .record
        .counter;
    rewrite_journal_record_and_rechain_the_records_after_it(
        &mut image,
        first_counter,
        |_record_kept_as_is| {},
    );
    let verdicts_with_every_record_landed = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts_with_every_record_landed),
        ["I-3.10", "I-5.4"],
        "阳性对照：B 的三条记录都落了，B 是下一次挂载先施加的那一版，两处坏法各红一条：{verdicts_with_every_record_landed:?}"
    );

    // mkfs 把整个 journal 环写清零（`crates/singlefs-core/src/make_filesystem.rs` 里 `write_zeroes_at(journal_ring_start, …)`），
    // 计数器 4 那一格在 B 之前是全零：写回全零就是首条那两次写没落。
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    for device in DEVICES {
        write(
            &mut image,
            device,
            record_offset(first_counter, JOURNAL_RING_DEFAULT_BYTES).0,
            &vec![0u8; record_bytes],
        );
    }
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "B 断了号、core 不施加：B 的分配记录树不在 I-5.4、I-3.10 判的版本里，两处坏法都不红：{verdicts:?}"
    );
    for invariant in ["I-5.4", "I-3.10"] {
        assert_eq!(
            verdict(&image, invariant),
            InvariantVerdict::Holds,
            "{invariant} 在 A 那一版上真被评估过且成立"
        );
    }

    let stream = SharedStream::new();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let mounted = mount_writable(&parameters(), &mut devices)
        .expect("B 不施加：可写挂载照 A 那一版收下，B 的分配记录树它不读");
    assert_eq!(
        (
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.journal.prefix_applied
        ),
        (CheckpointTxg(3), CheckpointTxg(3), 0),
        "择 A 的根 (1, 3)，前缀断在计数器 5（要的是 4），一条记录都不施加"
    );
}

/// A 的根落了之后实例 1 再覆盖写两次：B (1, 4)、C (1, 5)，各一条记录（计数器 4、5）。交回整段历史写完的镜像（根槽都在）与 B、C 两次发布的输出；
/// 要「B、C 都只在记录里」的，改完单元之后再把两次的根槽改坏（`tear_the_root_slot_written_for_txg`：改单元时沿引用链重封根槽，先改坏会被封回去）。
struct TwoSingleRecordPublishes {
    image: MemoryPool,
    first_publish: TransactionOutput,
    second_publish: TransactionOutput,
}

fn image_with_two_single_record_publishes(tag: &str) -> TwoSingleRecordPublishes {
    let mut pool = build_pool(tag);
    let content_of = |length: usize, seed: usize| -> Vec<u8> {
        (0..length)
            .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
            .collect()
    };
    let first_publish = overwrite_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_of(CONTENT_BYTES_OF_PUBLISH_B, 3),
    );
    let second_publish = overwrite_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_of(CONTENT_BYTES_OF_PUBLISH_C, 11),
    );
    assert_eq!(
        (
            (
                first_publish.root.checkpoint_txg,
                first_publish.record.counter
            ),
            (
                second_publish.root.checkpoint_txg,
                second_publish.record.counter
            )
        ),
        ((CheckpointTxg(4), 4), (CheckpointTxg(5), 5)),
        "B、C 各一条记录：txg 4 计数器 4、txg 5 计数器 5"
    );
    TwoSingleRecordPublishes {
        image: pool.memory_pool(),
        first_publish,
        second_publish,
    }
}

/// 改法 R1 的另一半：前缀停在后面那次发布的点名单元上时，停之前整次收下的发布 core 照施加（`crates/singlefs-core/src/recovery.rs`
/// 的 `replay_journal`：前缀停在验不过的那一条之前），第一次那一版仍是下一次挂载先施加的那一版。B、C 都只在记录里，
/// B 重写的一片分配记录叶里两条记录罩住同一个槽（补进 B 那条记录、C 那条按它重链），C 那条记录第一个点名单元在第二份落点那块盘上翻一个字节（不重封）：
/// 重放施加 B、停在 C 之前。I-4.1 报不适用（起点含介质损坏，那一行自己的射程）；I-5.4 照认 B，只红 I-5.4、点名下一次挂载先施加的那一版
/// （实例 1、txg 4）。core 的只读恢复施加 B 一条、验不过 C 一条，落在 (1, 4)。
#[test]
fn a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first(
) {
    let built = image_with_two_single_record_publishes(
        "known-bad-overlapping-records-of-the-next-mount-version-before-a-named-unit-that-does-not-verify",
    );
    let mut image = built.image.clone();
    overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(
        &mut image,
        &built.first_publish,
        2,
    );
    rewrite_journal_record_and_rechain_the_records_after_it(
        &mut image,
        built.first_publish.record.counter,
        |_record_kept_as_is| {},
    );
    for txg in [4u64, 5] {
        tear_the_root_slot_written_for_txg(&mut image, txg);
    }
    let offset_of_the_second_record = record_offset(
        built.second_publish.record.counter,
        JOURNAL_RING_DEFAULT_BYTES,
    )
    .0;
    let second_record = read(&image, 0, offset_of_the_second_record, 4096);
    assert!(
        get_u32(&second_record, JOURNAL_RECORD_NAMED_COUNT_OFFSET) > 0,
        "C 那条记录有点名项"
    );
    let second_location = JOURNAL_RECORD_HEADER_BYTES + 14;
    let named_device = get_u32(&second_record, second_location);
    let mut named_slot_bytes = [0u8; 8];
    named_slot_bytes[..6]
        .copy_from_slice(&second_record[second_location + 4..second_location + 10]);
    let named_slot = u64::from_le_bytes(named_slot_bytes);
    let mut unit = read(&image, named_device, named_slot * SLOT, 16384);
    unit[1000] ^= 0xff;
    write(&mut image, named_device, named_slot * SLOT, &unit);

    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        (
            report.effective_root,
            report.journal.prefix_applied,
            report.journal.verification_failed
        ),
        (Some((InstanceGeneration(1), CheckpointTxg(4))), 1, 1),
        "core 施加 B 那一条、验不过 C 那一条，落在 (1, 4)"
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-5.4"],
        "重放停在 C 之前，B 照认：它的两条记录罩住同一个槽只红 I-5.4：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-5.4") else {
        panic!("上面判过 I-5.4 红");
    };
    assert!(
        detail.starts_with("下一次挂载先施加的那一版（实例 1、txg 4）那棵分配记录树"),
        "红在下一次挂载先施加的那一版 B 的分配记录树上：{detail}"
    );
    match verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE) {
        InvariantVerdict::NotApplicable(reason) => assert!(
            reason.contains("介质损坏"),
            "I-4.1 报不适用的理由要说是起点含介质损坏：{reason}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("前缀停在 C 的点名单元上时 I-4.1 该报不适用，实际 {other:?}")
        }
    }
}

/// 计数器为 `counter` 的那条 journal 记录（两块盘各一份）新根段里的树表指针整条写零，经 `JournalRecord::to_bytes` 重封。
/// 不重链后面的记录：这里改的两条记录（由记录施加出来的 (1, 5)、下一次挂载先施加的 B）在环里都是本实例最后一条，
/// 下一条（若有）是别的实例的第一条，它的反向链不接在这一条上（`common::rewrite_journal_record_and_rechain_the_records_after_it`
/// 不分实例照接，会把那一条改坏、I-8.6 红）。
fn write_an_all_zero_tree_table_pointer_into_the_new_root_segment_of_journal_record(
    image: &mut MemoryPool,
    counter: u64,
) {
    let filesystem_identifier =
        singlefs_core::unit::unit_filesystem_identifier(&parameters().filesystem_identifier);
    let offset = record_offset(counter, JOURNAL_RING_DEFAULT_BYTES).0;
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    for device in DEVICES {
        let mut record = JournalRecord::parse(
            &read(image, device, offset, record_bytes),
            filesystem_identifier,
        )
        .expect("改之前这条记录自证过");
        assert_eq!(
            record.counter, counter,
            "环里那一格是计数器 {counter} 的记录"
        );
        record.new_tree_table = singlefs_core::pointer::NodePointer::empty_root();
        let rewritten = record.to_bytes();
        assert!(
            rewritten[JOURNAL_RECORD_NEW_ROOT_SEGMENT_OFFSET
                ..JOURNAL_RECORD_NEW_ROOT_SEGMENT_OFFSET + 86]
                .iter()
                .all(|byte| *byte == 0),
            "新根段里的树表指针整条全零"
        );
        write(image, device, offset, &rewritten);
    }
}

/// Q1 甲（用户 2026-10-06 定「甲 两类都判损坏」，判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第五节第 1 条），
/// 由记录施加出来的那一版那一支（正推探针 `zz_probe_q1` 的 q1b 形）：挂载之后只由记录施加出来的那一版 (1, 5) 那条记录（两块盘各一份）
/// 新根段里的树表指针整条写零、重封、后面的记录重链。core 施加记录之后拼出的根交给同一个根记录读者，照它读盘 0 槽 0、读不出；
/// 主走读与根环里的根同一读法跟过去：盘 0 槽 0 那两份对不上（I-2.1），从这一版出发的遍历读不出（I-4.8、I-7.4），这一版自己的单元
/// 没走到、遍历少算（I-3.1）。改之前读成「这一版没有树表」，只红 I-3.1、I-3.5。
#[test]
fn an_all_zero_tree_table_pointer_in_the_record_of_a_version_applied_only_by_its_journal_record_is_followed_like_the_core(
) {
    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
        "known-bad-all-zero-tree-table-pointer-of-a-version-applied-only-by-its-record",
    );
    let mut image = built.image.clone();
    write_an_all_zero_tree_table_pointer_into_the_new_root_segment_of_journal_record(
        &mut image,
        built.version_applied_only_by_its_record.record.counter,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-2.1", "I-3.1", "I-4.8", "I-7.4"],
        "(1, 5) 那条记录里树表指针全零：照 core 跟过去读不出，不再读成没有树表：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-2.1") else {
        panic!("上面判过 I-2.1 红");
    };
    assert!(
        detail.contains(
            "由记录施加出来的那一版（实例 1、txg 5）的树表（记录新根段里树表指针全零，照 core 的根记录读者读它那两条位置条目）"
        ),
        "I-2.1 写明读不对的是由记录施加出来的那一版的树表、指针从记录新根段里读来：{detail}"
    );
}

/// Q1 甲，下一次挂载先施加的那一版那一支（正推探针 `zz_probe_q1` 的 q1 形，攻方报告 P2-b 的 z6 形）：发布 B 的记录已落、根槽没落的崩溃镜像上，
/// B 那条记录（两块盘各一份）新根段里的树表指针整条写零、重封。两种挂载都拒成「槽 0 读不出」；checker 的重放（I-4.1）施加 B、
/// 施加出来的那一版照 core 跟过去走不完，I-4.1 红，别的判定与干净的崩溃镜像只差 I-5.4 那一句部分没判（认不出带不带树）。
/// 改之前这一版被读成「没有树表」，一条都不红。
#[test]
fn an_all_zero_tree_table_pointer_in_the_record_of_the_version_the_next_mount_applies_first_reddens_the_replayed_version_invariant(
) {
    let clean = crash_image_with_the_overwrite_record_but_not_its_root(
        "known-bad-all-zero-tree-table-pointer-of-the-version-the-next-mount-applies",
    );
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    let mut image = clean.clone();
    write_an_all_zero_tree_table_pointer_into_the_new_root_segment_of_journal_record(
        &mut image,
        COUNTER_OF_THE_OVERWRITE_RECORD,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE],
        "B 那条记录里树表指针全零：重放施加出来的那一版照 core 跟过去走不完：{verdicts:?}"
    );
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &verdicts_on_the_clean_image,
            &[TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE, "I-5.4"]
        ),
        [],
        "I-4.1 与 I-5.4 之外每一条的判定与干净的崩溃镜像逐项相同"
    );
    assert!(
        matches!(
            verdict(&image, "I-5.4"),
            InvariantVerdict::PartiallyJudged(unjudged)
                if unjudged.starts_with("树表指针全零的版本（core 照它读盘 0 槽 0、读不出），认不出带不带树")
                    && unjudged.ends_with("：下一次挂载先施加的那一版（实例 1、txg 4）")
        ),
        "I-5.4 照旧把这一版记成树表指针全零、认不出带不带树的部分没判：{:?}",
        verdict(&image, "I-5.4")
    );
    let InvariantVerdict::Violated(detail) =
        verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE)
    else {
        panic!("上面判过 I-4.1 红");
    };
    assert!(
        detail.contains("记录新根段里树表指针全零，照 core 的根记录读者读它那两条位置条目"),
        "I-4.1 写明施加出来的那一版走不完是因为记录新根段里树表指针全零：{detail}"
    );

    let stream = SharedStream::new();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let image_before_the_mount = memory_pool_of_sparse_devices(&devices);
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
            MountError::Recovery(RecoveryFailure::UnitUnreadable {
                slot: singlefs_core::address::SlotNumber(0)
            })
        ),
        "core 施加 B 之后照全零的树表指针读盘 0 槽 0、读不出：{refusal:?}"
    );
    assert_eq!(
        memory_pool_of_sparse_devices(&devices),
        image_before_the_mount,
        "拒在任何写之前：两块盘逐字节不变（{refusal:?}）"
    );
}

/// Q4 甲（用户 2026-10-06 定「甲 另写一句」，判决同上第五节第 4 条）：新池新建文件那一版（从 mkfs 水位第一次建树、认法 ①② 都满足）
/// 树表里 extent 树那一条的根指针整条写零、重封、沿引用链补到根槽：这一版认得出、却一个数据单元都没有，I-5.5 一版都不判，
/// 报「认出来了而没有数据单元」那一句专门理由，不落到缺省理由、也不落到「一版都没认出来」那一句。
#[test]
fn a_version_building_trees_from_the_mkfs_watermark_without_a_data_unit_leaves_the_first_data_unit_invariant_not_applicable_with_its_own_reason(
) {
    let mut image =
        build_pool("known-bad-first-tree-building-version-without-a-data-unit").memory_pool();
    let zeroed_entries = std::cell::Cell::new(0usize);
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            for index in 0..TREE_TABLE_ENTRIES_AFTER_THE_FIRST_FILE {
                let entry = tree_table_entry_offset(index);
                let kind = u16::from_le_bytes([
                    bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET],
                    bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET + 1],
                ]);
                if kind == TREE_KIND_EXTENT {
                    bytes[entry + TREE_TABLE_ENTRY_ROOT_POINTER_RANGE.start
                        ..entry + TREE_TABLE_ENTRY_ROOT_POINTER_RANGE.end]
                        .fill(0);
                    zeroed_entries.set(zeroed_entries.get() + 1);
                }
            }
        },
        true,
    );
    assert_eq!(zeroed_entries.get(), 1, "树表里 extent 树那一条恰一条");
    assert_eq!(
        verdict(
            &image,
            FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK
        ),
        InvariantVerdict::NotApplicable(
            singlefs_checker::walk::VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT
        ),
        "认出了从 mkfs 水位第一次建树的那一版而它没有数据单元：I-5.5 报这一句专门理由"
    );
}

/// 一条树表条目里树的根指针那 86 字节：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 之后（D8（核心索引结构） 已定项 8 的字段表）。
const TREE_TABLE_ENTRY_ROOT_POINTER_RANGE: std::ops::Range<usize> = 14..100;

/// 被崩溃恢复抛弃的根树表指针整条全零（判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P2-e：攻方读码报
/// 「说明写的是单元已被重新分配或抹头」）：C (2, 8) 被实例 3 的实例表判出局之后，把 C 的根槽里树表指针写零、重封。
/// 被抛弃根那一遍照 core 的根记录读者跟过去、读不出，I-7.4 照旧红；说明写明是这条根自己的树表指针全零，不再说成它引用的单元被重新分配或抹头。
#[test]
fn an_abandoned_root_whose_tree_table_pointer_is_all_zero_reddens_the_reuse_invariant_naming_the_root_itself(
) {
    let (mut image, _, _, abandoned_txg) = image_after_a_recovery_abandoned_the_third_root(
        "known-bad-abandoned-root-all-zero-tree-table-pointer",
    );
    rewrite_the_root_slot_of(&mut image, CheckpointTxg(abandoned_txg), |bytes| {
        bytes[ROOT_RECORD_TREE_TABLE_POINTER_RANGE].fill(0);
    });
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-7.4"],
        "被抛弃的根树表指针全零：只红 I-7.4（被抛弃根那一遍的判定丢掉）：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-7.4") else {
        panic!("上面判过 I-7.4 红");
    };
    assert!(
        detail.starts_with(&format!(
            "被抛弃的根（实例 2、txg {abandoned_txg}）走不完：根记录里树表指针全零，core 的根记录读者照它读盘 0 槽 0、读不出"
        )),
        "I-7.4 说明写明是这条根自己的树表指针全零：{detail}"
    );
}

// ─── 第 31 批（删除线）：墓碑树、谱系树层级 1 节点里类型段 0 的孩子照判 I-2.1（F-L0）、走读不往下读的孩子按哪种长度比的三形（测试尖度） ───

/// F-L0（判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第二节 K2 L1 第三支那一行、第三节第 1 条；攻方
/// `research/prompts/m3-batch29-code-r1-opus-output.md` 第一节 `tomb-L1T-sum01bad-type0z` / `lin-L1T-sum01bad-type0z` 那一形）：
/// 树根（层级 1）第一条条目类型段 `registered_type` → 0、身份三段清零，子指针两条位置条目的整单元校验和各翻一字节，重封到根；孩子不动。
/// 两份按头里那一类（码 3）的长度都对不上，孩子不是另一类、走类型段 0 那一支，身份引用那一半成立、孩子读不出「子单元是码 2」不判，
/// 层级 1 的节点下面不往下走、记走读失败；孩子每一份副本照判 I-2.1：红 I-2.1、I-4.8、I-7.2、I-7.4，身份那一条不红
/// （第 30 批这一支只记走读失败、I-2.1 报成立）。
fn assert_a_type_zero_entry_of_a_level_one_container_tree_root_over_two_wrong_location_checksums_reddens_the_checksum_invariant(
    mut image: MemoryPool,
    kind: u16,
    registered_type: u16,
    scene: &str,
) {
    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
        change_the_entry_type(
            entry,
            registered_type,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        zero_the_three_identity_fields(entry);
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 1);
    });
    assert_eq!(level, 1, "{scene}：树根直接管容器");
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &["I-2.1", "I-4.8", "I-7.2", "I-7.4"],
        scene,
    );
}

#[test]
fn a_tombstone_tree_entry_of_type_zero_without_an_identity_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
) {
    assert_a_type_zero_entry_of_a_level_one_container_tree_root_over_two_wrong_location_checksums_reddens_the_checksum_invariant(
        common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-entry-type-zero-no-identity-both-location-checksums-wrong",
        ),
        TREE_KIND_TOMBSTONE,
        PACKED_RECORD_TYPE_TOMBSTONE,
        "墓碑树根条目类型段 0、身份三段清零、子指针两条位置条目的校验和都坏",
    );
}

#[test]
fn a_lineage_tree_entry_of_type_zero_without_an_identity_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
) {
    assert_a_type_zero_entry_of_a_level_one_container_tree_root_over_two_wrong_location_checksums_reddens_the_checksum_invariant(
        image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-entry-type-zero-no-identity-both-location-checksums-wrong",
        )
        .image,
        TREE_KIND_LINEAGE,
        PACKED_RECORD_TYPE_LINEAGE,
        "谱系树根条目类型段 0、身份三段清零、子指针两条位置条目的校验和都坏",
    );
}

/// 内部条目（120 字节）子指针里第 `location_index` 条位置条目的整单元校验和写成 `checksum`（位置条目 = 盘 4 + 槽 6 + 整单元校验和 4）。
fn set_the_whole_unit_checksum_in_a_child_location(
    entry: &mut [u8],
    location_index: usize,
    checksum: u32,
) {
    let checksum_offset = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        + POINTER_FIRST_LOCATION_OFFSET
        + location_index * POINTER_LOCATION_BYTES
        + LOCATION_CHECKSUM_OFFSET;
    entry[checksum_offset..checksum_offset + 4].copy_from_slice(&checksum.to_le_bytes());
}

/// 不在池里的盘身份：两块盘是 0、1，写成 7 的位置条目照 I-2.5 仍按设备身份升序，那一份读不出。
const DEVICE_IDENTITY_NOT_IN_THE_POOL: u32 = 7;

/// 内部条目子指针里第 `location_index` 条位置条目的盘写成 `device`（槽与整单元校验和不动）。
fn set_the_device_in_a_child_location(entry: &mut [u8], location_index: usize, device: u32) {
    let device_offset = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        + POINTER_FIRST_LOCATION_OFFSET
        + location_index * POINTER_LOCATION_BYTES;
    entry[device_offset..device_offset + 4].copy_from_slice(&device.to_le_bytes());
}

/// 新池新建文件那一版 inode 树根（层级 1）第一条条目的孩子（码 3 叶容器，32 KiB）前 16 KiB 的 CRC-32C。
fn checksum_of_the_first_sixteen_kibibytes_of_the_first_inode_leaf(image: &MemoryPool) -> u32 {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(image, TREE_KIND_INODE, 0);
    crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
        image,
        common_pointer_head_corruption::the_slot_of(&child),
        common_pointer_head_corruption::node_bytes(),
    ))
}

/// 测试尖度 X3（正推 `research/prompts/m3-batch29-code-r1-sonnet-output.md` T-a 那张变异表；判决同一份第二节 K1 L1 测试尖度那一行、第三节第 4 条）：
/// 新池新建文件那一版 inode 树根（层级 1）第一条条目类型段 2 → 7、子指针第一条位置条目的整单元校验和换成叶容器前 16 KiB 的 CRC，重封到根；
/// 叶容器不动。孩子不往下走，第一份头里是登记过的码 3，只按 32 KiB 比、对不上：红 I-9.2、I-2.1、I-4.8、I-7.4；第二份对得上，不记走读失败、
/// I-7.2 不红。登记过的类也按 16 KiB 比时第一份对得上，I-2.1 不红。
#[test]
fn an_inode_tree_entry_with_an_unregistered_type_over_a_leaf_whose_first_location_checksum_matches_only_its_first_sixteen_kibibytes_reddens_the_checksum_invariant(
) {
    let mut image = build_pool(
        "known-bad-inode-entry-unregistered-type-first-location-checksum-of-sixteen-kibibytes",
    )
    .memory_pool();
    let checksum_of_the_first_sixteen_kibibytes =
        checksum_of_the_first_sixteen_kibibytes_of_the_first_inode_leaf(&image);
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
        );
        set_the_whole_unit_checksum_in_a_child_location(
            entry,
            0,
            checksum_of_the_first_sixteen_kibibytes,
        );
    });
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &["I-9.2", "I-2.1", "I-4.8", "I-7.4"],
        "inode 树根条目类型段 7、第一条位置条目的校验和是叶容器前 16 KiB 的 CRC",
    );
}

/// 测试尖度 X4（同上那张变异表）：新池新建文件那一版 inode 叶容器的类标签改成没登记的 9（不重封头，新的整单元校验和沿引用链补到根），
/// 再把 inode 树根第一条条目类型段 2 → 7、子指针第一条位置条目的整单元校验和换成改过类标签的叶容器前 16 KiB 的 CRC，重封到根。
/// 孩子不往下走；两份头里的码都没登记，16 KiB、32 KiB 两种长度都试：第一份按 16 KiB 对得上、第二份按 32 KiB 对得上，I-2.1 不红
/// （红的是 I-9.2 与映射树那一遍的 I-1.11）。没登记的只按 32 KiB 比时第一份对不上、只按 16 KiB 比时第二份对不上，I-2.1 都红。
#[test]
fn an_inode_tree_entry_with_an_unregistered_type_over_a_leaf_of_an_unregistered_class_whose_first_location_checksum_matches_its_first_sixteen_kibibytes_leaves_the_checksum_invariant_holding(
) {
    let mut image = build_pool(
        "known-bad-inode-entry-unregistered-type-unregistered-class-first-location-checksum-of-sixteen-kibibytes",
    )
    .memory_pool();
    let child = child_pointer_of_an_entry_of_the_root_of_kind(&image, TREE_KIND_INODE, 0);
    let leaf_slot = common_pointer_head_corruption::the_slot_of(&child);
    let mut leaf = common_pointer_head_corruption::unit_bytes_at(
        &image,
        leaf_slot,
        common_pointer_head_corruption::data_unit_bytes(),
    );
    assert_eq!(leaf[UNIT_HEADER_CLASS_TAG_OFFSET], 3, "孩子是码 3 叶容器");
    leaf[UNIT_HEADER_CLASS_TAG_OFFSET] = UNREGISTERED_UNIT_CLASS_TAG;
    common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum(
        &mut image, leaf_slot, &leaf,
    );
    let checksum_of_the_first_sixteen_kibibytes =
        checksum_of_the_first_sixteen_kibibytes_of_the_first_inode_leaf(&image);
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
        );
        set_the_whole_unit_checksum_in_a_child_location(
            entry,
            0,
            checksum_of_the_first_sixteen_kibibytes,
        );
    });
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariant_set(&verdicts),
        ["I-1.11", "I-9.2"]
            .into_iter()
            .collect::<std::collections::BTreeSet<&'static str>>(),
        "inode 叶容器类标签 9、根条目类型段 7、第一条位置条目的校验和是前 16 KiB 的 CRC：{verdicts:?}"
    );
}

/// 头里的码不在登记表里（登记的是 1、2、3）。
const UNREGISTERED_UNIT_CLASS_TAG: u8 = 9;

/// 测试尖度 X5（同上那张变异表）：新池新建文件那一版 inode 树根第一条条目类型段 2 → 7、子指针第二条位置条目的盘换成不在池里的 7，
/// 重封到根。孩子不往下走；第二份读不出（头与整单元都读不出），算对不上：红 I-9.2、I-2.1、I-4.8、I-7.4；第一份对得上，不记走读失败。
/// 读不出算对得上时 I-2.1 不红。
#[test]
fn an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_second_copy_does_not_read_back_reddens_the_checksum_invariant(
) {
    let mut image = build_pool(
        "known-bad-inode-entry-unregistered-type-second-location-on-a-device-not-in-the-pool",
    )
    .memory_pool();
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
        );
        set_the_device_in_a_child_location(entry, 1, DEVICE_IDENTITY_NOT_IN_THE_POOL);
    });
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &["I-9.2", "I-2.1", "I-4.8", "I-7.4"],
        "inode 树根条目类型段 7、第二条位置条目指着不在池里的盘",
    );
}

// ─── 第 32 批（删除线）：「走过」按读回那一份的落点认（F-KEY）、类型段 0 认「孩子是码 2」只认自证过的副本（F-ID-b）、层级 0 的根下 F-L0、
// 类型段 0 那一支的 F-W 与同长度别类（G1–G3） ───

/// 内部条目子指针里第 `location_index` 条位置条目的槽写成 `slot`（6 字节小端；盘与整单元校验和不动）。
fn set_the_slot_in_a_child_location(entry: &mut [u8], location_index: usize, slot: u64) {
    let slot_offset = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        + POINTER_FIRST_LOCATION_OFFSET
        + location_index * POINTER_LOCATION_BYTES
        + LOCATION_DEVICE_BYTES;
    entry[slot_offset..slot_offset + LOCATION_SLOT_BYTES]
        .copy_from_slice(&slot.to_le_bytes()[..LOCATION_SLOT_BYTES]);
}

/// 位置条目里盘身份的字节数（盘 4 + 槽 6 + 整单元校验和 4）。
const LOCATION_DEVICE_BYTES: usize = 4;
/// 位置条目里槽号的字节数。
const LOCATION_SLOT_BYTES: usize = 6;

/// 攻方 O3 的造法（判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K2 观察 O3 那一行、第三节第 2 条；攻方
/// `research/prompts/m3-batch31-code-r1-opus-output.md` 第四节 `variant=bad` 那两形，模型 `research/prompts/m3-batch31-code-r1-opus-model/probe_kbi.rs`
/// 的 `o3_probe`）：种类 `kind` 那棵树根（层级 1）第一条条目的孩子（码 3 容器或叶容器）两份头里的记录宽减 1（重封、整单元校验和沿引用链补到根，
/// 只这一处是 I-1.7 违例）；再把那条条目子指针第 `location_index` 条位置条目的槽改成这一遍先走过的码 2 节点 X（种类 `kind_walked_before`
/// 那棵树的根）的槽、整单元校验和写成 X 那 16 KiB 的 CRC 翻几位（这一份对不上），重封到根。另一条位置条目那一份就是带违例、
/// 按自己的位置条目自证得过的容器，读者用的就是它。
fn image_whose_first_root_entry_names_a_walked_node_beside_a_container_with_a_wrong_record_width(
    clean: &MemoryPool,
    kind: u16,
    record_bytes: u64,
    kind_walked_before: u16,
    location_index: usize,
) -> MemoryPool {
    image_whose_first_root_entry_names_a_walked_node_beside_a_container(
        clean,
        kind,
        RecordWidthOfTheFirstContainer::OneBelowTheRegistered { record_bytes },
        kind_walked_before,
        location_index,
    )
}

/// 种类 `kind` 那棵树根第一条条目指的那片容器头里的记录宽：照登记的不动，或比登记的 `record_bytes` 少 1（I-1.7 违例）。
#[derive(Clone, Copy, Debug)]
enum RecordWidthOfTheFirstContainer {
    AsRegistered,
    OneBelowTheRegistered { record_bytes: u64 },
}

/// 同 [`image_whose_first_root_entry_names_a_walked_node_beside_a_container_with_a_wrong_record_width`]，容器的记录宽照 `record_width`
/// （攻方 `research/prompts/m3-batch32-code-r1-opus-output.md` 第 1.1 节 O3 族的 `variant=bad` / `variant=good`）。
fn image_whose_first_root_entry_names_a_walked_node_beside_a_container(
    clean: &MemoryPool,
    kind: u16,
    record_width: RecordWidthOfTheFirstContainer,
    kind_walked_before: u16,
    location_index: usize,
) -> MemoryPool {
    let mut image = clean.clone();
    match record_width {
        RecordWidthOfTheFirstContainer::AsRegistered => {}
        RecordWidthOfTheFirstContainer::OneBelowTheRegistered { record_bytes } => {
            common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
                &mut image,
                kind,
                record_width_one_below(record_bytes),
            );
        }
    }
    let walked_node_pointer =
        common_pointer_head_corruption::tree_table_entry_of_kind(&image, kind_walked_before).root;
    let walked_node_slot = common_pointer_head_corruption::the_slot_of(&walked_node_pointer);
    let wrong_checksum_of_the_walked_node =
        crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
            &image,
            walked_node_slot,
            common_pointer_head_corruption::node_bytes(),
        )) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
        set_the_slot_in_a_child_location(entry, location_index, walked_node_slot.0);
        set_the_whole_unit_checksum_in_a_child_location(
            entry,
            location_index,
            wrong_checksum_of_the_walked_node,
        );
    });
    assert_eq!(level, 1, "根层级 1、直接管容器");
    image
}

/// 翻进校验和里的几位：那一份按哪种长度都对不上。
const CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY: u32 = 0x5a5a_5a5a;

/// F-KEY 的判法：两种盘序（指着先走过那个节点的是位置条目 0 / 1）各判一次，I-1.7 都红、违例说明点名那片容器的记录宽；红的同一批
/// （`allowed_to_differ` 里的几条除外，见各用例）。改之前「走过」按第 0 条位置条目认：位置条目 0 指着先走过的节点时读回来的那片容器当成走过、
/// 合并、返回，I-1.7 报成立（攻方 `PROBE o3 … collide=loc0 … i1_7=Holds`），位置条目 1 那一形照判、红。
fn assert_the_packed_container_invariant_reddens_alike_whichever_location_names_the_walked_node(
    clean: &MemoryPool,
    kind: u16,
    record_bytes: u64,
    kind_walked_before: u16,
    expected_violations_on_location_one: &[&'static str],
    allowed_to_differ: &[&'static str],
) {
    let mut violated_on_each_location = Vec::new();
    for location_index in 0..2 {
        let image = image_whose_first_root_entry_names_a_walked_node_beside_a_container_with_a_wrong_record_width(
            clean,
            kind,
            record_bytes,
            kind_walked_before,
            location_index,
        );
        let verdicts = check_pool_image(&image);
        let packed_container_verdict = verdicts
            .iter()
            .find(|(identifier, _)| *identifier == "I-1.7")
            .map(|(_, found)| found.clone())
            .expect("清单里有");
        assert!(
            matches!(
                &packed_container_verdict,
                InvariantVerdict::Violated(detail)
                    if detail.contains(&format!("记录宽 {}", record_bytes - 1))
            ),
            "指着先走过的节点的是位置条目 {location_index}：读回来的那片容器照判，I-1.7 红在它的记录宽：{verdicts:?}"
        );
        violated_on_each_location.push(violated_invariant_set(&verdicts));
    }
    assert_eq!(
        violated_on_each_location[1],
        expected_violations_on_location_one
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<&'static str>>(),
        "位置条目 1 指着先走过的节点（攻方 `collide=loc1` 那一形）红的那一批"
    );
    let outside_the_allowed = |violated: &std::collections::BTreeSet<&'static str>| {
        violated
            .iter()
            .copied()
            .filter(|invariant| !allowed_to_differ.contains(invariant))
            .collect::<std::collections::BTreeSet<&'static str>>()
    };
    assert_eq!(
        outside_the_allowed(&violated_on_each_location[0]),
        outside_the_allowed(&violated_on_each_location[1]),
        "同一张镜像只差指着先走过的节点的是哪一条位置条目：{allowed_to_differ:?} 之外红的同一批"
    );
}

/// F-KEY 墓碑那一形：三个单元的文件发完意图之后，墓碑容器记录宽 40 → 39，墓碑树根第一条条目子指针的一条位置条目指着 inode 树根（先走过）。
#[test]
fn a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before(
) {
    assert_the_packed_container_invariant_reddens_alike_whichever_location_names_the_walked_node(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-record-width-location-naming-a-walked-node",
        ),
        TREE_KIND_TOMBSTONE,
        singlefs_format::TOMBSTONE_RECORD_BYTES,
        TREE_KIND_INODE,
        &[
            "I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4",
        ],
        &[],
    );
}

/// F-KEY inode 那一形：新池新建文件那一版，inode 叶容器记录宽 140 → 139，inode 树根第一条条目子指针的一条位置条目指着 extent 树根（先走过）。
#[test]
fn an_inode_leaf_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before(
) {
    assert_the_packed_container_invariant_reddens_alike_whichever_location_names_the_walked_node(
        &build_pool("known-bad-inode-leaf-record-width-location-naming-a-walked-node")
            .memory_pool(),
        TREE_KIND_INODE,
        singlefs_format::INODE_RECORD_BYTES,
        TREE_KIND_EXTENT,
        &[
            "I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4",
        ],
        &[],
    );
}

/// F-KEY 谱系那一形：回退到 A 之后，谱系容器记录宽 24 → 23，谱系树根第一条条目子指针的一条位置条目指着 inode 树根（先走过，
/// 回退那一版照抄 A 的 inode 树，A 那条根也引用它）。读回来的那片容器（这一版新写的）的诞生代号记在它读回的那个落点上，不盖掉 inode 树根的
/// （攻方 allrbmap，判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 1 条）：两种盘序红的同一批，I-4.9 不多红
/// （改之前位置条目 0 那一形多红 I-4.9，攻方 `research/prompts/m3-batch32-code-r1-opus-output.md` 第 1.2 节 today 臂那 6 对之一）。
#[test]
fn a_lineage_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before(
) {
    assert_the_packed_container_invariant_reddens_alike_whichever_location_names_the_walked_node(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-record-width-location-naming-a-walked-node",
        )
        .image,
        TREE_KIND_LINEAGE,
        singlefs_format::LINEAGE_RECORD_BYTES,
        TREE_KIND_INODE,
        &[
            "I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4",
        ],
        &[],
    );
}

/// 攻方 A3 的造法（判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K2 N1 盘序翻面 A3 那一行；攻方同一份第一节 A3，
/// 模型 `probe_kbi.rs` 的 `matrix_probe` 里 `c16` 与 `node` 两种副本）：种类 `kind` 那棵树根第一条条目类型段 `registered_type` → 0、身份三段清零；
/// 孩子（码 3 容器）在位置条目 `container_location_index` 那块盘上不动、那条位置条目的整单元校验和换成容器前 16 KiB 的 CRC（`c16`：头里是码 3，
/// 按 32 KiB 自证不过）；另一块盘上孩子的槽写进这棵树根（码 2）的原样字节、那条位置条目的校验和换成它的 CRC（`node`：按 16 KiB 自证得过）。
/// `root_level` 给了就先把根头里的层级改成它（层级 0 的根，攻方 `l0root`）。
fn image_whose_type_zero_root_entry_names_a_container_checked_at_sixteen_kibibytes_beside_a_node(
    clean: &MemoryPool,
    kind: u16,
    registered_type: u16,
    container_location_index: usize,
    root_level: Option<u8>,
) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, kind, 0);
    let child_slot = common_pointer_head_corruption::the_slot_of(&child);
    let container = common_pointer_head_corruption::unit_bytes_at(
        clean,
        child_slot,
        common_pointer_head_corruption::data_unit_bytes(),
    );
    assert_eq!(
        container[UNIT_HEADER_CLASS_TAG_OFFSET], 3,
        "孩子是码 3 容器"
    );
    let checksum_of_the_first_sixteen_kibibytes =
        crc32_castagnoli(&container[..common_pointer_head_corruption::node_bytes()]);
    let node = root_node_bytes_of_the_tree_of_kind(clean, kind);
    let checksum_of_the_node = crc32_castagnoli(&node);
    let node_location_index = 1 - container_location_index;
    let mut image = clean.clone();
    write(
        &mut image,
        child.locations[node_location_index].device.0,
        child_slot.0 * SLOT,
        &node,
    );
    if let Some(level) = root_level {
        set_the_level_of_the_root_of_kind(&mut image, kind, level);
    }
    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
        change_the_entry_type(
            entry,
            registered_type,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        zero_the_three_identity_fields(entry);
        set_the_whole_unit_checksum_in_a_child_location(
            entry,
            container_location_index,
            checksum_of_the_first_sixteen_kibibytes,
        );
        set_the_whole_unit_checksum_in_a_child_location(
            entry,
            node_location_index,
            checksum_of_the_node,
        );
    });
    assert_eq!(level, root_level.unwrap_or(1), "根的层级是这一形要的那一级");
    image
}

/// 种类 `kind` 那棵树根头里的层级改成 `level`（重封、整单元校验和沿引用链补到根）。
fn set_the_level_of_the_root_of_kind(image: &mut MemoryPool, kind: u16, level: u8) {
    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        common_pointer_head_corruption::the_slot_of(&root_pointer),
        common_pointer_head_corruption::node_bytes(),
        |root| root[INDEX_NODE_LEVEL_OFFSET] = level,
    );
}

/// F-ID-b 的判法（用户 2026-10-07 定「只认自证过的副本」）：两种盘序（`c16` 那一份在位置条目 0 / 1）各判一次，认「孩子是码 2」只看按自己那一类长度
/// 整份自证过的副本——`c16` 不算、`node` 那一份是码 2：`identity_invariant` 两种盘序都不红；红的恰好是 I-2.1 一族与走读失败
/// （I-2.1、I-4.8、I-7.2、I-7.4：F-L0 按头里的码 3 比 `c16` 那一份对不上，层级 ≤ 1 不往下走、记走读失败），两种盘序同一批。
/// 改之前取第一份按 16 KiB 对得上的副本：`c16` 在位置条目 0 时认成码 3，`identity_invariant` 红，换盘就不红（攻方 `c0=c16 c1=node` 对 `c0=node c1=c16`）。
fn assert_the_entry_invariant_holds_alike_whichever_device_holds_the_copy_checked_at_sixteen_kibibytes(
    clean: &MemoryPool,
    kind: u16,
    registered_type: u16,
    identity_invariant: &'static str,
    root_level: Option<u8>,
) {
    for container_location_index in 0..2 {
        let image = image_whose_type_zero_root_entry_names_a_container_checked_at_sixteen_kibibytes_beside_a_node(
            clean,
            kind,
            registered_type,
            container_location_index,
            root_level,
        );
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariant_set(&verdicts),
            ["I-2.1", "I-4.8", "I-7.2", "I-7.4"]
                .into_iter()
                .collect::<std::collections::BTreeSet<&'static str>>(),
            "`c16` 那一份在位置条目 {container_location_index}：{identity_invariant} 不红，红的恰好是 I-2.1 一族与走读失败：{verdicts:?}"
        );
    }
}

#[test]
fn a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device(
) {
    assert_the_entry_invariant_holds_alike_whichever_device_holds_the_copy_checked_at_sixteen_kibibytes(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-entry-type-zero-sixteen-kibibyte-copy-beside-a-node",
        ),
        TREE_KIND_TOMBSTONE,
        PACKED_RECORD_TYPE_TOMBSTONE,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        None,
    );
}

#[test]
fn a_lineage_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_lineage_entry_invariant_holding_on_either_device(
) {
    assert_the_entry_invariant_holds_alike_whichever_device_holds_the_copy_checked_at_sixteen_kibibytes(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-entry-type-zero-sixteen-kibibyte-copy-beside-a-node",
        )
        .image,
        TREE_KIND_LINEAGE,
        PACKED_RECORD_TYPE_LINEAGE,
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        None,
    );
}

/// 同上，墓碑树根头里的层级改成 0（攻方 `l0root` 那一对）。
#[test]
fn a_tombstone_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device(
) {
    assert_the_entry_invariant_holds_alike_whichever_device_holds_the_copy_checked_at_sixteen_kibibytes(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-root-level-zero-entry-type-zero-sixteen-kibibyte-copy-beside-a-node",
        ),
        TREE_KIND_TOMBSTONE,
        PACKED_RECORD_TYPE_TOMBSTONE,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        Some(0),
    );
}

/// 同上，谱系树根头里的层级改成 0。
#[test]
fn a_lineage_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_lineage_entry_invariant_holding_on_either_device(
) {
    assert_the_entry_invariant_holds_alike_whichever_device_holds_the_copy_checked_at_sixteen_kibibytes(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-root-level-zero-entry-type-zero-sixteen-kibibyte-copy-beside-a-node",
        )
        .image,
        TREE_KIND_LINEAGE,
        PACKED_RECORD_TYPE_LINEAGE,
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        Some(0),
    );
}

/// F-L0 在层级 0 的根上（判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K1 N1 层级 0 的根下类型段 0 那一行；
/// 攻方 `PROBE l0root tree=tomb level=0 type=t0 c0=bad c1=bad`）：三个单元的文件发完意图之后，墓碑树根头里的层级 1 → 0，第一条条目类型段 1 → 0、
/// 身份三段清零、子指针两条位置条目的整单元校验和各翻一字节，重封到根。层级 0 的节点同样不往下走、记走读失败，孩子每一份副本照判 I-2.1：
/// 红 I-2.1、I-4.8、I-7.2、I-7.4（F-L0 只在层级恰为 1 时判的话，层级 0 这一形 I-2.1 不红）。
#[test]
fn a_tombstone_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
) {
    let mut image = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-root-level-zero-entry-type-zero-both-location-checksums-wrong",
    );
    set_the_level_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0);
    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_TOMBSTONE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        zero_the_three_identity_fields(entry);
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 1);
    });
    assert_eq!(level, 0, "墓碑树根头里的层级改成了 0");
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &["I-2.1", "I-4.8", "I-7.2", "I-7.4"],
        "墓碑树根层级 0、条目类型段 0、身份三段清零、子指针两条位置条目的校验和都坏",
    );
}

/// G1（判决 `research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M3 缺口 G1 那一行、第三节第 1 条）：类型段 0 那一支的 F-W。
/// 种类 `kind` 那棵树根（层级 1）第一条条目类型段 `registered_type` → 0、身份三段清零，孩子在一块盘上那一份换成这棵树根（码 2）的原样字节、
/// 按 16 KiB 自证得过（类型段 0 那一类），另一份是原来那片码 3 容器、自证成另一类
/// （[`image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device`]）。两种盘序各判一次，红的恰好是 `expected_violations`
/// （攻方 `research/prompts/m3-batch31-code-r1-opus-output.md` 那份探针 `type=t0 c0=node c1=ok` 与 `c0=ok c1=node` 两行量到的那一批），其中
/// 只有照类型段 0 那一支往下走那一份才红的 `invariant_only_the_type_zero_branch_reddens`，违例说明含 `detail_only_the_type_zero_branch_writes`。
/// 类型段 0 那一处见别类就停（不走 F-W，变异 S1 / S2）时那一份不往下走，那一条不红。
fn assert_a_type_zero_entry_whose_child_has_a_copy_verifying_as_a_node_goes_down_that_copy_on_either_device(
    clean: &MemoryPool,
    kind: u16,
    registered_type: u16,
    expected_violations: &[&'static str],
    invariant_only_the_type_zero_branch_reddens: &'static str,
    detail_only_the_type_zero_branch_writes: &str,
) {
    for location_index in 0..2 {
        let image = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
            clean,
            kind,
            location_index,
            &|entry| {
                change_the_entry_type(
                    entry,
                    registered_type,
                    CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
                );
                zero_the_three_identity_fields(entry);
            },
        );
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariant_set(&verdicts),
            expected_violations
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<&'static str>>(),
            "码 2 那一份在位置条目 {location_index}：照类型段 0 那一支往下走那一份，红的那一批：{verdicts:?}"
        );
        let only_the_type_zero_branch = verdicts
            .iter()
            .find(|(identifier, _)| *identifier == invariant_only_the_type_zero_branch_reddens)
            .map(|(_, found)| found.clone())
            .expect("清单里有");
        assert!(
            matches!(
                &only_the_type_zero_branch,
                InvariantVerdict::Violated(detail) if detail.contains(detail_only_the_type_zero_branch_writes)
            ),
            "码 2 那一份在位置条目 {location_index}：{invariant_only_the_type_zero_branch_reddens} 红在类型段 0 那一支往下走那一份上（说明含「{detail_only_the_type_zero_branch_writes}」）：{only_the_type_zero_branch:?}"
        );
    }
}

/// G1 墓碑那一形：三个单元的文件发完意图之后。类型段 0 那一支往下走那一份——层级 1 的节点下面不往下读、记走读失败（F-L0），I-7.2 红在那一句。
#[test]
fn a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device(
) {
    assert_a_type_zero_entry_whose_child_has_a_copy_verifying_as_a_node_goes_down_that_copy_on_either_device(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-entry-type-zero-container-copy-verifying-as-a-node-goes-down",
        ),
        TREE_KIND_TOMBSTONE,
        PACKED_RECORD_TYPE_TOMBSTONE,
        &[
            "I-2.1",
            "I-4.8",
            "I-7.2",
            "I-7.4",
            TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        ],
        "I-7.2",
        "的节点有类型段 0 的条目",
    );
}

/// G1 谱系那一形：回退到 A 之后。同墓碑那一形。
#[test]
fn a_lineage_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device(
) {
    assert_a_type_zero_entry_whose_child_has_a_copy_verifying_as_a_node_goes_down_that_copy_on_either_device(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-entry-type-zero-container-copy-verifying-as-a-node-goes-down",
        )
        .image,
        TREE_KIND_LINEAGE,
        PACKED_RECORD_TYPE_LINEAGE,
        &[
            "I-2.1",
            "I-4.8",
            "I-7.2",
            "I-7.4",
            LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        ],
        "I-7.2",
        "的节点有类型段 0 的条目",
    );
}

/// G1 inode 那一形：新池新建文件那一版。类型段 0 那一支往下走那一份——码 2 那一份当 inode 树的内部节点读（它就是 inode 树根，层级 1，
/// 不是父层级减一），I-1.1 红在「inode 树的内部节点」那一句，槽数按码 2 一槽登记（叶容器另一槽没人引用：I-3.1、I-3.11、I-5.1，攻方
/// `research/prompts/m3-batch30-code-r1-opus-output.md` 观察 O1 那几条）。
#[test]
fn an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device(
) {
    assert_a_type_zero_entry_whose_child_has_a_copy_verifying_as_a_node_goes_down_that_copy_on_either_device(
        &build_pool("known-bad-inode-entry-type-zero-leaf-copy-verifying-as-a-node-goes-down")
            .memory_pool(),
        TREE_KIND_INODE,
        PACKED_RECORD_TYPE_INODE,
        &[
            "I-1.1", "I-1.2", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.2", "I-7.4", "I-9.2",
        ],
        "I-1.1",
        "inode 树的内部节点",
    );
}

/// G2（判决 `research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M3 缺口 G2 那一行、第三节第 2 条）：新池新建文件那一版，
/// inode 树根第一条条目不改（类型段 2，说孩子是码 3），叶容器盘 1 那一份换成第一个文件的数据单元（码 1、32 KiB）原样字节、位置条目 1 的整单元校验和
/// 换成它的 CRC——按类型段那一类（码 3）的长度 32 KiB 校验和对得上、头里却是同长度的别类码 1；位置条目 0 的校验和翻一字节（盘 0 那一份自证不过）。
/// 没有一份头里是码 3 又自证得过：不走 F-W，I-9.2 红在「孩子头里的类标签是 DataUnit」那一句、不往下走，码 1 那一份不当叶容器判（I-1.3、I-1.7 不红）。
/// 只看整单元校验和、不看头里的类标签时（正推 S4），码 1 那一份算作类型段那一类自证过、照类型段那一支往下走，I-9.2 的说明写成往下走的那一句。
#[test]
fn an_inode_tree_entry_whose_only_leaf_copy_matching_at_the_container_length_is_a_data_unit_does_not_go_down_that_copy(
) {
    let mut image =
        build_pool("known-bad-inode-leaf-copy-matching-at-the-container-length-is-a-data-unit")
            .memory_pool();
    let child = child_pointer_of_an_entry_of_the_root_of_kind(&image, TREE_KIND_INODE, 0);
    let leaf_slot = common_pointer_head_corruption::the_slot_of(&child);
    let data_unit = common_pointer_head_corruption::unit_bytes_at(
        &image,
        singlefs_core::address::SlotNumber(DATA_UNIT),
        common_pointer_head_corruption::data_unit_bytes(),
    );
    assert_eq!(
        data_unit[UNIT_HEADER_CLASS_TAG_OFFSET], 1,
        "第一个文件的数据单元是码 1"
    );
    write(
        &mut image,
        child.locations[1].device.0,
        leaf_slot.0 * SLOT,
        &data_unit,
    );
    let checksum_of_the_data_unit = crc32_castagnoli(&data_unit);
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        set_the_whole_unit_checksum_in_a_child_location(entry, 1, checksum_of_the_data_unit);
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
    });
    assert_the_violation_names_the_child_class(
        &image,
        "I-9.2",
        PACKED_RECORD_TYPE_INODE,
        "DataUnit",
    );
    let verdicts = check_pool_image(&image);
    let violated = violated_invariant_set(&verdicts);
    for invariant_of_walking_that_copy_as_a_leaf in ["I-1.3", "I-1.7"] {
        assert!(
            !violated.contains(invariant_of_walking_that_copy_as_a_leaf),
            "码 1 那一份不往下走、不当叶容器判：{invariant_of_walking_that_copy_as_a_leaf} 不红：{verdicts:?}"
        );
    }
    assert!(
        matches!(
            verdict(&image, "I-9.2"),
            InvariantVerdict::Violated(detail) if !detail.contains("照类型段那一支往下走那一份")
        ),
        "I-9.2 红在「孩子是另一类、不往下走」那一句，不是 F-W 往下走那一句：{:?}",
        verdict(&image, "I-9.2")
    );
}

/// 码 3 打包记录单元头里记录宽（2 字节）的偏移（D18（块里携带什么信息） 已定项 11：记录数 69、记录宽 71）。
const PACKED_UNIT_HEADER_RECORD_WIDTH_OFFSET: usize = 71;

/// 旋钮下长到层级 2 的那张镜像里两块盘上都没写过的一段（32 KiB）：远在单元区写过的那几槽之后、4 GiB 的盘之内（写之前核全零）。
const SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE: u64 = 180_000;

/// G3（判决 `research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M3 缺口 G3 那一行、第三节第 3 条）：层级 2 的树里类型段 0、
/// F-W 往下走那一份子树。墓碑树在旋钮下长到层级 2（[`image_with_a_tombstone_tree_of_level_two`]），根最后一条条目（类型段 0）指层级 1 的节点 N，
/// N 最后一条条目指墓碑容器 C（旋钮一片只装一条墓碑，最后一次 unlink 的墓碑另起一片：C 与 N 只有最新那几版引用）。C 两份的记录宽 40 → 39
/// （重封、补到根）；再把 C 的 32 KiB 原样字节写进位置条目 `location_index` 那块盘上一段没写过的槽 [`SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE`]、
/// 根那条条目的那条位置条目改指它、校验和换成它的 CRC（那一份自证成码 3，另一类；不写进 N 自己的槽：N 是 16 KiB，写 32 KiB 会盖掉下一槽的单元），
/// 另一条位置条目指的 N 不动（码 2，类型段 0 那一类，自证得过）。F-W：I-9.23 红，照类型段 0 那一支往下读 N 那一份，N 下面 C 的 I-1.7 违例照红。
/// 类型段 0 那一处见别类就停（正推 S1）时 N 不往下走，I-1.7 不红。
fn assert_a_wrong_record_width_below_a_level_one_node_with_a_copy_verifying_as_a_container_reddens_the_packed_container_invariant(
    location_index: usize,
) {
    let mut image = image_with_a_tombstone_tree_of_level_two(&format!(
        "known-bad-tombstone-level-two-node-copy-verifying-as-a-container-location-{location_index}"
    ));
    let root_pointer =
        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_TOMBSTONE).root;
    let root =
        singlefs_core::unit::parse_index_node(&common_pointer_head_corruption::unit_bytes_at(
            &image,
            common_pointer_head_corruption::the_slot_of(&root_pointer),
            common_pointer_head_corruption::node_bytes(),
        ))
        .expect("根解得开");
    assert_eq!(root.level, 2, "墓碑树根长到层级 2");
    let last_entry_index = root.entries.len() - 1;
    let node_pointer = child_pointer_of_an_entry_of_the_root_of_kind(
        &image,
        TREE_KIND_TOMBSTONE,
        last_entry_index,
    );
    let node_slot = common_pointer_head_corruption::the_slot_of(&node_pointer);
    let node =
        singlefs_core::unit::parse_index_node(&common_pointer_head_corruption::unit_bytes_at(
            &image,
            node_slot,
            common_pointer_head_corruption::node_bytes(),
        ))
        .expect("层级 1 的节点解得开");
    assert_eq!(node.level, 1, "根最后一条条目指层级 1 的节点");
    let container_pointer =
        singlefs_core::pointer::NodePointer::read_from(&mut singlefs_core::bytes::ByteReader::at(
            node.entries.last().expect("层级 1 的节点至少一条条目"),
            CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
        ));
    let container_slot = common_pointer_head_corruption::the_slot_of(&container_pointer);
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        &mut image,
        container_slot,
        common_pointer_head_corruption::data_unit_bytes(),
        |container| {
            assert_eq!(
                container[UNIT_HEADER_CLASS_TAG_OFFSET], 3,
                "N 下面最后一个孩子是码 3 墓碑容器"
            );
            let width_field =
                PACKED_UNIT_HEADER_RECORD_WIDTH_OFFSET..PACKED_UNIT_HEADER_RECORD_WIDTH_OFFSET + 2;
            assert_eq!(
                u64::from(u16::from_le_bytes([
                    container[width_field.start],
                    container[width_field.start + 1]
                ])),
                singlefs_format::TOMBSTONE_RECORD_BYTES,
                "墓碑容器的记录宽是登记的 40"
            );
            container[width_field].copy_from_slice(
                &record_width_one_below(singlefs_format::TOMBSTONE_RECORD_BYTES).to_le_bytes(),
            );
        },
    );
    let container = common_pointer_head_corruption::unit_bytes_at(
        &image,
        container_slot,
        common_pointer_head_corruption::data_unit_bytes(),
    );
    let device_of_the_container_copy = node_pointer.locations[location_index].device.0;
    assert!(
        read(
            &image,
            device_of_the_container_copy,
            SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE * SLOT,
            common_pointer_head_corruption::data_unit_bytes(),
        )
        .iter()
        .all(|byte| *byte == 0),
        "槽 {SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE} 起 32 KiB 在盘 {device_of_the_container_copy} 上没写过"
    );
    write(
        &mut image,
        device_of_the_container_copy,
        SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE * SLOT,
        &container,
    );
    let checksum_of_the_container = crc32_castagnoli(&container);
    let level = change_an_entry_of_the_root_of_kind(
        &mut image,
        TREE_KIND_TOMBSTONE,
        last_entry_index,
        |entry| {
            assert_eq!(
                entry_type_of(entry),
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
                "层级 2 的根条目类型段 0"
            );
            set_the_slot_in_a_child_location(
                entry,
                location_index,
                SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE,
            );
            set_the_whole_unit_checksum_in_a_child_location(
                entry,
                location_index,
                checksum_of_the_container,
            );
        },
    );
    assert_eq!(level, 2, "墓碑树根长到层级 2");
    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
    assert!(
        matches!(
            verdict_under_the_tombstone_knob(&image, "I-1.7"),
            InvariantVerdict::Violated(detail)
                if detail.contains(&format!("记录宽 {}", singlefs_format::TOMBSTONE_RECORD_BYTES - 1))
        ),
        "码 3 那一份在位置条目 {location_index}：照类型段 0 那一支往下读 N 那一份，N 下面那片墓碑容器的 I-1.7 违例照红：{verdicts:?}"
    );
    assert!(
        violated_invariant_set(&verdicts).contains(TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER),
        "码 3 那一份在位置条目 {location_index}：根那条条目照判 I-9.23 红：{verdicts:?}"
    );
}

#[test]
fn a_tombstone_container_below_a_level_one_node_whose_first_copy_verifies_as_a_container_reddens_the_packed_container_invariant(
) {
    assert_a_wrong_record_width_below_a_level_one_node_with_a_copy_verifying_as_a_container_reddens_the_packed_container_invariant(0);
}

#[test]
fn a_tombstone_container_below_a_level_one_node_whose_second_copy_verifies_as_a_container_reddens_the_packed_container_invariant(
) {
    assert_a_wrong_record_width_below_a_level_one_node_with_a_copy_verifying_as_a_container_reddens_the_packed_container_invariant(1);
}

// ─── 第 33 批（删除线）：一个单元的各把键按读回那一份的落点认（allrbmap）、F-ID-b 两份都没自证过、几处 F-KEY 与 G4 补的坏镜像 ───

/// allrbmap 的判法（判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第二节 K1 / K2 P1 没改的几把键那一行、第三节第 1 条；攻方
/// `research/prompts/m3-batch32-code-r1-opus-output.md` 第 1.2、1.3 节 today 臂换盘判得不同的那 6 对）：种类 `kind` 那棵树根第一条条目子指针的一条
/// 位置条目指着这一遍先走过、又被更早的候选根共享的码 2 节点（种类 `kind_walked_before` 那棵树的根）、整单元校验和对不上，另一条位置条目那一份是
/// 按自己的位置条目自证得过的容器（记录宽照 `record_width`）。两种盘序（指着先走过的节点的是位置条目 0 / 1）各判一次：红的都是 `expected_violations`，
/// I-4.9 与 I-3.5 都成立——读回那片容器的诞生代号记在它自己读回的落点上、不盖掉先走过那个节点的，I-3.5 的折叠表把那个落点记成它自己
/// （改之前位置条目 0 那一形多红 I-4.9，只改诞生代号那张表时谱系那几形改红 I-3.5）。
fn assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding(
    clean: &MemoryPool,
    kind: u16,
    record_width: RecordWidthOfTheFirstContainer,
    kind_walked_before: u16,
    expected_violations: &[&'static str],
) {
    for location_index in 0..2 {
        let image = image_whose_first_root_entry_names_a_walked_node_beside_a_container(
            clean,
            kind,
            record_width,
            kind_walked_before,
            location_index,
        );
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariant_set(&verdicts),
            expected_violations
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<&'static str>>(),
            "指着先走过的节点的是位置条目 {location_index}：两种盘序红的同一批：{verdicts:?}"
        );
        for invariant_judged_by_the_placement in [
            UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
            REFERENCE_INTERVALS_ARE_EXACT,
        ] {
            let found = verdicts
                .iter()
                .find(|(identifier, _)| *identifier == invariant_judged_by_the_placement)
                .map(|(_, found)| found.clone())
                .expect("清单里有");
            assert_eq!(
                found,
                InvariantVerdict::Holds,
                "指着先走过的节点的是位置条目 {location_index}：{invariant_judged_by_the_placement} 按落点认、读落点上单元头里的诞生代号，成立"
            );
        }
    }
}

/// 墓碑撞 extent 树根、容器记录宽 40 → 39（攻方 `base=tomb walked=extent variant=bad`）：三个单元的文件发完意图之后，extent 树根被更早那条候选根共享。
#[test]
fn a_tombstone_container_with_a_wrong_record_width_whose_location_names_the_shared_extent_root_judges_both_device_orders_alike(
) {
    assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-record-width-location-naming-the-shared-extent-root",
        ),
        TREE_KIND_TOMBSTONE,
        RecordWidthOfTheFirstContainer::OneBelowTheRegistered {
            record_bytes: singlefs_format::TOMBSTONE_RECORD_BYTES,
        },
        TREE_KIND_EXTENT,
        &[
            "I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4",
        ],
    );
}

/// 墓碑撞 extent 树根、容器不改（攻方 `base=tomb walked=extent variant=good`）。
#[test]
fn a_tombstone_container_whose_location_names_the_shared_extent_root_judges_both_device_orders_alike(
) {
    assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-location-naming-the-shared-extent-root",
        ),
        TREE_KIND_TOMBSTONE,
        RecordWidthOfTheFirstContainer::AsRegistered,
        TREE_KIND_EXTENT,
        &["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4"],
    );
}

/// 谱系撞 inode 树根、容器不改（攻方 `base=lineage walked=inode variant=good`）：回退到 A 之后，回退那一版照抄 A 的 inode 树。
/// 只改诞生代号那张表、不改 I-3.5 折叠表时这一形改红 I-3.5（攻方第 1.3 节）。记录宽 24 → 23 那一形见
/// [`a_lineage_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before`]。
#[test]
fn a_lineage_container_whose_location_names_the_shared_inode_root_judges_both_device_orders_alike()
{
    assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-location-naming-the-shared-inode-root",
        )
        .image,
        TREE_KIND_LINEAGE,
        RecordWidthOfTheFirstContainer::AsRegistered,
        TREE_KIND_INODE,
        &["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4"],
    );
}

/// 谱系撞 extent 树根、容器记录宽 24 → 23（攻方 `base=lineage walked=extent variant=bad`）。
#[test]
fn a_lineage_container_with_a_wrong_record_width_whose_location_names_the_shared_extent_root_judges_both_device_orders_alike(
) {
    assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-record-width-location-naming-the-shared-extent-root",
        )
        .image,
        TREE_KIND_LINEAGE,
        RecordWidthOfTheFirstContainer::OneBelowTheRegistered {
            record_bytes: singlefs_format::LINEAGE_RECORD_BYTES,
        },
        TREE_KIND_EXTENT,
        &[
            "I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4",
        ],
    );
}

/// 谱系撞 extent 树根、容器不改（攻方 `base=lineage walked=extent variant=good`）。
#[test]
fn a_lineage_container_whose_location_names_the_shared_extent_root_judges_both_device_orders_alike()
{
    assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-location-naming-the-shared-extent-root",
        )
        .image,
        TREE_KIND_LINEAGE,
        RecordWidthOfTheFirstContainer::AsRegistered,
        TREE_KIND_EXTENT,
        &["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4"],
    );
}

/// 回退那一版（D）的 inode 树根：照抄回退目标 A 的那一个（两版经同一个树根共享叶容器），或原样另写一份到没写过的槽、D 的树表改指它
/// （两版各走自己的树根，叶容器由两个树根直接共享）。
#[derive(Clone, Copy, Debug)]
enum InodeRootOfTheRollbackPublish {
    SharedWithTheRollbackTarget,
    CopiedToASlotNothingWrote,
}

/// 两块盘上自己与前后各一槽都没写过的一个槽：单元区里写过的最后一槽再往后 16 槽。
fn a_slot_after_every_written_slot_of_the_unit_area(image: &MemoryPool) -> u64 {
    let last_written_slot = DEVICES
        .iter()
        .filter_map(|device| {
            image
                .devices
                .get(&DeviceIdentity(*device))
                .expect("盘")
                .written_sectors()
                .keys()
                .map(|sector| sector * 512 / SLOT)
                .filter(|slot| *slot >= UNIT_AREA_START_SLOT_OF_THE_POOL)
                .max()
        })
        .max()
        .expect("单元区写过");
    last_written_slot + 16
}

/// 分叉形（攻方 `research/prompts/m3-batch32-code-r1-opus-output.md` 第 1.4 节，`raise_birth=true corrupt_loc0=true`；第 32 批实现员报告停下第 1 条
/// 第二段推的那一形）：回退到 A（txg 3）之后的镜像，D（txg 9）照抄 A 的 inode 树。inode 树根第一条条目指的叶容器 L 头里的诞生代号抬到 D 的 txg
/// （重封、沿引用链补到根：A、D 两版照旧引用同一个 L），再把 L 在位置条目 `garbled_location_index` 那块盘上的那一份写成别的字节
/// （不补校验和，那个槽上没有别的单元）。`inode_root` 定两版经同一个树根共享 L，还是各走自己的树根、直接共享 L。
fn image_after_the_rollback_whose_shared_leaf_container_is_born_after_the_target_with_one_copy_garbled(
    rolled: &ImageAfterTheRollbackToTheFirstRoot,
    garbled_location_index: usize,
    inode_root: InodeRootOfTheRollbackPublish,
) -> MemoryPool {
    let mut image = rolled.image.clone();
    let leaf = child_pointer_of_an_entry_of_the_root_of_kind(&image, TREE_KIND_INODE, 0);
    let leaf_slot = common_pointer_head_corruption::the_slot_of(&leaf);
    let data_unit_bytes = common_pointer_head_corruption::data_unit_bytes();
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        &mut image,
        leaf_slot,
        data_unit_bytes,
        |bytes| {
            let birth_txg = u64::from_le_bytes(bytes[73..81].try_into().expect("8 字节"));
            assert!(
                birth_txg <= rolled.rollback_target_txg,
                "L 的诞生代号 {birth_txg} 本来不晚于 A"
            );
            bytes[73..81].copy_from_slice(&rolled.rollback_publish_txg.to_le_bytes());
        },
    );
    match inode_root {
        InodeRootOfTheRollbackPublish::SharedWithTheRollbackTarget => {}
        InodeRootOfTheRollbackPublish::CopiedToASlotNothingWrote => {
            let inode_root_pointer =
                common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_INODE)
                    .root;
            let inode_root_bytes = common_pointer_head_corruption::unit_bytes_at(
                &image,
                common_pointer_head_corruption::the_slot_of(&inode_root_pointer),
                common_pointer_head_corruption::node_bytes(),
            );
            let copy_slot = a_slot_after_every_written_slot_of_the_unit_area(&image);
            for device in DEVICES {
                write(&mut image, device, copy_slot * SLOT, &inode_root_bytes);
            }
            let pointer_bytes =
                common_pointer_head_corruption::node_pointer_bytes(&inode_root_pointer);
            common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
                &mut image,
                singlefs_core::address::SlotNumber(rolled.tree_table_slot_of_the_rollback_publish),
                common_pointer_head_corruption::node_bytes(),
                |table| {
                    let starts: Vec<usize> = (0..=table.len() - pointer_bytes.len())
                        .filter(|start| {
                            table[*start..*start + pointer_bytes.len()] == pointer_bytes[..]
                        })
                        .collect();
                    assert_eq!(starts.len(), 1, "D 的树表里 inode 树根指针恰好一处");
                    for location_index in 0..2 {
                        let slot_at = starts[0]
                            + POINTER_FIRST_LOCATION_OFFSET
                            + location_index * POINTER_LOCATION_BYTES
                            + LOCATION_DEVICE_BYTES;
                        table[slot_at..slot_at + LOCATION_SLOT_BYTES]
                            .copy_from_slice(&copy_slot.to_le_bytes()[..LOCATION_SLOT_BYTES]);
                    }
                },
            );
        }
    }
    write(
        &mut image,
        leaf.locations[garbled_location_index].device.0,
        leaf_slot.0 * SLOT,
        &vec![0xa5u8; data_unit_bytes],
    );
    image
}

/// 分叉形的判法：L 坏的那一份在位置条目 0 / 1 两种盘序各判一次，红的都是 `expected_violations`，I-4.9（A 引用 L、L 读得出的那一份诞生代号 9 > 3）
/// 与 I-3.5（D 引用、C 不引用的 inode 树根那一格：子树里 L 读得出、诞生代号 9 大于树根指针的出生 txg）都红。坏的是位置条目 0 那一份时，
/// 诞生代号与 I-3.5 的折叠表都按读回的盘 1 那一份的落点记（攻方 allrbmap），GPU 判器同改（对拍流
/// `the_gpu_judge_agrees_with_the_cpu_when_a_shared_leaf_container_born_after_the_rollback_target_reads_back_from_its_second_copy`）。
fn assert_the_shared_leaf_container_born_after_the_rollback_target_reddens_alike_whichever_copy_is_garbled(
    tag: &str,
    inode_root: InodeRootOfTheRollbackPublish,
    expected_violations: &[&'static str],
) {
    let rolled = image_after_rolling_back_to_the_first_root(tag);
    for garbled_location_index in 0..2 {
        let image = image_after_the_rollback_whose_shared_leaf_container_is_born_after_the_target_with_one_copy_garbled(
            &rolled,
            garbled_location_index,
            inode_root,
        );
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariant_set(&verdicts),
            expected_violations
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<&'static str>>(),
            "L 坏的是位置条目 {garbled_location_index} 那一份：两种盘序红的同一批（I-4.9、I-3.5 都在里面）：{verdicts:?}"
        );
    }
}

/// 分叉形、两版经同一个 inode 树根共享叶容器（攻方 `direct=false`）：A 走到树根时「走过」命中，按树根下面记下的落点并进来。
#[test]
fn a_shared_leaf_container_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device(
) {
    assert_the_shared_leaf_container_born_after_the_rollback_target_reddens_alike_whichever_copy_is_garbled(
        "known-bad-shared-leaf-container-born-after-the-target-garbled-copy",
        InodeRootOfTheRollbackPublish::SharedWithTheRollbackTarget,
        &[
            "I-1.11",
            "I-1.2",
            "I-2.1",
            "I-3.10",
            REFERENCE_INTERVALS_ARE_EXACT,
            "I-4.2",
            "I-4.8",
            UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
            "I-7.4",
        ],
    );
}

/// 分叉形、两版各走自己的 inode 树根、直接共享叶容器（攻方 `direct=true`）：A 走到叶容器时「走过」命中，按叶容器读回那一份的落点并进来。
/// 树根另写一份的那两槽没有分配记录（I-3.1、I-3.11 跟着红）。
#[test]
fn a_leaf_container_directly_shared_and_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device(
) {
    assert_the_shared_leaf_container_born_after_the_rollback_target_reddens_alike_whichever_copy_is_garbled(
        "known-bad-directly-shared-leaf-container-born-after-the-target-garbled-copy",
        InodeRootOfTheRollbackPublish::CopiedToASlotNothingWrote,
        &[
            "I-1.11",
            "I-1.2",
            "I-2.1",
            "I-3.1",
            "I-3.10",
            "I-3.11",
            REFERENCE_INTERVALS_ARE_EXACT,
            "I-4.2",
            "I-4.8",
            UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
            "I-7.4",
        ],
    );
}

/// 类型段 0 那条条目孩子的一份副本：这几种都不是按自己那一类的长度自证过的（正推 `research/prompts/m3-batch32-code-r1-sonnet-output.md`
/// 第 2.2 节那张网格里两份都没自证过的那几种）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CopyThatDoesNotVerifyAtTheLengthOfItsClass {
    /// 头里是码 3、位置条目里的校验和只对得上前 16 KiB（`C16`）。
    ContainerCheckedAtSixteenKibibytes,
    /// 位置条目里的校验和翻几位（`BAD`）。
    ChecksumBitsFlipped,
    /// 位置条目里的盘不在池里（`UNREAD`）。
    DeviceOutsideThePool,
    /// 位置条目指着没写过的槽、校验和是 16 KiB 全零的 CRC（`ZERO`：读出全零，头里类标签 0 不登记）。
    SlotNothingWroteWithTheChecksumOfZeros,
}

const COPIES_THAT_DO_NOT_VERIFY_AT_THE_LENGTH_OF_THEIR_CLASS:
    [CopyThatDoesNotVerifyAtTheLengthOfItsClass; 4] = [
    CopyThatDoesNotVerifyAtTheLengthOfItsClass::ContainerCheckedAtSixteenKibibytes,
    CopyThatDoesNotVerifyAtTheLengthOfItsClass::ChecksumBitsFlipped,
    CopyThatDoesNotVerifyAtTheLengthOfItsClass::DeviceOutsideThePool,
    CopyThatDoesNotVerifyAtTheLengthOfItsClass::SlotNothingWroteWithTheChecksumOfZeros,
];

/// 回退到 A 之后的镜像与三个单元的文件发完意图之后的镜像里两块盘上都没写过的一槽（造之前核过全零）。
const SLOT_NOTHING_WROTE_IN_THE_DELETION_AND_ROLLBACK_IMAGES: u64 = 190_000;

/// 种类 `kind` 那棵树根（层级 1）第一条条目类型段 `registered_type` → 0、身份三段清零，孩子（码 3 容器）的两条位置条目照 `copies` 各改成一种
/// 自证不过的副本。
fn image_whose_type_zero_root_entry_names_a_child_with_two_copies_that_do_not_verify(
    clean: &MemoryPool,
    kind: u16,
    registered_type: u16,
    copies: [CopyThatDoesNotVerifyAtTheLengthOfItsClass; 2],
) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, kind, 0);
    let container = common_pointer_head_corruption::unit_bytes_at(
        clean,
        common_pointer_head_corruption::the_slot_of(&child),
        common_pointer_head_corruption::data_unit_bytes(),
    );
    let checksum_of_the_container = crc32_castagnoli(&container);
    let checksum_of_the_first_sixteen_kibibytes =
        crc32_castagnoli(&container[..common_pointer_head_corruption::node_bytes()]);
    let checksum_of_sixteen_zero_kibibytes =
        crc32_castagnoli(&vec![0u8; common_pointer_head_corruption::node_bytes()]);
    let mut image = clean.clone();
    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
        change_the_entry_type(
            entry,
            registered_type,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        zero_the_three_identity_fields(entry);
        for (location_index, copy) in copies.iter().enumerate() {
            match copy {
                CopyThatDoesNotVerifyAtTheLengthOfItsClass::ContainerCheckedAtSixteenKibibytes => {
                    set_the_whole_unit_checksum_in_a_child_location(
                        entry,
                        location_index,
                        checksum_of_the_first_sixteen_kibibytes,
                    );
                }
                CopyThatDoesNotVerifyAtTheLengthOfItsClass::ChecksumBitsFlipped => {
                    set_the_whole_unit_checksum_in_a_child_location(
                        entry,
                        location_index,
                        checksum_of_the_container ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY,
                    );
                }
                CopyThatDoesNotVerifyAtTheLengthOfItsClass::DeviceOutsideThePool => {
                    set_the_device_in_a_child_location(
                        entry,
                        location_index,
                        DEVICE_IDENTITY_NOT_IN_THE_POOL,
                    );
                }
                CopyThatDoesNotVerifyAtTheLengthOfItsClass::SlotNothingWroteWithTheChecksumOfZeros => {
                    set_the_slot_in_a_child_location(
                        entry,
                        location_index,
                        SLOT_NOTHING_WROTE_IN_THE_DELETION_AND_ROLLBACK_IMAGES,
                    );
                    set_the_whole_unit_checksum_in_a_child_location(
                        entry,
                        location_index,
                        checksum_of_sixteen_zero_kibibytes,
                    );
                }
            }
        }
    });
    assert_eq!(level, 1, "根层级 1、直接管容器");
    image
}

/// F-ID-b 两份都没自证过（用户 2026-10-07 定「当读不出、这一小句不判」，`records/2026-10-01-删除与并发事务.md` 同日那一条；判决
/// `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 3 条）：两条位置条目各取一种自证不过的副本、两两组合，身份那一条
/// （`identity_invariant`）一格都不红；互换两条位置条目，红的同一批（盘不在池里那一种放在位置条目 0 时位置条目的盘序不升序，I-2.5 那一条不比）。
/// 改之前取第一份按 16 KiB 对得上的头里的类标签：组合里有 `C16`（码 3）或 `ZERO`（码 0）的那 12 格判身份红（正推第 2.2 节第 3 点）。
fn assert_a_type_zero_entry_over_two_copies_that_do_not_verify_leaves_the_identity_unjudged_on_either_device(
    clean: &MemoryPool,
    kind: u16,
    registered_type: u16,
    identity_invariant: &'static str,
) {
    for device in DEVICES {
        assert!(
            read(
                clean,
                device,
                SLOT_NOTHING_WROTE_IN_THE_DELETION_AND_ROLLBACK_IMAGES * SLOT,
                common_pointer_head_corruption::data_unit_bytes(),
            )
            .iter()
            .all(|byte| *byte == 0),
            "槽 {SLOT_NOTHING_WROTE_IN_THE_DELETION_AND_ROLLBACK_IMAGES} 起 32 KiB 在盘 {device} 上没写过"
        );
    }
    let mut violated_of_each_pair = std::collections::BTreeMap::new();
    for first in COPIES_THAT_DO_NOT_VERIFY_AT_THE_LENGTH_OF_THEIR_CLASS {
        for second in COPIES_THAT_DO_NOT_VERIFY_AT_THE_LENGTH_OF_THEIR_CLASS {
            let image =
                image_whose_type_zero_root_entry_names_a_child_with_two_copies_that_do_not_verify(
                    clean,
                    kind,
                    registered_type,
                    [first, second],
                );
            let verdicts = check_pool_image(&image);
            let violated = violated_invariant_set(&verdicts);
            assert!(
                !violated.contains(identity_invariant),
                "位置条目 0 是 {first:?}、1 是 {second:?}：两份都没自证过，「子单元是码 2」这一小句不判，{identity_invariant} 不红：{verdicts:?}"
            );
            violated_of_each_pair.insert(
                (format!("{first:?}"), format!("{second:?}")),
                violated
                    .into_iter()
                    .filter(|invariant| *invariant != "I-2.5")
                    .collect::<std::collections::BTreeSet<&'static str>>(),
            );
        }
    }
    for ((first, second), violated) in &violated_of_each_pair {
        assert_eq!(
            Some(violated),
            violated_of_each_pair.get(&(second.clone(), first.clone())),
            "位置条目 0 是 {first}、1 是 {second}，与互换之后红的同一批（I-2.5 之外）"
        );
    }
}

#[test]
fn a_tombstone_root_entry_of_type_zero_over_two_copies_that_do_not_verify_leaves_the_tombstone_entry_invariant_holding_on_either_device(
) {
    assert_a_type_zero_entry_over_two_copies_that_do_not_verify_leaves_the_identity_unjudged_on_either_device(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-entry-type-zero-two-copies-that-do-not-verify",
        ),
        TREE_KIND_TOMBSTONE,
        PACKED_RECORD_TYPE_TOMBSTONE,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    );
}

#[test]
fn a_lineage_root_entry_of_type_zero_over_two_copies_that_do_not_verify_leaves_the_lineage_entry_invariant_holding_on_either_device(
) {
    assert_a_type_zero_entry_over_two_copies_that_do_not_verify_leaves_the_identity_unjudged_on_either_device(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-entry-type-zero-two-copies-that-do-not-verify",
        )
        .image,
        TREE_KIND_LINEAGE,
        PACKED_RECORD_TYPE_LINEAGE,
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    );
}

/// 一条位置条目的盘上原样：盘 4 + 槽 6 + 整单元校验和 4，小端。
fn bytes_of_a_location_entry(device: u32, slot: u64, checksum: u32) -> Vec<u8> {
    let mut bytes = device.to_le_bytes().to_vec();
    bytes.extend_from_slice(&slot.to_le_bytes()[..LOCATION_SLOT_BYTES]);
    bytes.extend_from_slice(&checksum.to_le_bytes());
    bytes
}

/// G4 那一读另造的一形（判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第二节 K1 P3 那一行、第三节第 4 条；正推变异 E1；
/// 第 32 批实现员报告停下第 4 条）：三个单元的文件发完意图之后，H-V 那一步（墓碑树根第一条条目指的容器在位置条目 `location_index` 那块盘上的
/// 一份换成墓碑树根的码 2 字节、那条位置条目的校验和换成它的 CRC），再把中央映射里那片容器那一条在同一块盘上的位置条目校验和换成那块盘上
/// 此刻 32 KiB 的 CRC（重封、补到根）：映射那一读对得上，I-2.1 只剩 F-W 往下走时按类型段那一类（码 3）的 32 KiB 读那一份那一读来红。
fn image_whose_container_copy_verifying_as_a_node_matches_its_central_mapping_entry(
    clean: &MemoryPool,
    location_index: usize,
) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, TREE_KIND_TOMBSTONE, 0);
    let container_slot = common_pointer_head_corruption::the_slot_of(&child);
    let checksum_of_the_container =
        crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
            clean,
            container_slot,
            common_pointer_head_corruption::data_unit_bytes(),
        ));
    let mut image = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
        clean,
        TREE_KIND_TOMBSTONE,
        location_index,
        &leave_the_entry_as_it_is,
    );
    let device = child.locations[location_index].device.0;
    let checksum_of_the_copy_now_on_the_device = crc32_castagnoli(&read(
        &image,
        device,
        container_slot.0 * SLOT,
        common_pointer_head_corruption::data_unit_bytes(),
    ));
    let mapping_root = common_pointer_head_corruption::newest_root(&image).mapping_root;
    let before = bytes_of_a_location_entry(device, container_slot.0, checksum_of_the_container);
    let after = bytes_of_a_location_entry(
        device,
        container_slot.0,
        checksum_of_the_copy_now_on_the_device,
    );
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        &mut image,
        common_pointer_head_corruption::the_slot_of(&mapping_root),
        common_pointer_head_corruption::node_bytes(),
        |bytes| {
            let starts: Vec<usize> = (0..=bytes.len() - before.len())
                .filter(|start| bytes[*start..*start + before.len()] == before[..])
                .collect();
            assert_eq!(
                starts.len(),
                1,
                "中央映射树（根兼叶）里那片容器那一条在盘 {device} 上的位置条目恰好一处"
            );
            bytes[starts[0]..starts[0] + after.len()].copy_from_slice(&after);
        },
    );
    image
}

/// G4 那一读：两种盘序（码 2 那一份在位置条目 0 / 1）各判一次，I-2.1 红、第一处违例说的是墓碑树内部条目的子指针那一读（中央映射那一读对得上）。
/// 去掉 F-W 往下走时那一读的 I-2.1（正推变异 E1）这一条红。
#[test]
fn a_tombstone_container_copy_verifying_as_a_node_whose_mapping_entry_matches_it_reddens_the_checksum_invariant_on_either_device(
) {
    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-container-copy-verifying-as-a-node-matching-its-mapping-entry",
    );
    for location_index in 0..2 {
        let image =
            image_whose_container_copy_verifying_as_a_node_matches_its_central_mapping_entry(
                &clean,
                location_index,
            );
        let verdicts = check_pool_image(&image);
        assert!(
            matches!(
                verdict(&image, "I-2.1"),
                InvariantVerdict::Violated(detail) if detail.contains("墓碑树内部条目的子指针")
            ),
            "码 2 那一份在位置条目 {location_index}：F-W 往下走时按码 3 的 32 KiB 读回那一份对不上，I-2.1 红在这一读：{verdicts:?}"
        );
    }
}

/// F-KEY 数据单元那一处（正推变异 E2）：新池新建文件那一版，第一个文件的数据单元头里五元组的对象号加 1（重封、补到根，I-1.1 违例），
/// 再把 extent 树根（上段叶）里内联它的那条数据指针第 `location_index` 条位置条目的槽改成 extent 树根自己的槽（这一遍刚走过）、
/// 校验和写成它那 16 KiB 的 CRC 翻几位（这一份对不上）。另一条位置条目那一份就是带违例的数据单元。
fn image_whose_inline_data_pointer_names_the_walked_extent_root_beside_a_data_unit_with_another_object(
    clean: &MemoryPool,
    location_index: usize,
) -> MemoryPool {
    const DATA_UNIT_OBJECT_OFFSET: usize = 51;
    let mut image = clean.clone();
    let singlefs_core::extent_tree::ExtentUpperLeafTarget::InlineDataUnit(data_pointer) =
        common_pointer_head_corruption::first_upper_leaf_entry(&image).target
    else {
        panic!("新池新建文件那一版只有一个数据单元，上段叶第一条内联它的数据指针");
    };
    assert_eq!(
        data_pointer.locations[0].slot.0, DATA_UNIT,
        "新池新建文件那一版唯一的数据单元在登记过的那一槽"
    );
    // 码 1 的重封：`mutate_unit` 认三种码（`common_deletion_in_progress` 那一份只认码 2 与码 3）。
    mutate_unit(
        &mut image,
        DATA_UNIT,
        |bytes| {
            let object = get_u64(bytes, DATA_UNIT_OBJECT_OFFSET);
            set_u64(bytes, DATA_UNIT_OBJECT_OFFSET, object + 1);
        },
        true,
    );
    let singlefs_core::extent_tree::ExtentUpperLeafTarget::InlineDataUnit(resealed_pointer) =
        common_pointer_head_corruption::first_upper_leaf_entry(&image).target
    else {
        panic!("改了数据单元之后上段叶第一条照旧内联它的数据指针");
    };
    let extent_root =
        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root;
    let extent_root_slot = common_pointer_head_corruption::the_slot_of(&extent_root);
    let before = common_pointer_head_corruption::data_pointer_bytes(&resealed_pointer);
    let mut pointer_naming_the_extent_root = resealed_pointer;
    pointer_naming_the_extent_root.locations[location_index].slot = extent_root_slot;
    pointer_naming_the_extent_root.locations[location_index].unit_checksum =
        crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
            &image,
            extent_root_slot,
            common_pointer_head_corruption::node_bytes(),
        )) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
    let after = common_pointer_head_corruption::data_pointer_bytes(&pointer_naming_the_extent_root);
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        &mut image,
        extent_root_slot,
        common_pointer_head_corruption::node_bytes(),
        |bytes| {
            let starts: Vec<usize> = (0..=bytes.len() - before.len())
                .filter(|start| bytes[*start..*start + before.len()] == before[..])
                .collect();
            assert_eq!(starts.len(), 1, "上段叶里那条数据指针恰好一处");
            bytes[starts[0]..starts[0] + after.len()].copy_from_slice(&after);
        },
    );
    image
}

/// F-KEY 数据单元那一处：两种盘序（指着先走过的 extent 树根的是位置条目 0 / 1）各判一次，读回的那份数据单元照判，I-1.1 红在它的五元组、
/// 红的同一批。「走过」按第 0 条位置条目认的话，位置条目 0 那一形把读回来的数据单元当成走过（它的五元组一个字节都不判）。
#[test]
fn a_data_unit_with_another_object_whose_location_names_the_walked_extent_root_reddens_its_five_tuple_on_either_device(
) {
    let clean = build_pool("known-bad-data-unit-object-location-naming-the-walked-extent-root")
        .memory_pool();
    let mut violated_on_each_location = Vec::new();
    for location_index in 0..2 {
        let image =
            image_whose_inline_data_pointer_names_the_walked_extent_root_beside_a_data_unit_with_another_object(
                &clean,
                location_index,
            );
        let verdicts = check_pool_image(&image);
        assert!(
            matches!(
                verdict(&image, "I-1.1"),
                InvariantVerdict::Violated(detail) if detail.contains("数据单元五元组")
            ),
            "指着先走过的节点的是位置条目 {location_index}：读回来的那份数据单元照判，I-1.1 红在它的五元组：{verdicts:?}"
        );
        violated_on_each_location.push(violated_invariant_set(&verdicts));
    }
    assert_eq!(
        violated_on_each_location[0], violated_on_each_location[1],
        "同一张镜像只差指着先走过的节点的是哪一条位置条目：红的同一批"
    );
}

/// F-KEY inode 内部孩子那一处（正推变异 E4）：新池新建文件那一版，inode 树根第一条条目类型段 2 → 0、身份三段清零；位置条目 `location_index`
/// 的槽改成 extent 树根的槽（这一遍先走过）、校验和写成它那 16 KiB 的 CRC 翻几位；另一条位置条目那块盘上叶容器的槽换成 inode 树根的码 2 字节、
/// 那条位置条目的校验和换成它的 CRC（按 16 KiB 自证得过）。读回的就是那份码 2，当 inode 树的内部节点读（层级 1，不是父层级减一）。
fn image_whose_type_zero_inode_entry_names_the_walked_extent_root_beside_a_copy_verifying_as_a_node(
    clean: &MemoryPool,
    location_index: usize,
) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, TREE_KIND_INODE, 0);
    let inode_root_bytes = root_node_bytes_of_the_tree_of_kind(clean, TREE_KIND_INODE);
    let extent_root =
        common_pointer_head_corruption::tree_table_entry_of_kind(clean, TREE_KIND_EXTENT).root;
    let extent_root_slot = common_pointer_head_corruption::the_slot_of(&extent_root);
    let wrong_checksum_of_the_extent_root =
        crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
            clean,
            extent_root_slot,
            common_pointer_head_corruption::node_bytes(),
        )) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
    let other_location_index = 1 - location_index;
    let mut image = clean.clone();
    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        zero_the_three_identity_fields(entry);
        set_the_slot_in_a_child_location(entry, location_index, extent_root_slot.0);
        set_the_whole_unit_checksum_in_a_child_location(
            entry,
            location_index,
            wrong_checksum_of_the_extent_root,
        );
        set_the_whole_unit_checksum_in_a_child_location(
            entry,
            other_location_index,
            crc32_castagnoli(&inode_root_bytes),
        );
    });
    assert_eq!(level, 1, "新池新建文件那一版的 inode 树根层级 1");
    write(
        &mut image,
        child.locations[other_location_index].device.0,
        common_pointer_head_corruption::the_slot_of(&child).0 * SLOT,
        &inode_root_bytes,
    );
    image
}

/// F-KEY inode 内部孩子那一处：两种盘序各判一次，读回的那份码 2 照判，I-1.1 红在「inode 树的内部节点」那一句，红的同一批。
#[test]
fn a_type_zero_inode_entry_whose_location_names_the_walked_extent_root_walks_the_copy_verifying_as_a_node_on_either_device(
) {
    let clean =
        build_pool("known-bad-inode-entry-type-zero-location-naming-the-walked-extent-root")
            .memory_pool();
    let mut violated_on_each_location = Vec::new();
    for location_index in 0..2 {
        let image =
            image_whose_type_zero_inode_entry_names_the_walked_extent_root_beside_a_copy_verifying_as_a_node(
                &clean,
                location_index,
            );
        let verdicts = check_pool_image(&image);
        assert!(
            matches!(
                verdict(&image, "I-1.1"),
                InvariantVerdict::Violated(detail) if detail.contains("inode 树的内部节点")
            ),
            "指着先走过的节点的是位置条目 {location_index}：读回来的那份码 2 照判，I-1.1 红在「inode 树的内部节点」：{verdicts:?}"
        );
        violated_on_each_location.push(violated_invariant_set(&verdicts));
    }
    assert_eq!(
        violated_on_each_location[0], violated_on_each_location[1],
        "同一张镜像只差指着先走过的节点的是哪一条位置条目：红的同一批"
    );
}

/// `read_index_node` 那一处（判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 1 条，第 32 批实现员报告停下第 2 条）：
/// 新池新建文件那一版，inode 叶容器第一条记录的预留段写一个非零字节（两份、重封、补到根，I-9.7 违例）；再把树表里 inode 树那条条目的根指针
/// 第 `location_index` 条位置条目的槽改成树表单元自己的槽（这一遍先走过）、校验和写成它那 16 KiB 的 CRC 翻几位（这一份对不上）。
/// 另一条位置条目那一份就是 inode 树根，读者用的就是它。
fn image_whose_inode_root_pointer_names_the_walked_tree_table_above_a_leaf_with_a_non_zero_reserved_byte(
    clean: &MemoryPool,
    location_index: usize,
) -> MemoryPool {
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        INODE_LEAF,
        |bytes| bytes[INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_LAST_RESERVED_BYTE_OFFSET] = 1,
        true,
    );
    let inode_root =
        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
    let tree_table = common_pointer_head_corruption::newest_root(&image).tree_table;
    let tree_table_slot = common_pointer_head_corruption::the_slot_of(&tree_table);
    let before = common_pointer_head_corruption::node_pointer_bytes(&inode_root);
    let mut pointer_naming_the_tree_table = inode_root;
    pointer_naming_the_tree_table.locations[location_index].slot = tree_table_slot;
    pointer_naming_the_tree_table.locations[location_index].unit_checksum =
        crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
            &image,
            tree_table_slot,
            common_pointer_head_corruption::node_bytes(),
        )) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
    let after = common_pointer_head_corruption::node_pointer_bytes(&pointer_naming_the_tree_table);
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        &mut image,
        tree_table_slot,
        common_pointer_head_corruption::node_bytes(),
        |bytes| {
            let starts: Vec<usize> = (0..=bytes.len() - before.len())
                .filter(|start| bytes[*start..*start + before.len()] == before[..])
                .collect();
            assert_eq!(starts.len(), 1, "树表里 inode 树根指针恰好一处");
            bytes[starts[0]..starts[0] + after.len()].copy_from_slice(&after);
        },
    );
    image
}

/// `read_index_node` 那一处：两种盘序（指着先走过的树表单元的是位置条目 0 / 1）各判一次，读回的那份 inode 树根照走，它下面那片叶容器的
/// I-9.7 违例照红，红的同一批。「走过」按第 0 条位置条目认的话，位置条目 0 那一形把 inode 树根当成别的版本先走过（并进树表下面的落点、不往下走），
/// 叶容器一个字节都不判。
#[test]
fn an_inode_leaf_with_a_non_zero_reserved_byte_below_a_root_pointer_naming_the_walked_tree_table_reddens_the_reserved_bytes_invariant_on_either_device(
) {
    let clean = build_pool(
        "known-bad-inode-leaf-reserved-byte-below-a-root-pointer-naming-the-walked-tree-table",
    )
    .memory_pool();
    let mut violated_on_each_location = Vec::new();
    for location_index in 0..2 {
        let image =
            image_whose_inode_root_pointer_names_the_walked_tree_table_above_a_leaf_with_a_non_zero_reserved_byte(
                &clean,
                location_index,
            );
        let verdicts = check_pool_image(&image);
        let violated = violated_invariant_set(&verdicts);
        assert!(
            violated.contains("I-9.7"),
            "指着先走过的树表单元的是位置条目 {location_index}：读回的那份 inode 树根照走，叶容器里的 I-9.7 违例照红：{verdicts:?}"
        );
        violated_on_each_location.push(violated);
    }
    assert_eq!(
        violated_on_each_location[0], violated_on_each_location[1],
        "同一张镜像只差指着先走过的节点的是哪一条位置条目：红的同一批"
    );
}

// ─── 第 36 批（删除线）：I-3.5 按全部版本走完之后的折叠表取单元（H-S1-fold）、两份都读不出而第 0 条落点早被别的版本走过 ───

/// 一版的树表单元的槽（第一版两盘同槽）。
fn tree_table_slot_of_the_version(
    version: &singlefs_core::transaction::TransactionOutput,
) -> singlefs_core::address::SlotNumber {
    common_pointer_head_corruption::the_slot_of(&version.root.tree_table)
}

/// 那一版的树表单元里种类 `kind` 那条条目的根指针在单元里的起点（整条 86 字节逐字找，恰好一处）；那条根指针全零时 `None`。
fn root_pointer_start_in_the_tree_table_of(
    image: &MemoryPool,
    version: &singlefs_core::transaction::TransactionOutput,
    kind: u16,
) -> Option<usize> {
    let tree_table = common_pointer_head_corruption::unit_bytes_at(
        image,
        tree_table_slot_of_the_version(version),
        common_pointer_head_corruption::node_bytes(),
    );
    let root_pointer = singlefs_core::unit::parse_index_node(&tree_table)
        .expect("树表解得开")
        .entries
        .iter()
        .filter_map(|entry| singlefs_core::records::TreeTableEntry::parse(entry))
        .find(|entry| entry.kind == kind)
        .expect("树表里有这种树")
        .root;
    let pointer_bytes = common_pointer_head_corruption::node_pointer_bytes(&root_pointer);
    if pointer_bytes.iter().all(|byte| *byte == 0) {
        return None;
    }
    let starts: Vec<usize> = (0..=tree_table.len() - pointer_bytes.len())
        .filter(|start| tree_table[*start..*start + pointer_bytes.len()] == pointer_bytes[..])
        .collect();
    let [start] = starts.as_slice() else {
        panic!("种类 {kind} 的根指针在树表里恰好一处：{starts:?}")
    };
    Some(*start)
}

/// 指针里第 0 条位置条目的起点、每条位置条目的宽、条目里整单元校验和的偏移（D19（块指针的结构与宽度预算） 已定项 7 的字段表）。
const NODE_POINTER_FIRST_LOCATION_OFFSET: usize = 50;
const NODE_POINTER_LOCATION_BYTES: usize = 14;
const NODE_POINTER_LOCATION_CHECKSUM_OFFSET: usize = 10;

/// 那一版树表里种类 `kind` 那条根指针第 `location_indexes` 这几条位置条目的整单元校验和各异或几位（指着的单元本身不动），重封那张树表、
/// 沿引用链补到根槽：这一版读那棵树的根时那几份对不上（I-2.1 红）、读回别的那一份。
fn garble_location_checksums_of_a_root_pointer_in_the_tree_table_of(
    image: &mut MemoryPool,
    version: &singlefs_core::transaction::TransactionOutput,
    kind: u16,
    location_indexes: &[usize],
) {
    let start = root_pointer_start_in_the_tree_table_of(image, version, kind)
        .expect("这一版那棵树的根指针不是全零");
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        tree_table_slot_of_the_version(version),
        common_pointer_head_corruption::node_bytes(),
        |tree_table| {
            for location_index in location_indexes {
                let checksum = start
                    + NODE_POINTER_FIRST_LOCATION_OFFSET
                    + location_index * NODE_POINTER_LOCATION_BYTES
                    + NODE_POINTER_LOCATION_CHECKSUM_OFFSET;
                for byte in &mut tree_table[checksum..checksum + 4] {
                    *byte ^= 0x5a;
                }
            }
        },
    );
}

/// 三个单元的文件之后发意图、删第一批：交回镜像与顺序写（txg 4）、意图（txg 5，extent 树照抄 txg 4 的、树表另写一份）两版。
fn image_with_the_written_and_the_intent_versions_after_the_first_batch(
    tag: &str,
) -> (
    MemoryPool,
    singlefs_core::transaction::TransactionOutput,
    singlefs_core::transaction::TransactionOutput,
) {
    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(tag);
    let written = pool.output.clone();
    let intent = common_deletion_in_progress::publish_the_intent(&mut pool);
    common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool);
    (pool.memory_pool(), written, intent)
}

fn verdict_of_the_invariant(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(identifier, _)| *identifier == invariant)
        .map(|(_, verdict)| verdict)
        .expect("清单里有")
}

/// H-S1-fold 的 A 臂（判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S1 行与第三节第 1 条，攻方
/// `research/prompts/m3-batch33s-code-r1-opus-output.md` 第一节 1.2）：意图那一版（txg 5，较新、根环次序在 txg 4 之后走）树表里 extent 树根指针
/// 第 `location_index` 条位置条目的校验和坏，读回另一份。池级 checker 判 I-3.5 时按全部版本走完之后的折叠表取每版的单元（攻方 F1）：
/// txg 4 那一版先走、经第 0 条读回的那个落点，在意图那一版读回第 1 条那一份、把它改记成自己之后同算一个单元，意图那一版不算新引用它。
/// 两种盘序（坏的是第 0 / 1 条）I-3.5 都成立、I-2.1 都红（改之前每版走完当场折，坏的是第 0 条时多红 I-3.5，与 GPU 走读内核的取法分叉）。
#[test]
fn an_intent_version_reading_the_shared_extent_root_from_either_copy_leaves_the_appearance_rule_holding(
) {
    let (clean, _written, intent) =
        image_with_the_written_and_the_intent_versions_after_the_first_batch(
            "known-bad-i35-intent-reads-the-shared-extent-root-from-either-copy",
        );
    for location_index in 0..2 {
        let mut image = clean.clone();
        garble_location_checksums_of_a_root_pointer_in_the_tree_table_of(
            &mut image,
            &intent,
            singlefs_core::records::TREE_KIND_EXTENT,
            &[location_index],
        );
        let verdicts = check_pool_image(&image);
        let set = violated_invariant_set(&verdicts);
        assert!(
            set.contains("I-2.1"),
            "坏的是第 {location_index} 条：那一份照判 I-2.1 红：{verdicts:?}"
        );
        assert_eq!(
            verdict_of_the_invariant(&image, REFERENCE_INTERVALS_ARE_EXACT),
            InvariantVerdict::Holds,
            "坏的是第 {location_index} 条：I-3.5 按全部版本走完之后的折叠表取单元，成立：{verdicts:?}"
        );
    }
}

/// 攻方 S1 扫描（同上第一节 1.3）的一种历史：`versions` 是按发布次序的各版（新池新建文件那一版也算），每一对相邻、树表各写一份的版本里，
/// 较新那一版树表里根指针与较旧那一版同种类的根指针逐字相同的每一棵树，各造一份坏镜像：较新那一版那条根指针第 0 条位置条目的校验和坏。
/// 每一份 I-3.5 都不红。交回造了几份。
fn assert_garbling_the_first_location_of_each_shared_root_pointer_of_the_newer_version_leaves_the_appearance_rule_unviolated(
    clean: &MemoryPool,
    versions: &[singlefs_core::transaction::TransactionOutput],
) -> usize {
    let mut images_judged = 0;
    for pair in versions.windows(2) {
        let (older, newer) = (&pair[0], &pair[1]);
        if tree_table_slot_of_the_version(older) == tree_table_slot_of_the_version(newer) {
            continue;
        }
        for entry in &newer.tree_table_entries {
            if root_pointer_start_in_the_tree_table_of(clean, newer, entry.kind).is_none() {
                continue;
            }
            let pointer_bytes = common_pointer_head_corruption::node_pointer_bytes(&entry.root);
            let shared_with_the_older_version =
                older.tree_table_entries.iter().any(|older_entry| {
                    older_entry.kind == entry.kind
                        && common_pointer_head_corruption::node_pointer_bytes(&older_entry.root)
                            == pointer_bytes
                });
            if !shared_with_the_older_version {
                continue;
            }
            let mut image = clean.clone();
            garble_location_checksums_of_a_root_pointer_in_the_tree_table_of(
                &mut image,
                newer,
                entry.kind,
                &[0],
            );
            let verdict = verdict_of_the_invariant(&image, REFERENCE_INTERVALS_ARE_EXACT);
            assert!(
                !matches!(verdict, InvariantVerdict::Violated(_)),
                "txg {} → {}、种类 {} 的根指针第 0 条校验和坏：I-3.5 不红（{verdict:?}）",
                older.root.checkpoint_txg.0,
                newer.root.checkpoint_txg.0,
                entry.kind
            );
            images_judged += 1;
        }
    }
    images_judged
}

/// 攻方 S1 扫描 `write3-intent-batch`：三个单元的文件、意图、第一批（3 份坏镜像：txg 4 → 5 的 extent 树，5 → 6 的墓碑树与 `deleted_inodes`）。
#[test]
fn garbling_each_shared_root_pointer_of_a_newer_version_through_an_intent_and_a_batch_leaves_the_appearance_rule_unviolated(
) {
    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(
        "known-bad-i35-scan-write-intent-batch",
    );
    let mut versions = vec![pool.output.clone()];
    versions.push(common_deletion_in_progress::publish_the_intent(&mut pool));
    versions.push(common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool));
    assert_eq!(
        assert_garbling_the_first_location_of_each_shared_root_pointer_of_the_newer_version_leaves_the_appearance_rule_unviolated(
            &pool.memory_pool(),
            &versions,
        ),
        3,
        "攻方这一种历史扫的格数"
    );
}

/// 攻方 S1 扫描 `write3-unlink-to-the-end`：三个单元的文件、意图、按批大小 1 删到队列空（6 份坏镜像）。
#[test]
fn garbling_each_shared_root_pointer_of_a_newer_version_through_an_unlink_to_the_end_leaves_the_appearance_rule_unviolated(
) {
    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(
        "known-bad-i35-scan-unlink-to-the-end",
    );
    let mut versions = vec![pool.output.clone()];
    versions.push(common_deletion_in_progress::publish_the_intent(&mut pool));
    while !pool.output.deleted_inodes.is_empty() {
        versions
            .push(common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool));
    }
    assert_eq!(
        assert_garbling_the_first_location_of_each_shared_root_pointer_of_the_newer_version_leaves_the_appearance_rule_unviolated(
            &pool.memory_pool(),
            &versions,
        ),
        6,
        "攻方这一种历史扫的格数"
    );
}

/// 攻方 S1 扫描 `write3-new-inodes-x3`：三个单元的文件之后建三次空 inode（3 份坏镜像，都是 extent 树）。
#[test]
fn garbling_each_shared_root_pointer_of_a_newer_version_through_three_publishes_of_new_inodes_leaves_the_appearance_rule_unviolated(
) {
    let mut pool =
        common_deletion_in_progress::pool_with_a_three_unit_file("known-bad-i35-scan-new-inodes");
    let mut versions = vec![pool.output.clone()];
    for _publish in 0..3 {
        versions.push(
            common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1),
        );
    }
    assert_eq!(
        assert_garbling_the_first_location_of_each_shared_root_pointer_of_the_newer_version_leaves_the_appearance_rule_unviolated(
            &pool.memory_pool(),
            &versions,
        ),
        3,
        "攻方这一种历史扫的格数"
    );
}

/// 正推 O1-1（`research/prompts/m3-batch33s-code-r1-sonnet-output.md` K1-S1 末段两处观察第一处；判决 `research/prompts/m3-batch33s-code-r1-main-verification.md`
/// 第三节第 1 条「正推第 4 条那一形补坏镜像」）：顺序写那一版（txg 4，根环次序在最新的意图那一版之后走）树表里 extent 树根指针两条位置条目的校验和都坏，
/// 第 0 条落点上那个单元（两版共享的 extent 树根）已被意图那一版先走过。「走过」按读回那一份的落点认：两份都读不出就没有读回的落点，
/// 记走读失败、两份照判 I-2.1，不当成「别的版本先走过」（改之前读之前先按第 0 条落点查走过表，这一形判成先走过、并进前一版、两份都不判、不记走读失败）。
#[test]
fn a_root_pointer_whose_two_copies_do_not_read_back_and_whose_first_location_names_a_unit_walked_before_records_a_walk_failure(
) {
    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(
        "known-bad-two-copies-unreadable-first-location-walked-before",
    );
    let written = pool.output.clone();
    let intent = common_deletion_in_progress::publish_the_intent(&mut pool);
    let shared_extent_root = |version: &singlefs_core::transaction::TransactionOutput| {
        version
            .tree_table_entries
            .iter()
            .find(|entry| entry.kind == singlefs_core::records::TREE_KIND_EXTENT)
            .expect("树表里有 extent 树")
            .root
    };
    assert_eq!(
        common_pointer_head_corruption::node_pointer_bytes(&shared_extent_root(&written)),
        common_pointer_head_corruption::node_pointer_bytes(&shared_extent_root(&intent)),
        "意图那一版 extent 树照抄顺序写那一版的"
    );
    let mut image = pool.memory_pool();
    garble_location_checksums_of_a_root_pointer_in_the_tree_table_of(
        &mut image,
        &written,
        singlefs_core::records::TREE_KIND_EXTENT,
        &[0, 1],
    );
    let verdicts = check_pool_image(&image);
    let set = violated_invariant_set(&verdicts);
    assert!(
        set.contains("I-2.1") && set.contains("I-4.8"),
        "两份照判 I-2.1 红，走读失败记下、遍历读不出红 I-4.8：{verdicts:?}"
    );
}

// ─── I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上，W3 的 C2aL 形态，第 49 批） ───

/// 谱系记录里 H_cur 那 8 字节的偏移（T_rb 8、T_old 8 之后）与一片打包记录单元的记录区起点（D18（块里携带什么信息） 已定项 16）。
const LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET: usize = 16;
const PACKED_UNIT_FIRST_RECORD_OFFSET: usize = 136;
/// 记账树叶（key 宽 22）里条目数的偏移、第一条条目的偏移、一条条目宽与值、代段、设备段的偏移，单元头里声明长度的偏移
/// （偏移口径同 `set_accounting`）。
const ACCOUNTING_LEAF_ENTRY_COUNT_OFFSET: usize = 82 + 44;
const ACCOUNTING_LEAF_FIRST_ENTRY_OFFSET: usize = 159;
const ACCOUNTING_ENTRY_WIDTH: usize = 34;
const ACCOUNTING_ENTRY_DEVICE_OFFSET: usize = 10;
const ACCOUNTING_ENTRY_GENERATION_OFFSET: usize = 14;
const ACCOUNTING_ENTRY_VALUE_OFFSET: usize = 22;
const UNIT_DECLARED_LENGTH_OFFSET_OF_AN_INDEX_NODE: usize = 8;
/// inode 号水位（D5（快照 / 空间记账机制） 已定项 4 第 12 项）不带设备维。
const INODE_NUMBER_WATERMARK_STATISTIC: u16 = 12;
const NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;

/// 合法镜像：崩溃恢复抛弃建过三个 inode（水位 5）的那一版、它的根读得出（`common::rollback_after_a_recovery_abandoned_a_root_that_created_inodes`），
/// 实例 2 再建一个（2 号重发，被抛弃那段里发过的号，D8（核心索引结构） 已定项 6 射程允许；水位 3），再回退到 A（谱系记录 H_cur 3）。
/// 一条违例都没有：I-9.30 真被判过且成立（台阶 (3, 2)、(重发那次, 3)；谱系上 T_rb − 1 以前最大的台阶 3 等于 H_cur；被抛弃那段的台阶随它的根留在
/// 盘上、不在最新根的记账里），I-3.13 真被判过且成立（台阶行代段早于这一版，跳过；每次发布写的那一行代段等于这一版的 txg）。
#[test]
fn a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_and_created_again_holds_the_watermark_step_and_generation_invariants(
) {
    let history = common::rollback_after_a_recovery_abandoned_a_root_that_created_inodes(
        "known-bad-watermark-steps-legal-cross-abandon-reissue",
    );
    let verdicts = check_pool_image(&history.pool.memory_pool());
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "合法镜像一条违例都没有：{verdicts:?}"
    );
    for invariant in [
        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
        ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
    ] {
        assert_eq!(
            verdicts
                .iter()
                .find(|(name, _)| *name == invariant)
                .map(|(_, found)| found),
            Some(&InvariantVerdict::Holds),
            "{invariant} 真被判过且成立"
        );
    }
}

/// [`a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_and_created_again_holds_the_watermark_step_and_generation_invariants`]
/// 那份合法镜像，最新根（回退那一版）谱系树第一片容器里第一条谱系记录的 H_cur 改成 `watermark_rolled_back_from`（重封、补到根）。
fn image_after_the_legal_cross_abandon_rollback_with_the_watermark_rolled_back_from(
    tag: &str,
    watermark_rolled_back_from: u64,
) -> MemoryPool {
    let history = common::rollback_after_a_recovery_abandoned_a_root_that_created_inodes(tag);
    let mut image = history.pool.memory_pool();
    let lineage_container =
        common_deletion_in_progress::first_container_slot_of_the_tree_of_kind_of_the_newest_root(
            &image,
            TREE_KIND_LINEAGE,
        );
    let offset = PACKED_UNIT_FIRST_RECORD_OFFSET
        + LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET;
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        &mut image,
        lineage_container,
        32_768,
        |bytes| {
            assert_eq!(
                u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8")),
                3,
                "回退那一版谱系记录的 H_cur 是 cur 的水位 3"
            );
            bytes[offset..offset + 8].copy_from_slice(&watermark_rolled_back_from.to_le_bytes());
        },
    );
    image
}

/// I-9.30 ④（谱系那一判按「等于」比，W3 判决 `research/prompts/m3-w3-r2-main-verification.md` 第二节攻方 K3）：合法镜像上谱系记录的 H_cur 3 → 2。
/// T_rb − 1 以前最大的台阶是 3，H_cur 小于它——按「被台阶盖住」比（max ≥ H_cur）放过这一形，按「等于」比判红。只红 I-9.30。
#[test]
fn a_lineage_record_whose_watermark_rolled_back_from_is_below_the_last_step_reddens_only_the_watermark_step_invariant(
) {
    let image = image_after_the_legal_cross_abandon_rollback_with_the_watermark_rolled_back_from(
        "known-bad-watermark-steps-lineage-watermark-below-the-step",
        2,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE],
        "谱系记录的 H_cur 比台阶小：只红 I-9.30：{verdicts:?}"
    );
}

/// I-9.30 ④ 的「H_cur 为 0 也判红」（D18（块里携带什么信息） 已定项 24 第 5 条）：合法镜像上谱系记录的 H_cur 3 → 0。只红 I-9.30。
#[test]
fn a_lineage_record_whose_watermark_rolled_back_from_is_zero_reddens_only_the_watermark_step_invariant(
) {
    let image = image_after_the_legal_cross_abandon_rollback_with_the_watermark_rolled_back_from(
        "known-bad-watermark-steps-lineage-watermark-zero",
        0,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE],
        "谱系记录的 H_cur 为 0：只红 I-9.30：{verdicts:?}"
    );
}

/// 最新根记账树（根兼叶）里 inode 号水位那一统计量、代段 `generation` 的那一行：`change` 拿到那一行的 34 字节改值，`None` 把这一行从叶里拿掉
/// （后面的行往前挪、条目数与声明长度各减一行）；重封、补到根。
fn change_the_inode_number_watermark_row_of_the_newest_root(
    image: &mut MemoryPool,
    generation: u64,
    new_value: Option<u64>,
) {
    let slot = common_deletion_in_progress::accounting_tree_slot_of_the_newest_root(image);
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        slot,
        16_384,
        |node| {
            assert_eq!(node[50], 0, "这几份镜像上记账树只有根兼叶");
            let read_u16_at = |unit_bytes: &[u8], offset: usize| {
                u16::from_le_bytes(unit_bytes[offset..offset + 2].try_into().expect("2"))
            };
            let row_count = usize::from(read_u16_at(node, ACCOUNTING_LEAF_ENTRY_COUNT_OFFSET));
            let row_index = (0..row_count)
                .find(|index| {
                    let row = ACCOUNTING_LEAF_FIRST_ENTRY_OFFSET + index * ACCOUNTING_ENTRY_WIDTH;
                    read_u16_at(node, row) == INODE_NUMBER_WATERMARK_STATISTIC
                        && node[row + ACCOUNTING_ENTRY_DEVICE_OFFSET
                            ..row + ACCOUNTING_ENTRY_DEVICE_OFFSET + 4]
                            == NO_DEVICE_DIMENSION.to_le_bytes()
                        && node[row + ACCOUNTING_ENTRY_GENERATION_OFFSET
                            ..row + ACCOUNTING_ENTRY_GENERATION_OFFSET + 8]
                            == generation.to_le_bytes()
                })
                .unwrap_or_else(|| panic!("记账树里有 inode 号水位代段 {generation} 那一行"));
            let row = ACCOUNTING_LEAF_FIRST_ENTRY_OFFSET + row_index * ACCOUNTING_ENTRY_WIDTH;
            match new_value {
                Some(value) => node
                    [row + ACCOUNTING_ENTRY_VALUE_OFFSET..row + ACCOUNTING_ENTRY_VALUE_OFFSET + 8]
                    .copy_from_slice(&value.to_le_bytes()),
                None => {
                    assert!(
                        row_index > 0 && row_index + 1 < row_count,
                        "这一行不在首末：拿掉它头里的 key 区间不变"
                    );
                    let end =
                        ACCOUNTING_LEAF_FIRST_ENTRY_OFFSET + row_count * ACCOUNTING_ENTRY_WIDTH;
                    node.copy_within(row + ACCOUNTING_ENTRY_WIDTH..end, row);
                    node[end - ACCOUNTING_ENTRY_WIDTH..end].fill(0);
                    node[ACCOUNTING_LEAF_ENTRY_COUNT_OFFSET
                        ..ACCOUNTING_LEAF_ENTRY_COUNT_OFFSET + 2]
                        .copy_from_slice(
                            &u16::try_from(row_count - 1).expect("行数").to_le_bytes(),
                        );
                    node[UNIT_DECLARED_LENGTH_OFFSET_OF_AN_INDEX_NODE
                        ..UNIT_DECLARED_LENGTH_OFFSET_OF_AN_INDEX_NODE + 2]
                        .copy_from_slice(
                            &u16::try_from((row_count - 1) * ACCOUNTING_ENTRY_WIDTH)
                                .expect("声明长度")
                                .to_le_bytes(),
                        );
                }
            }
        },
    );
}

/// I-9.30 ④ 管「台阶行少了一级」：合法镜像上最新根记账里重发那次（2 号重发、水位 3）留的那级台阶拿掉。谱系记录的 H_cur 3 对不上
/// T_rb − 1 以前最大的台阶 2，判红；台阶行不算进记账行数（恢复读得回这一版）、I-3.13 不判台阶行。只红 I-9.30。
#[test]
fn an_inode_number_watermark_step_dropped_from_the_accounting_tree_reddens_only_the_watermark_step_invariant(
) {
    let history = common::rollback_after_a_recovery_abandoned_a_root_that_created_inodes(
        "known-bad-watermark-steps-a-step-dropped",
    );
    let mut image = history.pool.memory_pool();
    change_the_inode_number_watermark_row_of_the_newest_root(
        &mut image,
        history.created_again.root.checkpoint_txg.0,
        None,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE],
        "台阶行少了重发那一级：只红 I-9.30：{verdicts:?}"
    );
}

/// I-9.30 ① 管「水位低于台阶」：第一个文件之后建两个空 inode（txg 4，水位 4）、再覆盖写一次（txg 5）——最新根记账里台阶 (3, 2)、(4, 4) 与这一版那一行 (5, 4)。
/// 台阶 (4, 4) 的水位抬到 5：记账里的水位 4 低于它，判红；那一级之后出生的记录一条都没有（覆盖写不建 inode），② 不红；水位 4 仍大于树里最大的 key 3，
/// I-9.6 不红；台阶行 I-3.13 不判。只红 I-9.30。
#[test]
fn an_inode_number_watermark_step_raised_above_the_watermark_reddens_only_the_watermark_step_invariant(
) {
    let mut pool = common_deletion_in_progress::pool_with_new_inodes(
        "known-bad-watermark-steps-a-step-above-the-watermark",
        2,
    );
    let created = pool.output.clone();
    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, b"after the new inodes");
    assert_eq!(
        (
            created.root.checkpoint_txg.0,
            created.inode_number_watermark(),
            pool.output.inode_number_watermark()
        ),
        (4, 4, 4),
        "建两个空 inode 那一版 txg 4、水位 4；覆盖写不改水位"
    );
    let mut image = pool.memory_pool();
    assert_eq!(
        violated_invariants(&check_pool_image(&image)),
        Vec::<&str>::new(),
        "改之前一条都不红"
    );
    change_the_inode_number_watermark_row_of_the_newest_root(&mut image, 4, Some(5));
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE],
        "台阶抬到水位之上：只红 I-9.30：{verdicts:?}"
    );
}

// ───────────────────────── 第 56 批 G2、G3：容器号从 1 起、逐片加一、每片至少一条（I-9.21、I-9.22、I-9.36、I-9.37） ─────────────────────────

/// I-9.21 / I-9.22 / I-9.36 / I-9.37 判的三棵容器树（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第三节 G2、G3）：
/// 墓碑树判 I-9.21 / I-9.22，谱系树与退役树判 I-9.36 / I-9.37。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContainerTreeOfTheCapacityInvariants {
    Tombstone,
    Lineage,
    Retirement,
}

impl ContainerTreeOfTheCapacityInvariants {
    fn tree_kind(self) -> u16 {
        match self {
            ContainerTreeOfTheCapacityInvariants::Tombstone => TREE_KIND_TOMBSTONE,
            ContainerTreeOfTheCapacityInvariants::Lineage => TREE_KIND_LINEAGE,
            ContainerTreeOfTheCapacityInvariants::Retirement => TREE_KIND_RETIREMENT,
        }
    }

    /// 带旋钮值的入口判这一棵的那一条：墓碑树 I-9.22，谱系树与退役树 I-9.37。
    fn knob_capacity_invariant(self) -> &'static str {
        match self {
            ContainerTreeOfTheCapacityInvariants::Tombstone => {
                TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE
            }
            ContainerTreeOfTheCapacityInvariants::Lineage
            | ContainerTreeOfTheCapacityInvariants::Retirement => {
                LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE
            }
        }
    }

    /// 违例说明里怎么称呼这一棵（`walk.rs` 的说明照这个写）。
    fn name_in_the_violation(self) -> &'static str {
        match self {
            ContainerTreeOfTheCapacityInvariants::Tombstone => "墓碑树",
            ContainerTreeOfTheCapacityInvariants::Lineage => "谱系树",
            ContainerTreeOfTheCapacityInvariants::Retirement => "退役树",
        }
    }

    /// 池级 checker 带旋钮值的入口：只这一棵装 `records_per_container` 条一片的旋钮，另两棵当没装。
    fn checker_knobs(
        self,
        records_per_container: u64,
    ) -> singlefs_checker::walk::ContainerCapacityKnobs {
        let is_this_tree = |tree: ContainerTreeOfTheCapacityInvariants| tree == self;
        singlefs_checker::walk::ContainerCapacityKnobs {
            tombstone: is_this_tree(ContainerTreeOfTheCapacityInvariants::Tombstone).then(|| {
                singlefs_checker::walk::TombstoneContainerCapacityKnob::new(records_per_container)
                    .expect("1..=815")
            }),
            lineage: is_this_tree(ContainerTreeOfTheCapacityInvariants::Lineage).then(|| {
                singlefs_checker::walk::LineageContainerCapacityKnob::new(records_per_container)
                    .expect("1..=1359")
            }),
            retirement: is_this_tree(ContainerTreeOfTheCapacityInvariants::Retirement).then(|| {
                singlefs_checker::walk::RetirementContainerCapacityKnob::new(records_per_container)
                    .expect("1..=959")
            }),
        }
    }

    /// 写入口：只这一棵压到 `records_per_container` 条一片，另两棵与节点照产品。
    fn write_capacities(
        self,
        records_per_container: u64,
    ) -> singlefs_core::container_tree::ContainerTreeCapacities {
        let is_this_tree = |tree: ContainerTreeOfTheCapacityInvariants| tree == self;
        singlefs_core::container_tree::ContainerTreeCapacities {
            tombstone_container: if is_this_tree(ContainerTreeOfTheCapacityInvariants::Tombstone) {
                singlefs_core::tombstone_tree::TombstoneContainerCapacity::CappedForTests(
                    singlefs_core::tombstone_tree::TombstoneRecordsPerContainerCappedForTests::new(
                        records_per_container,
                    )
                    .expect("1..=815"),
                )
            } else {
                singlefs_core::tombstone_tree::TombstoneContainerCapacity::FromTheContainerFormat
            },
            lineage_container: if is_this_tree(ContainerTreeOfTheCapacityInvariants::Lineage) {
                singlefs_core::lineage_tree::LineageContainerCapacity::CappedForTests(
                    singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(
                        records_per_container,
                    )
                    .expect("1..=1359"),
                )
            } else {
                singlefs_core::lineage_tree::LineageContainerCapacity::FromTheContainerFormat
            },
            retirement_container: if is_this_tree(ContainerTreeOfTheCapacityInvariants::Retirement)
            {
                singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
                    singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(
                        records_per_container,
                    )
                    .expect("1..=959"),
                )
            } else {
                singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat
            },
            node_entries:
                singlefs_core::container_tree::ContainerTreeNodeEntries::FromTheNodeFormat,
        }
    }

    /// 这一版这一棵每片容器的记录数，按叶序。
    fn record_counts_of_each_container(self, output: &TransactionOutput) -> Vec<usize> {
        match self {
            ContainerTreeOfTheCapacityInvariants::Tombstone => output
                .tombstone_containers
                .iter()
                .map(|container| container.contents.records.len())
                .collect(),
            ContainerTreeOfTheCapacityInvariants::Lineage => output
                .lineage_containers
                .iter()
                .map(|container| container.contents.records.len())
                .collect(),
            ContainerTreeOfTheCapacityInvariants::Retirement => output
                .retirement_containers
                .iter()
                .map(|container| container.contents.records.len())
                .collect(),
        }
    }
}

/// 在这一棵树里写 `records` 条记录、写入口只这一棵压到 `records_per_container` 条一片（另两棵照产品）：
/// 墓碑树——建 `records` 个空 inode（2 号起），逐个 unlink 到底，每个一条墓碑；
/// 谱系树——建 1 个空 inode，分配器装上谱系旋钮，挂着回退到第一个文件那一版 `records` 次，每次一条谱系记录；
/// 退役树——再建 232 个空 inode（叶 [1..=233]、[234]），unlink 234 到底（最右那片删空、退役一条），之后每次建 1 个（最左那片满了，开新叶）、
/// unlink 它到底（又退役一条），共 `records` 次。
fn pool_with_records_in_one_container_tree_under_its_knob(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
    records_per_container: u64,
    records: u64,
) -> BuiltPool {
    pool_with_records_in_one_container_tree_written_under_these_knobs(
        tree,
        tag,
        &vec![records_per_container; usize::try_from(records).expect("装得进 usize")],
    )
}

/// 同 [`pool_with_records_in_one_container_tree_under_its_knob`]，第 k 条记录写的时候写入口只这一棵压到 `records_per_container_of_each_record[k]` 条一片
/// （另两棵照产品）：旋钮在一份镜像的历史中途换值。写侧每一步都照当时的旋钮「满了才关」，换小之后下一条开新片、换大之后接着往开放的那片里装，
/// 写得出「不是最右的那片少一条」（第 58 批 J3，攻方 `research/prompts/m3-batch57-code-r1-opus-model/probe-crate/tests/k4_middle_short_container.rs` 那一造法）。
fn pool_with_records_in_one_container_tree_written_under_these_knobs(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
    records_per_container_of_each_record: &[u64],
) -> BuiltPool {
    let records =
        u64::try_from(records_per_container_of_each_record.len()).expect("记录数装得进 u64");
    let pool = match tree {
        ContainerTreeOfTheCapacityInvariants::Tombstone => {
            let mut pool = common_deletion_in_progress::pool_with_new_inodes(tag, records);
            // 迭代上界：`records`。
            for (inode, records_per_container) in
                (2..2 + records).zip(records_per_container_of_each_record)
            {
                unlink_an_empty_inode_to_the_end_under_the_capacities(
                    &mut pool,
                    inode,
                    tree.write_capacities(*records_per_container),
                );
            }
            pool
        }
        ContainerTreeOfTheCapacityInvariants::Lineage => {
            let mut pool = build_pool(tag);
            let first_file_version = pool.output.clone();
            common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
            // 迭代上界：`records`。
            for records_per_container in records_per_container_of_each_record {
                let capacities = tree.write_capacities(*records_per_container);
                pool.allocator
                    .set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
                        batch_size: singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
                        tombstone_container_capacity: capacities.tombstone_container,
                        lineage_container_capacity: capacities.lineage_container,
                        retirement_container_capacity: capacities.retirement_container,
                        container_tree_node_entries: capacities.node_entries,
                    });
                common_deletion_in_progress::roll_back_to(&mut pool, &first_file_version);
            }
            pool
        }
        ContainerTreeOfTheCapacityInvariants::Retirement => {
            let leaf_records = singlefs_format::INODE_LEAF_RECORDS;
            let mut pool = common_deletion_in_progress::pool_with_new_inodes(tag, leaf_records);
            // 迭代上界：`records`。第 0 次删的是建出来的最后一个号（独占最右那片叶），之后每次先建一个再删它。
            for (retirement_index, records_per_container) in
                records_per_container_of_each_record.iter().enumerate()
            {
                if retirement_index > 0 {
                    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(
                        &mut pool, 1,
                    );
                }
                let inode = pool.output.inode_number_watermark() - 1;
                unlink_an_empty_inode_to_the_end_under_the_capacities(
                    &mut pool,
                    inode,
                    tree.write_capacities(*records_per_container),
                );
            }
            pool
        }
    };
    assert_eq!(
        tree.record_counts_of_each_container(&pool.output)
            .iter()
            .sum::<usize>(),
        usize::try_from(records).expect("装得进"),
        "{tree:?} 里一共 {records} 条记录"
    );
    pool
}

/// 同上，再接着建 30 次空 inode，把写容器树的那几版都转出回退候选集：留在候选集里的每一版都是同一棵容器树（照抄、不重写），
/// 改坏最新那一版的容器树就是改坏候选集里每一版的，没有一版还拿着改之前的身份或记录。交回改之前的镜像，带旋钮值的入口上一条都不红、
/// 那一条（I-9.22 / I-9.37）真被评估过且成立。
fn image_with_records_in_one_container_tree_and_the_root_ring_turned_over(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
    records: u64,
) -> MemoryPool {
    let mut pool = pool_with_records_in_one_container_tree_under_its_knob(tree, tag, 1, records);
    common_deletion_in_progress::turn_the_root_ring_over_by_creating_inodes(&mut pool);
    assert_eq!(
        tree.record_counts_of_each_container(&pool.output),
        vec![1; usize::try_from(records).expect("装得进")],
        "{tree:?} 一片 1 条：{records} 片"
    );
    let image = pool.memory_pool();
    let verdicts = singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
        &image,
        tree.checker_knobs(1),
    );
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "改之前（{tree:?}，带旋钮值）一条都不红：{verdicts:?}"
    );
    assert_eq!(
        verdict_among(&verdicts, tree.knob_capacity_invariant()),
        InvariantVerdict::Holds,
        "改之前 {} 真被评估过且成立",
        tree.knob_capacity_invariant()
    );
    image
}

/// 最新的根那一版这一棵树（根层级 1、直接管容器）每片容器的槽，按叶序。
fn container_slots_of_the_newest_tree(
    image: &MemoryPool,
    tree: ContainerTreeOfTheCapacityInvariants,
) -> Vec<singlefs_core::address::SlotNumber> {
    let root_pointer =
        common_pointer_head_corruption::tree_table_entry_of_kind(image, tree.tree_kind()).root;
    let root =
        singlefs_core::unit::parse_index_node(&common_pointer_head_corruption::unit_bytes_at(
            image,
            common_pointer_head_corruption::the_slot_of(&root_pointer),
            common_pointer_head_corruption::node_bytes(),
        ))
        .expect("容器树根解得开");
    assert_eq!(root.level, 1, "{tree:?} 的根直接管容器");
    root.entries
        .iter()
        .map(|entry| {
            common_pointer_head_corruption::the_slot_of(
                &singlefs_core::pointer::NodePointer::read_from(
                    &mut singlefs_core::bytes::ByteReader::at(
                        entry,
                        CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
                    ),
                ),
            )
        })
        .collect()
}

/// 最新的根那一版这一棵树每片容器头里的容器号与根里指它那条条目的身份容器号一起改成 `container_numbers_in_leaf_order` 里的号
/// （身份照旧与子头相符；分隔 key 与子指针不动），改过的单元重封、新的整单元校验和沿引用链补到根。
fn renumber_the_containers_of_the_newest_tree(
    image: &mut MemoryPool,
    tree: ContainerTreeOfTheCapacityInvariants,
    container_numbers_in_leaf_order: &[u64],
) {
    let container_slots = container_slots_of_the_newest_tree(image, tree);
    assert_eq!(
        container_slots.len(),
        container_numbers_in_leaf_order.len(),
        "{tree:?} 的容器片数与要写的号数相同"
    );
    for (entry_index, (container_slot, container_number)) in container_slots
        .into_iter()
        .zip(container_numbers_in_leaf_order)
        .enumerate()
    {
        common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
            image,
            container_slot,
            common_pointer_head_corruption::data_unit_bytes(),
            |container| {
                set_u64(
                    container,
                    PACKED_UNIT_CONTAINER_NUMBER_OFFSET,
                    *container_number,
                )
            },
        );
        change_an_entry_of_the_root_of_kind(image, tree.tree_kind(), entry_index, |entry| {
            set_u64(
                entry,
                CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET,
                *container_number,
            );
        });
    }
}

/// 打包记录单元的记录区起点（136；声明长度的偏移是上面的 `PACKED_UNIT_DECLARED_LENGTH_OFFSET`）。
const PACKED_UNIT_RECORDS_START: usize = 136;

/// 最新的根那一版这一棵树最右那片容器清空：记录数与声明长度写 0、记录区写 0（容器号、身份不动），重封、补校验和。
fn empty_the_rightmost_container_of_the_newest_tree(
    image: &mut MemoryPool,
    tree: ContainerTreeOfTheCapacityInvariants,
) {
    let rightmost_slot = *container_slots_of_the_newest_tree(image, tree)
        .last()
        .expect("至少一片容器");
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        rightmost_slot,
        common_pointer_head_corruption::data_unit_bytes(),
        |container| {
            assert_eq!(
                u16::from_le_bytes([
                    container[PACKED_UNIT_RECORD_COUNT_OFFSET],
                    container[PACKED_UNIT_RECORD_COUNT_OFFSET + 1]
                ]),
                1,
                "最右那片容器一条记录"
            );
            set_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET, 0);
            set_u16(container, PACKED_UNIT_DECLARED_LENGTH_OFFSET, 0);
            container[PACKED_UNIT_RECORDS_START..].fill(0);
        },
    );
}

/// 带旋钮值的入口（只这一棵装一片 1 条）上只红这一棵的那一条（I-9.22 / I-9.37），违例说明里有 `detail_fragment`。
fn assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
    image: &MemoryPool,
    tree: ContainerTreeOfTheCapacityInvariants,
    detail_fragment: &str,
) {
    assert_only_the_knob_capacity_invariant_of_the_tree_reddens_given_the_knob(
        image,
        tree,
        1,
        detail_fragment,
    );
}

/// 带旋钮值的入口（只这一棵装一片 `records_per_container` 条）上只红这一棵的那一条（I-9.22 / I-9.37），违例说明里有 `detail_fragment`。
fn assert_only_the_knob_capacity_invariant_of_the_tree_reddens_given_the_knob(
    image: &MemoryPool,
    tree: ContainerTreeOfTheCapacityInvariants,
    records_per_container: u64,
    detail_fragment: &str,
) {
    let verdicts = singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
        image,
        tree.checker_knobs(records_per_container),
    );
    assert_eq!(
        violated_invariants(&verdicts),
        [tree.knob_capacity_invariant()],
        "{tree:?}（旋钮值 {records_per_container}）：只红 {}：{verdicts:?}",
        tree.knob_capacity_invariant()
    );
    assert!(
        matches!(
            verdict_among(&verdicts, tree.knob_capacity_invariant()),
            InvariantVerdict::Violated(detail)
                if detail.contains(tree.name_in_the_violation()) && detail.contains(detail_fragment)
        ),
        "{tree:?}：违例说明写出「{detail_fragment}」：{:?}",
        verdict_among(&verdicts, tree.knob_capacity_invariant())
    );
}

/// 攻方 `mut-w-gap2` 那一形（写侧下一个号 = 最右号 + 2）：两片容器 1、2 号改成 1、3 号。
fn assert_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
) {
    let mut image =
        image_with_records_in_one_container_tree_and_the_root_ring_turned_over(tree, tag, 2);
    renumber_the_containers_of_the_newest_tree(&mut image, tree, &[1, 3]);
    assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
        &image,
        tree,
        "沿叶序容器号不是逐片加一：容器 1 之后是容器 3",
    );
}

/// 攻方 `mut-w-start2` 那一形（零根时的号写成 2）：两片容器 1、2 号改成 2、3 号。
fn assert_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
) {
    let mut image =
        image_with_records_in_one_container_tree_and_the_root_ring_turned_over(tree, tag, 2);
    renumber_the_containers_of_the_newest_tree(&mut image, tree, &[2, 3]);
    assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
        &image,
        tree,
        "沿叶序第一片容器的容器号是 2，不是 1",
    );
}

/// 攻方 `mut-w-eager` 那一形（满了就先开下一片、最右多一片 0 条）：三片各 1 条，最右那片清空。
fn assert_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
) {
    let mut image =
        image_with_records_in_one_container_tree_and_the_root_ring_turned_over(tree, tag, 3);
    empty_the_rightmost_container_of_the_newest_tree(&mut image, tree);
    assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
        &image,
        tree,
        "容器 3 一条记录都没有",
    );
}

/// 攻方 `mut-w-same` 那一形（下一个号 = 最右号）：两片容器 1、2 号改成 1、1 号。
fn assert_containers_numbered_one_and_one_redden_only_the_knob_capacity_invariant(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
) {
    let mut image =
        image_with_records_in_one_container_tree_and_the_root_ring_turned_over(tree, tag, 2);
    renumber_the_containers_of_the_newest_tree(&mut image, tree, &[1, 1]);
    assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
        &image,
        tree,
        "沿叶序容器号不是逐片加一：容器 1 之后是容器 1",
    );
}

#[test]
fn tombstone_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-numbered-one-and-three",
    );
}

#[test]
fn lineage_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-numbered-one-and-three",
    );
}

#[test]
fn retirement_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_one_and_three_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-numbered-one-and-three",
    );
}

#[test]
fn tombstone_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-numbered-two-and-three",
    );
}

#[test]
fn lineage_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-numbered-two-and-three",
    );
}

#[test]
fn retirement_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_two_and_three_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-numbered-two-and-three",
    );
}

#[test]
fn tombstone_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant() {
    assert_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-with-an-empty-rightmost-one",
    );
}

#[test]
fn lineage_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant() {
    assert_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-with-an-empty-rightmost-one",
    );
}

#[test]
fn retirement_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant() {
    assert_containers_with_an_empty_rightmost_one_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-with-an-empty-rightmost-one",
    );
}

/// G3 ①：谱系树两片同号（墓碑树那一形是 `two_tombstone_containers_with_one_container_number_redden_only_the_capacity_knob_invariant`）。
#[test]
fn lineage_containers_numbered_one_and_one_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_one_and_one_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-numbered-one-and-one",
    );
}

/// G3 ①：退役树两片同号。
#[test]
fn retirement_containers_numbered_one_and_one_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_one_and_one_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-numbered-one-and-one",
    );
}

/// 第 57 批 H3（旋钮支那一形，判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节；正推 K2.2）：两片容器 1、2 号改成 0、1 号——
/// 相邻加一成立、每片 1 条，只「沿叶序第一片容器号是 1」那句不成立（判成「第一片 ≤ 1」就放过它）。
fn assert_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
) {
    let mut image =
        image_with_records_in_one_container_tree_and_the_root_ring_turned_over(tree, tag, 2);
    renumber_the_containers_of_the_newest_tree(&mut image, tree, &[0, 1]);
    assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
        &image,
        tree,
        "沿叶序第一片容器的容器号是 0，不是 1",
    );
}

#[test]
fn tombstone_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-numbered-zero-and-one",
    );
}

#[test]
fn lineage_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-numbered-zero-and-one",
    );
}

#[test]
fn retirement_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant() {
    assert_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-numbered-zero-and-one",
    );
}

/// 第 57 批 H1（攻方 `research/prompts/m3-batch56-code-r1-opus-model/probe-crate/tests/k2_knob_bad_images.rs` 那一形，判决同上第三节）：
/// 只这一棵在旋钮 1 下写出两片各 1 条（另两棵照产品）。写侧在旋钮 K 下装满 K 条才关（D18（块里携带什么信息） 已定项 24 第 3 条「满了才关；
/// 关闭路径加只供测试的容量旋钮」），旋钮 2、3 写不出「不是最右的那片只装 1 条」。带旋钮值的入口：报写的时候那个旋钮值 1 一条不红、
/// 这一棵的那一条（I-9.22 / I-9.37）判过而成立；报旋钮 2、3——每片都不超旋钮值、容器号从 1 逐片加一、每片至少一条，只「不是最右的那片装满
/// 旋钮值」那句不成立，只红这一棵的那一条。
fn assert_two_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
) {
    let pool = pool_with_records_in_one_container_tree_under_its_knob(tree, tag, 1, 2);
    assert_eq!(
        tree.record_counts_of_each_container(&pool.output),
        [1, 1],
        "{tree:?} 两片各 1 条"
    );
    let image = pool.memory_pool();
    let verdicts_given_the_written_knob =
        singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
            &image,
            tree.checker_knobs(1),
        );
    assert_eq!(
        violated_invariants(&verdicts_given_the_written_knob),
        Vec::<&str>::new(),
        "{tree:?}：报写的时候那个旋钮值 1 一条不红：{verdicts_given_the_written_knob:?}"
    );
    assert_eq!(
        verdict_among(
            &verdicts_given_the_written_knob,
            tree.knob_capacity_invariant()
        ),
        InvariantVerdict::Holds,
        "{tree:?}：报旋钮值 1 时 {} 判过而成立",
        tree.knob_capacity_invariant()
    );
    // 迭代上界：两个旋钮值。
    for larger_knob in [2, 3] {
        assert_only_the_knob_capacity_invariant_of_the_tree_reddens_given_the_knob(
            &image,
            tree,
            larger_knob,
            &format!("不是最右的容器 1 装了 1 条，不是旋钮值 {larger_knob}"),
        );
    }
}

#[test]
fn two_tombstone_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
) {
    assert_two_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-closed-before-a-larger-knob",
    );
}

#[test]
fn two_lineage_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
) {
    assert_two_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-closed-before-a-larger-knob",
    );
}

#[test]
fn two_retirement_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
) {
    assert_two_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-closed-before-a-larger-knob",
    );
}

/// 第 58 批 J3 那几张盘的旋钮值：写的时候在 2 与 1 之间换，判的时候报 2。
const KNOB_RECORDS_OF_THE_SHORT_CONTAINER_IMAGES: u64 = 2;

/// 第 58 批 J3 那几张盘怎么造（判决 `research/prompts/m3-batch57-code-r1-main-verification.md` 第三节 J3；两条腿 K4）：一片几条、按叶序每片记录数、
/// 少一条的那片的容器号、判之前要不要把根环转过一圈。
struct ContainersUnderChangingKnobs {
    records_per_container_of_each_record: &'static [u64],
    record_counts_in_leaf_order: &'static [usize],
    short_container: u64,
    root_ring: RootRingBeforeTheJudgement,
}

/// 判之前根环转没转过一圈。
#[derive(Clone, Copy, Debug)]
enum RootRingBeforeTheJudgement {
    /// 写容器树的那几版都还在回退候选集里，池级 checker 逐版判。
    KeepsTheVersionsThatWroteTheContainerTree,
    /// 再建 30 次空 inode（`common_deletion_in_progress::turn_the_root_ring_over_by_creating_inodes`）：留在候选集里的每一版都是写完的那棵容器树。
    TurnedOverByCreatingInodes,
}

/// 中间那片少一条：旋钮 2、2、1、1 写出 [2, 1, 1]（攻方 `k4_middle_short_container` 那一形）。只判叶序第一片（攻方 `h1-first-only`）、
/// 只判最左片（正推 MB2）在它上面照绿，正确判法红在容器 2。
const MIDDLE_CONTAINER_SHORT: ContainersUnderChangingKnobs = ContainersUnderChangingKnobs {
    records_per_container_of_each_record: &[2, 2, 1, 1],
    record_counts_in_leaf_order: &[2, 1, 1],
    short_container: 2,
    root_ring: RootRingBeforeTheJudgement::KeepsTheVersionsThatWroteTheContainerTree,
};

/// 倒数第二片之前全满、倒数第二片少一条：旋钮 2、2、2、2、1、1 写出 [2, 2, 1, 1]。只判叶序第一片、只判最左片照绿，正确判法红在容器 3。
const ALL_FULL_BEFORE_A_SHORT_SECOND_TO_LAST_CONTAINER: ContainersUnderChangingKnobs =
    ContainersUnderChangingKnobs {
        records_per_container_of_each_record: &[2, 2, 2, 2, 1, 1],
        record_counts_in_leaf_order: &[2, 2, 1, 1],
        short_container: 3,
        root_ring: RootRingBeforeTheJudgement::KeepsTheVersionsThatWroteTheContainerTree,
    };

/// 少一条的那片之后还有一片满的：旋钮 2、2、1、1、2、1 写出 [2, 1, 2, 1]，再把根环转过一圈。只判倒数第二片（正推 MC）在它上面照绿——
/// 写的中途有一版是 [2, 1, 1]，容器 2 在那一版是倒数第二片，那一版留在候选集里时只判倒数第二片也红，所以要转出去；
/// 只判叶序第一片、只判最左片同样照绿，正确判法红在容器 2。
const A_SHORT_CONTAINER_BEFORE_A_FULL_ONE: ContainersUnderChangingKnobs =
    ContainersUnderChangingKnobs {
        records_per_container_of_each_record: &[2, 2, 1, 1, 2, 1],
        record_counts_in_leaf_order: &[2, 1, 2, 1],
        short_container: 2,
        root_ring: RootRingBeforeTheJudgement::TurnedOverByCreatingInodes,
    };

/// 少一条的那片离最右三片：旋钮 2、2、1、1、2、2、2、1 写出 [2, 1, 2, 2, 1]，再把根环转过一圈（第 59 批 L2）。上面三张盘少的那片离最右
/// 都不超过两片，只判最右之前两片（攻方 `research/prompts/m3-batch58-code-r1-opus-model/mutants.py` 的 `w-last-two`）在它们上面与正确判法同判；
/// 这一张的容器 2 在倒数第三片之前，那一形不判它、照绿，正确判法红在容器 2。写的中途 [2, 1, 1]、[2, 1, 2]、[2, 1, 2, 1] 那几版里容器 2
/// 离最右不超过两片，那几版留在候选集里时那一形也红，所以要转出去。
const A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST: ContainersUnderChangingKnobs =
    ContainersUnderChangingKnobs {
        records_per_container_of_each_record: &[2, 2, 1, 1, 2, 2, 2, 1],
        record_counts_in_leaf_order: &[2, 1, 2, 2, 1],
        short_container: 2,
        root_ring: RootRingBeforeTheJudgement::TurnedOverByCreatingInodes,
    };

/// 第 58 批 J3：只这一棵按 `layout` 逐条换旋钮写出三片以上的容器（另两棵照产品），带旋钮值的入口报这一棵旋钮 2：每片都不超旋钮值、
/// 容器号从 1 逐片加一、每片至少一条，只「不是最右的容器 `short_container` 装了 1 条，不是旋钮值 2」不成立，只红这一棵的那一条（I-9.22 / I-9.37）。
/// 第 57 批 H1 的会红用例都只有两片容器，两片时只判最左片、只判倒数第二片、只判叶序第一片与正确写法等价，这几张盘把它们分开。
fn assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
    layout: &ContainersUnderChangingKnobs,
) {
    let mut pool = pool_with_records_in_one_container_tree_written_under_these_knobs(
        tree,
        tag,
        layout.records_per_container_of_each_record,
    );
    match layout.root_ring {
        RootRingBeforeTheJudgement::KeepsTheVersionsThatWroteTheContainerTree => {}
        RootRingBeforeTheJudgement::TurnedOverByCreatingInodes => {
            common_deletion_in_progress::turn_the_root_ring_over_by_creating_inodes(&mut pool);
        }
    }
    assert_eq!(
        tree.record_counts_of_each_container(&pool.output),
        layout.record_counts_in_leaf_order,
        "{tree:?} 按叶序每片的记录数"
    );
    assert_only_the_knob_capacity_invariant_of_the_tree_reddens_given_the_knob(
        &pool.memory_pool(),
        tree,
        KNOB_RECORDS_OF_THE_SHORT_CONTAINER_IMAGES,
        &format!(
            "不是最右的容器 {} 装了 1 条，不是旋钮值 {KNOB_RECORDS_OF_THE_SHORT_CONTAINER_IMAGES}",
            layout.short_container
        ),
    );
}

#[test]
fn tombstone_containers_of_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-of-two-one-and-one-records",
        &MIDDLE_CONTAINER_SHORT,
    );
}

#[test]
fn lineage_containers_of_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-of-two-one-and-one-records",
        &MIDDLE_CONTAINER_SHORT,
    );
}

#[test]
fn retirement_containers_of_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-of-two-one-and-one-records",
        &MIDDLE_CONTAINER_SHORT,
    );
}

#[test]
fn tombstone_containers_of_two_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-of-two-two-one-and-one-records",
        &ALL_FULL_BEFORE_A_SHORT_SECOND_TO_LAST_CONTAINER,
    );
}

#[test]
fn lineage_containers_of_two_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-of-two-two-one-and-one-records",
        &ALL_FULL_BEFORE_A_SHORT_SECOND_TO_LAST_CONTAINER,
    );
}

#[test]
fn retirement_containers_of_two_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-of-two-two-one-and-one-records",
        &ALL_FULL_BEFORE_A_SHORT_SECOND_TO_LAST_CONTAINER,
    );
}

#[test]
fn tombstone_containers_of_two_one_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-of-two-one-two-and-one-records",
        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
    );
}

#[test]
fn lineage_containers_of_two_one_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-of-two-one-two-and-one-records",
        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
    );
}

#[test]
fn retirement_containers_of_two_one_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-of-two-one-two-and-one-records",
        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
    );
}

#[test]
fn tombstone_containers_of_two_one_two_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Tombstone,
        "known-bad-tombstone-containers-of-two-one-two-two-and-one-records",
        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
    );
}

#[test]
fn lineage_containers_of_two_one_two_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-lineage-containers-of-two-one-two-two-and-one-records",
        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
    );
}

#[test]
fn retirement_containers_of_two_one_two_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
) {
    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-retirement-containers-of-two-one-two-two-and-one-records",
        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
    );
}

/// G3 ③：退役容器压到一片 2 条写出两条退役记录（一片 2 条），带旋钮值的入口报退役旋钮 1：那片装了 2 条、多于旋钮值，只红 I-9.37；
/// 报写的时候那个旋钮值 2 一条不红。
#[test]
fn a_retirement_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant(
) {
    let tree = ContainerTreeOfTheCapacityInvariants::Retirement;
    let pool = pool_with_records_in_one_container_tree_under_its_knob(
        tree,
        "known-bad-retirement-container-past-the-knob",
        2,
        2,
    );
    assert_eq!(
        tree.record_counts_of_each_container(&pool.output),
        [2],
        "退役树一片容器 2 条"
    );
    let image = pool.memory_pool();
    let verdicts_given_the_written_knob =
        singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
            &image,
            tree.checker_knobs(2),
        );
    assert_eq!(
        violated_invariants(&verdicts_given_the_written_knob),
        Vec::<&str>::new(),
        "报写的时候那个旋钮值 2：一条不红：{verdicts_given_the_written_knob:?}"
    );
    assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
        &image,
        tree,
        "装了 2 条，多于旋钮值 1",
    );
}

/// G3 ②：产品容量下退役 960 片叶（每次建一个号开新叶、再删到底）：退役树 [959, 1]，同时 960 条墓碑把墓碑树写成 [815, 145]。
/// 产品入口一条不红，I-9.36（退役树：第一片装满 959、最右那片不要求满）与 I-9.21 真被评估过且成立——959 与谱系的 1359 对调、
/// 最右那片也要求满，这里就红。
#[test]
#[ignore = "harness 慢档用例：产品容量下退役 960 次（约 2900 次发布）"]
fn nine_hundred_sixty_retirements_at_product_capacity_fill_every_retirement_container_but_the_rightmost_and_redden_nothing(
) {
    const RETIREMENTS_THAT_OPEN_A_SECOND_CONTAINER: u64 = 960;
    let tree = ContainerTreeOfTheCapacityInvariants::Retirement;
    let leaf_records = singlefs_format::INODE_LEAF_RECORDS;
    let mut pool = common_deletion_in_progress::pool_with_new_inodes(
        "known-good-retirement-containers-at-product-capacity",
        leaf_records - 1,
    );
    // 迭代上界：常量 960。每一轮建一个号（最左那片叶满了，开新叶），再删到底：那片叶删空、退役一条，墓碑一条。
    for _ in 0..RETIREMENTS_THAT_OPEN_A_SECOND_CONTAINER {
        common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
        let inode = pool.output.inode_number_watermark() - 1;
        unlink_an_empty_inode_to_the_end_under_the_capacities(
            &mut pool,
            inode,
            singlefs_core::container_tree::ContainerTreeCapacities::FROM_THE_FORMAT,
        );
    }
    assert_eq!(
        tree.record_counts_of_each_container(&pool.output),
        [959, 1],
        "退役树第一片装满 959、第二片 1 条"
    );
    assert_eq!(
        ContainerTreeOfTheCapacityInvariants::Tombstone
            .record_counts_of_each_container(&pool.output),
        [815, 145],
        "墓碑树第一片装满 815、第二片 145 条"
    );
    let verdicts = check_pool_image(&pool.memory_pool());
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "产品容量的多容器合法镜像：一条不红：{verdicts:?}"
    );
    for invariant in [
        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
    ] {
        assert_eq!(
            verdict_among(&verdicts, invariant),
            InvariantVerdict::Holds,
            "{invariant} 真被评估过且成立"
        );
    }
}

/// G3 ②：产品容量下挂着回退 1360 次（每次退到紧前那一版）：谱系树 [1359, 1]。产品入口一条不红，I-9.36（谱系树：第一片装满 1359、
/// 最右那片不要求满）真被评估过且成立——1359 与退役的 959 对调、最右那片也要求满，这里就红。
#[test]
#[ignore = "harness 慢档用例：产品容量下回退 1360 次"]
fn one_thousand_three_hundred_sixty_rollbacks_at_product_capacity_fill_every_lineage_container_but_the_rightmost_and_redden_nothing(
) {
    const ROLLBACKS_THAT_OPEN_A_SECOND_CONTAINER: u64 = 1360;
    let tree = ContainerTreeOfTheCapacityInvariants::Lineage;
    let mut pool = build_pool("known-good-lineage-containers-at-product-capacity");
    let mut previous = pool.output.clone();
    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    // 迭代上界：常量 1360。每一次退到紧前那一版（两版轮流），一次一条谱系记录。
    for _ in 0..ROLLBACKS_THAT_OPEN_A_SECOND_CONTAINER {
        let target = previous;
        previous = pool.output.clone();
        common_deletion_in_progress::roll_back_to(&mut pool, &target);
    }
    assert_eq!(
        tree.record_counts_of_each_container(&pool.output),
        [1359, 1],
        "谱系树第一片装满 1359、第二片 1 条"
    );
    let verdicts = check_pool_image(&pool.memory_pool());
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "产品容量的多容器合法镜像：一条不红：{verdicts:?}"
    );
    assert_eq!(
        verdict_among(
            &verdicts,
            NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE
        ),
        InvariantVerdict::Holds,
        "I-9.36 真被评估过且成立"
    );
}

/// G3 ④ 判不了那一句的理由（`walk.rs` 的 `DeletionScanGap::reason`）里点得出是哪一形的那一段。
const NOT_JUDGED_LINEAGE_TREE_UNREADABLE: &str = "谱系树读不全";
const NOT_JUDGED_RETIREMENT_TREE_UNREADABLE: &str = "退役树读不全";
const NOT_JUDGED_TREE_TABLE_UNREADABLE: &str = "树表读不出";

/// 带旋钮值的入口上 I-9.37 报的不适用理由（判出违例或成立都不是这一形）。
fn reason_the_knob_capacity_invariant_is_not_judged(
    image: &MemoryPool,
    tree: ContainerTreeOfTheCapacityInvariants,
) -> &'static str {
    let verdicts = singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
        image,
        tree.checker_knobs(1),
    );
    match verdict_among(&verdicts, tree.knob_capacity_invariant()) {
        InvariantVerdict::NotApplicable(reason) => reason,
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::PartiallyJudged(_)) => panic!(
            "{tree:?}：{} 要报射程里判不了（不适用带理由），实际 {other:?}",
            tree.knob_capacity_invariant()
        ),
    }
}

/// G3 ④ 谱系树读不全：旋钮下回退两次（谱系 [1, 1]），第二片谱系容器两份全零（不补引用链）。回退一次那一版的谱系树读得全、照判；
/// 最新那一版读不全，I-9.37 报射程里判不了、不报成立。
#[test]
fn an_unreadable_lineage_container_leaves_the_knob_capacity_invariant_not_judged() {
    let tree = ContainerTreeOfTheCapacityInvariants::Lineage;
    let mut image = pool_with_records_in_one_container_tree_under_its_knob(
        tree,
        "known-bad-lineage-container-unreadable",
        1,
        2,
    )
    .memory_pool();
    let second_container_slot = container_slots_of_the_newest_tree(&image, tree)[1];
    common_deletion_in_progress::zero_both_copies_of_the_unit_at(
        &mut image,
        second_container_slot,
        common_pointer_head_corruption::data_unit_bytes(),
    );
    let reason = reason_the_knob_capacity_invariant_is_not_judged(&image, tree);
    assert!(
        reason.contains(NOT_JUDGED_LINEAGE_TREE_UNREADABLE),
        "谱系树读不全的那一版 I-9.37 判不了：{reason}"
    );
}

/// G3 ④ 退役树读不全：旋钮下退役两次（退役 [1, 1]），第二片退役容器两份全零。I-9.37 报射程里判不了。
#[test]
fn an_unreadable_retirement_container_leaves_the_knob_capacity_invariant_not_judged() {
    let tree = ContainerTreeOfTheCapacityInvariants::Retirement;
    let mut image = pool_with_records_in_one_container_tree_under_its_knob(
        tree,
        "known-bad-retirement-container-unreadable",
        1,
        2,
    )
    .memory_pool();
    let second_container_slot = container_slots_of_the_newest_tree(&image, tree)[1];
    common_deletion_in_progress::zero_both_copies_of_the_unit_at(
        &mut image,
        second_container_slot,
        common_pointer_head_corruption::data_unit_bytes(),
    );
    let reason = reason_the_knob_capacity_invariant_is_not_judged(&image, tree);
    assert!(
        reason.contains(NOT_JUDGED_RETIREMENT_TREE_UNREADABLE),
        "退役树读不全的那一版 I-9.37 判不了：{reason}"
    );
}

/// G3 ④ 整版树表读不出：旋钮下回退一次（谱系 [1]），再建一个号；回退那一版的树表两份全零（不补引用链）。那一版谱系与退役都判不了，
/// I-9.37 报射程里判不了；最新那一版照判。
#[test]
fn an_unreadable_tree_table_leaves_the_knob_capacity_invariant_not_judged() {
    let tree = ContainerTreeOfTheCapacityInvariants::Lineage;
    let mut pool = pool_with_records_in_one_container_tree_under_its_knob(
        tree,
        "known-bad-tree-table-unreadable-under-the-lineage-knob",
        1,
        1,
    );
    let rolled_back_once_tree_table_slot = pool.output.unit(TransactionUnit::TreeTable).slot;
    let rolled_back_once_txg = pool.output.root.checkpoint_txg;
    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let mut image = pool.memory_pool();
    assert_ne!(
        common_deletion_in_progress::tree_table_slot_of_the_newest_root(&image),
        rolled_back_once_tree_table_slot,
        "最新那一版（txg {} 之后又建了一个号）的树表另写一份",
        rolled_back_once_txg.0
    );
    common_deletion_in_progress::zero_both_copies_of_the_unit_at(
        &mut image,
        rolled_back_once_tree_table_slot,
        common_pointer_head_corruption::node_bytes(),
    );
    let reason = reason_the_knob_capacity_invariant_is_not_judged(&image, tree);
    assert!(
        reason.contains(NOT_JUDGED_TREE_TABLE_UNREADABLE),
        "树表读不出的那一版 I-9.37 判不了：{reason}"
    );
}

/// 旧的墓碑旋钮入口（`check_pool_image_given_the_tombstone_container_capacity_knob`）只给墓碑旋钮：谱系树与退役树当没装旋钮。
/// 这一棵在旋钮下写成两片各 1 条，交给旧的墓碑旋钮入口（墓碑旋钮给产品容量 815：退役那份历史 unlink 两次写的两条墓碑同在一片，旋钮给得再小
/// I-9.22 会跟着红）：这一棵按产品容量判，只红 I-9.36、说明点名这一棵；I-9.37 报不适用。
fn assert_the_old_tombstone_knob_entry_judges_the_tree_by_its_product_capacity(
    tree: ContainerTreeOfTheCapacityInvariants,
    tag: &str,
) {
    let image =
        pool_with_records_in_one_container_tree_under_its_knob(tree, tag, 1, 2).memory_pool();
    let verdicts =
        singlefs_checker::walk::check_pool_image_given_the_tombstone_container_capacity_knob(
            &image,
            Some(
                singlefs_checker::walk::TombstoneContainerCapacityKnob::new(
                    singlefs_format::TOMBSTONE_CONTAINER_RECORDS,
                )
                .expect("1..=815"),
            ),
        );
    assert_eq!(
        violated_invariants(&verdicts),
        [NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE],
        "{tree:?}：旧的墓碑旋钮入口按产品容量判这一棵，只红 I-9.36：{verdicts:?}"
    );
    assert!(
        matches!(
            verdict_among(
                &verdicts,
                NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE
            ),
            InvariantVerdict::Violated(detail) if detail.contains(tree.name_in_the_violation())
        ),
        "{tree:?}：I-9.36 的说明点名这一棵"
    );
    assert!(
        matches!(
            verdict_among(
                &verdicts,
                LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE
            ),
            InvariantVerdict::NotApplicable(_)
        ),
        "{tree:?}：I-9.37 报不适用"
    );
}

/// G3 ⑤：旧的墓碑旋钮入口给谱系那一棵 `None`。
#[test]
fn the_old_tombstone_knob_entry_judges_the_lineage_tree_by_its_product_capacity() {
    assert_the_old_tombstone_knob_entry_judges_the_tree_by_its_product_capacity(
        ContainerTreeOfTheCapacityInvariants::Lineage,
        "known-bad-old-tombstone-knob-entry-lineage",
    );
}

/// G3 ⑤：旧的墓碑旋钮入口给退役那一棵 `None`。
#[test]
fn the_old_tombstone_knob_entry_judges_the_retirement_tree_by_its_product_capacity() {
    assert_the_old_tombstone_knob_entry_judges_the_tree_by_its_product_capacity(
        ContainerTreeOfTheCapacityInvariants::Retirement,
        "known-bad-old-tombstone-knob-entry-retirement",
    );
}
