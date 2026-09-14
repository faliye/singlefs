//! checker 的已知坏镜像语料（C13（checker 判定失效））：第一版判的 23 条不变量，每条配一份「改坏了正好触发它」的镜像。
//! 每份都从写完第一个事务的干净镜像出发，改一处；要让被改的那一条之前的判定不先挡住，被改的单元重封校验和，
//! 再把新的整单元校验和沿引用链一路补到根记录（父节点里的位置条目、中央映射条目、根槽的自证校验和）。

mod common;

use common::build_pool;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
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
const DATA_UNIT: u64 = 50180;
const EXTENT_ROOT: u64 = 50240;
const INODE_LEAF: u64 = 50242;
const INODE_ROOT: u64 = 50244;
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
fn propagate(pool: &mut MemoryPool, mut changes: Vec<(u64, u32, u32)>) {
    while let Some((slot, old, new)) = changes.pop() {
        if old == new {
            continue;
        }
        for (container, length) in UNITS {
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
fn mutate_unit(pool: &mut MemoryPool, slot: u64, change: impl Fn(&mut Vec<u8>), reseal: bool) {
    let length = UNITS
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
        vec![(slot, crc32_castagnoli(&before), crc32_castagnoli(&bytes))],
    );
}

fn verdict(pool: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(pool)
        .into_iter()
        .find(|(identifier, _)| *identifier == invariant)
        .expect("清单里有")
        .1
}

/// 超级块槽重封：自证校验和在 155，罩整槽 4096。
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

/// 树表单元里第 index 条条目（条目区从 115 + 2 × 8 = 131 起，每条 148）。
fn tree_table_entry_offset(index: usize) -> usize {
    131 + 148 * index
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
        // 盘 1 被择的那个超级块槽里 fsid 改成别的池的（一块外来的旧盘）；单元头照旧。
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
        // 盘 0 被择的那个超级块槽（槽 1，世代 5）里逐区域设备改成 [0, 0, 0]。
        (
            "I-7.6",
            Box::new(|image: &mut MemoryPool| {
                mutate_superblock_slot(image, 0, 4096, |bytes| {
                    bytes[379..391].copy_from_slice(&[0u8; 12])
                })
            }),
        ),
        // C322（取号那一步的屏障怎么放没有条款） 的撞号镜像：没人引用的槽 50302 上放一份写序实例代号 2 的数据单元（头与载荷
        // 校验和都过），两盘超级块都还是实例 1 ⇒ ① 红（旧的「各盘相等且 ≥ 根环」在这里判成立）。
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

#[test]
fn the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target() {
    let clean = build_pool("known-bad").memory_pool();
    for (invariant, found) in check_pool_image(&clean) {
        assert_eq!(
            found,
            InvariantVerdict::Holds,
            "干净镜像上 {invariant} 要成立"
        );
    }
    let cases = known_bad_images(&clean);
    let targets: std::collections::BTreeSet<&str> =
        cases.iter().map(|(invariant, _)| *invariant).collect();
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
