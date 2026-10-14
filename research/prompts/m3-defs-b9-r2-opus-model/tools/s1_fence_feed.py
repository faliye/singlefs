#!/usr/bin/env python3
"""S1 喂法：派发提示里的代码块经各份副本的 runner-dispatch-guard.sh --check-prompt 判（与钩子入口同一份判法），打印退出码与首行。
另打 CommonMark 参照实现（markdown-it-py commonmark 预设）眼里不在任何代码块（围栏块、缩进代码块）里的「线程上限：」「多机：」行，
与报告里写的「读者（LLM 按行读、反引号成对配）眼里」那一行并列。
用法：s1_fence_feed.py --root <名>=<副本根> ... --work <新目录>（第一个 --root 是这一轮冻结副本）"""
import argparse, os, subprocess

parser = argparse.ArgumentParser()
parser.add_argument("--root", action="append", required=True)
parser.add_argument("--work", required=True)
arguments = parser.parse_args()
roots = [entry.split("=", 1) for entry in arguments.root]
os.makedirs(arguments.work, exist_ok=True)
scratch = os.path.join(arguments.work, "fake-scratch")
head = ("开工先读 `.claude/agent-common.md`。\n重型测试：不跑\n草稿目录：" + scratch + "/\n报告：" + scratch + "/report.md\n"
        "活：在 crates/singlefs-harness 里复现一个 harness 测试的失败，交回最小复现。\n现象：harness 的一个测试偶发失败。\n预计挂钟：2 小时内\n")
B = "```"
cases = [
    # (名字, 提示正文, 读者眼里那一行)
    ("C0-control-16", head + "线程上限：16\n", "16"),
    ("C1-list-fence-closed-at-column-0", head + "- 上一次派发的原文：\n  " + B + "\n  线程上限：31\n" + B + "\n线程上限：16\n上一次的骨架：\n" + B + "\n线程上限：31\n" + B + "\n",
     "16（列表项里开的围栏，关的那一行顶格写）"),
    ("C2-blockquote-fence-closed-without-marker", head + "> 上一次派发的原文：\n> " + B + "\n> 线程上限：31\n" + B + "\n线程上限：16\n上一次的骨架：\n" + B + "\n线程上限：31\n" + B + "\n",
     "16（引用块里开的围栏，关的那一行忘了写 >）"),
    ("C3-details-fold-with-blank-line", head + "<details><summary>上一次派发的原文</summary>\n" + B + "\n重型测试：不跑\n\n活：照上一次的骨架改。\n" + B + "\n</details>\n\n线程上限：16\n上一次的骨架：\n" + B + "\n线程上限：31\n" + B + "\n",
     "16（<details> 折起来的原文里有空行）"),
    ("C4-closing-fence-indented-4", head + "上一次派发的原文：\n" + B + "\n线程上限：31\n    " + B + "\n线程上限：16\n上一次的骨架：\n" + B + "\n线程上限：31\n" + B + "\n",
     "16（关的那一行多缩进了 4 格）"),
    ("C5-closing-fence-tab", head + "上一次派发的原文：\n" + B + "\n线程上限：31\n\t" + B + "\n线程上限：16\n上一次的骨架：\n" + B + "\n线程上限：31\n" + B + "\n",
     "16（关的那一行前面是制表符）"),
    ("C6-closing-fence-shorter", head + "上一次派发的原文：\n````\n线程上限：31\n" + B + "\n线程上限：16\n上一次的骨架：\n````\n线程上限：31\n````\n",
     "16（开四个反引号、关三个）"),
    ("D1-indented-code-block-example", head + "线程上限：31\n上一次派发的原文（缩进四格贴的）：\n\n    线程上限：31（本机）\n    预计挂钟：2 小时内\n", "31；缩进代码块里的示例（CommonMark 的代码块）"),
    ("D2-unclosed-fence-at-end-example", head + "线程上限：31\n上一次派发的原文（截到这里）：\n" + B + "\n线程上限：31（本机）\n", "31；没闭合的围栏到文末（CommonMark 的代码块）"),
    ("D3-list-fence-ended-by-container-example", head + "线程上限：31\n- 上一次派发的原文：\n  " + B + "\n  线程上限：31（本机）\n- 这一次照它改。\n", "31；列表项结束处结束的围栏（CommonMark 的代码块）"),
    ("E1-crlf-control-31", head.replace("\n", "\r\n") + "线程上限：31\r\n", "31（整份 \\r\\n 换行）"),
    ("E2-setext-heading-16", head + "线程上限：16\n---\n", "16（setext 标题）"),
    ("E3-table-cell-16", head + "| 项 | 值 |\n|---|---|\n| 线程上限：16 | 照用户 |\n", "表格单元里的「线程上限：16」"),
    ("E4-link-reference-definition", head + "[线程上限]: <16>\n线程上限：31\n", "31"),
]
for name, text, reader in cases:
    path = os.path.join(arguments.work, name + ".txt")
    with open(path, "w", encoding="utf-8", newline="") as handle:
        handle.write(text)
    cells, first_line = [], ""
    for label, root in roots:
        result = subprocess.run(["bash", os.path.join(root, ".claude/hooks/runner-dispatch-guard.sh"), "--check-prompt", path, "--type", "investigator", "--root", root],
                                capture_output=True, text=True, cwd=root, env=dict(os.environ, CLAUDE_PROJECT_DIR=root, PYTHONDONTWRITEBYTECODE="1"))
        first = next((line.strip() for line in (result.stdout + result.stderr).split("\n") if "✗" in line or "✓" in line), "")
        cells.append(f"{label}={result.returncode}")
        if label == roots[0][0]:
            first_line = first[:120]
    import markdown_it
    tokens = markdown_it.MarkdownIt("commonmark").parse(text.replace("\r", ""))
    code = set()
    for token in tokens:
        if token.type in ("fence", "code_block") and token.map:
            code.update(range(*token.map))
    live = [f"{i + 1}:{line.strip()}" for i, line in enumerate(text.replace("\r", "").split("\n"))
            if i not in code and (line.lstrip(" \t>-*").startswith("线程上限") or line.lstrip(" \t>-*").startswith("多机"))]
    print(f"{name}\t" + "\t".join(cells) + f"\t读者：{reader}\tCommonMark 代码块外：{' | '.join(live)}\t{roots[0][0]} 首行：{first_line}")
