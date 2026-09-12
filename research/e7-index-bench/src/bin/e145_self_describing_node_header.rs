//! E145：码 2 自描述头与映射树 key 宽的代价——C306（码 2 头不自描述 key 宽）与 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树）。
//!
//! 跑前登记 `research/prompts/e145-preregistration.md`。计数模型加一段扫描解头的模拟：三条码 2 头臂（A 今天、B 自描述、C 定宽区间）
//! 在五棵树上的头宽、扇出、树高；扫描期没有树表时各臂解得出几个头；映射树三条装法的条目宽、扇出、树高、一次点查读几个节点。确定性。

use e7_index_bench::Emitter;

const NODE_BYTES: u64 = 16384;
const NONCE_MAC_RESERVED_BYTES: u64 = 28;
/// E142（第一个事务的干跑）：码 2 头 = 81 + 2 × key 宽（共同前缀 42 + 树 ID 8 + 层级 1 + 诞生代号 8 + fsid 8 + 写序 4 + 出生序号 4 + 载荷 CRC 4 + 预留 2）。
const HEADER_WITHOUT_KEY_RANGE: u64 = 81;
/// A 臂载荷内部的条目数 u16 + 条目宽 u16（E142 的预想）。
const PAYLOAD_INTERNAL_COUNT_BYTES: u64 = 4;
/// B 臂头里多的：key 宽 1 + 条目数 2 + 条目宽 2。
const SELF_DESCRIBING_EXTRA_BYTES: u64 = 5;
/// C 臂：key 区间定宽 32 × 2。
const FIXED_KEY_RANGE_KEY_BYTES: u64 = 32;
const NODE_POINTER_BYTES: u64 = 83;
const LOCATION_ENTRY_PAIR_BYTES: u64 = 28;
const TREE_TABLE_ENTRY_BYTES: u64 = 145;
/// 10 TB ÷ 32 KiB。
const DATA_UNITS_IN_POOL: u64 = 335_544_320;
/// 码 2 / 码 3 单元数按数据单元的 1/8 计（跑前登记的粗档）。
const NODES_PER_DATA_UNITS: u64 = 8;
const CHECKSUM_FIELD_OFFSET: usize = 10;
const KEY_WIDTH_FIELD_OFFSET: usize = 51;
const UNIT_MAGIC: [u8; 4] = *b"SFSU";
const UNIT_CLASS_INDEX_NODE: u8 = 2;
const NODES_PER_TREE_IN_SCAN: usize = 200;
const UNREGISTERED_KEY_WIDTH: u64 = 16;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Tree {
    name: &'static str,
    key_width: u64,
    leaf_entry_bytes: u64,
    leaf_entries_in_pool: u64,
}

/// 五棵登记树：key 宽与叶条目宽照 E142（第一个事务的干跑）。
const TREES: [Tree; 5] = [
    Tree { name: "inode", key_width: 8, leaf_entry_bytes: 117, leaf_entries_in_pool: DATA_UNITS_IN_POOL / NODES_PER_DATA_UNITS },
    Tree { name: "allocation", key_width: 12, leaf_entry_bytes: 20, leaf_entries_in_pool: DATA_UNITS_IN_POOL },
    Tree { name: "accounting", key_width: 22, leaf_entry_bytes: 34, leaf_entries_in_pool: 1_000_000 },
    Tree { name: "extent", key_width: 24, leaf_entry_bytes: 109, leaf_entries_in_pool: DATA_UNITS_IN_POOL },
    Tree { name: "mapping", key_width: 27, leaf_entry_bytes: 55, leaf_entries_in_pool: DATA_UNITS_IN_POOL + DATA_UNITS_IN_POOL / NODES_PER_DATA_UNITS },
];
const REGISTERED_KEY_WIDTHS: [u64; 5] = [8, 12, 22, 24, 27];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum HeaderArm {
    Today,
    SelfDescribing,
    FixedKeyRange,
}

const HEADER_ARMS: [HeaderArm; 3] = [HeaderArm::Today, HeaderArm::SelfDescribing, HeaderArm::FixedKeyRange];

