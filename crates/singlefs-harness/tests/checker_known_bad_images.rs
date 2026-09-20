//! checker 的已知坏镜像语料（C13（checker 判定失效））：第一版判的 29 条不变量，每条配一份「改坏了正好触发它」的镜像。
//! 每份都从一份干净镜像出发（写完第一个事务的，或再覆盖写一次的），改一处；要让被改的那一条之前的判定不先挡住，
//! 被改的单元重封校验和，再把新的整单元校验和沿引用链一路补到根记录（父节点里的位置条目、中央映射条目、根槽的自证校验和）。

mod common;

use common::{build_pool, publish_overwrite_in_process, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::recovery::PoolReader;
use singlefs_harness::crash::MemoryPool;

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

/// 系统配置槽重封：自证校验和在 155，罩整槽 4096。
fn mutate_superblock_slot(
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
                mutate_superblock_slot(image, 1, 4096, |bytes| bytes[102] ^= 0xFF)
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
                mutate_superblock_slot(image, 0, 4096, |bytes| {
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
                mutate_superblock_slot(image, 0, 4096, |bytes| {
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

/// C374 那两条：发布 B 之后的干净镜像上每条不变量都成立，两份坏镜像各自**只**红在自己那一条上
/// （别的不变量跟着红就说明镜像改宽了，判别力算不到这一条头上）。
#[test]
fn each_c374_bad_image_reddens_only_its_own_invariant_after_the_overwrite() {
    let clean = image_after_the_overwrite("known-bad-after-overwrite");
    for (invariant, found) in check_pool_image(&clean) {
        assert_eq!(
            found,
            InvariantVerdict::Holds,
            "发布 B 之后的干净镜像上 {invariant} 要真被评估过且成立"
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

#[test]
fn the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target() {
    let clean = build_pool("known-bad").memory_pool();
    for (invariant, found) in check_pool_image(&clean) {
        let expected = if invariant == "I-9.14" {
            // 写完第一个事务的镜像上只有 A 的根有树表条目：没有一棵树的条目出现在两个树表单元里，跨根比不出来。
            // 这一条的坏镜像在发布 B 之后那一份上（`each_c374_bad_image_reddens_only_its_own_invariant_after_the_overwrite`）。
            InvariantVerdict::NotApplicable(
                "没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来",
            )
        } else {
            InvariantVerdict::Holds
        };
        assert_eq!(found, expected, "干净镜像上 {invariant} 的判定");
    }
    let cases = known_bad_images(&clean);
    let targets: std::collections::BTreeSet<&str> = cases
        .iter()
        .chain(known_bad_images_after_the_overwrite().iter())
        .chain(known_bad_images_of_overlapping_allocation_records().iter())
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
