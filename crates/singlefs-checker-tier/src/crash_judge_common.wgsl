// GPU 判器：每个调用判一个崩溃状态，读 crash_judge_tables 摊出来的表（一张打包的 u32 表 + 目录），
// 按持久掩码给每个落点查版本，在版本字节上按 singlefs-core 的恢复路径解结构、择系统配置、择根、扫 journal 并重放、
// 走到第一个文件，两种 journal 政策各走一遍；再按 oracle 判七类违例、按记录核对器判两条判据。
// 结论：每个状态 8 个 u32（布局见 crash_judge_gpu.rs 的 VERDICT_WORDS 那一段）。判不了的状态置最高位并写原因码，交 CPU 判。

struct Dispatch {
    block_start_lo: u32,
    block_start_hi: u32,
    state_count: u32,
    scratch_words: u32,
};

@group(0) @binding(0) var<uniform> dispatch: Dispatch;
@group(0) @binding(1) var<storage, read> tables: array<u32>;
@group(0) @binding(2) var<storage, read_write> verdicts: array<u32>;
@group(0) @binding(3) var<storage, read_write> scratch: array<u32>;

const NONE: u32 = 0xFFFFFFFFu;
const TABLE_COUNT: u32 = 19u;
// 表号（与 crash_judge_tables::JudgeTablesWords::every_table 同序）
const T_HEADER: u32 = 0u;
const T_RECORDED: u32 = 1u;
const T_REPLAYS: u32 = 2u;
const T_SEGMENTS: u32 = 3u;
const T_SEGMENT_WRITES: u32 = 4u;
const T_WRITES: u32 = 5u;
const T_ROOT_BYTES: u32 = 6u;
const T_LATERS: u32 = 7u;
const T_LOCATIONS: u32 = 8u;
const T_LOCATION_WRITES: u32 = 9u;
const T_VERSIONS: u32 = 10u;
const T_VERSION_BYTES: u32 = 11u;
const T_EQUAL_MASKS: u32 = 12u;
const T_PUBLISHES: u32 = 13u;
const T_PUBLISH_RECORDS: u32 = 14u;
const T_COPY_GROUPS: u32 = 15u;
const T_COPIES: u32 = 16u;
const T_EXPECTED: u32 = 17u;
const T_CHUNKS: u32 = 18u;

const WRITE_WORDS: u32 = 12u;
const LOCATION_WORDS: u32 = 12u;
const VERSION_WORDS: u32 = 8u;
const LATER_WORDS: u32 = 4u;
const PUBLISH_WORDS: u32 = 8u;
const EXPECTED_WORDS: u32 = 6u;

// 落点种类码
const LOCATION_SYSTEM_CONFIGURATION: u32 = 0u;
const LOCATION_ROOT_SLOT: u32 = 1u;
const LOCATION_JOURNAL_RECORD: u32 = 2u;
const LOCATION_UNIT_SLOT: u32 = 3u;
// 写的种类码
const WRITE_UNIT: u32 = 1u;
const WRITE_JOURNAL: u32 = 2u;
const WRITE_ROOT: u32 = 3u;

// 结论位（verdicts 第 0 字）
const BIT_CONSULTED_ORACLE: u32 = 0u;   // 位 0..6：七类
const BIT_IGNORED_ORACLE: u32 = 8u;     // 位 8..14
const BIT_ROOT_WITHOUT_RECORD: u32 = 16u;
const BIT_CLAIMED_STATE_MISSING_UNIT: u32 = 17u;
const BIT_UNDECIDABLE: u32 = 31u;
// oracle 七类（与 Layer0OracleViolationKind::EVERY_KIND 同序）
const ORACLE_NONE: u32 = NONE;
const ORACLE_NO_ROOT_CHOSEN: u32 = 0u;
const ORACLE_RECOVERED_TO_AN_OLDER_ROOT: u32 = 1u;
const ORACLE_READ_FAILED: u32 = 2u;
const ORACLE_NO_FILE_WHERE_AN_OLDER_ROOT_HAS_ONE: u32 = 3u;
const ORACLE_NO_FILE_WHERE_THIS_ROOT_HAS_ONE: u32 = 4u;
const ORACLE_FILE_READ_WHERE_THIS_ROOT_HAS_NONE: u32 = 5u;
const ORACLE_WRONG_CONTENT: u32 = 6u;
// 走读结局
const OUTCOME_NO_FILE: u32 = 0u;
const OUTCOME_FILE_READ: u32 = 1u;
const OUTCOME_FAILED: u32 = 2u;
// 判不了的原因
const UNDECIDABLE_NO_VERSION: u32 = 1u;
const UNDECIDABLE_TOO_MANY_RECORDS: u32 = 2u;
const UNDECIDABLE_TOO_MANY_ENTRIES: u32 = 3u;
const UNDECIDABLE_TOO_DEEP: u32 = 4u;
const UNDECIDABLE_TOO_MANY_DEVICES: u32 = 5u;
const UNDECIDABLE_TOO_MANY_ROOTS: u32 = 6u;

// 格式常量
const SLOT_SHIFT: u32 = 14u;                 // 16384 = 1 << 14
const NODE_BYTES: u32 = 16384u;
const DATA_UNIT_BYTES: u32 = 32768u;
const ROOT_RECORD_BYTES: u32 = 457u;
const JOURNAL_RECORD_BYTES: u32 = 4096u;
const JOURNAL_HEADER_BYTES: u32 = 311u;
const JOURNAL_NAMED_ENTRY_BYTES: u32 = 56u;
const JOURNAL_RING_START_BYTES: u32 = 16777216u;   // 1024 × 16384
const SYSTEM_CONFIGURATION_SLOT_BYTES: u32 = 4096u;
const ROOT_RING_REGION_STRIDE: u32 = 3145728u;     // P × chunk = 3 × 1 MiB
const ROOT_RING_BASE_BYTES: u32 = 1048576u;        // 64 × 16384
const DATA_UNIT_PAYLOAD_CAPACITY: u32 = 32634u;
const EXTENT_UPPER_LEAF_INODES: u32 = 143u;
const EXTENT_LOWER_LEAF_DATA_UNITS: u32 = 144u;
const EXTENT_INTERNAL_FANOUT: u32 = 147u;
const ALLOCATION_LEAF_SLOTS: u32 = 812u;
const ALLOCATION_INTERNAL_FANOUT: u32 = 169u;
const TREE_KIND_EXTENT: u32 = 1u;
const TREE_KIND_INODE: u32 = 2u;
const TREE_KIND_ALLOCATION: u32 = 3u;
const TREE_KIND_ACCOUNTING: u32 = 4u;
const TREE_KIND_LIVELIST: u32 = 6u;
const TREE_KIND_SPARSE_SIDE_TABLE: u32 = 7u;
const TREE_KIND_DEADLIST: u32 = 8u;
const FIRST_INODE_NUMBER: u32 = 1u;
const MAGIC_SFSB: u32 = 0x42534653u;
const MAGIC_SFSR: u32 = 0x52534653u;
const MAGIC_SFSU: u32 = 0x55534653u;
const MAGIC_SFSJ: u32 = 0x4A534653u;

// 草稿区（每个状态一片，单位 u32）
const MAX_RECORDS: u32 = 512u;
const RECORD_WORDS: u32 = 6u;
const R_RECORDS: u32 = 0u;                                   // 3072
const MAX_ROOTS: u32 = 48u;
const R_ROOTS: u32 = 3072u;                                  // 96：版本、区域
const MAX_MAPPING_ENTRIES: u32 = 1024u;
const R_MAPPING: u32 = 3168u;                                // 3072：字节基、条目偏移、条目宽
const MAX_ACCOUNTING_ENTRIES: u32 = 512u;
const R_ACCOUNTING: u32 = 6240u;                             // 1536
const MAX_ALLOCATION_RECORDS: u32 = 1024u;
const ALLOCATION_RECORD_WORDS: u32 = 6u;
const R_ALLOCATION: u32 = 7776u;                             // 6144
const MAX_SEEN: u32 = 512u;
const R_SEEN: u32 = 13920u;                                  // 1024
const MAX_DATA_POINTERS: u32 = 256u;
const R_DATA: u32 = 14944u;                                  // 512：字节基、偏移
const MAX_UPPER_ENTRIES: u32 = 256u;
const R_UPPER: u32 = 15456u;                                 // 512：字节基、偏移
const MAX_ROWS: u32 = 512u;
const R_ROWS: u32 = 15968u;                                  // 1536：实例、txg lo、txg hi
const SCRATCH_WORDS_REQUIRED: u32 = 17504u;

const MAX_DEVICES: u32 = 8u;
const MAX_FRAMES: u32 = 12u;

var<private> B: array<u32, 19>;
var<private> persisted: array<u32, 64>;
var<private> scratch_base: u32;
var<private> undecidable: u32;
var<private> record_count: u32;
// 看 journal 那一遍重放在哪一道停下（诊断：1 在飞上限、2 计数器对不上、3 无锚点首条不对、4 反向链断、5 首条序号不是 1、6 换了 txg、
// 7 序号不连续、8 提交标记没到就换事务、9 末条不带提交标记、10 末条之后同发布还有记录、11 点名单元验不过、12 施加了一版）
var<private> replay_stop_reason: u32;
var<private> root_count: u32;
var<private> mapping_count: u32;
var<private> accounting_count: u32;
var<private> allocation_count: u32;
var<private> seen_count: u32;
var<private> data_count: u32;
var<private> upper_count: u32;
var<private> row_count: u32;

// ───────────────────────── 表与草稿的读写 ─────────────────────────

fn hdr(index: u32) -> u32 {
    return tables[B[T_HEADER] + index];
}
fn tab(table: u32, index: u32) -> u32 {
    return tables[B[table] + index];
}
fn sw(index: u32) -> u32 {
    return scratch[scratch_base + index];
}
fn sws(index: u32, value: u32) {
    scratch[scratch_base + index] = value;
}

fn set_persisted(index: u32) {
    persisted[index >> 5u] = persisted[index >> 5u] | (1u << (index & 31u));
}
fn is_persisted(index: u32) -> bool {
    return (persisted[index >> 5u] & (1u << (index & 31u))) != 0u;
}

// 字节读：base 是 tables 里的绝对字下标，o 是字节偏移
fn rd8(base: u32, o: u32) -> u32 {
    let w = tables[base + (o >> 2u)];
    return (w >> ((o & 3u) << 3u)) & 0xFFu;
}
fn rd16(base: u32, o: u32) -> u32 {
    return rd8(base, o) | (rd8(base, o + 1u) << 8u);
}
fn rd32(base: u32, o: u32) -> u32 {
    if ((o & 3u) == 0u) {
        return tables[base + (o >> 2u)];
    }
    return rd8(base, o) | (rd8(base, o + 1u) << 8u) | (rd8(base, o + 2u) << 16u) | (rd8(base, o + 3u) << 24u);
}
fn rd48(base: u32, o: u32) -> vec2<u32> {
    return vec2<u32>(rd32(base, o), rd16(base, o + 4u));
}
fn rd64(base: u32, o: u32) -> vec2<u32> {
    return vec2<u32>(rd32(base, o), rd32(base, o + 4u));
}
fn all_zero(base: u32, o: u32, n: u32) -> bool {
    for (var i = 0u; i < n; i = i + 1u) {
        if (rd8(base, o + i) != 0u) {
            return false;
        }
    }
    return true;
}
fn bytes_equal(base_a: u32, oa: u32, base_b: u32, ob: u32, n: u32) -> bool {
    for (var i = 0u; i < n; i = i + 1u) {
        if (rd8(base_a, oa + i) != rd8(base_b, ob + i)) {
            return false;
        }
    }
    return true;
}

// ───────────────────────── u64 ─────────────────────────

fn lt64(a: vec2<u32>, b: vec2<u32>) -> bool {
    return a.y < b.y || (a.y == b.y && a.x < b.x);
}
fn le64(a: vec2<u32>, b: vec2<u32>) -> bool {
    return !lt64(b, a);
}
fn eq64(a: vec2<u32>, b: vec2<u32>) -> bool {
    return a.x == b.x && a.y == b.y;
}
fn is_zero64(a: vec2<u32>) -> bool {
    return a.x == 0u && a.y == 0u;
}
fn u64_of(a: u32) -> vec2<u32> {
    return vec2<u32>(a, 0u);
}
fn add64(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    let lo = a.x + b.x;
    var hi = a.y + b.y;
    if (lo < a.x) {
        hi = hi + 1u;
    }
    return vec2<u32>(lo, hi);
}
fn add64_overflows(a: vec2<u32>, b: vec2<u32>) -> bool {
    let s = add64(a, b);
    return lt64(s, a);
}
fn sub64(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    let lo = a.x - b.x;
    var hi = a.y - b.y;
    if (a.x < b.x) {
        hi = hi - 1u;
    }
    return vec2<u32>(lo, hi);
}
// 左移 s 位，s 在 1..31
fn shl64(a: vec2<u32>, s: u32) -> vec2<u32> {
    if (s == 0u) {
        return a;
    }
    return vec2<u32>(a.x << s, (a.y << s) | (a.x >> (32u - s)));
}
fn shr64(a: vec2<u32>, s: u32) -> vec2<u32> {
    if (s == 0u) {
        return a;
    }
    return vec2<u32>((a.x >> s) | (a.y << (32u - s)), a.y >> s);
}
fn mul32x32(a: u32, b: u32) -> vec2<u32> {
    let a_lo = a & 0xFFFFu;
    let a_hi = a >> 16u;
    let b_lo = b & 0xFFFFu;
    let b_hi = b >> 16u;
    let p0 = a_lo * b_lo;
    let p1 = a_lo * b_hi;
    let p2 = a_hi * b_lo;
    let p3 = a_hi * b_hi;
    let mid = (p0 >> 16u) + (p1 & 0xFFFFu) + (p2 & 0xFFFFu);
    let lo = (p0 & 0xFFFFu) | (mid << 16u);
    let hi = p3 + (p1 >> 16u) + (p2 >> 16u) + (mid >> 16u);
    return vec2<u32>(lo, hi);
}
struct Product64 {
    value: vec2<u32>,
    overflow: bool,
};
// a × b（b 是 u32）；溢出置标志（饱和用）
fn mul64_32(a: vec2<u32>, b: u32) -> Product64 {
    let low = mul32x32(a.x, b);
    let high = mul32x32(a.y, b);
    var out: Product64;
    out.overflow = high.y != 0u;
    let sum = add64(vec2<u32>(low.x, low.y), vec2<u32>(0u, high.x));
    if (lt64(sum, low)) {
        out.overflow = true;
    }
    out.value = sum;
    return out;
}
fn mul64_32_saturating(a: vec2<u32>, b: u32) -> vec2<u32> {
    let p = mul64_32(a, b);
    if (p.overflow) {
        return vec2<u32>(NONE, NONE);
    }
    return p.value;
}
fn add64_saturating(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    if (add64_overflows(a, b)) {
        return vec2<u32>(NONE, NONE);
    }
    return add64(a, b);
}
fn sub64_saturating(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    if (lt64(a, b)) {
        return vec2<u32>(0u, 0u);
    }
    return sub64(a, b);
}
struct QuotientRemainder {
    quotient: vec2<u32>,
    remainder: vec2<u32>,
};
// 长除法；除数为 0 时商与余数都是 0（调用方保证除数非 0）
fn divmod64(a: vec2<u32>, d: vec2<u32>) -> QuotientRemainder {
    var out: QuotientRemainder;
    out.quotient = vec2<u32>(0u, 0u);
    out.remainder = vec2<u32>(0u, 0u);
    if (is_zero64(d)) {
        return out;
    }
    for (var i = 0u; i < 64u; i = i + 1u) {
        let bit_index = 63u - i;
        var bit = 0u;
        if (bit_index >= 32u) {
            bit = (a.y >> (bit_index - 32u)) & 1u;
        } else {
            bit = (a.x >> bit_index) & 1u;
        }
        out.remainder = shl64(out.remainder, 1u);
        out.remainder.x = out.remainder.x | bit;
        if (le64(d, out.remainder)) {
            out.remainder = sub64(out.remainder, d);
            if (bit_index >= 32u) {
                out.quotient.y = out.quotient.y | (1u << (bit_index - 32u));
            } else {
                out.quotient.x = out.quotient.x | (1u << bit_index);
            }
        }
    }
    return out;
}
fn slot_to_bytes(slot: vec2<u32>) -> vec2<u32> {
    return shl64(slot, SLOT_SHIFT);
}

