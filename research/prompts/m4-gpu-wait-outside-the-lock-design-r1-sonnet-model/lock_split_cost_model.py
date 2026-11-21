#!/usr/bin/env python3
"""会话锁上「后台一批」拆成取段、锁外算、回锁收段之后的代价模型（纯算术，只用标准库，不读 crates/，不写文件，只打标准输出）。

这是设计推论轮第一轮正推腿的模型，只用来独立验证报告里的几句话，不与 crates/ 共用代码：
  1. 今天（整批持锁）前台等锁的下限：整批在锁里 T = a + W + b，前台均匀到达时平均等 T²/(2(T+gap))，最长等 T。
  2. 拆开之后前台只等取段 a 与收段 b 两小段：平均等 (a² + b²)/(2(T+gap))。
  3. 回锁复核「前台发过一次布就作废整批」（split-discard-on-any-publish）时，一批被作废的比例是 1 − e^{−λW}（λ 是前台发布的到达率，W 是锁外那一段）。
  4. 复核只看这一批的 R（split-recheck-the-region-only，前台每次发布碰到 R 的概率 q）时，作废比例是 1 − e^{−qλW}；q = 0 时一次都不作废。
  5. 下限（floor_k）：连着作废 k 次之后下一批整批放回锁里（今天的做法），连续作废次数不超过 k。

参数里 W 是拍的网格，不是量的数：真值由报告「要主 agent 跑的命令」一节的基线探针量（D24 已定项 2 射程写一次 GPU 提交约 5.3 ms，那是 E185 量的单次提交）。
前台与后台在锁上的排队只建两种：先到先得（FIFO）与前台优先（yield，放锁时队里有前台就先给前台）。std::sync::Mutex 的「放锁后原线程立刻再取」
（barging）不在这里建：它只会让今天的前台等得更久，量它是基线探针的事。

用法：
    python3 lock_split_cost_model.py --selftest          # 闭式对拍，退 0 全过、退 1 有格红
    python3 lock_split_cost_model.py --grid [--seconds N] # 打网格，每格一行 name=lock_split_cell ...
弄坏开关（只给 --selftest 证红）：
    LOCK_SPLIT_MODEL_BREAK=never-discard   回锁复核从不作废  → 「split-discard-on-any-publish 作废比例」「下限」两格红
    LOCK_SPLIT_MODEL_BREAK=fg-never-waits  前台取锁不排队    → 「today 前台平均等」「split-discard-on-any-publish 前台平均等」两格红
"""
import heapq
import math
import os
import random
import sys

BROKEN = os.environ.get("LOCK_SPLIT_MODEL_BREAK", "")


class Spec:
    """一种策略。split：拆成取段、锁外、收段；recheck：txg（前台发过布就作废）或 region（只看 R，前台每次发布碰到 R 的概率 q）；
    yield_to_fg：放锁时队里有前台就先给前台；floor_k：连着作废 k 次之后下一批整批放回锁里，0 是没有下限。"""

    def __init__(self, name, split, recheck="txg", yield_to_fg=False, floor_k=0):
        self.name = name
        self.split = split
        self.recheck = recheck
        self.yield_to_fg = yield_to_fg
        self.floor_k = floor_k


SPECS = {
    "today": Spec("today", split=False),
    "split-discard-on-any-publish": Spec("split-discard-on-any-publish", split=True, recheck="txg"),
    "split-discard-on-any-publish-foreground-first": Spec("split-discard-on-any-publish-foreground-first", split=True, recheck="txg", yield_to_fg=True),
    "split-discard-on-any-publish-floor2": Spec("split-discard-on-any-publish-floor2", split=True, recheck="txg", yield_to_fg=True, floor_k=2),
    "split-recheck-the-region-only": Spec("split-recheck-the-region-only", split=True, recheck="region", yield_to_fg=True),
    "split-recheck-the-region-only-floor2": Spec("split-recheck-the-region-only-floor2", split=True, recheck="region", yield_to_fg=True, floor_k=2),
}


def default_params():
    """毫秒。a 取段在锁里的时长、W 锁外那一段（读盘加 GPU 等加算）、b 收段在锁里的时长（复核加计划加一次发布）、gap 两批之间后台不要锁的空档。
    lam_pub 前台发布的到达率（每秒）、lam_read 前台只读的到达率（每秒）；h_pub、h_read 它们持锁的时长；q 前台一次发布碰到这一批 R 的概率。"""
    return dict(a=0.05, W=10.0, b=2.0, gap=0.05, lam_pub=0.0, lam_read=200.0, h_pub=2.0, h_read=0.01, q=0.0)


