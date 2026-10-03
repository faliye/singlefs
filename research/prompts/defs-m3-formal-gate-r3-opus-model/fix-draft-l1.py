#!/usr/bin/env python3
"""L1 改法草稿（只在攻方自己的副本上改、只在这份探针上量过，被攻过零轮）：python3 fix-draft-l1.py <仓副本根>
K1：判参数之前先照 preflight 摘掉每一个整词 --force（两份判法脚本都在 __main__ 里先调 preflight(__file__)，它从 sys.argv 里摘）。
K2：python -c 的代码里点了两份判法脚本的（文件名或模块名，不只 run_module），一律算 verus-stage，不看代码之后的词（代码自己能改 sys.argv、直接调函数）。"""
import sys

path = sys.argv[1] + "/.claude/hooks/lib_heavy_tests.py"
text = open(path, encoding="utf-8").read()
edits = [
    ('    literal = not any("$" in argument or "`" in argument for argument in arguments)\n',
     '    arguments = [argument for argument in arguments if argument != "--force"]   # K1：照 preflight 摘掉 --force\n'
     '    literal = not any("$" in argument or "`" in argument for argument in arguments)\n'),
    ('    "verus-proof-check.py": re.compile(r"verus-proof-check\\.py|run_module\\s*\\([^)]*verus-proof-check"),\n'
     '    "e165-pin-red-examples.py": re.compile(r"e165-pin-red-examples\\.py|run_module\\s*\\([^)]*e165-pin-red-examples"),\n',
     '    "verus-proof-check.py": re.compile(r"verus-proof-check"),\n'
     '    "e165-pin-red-examples.py": re.compile(r"e165-pin-red-examples"),\n'),
    ('    return verus_judge_arguments(named[0], arguments[invocation.script_index:], f"python -c 的代码里点了 {named[0]}")\n',
     '    return heavy_test("verus-stage", f"python -c 的代码里点了 {named[0]}（代码能自己改 sys.argv、直接调函数，不看代码之后的词）")   # K2\n'),
]
for old, new in edits:
    if text.count(old) != 1:
        sys.exit(f"没恰好命中一次：{old[:60]!r}（{text.count(old)} 次）")
    text = text.replace(old, new)
open(path, "w", encoding="utf-8").write(text)
print("改了 3 处")
