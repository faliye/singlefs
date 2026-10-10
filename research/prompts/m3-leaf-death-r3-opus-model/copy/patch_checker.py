#!/usr/bin/env python3
"""m3-leaf-death-r3 攻方腿：给仓副本的池级 checker（CPU，crates/singlefs-checker/src/walk.rs）加三个运行时开关，只改副本。
用法：python3 patch_checker.py <副本里的 walk.rs>
开关（环境变量，不设就是今天的 checker 原样）：
  OPUS_R3_DROP_I94_THIRD=1   删 I-9.4 第三句（左容器最大 key < 右容器号），K3 / N1 新形那一形。
  OPUS_R3_RANGE=r1           区间墓碑最小改法甲：I-9.19、I-9.20 不重号只判整对象墓碑；I-9.20「第一版只许整对象」换成「区间起 < 区间止」。
  OPUS_R3_RANGE=r2           区间墓碑最小改法乙：同上，「第一版只许整对象」换成「区间起 < 区间止 且 区间止 = 2⁶⁴ − 1」（只许尾段）。
每处替换在给定函数体里恰好命中一次，否则退出 3。
"""
import sys

path = sys.argv[1]
src = open(path, encoding="utf-8").read()


def replace_in_function(text, function_marker, old, new):
    start = text.find(function_marker)
    if start < 0 or text.count(function_marker) != 1:
        sys.exit(f"函数标记命中 {text.count(function_marker)} 次：{function_marker}")
    end = text.find("\nfn ", start + len(function_marker))
    end = len(text) if end < 0 else end
    body = text[start:end]
    if body.count(old) != 1:
        sys.exit(3 if body.count(old) != 1 else 0)
    return text[:start] + body.replace(old, new) + text[end:]


# K3：I-9.4 第三句。
src = replace_in_function(
    src,
    "fn judge_container_numbers_along_the_leaf_order(",
    '.judge("I-9.4", left.largest_inode < right.container, || {',
    '.judge("I-9.4", std::env::var_os("OPUS_R3_DROP_I94_THIRD").is_some() || left.largest_inode < right.container, || {',
)

RANGE = 'std::env::var("OPUS_R3_RANGE").ok()'
# I-9.19：只判整对象墓碑。
src = replace_in_function(
    src,
    "fn judge_tombstones_on_the_lineage_name_no_live_object(",
    ".filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))",
    f".filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))\n        .filter(|tombstone| {RANGE}.is_none() || tombstone.covers_the_whole_object())",
)
# I-9.20 不重号：只判整对象墓碑。
src = replace_in_function(
    src,
    "fn judge_tombstones_on_the_lineage_of_a_version_are_distinct(",
    ".filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))",
    f".filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))\n        .filter(|tombstone| {RANGE}.is_none() || tombstone.covers_the_whole_object())",
)
# I-9.20 记录合法：「第一版只许整对象」换掉。
src = replace_in_function(
    src,
    "fn judge_every_tombstone_of_a_version_is_well_formed(",
    "judgements.judge(invariant, tombstone.covers_the_whole_object(), || {",
    "judgements.judge(invariant, match " + RANGE + ".as_deref() {\n"
    "                Some(\"r1\") => tombstone.range_start < tombstone.range_end,\n"
    "                Some(\"r2\") => tombstone.range_start < tombstone.range_end && tombstone.range_end == TOMBSTONE_WHOLE_OBJECT_RANGE_END,\n"
    "                _ => tombstone.covers_the_whole_object(),\n"
    "            }, || {",
)
open(path, "w", encoding="utf-8").write(src)
print("patched", path)
