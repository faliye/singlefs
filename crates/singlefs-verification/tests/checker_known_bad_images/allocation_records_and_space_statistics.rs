//! 分配记录与空间统计：记录删掉、加宽、重叠、泄漏、隔离时的已分配与延迟两项统计、跨度与分配代号。

use super::*;

/// 分配记录树叶里槽号是 `slot` 的那条记录整条拿掉（两盘各一条，两片叶各一条）：后面的记录往前挪 20、末尾 20 字节清零、条目数与声明长度各减一条
/// （码 2 头：条目数 2 字节在 102、声明长度 2 字节在 8）。那条记录不在叶的首末，头里按位置定的 key 区间不变。返回拿掉了几条。
fn remove_allocation_record(bytes: &mut [u8], slot: u64) -> usize {
    let count = usize::from(u16::from_le_bytes([
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
    ]));
    let slot_of = |leaf_bytes: &[u8], index: usize| {
        let record = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * index;
        let mut slot_bytes = [0u8; 8];
        slot_bytes[..6].copy_from_slice(&leaf_bytes[record + 4..record + 10]);
        u64::from_le_bytes(slot_bytes)
    };
    let Some(index) = (0..count).find(|index| slot_of(bytes, *index) == slot) else {
        return 0;
    };
    assert!(index > 0 && index + 1 < count, "拿掉的那条不在叶的首末");
    let start = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * index;
    let end = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * count;
    bytes.copy_within(start + ALLOCATION_RECORD_BYTES..end, start);
    bytes[end - ALLOCATION_RECORD_BYTES..end].fill(0);
    set_u16(
        bytes,
        ALLOCATION_NODE_ENTRY_COUNT_OFFSET,
        u16::try_from(count - 1).expect("条目数装得进 u16"),
    );
    let declared = u16::from_le_bytes([bytes[8], bytes[9]]);
    set_u16(
        bytes,
        8,
        declared - u16::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
    );
    1
}

/// 阴性对照（只放过 I-3.18 判红的单元）：跨度段都对、数据单元那条记录整条不见了（两盘各一条），
/// 两槽 50180、50181 遍历走到而真没有记录，又没有一条跨度段不符的记录可归：I-3.1 照判红、点名这两槽；走到的引用起点 50180 上没有一条记录起头，
/// I-3.21（走到的引用起点是最新根计入的分配记录的起点，C944）也红、点名它——红的恰是这两条；
/// 别的每一条与干净镜像上的判定逐条相同（I-3.18 照判别的记录、成立）。
#[test]
fn data_unit_whose_allocation_record_is_gone_reddens_the_allocated_statistic_and_the_walked_reference_start_invariants(
) {
    let clean = build_pool("known-bad-data-unit-record-gone").memory_pool();
    let clean_verdicts = check_pool_image(&clean);
    assert_eq!(
        verdict(&clean, "I-3.1"),
        InvariantVerdict::Holds,
        "干净镜像上 I-3.1 真被评估过且成立"
    );
    let mut image = clean.clone();
    assert_eq!(
        mutate_allocation_leaves(&mut image, &UNITS, ALLOCATION_LEAVES, |bytes| {
            remove_allocation_record(bytes, DATA_UNIT)
        }),
        2,
        "数据单元那一个落点，两盘各一条"
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        vec![
            "I-3.1",
            WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS
        ],
        "红 I-3.1 与 I-3.21：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-3.1") else {
        panic!("上面判过 I-3.1 红");
    };
    assert!(
        detail.contains(&format!(
            "遍历走到而不在下沿那一份里的槽 [{DATA_UNIT}, {}]",
            DATA_UNIT + 1
        )),
        "违例点名数据单元的两槽：{detail}"
    );
    let InvariantVerdict::Violated(walked_start_detail) = verdict(
        &image,
        WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
    ) else {
        panic!("上面判过 I-3.21 红");
    };
    assert!(
        walked_start_detail.contains(&format!("槽 {DATA_UNIT}：遍历走到的引用从这一槽起")),
        "I-3.21 点名数据单元的起点槽：{walked_start_detail}"
    );
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &clean_verdicts,
            &[
                "I-3.1",
                WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS
            ]
        ),
        Vec::new(),
        "除 I-3.1、I-3.21 之外每一条与干净镜像上的判定逐条相同"
    );
}

