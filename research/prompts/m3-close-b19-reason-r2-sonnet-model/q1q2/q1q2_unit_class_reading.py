"""Q1 / Q2 的小世界（推的模型，不是入库装置，不跑 crates）：I-3.1 取宽那一支（整份改记读法加逐槽两判）今天的实现，与 `!=` 改 `>`、不取宽两个替代方案，
放在同一批盘面上逐个盘面比；再问 C944（走到的单元起点没有自己的记录）在没有别的不变量红的情况下造不造得出来。
移植对象（crates/singlefs-checker/src/walk.rs 今天的行号）：
  allocated_statistic_bounds_of_a_device（10398 起）、allocated_statistic_lies_within_its_bounds（10467 起）、
  records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes（10491 起，`!=` 在 10503）、
  allocated_statistic_lies_within_the_bounds_of_the_unit_class_reading（10523 起）、quarantined_slots_exempted_per_device（10180 起）。
世界：一块盘、槽 0..SLOTS-1；至多两个单元，类 1 占一槽、类 2 占两槽，种类 W（最新根走读到）或 Q（隔离：没有引用，记录带隔离记号）；
  每个单元一条记录（没有 / 跨度 类-1、类、类+1 的未释放 / W 的已释放且释放代 4 或 7），另可加一条额外记录（任一槽上一槽宽的未释放，或释放代 4 / 7 的已释放）；
  回收门槛 θ_high = 5，最新根自己带的 t 取 5 或 3；记账的已分配 A 取 {按单元该记的数 A_true - 1, A_true, A_true + 1, A_true + 释放代高于 t 的额外已释放槽数}。
  记录元组 (起点槽, 跨度, 释放代, 已释放?, 带隔离记号?)；未释放的记录释放代记 1。
判（对不上的写法逐条标出）：
  I-3.1：按记录自述三判（L、P、U）成立 -> holds；不成立再看取宽——mode 'ne'（今天，`!=`）/ 'gt'（`>`，只改记录比单元窄的）/ 'none'（不取宽）——
         改记之后整份三判加逐槽两判（P 已在三判里，另加「E(t) 的每一槽都在 Ω ∪ Q ∪ U 里」）成立 -> partial，否则 red。
         Q（隔离豁免的槽）按改记之前的记录算，walk.rs 里也是这样（quarantined_exempted_against_every_walked_version 不随改记重算）。
  I-3.18：有起点槽上单元头用得了（= 起点槽是某个单元的起点）的未释放记录，跨度 != 那一类的槽数。
  I-5.4：任意两条记录（不论已释放）按记录自述的跨度罩住的槽相交。
  I-3.15 只取一小块：未释放、不带记号的记录起点是 Q 单元的起点（头用得了、映射里没有 key）-> 红；起点在别处头用不了，没有对象。
  I-3.17 只取一小块：带记号的记录罩住的槽被最新根引用（Ω）-> 红。
  I-5.1：单元互不相交、引用按类的槽数，世界里不会红，不建。
  没建：I-3.10、I-3.11、I-3.16、I-5.2。这几条在世界里的判法没移植，下面「别的不变量没红」只对已建的几条说。
真值（盘面「病」的种类，对着单元算）：missing（单元没有记录）、released_live（单元的记录是已释放的）、span_wrong（未释放记录跨度 != 类）、
  stray_unreleased（不在任何单元起点的未释放额外记录）、duplicate_unreleased（单元起点上第二条未释放记录）、released_over_unit（额外已释放记录罩着单元的槽）、
  accounting_off（A 不是 A_true 也不是 A_true + defer 额外槽数）。
用法：python3 -B q1q2_unit_class_reading.py [slots]
"""
import sys
from collections import Counter

SLOTS = 7
THETA = 5
GENS = (4, 7)
T_CHOICES = (5, 3)
MODES = ('ne', 'gt', 'none')
SIDE = Counter()


