#!/usr/bin/env python3
"""Z1 / Z6 喂法：派发提示里围栏的认法（钩子的 without_code_blocks 与 CommonMark）、只有已判行、多机说明另起一行。
每个变体经各份副本的 runner-dispatch-guard.sh --check-prompt 判（与钩子入口同一份判法），打印退出码与第一行 ✗ / ✓。
用法：z1_fence_feed.py --root <名>=<副本根> ... --work <新目录>（第一个 --root 是这一轮冻结副本，首行取它的）"""
import argparse, os, subprocess

parser = argparse.ArgumentParser()
parser.add_argument("--root", action="append", required=True)
parser.add_argument("--work", required=True)
arguments = parser.parse_args()
roots = [entry.split("=", 1) for entry in arguments.root]
os.makedirs(arguments.work, exist_ok=True)
scratch = os.path.join(arguments.work, "fake-scratch")
CANONICAL = "台数从 layer0-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量"
head = ("开工先读 `.claude/agent-common.md`。\n重型测试：不跑\n草稿目录：" + scratch + "/\n报告：" + scratch + "/report.md\n"
        "活：在 crates/singlefs-harness 里复现一个 harness 测试的失败，交回最小复现。\n现象：harness 的一个测试偶发失败。\n")
writer_head = ("开工先读 `.claude/agent-common.md`。\n重型测试：不跑\n草稿目录：" + scratch + "/\n报告：" + scratch + "/report.md\n"
               "条款：照判决改法 Q9 改 harness 的一个断言。\n要动的 crates 文件：crates/singlefs-harness/src/lib.rs\n")
within = "预计挂钟：2 小时内\n"
over = "预计挂钟：超过 2 小时，按上一次同类活推的\n分片：按测试目标切，一个目标一片。\n续跑：同一条命令再起只补还没有结果文件的目标。\n"
tilde_lone = "上一次给工具员讲围栏时贴的样子（外层用波浪线围起来，里面露出一行三个反引号）：\n~~~\n```\n~~~\n"
quoted_31 = "上一次派发的原文：\n```\n线程上限：31\n```\n"
cases = [
    # (名字, 类型, 提示正文, 读者（CommonMark）眼里的那一行)
    ("T0-control-16", "investigator", head + within + "线程上限：16\n", "线程上限：16"),
    ("T1-tilde-lone-fence-16", "investigator", head + within + tilde_lone + "线程上限：16\n" + quoted_31, "线程上限：16（波浪线块里那一行三个反引号是块的内容；31 在代码块里是示例）"),
    ("T1b-tilde-lone-fence-load", "investigator", head + within + tilde_lone + "线程上限：31（机器忙，按负载降到 16）\n" + quoted_31, "线程上限：31（机器忙，按负载降到 16）"),
    ("T2-list-fence-container-end-16", "investigator", head + within + "- 上一次的起法：\n  ```\n  cargo test -p singlefs-harness --lib\n- 这一次照它起。\n\n线程上限：16\n" + quoted_31,
     "线程上限：16（列表项里没闭合的围栏在列表项结束处结束）"),
    ("T3-html-comment-fence-16", "investigator", head + within + "<!-- 旧写法：\n```\n-->\n线程上限：16\n" + quoted_31, "线程上限：16（HTML 注释里的三个反引号不开块）"),
    ("O0-control-example-in-backtick-block", "investigator", head + within + "线程上限：31\n上一次派发的原文：\n```\n线程上限：31（本机）\n```\n", "线程上限：31；代码块里的示例不判"),
    ("O1-tilde-block-example", "investigator", head + within + "线程上限：31\n上一次派发的原文：\n~~~\n线程上限：31（本机）\n~~~\n", "线程上限：31；波浪线代码块里的示例"),
    ("O2-nested-list-fence-example", "investigator", head + within + "线程上限：31\n- 上一次：\n  - 派发原文：\n    ```\n    线程上限：31（本机）\n    ```\n", "线程上限：31；嵌套列表里的反引号代码块示例"),
    ("O3-blockquote-fence-example", "investigator", head + within + "线程上限：31\n> 上一次派发的原文：\n> ```\n> 线程上限：31（本机）\n> ```\n", "线程上限：31；引用块里的反引号代码块示例"),
    ("O4-unclosed-fence-at-end-example", "investigator", head + within + "线程上限：31\n上一次派发的原文（截到这里）：\n```\n线程上限：31（本机）\n", "线程上限：31；没闭合的围栏到文末都是代码"),
    ("H0-control-canonical", "investigator", head + over + "线程上限：31\n多机：" + CANONICAL + "\n", "多机：<规范句>"),
    ("H1-control-live", "investigator", head + over + "线程上限：31\n多机：每台线程按此刻空闲现算，机器忙时按负载降\n", "多机：现算"),
    ("H2-tilde-lone-fence-live", "investigator", head + over + "线程上限：31\n" + tilde_lone + "多机：每台线程按此刻空闲现算，机器忙时按负载降\n骨架打的那一句：\n```\n多机：" + CANONICAL + "\n```\n",
     "多机：现算（代码块里的规范句是示例）"),
    ("H3-next-line-live", "investigator", head + over + "线程上限：31\n多机：" + CANONICAL + "\n每台线程按此刻空闲现算，机器忙时按负载降到登记值的一半。\n", "多机：<规范句>，下一行普通正文写现算"),
    ("H4-next-line-live-runner", "experiment-runner", head + "这一段回答的岔路：岔路单第 1 行（分片方式）\n" + over + "线程上限：31\n多机：" + CANONICAL + "\n每台线程按此刻空闲现算，机器忙时按负载降到登记值的一半。\n", "同上，派执行员"),
    ("H5-quoted-user-line-duplicate", "investigator", head + over + "线程上限：31\n用户原话：\n> 多机：每台按忙闲来\n多机：" + CANONICAL + "\n", "引用块里转述用户的「多机：」，真行是规范句"),
    ("V0-writer-missing-line", "implementation-writer", writer_head + within, "没写线程上限一行"),
    ("V1-writer-valve-only-16", "implementation-writer", writer_head + within + "线程上限已判：内存只装得下 16 份编译，这一次用 16\n", "照拒绝说明只补已判行（要别的数）"),
    ("V2-writer-valve-and-16", "implementation-writer", writer_head + within + "线程上限：16\n线程上限已判：内存只装得下 16 份编译\n", "线程上限：16 加已判行"),
]
for name, kind, text, reader in cases:
    path = os.path.join(arguments.work, name + ".txt")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    cells, first_line = [], ""
    for label, root in roots:
        result = subprocess.run(["bash", os.path.join(root, ".claude/hooks/runner-dispatch-guard.sh"), "--check-prompt", path, "--type", kind, "--root", root],
                                capture_output=True, text=True, cwd=root, env=dict(os.environ, CLAUDE_PROJECT_DIR=root, PYTHONDONTWRITEBYTECODE="1"))
        first = next((line.strip() for line in (result.stdout + result.stderr).split("\n") if "✗" in line or "✓" in line), "")
        cells.append(f"{label}={result.returncode}")
        if label == roots[0][0]:
            first_line = first[:150]
            if name.startswith("V"):
                with open(os.path.join(arguments.work, name + ".out"), "w", encoding="utf-8") as full:
                    full.write(result.stdout + result.stderr)
    print(f"{name}\t{kind}\t" + "\t".join(cells) + f"\t读者看到：{reader}\t{roots[0][0]} 首行：{first_line}")
