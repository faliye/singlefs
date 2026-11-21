#!/usr/bin/env python3
"""m4-gpu-wait-outside-the-lock-design-r2 攻方腿：正文第三节形态的离散事件模型（不读 crates/，不碰盘）。

一条维护线程照 crates/singlefs-mount-runtime/src/lib.rs 的线程体与一拍（每拍至多 64 次调用、调用之间放锁、trip 没完下一拍隔 1 秒），
几条前台线程（发布、只收下不发、只读）与一把会话锁。两条臂：
  today  照今天的代码：每批整批在锁里算，只为登记了的 fsync 让路，破例批不让。
  form   照第三节：锁里取输入、锁外算、回锁复核（txg 相等且待发为空）再发布；对不上整批作废、不扣预算、trip 仍在途；
         让路只在取输入之前问，任何前台 try_lock 失败才登记；连着让路 N 次之后一批不让；连着作废 k 批之后一批回锁里算；破例批恒在锁里算、不让。
形态没写死、这里放开扫的读法：作废与出错谁先（error_first）、让路打断不打断「连着作废」（yield_resets_k）、
作废之后这一拍接着叫还是停（continue_after_invalidation）、锁是抢占式（std Mutex 在 Linux 上放锁的线程能立刻再取到）还是排队交接（barging）。
时长与持锁时长全是拍的（PARAMETERS 一节），不是量的；读数只说「照这套形态与这些时长，会怎样」。

用法：python3 form_model.py --out <目录> [--quick]
  --quick  每组只跑一个种子、时长缩到五分之一（看装置通不通，不当读数）
退出码：0 跑完、自检过；1 自检红；2 用法错；3 估时超过 40 分钟没跑全量。
"""
import argparse
import collections
import heapq
import itertools
import os
import random
import sys
import time

# PARAMETERS（毫秒；都是拍的）
HOLD_TAKING_THE_INPUT = 1.0       # 判起跑、排空待发之外的取输入：读现行那一版、开批
HOLD_RECHECK = 0.1                # 回锁复核
HOLD_PUBLISH = 2.0                # 一次发布持锁
HOLD_FOREGROUND_PUBLISH = 2.0     # 前台发布（fsync、单发一次用户改动）
HOLD_READ = 0.2                   # 前台读
HOLD_PUBLISH_WHEN_DUE_CHECK = 0.1
GPU_BATCH = 10.0                  # 一批的 GPU 那一段（W）
HANG = 40000.0                    # 卡挂住时一段一次调用最多等的（4 × 等的上限 10 秒，正文第三节第 9 条）
POLL = 1000.0                     # publish_poll_period 默认
PERIOD = 30000.0                  # compaction_period 默认
TIMER_INTERVAL = 15000.0          # 定时兜底间隔取周期的一半
T_TIME = 5000.0
BUDGET = 20                       # 3S − 4，S = 8
CALLS_PER_PASS = 64               # COMPACTION_CALLS_PER_MAINTENANCE_PASS_AT_MOST
WAKE = 0.05                       # 放锁之后被唤醒的等者到它真去取锁的延迟

MAINTENANCE = "maintenance"
WAKE_EVENT = "wake"


