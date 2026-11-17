//! 已知坏镜像表：池级 checker 的回归语料（`tests/checker_known_bad_images.rs`）与 E171 装置（`src/bin/e171_three_oracle_collusion*.rs`）共用的那一份。
//! [`known_bad_images`] 每一条是「目标不变量名 + 一处改法」：在写完新池新建文件的干净镜像上改一处，被改的单元重封校验和，
//! 再把新的整单元校验和沿引用链一路补到根记录（`propagate`、[`mutate_unit`] 这一族），让被改的那一条之前的判定不先挡住。
//! 这里放这张表与它用到的常量、辅助函数；用例文件里别的族用到的同名辅助函数从这里取，表的判法留在用例文件里。

use crate::memory_pool::MemoryPool;
use singlefs_checker::image::{
    ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
    FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
    FREE_ALLOCATED_AND_UNRECLAIMABLE_SUM_TO_THE_CAPACITY_AND_THE_COMMITTED_RESERVATION_MATCHES_ITS_CLOSED_FORM,
    REFERENCE_INTERVALS_ARE_EXACT, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
    SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
    SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
    SYSTEM_CONFIGURATION_RECORDS_THE_WRITER_IDENTITY,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::journal::{back_chain_of, record_offset, JournalRecord};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::records::STATISTIC_UNRECLAIMABLE_BYTES;
use singlefs_core::recovery::{choose_system_configuration, readable_roots, PoolReader};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES};

/// E142 装置里的固定 fsid（`FIXED_FSID`）。
pub const E142_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];

/// 共用搭建模块（`tests/common`）建的那种池的 mkfs 参数：固定身份、默认 journal 环、根槽 512 字节。
pub fn parameters_of_the_common_test_pool() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: E142_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

/// `journal_ring_bytes` 长的 journal 环里计数器 `counter` 那一格在 `device` 上的 4096 字节（没写过的格读出来是全 0）。
///
/// # Panics
/// 那一格读不到。
pub fn journal_record_bytes_in_a_ring_of_bytes(
    image: &MemoryPool,
    journal_ring_bytes: u64,
    device: DeviceIdentity,
    counter: u64,
) -> Vec<u8> {
    PoolReader::read(
        image,
        device,
        record_offset(counter, journal_ring_bytes),
        usize::try_from(JOURNAL_RECORD_BYTES).expect("4096"),
    )
    .expect("环里那一格读得到")
}

/// 根环里每条自证过的根（`readable_roots`），把两块盘 journal 默认环里同 (实例, txg) 的记录的新根段改成照它的那七段
/// （树表指针、中央映射树根指针、树 ID 水位、回退下界 F、实例表指针、分配记录树根指针、卸载记号；I-8.10（记录新根段与同代根相同）
/// 判前四段，D23（journal 的角色与格式） 已定项 15 施加时七段都取记录里的，D23（journal 的角色与格式） 已定项 4：
/// 一次发布的每条记录带同一份新根段），`JournalRecord::to_bytes` 重封；再按计数器从小到大，前一条被改过、又与这一条同实例的，
/// 这一条的反向链按前一条改后的记录头重算、重封（I-8.6（反向链算法）只链本实例内逻辑前一条）。
/// 坏镜像改了根（新整单元校验和沿引用链补到根槽、改树表指针）之后调它：盘上留下的就是「写者从一开始就写出了这样的根」的样子，
/// 只改根不改记录的镜像每一份都会多红 I-8.10。同 (实例, txg) 有几条根时照第一条（I-8.10 那一行把它归 C88，这几份镜像上没有）；
/// 环里从计数器 1 起读到第一格解不出就停。什么都不用改时盘面逐字节不变。交回改了几条记录（按盘各算一条）。
///
/// # Panics
/// 镜像上择不出系统配置。
pub fn copy_each_root_into_the_new_root_segment_of_its_journal_records(
    image: &mut MemoryPool,
) -> usize {
    let parameters_of_the_pool = parameters_of_the_common_test_pool();
    copy_each_root_into_the_new_root_segment_of_its_journal_records_in_a_ring_of_bytes(
        image,
        parameters_of_the_pool.geometry.journal_ring_bytes,
        &parameters_of_the_pool.filesystem_identifier,
    )
}

