// GPU 判器池级 checker 两个内核（走读内核 crash_judge_checker.wgsl、扫描内核 crash_judge_checker_scan.wgsl）共用的那一半，
// 接在 crash_judge_common.wgsl 之后：不变量下标与违例位、根与 journal 记录的读法、草稿区各集合的位置、走读留给扫描的私有变量、
// 已发布谓词、key 形态、候选集与回退候选的判法，以及两个内核之间的交接段。
// 判不了的原因字：低 8 位是 crash_judge_common.wgsl 的 UNDECIDABLE_* 类别，「条目太多」那一类第 8 位起带站点号（每处 `| (N << 8u)` 各不相同），
// 好看是哪张表满了；流水线的 CRASH_AMPLIFICATION_GPU_FALLBACK 行原样打这个字。


// IMPLEMENTED_INVARIANTS 的下标
const INVARIANT_I_1_1: u32 = 0u;

const INVARIANT_I_1_2: u32 = 1u;

const INVARIANT_I_1_3: u32 = 2u;

const INVARIANT_I_1_4: u32 = 3u;

const INVARIANT_I_1_6: u32 = 4u;

const INVARIANT_I_1_7: u32 = 5u;

const INVARIANT_I_1_8: u32 = 6u;

const INVARIANT_I_1_10: u32 = 7u;

const INVARIANT_I_1_11: u32 = 8u;

const INVARIANT_I_2_1: u32 = 9u;

const INVARIANT_I_2_3: u32 = 10u;

const INVARIANT_I_2_4: u32 = 11u;

const INVARIANT_I_2_5: u32 = 12u;

const INVARIANT_I_3_1: u32 = 13u;

const INVARIANT_I_3_8: u32 = 14u;

const INVARIANT_I_3_9: u32 = 15u;

const INVARIANT_I_3_10: u32 = 16u;

const INVARIANT_I_3_11: u32 = 17u;

const INVARIANT_I_4_2: u32 = 18u;

const INVARIANT_I_4_8: u32 = 19u;

const INVARIANT_I_5_1: u32 = 20u;

const INVARIANT_I_5_2: u32 = 21u;

const INVARIANT_I_5_4: u32 = 22u;

const INVARIANT_I_7_1: u32 = 23u;

const INVARIANT_I_7_2: u32 = 24u;

const INVARIANT_I_7_3: u32 = 25u;

const INVARIANT_I_7_4: u32 = 26u;

const INVARIANT_I_7_6: u32 = 27u;

const INVARIANT_I_7_7: u32 = 28u;

const INVARIANT_I_7_8: u32 = 29u;

const INVARIANT_I_7_9: u32 = 30u;

const INVARIANT_I_7_12: u32 = 31u;

const INVARIANT_I_7_13: u32 = 32u;

const INVARIANT_I_7_14: u32 = 33u;

const INVARIANT_I_8_6: u32 = 34u;

const INVARIANT_I_8_7: u32 = 35u;

const INVARIANT_I_8_8: u32 = 36u;

const INVARIANT_I_8_9: u32 = 37u;

const INVARIANT_I_9_1: u32 = 38u;

const INVARIANT_I_9_2: u32 = 39u;

const INVARIANT_I_9_4: u32 = 40u;

const INVARIANT_I_9_6: u32 = 41u;

const INVARIANT_I_9_7: u32 = 42u;

const INVARIANT_I_9_10: u32 = 43u;

const INVARIANT_I_9_12: u32 = 44u;

const INVARIANT_I_9_13: u32 = 45u;

const INVARIANT_I_9_14: u32 = 46u;

const INVARIANT_I_9_15: u32 = 47u;

const INVARIANT_I_9_16: u32 = 48u;

// 第 49 条起是里程碑三二·1 那一批（`singlefs_checker::image::IMPLEMENTED_INVARIANTS` 只往末尾追加）
const INVARIANT_I_1_12: u32 = 49u;

const INVARIANT_I_8_10: u32 = 50u;

const INVARIANT_I_8_11: u32 = 51u;

const INVARIANT_I_4_9: u32 = 52u;

const INVARIANT_I_9_25: u32 = 53u;

const INVARIANT_I_3_13: u32 = 54u;

const INVARIANT_I_9_26: u32 = 55u;

const INVARIANT_I_3_12: u32 = 56u;

const INVARIANT_I_7_16: u32 = 57u;

const INVARIANT_I_7_17: u32 = 58u;

const INVARIANT_I_8_12: u32 = 59u;

const INVARIANT_I_7_15: u32 = 60u;

const INVARIANT_I_4_1: u32 = 61u;

const INVARIANT_I_8_1: u32 = 62u;

const INVARIANT_I_7_5: u32 = 63u;

const INVARIANT_I_8_5: u32 = 64u;

// 第 66 条：I-3.5（引用区间的精确性）的出现规则，走读内核在回退候选集的版本都走完之后逐对判
const INVARIANT_I_3_5: u32 = 65u;

// 第 67、68 条：C673 ⑤ ⑥ 那两条（只往末尾追加）
const INVARIANT_I_9_27: u32 = 66u;

const INVARIANT_I_5_5: u32 = 67u;

// 第 69 条：I-9.28（全部可读根里同一树 ID 的诞生 txg 相同），扫描内核挨着 I-9.14 判
const INVARIANT_I_9_28: u32 = 68u;

// 第 70–74 条：里程碑三·3 删除线那五条（只往末尾追加），扫描内核的删除态扫描判前四条；I-4.4 起初报不适用，里程碑四判 logged_ops 条目
// （引用与快照那一截的 g_judge_the_compaction_intents_of_the_version，walk.rs 的 judge_compaction_intents_of_the_versions）
const INVARIANT_I_9_5: u32 = 69u;

const INVARIANT_I_3_4: u32 = 70u;

const INVARIANT_I_4_7: u32 = 71u;

const INVARIANT_I_4_6: u32 = 72u;

const INVARIANT_I_4_4: u32 = 73u;

// 第 75–83 条：里程碑三·3 墓碑与谱系那九条（只往末尾追加）。走读内核判 I-9.17、I-9.23（墓碑树、谱系树每条内部条目的身份）；
// 扫描内核在删除态扫描那一份版本清单上判 I-9.18–I-9.21、I-9.24、I-9.29。I-9.22 恒不置位：它只在池级 checker 带旋钮值的入口上判，
// 崩溃点重放与放量都走旧入口（`check_pool_image`，当没装旋钮、恒报不适用），GPU 手里也没有旋钮值——照 I-4.4 起初（池级 checker 第一版对它恒报不适用）的先例登记、内核不置它那一位
const INVARIANT_I_9_17: u32 = 74u;

const INVARIANT_I_9_18: u32 = 75u;

const INVARIANT_I_9_19: u32 = 76u;

const INVARIANT_I_9_20: u32 = 77u;

const INVARIANT_I_9_21: u32 = 78u;

const INVARIANT_I_9_22: u32 = 79u;

const INVARIANT_I_9_23: u32 = 80u;

const INVARIANT_I_9_24: u32 = 81u;

const INVARIANT_I_9_29: u32 = 82u;

// 第 84 条：根环线第 10j 批新立的 I-3.14（实例表每片记录形态合法），走读内核在 w_walk_instance_table_chain 走到的每一片上判四格（第 10k 批）
const INVARIANT_I_3_14: u32 = 83u;

// 第 85 条：第 49 批新立的 I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上），扫描内核在最新根上判（walk.rs judge_inode_number_watermark_steps_of_the_newest_root）
const INVARIANT_I_9_30: u32 = 84u;

// 第 86–90 条：第 50 批退役树那五条（walk.rs 的 RETIREMENT_*）。走读内核在退役树的每条内部条目上判 I-9.31；扫描内核在墓碑与谱系那一遍的每一版上判
// I-9.32、I-9.33、I-9.34，扫完各版再跨相邻两版判 I-9.35（用户 2026-09-29「配了 GPU 就一定要 GPU 判，不退回 CPU」，不记判不了）
const INVARIANT_I_9_31: u32 = 85u;

const INVARIANT_I_9_32: u32 = 86u;

const INVARIANT_I_9_33: u32 = 87u;

const INVARIANT_I_9_34: u32 = 88u;

const INVARIANT_I_9_35: u32 = 89u;

// 第 91、92 条：第 53 批谱系树与退役树的容器容量那两条（walk.rs 的 NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE、
// LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE）。扫描内核在墓碑与谱系那一遍判 I-9.36；I-9.37 照 I-9.22 恒不置位
// （崩溃点重放与放量走池级 checker 的旧入口，GPU 也没有旋钮值）
const INVARIANT_I_9_36: u32 = 90u;

const INVARIANT_I_9_37: u32 = 91u;

// 第 93 条（下标 92）：I-1.13（扫描重建与遍历方向的索引相同），扫描内核源码里的第三个入口（crash_judge_checker_scan_rebuild.wgsl 的 scan_rebuild_main）
// 在另三个池级 checker 入口写完违例位之后判（欠账 C821，用户 2026-10-18 定「补 GPU 扫描重建内核」）
const INVARIANT_I_1_13: u32 = 92u;

// 第 94、95 条：里程碑四只读快照那两条（walk.rs 的 DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE、
// LIVE_SNAPSHOTS_ARE_ENUMERABLE；判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5）：引用与快照那一截
// （crash_judge_checker_scan.wgsl 的 judge_references_and_snapshots）在回退候选集每一版上判，I-3.5 ③ ④ 与逆向并进下标 65 那一位
const INVARIANT_I_3_6: u32 = 93u;

const INVARIANT_I_3_7: u32 = 94u;

// 第 96–99 条：里程碑四自建 GC 新判的四条（walk.rs 的 UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE、
// COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED、FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN、
// UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS；判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1、G3 的 N2、N3、N5、N6）：
// I-3.16 在扫描内核记账那一判之后判（judge_accounting 两遍隔离豁免之间收占着的槽），另三条在引用与快照那一截的版本清单上判
const INVARIANT_I_3_15: u32 = 95u;

const INVARIANT_I_5_6: u32 = 96u;

const INVARIANT_I_3_16: u32 = 97u;

const INVARIANT_I_1_14: u32 = 98u;

// 第 100、101 条：截断那一批新立的两条（walk.rs 的 DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT、
// DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE；判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K7 T-2、T-4）：
// 扫描内核的删除态扫描每条记录判，连同下标 72 的 I-4.6 改判（T-3）；下标 73 的 I-4.4 收截断条目（T-1）在引用与快照那一截判
const INVARIANT_I_4_10: u32 = 99u;

const INVARIANT_I_9_38: u32 = 100u;

// 第 102 条：I-9.39（截断字段之外的活单元不早于它，walk.rs 的 LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT；
// 判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R6）：扫描内核的删除态扫描每条记录判
const INVARIANT_I_9_39: u32 = 101u;

// 第 103 条：I-9.40（extent 上段形状由有条目的 inode 号集合定、根层级最低，walk.rs 的
// EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY；判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md`
// 第三节 M2）：扫描内核的删除态扫描每一版的每一棵 extent 树判
const INVARIANT_I_9_40: u32 = 102u;

// 第 104 条：I-3.17（隔离记号与引用相符，walk.rs 的 QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES；D19（块指针的结构与宽度预算） 已定项 5
// 「新加判据『记号与引用相符』」，用户 2026-10-22 定）：扫描内核记账那一判两遍隔离豁免之后在最新根那棵分配记录树上判三句
const INVARIANT_I_3_17: u32 = 103u;

// 第 105、106 条：钉也落盘那一批（C773，walk.rs 的 ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD、
// PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE）：走读内核在走完最新根之后判，只判建账的发布
const INVARIANT_I_7_18: u32 = 104u;

const INVARIANT_I_7_19: u32 = 105u;

// 第 107、108 条：里程碑三收尾补的两条（walk.rs 的 SYSTEM_CONFIGURATION_RECORDS_THE_WRITER_IDENTITY、
// ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS；C868、C411）：走读内核择系统配置时逐盘判 I-1.5，
// 引用与快照那一截在 I-3.15 那一份版本清单上判 I-3.18
const INVARIANT_I_1_5: u32 = 106u;

const INVARIANT_I_3_18: u32 = 107u;

// 第 109 条：I-8.13（自证过的 journal 记录落在它计数器该落的环槽：D23（journal 的角色与格式） 已定项 18「记录 n 落在环内偏移
// (计数器 − 1) mod 槽数 × 4096」的可执行形式，欠账 C897）：走读内核逐盘扫环时每条自证过的记录判一次（同键先到先留之前，被挤掉的那一条也判）
const INVARIANT_I_8_13: u32 = 108u;

// extent 树上段叶条目的标签（walk.rs 经 position_addressed.rs 的 extent_upper_leaf_entry_view；判决 m4-trunc-r1 第三节 K5 加 3、4：
// 3 同 1 指下段根、4 同 2 是内联的数据指针，最后一个存在的单元之后到 size 是洞）
const EXTENT_UPPER_LEAF_TAG_NO_DATA_UNIT: u32 = 0u;

const EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT: u32 = 1u;

const EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT: u32 = 2u;

const EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT_FOLLOWED_BY_A_HOLE: u32 = 3u;

const EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT_FOLLOWED_BY_A_HOLE: u32 = 4u;

// 墓碑树、谱系树叶容器的打包记录类型（walk.rs 的 PACKED_TYPE_TOMBSTONE / PACKED_TYPE_LINEAGE；登记的记录宽墓碑 40、谱系 24）
const PACKED_TYPE_TOMBSTONE: u32 = 1u;

const PACKED_TYPE_LINEAGE: u32 = 5u;

// 退役树叶容器的打包记录类型（walk.rs 的 PACKED_TYPE_RETIREMENT；登记的记录宽 34）
const PACKED_TYPE_RETIREMENT: u32 = 3u;

// deleted_inodes 根的条目：key = inode 号 8 字节、无 value（walk.rs 的 DELETED_INODES_ENTRY_BYTES）
const DELETED_INODES_ENTRY_BYTES: u32 = 8u;

// 树表条目里头 ID 8 字节的偏移 TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET 在 crash_judge_common.wgsl（恢复内核配快照条目也读它）

// mkfs 种下的树 ID 水位（singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS），I-5.5 认法 ① 拿 extent 树条目的树 ID 比它
const TREE_IDENTIFIER_WATERMARK_AT_MKFS: u32 = 65536u;

// 违例位：位 i 在第 i / 32 字的第 i % 32 位（位 i ↔ IMPLEMENTED_INVARIANTS[i]），存进结论的第 POOL_CHECKER_VIOLATED_FIRST_WORD 字起。
var<private> violated: array<u32, POOL_CHECKER_VIOLATED_WORDS>;

fn violate(invariant: u32) {
    let word_index = invariant / 32u;
    violated[word_index] = violated[word_index] | (1u << (invariant % 32u));
}

fn clear_violated() {
    for (var word_index = 0u; word_index < POOL_CHECKER_VIOLATED_WORDS; word_index = word_index + 1u) {
        violated[word_index] = 0u;
    }
}

fn load_violated(out: u32) {
    for (var word_index = 0u; word_index < POOL_CHECKER_VIOLATED_WORDS; word_index = word_index + 1u) {
        violated[word_index] = verdicts[out + POOL_CHECKER_VIOLATED_FIRST_WORD + word_index];
    }
}

fn store_violated(out: u32) {
    for (var word_index = 0u; word_index < POOL_CHECKER_VIOLATED_WORDS; word_index = word_index + 1u) {
        verdicts[out + POOL_CHECKER_VIOLATED_FIRST_WORD + word_index] = violated[word_index];
    }
}

// 判到位（与违例位同一个下标）：这一条在这个状态上「评估过」——CPU 池级 checker 的 `Judgements::judge` 调到过它（成立或违例都算），
// 而射程里没有一部分判不了（`part_of_the_range_not_judged`、`partially_judged` 那两种把整条降成不适用或部分没判）。存进结论的第
// POOL_CHECKER_JUDGED_FIRST_WORD 字起之前与违例位合起来：违例的一定判到（CPU `into_report` 违例压过别的三种）。
var<private> evaluated: array<u32, POOL_CHECKER_JUDGED_WORDS>;
var<private> part_not_judged: array<u32, POOL_CHECKER_JUDGED_WORDS>;