def simulate(spec, params, seed, sim_ms):
    """一条后台线程加两路前台（发布、只读）争一把锁，跑 sim_ms 毫秒。交回前台等锁的时长与后台发布、作废的计数。"""
    rng = random.Random(seed)
    events = []
    counter = [0]

    def push(when, kind, payload=None):
        heapq.heappush(events, (when, counter[0], kind, payload))
        counter[0] += 1

    queue = []
    state = dict(holder=None, txg=0, window_open=False, stamp_txg=0, touched=False, consecutive=0)
    stats = dict(waits_pub=[], waits_read=[], bg_publishes=0, discards=0, cycles=0, max_consecutive=0)
    a, w, b, gap = params["a"], params["W"], params["b"], params["gap"]

    def grant(request, now):
        state["holder"] = request
        if request["who"] == "fg":
            wait = 0.0 if BROKEN == "fg-never-waits" else now - request["arrived"]
            (stats["waits_pub"] if request["publishes"] else stats["waits_read"]).append(wait)
            hold = request["hold"]
        elif request["phase"] == "whole":
            hold = a + w + b
        elif request["phase"] == "take":
            hold = a
        else:
            # 收段：此刻比对。复核不过的这一批作废，只花一次取段那么长；过了发布，花 b。
            if BROKEN == "never-discard":
                discard = False
            elif spec.recheck == "txg":
                discard = state["txg"] != state["stamp_txg"]
            else:
                discard = state["touched"]
            request["discard"] = discard
            hold = a if discard else b
        push(now + hold, "release", request)

    def pick_next(now):
        if not queue:
            return
        index = 0
        if spec.yield_to_fg:
            index = next((i for i, request in enumerate(queue) if request["who"] == "fg"), 0)
        grant(queue.pop(index), now)

    def request_lock(request, now):
        if state["holder"] is None and not queue:
            grant(request, now)
        else:
            queue.append(request)

    def start_cycle(now):
        stats["cycles"] += 1
        floor_hit = spec.floor_k > 0 and state["consecutive"] >= spec.floor_k
        phase = "take" if spec.split and not floor_hit else "whole"
        request_lock(dict(who="bg", phase=phase, arrived=now), now)

    push(0.0, "bg_cycle_start")
    for kind, rate in (("fg_pub", params["lam_pub"]), ("fg_read", params["lam_read"])):
        if rate > 0:
            push(rng.expovariate(rate / 1000.0), kind)
    while events:
        now, _order, kind, payload = heapq.heappop(events)
        if now > sim_ms:
            break
        if kind == "bg_cycle_start":
            start_cycle(now)
        elif kind == "bg_outside_done":
            request_lock(dict(who="bg", phase="finish", arrived=now), now)
        elif kind in ("fg_pub", "fg_read"):
            is_pub = kind == "fg_pub"
            request_lock(dict(who="fg", publishes=is_pub, hold=params["h_pub"] if is_pub else params["h_read"], arrived=now), now)
            push(now + rng.expovariate((params["lam_pub"] if is_pub else params["lam_read"]) / 1000.0), kind)
        else:
            request = payload
            state["holder"] = None
            if request["who"] == "fg":
                if request["publishes"]:
                    state["txg"] += 1
                    if state["window_open"] and rng.random() < params["q"]:
                        state["touched"] = True
            elif request["phase"] == "take":
                state["window_open"] = True
                state["stamp_txg"] = state["txg"]
                state["touched"] = False
                push(now + w, "bg_outside_done")
            else:
                state["window_open"] = False
                if request["phase"] == "whole" or not request["discard"]:
                    stats["bg_publishes"] += 1
                    state["txg"] += 1
                    state["consecutive"] = 0
                else:
                    stats["discards"] += 1
                    state["consecutive"] += 1
                    stats["max_consecutive"] = max(stats["max_consecutive"], state["consecutive"])
                push(now + gap, "bg_cycle_start")
            pick_next(now)
    return stats


def percentile(sorted_values, fraction):
    if not sorted_values:
        return 0.0
    return sorted_values[min(len(sorted_values) - 1, int(fraction * len(sorted_values)))]


def summary(spec, params, seed, sim_ms):
    stats = simulate(spec, params, seed, sim_ms)
    waits = sorted(stats["waits_pub"] + stats["waits_read"])
    finished = stats["bg_publishes"] + stats["discards"]
    return dict(
        fg_calls=len(waits),
        fg_wait_ms_mean=(sum(waits) / len(waits)) if waits else 0.0,
        fg_wait_ms_p99=percentile(waits, 0.99),
        fg_wait_ms_max=waits[-1] if waits else 0.0,
        bg_publishes_per_s=stats["bg_publishes"] / (sim_ms / 1000.0),
        discards=stats["discards"],
        discard_fraction=(stats["discards"] / finished) if finished else 0.0,
        max_consecutive_discards=stats["max_consecutive"],
        cycles=stats["cycles"],
    )