/// 写完新池新建文件那份镜像上单元区里最后两个一槽单元（映射树根 `MAPPING_ROOT` 50251、树表 `TREE_TABLE` 50252，都被最新根引用）后面那一槽
/// （盘上全零、不是单元）：I-3.1 取宽宽的方向那一形（判决 `research/prompts/m3-close-b19-reason-r1-main-verification.md` 第二节 Q3 宽那一行，
/// 攻方最小镜像「两个一槽单元、一条跨 3 的记录」）改那两个单元的记录，跨度罩到这一槽。
const SLOT_PAST_THE_LAST_UNIT: u64 = 50253;

/// 分配记录树叶里槽号是 `slot`、排在叶末尾的那条记录整条拿掉：末尾 20 字节清零、条目数与声明长度各减一条（码 2 头：条目数 2 字节在 102、
/// 声明长度 2 字节在 8）。分配记录树按位置寻址，头里按位置定的 key 区间不随条目变。返回拿掉了几条。
fn remove_the_last_allocation_record(bytes: &mut [u8], slot: u64) -> usize {
    let count = usize::from(u16::from_le_bytes([
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
    ]));
    let last = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * (count - 1);
    let mut slot_bytes = [0u8; 8];
    slot_bytes[..6].copy_from_slice(&bytes[last + 4..last + 10]);
    if u64::from_le_bytes(slot_bytes) != slot {
        return 0;
    }
    bytes[last..last + ALLOCATION_RECORD_BYTES].fill(0);
    set_u16(
        bytes,
        ALLOCATION_NODE_ENTRY_COUNT_OFFSET,
        u16::try_from(count - 1).expect("条目数装得进 u16"),
    );
    let declared = u16::from_le_bytes([bytes[8], bytes[9]]);
    set_u16(
        bytes,
        8,
        declared - u16::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
    );
    1
}

/// I-3.1（已分配统计对得上） 取宽宽的方向（C937（I-3.1 照 I-3.18 改记槽数取宽的条件被攻过零轮），判决
/// `research/prompts/m3-close-b19-reason-r1-main-verification.md` 第二节 Q3 宽那一行的攻方最小镜像）：两盘各自那条起在映射树根 50251
/// （码 2、一槽）的未释放记录跨度改成 3（罩 50251、50252、50253），树表 50252 自己那条记录拿掉，记账照两个活单元记、不动。
/// 按记录自述 E 比记账多一槽（50253），(L) 不成立；改记成一槽之后遍历走到的 50252 不在记录里、上沿少一槽：两种读法各有一判不成立，
/// I-3.1 判红（改成只认改记那一份之前逐判混取，报部分没判）；I-3.18 判 50251 那条跨 3 而它是码 2 一槽；走到的引用起点 50252 上没有一条记录起头，
/// I-3.21（走到的引用起点是最新根计入的分配记录的起点，C944）也红、点名它（新立 I-3.21 之后这一张多红这一条，合入后验证任务 6223 量的）。
/// 别的每一条与干净镜像上的判定逐条相同。
#[test]
fn record_widened_over_a_walked_one_slot_neighbor_without_its_own_record_reddens_the_allocated_statistic_and_the_span_invariants(
) {
    let clean = build_pool("known-bad-record-widened-over-a-neighbor").memory_pool();
    let clean_verdicts = check_pool_image(&clean);
    assert_eq!(
        verdict(&clean, "I-3.1"),
        InvariantVerdict::Holds,
        "干净镜像上 I-3.1 真被评估过且成立"
    );
    let mut image = clean.clone();
    assert_eq!(
        mutate_allocation_leaves(&mut image, &UNITS, ALLOCATION_LEAVES, |bytes| {
            set_allocation_record_span(bytes, MAPPING_ROOT, 3)
                + remove_the_last_allocation_record(bytes, TREE_TABLE)
        }),
        4,
        "映射树根那条记录改宽、树表那条记录拿掉，两盘各一条"
    );
    assert_eq!(
        SLOT_PAST_THE_LAST_UNIT,
        TREE_TABLE + 1,
        "改宽的那条罩到树表后面那一槽"
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        vec![
            "I-3.1",
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
            WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS
        ],
        "红 I-3.1、I-3.18 与 I-3.21：{verdicts:?}"
    );
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &clean_verdicts,
            &[
                "I-3.1",
                ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
                WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS
            ]
        ),
        Vec::new(),
        "除 I-3.1、I-3.18、I-3.21 之外每一条与干净镜像上的判定逐条相同"
    );
}