impl HeaderArm {
    fn name(self) -> &'static str {
        match self {
            HeaderArm::Today => "A_today",
            HeaderArm::SelfDescribing => "B_self_describing",
            HeaderArm::FixedKeyRange => "C_fixed_key_range",
        }
    }
    fn header_bytes(self, key_width: u64) -> u64 {
        match self {
            HeaderArm::Today => HEADER_WITHOUT_KEY_RANGE + 2 * key_width,
            HeaderArm::SelfDescribing => HEADER_WITHOUT_KEY_RANGE + 2 * key_width + SELF_DESCRIBING_EXTRA_BYTES,
            HeaderArm::FixedKeyRange => HEADER_WITHOUT_KEY_RANGE + 2 * FIXED_KEY_RANGE_KEY_BYTES + SELF_DESCRIBING_EXTRA_BYTES,
        }
    }
    /// 载荷里能装条目的字节：A 臂还要减载荷内部的条目数 / 条目宽。
    fn payload_bytes(self, key_width: u64) -> u64 {
        let after_header = NODE_BYTES - self.header_bytes(key_width) - NONCE_MAC_RESERVED_BYTES;
        match self {
            HeaderArm::Today => after_header - PAYLOAD_INTERNAL_COUNT_BYTES,
            HeaderArm::SelfDescribing | HeaderArm::FixedKeyRange => after_header,
        }
    }
}

fn fanout(payload_bytes: u64, entry_bytes: u64) -> u64 {
    payload_bytes / entry_bytes
}

/// 满装 B 树的高：叶按叶条目宽装，内部节点条目 = key + 83 指针；高 1 是只有一个叶。
fn tree_height(leaf_entries: u64, leaf_fanout: u64, internal_fanout: u64) -> u64 {
    let mut nodes = leaf_entries.div_ceil(leaf_fanout).max(1);
    let mut height = 1;
    while nodes > 1 {
        nodes = nodes.div_ceil(internal_fanout);
        height += 1;
    }
    height
}

// ───────────────────────── 扫描解头的模拟 ─────────────────────────

fn castagnoli_crc32(bytes: &[u8]) -> u32 {
    let mut remainder = !0u32;
    for &byte in bytes {
        remainder ^= u32::from(byte);
        for _ in 0..8 {
            remainder = if remainder & 1 == 1 { (remainder >> 1) ^ 0x82F6_3B78 } else { remainder >> 1 };
        }
    }
    !remainder
}

/// 头校验和字段 32 字节：这里只关心「哪个头宽下校验和过」，用四个种子的 CRC32C 拼成 32 字节代替（算法本身归 E144）。
fn wide_checksum_with_field_zeroed(bytes: &[u8], cover_end: usize) -> [u8; 32] {
    let mut covered = bytes[..cover_end].to_vec();
    covered[CHECKSUM_FIELD_OFFSET..CHECKSUM_FIELD_OFFSET + 32].fill(0);
    let mut field = [0u8; 32];
    for seed in 0..4u32 {
        let mut salted = seed.to_le_bytes().to_vec();
        salted.extend_from_slice(&covered);
        field[seed as usize * 8..seed as usize * 8 + 4].copy_from_slice(&castagnoli_crc32(&salted).to_le_bytes());
        let mut twice = salted.clone();
        twice.push(seed as u8);
        field[seed as usize * 8 + 4..seed as usize * 8 + 8].copy_from_slice(&castagnoli_crc32(&twice).to_le_bytes());
    }
    field
}

struct Generator(u64);

impl Generator {
    fn next_byte(&mut self) -> u8 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 40) as u8
    }
}

/// 造一个码 2 节点：共同前缀（magic、类标签 2）+ 伪随机的类身份段 + 校验和；B 臂在偏移 51 放 key 宽，C 臂同样放 key 宽但头宽固定。
fn build_node(arm: HeaderArm, key_width: u64, generator: &mut Generator) -> Vec<u8> {
    let header_end = arm.header_bytes(key_width) as usize;
    let mut node = vec![0u8; NODE_BYTES as usize];
    for byte in node.iter_mut().take(header_end) {
        *byte = generator.next_byte();
    }
    node[..4].copy_from_slice(&UNIT_MAGIC);
    node[4..6].copy_from_slice(&1u16.to_le_bytes());
    node[6] = UNIT_CLASS_INDEX_NODE;
    node[7] = 0;
    match arm {
        HeaderArm::Today => {}
        HeaderArm::SelfDescribing | HeaderArm::FixedKeyRange => node[KEY_WIDTH_FIELD_OFFSET] = u8::try_from(key_width).expect("key 宽 1 字节"),
    }
    let digest = wide_checksum_with_field_zeroed(&node, header_end);
    node[CHECKSUM_FIELD_OFFSET..CHECKSUM_FIELD_OFFSET + 32].copy_from_slice(&digest);
    node
}