def unit_layouts():
    out = [()]
    for s1 in range(SLOTS):
        for c1 in (1, 2):
            if s1 + c1 > SLOTS:
                continue
            for k1 in 'WQ':
                out.append(((s1, c1, k1),))
                for s2 in range(s1 + c1, SLOTS):
                    for c2 in (1, 2):
                        if s2 + c2 > SLOTS:
                            continue
                        for k2 in 'WQ':
                            out.append(((s1, c1, k1), (s2, c2, k2)))
    return out


def record_options(unit):
    s, c, k = unit
    options = [(None, 'absent')]
    for span in (c - 1, c, c + 1):
        if span >= 1:
            options.append(((s, span, 1, False, k == 'Q'), 'u%d' % span))
    if k == 'W':
        for g in GENS:
            options.append(((s, c, g, True, False), 'r%d' % g))
    return options


def extra_options():
    options = [(None, 'none')]
    for slot in range(SLOTS):
        options.append(((slot, 1, 1, False, False), 'x_u@%d' % slot))
        for g in GENS:
            options.append(((slot, 1, g, True, False), 'x_r%d@%d' % (g, slot)))
    return options


def counted_at(records, theta):
    slots = set()
    for (slot, span, gen, released, _mark) in records:
        if (not released) or gen > theta:
            slots.update(range(slot, slot + span))
    return slots


def quarantine_exempted(records, walked_starts):
    """quarantined_slots_exempted_per_device：未释放、起点没有引用的记录按 (起点, 跨度) 归组，组里每条都带记号的整组豁免。"""
    groups = {}
    for r in records:
        if r[3] or r[0] in walked_starts:
            continue
        groups.setdefault((r[0], r[1]), []).append(r)
    slots, count = set(), 0
    for (slot, span), copies in groups.items():
        if all(c[4] for c in copies):
            for _ in copies:
                count += span
                slots.update(range(slot, slot + span))
    return slots, count


class Ctx:
    def __init__(self, units, records, t):
        self.t = t
        self.walked = {x for (s, c, k) in units if k == 'W' for x in range(s, s + c)}
        self.walked_starts = {s for (s, c, k) in units if k == 'W'}
        self.classes = {s: c for (s, c, k) in units}
        self.omega_count = len(self.walked)
        self.q_slots, self.q_count = quarantine_exempted(records, self.walked_starts)


def bounds(records, ctx):
    e_high = counted_at(records, THETA)
    e_t = counted_at(records, ctx.t)
    deferred = {x for (s, sp, g, rel, _m) in records if rel and g > ctx.t for x in range(s, s + sp) if x not in ctx.walked}
    return dict(
        L=len(e_high), U1=len(e_t), WD=ctx.omega_count + ctx.q_count + len(deferred),
        outside=[x for x in sorted(ctx.walked) if x not in e_high],
        upper_slots=[x for x in sorted(e_t) if x not in ctx.walked and x not in ctx.q_slots and x not in deferred])


def lies_within(allocated, b, unit_class_reading):
    ok = allocated >= b['L'] and not b['outside'] and allocated <= min(b['U1'], b['WD'])
    return ok and (not unit_class_reading or not b['upper_slots'])


def set_to_unit_classes(records, classes, mode):
    out, changed = [], False
    for r in records:
        s, span, g, rel, mk = r
        c = classes.get(s)
        if (not rel) and c is not None and ((mode == 'ne' and c != span) or (mode == 'gt' and c > span)):
            out.append((s, c, g, rel, mk))
            changed = True
        else:
            out.append(r)
    return out, changed


def verdict31(allocated, records, ctx, mode):
    if lies_within(allocated, bounds(records, ctx), False):
        return 'holds'
    if mode == 'none':
        return 'red'
    reset, changed = set_to_unit_classes(records, ctx.classes, mode)
    if not changed:
        return 'red'
    b2 = bounds(reset, ctx)
    if lies_within(allocated, b2, False) and b2['upper_slots']:
        SIDE['upper_slot_judge_alone_turns_partial_into_red_mode_' + mode] += 1
    return 'partial' if lies_within(allocated, b2, True) else 'red'


