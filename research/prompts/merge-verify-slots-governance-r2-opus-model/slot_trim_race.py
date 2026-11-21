#!/usr/bin/env python3
"""攻方腿模型：冻结的 merge-and-verify.sh 里挑槽的那段 python（SLOT_HOLDER_PROGRAM）原样起，几趟同时挑槽、清多出来的槽，
核它会不会删掉别的趟占着的槽、留着的槽，挑的是不是按 used 排的前 slots 槽（正文判据格第 1 格前两项）。

用法：slot_trim_race.py --frozen <冻结副本> --work <新的空目录> [--scenarios N] [--break-scenarios K] [--seed S]
  每个场景：槽那一层里先放 n 槽（used 的修改时刻各不相同，按 SLOT_HOLDER_PROGRAM 的排法定前 slots 槽与多出来的槽），
  其中一部分是留着的（kept 与 repo/ 都在）、一部分由场景另起的进程占着锁（模型自己的样本进程，lockf 拿着 <槽>/lock）；
  再起 m 个挑槽进程（slots>0 起 take、slots=0 起 trim），起的时刻错开几十毫秒，占到的各占一小段时间再放。
  不变量（判一格就打一行 name=slot_trim_race_violation）：
    held-slot-vanished   挑槽进程答了 SLOT held 之后、放之前，它占的槽目录不在了
    double-hold          同一槽同一时刻有两个占着的（两个挑槽进程，或挑槽进程与场景的样本进程）
    external-held-removed 场景的样本进程占着的槽到最后不在了
    kept-removed         留着的槽（kept 在、repo/ 也在）到最后不在了或 kept 没了
    surplus-taken        挑槽进程占了开场时排在前 slots 槽之后的那一槽
    surplus-free-left    开场时多出来的、空着的槽（不留着、没人占）到最后还在
    single-picker-choice 只起一个挑槽进程时，它占的不是「前 slots 槽里空着的、used 最新的那一槽」（没有就该新开或不占）
    slot-count-over      最后的槽数超过 max(开场槽数, slots)
    group-left           模型起的进程组到最后还有进程
  --break-scenarios：另把三个弄坏开关（slot-lock-skipped、slot-surplus-kept-removed、slot-surplus-taken）各跑 K 个偏向能打中的场景，
    每个开关要至少打出一格它该打的不变量（证这个模型会红），打不出的整趟退 1。
  先跑头 5 个场景量挂钟，打出按它估的全量挂钟；超过 40 分钟不跑全量、退 3。
退出码：0 跑完（不变量打中与否都在输出里）；1 弄坏开关没打红（模型判不出）；2 用法错；3 估时超过 40 分钟。
样本进程（sleep、占锁的 python、挑槽进程）各用新会话起，收尾对各自的进程组发 TERM、过 5 秒还在发 KILL，再到 /proc 里核组里没人。
"""
import argparse
import os
import queue
import random
import signal
import subprocess
import sys
import threading
import time

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import extract_program  # noqa: E402

EXTERNAL_HOLDER = ("import fcntl, sys, time\n"
                   "handle = open(sys.argv[1], 'a')\n"
                   "fcntl.lockf(handle, fcntl.LOCK_EX)\n"
                   "print('locked', flush=True)\n"
                   "time.sleep(3600)\n")
ESTIMATE_LIMIT_SECONDS = 40 * 60
EXPECTED_BY_BREAK = {"slot-lock-skipped": {"double-hold"},
                     "slot-surplus-kept-removed": {"kept-removed"},
                     "slot-surplus-taken": {"surplus-taken", "surplus-free-left"}}


def start_in_new_session(arguments, **keywords):
    return subprocess.Popen(arguments, start_new_session=True, **keywords)


def stop_own_group(process):
    """只对自己用新会话起的那一个子进程的组发信号：先 TERM，5 秒还在再 KILL。"""
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.wait()


def group_members(group):
    members = []
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            with open(f"/proc/{entry}/stat", encoding="utf-8", errors="replace") as handle:
                fields = handle.read().rsplit(") ", 1)[1].split()
        except (OSError, IndexError):
            continue
        if int(fields[2]) == group:
            members.append(int(entry))
    return members


