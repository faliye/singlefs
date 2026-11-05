"""Q3（C937）：I-3.1 取宽（记录自述 / 改记成单元那一类的槽数，两份界各取宽的那一边）在「记录比单元窄」「记录比单元宽」两个方向上
放过的盘面里，有没有真病不被 I-3.18 / I-5.4 的红说明盖住。推的小世界，不是入库装置，不跑 crates。

移植对象（crates/singlefs-checker/src/walk.rs，开工快照里的行号）：
  allocated_statistic_bounds_of_a_device（10270 起）：四个界
  allocated_statistic_lies_within_its_bounds（10337 起）：按记录自述的 RCW 三判
  records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes（10360 起）：改记
  allocated_statistic_lies_within_the_bounds_of_either_span_reading（10391 起）：两份界各取宽的那一边
窄向变体（narrow_only=True）：只改「那一类槽数 > 记录跨度」的记录（10372 的 `!=` 换成 `>`）。
世界：盘 0 一块盘、槽 0..SLOTS-1；真占用 = 若干不相交的单元（类 1 一槽，类 2 两槽），遍历走到的槽 = 单元占的槽；
  每个单元一条未释放记录，跨度取 {类-1, 类, 类+1} 或没有记录；可加一条游离记录（一槽、起点槽上没有单元头）；记账的已分配 A 取 0..SLOTS 槽；
  回收门槛 θ_high = t = 5，未释放记录的分配代 1（同 walk.rs 单测 span_reading_*）。
判：
  I-3.18 红 = 有起点槽上单元头用得了的未释放记录，跨度 != 那一类的槽数；
  I-5.4  红 = 同一块盘上任意两条记录罩住的槽相交；
  I-3.1  = 按记录自述三判成立 -> holds；不成立而改过记录且两份各取宽成立 -> partial（点名 I-3.18）；其余 red。
真值：A_true = 真占用的槽数，A_rec = 记录并集的槽数（几条记录罩同一槽只算一次）。
"""
import itertools, sys
from collections import Counter

SLOTS = 9
THETA = 5


def counted_at(records, theta):
    s = set()
    for (slot, span, gen, rel) in records:
        if (not rel) or gen > theta:
            s.update(range(slot, slot + span))
    return s


def bounds(records, walked):
    c_check = counted_at(records, THETA)
    c_root = counted_at(records, THETA)
    still_deferred = set()
    for (slot, span, gen, rel) in records:
        if rel and gen > THETA:
            still_deferred.update(x for x in range(slot, slot + span) if x not in walked)
    return dict(L=len(c_check), U1=len(c_root), WD=len(walked) + len(still_deferred),
                outside=sorted(x for x in walked if x not in c_check))


def lies_within(A, b):
    return A >= b['L'] and not b['outside'] and A <= min(b['U1'], b['WD'])


def set_to_classes(records, classes, narrow_only):
    changed = False
    out = []
    for rec in records:
        slot, span, gen, rel = rec
        if rel:
            out.append(rec)
            continue
        c = classes.get(slot)
        if c is not None and c != span and (not narrow_only or c > span):
            changed = True
            out.append((slot, c, gen, rel))
        else:
            out.append(rec)
    return out if changed else None


def either_reading(A, b_rec, b_cls):
    lower = A >= min(b_rec['L'], b_cls['L'])
    walked_ok = all(x not in b_cls['outside'] for x in b_rec['outside'])
    upper = A <= min(max(b_rec['U1'], b_cls['U1']), b_rec['WD'])
    return lower and walked_ok and upper


def verdict31(A, records, walked, classes, narrow_only):
    b = bounds(records, walked)
    if lies_within(A, b):
        return 'holds'
    cls = set_to_classes(records, classes, narrow_only)
    if cls is not None and either_reading(A, b, bounds(cls, walked)):
        return 'partial'
    return 'red'


def i318_red(records, classes):
    return any((not rel) and classes.get(slot) is not None and classes[slot] != span for (slot, span, gen, rel) in records)


def i54_red(records):
    rs = sorted((slot, span) for (slot, span, gen, rel) in records)
    return any(a[0] + a[1] > b[0] for a, b in zip(rs, rs[1:]))


def layouts():
    """不相交的单元摆法：[(起点, 类)]，至多三个。"""
    out = []

    def rec(pos, cur):
        out.append(list(cur))
        if len(cur) == 3:
            return
        for start in range(pos, SLOTS):
            for c in (1, 2):
                if start + c <= SLOTS:
                    cur.append((start, c))
                    rec(start + c, cur)
                    cur.pop()
    rec(0, [])
    return [l for l in out if l]


def span_choices(c):
    return [x for x in (c - 1, c, c + 1) if x >= 1] + [None]


