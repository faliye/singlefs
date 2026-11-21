#!/usr/bin/env python3
"""文件区间交集长度证明的小模型（只用来独立核，不与 crates 共用代码）。

对着 crates/singlefs-replay-prefix/src/file_range_proofs.rs 的规格与两条组合引理，在小整数格上穷举：
  1. 规格 range_intersection_length 等于「两个半开区间逐字节交集的元素个数」（含空、倒置区间）；
  2. 两条引理在登记的前提下对格内每个输入成立；
  3. 前提去掉之后有反例（必红行「needs-valid-boundary」红的就是前提）；并算出对「另一端任意」成立的最弱前提；
  4. bytes_in_unit、checked_bytes_in_indexed_unit 的执行模型满足新旧后置条件，Some 与 None 两支都有输入；
     必红行里把 `length == …` 改成 `+ 1` 之后，这一条在每个 Some 输入上都不成立；
  5. 见证函数里 11 条等式与六处引理调用的前提。
每个量都打一行 name=<名> … ；有任何一格不成立退 1。
"""
import itertools
import sys

failures = []


def say(name, **fields):
    print("name=" + name + "".join(f" {key}={value}" for key, value in fields.items()))


def expect(condition, what):
    if not condition:
        failures.append(what)


def intersection_length(ws, we, us, ue):
    start = ws if ws > us else us
    end = we if we < ue else ue
    return end - start if start < end else 0


def bytes_in_both(ws, we, us, ue, lo, hi):
    return [b for b in range(lo, hi) if ws <= b < we and us <= b < ue]


# ---- 1 规格等于逐字节交集 ----
LO, HI = -4, 9
GRID = range(LO, HI)
checked = 0
for ws, we, us, ue in itertools.product(GRID, repeat=4):
    expect(intersection_length(ws, we, us, ue) == len(bytes_in_both(ws, we, us, ue, LO - 1, HI + 1)),
           f"规格与逐字节交集不等 {(ws, we, us, ue)}")
    checked += 1
say("spec_equals_bytewise_intersection", inputs=checked, grid=f"[{LO},{HI})", bad=len(failures))

# ---- 2 引理在登记的前提下成立；3 前提去掉的反例与最弱前提 ----
unit_lemma_ok = unit_lemma_total = 0
unit_lemma_bad_without_premise = 0
first_unit_counterexample = None
for ws, we, us, b, ue in itertools.product(GRID, repeat=5):
    holds = intersection_length(ws, we, us, ue) == intersection_length(ws, we, us, b) + intersection_length(ws, we, b, ue)
    if us <= b <= ue:
        unit_lemma_total += 1
        unit_lemma_ok += holds
        expect(holds, f"单元分界引理在前提内不成立 {(ws, we, us, b, ue)}")
    elif not holds:
        unit_lemma_bad_without_premise += 1
        if first_unit_counterexample is None:
            first_unit_counterexample = (ws, we, us, b, ue)
say("unit_split_lemma_under_premise", inputs=unit_lemma_total, holds=unit_lemma_ok)
say("unit_split_lemma_premise_dropped", counterexamples=unit_lemma_bad_without_premise, first=first_unit_counterexample)
expect(unit_lemma_bad_without_premise > 0, "单元分界引理去掉前提没有反例")

write_lemma_ok = write_lemma_total = 0
write_lemma_bad_without_premise = 0
first_write_counterexample = None
for ws, split, we, us, ue in itertools.product(GRID, repeat=5):
    holds = intersection_length(ws, we, us, ue) == intersection_length(ws, split, us, ue) + intersection_length(split, we, us, ue)
    if ws <= split <= we:
        write_lemma_total += 1
        write_lemma_ok += holds
        expect(holds, f"请求分界引理在前提内不成立 {(ws, split, we, us, ue)}")
    elif not holds:
        write_lemma_bad_without_premise += 1
        if first_write_counterexample is None:
            first_write_counterexample = (ws, split, we, us, ue)
say("write_split_lemma_under_premise", inputs=write_lemma_total, holds=write_lemma_ok)
say("write_split_lemma_premise_dropped", counterexamples=write_lemma_bad_without_premise, first=first_write_counterexample)
expect(write_lemma_bad_without_premise > 0, "请求分界引理去掉前提没有反例")

# 最弱前提：对「另一端任意」的格都成立的 (us, b, ue) 是哪些
weaker_unit = []
for us, b, ue in itertools.product(GRID, repeat=3):
    always = all(
        intersection_length(ws, we, us, ue) == intersection_length(ws, we, us, b) + intersection_length(ws, we, b, ue)
        for ws, we in itertools.product(GRID, repeat=2)
    )
    registered = us <= b <= ue
    if always != registered:
        weaker_unit.append((us, b, ue, always, registered))
