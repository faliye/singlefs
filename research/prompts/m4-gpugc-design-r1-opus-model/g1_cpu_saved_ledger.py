#!/usr/bin/env python3
"""G1 算术核：按 D26 已定项 13 第 7 条那把尺（省下的主机 CPU 时间），从 E185 第 2 次重跑第二段产物逐批算
甲（crc_cpu）+ 乙增量（scan_increment_cpu）+ 丙（签发那一次的 choose_cpu）能省的上界，与一批的 CPU（call_cpu）比，
再算「GPU 模式每次提交在主机上至多能花多少 CPU 才不赔」的门槛，与第三段量到的提交挂钟截距 a 比。
只读产物，不改任何东西。用法：python3 -I g1_cpu_saved_ledger.py <stage-split 产物> <gpu-submission 产物>"""
import re, sys, statistics
from collections import defaultdict

def fields(line):
    return dict(m.groups() for m in re.finditer(r'(\w+)=(\S+)', line))

stage_path, gpu_path = sys.argv[1], sys.argv[2]
rows = [fields(l) for l in open(stage_path) if l.startswith('E7RESULT name=stage ')]
intercepts = {}
for l in open(gpu_path):
    if l.startswith('E7RESULT name=q11_q12 '):
        f = fields(l); intercepts[f['index']] = float(f['intercept_seconds'])

# 每条意图（f, P, K, 轮）：签发一次 + 若干批。每批之前都重扫 R（mounted_session.rs 第 2975 行），所以一批至少一次 GPU 提交。
intents = defaultdict(lambda: {'issue': None, 'batches': []})
for r in rows:
    key = (r['fraction'], r['pool_data_units'], r['batch_units'], r['round'])
    if r['kind'] == 'issue':
        intents[key]['issue'] = r
    else:
        intents[key]['batches'].append(r)

per_point = defaultdict(list)
for key, it in sorted(intents.items()):
    issue = it['issue']; batches = it['batches']
    call_cpu = sum(float(b['call_cpu_nanoseconds']) for b in batches) + float(issue['call_cpu_nanoseconds'])
    saved_batches = sum(float(b['crc_cpu_nanoseconds']) + float(b['scan_increment_cpu_nanoseconds']) for b in batches)
    saved_issue = float(issue['crc_cpu_nanoseconds']) + float(issue['scan_increment_cpu_nanoseconds']) + float(issue['choose_cpu_nanoseconds'])
    release_check = sum(float(b['release_check_crc_cpu_nanoseconds']) for b in batches)
    submissions = len(batches) + 1   # 签发一次、每批一次（每批之前重扫，结果不能跨批复用）
    saved = saved_batches + saved_issue
    per_point[key[:3]].append((call_cpu, saved, release_check, submissions))

print('name=g1_ledger_header columns=f,P,K,intent_cpu_ms_median,saved_upper_ms_median,saved_share_median,saved_share_max,submissions_per_intent,break_even_host_cpu_per_submission_ms,break_even_over_a_card0,break_even_over_a_card1,release_check_crc_ms_not_offloadable')
worst = []
for (f, P, K), vals in sorted(per_point.items(), key=lambda kv: (float(kv[0][0]), int(kv[0][1]), int(kv[0][2]))):
    cpu = statistics.median(v[0] for v in vals) / 1e6
    saved = statistics.median(v[1] for v in vals) / 1e6
    shares = [v[1] / v[0] for v in vals]
    subs = vals[0][3]
    be = saved / subs
    rc = statistics.median(v[2] for v in vals) / 1e6
    worst.append(be)
    print(f'name=g1_ledger f={f} P={P} K={K} intent_cpu_ms={cpu:.3f} saved_upper_ms={saved:.3f} saved_share={statistics.median(shares):.4f} saved_share_max={max(shares):.4f} submissions_per_intent={subs} break_even_host_cpu_per_submission_ms={be:.4f} break_even_over_a_card0={be/1e3/intercepts["0"]:.4f} break_even_over_a_card1={be/1e3/intercepts["1"]:.4f} release_check_crc_ms={rc:.3f}')
print(f'name=g1_ledger_summary points={len(per_point)} break_even_ms_min={min(worst):.4f} break_even_ms_max={max(worst):.4f} a_card0_ms={intercepts["0"]*1e3:.3f} a_card1_ms={intercepts["1"]*1e3:.3f}')
