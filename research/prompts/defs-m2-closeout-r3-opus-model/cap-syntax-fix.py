#!/usr/bin/env python3
"""defs-m2-closeout-r3 云端攻方 K1 的改法（只在我的副本上量过、被攻过零轮）：15、74 号在调包装之前先核上限写法，
要求「正整数加 K/M/G/T」（单位必写：不带单位的「16」在 systemd 写法里是 16 字节），写错单独判红、出路指到变量名。
用法：cap-syntax-fix.py <原阶段文件> <改后写到哪>（只写第二个参数给的临时路径，不改仓里任何文件）。"""
import sys

source, target = sys.argv[1], sys.argv[2]
text = open(source, encoding="utf-8").read()
if "15-research-build" in source:
    anchor, variable, value = 'out="$(cd "$R" && RUN_WITH_MEMORY_CAP_KEY=', "GATE_RESEARCH_BUILD_MEMORY_MAX", "$RESEARCH_BUILD_MEMORY_MAX"
else:
    anchor, variable, value = 'log="$(mktemp)"', "GATE_MODEL_DIFFERENTIAL_MEMORY_MAX", "$MODEL_DIFFERENTIAL_MEMORY_MAX"
guard = (f'if [[ ! "{value}" =~ ^[1-9][0-9]*[KMGT]$ ]]; then\n'
         f'  echo "  ✗ {variable} 写成了「{value}」，内存包装不认（或认成字节数）：这一道没跑"\n'
         f'  echo "     → 怎么办：写成正整数加 K / M / G / T（例 16G），单位必写；不设就用默认 8G"\n'
         f'  exit 1\nfi\n')
assert text.count(anchor) == 1, anchor
open(target, "w", encoding="utf-8").write(text.replace(anchor, guard + anchor))
