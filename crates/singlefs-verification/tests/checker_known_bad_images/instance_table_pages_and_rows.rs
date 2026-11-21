//! 实例表的页与行：被丢弃的根命名别的对象出生、未登记种类的记录、链记录、两页的实例表、行标志与发布代号。

use super::*;

/// 一条位置条目（盘 4 + 槽 6 + 整单元校验和 4，D19（块指针的结构与宽度预算） 已定项 11）里槽号那 6 字节的偏移。
const LOCATION_ENTRY_SLOT_OFFSET: usize = 4;

/// 实例表单元（码 3）占两槽的字节数。
const INSTANCE_TABLE_UNIT_BYTES: usize = 32768;

/// 根记录里实例代号（4 字节）与 checkpoint_txg（8 字节）的偏移（D22（单元原子性怎么合成） 已定项 7）。
const ROOT_RECORD_INSTANCE_OFFSET: usize = 24;

pub(super) const ROOT_RECORD_CHECKPOINT_TXG_OFFSET: usize = 28;

/// 根环里 (checkpoint_txg, 实例) 最大的那条根（择根序，D22（单元原子性怎么合成） 已定项 7）指着的实例表第 0 片的槽号（第一条位置条目，两盘同槽）。
fn instance_table_slot_of_the_newest_root(image: &MemoryPool) -> u64 {
    let newest_root = root_slot_offsets_of_both_devices()
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

/// I-9.29（同号只许一个出生代） 判不了那一形（`crates/singlefs-checker/invariants/09-inode树结构.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」）。
/// 镜像见
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

/// 码 3 打包单元头里声明长度（2 字节）与记录数（2 字节）的偏移，载荷 CRC（4 字节）的偏移与头校验和的端点（`reseal_the_unit_header_and_payload` 码 3 那一支的两个数；
/// D18（块里携带什么信息） 已定项 11）。
pub(super) const PACKED_UNIT_DECLARED_LENGTH_OFFSET: usize = 8;

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

/// N5-a：
/// 最新根的实例表在链指针记录之前多插一条没登记种类（kind 2）的记录，重封、补到根槽。D18（块里携带什么信息）
/// 已定项 11 补「kind 不是 0 或 1 ⇒ 这一片解不开（表不可读）」：
/// 主走读不收这一片的行、走不到末片，候选集滤不掉被抛弃的 C，I-9.29（同号只许一个出生代） 报部分没判。
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
/// 与坏镜像往那一格写的非 0 取值（同一项「第一版恒 0、非 0 拒收」）。
const INSTANCE_TABLE_ROW_FLAGS_OFFSET: usize = 21;

const INSTANCE_TABLE_ROW_FLAGS_NOT_ZERO: u8 = 1;

/// 攻方 N5 那一份基底上最新根的实例表那一片解不开（I-3.14 不成立）时判红的几条，按 `IMPLEMENTED_INVARIANTS` 的次序：
/// I-3.14 本身；走不到末片、那一片的行不收，被抛弃的 C 进了回退候选集，它那一版引用的单元算进遍历得到的已分配（I-3.1 与记账对不上）、
/// I-9.10 连带红。I-3.14 四形一律如此：引用集合那一遍（`singlefs_checker::walk` 的 `note_instance_table_pages_after_the_first`）
/// 四形都记数不全；下面是这四张镜像 `violated_invariants` 的原样。
/// I-3.21（走到的引用起点是最新根计入的分配记录的起点，C944）跟着红：C 那一版引用的单元起点（盘 0 槽 50184 起）在最新根的分配记录里
/// 没有一条起点在那里的 E(θ_high) 记录，与 I-3.1 红在同一批槽上（合入后验证任务 6223 量的：四张镜像 `violated_invariants` 多出的恰是这一条，
/// 说明点名盘 0 槽 50184）。
/// I-3.5（引用区间的精确性）跟着红：C（实例 2、txg 8）与最新根（实例 3、txg 9）成了相邻的一对，最新根引用、C 不引用的落点里有诞生代号
/// 不大于 C 的 txg、落在一棵整棵继承下来的用户可见树里的，(b) 前两半成立；谱系那一半不成立——C 是崩溃恢复抛掉的、不是回退，
/// 最新根的谱系树里没有一条 C.txg < T_rb ≤ 最新根 txg 的回退记录（`crates/singlefs-checker/invariants/05-空间记账.md` I-3.5 那一行：
/// (b) 三半都要，被崩溃恢复抛弃、又因实例表解不开留在候选集里的根与它后一版那一对照谱系那一半判）。
const VIOLATED_ONCE_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE: [&str; 5] = [
    "I-3.1",
    "I-9.10",
    REFERENCE_INTERVALS_ARE_EXACT,
    INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
    WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
];

/// 最新根的实例表第 0 片就解不开时，I-3.8（实例表行唯一且低于挂载根）一行都没收、报不适用的理由（`singlefs_checker::walk` 的
/// `walk_instance_table_chain`）。
const INSTANCE_TABLE_ROWS_NOT_JUDGED_AS_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE: &str =
    "最新根的实例表那一片解不开（I-3.14 不成立）、行没收全：没有行可判";

/// 最新根的实例表那一片解不开的四张镜像共用的判法：判红的恰好是 [`VIOLATED_ONCE_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE`]，
/// I-3.5 红在 C 与最新根那一对、落点在整棵继承下来的树里（红的是谱系那一半），
/// I-3.8 报不适用、理由是 [`INSTANCE_TABLE_ROWS_NOT_JUDGED_AS_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE`]。
fn assert_the_newest_instance_table_page_does_not_decode(image: &MemoryPool, what: &str) {
    let verdicts = check_pool_image(image);
    assert_eq!(
        violated_invariants(&verdicts),
        VIOLATED_ONCE_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE,
        "{what}：判红的该是 I-3.14 与 C 进候选集带出来的四条：{verdicts:?}"
    );
    match verdict(image, REFERENCE_INTERVALS_ARE_EXACT) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains("(实例 {2}、txg 8) → (实例 3、txg 9)")
                && detail.contains("整棵继承：true"),
            "{what}：I-3.5 红在被抛弃的 C（实例 2、txg 8）与最新根（实例 3、txg 9）那一对、落点在整棵继承下来的树里：{detail}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => panic!(
            "{what}：I-3.5 应当判违例（C 与最新根那一对谱系那一半不成立），实际 {other_verdict:?}"
        ),
    }
    assert_eq!(
        verdict(image, "I-3.8"),
        InvariantVerdict::NotApplicable(
            INSTANCE_TABLE_ROWS_NOT_JUDGED_AS_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE
        ),
        "{what}：最新根的实例表第 0 片解不开、一行都没收，I-3.8 报不适用并说明为什么"
    );
}