// CPU `Judgements::judge(invariant, holds, …)` 的对应：评估一次，不成立就置违例位
fn judge(invariant: u32, holds: bool) {
    let word_index = invariant / 32u;
    evaluated[word_index] = evaluated[word_index] | (1u << (invariant % 32u));
    if (!holds) {
        violate(invariant);
    }
}

// CPU `Judgements::part_of_the_range_not_judged` 与 `Judgements::partially_judged` 的对应：没有违例时这一条不算判到
fn mark_part_of_the_range_not_judged(invariant: u32) {
    let word_index = invariant / 32u;
    part_not_judged[word_index] = part_not_judged[word_index] | (1u << (invariant % 32u));
}

// 一次记同一个字里的几条（mask 是第 word_index 字里的位；墓碑、谱系、退役那几条都在下标 64..95 那一个字里）
fn mark_parts_of_the_range_not_judged(word_index: u32, mask: u32) {
    part_not_judged[word_index] = part_not_judged[word_index] | mask;
}

fn clear_judged() {
    for (var word_index = 0u; word_index < POOL_CHECKER_JUDGED_WORDS; word_index = word_index + 1u) {
        evaluated[word_index] = 0u;
        part_not_judged[word_index] = 0u;
    }
}

// 扫描内核接着走读内核记：评估过与部分判不了两张表随交接段存在草稿区里（结论字里只存合起来的判到位）
fn load_judged() {
    for (var word_index = 0u; word_index < POOL_CHECKER_JUDGED_WORDS; word_index = word_index + 1u) {
        evaluated[word_index] = sw(H_BASE + H_EVALUATED + word_index);
        part_not_judged[word_index] = sw(H_BASE + H_PART_NOT_JUDGED + word_index);
    }
}

fn store_judged(out: u32) {
    for (var word_index = 0u; word_index < POOL_CHECKER_JUDGED_WORDS; word_index = word_index + 1u) {
        sws(H_BASE + H_EVALUATED + word_index, evaluated[word_index]);
        sws(H_BASE + H_PART_NOT_JUDGED + word_index, part_not_judged[word_index]);
        verdicts[out + POOL_CHECKER_JUDGED_FIRST_WORD + word_index] =
            violated[word_index] | (evaluated[word_index] & ~part_not_judged[word_index]);
    }
}

struct CheckerRoot {
    valid: bool,
    instance: u32,
    txg: vec2<u32>,
    watermark: vec2<u32>,
    floor: vec2<u32>,
    unmount: bool,
    base: u32,
};

fn checker_root_of(index: u32) -> CheckerRoot {
    let version = sw(R_ROOTS + index * 2u);
    var r: CheckerRoot;
    r.valid = true;
    r.base = B[T_VERSION_BYTES] + tab(T_VERSIONS, version * VERSION_WORDS + 1u);
    r.instance = rd32(r.base, 24u);
    r.txg = rd64(r.base, 28u);
    r.watermark = rd64(r.base, 122u);
    r.floor = rd64(r.base, 130u);
    r.unmount = (rd32(r.base, 20u) & 1u) != 0u;
    return r;
}

// 根表第二个字：低 8 位是区域号，第 8 位起是区域内的槽号（checker_collect_roots 写，I-7.17 读槽号）
fn checker_root_region(index: u32) -> u32 {
    return sw(R_ROOTS + index * 2u + 1u) & 0xFFu;
}

fn checker_root_slot_in_its_region(index: u32) -> u32 {
    return sw(R_ROOTS + index * 2u + 1u) >> 8u;
}

struct CheckerRecord {
    base: u32,
    counter: vec2<u32>,
    instance: u32,
    txg: vec2<u32>,
    transaction: vec2<u32>,
    commit_byte: u32,
    ordinal: u32,
    flags: u32,
    back_chain: u32,
    chain_value: u32,
    named_count: u32,
};

fn checker_record_at(index: u32) -> CheckerRecord {
    return checker_record_of_row(R_RECORDS + index * RECORD_WORDS);
}

// 草稿区里一行记录（版本、计数器 lo、计数器 hi、实例，之后各表自己的字）解成 CheckerRecord：逐盘的记录表与重放读的那一份环共用
fn checker_record_of_row(row: u32) -> CheckerRecord {
    let version = sw(row);
    var r: CheckerRecord;
    r.base = B[T_VERSION_BYTES] + tab(T_VERSIONS, version * VERSION_WORDS + 1u);
    r.counter = vec2<u32>(sw(row + 1u), sw(row + 2u));
    r.instance = sw(row + 3u);
    r.txg = rd64(r.base, 26u);
    r.transaction = rd64(r.base, 78u);
    r.commit_byte = rd8(r.base, 86u);
    r.ordinal = rd32(r.base, 87u);
    r.flags = rd8(r.base, 7u);
    r.back_chain = rd32(r.base, 91u);
    r.chain_value = tab(T_VERSIONS, version * VERSION_WORDS + 4u);
    r.named_count = rd32(r.base, 12u);
    return r;
}

// checker 读法的记录表（走读内核的 checker_insert_record 写）按键 (计数器, 实例代号) 严格升序（walk.rs 的 JournalRecordKeyOnADevice：计数器在前，
// 同一计数器再按实例代号；`records/2026-10-24-singlefs-checker-审计复核.md` 的 C02）：第一个键不小于 (counter, instance) 的位置，都小交回 record_count。
// 二分：每一轮 [low, high) 之前的都小于它、从 high 起的都不小于它，至多 ⌈log2(MAX_RECORDS + 1)⌉ = 13 轮
fn checker_record_position_not_below(counter: vec2<u32>, instance: u32) -> u32 {
    var low = 0u;
    var high = record_count;
    while (low < high) {
        let middle = (low + high) / 2u;
        let middle_counter = vec2<u32>(sw(R_RECORDS + middle * RECORD_WORDS + 1u), sw(R_RECORDS + middle * RECORD_WORDS + 2u));
        if (lt64(middle_counter, counter) || (eq64(middle_counter, counter) && sw(R_RECORDS + middle * RECORD_WORDS + 3u) < instance)) {
            low = middle + 1u;
        } else {
            high = middle;
        }
    }
    return low;
}

fn checker_record_counter_at_is(position: u32, counter: vec2<u32>) -> bool {
    return position < record_count && sw(R_RECORDS + position * RECORD_WORDS + 1u) == counter.x
        && sw(R_RECORDS + position * RECORD_WORDS + 2u) == counter.y;
}

fn checker_record_key_at_is(position: u32, counter: vec2<u32>, instance: u32) -> bool {
    return checker_record_counter_at_is(position, counter) && sw(R_RECORDS + position * RECORD_WORDS + 3u) == instance;
}

// 键 (counter, instance) 那一条的下标，没有就是 NONE（walk.rs 按 JournalRecordKeyOnADevice 取的那几处）
fn checker_find_record_by_key(counter: vec2<u32>, instance: u32) -> u32 {
    let position = checker_record_position_not_below(counter, instance);
    if (checker_record_key_at_is(position, counter, instance)) {
        return position;
    }
    return NONE;
}

// 计数器 counter 上坐没坐着记录，哪个实例的都算（walk.rs 的 records_at_counter(..).next().is_some()）：键里实例代号最小是 0，
// 不小于 (counter, 0) 的第一个位置上计数器就是它才有
fn checker_some_record_sits_at_counter(counter: vec2<u32>) -> bool {
    return checker_record_counter_at_is(checker_record_position_not_below(counter, 0u), counter);
}

// ───────────────────────── 走读（walk.rs 的 Walk）─────────────────────────
//
// 草稿区（接在共用那一半的 SCRATCH_WORDS_REQUIRED 之后）。两套上下文：0 是主走读（判定进违例位、引用集合给 I-3.1 / I-5.1），
// 1 是「另起一份 Walk」的那几遍（被抛弃根、F 之下的根）：判定丢掉、只数走读失败与 I-2.1 不符、引用另放。
const C_REFERENCE_WORDS: u32 = 5u;          // 盘、槽 lo、槽 hi、跨度、0

// 引用集合（这里）与 I-3.5 落点表（C_APPEARANCE_MAX_PLACEMENTS）在不回收的流上各版累计下来与
// 最新根的分配记录同数（走过的各版的并集就是分配过的每个落点，两盘各一份）。两张都是 4096 条，分配记录那两张表（扫描内核的 C_MAX_ALLOC_RECORDS、
// 恢复内核的 MAX_ALLOCATION_RECORDS）抬到 8192 条之后比它们窄：不回收的流上分配记录多于 4096 条时先满的是走读内核这两张（站点 40、1；
// 同一条指针先记两个落点、再记两条引用，结论里留下的是哪一个站点看走读走到哪里，没量）。走读判不了扫描内核就不跑，扫描内核那张（站点 20）
// 在这种流上走不到；恢复内核那张到 8192 条才满，恢复照判
const C_MAX_REFERENCES: u32 = 4096u;

const C_REFERENCES0: u32 = 32864u;

const C_REFERENCES1: u32 = 53344u;

const C_MAX_VISITED: u32 = 2048u;

// 走读内核一遍记下的中央映射叶都是这一遍新走过的单元（w_walk_code_two_tree）：叶表（crash_judge_common.wgsl 的 R_MAPPING_LEAVES）装得下
const_assert C_MAX_VISITED <= R_MAPPING_LEAF_ROWS;

const C_VISITED0: u32 = 73824u;             // 盘、槽 lo、槽 hi

const C_VISITED1: u32 = 79968u;

const C_MAX_IN_VERSION: u32 = 2048u;

const C_IN_VERSION0: u32 = 86112u;          // 盘、槽 lo、槽 hi

const C_IN_VERSION1: u32 = 92256u;

const C_BELOW_WORDS: u32 = 6u;              // 父盘、父槽 lo/hi、子盘、子槽 lo/hi

const C_MAX_BELOW: u32 = 4096u;

const C_BELOW0: u32 = 98400u;

const C_BELOW1: u32 = 122976u;

const C_LEAF_WORDS: u32 = 9u;               // 父盘、父槽 lo/hi、容器 lo/hi、最小 lo/hi、最大 lo/hi

const C_MAX_LEAVES: u32 = 512u;

const C_LEAVES: u32 = 147552u;

// 152160 起的 32 768 个字（到 C_TREE_IDS 为止）原来是走读内核收的两张位置项校验和表（主走读那一张与候选集之外的根那一张，各 4096 条、4 字一条），
// 池级 checker 只认隔离记号、不读副本之后（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-10-22 定）两张都删了；那一段空着，其后各表的偏移不挪。

const C_MAX_TREE_IDS: u32 = 64u;

const C_TREE_IDS: u32 = 184928u;             // lo、hi

// 185056 起的 2048 个字（到 C_DATA_OBJECTS 为止）原来是 512 条的 inode → 对象出生代表，抬到 C_MAX_INODE_BIRTHS 条时挪到草稿区末尾
// （C_INODE_BIRTHS），其后各表的偏移不挪；那一段空着（第一个字原来记隔离判据「补没补过根环里没走过的根」，只认隔离记号之后不记了）

const C_MAX_DATA_OBJECTS: u32 = 1024u;

const C_DATA_OBJECTS: u32 = 187104u;         // inode lo/hi、出生代 lo/hi

const C_MAX_ACCOUNTING_ROWS: u32 = 64u;

const C_ACCOUNTING_ROWS: u32 = 191200u;      // 统计量、盘、值 lo/hi

const C_ROW_WORDS: u32 = 5u;                // 实例、txg lo/hi、W lo/hi

// 191456 起的 5120 个字（到 C_CHILDREN 为止）空着：原来是 1024 行的最新根实例表行，抬到 C_MAX_INSTANCE_ROWS 行时挪到草稿区末尾
// （C_INSTANCE_ROWS，连同它的散列索引），其后各表的偏移不挪

const C_CHILD_WORDS: u32 = 7u;              // 种类、最小 key 基/偏移、最大 key 基/偏移、分隔 key 基/偏移

const C_MAX_CHILDREN: u32 = 1024u;

const C_CHILDREN: u32 = 196576u;

const C_SEPARATOR_WORDS: u32 = 6u;          // 分隔 lo/hi、最小 lo/hi、最大 lo/hi

const C_MAX_SEPARATORS: u32 = 256u;

const C_SEPARATORS: u32 = 203744u;

// 205280 起到 C_SCRATCH_END 为止的 6144 个字原来是 w_merge_below 的显式栈（这里起 1024 条）与展开过的单元（208352 起 1024 条），
// 那两张抬上限时挪到草稿区末尾（C_MERGE_PENDING、C_MERGE_EXPANDED），其后各表的偏移不挪。这一段留给原来就借它的两处：一版中央映射只读读法的显式栈
// （read_the_central_mapping_without_judging，C_MAX_PENDING 条，每条字节基、指针偏移、层级）与 I-3.5 摊单元（crash_judge_checker.wgsl 的
// APPEARANCE_UNITS，整段），两处借它的时段不重叠（摊单元在主走读那几版都走完之后，那时不再读映射）
const C_MAX_PENDING: u32 = 1024u;

const C_PENDING: u32 = 205280u;

const C_SCRATCH_END: u32 = 211424u;

const_assert C_PENDING + 3u * C_MAX_PENDING <= C_SCRATCH_END;

// ───────────────────────── 第三批：只读不判的读法（walk.rs 的 *_without_judging）与扫描、记账那几条 ─────────────────────────
// 已释放记录是从最新根的分配记录那张表（C_MAX_ALLOC_RECORDS 条，满了记站点 20）里挑的，条数到不了那张表的条数：两张表取同一个上限，
// 扫描内核站点 29（已释放记录表满了）因此走不到，留着守这张表的写入边界（推理落成的断言在
// tests/crash_judge_gpu_tables_at_their_capacity.rs）
const C_MAX_RELEASED: u32 = C_MAX_ALLOC_RECORDS;

// 211424 起的 14336 个字（到 C_CANDIDATES 为止）原来空着：原来是 2048 条的已释放记录表（C_RELEASED，盘、槽 lo/hi、释放代 lo/hi）与它的
// 「哪条候选根引用它」（C_REFERENCED_BY，每条两个字：位 c ↔ 第 c 条候选根引用它），随 C_MAX_ALLOC_RECORDS 抬到 4096 条时挪到草稿区末尾，其后各表的偏移不挪。
// 候选根的信息行（C_CANDIDATE_INFO）随 C_MAX_TREES_PER_TABLE 抬到 32 时从 225808 挪到这一段的起点（见那一处）

const C_MAX_CANDIDATES: u32 = 48u;

const C_CANDIDATES: u32 = 225760u;            // 候选根在根表里的下标

const C_CANDIDATE_WORDS: u32 = 134u;         // 0 走全了、1 有树表落点、2 盘、3 槽 lo、4 槽 hi、5 树数、6.. 每树 4 字（树 lo/hi、诞生 lo/hi）：6 + 4 × C_MAX_TREES_PER_TABLE

const_assert C_CANDIDATE_WORDS == 6u + 4u * C_MAX_TREES_PER_TABLE;

// 一张树表最多记几棵不同的树：第一版合法镜像上一张树表能有的最多条目——进发号次序的种类 12 个（TREE_KINDS_OF_THE_TREE_TABLE，
// recovery.rs 与 walk.rs 的 TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER；每种至多一条，第一个文件版本那次就写满 12 条），
// 再加每个只读快照一对码 14 / 15 条目（各有自己的树 ID），快照至多 SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION 个
// （records.rs 的同名常量，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 1 用户定 10）：12 + 2 × 10 = 32，
// 不越过单节点树表能装的 34 条（同一份判决第七节岔路 1「树表封顶 34」）。多于 32 棵的只出现在坏镜像上（码 14 多于 10 条读树表的每一处都拒），
// 第 33 棵起判不了（站点 28）、交 CPU。从 12 抬到 32 时信息行每行从 54 字加到 134 字，48 行装不进 225808 起到 C_SCAN_TABLES_THE_WALK_BUCKETS_BORROW
// 那 2592 个字，整张挪到 211424 起空着的那一段（下面的 const_assert 核装得下）；225808 起那 2592 个字空着，其后各表的偏移不挪，草稿区末端（SCRATCH_END）不变
const C_MAX_TREES_PER_TABLE: u32 = 32u;

