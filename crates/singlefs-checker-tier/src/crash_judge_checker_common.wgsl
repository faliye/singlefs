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

// 第 70–74 条：里程碑三·3 删除线那五条（只往末尾追加），扫描内核的删除态扫描判前四条；I-4.4 第一版报不适用，内核不置它那一位
const INVARIANT_I_9_5: u32 = 69u;

const INVARIANT_I_3_4: u32 = 70u;

const INVARIANT_I_4_7: u32 = 71u;

const INVARIANT_I_4_6: u32 = 72u;

const INVARIANT_I_4_4: u32 = 73u;

// 第 75–83 条：里程碑三·3 墓碑与谱系那九条（只往末尾追加）。走读内核判 I-9.17、I-9.23（墓碑树、谱系树每条内部条目的身份）；
// 扫描内核在删除态扫描那一份版本清单上判 I-9.18–I-9.21、I-9.24、I-9.29。I-9.22 恒不置位：它只在池级 checker 带旋钮值的入口上判，
// 层 0 与放量都走旧入口（`check_pool_image`，当没装旋钮、恒报不适用），GPU 手里也没有旋钮值——照 I-4.4 的先例登记、内核不置它那一位
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
// （层 0 与放量走池级 checker 的旧入口，GPU 也没有旋钮值）
const INVARIANT_I_9_36: u32 = 90u;

const INVARIANT_I_9_37: u32 = 91u;

// 墓碑树、谱系树叶容器的打包记录类型（walk.rs 的 PACKED_TYPE_TOMBSTONE / PACKED_TYPE_LINEAGE；登记的记录宽墓碑 40、谱系 24）
const PACKED_TYPE_TOMBSTONE: u32 = 1u;

const PACKED_TYPE_LINEAGE: u32 = 5u;

// 退役树叶容器的打包记录类型（walk.rs 的 PACKED_TYPE_RETIREMENT；登记的记录宽 34）
const PACKED_TYPE_RETIREMENT: u32 = 3u;

// deleted_inodes 根的条目：key = inode 号 8 字节、无 value（walk.rs 的 DELETED_INODES_ENTRY_BYTES）
const DELETED_INODES_ENTRY_BYTES: u32 = 8u;

// 树表条目里头 ID 8 字节的偏移（walk.rs 的 TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET）
const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: u32 = 116u;

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

fn checker_find_record_by_counter(counter: vec2<u32>) -> u32 {
    for (var i = 0u; i < record_count; i = i + 1u) {
        if (sw(R_RECORDS + i * RECORD_WORDS + 1u) == counter.x && sw(R_RECORDS + i * RECORD_WORDS + 2u) == counter.y) {
            return i;
        }
    }
    return NONE;
}

// ───────────────────────── 走读（walk.rs 的 Walk）─────────────────────────
//
// 草稿区（接在共用那一半的 SCRATCH_WORDS_REQUIRED 之后）。两套上下文：0 是主走读（判定进违例位、引用集合给 I-3.1 / I-5.1），
// 1 是「另起一份 Walk」的那几遍（被抛弃根、F 之下的根、收位置项）：判定丢掉、只数走读失败与 I-2.1 不符、引用与位置项另放。
const C_REFERENCE_WORDS: u32 = 5u;          // 盘、槽 lo、槽 hi、跨度、0

const C_MAX_REFERENCES: u32 = 2048u;

const C_REFERENCES0: u32 = 17504u;

const C_REFERENCES1: u32 = 27744u;

const C_MAX_VISITED: u32 = 2048u;

const C_VISITED0: u32 = 37984u;             // 盘、槽 lo、槽 hi

const C_VISITED1: u32 = 44128u;

const C_MAX_IN_VERSION: u32 = 2048u;

const C_IN_VERSION0: u32 = 50272u;          // 盘、槽 lo、槽 hi

const C_IN_VERSION1: u32 = 56416u;

const C_BELOW_WORDS: u32 = 6u;              // 父盘、父槽 lo/hi、子盘、子槽 lo/hi

