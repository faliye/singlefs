#!/usr/bin/env python3
"""Z1 的读者一侧：用 CommonMark 参照实现 markdown-it-py（commonmark 预设）解析 z1_fence_feed.py 写下的每份提示，
列出不在代码块（fence、缩进代码块、HTML 块）里的「线程上限：」「多机：」行——读者与渲染眼里的声明行。
用法：z1_commonmark_view.py <z1_fence_feed.py 的 --work> 名字…（不给名字就全列）"""
import os, re, sys
import markdown_it

work = sys.argv[1]
names = sys.argv[2:] or sorted(name[:-4] for name in os.listdir(work) if name.endswith(".txt"))
parser = markdown_it.MarkdownIt("commonmark")
LABEL = re.compile(r"^[ \t>*_\-]*(线程上限|多机)[* \t]*[：:]")
print(f"markdown-it-py {markdown_it.__version__}（commonmark 预设）")
for name in names:
    text = open(os.path.join(work, name + ".txt"), encoding="utf-8").read()
    lines = text.split("\n")
    hidden = set()
    for token in parser.parse(text):
        if token.type in ("fence", "code_block", "html_block") and token.map:
            hidden.update(range(token.map[0], token.map[1]))
    shown = [f"第 {index + 1} 行「{line.strip()}」" for index, line in enumerate(lines) if LABEL.match(line) and index not in hidden]
    print(f"{name}\t不在代码块里的声明行：{'；'.join(shown) or '无'}")
