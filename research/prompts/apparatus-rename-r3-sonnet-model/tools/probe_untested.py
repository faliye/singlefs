#!/usr/bin/env python3
"""J3：自证从没跑到的几处（符号链接、没合并完的索引条目、搬迁行里的单个文件被换成目录），在这一腿的样本上跑一遍，看判得对不对。
覆盖面来自 python3 -m trace --count --missing 跑库自证的 .cover 文件（命令见 rerun.sh）。
用法：probe_untested.py --frozen <冻结树> --work <新的草稿目录>"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import Repo, load_library, row, short  # noqa: E402


def judge(lag, repo, label, expect_note=""):
    count, problems, note = lag.path_move_report(repo.root, "HEAD")
    changed = repo.changed("HEAD")
    moved, problem = lag.moved_only_paths("HEAD", changed, cwd=repo.root)
    print(f"name={label} table_problems={len(problems)} dropped={short(sorted(moved))} kept={short(p for p in changed if p not in moved and 'path-moves' not in p and 'records/' not in p)} problem={problem}")
    for what, _how in problems[:2]:
        print(f"    problem: {what[:160]}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    lag = load_library(arguments.frozen)
    os.makedirs(arguments.work, exist_ok=True)
    w, f = arguments.work, arguments.frozen
    # 符号链接：相对链接（目标不含前缀）、链接目标里写着旧前缀、链接指向被搬走的文件
    repo = Repo(os.path.join(w, "symlink"), f)
    repo.write("rig/a.sh", "echo a\n")
    os.symlink("a.sh", os.path.join(repo.root, "rig/rel-link"))
    os.symlink("../rig/a.sh", os.path.join(repo.root, "rig/path-link"))
    repo.write(".claude/gate.d/path-moves.tsv", "# 空\n")
    repo.commit("基准")
    repo.git("mv", "rig", "rig2")
    os.remove(os.path.join(repo.root, "rig2/path-link"))
    os.symlink("../rig2/a.sh", os.path.join(repo.root, "rig2/path-link"))
    repo.register(row("rig", "rig2"))
    judge(lag, repo, "symlink_dir_move_link_target_rewritten")
    # 同一件，链接目标没跟着改（仍指向旧前缀）：应当算改动
    repo = Repo(os.path.join(w, "symlink_stale"), f)
    repo.write("rig/a.sh", "echo a\n")
    os.symlink("../rig/a.sh", os.path.join(repo.root, "rig/path-link"))
    repo.write(".claude/gate.d/path-moves.tsv", "# 空\n")
    repo.commit("基准")
    repo.git("mv", "rig", "rig2")
    repo.register(row("rig", "rig2"))
    judge(lag, repo, "symlink_dir_move_link_target_untouched")
    # 搬迁后把一个文件换成指向别处的符号链接（模式变了）
    repo = Repo(os.path.join(w, "symlink_swap"), f)
    repo.write("rig/a.sh", "echo a\n")
    repo.write("elsewhere/evil.sh", "echo evil\n")
    repo.write(".claude/gate.d/path-moves.tsv", "# 空\n")
    repo.commit("基准")
    repo.git("mv", "rig", "rig2")
    os.remove(os.path.join(repo.root, "rig2/a.sh"))
    os.symlink("../elsewhere/evil.sh", os.path.join(repo.root, "rig2/a.sh"))
    repo.register(row("rig", "rig2"))
    judge(lag, repo, "file_replaced_by_symlink_after_move")
    # 搬迁后执行位变了
    repo = Repo(os.path.join(w, "mode"), f)
    repo.write("rig/a.sh", "echo a\n")
    repo.write(".claude/gate.d/path-moves.tsv", "# 空\n")
    repo.commit("基准")
    repo.git("mv", "rig", "rig2")
    os.chmod(os.path.join(repo.root, "rig2/a.sh"), 0o755)
    repo.register(row("rig", "rig2"))
    judge(lag, repo, "exec_bit_added_after_move")


main()