// ───────────────────────── 落点、版本、读 ─────────────────────────

fn location_word(location: u32, k: u32) -> u32 {
    return tab(T_LOCATIONS, location * LOCATION_WORDS + k);
}
fn find_location(device: u32, offset: vec2<u32>) -> u32 {
    var lo = 0u;
    var hi = hdr(6u);
    while (lo < hi) {
        let mid = (lo + hi) >> 1u;
        let d = location_word(mid, 0u);
        let o = vec2<u32>(location_word(mid, 1u), location_word(mid, 2u));
        if (d < device || (d == device && lt64(o, offset))) {
            lo = mid + 1u;
        } else {
            hi = mid;
        }
    }
    if (lo < hdr(6u) && location_word(lo, 0u) == device) {
        let o = vec2<u32>(location_word(lo, 1u), location_word(lo, 2u));
        if (eq64(o, offset)) {
            return lo;
        }
    }
    return NONE;
}
fn version_of_location(location: u32) -> u32 {
    let writes_first = location_word(location, 5u);
    let writes_count = location_word(location, 6u);
    // 组合掩码 64 位：第 i 次写在低字的第 i 位（i < 32）或高字的第 i − 32 位；版本表里第 0 字是低字、第 7 字是高字
    var combination = vec2<u32>(0u, 0u);
    for (var i = 0u; i < writes_count; i = i + 1u) {
        if (is_persisted(tab(T_LOCATION_WRITES, writes_first + i))) {
            if (i < 32u) {
                combination.x = combination.x | (1u << i);
            } else {
                combination.y = combination.y | (1u << (i - 32u));
            }
        }
    }
    let versions_first = location_word(location, 7u);
    let versions_count = location_word(location, 8u);
    for (var i = 0u; i < versions_count; i = i + 1u) {
        let version_words = (versions_first + i) * VERSION_WORDS;
        if (tab(T_VERSIONS, version_words) == combination.x && tab(T_VERSIONS, version_words + 7u) == combination.y) {
            return versions_first + i;
        }
    }
    undecidable = UNDECIDABLE_NO_VERSION;
    return NONE;
}
struct Slice {
    version: u32,
    base: u32,
    length: u32,
    c0: u32,
    c1: u32,
    c2: u32,
    c3: u32,
};
fn no_slice() -> Slice {
    var s: Slice;
    s.version = NONE;
    s.base = 0u;
    s.length = 0u;
    s.c0 = 0u;
    s.c1 = 0u;
    s.c2 = 0u;
    s.c3 = 0u;
    return s;
}
fn slice_of_version(version: u32, location: u32) -> Slice {
    var s: Slice;
    s.version = version;
    s.base = B[T_VERSION_BYTES] + tab(T_VERSIONS, version * VERSION_WORDS + 1u);
    s.length = location_word(location, 3u);
    s.c0 = tab(T_VERSIONS, version * VERSION_WORDS + 2u);
    s.c1 = tab(T_VERSIONS, version * VERSION_WORDS + 3u);
    s.c2 = tab(T_VERSIONS, version * VERSION_WORDS + 4u);
    s.c3 = tab(T_VERSIONS, version * VERSION_WORDS + 5u);
    return s;
}
// 读 (盘, 偏移, 长度)：起点是落点起点、长度不超过落点的才读得到
fn read_at(device: u32, offset: vec2<u32>, length: u32) -> Slice {
    let location = find_location(device, offset);
    if (location == NONE) {
        return no_slice();
    }
    if (length > location_word(location, 3u)) {
        return no_slice();
    }
    let version = version_of_location(location);
    if (version == NONE) {
        return no_slice();
    }
    return slice_of_version(version, location);
}
fn read_location(location: u32) -> Slice {
    let version = version_of_location(location);
    if (version == NONE) {
        return no_slice();
    }
    return slice_of_version(version, location);
}
fn device_is_in_the_pool(device: u32) -> bool {
    let count = hdr(3u);
    for (var i = 0u; i < count; i = i + 1u) {
        if (hdr(12u + i) == device) {
            return true;
        }
    }
    return false;
}
fn device_bytes() -> vec2<u32> {
    return vec2<u32>(hdr(4u), hdr(5u));
}

// ───────────────────────── 指针 ─────────────────────────

struct Ptr {
    all_zero: bool,
    head_ok: bool,
    mac_nonce_zero: bool,
    birth_tree: vec2<u32>,
    birth_txg: vec2<u32>,
    device0: u32,
    slot0: vec2<u32>,
    sum0: u32,
    device1: u32,
    slot1: vec2<u32>,
    sum1: u32,
    instance: u32,
    tail: vec2<u32>,
};
fn parse_pointer_common(base: u32, o: u32) -> Ptr {
    var p: Ptr;
    p.all_zero = false;
    p.mac_nonce_zero = all_zero(base, o, 28u);
    p.head_ok = p.mac_nonce_zero && rd8(base, o + 28u) == 0u && rd8(base, o + 29u) == 0u && rd16(base, o + 30u) == 0u;
    p.birth_tree = rd64(base, o + 34u);
    p.birth_txg = rd64(base, o + 42u);
    p.device0 = rd32(base, o + 50u);
    p.slot0 = rd48(base, o + 54u);
    p.sum0 = rd32(base, o + 60u);
    p.device1 = rd32(base, o + 64u);
    p.slot1 = rd48(base, o + 68u);
    p.sum1 = rd32(base, o + 74u);
    p.instance = rd32(base, o + 78u);
    p.tail = vec2<u32>(0u, 0u);
    return p;
}
fn parse_node_pointer(base: u32, o: u32) -> Ptr {
    var p = parse_pointer_common(base, o);
    p.tail = vec2<u32>(rd32(base, o + 82u), 0u);
    p.all_zero = all_zero(base, o, 86u);
    return p;
}
fn parse_data_pointer(base: u32, o: u32) -> Ptr {
    var p = parse_pointer_common(base, o);
    p.tail = rd48(base, o + 82u);
    p.all_zero = all_zero(base, o, 88u);
    return p;
}
fn same_pointer(a: Ptr, b: Ptr) -> bool {
    return a.device0 == b.device0 && eq64(a.slot0, b.slot0) && a.sum0 == b.sum0
        && a.device1 == b.device1 && eq64(a.slot1, b.slot1) && a.sum1 == b.sum1
        && a.instance == b.instance && eq64(a.tail, b.tail)
        && eq64(a.birth_tree, b.birth_tree) && eq64(a.birth_txg, b.birth_txg);
}
// 位置条目里的整单元校验和对得上这一读的 CRC 吗
fn slice_crc_matches(s: Slice, unit_bytes: u32, checksum: u32) -> bool {
    if (unit_bytes == NODE_BYTES) {
        return s.c0 == checksum;
    }
    if ((s.c2 & 4u) == 0u) {
        return false;
    }
    return s.c1 == checksum;
}
// 按两条位置条目读一个单元（recovery::read_unit_via_locations）
fn read_unit_via_locations(p: Ptr, unit_bytes: u32) -> Slice {
    let s0 = read_at(p.device0, slot_to_bytes(p.slot0), unit_bytes);
    if (s0.version != NONE && slice_crc_matches(s0, unit_bytes, p.sum0)) {
        return s0;
    }
    let s1 = read_at(p.device1, slot_to_bytes(p.slot1), unit_bytes);
    if (s1.version != NONE && slice_crc_matches(s1, unit_bytes, p.sum1)) {
        return s1;
    }
    return no_slice();
}

// ───────────────────────── 系统配置 ─────────────────────────

struct SysConfig {
    kind: u32,          // 0 可择、1 自证不过、2 incompat 不认识、3 整池拒
    refusal: u32,
    fsid: vec4<u32>,
    this_device: u32,
    device_count: u32,
    generation: vec2<u32>,
    physical_block_size: u32,
    spacing: u32,
    slots_per_region: u32,
    region0: u32,
    region1: u32,
    region2: u32,
    ring_bytes: vec2<u32>,
    floor: vec2<u32>,
    unit_area_start_slot: vec2<u32>,
};
fn empty_system_configuration(kind: u32, refusal: u32) -> SysConfig {
    var c: SysConfig;
    c.kind = kind;
    c.refusal = refusal;
    c.fsid = vec4<u32>(0u, 0u, 0u, 0u);
    c.this_device = 0u;
    c.device_count = 0u;
    c.generation = vec2<u32>(0u, 0u);
    c.physical_block_size = 0u;
    c.spacing = 0u;
    c.slots_per_region = 0u;
    c.region0 = 0u;
    c.region1 = 0u;
    c.region2 = 0u;
    c.ring_bytes = vec2<u32>(0u, 0u);
    c.floor = vec2<u32>(0u, 0u);
    c.unit_area_start_slot = vec2<u32>(0u, 0u);
    return c;
}
fn parse_system_configuration(s: Slice) -> SysConfig {
    if (s.version == NONE || s.length < SYSTEM_CONFIGURATION_SLOT_BYTES) {
        return empty_system_configuration(1u, 0u);
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSB || s.c0 != 1u) {
        return empty_system_configuration(1u, 0u);
    }
    let incompat0 = rd8(base, 6u);
    if ((incompat0 & 0xFDu) != 0u || !all_zero(base, 7u, 31u) || (incompat0 & 2u) == 0u) {
        return empty_system_configuration(2u, 0u);
    }
    let slots_per_region = rd8(base, 362u);
    if (slots_per_region < 4u || slots_per_region > 16u) {
        return empty_system_configuration(3u, 1u);
    }
    // 读者收不收的池级值（recovery::system_configuration_values_this_reader_accepts）
    if (rd16(base, 4u) != 1u) {
        return empty_system_configuration(3u, 2u);
    }
    if (rd8(base, 219u) != 0u) {
        return empty_system_configuration(3u, 3u);
    }
    let spacing = rd32(base, 429u);
    let root_ring_base_slot = rd64(base, 371u);
    var largest_spacing = vec2<u32>(NONE, NONE);
    if (root_ring_base_slot.y < (1u << 18u)) {
        let base_bytes = slot_to_bytes(root_ring_base_slot);
        if (lt64(base_bytes, u64_of(SYSTEM_CONFIGURATION_SLOT_BYTES))) {
            return empty_system_configuration(3u, 4u);
        }
        largest_spacing = sub64(base_bytes, u64_of(SYSTEM_CONFIGURATION_SLOT_BYTES));
    }
    if (spacing < 4096u || lt64(largest_spacing, u64_of(spacing))) {
        return empty_system_configuration(3u, 4u);
    }
    let physical_block_size = rd32(base, 317u);
    if (physical_block_size < ROOT_RECORD_BYTES || physical_block_size > spacing) {
        return empty_system_configuration(3u, 5u);
    }
    if (!eq64(root_ring_base_slot, u64_of(64u))) {
        return empty_system_configuration(3u, 6u);
    }
    if (!eq64(rd64(base, 325u), u64_of(1024u))) {
        return empty_system_configuration(3u, 7u);
    }
    let ring_bytes = rd64(base, 333u);
    let in_flight = divmod64(shr64(ring_bytes, 12u), u64_of(3u)).quotient;
    if (is_zero64(in_flight) || in_flight.y != 0u) {
        return empty_system_configuration(3u, 8u);
    }
    let ring_start = u64_of(JOURNAL_RING_START_BYTES);
    if (add64_overflows(ring_start, ring_bytes)) {
        return empty_system_configuration(3u, 8u);
    }
    let ring_end = add64(ring_start, ring_bytes);
    let unit_area_start_slot = rd64(base, 417u);
    if (unit_area_start_slot.y < (1u << 18u)) {
        if (lt64(slot_to_bytes(unit_area_start_slot), ring_end)) {
            return empty_system_configuration(3u, 8u);
        }
    }
    let slot_after_the_ring = add64(u64_of(1024u), shr64(add64(ring_bytes, u64_of(16383u)), SLOT_SHIFT));
    if (!eq64(unit_area_start_slot, slot_after_the_ring)) {
        return empty_system_configuration(3u, 9u);
    }
    if ((slot_after_the_ring.x & 63u) != 0u) {
        return empty_system_configuration(3u, 10u);
    }
    var c = empty_system_configuration(0u, 0u);
    c.fsid = vec4<u32>(rd32(base, 102u), rd32(base, 106u), rd32(base, 110u), rd32(base, 114u));
    c.this_device = rd32(base, 139u);
    c.device_count = rd32(base, 143u);
    c.generation = rd64(base, 147u);
    c.physical_block_size = physical_block_size;
    c.spacing = spacing;
    c.slots_per_region = slots_per_region;
    c.region0 = rd32(base, 379u);
    c.region1 = rd32(base, 383u);
    c.region2 = rd32(base, 387u);
    c.ring_bytes = ring_bytes;
    c.floor = rd64(base, 481u);
    c.unit_area_start_slot = unit_area_start_slot;
    return c;
}
// 全池没有一个槽 0 可择时逐档找槽 1
fn slot_one_by_trying_every_spacing(device: u32) -> SysConfig {
    var best = empty_system_configuration(1u, 0u);
    var have_best = false;
    var incompat = empty_system_configuration(1u, 0u);
    var have_incompat = false;
    let largest = ROOT_RING_BASE_BYTES - SYSTEM_CONFIGURATION_SLOT_BYTES;
    for (var candidate = 4096u; candidate <= largest; candidate = candidate + 512u) {
        let s = parse_system_configuration(read_at(device, u64_of(candidate), SYSTEM_CONFIGURATION_SLOT_BYTES));
        if (s.kind == 3u) {
            return s;
        }
        if (s.kind == 0u) {
            if (s.spacing != candidate) {
                continue;
            }
            if (!have_best || lt64(best.generation, s.generation)) {
                best = s;
                have_best = true;
            }
        } else if (s.kind == 2u && !have_incompat) {
            incompat = s;
            have_incompat = true;
        }
    }
    if (have_best) {
        return best;
    }
    if (have_incompat) {
        return incompat;
    }
    return empty_system_configuration(1u, 0u);
}
struct SysChoice {
    ok: bool,
    failure: u32,
    cfg: SysConfig,
};
fn choose_system_configuration() -> SysChoice {
    var out: SysChoice;
    out.ok = false;
    out.failure = 0u;
    out.cfg = empty_system_configuration(1u, 0u);
    let device_count = hdr(3u);
    if (device_count > MAX_DEVICES) {
        undecidable = UNDECIDABLE_TOO_MANY_DEVICES;
        return out;
    }
    var slot_zero: array<SysConfig, 8>;
    var first_spacing = NONE;
    for (var d = 0u; d < device_count; d = d + 1u) {
        let device = hdr(12u + d);
        slot_zero[d] = parse_system_configuration(read_at(device, u64_of(0u), SYSTEM_CONFIGURATION_SLOT_BYTES));
        if (slot_zero[d].kind == 3u) {
            out.failure = 100u + slot_zero[d].refusal;
            return out;
        }
        if (slot_zero[d].kind == 0u && first_spacing == NONE) {
            first_spacing = slot_zero[d].spacing;
        }
    }
    var chosen_valid = false;
    var first_no_valid = false;
    var first_incompat = false;
    for (var d = 0u; d < device_count; d = d + 1u) {
        let device = hdr(12u + d);
        var spacing = first_spacing;
        if (slot_zero[d].kind == 0u) {
            spacing = slot_zero[d].spacing;
        }
        var slot_one: SysConfig;
        if (spacing != NONE) {
            slot_one = parse_system_configuration(read_at(device, u64_of(spacing), SYSTEM_CONFIGURATION_SLOT_BYTES));
        } else {
            slot_one = slot_one_by_trying_every_spacing(device);
        }
        if (slot_one.kind == 3u) {
            out.failure = 100u + slot_one.refusal;
            return out;
        }
        if (!first_incompat && (slot_zero[d].kind == 2u || slot_one.kind == 2u)) {
            first_incompat = true;
        }
        var best: SysConfig;
        if (slot_zero[d].kind != 0u && slot_one.kind != 0u) {
            first_no_valid = true;
            continue;
        } else if (slot_zero[d].kind == 0u && slot_one.kind != 0u) {
            best = slot_zero[d];
        } else if (slot_zero[d].kind != 0u && slot_one.kind == 0u) {
            best = slot_one;
        } else {
            if (lt64(slot_zero[d].generation, slot_one.generation)) {
                best = slot_one;
            } else {
                best = slot_zero[d];
            }
        }
        if (!chosen_valid) {
            out.cfg = best;
            chosen_valid = true;
        } else if (!(out.cfg.fsid.x == best.fsid.x && out.cfg.fsid.y == best.fsid.y && out.cfg.fsid.z == best.fsid.z && out.cfg.fsid.w == best.fsid.w)
            || out.cfg.device_count != best.device_count) {
            out.failure = 3u;
            return out;
        }
    }
    if (chosen_valid) {
        out.ok = true;
        return out;
    }
    if (first_incompat) {
        out.failure = 2u;
    } else if (first_no_valid) {
        out.failure = 1u;
    } else {
        out.failure = 3u;
    }
    return out;
}