/// I-3.14（实例表每片记录形态合法） 的坏镜像，造法照 N5-a：
/// 最新根的实例表在链指针记录之前多插一条 kind 2，重封、补到根槽。实现读到它整张表解不开（恢复 `Failed`、可写挂载拒）；
/// 池级 checker 同一个读法：I-3.14 红（种类那一格），这一片的行不收、走不到末片，被抛弃的 C 滤不掉、进了回退候选集，
/// 它那一版的单元连带红 I-3.1、I-9.10，它与最新根那一对红 I-3.5 的谱系那一半；I-9.29 报部分没判，上面那一条用例钉着；引用集合那一遍同记数不全；
/// I-3.8 一行都没收、报不适用并给理由。
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

/// I-3.14 的坏镜像（「片中间多一条链指针」）：最新根的实例表在链指针记录之前多插一条一模一样的链指针记录（kind 1、无下一片），重封、补到根槽。
/// 最后一条仍是合法的链指针记录，I-3.14 红的是「链指针记录只许在一片末尾」那一格；这一片解不开，判红的另几条与 I-3.8 的理由同上一条用例。
/// 判别力：那一格恒判成立时只红那两条；这一片照收照走时只红 I-3.14。
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

/// I-3.14 的坏镜像（「kind 0 行 flags 非 0」）：最新根的实例表第一行的 flags 写成非 0（1），重封、补到根槽。
/// 实现读行时拒收（`InstanceRow::parse` 判 flags 非 0）；池级 checker 红 I-3.14 的 flags 那一格，这一片解不开，判红的另几条与 I-3.8 的理由同上。
/// 判别力：那一格恒判成立时只红那两条；这一片照收照走时只红 I-3.14。
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
                bytes[first_row_flags] = INSTANCE_TABLE_ROW_FLAGS_NOT_ZERO;
            },
            true,
        );
    assert_the_newest_instance_table_page_does_not_decode(
        &image,
        &format!("最新根的实例表（槽 {instance_table_slot}）第一行 flags 写 1"),
    );
}

