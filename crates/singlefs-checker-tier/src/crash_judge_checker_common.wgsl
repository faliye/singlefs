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

var<private> violated_lo: u32;

var<private> violated_hi: u32;

fn violate(invariant: u32) {
    if (invariant < 32u) {
        violated_lo = violated_lo | (1u << invariant);
    } else {
        violated_hi = violated_hi | (1u << (invariant - 32u));
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

fn checker_root_region(index: u32) -> u32 {
    return sw(R_ROOTS + index * 2u + 1u);
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
    let version = sw(R_RECORDS + index * RECORD_WORDS);
    var r: CheckerRecord;
    r.base = B[T_VERSION_BYTES] + tab(T_VERSIONS, version * VERSION_WORDS + 1u);
    r.counter = vec2<u32>(sw(R_RECORDS + index * RECORD_WORDS + 1u), sw(R_RECORDS + index * RECORD_WORDS + 2u));
    r.instance = sw(R_RECORDS + index * RECORD_WORDS + 3u);
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

const C_CANDIDATE_WORDS: u32 = 40u;          // 0 走全了、1 有树表落点、2 盘、3 槽 lo、4 槽 hi、5 树数、6.. 每树 4 字（树 lo/hi、诞生 lo/hi）

const C_MAX_TREES_PER_TABLE: u32 = 8u;

const C_CANDIDATE_INFO: u32 = 166416u;

const C_ALLOC_RECORD_WORDS: u32 = 8u;        // 盘、槽 lo/hi、跨度、代 lo/hi、已释放、0

const C_MAX_ALLOC_RECORDS: u32 = 2048u;

const C_ALLOC_RECORDS: u32 = 168336u;

const C_R_MAX_SEEN: u32 = 512u;

const C_R_SEEN: u32 = 184720u;                // 盘、槽 lo/hi

const C_R_PENDING_WORDS: u32 = 10u;

const C_R_MAX_PENDING: u32 = 512u;

const C_R_PENDING: u32 = 186256u;

const C_MAX_SEEN_NODES: u32 = 256u;

const C_SEEN_NODES: u32 = 191376u;           // 盘、槽 lo/hi、校验和

const C_MAX_EXAMINED: u32 = 2048u;

const C_EXAMINED: u32 = 192400u;             // 盘、槽 lo/hi、代 lo/hi

const C_MAX_SCANNED: u32 = 2048u;

const C_SCANNED: u32 = 202640u;              // 落点下标

const C_R_MAX_ROWS: u32 = 256u;

const C_R_ROWS: u32 = 204688u;               // 实例、txg lo/hi

const C_VALID_BEFORE_WORDS: u32 = 6u;        // 根下标、有指针、inode 基、inode 偏移、extent 基、extent 偏移

const C_VALID_BEFORE: u32 = 205456u;

const C_NON_EMPTY: u32 = 205744u;            // txg lo/hi、实例、valid_before 下标

const C_UNDETERMINABLE: u32 = 205936u;       // txg lo/hi、实例

const C_COUNTED: u32 = 206080u;              // txg lo/hi

const C_COUNTED_STATES: u32 = 206176u;       // valid_before 下标

const C_PICKED: u32 = 206224u;               // 选过的标记（非空 48 + 判不了 48）

const C_MAX_RELEASED_ABOVE: u32 = 2048u;

const C_RELEASED_ABOVE: u32 = 206320u;       // 盘、槽 lo/hi、跨度

const C_MAX_DEFERRED: u32 = 4096u;

const C_DEFERRED: u32 = 214512u;             // 盘、槽 lo/hi

const C_MAX_GROUPS: u32 = 2048u;

const C_GROUPS: u32 = 226800u;               // 槽 lo/hi、跨度、盘位掩码

const C_SCRATCH_END_SCAN: u32 = 234992u;

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

// 链指针记录：0 最后一片、1 有下一片、2 坏
fn w_chain_record_kind(base: u32, o: u32, width: u32) -> u32 {
    if (width != 88u || rd8(base, o) != 1u) {
        return 2u;
    }
    let pointer_zero = all_zero(base, o + 2u, 86u);
    let flag = rd8(base, o + 1u);
    if (flag == 0u && pointer_zero) {
        return 0u;
    }
    if (flag == 1u && !pointer_zero) {
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
const H_BASE: u32 = 234992u;
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
const H_MAX_BELOW_FLOOR_SLOTS: u32 = 4096u;
const H_BELOW_FLOOR_SLOTS: u32 = 235024u;   // 盘、槽 lo/hi
const H_END: u32 = 247312u;

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

