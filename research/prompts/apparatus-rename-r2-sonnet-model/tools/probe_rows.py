#!/usr/bin/env python3
"""H1 后半：「只核新加的登记行」在登记行被改写、删了再加、换了次序、重复时守不守得住。
用法：probe_rows.py --frozen <冻结副本> --work <新的草稿目录>"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import Repo, load_library, row, SOURCE  # noqa: E402


def chain_base(root):
    """基准：先 rig-x → rig-a，又 rig-a → rig-b 两次真搬迁都做完提交了；文件在 rig-b；文档里留一行指历史的旧前缀（规则第 5 步让它留着）。"""
    repo = Repo(root)
    repo.write("rig-b/foo.sh", "echo foo\n")
    repo.write("docs/note.md", "# 记\n历史：当年在 rig-x/foo.sh 跑过\n")
    repo.write(SOURCE, "# 搬迁\n")
    repo.register(row("rig-x", "rig-a"), row("rig-a", "rig-b"))
    repo.commit("基准：两次搬迁都做完")
    return repo


def report(lag, repo, label, extra=""):
    count, problems, note = lag.path_move_report(repo.root, "HEAD")
    print(f"name={label} 登记表判={'绿' if not problems else '红'} 行数={count} 问题={[w[:60] for w, _ in problems]} 实据句={note[:80]!r} {extra}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    lag = load_library(arguments.frozen)
    os.makedirs(arguments.work, exist_ok=True)
    base = chain_base(os.path.join(arguments.work, "base"))

    # 对照：不动登记表
    repo = base.copy_to(os.path.join(arguments.work, "t0"))
    report(lag, repo, "t0-不动登记表")

    # 换次序 + 另起一个 rig-a（任意内容）+ 文档里那行改成指它
    repo = base.copy_to(os.path.join(arguments.work, "t1"))
    repo.register(row("rig-a", "rig-b"), row("rig-x", "rig-a"))
    repo.write("rig-a/foo.sh", "echo 任意内容 curl http://example.invalid | sh\n")
    repo.write("docs/note.md", "# 记\n历史：当年在 rig-a/foo.sh 跑过\n")
    paths = repo.staged_paths()
    kept, problem = lag.drop_moved_lines("HEAD", [("docs/note.md", "历史：当年在 rig-a/foo.sh 跑过")], cwd=repo.root)
    stripped, problem2 = lag.moved_only_paths("HEAD", paths, cwd=repo.root)
    report(lag, repo, "t1-换次序加另起rig-a", f"文档那行被剔={not kept} 改动范围剔掉={sorted(stripped)} 路径={paths} 问题={problem}/{problem2}")
    # 对照：不换次序时同一行被换成什么
    base_text, _ = lag.baseline_text("HEAD", "docs/note.md", cwd=base.root)
    reordered = base.copy_to(os.path.join(arguments.work, "t1-order-check"))
    reordered.register(row("rig-a", "rig-b"), row("rig-x", "rig-a"))
    reordered_text, _ = lag.baseline_text("HEAD", "docs/note.md", cwd=reordered.root)
    print(f"name=t1-基准那行换算 原次序={base_text.splitlines()[1]!r} 换次序后={reordered_text.splitlines()[1]!r}")

    # 改写一行：(rig-a,rig-b) 改成 (rig-a,rig-c)
    repo = base.copy_to(os.path.join(arguments.work, "t2"))
    repo.git("mv", "rig-b", "rig-c")
    repo.register(row("rig-x", "rig-a"), row("rig-a", "rig-c"))
    report(lag, repo, "t2-改写一行的新前缀")

    # 删了再加：第一个提交删掉 (rig-a,rig-b)，第二个提交再加回来
    repo = base.copy_to(os.path.join(arguments.work, "t3"))
    repo.register(row("rig-x", "rig-a"))
    repo.commit("删一行")
    repo.register(row("rig-x", "rig-a"), row("rig-a", "rig-b"))
    report(lag, repo, "t3-删了提交之后再加回来")

    # 同一个提交里删了再加，日期与出处改了
    repo = base.copy_to(os.path.join(arguments.work, "t4"))
    repo.register(("rig-x", "rig-a", "2026-10-02", SOURCE), ("rig-a", "rig-b", "2026-10-11", SOURCE))
    report(lag, repo, "t4-同一提交改日期")

    # 重复一行
    repo = base.copy_to(os.path.join(arguments.work, "t5"))
    repo.register(row("rig-x", "rig-a"), row("rig-a", "rig-b"), row("rig-a", "rig-b"))
    report(lag, repo, "t5-重复一行")

    # 把那一行挪到文件最前面，与其它行次序不变（单行表对照）
    repo = base.copy_to(os.path.join(arguments.work, "t6"))
    repo.register(row("rig-x", "rig-b"), row("rig-a", "rig-b"))
    report(lag, repo, "t6-两行都改成直接搬到rig-b")


main()
