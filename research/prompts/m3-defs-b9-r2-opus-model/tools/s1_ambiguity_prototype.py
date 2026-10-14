#!/usr/bin/env python3
"""F-S1a 原型（只在本腿模型上量，没改钩子）：⑭ ⑮ 的标签行，在 CommonMark 认法（照抄这一轮 without_code_blocks 的判法：markdown-it commonmark 预设、只遮闭合了的围栏块）
与宽的逐行认法（反引号、波浪线都认，去掉行首的 > 与任意缩进，往后找第一行同一种字符、三个以上的当关块）之间遮不遮不一致，就当「代码块边界有歧义」拒。
打每个喂法：CommonMark 遮掉的标签行、逐行认法遮掉的标签行、有歧义的标签行（有就拒）。喂法取本腿 S1 的 C/D/H 与第一轮攻方 Z1 的 T/O 两组同形（提示正文按那两份工具重造）。"""
import re, markdown_it

LABEL = re.compile(r"^[ \t>*_\-]*(线程上限|多机|分片|续跑|预计挂钟)[* \t]*[：:]")
PREFIX = re.compile(r"^(?:[ \t]*>)*[ \t]*")
parser = markdown_it.MarkdownIt("commonmark")

def commonmark_hidden(prompt):
    lines = prompt.split("\n")
    parsed = "\n".join(line.rstrip("\r").replace("\r", " ") for line in lines)
    hidden = set()
    for token in parser.parse(parsed):
        if token.type != "fence" or not token.map:
            continue
        first, end = token.map
        if end - first < 2 or end > len(lines):
            continue
        closing = PREFIX.sub("", lines[end - 1].rstrip("\r"), count=1)
        if re.fullmatch(re.escape(token.markup[0]) + "{" + str(len(token.markup)) + r",}[ \t]*", closing):
            hidden.update(range(first, end))
    return hidden

def line_hidden(prompt):
    lines = prompt.split("\n")
    hidden, index = set(), 0
    while index < len(lines):
        opening = re.match(r"(`{3,}|~{3,})", PREFIX.sub("", lines[index], count=1))
        if opening:
            char = opening.group(1)[0]
            close = next((j for j in range(index + 1, len(lines)) if re.fullmatch(re.escape(char) + "{3,}[ \t]*", PREFIX.sub("", lines[j], count=1))), None)
            if close is not None:
                hidden.update(range(index, close + 1))
                index = close + 1
                continue
        index += 1
    return hidden

def verdict(prompt):
    labels = {i for i, line in enumerate(prompt.split("\n")) if LABEL.match(line)}
    cm, ln = commonmark_hidden(prompt) & labels, line_hidden(prompt) & labels
    return sorted(i + 1 for i in cm), sorted(i + 1 for i in ln), sorted(i + 1 for i in cm ^ ln)

B = "```"
CANON = "<规范句>"
cases = [
    ("C1-list-fence-closed-at-column-0", "- 上一次派发的原文：\n  " + B + "\n  线程上限：31\n" + B + "\n线程上限：16\n上一次的骨架：\n" + B + "\n线程上限：31\n" + B + "\n"),
    ("C2-blockquote-fence-closed-without-marker", "> 上一次派发的原文：\n> " + B + "\n> 线程上限：31\n" + B + "\n线程上限：16\n上一次的骨架：\n" + B + "\n线程上限：31\n" + B + "\n"),
    ("C3-details-fold-with-blank-line", "<details><summary>上一次派发的原文</summary>\n" + B + "\n重型测试：不跑\n\n活：照上一次的骨架改。\n" + B + "\n</details>\n\n线程上限：16\n上一次的骨架：\n" + B + "\n线程上限：31\n" + B + "\n"),
    ("C4-closing-fence-indented-4", "上一次派发的原文：\n" + B + "\n线程上限：31\n    " + B + "\n线程上限：16\n上一次的骨架：\n" + B + "\n线程上限：31\n" + B + "\n"),
    ("C5-closing-fence-tab", "上一次派发的原文：\n" + B + "\n线程上限：31\n\t" + B + "\n线程上限：16\n上一次的骨架：\n" + B + "\n线程上限：31\n" + B + "\n"),
    ("C6-closing-fence-shorter", "上一次派发的原文：\n````\n线程上限：31\n" + B + "\n线程上限：16\n上一次的骨架：\n````\n线程上限：31\n````\n"),
    ("H1-list-fence-closed-at-column-0-hosts", "线程上限：31\n- 上一次的起法：\n  " + B + "\n  cargo test -p singlefs-harness --lib\n" + B + "\n多机：每台线程按此刻空闲现算\n上一次的骨架：\n" + B + "\n多机：" + CANON + "\n" + B + "\n"),
    ("T1-tilde-lone-fence-16", "~~~\n" + B + "\n~~~\n线程上限：16\n上一次派发的原文：\n" + B + "\n线程上限：31\n" + B + "\n"),
    ("T2-list-fence-container-end-16", "- 上一次的起法：\n  " + B + "\n  cargo test -p singlefs-harness --lib\n- 这一次照它起。\n\n线程上限：16\n上一次派发的原文：\n" + B + "\n线程上限：31\n" + B + "\n"),
    ("T3-html-comment-fence-16", "<!-- 旧写法：\n" + B + "\n-->\n线程上限：16\n上一次派发的原文：\n" + B + "\n线程上限：31\n" + B + "\n"),
    ("O0-backtick-block-example", "线程上限：31\n上一次派发的原文：\n" + B + "\n线程上限：31（本机）\n" + B + "\n"),
    ("O1-tilde-block-example", "线程上限：31\n上一次派发的原文：\n~~~\n线程上限：31（本机）\n~~~\n"),
    ("O2-nested-list-fence-example", "线程上限：31\n- 上一次：\n  - 派发原文：\n    " + B + "\n    线程上限：31（本机）\n    " + B + "\n"),
    ("O3-blockquote-fence-example", "线程上限：31\n> 上一次派发的原文：\n> " + B + "\n> 线程上限：31（本机）\n> " + B + "\n"),
    ("O5-proper-nesting-four-backticks", "线程上限：31\n上一次给工具员讲围栏的原文：\n````\n" + B + "\n线程上限：31（本机）\n" + B + "\n````\n"),
    ("D1-indented-code-block-example", "线程上限：31\n上一次派发的原文（缩进四格贴的）：\n\n    线程上限：31（本机）\n"),
]
for name, prompt in cases:
    cm, ln, ambiguous = verdict(prompt)
    print(f"{name}\tCommonMark 遮的标签行 {cm}\t逐行认法遮的标签行 {ln}\t有歧义 {ambiguous}\t{'拒（边界有歧义）' if ambiguous else '照常判'}")
