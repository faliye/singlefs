#!/usr/bin/env python3
"""Z3 两道闸三种读法（串 / 合 / 串-重判）的条款层计数模型：纯算术、无随机源，跑 N 遍与 1 遍给的信息一样多。

只读导入的 crates 事实（草稿目录副本上现读，见报告第零节）：一次空发布写 4 个固定点单元并放掉上一版的 4 个
（`transaction.rs` 的 `rewritten_roles`）；可写挂载的写行发布另重写实例表单元（2 槽）；根环 3 区 × 8 槽、区域归属盘 0 / 1 / 0
（`MakeFilesystemParameters` 的 `region_devices`）；暖机推到本实例的根覆盖两块盘为止（`mount.rs` 的 `establish_instance`）。
条款取值（kb 现查）：c_max = 4、切换预留每块盘 56 块、checkpoint 保留池 4 块（D28 已定项 3 / 4 第一个事务几何）；
D16 已定项 1 的保留池 10 + 7 c_max、残留 5 + 6 c_max、B = 8、滞后量定义、抬 F 上限、生效定义。

建模选择（每一条都是这条腿自己取的读法，报告第三节逐条写明）：
- 两块盘互为镜像，统计量按「每块盘」一个数（C370 没定单位，取逐盘）；需求 = 对象的块数（含它自己的元数据），固定点走保留池。
- 回收时点取甲-T1 / 乙′ 的形态：分配之前按「此刻盘上的环与 F_生效」回收。
- 「活元数据」取 0（C371 没定义）。
- 被抛弃根独占量用 `isolate(n)` 这一步直接给数：回退之后的隔离槽数取副本装置上里程碑脚本量到的 36（`p-s3-crash-between.txt` 的 mem_isolated=36）。
"""
import sys

C_MAX = 4
D16_RESERVE = 10 + 7 * C_MAX      # 38
D16_RESIDUAL = 5 + 6 * C_MAX      # 29
B_PUBLISHES = 8
SWITCH_RESERVE = 56
CKPT_POOL = 4
FIXED_POINTS = 4
RING_SLOTS = 24
REGION_DEVICE = (0, 1, 0)


class Block:
    __slots__ = ("alloc", "release", "reclaimed", "by_empty")

    def __init__(self, alloc):
        self.alloc = alloc
        self.release = None
        self.reclaimed = False
        self.by_empty = False


