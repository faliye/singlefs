#!/usr/bin/env python3
"""甲（两端对扫）的时序模型：本机从头往后领、第二台从尾往前领，各自把「领到哪了」写成前沿，对方每隔一段读一次。

照引擎今天的样子排段：观察者的录入段一个观察者一遍（crash_amplification.rs 的 record_the_observers_of_the_flow 里
块序号在每个观察者那一遍从 0 起），之后判段一遍；判段判观察者结论要读这台机器库里的录入块（judge_the_observers_of_the_flow
读不到就报「先跑录入段」整趟出错）。前沿两种写法：
  single  前沿只是一个块序号（候选正文「领到哪了」的字面）；
  phased  前沿是（第几遍，块序号）按字典序比。
停的三种：stop-file（判完手里那一块再停）、term（手里那一块丢掉）、offline（同 term，且本机此后读不到第二台的前沿）。
只用 python3 标准库；输出一行一格，键=值。
"""
import itertools
import sys

DT = 0.05


def simulate(blocks, phases, cost, poll_seconds, front_kind, stop_at=None, stop_kind=None, done_front="last"):
    hosts = ("local", "peer")
    state = {h: {"phase": 0, "cursor": 0 if h == "local" else blocks - 1, "busy_until": None,
                 "claimed": None, "done": False, "stopped": False, "finish": None} for h in hosts}
    known = {"local": None, "peer": None}  # 这台读到的对方前沿
    completed = {}  # (phase, block) -> [host…]
    abandoned = []

    def front(h):
        s = state[h]
        if s["claimed"] is not None:
            return s["claimed"]
        if s["done"] and done_front == "sentinel":
            return (len(phases), blocks if h == "local" else -1)
        return s.get("last_front")

    def allowed(h, phase, block):
        other = known[h]
        if other is None:
            return True
        if front_kind == "single":
            other_block = other[1]
            return block < other_block if h == "local" else block > other_block
        other_phase, other_block = other
        if other_phase < phase:
            return True
        if other_phase > phase:
            return False
        return block < other_block if h == "local" else block > other_block

    t = 0.0
    next_poll = 0.0
    while not all(state[h]["done"] or state[h]["stopped"] for h in hosts):
        if t >= next_poll:
            for h in hosts:
                other = "peer" if h == "local" else "local"
                if h == "local" and stop_kind == "offline" and stop_at is not None and t >= stop_at:
                    continue
                known[h] = front(other)
            next_poll += poll_seconds
        for h in hosts:
            s = state[h]
            if s["done"] or s["stopped"]:
                continue
            if h == "peer" and stop_at is not None and t >= stop_at and stop_kind in ("term", "offline"):
                if s["claimed"] is not None:
                    abandoned.append(s["claimed"])
                s["stopped"] = True
                s["finish"] = t
                continue
            if s["busy_until"] is not None and t >= s["busy_until"]:
                completed.setdefault(s["claimed"], []).append(h)
                s["last_front"] = s["claimed"]
                s["claimed"] = None
                s["busy_until"] = None
                s["cursor"] += 1 if h == "local" else -1
            if s["busy_until"] is None:
                if h == "peer" and stop_at is not None and t >= stop_at and stop_kind == "stop-file":
                    s["stopped"] = True
                    s["finish"] = t
                    continue
                while True:
                    if s["phase"] >= len(phases):
                        s["done"] = True
                        s["finish"] = t
                        break
                    block = s["cursor"]
                    inside = 0 <= block < blocks
                    if inside and allowed(h, s["phase"], block):
                        s["claimed"] = (s["phase"], block)
                        s["busy_until"] = t + cost[h][phases[s["phase"]]]
                        break
                    s["phase"] += 1
                    s["cursor"] = 0 if h == "local" else blocks - 1
        t += DT
    return completed, abandoned, state, t


PHASES = ("every_state", "mount_continuation", "judge")
RECORD_PHASES = ("every_state", "mount_continuation")