only_inverted_unit = all(ue <= b <= us for us, b, ue, always, registered in weaker_unit if always)
say("unit_split_weakest_premise", triples_registered_premise_misses=len(weaker_unit),
    all_misses_are_inverted_unit_ue_le_b_le_us=only_inverted_unit,
    registered_premise_always_sufficient=all(always for us, b, ue, always, registered in weaker_unit if registered) )
weaker_write = []
for ws, s, we in itertools.product(GRID, repeat=3):
    always = all(
        intersection_length(ws, we, us, ue) == intersection_length(ws, s, us, ue) + intersection_length(s, we, us, ue)
        for us, ue in itertools.product(GRID, repeat=2)
    )
    registered = ws <= s <= we
    if always != registered:
        weaker_write.append((ws, s, we, always, registered))
only_inverted_write = all(we <= s <= ws for ws, s, we, always, registered in weaker_write if always)
say("write_split_weakest_premise", triples_registered_premise_misses=len(weaker_write),
    all_misses_are_inverted_write_we_le_s_le_ws=only_inverted_write)
expect(only_inverted_unit and only_inverted_write, "最弱前提比登记的前提多出的不只是倒置区间")

# ---- 4 执行模型与后置条件 ----
def bytes_in_unit(ws, we, us, ue):
    frm = ws if ws > us else us
    until = we if we < ue else ue
    if frm < until:
        return (frm - us, frm - ws, until - frm)
    return None


def post_bytes_in_unit(answer, ws, we, us, ue, lo, hi, with_new_conjunct=True, plus=0):
    both = set(bytes_in_both(ws, we, us, ue, lo, hi))
    length_spec = intersection_length(ws, we, us, ue)
    if answer is None:
        new = (length_spec == 0) if with_new_conjunct else True
        return new and not both
    in_unit, in_write, length = answer
    new = (length == length_spec + plus) if with_new_conjunct else True
    return (length > 0 and new and us + in_unit == ws + in_write
            and in_unit + length <= ue - us and in_write + length <= we - ws
            and both == set(range(us + in_unit, us + in_unit + length)))


N = 9
some = none = bad = bad_old_clauses_only = 0
none_conjunct_implied_by_forall = True
some_plus_one_fails_everywhere = True
for ws, we, us, ue in itertools.product(range(0, N), repeat=4):
    answer = bytes_in_unit(ws, we, us, ue)
    ok = post_bytes_in_unit(answer, ws, we, us, ue, -1, N + 2)
    bad += not ok
    if answer is None:
        none += 1
        both = bytes_in_both(ws, we, us, ue, -1, N + 2)
        none_conjunct_implied_by_forall &= (not both) == (intersection_length(ws, we, us, ue) == 0)
    else:
        some += 1
        some_plus_one_fails_everywhere &= not post_bytes_in_unit(answer, ws, we, us, ue, -1, N + 2, plus=1)
say("bytes_in_unit_postcondition", inputs=some + none, some_branch=some, none_branch=none, violations=bad)
say("bytes_in_unit_new_conjunct_is_not_vacuous", plus_one_fails_on_every_some_input=some_plus_one_fails_everywhere,
    none_conjunct_equivalent_to_old_forall=none_conjunct_implied_by_forall)
expect(bad == 0, "bytes_in_unit 的执行模型不满足后置条件")
expect(some > 0 and none > 0, "bytes_in_unit 有一支没有输入")
expect(some_plus_one_fails_everywhere, "改成 +1 之后有 Some 输入照样满足")

MAX = 15


def checked_bytes_in_indexed_unit(ws, we, unit_index, capacity, bytes_in_unit_function=bytes_in_unit):
    start = unit_index * capacity
    if start > MAX:
        return None
    end = start + capacity
    if end > MAX:
        return None
    return ("some", bytes_in_unit_function(ws, we, start, end))


def post_indexed(answer, ws, we, unit_index, capacity, plus=0, drop_none_conjunct_check=False):
    end = unit_index * capacity + capacity
    if answer is None:
        return end > MAX
    if end > MAX:
        return False
    inner = answer[1]
    both = set(bytes_in_both(ws, we, unit_index * capacity, end, -1, MAX + capacity + 2))
    spec = intersection_length(ws, we, unit_index * capacity, end)
    if inner is None:
        return spec == 0 and not both
    in_unit, in_write, length = inner
    return (length > 0 and length == spec + plus and unit_index * capacity + in_unit == ws + in_write
            and in_unit + length <= capacity and in_write + length <= we - ws
            and both == set(range(unit_index * capacity + in_unit, unit_index * capacity + in_unit + length)))