/// 同 [`copy_each_root_into_the_new_root_segment_of_its_journal_records`]，journal 环长与文件系统身份由调用方给（盘宽、环长不是共用参数的那几组池）。
///
/// # Panics
/// 镜像上择不出系统配置。
pub fn copy_each_root_into_the_new_root_segment_of_its_journal_records_in_a_ring_of_bytes(
    image: &mut MemoryPool,
    journal_ring_bytes: u64,
    filesystem_identifier_of_the_pool: &[u8; 16],
) -> usize {
    let system_configuration =
        choose_system_configuration(&*image).expect("坏镜像只改了单元与根槽，系统配置择得出");
    let mut root_of_each_instance_and_txg: std::collections::BTreeMap<
        (InstanceGeneration, CheckpointTxg),
        singlefs_core::root_record::RootRecord,
    > = std::collections::BTreeMap::new();
    for root in readable_roots(
        &*image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    ) {
        root_of_each_instance_and_txg
            .entry((root.instance, root.checkpoint_txg))
            .or_insert(root);
    }
    let filesystem_identifier = unit_filesystem_identifier(filesystem_identifier_of_the_pool);
    let record_slots_in_the_ring = journal_ring_bytes / JOURNAL_RECORD_BYTES;
    let mut records_rewritten = 0;
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        // 跨轮携带：计数器小 1 的那一条若被改过，记它的实例与改后的字节（下一条的反向链按它重算）。
        let mut rewritten_previous_record: Option<(InstanceGeneration, Vec<u8>)> = None;
        // 迭代次数的上界是环的槽数；环里第一格解不出的记录就是出口。
        for counter in 1..=record_slots_in_the_ring {
            let bytes =
                journal_record_bytes_in_a_ring_of_bytes(image, journal_ring_bytes, device, counter);
            let Some(record) = JournalRecord::parse(&bytes, filesystem_identifier) else {
                break;
            };
            let mut rewritten = record.clone();
            if let Some(root) =
                root_of_each_instance_and_txg.get(&(record.instance, record.checkpoint_txg))
            {
                rewritten.new_tree_table = root.tree_table;
                rewritten.new_mapping_root = root.mapping_root;
                rewritten.new_tree_identifier_watermark = root.tree_identifier_watermark;
                rewritten.new_rollback_floor = root.rollback_floor;
                rewritten.new_instance_table = root.instance_table;
                rewritten.new_allocation_record_tree_root = root.allocation_record_tree_root;
                rewritten.new_unmount_marker = root.unmount_marker;
                rewritten.new_reclaim_threshold = root.reclaim_threshold;
                rewritten.new_pinned_ring_slot = root.pinned_ring_slot;
            }
            if let Some((previous_instance, previous_bytes)) = &rewritten_previous_record {
                if *previous_instance == record.instance {
                    rewritten.back_chain = back_chain_of(previous_bytes);
                }
            }
            rewritten_previous_record = if rewritten == record {
                None
            } else {
                let rewritten_bytes = rewritten.to_bytes();
                image
                    .devices
                    .get_mut(&device)
                    .expect("有这块盘")
                    .write(record_offset(counter, journal_ring_bytes), &rewritten_bytes);
                records_rewritten += 1;
                Some((record.instance, rewritten_bytes))
            };
        }
    }
    records_rewritten
}

pub fn set_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

pub fn set_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

pub fn set_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub fn get_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

/// 坏镜像：在一份干净镜像上改一处。
pub type Mutation = Box<dyn Fn(&mut MemoryPool)>;

pub const INODE_RECORD_BLOCKS_OFFSET: usize = 48;

pub const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;

pub const SLOT: u64 = 16384;

pub const DEVICES: [u32; 2] = [0, 1];