const_assert C_MAX_TREES_PER_TABLE == TREE_KINDS_OF_THE_TREE_TABLE + 2u * SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION;

// I-9.28 在 I-3.9 与 I-9.14 判完之后借这几行，根表里第 k 条根记进第 k 行（只用 1..5 与每树那几字）：根表最多 MAX_ROOTS 条，
// 不多于 C_MAX_CANDIDATES 行（judge_tree_identifier_birth_txg_across_every_readable_root 开头判，越过就判不了）。
// 放在原来已释放记录表空出来的那一段（211424 起，到 C_CANDIDATES 为止）：C_PENDING 起的那一段到 C_SCRATCH_END 为止，与它首尾相接
const C_CANDIDATE_INFO: u32 = C_SCRATCH_END;

const_assert C_CANDIDATE_INFO + C_CANDIDATE_WORDS * C_MAX_CANDIDATES <= C_CANDIDATES;

const C_ALLOC_RECORD_WORDS: u32 = 8u;        // 盘、槽 lo/hi、跨度、代 lo/hi、已释放、0

// 扫描内核收一条根的分配记录那张表（c_append_allocation_record，满了记站点 20）：从 2048 抬到 4096 条，与恢复内核那张（MAX_ALLOCATION_RECORDS）同宽。
// 容器树旋钮压到一片 1 条、2 条的随机历史第 48、52 步起最新根的分配记录多于 2048 条（CPU 一侧数到第 59 步 2534 条、2342 条；已释放的 2292 条，
// 多于原来的 C_MAX_RELEASED）。表挪到草稿区末尾（C_ALLOC_RECORDS）；228400 起原来那 16384 个字留给走读内核的桶表借（W_REFERENCE_BUCKETS 起）。
// 再从 4096 抬到 8192 条，与恢复内核那张同时抬（C894）：整理夹着回退那一组比重 40 步交判的 17 张镜像每版至多 4018 条，离 4096 只差 78；
// 同比重拉到 80 步那一组 45 张里 11 张、160 步那一组 105 张里 27 张多于 4096 条，至多 4728 条（CPU 一侧数的）。与它同宽的已释放记录、
// 释放代高于 F 的已释放记录、I-3.10 去重表三张跟着抬（⑫ 里这四张照偏移链往后挪）；原来借的段装不下的两样另开两段（⑮）：I-5.4 与 I-3.16
// 排分配记录的次序表（原来借 C_ROW_ORDER 那 4096 个字）与 I-3.9、I-3.10 的两张桶表（原来借 C_JOURNAL_GROUP_KEYS 那 8192 个字）。
// 走读内核的引用集合、I-3.5 落点表两张仍是 4096 条，比这一张窄：不回收的流上分配记录多于 4096 条时先满的是那两张（站点 40、1，
// 见 C_MAX_REFERENCES 上面那段），这一张（站点 20）在那种流上走不到；那几条随机历史上三张各至多 800 条（各版累计，CPU 一侧数的）
const C_MAX_ALLOC_RECORDS: u32 = 8192u;

// 走读内核那几张桶表借的那一段的起点：扫描内核原来的分配记录表在这里起，挪走之后这一段只剩扫描内核写过才读的那几张表（C_R_SEEN、C_R_PENDING、
// C_SEEN_NODES；I-3.10 的去重表原来也在这一段，抬上限时挪到草稿区末尾）
const C_SCAN_TABLES_THE_WALK_BUCKETS_BORROW: u32 = 228400u;

const C_R_MAX_SEEN: u32 = 512u;

const C_R_SEEN: u32 = 244784u;                // 盘、槽 lo/hi

const C_R_PENDING_WORDS: u32 = 10u;

const C_R_MAX_PENDING: u32 = 512u;

const C_R_PENDING: u32 = 246320u;

const C_MAX_SEEN_NODES: u32 = 256u;

const C_SEEN_NODES: u32 = 251440u;           // 盘、槽 lo/hi、校验和

// I-3.10 的去重表（扫描内核 judge_allocation_generations_of_the_records；满了之后新来的记录不进表、照判，不再记判不了）：候选集里每条根、由记录施加出来的那几版与
// 下一次挂载先施加的那一版，各版未释放的分配记录按（盘、槽、代）去重，整个状态累计一张。从 2048 抬到与一条根的分配记录那张表
// （C_MAX_ALLOC_RECORDS）同宽：不回收的流上分配过的记录一直留在树里（释放了改成已释放），各版未释放记录去重累计的条数跟着最新那一版的
// 分配记录条数长。CPU 一侧数的：轮四遍再长到 56、57、76 个单元的三条流各版累计 2046、2048、2086 条（被判那一版分配记录 2048、2050、2088 条），
// 2086 条那一条在旧的界上每个状态判不了；轮八遍再长到 167 个那一条累计 4096 条。原来表满记判不了（站点 31），此刻表满之后不再去重、照判：
// I-3.10 是置位，同一条记录判两遍结论相同，表多大只管少读几次单元头，不管判不判得了。
// 表挪到草稿区末尾（⑫ 的 C_EXAMINED）；252464 起原来那 10240 个字扫描内核不再用（走读内核的桶表照旧借其中到 W_BUCKETS_END 为止的那一段），
// 其后各表的偏移不挪。去重桶表的桶数取表的两倍（扫描内核的 C_EXAMINED_BUCKET_COUNT），借 ⑮ 的 C_ALLOCATION_RECORD_BUCKETS 那一段
// （原来借 C_JOURNAL_GROUP_KEYS 那 8192 个字，表随 C_MAX_ALLOC_RECORDS 抬到 8192 条时桶表 16384 个字装不下）
const C_MAX_EXAMINED: u32 = C_MAX_ALLOC_RECORDS;

// I-1.8 收的扫描项（单元槽上码 1 / 码 3 的已发布单元）与扫描重建那一遍（SCAN_REBUILD_MAX_UNITS）同宽，满了记站点 36（扫描重建记站点 110）。
// 从 2048 抬到 4096：整理夹着回退那一组比重偏移 0–3、40 步交判的 17 张镜像里两张 2148、3030 项（CPU 一侧数的），在站点 36 判不了；
// 同比重 259 张（加种子、加长到 160 步）至多 3030 项。条数随盘上留着的已发布单元涨，内核里没有别的表给它上界；4096 是 I-1.8 借的次序表
// （C_ROW_ORDER，C_MAX_REFERENCES 个字）的宽，再宽要给次序另开一张、扫描重建那几张表挪到草稿区末尾
const C_MAX_SCANNED: u32 = 4096u;

// 262704 起原来那 2048 个字空着；挪进 267152 起空着的那 8192 个字（C_PICKED 之后、C_DEFERRED 之前），草稿区不涨
const C_SCANNED: u32 = 267152u;              // 读出来那一份切片的字节基（I-1.8 收的扫描项，按收的次序）

const_assert C_PICKED + 2u * C_MAX_CANDIDATES <= C_SCANNED;

const_assert C_SCANNED + C_MAX_SCANNED <= C_DEFERRED;

// r_instance_rows 读一条根自己那张实例表的全部行（扫描内核站点 32 判满）。与恢复内核读同一种表的 MAX_ROWS（crash_judge_common.wgsl）同宽：
// 256 行时取号之后崩溃 369 次再挂载写 370 行那条流（crash_enumeration_position_addressed_trees.rs）在这里判不了而恢复内核判得了；
// 同宽之后恢复内核装得下的行数扫描内核也装得下。从 256 加到 512 时这张表每行 3 字多 256 × 3 = 768 字，其后的草稿区偏移各后移 768 字
// 从 512 抬到 24576 时（取号之后崩溃 65 × 369 次再挂载十二次的那个场景，最新根的实例表 23997 行、66 片，在站点 32 判不了）挪到草稿区末尾
// （C_R_ROWS），264752 起原来那 1536 个字空着，其后各表的偏移不挪
const C_R_MAX_ROWS: u32 = MAX_ROWS;

// I-7.9 一条根之前的有效根，每条一行：根下标、有指针、inode 基、inode 偏移、extent 基、extent 偏移，再四个字是这条根的树表节点（字节基、条目起点、
// 条目宽、条目数；码 14、15 那几条快照树的根指针按树表次序从这里读，「非空」比它们，用户 2026-10-14 定建快照那次算非空）。
// 每行从 6 字加到 10 字时挪到 264752（上面那 1536 个字空着的那一段的起点），266288 起原来那 288 个字并进空着的那一段
const C_VALID_BEFORE_WORDS: u32 = 10u;

const C_VALID_BEFORE: u32 = 264752u;

const_assert C_VALID_BEFORE + C_VALID_BEFORE_WORDS * C_MAX_CANDIDATES <= 266576u;

const C_NON_EMPTY: u32 = 266576u;            // txg lo/hi、实例、valid_before 下标

const C_UNDETERMINABLE: u32 = 266768u;       // txg lo/hi、实例

const C_COUNTED: u32 = 266912u;              // txg lo/hi

const C_COUNTED_STATES: u32 = 267008u;       // valid_before 下标

const C_PICKED: u32 = 267056u;               // 选过的标记（非空 48 + 判不了 48）

// 扫描内核 c_deferred_below_the_floor 收最新根树表里分配记录树的已释放、释放代高于有效回退下界 F 的记录（满了记站点 34）：从 2048 抬到
// 与一条根的分配记录那张表（C_MAX_ALLOC_RECORDS）同宽。容器树旋钮压到一片 1 条、2 条的随机历史 F = 0，最新根的已释放记录全数进这张表：
// 旋钮 1 第 53 步 2078 条、第 54 步 2100 条，旋钮 2 第 55 步 2060 条、第 56 步 2114 条（CPU 一侧数的），在站点 34 判不了。
// 合法镜像的树表里分配记录树只有一棵，这几条是从那一棵的记录里挑的，不多于它；树表里多于一棵的坏镜像才走得到站点 34。
// 表挪到草稿区末尾（⑫ 的 C_RELEASED_ABOVE）；267152 起原来那 8192 个字空着，其后各表的偏移不挪（前 4096 个字后来给了 C_SCANNED）
const C_MAX_RELEASED_ABOVE: u32 = C_MAX_ALLOC_RECORDS;

const C_MAX_DEFERRED: u32 = 4096u;

const C_DEFERRED: u32 = 275344u;             // 盘、槽 lo/hi

const C_MAX_GROUPS: u32 = 2048u;

const C_GROUPS: u32 = 287632u;               // 槽 lo/hi、跨度、盘位掩码

const C_SCRATCH_END_SCAN: u32 = 295824u;

var<private> w_context: u32;

var<private> w_judging: bool;

var<private> w_reference_count: array<u32, 2>;

var<private> w_visited_count: array<u32, 2>;

var<private> w_in_version_count: array<u32, 2>;

var<private> w_below_count: array<u32, 2>;

// 挂父单元那张表的散列索引（W_BELOW_INDEX0 / W_BELOW_INDEX1 起的桶表）在这一份 Walk 里建过没有，按上下文分
var<private> w_below_rows_are_indexed: array<bool, 2>;

var<private> w_leaf_count: u32;

var<private> w_tree_id_count: u32;

var<private> w_inode_birth_count: u32;

var<private> w_data_object_count: u32;

var<private> w_accounting_row_count: u32;

var<private> w_accounting_seen: bool;

var<private> w_instance_row_count: u32;

var<private> w_failures: u32;

var<private> w_mismatches: u32;

var<private> w_inode_tree_walked: bool;

var<private> w_have_largest_inode: bool;

var<private> w_largest_inode: vec2<u32>;

var<private> w_parent_valid: bool;

var<private> w_parent_device: u32;

var<private> w_parent_slot: vec2<u32>;

var<private> w_mount_instance: u32;

var<private> w_mount_txg: vec2<u32>;

var<private> w_child_count: u32;

var<private> w_separator_count: u32;

var<private> c_newest_index: u32;

var<private> c_effective_floor: vec2<u32>;

var<private> c_version_count: u32;

var<private> c_candidate_count: u32;

var<private> c_newest_reference_count: u32;

// 已发布谓词（PublishedPredicate::holds_for）
fn w_published(birth_txg: vec2<u32>, instance: u32, is_data_unit: bool, transaction: vec2<u32>) -> bool {
    if (instance > w_mount_instance) {
        return false;
    }
    if (instance == w_mount_instance) {
        return le64(birth_txg, w_mount_txg);
    }
    // 看这个实例的第一行（按记进的次序）
    let first = w_first_instance_row_of(instance);
    if (first == NONE) {
        return true;
    }
    let row = C_INSTANCE_ROWS + first * C_ROW_WORDS;
    let row_txg = vec2<u32>(sw(row + 1u), sw(row + 2u));
    let row_high_water = vec2<u32>(sw(row + 3u), sw(row + 4u));
    if (is_data_unit) {
        return le64(birth_txg, row_txg) || le64(transaction, row_high_water);
    }
    return le64(birth_txg, row_txg);
}

// key 形态：0 记账 [2,8,4,8]、1 映射 [1,8,8,4,6]、2 树表与 inode [8]、3 extent [8,8,8]、4 分配记录 [4,6]、
// 5 deadlist [8,8,1,8,8,4,6]（crash_judge_common.wgsl 的 KEY_SCHEMA_DEADLIST，字段宽与 field_width 同一份）、
// 6 logged_ops [2,8]（crash_judge_common.wgsl 的 KEY_SCHEMA_LOGGED_OPERATIONS，字段宽与 field_width 同一份；段数 2 与分配记录相同，走末尾那一句）
fn schema_field_count(schema: u32) -> u32 {
    if (schema == 0u) { return 4u; }
    if (schema == 1u) { return 5u; }
    if (schema == KEY_SCHEMA_DEADLIST) { return field_count(schema); }
    if (schema == 2u) { return 1u; }
    if (schema == 3u) { return 3u; }
    return 2u;
}

fn schema_field_width(schema: u32, field: u32) -> u32 {
    if (schema == 0u || schema == 1u || schema == KEY_SCHEMA_DEADLIST || schema == KEY_SCHEMA_LOGGED_OPERATIONS) {
        return field_width(schema, field);
    }
    if (schema == 2u) { return 8u; }
    if (schema == 3u) { return 8u; }
    if (field == 0u) { return 4u; }
    return 6u;
}

fn schema_key_width(schema: u32) -> u32 {
    var width = 0u;
    let count = schema_field_count(schema);
    for (var f = 0u; f < count; f = f + 1u) {
        width = width + schema_field_width(schema, f);
    }
    return width;
}

// 按字段比两把 key：0 小于、1 等于、2 大于
fn schema_compare(schema: u32, base_a: u32, oa: u32, base_b: u32, ob: u32) -> u32 {
    var o1 = oa;
    var o2 = ob;
    let count = schema_field_count(schema);
    for (var f = 0u; f < count; f = f + 1u) {
        let w = schema_field_width(schema, f);
        let a = read_field(base_a, o1, w);
        let b = read_field(base_b, o2, w);
        if (lt64(a, b)) { return 0u; }
        if (lt64(b, a)) { return 2u; }
        o1 = o1 + w;
        o2 = o2 + w;
    }
    return 1u;
}

// 链指针记录：0 最后一片、1 有下一片、2 坏（walk.rs chain_record_view）。有下一片还要位置指针头部 extent 偏移及之前那几段都是第一版的取值
// （MAC / nonce、算法类型、压缩算法码、压后长度经 crash_judge_common.wgsl 的 pointer_encryption_and_compression_fields_are_the_first_version，
// extent 偏移经 pointer_extent_offset_is_zero）：有一段不过是 I-3.14 第五形，这一片解不开（用户 2026-10-06 定 K2-1 甲，第 10u 批）
fn w_chain_record_kind(base: u32, o: u32, width: u32) -> u32 {
    if (width != 88u || rd8(base, o) != 1u) {
        return 2u;
    }
    let pointer_zero = all_zero(base, o + 2u, 86u);
    let flag = rd8(base, o + 1u);
    if (flag == 0u && pointer_zero) {
        return 0u;
    }
    if (flag == 1u && !pointer_zero && pointer_encryption_and_compression_fields_are_the_first_version(base, o + 2u)
        && pointer_extent_offset_is_zero(base, o + 2u)) {
        return 1u;
    }
    return 2u;
}