fn header_checksum_holds(node: &[u8], header_end: usize) -> bool {
    wide_checksum_with_field_zeroed(node, header_end)[..] == node[CHECKSUM_FIELD_OFFSET..CHECKSUM_FIELD_OFFSET + 32]
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct ScanTally {
    resolved: u64,
    unresolved: u64,
    ambiguous: u64,
}

/// 没有树表的扫描器：A 臂逐个登记宽试，恰一个宽过校验和才算解出；B / C 臂读头里的 key 宽。
fn scan_header(arm: HeaderArm, node: &[u8]) -> Option<u64> {
    match arm {
        HeaderArm::Today => {
            let passing: Vec<u64> = REGISTERED_KEY_WIDTHS.iter().copied().filter(|width| header_checksum_holds(node, arm.header_bytes(*width) as usize)).collect();
            match passing.as_slice() {
                [only] => Some(*only),
                _ => None,
            }
        }
        HeaderArm::SelfDescribing | HeaderArm::FixedKeyRange => {
            let key_width = u64::from(node[KEY_WIDTH_FIELD_OFFSET]);
            header_checksum_holds(node, arm.header_bytes(key_width) as usize).then_some(key_width)
        }
    }
}

fn scan_ambiguity_count(node: &[u8]) -> usize {
    REGISTERED_KEY_WIDTHS.iter().filter(|width| header_checksum_holds(node, HeaderArm::Today.header_bytes(**width) as usize)).count()
}

fn scan_pool(arm: HeaderArm, key_widths: &[u64], nodes_per_width: usize, generator: &mut Generator) -> ScanTally {
    let mut tally = ScanTally::default();
    for &key_width in key_widths {
        for _ in 0..nodes_per_width {
            let node = build_node(arm, key_width, generator);
            if arm == HeaderArm::Today && scan_ambiguity_count(&node) > 1 {
                tally.ambiguous += 1;
            }
            match scan_header(arm, &node) {
                Some(found) if found == key_width => tally.resolved += 1,
                _ => tally.unresolved += 1,
            }
        }
    }
    tally
}

// ───────────────────────── 映射树的三条装法 ─────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MappingArm {
    PadTo27,
    TwoTrees,
    SelfDescribingEntries,
}

const MAPPING_ARMS: [MappingArm; 3] = [MappingArm::PadTo27, MappingArm::TwoTrees, MappingArm::SelfDescribingEntries];
const MAPPING_KEY_DATA: u64 = 27;
const MAPPING_KEY_NODE: u64 = 25;

impl MappingArm {
    fn name(self) -> &'static str {
        match self {
            MappingArm::PadTo27 => "a_pad_to_27",
            MappingArm::TwoTrees => "b_two_trees",
            MappingArm::SelfDescribingEntries => "c_self_describing_entries",
        }
    }
    /// (码 1 条目宽, 码 2 / 码 3 条目宽, 树表条目数)。
    fn shape(self) -> (u64, u64, u64) {
        match self {
            MappingArm::PadTo27 => (MAPPING_KEY_DATA + LOCATION_ENTRY_PAIR_BYTES, MAPPING_KEY_DATA + LOCATION_ENTRY_PAIR_BYTES, 1),
            MappingArm::TwoTrees => (MAPPING_KEY_DATA + LOCATION_ENTRY_PAIR_BYTES, MAPPING_KEY_NODE + LOCATION_ENTRY_PAIR_BYTES, 2),
            MappingArm::SelfDescribingEntries => (1 + MAPPING_KEY_DATA + LOCATION_ENTRY_PAIR_BYTES + 2, 1 + MAPPING_KEY_NODE + LOCATION_ENTRY_PAIR_BYTES + 2, 1),
        }
    }
}

struct MappingFigures {
    data_entry_bytes: u64,
    node_entry_bytes: u64,
    tree_table_entries: u64,
    data_leaf_fanout: u64,
    node_leaf_fanout: u64,
    data_height: u64,
    node_height: u64,
    reads_per_data_lookup: u64,
}