outer_none = inner_none = inner_some = indexed_bad = 0
indexed_plus_fails_everywhere = True
dropped_bytes_violations = dropped_bytes_violations_new_conjunct = 0
for ws, we, ui, cap in itertools.product(range(0, MAX + 1), range(0, MAX + 1), range(0, 6), range(0, 7)):
    answer = checked_bytes_in_indexed_unit(ws, we, ui, cap)
    indexed_bad += not post_indexed(answer, ws, we, ui, cap)
    if answer is None:
        outer_none += 1
        continue
    if answer[1] is None:
        inner_none += 1
    else:
        inner_some += 1
        indexed_plus_fails_everywhere &= not post_indexed(answer, ws, we, ui, cap, plus=1)
    # 必红行 indexed-intersection-does-not-drop-written-bytes：把 bytes_in_unit 的结果换成 None
    dropped = checked_bytes_in_indexed_unit(ws, we, ui, cap, bytes_in_unit_function=lambda *_: None)
    start, end = ui * cap, ui * cap + cap
    if intersection_length(ws, we, start, end) > 0:
        dropped_bytes_violations += 1
        # 旧后置条件的 None 支（forall 没有共同字节）与新加的「长度 == 0」各自不成立
        dropped_bytes_violations_new_conjunct += intersection_length(ws, we, start, end) != 0
say("checked_bytes_in_indexed_unit_postcondition", outer_none=outer_none, inner_none=inner_none, inner_some=inner_some, violations=indexed_bad)
say("indexed_row_plus_one_fails_on_every_some_input", ok=indexed_plus_fails_everywhere)
say("indexed_row_drop_written_bytes", inputs_where_the_mutation_breaks_the_old_forall=dropped_bytes_violations,
    of_which_the_new_length_zero_conjunct_breaks_too=dropped_bytes_violations_new_conjunct)
expect(indexed_bad == 0, "checked_bytes_in_indexed_unit 的执行模型不满足后置条件")
expect(outer_none > 0 and inner_none > 0 and inner_some > 0, "checked_bytes_in_indexed_unit 有一支没有输入")
expect(indexed_plus_fails_everywhere, "indexed 行改成 +1 之后有 Some 输入照样满足")
expect(dropped_bytes_violations > 0 and dropped_bytes_violations == dropped_bytes_violations_new_conjunct,
       "把 bytes_in_unit 的结果换成 None：旧条件与新条件不是同一批输入上一起不成立")

# ---- 5 见证 ----
U = 2**64 - 1
facts = [
    ((3, 7, 0, 8), 4), ((3, 7, 0, 4), 1), ((3, 7, 4, 8), 3), ((1, 9, 3, 7), 4), ((1, 5, 3, 7), 2), ((5, 9, 3, 7), 2),
    ((3, 3, 0, 8), 0), ((7, 3, 0, 8), 0), ((1, 9, 5, 5), 0), ((1, 9, 7, 3), 0), ((U - 1, U, U - 1, U), 1),
]
witness_bad = [args for args, expected in facts if intersection_length(*args) != expected]
say("witness_facts", count=len(facts), bad=len(witness_bad))
expect(not witness_bad, f"见证里的等式不成立 {witness_bad}")
unit_calls = [(3, 7, 0, 4, 8), (3, 7, 0, 0, 8), (3, 7, 0, 8, 8)]
write_calls = [(1, 5, 9, 3, 7), (1, 1, 9, 3, 7), (1, 9, 9, 3, 7)]
calls_ok = all(us <= b <= ue for ws, we, us, b, ue in unit_calls) and all(ws <= s <= we for ws, s, we, us, ue in write_calls)
say("witness_lemma_call_premises", unit_calls=len(unit_calls), write_calls=len(write_calls), all_premises_hold=calls_ok)
expect(calls_ok, "见证里某次引理调用不满足前提")
# 见证里的 11 条等式写的是 range_intersection_length(…)，引理调用对它们没有贡献：调用结论与这 11 条逐条无关
say("witness_lemma_calls_needed_by_the_eleven_facts", needed="no", note="十一条等式都是闭式求值；六处调用只证引理前提可满足")

say("model_result", failures=len(failures))
for failure in failures:
    print("✗ " + failure)
sys.exit(1 if failures else 0)