/// 窄的方向里那条泄漏记录罩的槽：数据单元之后、单元区里下一个单元（50240）之前那一段里的一槽，盘上全零、不是单元，遍历走不到。
const ALL_ZERO_SLOT_A_LEAKED_RECORD_COVERS: u64 = 50200;

/// 分配记录树叶里紧跟槽号是 `after_slot` 那条记录，插一条起在 `slot`、跨 1 槽、未释放的记录（盘与分配代抄 `after_slot` 那一条）：
/// 后面的记录往后挪 20、条目数与声明长度各加一条。返回插了几条。
fn insert_an_unreleased_one_slot_allocation_record_after(
    bytes: &mut [u8],
    after_slot: u64,
    slot: u64,
) -> usize {
    let count = usize::from(u16::from_le_bytes([
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
    ]));
    let slot_of = |leaf_bytes: &[u8], index: usize| {
        let record = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * index;
        let mut slot_bytes = [0u8; 8];
        slot_bytes[..6].copy_from_slice(&leaf_bytes[record + 4..record + 10]);
        u64::from_le_bytes(slot_bytes)
    };
    let Some(index) = (0..count).find(|index| slot_of(bytes, *index) == after_slot) else {
        return 0;
    };
    assert!(
        index + 1 < count && slot_of(bytes, index + 1) > slot,
        "插的那条排在 {after_slot} 那条与下一条之间"
    );
    let source = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * index;
    let inserted = source + ALLOCATION_RECORD_BYTES;
    let end = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * count;
    bytes.copy_within(inserted..end, inserted + ALLOCATION_RECORD_BYTES);
    bytes.copy_within(source..inserted, inserted);
    bytes[inserted + 4..inserted + 10].copy_from_slice(&slot.to_le_bytes()[..6]);
    set_u16(bytes, inserted + 10, 1);
    set_u16(
        bytes,
        ALLOCATION_NODE_ENTRY_COUNT_OFFSET,
        u16::try_from(count + 1).expect("条目数装得进 u16"),
    );
    let declared = u16::from_le_bytes([bytes[8], bytes[9]]);
    set_u16(
        bytes,
        8,
        declared + u16::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
    );
    1
}

/// I-3.1 取宽窄的方向（判决 `research/prompts/m3-close-b19-reason-r1-main-verification.md` 第二节 Q3 窄那一行，攻方第二次抽样的最小镜像）：
/// 起步是 I-3.18 那份坏镜像（数据单元那条未释放记录只记一槽，两盘各一条），另在两盘各插一条罩着全零槽 50200 的未释放泄漏记录；
/// 记账照记录数、不动（窄记录漏数的 50181 与泄漏多数的 50200 在个数上抵掉）。按记录自述 (P) 在 50181 不成立；改记成两槽之后 E 比记账多一槽、
/// 50200 不在遍历走到的槽里也不在 defer 里：改记那一份 (L) 与逐槽的上沿不成立，I-3.1 判红（改成只认改记那一份之前上沿只比个数，报部分没判）；
/// I-3.18 判数据单元那条跨 1。别的每一条与干净镜像上的判定逐条相同。
#[test]
fn narrowed_record_with_a_leaked_record_over_an_all_zero_slot_reddens_the_allocated_statistic_and_the_span_invariants(
) {
    let clean = build_pool("known-bad-narrowed-record-with-a-leak").memory_pool();
    let clean_verdicts = check_pool_image(&clean);
    let mut image = clean.clone();
    narrow_the_record_of_the_data_unit_to_one_slot(&mut image);
    assert_eq!(
        mutate_allocation_leaves(&mut image, &UNITS, ALLOCATION_LEAVES, |bytes| {
            insert_an_unreleased_one_slot_allocation_record_after(
                bytes,
                DATA_UNIT,
                ALL_ZERO_SLOT_A_LEAKED_RECORD_COVERS,
            )
        }),
        2,
        "泄漏记录两盘各一条"
    );
    assert_eq!(
        read(&image, 0, ALL_ZERO_SLOT_A_LEAKED_RECORD_COVERS * SLOT, 4096),
        vec![0u8; 4096],
        "泄漏记录罩的那一槽盘上全零"
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        vec![
            "I-3.1",
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS
        ],
        "红 I-3.1 与 I-3.18：{verdicts:?}"
    );
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &clean_verdicts,
            &[
                "I-3.1",
                ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS
            ]
        ),
        Vec::new(),
        "除 I-3.1、I-3.18 之外每一条与干净镜像上的判定逐条相同"
    );
}