const C_MAX_BELOW: u32 = 4096u;

const C_BELOW0: u32 = 62560u;

const C_BELOW1: u32 = 87136u;

const C_LEAF_WORDS: u32 = 9u;               // 父盘、父槽 lo/hi、容器 lo/hi、最小 lo/hi、最大 lo/hi

const C_MAX_LEAVES: u32 = 512u;

const C_LEAVES: u32 = 111712u;

const C_CHECKSUM_WORDS: u32 = 4u;           // 盘、槽 lo/hi、校验和

const C_MAX_CHECKSUMS: u32 = 2048u;

const C_CHECKSUMS: u32 = 116320u;

const C_HARVEST: u32 = 124512u;

const C_MAX_TREE_IDS: u32 = 64u;

const C_TREE_IDS: u32 = 132704u;             // lo、hi

const C_MAX_INODE_BIRTHS: u32 = 512u;

const C_INODE_BIRTHS: u32 = 132832u;         // inode lo/hi、出生代 lo/hi

const C_MAX_DATA_OBJECTS: u32 = 1024u;

const C_DATA_OBJECTS: u32 = 134880u;         // inode lo/hi、出生代 lo/hi

const C_MAX_ACCOUNTING_ROWS: u32 = 64u;

const C_ACCOUNTING_ROWS: u32 = 138976u;      // 统计量、盘、值 lo/hi

const C_ROW_WORDS: u32 = 5u;                // 实例、txg lo/hi、W lo/hi

const C_MAX_INSTANCE_ROWS: u32 = 1024u;

const C_INSTANCE_ROWS: u32 = 139232u;

const C_CHILD_WORDS: u32 = 7u;              // 种类、最小 key 基/偏移、最大 key 基/偏移、分隔 key 基/偏移

const C_MAX_CHILDREN: u32 = 1024u;

const C_CHILDREN: u32 = 144352u;

const C_SEPARATOR_WORDS: u32 = 6u;          // 分隔 lo/hi、最小 lo/hi、最大 lo/hi

const C_MAX_SEPARATORS: u32 = 256u;

const C_SEPARATORS: u32 = 151520u;

const C_MAX_PENDING: u32 = 1024u;

const C_PENDING: u32 = 153056u;              // 盘、槽 lo/hi

const C_EXPANDED: u32 = 156128u;

const C_SCRATCH_END: u32 = 159200u;

// ───────────────────────── 第三批：只读不判的读法（walk.rs 的 *_without_judging）与扫描、记账那几条 ─────────────────────────
const C_MAX_RELEASED: u32 = 1024u;

const C_RELEASED: u32 = 159200u;              // 盘、槽 lo/hi、释放代 lo/hi

const C_REFERENCED_BY: u32 = 164320u;         // 每条已释放记录两个字：位 c ↔ 第 c 条候选根引用它

const C_MAX_CANDIDATES: u32 = 48u;

const C_CANDIDATES: u32 = 166368u;            // 候选根在根表里的下标

const C_CANDIDATE_WORDS: u32 = 54u;          // 0 走全了、1 有树表落点、2 盘、3 槽 lo、4 槽 hi、5 树数、6.. 每树 4 字（树 lo/hi、诞生 lo/hi）：6 + 4 × C_MAX_TREES_PER_TABLE

// 一张树表最多记几棵不同的树：进树表的种类有 12 个（recovery.rs 与 walk.rs 的 TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER，
// 第一个文件版本那次就写满 12 条，退役树是第 50 批加的）；合法镜像上一张树表每种至多一条，更多的只出现在坏镜像上，第 13 棵起判不了、交 CPU。
// 从 11 加到 12 时 C_CANDIDATE_INFO 那 48 行每行多 4 字，其后的草稿区偏移各后移 48 × 4 = 192 字
const C_MAX_TREES_PER_TABLE: u32 = 12u;