class Simulation:
    def __init__(self, configuration, seed):
        self.configuration = configuration
        self.random = random.Random(seed)
        self.now = 0.0
        self.heap = []
        self.sequence = itertools.count()
        self.threads = {}
        self.holder = None
        self.queue = collections.deque()
        self.wake_scheduled = False
        # 会话里的
        self.txg = 0
        self.pending = False
        self.last_publish_at = 0.0
        self.publishes_since_the_last_user_change = 0
        self.trip_under_way = False
        self.exception_open = False
        self.last_timer_start = -TIMER_INTERVAL
        self.memo_txg = None
        self.task_stopped = False
        # 锁外的计数
        self.fsync_waiters = 0
        self.lock_waiters = 0
        self.consecutive_yields = 0
        self.consecutive_invalidations = 0
        self.waiting_since = {}
        self.overtakes = {}
        self.metrics = collections.Counter()
        self.maxima = collections.defaultdict(float)
        self.alarm_at = None

    def schedule(self, at, thread, value):
        heapq.heappush(self.heap, (at, next(self.sequence), thread, value))

    def spawn(self, name, generator):
        self.threads[name] = generator
        self.schedule(0.0, name, None)

    def note_the_holder(self, thread):
        if thread == MAINTENANCE:
            for waiting in self.waiting_since:
                self.overtakes[waiting] += 1
        elif thread in self.waiting_since:
            self.maxima["max_overtakes"] = max(self.maxima["max_overtakes"], self.overtakes.pop(thread))
            del self.waiting_since[thread]

    def give_the_lock(self, thread):
        self.holder = thread
        self.note_the_holder(thread)
        self.schedule(self.now, thread, True)

    def lock_is_takeable(self):
        return self.holder is None and (self.configuration["barging"] or not self.queue)

    def step(self, thread, value):
        try:
            request = self.threads[thread].send(value)
        except StopIteration:
            return
        kind = request[0]
        if kind == "sleep":
            self.schedule(self.now + request[1], thread, None)
        elif kind == "acquire":
            if self.lock_is_takeable():
                self.give_the_lock(thread)
            else:
                self.queue.append(thread)
        elif kind == "try_acquire":
            if self.lock_is_takeable():
                self.give_the_lock(thread)
            else:
                self.schedule(self.now, thread, False)
        elif kind == "release":
            assert self.holder == thread, (self.holder, thread)
            self.holder = None
            if self.queue:
                if self.configuration["barging"]:
                    if not self.wake_scheduled:
                        self.wake_scheduled = True
                        self.schedule(self.now + WAKE, WAKE_EVENT, None)
                else:
                    self.give_the_lock(self.queue.popleft())
            self.schedule(self.now, thread, None)
        else:
            raise AssertionError(kind)

    def wake_the_head_of_the_queue(self):
        self.wake_scheduled = False
        if self.holder is None and self.queue:
            self.give_the_lock(self.queue.popleft())

    def run(self, until):
        while self.heap:
            at, _, thread, value = heapq.heappop(self.heap)
            if at > until:
                break
            self.now = at
            if thread == WAKE_EVENT:
                self.wake_the_head_of_the_queue()
            else:
                self.step(thread, value)


def gpu_batch_starting_now(simulation):
    """这一批 GPU 那一段要多久、出不出错：卡挂住之后起的每一批都等满 HANG 再报错（超时重交用完）。"""
    hang_at = simulation.configuration["hang_at"]
    if hang_at is not None and simulation.now >= hang_at:
        return HANG, True
    return GPU_BATCH, False


def publish_a_user_change(simulation):
    simulation.txg += 1
    simulation.pending = False
    simulation.last_publish_at = simulation.now
    simulation.publishes_since_the_last_user_change = 0


def publish_when_due_once(simulation):
    yield ("acquire",)
    yield ("sleep", HOLD_PUBLISH_WHEN_DUE_CHECK)
    if simulation.pending and simulation.now - simulation.last_publish_at >= T_TIME:
        yield ("sleep", HOLD_PUBLISH)
        publish_a_user_change(simulation)
        simulation.metrics["pending_published_by_the_maintenance_thread"] += 1
    yield ("release",)


def start_or_not(simulation):
    """判起跑（锁里）：交回 True 是这一次调用接着推一趟。"""
    configuration = simulation.configuration
    if simulation.task_stopped:
        simulation.metrics["calls_while_stopped"] += 1
        return False
    if simulation.trip_under_way:
        if simulation.publishes_since_the_last_user_change >= BUDGET and not simulation.exception_open:
            simulation.trip_under_way = False
            return False
        return True
    if simulation.now - simulation.last_timer_start < TIMER_INTERVAL:
        return False
    simulation.last_timer_start = simulation.now
    if configuration["work"] == "nothing" and simulation.memo_txg == simulation.txg:
        return False
    if simulation.publishes_since_the_last_user_change >= BUDGET and not configuration["cannot_write"]:
        return False
    simulation.trip_under_way = True
    simulation.exception_open = configuration["cannot_write"]
    return True


