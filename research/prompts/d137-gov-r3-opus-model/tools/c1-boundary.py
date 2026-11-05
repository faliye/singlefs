#!/usr/bin/env python3
"""C1（main-agent.md 三种处置一行只归一种）在窗口跨过 7200 秒那一刻：同一行既写「声明解释得了」又写「拖过了它算数的 7200 秒」时归第一种（只记）。
用法：c1-boundary.py <根>。造的历史：程序甲第 0 秒写「此刻串行（预计 7200 秒）」、从不收；整棵树一直 r 核、6 条线程（r 扫 3.5–5.5），判第 7500、8100 秒那两个窗口。只调 core_usage 的函数。"""
import os
import sys

sys.path.insert(0, os.path.join(sys.argv[1], "scripts", "hosts"))
import core_usage as cu  # noqa: E402

origin = 1_000_000.0
text = f"{origin}\t{cu.SERIAL_MARK_WORD}（预计 7200 秒）：程序甲的计时窗口必须单线程\t{cu.WRITER_MARK}111\n"
for rate in (3.5, 4.5, 5.5):
    for now in (7500, 8100):
        moments = list(range(300, now, 300)) + [now]
        samples = [{"root": 1, "time": origin + m, "started": origin, "cpu": rate * m, "peer_drivers": 0,
                    "thread_cpu": {f"1/{i}/0": rate * m / 6 for i in range(6)}} for m in moments]
        verdict = cu.judge_series(samples, *cu.parse_declarations(text), 31, 600)
        line = cu.verdict_lines(verdict, "本机")[0]
        both = "声明解释得了" in line and "拖过了它算数的 7200 秒" in line
        print(f"name=c1_boundary rate={rate} now={now} status={verdict['status']} excused={verdict['declaration_excused']} both_phrases={both}")
        print(f"  line: {line[:260]}")
