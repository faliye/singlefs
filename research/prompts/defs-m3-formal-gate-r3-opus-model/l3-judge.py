#!/usr/bin/env python3
"""L3：把合成阶段的输出照两份定义 3-12 改后的字面机械地判一遍。用法：python3 l3-judge.py <输出文件>…
  crash-verifier 第 3 步：「一格里每一条「✗」都写着「没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 <码>」的 … 记「没跑成」；同一格里另有一条不是这样写的「✗」（真红）的，那一格记红」
    读法 a：「一格」= 那一格自己的段（── <格名>： 到下一个 ── 之前）；读法 b：再加汇总段里点名这一格的 ✗ 行。
  gate-triage 第 3 步：「一道里每一条 ✗ 都写着内存包装退 25x 的 … 不按它点名的文件判：…；同一道里另有一条不是这样写的 ✗（真红）的，那一道记红」——「一道」= 这一道的整段输出。"""
import re, sys

NOT_RUN = "没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 "
for path in sys.argv[1:]:
    lines = open(path, encoding="utf-8").read().split("\n")
    sections, current = {}, None
    for line in lines:
        match = re.match(r"^── (\S+?)：", line)
        if match:
            current = match.group(1)
            sections.setdefault(current, [])
            continue
        if current:
            sections[current].append(line)
    crosses = [line for line in lines if line.lstrip().startswith("✗")]
    cell = "verus-must-red-worlds"
    own = [line for line in sections.get(cell, []) if line.lstrip().startswith("✗")]
    summary_naming = [line for line in sections.get("汇总", []) if line.lstrip().startswith("✗") and cell in line]
    def verdict(group):
        if not group:
            return "没有 ✗"
        return "没跑成" if all(NOT_RUN in line for line in group) else "红"
    others = [line.strip()[:90] for line in crosses if NOT_RUN not in line]
    print(f"{path.rsplit('/', 1)[-1]}：格里 ✗ {len(own)} 条（没跑成形 {sum(NOT_RUN in l for l in own)} 条）；整道 ✗ {len(crosses)} 条（没跑成形 {sum(NOT_RUN in l for l in crosses)} 条）")
    print(f"  crash-verifier 读法 a（只看格段）：{verdict(own)}；读法 b（加汇总里点名这一格的 ✗）：{verdict(own + summary_naming)}")
    print(f"  gate-triage（整道）：{verdict(crosses)}；整道里不是没跑成形的 ✗：{others}")