class Pool:
    def __init__(self, capacity, arm, reading, lag_counts_empty=False, align_empty=False):
        self.align_empty = align_empty          # 判别力自证用：合的第一道闸把「空发布放掉、抬 F 放不回来」的也算可用（与 D16 的 df 同一口径）
        self.capacity = capacity
        self.arm = arm              # "乙′" 或 "甲"（式子里「已分配」含不含 defer）
        self.reading = reading      # "串" / "合" / "串-重判"
        self.lag_counts_empty = lag_counts_empty   # 判别力自证用：滞后量把空发布放掉的也算进去
        self.blocks = []
        self.isolated = 0
        self.txg = 0
        self.F = 0
        self.ring = {}              # txg % 24 → (txg, F, 非空)
        self.fixed = self.allocate_blocks(FIXED_POINTS)
        self.instance_table = self.allocate_blocks(2)
        self.ring[0] = (0, 0, False)
        self.objects = {}
        self.rows = []

    # ---- 盘上的环 ----
    def roots(self):
        return list(self.ring.values())

    def effective_floor(self):
        highest = {}
        for txg, floor, _ in self.roots():
            device = REGION_DEVICE[txg % 3]
            highest[device] = max(highest.get(device, floor), floor)
        return min(highest.values()) if highest else 0

    def oldest_valid(self):
        return min(txg for txg, _, _ in self.roots())

    def reclaim_floor(self):
        return max(self.effective_floor(), self.oldest_valid())

    def ceiling(self):
        valid = [r for r in self.roots() if r[0] >= self.F]
        newest_per_device = {}
        for txg, _, _ in valid:
            device = REGION_DEVICE[txg % 3]
            newest_per_device[device] = max(newest_per_device.get(device, txg), txg)
        non_empty = sorted((txg for txg, _, ne in valid if ne), reverse=True)
        fourth = non_empty[3] if len(non_empty) >= 4 else min(txg for txg, _, _ in valid)
        return min(min(newest_per_device.values()), fourth)

    def fourth_newest_non_empty(self):
        non_empty = sorted((txg for txg, _, ne in self.roots() if ne and txg >= self.F), reverse=True)
        return non_empty[3] if len(non_empty) >= 4 else None

    # ---- 统计量 ----
    def still(self):
        return sum(1 for b in self.blocks if b.release is None)

    def defer(self):
        return sum(1 for b in self.blocks if b.release is not None and not b.reclaimed)

    def allocatable(self):
        return self.capacity - self.still() - self.defer() - self.isolated

    def restorable(self):
        floor = max(self.ceiling(), self.oldest_valid())
        return sum(1 for b in self.blocks if b.release is not None and not b.reclaimed and b.release <= floor)

    def lag(self):
        floor = max(self.fourth_newest_non_empty() or -1, self.oldest_valid())
        return sum(1 for b in self.blocks if b.release is not None and not b.reclaimed and b.release > floor
                   and (self.lag_counts_empty or not b.by_empty))

    def available_d28(self, defer_term):
        allocated = self.still() + (self.defer() if self.arm == "甲" else 0)
        return self.capacity - allocated - defer_term - self.isolated - SWITCH_RESERVE - CKPT_POOL

    def df_d16(self):
        return self.capacity - self.still() - D16_RESERVE - D16_RESIDUAL - self.lag()

    def df(self):
        if self.reading == "合":
            return self.df_d16()
        return self.available_d28(self.defer())

    def empty_unrestorable(self):
        floor = max(self.ceiling(), self.oldest_valid())
        return sum(1 for b in self.blocks if b.release is not None and not b.reclaimed and b.release > floor and b.by_empty)

    def gate1_value(self):
        if self.reading == "合":
            credit = self.restorable() + (self.empty_unrestorable() if self.align_empty else 0)
            return self.available_d28(self.defer() - credit)
        return self.available_d28(self.defer())

    def allocatable_d16(self):
        return min(self.allocatable() - D16_RESERVE, self.df_d16())

    # ---- 发布 ----
    def reclaim(self):
        floor = self.reclaim_floor()
        before = len(self.blocks)
        self.blocks = [b for b in self.blocks if not (b.release is not None and b.release <= floor)]
        return before - len(self.blocks)

    def allocate_blocks(self, count):
        if self.capacity - self.still() - self.defer() - self.isolated < count:
            raise RuntimeError("物理放不下")
        made = [Block(self.txg) for _ in range(count)]
        self.blocks.extend(made)
        return made

    def publish(self, allocate, release, empty, floor=None, rewrite_instance_table=False, log=None, what=""):
        self.reclaim()                               # 分配之前按此刻盘上的环与 F_生效 回收
        self.txg += 1
        released = list(release) + self.fixed
        if rewrite_instance_table:
            released += self.instance_table
        for b in released:
            b.release = self.txg
            b.by_empty = empty
        self.fixed = self.allocate_blocks(FIXED_POINTS)
        if rewrite_instance_table:
            self.instance_table = self.allocate_blocks(2)
        made = self.allocate_blocks(allocate)
        if floor is not None:
            self.F = floor
        self.ring[self.txg % RING_SLOTS] = (self.txg, self.F, not empty)
        self.reclaim()                               # 这条根持久之后可再分配的（记账口径同甲-T1 / F-生）
        if log is not None:
            log.append(self.row(what))
        return made

    def row(self, what):
        return (f"txg={self.txg:3d} {what:<28} F={self.F:3d} F_eff={self.effective_floor():3d} oldest={self.oldest_valid():3d} "
                f"ceiling={self.ceiling():3d} still={self.still():3d} defer={self.defer():3d} isolated={self.isolated:2d} "
                f"restorable={self.restorable():3d} lag={self.lag():3d} df={self.df():4d} gate1={self.gate1_value():4d} "
                f"allocatable={self.allocatable():4d}")

    def push_raise(self, log):
        """推空发布把 F 抬到上限，直到两块盘都带上（生效）；返回推了几次，上限不高于现 F 时 0。"""
        target = self.ceiling()
        if target <= self.F:
            return 0
        pushes = 0
        covered = set()
        while len(covered) < 2 and pushes < B_PUBLISHES:
            self.publish(0, [], True, floor=target, log=log, what=f"推空抬F→{target}")
            covered.add(REGION_DEVICE[self.txg % 3])
            pushes += 1
        return pushes

    def request(self, name, size, log):
        """一次写请求：两道闸按读法走。返回 (成功, 请求前 df, 推了几次)。"""
        df_before = self.df()
        pushes = 0
        if self.reading == "串-重判":
            while self.gate1_value() < size and pushes < B_PUBLISHES:
                done = self.push_raise(log)
                if done == 0:
                    break
                pushes += done
        if self.gate1_value() < size:
            self.last_stage = "第一道闸"
            log.append(f"          请求 {name} {size} 块：df={df_before} 第一道闸 {self.gate1_value()} < {size} ⇒ ENOSPC")
            return False, df_before, pushes
        while self.allocatable_d16() < size and pushes < B_PUBLISHES:
            done = self.push_raise(log)
            if done == 0:
                break
            pushes += done
        if self.allocatable_d16() < size or self.allocatable() - FIXED_POINTS < size:
            self.last_stage = "第二道闸"
            log.append(f"          请求 {name} {size} 块：df={df_before} 第二道闸 可分配 {self.allocatable_d16()} < {size} ⇒ ENOSPC")
            return False, df_before, pushes
        previous = self.objects.get(name, [])
        self.objects[name] = self.publish(size, previous, False, log=log, what=f"写 {name}({size})")
        log.append(f"          请求 {name} {size} 块：df={df_before} 成功（推 {pushes} 次）")
        return True, df_before, pushes

    def delete(self, name, log):
        blocks = self.objects.pop(name)
        self.publish(0, blocks, False, log=log, what=f"删 {name}({len(blocks)})")
        return len(blocks)

    def remount(self, log):
        self.reclaim()
        self.publish(0, [], True, rewrite_instance_table=True, log=log, what="重开：写行")
        covered = {REGION_DEVICE[self.txg % 3]}
        while len(covered) < 2:
            self.publish(0, [], True, log=log, what="重开：暖机")
            covered.add(REGION_DEVICE[self.txg % 3])

    def isolate(self, count, log):
        self.isolated += count
        log.append(f"          回退：影子账隔离 {count} 块（被抛弃根引用、当前账里是空闲）")


