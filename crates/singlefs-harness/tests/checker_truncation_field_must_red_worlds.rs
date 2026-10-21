//! 池级 checker 为 inode 记录的截断字段新判、改判的几条（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R1、R3、R4、R6，
//! 第四节第 2 条，第六节用户定案）各配合法世界与必红世界：
//! I-9.7（记录预留段与截断字段）拆段——偏移 108–119 与 136–139 恒 0，池带 incompat 位 2 时截断字段判「T = 0 ⇒ N_w = 0」、不带时两段都要 0；
//! 扫描重建的读法 Z 第四条（I-1.13 那一侧，R3）；新立的 I-9.39（截断字段之外的活单元不早于它，R6）。
//!
//! 核心层写截断字段的那一半由别的实现员同时在写，这里不等它：池用真写路径造（顺序写、截断的意图、各批、变长），截断字段与 incompat 位 2
//! 按字节写进镜像——字段写进最新的根那一版第一个文件的记录、重封叶、补校验和到根（`common_deletion_in_progress::change_the_inode_record`），
//! incompat 位改每块盘两个系统配置槽、重封整槽校验和。系统配置最后改：改完之后核心层的择槽（`newest_root` 那一类辅助函数用它）不一定还收。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_truncation;

use common::{build_pool, parameters};
use common_deletion_in_progress::{
    change_the_inode_record, pool_with_a_three_unit_file, publish_new_inodes_and_advance_the_pool,
};
use common_pointer_head_corruption::{data_unit_bytes, unit_bytes_at};
use common_truncation::{
    batches_of_one, change_the_size_of_the_first_file_and_advance, payload_capacity,
    publish_the_next_truncation_step_and_advance, truncation_entry_of,
};
use singlefs_checker::image::{
    InvariantVerdict, LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT,
    SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET;
use singlefs_core::transaction::{TransactionOutput, FIRST_INODE_NUMBER};
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::memory_pool::MemoryPool;

/// inode 记录 140 里截断字段的两段（判决 R1）：T 在偏移 120、N_w 在偏移 128，各 8 字节小端；之前 108–119 是填充与 flags。
const INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET: usize = 120;
const INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET: usize = 128;
/// 截断字段之前那一段预留（填充 4、flags 8）的最后一个字节。
const INODE_RECORD_LAST_RESERVED_BYTE_BEFORE_THE_TRUNCATION_FIELD: usize = 119;
/// 系统配置槽里 incompat 位图第一个字节的偏移：magic 4 + 格式版本 2。
const FIRST_INCOMPAT_BYTE_OFFSET: usize = 6;
/// incompat 位 2：inode 记录带截断字段（D15（格式冻结政策） 已定项 4，用户 2026-10-17 定新开）。
const INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT: u8 = 0x04;
/// 码 1 单元头里诞生代号那 8 字节的偏移（D18（块里携带什么信息） 已定项 7 字段表）。
const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;
/// I-9.7 的编号。
const INODE_RECORD_RESERVED_BYTES_AND_TRUNCATION_FIELD: &str = "I-9.7";

fn read_u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

fn violations_of(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

fn violated_names(image: &MemoryPool) -> Vec<&'static str> {
    violations_of(image)
        .into_iter()
        .map(|(invariant, _)| invariant)
        .collect()
}

fn verdict_of(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(candidate, _)| *candidate == invariant)
        .map(|(_, verdict)| verdict)
        .expect("清单里有这一条")
}

fn assert_holds_every_invariant(image: &MemoryPool, what: &str) {
    let violations = violations_of(image);
    assert!(
        violations.is_empty(),
        "{what}：每一条都不该红，实际 {violations:?}"
    );
}

fn assert_judged_and_holding(image: &MemoryPool, invariants: &[&str], what: &str) {
    for invariant in invariants {
        assert_eq!(
            verdict_of(image, invariant),
            InvariantVerdict::Holds,
            "{what}：{invariant} 真判过且成立"
        );
    }
}

/// 最新的根那一版第一个文件的记录里截断字段写成 (T, N_w)，重封叶、补校验和。
fn write_the_truncation_field_of_the_first_file(
    image: &mut MemoryPool,
    truncation_field_txg: u64,
    truncation_field_size_in_bytes: u64,
) {
    change_the_inode_record(image, FIRST_INODE_NUMBER, |record| {
        record[INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET
            ..INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET + 8]
            .copy_from_slice(&truncation_field_txg.to_le_bytes());
        record[INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET
            ..INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET + 8]
            .copy_from_slice(&truncation_field_size_in_bytes.to_le_bytes());
    });
}