// ───────────────────────── 根 ─────────────────────────

struct RootFields {
    valid: bool,
    instance: u32,
    txg: vec2<u32>,
    watermark: vec2<u32>,
    floor: vec2<u32>,
    unmount: bool,
    tree_table: Ptr,
    instance_table: Ptr,
    mapping_root: Ptr,
    allocation_root: Ptr,
};
fn zero_pointer() -> Ptr {
    var p: Ptr;
    p.all_zero = true;
    p.head_ok = true;
    p.mac_nonce_zero = true;
    p.birth_tree = vec2<u32>(0u, 0u);
    p.birth_txg = vec2<u32>(0u, 0u);
    p.device0 = 0u;
    p.slot0 = vec2<u32>(0u, 0u);
    p.sum0 = 0u;
    p.device1 = 0u;
    p.slot1 = vec2<u32>(0u, 0u);
    p.sum1 = 0u;
    p.instance = 0u;
    p.tail = vec2<u32>(0u, 0u);
    return p;
}
fn invalid_root() -> RootFields {
    var r: RootFields;
    r.valid = false;
    r.instance = 0u;
    r.txg = vec2<u32>(0u, 0u);
    r.watermark = vec2<u32>(0u, 0u);
    r.floor = vec2<u32>(0u, 0u);
    r.unmount = false;
    r.tree_table = zero_pointer();
    r.instance_table = zero_pointer();
    r.mapping_root = zero_pointer();
    r.allocation_root = zero_pointer();
    return r;
}
fn fsid_matches(base: u32, o: u32, fsid: vec4<u32>) -> bool {
    return rd32(base, o) == fsid.x && rd32(base, o + 4u) == fsid.y && rd32(base, o + 8u) == fsid.z && rd32(base, o + 12u) == fsid.w;
}
fn parse_root(s: Slice, fsid: vec4<u32>) -> RootFields {
    if (s.version == NONE || s.length < ROOT_RECORD_BYTES) {
        return invalid_root();
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSR || s.c0 != 1u || !fsid_matches(base, 4u, fsid)) {
        return invalid_root();
    }
    let flags = rd32(base, 20u);
    if ((flags & 0xFFFFFFFEu) != 0u) {
        return invalid_root();
    }
    var r: RootFields;
    r.valid = true;
    r.unmount = (flags & 1u) != 0u;
    r.instance = rd32(base, 24u);
    r.txg = rd64(base, 28u);
    r.tree_table = parse_node_pointer(base, 36u);
    r.watermark = rd64(base, 122u);
    r.floor = rd64(base, 130u);
    r.instance_table = parse_node_pointer(base, 170u);
    r.mapping_root = parse_node_pointer(base, 256u);
    r.allocation_root = parse_node_pointer(base, 342u);
    return r;
}
fn region_device(cfg: SysConfig, region: u32) -> u32 {
    if (region == 0u) {
        return cfg.region0;
    }
    if (region == 1u) {
        return cfg.region1;
    }
    return cfg.region2;
}
struct RootChoice {
    found: bool,
    root: RootFields,
};
// 三个区域全部槽逐个验自证，取 (txg, 实例) 最大的；每条自证过的根记进草稿（版本、区域）
fn choose_root(cfg: SysConfig) -> RootChoice {
    var out: RootChoice;
    out.found = false;
    out.root = invalid_root();
    root_count = 0u;
    for (var region = 0u; region < 3u; region = region + 1u) {
        let device = region_device(cfg, region);
        for (var slot = 0u; slot < cfg.slots_per_region; slot = slot + 1u) {
            let offset = u64_of(ROOT_RING_BASE_BYTES + region * ROOT_RING_REGION_STRIDE + slot * cfg.spacing);
            let s = read_at(device, offset, cfg.physical_block_size);
            let candidate = parse_root(s, cfg.fsid);
            if (!candidate.valid) {
                continue;
            }
            if (root_count < MAX_ROOTS) {
                sws(R_ROOTS + root_count * 2u, s.version);
                sws(R_ROOTS + root_count * 2u + 1u, region);
                root_count = root_count + 1u;
            } else {
                undecidable = UNDECIDABLE_TOO_MANY_ROOTS;
            }
            if (!out.found || lt64(out.root.txg, candidate.txg)
                || (eq64(out.root.txg, candidate.txg) && out.root.instance < candidate.instance)) {
                out.root = candidate;
                out.found = true;
            }
        }
    }
    return out;
}

// ───────────────────────── journal ─────────────────────────

struct Rec {
    valid: bool,
    base: u32,
    instance: u32,
    counter: vec2<u32>,
    txg: vec2<u32>,
    transaction: vec2<u32>,
    commit: bool,
    ordinal: u32,
    last: bool,
    back_chain: u32,
    named_count: u32,
};
fn invalid_record() -> Rec {
    var r: Rec;
    r.valid = false;
    r.base = 0u;
    r.instance = 0u;
    r.counter = vec2<u32>(0u, 0u);
    r.txg = vec2<u32>(0u, 0u);
    r.transaction = vec2<u32>(0u, 0u);
    r.commit = false;
    r.ordinal = 0u;
    r.last = false;
    r.back_chain = 0u;
    r.named_count = 0u;
    return r;
}
// journal::JournalRecord::parse：magic、类型、整条校验和、标志、长度、提交标记、序号、新根段两条指针头部、fsid、载荷校验和
fn parse_record(s: Slice, fsid_low: vec2<u32>) -> Rec {
    if (s.version == NONE || s.length < JOURNAL_RECORD_BYTES) {
        return invalid_record();
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSJ || s.c0 != 1u || rd16(base, 4u) != 1u) {
        return invalid_record();
    }
    let flags = rd8(base, 7u);
    if (flags > 1u) {
        return invalid_record();
    }
    if (rd32(base, 8u) != JOURNAL_RECORD_BYTES) {
        return invalid_record();
    }
    let commit = rd8(base, 86u);
    if (commit > 1u) {
        return invalid_record();
    }
    let ordinal = rd32(base, 87u);
    if (ordinal == 0u) {
        return invalid_record();
    }
    let tree_table = parse_node_pointer(base, 99u);
    let mapping_root = parse_node_pointer(base, 185u);
    if (!tree_table.head_ok || !mapping_root.head_ok) {
        return invalid_record();
    }
    if (!eq64(rd64(base, 287u), fsid_low)) {
        return invalid_record();
    }
    if (s.c1 != 1u) {
        return invalid_record();
    }
    var r: Rec;
    r.valid = true;
    r.base = base;
    r.instance = rd32(base, 16u);
    r.counter = rd48(base, 20u);
    r.txg = rd64(base, 26u);
    r.transaction = rd64(base, 78u);
    r.commit = commit == 1u;
    r.ordinal = ordinal;
    r.last = flags == 1u;
    r.back_chain = rd32(base, 91u);
    r.named_count = rd32(base, 12u);
    return r;
}
fn record_at(index: u32) -> Rec {
    let version = sw(R_RECORDS + index * RECORD_WORDS);
    var r: Rec;
    r.valid = true;
    r.base = B[T_VERSION_BYTES] + tab(T_VERSIONS, version * VERSION_WORDS + 1u);
    r.instance = sw(R_RECORDS + index * RECORD_WORDS + 1u);
    r.counter = vec2<u32>(sw(R_RECORDS + index * RECORD_WORDS + 2u), sw(R_RECORDS + index * RECORD_WORDS + 3u));
    r.txg = vec2<u32>(sw(R_RECORDS + index * RECORD_WORDS + 4u), sw(R_RECORDS + index * RECORD_WORDS + 5u));
    r.transaction = rd64(r.base, 78u);
    r.commit = rd8(r.base, 86u) == 1u;
    r.ordinal = rd32(r.base, 87u);
    r.last = (rd8(r.base, 7u) & 1u) == 1u;
    r.back_chain = rd32(r.base, 91u);
    r.named_count = rd32(r.base, 12u);
    return r;
}
fn record_version_at(index: u32) -> u32 {
    return sw(R_RECORDS + index * RECORD_WORDS);
}
fn record_key_less(instance_a: u32, counter_a: vec2<u32>, instance_b: u32, counter_b: vec2<u32>) -> bool {
    return instance_a < instance_b || (instance_a == instance_b && lt64(counter_a, counter_b));
}
// 按 (实例代号, 计数器) 排序插入；已有同键的不换（BTreeMap::or_insert：先读到的留下）
fn insert_record(version: u32, r: Rec) {
    var position = 0u;
    while (position < record_count) {
        let instance = sw(R_RECORDS + position * RECORD_WORDS + 1u);
        let counter = vec2<u32>(sw(R_RECORDS + position * RECORD_WORDS + 2u), sw(R_RECORDS + position * RECORD_WORDS + 3u));
        if (instance == r.instance && eq64(counter, r.counter)) {
            return;
        }
        if (record_key_less(r.instance, r.counter, instance, counter)) {
            break;
        }
        position = position + 1u;
    }
    if (record_count >= MAX_RECORDS) {
        undecidable = UNDECIDABLE_TOO_MANY_RECORDS;
        return;
    }
    var i = record_count;
    while (i > position) {
        for (var k = 0u; k < RECORD_WORDS; k = k + 1u) {
            sws(R_RECORDS + i * RECORD_WORDS + k, sw(R_RECORDS + (i - 1u) * RECORD_WORDS + k));
        }
        i = i - 1u;
    }
    sws(R_RECORDS + position * RECORD_WORDS, version);
    sws(R_RECORDS + position * RECORD_WORDS + 1u, r.instance);
    sws(R_RECORDS + position * RECORD_WORDS + 2u, r.counter.x);
    sws(R_RECORDS + position * RECORD_WORDS + 3u, r.counter.y);
    sws(R_RECORDS + position * RECORD_WORDS + 4u, r.txg.x);
    sws(R_RECORDS + position * RECORD_WORDS + 5u, r.txg.y);
    record_count = record_count + 1u;
}
// recovery::scan_journal：每块盘按次序、环里每个记录槽按偏移升序
fn scan_journal(cfg: SysConfig) {
    record_count = 0u;
    let fsid_low = vec2<u32>(cfg.fsid.x, cfg.fsid.y);
    let device_count = hdr(3u);
    let location_count = hdr(6u);
    for (var d = 0u; d < device_count; d = d + 1u) {
        let device = hdr(12u + d);
        for (var l = 0u; l < location_count; l = l + 1u) {
            if (location_word(l, 0u) != device || location_word(l, 4u) != LOCATION_JOURNAL_RECORD) {
                continue;
            }
            let s = read_location(l);
            let r = parse_record(s, fsid_low);
            if (r.valid) {
                insert_record(s.version, r);
            }
        }
    }
}
fn find_record(instance: u32, counter: vec2<u32>) -> u32 {
    for (var i = 0u; i < record_count; i = i + 1u) {
        if (sw(R_RECORDS + i * RECORD_WORDS + 1u) == instance
            && sw(R_RECORDS + i * RECORD_WORDS + 2u) == counter.x
            && sw(R_RECORDS + i * RECORD_WORDS + 3u) == counter.y) {
            return i;
        }
    }
    return NONE;
}
// 两份记录解出来相等（磁盘上另一份与扫环时收下的那一条比）：比解析读的那几段
fn records_parse_equal(base_a: u32, base_b: u32) -> bool {
    if (base_a == base_b) {
        return true;
    }
    if (!bytes_equal(base_a, 4u, base_b, 4u, 34u)) {
        return false;
    }
    if (!bytes_equal(base_a, 78u, base_b, 78u, 21u)) {
        return false;
    }
    if (!bytes_equal(base_a, 99u, base_b, 99u, 196u)) {
        return false;
    }
    let named = rd32(base_a, 12u);
    return bytes_equal(base_a, 311u, base_b, 311u, named * JOURNAL_NAMED_ENTRY_BYTES);
}
// 反向链判定：0 成立、1 断、2 前一条不在盘上
fn judge_back_chain(r: Rec, ring_bytes: vec2<u32>) -> u32 {
    if (is_zero64(r.counter) || eq64(r.counter, u64_of(1u))) {
        return 2u;
    }
    let previous_counter = sub64(r.counter, u64_of(1u));
    let previous_index = find_record(r.instance, previous_counter);
    if (previous_index == NONE) {
        return 2u;
    }
    let previous_version = record_version_at(previous_index);
    let previous_base = B[T_VERSION_BYTES] + tab(T_VERSIONS, previous_version * VERSION_WORDS + 1u);
    let ring_slots = shr64(ring_bytes, 12u);
    let slot_in_ring = divmod64(sub64(previous_counter, u64_of(1u)), ring_slots).remainder;
    let offset = add64(u64_of(JOURNAL_RING_START_BYTES), shl64(slot_in_ring, 12u));
    let device_count = hdr(3u);
    let fsid_low = rd64(previous_base, 287u);
    for (var d = 0u; d < device_count; d = d + 1u) {
        let s = read_at(hdr(12u + d), offset, JOURNAL_RECORD_BYTES);
        if (s.version == NONE) {
            continue;
        }
        let candidate = parse_record(s, fsid_low);
        if (!candidate.valid || !records_parse_equal(s.base, previous_base)) {
            continue;
        }
        if (s.c2 == r.back_chain) {
            return 0u;
        }
        return 1u;
    }
    return 2u;
}
fn readable_record_of_the_same_publish_follows(r: Rec, index: u32) -> bool {
    var i = index + 1u;
    while (i < record_count) {
        let instance = sw(R_RECORDS + i * RECORD_WORDS + 1u);
        if (instance != r.instance) {
            break;
        }
        let txg = vec2<u32>(sw(R_RECORDS + i * RECORD_WORDS + 4u), sw(R_RECORDS + i * RECORD_WORDS + 5u));
        if (lt64(r.txg, txg)) {
            break;
        }
        if (eq64(txg, r.txg)) {
            return true;
        }
        i = i + 1u;
    }
    return false;
}
fn named_units_verified(r: Rec) -> bool {
    for (var i = 0u; i < r.named_count; i = i + 1u) {
        let o = JOURNAL_HEADER_BYTES + i * JOURNAL_NAMED_ENTRY_BYTES;
        let unit_class = rd8(r.base, o + 28u);
        var unit_bytes = 0u;
        if (unit_class == 1u || unit_class == 3u) {
            unit_bytes = DATA_UNIT_BYTES;
        } else if (unit_class == 2u) {
            unit_bytes = NODE_BYTES;
        } else {
            return false;
        }
        let s0 = read_at(rd32(r.base, o), slot_to_bytes(rd48(r.base, o + 4u)), unit_bytes);
        if (s0.version == NONE || !slice_crc_matches(s0, unit_bytes, rd32(r.base, o + 10u))) {
            return false;
        }
        let s1 = read_at(rd32(r.base, o + 14u), slot_to_bytes(rd48(r.base, o + 18u)), unit_bytes);
        if (s1.version == NONE || !slice_crc_matches(s1, unit_bytes, rd32(r.base, o + 24u))) {
            return false;
        }
    }
    return true;
}
struct Replay {
    ok: bool,
    failure: u32,
    root: RootFields,
};
// recovery::replay_journal（Consult：验点名单元）
fn replay_journal(chosen: RootFields, ring_bytes: vec2<u32>) -> Replay {
    var out: Replay;
    out.ok = false;
    out.failure = 0u;
    out.root = chosen;
    // 锚点：所选根那次发布里带末条标志的那一条
    var anchor_count = 0u;
    var anchor_counter = vec2<u32>(0u, 0u);
    for (var i = 0u; i < record_count; i = i + 1u) {
        let r = record_at(i);
        if (r.instance == chosen.instance && eq64(r.txg, chosen.txg) && r.last) {
            anchor_count = anchor_count + 1u;
            anchor_counter = r.counter;
        }
    }
    if (anchor_count > 1u) {
        out.failure = 3u;
        return out;
    }
    let has_anchor = anchor_count == 1u;
    if (chosen.txg.x == NONE && chosen.txg.y == NONE) {
        out.failure = 4u;
        return out;
    }
    let chain_start_txg = add64(chosen.txg, u64_of(1u));
    var expected_counter = add64(anchor_counter, u64_of(1u));
    var have_expected = has_anchor;
    let limit64 = divmod64(shr64(ring_bytes, 12u), u64_of(3u)).quotient;
    var limit = limit64.x;
    if (limit64.y != 0u) {
        limit = NONE;
    }
    var taken = 0u;
    var open_count = 0u;
    var previous = invalid_record();
    var rebuilt = chosen;
    replay_stop_reason = 0u;
    for (var i = 0u; i < record_count; i = i + 1u) {
        let r = record_at(i);
        if (r.instance != chosen.instance || !lt64(chosen.txg, r.txg)) {
            continue;
        }
        if (taken >= limit) {
            replay_stop_reason = 1u;
            break;
        }
        taken = taken + 1u;
        if (have_expected) {
            if (!eq64(r.counter, expected_counter)) {
                replay_stop_reason = 2u;
                break;
            }
        } else if (!eq64(r.txg, chain_start_txg) || r.ordinal != 1u) {
            replay_stop_reason = 3u;
            break;
        }
        if (judge_back_chain(r, ring_bytes) == 1u) {
            replay_stop_reason = 4u;
            break;
        }
        if (open_count == 0u && r.ordinal != 1u) {
            replay_stop_reason = 5u;
            break;
        }
        if (open_count > 0u) {
            if (!eq64(r.txg, previous.txg)) {
                replay_stop_reason = 6u;
                break;
            }
            if (r.ordinal != previous.ordinal + 1u) {
                replay_stop_reason = 7u;
                break;
            }
            if (!previous.commit && !eq64(r.transaction, previous.transaction)) {
                replay_stop_reason = 8u;
                break;
            }
        }
        expected_counter = add64(r.counter, u64_of(1u));
        have_expected = true;
        if (!r.commit && r.last) {
            replay_stop_reason = 9u;
            break;
        }
        if (r.last && readable_record_of_the_same_publish_follows(r, i)) {
            replay_stop_reason = 10u;
            break;
        }
        if (!named_units_verified(r)) {
            replay_stop_reason = 11u;
            break;
        }
        open_count = open_count + 1u;
        previous = r;
        if (!r.last) {
            continue;
        }
        open_count = 0u;
        var next = rebuilt;
        next.valid = true;
        next.unmount = false;
        next.instance = r.instance;
        next.txg = r.txg;
        next.tree_table = parse_node_pointer(r.base, 99u);
        next.watermark = rd64(r.base, 271u);
        next.floor = rd64(r.base, 279u);
        next.mapping_root = parse_node_pointer(r.base, 185u);
        rebuilt = next;
        replay_stop_reason = 12u;
    }
    out.ok = true;
    out.root = rebuilt;
    return out;
}