// 被最新根指着的实例表判抛弃：有行 (i, Ti, ·) 且 T > Ti
fn w_root_abandoned(instance: u32, txg: vec2<u32>) -> bool {
    for (var row_index = w_first_instance_row_of(instance); row_index != NONE; row_index = w_next_instance_row_after(row_index)) {
        let row = C_INSTANCE_ROWS + row_index * C_ROW_WORDS;
        if (lt64(vec2<u32>(sw(row + 1u), sw(row + 2u)), txg)) {
            return true;
        }
    }
    return false;
}

// (txg, 实例) 字典序：a < b
fn root_key_less(txg_a: vec2<u32>, instance_a: u32, txg_b: vec2<u32>, instance_b: u32) -> bool {
    return lt64(txg_a, txg_b) || (eq64(txg_a, txg_b) && instance_a < instance_b);
}

// record_can_enter_a_replay_prefix：计数器 0 进不了；本实例内逻辑前一条（同实例、计数器 − 1，按 (实例代号, 计数器) 找）在恢复视图里就要链值相等，
// 不在照进（别的实例坐在计数器 − 1 上也是不在）
fn record_can_enter_a_replay_prefix(index: u32) -> bool {
    let r = checker_record_at(index);
    if (is_zero64(r.counter)) {
        return false;
    }
    let previous_index = checker_find_record_by_key(sub64(r.counter, u64_of(1u)), r.instance);
    if (previous_index == NONE) {
        return true;
    }
    return r.back_chain == checker_record_at(previous_index).chain_value;
}

// 候选根（最新根，或没被抛弃且 txg ≥ F 生效值的）
fn is_candidate_root(index: u32) -> bool {
    if (index == c_newest_index) {
        return true;
    }
    let r = checker_root_of(index);
    return !w_root_abandoned(r.instance, r.txg) && !lt64(r.txg, c_effective_floor);
}

// ── 记账那几条：I-3.1、I-5.2、I-3.11
// 引用集合里有没有起在 (盘, 槽) 的引用（前 limit 条：只走最新根那一刻，或全部）
fn c_referenced_start(device: u32, slot: vec2<u32>, limit: u32) -> bool {
    for (var i = 0u; i < limit; i = i + 1u) {
        let row = C_REFERENCES0 + i * C_REFERENCE_WORDS;
        if (sw(row) == device && sw(row + 1u) == slot.x && sw(row + 2u) == slot.y) {
            return true;
        }
    }
    return false;
}

// 全部走过的版本引用的槽里有没有 (盘, 槽)（按跨度展开）
fn c_walked_slot(device: u32, slot: vec2<u32>) -> bool {
    for (var i = 0u; i < w_reference_count[0]; i = i + 1u) {
        let row = C_REFERENCES0 + i * C_REFERENCE_WORDS;
        if (sw(row) != device) {
            continue;
        }
        let start = vec2<u32>(sw(row + 1u), sw(row + 2u));
        if (le64(start, slot) && lt64(slot, add64(start, u64_of(sw(row + 3u))))) {
            return true;
        }
    }
    return false;
}

var<private> w_accounting_recording: bool;


// ───────────────────────── 走读内核交给扫描内核的东西（草稿区里的交接段）─────────────────────────
// 两个内核各自编（驱动编一个 4900 行的内核要 41 分钟、13 GiB；拆开各编各的、并行）；草稿区在同一次派活里两趟共用，
// 集合都留在草稿区里，只把私有变量里的计数与几何写进这一段。
const H_BASE: u32 = 295824u;
const H_VALID: u32 = 0u;                 // 1 = 走读那一批判完、判得了；0 = 扫描内核什么都不做
const H_ROW_COUNT: u32 = 1u;
const H_REFERENCE_COUNT: u32 = 2u;
const H_NEWEST_REFERENCE_COUNT: u32 = 3u;
// 4、5 两个字原来是走读内核两张位置项校验和表的条数，那两张删了之后空着
const H_ACCOUNTING_ROW_COUNT: u32 = 6u;
const H_ACCOUNTING_SEEN: u32 = 7u;
const H_CANDIDATE_COUNT: u32 = 8u;
const H_NEWEST_INDEX: u32 = 9u;
const H_FLOOR_LO: u32 = 10u;
const H_FLOOR_HI: u32 = 11u;
const H_VERSION_COUNT: u32 = 12u;
const H_RECORD_COUNT: u32 = 13u;
const H_MOUNT_INSTANCE: u32 = 14u;
const H_MOUNT_TXG_LO: u32 = 15u;
const H_MOUNT_TXG_HI: u32 = 16u;
const H_ROOT_COUNT: u32 = 17u;
const H_UNIT_AREA_START_LO: u32 = 18u;
const H_UNIT_AREA_START_HI: u32 = 19u;
const H_REGIONS: u32 = 20u;
const H_REGION0: u32 = 21u;
const H_REGION1: u32 = 22u;
const H_REGION2: u32 = 23u;
const H_FSID0: u32 = 24u;
const H_FSID1: u32 = 25u;
const H_FSID2: u32 = 26u;
const H_FSID3: u32 = 27u;
const H_BELOW_FLOOR_SLOT_COUNT: u32 = 28u;
const H_SLOTS_PER_REGION: u32 = 29u;     // 根环每区槽数 S：I-7.9 的一圈窗口宽 3S = H_REGIONS × 它
// 1 = 主走读沿最新根的实例表链走到了末片（走读内核的 w_newest_instance_table_chain_reached_its_last_page）：扫描内核 I-9.29 跨版本那张表比不比读它。
// 第 30、31 两个字原来空着（H_BELOW_FLOOR_SLOTS 从 H_BASE + 32 起），占第 30 个，交接段不变长
const H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE: u32 = 30u;
// 走读内核照 I-4.1 那一份重放（replay_the_next_mount_would_do）施加的第一次发布的末条，在重放读的那一份环（C_REPLAY_RECORDS）里的下标，
// 没有是 NONE：扫描内核认下一次挂载先施加的那一版（I-3.10、I-5.4）读它。占原来空着的第 31 个字，交接段不变长
const H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED: u32 = 31u;
const H_MAX_BELOW_FLOOR_SLOTS: u32 = 4096u;
const H_BELOW_FLOOR_SLOTS: u32 = 295856u;   // 盘、槽 lo/hi
const H_END: u32 = 308144u;

// ───────────────────────── 交接段之后：走读内核写的两张表（扫描内核只读重放那一张）─────────────────────────
// 主走读（上下文 0）每个走过的单元一行：有没有记下诞生代号、诞生代号 lo/hi（walk.rs 的 birth_txg_of_walked_units，与 C_VISITED0 同下标）。
// 别的版本再引用它时按这张表判 I-4.9（非最新候选根引用的单元不晚于它）。
const C_VISITED_BIRTH_WORDS: u32 = 3u;

const C_VISITED_BIRTHS0: u32 = 308144u;

// 一行第 0 字的低两位是诞生代号的标记：0 没记、1 被引用单元（w_judge_birth_identity 记）、2 实例表一片（appearance_note_the_birth_txg_of_an_instance_table_page 记）
const C_VISITED_BIRTH_MARK_MASK: u32 = 3u;

// 同一个字的位 2：先走那一遍头用得了（walk.rs 的 walked_units_whose_header_the_walk_could_use：码 1 / 码 2 头判得过，码 3 容器格式也判得过）。
// 别的版本再引用这个单元时，这一条引用自己定的那几格（出生身份、树 ID、五元组、片号、指针出生树）只在这一位置着时照判
// （w_judge_the_cells_the_reference_decides_on_a_unit_walked_earlier，`records/2026-10-24-singlefs-checker-审计复核.md` 的 C01）。
// 借这张表现有的字，不另开一张：一行与 C_VISITED0 同下标，C_MAX_VISITED 行都有这一位，草稿区不涨
const C_VISITED_HEADER_THE_WALK_COULD_USE: u32 = 4u;

// 同一个字的位 3：先走那一遍这个码 2 节点解得开（parse_index_node 交回 ok，walk.rs 的 index_node_view 交回 Ok）。别的版本再引用它时 I-1.3 只在这一位置着时照判
// （walk.rs 在命中那一处现解一遍，同一个落点同一份字节解出来一样；这里记一位，命中时不再展开一份解节点）
const C_VISITED_INDEX_NODE_PARSES: u32 = 8u;

const_assert 0u == (C_VISITED_BIRTH_MARK_MASK & (C_VISITED_HEADER_THE_WALK_COULD_USE | C_VISITED_INDEX_NODE_PARSES));

const_assert 0u == (C_VISITED_HEADER_THE_WALK_COULD_USE & C_VISITED_INDEX_NODE_PARSES);

const_assert 2u <= C_VISITED_BIRTH_MARK_MASK;

// I-4.1 重放读的那一份环（walk.rs 的 records_the_replay_reads）：版本、计数器 lo/hi、实例，按 (实例, 计数器) 升序。
// 走读内核在 I-4.1 那一阶段填；扫描内核照交接段的 H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED 读其中一行、不写
const C_REPLAY_RECORD_WORDS: u32 = 4u;

// MAX_RECORDS 抬到 4096 时挪到草稿区末尾（⑯，接在 journal 记录表 R_RECORDS 之后）：314288 起原来那 8192 个字空着，其后的偏移不挪
const C_REPLAY_RECORDS: u32 = END_OF_THE_JOURNAL_RECORD_TABLE_MOVED_PAST_THE_CHECKER_TABLES;

fn replay_record_at(index: u32) -> CheckerRecord {
    return checker_record_of_row(C_REPLAY_RECORDS + index * C_REPLAY_RECORD_WORDS);
}

// I-3.5（引用区间的精确性）的出现规则读的四张表，只记主走读（上下文 0）走的那几版（最新根、回退候选根、由记录施加出来的版本）。
// ① 落点表（walk.rs 的 unit_of_each_placement 与 user_visible_tree_of_each_placement_in_this_version）：跟着指针记下的每个落点一行——
//    落点盘、槽 lo/hi，它是哪个单元（第一次记下它的那条指针第 0 条位置条目的盘、槽 lo/hi，跨版本一张），正在走的这一版给它打的用户可见树标签：
//    打标签那一版的戳（走读内核的 appearance_label_stamp，0 是哪一版都没打过；戳不是正在走的这一版的，这一版当它没有标签）、那棵树的种类、树 ID lo/hi。
//    标签按版本记（判决 research/prompts/m4-segment-code-r1-main-verification.md 第三节 F9）：一版走完时冻进 ③，后走的版本改这几个字不影响先走的那一版。
const C_APPEARANCE_PLACEMENT_WORDS: u32 = 10u;

// 为什么取 4096 见 C_MAX_REFERENCES 上面那段
const C_APPEARANCE_MAX_PLACEMENTS: u32 = 4096u;

const C_APPEARANCE_PLACEMENTS: u32 = 322480u;

// ② 版本表（walk.rs 的 versions_walked_for_the_appearance_rule）：走完的每一版一行——实例、txg lo/hi、走读一条失败都没记（1 / 0）、
//    这一版引用的落点在 ③ 里的起点与条数、树表里用户可见树的根几棵，之后每棵 5 个字（种类、树 ID lo/hi、根指针出生 txg lo/hi）。
const C_APPEARANCE_VERSION_WORDS: u32 = 48u;

const C_APPEARANCE_MAX_USER_VISIBLE_ROOTS: u32 = 8u;

const C_APPEARANCE_USER_VISIBLE_ROOT_WORDS: u32 = 5u;

const C_APPEARANCE_VERSION_ROOTS_OFFSET: u32 = 7u;

const C_APPEARANCE_MAX_VERSIONS: u32 = 64u;

const C_APPEARANCE_VERSIONS: u32 = 363440u;

// ③ 每一版引用的落点（walk.rs 的 placements_referenced_in_this_version 在这一版走完那一刻的样子）：每个落点一个字，低 16 位是它在 ① 里的下标，
//    往上是这一版给它打的用户可见树标签码（walk.rs 的 VersionPlacementsForTheAppearanceRule::user_visible_tree_of_each_placement）：
//    0 是这一版没打，r + 1 是这一版 ② 那一行里第 r 棵用户可见树的根。
//    条数取结构上界：版本至多 C_APPEARANCE_MAX_VERSIONS 版（满了站点 43），每版至多 C_MAX_IN_VERSION 个落点（满了站点 4），这两处先于这张表判不了，
//    这张表满了的那一半（站点 44 里 appearance_version_placement_count 到顶）在每个输入上都走不到。原来 8192 个字：整理夹着回退那一组比重偏移 0–3、40 步
//    交判的 17 张镜像里两张各版合计 19916、18664 个（CPU 一侧数的），在站点 44 判不了；同比重拉到 160 步至多 23452 个
const C_APPEARANCE_MAX_VERSION_PLACEMENTS: u32 = C_APPEARANCE_MAX_VERSIONS * C_MAX_IN_VERSION;

const_assert C_APPEARANCE_MAX_VERSION_PLACEMENTS == C_APPEARANCE_MAX_VERSIONS * C_MAX_IN_VERSION;

// ③ 一个字里落点表下标那一段与标签码的起位
const APPEARANCE_PLACEMENT_INDEX_MASK: u32 = 0xFFFFu;

const APPEARANCE_LABEL_SHIFT: u32 = 16u;

const_assert C_APPEARANCE_MAX_PLACEMENTS <= APPEARANCE_PLACEMENT_INDEX_MASK + 1u;

// ④ 第 3 字里第一个走到这片树表的那一版的下标的起位（低 8 位是根几棵）
const APPEARANCE_FIRST_WALKER_SHIFT: u32 = 8u;

const_assert C_APPEARANCE_MAX_USER_VISIBLE_ROOTS < (1u << APPEARANCE_FIRST_WALKER_SHIFT);

// 366512 起原来那 8192 个字空着：每一版引用的落点那张表挪到草稿区末尾，接在被抛弃根引用的槽那张表（W_ABANDONED_REFERENCED_SLOTS）之后
const C_APPEARANCE_VERSION_PLACEMENTS: u32 = W_ABANDONED_REFERENCED_SLOTS + 3u * W_MAX_ABANDONED_REFERENCED_SLOTS;

// ④ 走过的树表（walk.rs 的 tree_table_entries_of_a_walked_tree_table 里 I-3.5 读的那一样）：第一个读得出它的那一版记下树表单元的盘、槽 lo/hi，
//    第 3 字低 8 位是它里面用户可见树的根几棵、往上是那一版在 ② 里的下标，之后每棵 5 个字（同 ②）；别的版本共享这片树表时照它取这一版的根，
//    标签照那一版冻进 ③ 的接上（walk.rs 的 user_visible_tree_of_each_placement_below_a_walked_tree_table）。
const C_APPEARANCE_WALKED_TREE_TABLE_WORDS: u32 = 44u;

const C_APPEARANCE_WALKED_TREE_TABLE_ROOTS_OFFSET: u32 = 4u;

const C_APPEARANCE_MAX_WALKED_TREE_TABLES: u32 = 64u;

const C_APPEARANCE_WALKED_TREE_TABLES: u32 = 374704u;

// ───────────────────────── 走读内核那几张表之后：扫描内核的删除态扫描（walk.rs judge_deletion_state_of_the_versions）与墓碑、谱系那一遍
// （walk.rs judge_tombstones_and_the_lineage_of_the_versions）用的表 ─────────────────────────
// ① 这一版一棵 inode 树的记录（walk.rs inode_records_without_judging）：号 lo/hi、nlink、size lo/hi、对象出生代 lo/hi、截断字段 T lo/hi 与 N_w lo/hi
//    （截断那一批第二轮起 I-9.39 读，一行从 7 字加到 11 字，其后各表照偏移链往后挪）；同一个号再读到时后读到的盖掉先读到的。一版一棵树地重填
const D_INODE_RECORD_WORDS: u32 = 11u;

