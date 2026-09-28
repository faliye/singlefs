#!/usr/bin/env python3
"""A5 / A6 的最小模型（m3-prune-gpu-r3 云端攻方腿的原型，不入库）：录入与核对两个进程经 KV 交接，穷举交错与被杀。

KV 里每块三样：录入的数据（没有 / 半块 / 整块）、「已录入」标记、「已核对」标记（布尔，或记下核它的判法摘要）、结果。
录入进程按块号写；核对进程（一张或两张卡）按批取「已录入且未核」的块，算完写回；两个进程在任意两次 KV 写之间都能被杀一次、从头重起
（重起只看 KV；录入跳过已标「已录入」的块，核对重扫）。积压上限：已录入未核的块数到了上限，录入就等。
门禁最后一次读：每块要么给出结果，要么报「本次未跑」。一个终局里下面任一样发生就算错一次：
  missing_result   标了「已核对」、结果却没有（门禁把它当没有违例）
  wrong_input      结果是拿半块或没有的数据算的
  double_tally     结果按「追加」计进计数时，同一块被计了两次
  stale_after_change 判法改了（摘要换了）之后，旧结果仍被当成已核对
  stuck            还有块没核、两个进程都动不了（死等）或块被一张掉线的卡领着不放
用法：pipeline_model.py            逐个变体打一行 E7RESULT
"""
import itertools
import sys

BLOCKS = 2


def run_variant(name, record_order, check_order, flag_kind, tally, claims_persist, cards, batch, backlog_cap, flush_when_blocked, change_digest, card_drops=False):
    # KV：每块 (data, recorded, checked, result, tally_count, claimed_by)
    initial_kv = tuple(("none", False, None, None, 0, None) for _ in range(BLOCKS))
    digest = "D1"
    # 进程局部状态：录入 (下一块, 本块写到第几步)；核对每张卡 (手里的批, 批里下一块, 本块写到第几步, 算出的结果)
    initial = (initial_kv, (0, 0), tuple(((), 0, 0, None) for _ in range(cards)), 0, 0, 0)
    seen = set()
    terminal_errors = {"missing_result": 0, "wrong_input": 0, "double_tally": 0, "stale_after_change": 0, "stuck": 0}
    terminal_count = 0
    stack = [initial]
    while stack:
        state = stack.pop()
        if state in seen:
            continue
        seen.add(state)
        kv, recorder, checkers, recorder_kills, checker_kills, phase = state
        current_digest = "D2" if phase == 1 else "D1"
        successors = []

        def checked_now(entry):
            return entry[2] == (current_digest if flag_kind == "digest" else True)

        pending = [block for block in range(BLOCKS) if kv[block][1] and not checked_now(kv[block])]
        backlog = len(pending)
        # 录入进程的下一步
        block, step = recorder
        if block < BLOCKS:
            if kv[block][1] and step == 0:
                successors.append((kv, (block + 1, 0), checkers, recorder_kills, checker_kills, phase))
            elif backlog < backlog_cap or step > 0:
                writes = {"atomic": [("data+flag",)], "flag_first": [("flag",), ("data",)], "data_first": [("data",), ("flag",)]}[record_order]
                action = writes[step][0]
                entry = list(kv[block])
                if "data" in action:
                    entry[0] = "full"
                if "flag" in action:
                    entry[1] = True
                new_kv = kv[:block] + (tuple(entry),) + kv[block + 1:]
                next_recorder = (block + 1, 0) if step + 1 == len(writes) else (block, step + 1)
                successors.append((new_kv, next_recorder, checkers, recorder_kills, checker_kills, phase))
            if recorder_kills == 0 and step > 0:
                successors.append((kv, (0, 0), checkers, 1, checker_kills, phase))
        recorder_done = block >= BLOCKS
        # 核对进程（每张卡）的下一步
        for card, card_state in enumerate(checkers):
            if card_state is None:
                continue
            held, position, step, computed = card_state
            if not held:
                free = [b for b in pending if kv[b][5] is None or kv[b][5] == card]
                if len(free) >= batch or (free and (recorder_done or (flush_when_blocked and backlog >= backlog_cap))):
                    chosen = tuple(free[:batch])
                    new_kv = kv
                    if claims_persist:
                        new_kv = tuple((e[0], e[1], e[2], e[3], e[4], card) if i in chosen else e for i, e in enumerate(kv))
                    new_checkers = checkers[:card] + ((chosen, 0, 0, None),) + checkers[card + 1:]
                    successors.append((new_kv, recorder, new_checkers, recorder_kills, checker_kills, phase))
                continue
            target = held[position]
            if computed is None:
                result = "right" if kv[target][0] == "full" else "from_" + kv[target][0]
                new_checkers = checkers[:card] + ((held, position, 0, result),) + checkers[card + 1:]
                successors.append((kv, recorder, new_checkers, recorder_kills, checker_kills, phase))
            else:
                writes = {"atomic": ["result+flag"], "flag_first": ["flag", "result"], "result_first": ["result", "flag"]}[check_order]
                action = writes[step]
                entry = list(kv[target])
                if "result" in action:
                    entry[3] = computed + "@" + current_digest
                    entry[4] += 1 if tally == "append" else 0
                if "flag" in action:
                    entry[2] = current_digest if flag_kind == "digest" else True
                    entry[5] = None
                new_kv = kv[:target] + (tuple(entry),) + kv[target + 1:]
                if step + 1 < len(writes):
                    held_state = (held, position, step + 1, computed)
                elif position + 1 < len(held):
                    held_state = (held, position + 1, 0, None)
                else:
                    held_state = ((), 0, 0, None)
                new_checkers = checkers[:card] + (held_state,) + checkers[card + 1:]
                successors.append((new_kv, recorder, new_checkers, recorder_kills, checker_kills, phase))
            if checker_kills == 0:
                new_checkers = checkers[:card] + ((None if card_drops else ((), 0, 0, None)),) + checkers[card + 1:]
                successors.append((kv, recorder, new_checkers, recorder_kills, 1, phase))
        if change_digest and phase == 0 and recorder_done and not any(card_state and card_state[0] for card_state in checkers) and not pending:
            successors.append((kv, recorder, checkers, recorder_kills, checker_kills, 1))
        if successors:
            stack.extend(successors)
            continue
        terminal_count += 1
        errors = set()
        for entry in kv:
            data, recorded, checked, result, count, claimed = entry
            if checked_now(entry) and result is None:
                errors.add("missing_result")
            if result is not None and not result.startswith("right"):
                errors.add("wrong_input")
            if count > 1:
                errors.add("double_tally")
            if checked_now(entry) and result is not None and not result.endswith("@" + current_digest):
                errors.add("stale_after_change")
            if not checked_now(entry):
                errors.add("stuck")
        for error in errors:
            terminal_errors[error] += 1
    total = sum(terminal_errors.values())
    detail = " ".join(f"{key}={value}" for key, value in terminal_errors.items())
    print(f"E7RESULT name=r3_pipeline variant={name} reachable_states={len(seen)} terminal_states={terminal_count} {detail} must_be_nonzero={total if name not in ('strongest', 'claims_in_memory_and_a_card_drops') else '-'}")
    return total


