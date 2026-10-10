#!/usr/bin/env python3
"""第 31 批代码轮第一轮攻方：在仓副本上打 / 撤一个坏实现（只改副本，不碰仓）。
用法：mutants.py <副本根> apply <名字> | restore | list
每个坏实现是一处或几处「旧串 → 新串」，旧串在文件里必须恰好命中一次；apply 之前把原文件存进 <副本根>/.b31-orig/，restore 照它写回。"""
import os
import shutil
import sys

RECORD_CALL = """    crate::mount::record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(
        &*pool.devices,
        pool.parameters,
        allocator,
        txg,
    );
"""
EARLY_ANCHOR = "    // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以这次重写的落点在装任何单元之前全部取定。\n"
LATE_COMMENT = "    // 记之前照 E2 重读挂载时读不出、记了这条根之后会是窗口里最小的那一槽（D16（发布语义） 已定项 1，用户 2026-10-03 定）。\n"

MUTANTS = {
    # 回收门槛高一代：多收一代（环里最旧有效根还引用着的也收）。
    "R1-floor-plus-one": [("crates/singlefs-core/src/allocator.rs",
        "self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);",
        "self.reclaim_released_records_up_to(CheckpointTxg(reclaim_floor.0 + 1), ReclaimedReuse::Immediately);")],
    # 回收门槛低一代：少收一代。
    "R2-floor-minus-one": [("crates/singlefs-core/src/allocator.rs",
        "self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);",
        "self.reclaim_released_records_up_to(CheckpointTxg(reclaim_floor.0.saturating_sub(1)), ReclaimedReuse::Immediately);")],
    # 释放代少记一：记录与盘上都写成 txg − 1，按谓词提早一代可再分配（用例的判法从同一份记录读释放代）。
    "R3-release-generation-minus-one": [("crates/singlefs-core/src/allocator.rs",
        "record.generation = release_generation;",
        "record.generation = CheckpointTxg(release_generation.0.saturating_sub(1));")],
    # 带单元的发布在取落点之前记根：这次记根回收出来的槽当场被这次发布取走，被盖掉的那条根落盘之前还引用着它们。
    "R4-record-the-root-before-taking-placements": [
        ("crates/singlefs-core/src/transaction.rs", LATE_COMMENT + RECORD_CALL, LATE_COMMENT),
        ("crates/singlefs-core/src/transaction.rs", EARLY_ANCHOR, RECORD_CALL + EARLY_ANCHOR),
    ],
    # 根环公式少用每区一槽（环长 21 而不是 24）：core 记根、写根与用例的模拟都走它，一起偏。
    "B3-target-for-publish-one-slot-short": [("crates/singlefs-core/src/root_ring.rs",
        "        slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),",
        "        slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % (slots_per_region.count() - 1),")],
}


def main() -> int:
    root, command = sys.argv[1], sys.argv[2]
    originals = os.path.join(root, ".b31-orig")
    if command == "list":
        print("\n".join(MUTANTS))
        return 0
    if command == "restore":
        if not os.path.isdir(originals):
            print("没有存过原文件")
            return 0
        for directory, _, files in os.walk(originals):
            for name in files:
                saved = os.path.join(directory, name)
                relative = os.path.relpath(saved, originals)
                # 写回之后把修改时间改成现在：照原样留旧的修改时间，cargo 会把坏实现编出来的产物当成新的、不重编（一次实测踩过）。
                target = os.path.join(root, relative)
                shutil.copyfile(saved, target)
                os.utime(target, None)
                print("restored", relative)
        shutil.rmtree(originals)
        return 0
    name = sys.argv[3]
    if os.path.isdir(originals):
        print("上一个坏实现还没撤，先 restore")
        return 2
    if name.startswith("row:"):
        # 仓副本 crates/mutations.tsv 第 N 行（六段：名字、文件、原文、替换文、cargo 参数、点名的测试；原文与替换文里 \\n 是换行）。
        line = open(os.path.join(root, "crates/mutations.tsv"), encoding="utf-8").read().split("\n")[int(name[4:]) - 1]
        fields = line.split("\t")
        changes = [(fields[1], fields[2].replace("\\n", "\n"), fields[3].replace("\\n", "\n"))]
        print("row", name, fields[0])
    else:
        changes = MUTANTS[name]
    for relative, old, new in changes:
        path = os.path.join(root, relative)
        saved = os.path.join(originals, relative)
        if not os.path.exists(saved):
            os.makedirs(os.path.dirname(saved), exist_ok=True)
            shutil.copy2(path, saved)
        text = open(path, encoding="utf-8").read()
        count = text.count(old)
        if count != 1:
            print(f"{name}: {relative} 旧串命中 {count} 次")
            return 3
        open(path, "w", encoding="utf-8").write(text.replace(old, new))
        print("applied", name, relative)
    return 0


if __name__ == "__main__":
    sys.exit(main())