// 从 1024 抬到 16384 时（一次建的 inode 多到点名超过一条 journal 记录的那个场景，一版一万两千多个 inode：CPU 一侧数到各版合起来 12582 个号）
// 挪到草稿区末尾（D_INODE_RECORDS，连同它的散列索引），
// 377520 起原来那 7168 个字空着，其后各表的偏移不挪
const D_MAX_INODE_RECORDS: u32 = 16384u;

// ② 这一版 extent 树里每个号的数据单元原来记在 384688 起 3 字一行的一张（号 lo/hi、单元数）；截断那一批起每个号还要上段叶条目的声明与每个单元的偏移、
//    数据指针（walk.rs data_units_of_each_inode_without_judging），两张表挪进扫描内核借的那一段（crash_judge_checker_scan.wgsl 的
//    D_DATA_UNITS_OF_EACH_INODE 与 D_DATA_UNITS），384688 起原来那 3072 个字空着，其后各表的偏移不挪。号的条数上限照旧
const D_MAX_DATA_UNIT_INODES: u32 = 1024u;

// ③ 一棵墓碑树或谱系树按叶序读出的容器（walk.rs containers_of_a_container_tree_without_judging）：容器在打包表里的字节基、记录数、
//    第一条记录的偏移、头里的容器号 lo/hi。一棵树地重填；多于 D_MAX_CONTAINERS_OF_A_CONTAINER_TREE 片判不了（交 CPU）
const D_CONTAINER_OF_A_CONTAINER_TREE_WORDS: u32 = 5u;

const D_MAX_CONTAINERS_OF_A_CONTAINER_TREE: u32 = 256u;

const D_CONTAINERS_OF_A_CONTAINER_TREE: u32 = 387760u;

// ④ 这一版谱系树的谱系记录（walk.rs lineage_records_without_judging）不抄进草稿区，一片一片地从容器字节里读，条数不设上限
//    （用户 2026-10-19 定「按块分批读，不设上限」；C779：产品容量下一片 1359 条，原来那张 256 条的表装不下第二片）。草稿区只记谱系树每片容器
//    在打包表里的字节基、记录数、第一条记录的偏移（D_CONTAINERS_OF_A_CONTAINER_TREE 一棵树地重填，退役树读完之后还要按谱系判「在谱系上」）；
//    片数的上限就是读一棵容器树的上限 D_MAX_CONTAINERS_OF_A_CONTAINER_TREE。原来那张表 389040 起 1536 个字，这张占它的前一半
const D_LINEAGE_CONTAINER_WORDS: u32 = 3u;

const D_LINEAGE_CONTAINERS: u32 = 389040u;

const_assert D_LINEAGE_CONTAINERS + D_LINEAGE_CONTAINER_WORDS * D_MAX_CONTAINERS_OF_A_CONTAINER_TREE <= 389040u + 1536u;

// ⑤ I-9.29 跨版本那张表（walk.rs ObjectBirthsOfEachInodeNumber）：inode 树的树 ID lo/hi、inode 号 lo/hi、第一次见到的对象出生代 lo/hi。
//    回退候选集里各版读完一起判，不一版一版重填；多于 D_MAX_OBJECT_BIRTHS 个 (树, 号) 判不了
const D_OBJECT_BIRTH_WORDS: u32 = 6u;

// 从 2048 抬到 16384 时（同上那个场景，跨版本 12582 个号）挪到草稿区末尾（D_OBJECT_BIRTHS，连同它的散列索引），390576 起原来那 12288 个字空着
const D_MAX_OBJECT_BIRTHS: u32 = 16384u;

// ⑥ I-9.30 那一判读的最新根记账里 inode 号水位那一统计量（不带设备维）的每一行（walk.rs Walk::inode_number_watermark_rows）：代段 lo/hi、值 lo/hi。
//    最新根一棵记账树一棵树地收（读不全的那棵退回去不收）；多于 D_MAX_INODE_NUMBER_WATERMARK_ROWS 行判不了
const D_INODE_NUMBER_WATERMARK_ROW_WORDS: u32 = 4u;

const D_MAX_INODE_NUMBER_WATERMARK_ROWS: u32 = 256u;

const D_INODE_NUMBER_WATERMARK_ROWS: u32 = 402864u;

// ⑦ 退役那几条读的两张叶身份表（walk.rs inode_leaf_identities_without_judging）：一版一棵 inode 树里每片叶容器的身份（出生树 lo/hi、打包记录类型、
//    容器号 lo/hi、容器出生代 lo/hi），集合、一棵树地重填。前一张装被判的那一版（I-9.34、I-9.35），后一张装 I-9.35 的父版；
//    一棵树多于 D_MAX_LEAF_IDENTITIES 片判不了（一个码 2 根装 135 条 inode 内部条目，合法树至多 135 片）
const D_LEAF_IDENTITY_WORDS: u32 = 7u;

const D_MAX_LEAF_IDENTITIES: u32 = 256u;

const D_LEAF_IDENTITIES_OF_THE_VERSION: u32 = 403888u;

const D_LEAF_IDENTITIES_OF_THE_PARENT_VERSION: u32 = 405680u;

// ⑧ 判到位的两张表（走读内核记、扫描内核接着记；判决 m3-crash-enumeration-merge-r2 第二节 L1-4）：评估过、射程里有一部分判不了，
//    各 POOL_CHECKER_JUDGED_WORDS 个字。交接段 32 个字都占满了，接在草稿区末尾（H_BASE 起数的偏移）
const H_EVALUATED: u32 = 407472u - 295824u;
const H_PART_NOT_JUDGED: u32 = H_EVALUATED + POOL_CHECKER_JUDGED_WORDS;

// ⑨ 两两比与组首判那几处换成「排一次序、按序走一遍」用的两张表（一个状态一条线程，那几处随条数平方长，一次派活撞驱动的上下文切换时限）。
//    C_ROW_ORDER：一张表按 order_rows_by_their_first_four_words 排出的行号次序，走读内核逐盘 journal 那几条的组（至多 MAX_RECORDS 条）、
//    I-3.5 一版引用的落点（至多 C_MAX_IN_VERSION 个）、I-5.1 的主走读引用集合（至多 C_MAX_REFERENCES 条）、扫描内核 I-1.8 的扫描项（至多 C_MAX_SCANNED 个）
//    与墓碑两两比的那几把 key（至多 DELETION_MAX_TOMBSTONES_COMPARED_PAIRWISE 把）各借一次，取其中最大的 C_MAX_REFERENCES 个字。I-5.4 与 I-3.16
//    排分配记录的次序原来也借它，分配记录那张表抬到 8192 条时改借 ⑮ 的 C_ALLOCATION_RECORD_ROW_ORDER。C_JOURNAL_GROUP_KEYS：走读内核逐盘 journal
//    那几条的组键（实例、键 lo/hi、0），每条记录一行；扫描内核不读它，I-1.8 的检查字借这一段（I-3.9 已释放记录的桶表与 I-3.10 的去重桶表原来也借它，
//    分配记录那张表抬到 8192 条时改借 ⑮ 的 C_ALLOCATION_RECORD_BUCKETS）。MAX_RECORDS 抬到 4096 时 C_JOURNAL_GROUP_KEYS 跟着长、
//    挪到草稿区末尾（⑯）：411576 起原来那 8192 个字空着，C_ROW_ORDER 不挪，借它的几处按 C_ROW_ORDER_END 判装不装得下
const C_ROW_ORDER: u32 = 407480u;

// C_ROW_ORDER 那一段的末端：借它的几处里最大的 C_MAX_REFERENCES 个字
const C_ROW_ORDER_END: u32 = C_ROW_ORDER + C_MAX_REFERENCES;

const_assert MAX_RECORDS <= C_ROW_ORDER_END - C_ROW_ORDER;

const_assert C_ROW_ORDER_END <= SCRATCH_END_BEFORE_THE_TABLES_GREW;

const C_JOURNAL_GROUP_KEY_WORDS: u32 = 4u;

const C_JOURNAL_GROUP_KEYS: u32 = C_REPLAY_RECORDS + C_REPLAY_RECORD_WORDS * MAX_RECORDS;

// C_JOURNAL_GROUP_KEYS 那一段的末端（每条记录一行，至多 MAX_RECORDS 行）：扫描内核借这一段的几处按它判装不装得下
const C_JOURNAL_GROUP_KEYS_END: u32 = C_JOURNAL_GROUP_KEYS + C_JOURNAL_GROUP_KEY_WORDS * MAX_RECORDS;

// ⑪ 走读内核主走读收的 inode → 对象出生代（I-9.10 拿数据单元的对象出生代与它比），inode lo/hi、出生代 lo/hi 一条四个字。
//    条数是走到的 inode 记录里不同的号：原来 512 条，unlink 借、借完再吸收的两个场景（各七百来个 inode）在站点 13 判不了；1024 条时
//    一次建的 inode 多到点名超过一条 journal 记录的那个场景（各版合起来 12582 个号，CPU 一侧数的）同样判不了。取 inode 树一版的记录那张表
//    （D_MAX_INODE_RECORDS）的条数。419768 起原来那 4096 个字（1024 条时的这张表）空着
const C_MAX_INODE_BIRTHS: u32 = 16384u;

const_assert C_MAX_INODE_BIRTHS == D_MAX_INODE_RECORDS;

// 草稿区抬上限之前到这里为止；共用那一半挪过来的两张表（crash_judge_common.wgsl 的 R_ALLOCATION、R_ROWS）从这里接，抬上限的几张接在它们后面
const SCRATCH_END_BEFORE_THE_TABLES_GREW: u32 = 423864u;

const_assert R_ALLOCATION == SCRATCH_END_BEFORE_THE_TABLES_GREW;

// ⑫ 抬了上限、挪到末尾的几张表（各自的条数与为什么抬见定义它们上限的那一处），按这个次序首尾相接：
//    扫描内核一条根的分配记录、最新根的已释放记录与「哪条候选根引用它」、最新根实例表的行与它的散列索引（桶表、同实例下一行）、
//    扫描内核一条根自己实例表的行、主走读的 inode → 对象出生代与它的桶表、这一版 inode 记录与它的桶表、I-9.29 跨版本那张表与它的桶表、
//    释放代高于 F 的已释放记录、I-3.10 的去重表。几张桶表的写法见 ⑬
const C_ALLOC_RECORDS: u32 = END_OF_THE_RECOVERY_TABLES_MOVED_PAST_THE_CHECKER_TABLES;

const C_RELEASED: u32 = C_ALLOC_RECORDS + C_ALLOC_RECORD_WORDS * C_MAX_ALLOC_RECORDS;   // 盘、槽 lo/hi、释放代 lo/hi

const C_REFERENCED_BY: u32 = C_RELEASED + 5u * C_MAX_RELEASED;   // 每条已释放记录两个字：位 c ↔ 第 c 条候选根引用它

// 最新根实例表的行（walk.rs 的 Walk::instance_table_rows）：一张表 66 片、23997 行的那个场景之外，取号之后崩溃 24575 次再挂载的那条流 24576 行；
// 比扫描内核与恢复内核那两张（C_R_MAX_ROWS、MAX_ROWS）宽，那两张满了而这一张装得下（扫描内核站点 32、恢复内核记判不了的那一形照旧走得到）
const C_MAX_INSTANCE_ROWS: u32 = 32768u;

const C_INSTANCE_ROWS: u32 = C_REFERENCED_BY + 2u * C_MAX_RELEASED;

const W_INSTANCE_ROW_BUCKET_COUNT: u32 = 2u * C_MAX_INSTANCE_ROWS;

const W_INSTANCE_ROW_BUCKETS: u32 = C_INSTANCE_ROWS + C_ROW_WORDS * C_MAX_INSTANCE_ROWS;

// 第 k 行之后、同一个实例的下一行的下标 + 1（0 是没有下一行）：记第 k 行时写 0，同实例再记一行时改它前一行的这个字
const W_INSTANCE_ROW_NEXT: u32 = W_INSTANCE_ROW_BUCKETS + W_INSTANCE_ROW_BUCKET_COUNT;

const C_R_ROWS: u32 = W_INSTANCE_ROW_NEXT + C_MAX_INSTANCE_ROWS;   // 实例、txg lo/hi

const C_INODE_BIRTHS: u32 = C_R_ROWS + 3u * C_R_MAX_ROWS;

const W_INODE_BIRTH_BUCKET_COUNT: u32 = 2u * C_MAX_INODE_BIRTHS;

const W_INODE_BIRTH_BUCKETS: u32 = C_INODE_BIRTHS + 4u * C_MAX_INODE_BIRTHS;

const D_INODE_RECORDS: u32 = W_INODE_BIRTH_BUCKETS + W_INODE_BIRTH_BUCKET_COUNT;

const D_INODE_RECORD_BUCKET_COUNT: u32 = 2u * D_MAX_INODE_RECORDS;

const D_INODE_RECORD_BUCKETS: u32 = D_INODE_RECORDS + D_INODE_RECORD_WORDS * D_MAX_INODE_RECORDS;

const D_OBJECT_BIRTHS: u32 = D_INODE_RECORD_BUCKETS + D_INODE_RECORD_BUCKET_COUNT;

const D_OBJECT_BIRTH_BUCKET_COUNT: u32 = 2u * D_MAX_OBJECT_BIRTHS;

const D_OBJECT_BIRTH_BUCKETS: u32 = D_OBJECT_BIRTHS + D_OBJECT_BIRTH_WORDS * D_MAX_OBJECT_BIRTHS;

// 一条四个字：盘、槽 lo/hi、跨度
const C_RELEASED_ABOVE_WORDS: u32 = 4u;

const C_RELEASED_ABOVE: u32 = D_OBJECT_BIRTH_BUCKETS + D_OBJECT_BIRTH_BUCKET_COUNT;

const C_EXAMINED: u32 = C_RELEASED_ABOVE + C_RELEASED_ABOVE_WORDS * C_MAX_RELEASED_ABOVE;   // 盘、槽 lo/hi、代 lo/hi

// ⑭ 里程碑四接在 ⑫ 那几张表与扫描内核挪到末尾的两张（C_RELEASED_ABOVE、C_EXAMINED）后面的几张（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3、`research/prompts/m4-snapshot-r1-main-verification.md`
//    第三节 S5）：
//    主走读的「中央映射 key → 走读认它作哪个单元」（walk.rs 的 unit_of_each_central_mapping_key）：搬迁只改映射一条条目、单元逐字节不变，
//    搬前那一版在源落点、搬后那一版在目的落点，两个落点按 key 折成同一个单元（I-3.5 的折叠表）。一行 11 字：类标签与尾段高 16 位、出生树 lo/hi、
//    出生 txg lo/hi、实例代号、尾段低 32 位，第一次记下它的那条指针认的第 0 条位置（盘、槽 lo/hi）；条数与 I-3.5 落点表同宽，桶表两倍。
const W_MAPPING_KEY_UNIT_WORDS: u32 = 11u;

const W_MAX_MAPPING_KEY_UNITS: u32 = C_APPEARANCE_MAX_PLACEMENTS;

const W_MAPPING_KEY_UNITS: u32 = C_EXAMINED + 5u * C_MAX_EXAMINED;

const W_MAPPING_KEY_UNIT_BUCKET_COUNT: u32 = 2u * W_MAX_MAPPING_KEY_UNITS;

const W_MAPPING_KEY_UNIT_BUCKETS: u32 = W_MAPPING_KEY_UNITS + W_MAPPING_KEY_UNIT_WORDS * W_MAX_MAPPING_KEY_UNITS;

//    一版的中央映射（read_the_central_mapping_without_judging，walk.rs 的 CentralMappingLocationsOfAVersion）：走读每走一版、引用与快照那一截每读一条根的
//    引用集合都先读它，豁免三类之外的指针按它认位置。每条叶条目一行 16 字，读的时候把 key 与两条位置条目解好（找、认位置只比字、不再读条目的字节）：
//    key 7 字（同下面那几张），同一个 key 在叶里出现过两次的记号，位置条目 0 的盘、槽 lo/hi、整单元校验和，位置条目 1 的四样；
//    条数上限 C_MAX_VERSION_MAPPING_ROWS（满了判不了，站点 90），桶表两倍。
const C_VERSION_MAPPING_ROW_WORDS: u32 = 16u;

