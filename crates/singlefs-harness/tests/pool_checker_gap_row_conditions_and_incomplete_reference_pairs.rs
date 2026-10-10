//! 池级 checker 的 I-3.5（引用区间的精确性） 出现规则里两处判法的钉（m3-root-ring-code-r2 判决 RS4，`research/prompts/m3-root-ring-code-r2-main-verification.md`
//! 第三节；正推腿探针 bb、bc、bd、be、bf、bg 在 `research/prompts/m3-root-ring-code-r2-sonnet-output.md` 第 2.2 节）：
//! - gap-row：候选集里相邻两版 (R⁻, R) 之间有洞时，只有最新根的实例表里有一行 (R⁻ 的实例, T = R⁻.txg)、R 的实例比 R⁻ 的大、两者之间每个实例都有一行
//!   且都是 (j, 0, 0)，这一对才照常判；三个子条件各缺一个，这一对都只判用户可见树之外的单元（N3）；
//! - 引用不全：R⁻ 或 R 的走读记过失败（引用集合数不全），这一对整对不判。
//!
//! 每份镜像都放进同一种坏东西：洞右边（或这一对后一版）新写的一个单元诞生代号改小到 R⁻ 的 txg——按 (a) 它该红。条件成立时它红（对照），
//! 条件缺一个时这一对不该判到它（N3 放过用户可见树里的单元、引用不全整对不判），I-3.5 不许红。把那个子条件放松的探针，这几份镜像就红。
//! 镜像改法与 `checker_judges_reference_intervals_and_every_new_root_segment_field.rs` 同一种（改一个单元、重封、沿引用链补到根槽），在这里另写一份。
mod common;

use common::{
    build_pool, parameters, publish_overwrite_in_process, tear_the_root_slot_written_for_txg,
    BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::{InvariantVerdict, REFERENCE_INTERVALS_ARE_EXACT};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::mount::mount_writable;
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::TREE_KIND_EXTENT;
use singlefs_core::recovery::PoolReader;
use singlefs_core::transaction::TransactionOutput;
use singlefs_core::unit::{UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED, UNIT_MAGIC};
use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES};
use singlefs_harness::memory_pool::MemoryPool;

const SLOT: u64 = 16384;
const DEVICES: [u32; 2] = [0, 1];
/// 单元区第一个槽（`common::build_pool` 那份 4 GiB 两盘池）。
const FIRST_UNIT_SLOT: u64 = 50176;
/// 找单元时每个槽读开头一个扇区看单元头。
const SECTOR_BYTES_READ_FOR_A_UNIT_HEADER: usize = 512;
/// 找单元时往后扫几个槽：这几份镜像上写过的单元都在这一段里。
const UNIT_SLOTS_SCANNED: u64 = 2048;
/// 根记录的自证校验和在 138 起 32 字节，罩整个 512 字节的根槽（D22（单元原子性怎么合成） 已定项 7）。
const ROOT_SLOT_BYTES: usize = 512;
const ROOT_RECORD_CHECKSUM_OFFSET: usize = 138;
/// 码 2 节点的 key 宽在 51。
const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
/// 单元头里诞生代号的偏移（D18（块里携带什么信息） 已定项 7 / 已定项 11）：码 1 在 75，码 3 在 73，码 2 在 52 + 2k。
const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;
const PACKED_UNIT_BIRTH_TXG_OFFSET: usize = 73;
const INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS: usize = 52;
/// 实例表单元（码 3 打包记录单元，D18（块里携带什么信息） 已定项 11）：记录数在 69，记录从 136 起、每条 88；
/// 一行 = 种类 1（行是 0）+ 实例代号 4 + 所选根的 checkpoint_txg 8 + W 8 + flags 1 + 预留 66。
const PACKED_UNIT_RECORD_COUNT_OFFSET: usize = 69;
const PACKED_UNIT_RECORDS_START: usize = 136;
const INSTANCE_TABLE_RECORD_BYTES: usize = 88;
const INSTANCE_TABLE_ROW_KIND: u8 = 0;
const INSTANCE_TABLE_ROW_INSTANCE: std::ops::Range<usize> = 1..5;
const INSTANCE_TABLE_ROW_PUBLISHED_TXG: std::ops::Range<usize> = 5..13;
/// 改坏一个单元时翻它载荷里的哪一个字节（头之后、条目区里，任一份读出来整单元校验和都对不上位置条目）。
const BYTE_INSIDE_THE_PAYLOAD_OF_A_NODE: u64 = 4000;