def answer_of_the_batch(simulation, is_exception):
    """一批算完、可以用（today 的每一批，form 里复核过的）：照池子的样子发一次或停。交回这一拍接着叫不接着。"""
    configuration = simulation.configuration
    simulation.consecutive_invalidations = 0
    if configuration["work"] == "nothing":
        simulation.memo_txg = simulation.txg
        simulation.trip_under_way = False
        simulation.metrics["stops_with_nothing_eligible"] += 1
        return True
    yield ("sleep", HOLD_PUBLISH)
    simulation.txg += 1
    simulation.publishes_since_the_last_user_change += 1
    simulation.metrics["background_publishes"] += 1
    if is_exception:
        simulation.exception_open = False
        simulation.metrics["exception_publishes"] += 1
    return True


def stop_after_the_error(simulation):
    simulation.task_stopped = True
    simulation.trip_under_way = False
    simulation.alarm_at = simulation.now
    simulation.metrics["alarms"] += 1


def drain_the_pending_changes(simulation):
    if simulation.pending:
        yield ("sleep", HOLD_PUBLISH)
        publish_a_user_change(simulation)
        simulation.metrics["pending_published_before_the_batch"] += 1


def one_call(simulation):
    """放锁的后台入口一次调用。交回这一拍接着叫不接着（照 lib.rs 一拍里的 keeps_going）。"""
    configuration = simulation.configuration
    yield ("acquire",)
    yield ("sleep", HOLD_TAKING_THE_INPUT)
    simulation.metrics["calls"] += 1
    if not start_or_not(simulation):
        yield ("release",)
        return False
    is_exception = simulation.exception_open and configuration["cannot_write"]
    if configuration["arm"] == "today":
        # 今天的代码：排空待发在判起跑之后、问调用方在推一趟的循环里（排空之后）
        yield from drain_the_pending_changes(simulation)
        wants_to_yield = simulation.fsync_waiters > 0
        may_yield = True
    else:
        wants_to_yield = simulation.lock_waiters > 0 or configuration["constant_waiters"]
        may_yield = configuration["n_yield"] is None or simulation.consecutive_yields < configuration["n_yield"]
    if wants_to_yield and may_yield and not is_exception:
        simulation.consecutive_yields += 1
        simulation.metrics["yields"] += 1
        if configuration["yield_resets_k"]:
            simulation.consecutive_invalidations = 0
        if configuration["cannot_write"]:
            simulation.trip_under_way = False
        yield ("release",)
        return False
    simulation.consecutive_yields = 0
    if configuration["arm"] == "form":
        # 第三节第 2 条：先问让路，再排空待发
        yield from drain_the_pending_changes(simulation)
    if configuration["arm"] == "today":
        in_the_lock = True
    else:
        floor = configuration["k"]
        in_the_lock = is_exception or (floor is not None and simulation.consecutive_invalidations >= floor)
    duration, failed = gpu_batch_starting_now(simulation)
    simulation.metrics["gpu_batches"] += 1
    if in_the_lock:
        simulation.metrics["batches_in_the_lock"] += 1
        if is_exception:
            simulation.metrics["exception_batches_in_the_lock"] += 1
        if failed:
            simulation.metrics["hung_batches_in_the_lock"] += 1
        yield ("sleep", duration)
        if failed:
            stop_after_the_error(simulation)
            yield ("release",)
            return False
        keeps_going = yield from answer_of_the_batch(simulation, is_exception)
        yield ("release",)
        return keeps_going
    stamp = simulation.txg
    simulation.metrics["batches_outside_the_lock"] += 1
    if failed:
        simulation.metrics["hung_batches_outside_the_lock"] += 1
    yield ("release",)
    yield ("sleep", duration)
    yield ("acquire",)
    yield ("sleep", HOLD_RECHECK)
    is_valid = simulation.txg == stamp and not simulation.pending
    if failed and (configuration["error_first"] or is_valid):
        stop_after_the_error(simulation)
        yield ("release",)
        return False
    if not is_valid:
        simulation.metrics["invalidations"] += 1
        if failed:
            simulation.metrics["errors_swallowed_by_an_invalidation"] += 1
        simulation.consecutive_invalidations += 1
        simulation.maxima["max_consecutive_invalidations"] = max(
            simulation.maxima["max_consecutive_invalidations"], simulation.consecutive_invalidations)
        yield ("release",)
        return configuration["continue_after_invalidation"]
    keeps_going = yield from answer_of_the_batch(simulation, is_exception)
    yield ("release",)
    return keeps_going