// I-9.28 在 I-3.9 与 I-9.14 判完之后借这几行，根表里第 k 条根记进第 k 行（只用 1..5 与每树那几字）：根表最多 MAX_ROOTS 条，
// 不多于 C_MAX_CANDIDATES 行（judge_tree_identifier_birth_txg_across_every_readable_root 开头判，越过就判不了）
const C_CANDIDATE_INFO: u32 = 166416u;

const C_ALLOC_RECORD_WORDS: u32 = 8u;        // 盘、槽 lo/hi、跨度、代 lo/hi、已释放、0

const C_MAX_ALLOC_RECORDS: u32 = 2048u;

const C_ALLOC_RECORDS: u32 = 169008u;

const C_R_MAX_SEEN: u32 = 512u;

const C_R_SEEN: u32 = 185392u;                // 盘、槽 lo/hi

const C_R_PENDING_WORDS: u32 = 10u;

const C_R_MAX_PENDING: u32 = 512u;

const C_R_PENDING: u32 = 186928u;

const C_MAX_SEEN_NODES: u32 = 256u;

const C_SEEN_NODES: u32 = 192048u;           // 盘、槽 lo/hi、校验和

const C_MAX_EXAMINED: u32 = 2048u;

const C_EXAMINED: u32 = 193072u;             // 盘、槽 lo/hi、代 lo/hi

const C_MAX_SCANNED: u32 = 2048u;

const C_SCANNED: u32 = 203312u;              // 落点下标

const C_R_MAX_ROWS: u32 = 256u;

const C_R_ROWS: u32 = 205360u;               // 实例、txg lo/hi

const C_VALID_BEFORE_WORDS: u32 = 6u;        // 根下标、有指针、inode 基、inode 偏移、extent 基、extent 偏移

const C_VALID_BEFORE: u32 = 206128u;

const C_NON_EMPTY: u32 = 206416u;            // txg lo/hi、实例、valid_before 下标

const C_UNDETERMINABLE: u32 = 206608u;       // txg lo/hi、实例

const C_COUNTED: u32 = 206752u;              // txg lo/hi

const C_COUNTED_STATES: u32 = 206848u;       // valid_before 下标

const C_PICKED: u32 = 206896u;               // 选过的标记（非空 48 + 判不了 48）

const C_MAX_RELEASED_ABOVE: u32 = 2048u;

const C_RELEASED_ABOVE: u32 = 206992u;       // 盘、槽 lo/hi、跨度

const C_MAX_DEFERRED: u32 = 4096u;

const C_DEFERRED: u32 = 215184u;             // 盘、槽 lo/hi

const C_MAX_GROUPS: u32 = 2048u;

const C_GROUPS: u32 = 227472u;               // 槽 lo/hi、跨度、盘位掩码

const C_SCRATCH_END_SCAN: u32 = 235664u;

var<private> w_context: u32;

var<private> w_judging: bool;

var<private> w_harvesting: bool;

var<private> w_reference_count: array<u32, 2>;

var<private> w_visited_count: array<u32, 2>;

var<private> w_in_version_count: array<u32, 2>;

var<private> w_below_count: array<u32, 2>;

var<private> w_leaf_count: u32;

var<private> w_checksum_count: u32;

var<private> w_harvest_count: u32;

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
    for (var i = 0u; i < w_instance_row_count; i = i + 1u) {
        if (sw(C_INSTANCE_ROWS + i * C_ROW_WORDS) != instance) {
            continue;
        }
        let row_txg = vec2<u32>(sw(C_INSTANCE_ROWS + i * C_ROW_WORDS + 1u), sw(C_INSTANCE_ROWS + i * C_ROW_WORDS + 2u));
        let row_high_water = vec2<u32>(sw(C_INSTANCE_ROWS + i * C_ROW_WORDS + 3u), sw(C_INSTANCE_ROWS + i * C_ROW_WORDS + 4u));
        if (is_data_unit) {
            return le64(birth_txg, row_txg) || le64(transaction, row_high_water);
        }
        return le64(birth_txg, row_txg);
    }
    return true;
}

