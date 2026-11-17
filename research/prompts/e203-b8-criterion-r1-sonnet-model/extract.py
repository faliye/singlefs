#!/usr/bin/env python3
"""从 E206 的单点产物逐次发布算 B1、B4、B8 的准入之前扫类步数与几种分母，写 TSV。
B8 的式子照 e206_fixed_point_cost.rs 的 by_delta_rows 逐行重写（同一套式子在 b8_rows.py 里对过 63 行 name=q8）；
B1 = 三段里类型为扫的行的 steps 之和；B4 = 按 e203_admission_scan_cost.rs narrowed_steps_by_segment。
用法：extract.py <输出tsv> <产物>..."""
import sys, re, collections
SCAN = """whole_allocator_copy_before_the_judgement inode_record_linear_find files_other_than_the_first_plan file_other_than_the_first_linear_find inodes_with_an_extent_entry_set extent_upper_segment_plan extent_roles_rewritten_scan root_header_linear_find checkpoint_cost_accounting_tree_plan cluster_segment_sum allocation_record_tree_node_set allocation_record_tree_plan inode_number_watermark_steps deadlist_changes_indexes multi_level_tree_planning mapped_units_of_container_trees moved_entries_path_tail_filter release_judgement_version_indexes whole_allocator_copy_in_a_settling_round allocator_release_linear_find user_data_slot_linear_scan record_intersection_filter empty_segment_linear_scan fallback_slot_linear_scan records_of_each_leaf accounting_entries_of_this_publish instance_table_chain mapped_units_whole_table""".split()
INODE, SEG = 233, 64
def lg(v): return v.bit_length()
def l64(v):
    l = 1
    while 64 ** l < v: l += 1
    return l
def fields(line): return dict(t.split('=', 1) for t in line.split()[1:] if '=' in t)

def load(path):
    J = collections.defaultdict(lambda: {'rows': {}, 'deltas': [], 'rounds': None})
    states, settles, rounds_rows, point = {}, [], collections.defaultdict(list), {}
    for line in open(path):
        if not line.startswith('E206RESULT '): continue
        f = fields(line); n = f.get('name')
        if n == 'point': point = f
        elif f.get('phase') != 'window': continue
        elif n == 'settle': settles.append(f)
        elif n == 'state': states[f"{f['state']}/{f['op']}/{f['op_index']}/"] = f
        elif n == 'row': J[f['publish']]['rows'][(f['segment'], f['row'])] = (int(f['calls']), int(f['query_levels']), int(f['steps']))
        elif n == 'delta': J[f['publish']]['deltas'].append((f['segment'], int(f['fixed_point_round']), f['key'], int(f['value'])))
        elif n == 'publish': J[f['publish']]['rounds'] = int(f['fixed_point_rounds'])
        elif n == 'round': rounds_rows[(f['state'], f['op'], f['op_index'], f['settle_call'])].append(f)
    return point, J, states, settles, rounds_rows