/// I-3.1 取宽那一支第五判（判决 `research/prompts/m3-close-b19-reason-r2-main-verification.md` 第二节 Q1「改法」那一行）的坏镜像，辩方承接腿
/// 「窄记录加尾槽上另一条记录」那一形（`research/prompts/m3-close-b19-reason-r2-sonnet-c2-output.md` 模型输出第 140 行：宽记录、窄记录、
/// 一条未释放记录落在窄记录那个单元的尾槽）：两盘各自那条数据单元的未释放记录只记一槽（50180），尾槽 50181 上另插一条未释放、跨 1 的记录，
/// 树表（50252，一槽）那条记录改宽成 2（罩到盘上全零的 50253）；记账不动。按记录自述 E 比记账多一槽（50253），(L) 不成立；
/// 改记之后（数据单元那条两槽、树表那条一槽）E 与干净镜像同一组槽，三判与逐槽两判都成立；
/// 改记之后数据单元那条的 [50180, 50182) 与尾槽上那条在 50181 相交，I-3.1 判红、违例点名两条记录与相交的槽。
/// I-3.18 判数据单元那条跨 1、树表那条跨 2；别的每一条与干净镜像上的判定逐条相同（尾槽上那条起点头用不了，I-3.10、I-3.15 没有对象；
/// 自述的跨度互不相交，I-5.4 成立；走到的起点都有自己的记录，I-3.21 成立）。
#[test]
fn narrowed_record_with_another_record_on_its_tail_slot_reddens_the_allocated_statistic_and_the_span_invariants(
) {
    let clean = build_pool("known-bad-narrowed-record-with-a-record-on-its-tail").memory_pool();
    let clean_verdicts = check_pool_image(&clean);
    let mut image = clean.clone();
    narrow_the_record_of_the_data_unit_to_one_slot(&mut image);
    assert_eq!(
        mutate_allocation_leaves(&mut image, &UNITS, ALLOCATION_LEAVES, |bytes| {
            insert_an_unreleased_one_slot_allocation_record_after(bytes, DATA_UNIT, DATA_UNIT + 1)
                + set_allocation_record_span(bytes, TREE_TABLE, 2)
        }),
        4,
        "尾槽上那条两盘各插一条、树表那条两盘各改宽一条"
    );
    assert_eq!(
        read(&image, 0, SLOT_PAST_THE_LAST_UNIT * SLOT, 4096),
        vec![0u8; 4096],
        "树表那条改宽之后多罩的那一槽盘上全零"
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        vec![
            "I-3.1",
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS
        ],
        "红 I-3.1 与 I-3.18：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-3.1") else {
        panic!("上面判过 I-3.1 红");
    };
    assert!(
        detail.contains(&format!("槽 {DATA_UNIT} 的记录（未释放，改记之后跨 2 槽）"))
            && detail.contains(&format!(
                "槽 {} 的记录（未释放，跨 1 槽）在槽 [{}, {}) 相交",
                DATA_UNIT + 1,
                DATA_UNIT + 1,
                DATA_UNIT + 2
            )),
        "违例点名数据单元那条、尾槽上那条与相交的槽 50181：{detail}"
    );
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &clean_verdicts,
            &[
                "I-3.1",
                ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS
            ]
        ),
        Vec::new(),
        "除 I-3.1、I-3.18 之外每一条与干净镜像上的判定逐条相同"
    );
}