/// 每块盘两个系统配置槽（槽 0 在 0、槽 1 在槽距处，D22（单元原子性怎么合成） 已定项 16）incompat 第一个字节换成 `change` 交回的值，
/// 整槽校验和重封。
fn change_the_first_incompat_byte_of_every_system_configuration_slot(
    image: &mut MemoryPool,
    change: impl Fn(u8) -> u8,
) {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let slot_spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    for device in image.devices.values_mut() {
        for slot_offset in [DeviceOffsetInBytes(0), DeviceOffsetInBytes(slot_spacing)] {
            let mut slot = device.read(slot_offset, slot_bytes);
            slot[FIRST_INCOMPAT_BYTE_OFFSET] = change(slot[FIRST_INCOMPAT_BYTE_OFFSET]);
            let digest = wide_checksum_with_field_zeroed(
                &slot,
                slot_bytes,
                SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
            );
            slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            device.write(slot_offset, &slot);
        }
    }
}

fn set_incompat_bit_two(image: &mut MemoryPool) {
    change_the_first_incompat_byte_of_every_system_configuration_slot(image, |first_byte| {
        first_byte | INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT
    });
}

fn clear_incompat_bit_two(image: &mut MemoryPool) {
    change_the_first_incompat_byte_of_every_system_configuration_slot(image, |first_byte| {
        first_byte & !INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT
    });
}

/// 这一版第 `index` 个数据单元单元头里的诞生代号（盘 0 那一份，第一版两盘同槽同内容）。
fn birth_txg_of_the_data_unit(
    image: &MemoryPool,
    version: &TransactionOutput,
    index: usize,
) -> u64 {
    let pointer = &version.data_pointers[index];
    read_u64_at(
        &unit_bytes_at(image, pointer.locations[0].slot, data_unit_bytes()),
        DATA_UNIT_BIRTH_TXG_OFFSET,
    )
}

/// 三个单元（2.5 个净荷）在同一次顺序写里写下，交回池、写下它们的那一版与它们共同的诞生代号。
fn three_unit_file_born_together(tag: &str) -> (common::BuiltPool, TransactionOutput, u64) {
    let pool = pool_with_a_three_unit_file(tag);
    let written = pool.output.clone();
    let image = pool.memory_pool();
    let birth_txg = birth_txg_of_the_data_unit(&image, &written, 0);
    for index in 1..3 {
        assert_eq!(
            birth_txg_of_the_data_unit(&image, &written, index),
            birth_txg,
            "同一次顺序写的三个单元诞生代号相同"
        );
    }
    (pool, written, birth_txg)
}

/// 截断字段 (T, 0) 而三个单元都在 txg T 写下（截到 0 之后整份写回三个单元那一形的字段与单元）：单元 1、2 在截断字段之外、size 之内，
/// 诞生代号恰为 T、不早于它——I-9.39 判过且成立，扫描重建第四条不判死它们（出生 txg 不早于 T），I-1.13 成立。
#[test]
fn live_data_units_born_at_the_truncation_field_txg_hold_every_invariant() {
    let (pool, _, birth_txg) = three_unit_file_born_together("truncation-field-born-at-the-txg");
    let mut image = pool.memory_pool();
    write_the_truncation_field_of_the_first_file(&mut image, birth_txg, 0);
    set_incompat_bit_two(&mut image);
    assert_holds_every_invariant(&image, "截断字段 (T, 0)、单元都在 txg T 写下");
    assert_judged_and_holding(
        &image,
        &[
            INODE_RECORD_RESERVED_BYTES_AND_TRUNCATION_FIELD,
            LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT,
            SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
        ],
        "截断字段 (T, 0)、单元都在 txg T 写下",
    );
}