// ───────────────────────── 单元解析 ─────────────────────────

struct Node {
    ok: bool,
    base: u32,
    tree: vec2<u32>,
    level: u32,
    key_width: u32,
    smallest_offset: u32,
    largest_offset: u32,
    birth_txg: vec2<u32>,
    fsid: vec2<u32>,
    instance: u32,
    birth_sequence: u32,
    entry_count: u32,
    entry_width: u32,
    entries_start: u32,
};
fn bad_node() -> Node {
    var n: Node;
    n.ok = false;
    n.base = 0u;
    n.tree = vec2<u32>(0u, 0u);
    n.level = 0u;
    n.key_width = 0u;
    n.smallest_offset = 0u;
    n.largest_offset = 0u;
    n.birth_txg = vec2<u32>(0u, 0u);
    n.fsid = vec2<u32>(0u, 0u);
    n.instance = 0u;
    n.birth_sequence = 0u;
    n.entry_count = 0u;
    n.entry_width = 0u;
    n.entries_start = 0u;
    return n;
}
// unit::parse_index_node（读 16 KiB）
fn parse_index_node(s: Slice) -> Node {
    if (s.version == NONE || s.length < NODE_BYTES) {
        return bad_node();
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSU || rd16(base, 4u) != 1u || rd8(base, 6u) != 2u || rd8(base, 7u) != 0u) {
        return bad_node();
    }
    if ((s.c2 & 11u) != 11u) {
        return bad_node();
    }
    let key_width = rd8(base, 51u);
    let header_end = 86u + 2u * key_width;
    let declared_length = rd16(base, 8u);
    let entry_count = rd16(base, header_end - 4u);
    let entry_width = rd16(base, header_end - 2u);
    if (entry_count * entry_width != declared_length) {
        return bad_node();
    }
    if (entry_width == 0u && entry_count != 0u) {
        return bad_node();
    }
    if (entry_width < key_width) {
        return bad_node();
    }
    let entries_start = header_end + 29u;
    if (entries_start + declared_length > NODE_BYTES) {
        return bad_node();
    }
    var n: Node;
    n.ok = true;
    n.base = base;
    n.tree = rd64(base, 42u);
    n.level = rd8(base, 50u);
    n.key_width = key_width;
    n.smallest_offset = 52u;
    n.largest_offset = 52u + key_width;
    n.birth_txg = rd64(base, 52u + 2u * key_width);
    n.fsid = rd64(base, 60u + 2u * key_width);
    n.instance = rd32(base, 68u + 2u * key_width);
    n.birth_sequence = rd32(base, 72u + 2u * key_width);
    n.entry_count = entry_count;
    n.entry_width = entry_width;
    n.entries_start = entries_start;
    return n;
}
fn entry_offset(n: Node, index: u32) -> u32 {
    return n.entries_start + index * n.entry_width;
}
struct Packed {
    ok: bool,
    base: u32,
    birth_tree: vec2<u32>,
    record_type: u32,
    container: vec2<u32>,
    container_birth: vec2<u32>,
    record_count: u32,
    record_width: u32,
    birth_txg: vec2<u32>,
    fsid: vec2<u32>,
    instance: u32,
    transaction: vec2<u32>,
    birth_sequence: u32,
};
fn bad_packed() -> Packed {
    var p: Packed;
    p.ok = false;
    p.base = 0u;
    p.birth_tree = vec2<u32>(0u, 0u);
    p.record_type = 0u;
    p.container = vec2<u32>(0u, 0u);
    p.container_birth = vec2<u32>(0u, 0u);
    p.record_count = 0u;
    p.record_width = 0u;
    p.birth_txg = vec2<u32>(0u, 0u);
    p.fsid = vec2<u32>(0u, 0u);
    p.instance = 0u;
    p.transaction = vec2<u32>(0u, 0u);
    p.birth_sequence = 0u;
    return p;
}
// unit::parse_packed_unit（读 32 KiB）
fn parse_packed_unit(s: Slice) -> Packed {
    if (s.version == NONE || s.length < DATA_UNIT_BYTES) {
        return bad_packed();
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSU || rd16(base, 4u) != 1u || rd8(base, 6u) != 3u || rd8(base, 7u) != 0u) {
        return bad_packed();
    }
    if ((s.c2 & 11u) != 11u || rd8(base, 42u) != 3u) {
        return bad_packed();
    }
    let declared_length = rd16(base, 8u);
    let record_count = rd16(base, 69u);
    let record_width = rd16(base, 71u);
    if (record_count * record_width != declared_length) {
        return bad_packed();
    }
    if (record_width == 0u && record_count != 0u) {
        return bad_packed();
    }
    if (136u + declared_length > DATA_UNIT_BYTES) {
        return bad_packed();
    }
    var p: Packed;
    p.ok = true;
    p.base = base;
    p.birth_tree = rd64(base, 43u);
    p.record_type = rd16(base, 51u);
    p.container = rd64(base, 53u);
    p.container_birth = rd64(base, 61u);
    p.record_count = record_count;
    p.record_width = record_width;
    p.birth_txg = rd64(base, 73u);
    p.fsid = rd64(base, 81u);
    p.instance = rd32(base, 93u);
    p.transaction = rd48(base, 97u);
    p.birth_sequence = rd32(base, 103u);
    return p;
}
fn packed_record_offset(p: Packed, index: u32) -> u32 {
    return 136u + index * p.record_width;
}
struct DataUnit {
    ok: bool,
    base: u32,
    tree: vec2<u32>,
    object: vec2<u32>,
    object_birth: vec2<u32>,
    anchor_offset: vec2<u32>,
    birth_txg: vec2<u32>,
    fsid: vec2<u32>,
    instance: u32,
    transaction: vec2<u32>,
    declared_length: u32,
    padding_zero: bool,
    payload_crc: u32,
};
fn bad_data_unit() -> DataUnit {
    var d: DataUnit;
    d.ok = false;
    d.base = 0u;
    d.tree = vec2<u32>(0u, 0u);
    d.object = vec2<u32>(0u, 0u);
    d.object_birth = vec2<u32>(0u, 0u);
    d.anchor_offset = vec2<u32>(0u, 0u);
    d.birth_txg = vec2<u32>(0u, 0u);
    d.fsid = vec2<u32>(0u, 0u);
    d.instance = 0u;
    d.transaction = vec2<u32>(0u, 0u);
    d.declared_length = 0u;
    d.padding_zero = false;
    d.payload_crc = 0u;
    return d;
}
// unit::parse_data_unit（读 32 KiB）
fn parse_data_unit(s: Slice) -> DataUnit {
    if (s.version == NONE || s.length < DATA_UNIT_BYTES) {
        return bad_data_unit();
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSU || rd16(base, 4u) != 1u || rd8(base, 6u) != 1u || rd8(base, 7u) != 0u) {
        return bad_data_unit();
    }
    if ((s.c2 & 11u) != 11u || rd8(base, 42u) != 1u) {
        return bad_data_unit();
    }
    let declared_length = rd16(base, 8u);
    if (134u + declared_length > DATA_UNIT_BYTES) {
        return bad_data_unit();
    }
    var d: DataUnit;
    d.ok = true;
    d.base = base;
    d.tree = rd64(base, 43u);
    d.object = rd64(base, 51u);
    d.object_birth = rd64(base, 59u);
    d.anchor_offset = rd64(base, 67u);
    d.birth_txg = rd64(base, 75u);
    d.fsid = rd64(base, 83u);
    d.instance = rd32(base, 91u);
    d.transaction = rd48(base, 95u);
    d.declared_length = declared_length;
    d.padding_zero = (s.c2 & 16u) != 0u;
    d.payload_crc = rd32(base, 101u);
    return d;
}

// ───────────────────────── 中央映射 ─────────────────────────

