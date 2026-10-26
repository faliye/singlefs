#!/usr/bin/env python3
"""m4-publish-trigger-r1 正推腿的小模型：触发判据、空闲丢失窗口、脏字节读法、RefFS 允许集合的口径。
只用标准库，种子写死，输出逐行 name=... 便于比对。
模型不读 crates/ 的代码：数据单元字节、日志记录字节、F 等常量由命令行参数给（rerun.sh 从冻结副本 grep 出来传入）。
用法：trigger_model.py <子命令> [--data-unit-bytes N --payload-bytes N --record-bytes N --factor N --ring-bytes N]
子命令：window | dirty | firstcross | refs-cases <输出文件> | refs-judge <用例文件>
"""
import os
import random
import sys
from dataclasses import dataclass, field

ARGS = {"data_unit_bytes": 32768, "payload_bytes": 32768 - 134, "record_bytes": 4096, "factor": 3,
        "ring_bytes": 768 * 1024 * 1024}


def parse_flags(argv):
    rest, i = [], 0
    while i < len(argv):
        a = argv[i]
        if a.startswith("--") and a[2:].replace("-", "_") in ARGS:
            ARGS[a[2:].replace("-", "_")] = int(argv[i + 1])
            i += 2
        else:
            rest.append(a)
            i += 1
    return rest


# ---------------------------------------------------------------- 会话模型
@dataclass
class Change:
    kind: str            # "write" 整份写某个文件 | "new" 建 count 个空 inode
    inode: int = 0
    nbytes: int = 0
    count: int = 0
    at: int = 0          # 调用方钟（毫秒）


