#!/usr/bin/env python3
"""往公共表里插一行：按锚点定位，写之前复核文件没被别人改过。

几个会话共写一个仓时，`replace-once.py` 只解决「改一处已有的文字」——**插入没有旧串可替**，
于是只能读整份、插一行、写回，两个会话一前一后就互相覆盖。2026-09-21 实测：两个会话各往
`.claude/kb/checks-owed.md` 插一行欠账，后写的那份把先写的那一行整行压掉，两边都没报错。

用法：
  insert-row.py 文件 --after 正则 --line 文本 [--absent 正则] [--dry-run]
  insert-row.py 文件 --before 正则 --line 文本 …
  insert-row.py --selftest

  --after / --before  锚点正则，必须在文件里**恰好命中一次**（命中 0 次或多次都拒绝，与 replace-once 同规矩）
  --absent            插之前先确认这个正则一处都没有，用来挡重复插入（例如已经有同编号的行）
写之前复核：读文件时记下 sha256，写之前再算一次，变了就拒绝并让人重跑——
别人在这中间写过，读到的内容已经不是要改的那一份。

写回不在原文件上就地写：同目录排他新建临时文件、fsync、照原权限位与属主、改名换上（`lib_atomic_replace.py`）；
写前复核排在临时文件写完之后、改名换上之前。正在按偏移边跑边读这份文件的进程读的仍是旧 inode 的旧内容。

退出码：0 插好了（或干跑）；1 用法不对或自检没过；2 --absent 已有命中；3 锚点不是恰好命中一次；
4 读到写之间文件被改过；5 没换上（有多个硬链接、建不了临时文件、写或改名失败）。2–5 都是原文件没动、临时文件已删。
"""
import argparse
import hashlib
import os
import re
import sys

from lib_atomic_replace import ReplaceRefused, leftover_temporary_files, problems_with_replacement_by_rename, \
    replace_file_contents_by_rename


class ChangedSinceRead(Exception):
    """写前复核：读到写之间文件的 sha256 变了。"""

    def __init__(self, digest_now):
        super().__init__(digest_now)
        self.digest_now = digest_now


