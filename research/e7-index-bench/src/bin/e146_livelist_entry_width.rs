//! E146：livelist 条目按映射 key 定身份之后的宽度与代价——D6（快照实现模型） 已定项 2 ① 的代价表。
//!
//! 只报数不判输赢：几种装法各算成 key / value 宽、码 2 节点的扇出、E132 规模网格上的树高、
//! 每次 FREE 的预付字节（C260）与第一个事务上 day-1 注册对惰性创建的字节差。
//! 判据与失败条款写在 `research/prompts/e146-preregistration.md`，装置写之前。

use e7_index_bench::Emitter;

/// D8 已定项 2。
const NODE_BYTES: u64 = 16384;
/// D18 已定项 7：码 2 头里 key 区间之外的部分。
const HEADER_WITHOUT_KEY_RANGE: u64 = 81;
/// D18 已定项 14：nonce / MAC 预留位。
const NONCE_MAC_RESERVED_BYTES: u64 = 28;
/// D19 已定项 7 / 已定项 8：指向码 2 / 码 3 的指针。
const NODE_POINTER_BYTES: u64 = 83;
/// D8 已定项 8。
const TREE_TABLE_ENTRY_BYTES: u64 = 200;
/// 树表一个单元装几条，取 D22（单元原子性怎么合成） 已定项 7 的口径 ⌊(16384 − 131) / 200⌋ = 81。⚠️ 它随条目宽变，改 TREE_TABLE_ENTRY_BYTES 要一起改这一行。
const TREE_TABLE_ENTRIES_PER_UNIT: u64 = 81;
/// first-txn-layout.md 第 1 版树表的条目数：2026-09-13 用户定案后 7（extent、inode、分配记录、记账、映射，加 day-1 注册的 livelist 与稀疏旁表）；本实验建模时是 5，两处都装得进 112 条的单元。
const TREE_TABLE_FIRST_VERSION_ENTRIES: u64 = 7;
/// D6 已定项 3：共享树的 key 以头的树 ID 打头。
const HEAD_TREE_IDENTIFIER_BYTES: u64 = 8;
/// D19 已定项 6：码 1 的映射 key。
const MAPPING_KEY_DATA_BYTES: u64 = 27;
/// D19 已定项 6：码 2 / 码 3 的映射 key。
const MAPPING_KEY_NODE_BYTES: u64 = 25;
/// ALLOC / FREE 单独占一个字节时的宽度。
const EVENT_TYPE_BYTES: u64 = 1;
/// D19 已定项 4：位置条目（作提示用）。
const LOCATION_HINT_BYTES: u64 = 14;
/// E130 用过的作废形态：类型标签 2 + 位置条目 14 + birth txg 8。
const LEGACY_E130_ENTRY_BYTES: u64 = 2 + 14 + 8;
/// D4 已定项 1 / 已定项 5。
const UNIT_BYTES: u64 = 32768;
/// D23 已定项 4 口径的点名项。
const JOURNAL_NAMED_ENTRY_BYTES: u64 = 56;
/// D19 已定项 6：指向码 2 的映射条目。
const MAPPING_ENTRY_NODE_BYTES: u64 = 53;
/// E145 的口径：每 8 个数据单元一个索引节点，两棵树那一臂按它分条目。
const DATA_UNITS_PER_NODE: u64 = 8;
const BASIS_POINTS: u64 = 10000;

/// E132 的规模网格。
const HEADS_SWEPT: [u64; 7] = [1, 4, 16, 64, 256, 1024, 4096];
const ENTRIES_PER_HEAD_SWEPT: [u64; 3] = [1024, 65536, 1048576];
/// 跑前登记的敏感取样点：460 条时补齐臂一个叶装得下、带事件字节的臂要两个叶。
const SENSITIVE_ENTRY_COUNT: u64 = 460;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EntryForm {
    LegacyE130,
    PadEventInTag,
    PadEventByte,
    TwoTreesEventInTag,
    PadEventInTagWithHint,
}

const FORMS: [EntryForm; 5] = [
    EntryForm::LegacyE130,
    EntryForm::PadEventInTag,
    EntryForm::PadEventByte,
    EntryForm::TwoTreesEventInTag,
    EntryForm::PadEventInTagWithHint,
];