struct Lookup {
    found: bool,
    failed: bool,
    p: Ptr,
};
// 在映射叶条目里找一个 27 字节 key：同 key 多条取最后一条；条目窄于 55 判红
fn mapping_lookup(key_base: u32, key_offset: u32, key_class: u32, key_tail: vec2<u32>, key_tail_is_transaction: bool) -> Lookup {
    var out: Lookup;
    out.found = false;
    out.failed = false;
    out.p = zero_pointer();
    for (var i = 0u; i < mapping_count; i = i + 1u) {
        let base = sw(R_MAPPING + i * 3u);
        let o = sw(R_MAPPING + i * 3u + 1u);
        let width = sw(R_MAPPING + i * 3u + 2u);
        if (width < 55u) {
            out.failed = true;
            return out;
        }
        // key：类标签 1 + 出生树 8 + 出生 txg 8 + 实例 4 + 尾段 6
        if (rd8(base, o) != key_class) {
            continue;
        }
        if (!bytes_equal(base, o + 1u, key_base, key_offset + 34u, 16u)) {
            continue;
        }
        if (rd32(base, o + 17u) != rd32(key_base, key_offset + 78u)) {
            continue;
        }
        var tail_matches = false;
        if (key_tail_is_transaction) {
            tail_matches = eq64(rd48(base, o + 21u), key_tail);
        } else {
            tail_matches = rd32(base, o + 21u) == key_tail.x && rd16(base, o + 25u) == 0u;
        }
        if (!tail_matches) {
            continue;
        }
        var p = zero_pointer();
        p.all_zero = false;
        p.device0 = rd32(base, o + 27u);
        p.slot0 = rd48(base, o + 31u);
        p.sum0 = rd32(base, o + 37u);
        p.device1 = rd32(base, o + 41u);
        p.slot1 = rd48(base, o + 45u);
        p.sum1 = rd32(base, o + 51u);
        out.p = p;
        out.found = true;
    }
    return out;
}
struct MappedRead {
    s: Slice,
    failed: bool,
};
// 按位置提示读，读不出经中央映射回退（recovery::read_mapped_tree_node_via_hint_then_central_mapping / read_data_unit_via_hint_then_central_mapping）
// pointer_base / pointer_offset：这条指针在字节里的位置（映射 key 从它的出生树、出生 txg、实例、尾段拼）
fn read_via_hint_then_mapping(p: Ptr, pointer_base: u32, pointer_offset: u32, unit_class: u32, unit_bytes: u32) -> MappedRead {
    var out: MappedRead;
    out.failed = false;
    out.s = read_unit_via_locations(p, unit_bytes);
    if (out.s.version != NONE) {
        return out;
    }
    let looked = mapping_lookup(pointer_base, pointer_offset, unit_class, p.tail, unit_class == 1u);
    if (looked.failed || !looked.found) {
        out.failed = true;
        return out;
    }
    out.s = read_unit_via_locations(looked.p, unit_bytes);
    if (out.s.version == NONE) {
        out.failed = true;
    }
    return out;
}

// ───────────────────────── 走读用的判定 ─────────────────────────

fn root_header_checks(n: Node, tree: vec2<u32>, key_width: u32, p: Ptr, root: RootFields, fsid_low: vec2<u32>) -> bool {
    if (!eq64(n.tree, tree)) {
        return false;
    }
    if (n.key_width != key_width) {
        return false;
    }
    if (lt64(root.txg, n.birth_txg) || n.instance > root.instance) {
        return false;
    }
    if (!eq64(n.fsid, fsid_low)) {
        return false;
    }
    if (n.birth_sequence != p.tail.x) {
        return false;
    }
    return true;
}
// recovery::tree_root_checked_against_its_pointer
fn tree_root_checked(n: Node, tree: vec2<u32>, key_width: u32, p: Ptr, root: RootFields, fsid_low: vec2<u32>) -> bool {
    if (!n.ok || !root_header_checks(n, tree, key_width, p, root, fsid_low)) {
        return false;
    }
    if (n.entry_count > 0u) {
        if (!bytes_equal(n.base, entry_offset(n, 0u), n.base, n.smallest_offset, key_width)
            || !bytes_equal(n.base, entry_offset(n, n.entry_count - 1u), n.base, n.largest_offset, key_width)) {
            return false;
        }
    }
    return true;
}
// 见过的单元（按第一条位置条目的槽号）：重复交回 false
fn seen_insert(slot: vec2<u32>) -> bool {
    for (var i = 0u; i < seen_count; i = i + 1u) {
        if (sw(R_SEEN + i * 2u) == slot.x && sw(R_SEEN + i * 2u + 1u) == slot.y) {
            return false;
        }
    }
    if (seen_count >= MAX_SEEN) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
        return false;
    }
    sws(R_SEEN + seen_count * 2u, slot.x);
    sws(R_SEEN + seen_count * 2u + 1u, slot.y);
    seen_count = seen_count + 1u;
    return true;
}

// ───────────────────────── 多层码 2 树（记账、中央映射）─────────────────────────

// key 按字段比：field_widths 0 = 记账 [2,8,4,8]、1 = 映射 [1,8,8,4,6]
fn field_width(schema: u32, field: u32) -> u32 {
    if (schema == 0u) {
        if (field == 0u) { return 2u; }
        if (field == 1u) { return 8u; }
        if (field == 2u) { return 4u; }
        return 8u;
    }
    if (field == 0u) { return 1u; }
    if (field == 1u) { return 8u; }
    if (field == 2u) { return 8u; }
    if (field == 3u) { return 4u; }
    return 6u;
}
fn field_count(schema: u32) -> u32 {
    if (schema == 0u) {
        return 4u;
    }
    return 5u;
}
fn read_field(base: u32, o: u32, width: u32) -> vec2<u32> {
    var v = vec2<u32>(0u, 0u);
    for (var i = 0u; i < width; i = i + 1u) {
        let b = rd8(base, o + i);
        if (i < 4u) {
            v.x = v.x | (b << (i * 8u));
        } else {
            v.y = v.y | (b << ((i - 4u) * 8u));
        }
    }
    return v;
}
// -1 / 0 / 1 → 0 / 1 / 2
fn compare_keys(schema: u32, base_a: u32, oa: u32, base_b: u32, ob: u32) -> u32 {
    var o1 = oa;
    var o2 = ob;
    let n = field_count(schema);
    for (var f = 0u; f < n; f = f + 1u) {
        let w = field_width(schema, f);
        let a = read_field(base_a, o1, w);
        let b = read_field(base_b, o2, w);
        if (lt64(a, b)) {
            return 0u;
        }
        if (lt64(b, a)) {
            return 2u;
        }
        o1 = o1 + w;
        o2 = o2 + w;
    }
    return 1u;
}
struct Code2Frame {
    n: Node,
    child_index: u32,
    first_child_smallest_base: u32,
    first_child_smallest_offset: u32,
    previous_child_largest_base: u32,
    previous_child_largest_offset: u32,
    has_previous_child: bool,
    separator_base: u32,
    separator_offset: u32,
    previous_separator_offset: u32,
    has_previous_separator: bool,
    expected_level: u32,
    is_root: bool,
};
var<private> code2_frames: array<Code2Frame, 12>;

struct TreeRead {
    ok: bool,
    node_count: u32,
    entry_count: u32,
};
fn failed_tree_read() -> TreeRead {
    var t: TreeRead;
    t.ok = false;
    t.node_count = 0u;
    t.entry_count = 0u;
    return t;
}
// code_two_tree::read_code_two_tree（EveryHeaderAgainstItsReference）：schema 0 记账（提示读不出经映射回退）、1 映射（只按位置条目读）
// 叶条目写进 R_ACCOUNTING / R_MAPPING（字节基、条目偏移、条目宽）
fn read_code_two_tree(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, tree: vec2<u32>, schema: u32, root: RootFields, fsid_low: vec2<u32>) -> TreeRead {
    var out = failed_tree_read();
    seen_count = 0u;
    let key_width = field_width(schema, 0u) + field_width(schema, 1u) + field_width(schema, 2u) + field_width(schema, 3u);
    var kw = key_width;
    if (schema == 1u) {
        kw = kw + field_width(schema, 4u);
    }
    var region = R_ACCOUNTING;
    var capacity = MAX_ACCOUNTING_ENTRIES;
    if (schema == 1u) {
        region = R_MAPPING;
        capacity = MAX_MAPPING_ENTRIES;
        mapping_count = 0u;
    } else {
        accounting_count = 0u;
    }
    var entries = 0u;
    var nodes = 0u;
    var depth = 0u;
    // 压根
    var pending_pointer = root_pointer;
    var pending_pointer_base = root_pointer_base;
    var pending_pointer_offset = root_pointer_offset;
    var pending_expected_level = NONE;
    var pending_is_root = true;
    var enter = true;
    loop {
        if (enter) {
            // 进一个节点
            if (!seen_insert(pending_pointer.slot0)) {
                return out;
            }
            var s: Slice;
            if (schema == 1u) {
                s = read_unit_via_locations(pending_pointer, NODE_BYTES);
            } else {
                let m = read_via_hint_then_mapping(pending_pointer, pending_pointer_base, pending_pointer_offset, 2u, NODE_BYTES);
                if (m.failed) {
                    return out;
                }
                s = m.s;
            }
            let n = parse_index_node(s);
            if (!n.ok) {
                return out;
            }
            if (!root_header_checks(n, tree, kw, pending_pointer, root, fsid_low)) {
                return out;
            }
            if (n.level == 0u && n.entry_count > 0u) {
                if (!bytes_equal(n.base, entry_offset(n, 0u), n.base, n.smallest_offset, kw)
                    || !bytes_equal(n.base, entry_offset(n, n.entry_count - 1u), n.base, n.largest_offset, kw)) {
                    return out;
                }
            }
            if (pending_expected_level != NONE && n.level != pending_expected_level) {
                return out;
            }
            if (n.entry_count == 0u) {
                return out;
            }
            nodes = nodes + 1u;
            if (n.level == 0u) {
                // 叶：条目按 key 严格递增，记条目
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    if (e > 0u && compare_keys(schema, n.base, entry_offset(n, e - 1u), n.base, entry_offset(n, e)) != 0u) {
                        return out;
                    }
                    if (entries >= capacity) {
                        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
                        return out;
                    }
                    sws(region + entries * 3u, n.base);
                    sws(region + entries * 3u + 1u, entry_offset(n, e));
                    sws(region + entries * 3u + 2u, n.entry_width);
                    entries = entries + 1u;
                }
                // 把叶交给父节点当孩子
                if (depth == 0u) {
                    out.ok = true;
                    break;
                }
                let parent = depth - 1u;
                // 分隔 key ≤ 孩子最小 key
                if (compare_keys(schema, code2_frames[parent].separator_base, code2_frames[parent].separator_offset, n.base, n.smallest_offset) == 2u) {
                    return out;
                }
                if (code2_frames[parent].child_index == 0u) {
                    code2_frames[parent].first_child_smallest_base = n.base;
                    code2_frames[parent].first_child_smallest_offset = n.smallest_offset;
                }
                code2_frames[parent].previous_child_largest_base = n.base;
                code2_frames[parent].previous_child_largest_offset = n.largest_offset;
                code2_frames[parent].has_previous_child = true;
                code2_frames[parent].child_index = code2_frames[parent].child_index + 1u;
                enter = false;
                continue;
            }
            if (n.entry_width < kw + 86u) {
                return out;
            }
            if (depth >= MAX_FRAMES) {
                undecidable = UNDECIDABLE_TOO_DEEP;
                return out;
            }
            code2_frames[depth].n = n;
            code2_frames[depth].child_index = 0u;
            code2_frames[depth].has_previous_child = false;
            code2_frames[depth].has_previous_separator = false;
            code2_frames[depth].expected_level = pending_expected_level;
            code2_frames[depth].is_root = pending_is_root;
            depth = depth + 1u;
            enter = false;
            continue;
        }
        // 栈顶内部节点：下一个孩子，或收尾
        let top = depth - 1u;
        let n = code2_frames[top].n;
        if (code2_frames[top].child_index >= n.entry_count) {
            // 区间 = 第一个孩子的最小 key 到最后一个孩子的最大 key
            if (!bytes_equal(code2_frames[top].first_child_smallest_base, code2_frames[top].first_child_smallest_offset, n.base, n.smallest_offset, kw)
                || !bytes_equal(code2_frames[top].previous_child_largest_base, code2_frames[top].previous_child_largest_offset, n.base, n.largest_offset, kw)) {
                return out;
            }
            depth = depth - 1u;
            if (depth == 0u) {
                out.ok = true;
                break;
            }
            let parent = depth - 1u;
            if (compare_keys(schema, code2_frames[parent].separator_base, code2_frames[parent].separator_offset, n.base, n.smallest_offset) == 2u) {
                return out;
            }
            if (code2_frames[parent].child_index == 0u) {
                code2_frames[parent].first_child_smallest_base = n.base;
                code2_frames[parent].first_child_smallest_offset = n.smallest_offset;
            }
            code2_frames[parent].previous_child_largest_base = n.base;
            code2_frames[parent].previous_child_largest_offset = n.largest_offset;
            code2_frames[parent].has_previous_child = true;
            code2_frames[parent].child_index = code2_frames[parent].child_index + 1u;
            continue;
        }
        let e = code2_frames[top].child_index;
        let separator_offset = entry_offset(n, e);
        let child_pointer = parse_node_pointer(n.base, separator_offset + kw);
        if (!child_pointer.head_ok) {
            return out;
        }
        if (code2_frames[top].has_previous_separator) {
            if (compare_keys(schema, n.base, separator_offset, n.base, code2_frames[top].previous_separator_offset) != 2u) {
                return out;
            }
            if (compare_keys(schema, n.base, separator_offset, code2_frames[top].previous_child_largest_base, code2_frames[top].previous_child_largest_offset) != 2u) {
                return out;
            }
        }
        code2_frames[top].separator_base = n.base;
        code2_frames[top].separator_offset = separator_offset;
        code2_frames[top].previous_separator_offset = separator_offset;
        code2_frames[top].has_previous_separator = true;
        pending_pointer = child_pointer;
        pending_pointer_base = n.base;
        pending_pointer_offset = separator_offset + kw;
        pending_expected_level = n.level - 1u;
        pending_is_root = false;
        enter = true;
    }
    out.node_count = nodes;
    out.entry_count = entries;
    if (schema == 1u) {
        mapping_count = entries;
    } else {
        accounting_count = entries;
    }
    return out;
}

// ───────────────────────── 分配记录树 ─────────────────────────

fn allocation_span(level: u32) -> vec2<u32> {
    var span = u64_of(ALLOCATION_LEAF_SLOTS);
    for (var i = 0u; i < level; i = i + 1u) {
        span = mul64_32_saturating(span, ALLOCATION_INTERNAL_FANOUT);
    }
    return span;
}
fn allocation_root_level() -> u32 {
    let slots = shr64(device_bytes(), SLOT_SHIFT);
    let device_count = hdr(3u);
    for (var level = 1u; level < 256u; level = level + 1u) {
        let child_span = allocation_span(level - 1u);
        var cells = vec2<u32>(0u, 0u);
        for (var d = 0u; d < device_count; d = d + 1u) {
            let q = divmod64(add64_saturating(slots, sub64(child_span, u64_of(1u))), child_span).quotient;
            cells = add64_saturating(cells, q);
        }
        if (le64(cells, u64_of(ALLOCATION_INTERNAL_FANOUT))) {
            return level;
        }
    }
    return 255u;
}
// 根之下的节点位置：level、device、index；根 level = root_level、device = NONE
struct AllocationNode {
    is_root: bool,
    level: u32,
    device: u32,
    index: vec2<u32>,
};
fn allocation_key_bytes_equal(base: u32, o: u32, device: u32, slot: vec2<u32>) -> bool {
    return rd32(base, o) == device && eq64(rd48(base, o + 4u), slot);
}
struct AllocationFrame {
    n: Node,
    node: AllocationNode,
    child_index: u32,
    has_previous_child: bool,
    previous_child_device: u32,
    previous_child_index: vec2<u32>,
};
var<private> allocation_frames: array<AllocationFrame, 12>;

