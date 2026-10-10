#!/usr/bin/env python3
"""把一个变体按「原文恰好命中一次」换进副本（b30 攻方腿 M1）。用法：apply_variant.py <树根> <变体名>；--list 列出变体。"""
import sys

GLUE = "crates/singlefs-harness/src/model_comparison.rs"
MODEL = "crates/singlefs-harness/src/model.rs"
GLUE_ANCHOR = "            ModelUnitRole::inode_leaf_container_at(u64::from(index.0))\n"
MODEL_ANCHOR = "    let position_from_the_leftmost = records_before / INODE_LEAF_RECORDS;\n"
EXTEND_ANCHOR = ("                        .filter(|(role, _)| {\n"
                 "                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n"
                 "                        })\n")
RETAIN_ANCHOR = ("                role_written_at.retain(|role, _| {\n"
                 "                    !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n"
                 "                });\n")


def extend_with(condition):
    return ("                        .filter(|(role, _)| {\n"
            f"                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(leaf) if {condition})\n"
            "                        })\n")


def retain_with(condition):
    return ("                role_written_at.retain(|role, _| {\n"
            f"                    !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(leaf) if {condition})\n"
            "                });\n")


VARIANTS = {
    # 第 30 批入库的四行（B1、B2）
    "kx1_cap_at_third": (GLUE, GLUE_ANCHOR, GLUE_ANCHOR.replace("u64::from(index.0))", "u64::from(index.0).min(2))")),
    "kx2_cap_at_third": (MODEL, MODEL_ANCHOR, MODEL_ANCHOR.replace("INODE_LEAF_RECORDS;", "INODE_LEAF_RECORDS).min(2);").replace("= records_before", "= (records_before")),
    "retain_only_second": (MODEL, EXTEND_ANCHOR, extend_with("leaf.position_from_the_leftmost.get() == 1")),
    "drop_only_third": (MODEL, RETAIN_ANCHOR, retain_with("leaf.position_from_the_leftmost.get() == 2")),
    # 往后挪一片的四个（攻方新造）
    "glue_cap_at_fourth": (GLUE, GLUE_ANCHOR, GLUE_ANCHOR.replace("u64::from(index.0))", "u64::from(index.0).min(3))")),
    "model_cap_at_fourth": (MODEL, MODEL_ANCHOR, MODEL_ANCHOR.replace("INODE_LEAF_RECORDS;", "INODE_LEAF_RECORDS).min(3);").replace("= records_before", "= (records_before")),
    "retain_up_to_third": (MODEL, EXTEND_ANCHOR, extend_with("leaf.position_from_the_leftmost.get() <= 2")),
    "drop_up_to_fourth": (MODEL, RETAIN_ANCHOR, retain_with("leaf.position_from_the_leftmost.get() <= 3")),
}

if sys.argv[1:] == ["--list"]:
    print("\n".join(VARIANTS))
    sys.exit(0)
root, name = sys.argv[1], sys.argv[2]
path, anchor, replacement = VARIANTS[name]
full = f"{root}/{path}"
text = open(full, encoding="utf-8").read()
count = text.count(anchor)
if count != 1:
    sys.exit(f"{name}: 原文在 {path} 里命中 {count} 次")
open(full, "w", encoding="utf-8").write(text.replace(anchor, replacement))
print(f"applied {name} to {path}")
