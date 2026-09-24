//! checker 的已知坏镜像语料（C13（checker 判定失效））：第一版判的 36 条不变量，每条配一份「改坏了正好触发它」的镜像。
//! 每份都从一份干净镜像出发（写完第一个事务的，或再覆盖写一次的），改一处；要让被改的那一条之前的判定不先挡住，
//! 被改的单元重封校验和，再把新的整单元校验和沿引用链一路补到根记录（父节点里的位置条目、中央映射条目、根槽的自证校验和）。

mod common;

use common::{
    build_pool, crash_state_devices, geometry, memory_pool_of_sparse_devices, parameters,
    publish_overwrite_in_process, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::mount::{
    mount_rollback, mount_writable, InstanceRow, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::PoolReader;
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionUnit,
};
use singlefs_format::{NODE_BYTES, TREE_IDENTIFIER_EXTENT};
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RetainedOperation, SharedStream};

const SLOT: u64 = 16384;
const DEVICES: [u32; 2] = [0, 1];
/// 写完第一个事务之后单元区里的单元：(起点槽, 字节数)。
const UNITS: [(u64, usize); 10] = [
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
];
/// 再覆盖写一次（发布 B，txg 4）之后单元区里的单元：A 的十个槽仍在盘上（根环里 A 的根还引用着它们），加上 B 写出的八个
/// （落点由 `second_transaction_step_one_overwrite.rs` 的验收钉住）。
const UNITS_AFTER_OVERWRITE: [(u64, usize); 18] = [
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
    (50250, 32768),
    (50252, 16384),
    (50253, 16384),
    (50254, 16384),
    (50255, 16384),
    (50256, 16384),
];
/// 发布 B 写出的分配记录树与树表（I-3.9 与 I-9.14 那两份坏镜像改的就是这两个单元）。
const ALLOCATION_TREE_AFTER_OVERWRITE: u64 = 50253;
const TREE_TABLE_AFTER_OVERWRITE: u64 = 50256;
const INSTANCE_TABLE: u64 = 50176;
/// mkfs 种下的第 0 版树表单元：第一个文件版本换下它之后只有第 0 代根还引用。
const GENESIS_TREE_TABLE: u64 = 50178;
const DATA_UNIT: u64 = 50180;
const EXTENT_ROOT: u64 = 50240;
const INODE_LEAF: u64 = 50242;
const INODE_ROOT: u64 = 50244;
/// 第一个事务（A）写出的分配记录树：发布 B 之后只有 A 的根还指着它。
const ALLOCATION_TREE: u64 = 50245;
const ACCOUNTING_ROOT: u64 = 50246;
const TREE_TABLE: u64 = 50248;

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
/// `units` 是这份镜像单元区里的单元表：写完第一个事务的是 `UNITS`，再覆盖写一次的是 `UNITS_AFTER_OVERWRITE`。
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

/// 写完第一个事务那份镜像上改一个单元。
fn mutate_unit(pool: &mut MemoryPool, slot: u64, change: impl Fn(&mut Vec<u8>), reseal: bool) {
    mutate_unit_of(pool, &UNITS, slot, change, reseal);
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
/// extent 树叶（key 宽 24）里第一条记录的数据指针中诞生代号那 8 字节的偏移：
/// 条目区从 115 + 2 × 24 = 163 起，记录 = key (树 8, inode 8, 偏移 8) + 数据指针 88，指针里诞生代号在 42。
const EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET: usize = 163 + 24 + 42;
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
        set_u64(&mut stray, 42, 99);
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
        // 在没人引用的槽 50300 上放一个树 ID 99 的码 2 节点（头校验和过、诞生代号 3）。
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
        // 它不是本实例第一条（本实例第一条是 w1、计数器 1），链值必须等于 CRC32C(w1 的 311 字节头)。
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
                )
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
                mutate_unit(
                    image,
                    ALLOCATION_TREE,
                    |bytes| {
                        assert_eq!(
                            rewrite_unreleased_allocation_generation(bytes, DATA_UNIT, 3, 4),
                            2,
                            "数据单元两盘各一条未释放记录，分配代本来是 3"
                        );
                    },
                    true,
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
    ]
}