/// C742（实例表没收全时行唯一的不适用理由没分清），任一片解不开就报不适用、第 0 片三种退出各点名理由：
/// 最新根的实例表第 0 片两块盘上那一份都把载荷 CRC 翻一位、不补引用链（根记录里那条指针的整单元校验和对不上），两份都读不到对得上的。
/// I-3.8 一行都没收，报不适用、理由点名第 0 片两份都读不到（不落到缺省理由「这个镜像上没有它判的对象」）。
/// 判别力（`crates/mutations.tsv` 里点名这条的行）：第 0 片读不出那一处出口不记停在哪时落回缺省理由。
#[test]
fn the_first_page_of_the_newest_instance_table_reading_back_from_neither_copy_leaves_the_row_invariant_not_applicable_naming_that_exit(
) {
    let mut image =
        image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth(
            "known-bad-instance-table-first-page-reads-back-from-neither-copy",
        );
    let instance_table_slot = instance_table_slot_of_the_newest_root(&image);
    for device in DEVICES {
        let mut bytes = read(
            &image,
            device,
            instance_table_slot * SLOT,
            INSTANCE_TABLE_UNIT_BYTES,
        );
        bytes[PACKED_UNIT_PAYLOAD_CRC_OFFSET] ^= 1;
        write(&mut image, device, instance_table_slot * SLOT, &bytes);
    }
    assert_eq!(
        verdict(&image, "I-3.8"),
        InvariantVerdict::NotApplicable(
            "最新根的实例表第 0 片两份都读不到对得上的、行没收全：没有行可判"
        ),
        "最新根的实例表（槽 {instance_table_slot}）两份都对不上整单元校验和：I-3.8 报不适用并点名第 0 片读不出"
    );
}

/// C742 第 0 片的第二种退出：最新根的实例表第 0 片头里的容器号（偏移 53）写成 1，重封、补到根槽。身份四元组不是 (0, 4, 0, 0)，
/// I-1.1 红、这一片不收；I-3.8 报不适用、理由点名第 0 片身份不对。
/// 判别力：身份那一处出口不记停在哪时落回缺省理由。
#[test]
fn the_first_page_of_the_newest_instance_table_naming_another_page_number_leaves_the_row_invariant_not_applicable_naming_that_exit(
) {
    let (image, instance_table_slot) =
        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
            "known-bad-instance-table-first-page-names-another-page-number",
            |bytes| set_u64(bytes, PACKED_UNIT_CONTAINER_NUMBER_OFFSET, 1),
            true,
        );
    assert_eq!(
        verdict(&image, "I-3.8"),
        InvariantVerdict::NotApplicable(
            "最新根的实例表第 0 片身份不对（不是链上第 0 片）、行没收全：没有行可判"
        ),
        "最新根的实例表（槽 {instance_table_slot}）容器号写成 1：I-3.8 报不适用并点名第 0 片身份不对"
    );
    assert!(
        matches!(verdict(&image, "I-1.1"), InvariantVerdict::Violated(_)),
        "身份四元组不对，I-1.1 红"
    );
}