def worlds():
    for lay in layouts():
        classes = {s: c for (s, c) in lay}
        walked = set()
        for (s, c) in lay:
            walked.update(range(s, s + c))
        free = [x for x in range(SLOTS) if x not in walked]
        stray_options = [None] + [(x, 1, 1, False) for x in free]
        for spans in itertools.product(*[span_choices(c) for (s, c) in lay]):
            recs = [(s, sp, 1, False) for ((s, c), sp) in zip(lay, spans) if sp is not None]
            missing = sum(1 for sp in spans if sp is None)
            for stray in stray_options:
                records = recs + ([stray] if stray else [])
                for A in range(0, SLOTS + 1):
                    yield lay, classes, walked, records, missing, stray is not None, A


def main():
    tally = Counter()
    examples = {}
    for lay, classes, walked, records, missing, stray, A in worlds():
        both = verdict31(A, records, walked, classes, False)
        narrow = verdict31(A, records, walked, classes, True)
        r318 = i318_red(records, classes)
        r54 = i54_red(records)
        a_true = len(walked)
        a_rec = len(counted_at(records, THETA))
        tally[('worlds',)] += 1
        b_rec = bounds(records, walked)
        if lies_within(A, b_rec) and (missing or stray):
            tally[('holds_as_recorded_with_a_missing_or_stray_record', 'missing=%d' % missing, 'stray=%d' % int(stray))] += 1
        tally[('both', both)] += 1
        tally[('narrow_only', narrow)] += 1
        if both != 'holds':
            # I-3.1 没绿：取宽把红换成部分没判的，各算一类
            pass
        if both == 'partial':
            if missing or stray:
                cause = 'other_record_defect(missing=%d,stray=%d)' % (missing, int(stray))
            elif A in (a_true, a_rec):
                cause = 'stat_follows_units_or_records'
            else:
                cause = 'stat_strictly_between'
            direction = []
            for (s, c), rec_ in zip(lay, [None] * len(lay)):
                pass
            narrow_n = sum(1 for (s, sp, g, r_) in records if s in classes and sp < classes[s])
            wide_n = sum(1 for (s, sp, g, r_) in records if s in classes and sp > classes[s])
            kind = ('narrow' if narrow_n else '') + ('+' if narrow_n and wide_n else '') + ('wide' if wide_n else '')
            if kind == 'wide' and missing:
                tally[('wide_missing_A_minus_A_true', A - a_true, 'narrow_only=%s' % narrow)] += 1
            tally[('both_partial', kind, cause, 'i318=%d' % r318, 'i54=%d' % r54, 'narrow_only=%s' % narrow)] += 1
            key = (kind, cause, 'narrow_only=%s' % narrow)
            size = (len(lay), len(records), SLOTS)
            if key not in examples or size < examples[key][0]:
                examples[key] = (size, lay, records, A, a_true, a_rec, r318, r54)
    print('worlds', tally[('worlds',)])
    for v in ('holds', 'partial', 'red'):
        print('I-3.1 both-directions  ', v, tally[('both', v)])
    for v in ('holds', 'partial', 'red'):
        print('I-3.1 narrow-direction ', v, tally[('narrow_only', v)])
    print('--- both-directions partial worlds by (direction of the misfit records, cause, I-3.18 red, I-5.4 red, what narrow-only says)')
    for k in sorted(tally):
        if k[0] == 'both_partial':
            print(k[1:], tally[k])
    print('--- 取宽之前就成立的盘面里带「单元没有记录」或「游离记录」的数（取宽不是它们放过的原因）')
    for k in sorted(tally):
        if k[0] == 'holds_as_recorded_with_a_missing_or_stray_record':
            print(k[1:], tally[k])
    print('--- 宽向 + 有单元没记录 的取宽后部分没判盘面：A - A_true 的分布（以及只放窄一个方向时判什么）')
    for k in sorted(tally):
        if k[0] == 'wide_missing_A_minus_A_true':
            print(k[1:], tally[k])
    print('--- smallest example per class: (units, records, A, A_true, A_rec, I-3.18 red, I-5.4 red); units=(start,class), records=(start,span,gen,released)')
    for key in sorted(examples):
        size, lay, records, A, a_true, a_rec, r318, r54 = examples[key]
        print(key, '| units', lay, '| records', records, '| A', A, 'A_true', a_true, 'A_rec', a_rec, '| I-3.18 red', r318, 'I-5.4 red', r54)