/// 发布 B（txg 4）之后的干净镜像：A 的八个单元（十个落点）在这次发布里改写成已释放、释放代 4，树表是 B 写的那一版，
/// A 那一版仍被根环里 A 的根指着。I-3.9（释放代落在停止引用它的那一格区间里）与 I-9.14（树表条目的诞生 txg 跨根不变）
/// 要的跨根比较到这里才有内容：写完第一个事务的那份镜像上只有 A 的根有树表条目（mkfs 种的第 0 版树表是空的、
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
        // 发布 B 那次的释放代写成上一次发布的 txg（4 → 3）：A 的十个落点两盘各一条，十六条一起改。
        (
            "I-3.9",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit_of(
                    image,
                    &UNITS_AFTER_OVERWRITE,
                    ALLOCATION_TREE_AFTER_OVERWRITE,
                    |bytes| {
                        assert_eq!(
                            rewrite_released_generations(bytes, 4, 3),
                            16,
                            "A 的八个单元十个落点、两盘各一条"
                        );
                    },
                    true,
                );
            }),
        ),
        // 树表里 extent 树（树 ID 11）那条的诞生 txg 改成跟着这次发布的 txg（3 → 4）：A 写的那一版树表里仍是 3。
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
    ]
}

/// 两个各自提交了的事务撞号、而又紧挨着：从**发布 B 之后**的干净镜像出发（实例 1 有 A 的计数器 3、事务号 1，
/// B 的计数器 4、事务号 2），把 B 那一条的事务号改回 1、两盘各一份都改、头校验和重封。号不减、同号相连，
/// I-8.7（实例内事务号不重号） 分不出它与「一事务两条」；分得出的是提交标记——事务号 1 那一组两条都带 ⇒
/// 红的是 I-8.8（前缀里的事务不被切开） 的 ③。
fn two_committed_transactions_sharing_one_number_side_by_side(image: &mut MemoryPool) {
    set_transaction_number_of_journal_record(image, 4, 2, 1);
}

/// 记录头里 fsid 那 8 字节的偏移：事务段 78 + 事务号 8 + 提交标记 1 + 本次发布内序号 4 + 反向链 4 + 载荷校验和 4 + 新根段 188
/// （D23（journal 的角色与格式） 已定项 4 的字段表）。
const JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET: usize = 287;

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
fn expected_verdict_with_one_record_per_transaction(invariant: &str) -> InvariantVerdict {
    match invariant {
        "I-8.8" => InvariantVerdict::NotApplicable(COMMIT_MARKERS_NOT_APPLICABLE),
        _ => InvariantVerdict::Holds,
    }
}

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
            &["I-8.7", "I-8.8"],
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

/// 第一个事务那条根（txg 3）的根槽：区域 `3 mod 3` = 0（盘 0）的槽 `(3 div 3) mod 8` = 1（D22（单元原子性怎么合成）
/// 已定项 2 / 已定项 16；区域 0 起点 64 × 16384，槽距 4096）。
const FIRST_TRANSACTION_ROOT_SLOT: (u32, u64) = (0, 64 * SLOT + 4096);