/// I-3.1 取宽那一支第五判的违例说明按实际情形写（判决 `research/prompts/m3-close-b19-reason-r3-main-verification.md` 第二节 Q1 第 2 问那一行，
/// 攻方第二样本 Q1-2 的构造法）：两盘各自那条数据单元的未释放记录只记一槽（I-3.18 判红、改记成两槽，取宽那一支才走得进来），
/// inode 叶容器（50242，码 3 两槽）尾槽 50243 上另插一条未释放、跨 1 的记录；记账不动。按记录自述 (P) 在 50181 不成立；
/// 改记之后 E 与干净镜像同一组槽，三判与逐槽两判都成立，而 inode 叶容器那条（照记录自述跨 2、没被改记）与尾槽上那条（跨 1、没被改记）
/// 在 50243 相交：I-3.1 判红，说明点名这一对、只写跨几槽，不说「改记之后」（两条都没被改记）。这一对在记录自述那一份里就相交，I-5.4 跟着红（重复报，不矛盾）；
/// I-3.18 判数据单元那条跨 1。别的每一条与干净镜像上的判定逐条相同。
#[test]
fn records_overlapping_without_either_being_set_to_its_unit_class_redden_the_allocated_statistic_without_saying_they_were_set(
) {
    let clean = build_pool("known-bad-records-overlapping-without-either-set").memory_pool();
    let clean_verdicts = check_pool_image(&clean);
    let mut image = clean.clone();
    narrow_the_record_of_the_data_unit_to_one_slot(&mut image);
    assert_eq!(
        mutate_allocation_leaves(&mut image, &UNITS, ALLOCATION_LEAVES, |bytes| {
            insert_an_unreleased_one_slot_allocation_record_after(bytes, INODE_LEAF, INODE_LEAF + 1)
        }),
        2,
        "inode 叶容器尾槽上那条两盘各插一条"
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariant_set(&verdicts),
        [
            "I-3.1",
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
            "I-5.4"
        ]
        .into_iter()
        .collect::<std::collections::BTreeSet<&'static str>>(),
        "红 I-3.1、I-3.18 与 I-5.4：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-3.1") else {
        panic!("上面判过 I-3.1 红");
    };
    assert!(
        detail.contains(&format!(
            "槽 {INODE_LEAF} 的记录（未释放，跨 2 槽）与槽 {} 的记录（未释放，跨 1 槽）在槽 [{}, {}) 相交",
            INODE_LEAF + 1,
            INODE_LEAF + 1,
            INODE_LEAF + 2
        )) && !detail.contains("改记之后"),
        "违例点名 inode 叶容器那条与尾槽上那条、相交的槽 50243，两条都没被改记、不说改记之后：{detail}"
    );
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &clean_verdicts,
            &[
                "I-3.1",
                ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
                "I-5.4"
            ]
        ),
        Vec::new(),
        "除 I-3.1、I-3.18、I-5.4 之外每一条与干净镜像上的判定逐条相同"
    );
}

/// 分配记录里跨度段的隔离记号（D19（块指针的结构与宽度预算） 已定项 5，跨度段次高位）与跨度值那几位。
const ALLOCATION_RECORD_QUARANTINE_MARK: u16 = 0x4000;

const ALLOCATION_RECORD_SPAN_VALUE_BITS: u16 = 0x3FFF;