def by_delta(j, st, seg, spd):
    S = lambda k: int(st[k])
    rounds = j['rounds'] if seg == 'between' else 0
    D = j['deltas']
    def prm(key):
        return [max(x[3] for x in D if x[0] == 'between' and x[1] == r and x[2] == key) for r in range(1, rounds + 1)]
    pub = lambda k: max([x[3] for x in D if x[2] == k], default=0)
    segv = lambda k: [x[3] for x in D if x[0] == seg and x[2] == k]
    cell = lambda r: j['rows'].get((seg, r), (0, 0, 0))
    c = lambda r: cell(r)[0]
    tf, lower, acc = pub('touched_files'), pub('lower_nodes_of_touched_files'), pub('accounting_rows_written_by_the_built_on_publish')
    other, ar, mu, dl = S('other_files'), S('allocation_records'), S('mapped_units'), S('deadlist_entries')
    cl, rel, rw = prm('changed_leaves'), prm('round_placements_released'), prm('round_roles_rewritten')
    keys = segv('code_two_keys_changed')
    hs = [S('accounting_tree_levels'), S('deadlist_tree_levels'), S('central_mapping_levels')]
    assert len(keys) == 3 * rounds
    trees = sum(k * hs[i % 3] for i, k in enumerate(keys))
    changed, atl, mc = sum(cl), S('allocation_tree_levels'), 2 * sum(rw)
    r = {}
    r['whole_allocator_copy_before_the_judgement'] = 0
    r['inode_record_linear_find'] = c('inode_record_linear_find') * (lg(S('inode_leaf_containers')) + lg(INODE))
    r['files_other_than_the_first_plan'] = c('files_other_than_the_first_plan') * tf * (lg(other) + lower)
    r['file_other_than_the_first_linear_find'] = c('file_other_than_the_first_linear_find') * lg(other)
    r['inodes_with_an_extent_entry_set'] = c('inodes_with_an_extent_entry_set') * tf * lg(other)
    r['extent_upper_segment_plan'] = c('extent_upper_segment_plan') * tf * S('extent_upper_levels') * lg(S('extent_upper_nodes'))
    r['extent_roles_rewritten_scan'] = c('extent_roles_rewritten_scan') * tf * lower
    r['root_header_linear_find'] = c('root_header_linear_find')
    r['checkpoint_cost_accounting_tree_plan'] = c('checkpoint_cost_accounting_tree_plan') * acc * S('accounting_tree_levels')
    r['cluster_segment_sum'] = c('cluster_segment_sum')
    r['allocation_record_tree_node_set'] = changed * atl
    r['allocation_record_tree_plan'] = changed * atl
    r['inode_number_watermark_steps'] = c('inode_number_watermark_steps')
    r['deadlist_changes_indexes'] = cell('deadlist_changes_indexes')[1] + pub('deadlist_entries_changed') * lg(dl)
    r['multi_level_tree_planning'] = trees
    r['mapped_units_of_container_trees'] = sum(segv('container_units_rewritten'))
    r['moved_entries_path_tail_filter'] = rounds * pub('moved_entries') * S('central_mapping_levels')
    r['release_judgement_version_indexes'] = cell('release_judgement_version_indexes')[1]
    r['whole_allocator_copy_in_a_settling_round'] = sum(a + b for a, b in zip(rel, rw))
    r['allocator_release_linear_find'] = c('allocator_release_linear_find') * lg(ar)
    r['user_data_slot_linear_scan'] = c('user_data_slot_linear_scan') * (l64(spd) + 32)
    r['record_intersection_filter'] = c('record_intersection_filter') * lg(ar) + sum(segv('intersecting_records'))
    r['empty_segment_linear_scan'] = c('empty_segment_linear_scan') * l64(spd)
    r['fallback_slot_linear_scan'] = c('fallback_slot_linear_scan') * (l64(spd) + 64)
    r['records_of_each_leaf'] = sum(segv('dirty_records'))
    r['accounting_entries_of_this_publish'] = c('accounting_entries_of_this_publish') * acc
    r['instance_table_chain'] = c('instance_table_chain')
    r['mapped_units_whole_table'] = mc + mc * lg(mu)
    assert set(r) == set(SCAN), set(r) ^ set(SCAN)
    return r

def main():
    out = open(sys.argv[1], 'x')
    out.write('\t'.join(['item', 'point', 'arm', 'unit_area_slots', 'unlinks', 'data_units', 'files', 'state', 'op', 'op_index', 'rounds',
        'rel_first', 'rw_first', 'rel_last', 'final_roles', 'sum_rel', 'slots_placed', 'B1', 'B4', 'B8', 'B4_pre', 'B1_pre', 'mapped', 'records', 'units']) + '\n')
    for path in sys.argv[2:]:
        m = re.search(r'fixed-point-cost-(.+)-(p1|p2a|plain|closure_off|single_step)-2026', path)
        point, J, states, settles, rounds_rows = load(path)
        if not any(J[k]['rows'] for k in J): continue
        spd = -(-int(point['unit_area_slots']) // SEG)
        for s in settles:
            key = f"{s['state']}/{s['op']}/{s['op_index']}/{s['judgement']}"
            j = J[key]; st = states[f"{s['state']}/{s['op']}/{s['op_index']}/"]
            b8 = {g: by_delta(j, st, g, spd) for g in ('caller', 'before_landing', 'between')}
            b1 = {g: {row: j['rows'].get((g, row), (0, 0, 0))[2] for row in SCAN} for g in b8}
            called = {row for g in ('caller', 'before_landing') for row in SCAN if j['rows'].get((g, row), (0, 0, 0))[0] > 0}
            B8 = sum(sum(b8[g].values()) for g in b8)
            B1 = sum(sum(b1[g].values()) for g in b1)
            pre = sum(b8['caller'].values()) + sum(b8['before_landing'].values())
            mid = sum((b8['between'][row] if row in called else b1['between'][row]) for row in SCAN)
            rr = sorted(rounds_rows[(s['state'], s['op'], s['op_index'], s['settle_call'])], key=lambda x: int(x['round']))
            out.write('\t'.join(map(str, [m[1] + '__' + m[2], point['point'], m[2], point['unit_area_slots'], point['unlinks'], point['data_units'], point['files'],
                s['state'], s['op'], s['op_index'], s['rounds'], rr[0]['placements_released'], rr[0]['roles_rewritten'], rr[-1]['placements_released'],
                s['final_roles'], sum(int(x['placements_released']) for x in rr), s['slots_placed'], B1, pre + mid, B8, pre,
                sum(sum(b1[g].values()) for g in ('caller', 'before_landing')), st['mapped_units'], st['allocation_records'], st['units']])) + '\n')
main()