def allowed_accounting(units, extra, ctx):
    """按单元该记的数：(a_true, 下限, 上限)。额外的已释放记录若释放代高于 θ_high 写者一定把它记在已分配里（下限含它），
    释放代只高于 t 时可记可不记（下限不含、上限含）。"""
    a_true = len(ctx.walked) + sum(c for (s, c, k) in units if k == 'Q')
    extra_slots = set()
    if extra and extra[3]:
        extra_slots = {x for x in range(extra[0], extra[0] + extra[1]) if x not in ctx.walked}
    over_t = len(extra_slots) if extra and extra[2] > ctx.t else 0
    over_theta = len(extra_slots) if extra and extra[2] > THETA else 0
    return a_true, a_true + over_theta, a_true + over_t


def diseases(units, unit_records, extra, allocated, ctx):
    kinds = set()
    unit_slots = {x for (s, c, k) in units for x in range(s, s + c)}
    records = [r for r in unit_records if r] + ([extra] if extra else [])
    for (s, c, k) in units:
        at = [r for r in unit_records if r and r[0] == s]
        if not at:
            kinds.add('missing')
        elif all(r[3] for r in at):
            kinds.add('released_live')
        elif any((not r[3]) and r[1] != c for r in at):
            kinds.add('span_wrong')
    if extra:
        if extra[0] in ctx.classes and not extra[3]:
            kinds.add('duplicate_unreleased')
        elif not extra[3]:
            kinds.add('stray_unreleased')
        elif any(x in unit_slots for x in range(extra[0], extra[0] + extra[1])):
            kinds.add('released_over_unit')
    a_true, lower, upper = allowed_accounting(units, extra, ctx)
    deferred_extra = set()
    if not (lower <= allocated <= upper):
        kinds.add('accounting_off')
    return kinds, records, a_true, deferred_extra


def other_invariants(records, ctx):
    i318 = any((not r[3]) and r[0] in ctx.classes and ctx.classes[r[0]] != r[1] for r in records)
    spans = [(r[0], r[0] + r[1]) for r in records]
    i54 = any(spans[i][0] < spans[j][1] and spans[j][0] < spans[i][1]
              for i in range(len(spans)) for j in range(i + 1, len(spans)))
    quar_starts = {s for s, c in ctx.classes.items() if s not in ctx.walked_starts}
    i315 = any((not r[3]) and (not r[4]) and r[0] in quar_starts for r in records)
    i317 = any(r[4] and any(x in ctx.walked for x in range(r[0], r[0] + r[1])) for r in records)
    return i318, i54, i315, i317


