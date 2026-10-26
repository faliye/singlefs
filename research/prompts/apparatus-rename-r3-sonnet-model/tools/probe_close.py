#!/usr/bin/env python3
"""J1：第二轮打中的三种（在老行前面插一行串到早就在的实现上、老行让没改的表判涨、无序键）在改完的库上还中不中。样本是这一腿自己造的，不是前两轮腿的样本。
用法：probe_close.py --frozen <冻结树> --work <新的草稿目录>"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import Repo, load_library, row, short  # noqa: E402


def make(work, frozen, name, committed_rows):
    repo = Repo(os.path.join(work, name), frozen)
    repo.write("app/run.sh", "echo 早就在的另一份实现 app\n")
    repo.write("rigs/y/run.sh", "echo rigs y\n")
    repo.write(".claude/hooks/live.sh", "bash rigs/y/run.sh\n")
    repo.write("exempt.tsv", "rigs/y/run.sh\t豁免\napp/run.sh\t豁免\n")
    repo.register(*committed_rows)
    repo.commit("基准（老行已提交）")
    return repo


def judge(lag, repo, label):
    base = "HEAD"
    count, problems, note = lag.path_move_report(repo.root, base)
    effective, _ = lag.effective_moves(base, repo.root)
    changed = repo.changed(base)
    moved, _ = lag.moved_only_paths(base, changed, cwd=repo.root)
    text, _ = lag.baseline_text(base, "exempt.tsv", cwd=repo.root)
    current = repo.read("exempt.tsv")
    grown = [line for line in current.splitlines() if line not in text.splitlines()]
    lines, _ = lag.drop_moved_lines(base, [(".claude/hooks/live.sh", repo.read(".claude/hooks/live.sh").rstrip("\n"), 1)], cwd=repo.root)
    print(f"name={label} effective={short(f'{m.old}->{m.new}' for m in effective)} table_problems={len(problems)} dropped={short(sorted(moved))} "
          f"exempt_table_grown={short(repr(x) for x in grown)} caller_line_kept={len(lines)==1}")
    for what, _how in problems[:3]:
        print(f"    problem: {what[:150]}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    lag = load_library(arguments.frozen)
    os.makedirs(arguments.work, exist_ok=True)
    old = row("legacy", "app")   # 早先搬完的老行：legacy 在基准里没有，app 早就在
    # H1-1：新行插在老行前面，rigs/y -> legacy，指望串上老行落到 app；调用方与豁免表一并改成 app/run.sh
    repo = make(arguments.work, arguments.frozen, "h1_1_land_nowhere", [old])
    repo.register(row("rigs/y", "legacy"), old)
    repo.rm("rigs/y")
    repo.write(".claude/hooks/live.sh", "bash app/run.sh\n")
    repo.write("exempt.tsv", "app/run.sh\t豁免\napp/run.sh\t豁免\n")
    judge(lag, repo, "h1_1_no_landing")
    # H1-1 变体：真把 rigs/y 搬到 legacy，调用方却指向 app
    repo = make(arguments.work, arguments.frozen, "h1_1_land_legacy", [old])
    repo.register(row("rigs/y", "legacy"), old)
    repo.git("mv", "rigs/y", "legacy")
    repo.write(".claude/hooks/live.sh", "bash app/run.sh\n")
    judge(lag, repo, "h1_1_landing_at_legacy_caller_to_app")
    # H1-1 变体对照：真搬迁，调用方指向落点
    repo = make(arguments.work, arguments.frozen, "h1_1_control", [old])
    repo.register(row("rigs/y", "legacy"), old)
    repo.git("mv", "rigs/y", "legacy")
    repo.write(".claude/hooks/live.sh", "bash legacy/run.sh\n")
    repo.write("exempt.tsv", "legacy/run.sh\t豁免\napp/run.sh\t豁免\n")
    judge(lag, repo, "h1_1_control_real_move")
    # H1-4：老行留在表里、什么都没改：不能判涨，也不能剔任何东西
    repo = make(arguments.work, arguments.frozen, "h1_4_untouched", [old])
    judge(lag, repo, "h1_4_untouched_after_commit")
    # H1-4 另一半：提交之后把留旧路径的那一行改成新路径，照算改动
    repo = make(arguments.work, arguments.frozen, "h1_4_edit", [old])
    repo.write("exempt.tsv", "rigs/y/run.sh\t豁免\napp/run.sh\t豁免\nlegacy/new.sh\t新写的一行\n")
    judge(lag, repo, "h1_4_new_row_in_table")
    # 无序键：老行 legacy->app 换成 app->legacy，再加 legacy->mid；中间目录 mid 不存在
    repo = make(arguments.work, arguments.frozen, "unordered_key", [old])
    repo.register(row("app", "legacy"), row("legacy", "mid"))
    repo.write(".claude/hooks/live.sh", "bash mid/run.sh\n")
    repo.write("exempt.tsv", "rigs/y/run.sh\t豁免\nmid/run.sh\t豁免\n")
    judge(lag, repo, "unordered_key_reordered_plus_middle")


main()
