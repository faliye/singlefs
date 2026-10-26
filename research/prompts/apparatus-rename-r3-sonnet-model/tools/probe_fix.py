#!/usr/bin/env python3
"""J1（零轮）：把本腿提的两条收严写成不改库的原型，在本腿造的几份样本上跑，看它们咬不咬得到、会不会误伤真搬迁。没被攻过。
收严 X：不在基准那一版登记表里的新行，必须此刻生效（窗口开着）才不判红——从来不生效的新行（新前缀早就有文件、旧前缀没有文件、已搬完的假行）判红。
收严 Y：基准那一版登记表里已有的行不再生效（老行用过了窗口就作废），睡着的老行不会因为基准树的形状变了而醒来。
原型只读：用 git ls-tree 自己数窗口，不调库里的换算。每个局面在造好的那一刻就判，不等后面的提交。
用法：probe_fix.py --frozen <冻结树> --work <新的草稿目录>"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import Repo, TABLE, row  # noqa: E402


def under(path, prefix):
    return path == prefix or path.startswith(prefix + "/")


def table_rows(text):
    return [tuple(line.split("\t")[:2]) for line in text.split("\n") if line.strip() and not line.lstrip().startswith("#")]


def record(label, repo, base):
    files = repo.git("ls-tree", "-r", "--name-only", base).stdout.split("\n")
    shown = repo.git("show", f"{base}:{TABLE}", check=False)
    in_base = set(table_rows(shown.stdout)) if shown.returncode == 0 else set()
    print(f"name={label}")
    for old, new in table_rows(repo.read(TABLE)):
        window = any(under(f, old) for f in files) and not any(under(f, new) for f in files)
        known = (old, new) in in_base
        print(f"    {old}->{new}: window_open={window} row_in_base_table={known} 收严X判红={(not known) and (not window)} 收严Y后仍生效={window and not known}")


def sample(work, frozen, name, files):
    repo = Repo(os.path.join(work, name), frozen)
    for relative, text in files.items():
        repo.write(relative, text)
    repo.write(TABLE, "# 空\n")
    repo.commit("基准")
    return repo


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    os.makedirs(arguments.work, exist_ok=True)
    w, f = arguments.work, arguments.frozen
    repo = sample(w, f, "occupied", {"tools/check_a/run.sh": "echo a\n", "tools/check_b/run.sh": "echo b\n"})
    repo.register(row("tools/check_a", "tools/check_b"))
    record("occupied_row（新前缀早就有文件）", repo, "HEAD")
    repo = sample(w, f, "token", {"frozen/j.py": "RETRIES = 3\n"})
    repo.register(row("3", "4"))
    record("token_row（旧前缀根本不存在）", repo, "HEAD")
    repo = sample(w, f, "landed_forged", {"tools/check_b/run.sh": "echo b\n"})
    repo.register(row("tools/check_a", "tools/check_b"))
    record("landed_looking_forged_row（新前缀有、旧前缀无、从没搬过）", repo, "HEAD")
    repo = sample(w, f, "real", {"tools/check_a/run.sh": "echo a\n"})
    repo.git("mv", "tools/check_a", "tools/check_c")
    repo.register(row("tools/check_a", "tools/check_c"))
    record("real_move_uncommitted（真搬迁，基准是搬迁之前）", repo, "HEAD")
    repo.commit("搬迁已提交")
    record("real_move_after_commit（老行已在基准表里，窗口已关）", repo, "HEAD")
    repo = sample(w, f, "stale", {"rig/a.sh": "echo a\n"})
    repo.git("mv", "rig", "rig2")
    repo.register(row("rig", "rig2"))
    repo.commit("搬迁已提交")
    repo.git("mv", "rig2", "rig3")
    repo.write("rig/new.sh", "echo 旧名字另起一份\n")
    repo.commit("新目录又改名、旧名字重新用起来")
    record("stale_row_wakes_up（老行睡着后醒来）", repo, "HEAD")
    remote = Repo(os.path.join(w, "unpushed"), f, upstream=True)
    remote.write("tools/old/a.sh", "echo a\n")
    remote.write(TABLE, "# 空\n")
    s0 = remote.commit("S0", push=True)
    remote.git("mv", "tools/old", "tools/new")
    remote.register(row("tools/old", "tools/new"))
    remote.commit("搬迁已提交未推")
    record("real_move_committed_unpushed（gate 基准是 S0）", remote, s0)


main()
