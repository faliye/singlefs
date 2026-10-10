#!/usr/bin/env python3
"""攻方副本上的变体：python3 apply_variant.py <tree> <variant>；原文必须恰好命中一次，否则退 3。"""
import sys, pathlib
VARIANTS = {
    "debug_units_listing": ("crates/singlefs-harness/src/model_comparison.rs",
        "    let mut records_of_roles: Vec<(ModelUnitRole, Vec<ObservedAllocationRecord>)> = units\n",
        "    {\n        let leaves: Vec<_> = units.iter().filter(|u| matches!(u.identity, TransactionUnit::InodeLeafContainer(_))).map(|u| (u.identity, u.slot)).collect();\n        if leaves.len() >= 3 { eprintln!(\"b29dbg txg={:?} leaf_units={:?} total_units={}\", root.checkpoint_txg, leaves, units.len()); }\n    }\n    let mut records_of_roles: Vec<(ModelUnitRole, Vec<ObservedAllocationRecord>)> = units\n"),
    "kx1_original_cap_at_second": ("crates/singlefs-harness/src/model_comparison.rs",
        "ModelUnitRole::inode_leaf_container_at(u64::from(index.0))",
        "ModelUnitRole::inode_leaf_container_at(u64::from(index.0).min(1))"),
    "kx2_original_cap_at_second": ("crates/singlefs-harness/src/model.rs",
        "let position_from_the_leftmost = records_before / INODE_LEAF_RECORDS;",
        "let position_from_the_leftmost = (records_before / INODE_LEAF_RECORDS).min(1);"),
    "drop_retain_original": ("crates/singlefs-harness/src/model.rs",
        "                role_written_at.retain(|role, _| {\n                    !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                });",
        "                let _ = &role_written_at;"),
    "kx1_cap_at_third": ("crates/singlefs-harness/src/model_comparison.rs",
        "ModelUnitRole::inode_leaf_container_at(u64::from(index.0))",
        "ModelUnitRole::inode_leaf_container_at(u64::from(index.0).min(2))"),
    "kx2_cap_at_third": ("crates/singlefs-harness/src/model.rs",
        "let position_from_the_leftmost = records_before / INODE_LEAF_RECORDS;",
        "let position_from_the_leftmost = (records_before / INODE_LEAF_RECORDS).min(2);"),
    "extend_only_the_second_leaf": ("crates/singlefs-harness/src/model.rs",
        "                        .filter(|(role, _)| {\n                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                        })",
        "                        .filter(|(role, _)| {\n                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(leaf) if leaf.position_from_the_leftmost.get() == 1)\n                        })"),
    "retain_drops_only_the_third_leaf": ("crates/singlefs-harness/src/model.rs",
        "                role_written_at.retain(|role, _| {\n                    !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                });",
        "                role_written_at.retain(|role, _| {\n                    !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(leaf) if leaf.position_from_the_leftmost.get() == 2)\n                });"),
}
tree, name = sys.argv[1], sys.argv[2]
path, old, new = VARIANTS[name]
p = pathlib.Path(tree) / path
text = p.read_text()
if text.count(old) != 1:
    print(f"variant={name} anchor_hits={text.count(old)}"); sys.exit(3)
p.write_text(text.replace(old, new))
print(f"variant={name} applied file={path}")