const C_MAX_VERSION_MAPPING_ROWS: u32 = 1024u;

const C_VERSION_MAPPING_ROWS: u32 = W_MAPPING_KEY_UNIT_BUCKETS + W_MAPPING_KEY_UNIT_BUCKET_COUNT;

const C_VERSION_MAPPING_BUCKET_COUNT: u32 = 2u * C_MAX_VERSION_MAPPING_ROWS;

const C_VERSION_MAPPING_BUCKETS: u32 = C_VERSION_MAPPING_ROWS + C_VERSION_MAPPING_ROW_WORDS * C_MAX_VERSION_MAPPING_ROWS;

//    引用与快照那一截（crash_judge_checker_scan.wgsl 的 judge_references_and_snapshots）判 I-3.5 ③ ④ 与逆向、I-3.6、I-3.7 用的四张：
//    一版里活头与每个活快照引用的单元（中央映射 key 7 字：类标签与尾段高 16 位、出生树 lo/hi、出生 txg lo/hi、实例代号、尾段低 32 位，
//    再一字是谁引用它的位图：位 0..15 是按（S.txg, 树 ID）排的第几个活快照，位 31 是活头；再两字是 deadlist 里钉它的条目有几条、最后一条的下标）
//    与它的桶表；一版 deadlist 的叶条目（字节基、偏移）；版本清单里每一版读到的树表（读得出没有、txg lo/hi、树表节点的字节基、条目起点、条目宽、
//    条目数、树表指针第 0 条位置条目的盘、槽 lo/hi、校验和、0）；读树的显式栈（字节基、指针偏移、层级、0）。
const S_KEYED_UNIT_WORDS: u32 = 10u;

const S_MAX_KEYED_UNITS: u32 = 8192u;

const S_KEYED_UNITS: u32 = C_VERSION_MAPPING_BUCKETS + C_VERSION_MAPPING_BUCKET_COUNT;

const S_KEYED_UNIT_BUCKET_COUNT: u32 = 2u * S_MAX_KEYED_UNITS;

const S_KEYED_UNIT_BUCKETS: u32 = S_KEYED_UNITS + S_KEYED_UNIT_WORDS * S_MAX_KEYED_UNITS;

const S_MAX_DEADLIST_ENTRIES: u32 = 4096u;

const S_DEADLIST_ENTRIES: u32 = S_KEYED_UNIT_BUCKETS + S_KEYED_UNIT_BUCKET_COUNT;

const S_VERSION_WORDS: u32 = 12u;

// 版本清单至多 C_MAX_CANDIDATES 条候选根加 R_DATA_WORDS 版由记录施加出来的（walk_and_hand_off 收它们的两处上限；走读内核把后者记在 R_DATA 那一段，每版 1 字）
const S_MAX_VERSIONS: u32 = C_MAX_CANDIDATES + R_DATA_WORDS;

const S_VERSIONS: u32 = S_DEADLIST_ENTRIES + 2u * S_MAX_DEADLIST_ENTRIES;

const S_PENDING_WORDS: u32 = 4u;

const S_MAX_PENDING: u32 = 1024u;

const S_PENDING: u32 = S_VERSIONS + S_VERSION_WORDS * S_MAX_VERSIONS;

//    走读内核 w_merge_below（walk.rs 的 merge_placements_below_a_unit_walked_by_an_earlier_version：别的版本先走过的单元下面的落点并进这一版）的
//    显式栈与展开过的单元，每条三个字（盘、槽 lo/hi）。CPU 那边两样都是不设上限的集合；这里两张的条数都由挂父单元那张表（C_MAX_BELOW 条，
//    满了记站点 7）定：展开过的单元是起点加孩子落点里不同的那几个，孩子落点都从那张表的行里取，至多 C_MAX_BELOW 个；每个单元只展开一次，
//    挂在它下面的那几行只在展开它时压栈，每行至多压一次，栈里至多是起点加 C_MAX_BELOW 条。两张都取 C_MAX_BELOW + 1 条，满了的两处（站点 5、6）
//    合法镜像与坏镜像上都走不到，留着守写入边界，照旧记判不了、不截断。原来各 1024 条，一个 900 个数据单元的长文件被后一版共用时
//    （两盘各一份，展开过的落点多于 1024 个；tests/crash_judge_gpu_reads_long_files_and_lineages_block_by_block.rs 长文件旁边那条流）在站点 5 判不了。
//    两样找法按 ⑬ 的写法建散列索引（见 C_MERGE_EXPANDED 之后那一段）：抬到 4097 条之后展开过的单元从表头比到表尾去重、每展开一个单元把挂父单元那张表
//    从头比到尾，一次合并最坏 4097 × 4096 次比较，那条流一次派活撞驱动的上下文切换时限（内核日志 Xid 109）。
const C_MERGE_UNIT_WORDS: u32 = 3u;

const C_MAX_MERGE_PENDING: u32 = C_MAX_BELOW + 1u;

const C_MERGE_PENDING: u32 = S_PENDING + S_PENDING_WORDS * S_MAX_PENDING;

const C_MAX_MERGE_EXPANDED: u32 = C_MAX_BELOW + 1u;

const C_MERGE_EXPANDED: u32 = C_MERGE_PENDING + C_MERGE_UNIT_WORDS * C_MAX_MERGE_PENDING;

//    w_merge_below 的两样找法的散列索引，写法同 ⑬（KeyedTable：过 KEYED_TABLE_ROWS_FOUND_BY_SCANNING_AT_MOST 条才清桶表、把已有各行记进去，
//    之前照旧从表头比到表尾）：
//    展开过的单元（键是盘、槽 lo/hi）：每次合并是一张新表，「建过没有」是 w_merge_below 自己的局部变量，桶表每次合并过界那一刻整张清。
//    桶数取不少于表的两倍的 2 的幂（4097 条的两倍是 8194，取 16384）：表满时也至少一半是空桶。
const W_MERGE_EXPANDED_BUCKET_COUNT: u32 = 16384u;

const_assert W_MERGE_EXPANDED_BUCKET_COUNT >= 2u * C_MAX_MERGE_EXPANDED;

const_assert 0u == (W_MERGE_EXPANDED_BUCKET_COUNT & (W_MERGE_EXPANDED_BUCKET_COUNT - 1u));

const W_MERGE_EXPANDED_BUCKETS: u32 = C_MERGE_EXPANDED + C_MERGE_UNIT_WORDS * C_MAX_MERGE_EXPANDED;

//    挂父单元那张表（C_BELOW0 / C_BELOW1，键是父盘、父槽 lo/hi，同一个父单元有几行）：两套上下文各一份，每份依次是桶表（只记同一个父单元的第一行）、
//    同一个父单元的下一行的下标 + 1（0 是没有下一行，只指向更靠后的行）、同一个父单元此刻最后一行的下标（只在第一行那一格有意义）。
//    记一行时找父单元的第一行、接到最后一行后面，展开一个单元时从第一行沿着下一行走：走到的行与次序和从头比到尾挑父单元相同的那几行一样。
//    「建过没有」记在 w_below_rows_are_indexed，w_start 清；桶表在那一份过界那一刻整张清
const W_BELOW_BUCKET_COUNT: u32 = 2u * C_MAX_BELOW;

const_assert 0u == (W_BELOW_BUCKET_COUNT & (W_BELOW_BUCKET_COUNT - 1u));

const W_BELOW_NEXT_OFFSET: u32 = W_BELOW_BUCKET_COUNT;

const W_BELOW_LAST_OFFSET: u32 = W_BELOW_NEXT_OFFSET + C_MAX_BELOW;

const W_BELOW_INDEX_WORDS: u32 = W_BELOW_LAST_OFFSET + C_MAX_BELOW;

const W_BELOW_INDEX0: u32 = W_MERGE_EXPANDED_BUCKETS + W_MERGE_EXPANDED_BUCKET_COUNT;

const W_BELOW_INDEX1: u32 = W_BELOW_INDEX0 + W_BELOW_INDEX_WORDS;

//    I-3.1 钉带出来的那一截要排除的槽（walk.rs 的 slots_referenced_by_abandoned_roots，C773）：走读内核在被抛弃根那一阶段每走完一条，
//    把它引用的槽里候选版本没走到的去重记进这一张（盘、槽 lo/hi；满了判不了，站点 117），扫描内核的 c_deferred_below_the_floor 按它排除。
//    头一个字是条数，走读内核开工时清零
const W_MAX_ABANDONED_REFERENCED_SLOTS: u32 = 4096u;

const W_ABANDONED_REFERENCED_SLOT_COUNT: u32 = W_BELOW_INDEX1 + W_BELOW_INDEX_WORDS;

const W_ABANDONED_REFERENCED_SLOTS: u32 = W_ABANDONED_REFERENCED_SLOT_COUNT + 1u;

// ⑮ 分配记录那张表（C_MAX_ALLOC_RECORDS）从 4096 抬到 8192 条时，原来借的段装不下的两样另开的两段，接在 I-3.5 每一版引用的落点
//    （C_APPEARANCE_VERSION_PLACEMENTS）之后。原来那两段（C_ROW_ORDER 的 4096 个字、C_JOURNAL_GROUP_KEYS 的 8192 个字）照旧给别的借用者，偏移不挪。
//    C_ALLOCATION_RECORD_ROW_ORDER：一棵分配记录树按 order_rows_by_their_first_four_words 排出的行号次序，扫描内核 I-5.4（judge_allocation_record_ranges）
//    与 I-3.16（judge_fragmentation_and_empty_segment_statistics 按记录口径数全空段）各借一次，至多 C_MAX_ALLOC_RECORDS 个字。
const C_ALLOCATION_RECORD_ROW_ORDER: u32 = C_APPEARANCE_VERSION_PLACEMENTS + C_APPEARANCE_MAX_VERSION_PLACEMENTS;

//    C_ALLOCATION_RECORD_BUCKETS：I-3.9 已释放记录的桶表（扫描内核的 C_RELEASED_BUCKETS，2 × C_MAX_RELEASED 个桶）与 I-3.10 去重表的桶表
//    （C_EXAMINED_BUCKETS，2 × C_MAX_EXAMINED 个桶）先后借同一段，取两张里大的那个桶数。两判不同时用：I-3.10 在扫描内核主入口（main 跑 scan_stage，
//    judge_allocation_generations_against_unit_births），I-3.9 在引用与快照那一截的入口（references_and_snapshots_main 跑 references_and_snapshots_stage，
//    judge_release_generation_and_tree_table_birth），派活按恢复、走读、扫描、引用与快照、扫描重建五趟依次跑（crash_judge_dispatch.rs 里
//    「五趟按次序」那一段）；两处开判时各自把整张桶表清零再建，读已释放记录桶表的 r_note 只在 I-3.9 那一圈 r_noting 为真时读
const C_ALLOCATION_RECORD_BUCKETS: u32 = C_ALLOCATION_RECORD_ROW_ORDER + C_MAX_ALLOC_RECORDS;

const C_ALLOCATION_RECORD_BUCKET_WORDS: u32 = 2u * C_MAX_ALLOC_RECORDS;

// ⑯ journal 记录表（MAX_RECORDS）从 2048 抬到 4096 条时跟着长的三张，接在 ⑮ 之后、按这个次序首尾相接（原处都空着、其后的偏移不挪）：
//    恢复内核与走读内核收环里记录的那张（crash_judge_common.wgsl 的 R_RECORDS，那份文件只有恢复内核与两个池级 checker 内核共用的那一半，
//    起点写成字面数，这里核它接在 ⑮ 的末端）、走读内核 I-4.1 重放读的那一份环（I-3.5 那四张表前面定义的 C_REPLAY_RECORDS）、
//    走读内核逐盘 journal 那几条的组键（⑨ 的 C_JOURNAL_GROUP_KEYS，扫描内核 I-1.8 的检查字借它）
const_assert R_RECORDS == C_ALLOCATION_RECORD_BUCKETS + C_ALLOCATION_RECORD_BUCKET_WORDS;

// 每个状态的草稿区到这里为止：crash_judge_gpu.rs 的 SCRATCH_WORDS 与它相等（那一份的单测核）
const SCRATCH_END: u32 = 2009599u;

const_assert SCRATCH_END == C_JOURNAL_GROUP_KEYS_END;

// 被抛弃根引用着、候选版本没走到的那几槽里有没有 (盘, 槽)
fn c_slot_referenced_by_an_abandoned_root(device: u32, slot: vec2<u32>) -> bool {
    let count = sw(W_ABANDONED_REFERENCED_SLOT_COUNT);
    for (var i = 0u; i < count; i = i + 1u) {
        let row = W_ABANDONED_REFERENCED_SLOTS + i * 3u;
        if (sw(row) == device && sw(row + 1u) == slot.x && sw(row + 2u) == slot.y) {
            return true;
        }
    }
    return false;
}

// 扫描内核 I-5.4 与 I-3.16 把一棵分配记录树的记录按行号次序排进 C_ALLOCATION_RECORD_ROW_ORDER（那一段到 C_ALLOCATION_RECORD_BUCKETS 为止）：装得下
const_assert C_MAX_ALLOC_RECORDS <= C_ALLOCATION_RECORD_BUCKETS - C_ALLOCATION_RECORD_ROW_ORDER;

// ⑩ 随条数线性找的几张集合加散列索引（一个状态一条线程，那几处每记一条、每找一条都从表头比到表尾，条数平方长，一次派活撞驱动的上下文切换时限）。
//    表本身照旧按记进的次序一行一行追加（读前若干条的、按行号排序的那几处都不变），旁边另放一张桶表：每个桶记一行的下标 + 1，0 是空桶，
//    按 (盘, 槽) 散列、撞了往后挪一个（线性探测，不删）。同 (盘, 槽) 的几行都落在从它的首桶起、到第一个空桶为止的那一段里。
//    桶数取表的两倍（都是 2 的幂）：表满时也至少一半是空桶，往后挪总停得下；找的循环按桶数封顶，走不满（至少有一个空桶）
fn slot_key_first_bucket(device: u32, slot: vec2<u32>, bucket_count: u32) -> u32 {
    var hash = device * 0x9E3779B1u;
    hash = (hash ^ slot.x) * 0x85EBCA77u;
    hash = (hash ^ slot.y) * 0xC2B2AE3Du;
    return (hash ^ (hash >> 15u)) & (bucket_count - 1u);
}

struct HashedRow {
    // 找到的那一行的下标；没有是 NONE
    row: u32,
    // 没找到时从首桶往后第一个空桶（要把这一行记进表就写这个桶）；找到了是 NONE
    empty_bucket: u32,
};

// rows 起每行 stride 个字、前三个字是 (盘, 槽 lo, 槽 hi) 的一张表里，与 (device, slot) 相同、compares_the_fourth_word 时第四个字也等于
// fourth_word 的那一行（表里这样的行至多一行：几处都是先找、找不到才记）
fn hashed_row_of(buckets: u32, bucket_count: u32, rows: u32, stride: u32, device: u32, slot: vec2<u32>, fourth_word: u32,
    compares_the_fourth_word: bool) -> HashedRow {
    var found: HashedRow;
    found.row = NONE;
    found.empty_bucket = NONE;
    var bucket = slot_key_first_bucket(device, slot, bucket_count);
    for (var probe = 0u; probe < bucket_count; probe = probe + 1u) {
        let row_plus_one = sw(buckets + bucket);
        if (row_plus_one == 0u) {
            found.empty_bucket = bucket;
            return found;
        }
        let row = rows + (row_plus_one - 1u) * stride;
        if (sw(row) == device && sw(row + 1u) == slot.x && sw(row + 2u) == slot.y
            && (!compares_the_fourth_word || sw(row + 3u) == fourth_word)) {
            found.row = row_plus_one - 1u;
            return found;
        }
        bucket = (bucket + 1u) & (bucket_count - 1u);
    }
    return found;
}