def one_pass(simulation):
    simulation.metrics["passes"] += 1
    yield from publish_when_due_once(simulation)
    calls = 0
    while calls < CALLS_PER_PASS:
        keeps_going = yield from one_call(simulation)
        calls += 1
        if not keeps_going:
            break
        yield from publish_when_due_once(simulation)
    simulation.maxima["max_calls_in_a_pass"] = max(simulation.maxima["max_calls_in_a_pass"], calls)
    if calls == CALLS_PER_PASS:
        simulation.metrics["passes_reaching_the_call_limit"] += 1
    return simulation.trip_under_way


def maintenance_thread(simulation):
    next_pass_at = 0.0
    while True:
        started = simulation.now
        if started >= next_pass_at:
            under_way = yield from one_pass(simulation)
            next_pass_at = started + (POLL if under_way else PERIOD)
        else:
            yield from publish_when_due_once(simulation)
        yield ("sleep", max(min(POLL, next_pass_at - simulation.now), 0.001))


def foreground_thread(simulation, name, kind, per_second):
    """一条前台线程：间隔按指数分布到达；kind 是 fsync（发布）、accept（只收下）、read（只读）。"""
    configuration = simulation.configuration
    hold = HOLD_READ if kind == "read" else HOLD_FOREGROUND_PUBLISH
    while True:
        yield ("sleep", simulation.random.expovariate(per_second) * 1000.0)
        arrival = simulation.now
        registered_as_fsync = False
        if configuration["arm"] == "today":
            if kind == "fsync":
                simulation.fsync_waiters += 1
                registered_as_fsync = True
            if not simulation.lock_is_takeable():
                simulation.waiting_since[name] = arrival
                simulation.overtakes[name] = 0
            yield ("acquire",)
        else:
            got = yield ("try_acquire",)
            if not got:
                simulation.waiting_since[name] = arrival
                simulation.overtakes[name] = 0
                if configuration["registration_delay"] > 0:
                    yield ("sleep", configuration["registration_delay"])
                simulation.lock_waiters += 1
                yield ("acquire",)
                simulation.lock_waiters -= 1
        if name in simulation.waiting_since:
            simulation.note_the_holder(name)
        waited = simulation.now - arrival
        key = kind + "_wait_max_ms"
        simulation.maxima[key] = max(simulation.maxima[key], waited)
        if waited > 1000.0:
            simulation.metrics[kind + "_waits_over_a_second"] += 1
        yield ("sleep", hold)
        if kind == "fsync":
            if configuration["cannot_write"]:
                # 前台写不进：被空间拒、当场推一串抬 F 的空发布（txg 动，不算用户改动）
                simulation.txg += 1
            else:
                publish_a_user_change(simulation)
            simulation.metrics["foreground_publishes"] += 1
        elif kind == "accept":
            simulation.pending = True
            if simulation.now - simulation.last_publish_at >= T_TIME:
                yield ("sleep", HOLD_FOREGROUND_PUBLISH)
                publish_a_user_change(simulation)
        simulation.metrics[kind + "_operations"] += 1
        yield ("release",)
        if registered_as_fsync:
            simulation.fsync_waiters -= 1


