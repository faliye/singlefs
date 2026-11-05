"""Q3 窄方向的小世界穷举（槽级，推的）：一块盘 N 个槽，单元（码 2 一槽 / 码 1 两槽，头都用得了）与全零槽的每种排法、
每个单元引用或不引用、每个槽上记录不记 / 记一槽 / 记两槽（只取「记录不比起点单元宽」：起点有单元头时跨度 ≤ 那一类）、
记账已分配 0..N，全是未释放记录、只有一个候选版本（遍历 = 最新根走读，defer 行 0）。
数今天的 I-3.1 报部分没判的盘面里，有几张另有「未释放记录罩着、没人引用、起点头用不了」的槽（泄漏，I-3.15 / I-3.18 / I-3.10 都没有对象），
其中又有几张 I-3.11、I-5.4 成立、I-3.16 判得到且 runs 照记录口径写也对得上（别的不变量一条都不红，只剩 I-3.18 红）。
用法：python3 i31_sweep.py N
"""
import itertools
import sys
from i31_rules import Record, i31_verdict, set_to_unit_classes

N = int(sys.argv[1]) if len(sys.argv) > 1 else 5


def layouts(n):
    """[(起点, 槽数)]：单元排法，空着的槽全零。"""
    if n <= 0:
        yield []
        return
    for rest in layouts(n - 1):
        yield [(None, 1)] + [(None if s is None else s + 1, c) for s, c in rest]
    for c in (1, 2):
        if c <= n:
            for rest in layouts(n - c):
                yield [(0, c)] + [(None if s is None else s + c, k) for s, k in rest]


def runs_of(free, n):
    runs, prev = 0, False
    for s in range(n):
        cur = s in free
        runs += cur and not prev
        prev = cur
    return runs


counts = dict(worlds=0, partial=0, partial_with_leak=0, partial_with_leak_nothing_else_red=0,
              partial_dead_record_only=0, partial_clean=0,
              fix_partial_with_leak=0, fix_partial_dead_record_only=0, fix_partial_clean=0)
for layout in layouts(N):
    units = [(s, c) for s, c in layout if s is not None]
    headers = {s: c for s, c in units}
    for referenced in itertools.product((True, False), repeat=len(units)):
        walked = sorted(x for (s, c), ref in zip(units, referenced) if ref for x in range(s, s + c))
        wset = {(0, x) for x in walked}
        choices = []
        for slot in range(N):
            opts = [0, 1, 2] if slot not in headers else list(range(0, headers[slot] + 1))
            choices.append([o for o in opts if slot + o <= N])
        for spans in itertools.product(*choices):
            records = [Record(0, s, sp, 1, False) for s, sp in enumerate(spans) if sp]
            covered = [x for r in records for x in range(r.slot, r.slot + r.span)]
            disjoint = len(covered) == len(set(covered))
            leak = {x for r in records if r.slot not in headers for x in range(r.slot, r.slot + r.span)
                    if x not in walked}
            # 起点头用得了、罩着没人引用的槽：I-3.15 判它「已死」（有对象、红）
            dead = {x for r in records if r.slot in headers for x in range(r.slot, r.slot + r.span)
                    if x not in walked}
            for allocated in range(N + 1):
                counts["worlds"] += 1
                cls = lambda r: headers.get(r.slot)
                v = i31_verdict(0, records, wset, 0, 5, 5, allocated, cls)
                if v != "partial":
                    continue
                counts["partial"] += 1
                reset = set_to_unit_classes(records, cls)
                e_unit = {x for r in reset for x in range(r.slot, r.slot + r.span)}
                fixed = "violated" if not e_unit <= set(walked) else "partial"
                if leak:
                    counts["partial_with_leak"] += 1
                    counts["fix_partial_with_leak"] += fixed == "partial"
                    i311 = allocated == len(walked)
                    i316_runs = runs_of(set(range(N)) - set(walked), N) == runs_of(set(range(N)) - set(covered), N)
                    i315_red = any(r.slot in headers and r.slot not in walked for r in records)
                    if i311 and disjoint and i316_runs and not i315_red:
                        counts["partial_with_leak_nothing_else_red"] += 1
                elif dead:
                    counts["partial_dead_record_only"] += 1
                    counts["fix_partial_dead_record_only"] += fixed == "partial"
                else:
                    counts["partial_clean"] += 1
                    counts["fix_partial_clean"] += fixed == "partial"
print("name=i31_narrow_sweep N=%d " % N + " ".join(f"{k}={v}" for k, v in counts.items()))