// 走读内核主走读（上下文 0）的三张桶表：引用集合 C_REFERENCES0（比到跨度）、I-3.5 落点表、
// 走过的单元 C_VISITED0。w_start(0) 清空。借扫描内核从 C_SCAN_TABLES_THE_WALK_BUCKETS_BORROW 起的那几张表的地方（到 W_BUCKETS_END 为止，不过 C_SCANNED）：
// 扫描内核在走读内核之后跑，那几张表都按自己的条数从头写、写过才读，不读走读内核留在那里的字；走读内核在那一段里只写这三张桶表
// （它写的 C_CANDIDATES 在那一段之前）。实例表行与 inode → 对象出生代那两张的桶表不借这一段：前一张扫描内核还要读（w_published），
// 两张都放在草稿区末尾（⑫），按 ⑬ 的写法清
const W_REFERENCE_BUCKET_COUNT: u32 = 2u * C_MAX_REFERENCES;

const W_REFERENCE_BUCKETS: u32 = C_SCAN_TABLES_THE_WALK_BUCKETS_BORROW;

const W_PLACEMENT_BUCKET_COUNT: u32 = 2u * C_APPEARANCE_MAX_PLACEMENTS;

const W_PLACEMENT_BUCKETS: u32 = W_REFERENCE_BUCKETS + W_REFERENCE_BUCKET_COUNT;

const W_VISITED_BUCKET_COUNT: u32 = 2u * C_MAX_VISITED;

const W_VISITED_BUCKETS: u32 = W_PLACEMENT_BUCKETS + W_PLACEMENT_BUCKET_COUNT;

const W_BUCKETS_END: u32 = W_VISITED_BUCKETS + W_VISITED_BUCKET_COUNT;

// 三张桶表不越进扫描内核 I-1.8 存扫描项的那一段（解析内核时判，不进展开量）
const_assert W_BUCKETS_END <= C_SCANNED;

// ⑬ 抬到上万条的四张表的散列索引：最新根实例表的行（C_INSTANCE_ROWS，键是实例，同实例可以有几行）、主走读的 inode → 对象出生代（C_INODE_BIRTHS，
//    键是 inode 号）、这一版 inode 记录（D_INODE_RECORDS，键是 inode 号）、I-9.29 跨版本那张表（D_OBJECT_BIRTHS，键是 (inode 树, 号)）。
//    那几处原来每记一行、每找一行都从表头比到表尾，几万行时条数平方长（一个状态一条线程，撞驱动的上下文切换时限）。表照旧按记进的次序一行一行追加；
//    条数不过 KEYED_TABLE_ROWS_FOUND_BY_SCANNING_AT_MOST 时照旧从表头比到表尾找（一个状态多半只有几行，不为它清一张上万个桶的桶表），
//    记进第 KEYED_TABLE_ROWS_FOUND_BY_SCANNING_AT_MOST + 1 行那一刻把桶表整张清掉、把已有各行按键记进去，之后按桶表找。桶的写法同 ⑩：
//    一个桶记一行的下标 + 1，0 是空桶，撞了往后挪一个（线性探测，不删），桶数取表的两倍（都是 2 的幂，表满时也至少一半是空桶）。
//    键相同的几行只有第一行进桶表，找到的总是按记进次序的第一行，与从表头比到表尾找到的是同一行。草稿区里的桶表可能留着上一个状态的字：
//    只在这一个状态清过之后才读它（「建过没有」记在每张表各自的私有变量里，扫描内核按交接来的条数照同一个界认实例表行那一张）
const KEYED_TABLE_ROWS_FOUND_BY_SCANNING_AT_MOST: u32 = 64u;

// 一张带散列索引的表：rows 起每行 stride 个字、前 key_words 个字是键（至多四个），桶表在 buckets 起、bucket_count 个桶
struct KeyedTable {
    rows: u32,
    stride: u32,
    key_words: u32,
    buckets: u32,
    bucket_count: u32,
};

fn keyed_first_bucket(key: vec4<u32>, bucket_count: u32) -> u32 {
    var hash = key.x * 0x9E3779B1u;
    hash = (hash ^ key.y) * 0x85EBCA77u;
    hash = (hash ^ key.z) * 0xC2B2AE3Du;
    hash = (hash ^ key.w) * 0x27D4EB2Fu;
    return (hash ^ (hash >> 15u)) & (bucket_count - 1u);
}

// 起点在 row 的那一行的键是不是 key（比前 key_words 个字）
fn keyed_row_carries_the_key(row: u32, key: vec4<u32>, key_words: u32) -> bool {
    return sw(row) == key.x && (key_words < 2u || sw(row + 1u) == key.y)
        && (key_words < 3u || sw(row + 2u) == key.z) && (key_words < 4u || sw(row + 3u) == key.w);
}

// 表前 count 行里键是 key 的第一行（按记进的次序）的下标，没有是 NONE；indexed：这一个状态里桶表建过没有。
// 桶表没建过时第 step 步比第 step 行（至多 count 步）；建过时第 step 步比从首桶往后第 step 个桶记的那一行，撞到空桶就是没有（至多 bucket_count 步，
// 表满时也至少一半是空桶，走不满）。两种找法合成一个循环、比键只一个调用点（展开预算，同 keyed_table_after_a_row_was_appended）
fn keyed_first_row_of(table: KeyedTable, count: u32, indexed: bool, key: vec4<u32>) -> u32 {
    var bucket = keyed_first_bucket(key, table.bucket_count);
    let steps = select(count, table.bucket_count, indexed);
    for (var step = 0u; step < steps; step = step + 1u) {
        var row_index = step;
        if (indexed) {
            let row_plus_one = sw(table.buckets + bucket);
            if (row_plus_one == 0u) {
                return NONE;
            }
            row_index = row_plus_one - 1u;
            bucket = (bucket + 1u) & (table.bucket_count - 1u);
        }
        if (keyed_row_carries_the_key(table.rows + row_index * table.stride, key, table.key_words)) {
            return row_index;
        }
    }
    return NONE;
}

// 第 row_index 行记进桶表：从它的键的首桶起往后找，先撞到键相同的一行（记进得更早）就不记，撞到空桶就记在那里
fn keyed_put_the_row_into_the_buckets(table: KeyedTable, row_index: u32) {
    let row = table.rows + row_index * table.stride;
    let key = vec4<u32>(sw(row), select(0u, sw(row + 1u), table.key_words >= 2u), select(0u, sw(row + 2u), table.key_words >= 3u),
        select(0u, sw(row + 3u), table.key_words >= 4u));
    var bucket = keyed_first_bucket(key, table.bucket_count);
    for (var probe = 0u; probe < table.bucket_count; probe = probe + 1u) {
        let row_plus_one = sw(table.buckets + bucket);
        if (row_plus_one == 0u) {
            sws(table.buckets + bucket, row_index + 1u);
            return;
        }
        if (keyed_row_carries_the_key(table.rows + (row_plus_one - 1u) * table.stride, key, table.key_words)) {
            return;
        }
        bucket = (bucket + 1u) & (table.bucket_count - 1u);
    }
}

// 第 row_index 行刚记进表（它是最后一行，count 是记进之后的行数）：桶表建过就把这一行记进去；没建过而条数刚越过
// KEYED_TABLE_ROWS_FOUND_BY_SCANNING_AT_MOST，清桶表、按记进的次序把每一行记进去。交回记过之后桶表建过没有。
// 两种情形合成一个循环、一个调用点（GPU 程序把每个调用点展开成一份拷贝，扫描内核离展开预算只剩几千个表达式）
fn keyed_table_after_a_row_was_appended(table: KeyedTable, count: u32, indexed: bool, row_index: u32) -> bool {
    if (!indexed && count <= KEYED_TABLE_ROWS_FOUND_BY_SCANNING_AT_MOST) {
        return false;
    }
    var first_row_to_put = row_index;
    if (!indexed) {
        for (var bucket = 0u; bucket < table.bucket_count; bucket = bucket + 1u) {
            sws(table.buckets + bucket, 0u);
        }
        first_row_to_put = 0u;
    }
    for (var row_to_put = first_row_to_put; row_to_put < count; row_to_put = row_to_put + 1u) {
        keyed_put_the_row_into_the_buckets(table, row_to_put);
    }
    return true;
}

// 最新根实例表的行那一张（键是实例）
fn w_instance_row_table() -> KeyedTable {
    return KeyedTable(C_INSTANCE_ROWS, C_ROW_WORDS, 1u, W_INSTANCE_ROW_BUCKETS, W_INSTANCE_ROW_BUCKET_COUNT);
}

var<private> w_instance_rows_are_indexed: bool;

// 实例 instance 在最新根实例表里的第一行（按记进的次序）的下标，没有是 NONE
fn w_first_instance_row_of(instance: u32) -> u32 {
    return keyed_first_row_of(w_instance_row_table(), w_instance_row_count, w_instance_rows_are_indexed, vec4<u32>(instance, 0u, 0u, 0u));
}

// 第 row_index 行之后同一个实例的下一行的下标，没有是 NONE（W_INSTANCE_ROW_NEXT 只指向更靠后的行：沿着走总走得到头）
fn w_next_instance_row_after(row_index: u32) -> u32 {
    let next_plus_one = sw(W_INSTANCE_ROW_NEXT + row_index);
    if (next_plus_one == 0u) {
        return NONE;
    }
    return next_plus_one - 1u;
}

// 最新根实例表记一行 (instance, txg, W)：同实例已有行就接在那几行的最后一行后面（W_INSTANCE_ROW_NEXT），交回记之前这个实例有没有行；
// 表满记站点 8 判不了、不记
fn w_append_an_instance_row(instance: u32, row_txg: vec2<u32>, row_high_water: vec2<u32>) -> bool {
    let first = w_first_instance_row_of(instance);
    if (w_instance_row_count >= C_MAX_INSTANCE_ROWS) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (8u << 8u);
        return first != NONE;
    }
    let row_index = w_instance_row_count;
    if (first != NONE) {
        var last = first;
        for (var next = w_next_instance_row_after(first); next != NONE; next = w_next_instance_row_after(next)) {
            last = next;
        }
        sws(W_INSTANCE_ROW_NEXT + last, row_index + 1u);
    }
    let row = C_INSTANCE_ROWS + row_index * C_ROW_WORDS;
    sws(row, instance);
    sws(row + 1u, row_txg.x);
    sws(row + 2u, row_txg.y);
    sws(row + 3u, row_high_water.x);
    sws(row + 4u, row_high_water.y);
    sws(W_INSTANCE_ROW_NEXT + row_index, 0u);
    w_instance_row_count = row_index + 1u;
    w_instance_rows_are_indexed = keyed_table_after_a_row_was_appended(w_instance_row_table(), w_instance_row_count,
        w_instance_rows_are_indexed, row_index);
    return first != NONE;
}

// 主走读 inode → 对象出生代那一张（键是 inode 号）
fn w_inode_birth_table() -> KeyedTable {
    return KeyedTable(C_INODE_BIRTHS, 4u, 2u, W_INODE_BIRTH_BUCKETS, W_INODE_BIRTH_BUCKET_COUNT);
}

var<private> w_inode_births_are_indexed: bool;

// inode 号 inode 在那一张里的行的下标，没有是 NONE
fn w_inode_birth_row_of(inode: vec2<u32>) -> u32 {
    return keyed_first_row_of(w_inode_birth_table(), w_inode_birth_count, w_inode_births_are_indexed, vec4<u32>(inode.x, inode.y, 0u, 0u));
}

// 草稿区里 rows 起每行 stride 个字、前四个字当键（键 0、键 1 lo/hi、键 2）的一张表：第 a 行排在第 b 行前面——按 (键 0, 键 1, 键 2, 行号) 比，
// 行号放最后，次序是全序、同键的按行号升序（「同组里按表里的次序走」那几处靠这一点）
fn ordered_row_precedes(rows: u32, stride: u32, a: u32, b: u32) -> bool {
    let row_a = rows + a * stride;
    let row_b = rows + b * stride;
    if (sw(row_a) != sw(row_b)) {
        return sw(row_a) < sw(row_b);
    }
    let middle_a = vec2<u32>(sw(row_a + 1u), sw(row_a + 2u));
    let middle_b = vec2<u32>(sw(row_b + 1u), sw(row_b + 2u));
    if (!eq64(middle_a, middle_b)) {
        return lt64(middle_a, middle_b);
    }
    if (sw(row_a + 3u) != sw(row_b + 3u)) {
        return sw(row_a + 3u) < sw(row_b + 3u);
    }
    return a < b;
}

// 那张表前 count 行按 ordered_row_precedes 的次序，行号写进 order 起的 count 个字。堆排序：建堆 ⌊count / 2⌋ 步、出堆 count − 1 步，
// 共用一处下沉（GPU 程序把每个调用点展开成一份拷贝）；每步下沉至多 ⌊log2 count⌋ 层，比较至多 2 ⌊log2 count⌋ 次，与到达的次序无关
fn order_rows_by_their_first_four_words(rows: u32, stride: u32, count: u32, order: u32) {
    for (var i = 0u; i < count; i = i + 1u) {
        sws(order + i, i);
    }
    if (count < 2u) {
        return;
    }
    let heap_building_steps = count / 2u;
    var heap_size = count;
    for (var step = 0u; step < heap_building_steps + count - 1u; step = step + 1u) {
        var node = 0u;
        if (step < heap_building_steps) {
            node = heap_building_steps - 1u - step;
        } else {
            heap_size = heap_size - 1u;
            let largest = sw(order);
            sws(order, sw(order + heap_size));
            sws(order + heap_size, largest);
        }
        loop {
            let left = 2u * node + 1u;
            if (left >= heap_size) {
                break;
            }
            var larger = left;
            if (left + 1u < heap_size && ordered_row_precedes(rows, stride, sw(order + left), sw(order + left + 1u))) {
                larger = left + 1u;
            }
            if (!ordered_row_precedes(rows, stride, sw(order + node), sw(order + larger))) {
                break;
            }
            let parent_row = sw(order + node);
            sws(order + node, sw(order + larger));
            sws(order + larger, parent_row);
            node = larger;
        }
    }
}

fn handoff_region_device(region: u32) -> u32 {
    if (region == 0u) {
        return sw(H_BASE + H_REGION0);
    }
    if (region == 1u) {
        return sw(H_BASE + H_REGION1);
    }
    return sw(H_BASE + H_REGION2);
}
fn handoff_unit_area_start_slot() -> vec2<u32> {
    return vec2<u32>(sw(H_BASE + H_UNIT_AREA_START_LO), sw(H_BASE + H_UNIT_AREA_START_HI));
}
// 扫描内核开工：把走读内核留下的计数装回私有变量
fn handoff_load() {
    w_instance_row_count = sw(H_BASE + H_ROW_COUNT);
    // 走读内核记到第 KEYED_TABLE_ROWS_FOUND_BY_SCANNING_AT_MOST + 1 行那一刻建了这张表的桶表（⑬），之后没清过：条数过了那个界就是建过
    w_instance_rows_are_indexed = w_instance_row_count > KEYED_TABLE_ROWS_FOUND_BY_SCANNING_AT_MOST;
    w_reference_count[0] = sw(H_BASE + H_REFERENCE_COUNT);
    w_reference_count[1] = 0u;
    c_newest_reference_count = sw(H_BASE + H_NEWEST_REFERENCE_COUNT);
    w_accounting_row_count = sw(H_BASE + H_ACCOUNTING_ROW_COUNT);
    w_accounting_seen = sw(H_BASE + H_ACCOUNTING_SEEN) == 1u;
    c_candidate_count = sw(H_BASE + H_CANDIDATE_COUNT);
    c_newest_index = sw(H_BASE + H_NEWEST_INDEX);
    c_effective_floor = vec2<u32>(sw(H_BASE + H_FLOOR_LO), sw(H_BASE + H_FLOOR_HI));
    c_version_count = sw(H_BASE + H_VERSION_COUNT);
    record_count = sw(H_BASE + H_RECORD_COUNT);
    w_mount_instance = sw(H_BASE + H_MOUNT_INSTANCE);
    w_mount_txg = vec2<u32>(sw(H_BASE + H_MOUNT_TXG_LO), sw(H_BASE + H_MOUNT_TXG_HI));
    root_count = sw(H_BASE + H_ROOT_COUNT);
}


