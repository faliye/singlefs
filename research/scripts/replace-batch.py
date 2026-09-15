#!/usr/bin/env python3
"""批量定点替换：规格里的每一处都先在内存里核「恰好命中 count 次」（默认 1），全过了才写盘并回读。

用法：
  replace-batch.py 规格.json            # 核完再写
  replace-batch.py --dry-run 规格.json  # 只核不写
  replace-batch.py --selftest           # 自证会红

规格是一个 JSON 数组，每项 {"file": 路径, "old": 旧串, "new": 新串, "count": 可省，默认 1}；
同一个文件的多处按数组顺序在内存里依次施加，后一处的旧串按前一处改完的文本核。

为什么两阶段：逐处写盘的批量替换在第 k 处锚点不中时，前 k − 1 处已经落盘，留下半套改动。
2026-09-14 实测：一次 54 处的 kb 写回里有一处原文是「没被」而锚点写成「没有被」，两阶段让它一个文件都没写。
REPLACE_BATCH_WRITE_EACH=1 强制走回逐处写盘的旧行为，--selftest 在它下面必须判红。
"""
import json
import os
import pathlib
import sys
import tempfile


def plan_in_memory(edits):
    texts = {}
    for index, edit in enumerate(edits, 1):
        path = edit["file"]
        expected_hits = edit.get("count", 1)
        if path not in texts:
            texts[path] = pathlib.Path(path).read_text(encoding="utf-8")
        hits = texts[path].count(edit["old"])
        if hits != expected_hits:
            return None, f"第 {index} 处 {path}：期望命中 {expected_hits} 次，实际 {hits}：{edit['old'][:60]!r}"
        texts[path] = texts[path].replace(edit["old"], edit["new"])
    return texts, None


def apply_each_edit_to_disk(edits):
    """旧行为：逐处写盘。只给自证强制走一遍，证明它会留下半套改动。"""
    for index, edit in enumerate(edits, 1):
        path = pathlib.Path(edit["file"])
        text = path.read_text(encoding="utf-8")
        expected_hits = edit.get("count", 1)
        hits = text.count(edit["old"])
        if hits != expected_hits:
            return f"第 {index} 处 {edit['file']}：期望命中 {expected_hits} 次，实际 {hits}"
        path.write_text(text.replace(edit["old"], edit["new"]), encoding="utf-8")
    return None


def run(edits, dry_run):
    if os.environ.get("REPLACE_BATCH_WRITE_EACH") == "1":
        error = apply_each_edit_to_disk(edits)
        if error:
            print(f"✗ {error}")
            print("→ 对一下原文再跑（这是逐处写盘的旧行为，前面几处已经落盘了）")
            return 1
        return 0
    texts, error = plan_in_memory(edits)
    if error:
        print(f"✗ {error}")
        print("→ 什么都没写：对一下这一处的原文（空白、全半角、同义字），改规格再跑")
        return 1
    if dry_run:
        print(f"✓ {len(edits)} 处都命中得对（{len(texts)} 个文件），--dry-run 没写")
        return 0
    for path, text in texts.items():
        pathlib.Path(path).write_text(text, encoding="utf-8")
    for path, text in texts.items():
        if pathlib.Path(path).read_text(encoding="utf-8") != text:
            print(f"✗ {path} 回读与内存里的结果不一致")
            print("→ 看是不是有别的进程同时在写这个文件，核一遍再跑")
            return 3
    print(f"✓ {len(edits)} 处替换，写了 {len(texts)} 个文件，回读一致")
    return 0


def selftest():
    with tempfile.TemporaryDirectory() as directory:
        first = pathlib.Path(directory, "first.md")
        second = pathlib.Path(directory, "second.md")
        first.write_text("甲乙丙\n", encoding="utf-8")
        second.write_text("子丑寅\n", encoding="utf-8")
        before = (first.read_text(encoding="utf-8"), second.read_text(encoding="utf-8"))
        broken_batch = [{"file": str(first), "old": "乙", "new": "B"}, {"file": str(second), "old": "卯", "new": "M"}]
        exit_code = run(broken_batch, dry_run=False)
        after = (first.read_text(encoding="utf-8"), second.read_text(encoding="utf-8"))
        if exit_code == 0 or after != before:
            print("✗ 自证：第二处锚点不中时第一处已经落盘（或退出码是 0）——两阶段没有生效")
            print("→ 看 run() 是不是逐处写盘了；REPLACE_BATCH_WRITE_EACH=1 就是故意走这条路")
            return 1
        good_batch = [{"file": str(first), "old": "乙", "new": "B"}, {"file": str(first), "old": "甲B", "new": "AB"},
                      {"file": str(second), "old": "丑", "new": "C", "count": 1}]
        if run(good_batch, dry_run=False) != 0 or first.read_text(encoding="utf-8") != "AB丙\n" or second.read_text(encoding="utf-8") != "子C寅\n":
            print("✗ 自证：全部命中的一批没有按顺序写对")
            print("→ 看同一个文件的多处是不是按数组顺序在内存里依次施加")
            return 1
        if run([{"file": str(second), "old": "子", "new": "Z", "count": 2}], dry_run=False) == 0:
            print("✗ 自证：期望命中 2 次、实际 1 次也放行了")
            print("→ 看 plan_in_memory() 是不是按 count 比命中数")
            return 1
    print("✓ replace-batch 自证：锚点不中时一个文件都不写，全中时按序写对，命中数不等就拒绝")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    dry_run = "--dry-run" in arguments
    positional = [argument for argument in arguments if argument != "--dry-run"]
    if len(positional) != 1:
        print("✗ 用法：replace-batch.py [--dry-run] 规格.json，或 replace-batch.py --selftest")
        print("→ 规格是 JSON 数组，每项 {\"file\", \"old\", \"new\", \"count\"（可省，默认 1）}")
        return 2
    edits = json.loads(pathlib.Path(positional[0]).read_text(encoding="utf-8"))
    return run(edits, dry_run)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
