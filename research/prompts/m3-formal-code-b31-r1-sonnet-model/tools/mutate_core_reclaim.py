#!/usr/bin/env python3
"""腿自己的 N2 / N3 变异（只在草稿副本上改，不进仓）：改 core 记根时回收门槛那一段的几种错法，看第 31 批新加的两条用例
（root_ring_turn_in_the_make_filesystem_session 与 overwrite_in_one_instance 里改了判法的那条）红不红。
用法：mutate_core_reclaim.py <草稿副本根> <变异号|restore>   变异号：a b c d e；restore 把 allocator.rs 还成冻结副本那一份
"""
import shutil
import sys

tree, which = sys.argv[1], sys.argv[2]
frozen = "/tmp/claude-1000/m3-formal-code-b31-r1/tree/crates/singlefs-core/src/allocator.rs"
target = tree + "/crates/singlefs-core/src/allocator.rs"
if which == "restore":
    shutil.copyfile(frozen, target)
    sys.exit(0)
source = open(frozen, encoding="utf-8").read()
anchor = """        let reclaim_floor = crate::mount::reclaim_floor(
            occupancy.effective_floor,
            occupancy.oldest_valid_root_in_the_ring(),
        );
"""
assert source.count(anchor) == 1
replacements = {
    # a：门槛多一代（比谓词严一格：释放代 = 最旧有效根 + 1 的也回收）
    "a": anchor + "        let reclaim_floor = CheckpointTxg(reclaim_floor.0 + 1);\n",
    # b：门槛少一代（比谓词宽一格：释放代 = 最旧有效根 的不回收）
    "b": anchor + "        let reclaim_floor = CheckpointTxg(reclaim_floor.0.saturating_sub(1));\n",
    # c：只在环转过之后才按环里最旧有效根回收，转之前（最旧有效根是第 0 代）提前拿 txg - 2 当门槛（提前回收，转过之后与正确的相同）
    "c": anchor + "        let reclaim_floor = if reclaim_floor.0 == 0 { CheckpointTxg(checkpoint_txg.0.saturating_sub(2)) } else { reclaim_floor };\n",
    # d：环转之前只提前回收释放代 3（第一个文件换下的 mkfs 树表，转过之后本来就该回收的那一槽）
    "d": anchor + "        let reclaim_floor = if reclaim_floor.0 == 0 { CheckpointTxg(3) } else { reclaim_floor };\n",
    # e：门槛不看 F，只看环里最旧有效根（F 恒 0 的会话里与正确的相同）
    "e": anchor.replace("occupancy.effective_floor,", "CheckpointTxg(0),"),
}
open(target, "w", encoding="utf-8").write(source.replace(anchor, replacements[which]))