def build_scenario(rng, directory, biased):
    """槽那一层与开场状态。biased：给弄坏开关用，偏向多槽、小上限、多个挑槽进程、多留着的槽。"""
    root = os.path.join(directory, "merge-and-verify")
    os.makedirs(root)
    count = rng.randint(3, 5) if biased else rng.randint(0, 5)
    limit = rng.randint(1, 2) if biased else rng.randint(0, 4)
    pickers = rng.randint(2, 4) if biased else rng.randint(1, 4)
    kept_chance, held_chance = (0.5, 0.3) if biased else (0.3, 0.3)
    base = time.time() - 10000
    order = list(range(count))
    rng.shuffle(order)
    slots = []
    for index in range(count):
        slot = os.path.realpath(os.path.join(root, f"slot.case{index}"))
        os.makedirs(os.path.join(slot, "repo"))
        stamp = os.path.join(slot, "used")
        with open(stamp, "w") as handle:
            handle.write("0\n")
        os.utime(stamp, (base + order[index] * 10, base + order[index] * 10))
        kind = "free"
        roll = rng.random()
        if roll < kept_chance:
            kind = "kept"
            with open(os.path.join(slot, "kept"), "w") as handle:
                handle.write("log: model\ncreated: 2026-11-20\n")
        elif roll < kept_chance + held_chance:
            kind = "held"
        elif rng.random() < 0.3:
            with open(os.path.join(slot, "kept"), "w") as handle:   # kept 在而 repo/ 不在：不算留着
                handle.write("log: model\n")
            os.rmdir(os.path.join(slot, "repo"))
        slots.append((slot, order[index], kind))
    ranking = [slot for slot, _, _ in sorted(slots, key=lambda item: item[1], reverse=True)]
    kinds = {slot: kind for slot, _, kind in slots}
    return {"root": root, "limit": limit, "pickers": pickers, "ranking": ranking, "kinds": kinds,
            "eligible": ranking[:limit], "surplus": ranking[limit:]}


