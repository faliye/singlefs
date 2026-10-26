#!/usr/bin/env python3
"""J1：钉值那一侧按全部登记行倒换、门禁一侧只认生效行，对同一份内容判得一不一致。
用法：probe_pinned.py --frozen <冻结树> --work <新的草稿目录> [--repo <仓根>]
每个局面打 name=… 一行：门禁一侧（path_move_report 的问题数、改动范围里 moved_only_paths 剔了哪些）与钉值一侧
（frozen_o3.RegisteredMoves.contents_before_moves 倒出来的几版里有没有钉住的那一版；给了 --repo 的另用 reuse_library.pinned_sha256_now 核）。"""
import argparse
import hashlib
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import Repo, load_library, row, short  # noqa: E402

E171 = "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes"


def sha(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def pin_side(frozen_o3, root, relative, pinned):
    moves = frozen_o3.registered_moves(root)
    with open(os.path.join(root, relative), "rb") as handle:
        data = handle.read()
    return any(hashlib.sha256(version).hexdigest() == pinned for version in moves.contents_before_moves(data))


def build(work, frozen, name, original, files):
    repo = Repo(os.path.join(work, name), frozen)
    for relative, text in files.items():
        repo.write(relative, text)
    repo.write("frozen/judge.py", original)
    repo.write(".claude/gate.d/path-moves.tsv", "# 空\n")
    repo.commit("基准")
    return repo


def report(frozen_o3, lag, repo, name, pinned, edit_text, rows, reuse=None, break_window=False):
    repo.write("frozen/judge.py", edit_text)
    if rows:
        repo.register(*rows)
    base, _ = repo.base(lag)
    if break_window:
        os.environ["LAG_BASELINE_BREAK"] = "window-ignored"
    count, problems, note = lag.path_move_report(repo.root, base)
    changed = repo.changed(base)
    moved, _problem = lag.moved_only_paths(base, changed, cwd=repo.root)
    os.environ.pop("LAG_BASELINE_BREAK", None)
    gate_kept = [path for path in changed if path not in moved]
    side = pin_side(frozen_o3, repo.root, "frozen/judge.py", pinned)
    extra = ""
    if reuse is not None:
        extra = f" reuse_library_pinned_accepts={reuse.pinned_sha256_now(os.path.join(repo.root, 'frozen/judge.py'), pinned, repo.root) is not None}"
    print(f"name={name} gate_problems={len(problems)} gate_note={note!r} gate_changed_after_filter={short(gate_kept)} pin_side_accepts={side}{extra}")
    for what, _how in problems:
        print(f"    gate_problem: {what}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    parser.add_argument("--repo")
    arguments = parser.parse_args()
    lag = load_library(arguments.frozen)
    frozen_o3 = load_library(arguments.frozen, "frozen_o3_copy", f"{E171}/frozen_o3.py")
    reuse = None
    if arguments.repo:
        sys.dont_write_bytecode = True
        sys.path.insert(0, os.path.join(arguments.repo, E171))
        import reuse_library as reuse  # noqa: E402
    os.makedirs(arguments.work, exist_ok=True)
    original = "import subprocess\nsubprocess.run(['bash', 'tools/check_a/run.sh'])\n"
    both = {"tools/check_a/run.sh": "echo a\n", "tools/check_b/run.sh": "echo b\n"}
    edited = original.replace("check_a", "check_b")
    # 对照：改了不登记，钉值一侧不认
    repo = build(arguments.work, arguments.frozen, "control_unregistered", original, both)
    report(frozen_o3, lag, repo, "control_unregistered", sha(original), edited, [], reuse)
    # 新前缀早就有文件的行：门禁一侧不生效、不核；钉值一侧照用
    repo = build(arguments.work, arguments.frozen, "occupied_row", original, both)
    report(frozen_o3, lag, repo, "occupied_row", sha(original), edited, [row("tools/check_a", "tools/check_b")], reuse)
    repo = build(arguments.work, arguments.frozen, "occupied_row_window_ignored", original, both)
    report(frozen_o3, lag, repo, "occupied_row_window_ignored", sha(original), edited, [row("tools/check_a", "tools/check_b")], None, break_window=True)
    # 旧前缀在基准里根本没有文件、什么都没搬：单个词当前缀
    token_original = "RETRIES = 3\nLIMIT = 'a'\n"
    repo = build(arguments.work, arguments.frozen, "token_row", token_original, {})
    report(frozen_o3, lag, repo, "token_row", sha(token_original), token_original.replace("3", "4"), [row("3", "4")], reuse)
    # 真搬迁（对照）：目录搬了家，调用方只差前缀
    moved_original = "import subprocess\nsubprocess.run(['bash', 'tools/check_a/run.sh'])\n"
    repo = build(arguments.work, arguments.frozen, "real_move", moved_original, {"tools/check_a/run.sh": "echo a\n"})
    repo.git("mv", "tools/check_a", "tools/check_c")
    report(frozen_o3, lag, repo, "real_move", sha(moved_original), moved_original.replace("check_a", "check_c"), [row("tools/check_a", "tools/check_c")], reuse)
    repo.commit("搬迁提交，无上游：之后 gate 基准是 HEAD")
    chain_case(lag, frozen_o3, arguments.work, arguments.frozen, reuse)
    report(frozen_o3, lag, repo, "real_move_after_commit", sha(moved_original), moved_original.replace("check_a", "check_c"), [], reuse)


def chain_case(lag, frozen_o3, work, frozen, reuse):
    """老行 legacy->app 早就提交（legacy 已不在、app 是早就在的另一份实现）；新行 rigs/y->legacy 是一次真搬迁（实据齐、门禁一侧绿、老行不生效）。
    钉住的文件原来指 rigs/y/run.sh，被改成指 app/run.sh：门禁一侧这一处照算改动，钉值一侧按全部行逐次倒回去，倒到的正是钉住的那一版。"""
    original = "import subprocess\nsubprocess.run([\"bash\", \"rigs/y/run.sh\"])\n"
    repo = Repo(os.path.join(work, "chain_old_row"), frozen)
    repo.write("app/run.sh", "echo 早就在的另一份实现\n")
    repo.write("rigs/y/run.sh", "echo rigs y\n")
    repo.write("frozen/judge.py", original)
    repo.register(row("legacy", "app"))
    repo.commit("基准（老行已提交）")
    repo.register(row("rigs/y", "legacy"), row("legacy", "app"))
    repo.git("mv", "rigs/y", "legacy")
    report(frozen_o3, lag, repo, "chain_old_row", sha(original), original.replace("rigs/y", "app"), [], reuse)


main()