fn read(image: &MemoryPool, device: u32, offset: u64, length: usize) -> Vec<u8> {
    PoolReader::read(
        image,
        DeviceIdentity(device),
        DeviceOffsetInBytes(offset),
        length,
    )
    .expect("这几份镜像上读的都是盘内的落点")
}

fn write(image: &mut MemoryPool, device: u32, offset: u64, bytes: &[u8]) {
    image
        .devices
        .get_mut(&DeviceIdentity(device))
        .expect("这几份镜像都是两块盘")
        .write(DeviceOffsetInBytes(offset), bytes);
}

fn reference_interval_verdict(image: &MemoryPool) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(identifier, _)| *identifier == REFERENCE_INTERVALS_ARE_EXACT)
        .expect("池级 checker 的清单里有 I-3.5")
        .1
}

/// I-3.5 判违例，第一处违例的说明里有 `expected_in_the_detail`。
fn assert_the_reference_interval_invariant_is_violated_mentioning(
    image: &MemoryPool,
    expected_in_the_detail: &str,
) {
    match reference_interval_verdict(image) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains(expected_in_the_detail),
            "I-3.5 判违例，说明里该有「{expected_in_the_detail}」：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("I-3.5 应当判违例（说明里有「{expected_in_the_detail}」），实际 {other:?}")
        }
    }
}

/// 按类重封：先算载荷 CRC，再算头校验和（码 1：头 105、CRC 在 101；码 3：头 107、CRC 在 89；码 2：头 86 + 2k、CRC 在 76 + 2k）。
fn reseal_unit(bytes: &mut [u8]) {
    let key_width = usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
    let (header_end, payload_crc_offset) = match bytes[6] {
        UNIT_CLASS_DATA => (105usize, 101usize),
        UNIT_CLASS_PACKED => (107, 89),
        UNIT_CLASS_INDEX_NODE => (86 + 2 * key_width, 76 + 2 * key_width),
        other_class => panic!("这几份镜像上只有码 1、2、3 的单元，读到类 {other_class}"),
    };
    let payload_crc = crc32_castagnoli(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    singlefs_core::unit::seal_header_checksum(bytes, header_end);
}

fn set_birth_txg_in_the_unit_header(bytes: &mut [u8], birth_txg: u64) {
    let offset = match bytes[6] {
        UNIT_CLASS_DATA => DATA_UNIT_BIRTH_TXG_OFFSET,
        UNIT_CLASS_PACKED => PACKED_UNIT_BIRTH_TXG_OFFSET,
        UNIT_CLASS_INDEX_NODE => {
            INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS
                + 2 * usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET])
        }
        other_class => panic!("这几份镜像上只有码 1、2、3 的单元，读到类 {other_class}"),
    };
    bytes[offset..offset + 8].copy_from_slice(&birth_txg.to_le_bytes());
}

/// 盘 0 单元区开头那一段里写过的每个单元：(起点槽, 字节数)。两盘同槽（D2（RAID 条带策略） 已定项 10），读一块盘就够。
/// 迭代上界是扫的槽数；码 1 / 码 3 的单元占两槽，第二槽不当成单元头读。
fn units_on_disk(image: &MemoryPool) -> Vec<(u64, usize)> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    let mut units = Vec::new();
    let mut slot = FIRST_UNIT_SLOT;
    while slot < FIRST_UNIT_SLOT + UNIT_SLOTS_SCANNED {
        let header = read(image, 0, slot * SLOT, SECTOR_BYTES_READ_FOR_A_UNIT_HEADER);
        if header[..4] != UNIT_MAGIC {
            slot += 1;
            continue;
        }
        if header[6] == UNIT_CLASS_INDEX_NODE {
            units.push((slot, node_bytes));
            slot += 1;
        } else {
            units.push((slot, data_unit_bytes));
            slot += 2;
        }
    }
    units
}