/// 一棵树的 key / value 宽。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct TreeShape {
    key_bytes: u64,
    value_bytes: u64,
}

impl TreeShape {
    fn entry_bytes(self) -> u64 {
        self.key_bytes + self.value_bytes
    }
    fn header_bytes(self) -> u64 {
        HEADER_WITHOUT_KEY_RANGE + 2 * self.key_bytes + NONCE_MAC_RESERVED_BYTES
    }
    fn payload_bytes(self) -> u64 {
        NODE_BYTES - self.header_bytes()
    }
    fn leaf_fanout(self) -> u64 {
        self.payload_bytes() / self.entry_bytes()
    }
    fn internal_fanout(self) -> u64 {
        self.payload_bytes() / (self.key_bytes + NODE_POINTER_BYTES)
    }
    fn height(self, entries: u64) -> u64 {
        tree_height(entries, self.leaf_fanout(), self.internal_fanout())
    }
}

impl EntryForm {
    fn name(self) -> &'static str {
        match self {
            EntryForm::LegacyE130 => "legacy_e130",
            EntryForm::PadEventInTag => "pad_event_in_tag",
            EntryForm::PadEventByte => "pad_event_byte",
            EntryForm::TwoTreesEventInTag => "two_trees_event_in_tag",
            EntryForm::PadEventInTagWithHint => "pad_event_in_tag_with_hint",
        }
    }
    /// 这一臂的树：一棵或两棵，各带自己的 key / value 宽。
    fn trees(self) -> Vec<(&'static str, TreeShape)> {
        let padded_key = HEAD_TREE_IDENTIFIER_BYTES + MAPPING_KEY_DATA_BYTES;
        match self {
            EntryForm::LegacyE130 => vec![("shared", TreeShape { key_bytes: LEGACY_E130_ENTRY_BYTES, value_bytes: 0 })],
            EntryForm::PadEventInTag => vec![("shared", TreeShape { key_bytes: padded_key, value_bytes: 0 })],
            EntryForm::PadEventByte => vec![("shared", TreeShape { key_bytes: padded_key + EVENT_TYPE_BYTES, value_bytes: 0 })],
            EntryForm::TwoTreesEventInTag => vec![
                ("data", TreeShape { key_bytes: HEAD_TREE_IDENTIFIER_BYTES + MAPPING_KEY_DATA_BYTES, value_bytes: 0 }),
                ("node", TreeShape { key_bytes: HEAD_TREE_IDENTIFIER_BYTES + MAPPING_KEY_NODE_BYTES, value_bytes: 0 }),
            ],
            EntryForm::PadEventInTagWithHint => vec![("shared", TreeShape { key_bytes: padded_key, value_bytes: LOCATION_HINT_BYTES })],
        }
    }
    /// 一个规模格里每棵树各分到多少条：两棵树按数据 : 节点 = 8 : 1。
    fn entries_per_tree(self, total_entries: u64) -> Vec<u64> {
        match self {
            EntryForm::LegacyE130 | EntryForm::PadEventInTag | EntryForm::PadEventByte | EntryForm::PadEventInTagWithHint => vec![total_entries],
            EntryForm::TwoTreesEventInTag => {
                let node_entries = total_entries / (DATA_UNITS_PER_NODE + 1);
                vec![total_entries - node_entries, node_entries]
            }
        }
    }
    /// 每次 FREE 追加一条叶条目要预付的字节：码 1 那棵树的叶条目宽。
    fn prepaid_bytes_per_free(self) -> u64 {
        self.trees()[0].1.entry_bytes()
    }
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

/// 预付量占一个 32 KiB 单元的万分点，向下取整。
fn unit_share_basis_points(prepaid_bytes: u64) -> u64 {
    prepaid_bytes * BASIS_POINTS / UNIT_BYTES
}

/// 第一个事务上 day-1 注册 livelist 树要写的字节：空树根指针为零时只多树表条目；实现要求空树也有根节点时再加一个节点、一个点名项、一条映射条目。
fn first_transaction_day1_bytes(tree_count: u64, empty_tree_needs_root: bool) -> u64 {
    let per_tree = if empty_tree_needs_root {
        TREE_TABLE_ENTRY_BYTES + NODE_BYTES + JOURNAL_NAMED_ENTRY_BYTES + MAPPING_ENTRY_NODE_BYTES
    } else {
        TREE_TABLE_ENTRY_BYTES
    };
    tree_count * per_tree
}