def selftest():
    """闭式对拍。每格 (名字, 是否通过, 读数)；全过退 0。"""
    cells = []
    seed = 20261118

    reads_only = dict(default_params(), lam_pub=0.0, lam_read=200.0)
    cycle = reads_only["a"] + reads_only["W"] + reads_only["b"] + reads_only["gap"]
    held = reads_only["a"] + reads_only["W"] + reads_only["b"]
    today = summary(SPECS["today"], reads_only, seed, 300_000.0)
    expected = held * held / (2 * cycle)
    cells.append(("today 前台平均等 ≈ T²/(2(T+gap))", abs(today["fg_wait_ms_mean"] - expected) / expected < 0.05, (today["fg_wait_ms_mean"], expected)))
    cells.append(("today 前台最长等 ≤ T", today["fg_wait_ms_max"] <= held + 0.5, (today["fg_wait_ms_max"], held)))
    cells.append(("today 后台每秒发布 ≈ 1000/(T+gap)", abs(today["bg_publishes_per_s"] - 1000.0 / cycle) / (1000.0 / cycle) < 0.01, (today["bg_publishes_per_s"], 1000.0 / cycle)))
    split = summary(SPECS["split-discard-on-any-publish"], reads_only, seed, 300_000.0)
    expected = (reads_only["a"] ** 2 + reads_only["b"] ** 2) / (2 * cycle)
    cells.append(("split-discard-on-any-publish 前台平均等 ≈ (a²+b²)/(2(T+gap))", abs(split["fg_wait_ms_mean"] - expected) / expected < 0.10, (split["fg_wait_ms_mean"], expected)))

    with_publishes = dict(default_params(), lam_pub=20.0, lam_read=0.0, h_pub=0.5, W=50.0)
    expected = 1.0 - math.exp(-with_publishes["lam_pub"] / 1000.0 * with_publishes["W"])
    discard_on_any_publish = summary(SPECS["split-discard-on-any-publish"], with_publishes, seed, 600_000.0)
    cells.append(("split-discard-on-any-publish 作废比例 ≈ 1 − e^{−λW}", abs(discard_on_any_publish["discard_fraction"] - expected) < 0.06, (discard_on_any_publish["discard_fraction"], expected)))
    cells.append(("split-discard-on-any-publish 没有下限时连续作废会很长（≥ 5）", discard_on_any_publish["max_consecutive_discards"] >= 5, discard_on_any_publish["max_consecutive_discards"]))
    floored = summary(SPECS["split-discard-on-any-publish-floor2"], with_publishes, seed, 600_000.0)
    cells.append(("下限 2：连续作废恰好封在 2，后台仍有发布", floored["max_consecutive_discards"] == 2 and floored["bg_publishes_per_s"] > 0, (floored["max_consecutive_discards"], floored["bg_publishes_per_s"])))
    touch_never = summary(SPECS["split-recheck-the-region-only"], dict(with_publishes, q=0.0), seed, 600_000.0)
    cells.append(("split-recheck-the-region-only q=0 一次都不作废", touch_never["discards"] == 0, touch_never["discards"]))
    touch_always = summary(SPECS["split-recheck-the-region-only"], dict(with_publishes, q=1.0), seed, 600_000.0)
    cells.append(("split-recheck-the-region-only q=1 与 split-discard-on-any-publish 的作废比例同量级", abs(touch_always["discard_fraction"] - expected) < 0.06, (touch_always["discard_fraction"], expected)))

    red = [cell for cell in cells if not cell[1]]
    for label, passed, reading in cells:
        print(f"  {'✓' if passed else '✗ 自检：'}{label} 读数={reading}")  # gate-lint:detail
    print(f"  自检查了 {len(cells)} 格，红 {len(red)} 格")
    return 1 if red else 0


def grid(seconds):
    sim_ms = seconds * 1000.0
    for window in (5.0, 20.0, 100.0, 1000.0):
        for lam_pub in (1.0, 10.0, 100.0, 300.0):
            for name in ("today", "split-discard-on-any-publish", "split-discard-on-any-publish-foreground-first", "split-discard-on-any-publish-floor2", "split-recheck-the-region-only", "split-recheck-the-region-only-floor2"):
                params = dict(default_params(), W=window, lam_pub=lam_pub, q=0.02)
                reading = summary(SPECS[name], params, 7, sim_ms)
                print(
                    f"name=lock_split_cell policy={name} W_ms={window} lam_pub_per_s={lam_pub} q={params['q']} "
                    f"fg_calls={reading['fg_calls']} fg_wait_ms_mean={reading['fg_wait_ms_mean']:.3f} fg_wait_ms_p99={reading['fg_wait_ms_p99']:.3f} "
                    f"fg_wait_ms_max={reading['fg_wait_ms_max']:.3f} bg_publishes_per_s={reading['bg_publishes_per_s']:.3f} "
                    f"discards={reading['discards']} discard_fraction={reading['discard_fraction']:.3f} "
                    f"max_consecutive_discards={reading['max_consecutive_discards']} cycles={reading['cycles']}"
                )


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    if arguments[:1] == ["--grid"]:
        seconds = 120.0
        if len(arguments) == 3 and arguments[1] == "--seconds":
            seconds = float(arguments[2])
        elif len(arguments) != 1:
            print("用法：--grid [--seconds N]", file=sys.stderr)
            return 2
        grid(seconds)
        return 0
    print("用法：lock_split_cost_model.py --selftest | --grid [--seconds N]", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.dont_write_bytecode = True
    sys.exit(main(sys.argv[1:]))
