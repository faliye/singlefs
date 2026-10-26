#!/usr/bin/env python3
"""G2 算术核：攒批（乙）与放宽 R（丙）要攒到多大才把提交的固定开销摊下去，与今天 crates/ 里三样串行约束
（每批之前重扫 R、全池至多一条活意图、一趟整理至多 B_gc 次发布）各差多少。
a、b 取 E185 第 1 次重跑第三段 name=q11_q12 两张卡的拟合；常量取 singlefs-format（槽 16384、聚簇段 64 槽、数据单元 32768、根环 3 区 × 8 槽）。
用法：python3 -I g2_merge_bounds.py <gpu-submission 产物>"""
import re, sys

def fields(line):
    return dict(m.groups() for m in re.finditer(r'(\w+)=(\S+)', line))

cards = {}
for l in open(sys.argv[1]):
    if l.startswith('E7RESULT name=q11_q12 '):
        f = fields(l); cards[f['index']] = (float(f['intercept_seconds']), float(f['slope_seconds_per_byte']))

SLOT, SEG_SLOTS, UNIT = 16384, 64, 32768
R_BYTES = SLOT * SEG_SLOTS                 # 一条意图的 R 在一块盘上的字节上界（R 槽数恒 64）
B_GC = 3 * 8 - 4                           # publish_budget_of_a_compaction_run 在刚有用户改动之后的值
MAPPING_ENTRY_GPU_BYTES = 28 + 2 * 16      # 27 字节 key 补到 4 的倍数 + 两条位置项（设备 4、槽号拆成两个 u32、校验和 4）——推的布局
for idx, (a, b) in sorted(cards.items()):
    for phi in (0.5, 0.1):
        need = a / b * (1 - phi) / phi       # φ = a / (a + b·B) ≤ phi 要的 B
        print(f'name=g2_merge card={idx} a_ms={a*1e3:.3f} b_ns_per_byte={b*1e9:.4f} phi_target={phi} bytes_needed={need:.0f} '
              f'intents_needed_at_full_R={need/R_BYTES:.1f} intents_needed_at_786432={need/786432:.1f} intents_needed_at_262144={need/262144:.1f}')
    phi_one_R = a / (a + b * R_BYTES)
    print(f'name=g2_one_intent card={idx} phi_when_a_whole_R_is_one_submission={phi_one_R:.4f}')
    for P in (1024, 8192):
        pool = P * UNIT
        print(f'name=g2_pool card={idx} P={P} pool_data_bytes={pool} phi_if_the_whole_pool_is_one_submission={a/(a+b*pool):.4f} '
              f'mapping_table_bytes_one_copy={P*MAPPING_ENTRY_GPU_BYTES} two_copies={2*P*MAPPING_ENTRY_GPU_BYTES}')
# 每条意图的批数 ⌈m/K⌉（m = 64f/2 个数据单元），攒 n 批要几条意图、几次发布，与 B_gc 比
for f in (0.25, 0.75):
    m = int(SEG_SLOTS * f) // 2
    for K in (4, 8, 16):
        batches = -(-m // K)
        publishes_per_intent = batches + 1      # 签发一次 + 每批一次（最后一批同发布删意图）
        for n in (125, 374):
            intents = -(-n // batches)
            print(f'name=g2_publishes f={f} K={K} batches_per_intent={batches} merged_batches={n} intents_spanned={intents} '
                  f'publishes_to_consume={intents*publishes_per_intent} b_gc_after_a_user_change={B_GC} '
                  f'runs_needed_at_least={-(-intents*publishes_per_intent//B_GC)}')