def minimal_mirrors():
    """两张最小镜像：一个两槽单元（类 2，起点槽 0），记录跨 1 槽（窄）/ 跨 3 槽（宽），记账 A 取 0..3 槽；其余同上。"""
    print('=== 两张最小镜像（单元 = 起点槽 0、类 2；槽 1 / 槽 2 盘上没有别的单元与记录）')
    classes = {0: 2}
    walked = {0, 1}
    for name, span in (('窄：记录跨 1 槽', 1), ('宽：记录跨 3 槽', 3)):
        records = [(0, span, 1, False)]
        for A in range(0, 4):
            b = bounds(records, walked)
            rec_holds = lies_within(A, b)
            both = verdict31(A, records, walked, classes, False)
            narrow = verdict31(A, records, walked, classes, True)
            print('%s | 记账 A = %d 槽 | 按记录自述三判 %s（下沿 L=%d, 遍历走到而不在记录里的槽 %s, 上沿 min(U1=%d, WD=%d)）| 取宽（两向）-> I-3.1 %s | 只放窄 -> I-3.1 %s | I-3.18 %s | I-5.4 %s'
                  % (name, A, '成立' if rec_holds else '不成立', b['L'], b['outside'], b['U1'], b['WD'], both, narrow,
                     '红' if i318_red(records, classes) else '绿', '红' if i54_red(records) else '绿'))


def named_scenarios():
    """被问到的几种摆法，各一条：(名字, 单元[(起点, 类)], 记录[(起点, 跨度)], 记账 A)。输出每条上三种判法与另两条不变量。"""
    print('=== 被问到的摆法')
    cases = [
        ('窄 记账照单元 A=2', [(0, 2)], [(0, 1)], 2),
        ('窄 记账照记录 A=1', [(0, 2)], [(0, 1)], 1),
        ('宽 记账照单元 A=2', [(0, 2)], [(0, 3)], 2),
        ('宽 记账照记录 A=3', [(0, 2)], [(0, 3)], 3),
        ('宽 邻居一槽单元没有记录 A=1', [(0, 1), (1, 1)], [(0, 2)], 1),
        ('宽 邻居一槽单元没有记录 A=2（记账照单元）', [(0, 1), (1, 1)], [(0, 2)], 2),
        ('宽 多罩的槽被邻居的记录合法罩住 A=2', [(0, 1), (1, 1)], [(0, 2), (1, 1)], 2),
        ('宽 多罩的槽被邻居的记录合法罩住 A=3（记账把重叠数两遍）', [(0, 1), (1, 1)], [(0, 2), (1, 1)], 3),
        ('窄 少罩的槽被一条游离记录罩住 A=2', [(0, 2)], [(0, 1), (1, 1)], 2),
        ('窄 少罩的槽被一条游离记录罩住 A=3', [(0, 2)], [(0, 1), (1, 1)], 3),
        ('窄两条 记账介于记录与单元之间 A=3', [(0, 2), (2, 2)], [(0, 1), (2, 1)], 3),
        ('窄 加一条别处的游离记录 A=2', [(0, 2)], [(0, 1), (3, 1)], 2),
    ]
    for name, lay, spans, A in cases:
        classes = {s: c for (s, c) in lay}
        walked = set()
        for (s, c) in lay:
            walked.update(range(s, s + c))
        records = [(s, sp, 1, False) for (s, sp) in spans]
        b = bounds(records, walked)
        print('%s | 单元 %s 记录 %s A=%d | 自述三判 %s | 两向取宽 -> %s | 只放窄 -> %s | I-3.18 %s | I-5.4 %s | 遍历走到而不在记录里的槽 %s'
              % (name, lay, spans, A, '成立' if lies_within(A, b) else '不成立', verdict31(A, records, walked, classes, False),
                 verdict31(A, records, walked, classes, True), '红' if i318_red(records, classes) else '绿',
                 '红' if i54_red(records) else '绿', b['outside']))


def walk_rs_unit_tests():
    """walk.rs 单测（25653 / 25673 / 25687 / 25698 / 25712）的同一批输入过移植：期望是 (按记录自述三判成立吗, 两份各取宽成立吗；没有可改的记录时 None)。"""
    print('=== walk.rs 单测同一批输入')
    classes = {10: 2, 12: 1, 30: 2}
    cases = [
        ('25653 窄 记账照两槽记 A=3', [(10, 1), (12, 1)], {10, 11, 12}, 3, (False, True)),
        ('25673 窄 记账也只记一槽 A=2', [(10, 1), (12, 1)], {10, 11, 12}, 2, (False, True)),
        ('25687 宽 A=2', [(30, 3)], {30, 31}, 2, (False, True)),
        ('25698 窄 另有遍历走到而没有记录的槽 20 A=3', [(10, 1), (12, 1)], {10, 11, 12, 20}, 3, (False, False)),
        ('25712 单元的记录整条不见 A=3', [(12, 1)], {10, 11, 12}, 3, (False, None)),
    ]
    for name, spans, walked, A, expected in cases:
        records = [(s, sp, 1, False) for (s, sp) in spans]
        b = bounds(records, walked)
        cls = set_to_classes(records, classes, False)
        got = (lies_within(A, b), None if cls is None else either_reading(A, b, bounds(cls, walked)))
        print('%s | 移植得 %s | 单测期望 %s | %s' % (name, got, expected, '对上' if got == expected else '对不上'))


if __name__ == '__main__':
    main()
    minimal_mirrors()
    named_scenarios()
    walk_rs_unit_tests()