/// C962（第五判对两条都带隔离记号的相交照判红）：「改宽罩进隔离豁免的槽（18 个）」取手造的
/// `quarantined_unit_tail_on_another_quarantined_record` 那一形（`research/prompts/m3-close-b19-reason-r2-opus-model/i31_q1_sweep_quarantine_counted_in_i311.py`
/// 的 `mirrors`），做成字节级镜像，钉住「照红、不改判法」。底子是两块盘都带记号的隔离（A 的数据单元 X 两份都坏、
/// 覆盖写 B 隔离它，抬 F 到 4 之后没有任何有效根引用它）；每一版的分配记录树叶里，X 那条带记号的未释放记录只记一槽（记号照留），
/// X 尾槽上另插一条带记号的未释放记录 Z（跨 1），最新那一版的数据单元 Y 那条未释放记录也只记一槽（Y 被走到，按记录自述 (P) 不成立，取宽那一支才走得进来）；
/// 记账不动。隔离豁免按记录自述归组：X、Z 各是一组、组里两块盘都带记号，整组豁免，豁免的槽仍是 X 那两槽；改记之后 Y、X 都是两槽，
/// E 与豁免都与干净镜像同一组槽，三判与逐槽两判都成立，而 X 改记之后的 [x, x + 2) 与 Z 在 x + 1 相交：I-3.1 判红（第五判）、I-3.18 判 X、Y 两条跨 1；
/// I-3.17（隔离记号与引用相符） 三句都成立。别的每一条与干净镜像上的判定逐条相同。
#[test]
fn a_quarantined_record_narrowed_beside_another_quarantined_record_on_its_tail_slot_reddens_the_allocated_statistic_and_the_span_invariant(
) {
    let built =
        scenarios_of_quarantine_mark_agrees_with_the_references::quarantined_unit_no_root_references(
            "known-bad-quarantined-record-narrowed-beside-another-quarantined-record",
            scenarios_of_quarantine_mark_agrees_with_the_references::leave_the_account_as_the_writer_left_it,
            scenarios_of_quarantine_mark_agrees_with_the_references::leave_the_account_as_the_writer_left_it,
        );
    let quarantined_slot = built.data_slot.0;
    let newest_data_slot = built.newest_data_slot.0;
    let clean = built.pool.memory_pool();
    let clean_verdicts = check_pool_image(&clean);
    assert_eq!(
        verdict(&clean, QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES),
        InvariantVerdict::Holds,
        "改之前：两块盘都带记号的隔离，I-3.17 真被评估过且成立"
    );
    let mut image = clean.clone();
    let slot_of = |record: &[u8]| {
        let mut slot_bytes = [0u8; 8];
        slot_bytes[..6].copy_from_slice(&record[4..10]);
        u64::from_le_bytes(slot_bytes)
    };
    let span_field_of = |record: &[u8]| u16::from_le_bytes([record[10], record[11]]);
    let changed = scenarios_of_quarantine_mark_agrees_with_the_references::change_the_allocation_records_in_every_leaf_copy(
        &mut image,
        |records| {
            let mut changed_here = 0;
            let mut index = 0;
            // 迭代上界：每条记录看一次，插进去的那条跳过。
            while index < records.len() {
                let span_field = span_field_of(&records[index][..]);
                let is_unreleased = span_field & ALLOCATION_RECORD_RELEASED_FLAG == 0;
                let carries_the_mark = span_field & ALLOCATION_RECORD_QUARANTINE_MARK != 0;
                let span_slots = span_field & ALLOCATION_RECORD_SPAN_VALUE_BITS;
                let slot = slot_of(&records[index][..]);
                if is_unreleased && carries_the_mark && span_slots == 2 && slot == quarantined_slot {
                    set_u16(&mut records[index], 10, ALLOCATION_RECORD_QUARANTINE_MARK | 1);
                    let mut on_the_tail_slot = records[index].clone();
                    on_the_tail_slot[4..10].copy_from_slice(&(quarantined_slot + 1).to_le_bytes()[..6]);
                    records.insert(index + 1, on_the_tail_slot);
                    changed_here += 2;
                    index += 2;
                    continue;
                }
                if is_unreleased && !carries_the_mark && span_slots == 2 && slot == newest_data_slot {
                    set_u16(&mut records[index], 10, 1);
                    changed_here += 1;
                }
                index += 1;
            }
            changed_here
        },
    );
    assert!(changed > 0, "分配记录树叶里改到了 X、Z、Y 那几条");
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariant_set(&verdicts),
        [
            "I-3.1",
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS
        ]
        .into_iter()
        .collect::<std::collections::BTreeSet<&'static str>>(),
        "红 I-3.1 与 I-3.18：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-3.1") else {
        panic!("上面判过 I-3.1 红");
    };
    assert!(
        detail.contains(&format!(
            "槽 {quarantined_slot} 的记录（未释放，改记之后跨 2 槽）与槽 {} 的记录（未释放，跨 1 槽）在槽 [{}, {}) 相交",
            quarantined_slot + 1,
            quarantined_slot + 1,
            quarantined_slot + 2
        )),
        "第五判点名 X、Z 两条带记号的记录与相交的那一槽：{detail}"
    );
    assert_eq!(
        verdict(&image, QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES),
        InvariantVerdict::Holds,
        "X、Z 各自两块盘都带记号、都没有根引用：I-3.17 照旧成立"
    );
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &clean_verdicts,
            &[
                "I-3.1",
                ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS
            ]
        ),
        Vec::new(),
        "除 I-3.1、I-3.18 之外每一条与改之前那张镜像上的判定逐条相同"
    );
}