/// 写完新池新建文件之后单元区里的单元：(起点槽, 字节数)。分配记录树按位置寻址（D8（核心索引结构） 已定项 14）：4 GiB 两块盘上根在第 2 层，
/// A 写出它的五个节点——两块盘各自的叶 61（50245、50246）、两块盘各自的第 1 层节点 0（50247、50248）与根（50249），先叶后根。
pub const UNITS: [(u64, usize); 14] = [
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

const INSTANCE_TABLE: u64 = 50176;

/// mkfs 种下的第 0 版树表单元：第一个文件版本换下它之后只有第 0 代根还引用。
pub const GENESIS_TREE_TABLE: u64 = 50178;

pub const DATA_UNIT: u64 = 50180;

pub const EXTENT_ROOT: u64 = 50240;

pub const INODE_LEAF: u64 = 50242;

pub const INODE_ROOT: u64 = 50244;

/// 新池新建文件（A）写出的分配记录树两片叶（盘 0、盘 1 各自的叶 61）：发布 B 之后只有 A 的根还（经 A 的第 1 层节点）指着它们。
pub const ALLOCATION_LEAVES: [u64; 2] = [50245, 50246];

pub const ACCOUNTING_ROOT: u64 = 50250;

pub const TREE_TABLE: u64 = 50252;

pub fn read_bytes_of_the_image(
    pool: &MemoryPool,
    device: u32,
    offset: u64,
    length: usize,
) -> Vec<u8> {
    PoolReader::read(
        pool,
        DeviceIdentity(device),
        DeviceOffsetInBytes(offset),
        length,
    )
    .expect("读")
}

pub fn write_bytes_into_the_image(pool: &mut MemoryPool, device: u32, offset: u64, bytes: &[u8]) {
    pool.devices
        .get_mut(&DeviceIdentity(device))
        .expect("盘")
        .write(DeviceOffsetInBytes(offset), bytes);
}

/// 按类重封：先算载荷 CRC，再算头校验和（码 1：头 105、CRC 在 101；码 3：头 107、CRC 在 89；码 2：头 86 + 2k、CRC 在 76 + 2k）。
pub fn reseal_the_unit_header_and_payload(bytes: &mut [u8]) {
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
pub fn root_slot_offsets_of_both_devices() -> Vec<(u32, u64)> {
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

pub fn location_pattern(device: u32, slot: u64, checksum: u32) -> Vec<u8> {
    [
        device.to_le_bytes().to_vec(),
        slot.to_le_bytes()[..6].to_vec(),
        checksum.to_le_bytes().to_vec(),
    ]
    .concat()
}

pub fn replace_all(haystack: &mut [u8], needle: &[u8], replacement: &[u8]) -> bool {
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
            let mut bytes = read_bytes_of_the_image(pool, 0, container * SLOT, length);
            let mut touched = false;
            for device in DEVICES {
                touched |= replace_all(
                    &mut bytes,
                    &location_pattern(device, slot, old),
                    &location_pattern(device, slot, new),
                );
            }
            if touched {
                let before =
                    crc32_castagnoli(&read_bytes_of_the_image(pool, 0, container * SLOT, length));
                reseal_the_unit_header_and_payload(&mut bytes);
                for device in DEVICES {
                    write_bytes_into_the_image(pool, device, container * SLOT, &bytes);
                }
                changes.push((container, before, crc32_castagnoli(&bytes)));
            }
        }
        for (device, offset) in root_slot_offsets_of_both_devices() {
            let mut bytes = read_bytes_of_the_image(pool, device, offset, 512);
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
                write_bytes_into_the_image(pool, device, offset, &bytes);
            }
        }
    }
    copy_each_root_into_the_new_root_segment_of_its_journal_records(pool);
}

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
    let before = read_bytes_of_the_image(pool, device, slot * SLOT, length);
    let mut bytes = before.clone();
    change(&mut bytes);
    reseal_the_unit_header_and_payload(&mut bytes);
    write_bytes_into_the_image(pool, device, slot * SLOT, &bytes);
    let (old, new) = (crc32_castagnoli(&before), crc32_castagnoli(&bytes));
    let mut parents_changed = Vec::new();
    for (container, container_length) in units.iter().copied() {
        if container == slot {
            continue;
        }
        let container_before = read_bytes_of_the_image(pool, 0, container * SLOT, container_length);
        let mut container_bytes = container_before.clone();
        if replace_all(
            &mut container_bytes,
            &location_pattern(device, slot, old),
            &location_pattern(device, slot, new),
        ) {
            reseal_the_unit_header_and_payload(&mut container_bytes);
            for each_device in DEVICES {
                write_bytes_into_the_image(pool, each_device, container * SLOT, &container_bytes);
            }
            parents_changed.push((
                container,
                crc32_castagnoli(&container_before),
                crc32_castagnoli(&container_bytes),
            ));
        }
    }
    for (root_device, offset) in root_slot_offsets_of_both_devices() {
        let mut root_bytes = read_bytes_of_the_image(pool, root_device, offset, 512);
        if replace_all(
            &mut root_bytes,
            &location_pattern(device, slot, old),
            &location_pattern(device, slot, new),
        ) {
            let digest = wide_checksum_with_field_zeroed(&root_bytes, 512, 138);
            root_bytes[138..170].copy_from_slice(&digest);
            write_bytes_into_the_image(pool, root_device, offset, &root_bytes);
        }
    }
    propagate(pool, units, parents_changed);
}