def tally(blocks, completed, abandoned):
    rows = {}
    for index, name in enumerate(PHASES):
        done_by = [completed.get((index, block), []) for block in range(blocks)]
        rows[name] = {
            "duplicated": sum(1 for hosts in done_by if len(hosts) > 1),
            "holes": sum(1 for hosts in done_by if not hosts),
            "local": sum(1 for hosts in done_by if "local" in hosts),
            "peer": sum(1 for hosts in done_by if "peer" in hosts),
        }
    judge_index = PHASES.index("judge")
    misses = {"local": 0, "peer": 0}
    for block in range(blocks):
        for host in completed.get((judge_index, block), []):
            for record_index, _ in enumerate(RECORD_PHASES):
                if host not in completed.get((record_index, block), []):
                    misses[host] += 1
                    break
    rows["judge_reads_a_block_recorded_elsewhere"] = misses
    rows["abandoned_claims"] = len(abandoned)
    return rows


def costs(local_cpu_factor, local_judge_seconds, peer_judge_seconds):
    # 每块秒数。第二台录入：every_state 13402 块 9611 秒、mount_continuation 每秒 1.0 块（records/2026-10-13-复用键与全仓改名.md 监控第 39 轮，量的）；
    # 判段：上一趟本机 8.8 万、第二台 17.5 万态每秒（同一份记录 2026-10-17 那一条，量的），块按 65536 态折。本机录入速度没量，按因子扫。
    peer = {"every_state": 9611 / 13402, "mount_continuation": 1.0, "judge": peer_judge_seconds}
    local = {"every_state": peer["every_state"] / local_cpu_factor,
             "mount_continuation": peer["mount_continuation"] / local_cpu_factor,
             "judge": local_judge_seconds}
    return {"local": local, "peer": peer}


def main():
    blocks = int(sys.argv[1]) if len(sys.argv) > 1 else 400
    judge_local = 65536 / 88000
    judge_peer = 65536 / 175000
    print(f"name=jia_parameters blocks_per_phase={blocks} phases={','.join(PHASES)} judge_local_seconds_per_block={judge_local:.3f} judge_peer_seconds_per_block={judge_peer:.3f}")
    for front_kind, poll, factor, done_front in itertools.product(("single", "phased"), (5, 30, 120), (0.5, 1.0, 2.0), ("last", "sentinel")):
        completed, abandoned, state, end = simulate(blocks, PHASES, costs(factor, judge_local, judge_peer), poll, front_kind, done_front=done_front)
        rows = tally(blocks, completed, abandoned)
        print(f"name=jia_no_stop front={front_kind} poll_seconds={poll} local_cpu_factor={factor} done_front={done_front} "
              + " ".join(f"{phase}_dup={rows[phase]['duplicated']} {phase}_holes={rows[phase]['holes']} {phase}_local={rows[phase]['local']} {phase}_peer={rows[phase]['peer']}" for phase in PHASES)
              + f" judge_reads_elsewhere_local={rows['judge_reads_a_block_recorded_elsewhere']['local']} judge_reads_elsewhere_peer={rows['judge_reads_a_block_recorded_elsewhere']['peer']}"
              + f" finish_local={state['local']['finish']:.0f} finish_peer={state['peer']['finish']:.0f}")
    # 停：第二台在录入段的不同时刻停下，三种停法；只固定前沿写法与轮询，停的时刻放开扫
    for front_kind, kind in itertools.product(("single", "phased"), ("stop-file", "term", "offline")):
        for stop_at in range(20, 400, 40):
            completed, abandoned, state, end = simulate(blocks, PHASES, costs(1.0, judge_local, judge_peer), 30, front_kind, stop_at=stop_at, stop_kind=kind)
            rows = tally(blocks, completed, abandoned)
            print(f"name=jia_peer_stops front={front_kind} stop_kind={kind} stop_at={stop_at} abandoned_claims={rows['abandoned_claims']} "
                  + " ".join(f"{phase}_holes={rows[phase]['holes']} {phase}_peer={rows[phase]['peer']}" for phase in PHASES)
                  + f" judge_reads_elsewhere_local={rows['judge_reads_a_block_recorded_elsewhere']['local']}")


if __name__ == "__main__":
    main()