/// 映射树的头按 B 臂（自描述）算，key 宽 27；两棵树那一臂里码 2 / 码 3 那棵按 key 25。
fn mapping_figures(arm: MappingArm) -> MappingFigures {
    let (data_entry_bytes, node_entry_bytes, tree_table_entries) = arm.shape();
    let data_units = DATA_UNITS_IN_POOL;
    let node_units = DATA_UNITS_IN_POOL / NODES_PER_DATA_UNITS;
    let payload_27 = HeaderArm::SelfDescribing.payload_bytes(MAPPING_KEY_DATA);
    let internal_27 = fanout(payload_27, MAPPING_KEY_DATA + NODE_POINTER_BYTES);
    match arm {
        MappingArm::PadTo27 | MappingArm::SelfDescribingEntries => {
            let mixed_entry_bytes = if arm == MappingArm::PadTo27 { data_entry_bytes } else { (data_entry_bytes * data_units + node_entry_bytes * node_units) / (data_units + node_units) };
            let leaf_fanout = fanout(payload_27, mixed_entry_bytes);
            let height = tree_height(data_units + node_units, leaf_fanout, internal_27);
            MappingFigures {
                data_entry_bytes,
                node_entry_bytes,
                tree_table_entries,
                data_leaf_fanout: leaf_fanout,
                node_leaf_fanout: leaf_fanout,
                data_height: height,
                node_height: height,
                reads_per_data_lookup: height,
            }
        }
        MappingArm::TwoTrees => {
            let payload_25 = HeaderArm::SelfDescribing.payload_bytes(MAPPING_KEY_NODE);
            let internal_25 = fanout(payload_25, MAPPING_KEY_NODE + NODE_POINTER_BYTES);
            let data_leaf_fanout = fanout(payload_27, data_entry_bytes);
            let node_leaf_fanout = fanout(payload_25, node_entry_bytes);
            let data_height = tree_height(data_units, data_leaf_fanout, internal_27);
            let node_height = tree_height(node_units, node_leaf_fanout, internal_25);
            MappingFigures { data_entry_bytes, node_entry_bytes, tree_table_entries, data_leaf_fanout, node_leaf_fanout, data_height, node_height, reads_per_data_lookup: data_height }
        }
    }
}

fn emit(emitter: &mut Emitter, body: &str) {
    println!("{}", emitter.emit_raw(body));
}