/// C961（全零根槽没并进 checker 的抬 F 上限）的合法写者那一张（[`common_root_ring_holes::pool_raising_the_floor_over_a_ring_slot_reading_back_all_zero`]）：
/// 照 D16（发布语义） 已定项 1 字面把读回全零的 txg 3 那一槽并进上限、抬到 3。一条都不红，I-7.9 真被评估过且成立
/// （读回全零的那一槽能住的最大 txg 3 不低于 F 3；窗口里的上限 4 也不低于它）。
#[test]
fn raising_the_floor_to_the_ceiling_that_merges_a_ring_slot_reading_back_all_zero_holds_every_invariant(
) {
    let image = common_root_ring_holes::pool_raising_the_floor_over_a_ring_slot_reading_back_all_zero(
        common_root_ring_holes::FloorRaiseOverTheRingSlotReadingBackAllZero::ToTheCeilingWithTheAllZeroSlotMerged,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "抬到并进全零槽之后的上限 3：一条都不许红：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-7.9"),
        InvariantVerdict::Holds,
        "I-7.9 要真被评估过且成立"
    );
}

/// C961 那一段历史上不并全零槽的坏写者（攻方第二样本模型 `first_example`：drop_writer_floor 4、literal_ceiling 3、checker_ceiling 4）：
/// F 抬到 4，越过读回全零的 txg 3 那一槽（区域 0 槽 1）能住的最大 txg 3。checker 不并全零槽时，按环里读得出的根算出的上限是 4，判成立；
/// 照 D16（发布语义） 已定项 1「抬 F 的上限」那一行并进它之后只红 I-7.9，红在「从 0 抬到 4、全零的区域 0 槽 1 能住的最大 txg 3」那一格。
#[test]
fn raising_the_floor_past_a_ring_slot_reading_back_all_zero_reddens_only_the_rollback_floor_invariant(
) {
    let image = common_root_ring_holes::pool_raising_the_floor_over_a_ring_slot_reading_back_all_zero(
        common_root_ring_holes::FloorRaiseOverTheRingSlotReadingBackAllZero::PastTheAllZeroSlotToTheCeilingWithoutIt,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-7.9"],
        "F 抬到 4（并进全零槽之后的上限 3）：只红 I-7.9：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-7.9") else {
        panic!("上面判过 I-7.9 红");
    };
    assert!(
        detail.contains(&format!(
            "txg {} 那条根把回退下界 F 从 0 抬到 4，高于读回全零的根环槽（区域 0、槽 1）能住的最大 txg 3",
            common_root_ring_holes::TXG_OF_THE_ROOT_RAISING_THE_FLOOR_OVER_THE_ALL_ZERO_SLOT
        )),
        "红在抬 F 的那条根、全零的区域 0 槽 1 那一格：{detail}"
    );
}

/// I-3.21（[`WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS`]，C944（走到的单元起点没有自己的记录））的坏镜像，
/// 由那张宽镜像
/// （[`record_widened_over_a_walked_one_slot_neighbor_without_its_own_record_reddens_the_allocated_statistic_and_the_span_invariants`]）改一个数：
/// 两盘各自那条起在映射树根 50251（码 2、一槽）的未释放记录跨度改成 2（恰好罩住树表 50252），树表自己那条记录拿掉，记账不动。
/// 按记录自述 E 与干净镜像同一组槽，I-3.1 成立、不进取宽那一支；I-3.18 判 50251 那条跨 2；走到的引用起点 50252（两盘各一个）上没有一条记录起头，
/// I-3.21 判红、点名它。别的每一条与干净镜像上的判定逐条相同（只红这两条）。
#[test]
fn record_widened_exactly_over_a_walked_one_slot_neighbor_without_its_own_record_reddens_only_the_span_and_the_walked_reference_start_invariants(
) {
    let clean = build_pool("known-bad-record-widened-exactly-over-a-neighbor").memory_pool();
    let clean_verdicts = check_pool_image(&clean);
    assert_eq!(
        verdict(
            &clean,
            WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS
        ),
        InvariantVerdict::Holds,
        "干净镜像上 I-3.21 真被评估过且成立"
    );
    let mut image = clean.clone();
    assert_eq!(
        mutate_allocation_leaves(&mut image, &UNITS, ALLOCATION_LEAVES, |bytes| {
            set_allocation_record_span(bytes, MAPPING_ROOT, 2)
                + remove_the_last_allocation_record(bytes, TREE_TABLE)
        }),
        4,
        "映射树根那条记录改宽、树表那条记录拿掉，两盘各一条"
    );
    assert_eq!(TREE_TABLE, MAPPING_ROOT + 1, "改宽的那条恰好罩到树表");
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        vec![
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
            WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS
        ],
        "只红 I-3.18 与 I-3.21：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(
        &image,
        WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
    ) else {
        panic!("上面判过 I-3.21 红");
    };
    assert!(
        detail.contains(&format!("槽 {TREE_TABLE}：遍历走到的引用从这一槽起")),
        "违例点名树表那一槽：{detail}"
    );
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &clean_verdicts,
            &[
                ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
                WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS
            ]
        ),
        Vec::new(),
        "除 I-3.18、I-3.21 之外每一条与干净镜像上的判定逐条相同"
    );
}