def run_one(configuration, seed):
    simulation = Simulation(configuration, seed)
    simulation.spawn(MAINTENANCE, maintenance_thread(simulation))
    for kind in ("fsync", "accept", "read"):
        per_second = configuration[kind + "_per_second"]
        if per_second > 0:
            name = "foreground-" + kind
            simulation.spawn(name, foreground_thread(simulation, name, kind, per_second))
    simulation.run(configuration["duration"])
    hang_at = configuration["hang_at"]
    alarm_after = "none" if simulation.alarm_at is None or hang_at is None else round((simulation.alarm_at - hang_at) / 1000.0, 1)
    fields = [("seed", seed)]
    for key in ("arm", "k", "n_yield", "error_first", "yield_resets_k", "continue_after_invalidation", "barging",
                "registration_delay", "constant_waiters", "fsync_per_second", "accept_per_second", "read_per_second",
                "work", "cannot_write", "hang_at", "duration"):
        fields.append((key, configuration[key]))
    for key in ("background_publishes", "exception_publishes", "gpu_batches", "batches_outside_the_lock", "batches_in_the_lock",
                "exception_batches_in_the_lock", "invalidations", "errors_swallowed_by_an_invalidation",
                "hung_batches_outside_the_lock", "hung_batches_in_the_lock", "alarms", "yields", "calls", "passes",
                "passes_reaching_the_call_limit", "stops_with_nothing_eligible", "foreground_publishes",
                "read_waits_over_a_second", "fsync_waits_over_a_second", "accept_waits_over_a_second"):
        fields.append((key, simulation.metrics[key]))
    fields.append(("alarm_after_the_hang_s", alarm_after))
    for key in ("max_consecutive_invalidations", "max_calls_in_a_pass", "max_overtakes"):
        fields.append((key, int(simulation.maxima[key])))
    for key in ("read_wait_max_ms", "fsync_wait_max_ms", "accept_wait_max_ms"):
        fields.append((key, round(simulation.maxima[key], 1)))
    return dict(fields)


def configuration_of(**overrides):
    configuration = {
        "arm": "form", "k": None, "n_yield": None, "error_first": True, "yield_resets_k": False,
        "continue_after_invalidation": True, "barging": True, "registration_delay": 0.0, "constant_waiters": False,
        "fsync_per_second": 0.0, "accept_per_second": 0.0, "read_per_second": 0.0, "work": "compactable",
        "cannot_write": False, "hang_at": None, "duration": 120000.0,
    }
    for key, value in overrides.items():
        assert key in configuration, key
        configuration[key] = value
    return configuration