// ───────────────────────── 中央映射 key 与一版的中央映射（walk.rs 的 CentralMappingKey、CentralMappingLocationsOfAVersion）─────────────────────────
// 判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3 与第七节用户定案「重构」：豁免三类（映射树自己的节点、树表单元、实例表单元）之外的
// 码 1 / 码 2 / 码 3 指针，位置条目只是提示，单元落在哪以这一版中央映射里同一把 key 的条目为准（D19（块指针的结构与宽度预算） 已定项 5、8）。

// 中央映射 key 27 字节（D19 已定项 6 / 10：类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 尾段 6）
const MAPPING_KEY_BYTES: u32 = 27u;
// 中央映射叶条目 55 字节：key 27 之后两条位置条目各 14（设备身份 4、槽号 6、整单元校验和 4）
const MAPPING_ENTRY_BYTES: u32 = 55u;
// 查找路径给的类（walk.rs 的 MappingKeyUnitClass 的类标签）：0 是按指针自己的位置条目（自举豁免的三类，不查映射）
const MAPPING_CLASS_THE_POINTER_ITSELF: u32 = 0u;
const MAPPING_CLASS_DATA_UNIT: u32 = 1u;
const MAPPING_CLASS_INDEX_NODE: u32 = 2u;
const MAPPING_CLASS_PACKED_RECORD_UNIT: u32 = 3u;

// 一把中央映射 key 拆成的 7 个字：类标签与尾段高 16 位、出生树、出生 txg、实例代号、尾段低 32 位
struct MappingKey {
    class_and_tail_high: u32,
    birth_tree: vec2<u32>,
    birth_txg: vec2<u32>,
    instance: u32,
    tail_low: u32,
};

// 盘上 27 字节（base, o 起：中央映射叶条目的 key、deadlist 条目里的映射 key）解成的 key
fn mapping_key_of_the_bytes(base: u32, o: u32) -> MappingKey {
    var key: MappingKey;
    key.class_and_tail_high = rd8(base, o) | (rd16(base, o + 25u) << 16u);
    key.birth_tree = rd64(base, o + 1u);
    key.birth_txg = rd64(base, o + 9u);
    key.instance = rd32(base, o + 17u);
    key.tail_low = rd32(base, o + 21u);
    return key;
}

// 一条指针指的那个单元的 key（walk.rs central_mapping_key_of_a_pointer）：类标签由查找路径给，出生树、出生 txg、实例代号取指针头里的，
// 尾段取指针尾段（码 1 是写序的事务号 6 字节，码 2 / 码 3 是出生序号 4 字节、其后 2 字节补零）
fn mapping_key_of_a_pointer(p: Ptr, unit_class: u32) -> MappingKey {
    var key: MappingKey;
    key.class_and_tail_high = unit_class | ((p.tail.y & 0xFFFFu) << 16u);
    key.birth_tree = p.birth_tree;
    key.birth_txg = p.birth_txg;
    key.instance = p.instance;
    key.tail_low = p.tail.x;
    return key;
}

fn same_mapping_key(a: MappingKey, b: MappingKey) -> bool {
    return a.class_and_tail_high == b.class_and_tail_high && eq64(a.birth_tree, b.birth_tree) && eq64(a.birth_txg, b.birth_txg)
        && a.instance == b.instance && a.tail_low == b.tail_low;
}

fn mapping_key_first_bucket(key: MappingKey, bucket_count: u32) -> u32 {
    return keyed_first_bucket(vec4<u32>(key.birth_tree.x ^ key.birth_tree.y, key.birth_txg.x ^ key.birth_txg.y, key.instance,
        key.tail_low ^ key.class_and_tail_high), bucket_count);
}

// 一版中央映射叶里收了几条（C_VERSION_MAPPING_ROWS 那一张；0 是这一版没有映射、读不全或还没读）与桶表建过没有（条数过了
// KEYED_TABLE_ROWS_FOUND_BY_SCANNING_AT_MOST 才建，同 ⑬）
var<private> version_mapping_count: u32;
var<private> version_mapping_is_indexed: bool;
// 最近那一次 read_the_central_mapping_without_judging 读全了没有（根指针全零也算读全：那一版没有映射）。走读与引用集合那两处读不全照提示走、不看它；
// 引用与快照那一截的 I-3.15、I-1.14 照 walk.rs 的 central_mapping_tree_of_a_version_without_judging 拿它分「这一版的映射读不全」
var<private> version_mapping_read_in_full: bool;

const UNDECIDABLE_VERSION_MAPPING_ENTRIES_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (90u << 8u);
const UNDECIDABLE_VERSION_MAPPING_PENDING_PAST_THE_STACK: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (91u << 8u);

// 7 个字记下的一把 key（行从 row 起；一版的中央映射与引用与快照那一截的单元表都这样记）
fn mapping_key_of_the_words(row: u32) -> MappingKey {
    var key: MappingKey;
    key.class_and_tail_high = sw(row);
    key.birth_tree = vec2<u32>(sw(row + 1u), sw(row + 2u));
    key.birth_txg = vec2<u32>(sw(row + 3u), sw(row + 4u));
    key.instance = sw(row + 5u);
    key.tail_low = sw(row + 6u);
    return key;
}

fn put_the_mapping_key_into_the_words(row: u32, key: MappingKey) {
    sws(row, key.class_and_tail_high);
    sws(row + 1u, key.birth_tree.x);
    sws(row + 2u, key.birth_tree.y);
    sws(row + 3u, key.birth_txg.x);
    sws(row + 4u, key.birth_txg.y);
    sws(row + 5u, key.instance);
    sws(row + 6u, key.tail_low);
}

fn version_mapping_key_of_row(row_index: u32) -> MappingKey {
    return mapping_key_of_the_words(C_VERSION_MAPPING_ROWS + row_index * C_VERSION_MAPPING_ROW_WORDS);
}

// 表里键是 key 的那一行的下标（按记进的次序第一行；没有是 NONE）。找到的那一行可能带「叶里出现过两次」的记号，由调用方看
fn version_mapping_first_row_of(key: MappingKey) -> u32 {
    var bucket = mapping_key_first_bucket(key, C_VERSION_MAPPING_BUCKET_COUNT);
    let steps = select(version_mapping_count, C_VERSION_MAPPING_BUCKET_COUNT, version_mapping_is_indexed);
    for (var step = 0u; step < steps; step = step + 1u) {
        var row_index = step;
        if (version_mapping_is_indexed) {
            let row_plus_one = sw(C_VERSION_MAPPING_BUCKETS + bucket);
            if (row_plus_one == 0u) {
                return NONE;
            }
            row_index = row_plus_one - 1u;
            bucket = (bucket + 1u) & (C_VERSION_MAPPING_BUCKET_COUNT - 1u);
        }
        if (same_mapping_key(version_mapping_key_of_row(row_index), key)) {
            return row_index;
        }
    }
    return NONE;
}

// walk.rs CentralMappingLocationsOfAVersion::read_without_judging：从一版的映射根指针只读读出叶里全部条目（多层，每下一层层级减一；key 宽 27、
// 叶条目宽 ≥ 55、内部条目宽 ≥ 27 + 86、子指针非全零）。根指针全零或有一处读不全：表是空的（整版照提示走，读不全由主走读的映射那一遍判）。
// 同一个 key 在叶里出现两次的那几条不收（哪一条作数没有条款）：第一次记下的那一行带记号，找到它时当没有。显式栈借 C_PENDING（I-3.5 摊单元也借那一段，
// 借它的时段不在读映射的时候；w_merge_below 的两张表在草稿区末尾，不借这一段）
fn read_the_central_mapping_without_judging(mapping_root: Ptr) {
    version_mapping_count = 0u;
    version_mapping_is_indexed = false;
    version_mapping_read_in_full = mapping_root.all_zero;
    if (mapping_root.all_zero) {
        return;
    }
    var count = 0u;
    var pending = 0u;
    var current = mapping_root;
    var expected_level = NONE;
    loop {
        let n = parse_index_node(read_unit_via_locations(current, NODE_BYTES));
        if (!n.ok || n.key_width != MAPPING_KEY_BYTES || (expected_level != NONE && n.level != expected_level)) {
            return;
        }
        if (n.level == 0u) {
            if (n.entry_width < MAPPING_ENTRY_BYTES) {
                return;
            }
            for (var e = 0u; e < n.entry_count; e = e + 1u) {
                if (count >= C_MAX_VERSION_MAPPING_ROWS) {
                    undecidable = UNDECIDABLE_VERSION_MAPPING_ENTRIES_PAST_THE_TABLE;
                    return;
                }
                let row = C_VERSION_MAPPING_ROWS + count * C_VERSION_MAPPING_ROW_WORDS;
                let o = entry_offset(n, e);
                put_the_mapping_key_into_the_words(row, mapping_key_of_the_bytes(n.base, o));
                sws(row + 7u, 0u);
                for (var location = 0u; location < 2u; location = location + 1u) {
                    let location_o = o + MAPPING_KEY_BYTES + location * 14u;
                    let slot = rd48(n.base, location_o + 4u);
                    sws(row + 8u + location * 4u, rd32(n.base, location_o));
                    sws(row + 9u + location * 4u, slot.x);
                    sws(row + 10u + location * 4u, slot.y);
                    sws(row + 11u + location * 4u, rd32(n.base, location_o + 10u));
                }
                count = count + 1u;
            }
        } else {
            if (n.entry_width < MAPPING_KEY_BYTES + 86u) {
                return;
            }
            for (var e = 0u; e < n.entry_count; e = e + 1u) {
                let child_offset = entry_offset(n, e) + MAPPING_KEY_BYTES;
                if (parse_node_pointer(n.base, child_offset).all_zero) {
                    return;
                }
                if (pending >= C_MAX_PENDING) {
                    undecidable = UNDECIDABLE_VERSION_MAPPING_PENDING_PAST_THE_STACK;
                    return;
                }
                sws(C_PENDING + pending * 3u, n.base);
                sws(C_PENDING + pending * 3u + 1u, child_offset);
                sws(C_PENDING + pending * 3u + 2u, n.level - 1u);
                pending = pending + 1u;
            }
        }
        if (pending == 0u) {
            break;
        }
        pending = pending - 1u;
        current = parse_node_pointer(sw(C_PENDING + pending * 3u), sw(C_PENDING + pending * 3u + 1u));
        expected_level = sw(C_PENDING + pending * 3u + 2u);
    }
    // 记进表：同一个 key 的第二条起不记，第一条带上「出现过两次」的记号；条数过了界才建桶表（找与记都只一个比键的调用点）
    let indexed = count > KEYED_TABLE_ROWS_FOUND_BY_SCANNING_AT_MOST;
    if (indexed) {
        for (var bucket = 0u; bucket < C_VERSION_MAPPING_BUCKET_COUNT; bucket = bucket + 1u) {
            sws(C_VERSION_MAPPING_BUCKETS + bucket, 0u);
        }
    }
    version_mapping_is_indexed = indexed;
    for (var row_index = 0u; row_index < count; row_index = row_index + 1u) {
        let key = version_mapping_key_of_row(row_index);
        // 找的是已经记进去的前 row_index 行
        version_mapping_count = row_index;
        let earlier = version_mapping_first_row_of(key);
        if (earlier != NONE) {
            sws(C_VERSION_MAPPING_ROWS + earlier * C_VERSION_MAPPING_ROW_WORDS + 7u, 1u);
            continue;
        }
        if (indexed) {
            var bucket = mapping_key_first_bucket(key, C_VERSION_MAPPING_BUCKET_COUNT);
            for (var probe = 0u; probe < C_VERSION_MAPPING_BUCKET_COUNT; probe = probe + 1u) {
                if (sw(C_VERSION_MAPPING_BUCKETS + bucket) == 0u) {
                    sws(C_VERSION_MAPPING_BUCKETS + bucket, row_index + 1u);
                    break;
                }
                bucket = (bucket + 1u) & (C_VERSION_MAPPING_BUCKET_COUNT - 1u);
            }
        }
    }
    version_mapping_count = count;
    version_mapping_read_in_full = true;
}

// walk.rs CentralMappingLocationsOfAVersion::locate：按查找路径给的类（mapping_class）认一条指针指的单元落在哪。类是 0（自举豁免）、指针全零、
// 这一版的映射里查不到（或那把 key 在叶里出现过两次）的原样交回；查得到、且映射条目两条位置条目带的整单元校验和与指针里的逐条相同（同一把 key 的单元
// 逐字节不变，过期的提示只差落点，D19 已定项 8、判决 G3 的 N6）时，换成映射条目里的两条位置（盘、槽；校验和本来就相同）。
// 校验和不同的那一条指针照它自己的位置条目读、判（实现员 2026-10-15 推的读法，walk.rs 那一处原文）
fn located_by_the_central_mapping(p: Ptr, mapping_class: u32) -> Ptr {
    if (mapping_class == MAPPING_CLASS_THE_POINTER_ITSELF || p.all_zero || version_mapping_count == 0u) {
        return p;
    }
    let row_index = version_mapping_first_row_of(mapping_key_of_a_pointer(p, mapping_class));
    if (row_index == NONE) {
        return p;
    }
    let row = C_VERSION_MAPPING_ROWS + row_index * C_VERSION_MAPPING_ROW_WORDS;
    if (sw(row + 7u) != 0u || sw(row + 11u) != p.sum0 || sw(row + 15u) != p.sum1) {
        return p;
    }
    var located = p;
    located.device0 = sw(row + 8u);
    located.slot0 = vec2<u32>(sw(row + 9u), sw(row + 10u));
    located.device1 = sw(row + 12u);
    located.slot1 = vec2<u32>(sw(row + 13u), sw(row + 14u));
    return located;
}

// 读回那一份头里的出生身份（诞生代号、写序实例、尾段）与查找路径的那一把相同（walk.rs birth_identity_in_the_header 那三段比指针）：
// 提示读到一个头与查找路径不符的单元照「读不出」走映射（判决 G3 那一格里的攻方 K-f）。类标签不在 1 / 2 / 3 里的不认。
// 走读内核的主走读（w_read_unit_to_walk）与扫描内核的次级读法（r_read_located_unit）先按提示读时都经它
fn unit_birth_identity_matches_the_pointer(s: Slice, p: Ptr) -> bool {
    let unit_class = rd8(s.base, 6u);
    if (unit_class == 1u) {
        return eq64(rd64(s.base, 75u), p.birth_txg) && rd32(s.base, 91u) == p.instance && eq64(rd48(s.base, 95u), p.tail);
    }
    if (unit_class == 2u) {
        let key_span = 2u * rd8(s.base, 51u);
        return eq64(rd64(s.base, 52u + key_span), p.birth_txg) && rd32(s.base, 68u + key_span) == p.instance
            && eq64(vec2<u32>(rd32(s.base, 72u + key_span), 0u), p.tail);
    }
    if (unit_class == 3u) {
        return eq64(rd64(s.base, 73u), p.birth_txg) && rd32(s.base, 93u) == p.instance && eq64(vec2<u32>(rd32(s.base, 103u), 0u), p.tail);
    }
    return false;
}

// 树表第 e 条（码 14 或 15）是不是销毁中的快照的树（walk.rs snapshot_trees_of_snapshots_being_destroyed）：码 14 看自己 flags 位 0，
// 码 15 看头 ID 配上的那几条码 14 里有没有带这一位的
fn snapshot_tree_is_being_destroyed(tt: Node, e: u32) -> bool {
    let o = entry_offset(tt, e);
    if (rd16(tt.base, o + 10u) == TREE_KIND_SNAPSHOT_INODE_ROOT) {
        return (rd16(tt.base, o + TREE_TABLE_ENTRY_FLAGS_OFFSET) & TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED) != 0u;
    }
    let head = rd64(tt.base, o + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
    for (var other = 0u; other < tt.entry_count; other = other + 1u) {
        let other_o = entry_offset(tt, other);
        if (rd16(tt.base, other_o + 10u) == TREE_KIND_SNAPSHOT_INODE_ROOT && eq64(rd64(tt.base, other_o), head)
            && (rd16(tt.base, other_o + TREE_TABLE_ENTRY_FLAGS_OFFSET) & TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED) != 0u) {
            return true;
        }
    }
    return false;
}