fn allocation_first_slot(level: u32, index: vec2<u32>) -> vec2<u32> {
    let span = allocation_span(level);
    let p = mul64_32(index, span.x);
    var first = p.value;
    if (p.overflow || span.y != 0u) {
        first = vec2<u32>(NONE, NONE);
    }
    // 槽号最大 2^48 − 1
    let largest = vec2<u32>(NONE, 0xFFFFu);
    if (lt64(largest, first)) {
        first = largest;
    }
    return first;
}
fn allocation_last_slot(level: u32, index: vec2<u32>) -> vec2<u32> {
    let span = allocation_span(level);
    let first = allocation_first_slot(level, index);
    var last = add64_saturating(first, sub64(span, u64_of(1u)));
    let largest = vec2<u32>(NONE, 0xFFFFu);
    if (lt64(largest, last)) {
        last = largest;
    }
    return last;
}
struct AllocationRead {
    ok: bool,
    node_count: u32,
    record_count: u32,
};
// allocation_record_tree::read_allocation_record_tree（EveryHeaderAgainstItsReference，提示读不出经映射回退）；记录写进 R_ALLOCATION
fn read_allocation_tree(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, tree: vec2<u32>, root: RootFields, fsid_low: vec2<u32>) -> AllocationRead {
    var out: AllocationRead;
    out.ok = false;
    out.node_count = 0u;
    out.record_count = 0u;
    seen_count = 0u;
    allocation_count = 0u;
    let root_level = allocation_root_level();
    var nodes = 0u;
    var depth = 0u;
    var pending_pointer = root_pointer;
    var pending_pointer_base = root_pointer_base;
    var pending_pointer_offset = root_pointer_offset;
    var pending_node: AllocationNode;
    pending_node.is_root = true;
    pending_node.level = root_level;
    pending_node.device = NONE;
    pending_node.index = vec2<u32>(0u, 0u);
    var enter = true;
    loop {
        if (enter) {
            if (!seen_insert(pending_pointer.slot0)) {
                return out;
            }
            let m = read_via_hint_then_mapping(pending_pointer, pending_pointer_base, pending_pointer_offset, 2u, NODE_BYTES);
            if (m.failed) {
                return out;
            }
            let n = parse_index_node(m.s);
            if (!n.ok) {
                return out;
            }
            if (!root_header_checks(n, tree, 10u, pending_pointer, root, fsid_low)) {
                return out;
            }
            if (n.level != pending_node.level) {
                return out;
            }
            // key 区间是它的位置规定罩的那一段
            if (pending_node.is_root) {
                if (!all_zero(n.base, n.smallest_offset, 10u)) {
                    return out;
                }
                for (var i = 0u; i < 10u; i = i + 1u) {
                    if (rd8(n.base, n.largest_offset + i) != 0xFFu) {
                        return out;
                    }
                }
            } else {
                let first = allocation_first_slot(pending_node.level, pending_node.index);
                let last = allocation_last_slot(pending_node.level, pending_node.index);
                if (!allocation_key_bytes_equal(n.base, n.smallest_offset, pending_node.device, first)
                    || !allocation_key_bytes_equal(n.base, n.largest_offset, pending_node.device, last)) {
                    return out;
                }
            }
            if (n.entry_count == 0u) {
                return out;
            }
            nodes = nodes + 1u;
            if (n.level == 0u) {
                if (n.entry_width < 20u) {
                    return out;
                }
                let leaf_last = allocation_last_slot(0u, pending_node.index);
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let o = entry_offset(n, e);
                    let device = rd32(n.base, o);
                    let slot = rd48(n.base, o + 4u);
                    let span_field = rd16(n.base, o + 10u);
                    let span = span_field & 0x7FFFu;
                    let generation = rd64(n.base, o + 12u);
                    // 落在这片叶里：同盘、叶序号 = 槽 / 812、末槽不越过叶末槽
                    let leaf_index = divmod64(slot, u64_of(ALLOCATION_LEAF_SLOTS)).quotient;
                    if (pending_node.is_root || device != pending_node.device || !eq64(leaf_index, pending_node.index)) {
                        return out;
                    }
                    if (span == 0u || lt64(leaf_last, add64(slot, u64_of(span - 1u)))) {
                        return out;
                    }
                    if (e > 0u) {
                        let previous_device = sw(R_ALLOCATION + (allocation_count - 1u) * ALLOCATION_RECORD_WORDS);
                        let previous_slot = vec2<u32>(sw(R_ALLOCATION + (allocation_count - 1u) * ALLOCATION_RECORD_WORDS + 1u), sw(R_ALLOCATION + (allocation_count - 1u) * ALLOCATION_RECORD_WORDS + 2u));
                        if (!(previous_device < device || (previous_device == device && lt64(previous_slot, slot)))) {
                            return out;
                        }
                    }
                    if (allocation_count >= MAX_ALLOCATION_RECORDS) {
                        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
                        return out;
                    }
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS, device);
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS + 1u, slot.x);
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS + 2u, slot.y);
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS + 3u, span_field);
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS + 4u, generation.x);
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS + 5u, generation.y);
                    allocation_count = allocation_count + 1u;
                }
                if (depth == 0u) {
                    out.ok = true;
                    break;
                }
                allocation_frames[depth - 1u].child_index = allocation_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            if (n.entry_width < 96u) {
                return out;
            }
            if (depth >= MAX_FRAMES) {
                undecidable = UNDECIDABLE_TOO_DEEP;
                return out;
            }
            allocation_frames[depth].n = n;
            allocation_frames[depth].node = pending_node;
            allocation_frames[depth].child_index = 0u;
            allocation_frames[depth].has_previous_child = false;
            depth = depth + 1u;
            enter = false;
            continue;
        }
        let top = depth - 1u;
        let n = allocation_frames[top].n;
        let parent = allocation_frames[top].node;
        if (allocation_frames[top].child_index >= n.entry_count) {
            depth = depth - 1u;
            if (depth == 0u) {
                out.ok = true;
                break;
            }
            allocation_frames[depth - 1u].child_index = allocation_frames[depth - 1u].child_index + 1u;
            continue;
        }
        let e = allocation_frames[top].child_index;
        let o = entry_offset(n, e);
        // 孩子由条目 key 点名：槽落在孩子那一层的格点上、盘在池里、孩子的父就是这个节点
        let child_level = parent.level - 1u;
        let device = rd32(n.base, o);
        let slot = rd48(n.base, o + 4u);
        let child_span = allocation_span(child_level);
        let qr = divmod64(slot, child_span);
        if (!is_zero64(qr.remainder) || !device_is_in_the_pool(device)) {
            return out;
        }
        var child: AllocationNode;
        child.is_root = false;
        child.level = child_level;
        child.device = device;
        child.index = qr.quotient;
        // parent_of(child) == parent
        if (child_level + 1u == root_level) {
            if (!parent.is_root) {
                return out;
            }
        } else {
            if (parent.is_root) {
                return out;
            }
            let parent_index = divmod64(child.index, u64_of(ALLOCATION_INTERNAL_FANOUT)).quotient;
            if (parent.device != device || !eq64(parent_index, parent.index)) {
                return out;
            }
        }
        if (allocation_frames[top].has_previous_child) {
            let previous_device = allocation_frames[top].previous_child_device;
            let previous_index = allocation_frames[top].previous_child_index;
            if (!(previous_device < device || (previous_device == device && lt64(previous_index, child.index)))) {
                return out;
            }
        }
        allocation_frames[top].has_previous_child = true;
        allocation_frames[top].previous_child_device = device;
        allocation_frames[top].previous_child_index = child.index;
        pending_pointer = parse_node_pointer(n.base, o + 10u);
        pending_pointer_base = n.base;
        pending_pointer_offset = o + 10u;
        pending_node = child;
        enter = true;
    }
    out.node_count = nodes;
    out.record_count = allocation_count;
    return out;
}

// ───────────────────────── extent 树 ─────────────────────────

fn extent_upper_span(level: u32) -> vec2<u32> {
    var span = u64_of(EXTENT_UPPER_LEAF_INODES);
    for (var i = 0u; i < level; i = i + 1u) {
        span = mul64_32_saturating(span, EXTENT_INTERNAL_FANOUT);
    }
    return span;
}
fn extent_lower_span(level: u32) -> vec2<u32> {
    var span = u64_of(EXTENT_LOWER_LEAF_DATA_UNITS);
    for (var i = 0u; i < level; i = i + 1u) {
        span = mul64_32_saturating(span, EXTENT_INTERNAL_FANOUT);
    }
    return span;
}
fn mul64_64_saturating(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    if (b.y != 0u) {
        if (is_zero64(a)) {
            return a;
        }
        return vec2<u32>(NONE, NONE);
    }
    return mul64_32_saturating(a, b.x);
}
// extent key 的三段
fn extent_key_locality(base: u32, o: u32) -> vec2<u32> { return rd64(base, o); }
fn extent_key_inode(base: u32, o: u32) -> vec2<u32> { return rd64(base, o + 8u); }
fn extent_key_offset(base: u32, o: u32) -> vec2<u32> { return rd64(base, o + 16u); }
fn extent_key_equals(base: u32, o: u32, inode: vec2<u32>, offset: vec2<u32>) -> bool {
    return is_zero64(rd64(base, o)) && eq64(rd64(base, o + 8u), inode) && eq64(rd64(base, o + 16u), offset);
}
struct ExtentFrame {
    n: Node,
    level: u32,
    index: vec2<u32>,
    child_index: u32,
    has_previous_child: bool,
    previous_child_index: vec2<u32>,
};
var<private> extent_frames: array<ExtentFrame, 12>;