import copy


def fill(pool, size, log):
    """先放一个 1 块的小对象（之后当「改用户可见状态的发布」用），再按 size 块一个地写，写到被拒为止。"""
    false_enospc = []
    pool.request("tiny", 1, log)
    index = 0
    while True:
        ok, df_before, _ = pool.request(f"o{index}", size, log)
        if not ok:
            if df_before >= size:
                false_enospc.append((pool.txg, f"o{index}", size, df_before, pool.last_stage))
            return index, false_enospc
        index += 1


def history_delete_then_write(arm, reading, capacity, size, attempts=30):
    """H1：写满 → 删 o0（size 块）→ 每做一次改用户可见状态的发布（1 块的覆盖写；被拒就删下一个对象）就试一次写 size 块。"""
    pool = Pool(capacity, arm, reading)
    log = []
    written, false_enospc = fill(pool, size, log)
    pool.delete("o0", log)
    steps = None
    for step in range(attempts):
        probe = copy.deepcopy(pool)
        probe_log = []
        ok, df_before, _ = probe.request("again", size, probe_log)
        log.append(f"  删后第 {step} 次改用户可见状态的发布之后试写 {size} 块：{'成功' if ok else 'ENOSPC'}（df={df_before}）")
        if not ok and df_before >= size:
            false_enospc.append((probe.txg, "again", size, df_before, probe.last_stage))
        if ok:
            steps = step
            log.extend(probe_log)
            break
        ok_small, _, _ = pool.request("tiny", 1, log)
        if not ok_small:
            others = sorted((name for name in pool.objects if name.startswith("o")), key=lambda name: int(name[1:]))
            if not others:
                log.append("  1 块的覆盖写被拒、也没有别的对象可删，用户再也改不了状态")
                break
            log.append(f"  1 块的覆盖写被拒，改删 {others[0]} 当这一次改用户可见状态的发布")
            pool.delete(others[0], log)
    return {"written": written, "steps_to_reuse": steps, "false_enospc": false_enospc, "log": log}


def history_isolation_then_write(arm, reading, capacity, size, isolated, align_empty=False):
    """H3：写 3 个对象 → 回退（隔离 isolated 块）→ 写满 → 按 df 报的量写一次。"""
    pool = Pool(capacity, arm, reading, align_empty=align_empty)
    log = []
    pool.request("tiny", 1, log)
    for index in range(3):
        pool.request(f"pre{index}", size, log)
    pool.isolate(isolated, log)
    written, false_enospc = fill(pool, size, log)
    df_now = pool.df()
    if df_now > 0:
        ok, df_before, _ = pool.request("by-df", df_now, log)
        if not ok:
            false_enospc.append((pool.txg, "by-df", df_now, df_before, pool.last_stage))
    return {"written": written, "false_enospc": false_enospc, "log": log}


