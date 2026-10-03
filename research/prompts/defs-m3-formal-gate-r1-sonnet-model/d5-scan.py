#!/usr/bin/env python3
"""D5 机械那一半：把上游 doc-lint.sh 的「上下文指代」判据（.claude/singlefs-ai-sop/scripts/doc-lint.sh 里 ctx 那几行，抄成 python 正则）
与自称词「本节 本章节 本表 本文档 本条」跑在被判 diff 的加行上；只读，不写仓内文件。
用法：python3 d5-scan.py <diff 文件>"""
import re, sys
diff = open(sys.argv[1], encoding="utf-8").read().split("\n")
ctx = '如上所述|如前所述|同上[，。；：、）」]|同上$|见上文|见上面|前面提到|上一节([^点拍]|$)|前述|下面会说|后面会说'
ctx += '|见[上下](表|文|节|条|图|面|$|[，。；：、）」])|[如照][上下](表|图)|上面[那这]|下面[那这]'
dlead = '(^|[，。；：、！？（）()「」『』【】〔〕〈〉《》“”"*|\\s])'
ctx += '|[上下]述|[上下]表([^格头面达]|$)|上文([^件]|$)|(^|[^上])下文([^件]|$)'
ctx += '|见[上下]一(段|行|节)|同上一行|[早晚]于[上下]一行|如下[：:，。；—]'
ctx += '|见[上下]方|下一节([^点拍]|$)'
ctx += '|' + dlead + '以[上下](是|几[条点项段]|各[条点项段])'
ctx += '|(' + dlead + '|[以把按取在是了成进含从为])(上面|下面)(的|是|[一二三四五六七八九十几两各每]|第|保留| ?[0-9①②③④⑤⑥⑦⑧⑨⑩⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳])'
ctx += '|(列|照录|附|放)在(上面|下面)'
selfref = '本节|本章节|本表|本文档|本条'
inscope = (".claude/agent-common.md", ".claude/agents/", ".claude/rules/", ".claude/skills/", ".claude/main-agent.md")
current, scanned, hits = None, 0, 0
for number, line in enumerate(diff, 1):
    if line.startswith("diff --git"): current = line.split(" b/")[-1]
    if line.startswith("+") and not line.startswith("+++") and current and current.startswith(inscope):
        scanned += 1
        match = re.search(ctx, line[1:]) or re.search(selfref, line[1:])
        if match:
            hits += 1
            print(f"命中 {current} diff 文件第 {number} 行 {match.group(0)!r}")
print(f"扫了 {scanned} 条射程内的加行，命中 {hits} 条")