struct ExtentRead {
    ok: bool,
    node_count: u32,
};
// 判一个 extent 树节点（extent_tree::ExtentTreeReading::read_and_judge_node，Every）
fn judge_extent_node(n: Node, tree: vec2<u32>, p: Ptr, root: RootFields, fsid_low: vec2<u32>, expected_level: u32,
    range_inode_first: vec2<u32>, range_offset_first: vec2<u32>, range_inode_last: vec2<u32>, range_offset_last: vec2<u32>) -> bool {
    if (!n.ok) {
        return false;
    }
    if (!root_header_checks(n, tree, 24u, p, root, fsid_low)) {
        return false;
    }
    if (n.level != expected_level) {
        return false;
    }
    if (!extent_key_equals(n.base, n.smallest_offset, range_inode_first, range_offset_first)
        || !extent_key_equals(n.base, n.largest_offset, range_inode_last, range_offset_last)) {
        return false;
    }
    if (n.entry_count == 0u) {
        return false;
    }
    return true;
}
// 上段：叶条目写进 R_UPPER（字节基、条目偏移）
fn read_upper_segment(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, tree: vec2<u32>, root: RootFields, fsid_low: vec2<u32>) -> ExtentRead {
    var out: ExtentRead;
    out.ok = false;
    out.node_count = 0u;
    upper_count = 0u;
    var nodes = 0u;
    var depth = 0u;
    var pending_pointer = root_pointer;
    var pending_pointer_base = root_pointer_base;
    var pending_pointer_offset = root_pointer_offset;
    var pending_level = NONE;
    var pending_index = vec2<u32>(0u, 0u);
    var enter = true;
    loop {
        if (enter) {
            if (!seen_insert(pending_pointer.slot0)) {
                return out;
            }
            let m = read_via_hint_then_mapping(pending_pointer, pending_pointer_base, pending_pointer_offset, 2u, NODE_BYTES);
            if (m.failed) {
                return out;
            }
            let n = parse_index_node(m.s);
            if (!n.ok) {
                return out;
            }
            var level = pending_level;
            if (level == NONE) {
                level = n.level;
            }
            let span = extent_upper_span(level);
            let first_inode = mul64_64_saturating(pending_index, span);
            let last_inode = add64_saturating(first_inode, sub64(span, u64_of(1u)));
            if (!judge_extent_node(n, tree, pending_pointer, root, fsid_low, level, first_inode, vec2<u32>(0u, 0u), last_inode, vec2<u32>(NONE, NONE))) {
                return out;
            }
            nodes = nodes + 1u;
            if (level == 0u) {
                if (n.entry_width < 113u) {
                    return out;
                }
                var previous_inode = vec2<u32>(0u, 0u);
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let o = entry_offset(n, e);
                    if (!is_zero64(extent_key_locality(n.base, o)) || !is_zero64(extent_key_offset(n.base, o))) {
                        return out;
                    }
                    let inode = extent_key_inode(n.base, o);
                    let tag = rd8(n.base, o + 24u);
                    if (tag == 0u) {
                        if (!all_zero(n.base, o + 25u, 88u)) {
                            return out;
                        }
                    } else if (tag == 1u) {
                        if (!all_zero(n.base, o + 25u + 86u, 2u)) {
                            return out;
                        }
                    } else if (tag != 2u) {
                        return out;
                    }
                    if (lt64(inode, first_inode) || lt64(last_inode, inode)) {
                        return out;
                    }
                    if (e > 0u && !lt64(previous_inode, inode)) {
                        return out;
                    }
                    previous_inode = inode;
                    if (upper_count >= MAX_UPPER_ENTRIES) {
                        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
                        return out;
                    }
                    sws(R_UPPER + upper_count * 2u, n.base);
                    sws(R_UPPER + upper_count * 2u + 1u, o);
                    upper_count = upper_count + 1u;
                }
                if (depth == 0u) {
                    out.ok = true;
                    break;
                }
                extent_frames[depth - 1u].child_index = extent_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            if (n.entry_width < 110u) {
                return out;
            }
            if (depth >= MAX_FRAMES) {
                undecidable = UNDECIDABLE_TOO_DEEP;
                return out;
            }
            extent_frames[depth].n = n;
            extent_frames[depth].level = level;
            extent_frames[depth].index = pending_index;
            extent_frames[depth].child_index = 0u;
            extent_frames[depth].has_previous_child = false;
            depth = depth + 1u;
            enter = false;
            continue;
        }
        let top = depth - 1u;
        let n = extent_frames[top].n;
        if (extent_frames[top].child_index >= n.entry_count) {
            depth = depth - 1u;
            if (depth == 0u) {
                out.ok = true;
                break;
            }
            extent_frames[depth - 1u].child_index = extent_frames[depth - 1u].child_index + 1u;
            continue;
        }
        let e = extent_frames[top].child_index;
        let o = entry_offset(n, e);
        let child_level = extent_frames[top].level - 1u;
        let child_span = extent_upper_span(child_level);
        let first_inode = extent_key_inode(n.base, o);
        if (!is_zero64(extent_key_locality(n.base, o)) || !is_zero64(extent_key_offset(n.base, o))) {
            return out;
        }
        let qr = divmod64(first_inode, child_span);
        if (!is_zero64(qr.remainder)) {
            return out;
        }
        let child_index = qr.quotient;
        // child.parent() == parent：index / 147
        let parent_index = divmod64(child_index, u64_of(EXTENT_INTERNAL_FANOUT)).quotient;
        if (!eq64(parent_index, extent_frames[top].index)) {
            return out;
        }
        if (extent_frames[top].has_previous_child && !lt64(extent_frames[top].previous_child_index, child_index)) {
            return out;
        }
        extent_frames[top].has_previous_child = true;
        extent_frames[top].previous_child_index = child_index;
        pending_pointer = parse_node_pointer(n.base, o + 24u);
        pending_pointer_base = n.base;
        pending_pointer_offset = o + 24u;
        pending_level = child_level;
        pending_index = child_index;
        enter = true;
    }
    out.node_count = nodes;
    return out;
}
// 下段：数据指针写进 R_DATA（字节基、指针偏移），按单元号升序；单元号要从 0 起连号由调用方判
fn read_lower_segment(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, inode: vec2<u32>, tree: vec2<u32>, root: RootFields, fsid_low: vec2<u32>) -> ExtentRead {
    var out: ExtentRead;
    out.ok = false;
    out.node_count = 0u;
    var nodes = 0u;
    var depth = 0u;
    var pending_pointer = root_pointer;
    var pending_pointer_base = root_pointer_base;
    var pending_pointer_offset = root_pointer_offset;
    var pending_level = NONE;
    var pending_index = vec2<u32>(0u, 0u);
    var enter = true;
    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
    loop {
        if (enter) {
            if (!seen_insert(pending_pointer.slot0)) {
                return out;
            }
            let m = read_via_hint_then_mapping(pending_pointer, pending_pointer_base, pending_pointer_offset, 2u, NODE_BYTES);
            if (m.failed) {
                return out;
            }
            let n = parse_index_node(m.s);
            if (!n.ok) {
                return out;
            }
            var level = pending_level;
            if (level == NONE) {
                level = n.level;
            }
            let span = extent_lower_span(level);
            let first_unit = mul64_64_saturating(pending_index, span);
            let last_unit = add64_saturating(first_unit, sub64(span, u64_of(1u)));
            let first_offset = mul64_64_saturating(first_unit, payload);
            let last_offset = sub64_saturating(mul64_64_saturating(add64_saturating(last_unit, u64_of(1u)), payload), u64_of(1u));
            if (!judge_extent_node(n, tree, pending_pointer, root, fsid_low, level, inode, first_offset, inode, last_offset)) {
                return out;
            }
            nodes = nodes + 1u;
            if (level == 0u) {
                if (n.entry_width < 112u) {
                    return out;
                }
                var previous_unit = vec2<u32>(0u, 0u);
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let o = entry_offset(n, e);
                    let offset = extent_key_offset(n.base, o);
                    let qr = divmod64(offset, payload);
                    if (!is_zero64(extent_key_locality(n.base, o)) || !eq64(extent_key_inode(n.base, o), inode) || !is_zero64(qr.remainder)) {
                        return out;
                    }
                    let unit = qr.quotient;
                    if (lt64(unit, first_unit) || lt64(last_unit, unit)) {
                        return out;
                    }
                    if (e > 0u && !lt64(previous_unit, unit)) {
                        return out;
                    }
                    previous_unit = unit;
                    if (data_count >= MAX_DATA_POINTERS) {
                        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
                        return out;
                    }
                    // 单元号要等于它在整个文件里的位置（extents_of_the_first_file 的「没有洞」）：这里记下来，调用方判
                    sws(R_DATA + data_count * 2u, n.base);
                    sws(R_DATA + data_count * 2u + 1u, o + 24u);
                    if (!eq64(unit, u64_of(data_count))) {
                        // 记一个不可能的偏移让调用方判红：单元有洞
                        sws(R_DATA + data_count * 2u + 1u, NONE);
                    }
                    data_count = data_count + 1u;
                }
                if (depth == 0u) {
                    out.ok = true;
                    break;
                }
                extent_frames[depth - 1u].child_index = extent_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            if (n.entry_width < 110u) {
                return out;
            }
            if (depth >= MAX_FRAMES) {
                undecidable = UNDECIDABLE_TOO_DEEP;
                return out;
            }
            extent_frames[depth].n = n;
            extent_frames[depth].level = level;
            extent_frames[depth].index = pending_index;
            extent_frames[depth].child_index = 0u;
            extent_frames[depth].has_previous_child = false;
            depth = depth + 1u;
            enter = false;
            continue;
        }
        let top = depth - 1u;
        let n = extent_frames[top].n;
        if (extent_frames[top].child_index >= n.entry_count) {
            depth = depth - 1u;
            if (depth == 0u) {
                out.ok = true;
                break;
            }
            extent_frames[depth - 1u].child_index = extent_frames[depth - 1u].child_index + 1u;
            continue;
        }
        let e = extent_frames[top].child_index;
        let o = entry_offset(n, e);
        let level = extent_frames[top].level;
        let child_span = extent_lower_span(level - 1u);
        let child_offset_span = mul64_64_saturating(child_span, payload);
        let offset = extent_key_offset(n.base, o);
        if (!is_zero64(extent_key_locality(n.base, o)) || !eq64(extent_key_inode(n.base, o), inode)) {
            return out;
        }
        let qr = divmod64(offset, child_offset_span);
        if (!is_zero64(qr.remainder)) {
            return out;
        }
        let child_index = qr.quotient;
        // child.data_unit_range().0 / span(level) == index
        let child_first_unit = mul64_64_saturating(child_index, child_span);
        let parent_index = divmod64(child_first_unit, extent_lower_span(level)).quotient;
        if (!eq64(parent_index, extent_frames[top].index)) {
            return out;
        }
        if (extent_frames[top].has_previous_child && !lt64(extent_frames[top].previous_child_index, child_index)) {
            return out;
        }
        extent_frames[top].has_previous_child = true;
        extent_frames[top].previous_child_index = child_index;
        pending_pointer = parse_node_pointer(n.base, o + 24u);
        pending_pointer_base = n.base;
        pending_pointer_offset = o + 24u;
        pending_level = level - 1u;
        pending_index = child_index;
        enter = true;
    }
    out.node_count = nodes;
    return out;
}
// 整棵 extent 树 + extents_of_the_first_file：上段整段，再按每条叶条目读下段；第一版只有第一个文件
fn read_extent_tree_of_the_first_file(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, tree: vec2<u32>, root: RootFields, fsid_low: vec2<u32>) -> ExtentRead {
    var out: ExtentRead;
    out.ok = false;
    out.node_count = 0u;
    seen_count = 0u;
    data_count = 0u;
    let upper = read_upper_segment(root_pointer, root_pointer_base, root_pointer_offset, tree, root, fsid_low);
    if (!upper.ok) {
        return out;
    }
    var nodes = upper.node_count;
    var files_with_units = 0u;
    for (var i = 0u; i < upper_count; i = i + 1u) {
        let base = sw(R_UPPER + i * 2u);
        let o = sw(R_UPPER + i * 2u + 1u);
        let inode = extent_key_inode(base, o);
        let tag = rd8(base, o + 24u);
        if (tag == 1u) {
            let lower_root = parse_node_pointer(base, o + 25u);
            let lower = read_lower_segment(lower_root, base, o + 25u, inode, tree, root, fsid_low);
            if (!lower.ok) {
                return out;
            }
            nodes = nodes + lower.node_count;
        } else if (tag == 2u) {
            if (data_count >= MAX_DATA_POINTERS) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
                return out;
            }
            sws(R_DATA + data_count * 2u, base);
            sws(R_DATA + data_count * 2u + 1u, o + 25u);
            data_count = data_count + 1u;
        }
    }
    // extents_of_the_first_file：恰好一条叶条目、是第一个文件、至少一个数据单元、单元从 0 起连号
    if (upper_count != 1u) {
        return out;
    }
    if (!eq64(extent_key_inode(sw(R_UPPER), sw(R_UPPER + 1u)), u64_of(FIRST_INODE_NUMBER))) {
        return out;
    }
    if (data_count == 0u) {
        return out;
    }
    for (var i = 0u; i < data_count; i = i + 1u) {
        if (sw(R_DATA + i * 2u + 1u) == NONE) {
            return out;
        }
    }
    out.ok = true;
    out.node_count = nodes;
    return out;
}

// ───────────────────────── 实例表 ─────────────────────────

// InstanceTablePage::parse 的每一片：身份 (0, 4, 片号, 0)、行 kind 0 且 flags 0、末条是合法链指针记录；交回 0 最后一片、1 有下一片（写进 next）、2 坏
struct PageParse {
    verdict: u32,
    next: Ptr,
};
fn parse_instance_table_page(p: Packed, page_index: vec2<u32>) -> PageParse {
    var out: PageParse;
    out.verdict = 2u;
    out.next = zero_pointer();
    if (!p.ok) {
        return out;
    }
    if (!is_zero64(p.birth_tree) || p.record_type != 4u || !eq64(p.container, page_index) || !is_zero64(p.container_birth)) {
        return out;
    }
    if (p.record_count == 0u || p.record_width != 88u) {
        return out;
    }
    let last = packed_record_offset(p, p.record_count - 1u);
    if (rd8(p.base, last) != 1u) {
        return out;
    }
    let flag = rd8(p.base, last + 1u);
    let pointer_zero = all_zero(p.base, last + 2u, 86u);
    if (flag == 0u) {
        if (!pointer_zero) {
            return out;
        }
        out.verdict = 0u;
    } else if (flag == 1u) {
        if (pointer_zero) {
            return out;
        }
        out.verdict = 1u;
        out.next = parse_node_pointer(p.base, last + 2u);
    } else {
        return out;
    }
    for (var r = 0u; r + 1u < p.record_count; r = r + 1u) {
        let o = packed_record_offset(p, r);
        if (rd8(p.base, o) != 0u || rd8(p.base, o + 21u) != 0u) {
            out.verdict = 2u;
            return out;
        }
    }
    return out;
}
// recovery::read_instance_table_chain：沿链读整张表；true = 读得出解得开
fn instance_table_chain_readable(root: RootFields) -> bool {
    var pointer = root.instance_table;
    var page_index = vec2<u32>(0u, 0u);
    for (var page = 0u; page < 256u; page = page + 1u) {
        let s = read_unit_via_locations(pointer, DATA_UNIT_BYTES);
        if (s.version == NONE) {
            return false;
        }
        let parsed = parse_instance_table_page(parse_packed_unit(s), page_index);
        if (parsed.verdict == 2u) {
            return false;
        }
        if (parsed.verdict == 0u) {
            return true;
        }
        pointer = parsed.next;
        page_index = add64(page_index, u64_of(1u));
    }
    undecidable = UNDECIDABLE_TOO_DEEP;
    return false;
}

// ───────────────────────── 走到第一个文件 ─────────────────────────

