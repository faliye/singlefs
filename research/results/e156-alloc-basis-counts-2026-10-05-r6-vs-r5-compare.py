#!/usr/bin/env python3
# E156（alloc-basis 四条岔路的代价数） 第 6 次重跑登记 `research/prompts/e156-r6-prereg.md` 第一节 W22、第六节 R2b / R2c 的比较。
# 只读两份产物，不 import 装置、不读 crates/。用法：python3 本脚本 <第 5 次产物> <第 6 次产物>
# 比的行：failure_clauses、q1c、q1d、q1c_front_threshold_moved、geometry_sensitivity、verdict stage=second_group；
# 配对键 = 行名 + 组标签（s、rho、timing、quantity 中这一行有的）；配上的按空白拆成 键=值 逐字段比。
# 「实 / 每」：组标签 timing=real 归实、timing=every_publish 归每；没有 timing 的按字段名（只带 real 归实、只带 every 归每），其余「不分列」。
# 判定翻面（R2b、F33、F34）：failure_clauses 的 f1_front_above_two_groups 由 none 变非 none 或反过来；Q1c-前（quantity=q1c_front 的 q1c 行）的
# judgement_two_slots / judgement_one_slot、q1d 的 growth_by_h_que、verdict second_group 的 q1c_front / q1d_growth、
# geometry_sensitivity quantity ∈ {q1c_front, q1d_growth} 的 result 变了。其余判定字段变了单列（不触发 F33、F34）。
import re, sys

NAMES = ('failure_clauses', 'q1c', 'q1d', 'q1c_front_threshold_moved', 'geometry_sensitivity', 'verdict')
LABEL_KEYS = ('s', 'rho', 'timing', 'quantity')
JUDGEMENT_FIELDS = ('judgement_two_slots', 'judgement_one_slot', 'growth_by_h_que', 'q1c', 'q1c_on_holes',
                    'q1c_front', 'q1c_back', 'q1d_growth', 'result', 'judgement_between', 'judgement_at_largest', 'flips')


def load(path):
    rows = {}
    for line in open(path, encoding='utf-8'):
        fields = line.split()
        if not fields or fields[0] != 'E7RESULT':
            continue
        pairs = [field.split('=', 1) for field in fields[1:] if '=' in field]
        values = dict(pairs)
        name = values.get('name')
        if name not in NAMES:
            continue
        if name == 'verdict' and values.get('stage') != 'second_group':
            continue
        label = tuple((key, values[key]) for key in LABEL_KEYS if key in values)
        key = (name,) + label
        if key in rows:
            raise SystemExit(f'配对键重复：{key}')
        rows[key] = values
    return rows


def column(key, field):
    label = dict(key[1:])
    timing = label.get('timing')
    if timing == 'real':
        return '实'
    if timing == 'every_publish':
        return '每'
    has_real, has_every = 'real' in field, 'every' in field
    if has_real and not has_every:
        return '实'
    if has_every and not has_real:
        return '每'
    return '不分列'


def is_flip(key, field, old, new):
    name, label = key[0], dict(key[1:])
    if name == 'failure_clauses' and field == 'f1_front_above_two_groups':
        return (old == 'none') != (new == 'none')
    if name == 'q1c' and label.get('quantity') == 'q1c_front' and field in ('judgement_two_slots', 'judgement_one_slot'):
        return True
    if name == 'q1d' and field == 'growth_by_h_que':
        return True
    if name == 'verdict' and field in ('q1c_front', 'q1d_growth'):
        return True
    if name == 'geometry_sensitivity' and label.get('quantity') in ('q1c_front', 'q1d_growth') and field == 'result':
        return True
    return False


old_rows, new_rows = load(sys.argv[1]), load(sys.argv[2])
pairs = sorted(set(old_rows) & set(new_rows))
only_old = sorted(set(old_rows) - set(new_rows))
only_new = sorted(set(new_rows) - set(old_rows))
unequal = 0
per_column = {'实': 0, '每': 0, '不分列': 0}
flips, other_judgement_changes, lines = [], [], []
for key in pairs:
    old, new = old_rows[key], new_rows[key]
    changed = [field for field in sorted(set(old) | set(new)) if old.get(field) != new.get(field)]
    if not changed:
        continue
    unequal += 1
    for field in changed:
        col = column(key, field)
        per_column[col] += 1
        old_value, new_value = old.get(field, '<无>'), new.get(field, '<无>')
        lines.append(f"DIFF {' '.join('='.join(part) for part in key[1:]) or '-'} name={key[0]} field={field} column={col} old={old_value} new={new_value}")
        if is_flip(key, field, old_value, new_value):
            flips.append(f"FLIP name={key[0]} {' '.join('='.join(part) for part in key[1:])} field={field} old={old_value} new={new_value}")
        elif field in JUDGEMENT_FIELDS:
            other_judgement_changes.append(f"OTHER_JUDGEMENT name={key[0]} {' '.join('='.join(part) for part in key[1:])} field={field} old={old_value} new={new_value}")
print(f"pairs={len(pairs)} unequal_pairs={unequal} only_old={len(only_old)} only_new={len(only_new)} differing_fields_real={per_column['实']} differing_fields_every={per_column['每']} differing_fields_unsplit={per_column['不分列']} judgement_flips={len(flips)} other_judgement_changes={len(other_judgement_changes)}")
for key in only_old:
    print(f"ONLY_OLD {key}")
for key in only_new:
    print(f"ONLY_NEW {key}")
for line in flips + other_judgement_changes + lines:
    print(line)