/// C742 第 0 片的第三种退出：最新根的实例表第 0 片头里的记录宽（偏移 71）写成 89（登记的是 88），重封、补到根槽。码 3 容器那几条不成立，
/// I-1.7 红、记录取不出来；I-3.8 报不适用、理由点名第 0 片容器记录不合法。
/// 判别力：容器那一处出口不记停在哪时落回缺省理由。
#[test]
fn the_first_page_of_the_newest_instance_table_with_an_unregistered_record_width_leaves_the_row_invariant_not_applicable_naming_that_exit(
) {
    let (image, instance_table_slot) =
        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
            "known-bad-instance-table-first-page-unregistered-record-width",
            |bytes| {
                let registered_width = u16::try_from(INSTANCE_TABLE_RECORD_BYTES).expect("88");
                bytes[PACKED_UNIT_HEADER_RECORD_WIDTH_OFFSET..PACKED_UNIT_HEADER_RECORD_WIDTH_OFFSET + 2]
                    .copy_from_slice(&(registered_width + 1).to_le_bytes());
            },
            true,
        );
    assert_eq!(
        verdict(&image, "I-3.8"),
        InvariantVerdict::NotApplicable(
            "最新根的实例表第 0 片容器记录不合法（头或 I-1.7 不成立）、行没收全：没有行可判"
        ),
        "最新根的实例表（槽 {instance_table_slot}）记录宽写成 89：I-3.8 报不适用并点名第 0 片容器记录不合法"
    );
    assert!(
        matches!(verdict(&image, "I-1.7"), InvariantVerdict::Violated(_)),
        "记录宽不是登记的宽，I-1.7 红"
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

/// 攻方 N5-b（出处同上）：最新根的实例表链指针记录「有无下一片」写 2，重封、补到根槽。I-3.14 红，这一片解不开、行不收，
/// 不知道后面还有没有片，走不到末片：I-9.29 报部分没判，不报成立（这一片其实就是整张表，可镜像上分不出「只有一片」与
/// 「后面还有片」）；C 进了候选集，判红的与另三形同是 [`VIOLATED_ONCE_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE`]，I-3.8 的理由同上
/// （四形的引用集合那一遍同样记数不全）。
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

/// I-3.14 第五形的造法（K2-1 甲：「有下一片而位置指针头部有一段不是第一版的取值」算这一片解不开）：
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
/// 池级 checker 同一个读法：I-3.14 红（最后一条是合法的链指针记录那一格），这一片的行不收、链不往下走、走不到末片，
/// 判红的另几条与 I-3.8 的理由同另四形（[`assert_the_newest_instance_table_page_does_not_decode`]：I-3.8 一行都没收、报不适用），
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
    root_slot_offsets_of_both_devices()
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
pub(super) const INSTANCE_TABLE_ROW_INSTANCE_OFFSET: usize = 1;

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
    reseal_the_unit_header_and_payload(&mut second_page);
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
/// 盘 0 那份对不上（I-2.1）、盘 1 那份读回、自证过，「走过」按读回的那一份的落点认：第 1 片照走、行收全、走到末片，C 滤掉，
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
/// 与实现同一个读法：行的 flags 不是 0 的一片解不开（D18（块里携带什么信息） 已定项 11「第一版恒 0、非 0 拒收」）。
/// 镜像：最新根的实例表第一行 flags 写成非 0（1）、它的 T 抬到不低于最新根的 txg，重封、补到根槽。最新根那张表这条读法读不出，
/// I-3.12（实例表行 T 低于引用它的根的 txg）不判它、不红。
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
                    INSTANCE_TABLE_ROW_FLAGS_NOT_ZERO;
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
        singlefs_verification::RecordingBlockDevice<
            singlefs_verification::memory_pool::SparseBlockDevice,
        >,
    )>;
    let stream = SharedStream::new();
    let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
        .iter()
        .map(|identity| {
            (
                *identity,
                singlefs_verification::RecordingBlockDevice::with_shared_stream(
                    *identity,
                    singlefs_verification::memory_pool::SparseBlockDevice::new(
                        common::IMAGE_BYTES,
                        singlefs_core::block_device::PhysicalBlockSizeInBytes(512),
                    ),
                    stream.clone(),
                ),
            )
        })
        .collect();
    singlefs_verification::scenario::run_new_pool_file_creation(
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
pub(super) const INSTANCE_TABLE_ROWS_NOT_APPLICABLE: &str =
    "根环里没有哪条根指着一张读得出、有行的实例表";

/// I-3.12（实例表行 T 低于引用它的根的 txg） 的坏镜像，建在「可写挂载施加了一条记录」那一份上：实例表里实例 1 那一行 (1, 5, 3)
/// 的 T 5 → 6（= 写行发布那条根的 txg），重封、补到引用它的根槽。
pub(super) fn known_bad_images_of_the_instance_table_rows() -> Vec<(&'static str, Mutation)> {
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
pub(super) const INSTANCE_TABLE_FIRST_ROW_START: usize = 136;

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