struct WalkResult {
    outcome: u32,
    failure: u32,
    content_matches: bool,
};
fn walk_failed(code: u32) -> WalkResult {
    var w: WalkResult;
    w.outcome = OUTCOME_FAILED;
    w.failure = code;
    w.content_matches = false;
    return w;
}
fn walk_no_file() -> WalkResult {
    var w: WalkResult;
    w.outcome = OUTCOME_NO_FILE;
    w.failure = 0u;
    w.content_matches = false;
    return w;
}
fn tree_kind_key_width(kind: u32) -> u32 {
    if (kind == TREE_KIND_EXTENT) { return 24u; }
    if (kind == TREE_KIND_INODE) { return 8u; }
    if (kind == TREE_KIND_ALLOCATION) { return 10u; }
    if (kind == TREE_KIND_ACCOUNTING) { return 22u; }
    return NONE;
}
fn expected_version_index(instance: u32, txg: vec2<u32>) -> u32 {
    let count = hdr(8u);
    for (var i = 0u; i < count; i = i + 1u) {
        if (tab(T_EXPECTED, i * EXPECTED_WORDS) == instance
            && tab(T_EXPECTED, i * EXPECTED_WORDS + 1u) == txg.x
            && tab(T_EXPECTED, i * EXPECTED_WORDS + 2u) == txg.y) {
            return i;
        }
    }
    return NONE;
}
// recovery::walk_to_file_in_the_unit_area_starting_at
fn walk_to_file(root: RootFields, cfg: SysConfig, expected: u32) -> WalkResult {
    let fsid_low = vec2<u32>(cfg.fsid.x, cfg.fsid.y);
    // 实例表：根直接持有第 0 片
    let instance_table_slice = read_unit_via_locations(root.instance_table, DATA_UNIT_BYTES);
    if (instance_table_slice.version == NONE) {
        return walk_failed(10u);
    }
    let instance_table = parse_packed_unit(instance_table_slice);
    if (!instance_table.ok) {
        return walk_failed(11u);
    }
    if (instance_table.record_type != 4u || instance_table.record_count == 0u) {
        return walk_failed(12u);
    }
    let first_page = parse_instance_table_page(instance_table, vec2<u32>(0u, 0u));
    if (first_page.verdict == 2u) {
        return walk_failed(13u);
    }
    if (first_page.verdict == 1u) {
        if (!instance_table_chain_readable(root)) {
            return walk_failed(14u);
        }
    }
    // 树表
    let tree_table_slice = read_unit_via_locations(root.tree_table, NODE_BYTES);
    if (tree_table_slice.version == NONE) {
        return walk_failed(20u);
    }
    let tree_table = parse_index_node(tree_table_slice);
    if (!tree_table.ok) {
        return walk_failed(21u);
    }
    if (tree_table.key_width != 8u) {
        return walk_failed(22u);
    }
    if (tree_table.entry_count == 0u) {
        return walk_no_file();
    }
    // 条目逐条解开、每种至多一条
    if (tree_table.entry_width != 200u) {
        return walk_failed(23u);
    }
    var kinds_seen = 0u;
    for (var e = 0u; e < tree_table.entry_count; e = e + 1u) {
        let o = entry_offset(tree_table, e);
        if (rd16(tree_table.base, o + 8u) != 200u || rd16(tree_table.base, o + 12u) != 0u || !all_zero(tree_table.base, o + 124u, 76u)) {
            return walk_failed(23u);
        }
        let kind = rd16(tree_table.base, o + 10u);
        if (kind < 32u) {
            if ((kinds_seen & (1u << kind)) != 0u) {
                return walk_failed(24u);
            }
            kinds_seen = kinds_seen | (1u << kind);
        }
        // 与前一条比种类：种类 ≥ 32 的两条同种拦不到，走读同样拦不到（kinds 是 u16，但一条流写不出这种树）
    }
    // I-9.16 ①：树 ID 严格升序
    for (var e = 1u; e < tree_table.entry_count; e = e + 1u) {
        let previous = rd64(tree_table.base, entry_offset(tree_table, e - 1u));
        let current = rd64(tree_table.base, entry_offset(tree_table, e));
        if (!lt64(previous, current)) {
            return walk_failed(25u);
        }
    }
    // I-9.16 ②：按发号次序 1,2,3,4,6,7,8 相邻两棵严格升序
    var previous_tree = vec2<u32>(0u, 0u);
    var have_previous = false;
    for (var k = 0u; k < 7u; k = k + 1u) {
        var kind = k + 1u;
        if (k >= 4u) {
            kind = k + 2u;
        }
        for (var e = 0u; e < tree_table.entry_count; e = e + 1u) {
            let o = entry_offset(tree_table, e);
            if (rd16(tree_table.base, o + 10u) == kind) {
                let tree = rd64(tree_table.base, o);
                if (have_previous && !lt64(previous_tree, tree)) {
                    return walk_failed(26u);
                }
                previous_tree = tree;
                have_previous = true;
                break;
            }
        }
    }
    // 中央映射树：走读到 TreeRoots 那一步总要读它；先读也罢（读得出读不出与次序无关，见 crash_judge_gpu.rs 的说明）
    let mapping = read_code_two_tree(root.mapping_root, 0u, 0u, root.mapping_root.birth_tree, 1u, root, fsid_low);
    if (!mapping.ok) {
        return walk_failed(30u);
    }
    var have_extent = false;
    var have_inode = false;
    var have_allocation = false;
    var have_accounting = false;
    var extent_tree = vec2<u32>(0u, 0u);
    var extent_nodes = 0u;
    var allocation_nodes = 0u;
    var accounting_nodes = 0u;
    var inode_root = bad_node();
    var inode_root_pointer = zero_pointer();
    for (var e = 0u; e < tree_table.entry_count; e = e + 1u) {
        let o = entry_offset(tree_table, e);
        let tree = rd64(tree_table.base, o);
        // I-7.8：树 ID 低于水位
        if (!lt64(tree, root.watermark)) {
            return walk_failed(27u);
        }
        let pointer = parse_node_pointer(tree_table.base, o + 14u);
        if (pointer.all_zero) {
            continue;
        }
        let kind = rd16(tree_table.base, o + 10u);
        if (kind == TREE_KIND_ALLOCATION) {
            let read = read_allocation_tree(pointer, tree_table.base, o + 14u, tree, root, fsid_low);
            if (!read.ok) {
                return walk_failed(31u);
            }
            allocation_nodes = read.node_count;
            have_allocation = true;
            continue;
        }
        if (kind == TREE_KIND_EXTENT) {
            let read = read_extent_tree_of_the_first_file(pointer, tree_table.base, o + 14u, tree, root, fsid_low);
            if (!read.ok) {
                return walk_failed(32u);
            }
            extent_nodes = read.node_count;
            extent_tree = tree;
            have_extent = true;
            continue;
        }
        if (kind == TREE_KIND_ACCOUNTING) {
            let read = read_code_two_tree(pointer, tree_table.base, o + 14u, tree, 0u, root, fsid_low);
            if (!read.ok) {
                return walk_failed(33u);
            }
            accounting_nodes = read.node_count;
            have_accounting = true;
            continue;
        }
        let key_width = tree_kind_key_width(kind);
        if (key_width == NONE) {
            return walk_failed(34u);
        }
        // inode 树根（进映射的树根：提示读不出经映射回退，再核自描述）
        let m = read_via_hint_then_mapping(pointer, tree_table.base, o + 14u, 2u, NODE_BYTES);
        if (m.failed) {
            return walk_failed(35u);
        }
        let n = parse_index_node(m.s);
        if (!tree_root_checked(n, tree, key_width, pointer, root, fsid_low)) {
            return walk_failed(36u);
        }
        if (kind == TREE_KIND_INODE) {
            inode_root = n;
            inode_root_pointer = pointer;
            have_inode = true;
        }
    }
    if (!have_extent || !have_inode || !have_allocation || !have_accounting) {
        return walk_failed(37u);
    }
    // 分配记录落在池几何里：盘在池里、槽不低于单元区起点、跨度不越过单元区末尾、同盘不相交（记录按 (盘, 槽) 升序，相邻比即可）
    let device_slots = shr64(device_bytes(), SLOT_SHIFT);
    if (lt64(device_slots, cfg.unit_area_start_slot)) {
        return walk_failed(38u);
    }
    for (var r = 0u; r < allocation_count; r = r + 1u) {
        let device = sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS);
        let slot = vec2<u32>(sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS + 1u), sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS + 2u));
        let span = sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS + 3u) & 0x7FFFu;
        if (!device_is_in_the_pool(device)) {
            return walk_failed(38u);
        }
        if (lt64(slot, cfg.unit_area_start_slot)) {
            return walk_failed(38u);
        }
        if (lt64(device_slots, add64(slot, u64_of(span)))) {
            return walk_failed(38u);
        }
        if (r > 0u) {
            let previous_device = sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS);
            let previous_slot = vec2<u32>(sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS + 1u), sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS + 2u));
            let previous_span = sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS + 3u) & 0x7FFFu;
            if (previous_device == device && lt64(slot, add64(previous_slot, u64_of(previous_span)))) {
                return walk_failed(38u);
            }
        }
    }
    // 每个落点每盘各一条：各盘的 (槽, 跨度, 代, 已释放) 集合相同、同盘槽号唯一（升序已判过）、不少于 10 个落点
    let device_count = hdr(3u);
    var first_device_start = NONE;
    var first_device_count = 0u;
    var devices_with_records = 0u;
    var r = 0u;
    while (r < allocation_count) {
        let device = sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS);
        var end = r;
        while (end < allocation_count && sw(R_ALLOCATION + end * ALLOCATION_RECORD_WORDS) == device) {
            end = end + 1u;
        }
        devices_with_records = devices_with_records + 1u;
        if (first_device_start == NONE) {
            first_device_start = r;
            first_device_count = end - r;
        } else {
            if (end - r != first_device_count) {
                return walk_failed(39u);
            }
            for (var i = 0u; i < first_device_count; i = i + 1u) {
                let a = R_ALLOCATION + (first_device_start + i) * ALLOCATION_RECORD_WORDS;
                let b = R_ALLOCATION + (r + i) * ALLOCATION_RECORD_WORDS;
                if (sw(a + 1u) != sw(b + 1u) || sw(a + 2u) != sw(b + 2u) || sw(a + 3u) != sw(b + 3u) || sw(a + 4u) != sw(b + 4u) || sw(a + 5u) != sw(b + 5u)) {
                    return walk_failed(39u);
                }
            }
        }
        r = end;
    }
    if (devices_with_records != device_count || first_device_count < 10u) {
        return walk_failed(39u);
    }
    // 记账条目数 3 + 6 × 盘数
    if (accounting_count != 3u + 6u * device_count) {
        return walk_failed(40u);
    }
    // 映射条目数 = 1 + extent 节点 + 分配记录树节点 + 记账树节点 + inode 叶容器数 + 数据单元数
    if (mapping_count != 1u + extent_nodes + allocation_nodes + accounting_nodes + inode_root.entry_count + data_count) {
        return walk_failed(41u);
    }
    // 分配记录跨度非 0、代不晚于根
    for (var r2 = 0u; r2 < allocation_count; r2 = r2 + 1u) {
        let span = sw(R_ALLOCATION + r2 * ALLOCATION_RECORD_WORDS + 3u) & 0x7FFFu;
        let generation = vec2<u32>(sw(R_ALLOCATION + r2 * ALLOCATION_RECORD_WORDS + 4u), sw(R_ALLOCATION + r2 * ALLOCATION_RECORD_WORDS + 5u));
        if (span == 0u || lt64(root.txg, generation)) {
            return walk_failed(42u);
        }
    }
    // 记账条目：宽 ≥ 34、代不晚于根、seq ≠ 0
    for (var a = 0u; a < accounting_count; a = a + 1u) {
        let base = sw(R_ACCOUNTING + a * 3u);
        let o = sw(R_ACCOUNTING + a * 3u + 1u);
        let width = sw(R_ACCOUNTING + a * 3u + 2u);
        if (width < 34u) {
            return walk_failed(43u);
        }
        if (lt64(root.txg, rd64(base, o + 14u)) || rd32(base, o + 30u) == 0u) {
            return walk_failed(43u);
        }
    }
    // inode 树：根层级 1，每条内部条目 → 叶容器 → 记录
    if (inode_root.level != 1u) {
        return walk_failed(44u);
    }
    if (inode_root.entry_width < 120u) {
        return walk_failed(45u);
    }
    var found_inode = false;
    var inode_size = vec2<u32>(0u, 0u);
    var inode_object_birth = vec2<u32>(0u, 0u);
    for (var e = 0u; e < inode_root.entry_count; e = e + 1u) {
        let o = entry_offset(inode_root, e);
        let separator = rd64(inode_root.base, o);
        let identity_birth_tree = rd64(inode_root.base, o + 8u);
        let identity_type = rd16(inode_root.base, o + 16u);
        let identity_container = rd64(inode_root.base, o + 18u);
        let identity_birth = rd64(inode_root.base, o + 26u);
        if (identity_type != 2u) {
            return walk_failed(46u);
        }
        let child = parse_node_pointer(inode_root.base, o + 34u);
        let m = read_via_hint_then_mapping(child, inode_root.base, o + 34u, 3u, DATA_UNIT_BYTES);
        if (m.failed) {
            return walk_failed(47u);
        }
        let leaf = parse_packed_unit(m.s);
        if (!leaf.ok) {
            return walk_failed(48u);
        }
        if (!eq64(leaf.birth_tree, identity_birth_tree) || leaf.record_type != identity_type || !eq64(leaf.container, identity_container)
            || !eq64(leaf.container_birth, identity_birth) || leaf.record_width != 140u || !eq64(leaf.fsid, fsid_low)) {
            return walk_failed(49u);
        }
        if (!eq64(child.birth_tree, identity_birth_tree) || leaf.birth_sequence != child.tail.x) {
            return walk_failed(50u);
        }
        if (lt64(root.txg, leaf.birth_txg) || leaf.instance != child.instance) {
            return walk_failed(51u);
        }
        for (var rec = 0u; rec < leaf.record_count; rec = rec + 1u) {
            let ro = packed_record_offset(leaf, rec);
            if (!all_zero(leaf.base, ro + 108u, 32u)) {
                return walk_failed(52u);
            }
            let inode = rd64(leaf.base, ro);
            if (lt64(inode, separator) || lt64(inode, identity_container)) {
                return walk_failed(53u);
            }
            if (eq64(inode, u64_of(FIRST_INODE_NUMBER))) {
                found_inode = true;
                inode_object_birth = rd64(leaf.base, ro + 8u);
                inode_size = rd64(leaf.base, ro + 40u);
            }
        }
    }
    if (!found_inode) {
        return walk_no_file();
    }
    // 数据单元数 = max(1, ⌈size ÷ 32634⌉)
    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
    var expected_units = divmod64(add64_saturating(inode_size, sub64(payload, u64_of(1u))), payload).quotient;
    if (is_zero64(expected_units)) {
        expected_units = u64_of(1u);
    }
    if (!eq64(expected_units, u64_of(data_count))) {
        return walk_failed(54u);
    }
    var content_matches = true;
    var chunk_first = 0u;
    var chunk_count = 0u;
    if (expected != NONE) {
        chunk_first = tab(T_EXPECTED, expected * EXPECTED_WORDS + 3u);
        chunk_count = tab(T_EXPECTED, expected * EXPECTED_WORDS + 4u);
        if (chunk_count != data_count) {
            content_matches = false;
        }
    }
    for (var i = 0u; i < data_count; i = i + 1u) {
        let base = sw(R_DATA + i * 2u);
        let o = sw(R_DATA + i * 2u + 1u);
        let pointer = parse_data_pointer(base, o);
        let m = read_via_hint_then_mapping(pointer, base, o, 1u, DATA_UNIT_BYTES);
        if (m.failed) {
            return walk_failed(55u);
        }
        let unit = parse_data_unit(m.s);
        if (!unit.ok) {
            return walk_failed(56u);
        }
        let first_file_byte = mul64_64_saturating(u64_of(i), payload);
        if (!eq64(unit.tree, extent_tree) || !eq64(unit.object, u64_of(FIRST_INODE_NUMBER)) || !eq64(unit.anchor_offset, first_file_byte)) {
            return walk_failed(57u);
        }
        if (!eq64(unit.object_birth, inode_object_birth)) {
            return walk_failed(58u);
        }
        if (!eq64(unit.birth_txg, pointer.birth_txg) || unit.instance != pointer.instance || !eq64(unit.transaction, pointer.tail) || !eq64(unit.fsid, fsid_low)) {
            return walk_failed(59u);
        }
        var expected_length = sub64_saturating(inode_size, first_file_byte);
        if (lt64(payload, expected_length)) {
            expected_length = payload;
        }
        if (!eq64(u64_of(unit.declared_length), expected_length)) {
            return walk_failed(60u);
        }
        if (!unit.padding_zero) {
            return walk_failed(61u);
        }
        if (expected != NONE && i < chunk_count) {
            if (tab(T_CHUNKS, (chunk_first + i) * 2u) != unit.payload_crc || tab(T_CHUNKS, (chunk_first + i) * 2u + 1u) != unit.declared_length) {
                content_matches = false;
            }
        }
    }
    var w: WalkResult;
    w.outcome = OUTCOME_FILE_READ;
    w.failure = 0u;
    w.content_matches = content_matches;
    return w;
}


// ───────────────────────── 每个状态的开场：表的目录、草稿区、持久掩码 ─────────────────────────

fn begin_state(local: u32) {
    for (var t = 0u; t < TABLE_COUNT; t = t + 1u) {
        B[t] = tables[t * 2u];
    }
    scratch_base = local * dispatch.scratch_words;
    undecidable = 0u;
    record_count = 0u;
    root_count = 0u;
    mapping_count = 0u;
    accounting_count = 0u;
    allocation_count = 0u;
    seen_count = 0u;
    data_count = 0u;
    upper_count = 0u;
    row_count = 0u;
    var ordinal_lo = dispatch.block_start_lo + local;
    var ordinal_hi = dispatch.block_start_hi;
    if (ordinal_lo < dispatch.block_start_lo) {
        ordinal_hi = ordinal_hi + 1u;
    }
    for (var word = 0u; word < 64u; word = word + 1u) {
        persisted[word] = 0u;
    }
    // 序号 → 持久掩码（与 crash_facts::persisted_writes_of_state 同一套数字次序）
    let segment_count = hdr(2u);
    var segment_of_state = segment_count;
    for (var s = 0u; s < segment_count; s = s + 1u) {
        let end = vec2<u32>(tab(T_SEGMENTS, s * 6u + 4u), tab(T_SEGMENTS, s * 6u + 5u));
        if (lt64(vec2<u32>(ordinal_lo, ordinal_hi), end)) {
            segment_of_state = s;
            break;
        }
    }
    for (var s = 0u; s < segment_of_state; s = s + 1u) {
        let first = tab(T_SEGMENTS, s * 6u);
        let count = tab(T_SEGMENTS, s * 6u + 1u);
        for (var k = 0u; k < count; k = k + 1u) {
            set_persisted(tab(T_SEGMENT_WRITES, first + k));
        }
    }
    var torn: array<u32, 64>;
    for (var word = 0u; word < 64u; word = word + 1u) {
        torn[word] = 0u;
    }
    if (segment_of_state < segment_count) {
        let s = segment_of_state;
        var remaining = ordinal_lo - tab(T_SEGMENTS, s * 6u + 2u);
        let first = tab(T_SEGMENTS, s * 6u);
        let count = tab(T_SEGMENTS, s * 6u + 1u);
        for (var k = 0u; k < count; k = k + 1u) {
            let write = tab(T_SEGMENT_WRITES, first + k);
            let choices = tab(T_RECORDED, write * 4u);
            let digit = remaining % choices;
            remaining = remaining / choices;
            if (digit == choices - 1u) {
                set_persisted(write);
            } else if (digit == 1u) {
                torn[write >> 5u] = torn[write >> 5u] | (1u << (write & 31u));
            }
        }
    }
    let recorded_count = hdr(0u);
    for (var w = 0u; w < recorded_count; w = w + 1u) {
        if ((torn[w >> 5u] & (1u << (w & 31u))) == 0u) {
            continue;
        }
        let torn_index = tab(T_RECORDED, w * 4u + 1u);
        if (torn_index == NONE) {
            continue;
        }
        set_persisted(torn_index);
        let replay_first = tab(T_RECORDED, w * 4u + 2u);
        let replay_count = tab(T_RECORDED, w * 4u + 3u);
        for (var r = 0u; r < replay_count; r = r + 1u) {
            let replay_index = tab(T_REPLAYS, (replay_first + r) * 2u);
            let replayed = tab(T_REPLAYS, (replay_first + r) * 2u + 1u);
            if (is_persisted(replayed)) {
                set_persisted(replay_index);
            }
        }
    }
}