/// 必红世界：三个单元在 txg A 写下，之后再发一版（建一个空 inode，txg A + 1），记录的截断字段写成 (A + 1, 0)：单元 1、2 在截断字段之外、
/// size 之内，诞生代号 A 早于 T——只红 I-9.39；扫描重建第四条判死它们、遍历方向读得到，I-1.13 报不适用并点名 I-9.39（照 I-9.34 的先例）。
#[test]
fn a_live_data_unit_born_before_the_truncation_field_reddens_only_the_birth_txg_invariant() {
    let (mut pool, _, birth_txg) =
        three_unit_file_born_together("truncation-field-after-the-live-units");
    let advanced = publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let truncation_field_txg = birth_txg + 1;
    assert!(
        advanced.root.checkpoint_txg.0 >= truncation_field_txg,
        "截断字段的 txg 不晚于最新的那一版"
    );
    let mut image = pool.memory_pool();
    write_the_truncation_field_of_the_first_file(&mut image, truncation_field_txg, 0);
    set_incompat_bit_two(&mut image);
    assert_eq!(
        violated_names(&image),
        vec![LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT],
        "截断字段 (A + 1, 0)、单元 1、2 在 txg A 写下：只红 I-9.39，实际 {:?}",
        violations_of(&image)
    );
    match verdict_of(&image, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX) {
        InvariantVerdict::NotApplicable(reason) => assert!(
            reason.contains("I-9.39"),
            "I-1.13 不适用的理由点名 I-9.39：{reason}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("I-1.13 应当报不适用并点名 I-9.39，实际 {other:?}")
        }
    }
}

/// 一个单元的文件（新池新建文件，txg A）之后再发一版（txg A + 1），截断字段写成 (A + 1, 0)：⌈0 ÷ 净荷⌉.max(1) = 1，单元 0 不在截断字段之外，
/// 诞生代号早于 T 也是活的——两侧都不判死它，每一条都不红，I-1.13 判过且成立（去掉任一侧的 .max(1) 都会判死单元 0）。
#[test]
fn a_truncation_field_to_size_zero_keeps_the_first_data_unit_live() {
    let mut pool = build_pool("truncation-field-to-size-zero");
    let first_file = pool.output.clone();
    let image_of_the_first_file = pool.memory_pool();
    let birth_txg = birth_txg_of_the_data_unit(&image_of_the_first_file, &first_file, 0);
    assert_eq!(first_file.data_pointers.len(), 1, "新池新建文件一个单元");
    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let mut image = pool.memory_pool();
    write_the_truncation_field_of_the_first_file(&mut image, birth_txg + 1, 0);
    set_incompat_bit_two(&mut image);
    assert_holds_every_invariant(&image, "截断字段 (A + 1, 0)、一个单元的文件");
    assert_judged_and_holding(
        &image,
        &[
            LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT,
            SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
        ],
        "截断字段 (A + 1, 0)、一个单元的文件",
    );
}

/// 三个单元截到 100、批大小 1 的意图那一版，截断字段照用户定案写成（意图那次的 txg, 100）：size 之外还挂着的单元 1、2 诞生代号早于 T，
/// 它们是截断中间态等着放的单元、不是活单元，I-9.39 不判它们（「活单元」取 size 之内的读法），每一条都不红。
#[test]
fn the_intent_version_of_a_truncation_with_its_field_written_holds_every_invariant() {
    let (mut pool, _, _) = three_unit_file_born_together("truncation-field-on-the-intent");
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, 100);
    assert!(
        truncation_entry_of(&intent).is_some() && intent.data_pointers.len() == 3,
        "意图那一版：条目挂着、三个单元都在"
    );
    let mut image = pool.memory_pool();
    write_the_truncation_field_of_the_first_file(&mut image, intent.root.checkpoint_txg.0, 100);
    set_incompat_bit_two(&mut image);
    assert_holds_every_invariant(&image, "截断意图那一版带截断字段");
    assert_judged_and_holding(
        &image,
        &[
            LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT,
            SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
        ],
        "截断意图那一版带截断字段",
    );
}