def run_scenario(program, scenario, rng, broken):
    violations, groups, answers = [], [], []
    root, limit = scenario["root"], scenario["limit"]
    mode = "take" if limit > 0 else "trim"
    external = []
    for slot, kind in scenario["kinds"].items():
        if kind != "held":
            continue
        holder = start_in_new_session(["python3", "-c", EXTERNAL_HOLDER, os.path.join(slot, "lock")],
                                      stdout=subprocess.PIPE, text=True)
        groups.append(holder.pid)
        if holder.stdout.readline().strip() != "locked":
            violations.append(f"model-error external holder on {slot} did not lock")
        external.append((slot, holder))
    externally_held = {slot for slot, _ in external}
    active, active_lock = {}, threading.Lock()
    results = queue.Queue()

    def picker_body(number, delay, hold):
        time.sleep(delay)
        sleeper = start_in_new_session(["sleep", "3600"])
        picker = start_in_new_session(["python3", "-c", program, root, str(limit), "", str(sleeper.pid), broken, mode],
                                      stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        with active_lock:
            groups.extend([sleeper.pid, picker.pid])
        held, lines = None, []
        for line in picker.stdout:
            line = line.rstrip("\n")
            lines.append(line)
            if line.startswith("SLOT held "):
                held = line[len("SLOT held "):].split("\t", 1)[0]
                break
            if line.startswith("SLOT none "):
                break
        if held is not None:
            with active_lock:
                if held in active or held in externally_held:
                    results.put(("double-hold", f"picker {number} holds {held} while it is held by {active.get(held, 'external sample')}"))
                active[held] = f"picker {number}"
            if not os.path.isdir(held):
                results.put(("held-slot-vanished", f"picker {number}: {held} missing right after SLOT held"))
            time.sleep(hold)
            if not os.path.isdir(held):
                results.put(("held-slot-vanished", f"picker {number}: {held} missing before release"))
            with active_lock:
                active.pop(held, None)
        stop_own_group(sleeper)
        try:
            rest, _ = picker.communicate(timeout=10)
            lines.extend(rest.splitlines())
        except subprocess.TimeoutExpired:
            stop_own_group(picker)
            results.put(("model-error", f"picker {number} did not exit after its parent went away"))
        answers.append((number, held, lines))

    threads = [threading.Thread(target=picker_body, args=(number, rng.uniform(0, 0.2), rng.uniform(0, 0.5)))
               for number in range(scenario["pickers"])]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    while not results.empty():
        violations.append(" ".join(results.get()))
    for _, holder in external:
        stop_own_group(holder)
    return violations, groups, answers, externally_held


def judge_end_state(scenario, answers, externally_held, initial_count):
    violations = []
    kinds, surplus, eligible, limit = scenario["kinds"], scenario["surplus"], scenario["eligible"], scenario["limit"]
    for slot in externally_held:
        if not os.path.isdir(slot):
            violations.append(f"external-held-removed {slot}")
    for slot, kind in kinds.items():
        if kind == "kept" and not (os.path.isdir(slot) and os.path.isfile(os.path.join(slot, "kept"))):
            violations.append(f"kept-removed {slot}")
    held_slots = [held for _, held, _ in answers if held]
    for held in held_slots:
        if held in surplus:
            violations.append(f"surplus-taken {held}")
    for slot in surplus:
        if kinds[slot] == "free" and slot not in held_slots and os.path.isdir(slot):
            violations.append(f"surplus-free-left {slot}")
    if len(answers) == 1:
        free_eligible = [slot for slot in eligible if kinds[slot] == "free"]
        held = answers[0][1]
        if limit == 0:
            expected = None
        elif free_eligible:
            expected = free_eligible[0]
        elif len(kinds) < limit:
            expected = "new"
        else:
            expected = None
        fine = (held == expected) or (expected == "new" and held is not None and held not in kinds)
        if not fine:
            violations.append(f"single-picker-choice expected {expected} got {held}")
    final = [entry for entry in os.listdir(scenario["root"]) if entry.startswith("slot.")]
    if len(final) > max(initial_count, limit):
        violations.append(f"slot-count-over {len(final)} > max({initial_count}, {limit})")
    return violations


def one_scenario(program, work, index, rng, broken, biased):
    directory = os.path.join(work, f"{broken or 'intact'}-{index}")
    os.makedirs(directory)
    scenario = build_scenario(rng, directory, biased)
    violations, groups, answers, externally_held = run_scenario(program, scenario, rng, broken)
    violations += judge_end_state(scenario, answers, externally_held, len(scenario["kinds"]))
    for group in groups:
        if group_members(group):
            violations.append(f"group-left {group}: {group_members(group)}")
    counts = {kind: sum(1 for value in scenario["kinds"].values() if value == kind) for kind in ("free", "kept", "held")}
    print(f"name=slot_trim_race_scenario arm={broken or 'intact'} index={index} slots={len(scenario['kinds'])} "
          f"free={counts['free']} kept={counts['kept']} held={counts['held']} limit={scenario['limit']} pickers={scenario['pickers']} "
          f"took={sum(1 for _, held, _ in answers if held)} violations={len(violations)}", flush=True)
    for violation in violations:
        print(f"name=slot_trim_race_violation arm={broken or 'intact'} index={index} {violation}", flush=True)
    return violations


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    parser.add_argument("--scenarios", type=int, default=120)
    parser.add_argument("--break-scenarios", type=int, default=30)
    parser.add_argument("--seed", type=int, default=20261120)
    arguments = parser.parse_args()
    program = extract_program.extract(os.path.join(arguments.frozen, "merge-and-verify.sh"), "SLOT_HOLDER_PROGRAM")
    os.makedirs(arguments.work, exist_ok=False)
    rng = random.Random(arguments.seed)
    started = time.monotonic()
    probe = min(5, arguments.scenarios)
    intact = []
    for index in range(probe):
        intact += one_scenario(program, arguments.work, index, rng, "", False)
    per_scenario = (time.monotonic() - started) / max(probe, 1)
    total_runs = arguments.scenarios + 3 * arguments.break_scenarios
    estimate = per_scenario * total_runs
    print(f"name=slot_trim_race_estimate probe_scenarios={probe} seconds_per_scenario={per_scenario:.2f} "
          f"planned_scenarios={total_runs} estimated_seconds={estimate:.0f} limit_seconds={ESTIMATE_LIMIT_SECONDS}", flush=True)
    if estimate > ESTIMATE_LIMIT_SECONDS:
        print("  ✗ 估的全量挂钟超过 40 分钟，不跑全量：缩 --scenarios 或 --break-scenarios 再跑", flush=True)
        return 3
    for index in range(probe, arguments.scenarios):
        intact += one_scenario(program, arguments.work, index, rng, "", False)
    print(f"name=slot_trim_race_summary arm=intact scenarios={arguments.scenarios} violations={len(intact)}", flush=True)
    unproven = []
    for broken, expected in EXPECTED_BY_BREAK.items():
        found = []
        for index in range(arguments.break_scenarios):
            found += one_scenario(program, arguments.work, index, rng, broken, True)
        hit = sorted({violation.split()[0] for violation in found} & expected)
        print(f"name=slot_trim_race_summary arm={broken} scenarios={arguments.break_scenarios} violations={len(found)} "
              f"expected_kinds_hit={','.join(hit) or 'none'}", flush=True)
        if not hit:
            unproven.append(broken)
    print(f"name=slot_trim_race_done seconds={time.monotonic() - started:.0f} unproven_breaks={','.join(unproven) or 'none'}", flush=True)
    return 1 if unproven else 0


if __name__ == "__main__":
    sys.exit(main())
