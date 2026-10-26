#!/usr/bin/env python3
"""z3_membership.py <闭包目录>：发布 ∪ 回退（U2）、再加恢复（U3）之外，几条 GC 相关入口的闭包还剩多少 fn；回收门槛调用点在不在 U2 / U3；几个关键 fn 在不在。
闭包文件名：closure-<名>.txt（lib_function_units.py closure 的输出），dump-frozen.txt 是同一个工具 dump 的输出。冻结副本的源码路径从环境变量 FROZEN 取。"""
import collections, os, re, sys
work = sys.argv[1]
FZ = os.environ['FROZEN']
def fns(n):
    rows = {}
    for l in open(f'{work}/closure-{n}.txt', encoding='utf-8'):
        k, full, place, fp = l.rstrip('\n').split('\t')
        if k == 'fn': rows[full] = place
    return rows
def span(p):
    s, e = p.rsplit(':', 1)[1].split('-'); return int(e) - int(s) + 1
pub, rb, rec = fns('publish'), fns('roll_back_by_a_forward_publish'), fns('recover')
U2 = set(pub) | set(rb); U3 = U2 | set(rec)
print('name=z3_union U2_fn=%d U3_fn=%d recover_only_fn=%d recover_only_lines=%d' % (len(U2), len(U3), len(set(rec) - U2), sum(span(rec[f]) for f in set(rec) - U2)))
for n in ['run_compaction', 'run_compaction_in_the_background', 'discard_segments', 'raise_rollback_floor', 'raise_floor_to_ceiling', 'sweep_end_of_mount', 'sweep_while_mounted', 'scan_region', 'mount_writable']:
    c = fns(n); out = set(c) - U3
    print('name=z3_entry entry=%s closure_fn=%d outside_U3_fn=%d outside_U3_lines=%d' % (n, len(c), len(out), sum(span(c[f]) for f in out)))
units = []
for l in open(f'{work}/dump-frozen.txt', encoding='utf-8'):
    p = l.rstrip('\n').split('\t')
    if p[0] == 'fn' and p[1].startswith('singlefs_core::'):
        f, sp = p[2].rsplit(':', 1); a, b = map(int, sp.split('-')); units.append((p[1], f, a, b))
pat = re.compile(r'reclaim_released_up_to|reclaim_released_records_up_to|released_record_can_be_reclaimed')
cache = {}
for full, f, a, b in units:
    if '/tests/' in f or '::tests::' in full: continue
    if f not in cache: cache[f] = open(f'{FZ}/{f}', encoding='utf-8').read().split('\n')
    code = [ln for ln in cache[f][a - 1:b] if not ln.strip().startswith('//')]
    if any(pat.search(ln) for ln in code):
        tag = 'U2' if full in U2 else ('U3_only' if full in U3 else 'outside')
        print('name=z3_gate_caller where=%s fn=%s file=%s:%d-%d' % (tag, full.replace('singlefs_core::', ''), f.split('/')[-1], a, b))
for n in ['mount::raise_rollback_floor', 'mount::reclaim_released_in_the_sequence_raising_the_floor_up_to', 'mount::rebuilt_allocator', 'mount::reclaim_the_released_slots_of_the_rebuilt_allocator', 'mounted_session::discard_the_segments_reusable_as_a_whole', 'instance_table_sweep::erase_the_heads_and_publish_the_sweep', 'transaction::write_the_raised_floor_into_every_system_configuration']:
    full = 'singlefs_core::' + n
    print('name=z3_membership fn=%s in_U2=%d in_U3=%d' % (n, int(full in U2), int(full in U3)))
