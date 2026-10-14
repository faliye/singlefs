#!/usr/bin/env python3
"""S7：定义说『提示里别处写的台数、线程取法一概不照』之后，钩子判不判提示正文里别处写的台数与线程取法。
超过 2 小时的提示，分片、续跑、多机三行齐全（多机一行是规范句），线程上限一行等于登记值；再在正文别处（任务描述、说明另起一行、引用块、括号里）写各种台数与线程取法。
钩子（--check-prompt，⑭ ⑮）对这些放还是拒；定义一侧是『子 agent 一概不照』，没有第二处源码核。
用法：s7_prose_feed.py <b9 树> <b10 树> <工作目录>"""
import os, re, subprocess, sys
t9, t10, work = (os.path.abspath(p) for p in sys.argv[1:4]); os.makedirs(work, exist_ok=True)
sys.path.insert(0, os.path.join(t10, ".claude/hooks"))
import lib_long_run
CANON = lib_long_run.CANONICAL_HOSTS_SENTENCE
HEAD = ("重型测试：不跑\n开工先读 `.claude/agent-common.md`\n草稿目录：/home/fy5090/.local/share/singlefs/work/zz-nonexistent/\n报告：/home/fy5090/.local/share/singlefs/work/zz-nonexistent/r.md\n"
        "这一段回答的岔路：岔路单第 1 行\n预计挂钟：超过 2 小时，按上一次同类活推的\n分片：按测试目标切，一个目标一片。\n续跑：同一条命令再起只补还没有结果文件的目标。\n"
        "多机：" + CANON + "\n线程上限：31\n")
prose = [
 ("对照：只有规范句", ""),
 ("下一行：每台线程按此刻空闲现算", "每台线程按此刻空闲现算，机器忙时按负载降。\n"),
 ("下一行：两台，第一台 64 线程、第二台 4 线程", "两台，第一台 64 线程、第二台 4 线程。\n"),
 ("下一行：线程取 nproc 的一半", "每台线程取 nproc 的一半。\n"),
 ("任务描述里：只用本机一台，开 8 个 worker", "活：把三个目标跑完，只用本机一台，开 8 个 worker。\n"),
 ("引用块里：用户说每台线程看忙闲", "> 用户：每台线程看忙闲来定\n"),
 ("括号里：（台数取 3）", "（台数取 3，每台 12 线程）\n"),
 ("KS_THREADS 环境变量", "起命令前设 KS_THREADS=4。\n"),
 ("多机行本身写死台数（对照，应拒）", None),
]
def verdict(tree, text, kind):
    path = os.path.join(work, "p.txt"); open(path, "w", encoding="utf-8").write(text)
    r = subprocess.run(["bash", os.path.join(tree, ".claude/hooks/runner-dispatch-guard.sh"), "--check-prompt", path, "--type", kind, "--root", tree],
                       capture_output=True, text=True, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1", CLAUDE_PROJECT_DIR=tree))
    out = r.stdout + r.stderr
    m = re.search(r"会拒 \d+ 处（([^）]*)）", out)
    names = set(m.group(1).split("、")) if m else set()
    return "拒" if names & {"线程上限", "预计挂钟与长活声明"} else "放"
print("例名\t类型\tb9\tb10")
for name, extra in prose:
    for kind in ("experiment-runner", "investigator"):
        text = HEAD + (extra if extra is not None else "")
        if extra is None:
            text = text.replace("多机：" + CANON, "多机：台数 2，每台线程 64")
        print(f"{name}\t{kind}\t{verdict(t9, text, kind)}\t{verdict(t10, text, kind)}")