/// I-7.3（环健康性） 的坏镜像，从 **mkfs 刚写完**那份镜像出发：三个区域槽 0 上那三条第 0 代根的 `checkpoint_txg`
/// 一起改成 1、各自重封自证校验和——「第一次发布把三个区域全盖了一遍」（轮换键算错，每次发布盖满整个环）在盘上的样子。
/// S 里代号全是 1 ⇒ 除代号最大者外一条更早代号的记录都没有，而 S 又不全是第 0 代 ⇒ 例外那一支够不着 ⇒ 必须红。
/// **为什么不建在写完第一个事务那份镜像上**：那一份上要让 S 只剩代号最大的一条，就得把 mkfs 那条第 0 代根也去掉，
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
/// 跨度改成 2（已释放标志照留），两盘各一条——它罩住的 50245 上另有一条记录（A 的分配记录树那个落点）。
/// 这是代码三方第二轮 Z1-d 在盘上的形状（复用时跨度变了、新记录罩住的旧记录留着）；树里的引用一个没动，I-5.1（物理范围不重叠） 看不见它。
fn known_bad_images_of_overlapping_allocation_records() -> Vec<(&'static str, Mutation)> {
    vec![
        // 改在 B 写的那棵账里（最新根下面）：两条都是已释放、释放代 4 的记录。
        (
            "I-5.4",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit_of(
                    image,
                    &UNITS_AFTER_OVERWRITE,
                    ALLOCATION_TREE_AFTER_OVERWRITE,
                    |bytes| {
                        assert_eq!(
                            set_allocation_record_span(bytes, INODE_ROOT, 2),
                            2,
                            "A 的 inode 树根那一个落点，两盘各一条"
                        );
                    },
                    true,
                );
            }),
        ),
        // 改在 A 写的那棵账里（只有 A 的根指着它，B 之后 A 的根仍在候选集里）：两条都是仍分配、分配代 3 的记录。
        // 「候选集里每条有效根」那个量词靠它：只判最新根那棵账的 checker 在这份镜像上判成立。
        (
            "I-5.4",
            Box::new(|image: &mut MemoryPool| {
                mutate_unit_of(
                    image,
                    &UNITS_AFTER_OVERWRITE,
                    ALLOCATION_TREE,
                    |bytes| {
                        assert_eq!(
                            set_allocation_record_span(bytes, INODE_ROOT, 2),
                            2,
                            "A 的 inode 树根那一个落点，两盘各一条"
                        );
                    },
                    true,
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
fn a_repeated_transaction_number_in_one_instance_reddens_the_uncut_transaction_invariant_and_only_when_separated_the_transaction_number_invariant(
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

/// C374 那两条：发布 B 之后的干净镜像上每条不变量都成立，两份坏镜像各自**只**红在自己那一条上
/// （别的不变量跟着红就说明镜像改宽了，判别力算不到这一条头上）。
#[test]
fn each_c374_bad_image_reddens_only_its_own_invariant_after_the_overwrite() {
    let clean = image_after_the_overwrite("known-bad-after-overwrite");
    for (invariant, found) in check_pool_image(&clean) {
        assert_eq!(
            found,
            expected_verdict_with_one_record_per_transaction(invariant),
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
/// 第二段是写完第一个事务那份镜像上的形态：S 只剩代号最大的那一条（除 txg 3 之外的根槽自证校验和都改坏）。
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
        assert_eq!(
            violated,
            [invariant],
            "判红的该只有 {invariant}：{verdicts:?}"
        );
        let differences: Vec<(&str, &InvariantVerdict, &InvariantVerdict)> = verdicts
            .iter()
            .zip(&verdicts_right_after_mkfs)
            .filter(|((name, found), (_, before))| *name != invariant && found != before)
            .map(|((name, found), (_, before))| (*name, before, found))
            .collect();
        assert!(
            differences.is_empty(),
            "除 {invariant} 之外每一条的判定都该与 mkfs 刚写完那一份相同：{differences:?}"
        );
    }
    // 写完第一个事务那份镜像上的同一个形态：S 只剩 txg 3 那一条。
    let after_the_first_transaction = built.memory_pool();
    assert_eq!(
        verdict(&after_the_first_transaction, "I-7.3"),
        InvariantVerdict::Holds,
        "写完第一个事务的干净镜像上 I-7.3 要真被评估过且成立"
    );
    let mut only_the_newest_root_left = after_the_first_transaction.clone();
    let mut corrupted_root_slots = 0;
    for (device, offset) in root_slots() {
        if (device, offset) == FIRST_TRANSACTION_ROOT_SLOT {
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

#[test]
fn the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target() {
    let clean = build_pool("known-bad").memory_pool();
    for (invariant, found) in check_pool_image(&clean) {
        let expected = match invariant {
            // 写完第一个事务的镜像上只有 A 的根有树表条目：没有一棵树的条目出现在两个树表单元里，跨根比不出来。
            // 这一条的坏镜像在发布 B 之后那一份上（`each_c374_bad_image_reddens_only_its_own_invariant_after_the_overwrite`）。
            "I-9.14" => InvariantVerdict::NotApplicable(
                "没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来",
            ),
            // 这一份上实例 1 只有 A 一条非 0 事务号的记录（写行与两次暖机空发布都写 0）：一对可比的号凑不出来。
            // 这一条的坏镜像与阳性对照都在发布 B 之后那一份上
            // （`a_repeated_transaction_number_in_one_instance_reddens_the_uncut_transaction_invariant_and_only_when_separated_the_transaction_number_invariant` 与
            // `each_transaction_boundary_bad_image_reddens_only_its_own_invariant_on_its_registered_criteria`）。
            "I-8.7" => InvariantVerdict::NotApplicable(
                "环里没有哪个实例写出过两条非 0 事务号的记录：一对可比的号都凑不出来",
            ),
            // 一事务一条、每条都带提交标记：③ ④ 没有对象。写行与两次暖机那两条事务号 0 的记录各带提交标记 1，
            // 拿事务号 0 成组的写法在这一份上就红（③：一组里两条带提交标记）——这一格是「事务号 0 不成组」那条射程的阳性对照。
            // 阳性对照与坏镜像在发布 B 之后那一份上
            // （`each_transaction_boundary_bad_image_reddens_only_its_own_invariant_on_its_registered_criteria`）。
            "I-8.8" => InvariantVerdict::NotApplicable(COMMIT_MARKERS_NOT_APPLICABLE),
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
        .chain(known_bad_images_of_the_allocation_generation().iter())
        .chain(known_bad_images_of_a_pure_leak().iter())
        .chain(known_bad_images_of_the_deferred_statistic().iter())
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

/// 第一个事务写出的中央映射树根（bump 次序：分配记录树、记账树、映射树、树表）。
const MAPPING_ROOT: u64 = 50247;
/// 根记录里中央映射树根指针的出生树：指针在根记录偏移 256（D22（单元原子性怎么合成） 已定项 7），出生树在指针偏移 34
/// （D19（块指针的结构与宽度预算） 已定项 11）。
const ROOT_RECORD_MAPPING_POINTER_BIRTH_TREE_OFFSET: usize = 256 + 34;
/// 这个池发过、又不是中央映射树的一个号（livelist 树 16），低于水位 19。
const ANOTHER_ISSUED_TREE_BELOW_THE_WATERMARK: u64 = 16;

/// 中央映射树不进树表：I-1.3（块头树 ID 一致） 对它的根，「实际引用它的树」按根记录里那条根指针的出生树取
/// （回退到树表 0 条的一版之后再发第一个文件版本，八棵树从水位重新发号，映射树不再恒是 15——C511（回退到无文件那一版之后诞生代怎么接））。
/// 两份坏镜像各只改一边、都只有 I-1.3 红：映射树根头里的树 ID 改成 16（指针仍说 15），与根记录里那条指针的出生树改成 16
/// （头仍说 15，重封根槽的自证校验和）。后一份钉的是「按指针判」这个读法：checker 退回写死 15 时它不红。
/// 取 16 不取一个大数：16 是这个池发过的号（livelist 树）、低于水位 19，改成水位之上的号会让 I-7.8（根记录树 ID 水位不低于全池最大树 ID）
/// 跟着红（它扫码 2 单元头），判别力就算不到 I-1.3 一条头上。
#[test]
fn a_central_mapping_root_whose_header_tree_differs_from_its_root_pointer_birth_tree_reddens_only_the_tree_identifier_invariant(
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
        let (device, offset) = FIRST_TRANSACTION_ROOT_SLOT;
        let mut root_slot = read(image, device, offset, 512);
        assert_eq!(
            get_u64(&root_slot, ROOT_RECORD_MAPPING_POINTER_BIRTH_TREE_OFFSET),
            15,
            "第一个事务那条根的映射树根指针说出生树 15"
        );
        set_u64(
            &mut root_slot,
            ROOT_RECORD_MAPPING_POINTER_BIRTH_TREE_OFFSET,
            ANOTHER_ISSUED_TREE_BELOW_THE_WATERMARK,
        );
        let digest = wide_checksum_with_field_zeroed(&root_slot, 512, 138);
        root_slot[138..170].copy_from_slice(&digest);
        write(image, device, offset, &root_slot);
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
        ["I-3.1", "I-3.11", "I-5.2", "I-9.6"],
        "跟着转成不适用的该只有这四条（走读在 I-1.10 那一步停下，记账条目不再解）：{verdicts:?}"
    );
}

/// 系统配置自述的区域数 R 越过字段表给逐区域设备身份留的那三个字段（偏移 379 / 383 / 387）时，
/// `geometry_of` 交回 [`singlefs_checker::Verdict::RegionCountPastTheRegionDeviceFields`]，不夹成 3、也不 panic
/// （panic 面普查 R12：`root_slot_positions` 拿这个数下标一个长 3 的数组）。
///
/// 写死的字节：槽里偏移 361 那一字节（R）从 3 改成 4，整槽自证校验和照新内容重算 ⇒
/// `check_system_configuration_slot` 那一道照样过，**只有**新加的这一道拦得住它。
#[test]
fn a_region_count_past_the_region_device_fields_is_refused_by_the_geometry_reader() {
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

/// 回退到 A 之后的干净镜像，与 `second_transaction_step_four_rollback.rs` 的
/// `rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content`
/// 同一段脚本（落点、行与各条根由那条验收钉住）：A (1, 3)、B (1, 4) → 重开取号 2、写行、暖机两次、C (2, 8) →
/// 重开回退到 A：取号 3、在 A 那一版实例表上写回退行 (1, 3, 0) 与中间实例行 (2, 0, 0)、发布 D (3, 9)、暖机 (3, 10)。
///
/// 这份镜像上最新根指着的实例表**有行**：A 的 T = 3 = 回退行的 Ti，仍在现行线上；B 与实例 2 的根被行判出局。
/// I-9.14（树表条目的诞生 txg 跨根不变） 的射程收窄（C511（回退到无文件那一版之后诞生代怎么接），2026-09-23 用户定案：
/// 只在同一条时间线内比）要在这里两头都判得出——跨过回退行、同一条线上的不一致照样红，被切掉的那条线上的不一致不比。
/// 发布 B 之后那份镜像（C374 那两份）上实例表一行都没有，这两头在那里分不出来。
struct ImageAfterTheRollbackToTheFirstRoot {
    image: MemoryPool,
    /// 回退目标 A 的 txg：它那一版树表里 extent 树那条的诞生 txg 就是它。
    rollback_target_txg: u64,
    /// 回退之后这条线上每条根各自那片树表单元的槽号（两盘同槽）：D 一片、暖机 (3, 10) 另写了一片。
    tree_table_slots_of_the_line_after_the_rollback: Vec<u64>,
    /// D 那次发布的 txg。
    rollback_publish_txg: u64,
    /// 被回退切掉的那条线上最后一条根 C 的树表单元槽号（两盘同槽），与它那次发布的 txg。
    tree_table_slot_of_the_abandoned_root: u64,
    abandoned_publish_txg: u64,
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
    let mut devices_reopened_for_the_rollback = pool.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut devices_reopened_for_the_rollback,
        RollbackTarget {
            instance: first.root.instance,
            checkpoint_txg: first.root.checkpoint_txg,
        },
        ShadowLedger::On,
    )
    .expect("回退到 A");
    pool.devices = Some(devices_reopened_for_the_rollback);
    let rollback_publish = rolled_back
        .output
        .row_publish
        .file_version()
        .expect("回退到带文件的 A：D 是带文件的一版");
    let mut tree_table_slots_of_the_line_after_the_rollback = vec![slot_shared_by_both_copies(
        rollback_publish.root.tree_table.locations,
    )];
    for warm_up in &rolled_back.output.warm_up_publishes {
        let slot = slot_shared_by_both_copies(
            warm_up
                .file_version()
                .expect("带文件的一版上的暖机")
                .root
                .tree_table
                .locations,
        );
        if !tree_table_slots_of_the_line_after_the_rollback.contains(&slot) {
            tree_table_slots_of_the_line_after_the_rollback.push(slot);
        }
    }
    ImageAfterTheRollbackToTheFirstRoot {
        image: pool.memory_pool(),
        rollback_target_txg: first.root.checkpoint_txg.0,
        tree_table_slots_of_the_line_after_the_rollback,
        rollback_publish_txg: rollback_publish.root.checkpoint_txg.0,
        tree_table_slot_of_the_abandoned_root: slot_shared_by_both_copies(
            third.root.tree_table.locations,
        ),
        abandoned_publish_txg: third.root.checkpoint_txg.0,
    }
}

/// 一片树表单元里 extent 树（树 ID 11，按树 ID 升序排第一条）那条的诞生 txg 从 `from` 改成 `to`，重封、沿引用链补到根槽。
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

/// 反向那一头（C511 收窄之后仍要红的那一格）：回退之后的这条线把 extent 树的诞生 txg 一致地记成 D 那次发布的 txg
/// （回退之后在现行线上重新建树的样子），而回退目标 A 那一版仍记 3。A 的 T = 3 = 回退行的 Ti，与 D、(3, 10) 在**同一条时间线上**
/// ⇒ I-9.14（树表条目的诞生 txg 跨根不变） 必须红，而且只红它。
///
/// 回退之后那两片树表都改：只改 D 的话 D 与 (3, 10) 之间也不一致，把 A 错判出局照样判得出，钉不住「A 在现行线上」这一条。
/// 两片都改之后唯一的不一致在 A 与回退之后这条线之间。只在 I-9.14 那一遍上把收窄做宽的（例如把「同一条时间线」读成
/// 「同一个实例」、只拿最新根那个实例的根比），I-9.14 就判不出这一格、这条用例红；C374 那份镜像上实例表一行都没有、
/// 只有一个实例，那样做宽它照样绿。做宽做在共用的候选集上的（连回退目标一起判出局），这条在干净镜像那一段先红在
/// I-3.1（已分配统计对得上） 上。
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
    for tree_table_slot in rolled_back
        .tree_table_slots_of_the_line_after_the_rollback
        .iter()
        .copied()
    {
        rewrite_the_extent_tree_birth_txg_in_tree_table(
            &mut image,
            tree_table_slot,
            rolled_back.rollback_target_txg,
            rolled_back.rollback_publish_txg,
        );
    }
    let verdicts = check_pool_image(&image);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(
        violated,
        ["I-9.14"],
        "回退目标 A 与回退之后这条线同在现行线上，诞生 txg 不一致只该红 I-9.14：{verdicts:?}"
    );
}

/// 正向那一头（C511 的收窄本身）：只有被回退切掉的那条线上的 C 把 extent 树的诞生 txg 记成它自己那次发布的 txg，
/// 现行线上 A、D、(3, 10) 仍一致记 3 ⇒ I-9.14（树表条目的诞生 txg 跨根不变） 不拿 C 比，判成立；别的每一条也都成立。
/// 这是回退到无文件那一版之后「再发一次第一个文件版本」重新建树那一格在盘上的关系（旧线与新线记的诞生 txg 不同），
/// 这里在回退到带文件的 A 那一格上造出来——回退到无文件那一版今天仍由挂载那道拒绝挡着（C511 的第 3 步没做）。
/// 把收窄去掉（I-9.14 拿根环里每一条根比，不按实例表判出局）这条就红。
#[test]
fn birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_rollback_is_not_compared_and_every_invariant_holds(
) {
    let rolled_back =
        image_after_rolling_back_to_the_first_root("known-good-after-rollback-abandoned-line");
    let mut image = rolled_back.image.clone();
    rewrite_the_extent_tree_birth_txg_in_tree_table(
        &mut image,
        rolled_back.tree_table_slot_of_the_abandoned_root,
        rolled_back.rollback_target_txg,
        rolled_back.abandoned_publish_txg,
    );
    let verdicts = check_pool_image(&image);
    let birth_invariant = verdicts
        .iter()
        .find(|(name, _)| *name == "I-9.14")
        .expect("清单里有 I-9.14");
    assert_eq!(
        birth_invariant.1,
        InvariantVerdict::Holds,
        "被回退切掉的 C 不与现行线比；现行线上 A 与回退之后那几片树表一致，I-9.14 要真被评估过且成立"
    );
    for (invariant, found) in &verdicts {
        assert_eq!(
            *found,
            expected_verdict_with_one_record_per_transaction(invariant),
            "只改了被抛弃根 C 那片树表里的一个诞生 txg，{invariant} 的判定"
        );
    }
}

/// 发布 B 写出的数据单元：它的分配记录在 B 那一版分配记录树（`ALLOCATION_TREE_AFTER_OVERWRITE`）里，未释放、分配代 4，
/// 单元头里的诞生代号也是 4（落点由 `second_transaction_step_one_overwrite.rs` 的验收钉住）。
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
            mutate_unit_of(
                image,
                &UNITS_AFTER_OVERWRITE,
                ALLOCATION_TREE_AFTER_OVERWRITE,
                |bytes| {
                    assert_eq!(
                        rewrite_unreleased_allocation_generation(
                            bytes,
                            DATA_UNIT_AFTER_OVERWRITE,
                            4,
                            3
                        ),
                        2,
                        "B 的数据单元两盘各一条未释放记录，分配代本来是 4"
                    );
                },
                true,
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
/// 两份纯泄漏坏镜像，建在写完第一个事务那份干净镜像上：Z3-A（只泄漏）与 Z3-B（泄漏 + defer 行跟着抬高）。
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

/// 候选 b（回退候选集补上「由记录施加出来、根槽从没落盘的那一版」，2026-09-23 用户定）不放宽 I-3.1 的等式、只补等式右边：
/// 纯泄漏的两份坏镜像照样都红 I-3.1，别的判定除 `INVARIANTS_EACH_PURE_LEAK_REDDENS` 登记的之外与干净镜像逐项相同
/// （Z3-A 让 I-5.2 照旧绿，是这份镜像要的样子）。
#[test]
fn a_pure_leak_that_keeps_free_plus_allocated_equal_to_the_unit_area_reddens_only_the_allocated_statistic_invariant(
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

/// 发布 B 写出的记账树（bump 次序：分配记录树 50253、记账树、映射树、树表 50256）：I-3.11 那几份镜像改的就是它。
const ACCOUNTING_ROOT_AFTER_OVERWRITE: u64 = 50254;
/// 发布 B 之后记账里每盘的「已分配」与「defer 待释放」（槽）：mkfs 3 + A 10 + B 10 = 23；A 的 10 槽 + A 换下的 mkfs 树表 1 槽 = 11
/// （`second_transaction_step_one_overwrite.rs` 的验收钉着同一对数）。最新根（B）走读到的是 23 − 11 = 12 槽：B 的 10 槽 + mkfs 那片实例表 2 槽。
const ALLOCATED_SLOTS_AFTER_OVERWRITE: u64 = 23;
const DEFERRED_SLOTS_AFTER_OVERWRITE: u64 = 11;

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
                        "改之前盘 0 的 defer 待释放是 11 槽"
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

/// I-3.11（已分配减 defer 等于最新根走读）：发布 B 之后的干净镜像上它真被评估过且成立（记账的两行先钉绝对值：已分配 23 槽、defer 11 槽），
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
/// （每盘已分配 −11 槽、空闲 +11 槽、defer 置 0）。这份基底上 I-3.11 与 I-5.2 成立、I-3.1 红（记账少于遍历，造它就是这样），
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
                    "挪之前盘 {device} 的 defer 待释放是 11 槽"
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
            "造出来的基底：盘 {device} 已分配 12 槽（= 最新根走读）、defer 0"
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
        "造出来的基底上 I-3.11 真被评估过且成立（已分配 12 − defer 0 = 最新根走读 12）"
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
        "defer 多记一槽：I-3.11 红（12 − 1 ≠ 12），I-3.1 照旧是基底那一份红：{verdicts:?}"
    );
    assert_eq!(
        verdicts_that_differ_outside(&verdicts, &verdicts_on_the_synthetic_base, &["I-3.11"]),
        [],
        "I-3.11 之外每一条的判定与造出来的基底逐项相同"
    );
}

/// 残留记录那一版的内容：与 A、B 都不同长、不同字节（具体哪些字节不承重）。
fn content_applied_only_by_its_journal_record() -> Vec<u8> {
    (0..3700usize)
        .map(|index| u8::try_from((index * 7 + 5) % 253).expect("小于 256"))
        .collect()
}

/// 一次可写挂载接在「由记录施加出来、根槽从没落盘」的那一版之后的镜像（增补 2 收口表第 54 行那段历史，
/// 与 `second_transaction_step_zero_layer0.rs` 残留记录那条流里实例 2 的根已落盘的那 12 个状态同形）：
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
        singlefs_harness::RecordingBlockDevice<singlefs_harness::crash::SparseBlockDevice>,
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
            is_rollback: false,
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
fn a_version_applied_only_by_its_journal_record_is_walked_so_the_allocated_statistic_holds_and_a_leak_on_top_still_reddens_it(
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
/// （`second_transaction_step_zero_layer0.rs` 残留记录那条流里「链接得到残留记录」的 19 个状态就是这一格）。
#[test]
fn a_journal_record_that_no_mount_has_applied_yet_is_not_walked_and_every_invariant_holds() {
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
fn a_version_applied_only_by_its_journal_record_leaves_the_walk_once_the_rollback_floor_is_raised_past_it(
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
fn a_version_applied_only_by_its_journal_record_leaves_the_walk_once_the_root_ring_has_turned_past_it_and_a_mount_reclaimed_its_units(
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

/// 发布 B（txg 4）的记录已落盘、根槽没落盘的崩溃镜像：B 的八个单元、两块盘上各一份的记录与记录之后那道屏障都在，
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
/// 已定项 17 的字段表：magic 4 + 类型 2 + 算法 1 + 填充 1 + 记录长 4 之后是点名项数；事务段 78 + 事务号 8 + 提交标记 1 +
/// 本次发布内序号 4 + 反向链 4 之后是载荷校验和，罩记录头之后的点名项）。
const JOURNAL_RECORD_NAMED_COUNT_OFFSET: usize = 12;
const JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET: usize = 95;
const JOURNAL_RECORD_HEADER_BYTES: usize = 311;
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
    mutate_unit_of(
        image,
        &UNITS_AFTER_OVERWRITE,
        ALLOCATION_TREE_AFTER_OVERWRITE,
        |bytes| {
            assert_eq!(
                rewrite_unreleased_allocation_generation(bytes, DATA_UNIT_AFTER_OVERWRITE, 4, 5),
                2,
                "B 的数据单元两盘各一条未释放记录，分配代本来是 4"
            );
        },
        true,
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
///    E 之前的读法在这张镜像上判成立，要等挂载之后才红；
/// ③ 这张坏镜像可写挂载，恢复择 A 的根、施加 B 那条记录（前缀 1 条，现行版是 txg 4），挂载之后 I-3.10 照样红：
///    坏镜像不是恢复会拒掉的样子，E 读的就是下一次挂载会施加的那一版。
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

    let mut devices = crash_state_devices(&image, &[], &[], &SharedStream::new());
    let mounted = mount_writable(&parameters(), &mut devices).expect("坏镜像照样可写挂载");
    assert_eq!(
        (
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.journal.prefix_applied
        ),
        (CheckpointTxg(3), CheckpointTxg(4), 1),
        "恢复择 A 的根 (1, 3)、施加 B 那条记录，现行版是 txg 4"
    );
    let image_after_the_mount = memory_pool_of_sparse_devices(&devices);
    assert!(
        matches!(
            verdict(&image_after_the_mount, "I-3.10"),
            InvariantVerdict::Violated(_)
        ),
        "挂载之后 B 那一版进了候选集，I-3.10 照样红：{:?}",
        check_pool_image(&image_after_the_mount)
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
/// `research/prompts/m2-wave3-code-r1-main-verification.md` 第三节 Y1、第四节第 1 条，被攻过零轮）的两道「不排除」，
/// 各配一份只红 I-7.8 的坏镜像：
/// ① **回退行不排除**：回退到暖机根 (1, 2)（实例 2，回退行 (1, 2, 0)）之后，(1, 3) 那一版写出的码 2 节点诞生代号 3 > 2，
///    按回退行的 T 看是「之后写的」，而它们发布过——被抛弃时间线的根发过号，水位要盖住它们（D8（核心索引结构） 已定项 8 ②）。
///    (1, 3) 发了 11..18，其中 16..18 只有树表条目、没有单元，而 (1, 3) 被抛弃、它的树表不走 ⇒ 扫出来最大的是映射树 15。
///    根环里的水位全压到 11，I-7.8 要红在 15 上。
/// ② **T = 0 的行不排除**：实例 2 在回退那个会话里发第一个文件版本（19..26），再回退到 (1, 2)（实例 3，回退行 (1, 2, 0) 与中间实例行 (2, 0, 0)）：
///    实例 2 写出的码 2 节点只由 (2, 0, 0) 那一行说到，行里说不出它发布到哪一代；扫出来最大的是它的映射树 23（24..26 只在树表里）。
///    根环里的水位全压到 19，I-7.8 要红在 23 上。
/// 两份干净镜像上一条都不红（水位 19、27 都高于各自盘上发过的号）。
#[test]
fn published_nodes_behind_a_rollback_row_or_an_intermediate_row_still_count_against_the_tree_identifier_watermark(
) {
    let mut pool = build_pool("known-bad-watermark-behind-rollback-and-intermediate-rows");
    let warm_up_root = RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(2),
    };
    let mut devices_for_the_first_rollback = pool.reopen_recorded();
    let first_rollback = mount_rollback(
        &parameters(),
        &mut devices_for_the_first_rollback,
        warm_up_root,
        ShadowLedger::On,
    )
    .expect("环里还留着 (1, 3) 时回退到暖机根 (1, 2)");
    pool.devices = Some(devices_for_the_first_rollback);
    assert_eq!(
        first_rollback.output.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(2),
            applied_transaction_high_water: 0,
            is_rollback: true,
        }],
        "第一次回退：实例 2 只写回退行 (1, 2, 0)"
    );
    assert_lowering_the_watermark_reddens_only_the_watermark_invariant(
        &pool.memory_pool(),
        11,
        15,
        "① 回退行后面的 (1, 3)",
    );

    let PoolVersion::WithoutFile(version) = &first_rollback.current else {
        panic!("回退到树表 0 条的暖机根：现行那一版仍是「没有文件版本」的一版")
    };
    let mut allocator = first_rollback.allocator.clone();
    let content: Vec<u8> = (0..3300usize)
        .map(|index| u8::try_from((index * 7 + 17) % 253).expect("小于 256"))
        .collect();
    let first_file_of_the_second_instance = {
        let publish_parameters = parameters();
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            &version.root,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
            },
            first_rollback.output.instance,
            &version.record_bytes,
        )
        .expect("实例 2 在回退之后发第一个文件版本")
    };
    assert_eq!(
        first_file_of_the_second_instance
            .tree_identifiers
            .highest()
            .0,
        26,
        "实例 2 从带过来的水位 19 起发 19..26"
    );
    let mut devices_for_the_second_rollback = pool.reopen_recorded();
    let second_rollback = mount_rollback(
        &parameters(),
        &mut devices_for_the_second_rollback,
        warm_up_root,
        ShadowLedger::On,
    )
    .expect("再回退到暖机根 (1, 2)：实例 2 那一版实例表里 (1, 2, 0) 的 T = 2，(1, 2) 仍可选");
    pool.devices = Some(devices_for_the_second_rollback);
    assert_eq!(
        second_rollback.output.rows_written,
        vec![
            InstanceRow {
                instance: InstanceGeneration(1),
                selected_root_txg: CheckpointTxg(2),
                applied_transaction_high_water: 0,
                is_rollback: true,
            },
            InstanceRow {
                instance: InstanceGeneration(2),
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
                is_rollback: false,
            }
        ],
        "第二次回退：实例 3 写回退行 (1, 2, 0) 与中间实例行 (2, 0, 0)，后者不带回退标志"
    );
    assert_lowering_the_watermark_reddens_only_the_watermark_invariant(
        &pool.memory_pool(),
        19,
        23,
        "② 中间实例行后面的实例 2",
    );
}
