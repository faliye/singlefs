#!/usr/bin/env python3
"""把 sweep 的 results.txt 合成两张表（r2 辩方腿）。
用法：summarize.py <results.txt>…
表一（覆盖）：每个候选 × 每段历史，守规矩且跑过门禁的续作里一次都没判到的条数 / 守规矩续作条数（与 grid.py 同口径）。
表二（重判代价）：每个候选 × 每段历史，守规矩且跑过门禁的续作里，被追踪的那一行在前缀之后又进过窗口的总次数、单条续作里的最大次数、至少进过一次的续作条数。
并列出「与庚二（geng2）覆盖逐段相同」的候选。
"""
import collections, sys
rows = collections.defaultdict(list)
for path in sys.argv[1:]:
    for line in open(path, encoding='utf-8'):
        if line.startswith('result '):
            f = dict(p.split('=', 1) for p in line.split()[1:] if '=' in p)
            rows[(f['hist'], f['cand'])].append(f)
def gated(c): return c != 'none' and any(x in 'NSG' for x in c)
PREFIX_ENDS_WITH_GATE = {'h1_c439', 'h9_aba', 'h9_stash', 'h9_commit_undo', 'h15_override'}
def disciplined(c, hist):
    if c == 'none': return False
    return all(c[i] not in 'ck' or (c[i-1] in 'NSG' if i > 0 else hist in PREFIX_ENDS_WITH_GATE) for i in range(len(c)))
hists = sorted({h for h, _ in rows}); cands = sorted({c for _, c in rows})
cover = {}; cost = {}
for (h, c), items in rows.items():
    assert all(f.get('rc') == '0' for f in items), (h, c)
    d = [f for f in items if gated(f['cont']) and disciplined(f['cont'], h)]
    cover[(h, c)] = (sum(1 for f in d if f['judged_ever'] == 'no'), len(d))
    ac = [int(f.get('after_count', '0')) for f in d]
    cost[(h, c)] = (sum(ac), max(ac, default=0), sum(1 for x in ac if x > 0))
print('# 表一：守规矩续作里一次都没判到的条数/守规矩续作条数')
for h in hists:
    print(f'name=cover hist={h} ' + ' '.join(f'{c}={cover[(h,c)][0]}/{cover[(h,c)][1]}' for c in cands if (h, c) in cover))
print('# 表二：被追踪的行在前缀之后进窗口的 总次数/单条续作最大次数/至少进过一次的续作条数')
for h in hists:
    print(f'name=cost hist={h} ' + ' '.join(f'{c}={cost[(h,c)][0]}/{cost[(h,c)][1]}/{cost[(h,c)][2]}' for c in cands if (h, c) in cost))
print('# 与 geng2 覆盖逐段相同（22 段里相同的段数）')
for c in cands:
    same = sum(1 for h in hists if cover.get((h, c)) == cover.get((h, 'geng2')))
    print(f'name=same_as_geng2 cand={c} same={same}/{len(hists)}')
BUG = ['h1_c439','h2_noverify','h3_split','h4_other','h5_pushed','h2u_unpushed','h6_stamp_S','h6_stamp_N','h6_stamp_revert','h8_amend','h9_aba','h9_stash','h9_commit_undo','h15_override']
WU = ['h11_dup','h12_exempt_grow','h17_wu_move','h18_wu_copy_other_page']
print('# 合计：14 段「错行绕过门禁提交 / 交错 / 人给 GATE_BASE」历史上守规矩续作一次都没判到的条数 / 守规矩续作条数')
for c in cands:
    if not all((h, c) in cover for h in BUG): continue
    never = sum(cover[(h, c)][0] for h in BUG); total = sum(cover[(h, c)][1] for h in BUG)
    nogate = 0
    for h in BUG:
        for f in rows[(h, c)]:
            k = f['cont']
            if gated(k) and disciplined(k, h) and f['judged_ever'] == 'no' and not any(x in 'NS' for x in k): nogate += 1
    print(f'name=total14 cand={c} never={never}/{total} of_which_no_N_or_S={nogate}')
print('# 合计：戊一族的 4 段历史（重复抄存量豁免行 / 绕过提交同时涨表 / 同页搬家 / 抄到另一页）')
for c in cands:
    if not all((h, c) in cover for h in WU): continue
    print(f'name=total_wu cand={c} ' + ' '.join(f'{h}={cover[(h,c)][0]}/{cover[(h,c)][1]}' for h in WU))
print('# 全部历史：守规矩续作里「带 N 或 S 而一次都没判到」的条数 / 守规矩续作条数（G-only 的续作不算：那是人每一趟都带 GATE_BASE 自己收窄窗口）')
for c in cands:
    never = total = 0
    for h in hists:
        for f in rows.get((h, c), []):
            k = f['cont']
            if gated(k) and disciplined(k, h):
                total += 1
                if f['judged_ever'] == 'no' and any(x in 'NS' for x in k): never += 1
    print(f'name=never_with_NS cand={c} never={never}/{total}')