/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 的坏镜像，从发布 B 之后的干净镜像出发：B 那一版分配记录树里
/// B 的数据单元那两条未释放记录，分配代从 4 改回上一次发布的 txg 3，单元一个字节不动。
/// 盘上留下的形状就是增补 3 第 1 件那条变异（复用一个已回收的落点、改写那条分配记录时分配代没改）
/// 留下的：一条未释放记录的分配代停在上一次的代上，而它罩住的单元是这一次写的。
/// 释放代、跨度、key 一样没动 ⇒ I-3.9（释放代落在停止引用它的那一格区间里）、I-5.4（分配记录罩住的槽互不相交） 与引用集合都不受影响。
pub(super) fn known_bad_images_of_the_allocation_generation() -> Vec<(&'static str, Mutation)> {
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
pub(super) fn reduce_accounting(bytes: &mut [u8], statistic: u16, device: u32, delta: u64) {
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
pub(super) fn leak_one_slot_keeping_free_plus_allocated(
    bytes: &mut [u8],
    with_inflated_defer_queue: bool,
) {
    adjust_accounting(bytes, 1, 0, 16384);
    reduce_accounting(bytes, 2, 0, 16384);
    if with_inflated_defer_queue {
        adjust_accounting(bytes, STATISTIC_DEFER_QUEUE_BYTES, 0, 16384);
    }
}

/// 增补 2 收口表第 54 行（Z3）的
/// 两份纯泄漏坏镜像，建在写完新池新建文件那份干净镜像上：Z3-A（只泄漏）与 Z3-B（泄漏 + defer 行跟着抬高）。
/// 候选 c（记账 ≥ 遍历）在 Z3-A 上哑、候选 f（记账 − 遍历 ≤ defer 行）在两份上都哑。
pub(super) fn known_bad_images_of_a_pure_leak() -> Vec<(&'static str, Mutation)> {
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
/// （`singlefs_verification::history::allocation_statistic_mechanism`）：「已知红」清单按它分辨机理，少写一项（比如回退下界 F 那一项）
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

/// 候选 b（回退候选集补上「由记录施加出来、根槽从没落盘的那一版」）不放宽 I-3.1 的等式、只补等式右边：
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
        // I-3.16（碎片度与全空段统计对得上） 跟着报部分没判（C701（别的判法静默跳过读不出的部分） 之前报不适用）：已分配那一行对不上
        // 占着的槽那一份并集，占着的是哪几槽认不出，runs 不比。
        let fragmentation_verdict = verdict_among(
            &verdicts,
            FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
        );
        assert!(
            matches!(
                &fragmentation_verdict,
                InvariantVerdict::PartiallyJudged(detail) if detail.contains("占着哪几槽认不出")
            ),
            "泄漏之后 I-3.16 报部分没判、点出占着哪几槽认不出，不报违例：{fragmentation_verdict:?}"
        );
        let allowed_to_differ: Vec<&str> = expected_violations
            .iter()
            .copied()
            .chain([FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN])
            .collect();
        assert_eq!(
            verdicts_that_differ_outside(
                &verdicts,
                &verdicts_on_the_clean_image,
                &allowed_to_differ
            ),
            [],
            "{expected_violations:?} 与 I-3.16 之外每一条的判定与干净镜像逐项相同"
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
pub(super) fn known_bad_images_of_the_deferred_statistic() -> Vec<(&'static str, Mutation)> {
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

/// I-3.11（已分配减 defer 等于最新根走读） 判别力自证那一格（`crates/singlefs-checker/invariants/05-空间记账.md` I-3.11 那一行的 ⚠️）：
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