// key 形态：0 记账 [2,8,4,8]、1 映射 [1,8,8,4,6]、2 树表与 inode [8]、3 extent [8,8,8]、4 分配记录 [4,6]
fn schema_field_count(schema: u32) -> u32 {
    if (schema == 0u) { return 4u; }
    if (schema == 1u) { return 5u; }
    if (schema == 2u) { return 1u; }
    if (schema == 3u) { return 3u; }
    return 2u;
}

fn schema_field_width(schema: u32, field: u32) -> u32 {
    if (schema == 0u || schema == 1u) {
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
    for (var i = 0u; i < w_instance_row_count; i = i + 1u) {
        if (sw(C_INSTANCE_ROWS + i * C_ROW_WORDS) == instance) {
            let row_txg = vec2<u32>(sw(C_INSTANCE_ROWS + i * C_ROW_WORDS + 1u), sw(C_INSTANCE_ROWS + i * C_ROW_WORDS + 2u));
            if (lt64(row_txg, txg)) {
                return true;
            }
        }
    }
    return false;
}

// (txg, 实例) 字典序：a < b
fn root_key_less(txg_a: vec2<u32>, instance_a: u32, txg_b: vec2<u32>, instance_b: u32) -> bool {
    return lt64(txg_a, txg_b) || (eq64(txg_a, txg_b) && instance_a < instance_b);
}

// record_can_enter_a_replay_prefix：计数器 0 进不了；前一条同实例要链值相等；前一条别的实例或不在盘上照进
fn record_can_enter_a_replay_prefix(index: u32) -> bool {
    let r = checker_record_at(index);
    if (is_zero64(r.counter)) {
        return false;
    }
    let previous_index = checker_find_record_by_counter(sub64(r.counter, u64_of(1u)));
    if (previous_index == NONE) {
        return true;
    }
    let previous = checker_record_at(previous_index);
    if (previous.instance != r.instance) {
        return true;
    }
    return r.back_chain == previous.chain_value;
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
const H_BASE: u32 = 235664u;
const H_VALID: u32 = 0u;                 // 1 = 走读那一批判完、判得了；0 = 扫描内核什么都不做
const H_ROW_COUNT: u32 = 1u;
const H_REFERENCE_COUNT: u32 = 2u;
const H_NEWEST_REFERENCE_COUNT: u32 = 3u;
const H_CHECKSUM_COUNT: u32 = 4u;
const H_HARVEST_COUNT: u32 = 5u;
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
const H_BELOW_FLOOR_SLOTS: u32 = 235696u;   // 盘、槽 lo/hi
const H_END: u32 = 247984u;

// ───────────────────────── 交接段之后：走读内核写的两张表（扫描内核只读重放那一张）─────────────────────────
// 主走读（上下文 0）每个走过的单元一行：有没有记下诞生代号、诞生代号 lo/hi（walk.rs 的 birth_txg_of_walked_units，与 C_VISITED0 同下标）。
// 别的版本再引用它时按这张表判 I-4.9（非最新候选根引用的单元不晚于它）。
const C_VISITED_BIRTH_WORDS: u32 = 3u;

const C_VISITED_BIRTHS0: u32 = 247984u;

// I-4.1 重放读的那一份环（walk.rs 的 records_the_replay_reads）：版本、计数器 lo/hi、实例，按 (实例, 计数器) 升序。
// 走读内核在 I-4.1 那一阶段填；扫描内核照交接段的 H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED 读其中一行、不写
const C_REPLAY_RECORD_WORDS: u32 = 4u;

const C_REPLAY_RECORDS: u32 = 254128u;

fn replay_record_at(index: u32) -> CheckerRecord {
    return checker_record_of_row(C_REPLAY_RECORDS + index * C_REPLAY_RECORD_WORDS);
}

// I-3.5（引用区间的精确性）的出现规则读的四张表，只记主走读（上下文 0）走的那几版（最新根、回退候选根、由记录施加出来的版本）。
// ① 落点表（walk.rs 的 unit_of_each_placement 与 user_visible_tree_of_each_placement）：跟着指针记下的每个落点一行——
//    落点盘、槽 lo/hi，它是哪个单元（第一次记下它的那条指针第 0 条位置条目的盘、槽 lo/hi），记下它那一刻走读在不在一棵用户可见树里
//    （第一次在树里记下它时打上，之后不换）、那棵树的种类、树 ID lo/hi。
const C_APPEARANCE_PLACEMENT_WORDS: u32 = 10u;

const C_APPEARANCE_MAX_PLACEMENTS: u32 = 2048u;

const C_APPEARANCE_PLACEMENTS: u32 = 256176u;

// ② 版本表（walk.rs 的 versions_walked_for_the_appearance_rule）：走完的每一版一行——实例、txg lo/hi、走读一条失败都没记（1 / 0）、
//    这一版引用的落点在 ③ 里的起点与条数、树表里用户可见树的根几棵，之后每棵 5 个字（种类、树 ID lo/hi、根指针出生 txg lo/hi）。
const C_APPEARANCE_VERSION_WORDS: u32 = 48u;

const C_APPEARANCE_MAX_USER_VISIBLE_ROOTS: u32 = 8u;

const C_APPEARANCE_USER_VISIBLE_ROOT_WORDS: u32 = 5u;

const C_APPEARANCE_VERSION_ROOTS_OFFSET: u32 = 7u;

const C_APPEARANCE_MAX_VERSIONS: u32 = 64u;

const C_APPEARANCE_VERSIONS: u32 = 276656u;

// ③ 每一版引用的落点（walk.rs 的 placements_referenced_in_this_version 在这一版走完那一刻的样子）：每个落点一个字，是它在 ① 里的下标。
const C_APPEARANCE_MAX_VERSION_PLACEMENTS: u32 = 8192u;

const C_APPEARANCE_VERSION_PLACEMENTS: u32 = 279728u;

// ④ 走过的树表（walk.rs 的 tree_table_entries_of_a_walked_tree_table 里 I-3.5 读的那一样）：第一个读得出它的那一版记下树表单元的盘、槽 lo/hi
//    与它里面用户可见树的根几棵、每棵 5 个字（同 ②）；别的版本共享这片树表时照它取这一版的根。
const C_APPEARANCE_WALKED_TREE_TABLE_WORDS: u32 = 44u;

const C_APPEARANCE_WALKED_TREE_TABLE_ROOTS_OFFSET: u32 = 4u;

const C_APPEARANCE_MAX_WALKED_TREE_TABLES: u32 = 64u;

const C_APPEARANCE_WALKED_TREE_TABLES: u32 = 287920u;

// ───────────────────────── 走读内核那几张表之后：扫描内核的删除态扫描（walk.rs judge_deletion_state_of_the_versions）与墓碑、谱系那一遍
// （walk.rs judge_tombstones_and_the_lineage_of_the_versions）用的表 ─────────────────────────
// ① 这一版一棵 inode 树的记录（walk.rs inode_records_without_judging）：号 lo/hi、nlink、size lo/hi、对象出生代 lo/hi；同一个号再读到时
//    后读到的盖掉先读到的。一版一棵树地重填
const D_INODE_RECORD_WORDS: u32 = 7u;

const D_MAX_INODE_RECORDS: u32 = 1024u;

const D_INODE_RECORDS: u32 = 290736u;

// ② 这一版 extent 树里每个号的数据单元数（walk.rs data_unit_count_of_each_inode_without_judging，全零数据指针不数）：号 lo/hi、单元数
const D_DATA_UNIT_COUNT_WORDS: u32 = 3u;

const D_MAX_DATA_UNIT_INODES: u32 = 1024u;

const D_DATA_UNIT_COUNTS: u32 = 297904u;

// ③ 一棵墓碑树或谱系树按叶序读出的容器（walk.rs containers_of_a_container_tree_without_judging）：容器在打包表里的字节基、记录数、
//    第一条记录的偏移、头里的容器号 lo/hi。一棵树地重填；多于 D_MAX_CONTAINERS_OF_A_CONTAINER_TREE 片判不了（交 CPU）
const D_CONTAINER_OF_A_CONTAINER_TREE_WORDS: u32 = 5u;

const D_MAX_CONTAINERS_OF_A_CONTAINER_TREE: u32 = 256u;

const D_CONTAINERS_OF_A_CONTAINER_TREE: u32 = 300976u;

// ④ 这一版谱系树里的谱系记录（walk.rs lineage_records_without_judging），按 T_rb 从大到小排好：T_rb lo/hi、T_old lo/hi、H_cur lo/hi。
//    多于 D_MAX_LINEAGE_RECORDS 条判不了
const D_LINEAGE_RECORD_WORDS: u32 = 6u;

const D_MAX_LINEAGE_RECORDS: u32 = 256u;

const D_LINEAGE_RECORDS: u32 = 302256u;

// ⑤ I-9.29 跨版本那张表（walk.rs ObjectBirthsOfEachInodeNumber）：inode 树的树 ID lo/hi、inode 号 lo/hi、第一次见到的对象出生代 lo/hi。
//    回退候选集里各版读完一起判，不一版一版重填；多于 D_MAX_OBJECT_BIRTHS 个 (树, 号) 判不了
const D_OBJECT_BIRTH_WORDS: u32 = 6u;

const D_MAX_OBJECT_BIRTHS: u32 = 2048u;

const D_OBJECT_BIRTHS: u32 = 303792u;

// ⑥ I-9.30 那一判读的最新根记账里 inode 号水位那一统计量（不带设备维）的每一行（walk.rs Walk::inode_number_watermark_rows）：代段 lo/hi、值 lo/hi。
//    最新根一棵记账树一棵树地收（读不全的那棵退回去不收）；多于 D_MAX_INODE_NUMBER_WATERMARK_ROWS 行判不了
const D_INODE_NUMBER_WATERMARK_ROW_WORDS: u32 = 4u;

const D_MAX_INODE_NUMBER_WATERMARK_ROWS: u32 = 256u;

const D_INODE_NUMBER_WATERMARK_ROWS: u32 = 316080u;

// ⑦ 退役那几条读的两张叶身份表（walk.rs inode_leaf_identities_without_judging）：一版一棵 inode 树里每片叶容器的身份（出生树 lo/hi、打包记录类型、
//    容器号 lo/hi、容器出生代 lo/hi），集合、一棵树地重填。前一张装被判的那一版（I-9.34、I-9.35），后一张装 I-9.35 的父版；
//    一棵树多于 D_MAX_LEAF_IDENTITIES 片判不了（一个码 2 根装 135 条 inode 内部条目，合法树至多 135 片）
const D_LEAF_IDENTITY_WORDS: u32 = 7u;

const D_MAX_LEAF_IDENTITIES: u32 = 256u;

const D_LEAF_IDENTITIES_OF_THE_VERSION: u32 = 317104u;

const D_LEAF_IDENTITIES_OF_THE_PARENT_VERSION: u32 = 318896u;

// ⑧ 判到位的两张表（走读内核记、扫描内核接着记；判决 m3-crash-enumeration-merge-r2 第二节 L1-4）：评估过、射程里有一部分判不了，
//    各 POOL_CHECKER_JUDGED_WORDS 个字。交接段 32 个字都占满了，接在草稿区末尾（H_BASE 起数的偏移）
const H_EVALUATED: u32 = 320688u - 235664u;
const H_PART_NOT_JUDGED: u32 = H_EVALUATED + POOL_CHECKER_JUDGED_WORDS;

// 每个状态的草稿区到这里为止：crash_judge_gpu.rs 的 SCRATCH_WORDS 与它相等（那一份的单测核）
const SCRATCH_END: u32 = 320696u;

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
    w_reference_count[0] = sw(H_BASE + H_REFERENCE_COUNT);
    w_reference_count[1] = 0u;
    c_newest_reference_count = sw(H_BASE + H_NEWEST_REFERENCE_COUNT);
    w_checksum_count = sw(H_BASE + H_CHECKSUM_COUNT);
    w_harvest_count = sw(H_BASE + H_HARVEST_COUNT);
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

