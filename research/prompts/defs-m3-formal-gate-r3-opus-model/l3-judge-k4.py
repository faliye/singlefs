#!/usr/bin/env python3
"""L3 改法草稿 K4（只在这份合成阶段上量过，被攻过零轮）：gate-triage 第 3 步「一道里每一条 ✗」改成只数格自己的 ✗，
不数格共用库的汇总行「✗ <阶段>：N 格里 M 格红（…）」与包装自己打的 ✗（Verus 一侧已带进同一行，这份合成阶段里没有单独的）。用法同 l3-judge.py。"""
import re, sys

NOT_RUN = "没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 "
SUMMARY = re.compile(r"✗ \S+\.sh：\d+ 格里 \d+ 格红（")
for path in sys.argv[1:]:
    crosses = [line for line in open(path, encoding="utf-8").read().split("\n") if line.lstrip().startswith("✗") and not SUMMARY.search(line)]
    verdict = "没有 ✗" if not crosses else ("没跑成（按码归环境 / 并发）" if all(NOT_RUN in line for line in crosses) else "红")
    print(f"{path.rsplit('/', 1)[-1]}：K4 读法下 gate-triage（整道，去掉汇总行）：{verdict}")
