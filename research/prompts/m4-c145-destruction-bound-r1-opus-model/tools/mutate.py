#!/usr/bin/env python3
"""在一棵仓副本上打一个点名的变体（逐字替换，旧串必须恰好命中一次），或列出全部变体。
用法：mutate.py <树根> <变体名>      mutate.py --list
"""
import os
import sys

TRANSACTION = "crates/singlefs-core/src/transaction.rs"
WALK = "crates/singlefs-checker/src/walk.rs"
HEAD_ARM = "        Some(NextSideOfASnapshot::LiveHead) | None => hi_when_the_next_side_is_the_live_head,"
PENDING_BOUND = "                && upper_bound.is_none_or(|bound| entry.epoch < bound)"

MUTANTS = {
    # core：下一侧是活头时 hi 取「已发布的最大号」（这一批之前那一版的 txg），C145 判别力自证那一句的实现版。
    "core_hi_published_at_the_batch": (
        TRANSACTION,
        HEAD_ARM,
        "        Some(NextSideOfASnapshot::LiveHead) | None => CheckpointTxg(hi_when_the_next_side_is_the_live_head.0 - 1),",
    ),
    # core 对照：区间收成 [lo, lo + 1)，只放纪元 == lo 的条目。
    "core_hi_lo_plus_one": (
        TRANSACTION,
        HEAD_ARM,
        "        Some(NextSideOfASnapshot::LiveHead) | None => CheckpointTxg(lo.0 + 1),",
    ),
    # checker：下一个活侧是活头时待放集的上界从 ∞ 收成 lo + 1。
    "checker_bound_lo_plus_one": (
        WALK,
        PENDING_BOUND,
        "                && upper_bound.map_or(entry.epoch < destroyed.snapshot_txg + 1, |bound| entry.epoch < bound)",
    ),
}


def main(argv):
    if len(argv) == 2 and argv[1] == "--list":
        for name in MUTANTS:
            print(name)
        return 0
    if len(argv) != 3 or argv[2] not in MUTANTS:
        print(__doc__, file=sys.stderr)
        return 2
    root, name = argv[1], argv[2]
    relative, old, new = MUTANTS[name]
    path = os.path.join(root, relative)
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    hits = text.count(old)
    if hits != 1:
        print(f"✗ {name}：旧串在 {relative} 命中 {hits} 次，要恰好一次", file=sys.stderr)
        return 1
    with open(path + ".mutating", "w", encoding="utf-8") as handle:
        handle.write(text.replace(old, new, 1))
    os.replace(path + ".mutating", path)
    print(f"name=mutant_applied mutant={name} file={relative}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
