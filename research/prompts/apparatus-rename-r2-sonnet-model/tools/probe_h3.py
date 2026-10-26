#!/usr/bin/env python3
"""H3：跨管辖类的单文件搬家，改动范围那一层（戊）不剔，新增行那一层（drop_moved_lines）照旧剔；以及 moved_script_unchanged 的边界字符误判方向。
用法：probe_h3.py --frozen <冻结副本> --work <新的草稿目录>"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import Repo, load_library, row, SOURCE  # noqa: E402

LINES = ["# 记", "- 推迟：下一轮再补这条，没有登记进 deferred 表", "- 钟点 上午 9:30 开会"]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    lag = load_library(arguments.frozen)
    os.makedirs(arguments.work, exist_ok=True)
    repo = Repo(os.path.join(arguments.work, "cross"))
    repo.write(".claude/rules/draft.md", "\n".join(LINES) + "\n")
    repo.write(SOURCE, "# 搬迁\n")
    repo.register()
    repo.commit("基准")
    for label, kind in (("c1-从rules搬进records", "records/2026-10-23-draft.md"), ("c2-同一类里改名", ".claude/rules/renamed.md"), ("c3-搬进kb", ".claude/kb/draft.md")):
        work = repo.copy_to(os.path.join(arguments.work, label))
        os.makedirs(os.path.dirname(os.path.join(work.root, kind)), exist_ok=True)
        work.git("mv", ".claude/rules/draft.md", kind)
        work.register(row(".claude/rules/draft.md", kind))
        count, problems, note = lag.path_move_report(work.root, "HEAD")
        paths = work.staged_paths()
        stripped, p1 = lag.moved_only_paths("HEAD", paths, cwd=work.root)
        entries = [(kind, text, number) for number, text in enumerate(LINES, 1)]
        kept, p2 = lag.drop_moved_lines("HEAD", entries, cwd=work.root)
        print(f"name={label} 登记表问题={[w[:50] for w, _ in problems]} 改动范围剔掉={sorted(stripped)} 新增行共{len(entries)}条留下{len(kept)}条 问题={p1}/{p2}")
    # 对照：不登记
    work = repo.copy_to(os.path.join(arguments.work, "c0"))
    work.git("mv", ".claude/rules/draft.md", "records/2026-10-23-draft.md") if os.makedirs(os.path.join(work.root, "records"), exist_ok=True) is None else None
    entries = [("records/2026-10-23-draft.md", text, number) for number, text in enumerate(LINES, 1)]
    kept, p2 = lag.drop_moved_lines("HEAD", entries, cwd=work.root)
    print(f"name=c0-没登记的对照 新增行共{len(entries)}条留下{len(kept)}条")


main()
