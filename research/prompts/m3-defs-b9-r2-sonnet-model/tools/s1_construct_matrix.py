#!/usr/bin/env python3
"""S1 专问：markdown-it 与『人/LLM 读派发提示』不一致的写法（HTML 块、链接引用定义、setext 标题、表格里的反引号、制表符缩进、\\r\\n 等）。
每例一份提示：先写一行真声明「线程上限：31」，再放一段构造，构造里有一行「线程上限：16」（块里的示例或引文）。
列：钩子 b9 / b10 对「线程上限」这一项拒没拒（拒＝钩子把 16 当声明判了，放＝钩子把 16 遮掉了）；markdown-it 眼里 16 那一行在哪种块里；『预期读者』是人工写的读法。
用法：s1_construct_matrix.py <b9 树> <b10 树> <工作目录>"""
import os, re, subprocess, sys
t9, t10, work = (os.path.abspath(p) for p in sys.argv[1:4])
os.makedirs(work, exist_ok=True)
import markdown_it
parser = markdown_it.MarkdownIt("commonmark")
HEAD = ("重型测试：不跑\n开工先读 `.claude/agent-common.md`\n草稿目录：/home/fy5090/.local/share/singlefs/work/zz-nonexistent/\n报告：/home/fy5090/.local/share/singlefs/work/zz-nonexistent/r.md\n预计挂钟：2 小时内\n")
SAMPLE = "线程上限：16"
cases = [
 # (名, 构造正文（含 SAMPLE 一行）, 预期读者)
 ("对照：闭合的反引号围栏", "```\n%s\n```\n", "示例"),
 ("HTML 块：<div> 里紧跟围栏", "<div>\n```\n%s\n```\n</div>\n", "示例（人会当代码）"),
 ("HTML 块：<pre> 里的一行", "<pre>\n%s\n</pre>\n", "示例（预格式）"),
 ("HTML 块：<details> 空行后的围栏", "<details>\n<summary>旧写法</summary>\n\n```\n%s\n```\n</details>\n", "示例"),
 ("HTML 注释里的一行", "<!--\n%s\n-->\n", "被注释掉的（人读原文会当旧写法）"),
 ("链接引用定义的标题跨行", "[a]: /u \"旧\n%s\n续\"\n", "引文"),
 ("setext 标题下划线", "%s\n---\n", "标题文字（仍是一行声明）"),
 ("表格里的反引号", "| 写法 |\n|---|\n| ``` |\n| %s |\n| ``` |\n", "表格单元（commonmark 预设不认表格）"),
 ("制表符缩进的围栏", "\t```\n\t%s\n\t```\n", "缩进代码块（人当示例）"),
 ("列表项里制表符缩进的围栏", "- 上一次：\n\t```\n\t%s\n\t```\n", "示例"),
 ("四空格缩进的行", "    %s\n", "缩进代码块（人当示例）"),
 ("围栏的闭合行缩进 4 格", "```\n%s\n    ```\n", "示例？（CommonMark：围栏没闭合，到文末都是代码）"),
 ("围栏的闭合行用制表符缩进", "```\n%s\n\t```\n", "同上"),
 ("围栏没闭合", "```\n%s\n", "示例（到文末）"),
 ("四个波浪线围栏里夹三个波浪线", "~~~~\n~~~\n%s\n~~~\n~~~~\n", "示例"),
 ("围栏信息串带反引号", "```a`b\n%s\n```\n", "行内代码，不是围栏"),
 ("引用块里的围栏", "> ```\n> %s\n> ```\n", "示例"),
 ("引用块里的围栏没闭合又接普通行", "> ```\n> 例\n\n%s\n```\n", "第二个围栏在块外，16 在两个围栏之间"),
 ("围栏里套反引号行内代码", "```\n`%s`\n```\n", "示例"),
 ("行内三反引号后一行", "```x``` 是行内代码\n%s\n", "声明（行内代码不开块）"),
 ("全角反引号", "｀｀｀\n%s\n｀｀｀\n", "不是围栏，LLM 也许当围栏"),
 ("三个以上反引号后接文字（信息串）", "```text\n%s\n```\n", "示例"),
]
rows = []
def reader_view(prompt, needle):
    lines = prompt.split("\n")
    tokens = parser.parse("\n".join(l.rstrip("\r") for l in lines))
    index = next(i for i, l in enumerate(lines) if needle in l)
    for t in tokens:
        if t.map and t.map[0] <= index < t.map[1] and t.type in ("fence", "code_block", "html_block"):
            return t.type
    return "正文"
def verdict(tree, prompt, newline="\n"):
    path = os.path.join(work, "p.txt")
    open(path, "w", encoding="utf-8", newline="").write(prompt.replace("\n", newline))
    r = subprocess.run(["bash", os.path.join(tree, ".claude/hooks/runner-dispatch-guard.sh"), "--check-prompt", path, "--type", "clerk", "--root", tree],
                       capture_output=True, text=True, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1", CLAUDE_PROJECT_DIR=tree))
    out = r.stdout + r.stderr
    m = re.search(r"会拒 \d+ 处（([^）]*)）", out)
    return "拒" if (m and "线程上限" in m.group(1).split("、")) else "放"
print("例名\tb9\tb10\tmarkdown-it 眼里 16 那行\t预期读者")
for name, body, reader in cases:
    prompt = HEAD + "线程上限：31\n" + (body % SAMPLE)
    print(f"{name}\t{verdict(t9, prompt)}\t{verdict(t10, prompt)}\t{reader_view(prompt, SAMPLE)}\t{reader}")
prompt = HEAD + "线程上限：31\n" + "```\n%s\n```\n" % SAMPLE
print(f"CRLF：闭合的反引号围栏\t{verdict(t9, prompt, chr(13)+chr(10))}\t{verdict(t10, prompt, chr(13)+chr(10))}\t(CRLF)\t示例")
prompt = HEAD + "线程上限：31\n"
print(f"CRLF：只有真声明 31\t{verdict(t9, prompt, chr(13)+chr(10))}\t{verdict(t10, prompt, chr(13)+chr(10))}\t(CRLF)\t声明；拒＝线程上限被拒")