VARIANTS = [
    ("strongest", dict(record_order="atomic", check_order="atomic", flag_kind="digest", tally="keyed", claims_persist=False, cards=2, batch=1, backlog_cap=1, flush_when_blocked=True, change_digest=True)),
    ("record_flag_before_data", dict(record_order="flag_first", check_order="atomic", flag_kind="digest", tally="keyed", claims_persist=False, cards=1, batch=1, backlog_cap=2, flush_when_blocked=True, change_digest=False)),
    ("checked_flag_before_result", dict(record_order="atomic", check_order="flag_first", flag_kind="digest", tally="keyed", claims_persist=False, cards=1, batch=1, backlog_cap=2, flush_when_blocked=True, change_digest=False)),
    ("result_then_flag_with_appended_tally", dict(record_order="atomic", check_order="result_first", flag_kind="digest", tally="append", claims_persist=False, cards=1, batch=1, backlog_cap=2, flush_when_blocked=True, change_digest=False)),
    ("checked_flag_is_a_boolean", dict(record_order="atomic", check_order="atomic", flag_kind="bool", tally="keyed", claims_persist=False, cards=1, batch=1, backlog_cap=2, flush_when_blocked=True, change_digest=True)),
    ("batch_larger_than_backlog_cap_without_flush", dict(record_order="atomic", check_order="atomic", flag_kind="digest", tally="keyed", claims_persist=False, cards=1, batch=2, backlog_cap=1, flush_when_blocked=False, change_digest=False)),
    ("claims_persisted_without_lease_and_a_card_drops", dict(record_order="atomic", check_order="atomic", flag_kind="digest", tally="keyed", claims_persist=True, cards=2, batch=1, backlog_cap=2, flush_when_blocked=True, change_digest=False, card_drops=True)),
    ("claims_in_memory_and_a_card_drops", dict(record_order="atomic", check_order="atomic", flag_kind="digest", tally="keyed", claims_persist=False, cards=2, batch=1, backlog_cap=2, flush_when_blocked=True, change_digest=False, card_drops=True)),
]

if __name__ == "__main__":
    failures = 0
    for name, options in VARIANTS:
        total = run_variant(name, **options)
        failures += int((name in ("strongest", "claims_in_memory_and_a_card_drops")) != (total == 0))
    sys.exit(1 if failures else 0)