def digest(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def insert(path, anchor, where, line, absent, dry_run):
    before = digest(path)
    text = open(path, encoding="utf-8").read()
    if absent:
        hits = len(re.findall(absent, text, re.M))
        if hits:
            print("  ✗ %s 里已经有 %d 处命中 --absent 的内容，不重复插" % (path, hits))
            print("     → 怎么办：先看那几处是不是你要插的这一行；确实要再插一行就换一个 --absent，或者去掉它。")
            return 2
    lines = text.split("\n")
    matched = [index for index, value in enumerate(lines) if re.search(anchor, value)]
    if len(matched) != 1:
        print("  ✗ 锚点在 %s 里命中 %d 次，只认恰好一次" % (path, len(matched)))
        print("     → 怎么办：把 --%s 的正则写得更具体些，让它只命中要插的那个位置。" % where)
        return 3
    at = matched[0] + (1 if where == "after" else 0)
    lines.insert(at, line)
    if dry_run:
        print("  （干跑）会插在 %s:%d，锚点是第 %d 行" % (path, at + 1, matched[0] + 1))
        return 0

    def verify_unchanged_since_read():
        # 每次现算，不复用读的时候那个值；排在临时文件写完之后、改名换上之前
        digest_now = digest(path)
        if digest_now != before:
            raise ChangedSinceRead(digest_now)

    try:
        notes = replace_file_contents_by_rename(path, "\n".join(lines), check_before_rename=verify_unchanged_since_read)
    except ChangedSinceRead as changed:
        print("  ✗ 读到写之间 %s 被改过（sha256 %s → %s），没写" % (path, before[:12], changed.digest_now[:12]))
        print("     → 怎么办：别的会话刚写过这份文件，重跑一次本命令；手里那份内容已经不是要改的那一份了。")
        return 4
    except ReplaceRefused as refused:
        print("  ✗ " + str(refused))
        # → 怎么办：str(refused) 的第二行就是下一步（lib_atomic_replace.py 里每个 ReplaceRefused 都带着），这里不重复打印
        return 5
    print("  ✓ 插在 %s:%d（锚点第 %d 行），写前复核 sha256 %s 未变，改名换上新 inode" % (path, at + 1, matched[0] + 1, before[:12]))
    for note in notes:
        print("  ! " + note)
    return 0


def selftest():
    import tempfile
    with tempfile.TemporaryDirectory() as work:
        path = os.path.join(work, "table.md")
        open(path, "w", encoding="utf-8").write("| A | 一 |\n| B | 二 |\n\n### 已还清\n")
        if insert(path, r"^### 已还清", "before", "| C | 三 |", None, False) != 0:
            print("  ✗ 自检失败：正常插入没成")
            print("     → 这个工具自己坏了，别再拿它改公共表。查 insert(...) 里 anchor 那条正则怎么编的，"
                  "以及 where 是 before 还是 after——自检用的锚点是 ^### 已还清，应当恰好命中一次。")
            return 1
        body = open(path, encoding="utf-8").read()
        if "| C | 三 |" not in body or body.index("| C | 三 |") > body.index("### 已还清"):
            print("  ✗ 自检失败：插的位置不对")
            print("     → 这个工具自己坏了，别再拿它改公共表。查 lines.insert() 那一行的下标："
                  "before 要插在锚点行之前、after 在之后，两者差 1。")
            return 1
        if insert(path, r"^### 已还清", "before", "| C | 三 |", r"^\| C \|", False) != 2:
            print("  ✗ 自检失败：--absent 没挡住重复插入")
            print("     → 这个工具自己坏了，别再拿它改公共表：挡不住重复插入意味着同一行会被插两遍。"
                  "查 absent 那条正则是不是拿去逐行匹配了，以及命中时有没有真的 return 2。")
            return 1
        if insert(path, r"^\| [AB] \|", "after", "| D | 四 |", None, False) != 3:
            print("  ✗ 自检失败：锚点命中两次时没拒绝")
            print("     → 这个工具自己坏了，别再拿它改公共表：锚点不唯一还照插，等于插在了没人指定的位置。"
                  "查数命中那一段是不是只找了第一处就停（应当数完全部行再判）。")
            return 1
        # 写前复核：让 digest 在读之后返回不同的值，模拟别的会话在这中间写过
        global digest
        original = digest
        state = {"n": 0}

        def racing(p):
            state["n"] += 1
            return original(p) if state["n"] == 1 else "0" * 64
        digest = racing
        code = insert(path, r"^### 已还清", "before", "| E | 五 |", None, False)
        digest = original
        if code != 4:
            print("  ✗ 自检失败：读到写之间文件被改过，却照写不误")
            print("     → 这个工具自己坏了，**别再拿它改任何公共表**：这一格坏掉意味着它会把别的会话"
                  "刚写进去的行整行压掉，而两边都不报错。查写之前那次 digest(path) 是不是真的重算了"
                  "（不能复用读的时候那个值），以及比对不等时有没有 return 4、有没有在 return 之前就写了。")
            return 1
        if "| E | 五 |" in open(path, encoding="utf-8").read():
            print("  ✗ 自检失败：复核判红了却还是写了进去")
            print("     → 这个工具自己坏了，**别再拿它改任何公共表**：复核判红之后文件仍被改，等于复核形同虚设。"
                  "查写文件那一步是不是排在复核之前——正确次序是先写临时文件、再算一次 sha256 比对相等、最后改名换上。")
            return 1
        if leftover_temporary_files(work):
            print("  ✗ 自检失败：复核判红之后还留着临时文件 %s" % leftover_temporary_files(work))
            print("     → 查 lib_atomic_replace.py 的 replace_file_contents_by_rename：check_before_rename 抛异常时，"
                  "except 那一段要先删临时文件再往外抛。")
            return 1
    # 写回的方式：inode 要换、在读的进程读旧内容、权限位与符号链接保住、硬链接拒绝、失败不留临时文件（判据在 lib_atomic_replace.py）
    table = "| 头 | 〇 |\n" * 20 + "| A | 一 |\n\n### 已还清\n"
    problems = problems_with_replacement_by_rename(
        lambda target: insert(target, r"^### 已还清", "before", "| C | 三 |", None, False),
        table, table.replace("\n### 已还清", "\n| C | 三 |\n### 已还清", 1))
    if problems:
        print("  ✗ 自检失败：写回的方式不对")
        for problem in problems:
            print("     " + problem)  # gate-lint:detail
        print("     → 这个工具自己坏了，别再拿它改脚本：就地写会让正在边跑边读那份脚本的进程读到别的内容。"
              "按方括号里那一格查 lib_atomic_replace.py 的 replace_file_contents_by_rename。")
        return 1
    print("  ✓ 自检：正常插入判绿、重复插入被 --absent 挡、锚点不唯一拒绝、读到写之间被改过拒绝且不写也不留临时文件；"
          "换上的是新 inode、权限位不变、改之前打开文件的读者接着读到旧内容、fsync 在改名之前、"
          "经符号链接改的是它指向的文件、有两个硬链接或当前用户只读都拒绝、fsync 或改名失败都不动原文件也不留临时文件")
    return 0


def main(argv):
    parser = argparse.ArgumentParser(add_help=True)
    parser.add_argument("file", nargs="?")
    parser.add_argument("--after")
    parser.add_argument("--before")
    parser.add_argument("--line")
    parser.add_argument("--absent")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--selftest", action="store_true")
    arguments = parser.parse_args(argv)
    if arguments.selftest:
        return selftest()
    if not arguments.file or not arguments.line or bool(arguments.after) == bool(arguments.before):
        print("  ✗ 用法：insert-row.py 文件 --after 正则 --line 文本（--after 与 --before 二选一）")
        print("     → 怎么办：看文件头的用法一节；自检跑 insert-row.py --selftest。")
        return 1
    where = "after" if arguments.after else "before"
    return insert(arguments.file, arguments.after or arguments.before, where,
                  arguments.line, arguments.absent, arguments.dry_run)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
