#!/usr/bin/env python3
"""gen_runs.py <旧日志> ...：从上一轮探针日志的 S2 行里读出每个批的前缀、R、故障组、动作串，生成 S2_* 环境变量（每个 前缀×R×故障 一行）。
输出一行一件： <新日志名> S2_SLOTS=.. S2_PREFIXES=.. S2_RS=.. S2_FAULTS=.. S2_ACTIONS=.. S2_LABEL=.. [S2_PERSIST=1]"""
import re, sys, os, collections
extra = os.environ.get('EXTRA', '')
for f in sys.argv[1:]:
    name = os.path.basename(f)[:-4]
    groups = collections.OrderedDict()
    for line in open(f, errors='replace'):
        if not line.startswith('S2 label='): continue
        kv = dict(re.findall(r'(\w+)=(\S*)', line))
        if 'record' in kv: 
            key = (kv.get('slots', '384'), kv['prefix'], kv['mid'], kv['r'], None, None)
            continue
        key = (kv['slots'], kv['prefix'], kv['mid'], kv['r'], kv['faults'], kv['persist'])
        groups.setdefault(key, []).append(kv['actions'])
    # 合并：同 (slots, faults, persist, 动作集) 的 前缀、R
    merged = collections.OrderedDict()
    for (slots, prefix, mid, r, faults, persist), acts in groups.items():
        k = (slots, mid, faults, persist, tuple(acts))
        merged.setdefault(k, [[], []])
        if prefix not in merged[k][0]: merged[k][0].append(prefix)
        if r not in merged[k][1]: merged[k][1].append(r)
    for i, ((slots, mid, faults, persist, acts), (prefixes, rs)) in enumerate(merged.items()):
        env = f"S2_LABEL={name}{'' if len(merged)==1 else '_'+str(i)} S2_SLOTS={slots} S2_PREFIXES='{';'.join(prefixes)}' S2_RS='{';'.join(rs)}' S2_FAULTS='{faults}' S2_MID='{mid}' S2_ACTIONS='{','.join(acts)}'"
        if persist == 'true': env += ' S2_PERSIST=1'
        print(f"{name}{'' if len(merged)==1 else '_'+str(i)} {env} {extra}")