/// 题面那一形（判决 R3）：三个单元（同一个诞生代号）截到恰好一个净荷（单元 0 不重写）、按批放完，再变长到三个净荷（之后到 size 是洞）。
/// 单元 1、2 还在盘上、诞生代号与单元 0 相同，写序那一条判不死它们：截断字段清零时扫描重建认回它们、I-1.13 红；
/// 截断字段写成（意图那次的 txg, 一个净荷）时第四条判死它们，每一条都不红、I-1.13 判过且成立。
#[test]
fn extending_after_a_truncation_at_a_data_unit_boundary_holds_the_scan_rebuild_only_with_the_truncation_field(
) {
    let (mut pool, written, _) = three_unit_file_born_together("truncation-field-at-a-boundary");
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, payload_capacity());
    assert!(
        truncation_entry_of(&intent).is_some()
            && intent.data_pointers.len() == 3
            && intent.data_pointers[0] == written.data_pointers[0],
        "意图那一版：条目挂着、三个单元都在、单元 0 没重写"
    );
    let mut steps = 0;
    while truncation_entry_of(&pool.output).is_some() {
        assert!(steps < 4, "批大小 1、放两个单元：几步之内放完");
        publish_the_next_truncation_step_and_advance(&mut pool, batches_of_one());
        steps += 1;
    }
    let extended = change_the_size_of_the_first_file_and_advance(&mut pool, 3 * payload_capacity());
    assert_eq!(
        extended.data_pointers.len(),
        1,
        "变长不写新单元：只剩单元 0，之后到 size 是洞"
    );
    let mut image_without_the_field = pool.memory_pool();
    write_the_truncation_field_of_the_first_file(&mut image_without_the_field, 0, 0);
    set_incompat_bit_two(&mut image_without_the_field);
    assert!(
        matches!(
            verdict_of(
                &image_without_the_field,
                SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX
            ),
            InvariantVerdict::Violated(_)
        ),
        "截断字段清零：扫描重建认回放掉的单元 1、2，I-1.13 红"
    );
    let mut image = pool.memory_pool();
    write_the_truncation_field_of_the_first_file(
        &mut image,
        intent.root.checkpoint_txg.0,
        payload_capacity(),
    );
    set_incompat_bit_two(&mut image);
    assert_holds_every_invariant(&image, "截在单元边界再变长、带截断字段");
    assert_judged_and_holding(
        &image,
        &[SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX],
        "截在单元边界再变长、带截断字段",
    );
}

/// 必红世界：截断字段之前那一段预留（偏移 108–119）的最后一个字节写成 1：只红 I-9.7。
#[test]
fn a_nonzero_byte_before_the_truncation_field_reddens_only_the_reserved_bytes_invariant() {
    let pool = build_pool("truncation-field-reserved-byte-before");
    let mut image = pool.memory_pool();
    change_the_inode_record(&mut image, FIRST_INODE_NUMBER, |record| {
        record[INODE_RECORD_LAST_RESERVED_BYTE_BEFORE_THE_TRUNCATION_FIELD] = 1;
    });
    assert_eq!(
        violated_names(&image),
        vec![INODE_RECORD_RESERVED_BYTES_AND_TRUNCATION_FIELD],
        "偏移 119 非零：只红 I-9.7，实际 {:?}",
        violations_of(&image)
    );
}

/// 必红世界：池带 incompat 位 2，记录的截断字段 T = 0 而 N_w = 5（没写过的字段两段都是 0）：只红 I-9.7。
#[test]
fn a_truncation_field_size_without_its_txg_reddens_only_the_reserved_bytes_invariant() {
    let pool = build_pool("truncation-field-size-without-txg");
    let mut image = pool.memory_pool();
    write_the_truncation_field_of_the_first_file(&mut image, 0, 5);
    set_incompat_bit_two(&mut image);
    assert_eq!(
        violated_names(&image),
        vec![INODE_RECORD_RESERVED_BYTES_AND_TRUNCATION_FIELD],
        "T = 0、N_w = 5：只红 I-9.7，实际 {:?}",
        violations_of(&image)
    );
}

/// 必红世界：与 [`live_data_units_born_at_the_truncation_field_txg_hold_every_invariant`] 同一份记录与单元，系统配置不带 incompat 位 2：
/// 截断字段必须全零，只红 I-9.7（扫描重建第四条与 I-9.39 在这份字段与单元上都不判死、不红）。
#[test]
fn a_truncation_field_without_incompat_bit_two_reddens_only_the_reserved_bytes_invariant() {
    let (pool, _, birth_txg) = three_unit_file_born_together("truncation-field-without-the-bit");
    let mut image = pool.memory_pool();
    write_the_truncation_field_of_the_first_file(&mut image, birth_txg, 0);
    clear_incompat_bit_two(&mut image);
    assert_eq!(
        violated_names(&image),
        vec![INODE_RECORD_RESERVED_BYTES_AND_TRUNCATION_FIELD],
        "不带 incompat 位 2、T ≠ 0：只红 I-9.7，实际 {:?}",
        violations_of(&image)
    );
}