fn main() {
    let mut emitter = Emitter::new();
    emit(&mut emitter, &format!("name=config node_bytes={NODE_BYTES} reserved={NONCE_MAC_RESERVED_BYTES} header_without_key_range={HEADER_WITHOUT_KEY_RANGE} data_units_in_pool={DATA_UNITS_IN_POOL} nodes_per_data_units={NODES_PER_DATA_UNITS}"));
    let mut height_differences = 0u64;
    for tree in TREES {
        let mut heights = Vec::new();
        for arm in HEADER_ARMS {
            let header_bytes = arm.header_bytes(tree.key_width);
            let payload = arm.payload_bytes(tree.key_width);
            let leaf_fanout = fanout(payload, tree.leaf_entry_bytes);
            let internal_fanout = fanout(payload, tree.key_width + NODE_POINTER_BYTES);
            let height = tree_height(tree.leaf_entries_in_pool, leaf_fanout, internal_fanout);
            heights.push(height);
            emit(&mut emitter, &format!("name=header tree={} arm={} key_width={} header_bytes={header_bytes} with_reserved={}", tree.name, arm.name(), tree.key_width, header_bytes + NONCE_MAC_RESERVED_BYTES));
            emit(&mut emitter, &format!("name=fanout tree={} arm={} payload_bytes={payload} leaf_entry_bytes={} leaf_fanout={leaf_fanout} internal_fanout={internal_fanout}", tree.name, arm.name(), tree.leaf_entry_bytes));
            emit(&mut emitter, &format!("name=height tree={} arm={} leaf_entries={} height={height}", tree.name, arm.name(), tree.leaf_entries_in_pool));
        }
        if heights.iter().any(|height| *height != heights[0]) {
            height_differences += 1;
        }
    }

    let mut generator = Generator(0x4534_3134_3520_5343);
    let mut scan_summary = Vec::new();
    for arm in HEADER_ARMS {
        let registered = scan_pool(arm, &REGISTERED_KEY_WIDTHS, NODES_PER_TREE_IN_SCAN, &mut generator);
        let unregistered = scan_pool(arm, &[UNREGISTERED_KEY_WIDTH], NODES_PER_TREE_IN_SCAN, &mut generator);
        emit(&mut emitter, &format!(
            "name=scan arm={} registered_nodes={} resolved={} unresolved={} ambiguous={} unregistered_nodes={} unregistered_resolved={} unregistered_unresolved={}",
            arm.name(), REGISTERED_KEY_WIDTHS.len() * NODES_PER_TREE_IN_SCAN, registered.resolved, registered.unresolved, registered.ambiguous, NODES_PER_TREE_IN_SCAN, unregistered.resolved, unregistered.unresolved
        ));
        scan_summary.push((arm, registered, unregistered));
    }

    for arm in MAPPING_ARMS {
        let figures = mapping_figures(arm);
        emit(&mut emitter, &format!("name=mapping arm={} data_entry_bytes={} node_entry_bytes={} tree_table_entries={} tree_table_bytes={} data_leaf_fanout={} node_leaf_fanout={}", arm.name(), figures.data_entry_bytes, figures.node_entry_bytes, figures.tree_table_entries, figures.tree_table_entries * TREE_TABLE_ENTRY_BYTES, figures.data_leaf_fanout, figures.node_leaf_fanout));
        emit(&mut emitter, &format!("name=mapping_height arm={} data_height={} node_height={} reads_per_data_lookup={}", arm.name(), figures.data_height, figures.node_height, figures.reads_per_data_lookup));
    }

    let today = scan_summary.iter().find(|(arm, _, _)| *arm == HeaderArm::Today).expect("A 臂");
    let self_describing = scan_summary.iter().find(|(arm, _, _)| *arm == HeaderArm::SelfDescribing).expect("B 臂");
    emit(&mut emitter, &format!(
        "name=verdict trees_with_height_difference={height_differences} today_resolves_registered={} today_resolves_unregistered={} today_ambiguous={} self_describing_resolves_all={}",
        today.1.resolved, today.2.resolved, today.1.ambiguous,
        self_describing.1.unresolved == 0 && self_describing.2.unresolved == 0
    ));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_widths_pin_to_the_e142_numbers() {
        assert_eq!(HeaderArm::Today.header_bytes(8), 97);
        assert_eq!(HeaderArm::Today.header_bytes(12), 105);
        assert_eq!(HeaderArm::Today.header_bytes(22), 125);
        assert_eq!(HeaderArm::Today.header_bytes(24), 129);
        assert_eq!(HeaderArm::Today.header_bytes(27), 135);
        assert_eq!(HeaderArm::SelfDescribing.header_bytes(8), 102);
        assert_eq!(HeaderArm::SelfDescribing.header_bytes(27), 140);
        assert_eq!(HeaderArm::FixedKeyRange.header_bytes(8), 150);
        assert_eq!(HeaderArm::FixedKeyRange.header_bytes(27), 150);
    }

    #[test]
    fn payload_bytes_of_arm_a_subtract_the_in_payload_count_fields() {
        assert_eq!(HeaderArm::Today.payload_bytes(8), 16255);
        assert_eq!(HeaderArm::SelfDescribing.payload_bytes(8), 16254);
        assert_eq!(HeaderArm::FixedKeyRange.payload_bytes(8), 16206);
    }

    #[test]
    fn registered_key_widths_are_exactly_the_five_trees() {
        let mut from_trees: Vec<u64> = TREES.iter().map(|tree| tree.key_width).collect();
        from_trees.sort_unstable();
        assert_eq!(from_trees, REGISTERED_KEY_WIDTHS.to_vec());
        assert!(!REGISTERED_KEY_WIDTHS.contains(&UNREGISTERED_KEY_WIDTH));
    }

    /// 等价变异留档：A 臂「恰一个宽过校验和」与「取第一个过的宽」在 32 字节校验和下行为相同——两个头宽下同一字段同时过，
    /// 要两个不同覆盖范围的摘要相等，结构上不会发生；这里把「每个节点至多一个宽过」钉住。
    #[test]
    fn arm_a_never_sees_two_widths_pass_the_header_checksum() {
        let mut generator = Generator(13);
        for &key_width in &REGISTERED_KEY_WIDTHS {
            for _ in 0..50 {
                let node = build_node(HeaderArm::Today, key_width, &mut generator);
                assert_eq!(scan_ambiguity_count(&node), 1);
            }
        }
    }

    #[test]
    fn fanouts_match_hand_arithmetic() {
        // A：(16384 − 97 − 28 − 4) / 117 = 16255 / 117 = 138；B：(16384 − 102 − 28) / 117 = 16254 / 117 = 138；C：(16384 − 150 − 28) / 117 = 138
        assert_eq!(fanout(HeaderArm::Today.payload_bytes(8), 117), 138);
        assert_eq!(fanout(HeaderArm::SelfDescribing.payload_bytes(8), 117), 138);
        assert_eq!(fanout(HeaderArm::FixedKeyRange.payload_bytes(8), 117), 138);
        // extent：A (16384 − 129 − 28 − 4) / 109 = 16223 / 109 = 148；C (16384 − 150 − 28) / 109 = 148
        assert_eq!(fanout(HeaderArm::Today.payload_bytes(24), 109), 148);
        assert_eq!(fanout(HeaderArm::FixedKeyRange.payload_bytes(24), 109), 148);
        // 分配树：A (16384 − 105 − 28 − 4) / 20 = 812；B (16384 − 110 − 28) / 20 = 812；C (16384 − 178) / 20 = 810
        assert_eq!(fanout(HeaderArm::Today.payload_bytes(12), 20), 812);
        assert_eq!(fanout(HeaderArm::SelfDescribing.payload_bytes(12), 20), 812);
        assert_eq!(fanout(HeaderArm::FixedKeyRange.payload_bytes(12), 20), 810);
    }

    #[test]
    fn tree_height_counts_levels_from_a_single_leaf() {
        assert_eq!(tree_height(1, 100, 100), 1);
        assert_eq!(tree_height(100, 100, 100), 1);
        assert_eq!(tree_height(101, 100, 100), 2);
        assert_eq!(tree_height(10_000, 100, 100), 2);
        assert_eq!(tree_height(10_001, 100, 100), 3);
        assert_eq!(tree_height(335_544_320, 148, 133), 4, "extent 树 10 TB 池：叶 2 267 191 个 → 17 047 → 129 → 1，共 4 层");
    }

    #[test]
    fn scanner_without_tree_table_resolves_registered_trees_only_under_arm_today() {
        let mut generator = Generator(5);
        let registered = scan_pool(HeaderArm::Today, &REGISTERED_KEY_WIDTHS, 20, &mut generator);
        assert_eq!(registered.resolved, 100);
        assert_eq!(registered.ambiguous, 0);
        let unregistered = scan_pool(HeaderArm::Today, &[UNREGISTERED_KEY_WIDTH], 20, &mut generator);
        assert_eq!(unregistered.resolved, 0);
        assert_eq!(unregistered.unresolved, 20);
        for arm in [HeaderArm::SelfDescribing, HeaderArm::FixedKeyRange] {
            let all = scan_pool(arm, &[8, 12, 22, 24, 27, UNREGISTERED_KEY_WIDTH], 20, &mut generator);
            assert_eq!(all.resolved, 120, "{}", arm.name());
            assert_eq!(all.unresolved, 0);
        }
    }

    #[test]
    fn one_node_whose_key_width_byte_is_wrong_is_rejected_by_the_self_describing_scanner() {
        let mut generator = Generator(9);
        let mut node = build_node(HeaderArm::SelfDescribing, 24, &mut generator);
        node[KEY_WIDTH_FIELD_OFFSET] = 8;
        assert_eq!(scan_header(HeaderArm::SelfDescribing, &node), None);
    }

    #[test]
    fn mapping_arms_pin_entry_widths_fanouts_and_heights() {
        let pad = mapping_figures(MappingArm::PadTo27);
        assert_eq!(pad.data_entry_bytes, 55);
        assert_eq!(pad.tree_table_entries, 1);
        // (16384 − 140 − 28) / 55 = 16216 / 55 = 294
        assert_eq!(pad.data_leaf_fanout, 294);
        let two = mapping_figures(MappingArm::TwoTrees);
        assert_eq!((two.data_entry_bytes, two.node_entry_bytes, two.tree_table_entries), (55, 53, 2));
        // (16384 − 136 − 28) / 53 = 16220 / 53 = 306
        assert_eq!(two.node_leaf_fanout, 306);
        let described = mapping_figures(MappingArm::SelfDescribingEntries);
        assert_eq!((described.data_entry_bytes, described.node_entry_bytes), (58, 56));
        // 补零：377 487 360 ÷ 294 = 1 283 970 叶 → 内部扇出 (16384 − 140 − 28) ÷ 110 = 147 → 8735 → 60 → 1，共 4 层
        assert_eq!(pad.data_height, 4);
        assert_eq!(two.data_height, 4);
        assert_eq!(described.data_height, 4);
        // 两棵树的码 2 / 码 3 树：41 943 040 ÷ 306 = 137 069 叶 → 内部扇出 (16384 − 136 − 28) ÷ 108 = 150 → 914 → 7 → 1，共 4 层
        assert_eq!(two.node_height, 4);
        assert_eq!(two.tree_table_entries * TREE_TABLE_ENTRY_BYTES, 290);
    }
}
