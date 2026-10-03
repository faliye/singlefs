#!/usr/bin/env python3
# 汇总 window_fuzz 的输出：每份文件一行。
import re, sys
for path in sys.argv[1:]:
    lines = [l for l in open(path) if l.startswith('FUZZ ')]
    n = len(lines)
    ch = sum(int(re.search(r'candidate_hits=(\d+)', l).group(1)) for l in lines)
    ah = sum(int(re.search(r'abandoned_hits=(\d+)', l).group(1)) for l in lines)
    chs = sum(1 for l in lines if int(re.search(r'candidate_hits=(\d+)', l).group(1)) > 0)
    ahs = sum(1 for l in lines if int(re.search(r'abandoned_hits=(\d+)', l).group(1)) > 0)
    uw = sum(int(re.search(r'unit_writes=(\d+)', l).group(1)) for l in lines)
    out = sum(1 for l in lines if int(re.search(r'outside_seen=(\d+)', l).group(1)) > 0)
    red = sum(1 for l in lines if 'checker_reds=[]' not in l)
    ab = sum(1 for l in lines if 'aborted=""' not in l)
    rolls = sum(len(re.findall(r'R\d+', l.split('log=')[1])) for l in lines)
    refused_r = sum(len(re.findall(r'r\d+', l.split('log=')[1])) for l in lines)
    fs = sum(l.split('log=')[1].count('F') for l in lines)
    hides = sum(l.count('|hideNewest') for l in lines)
    print(f"{path}: histories={n} unit_writes={uw} candidate_hit_histories={chs} candidate_hits={ch} abandoned_hit_histories={ahs} abandoned_hits={ah} histories_with_roots_outside_window={out} checker_red_histories={red} aborted={ab} rollbacks_done={rolls} rollbacks_refused={refused_r} raiseF_done={fs} hideNewest={hides}")