/// day-1 注册之后第 1 版树表还装不装得进一个单元。
fn tree_table_fits_in_first_unit(extra_trees: u64) -> bool {
    TREE_TABLE_FIRST_VERSION_ENTRIES + extra_trees <= TREE_TABLE_ENTRIES_PER_UNIT
}

fn emit(emitter: &mut Emitter, line: &str) {
    println!("{}", emitter.emit_raw(line));
}

fn main() {
    let mut emitter = Emitter::new();
    emit(&mut emitter, &format!(
        "name=config node_bytes={NODE_BYTES} header_without_key_range={HEADER_WITHOUT_KEY_RANGE} reserved={NONCE_MAC_RESERVED_BYTES} node_pointer={NODE_POINTER_BYTES} head_tree_identifier={HEAD_TREE_IDENTIFIER_BYTES} mapping_key_data={MAPPING_KEY_DATA_BYTES} mapping_key_node={MAPPING_KEY_NODE_BYTES} unit_bytes={UNIT_BYTES} tree_table_entry={TREE_TABLE_ENTRY_BYTES} data_units_per_node={DATA_UNITS_PER_NODE}"
    ));
    let mut height_grid_differences_from_legacy = 0u64;
    let mut pad_heights = Vec::new();
    let mut legacy_heights = Vec::new();
    for form in FORMS {
        let trees = form.trees();
        emit(&mut emitter, &format!(
            "name=form arm={} trees={} key_bytes={} value_bytes={} entry_bytes={}",
            form.name(), trees.len(), trees[0].1.key_bytes, trees[0].1.value_bytes, trees[0].1.entry_bytes()
        ));
        for (tree_name, shape) in &trees {
            emit(&mut emitter, &format!(
                "name=node arm={} tree={tree_name} key_bytes={} entry_bytes={} header_bytes={} payload_bytes={} leaf_fanout={} internal_fanout={}",
                form.name(), shape.key_bytes, shape.entry_bytes(), shape.header_bytes(), shape.payload_bytes(), shape.leaf_fanout(), shape.internal_fanout()
            ));
        }
        for heads in HEADS_SWEPT {
            for per_head in ENTRIES_PER_HEAD_SWEPT {
                let total = heads * per_head;
                let heights: Vec<u64> = trees.iter().zip(form.entries_per_tree(total)).map(|((_, shape), entries)| shape.height(entries)).collect();
                let tallest = *heights.iter().max().expect("每臂至少一棵树");
                let per_tree: Vec<String> = trees.iter().zip(&heights).map(|((tree_name, _), height)| format!("{tree_name}={height}")).collect();
                emit(&mut emitter, &format!(
                    "name=height arm={} heads={heads} entries_per_head={per_head} entries={total} height={tallest} per_tree={}",
                    form.name(), per_tree.join(",")
                ));
                match form {
                    EntryForm::PadEventInTag => pad_heights.push(tallest),
                    EntryForm::LegacyE130 => legacy_heights.push(tallest),
                    EntryForm::PadEventByte | EntryForm::TwoTreesEventInTag | EntryForm::PadEventInTagWithHint => {}
                }
            }
        }
        let prepaid = form.prepaid_bytes_per_free();
        emit(&mut emitter, &format!(
            "name=prepaid arm={} bytes_per_free={prepaid} unit_share_basis_points={}",
            form.name(), unit_share_basis_points(prepaid)
        ));
        let tree_count = trees.len() as u64;
        emit(&mut emitter, &format!(
            "name=first_transaction arm={} tree_table_entries={tree_count} day1_null_root_bytes={} day1_root_required_bytes={} lazy_bytes=0 tree_table_fits_first_unit={}",
            form.name(), first_transaction_day1_bytes(tree_count, false), first_transaction_day1_bytes(tree_count, true), tree_table_fits_in_first_unit(tree_count)
        ));
    }
    for (pad, legacy) in pad_heights.iter().zip(&legacy_heights) {
        if pad != legacy {
            height_grid_differences_from_legacy += 1;
        }
    }
    let pad = EntryForm::PadEventInTag.trees()[0].1;
    let with_event_byte = EntryForm::PadEventByte.trees()[0].1;
    emit(&mut emitter, &format!(
        "name=sensitive entries={SENSITIVE_ENTRY_COUNT} pad_event_in_tag_height={} pad_event_byte_height={}",
        pad.height(SENSITIVE_ENTRY_COUNT), with_event_byte.height(SENSITIVE_ENTRY_COUNT)
    ));
    emit(&mut emitter, &format!(
        "name=verdict pad_entry_bytes={} legacy_entry_bytes={LEGACY_E130_ENTRY_BYTES} pad_leaf_fanout={} legacy_leaf_fanout={} grid_cells={} cells_where_pad_taller_than_legacy={height_grid_differences_from_legacy} prepaid_pad={} prepaid_legacy={}",
        pad.entry_bytes(), pad.leaf_fanout(), EntryForm::LegacyE130.trees()[0].1.leaf_fanout(), HEADS_SWEPT.len() * ENTRIES_PER_HEAD_SWEPT.len(),
        EntryForm::PadEventInTag.prepaid_bytes_per_free(), EntryForm::LegacyE130.prepaid_bytes_per_free()
    ));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 绝对值断言 1：格式常量按 kb 逐段钉住。
    #[test]
    fn constants_are_the_knowledge_base_values() {
        assert_eq!(NODE_BYTES, 16384, "D8 已定项 2");
        assert_eq!(HEADER_WITHOUT_KEY_RANGE, 81, "D18 已定项 7");
        assert_eq!(NONCE_MAC_RESERVED_BYTES, 28, "D18 已定项 14");
        assert_eq!(NODE_POINTER_BYTES, 83, "D19 已定项 7");
        assert_eq!(HEAD_TREE_IDENTIFIER_BYTES + MAPPING_KEY_DATA_BYTES, 35, "头树 ID 8 + 码 1 映射 key 27");
        assert_eq!(HEAD_TREE_IDENTIFIER_BYTES + MAPPING_KEY_NODE_BYTES, 33, "头树 ID 8 + 码 2 / 3 映射 key 25");
        assert_eq!(LEGACY_E130_ENTRY_BYTES, 24, "E130 的作废形态");
        assert_eq!(EntryForm::PadEventByte.trees()[0].1.key_bytes, 36);
        assert_eq!(EntryForm::PadEventInTagWithHint.trees()[0].1.entry_bytes(), 49);
        assert_eq!(EntryForm::TwoTreesEventInTag.trees()[1].1.key_bytes, 33);
    }

    /// 绝对值断言 2：头宽、载荷、扇出按跑前登记的手算钉住，与装置的公式是两条路。
    #[test]
    fn fanouts_match_the_preregistered_hand_arithmetic() {
        let pad = EntryForm::PadEventInTag.trees()[0].1;
        assert_eq!(pad.header_bytes(), 81 + 70 + 28);
        assert_eq!(pad.payload_bytes(), 16205);
        assert_eq!(pad.leaf_fanout(), 463);
        assert_eq!(pad.internal_fanout(), 137, "16205 / (35 + 83)");
        let legacy = EntryForm::LegacyE130.trees()[0].1;
        assert_eq!(legacy.header_bytes(), 157);
        assert_eq!(legacy.payload_bytes(), 16227);
        assert_eq!(legacy.leaf_fanout(), 676);
        assert_eq!(legacy.internal_fanout(), 151, "16227 / (24 + 83)");
        assert_eq!(EntryForm::PadEventByte.trees()[0].1.leaf_fanout(), 450);
        let node_tree = EntryForm::TwoTreesEventInTag.trees()[1].1;
        assert_eq!((node_tree.header_bytes(), node_tree.payload_bytes(), node_tree.leaf_fanout(), node_tree.internal_fanout()), (175, 16209, 491, 139));
        assert_eq!(EntryForm::PadEventInTagWithHint.trees()[0].1.leaf_fanout(), 330);
    }

    /// 跑前登记的取样点：460 条时补齐臂一个叶、带事件字节的臂两个叶；65536 条时新形态高 3、作废形态高 2。
    #[test]
    fn sensitive_points_separate_the_arms() {
        let pad = EntryForm::PadEventInTag.trees()[0].1;
        let with_event_byte = EntryForm::PadEventByte.trees()[0].1;
        assert_eq!(pad.height(SENSITIVE_ENTRY_COUNT), 1);
        assert_eq!(with_event_byte.height(SENSITIVE_ENTRY_COUNT), 2);
        assert_eq!(pad.height(65536), 3, "142 个叶 > 内部扇出 137");
        assert_eq!(EntryForm::LegacyE130.trees()[0].1.height(65536), 2, "97 个叶 ≤ 151");
    }

    /// 树高从一个叶数起，且逐层按内部扇出除。
    #[test]
    fn tree_height_counts_levels_from_a_single_leaf() {
        assert_eq!(tree_height(0, 463, 137), 1);
        assert_eq!(tree_height(463, 463, 137), 1);
        assert_eq!(tree_height(464, 463, 137), 2);
        assert_eq!(tree_height(463 * 137, 463, 137), 2);
        assert_eq!(tree_height(463 * 137 + 1, 463, 137), 3);
        assert_eq!(tree_height(4096 * 1048576, 463, 137), 5, "9 276 388 个叶 → 67 711 → 495 → 4 → 1");
    }

    /// 两棵树那一臂按 8 : 1 分条目，节点那棵在最大格上只有 1/9。
    #[test]
    fn two_trees_split_entries_eight_to_one() {
        let split = EntryForm::TwoTreesEventInTag.entries_per_tree(4096 * 1048576);
        assert_eq!(split, vec![4_294_967_296 - 477_218_588, 477_218_588]);
        assert_eq!(EntryForm::PadEventInTag.entries_per_tree(900), vec![900]);
    }

    /// C260 要的数：每次 FREE 预付 = 叶条目宽，占 32 KiB 单元的万分点向下取整。
    #[test]
    fn prepaid_bytes_per_free_are_pinned() {
        assert_eq!(EntryForm::LegacyE130.prepaid_bytes_per_free(), 24);
        assert_eq!(EntryForm::PadEventInTag.prepaid_bytes_per_free(), 35);
        assert_eq!(EntryForm::PadEventByte.prepaid_bytes_per_free(), 36);
        assert_eq!(EntryForm::PadEventInTagWithHint.prepaid_bytes_per_free(), 49);
        assert_eq!(unit_share_basis_points(24), 7);
        assert_eq!(unit_share_basis_points(35), 10);
        assert_eq!(unit_share_basis_points(49), 14);
    }

    /// 第一个事务：day-1 注册只多一条树表条目，空树要根时再加一个节点、一个点名项、一条映射条目；两棵树翻倍；都装进第 1 版树表单元。
    #[test]
    fn first_transaction_bytes_are_pinned() {
        assert_eq!(first_transaction_day1_bytes(1, false), 200);
        assert_eq!(first_transaction_day1_bytes(1, true), 200 + 16384 + 56 + 53);
        assert_eq!(first_transaction_day1_bytes(2, true), 2 * (200 + 16384 + 56 + 53));
        assert!(tree_table_fits_in_first_unit(2));
        // 用 saturating_sub 而不是减法：变异把第 1 版条目数改得比每单元容量还大时，减法在常量求值期就溢出，
        // 那一条会被记成「无效变异」而不是被抓（`.claude/rules/mutation-sampling.md`「常量断言写加法，别写减法」）。
        assert!(!tree_table_fits_in_first_unit(
            TREE_TABLE_ENTRIES_PER_UNIT.saturating_sub(TREE_TABLE_FIRST_VERSION_ENTRIES) + 1
        ));
    }

    /// 阳性对照跑遍每一臂：每臂至少一棵树，叶扇出都大于内部扇出（叶条目比内部条目窄）。
    #[test]
    fn positive_control_runs_on_every_arm() {
        for form in FORMS {
            let trees = form.trees();
            assert!(!trees.is_empty(), "{:?}", form);
            for (_, shape) in trees {
                assert!(shape.leaf_fanout() > shape.internal_fanout(), "{:?}", form);
                assert!(shape.payload_bytes() < NODE_BYTES);
            }
        }
    }
}