def history_remounts_then_write(arm, reading, capacity, size, remounts, lag_counts_empty=False, align_empty=False):
    """H4：写满 → 重开 remounts 次（写行 + 暖机都是空发布）→ 按 df 报的量写一次。"""
    pool = Pool(capacity, arm, reading, lag_counts_empty=lag_counts_empty, align_empty=align_empty)
    log = []
    written, false_enospc = fill(pool, size, log)
    for _ in range(remounts):
        pool.remount(log)
    df_now = pool.df()
    if df_now > 0:
        ok, df_before, _ = pool.request("by-df", df_now, log)
        if not ok:
            false_enospc.append((pool.txg, "by-df", df_now, df_before, pool.last_stage))
    return {"written": written, "false_enospc": false_enospc, "log": log}


CAPACITY = 300
SIZE = 16
READINGS = ("串", "合", "串-重判")
ARMS = ("乙′", "甲")


def run_all(verbose):
    results = {}
    for arm in ARMS:
        for reading in READINGS:
            h1 = history_delete_then_write(arm, reading, CAPACITY, SIZE)
            h3 = history_isolation_then_write(arm, reading, CAPACITY, SIZE, 36)
            h4 = history_remounts_then_write(arm, reading, CAPACITY, SIZE, 2)
            results[(arm, reading)] = (h1, h3, h4)
            print(f"Z3 arm={arm} reading={reading} H1_steps_to_reuse={h1['steps_to_reuse']} H1_false_enospc={len(h1['false_enospc'])} "
                  f"H3_false_enospc={len(h3['false_enospc'])} H4_false_enospc={len(h4['false_enospc'])}")
            if verbose:
                for name, history in (("H1", h1), ("H3", h3), ("H4", h4)):
                    print(f"--- {arm} {reading} {name} false={history['false_enospc']}")
                    for line in history["log"]:
                        print("   ", line)
    return results


def selftest():
    failures = []

    def expect(condition, message):
        print(("ok   " if condition else "FAIL ") + message)
        if not condition:
            failures.append(message)

    results = run_all(False)
    for arm in ARMS:
        h1_serial = results[(arm, "串")][0]
        h1_rejudge = results[(arm, "串-重判")][0]
        expect(h1_serial["steps_to_reuse"] is None, f"{arm} 串：删掉 {SIZE} 块之后一直做改用户可见状态的发布（1 块覆盖写被拒就删下一个对象，删光或 30 次为止），写回同样大小一次都没成功（第 2 条的界 3 不成立）")
        expected_rejudge = {"乙′": 5, "甲": 8}[arm]
        verdict_note = {"乙′": "越过界 3，落在 D3 已定项 9 ⚠️ 承认的 6 以内", "甲": "越过界 3 也越过 ⚠️ 的 6；式子里 defer 被扣两次（第一轮 Y1）"}[arm]
        expect(h1_rejudge["steps_to_reuse"] == expected_rejudge, f"{arm} 串-重判：删后第 {expected_rejudge} 次改用户可见状态的发布之后写回成功（钉绝对值；{verdict_note}）")
        expect(len(results[(arm, "串")][1]["false_enospc"]) == 0, f"{arm} 串：回退隔离之后按 df 写不出假性 ENOSPC")
        expect(len(results[(arm, "串-重判")][2]["false_enospc"]) == 0, f"{arm} 串-重判：两次重开之后按 df 写不出假性 ENOSPC")
    expect(results[("乙′", "合")][0]["steps_to_reuse"] == 3, "乙′ 合：删后第 3 次改用户可见状态的发布之后写回成功（界 3）")
    expect(len(results[("乙′", "合")][1]["false_enospc"]) == 2, "乙′ 合：回退隔离 36 块之后按 D16 的 df 写，第一道闸拒 ⇒ 假性 ENOSPC")
    expect(len(results[("乙′", "合")][2]["false_enospc"]) == 1, "乙′ 合：两次重开之后按 D16 的 df 写（空发布放掉的不算滞后量）⇒ 假性 ENOSPC")
    expect(len(results[("甲", "合")][0]["false_enospc"]) == 12, "甲 合：式子里「已分配」含 defer，抬 F 放得回来的那一截仍被扣一次 ⇒ 假性 ENOSPC")
    # 判别力自证：拿掉打中的那一个量，同一段历史必须由红转绿。
    # 判别力自证（控制矩阵）：每一截只改一个量，那一截的假性 ENOSPC 必须跟着出现 / 消失，并且停在它该停的那一道闸上。
    def cell(history):
        return [(event[1], event[4]) for event in history["false_enospc"]]
    expect(cell(history_remounts_then_write("乙′", "合", CAPACITY, SIZE, 0)) == [("by-df", "第一道闸")],
           "① 不重开、不对齐：上一次准入推空放掉的固定点，D16 的 df 算可用、合的第一道闸不算 ⇒ 第一道闸拒")
    expect(cell(history_remounts_then_write("乙′", "合", CAPACITY, SIZE, 0, align_empty=True)) == [],
           "① 的控制：第一道闸把空发布放掉的也算可用 ⇒ 归 0")
    expect(cell(history_remounts_then_write("乙′", "合", CAPACITY, SIZE, 2, align_empty=True)) == [("by-df", "第二道闸")],
           "② 对齐之后重开两次：写行与暖机放掉的超过 D16 残留那一截 ⇒ D16 自己的第二道闸拒（df 仍报 6）")
    expect(cell(history_isolation_then_write("乙′", "合", CAPACITY, SIZE, 0, align_empty=True)) == [],
           "③ 的控制：对齐、隔离 0 ⇒ 归 0")
    expect(cell(history_isolation_then_write("乙′", "合", CAPACITY, SIZE, 36, align_empty=True)) == [("o8", "第一道闸"), ("by-df", "第一道闸")],
           "③ 对齐、隔离 36 块：D16 的 df 没有被抛弃根独占量那一项 ⇒ df 报 38 时写 16 块被第一道闸拒")
    return failures


