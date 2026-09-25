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

写回不在原文件上就地写：同目录排他新建临时文件、fsync、照原权限位与属主、改名换上（`lib_atomic_replace.py`）。
正在按偏移边跑边读这份文件的进程（bash 跑脚本）读的仍是旧 inode 的旧内容。
换之前整批逐个文件预演一遍（临时文件写完、fsync 完，在改名之前停下、删掉）：有一个文件会被拒（多个硬链接、
当前用户只读、建不了临时文件、写或 fsync 失败），一个文件都不换，与「锚点不中一个文件都不写」同一条。

退出码：0 写成功（或 --dry-run 核过）；1 锚点命中数不对，一个文件都没写；2 用法不对；3 写完回读不一致；
4 没换上：预演时被拒，一个文件都没换；预演过了、真换时失败，已经换上的文件逐个列出。
"""
import json
import os
import pathlib
import sys
import tempfile

from lib_atomic_replace import ReplaceRefused, leftover_temporary_files, problems_with_replacement_by_rename, \
    replace_file_contents_by_rename


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


class RehearsalStoppedBeforeRename(Exception):
    """预演：临时文件写完、fsync 完，在改名换上之前停下；库删掉临时文件后把它原样抛出。"""


def rehearse_replacement(path, text):
    """走一遍整条换法、停在改名之前：拒绝与写临时文件时的失败在这里抛 ReplaceRefused，原文件一个字节没动。"""
    def stop_before_rename():
        raise RehearsalStoppedBeforeRename()
    try:
        replace_file_contents_by_rename(path, text, check_before_rename=stop_before_rename)
    except RehearsalStoppedBeforeRename:
        return


def replace_planned_texts(texts):
    """先整批预演，全过了再逐个改名换上。返回退出码；0 以外都已打印原因与下一步。"""
    for path, text in texts.items():
        try:
            rehearse_replacement(path, text)
        except ReplaceRefused as refused:
            print(f"✗ {refused}")
            print(f"→ 一个文件都没换（{len(texts)} 个文件逐个预演过，这一个会被拒）：处理完这个文件再跑同一份规格")
            return 4
    replaced_paths = []
    for path, text in texts.items():
        try:
            notes = replace_file_contents_by_rename(path, text)
        except ReplaceRefused as refused:
            print(f"✗ {refused}")
            print(f"→ 预演过了、真换时失败，这一批留了半套：已经换上的 {len(replaced_paths)} 个是 {replaced_paths}；"
                  "先核这几个的内容，再只拿剩下的几处重跑")
            return 4
        replaced_paths.append(path)
        for note in notes:
            print(f"! {note}")
    return 0


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
    exit_code = replace_planned_texts(texts)
    if exit_code != 0:
        return exit_code
    for path, text in texts.items():
        if pathlib.Path(path).read_text(encoding="utf-8") != text:
            print(f"✗ {path} 回读与内存里的结果不一致")
            print("→ 看是不是有别的进程同时在写这个文件，核一遍再跑")
            return 3
    print(f"✓ {len(edits)} 处替换，写了 {len(texts)} 个文件（改名换上新 inode），回读一致")
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
    # 写回的方式：inode 要换、在读的进程读旧内容、权限位与符号链接保住、硬链接与只读拒绝、失败不留临时文件（判据在 lib_atomic_replace.py）
    running_script = "#!/usr/bin/env bash\n" + "echo 前面这几行正在被 bash 边跑边读\n" * 20 + "run_step 甲\nexit $?\n"
    problems = problems_with_replacement_by_rename(
        lambda path: run([{"file": path, "old": "run_step 甲", "new": "run_step 戊"}], dry_run=False),
        running_script, running_script.replace("run_step 甲", "run_step 戊", 1))
    if problems:
        print("✗ 自证：写回的方式不对")
        for problem in problems:
            print(f"   {problem}")  # gate-lint:detail
        print("→ 按方括号里那一格查 lib_atomic_replace.py 的 replace_file_contents_by_rename，"
              "以及 replace_planned_texts() 是不是还有就地写的那一步")
        return 1
    # 整批预演：第二个文件有两个硬链接会被拒，第一个文件也不许先换上
    with tempfile.TemporaryDirectory() as directory:
        first = pathlib.Path(directory, "first.md")
        shared = pathlib.Path(directory, "shared.md")
        first.write_text("甲乙丙\n", encoding="utf-8")
        shared.write_text("子丑寅\n", encoding="utf-8")
        os.link(shared, pathlib.Path(directory, "other-name.md"))
        inode_before = first.stat().st_ino
        batch = [{"file": str(first), "old": "乙", "new": "B"}, {"file": str(shared), "old": "丑", "new": "C"}]
        exit_code = run(batch, dry_run=False)
        if exit_code != 4 or first.read_text(encoding="utf-8") != "甲乙丙\n" or first.stat().st_ino != inode_before \
                or shared.read_text(encoding="utf-8") != "子丑寅\n" or leftover_temporary_files(directory):
            print(f"✗ 自证：批里第二个文件会被拒（两个硬链接），退出码 {exit_code}（该是 4），"
                  "第一个文件被先换上了、或留了临时文件——这一批留了半套")
            print("→ 看 replace_planned_texts() 是不是先把整批逐个预演完、全过了才开始改名换上")
            return 1
    print("✓ replace-batch 自证：锚点不中时一个文件都不写，全中时按序写对，命中数不等就拒绝；"
          "换上的是新 inode、权限位不变、改之前打开文件的读者接着读到旧内容、fsync 在改名之前、"
          "经符号链接改的是它指向的文件、有两个硬链接或当前用户只读都拒绝、fsync 或改名失败都不动原文件也不留临时文件；"
          "批里有一个文件会被拒时一个文件都不换")
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