/// 根环全部槽：(盘, 偏移)。区域 r 在盘 [0, 1, 0][r] 上，起点 64 × 16384 + r × 3 MiB，每区 8 槽、槽距 4096。
fn root_slots() -> Vec<(u32, u64)> {
    (0..3u64)
        .flat_map(|region| {
            (0..8u64).map(move |slot| {
                (
                    [0u32, 1, 0][usize::try_from(region).expect("区域号")],
                    64 * SLOT + region * 3 * (1 << 20) + slot * 4096,
                )
            })
        })
        .collect()
}

/// 一条位置条目：设备 4 + 槽号 6 + 整单元校验和 4。
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

/// 一个单元（两盘同槽）的整单元校验和从 old 变成 new：把引用它的位置条目逐个补上（单元里的、根槽里的），被补的单元重封、再往上补，直到根槽；
/// 最后把每条根照抄进同 (实例, txg) 的 journal 记录的新根段（`common` 的那一份）。
/// 迭代上界：每补一个单元，它自己的新校验和进待补的表一次；引用链是有向无环的（子先于父写），补到根槽就停。
fn propagate(image: &mut MemoryPool, units: &[(u64, usize)], mut changes: Vec<(u64, u32, u32)>) {
    while let Some((slot, old, new)) = changes.pop() {
        if old == new {
            continue;
        }
        for (container, length) in units.iter().copied() {
            if container == slot {
                continue;
            }
            let before = read(image, 0, container * SLOT, length);
            let mut bytes = before.clone();
            let mut touched = false;
            for device in DEVICES {
                touched |= replace_all(
                    &mut bytes,
                    &location_pattern(device, slot, old),
                    &location_pattern(device, slot, new),
                );
            }
            if touched {
                reseal_unit(&mut bytes);
                for device in DEVICES {
                    write(image, device, container * SLOT, &bytes);
                }
                changes.push((
                    container,
                    crc32_castagnoli(&before),
                    crc32_castagnoli(&bytes),
                ));
            }
        }
        for (device, offset) in root_slots() {
            let mut bytes = read(image, device, offset, ROOT_SLOT_BYTES);
            let mut touched = false;
            for location_device in DEVICES {
                touched |= replace_all(
                    &mut bytes,
                    &location_pattern(location_device, slot, old),
                    &location_pattern(location_device, slot, new),
                );
            }
            if touched {
                let digest = wide_checksum_with_field_zeroed(
                    &bytes,
                    ROOT_SLOT_BYTES,
                    ROOT_RECORD_CHECKSUM_OFFSET,
                );
                bytes[ROOT_RECORD_CHECKSUM_OFFSET..ROOT_RECORD_CHECKSUM_OFFSET + 32]
                    .copy_from_slice(&digest);
                write(image, device, offset, &bytes);
            }
        }
    }
    common::copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
}

/// 改一个单元（两盘一起改）：改完重封它的两道校验和，新的整单元校验和沿引用链补到根槽。
fn rewrite_unit_and_propagate(image: &mut MemoryPool, slot: u64, change: impl Fn(&mut Vec<u8>)) {
    let units = units_on_disk(image);
    let length = units
        .iter()
        .find(|(start, _)| *start == slot)
        .unwrap_or_else(|| panic!("槽 {slot} 是盘上写过的一个单元的起点"))
        .1;
    let before = read(image, 0, slot * SLOT, length);
    let mut bytes = before.clone();
    change(&mut bytes);
    reseal_unit(&mut bytes);
    for device in DEVICES {
        write(image, device, slot * SLOT, &bytes);
    }
    propagate(
        image,
        &units,
        vec![(slot, crc32_castagnoli(&before), crc32_castagnoli(&bytes))],
    );
}

/// 改坏一个单元（两盘各翻载荷里一个字节，不重封、不往上补）：引用它的位置条目里的整单元校验和对不上，走到它的那一版走读记一条失败。
fn corrupt_the_unit_on_both_devices(image: &mut MemoryPool, slot: u64) {
    for device in DEVICES {
        image.flip_byte(
            DeviceIdentity(device),
            DeviceOffsetInBytes(slot * SLOT),
            BYTE_INSIDE_THE_PAYLOAD_OF_A_NODE,
        );
    }
}

