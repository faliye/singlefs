#!/usr/bin/env python3
"""P1 与 P7 的计数模型（纯算术，不读仓里的代码；按发布与秒计，事件逐个模拟）。

一、丢窗口：一次改动收下到它所在那一批发布之间的秒数。三种「谁判时间那一支」：
  entry  只在下一次请求进门时判（空闲时没有下一次请求）；
  tick   调用方每 tick_period 秒调一次「到点了没」入口，入口与别的会话入口共用 &mut self，
         调用方线程上正跑着的一次长调用（整理一趟、一次大 unlink）要跑完才轮得到它；
  tick_monotonic_backwards  同 tick，但调用方交进来的钟在收下改动之后往回拨了 backwards 秒
         （核心层照整理定时兜底的写法 saturating_sub：钟往回走按没过间隔算）。
  由用户决定的几样放开扫：改动之后空闲多久、tick 周期、长调用多长、钟回拨多少。
二、空闲时空发布挤根环（D16 已定项 2 的式子照字面：now − 上次 ≥ T_time 就触发，不看脏字节是不是 0）：
  根环 3S 槽，先发 w 次改了用户可见状态的发布（w 由用户定），再空闲 idle 秒；照字面每 T_time 秒一次空发布。
  数环里还留着的不同用户可见状态个数（回退候选集至多 4 个、只数环里有的）。对照臂：脏字节为 0 不触发。
三、串行下 T_time 小于一次发布耗时 D（C425）：调用方 tick 周期 p，上次 checkpoint 的时刻取触发那一刻。
  空闲一小时里发几次：照字面每次发完立刻到点；加「脏字节 > 0」那一臂为 0。持续写的负载下，一次改动收下到发布至多多久。
"""
import itertools


def loss_window(arm, idle, tick_period, blocking_call, backwards, t_time):
    """一次改动在 t=0 收下，之后 idle 秒没有请求；返回它被发布的时刻（秒），发布不了返回 None。"""
    if arm == "entry":
        return None  # 空闲期间没有下一次请求，永远不判
    # tick：调用方从 0 起每 tick_period 秒调一次；t=0 起有一次长调用占着调用方线程 blocking_call 秒
    clock_offset = -backwards if arm == "tick_monotonic_backwards" else 0.0
    last_checkpoint_reading = 0.0  # 收下改动那一刻的钟读数当成上次 checkpoint（最有利的读法）
    k = 0
    while True:
        k += 1
        wall = k * tick_period
        if wall < blocking_call:
            continue
        if wall > idle:
            return None
        reading = wall + clock_offset
        elapsed = max(0.0, reading - last_checkpoint_reading)  # saturating_sub
        if elapsed >= t_time:
            return wall


def section_loss_window():
    t_time = 5.0
    print("# 一、丢窗口（T_time = 5 s；None = 空闲期间发布不了，窗口等于空闲时长、无界）")
    rows = 0
    worst = {}
    for arm, idle, tick_period, blocking_call, backwards in itertools.product(
        ["entry", "tick", "tick_monotonic_backwards"],
        [10.0, 60.0, 3600.0],
        [0.1, 1.0, 5.0],
        [0.0, 30.0],
        [0.0, 120.0],
    ):
        if arm != "tick_monotonic_backwards" and backwards:
            continue
        rows += 1
        published = loss_window(arm, idle, tick_period, blocking_call, backwards, t_time)
        key = arm
        value = idle if published is None else published
        worst[key] = max(worst.get(key, 0.0), value)
        print(f"name=loss_window arm={arm} idle={idle:g} tick_period={tick_period:g} blocking_call={blocking_call:g} backwards={backwards:g} published_at={published}")
    for arm in ["entry", "tick", "tick_monotonic_backwards"]:
        print(f"name=loss_window_summary arm={arm} worst_window_seconds={worst[arm]:g} formula_t_time={t_time:g}")
    print(f"name=loss_window_rows rows={rows}")


def distinct_states_left(slots, writes, empty_publishes):
    """根环 slots 槽按 txg 轮转；先 writes 次各改一次状态，再 empty_publishes 次空发布。返回环里不同状态的个数。"""
    ring = []
    state = 0
    for _ in range(writes):
        state += 1
        ring.append(state)
    for _ in range(empty_publishes):
        ring.append(state)  # 空发布：状态不变
    kept = ring[-slots:]
    return len(set(kept))


def section_ring_churn():
    print("# 二、空闲时照字面推空发布：环里还剩几个不同状态（回退候选集至多 4）")
    rows = 0
    collapsed = 0
    for regions_slots, t_time, idle, writes in itertools.product([4, 8, 16], [0.2, 1.0, 5.0], [10, 60, 120, 600, 3600], [1, 4, 6]):
        slots = 3 * regions_slots
        empty_literal = int(idle // t_time)
        literal = min(4, distinct_states_left(slots, writes, empty_literal))
        guarded = min(4, distinct_states_left(slots, writes, 0))
        rows += 1
        if literal == 1 and guarded > 1:
            collapsed += 1
        print(f"name=ring_churn S={regions_slots} T_time={t_time:g} idle={idle} writes_before_idle={writes} empty_publishes_literal={empty_literal} candidates_literal={literal} candidates_dirty_guarded={guarded}")
    seconds_to_collapse = {s: 3 * s * 5.0 for s in [4, 8, 16]}
    print(f"name=ring_churn_summary rows={rows} rows_collapsed_to_one_state_only_under_the_literal_formula={collapsed} idle_seconds_to_collapse_at_T_time_5={seconds_to_collapse}")


def section_serial_short_t_time():
    print("# 三、串行、T_time < 发布耗时 D：空闲一小时的发布次数，与持续写下一次改动收下到发布的最长时间")
    rows = 0
    for t_time, publish_seconds, tick_period in itertools.product([0.2, 1.0], [0.5, 2.0, 10.0], [0.05, 1.0]):
        # 照字面：每次 tick 判 now − 上次触发时刻 ≥ T_time；发布占着线程 D 秒，发完下一次 tick 一定到点
        cycle = max(t_time, publish_seconds, tick_period) if publish_seconds >= t_time else max(t_time, tick_period)
        idle_publishes_literal = int(3600 // (publish_seconds + max(0.0, cycle - publish_seconds)))
        # 持续写：一次改动最坏在一次发布刚开始之后收下，要等这次发完、下一次 tick 到点、再发完
        worst_wait = publish_seconds + max(t_time, tick_period) + publish_seconds
        rows += 1
        print(f"name=serial_short_t_time T_time={t_time:g} D={publish_seconds:g} tick_period={tick_period:g} idle_hour_publishes_literal={idle_publishes_literal} idle_hour_publishes_dirty_guarded=0 busy_worst_accept_to_durable_seconds={worst_wait:g}")
    print(f"name=serial_short_t_time_rows rows={rows}")


if __name__ == "__main__":
    section_loss_window()
    section_ring_churn()
    section_serial_short_t_time()