@dataclass
class Session:
    t_time: int
    t_dirty: int
    reading: str = "units"           # content | folded_content | units
    check_on_accept: bool = True
    guard_nonempty: bool = True
    baseline: str = "first_reading"  # first_reading | mount
    pending: list = field(default_factory=list)
    last: object = None
    publishes: list = field(default_factory=list)   # (时刻, 原因, 含几个改动)

    def dirty(self):
        if self.reading == "content":
            return sum(c.nbytes for c in self.pending if c.kind == "write")
        latest = {}
        for c in self.pending:
            if c.kind == "write":
                latest[c.inode] = c.nbytes          # 同一 inode 整份写，后到的盖掉前面的（D8 已定项 13 的折叠）
        if self.reading == "folded_content":
            return sum(latest.values())
        units = sum(-(-n // ARGS["payload_bytes"]) if n else 0 for n in latest.values())
        return units * ARGS["data_unit_bytes"]

    def see(self, now):
        if self.last is None:
            self.last = now

    def check(self, now):
        self.see(now)
        if self.guard_nonempty and not self.pending:
            return None
        by_time = now - self.last >= self.t_time if now >= self.last else False
        by_dirty = self.dirty() >= self.t_dirty
        if by_time or by_dirty:
            cause = "both" if (by_time and by_dirty) else ("time" if by_time else "dirty")
            self.publishes.append((now, cause, len(self.pending)))
            self.pending = []
            self.last = now
            return cause
        return None

    def accept(self, change, now):
        self.see(now)
        self.pending.append(change)
        return self.check(now) if self.check_on_accept else None

    def poll(self, now):
        return self.check(now)

    def flush(self, now):
        self.see(now)
        if self.pending:
            self.publishes.append((now, "flush", len(self.pending)))
            self.pending = []
            self.last = now


# ---------------------------------------------------------------- P1 丢失窗口
def cmd_window():
    T, P = 5000, 1000
    # (a) 只在请求进门时判：一次改动之后再没有请求
    s = Session(T, 1 << 40, check_on_accept=True)
    s.accept(Change("write", 1, 100), 0)
    s.accept(Change("write", 1, 100), 1)
    for clock in (T - 1, T, 10 * T, 1000 * T):
        pass
    print("name=window_entry_only pending_after_idle_forever=%d publishes=%d (无请求进门，条件永远没人判)" % (len(s.pending), len(s.publishes)))
    # (b) 调用方每 P 毫秒调一次 poll，钟由调用方给
    rng = random.Random(20261026)
    worst = 0
    for trial in range(2000):
        s = Session(T, 1 << 40)
        t, ages, oldest = 0, [], None
        accepted_at = []
        events = sorted(rng.sample(range(0, 60000), 12))
        polls, p_at = [], rng.randrange(1, P + 1)
        while p_at < 60000 + 3 * T:          # 相邻两次 poll 的间隔不超过 P（调用方保证的周期上界），间隔本身随机
            polls.append(p_at)
            p_at += rng.randrange(1, P + 1)
        timeline = sorted([(e, "a") for e in events] + [(p, "p") for p in polls])
        for when, kind in timeline:
            if kind == "a":
                if not s.pending:
                    oldest = when
                cause = s.accept(Change("write", 1, 100), when)
            else:
                cause = s.poll(when)
            if cause:
                oldest = None
            if s.pending and oldest is not None:
                worst = max(worst, when - oldest)
        assert not s.pending
    print("name=window_with_poll period_ms=%d t_time_ms=%d worst_pending_age_ms_seen=%d bound_t_time_plus_period=%d ok=%s"
          % (P, T, worst, T + P, worst <= T + P))
    # (b2) 最坏排布：上一次发布恰在一次 poll 上，紧接着进来一条改动，T_time 比 poll 周期的整倍数多 1 毫秒
    T2, P2 = 4001, 2000
    s = Session(T2, 1 << 40)
    s.accept(Change("write", 1, 100), 0)
    s.poll(0)                                   # 第一次读数当基线，publish 发生在下一个够格的 poll
    s.pending = []                              # 清掉，模拟「上一次发布恰在 t=0」
    s.last = 0
    s.accept(Change("write", 1, 100), 1)
    trig = None
    for now in range(P2, 10 * P2, P2):
        if s.poll(now):
            trig = now
            break
    print("name=window_worst_layout t_time_ms=%d period_ms=%d accepted_at_ms=1 published_at_ms=%s age_ms=%s bound_t_time_plus_period=%d" % (T2, P2, trig, trig - 1, T2 + P2))
    # (c) 空闲且没有待发改动：有没有守门
    for guard in (True, False):
        s = Session(T, 1 << 40, guard_nonempty=guard)
        for now in range(0, 100 * T, 500):
            s.poll(now)
        print("name=idle_publishes guard_nonempty=%s publishes_in_100_t_time=%d" % (guard, len(s.publishes)))
    # (d) 基线：第一次读数当上次发布时刻 vs 以挂载读数为基线
    for base_mount in (False, True):
        s = Session(T, 1 << 40)
        if base_mount:
            s.last = 0
        c = s.accept(Change("write", 1, 100), 3 * T)
        print("name=baseline mount_reading_given=%s first_accept_at_ms=%d triggered_at_accept=%s"
              % (base_mount, 3 * T, c))


# ---------------------------------------------------------------- P2 脏字节读法
def cmd_dirty():
    du, pay = ARGS["data_unit_bytes"], ARGS["payload_bytes"]
    ring, f = ARGS["ring_bytes"], ARGS["factor"]
    field_value = 2 << 30
    eff = min(field_value, ring // f)
    print("name=effective_t_dirty field_bytes=%d ring_bytes=%d factor=%d ring_div_factor=%d effective_bytes=%d" % (field_value, ring, f, ring // f, eff))
    units = eff // du
    print("name=effective_in_data_units data_unit_bytes=%d units=%d journal_bytes_of_those_units=%d in_flight_record_limit=%d"
          % (du, units, units * ARGS["record_bytes"], ring // ARGS["record_bytes"] // f))
    print("name=field_2gib_in_data_units units=%d journal_bytes=%d" % (field_value // du, field_value // du * ARGS["record_bytes"]))
    loads = {
        "same_file_1KiB_x1000": [Change("write", 1, 1024) for _ in range(1000)],
        "1000_files_1KiB": [Change("write", i, 1024) for i in range(1000)],
        "one_file_70000B_x3": [Change("write", 1, 70000) for _ in range(3)],
        "100000_empty_inodes": [Change("new", count=1) for _ in range(100000)],
    }
    for name, load in loads.items():
        row = []
        for reading in ("content", "folded_content", "units"):
            s = Session(1 << 40, 1 << 62, reading=reading)
            s.pending = list(load)
            row.append("%s=%d" % (reading, s.dirty()))
        print("name=dirty_reading load=%s %s" % (name, " ".join(row)))
    # 测试里能不能把脏量那一支单独跨过：阈值压到 2 个数据单元，时间阈值给足
    for reading in ("content", "folded_content", "units"):
        s = Session(1 << 40, 2 * du, reading=reading)
        cause = None
        n = 0
        while cause is None and n < 10000:
            n += 1
            cause = s.accept(Change("write", 1, 1024), 0)   # 反复整份覆盖同一个文件
        print("name=dirty_branch_reachable reading=%s threshold_bytes=%d accepts_until_cross=%s cause=%s" % (reading, 2 * du, n if cause else "never(10000)", cause))


# ---------------------------------------------------------------- 先到哪个触发
def cmd_firstcross():
    T, D = 1000, 4 * ARGS["data_unit_bytes"]
    # 时间先到：脏量始终在阈值下
    s = Session(T, D)
    s.accept(Change("write", 1, 10), 0)
    r1 = [s.poll(T - 1), s.poll(T)]
    print("name=time_first poll_at_T_minus_1=%s poll_at_T=%s dirty_bytes_at_cross=%d" % (r1[0], r1[1], 0))
    # 脏量先到：钟不动
    s = Session(T, D)
    got = None
    for k in range(1, 10):
        got = s.accept(Change("write", k, 10), 5)
        if got:
            print("name=dirty_first accepts=%d cause=%s clock_ms=5" % (k, got))
            break
    # 同时到达
    s = Session(T, ARGS["data_unit_bytes"])
    s.accept(Change("write", 1, 10), 0)
    print("name=both_at_once cause=%s" % s.accept(Change("write", 2, 10), T))
    # 钟往回走：按没过间隔算
    s = Session(T, 1 << 40)
    s.accept(Change("write", 1, 10), 5000)
    print("name=clock_backwards poll_at_100=%s" % s.poll(100))


# ---------------------------------------------------------------- RefFS 口径
# 时间线事件：("O",k) 接受第 k 号内容的整份覆盖；("F",) 调用方显式发布；("A",) 自动触发发布（调用方没点名，会话在调用里自己发的）
def gen_timeline(rng, n):
    ev, k = [], 0
    for _ in range(n):
        r = rng.random()
        if r < 0.55:
            k += 1
            ev.append(("O", k))
        elif r < 0.7:
            ev.append(("F",))
        else:
            ev.append(("A",))
    return ev


def truth(ev, crash):
    """真值：发布返回即落盘。crash = ("after", i) 前 i 个事件都返回之后崩；("inside", i) 崩在第 i 个事件（F 或 A）的发布里。"""
    kind, i = crash
    state = durable = 0
    for e in ev[:i]:
        if e[0] == "O":
            state = e[1]
        else:
            durable = state
    return {durable} if kind == "after" else {durable, state}


def refs_judge(ops, crash, recovered):
    """RefFS 的允许集合（lib.rs allowed_set_for_a_crash_in 的 Python 口径）。ops：("O",k) | ("P",)；
    crash = ("after",) 崩在全部返回之后（hi = 长度 + 1）| ("inside", j) 崩在第 j 个（0 起）操作里（必须是 P）。
    lo = 崩溃前最后一个返回的 P 的末尾；Q = {S_lo} ∪ {lo 之后每个 P（含崩溃所在那个）的末尾}。"""
    def state_after(m):
        s = 0
        for o in ops[:m]:
            if o[0] == "O":
                s = o[1]
        return s
    hi = len(ops) + 1 if crash[0] == "after" else crash[1] + 1
    lo = 0
    for p, o in enumerate(ops[:hi - 1]):
        if o[0] == "P" and os.environ.get("TRIGGER_MODEL_BREAK") != "publish-does-not-raise-lo":   # 弄坏开关：只给自证用
            lo = p + 1
    allowed = {state_after(lo)}
    for p, o in enumerate(ops[:hi]):
        if p + 1 > lo and o[0] == "P":
            allowed.add(state_after(p + 1))
    return recovered in allowed


def fmt_ops(ops):
    return " ".join(o[0] + (str(o[1]) if o[0] == "O" else "") for o in ops)


def fmt_crash(c):
    return c[0] + (str(c[1]) if len(c) > 1 else "")


def parse_crash(text):
    return ("after",) if text == "after" else ("inside", int(text[len("inside"):]))


def parse_ops(text):
    return [("P",) if t == "P" else ("O", int(t[1:])) for t in text.split()]


def views(ev, crash):
    """同一个时间线的两个视角：full = 自动发布也进 RefFS 的序列（本方案）；explicit = 只有调用方点名的发布（RefFS 今天的口径）。"""
    full = [("O", e[1]) if e[0] == "O" else ("P",) for e in ev]
    explicit = [("O", e[1]) if e[0] == "O" else ("P",) for e in ev if e[0] != "A"]
    if crash[0] == "after":
        return full, ("after",), explicit, ("after",)
    idx = crash[1]
    c_full = ("inside", idx)
    if ev[idx][0] == "A":     # 崩在调用方没点名的发布里：explicit 视角只看得到它之前返回的操作
        trimmed = [("O", e[1]) if e[0] == "O" else ("P",) for e in ev[:idx] if e[0] != "A"]
        return full, c_full, trimmed, ("after",)
    return full, c_full, explicit, ("inside", sum(1 for e in ev[:idx] if e[0] != "A"))


def cmd_refs_cases(out_path):
    rng = random.Random(20261026)
    lines = []
    stats = dict(cases=0, truth_legal=0, cur_false_red=0, cur_missed_red=0, auto_false_red=0, auto_missed_red=0)
    for _ in range(3000):
        ev = gen_timeline(rng, rng.randrange(1, 9))
        crashes = [("after", len(ev))] + [("inside", i) for i, e in enumerate(ev) if e[0] in ("F", "A")]
        for crash in crashes:
            legal = truth(ev, crash)
            full, c_full, exp, c_exp = views(ev, crash)
            maxk = max([e[1] for e in ev if e[0] == "O"] + [0])
            for rec in range(maxk + 1):
                ok = rec in legal
                v_full, v_cur = refs_judge(full, c_full, rec), refs_judge(exp, c_exp, rec)
                stats["cases"] += 1
                stats["truth_legal"] += int(ok)
                stats["cur_false_red"] += int(ok and not v_cur)
                stats["cur_missed_red"] += int((not ok) and v_cur)
                stats["auto_false_red"] += int(ok and not v_full)
                stats["auto_missed_red"] += int((not ok) and v_full)
                lines.append("|".join([fmt_ops(full), fmt_crash(c_full), fmt_ops(exp), fmt_crash(c_exp), str(rec),
                                       "legal" if ok else "illegal"]))
    with open(out_path, "x") as f:
        f.write("\n".join(lines) + "\n")
    print("name=refs_cases " + " ".join("%s=%d" % kv for kv in stats.items()))


def cmd_refs_py_verdicts(path):
    """Python 口径对每条用例的判，两个视角各一列：full cur"""
    for line in open(path):
        full, c_full, exp, c_exp, rec, _truth = line.rstrip("\n").split("|")
        rec = int(rec)
        print("%d %d" % (int(refs_judge(parse_ops(full), parse_crash(c_full), rec)),
                         int(refs_judge(parse_ops(exp), parse_crash(c_exp), rec))))


if __name__ == "__main__":
    rest = parse_flags(sys.argv[1:])
    cmd = rest[0]
    if cmd == "window":
        cmd_window()
    elif cmd == "dirty":
        cmd_dirty()
    elif cmd == "firstcross":
        cmd_firstcross()
    elif cmd == "refs-cases":
        cmd_refs_cases(rest[1])
    elif cmd == "refs-py-verdicts":
        cmd_refs_py_verdicts(rest[1])
    else:
        sys.exit("子命令：window | dirty | firstcross | refs-cases <输出> | refs-py-verdicts <用例>")
