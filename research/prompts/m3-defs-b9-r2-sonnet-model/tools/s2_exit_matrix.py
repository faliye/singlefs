#!/usr/bin/env python3
"""S2：缺行出路两种写法之间有没有两头都不落的写法。
出路一＝「线程上限：N」再加已判行（要别的数）；出路二＝单独的已判行（确实不起重活）。
每例一份提示（类型 clerk），经 --check-prompt 判 ⑮ 放还是拒；再列『定义一侧的读法』（agent-common 起重活那一条与九份定义的「线程上限」一行）：
  有 N 行 → 子 agent 照 N 起重活；只有已判行 → 子 agent 不起重活、要起就停下交回；已判行里写的数不照。
列：钩子 b9 / b10（实测）；定义读法（人工按定义写，不是实测）。两头不落＝钩子放而按定义读出来的结果不是主 agent 要的，看『定义读法』一列里写了「主 agent 要的是」的行。
用法：s2_exit_matrix.py <b9 树> <b10 树> <工作目录>"""
import os, re, subprocess, sys
t9, t10, work = (os.path.abspath(p) for p in sys.argv[1:4]); os.makedirs(work, exist_ok=True)
HEAD = ("重型测试：不跑\n开工先读 `.claude/agent-common.md`\n草稿目录：/home/fy5090/.local/share/singlefs/work/zz-nonexistent/\n报告：/home/fy5090/.local/share/singlefs/work/zz-nonexistent/r.md\n预计挂钟：2 小时内\n")
V = "线程上限已判：用户给了参数，这一次用 16 线程"
cases = [
 ("缺行也没有已判行", "", "—", "拒"),
 ("只有 N 等于登记值", "线程上限：31\n", "子 agent 按 31 起重活", "放"),
 ("N 不等于登记值，没有已判行", "线程上限：16\n", "—", "拒"),
 ("N 不等于登记值 + 已判行（出路一）", "线程上限：16\n" + V + "\n", "子 agent 按 16 起重活", "放"),
 ("N 不等于登记值 + 已判行没写理由", "线程上限：16\n线程上限已判：\n", "—", "拒"),
 ("N 等于登记值 + 已判行没写理由", "线程上限：31\n线程上限已判：\n", "按 31", "放或拒见结果"),
 ("单独已判行，理由说『不起重活』（出路二）", "线程上限已判：这件活只读，不起重活\n", "不起重活", "放"),
 ("单独已判行，理由里写了要用的数（旧出路的写法）", "线程上限已判：内存只装得下 16 份编译，这一次用 16\n", "不起重活（已判行里的数不照）；主 agent 要的是按 16 起重活", "放；主 agent 的意图与子 agent 的读法相反"),
 ("N 等于登记值 + 已判行里写了别的数", "线程上限：31\n线程上限已判：用户明说这一次用 8 线程\n", "按 31（已判行里的数不照）；主 agent 要的是 8", "放；意图与读法相反"),
 ("N 不等于登记值 + 已判行里写了第三个数", "线程上限：16\n线程上限已判：用户明说这一次用 8 线程\n", "按 16", "放；已判行与 N 不一致没人判"),
 ("N 不等于登记值写在代码块里 + 已判行在外", "```\n线程上限：16\n```\n线程上限已判：内存只装得下 16 份编译，这一次用 16\n", "按定义：代码块里的行不判 → 只有已判行 → 不起重活", "放；N 被遮，意图与读法相反"),
 ("已判行写在代码块里 + N 不等于登记值", "线程上限：16\n```\n线程上限已判：内存装不下\n```\n", "—", "拒"),
 ("N 带括注 + 已判行", "线程上限：16（内存装不下）\n线程上限已判：内存装不下\n", "—", "拒"),
 ("两行线程上限（31 与 16）+ 已判行", "线程上限：31\n线程上限：16\n线程上限已判：内存装不下\n", "两行 N 以哪个为准没写", "放或拒见结果"),
 ("已判行放在引用块里（行首 >）", "线程上限：16\n> 线程上限已判：内存装不下\n", "引用块不是代码块，已判行照算", "放"),
]
def verdict(tree, text):
    path = os.path.join(work, "p.txt"); open(path, "w", encoding="utf-8").write(HEAD + text)
    r = subprocess.run(["bash", os.path.join(tree, ".claude/hooks/runner-dispatch-guard.sh"), "--check-prompt", path, "--type", "clerk", "--root", tree],
                       capture_output=True, text=True, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1", CLAUDE_PROJECT_DIR=tree))
    out = r.stdout + r.stderr
    m = re.search(r"会拒 \d+ 处（([^）]*)）", out)
    return ("拒" if (m and "线程上限" in m.group(1).split("、")) else "放"), out
print("例名\tb9\tb10\t定义读法")
for name, text, reading, note in cases:
    (a, _), (b, out) = verdict(t9, text), verdict(t10, text)
    print(f"{name}\t{a}\t{b}\t{reading}")
for name, text in (("缺行也没有已判行", ""), ("N 不等于登记值，没有已判行", "线程上限：16\n")):
    for label, tree in (("b9", t9), ("b10", t10)):
        _, out = verdict(tree, text)
        hint = [l for l in out.split("\n") if "怎么办" in l and "线程上限" in l]
        print(f"## {name} / {label} 的拒绝说明里『怎么办』那一行（线程上限一项）")
        print("\n".join(h.strip()[:700] for h in hint))