def main():
    tally = Counter()
    examples = {}
    layouts = unit_layouts()
    extras = extra_options()
    for units in layouts:
        per_unit = [record_options(u) for u in units]
        combos = [[]]
        for opts in per_unit:
            combos = [c + [o] for c in combos for o in opts]
        for combo in combos:
            unit_records = [o[0] for o in combo]
            unit_tags = '+'.join(o[1] for o in combo) or '-'
            for extra, extra_tag in extras:
                for t in T_CHOICES:
                    records = [r for r in unit_records if r] + ([extra] if extra else [])
                    ctx = Ctx(units, records, t)
                    a_true, lower, upper = allowed_accounting(units, extra, ctx)
                    i318, i54, i315, i317 = other_invariants(records, ctx)
                    named_elsewhere = i54 or i315 or i317
                    for allocated in sorted({max(a_true - 1, 0), a_true, a_true + 1, lower, upper}):
                        kinds, _records, _a_true, _d = diseases(units, unit_records, extra, allocated, ctx)
                        verdicts = {m: verdict31(allocated, records, ctx, m) for m in MODES}
                        tally['worlds'] += 1
                        for m in MODES:
                            tally[('verdict', m, verdicts[m])] += 1
                        tally[('triple', verdicts['ne'], verdicts['gt'], verdicts['none'])] += 1
                        size = (len(units), len(records), SLOTS)
                        world = (units, unit_tags, extra_tag, t, allocated)
                        if not kinds:
                            for m in MODES:
                                if verdicts[m] != 'holds':
                                    tally[('healthy_but_not_holds', m, verdicts[m])] += 1
                                    examples.setdefault(('healthy_but_not_holds', m), (size, world))
                        only_span = kinds == {'span_wrong'}
                        if only_span:
                            for m in MODES:
                                tally[('only_span_wrong', m, verdicts[m])] += 1
                        if verdicts['ne'] == 'partial':
                            other = sorted(kinds - {'span_wrong'})
                            label = '+'.join(other) if other else 'only_span_wrong'
                            tally[('partial_ne', label, 'named_by_another_invariant' if named_elsewhere else 'only_I-3.18')] += 1
                            tally[('partial_ne_i318_red', i318)] += 1
                            if other and not named_elsewhere:
                                key = ('partial_ne_hidden', label)
                                if key not in examples or size < examples[key][0]:
                                    examples[key] = (size, world, sorted(kinds))
                            if verdicts['gt'] != 'partial':
                                tally[('partial_ne_but_gt_says', verdicts['gt'], 'only_span_wrong' if only_span else 'other_kinds')] += 1
                        if verdicts['gt'] == 'partial' and verdicts['ne'] != 'partial':
                            tally[('partial_gt_but_ne_says', verdicts['ne'])] += 1
                        pure = ({'missing', 'released_live'} & kinds) and not (i318 or i54 or i315 or i317)
                        if pure:
                            for m in MODES:
                                if verdicts[m] in ('holds', 'partial'):
                                    label = '+'.join(sorted(kinds))
                                    tally[('c944_pure', m, verdicts[m], label)] += 1
                                    key = ('c944_pure', m, verdicts[m], label)
                                    if key not in examples or size < examples[key][0]:
                                        examples[key] = (size, world, sorted(kinds))
                        if {'missing', 'released_live'} & kinds and verdicts['ne'] in ('holds', 'partial'):
                            tally[('c944_ne_not_red', verdicts['ne'], 'with_another_red' if (i318 or i54 or i315 or i317) else 'alone')] += 1
    print('name=q1q2 slots=%d worlds=%d' % (SLOTS, tally['worlds']))
    for k in sorted(SIDE):
        print('name=side %s=%d' % (k, SIDE[k]))
    for m in MODES:
        print('name=verdict mode=%s %s' % (m, ' '.join('%s=%d' % (v, tally[('verdict', m, v)]) for v in ('holds', 'partial', 'red'))))
    for k in sorted((k for k in tally if k[0] == 'healthy_but_not_holds'), key=str):
        print('name=healthy_but_not_holds', k[1:], tally[k])
    for k in sorted((k for k in tally if k[0] == 'only_span_wrong'), key=str):
        print('name=only_span_wrong mode=%s verdict=%s worlds=%d' % (k[1], k[2], tally[k]))
    for k in sorted((k for k in tally if k[0] == 'partial_ne'), key=str):
        print('name=partial_ne kinds=%s %s worlds=%d' % (k[1], k[2], tally[k]))
    print('name=partial_ne_i318_red', {k[1]: v for k, v in tally.items() if k[0] == 'partial_ne_i318_red'})
    for k in sorted((k for k in tally if k[0].startswith('partial_') and k[0] in ('partial_ne_but_gt_says', 'partial_gt_but_ne_says')), key=str):
        print('name=%s' % k[0], k[1:], tally[k])
    for k in sorted((k for k in tally if k[0] == 'triple'), key=str):
        print('name=triple ne=%s gt=%s none=%s worlds=%d' % (k[1], k[2], k[3], tally[k]))
    for k in sorted((k for k in tally if k[0] == 'c944_pure'), key=str):
        print('name=c944_pure mode=%s verdict=%s kinds=%s worlds=%d' % (k[1], k[2], k[3], tally[k]))
    for k in sorted((k for k in tally if k[0] == 'c944_ne_not_red'), key=str):
        print('name=c944_ne_not_red verdict=%s %s worlds=%d' % (k[1], k[2], tally[k]))
    for k in sorted(examples, key=str):
        print('example', k, examples[k])


if __name__ == '__main__':
    if len(sys.argv) > 1:
        SLOTS = int(sys.argv[1])
    main()