def scenarios(quick):
    """交回 [(场景名, 配置, 种子)]。场景名就是产物文件名。"""
    seeds_of_the_hang = [1] if quick else [1, 2, 3, 4, 5]
    shrink = 5.0 if quick else 1.0
    runs = []
    # 下限把卡挂住的那一批带回锁里（Q1）：60 秒时卡挂住，之后起的每一批等满 40 秒报错
    for fsync in (1.0, 10.0, 100.0):
        for seed in seeds_of_the_hang:
            runs.append(("hang_brought_back_into_the_lock", configuration_of(
                arm="today", fsync_per_second=fsync, read_per_second=20.0, hang_at=60000.0, duration=600000.0 / shrink), seed))
            for k in (1, 2, 4, 8, None):
                for error_first in (True, False):
                    for keep_calling in (True, False):
                        runs.append(("hang_brought_back_into_the_lock", configuration_of(
                            k=k, error_first=error_first, continue_after_invalidation=keep_calling, fsync_per_second=fsync,
                            read_per_second=20.0, hang_at=60000.0, duration=600000.0 / shrink), seed))
    for k in (2, 8, None):
        for error_first in (True, False):
            runs.append(("hang_with_accepts_only", configuration_of(
                k=k, error_first=error_first, accept_per_second=1000.0, read_per_second=20.0, hang_at=30000.0,
                duration=300000.0 / shrink), 1))
    runs.append(("hang_with_accepts_only", configuration_of(
        arm="today", accept_per_second=1000.0, read_per_second=20.0, hang_at=30000.0, duration=300000.0 / shrink), 1))
    # 破例批恒在锁里算（Q1）：前台写不进（每次写被拒推一次抬 F），读照常
    for fsync in (1.0, 10.0):
        for seed in seeds_of_the_hang[:3]:
            runs.append(("exception_batch_hang", configuration_of(
                arm="today", cannot_write=True, fsync_per_second=fsync, read_per_second=20.0, hang_at=60000.0,
                duration=300000.0 / shrink), seed))
            for k in (4, None):
                runs.append(("exception_batch_hang", configuration_of(
                    k=k, cannot_write=True, fsync_per_second=fsync, read_per_second=20.0, hang_at=60000.0,
                    duration=300000.0 / shrink), seed))
    # 让路与两个下限叠在一起（Q3）：锁上一直有前台在等（读的人多）
    for fsync in (10.0, 100.0, 1000.0):
        runs.append(("yield_and_floors", configuration_of(
            arm="today", constant_waiters=True, fsync_per_second=fsync, read_per_second=20.0, duration=120000.0 / shrink), 1))
        for k in (1, 2, 4, None):
            for n_yield in (1, 4):
                for resets in (True, False):
                    runs.append(("yield_and_floors", configuration_of(
                        k=k, n_yield=n_yield, yield_resets_k=resets, constant_waiters=True, fsync_per_second=fsync,
                        read_per_second=20.0, duration=120000.0 / shrink), 1))
    # 一拍空转（Q2，C1008 的同形）：池子过了水位却没有可选的段，前台忙
    for workload in ({"fsync_per_second": 100.0}, {"fsync_per_second": 1000.0}, {"accept_per_second": 1000.0}):
        runs.append(("idle_pool_spin", configuration_of(arm="today", work="nothing", duration=120000.0 / shrink, **workload), 1))
        for k in (2, 8, None):
            for keep_calling in (True, False):
                runs.append(("idle_pool_spin", configuration_of(
                    k=k, continue_after_invalidation=keep_calling, work="nothing", duration=120000.0 / shrink, **workload), 1))
    # try_lock 失败才登记的那个窗（Q3）
    for barging in (True, False):
        runs.append(("registration_window", configuration_of(
            arm="today", barging=barging, fsync_per_second=100.0, read_per_second=100.0, duration=60000.0 / shrink), 1))
        for delay in (0.0, 0.5, 5.0):
            runs.append(("registration_window", configuration_of(
                barging=barging, registration_delay=delay, fsync_per_second=100.0, read_per_second=100.0,
                duration=60000.0 / shrink), 1))
    # 判法自证：前台一个改动都不发时一批都不作废；只收下不发时照样作废（「待发为空」那一判起作用）
    runs.append(("self_check_no_change", configuration_of(read_per_second=20.0, duration=60000.0 / shrink), 1))
    return runs


def operations_of(configuration):
    seconds = configuration["duration"] / 1000.0
    return seconds * (configuration["fsync_per_second"] + configuration["accept_per_second"] + configuration["read_per_second"]) + seconds


def line_of(name, result):
    return "RESULT name=" + name + " " + " ".join(key + "=" + str(value) for key, value in result.items())


CONFIGURATION_KEYS = ("arm", "k", "n_yield", "error_first", "yield_resets_k", "continue_after_invalidation", "barging",
                      "registration_delay", "constant_waiters", "fsync_per_second", "accept_per_second", "read_per_second",
                      "work", "cannot_write", "hang_at", "duration")
SUMMED_OVER_SEEDS = ("background_publishes", "invalidations", "batches_in_the_lock", "hung_batches_in_the_lock",
                     "hung_batches_outside_the_lock", "errors_swallowed_by_an_invalidation", "gpu_batches", "yields",
                     "max_consecutive_invalidations", "max_calls_in_a_pass", "passes_reaching_the_call_limit",
                     "read_wait_max_ms", "fsync_wait_max_ms", "accept_wait_max_ms", "max_overtakes", "read_waits_over_a_second")