/// 改一个单元（两盘一起改）：`reseal` 为真时重封它的两道校验和；新的整单元校验和沿引用链补上去。
pub fn mutate_unit_of(
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
    let before = read_bytes_of_the_image(pool, 0, slot * SLOT, length);
    let mut bytes = before.clone();
    change(&mut bytes);
    if reseal {
        reseal_the_unit_header_and_payload(&mut bytes);
    }
    for device in DEVICES {
        write_bytes_into_the_image(pool, device, slot * SLOT, &bytes);
    }
    propagate(
        pool,
        units,
        vec![(slot, crc32_castagnoli(&before), crc32_castagnoli(&bytes))],
    );
}

/// 写完新池新建文件那份镜像上改一个单元。
pub fn mutate_unit(pool: &mut MemoryPool, slot: u64, change: impl Fn(&mut Vec<u8>), reseal: bool) {
    mutate_unit_of(pool, &UNITS, slot, change, reseal);
}

/// 分配记录树的两片叶（两块盘各一片）逐片改、各自重封，新的整单元校验和沿引用链补上去（叶 → 那块盘的第 1 层节点 → 根 → 树表 → 根槽）；
/// `change` 返回它在那一片叶里改了几条记录，这里交回两片合起来的条数。
pub fn mutate_allocation_leaves(
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

/// 系统配置字段表（`.claude/kb/layout/01-first-txn.md` 一）：加密类型在整槽校验和（155，宽 32）、自身 MAC（16）、nonce 水位（12）、
/// KDF 标识（4）之后，偏移 219；第一版只读 0（不加密）。
const SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET: usize = 219;

/// 登记过的加密算法，第一版读者不读加密的卷：带它的系统配置槽自证过也整池拒绝挂载。
const ENCRYPTION_TYPE_THE_FIRST_VERSION_DOES_NOT_READ: u8 = 1;

/// I-7.13（系统配置池级字段在读者收的范围里） 那份坏镜像改的那一槽：盘 1 的槽 0。
pub const DEVICE_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT: u32 = 1;

pub const OFFSET_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT: u64 = 0;

/// 系统配置自举头里的本盘设备号（4 字节）：fsid（偏移 102，宽 16）之后 20 字节、再 1 字节之后，偏移 139。
const SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_OFFSET: usize = 139;

/// 系统配置槽重封：自证校验和在 155，罩整槽 4096。
pub fn mutate_system_configuration_slot(
    pool: &mut MemoryPool,
    device: u32,
    slot_offset: u64,
    change: impl Fn(&mut Vec<u8>),
) {
    let mut bytes = read_bytes_of_the_image(pool, device, slot_offset, 4096);
    change(&mut bytes);
    let digest = wide_checksum_with_field_zeroed(&bytes, 4096, 155);
    bytes[155..187].copy_from_slice(&digest);
    write_bytes_into_the_image(pool, device, slot_offset, &bytes);
}

/// 两块盘对调盘体：每块盘两个系统配置槽里的本盘设备号改成另一块盘的身份（盘 0 的两槽写 1、盘 1 的两槽写 0），整槽重封，
/// 别的字节不动。与盘体真对调不同的只有这一格：根环区域、单元都还在原来的盘上，所以别的不变量照判成立。
pub fn swap_the_own_device_numbers_of_the_two_devices(image: &mut MemoryPool) {
    let spacing = u64::from(
        parameters_of_the_common_test_pool()
            .geometry
            .fixed_structure_slot_spacing,
    );
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

/// 树表单元里第 index 条条目（条目区从 115 + 2 × 8 = 131 起，每条 200）。
pub fn tree_table_entry_offset(index: usize) -> usize {
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
pub fn narrow_the_entry_width_to_the_key_width(bytes: &mut [u8]) {
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
pub const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;

/// extent 树上段叶（key 宽 24）里第一条条目内联的那条数据指针中诞生代号那 8 字节的偏移：
/// 条目区从 115 + 2 × 24 = 163 起，条目 = key (0, inode 8, 0) 24 + 标签 1（2 = 单单元内联）+ 数据指针 88
/// （D8（核心索引结构） 已定项 14），指针里诞生代号在 42。
pub const EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET: usize = 163 + 24 + 1 + 42;

/// 分配记录树叶（key 宽 10）：条目数在 82 + 2 × 10、条目区从 115 + 2 × 10 起，每条 20 = key (设备 4, 槽号 6) +
/// value (跨度段 2，最高位是已释放标志, 分配代或释放代 8)。
pub const ALLOCATION_NODE_ENTRY_COUNT_OFFSET: usize = 102;

pub const ALLOCATION_NODE_ENTRY_START: usize = 135;

pub const ALLOCATION_RECORD_BYTES: usize = 20;

pub const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;

/// 中央映射树（根兼叶，key 宽 27、条目 55）里唯一那条码 1 条目：key 的出生 txg（key 偏移 9）从 `from` 改成 `to`。
/// 码 1 排在最前（类标签 1 最小），头里的最小 key 就是它的 key，跟着改；改完仍按 key 升序。
pub fn rewrite_the_data_unit_mapping_key_birth_txg(bytes: &mut [u8], from: u64, to: u64) {
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

/// 分配记录树叶里槽号是 `slot` 的记录（两盘各一条）跨度改成 `span_slots`，已释放标志照留；返回改了几条。
pub fn set_allocation_record_span(bytes: &mut [u8], slot: u64, span_slots: u16) -> usize {
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

pub fn known_bad_images(clean: &MemoryPool) -> Vec<(&'static str, Mutation)> {
    let leaf_head_checksum =
        crc32_castagnoli(&read_bytes_of_the_image(clean, 0, INODE_LEAF * SLOT, 16384));
    let stray_node = {
        let mut stray = read_bytes_of_the_image(clean, 0, EXTENT_ROOT * SLOT, 16384);
        set_u64(&mut stray, 42, 65624);
        reseal_the_unit_header_and_payload(&mut stray);
        stray
    };
    let orphan_with_instance_two = {
        let mut orphan = read_bytes_of_the_image(clean, 0, DATA_UNIT * SLOT, 32768);
        orphan[91..95].copy_from_slice(&2u32.to_le_bytes());
        reseal_the_unit_header_and_payload(&mut orphan);
        orphan
    };
    let orphan_for_second_sentence = orphan_with_instance_two.clone();
    let record_with_instance_two = {
        let offset =
            singlefs_core::journal::record_offset(3, singlefs_format::JOURNAL_RING_DEFAULT_BYTES).0;
        let mut record = read_bytes_of_the_image(clean, 0, offset, 4096);
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
                let mut unit = read_bytes_of_the_image(image, 0, GENESIS_TREE_TABLE * SLOT, 16384);
                set_u64(&mut unit, 42, 77);
                reseal_the_unit_header_and_payload(&mut unit);
                for device in DEVICES {
                    write_bytes_into_the_image(image, device, GENESIS_TREE_TABLE * SLOT, &unit);
                }
            }),
        ),
        // 同一个单元的头抹成零、不重封：从第 0 代根出发的遍历读不到它。
        (
            "I-4.8",
            Box::new(|image: &mut MemoryPool| {
                let erased = vec![0u8; 16384];
                for device in DEVICES {
                    write_bytes_into_the_image(image, device, GENESIS_TREE_TABLE * SLOT, &erased);
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
        // 盘 1 被择的那个系统配置槽（两槽里世代号大的那一槽，按盘上现读）里 fsid 改成别的池的（一块外来的旧盘）；单元头照旧。
        (
            "I-1.4",
            Box::new(|image: &mut MemoryPool| {
                let [(chosen_slot_offset, _), _] =
                    system_configuration_slots_chosen_first(image, 1);
                mutate_system_configuration_slot(image, 1, chosen_slot_offset, |bytes| {
                    bytes[102] ^= 0xFF
                })
            }),
        ),
        // 盘 1 被择的那个系统配置槽里写入者身份的实现标识 16 字节清零、整槽重封（C868）。
        (
            SYSTEM_CONFIGURATION_RECORDS_THE_WRITER_IDENTITY,
            Box::new(clear_the_writer_implementation_identifier_of_the_chosen_slot_of_device_one),
        ),
        // 数据单元（码 1、两槽）那条未释放的分配记录跨度改成 1，两盘各一条（C411）。
        (
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
            Box::new(narrow_the_record_of_the_data_unit_to_one_slot),
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
                let mut bytes = read_bytes_of_the_image(image, 0, DATA_UNIT * SLOT, 32768);
                bytes[200] ^= 1;
                write_bytes_into_the_image(image, 0, DATA_UNIT * SLOT, &bytes);
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
        // 中央映射里分配记录树根那一条，盘 0 那条位置条目改指槽 50243（压在 inode 叶容器的第二个槽上）。分配记录树根按映射认位置、
        // 树表里那条根指针只是提示（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3）：只改树表里的指针，走读按映射认回原槽、不重叠。
        (
            "I-5.1",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    MAPPING_ROOT,
                    |bytes| {
                        let mapping_entry_of_the_allocation_record_tree_root =
                            (0..usize::from(u16::from_le_bytes([bytes[136], bytes[137]])))
                                .map(|entry_index| 86 + 54 + 29 + entry_index * 55)
                                .find(|entry_start| {
                                    bytes[entry_start + 27 + 4..entry_start + 27 + 10]
                                        == ALLOCATION_RECORD_TREE_ROOT.to_le_bytes()[..6]
                                })
                                .expect("映射里有分配记录树根那一条");
                        let location = mapping_entry_of_the_allocation_record_tree_root + 27 + 4;
                        bytes[location..location + 6].copy_from_slice(&50243u64.to_le_bytes()[..6]);
                    },
                    true,
                );
            }),
        ),
        // 记账树里盘 0 的「不可回收」（统计量 3）记成 16384（C70）：空闲 + 已分配照旧等于单元区容量，三项之和多出 16384，
        // 不可回收也不再是第一版非 zoned 池的 0——两套增量各自对而和对不上的那一形。
        (
            FREE_ALLOCATED_AND_UNRECLAIMABLE_SUM_TO_THE_CAPACITY_AND_THE_COMMITTED_RESERVATION_MATCHES_ITS_CLOSED_FORM,
            Box::new(|image: &mut MemoryPool| {
                mutate_unit(
                    image,
                    ACCOUNTING_ROOT,
                    |bytes| adjust_accounting(bytes, STATISTIC_UNRECLAIMABLE_BYTES, 0, 16384),
                    true,
                )
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
                for (device, offset) in root_slot_offsets_of_both_devices() {
                    write_bytes_into_the_image(image, device, offset, &[0u8; 512]);
                }
            }),
        ),
        // inode 叶容器两份都改坏一个字节（不补校验和）：最新的根走不完。
        (
            "I-7.2",
            Box::new(|image: &mut MemoryPool| {
                for device in DEVICES {
                    let mut bytes = read_bytes_of_the_image(image, device, INODE_LEAF * SLOT, 32768);
                    bytes[5000] ^= 1;
                    write_bytes_into_the_image(image, device, INODE_LEAF * SLOT, &bytes);
                }
            }),
        ),
        // 盘 0 被择的那个系统配置槽（两槽里世代号大的那一槽，按盘上现读）里逐区域设备改成 [0, 0, 0]。
        (
            "I-7.6",
            Box::new(|image: &mut MemoryPool| {
                let [(chosen_slot_offset, _), _] =
                    system_configuration_slots_chosen_first(image, 0);
                mutate_system_configuration_slot(image, 0, chosen_slot_offset, |bytes| {
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
        // 校验和都过），两盘系统配置都还是实例 1 ⇒ ① 红。
        (
            "I-7.7",
            Box::new(move |image: &mut MemoryPool| {
                write_bytes_into_the_image(image, 0, 50302 * SLOT, &orphan_with_instance_two)
            }),
        ),
        // journal 环里一个空槽放一条实例代号 2 的记录（头校验和重封）⇒ ① 红：扫描方向要读 journal 环。
        (
            "I-7.7",
            Box::new(move |image: &mut MemoryPool| {
                write_bytes_into_the_image(
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
        // 盘 0 槽 1（偏移 4096）的实例代号改成 2，再放那份实例代号 2 的孤儿 ⇒ 各盘最大号 [2, 1] 不等而较大者有东西带着，② 红（① 成立）。
        // 改的是不是被择的那一槽不碍事：每盘的实例代号取它两槽里全部自证过的槽中最大的，取号写的轮数翻了被择的槽也照样判红。
        (
            "I-7.7",
            Box::new(move |image: &mut MemoryPool| {
                mutate_system_configuration_slot(image, 0, 4096, |bytes| {
                    bytes[477..481].copy_from_slice(&2u32.to_le_bytes())
                });
                write_bytes_into_the_image(image, 0, 50302 * SLOT, &orphan_for_second_sentence);
            }),
        ),
        // 在没人引用的槽 50300 上放一个树 ID 65624（高于水位 65548）的码 2 节点（头校验和过、诞生代号 3）。
        (
            "I-7.8",
            Box::new(move |image: &mut MemoryPool| write_bytes_into_the_image(image, 0, 50300 * SLOT, &stray_node)),
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
        // 它不是本实例第一条（本实例第一条是 w1、计数器 1），链值必须等于 CRC32C(w1 的 496 字节头)。
        (
            "I-8.6",
            Box::new(|image: &mut MemoryPool| {
                let offset = singlefs_core::journal::record_offset(
                    2,
                    singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
                )
                .0;
                let mut record = read_bytes_of_the_image(image, 0, offset, 4096);
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
                write_bytes_into_the_image(image, 0, offset, &record);
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
        // C113（扫描重建时多版单元的现行版本判定无输入） P2 的已发布谓词判不成已发布（i == 挂载根实例 ⇒ 要 b ≤ 挂载根的 txg）
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
        // I-1.13（扫描重建与遍历方向的索引相同） 的必红世界 ②（`scan_rebuild_agrees_with_the_walked_index.rs` 同一形）：A 的数据单元（50180）
        // 两份副本头里的对象 key（五元组，42..75）清零、不重封——头校验和对不上，扫描重建判它损坏、不认领，遍历方向照样经 extent 树引用它。
        // 读不出那个单元，I-2.1、I-4.8、I-7.2、I-7.4 跟着红，所以放在这一张表里（只判「它红」，不判「只红它」）。
        (
            SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
            Box::new(|image: &mut MemoryPool| {
                for device in DEVICES {
                    let mut first_sector = read_bytes_of_the_image(image, device, DATA_UNIT * SLOT, 512);
                    first_sector[DATA_UNIT_OBJECT_KEY].fill(0);
                    write_bytes_into_the_image(image, device, DATA_UNIT * SLOT, &first_sector);
                }
            }),
        ),
    ]
}

/// 码 1 单元头里对象 key 那一段（五元组：类标签 42、树 43、对象 51、对象出生代 59、锚点偏移 67，D18（块里携带什么信息） 已定项 7）。
const DATA_UNIT_OBJECT_KEY: std::ops::Range<usize> = 42..75;

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
    let data_unit = read_bytes_of_the_image(image, 0, DATA_UNIT * SLOT, DATA_UNIT_BYTES);
    for device in DEVICES {
        write_bytes_into_the_image(image, device, moved_to * SLOT, &data_unit);
    }
    let checksum = crc32_castagnoli(&data_unit);
    let mut parents_changed = Vec::new();
    for (container, length) in UNITS.iter().copied() {
        if container == DATA_UNIT {
            continue;
        }
        let before = read_bytes_of_the_image(image, 0, container * SLOT, length);
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
            reseal_the_unit_header_and_payload(&mut bytes);
            for device in DEVICES {
                write_bytes_into_the_image(image, device, container * SLOT, &bytes);
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

/// 记录头里反向链那 4 字节的偏移：事务段 78 + 事务号 8 + 提交标记 1 + 本次发布内序号 4（D23（journal 的角色与格式） 已定项 4 / 已定项 8）。
pub const JOURNAL_RECORD_BACK_CHAIN_OFFSET: usize = 91;

/// 系统配置自举头里写入者身份的实现标识那 16 字节：紧跟 fsid（偏移 102）16 字节（D22（单元原子性怎么合成） 已定项 9 的字段表）。
const SYSTEM_CONFIGURATION_WRITER_IMPLEMENTATION_IDENTIFIER: std::ops::Range<usize> = 118..134;

/// 盘 1 被择的那一槽（两槽里世代号大的那一槽，按盘上现读）写入者身份的实现标识 16 字节清零、整槽重封；版本那 4 字节与别的槽照原样。
pub fn clear_the_writer_implementation_identifier_of_the_chosen_slot_of_device_one(
    image: &mut MemoryPool,
) {
    let [(chosen_slot_offset, _), _] = system_configuration_slots_chosen_first(image, 1);
    mutate_system_configuration_slot(image, 1, chosen_slot_offset, |bytes| {
        assert_ne!(
            bytes[SYSTEM_CONFIGURATION_WRITER_IMPLEMENTATION_IDENTIFIER], [0u8; 16],
            "干净镜像上实现标识是 core 写的 singlefs-rs"
        );
        bytes[SYSTEM_CONFIGURATION_WRITER_IMPLEMENTATION_IDENTIFIER].fill(0);
    });
}

/// 写完新池新建文件那份镜像上，数据单元（码 1、32 KiB、两槽）那条未释放的分配记录跨度改成 1，两盘各一条（两片叶各一条），已释放标志照留。
pub fn narrow_the_record_of_the_data_unit_to_one_slot(image: &mut MemoryPool) {
    assert_eq!(
        mutate_allocation_leaves(image, &UNITS, ALLOCATION_LEAVES, |bytes| {
            set_allocation_record_span(bytes, DATA_UNIT, 1)
        }),
        2,
        "数据单元那一个落点，两盘各一条"
    );
}

/// 新池新建文件那一版树表里十一条按发号次序排：livelist 是第 4 条（从 0 数，号 65541）、稀疏旁表第 5 条（号 65542）。
pub const LIVELIST_TREE_TABLE_ENTRY_INDEX: usize = 4;

const SPARSE_SIDE_TABLE_TREE_TABLE_ENTRY_INDEX: usize = 5;

pub const TREE_KIND_LIVELIST: u16 = 6;

pub const TREE_KIND_SPARSE_SIDE_TABLE: u16 = 7;

/// 树表单元里 livelist 与稀疏旁表两条互换种类（条目偏移 10 的 2 字节）；树 ID、根指针不动。
pub fn swap_the_kinds_of_the_livelist_and_sparse_side_table_entries(tree_table: &mut [u8]) {
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

/// 新池新建文件写出的中央映射树根（bump 次序：分配记录树五个节点、记账树、映射树、树表）。
pub const MAPPING_ROOT: u64 = 50251;

/// 记账树叶里 (统计量, 设备) 那一行的值改成 value（偏移口径同 `adjust_accounting`）。
pub fn set_accounting(bytes: &mut [u8], statistic: u16, device: u32, value: u64) {
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
pub fn adjust_accounting(bytes: &mut [u8], statistic: u16, device: u32, delta: u64) {
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

/// 分配记录树叶里槽号是 `slot`、**未带**已释放标志、分配代等于 `from` 的记录（两盘各一条）改成 `to`；返回改了几条。
pub fn rewrite_unreleased_allocation_generation(
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

/// 系统配置槽里世代号（8 字节）、设备数（4 字节）、槽距（4 字节，槽 1 在这个偏移上）的偏移（`.claude/kb/layout/01-first-txn.md` 一）。
pub const SYSTEM_CONFIGURATION_SLOT_GENERATION_OFFSET: usize = 147;

const SYSTEM_CONFIGURATION_SLOT_SPACING_OFFSET: usize = 429;

/// 一块盘上系统配置槽 1 的偏移：槽 0 记的槽距。
pub fn system_configuration_slot_one_offset(image: &MemoryPool, device: u32) -> u64 {
    let slot_zero = read_bytes_of_the_image(image, device, 0, 4096);
    u64::from(u32::from_le_bytes(
        slot_zero[SYSTEM_CONFIGURATION_SLOT_SPACING_OFFSET
            ..SYSTEM_CONFIGURATION_SLOT_SPACING_OFFSET + 4]
            .try_into()
            .expect("4 字节"),
    ))
}

/// 一块盘的两个系统配置槽：(偏移, 世代号)，被择的那一槽（世代号大的）在前。只在两槽都自证过的镜像上用（不核自证）。
/// 被择的是槽 0 还是槽 1 随取号写几轮、暖机几次翻（取号写两轮之后从槽 1 翻到槽 0），
/// 坏镜像要改被择的那一槽时按盘上现读的找，不写死偏移。
pub fn system_configuration_slots_chosen_first(image: &MemoryPool, device: u32) -> [(u64, u64); 2] {
    let slots = [0, system_configuration_slot_one_offset(image, device)].map(|slot_offset| {
        (
            slot_offset,
            get_u64(
                &read_bytes_of_the_image(image, device, slot_offset, 4096),
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

/// 新池新建文件写出的分配记录树根（`UNITS` 里 50249，先叶后根）。
pub const ALLOCATION_RECORD_TREE_ROOT: u64 = 50249;