/// 两盘同槽：一条指针两条位置条目落在同一个槽号上，按一个槽号改两盘。
fn slot_shared_by_both_copies(pointer: &NodePointer) -> u64 {
    assert_eq!(
        pointer.locations[0].slot, pointer.locations[1].slot,
        "两盘同槽：这份镜像的改法按一个槽号改两盘"
    );
    pointer.locations[0].slot.0
}

fn root_slot_of_the_extent_tree(version: &TransactionOutput) -> u64 {
    let entry = version
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_EXTENT)
        .expect("这一版的树表里有 extent 树");
    slot_shared_by_both_copies(&entry.root)
}

fn data_unit_slot(version: &TransactionOutput) -> u64 {
    let [pointer] = version.data_pointers.as_slice() else {
        panic!(
            "这几版文件都只有一个数据单元：{} 条数据指针",
            version.data_pointers.len()
        );
    };
    assert_eq!(
        pointer.locations[0].slot, pointer.locations[1].slot,
        "两盘同槽：数据单元按一个槽号改两盘"
    );
    pointer.locations[0].slot.0
}

/// 覆盖写那几版的内容：每版一个数据单元装得下，字节各不相同。
fn content_of_the_overwrite(seed: usize) -> Vec<u8> {
    (0..4100usize)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

/// 新池新建文件 A (1, 3) 之后同一个进程里再覆盖写 `overwrites` 次（B (1, 4)、C (1, 5)……）：交回池与每一版（A 在最前）。
fn pool_through_overwrites(tag: &str, overwrites: usize) -> (BuiltPool, Vec<TransactionOutput>) {
    let mut pool = build_pool(tag);
    let mut versions = vec![pool.output.clone()];
    for overwrite in 0..overwrites {
        let previous = versions.last().expect("至少有 A").clone();
        let next = publish_overwrite_in_process(
            &mut pool,
            &previous,
            &content_of_the_overwrite(overwrite + 3),
            FIXED_WRITE_TIME_SECONDS + 60 * (u64::try_from(overwrite).expect("次数") + 1),
            InstanceGeneration(1),
        )
        .expect("覆盖写");
        pool.output = next.clone();
        versions.push(next);
    }
    (pool, versions)
}

/// 重开、可写挂载取下一个实例（写行、暖机），交回现行那一版（带文件）。
fn remount_writable_for_the_next_instance(pool: &mut BuiltPool) -> TransactionOutput {
    let mut devices_reopened = pool.reopen_recorded();
    let mounted =
        mount_writable(&parameters(), &mut devices_reopened).expect("重开取下一个实例：写行、暖机");
    pool.devices = Some(devices_reopened);
    pool.allocator = mounted.allocator;
    mounted
        .current
        .into_file_version()
        .expect("重开之前的现行那一版带文件")
}

/// 一段跨实例的历史：A (1, 3) 之后重开 `instances_after_the_first - 1` 次（每次写行、暖机），最后一个实例覆盖写一次 D。
/// 交回池、A、D，与 A 到 D 之间每一条根的 txg（它们的根槽之后一起改坏，候选集里 A 与 D 之间就是一个洞）。
struct HoleBetweenTheFirstVersionAndAnOverwriteOfALaterInstance {
    pool: BuiltPool,
    first: TransactionOutput,
    overwrite: TransactionOutput,
}

impl HoleBetweenTheFirstVersionAndAnOverwriteOfALaterInstance {
    fn build(tag: &str, instances_after_the_first: u32) -> Self {
        let mut pool = build_pool(tag);
        let first = pool.output.clone();
        let mut current = first.clone();
        for _ in 0..instances_after_the_first {
            current = remount_writable_for_the_next_instance(&mut pool);
        }
        let overwrite = publish_overwrite_in_process(
            &mut pool,
            &current,
            &content_of_the_overwrite(17),
            FIXED_WRITE_TIME_SECONDS + 120,
            InstanceGeneration(1 + instances_after_the_first),
        )
        .expect("最后一个实例覆盖写 D");
        assert!(
            overwrite.root.checkpoint_txg.0 > first.root.checkpoint_txg.0 + 1,
            "A 与 D 之间有别的实例写过的根"
        );
        Self {
            pool,
            first,
            overwrite,
        }
    }

    /// 干净镜像加两处改法：D 新写的数据单元诞生代号改小到 A 的 txg；`change_the_rows` 改 D 指着的那片实例表的行（重封、补到根槽）；
    /// 最后把 A 与 D 之间每一条根的根槽改坏。
    fn image_with_the_overwrite_unit_born_at_the_first_txg(
        &self,
        change_the_rows: impl Fn(&mut [u8]),
    ) -> MemoryPool {
        let mut image = self.pool.memory_pool();
        let first_txg = self.first.root.checkpoint_txg.0;
        rewrite_unit_and_propagate(&mut image, data_unit_slot(&self.overwrite), |bytes| {
            set_birth_txg_in_the_unit_header(bytes, first_txg);
        });
        rewrite_unit_and_propagate(
            &mut image,
            slot_shared_by_both_copies(&self.overwrite.root.instance_table),
            |bytes| change_the_rows(bytes),
        );
        for txg in first_txg + 1..self.overwrite.root.checkpoint_txg.0 {
            tear_the_root_slot_written_for_txg(&mut image, txg);
        }
        image
    }
}

/// 实例表单元里第 `index` 条记录的起点。
fn instance_table_record_offset(index: usize) -> usize {
    PACKED_UNIT_RECORDS_START + INSTANCE_TABLE_RECORD_BYTES * index
}

/// 实例表单元里实例 `instance` 那一行的起点（这几份历史都只有一片实例表）。
fn instance_table_row_of(table: &[u8], instance: u32) -> usize {
    let record_count = usize::from(u16::from_le_bytes([
        table[PACKED_UNIT_RECORD_COUNT_OFFSET],
        table[PACKED_UNIT_RECORD_COUNT_OFFSET + 1],
    ]));
    (0..record_count)
        .map(instance_table_record_offset)
        .find(|offset| {
            table[*offset] == INSTANCE_TABLE_ROW_KIND
                && table[offset + INSTANCE_TABLE_ROW_INSTANCE.start
                    ..offset + INSTANCE_TABLE_ROW_INSTANCE.end]
                    == instance.to_le_bytes()
        })
        .unwrap_or_else(|| panic!("实例表里有实例 {instance} 那一行"))
}

fn published_txg_of_the_row(table: &[u8], row: usize) -> u64 {
    u64::from_le_bytes(
        table[row + INSTANCE_TABLE_ROW_PUBLISHED_TXG.start
            ..row + INSTANCE_TABLE_ROW_PUBLISHED_TXG.end]
            .try_into()
            .expect("8 字节"),
    )
}

/// gap-row 的「有行 (R⁻ 的实例, T = R⁻.txg)」：A (1, 3) 之后重开取号 2（写行 (1, 3)、暖机）、覆盖写 D，实例 2 在 D 之前的根槽都改坏，
/// D 的数据单元诞生代号改小到 3。行是 (1, 3) 时接得上，这一对照常判、红在那个数据单元上（对照）；把那一行的 T 改成 4（A 仍按这一行有效：3 ≤ 4），
/// T 不等于 R⁻ 的 txg，接不上：数据单元在用户可见树里、N3 不判它，I-3.5 成立（D 的树表这些用户可见树之外的单元照 (a) 判过）。
/// 去掉「有行」整个条件（探针 bb）或只去掉「T = R⁻.txg」（探针 be）这一条红。
#[test]
fn a_hole_whose_row_for_the_previous_instance_names_another_txg_is_not_bridged_and_a_user_visible_unit_born_before_it_is_not_judged(
) {
    let history = HoleBetweenTheFirstVersionAndAnOverwriteOfALaterInstance::build(
        "gap-row-row-names-another-txg",
        1,
    );
    let first_txg = history.first.root.checkpoint_txg.0;
    let data_unit = data_unit_slot(&history.overwrite);
    let bridged = history.image_with_the_overwrite_unit_born_at_the_first_txg(|table| {
        let row = instance_table_row_of(table, 1);
        assert_eq!(
            published_txg_of_the_row(table, row),
            first_txg,
            "写行写的是 (1, A 的 txg)"
        );
    });
    assert_the_reference_interval_invariant_is_violated_mentioning(
        &bridged,
        &format!("槽 {data_unit}"),
    );
    let not_bridged = history.image_with_the_overwrite_unit_born_at_the_first_txg(|table| {
        let row = instance_table_row_of(table, 1);
        table[row + INSTANCE_TABLE_ROW_PUBLISHED_TXG.start
            ..row + INSTANCE_TABLE_ROW_PUBLISHED_TXG.end]
            .copy_from_slice(&(first_txg + 1).to_le_bytes());
    });
    assert_eq!(
        reference_interval_verdict(&not_bridged),
        InvariantVerdict::Holds,
        "行 (1, T) 的 T 不是 A 的 txg：洞接不上，用户可见树里的数据单元不判"
    );
}

/// gap-row 的「两者之间每个实例都有一行」：A (1, 3) 之后重开取号 2、再重开取号 3（各自写行、暖机）、实例 3 覆盖写 D，A 与 D 之间的根槽都改坏，
/// D 的数据单元诞生代号改小到 3。D 指着的实例表里实例 2 那一行（实例 3 挂载写的，T 不是 0）的实例号改成 9：实例 1 与 3 之间的实例 2 没有行，
/// 中间也就没有不是 (0, 0) 的行——只缺「每个实例都有一行」这一个子条件，接不上：数据单元不判，I-3.5 成立。实例 2 没有行，它那几条记录也就
/// 不算被哪一次恢复施加过、不并进候选集，D 的前一版是 A。去掉这个子条件（探针 bc）这一条红。
#[test]
fn a_hole_with_an_instance_between_that_has_no_row_is_not_bridged_and_a_user_visible_unit_born_before_it_is_not_judged(
) {
    let history = HoleBetweenTheFirstVersionAndAnOverwriteOfALaterInstance::build(
        "gap-row-instance-between-without-a-row",
        2,
    );
    let image = history.image_with_the_overwrite_unit_born_at_the_first_txg(|table| {
        let row = instance_table_row_of(table, 2);
        table[row + INSTANCE_TABLE_ROW_INSTANCE.start..row + INSTANCE_TABLE_ROW_INSTANCE.end]
            .copy_from_slice(&9u32.to_le_bytes());
    });
    assert_eq!(
        reference_interval_verdict(&image),
        InvariantVerdict::Holds,
        "实例 1 与实例 3 之间的实例 2 没有行：洞接不上，用户可见树里的数据单元不判"
    );
}

/// gap-row 的「中间的行都是 (j, 0, 0)」：同一段三个实例的历史，D 指着的实例表里实例 2 那一行（实例 3 挂载写的，T 是实例 2 最后那条根的 txg）的 T
/// 改成 1——行还在、不是 (0, 0)；T 改小是为了让实例 2 那几条记录按这一行判成被抛弃、不当成由记录施加出来的版本并进候选集（不改的话
/// 实例 2 最后那一版由记录施加出来，紧挨着 D 的前一版是它，那一对接得上、判的不是 A 与 D 之间的洞）。实例 1 与 3 之间那一行不是 (0, 0)，
/// 接不上：数据单元不判，I-3.5 成立。去掉这个子条件（探针 bd）这一条红。
#[test]
fn a_hole_whose_instance_between_has_a_row_that_is_not_zero_is_not_bridged_and_a_user_visible_unit_born_before_it_is_not_judged(
) {
    let history = HoleBetweenTheFirstVersionAndAnOverwriteOfALaterInstance::build(
        "gap-row-instance-between-not-zero",
        2,
    );
    let image = history.image_with_the_overwrite_unit_born_at_the_first_txg(|table| {
        let row = instance_table_row_of(table, 2);
        assert_ne!(
            published_txg_of_the_row(table, row),
            0,
            "实例 3 挂载写的实例 2 那一行记着它发布到的 txg"
        );
        table[row + INSTANCE_TABLE_ROW_PUBLISHED_TXG.start
            ..row + INSTANCE_TABLE_ROW_PUBLISHED_TXG.end]
            .copy_from_slice(&1u64.to_le_bytes());
    });
    assert_eq!(
        reference_interval_verdict(&image),
        InvariantVerdict::Holds,
        "实例 2 那一行不是 (0, 0)：洞接不上，用户可见树里的数据单元不判"
    );
}

/// 前一版引用不全：A (1, 3) → B (1, 4)，B 的数据单元诞生代号改小到 3。A 与 B 相邻、两版都走得完时它红在那个数据单元上（对照）；
/// 再把 A 的 extent 树根（只有 A 引用：B 覆盖写换了它）两盘都改坏，A 的走读记一条失败、引用集合数不全，(A, B) 这一对整对不判，
/// I-3.5 不许红。前一版引用不全不再跳过这一对（探针 bf）这一条红。
#[test]
fn a_pair_whose_previous_version_has_an_incomplete_reference_set_is_not_judged() {
    let (pool, versions) = pool_through_overwrites("reference-interval-previous-incomplete", 1);
    let (first, second) = (&versions[0], &versions[1]);
    let data_unit = data_unit_slot(second);
    let mut complete = pool.memory_pool();
    rewrite_unit_and_propagate(&mut complete, data_unit, |bytes| {
        set_birth_txg_in_the_unit_header(bytes, first.root.checkpoint_txg.0);
    });
    assert_the_reference_interval_invariant_is_violated_mentioning(
        &complete,
        &format!("槽 {data_unit}"),
    );
    let extent_root_of_the_first = root_slot_of_the_extent_tree(first);
    assert_ne!(
        extent_root_of_the_first,
        root_slot_of_the_extent_tree(second),
        "B 覆盖写换了 extent 树根：A 那一片只有 A 引用"
    );
    let mut incomplete = complete.clone();
    corrupt_the_unit_on_both_devices(&mut incomplete, extent_root_of_the_first);
    assert!(
        !matches!(
            reference_interval_verdict(&incomplete),
            InvariantVerdict::Violated(_)
        ),
        "A 的引用集合数不全，(A, B) 那一对不判：{:?}",
        reference_interval_verdict(&incomplete)
    );
}

/// 后一版引用不全：A (1, 3) → B (1, 4) → C (1, 5)，B 的树表（不在用户可见树里）诞生代号改小到 3。A 与 B 相邻、两版都走得完时它照 (a) 红（对照）；
/// 再把 B 的 extent 树根（只有 B 引用：C 覆盖写换了它）两盘都改坏，B 的走读记一条失败、引用集合数不全，(A, B) 这一对整对不判，
/// I-3.5 不许红。后一版引用不全不再跳过这一对（探针 bg）这一条红。
#[test]
fn a_pair_whose_later_version_has_an_incomplete_reference_set_is_not_judged() {
    let (pool, versions) = pool_through_overwrites("reference-interval-later-incomplete", 2);
    let (first, second, third) = (&versions[0], &versions[1], &versions[2]);
    let tree_table_of_the_second = slot_shared_by_both_copies(&second.root.tree_table);
    let mut complete = pool.memory_pool();
    rewrite_unit_and_propagate(&mut complete, tree_table_of_the_second, |bytes| {
        set_birth_txg_in_the_unit_header(bytes, first.root.checkpoint_txg.0);
    });
    assert_the_reference_interval_invariant_is_violated_mentioning(
        &complete,
        &format!("槽 {tree_table_of_the_second}（用户可见树 None）"),
    );
    let extent_root_of_the_second = root_slot_of_the_extent_tree(second);
    assert_ne!(
        extent_root_of_the_second,
        root_slot_of_the_extent_tree(third),
        "C 覆盖写换了 extent 树根：B 那一片只有 B 引用"
    );
    let mut incomplete = complete.clone();
    corrupt_the_unit_on_both_devices(&mut incomplete, extent_root_of_the_second);
    assert!(
        !matches!(
            reference_interval_verdict(&incomplete),
            InvariantVerdict::Violated(_)
        ),
        "B 的引用集合数不全，(A, B) 那一对不判：{:?}",
        reference_interval_verdict(&incomplete)
    );
}