def sweep():
    """几何敏感性（`.claude/rules/mutation-sampling.md` 第六类）：容量、对象大小、c_max 各换几档，重判 3.2 / 3.3 / 3.4 那几格。"""
    global C_MAX, D16_RESERVE, D16_RESIDUAL, SWITCH_RESERVE, CKPT_POOL
    for c_max, switch_reserve in ((4, 56), (9, 116)):
        C_MAX = c_max
        D16_RESERVE = 10 + 7 * c_max
        D16_RESIDUAL = 5 + 6 * c_max
        SWITCH_RESERVE = switch_reserve
        CKPT_POOL = c_max
        for capacity in (300, 600, 1200):
            for size in (4, 16, 64):
                cells = []
                for arm in ARMS:
                    for reading in READINGS:
                        h1 = history_delete_then_write(arm, reading, capacity, size)
                        h3 = history_isolation_then_write(arm, reading, capacity, size, 36)
                        h4 = history_remounts_then_write(arm, reading, capacity, size, 2)
                        cells.append(f"{arm}/{reading}:steps={h1['steps_to_reuse']},H1f={len(h1['false_enospc'])},H3f={len(h3['false_enospc'])},H4f={len(h4['false_enospc'])}")
                controls = (
                    len(history_remounts_then_write("乙′", "合", capacity, size, 0)["false_enospc"]),
                    len(history_remounts_then_write("乙′", "合", capacity, size, 0, align_empty=True)["false_enospc"]),
                    len(history_remounts_then_write("乙′", "合", capacity, size, 2, align_empty=True)["false_enospc"]),
                    len(history_isolation_then_write("乙′", "合", capacity, size, 0, align_empty=True)["false_enospc"]),
                    len(history_isolation_then_write("乙′", "合", capacity, size, 36, align_empty=True)["false_enospc"]),
                )
                print(f"SWEEP c_max={c_max} capacity={capacity} size={size} controls(①,①ctl,②,③ctl,③)={controls} " + " ".join(cells))
    print("SWEEP-COMPLETE")


if __name__ == "__main__":
    if "--sweep" in sys.argv:
        sweep()
        sys.exit(0)
    if "--selftest" in sys.argv:
        failed = selftest()
        print("SELFTEST-FAILED" if failed else "SELFTEST-COMPLETE")
        sys.exit(1 if failed else 0)
    run_all("--verbose" in sys.argv)
    print("RUN-COMPLETE")