def summary_lines(name, results):
    groups = collections.OrderedDict()
    for result in results:
        groups.setdefault(tuple(result[key] for key in CONFIGURATION_KEYS), []).append(result)
    lines = []
    for key, members in groups.items():
        fields = ["SUMMARY name=" + name] + [k + "=" + str(v) for k, v in zip(CONFIGURATION_KEYS, key)]
        fields.append("seeds=" + str(len(members)))
        for field in SUMMED_OVER_SEEDS:
            values = [member[field] for member in members]
            fields.append(field + "_min=" + str(min(values)))
            fields.append(field + "_max=" + str(max(values)))
        alarms = [member["alarm_after_the_hang_s"] for member in members]
        numeric = [value for value in alarms if value != "none"]
        fields.append("alarm_after_the_hang_s_max=" + (str(max(numeric)) if numeric else "none"))
        fields.append("runs_without_an_alarm=" + str(len(alarms) - len(numeric)))
        lines.append(" ".join(fields))
    return lines


def self_checks(results_by_name, quick):
    failures = []
    checked = 0
    for result in results_by_name.get("self_check_no_change", []):
        checked += 1
        if result["invalidations"] != 0 or result["background_publishes"] == 0:
            failures.append("前台一个改动都不发时该一批都不作废、后台发得出：" + line_of("self_check_no_change", result))
    for result in results_by_name.get("hang_with_accepts_only", []):
        if result["arm"] == "form":
            checked += 1
            if result["invalidations"] == 0:
                failures.append("只收下不发时「待发为空」那一判该作废批：" + line_of("hang_with_accepts_only", result))
    for result in results_by_name.get("idle_pool_spin", []):
        if result["arm"] == "today":
            checked += 1
            if result["max_calls_in_a_pass"] > 2:
                failures.append("today 臂没有可选的段时一拍该至多叫 2 次（停了、下一次定时兜底间隔没到）：" + line_of("idle_pool_spin", result))
    if not quick:
        for result in results_by_name.get("hang_brought_back_into_the_lock", []):
            if result["arm"] == "today":
                checked += 1
                if result["read_wait_max_ms"] < 0.9 * HANG:
                    failures.append("today 臂卡挂住的那一批在锁里，读该等满一段：" + line_of("hang_brought_back_into_the_lock", result))
            elif result["k"] is None and result["error_first"]:
                checked += 1
                if result["read_wait_max_ms"] >= 1000.0:
                    failures.append("没有下限、出错先判时挂住的那一批在锁外，读不该等过 1 秒：" + line_of("hang_brought_back_into_the_lock", result))
    return checked, failures


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True)
    parser.add_argument("--quick", action="store_true")
    arguments = parser.parse_args()
    os.makedirs(arguments.out, exist_ok=True)
    runs = scenarios(arguments.quick)
    probe = configuration_of(accept_per_second=1000.0, read_per_second=20.0, duration=10000.0)
    started = time.monotonic()
    run_one(probe, 99)
    seconds_per_operation = (time.monotonic() - started) / operations_of(probe)
    estimate = seconds_per_operation * sum(operations_of(configuration) for _, configuration, _ in runs)
    print("ESTIMATE name=form_model runs=%d seconds_per_operation=%.2e estimated_seconds=%.0f" % (
        len(runs), seconds_per_operation, estimate), flush=True)
    if estimate > 2400.0:
        print("  ✗ 估时超过 40 分钟，没跑全量：缩场景再交", flush=True)
        return 3
    results_by_name = collections.OrderedDict()
    wall_started = time.monotonic()
    for name, configuration, seed in runs:
        results_by_name.setdefault(name, []).append(run_one(configuration, seed))
    for name, results in results_by_name.items():
        with open(os.path.join(arguments.out, name + ".out"), "w", encoding="utf-8") as output:
            for result in results:
                output.write(line_of(name, result) + "\n")
            for line in summary_lines(name, results):
                output.write(line + "\n")
        for line in summary_lines(name, results):
            print(line)
    print("WALL_SECONDS name=form_model seconds=%.0f" % (time.monotonic() - wall_started))
    checked, failures = self_checks(results_by_name, arguments.quick)
    for failure in failures:
        print("  ✗ 自检：" + failure)
    print("name=self_checks checked=%d failed=%d" % (checked, len(failures)))
    return 1 if failures or checked == 0 else 0


if __name__ == "__main__":
    sys.exit(main())
