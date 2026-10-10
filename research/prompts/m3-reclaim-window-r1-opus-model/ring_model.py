#!/usr/bin/env python3
"""m3-reclaim-window-r1 攻方计数模型（Opus 腿，被攻过零轮）：根环 + 回收门槛 + 回退候选 + 丢盘恢复，纯算术。

几何：R = 3 区、区域归属 [0, 1, 0]（第一版），每区 S 槽；txg t 写区 t % 3 的槽 (t // 3) % S。
每次发布新分配 U 个单元、把上一版的单元全部以释放代 t 释放（每个单元只活在一版里，最坏情形）；分配取编号最小的空闲单元
（不建聚簇段的全空段政策：判的是门槛本身，不靠落点政策兜）。F 恒 0（不建抬 F）。
事件（历史里由用户 / 故障决定的每一步都放开枚举）：
  P   一次正常发布
  X   一次发布根槽写失败（D23 已定项 14 第 399 行目标形态：txg 推进一格、同一版原样换到下一个 txg 再发；一次处置里最多 k_tol = 2 次）
  K   崩在写行那次（记录持久、根没落），下次挂载跳过这个 txg（今天实现与目标形态都有）
  M   崩溃之后完整挂载：写行 + 暖机发布，直到两块盘各落一条新实例的根（D16 已定项 8 戊）
门槛臂：
  G    max(F, 一圈窗口里最旧有效根)，进程内锚 = 这个进程记到的最新 txg（含跳过的），挂载时锚 = 环里最新有效根
  RG   max(F, 环里最旧有效根)
  WD   max(F, min(一圈窗口里最旧有效根, 每块盘最新有效根取最小))（攻方提，零轮）
判（每个落满状态、每个「单元已写、根没落」的崩溃状态都判）：
  fake      回退候选集（G / WD：窗口里、≥ F；RG：环里、≥ F）里有根引用了已被别的版本覆写的单元
  lone_d    丢掉另一块盘，盘 d 上最新有效根引用了已被覆写的单元（不算 journal 重放，偏保守）
  newest    两块盘都在，环里最新根引用了已被覆写的单元
"""
import itertools, random, sys

REGION_DEVICE = [0, 1, 0]


class World:
    def __init__(s, S, U, arm):
        s.S, s.U, s.arm = S, U, arm
        s.ring = {}            # slot -> (txg, instance)
        s.owner = {}           # unit -> version txg 最后写它的那一版
        s.version_units = {}   # txg -> units
        s.released = {}        # unit -> release gen
        s.free = set()
        s.rel_gen = {}         # 盘上分配记录里的「已释放 + 释放代」（挂载重建读它）
        s.next_unit = 0
        s.txg = 0
        s.instance = 1
        s.anchor = 0           # 进程内锚
        s.live = None          # 现行那一版的 txg
        s.root_txgs_written = set()
        s.publish(initial=True)

    def slot(s, t):
        return (t % 3, (t // 3) % s.S)

    def valid_roots(s):
        return sorted(t for (t, _i) in s.ring.values())

    def window_oldest(s, anchor, include_unknown=()):
        ts = [t for t in s.valid_roots() if t + 3 * s.S > anchor]
        return min(ts) if ts else None

    def newest_on_device(s, d):
        ts = [t for (sl, (t, _)) in s.ring.items() if REGION_DEVICE[sl[0]] == d]
        return max(ts) if ts else None

    def floor(s, anchor):
        ring = s.valid_roots()
        if s.arm == "RG":
            return min(ring) if ring else 0
        if s.arm == "ALL":                  # 必红对照：释放了就回收
            return anchor
        w = s.window_oldest(anchor)
        g = w if w is not None else 0
        if s.arm == "WD":
            m = min((s.newest_on_device(d) if s.newest_on_device(d) is not None else 0) for d in (0, 1))
            g = min(g, m)
        return g

    def reclaim(s, anchor):
        f = s.floor(anchor)
        for u, gen in list(s.released.items()):
            if gen <= f:
                del s.released[u]
                s.free.add(u)

    def allocate(s):
        if s.free:
            u = min(s.free)
            s.free.remove(u)
            s.rel_gen.pop(u, None)
            return u
        u = s.next_unit
        s.next_unit += 1
        return u

    # ---- 判 ----
    def intact(s, t):
        return all(s.owner.get(u) == t for u in s.version_units.get(t, ()))

    def judge(s, tally, label, newest_version):
        roots = s.valid_roots()
        if not roots:
            return
        newest = max(roots)
        if s.arm == "RG":
            cands = roots
        else:
            cands = [t for t in roots if t + 3 * s.S > newest]
        bad = []
        if any(not s.intact(t) for t in cands):
            bad.append("fake")
        for d in (0, 1):
            n = s.newest_on_device(d)
            if n is not None and not s.intact(n):
                bad.append(f"lone{d}")
        if not s.intact(newest_version if newest_version is not None else newest):
            bad.append("newest")
        tally["states"] += 1
        for b in bad:
            tally[b] = tally.get(b, 0) + 1
            if (b + "_sample") not in tally:
                tally[b + "_sample"] = label
        outside = [t for t in roots if t + 3 * s.S <= newest]
        if outside:
            tally["states_with_valid_root_outside_window"] = tally.get("states_with_valid_root_outside_window", 0) + 1
        held = len(s.released)
        tally["max_held"] = max(tally.get("max_held", 0), held)

    # ---- 一次发布（进程内）。root_fails：这一次根槽写失败（目标形态推进一格）。交回落没落根。
    def publish(s, initial=False, root_fails=False, tally=None, label=""):
        if initial:
            t = 0
            units = [s.allocate() for _ in range(s.U)]
            for u in units:
                s.owner[u] = t
            s.version_units[t] = units
            s.ring[s.slot(t)] = (t, s.instance)
            s.live = t
            s.anchor = 0
            return True
        t = s.anchor + 1
        units = [s.allocate() for _ in range(s.U)]
        for u in units:
            s.owner[u] = t          # 单元先落盘
        s.version_units[t] = units
        if tally is not None:      # 崩溃状态：单元已写、根没落
            s.judge(tally, label + f"|crash-in-{t}", s.live)
        for u in s.version_units.get(s.live, ()):
            s.released[u] = t
            s.rel_gen[u] = t
        s.live = t
        s.anchor = t                # 取完落点之后记根、按锚 t 回收（今天的 record_root_written_by_this_process）
        s.reclaim(s.anchor)
        if root_fails:
            return False
        s.ring[s.slot(t)] = (t, s.instance)
        return True

    def republish_after_root_failure(s, root_fails=False):
        """目标形态：同一版换到 txg + 1 再发（单元不动，算作新 txg 的一版）。"""
        t = s.live
        nt = t + 1
        s.version_units[nt] = s.version_units.pop(t)
        for u in s.version_units[nt]:
            s.owner[u] = nt
        for u, gen in list(s.released.items()):
            if gen == t:
                s.released[u] = nt
        for u, gen in list(s.rel_gen.items()):
            if gen == t:
                s.rel_gen[u] = nt
        s.live = nt
        s.anchor = nt
        s.reclaim(s.anchor)
        if not root_fails:
            s.ring[s.slot(nt)] = (nt, s.instance)

    # ---- 崩溃 + 挂载 ----
    def remount(s, tally, label, crash_row=False):
        """崩溃之后可写挂载：现行版 = s.live（根落了，或由同实例记录施加）。重建：已释放且 gen ≤ 挂载门槛的回收。
        crash_row：写行那次单元与记录落了、根没落，又崩（K1）。"""
        s.instance += 1
        newest = max(s.valid_roots())
        f = s.floor(newest)         # 挂载锚 = 环里读得出的最新有效根
        live_units = set(s.version_units.get(s.live, ()))
        s.free, s.released = set(), {}
        for u in range(s.next_unit):
            if u in live_units:
                continue
            g = s.rel_gen.get(u)
            if g is not None and g > f:
                s.released[u] = g
            else:
                s.free.add(u)
        # first_txg − 1 = max(环里 txg, 记录 txg)
        s.anchor = max(s.anchor, newest)
        if crash_row:
            saved_rel_gen = dict(s.rel_gen)   # 写行那次的记录不进谱系：盘上分配记录照旧
            t = s.anchor + 1
            units = [s.allocate() for _ in range(s.U)]
            for u in units:
                s.owner[u] = t
            s.version_units[t] = units
            s.judge(tally, label + f"|K1-row-{t}", s.live)
            # 这一版不进谱系：它的单元在下次挂载时是空闲的（不在任何一版的分配记录里）
            for u in units:
                s.free.add(u)
            s.rel_gen = saved_rel_gen
            s.anchor = t
            return
        devs_with_new_root = set()
        while devs_with_new_root != {0, 1}:
            t = s.anchor + 1
            ok = s.publish(tally=tally, label=label)
            devs_with_new_root.add(REGION_DEVICE[t % 3])
            s.judge(tally, label + f"|mount-pub-{t}", s.live)


def run_history(S, U, arm, history, tally):
    w = World(S, U, arm)
    for _ in range(3 * S + 2):          # 前缀：环填满
        w.publish()
    label = history
    for i, ev in enumerate(history):
        lab = f"{label}@{i}{ev}"
        if ev == "P":
            w.publish(tally=tally, label=lab)
        elif ev in "XY":                # 目标形态：失败 1 / 2 次再推进 txg 发
            w.publish(root_fails=True, tally=tally, label=lab)
            if ev == "Y":
                w.republish_after_root_failure(root_fails=True)   # 第二次也失败：那一槽照旧内容
            w.republish_after_root_failure()
        elif ev == "D":                 # 盘 1 的根槽持续写不进（目标形态）：txg 落在区 1 的那次失败一次、推进再发
            if (w.anchor + 1) % 3 == 1:
                w.publish(root_fails=True, tally=tally, label=lab)
                w.republish_after_root_failure()
            else:
                w.publish(tally=tally, label=lab)
        elif ev == "R":                 # 记录落了、根没落就崩（同实例，恢复由记录施加）→ 再挂
            w.publish(root_fails=True, tally=tally, label=lab)
            w.remount(tally, lab)
        elif ev == "K":                 # 崩 → 下一次挂载崩在写行根槽之前
            w.remount(tally, lab, crash_row=True)
        elif ev == "C":                 # 崩在发布边界 → 完整挂载
            w.remount(tally, lab)
        w.judge(tally, lab, w.live)
    return w


def main():
    S = int(sys.argv[1]) if len(sys.argv) > 1 else 4
    L = int(sys.argv[2]) if len(sys.argv) > 2 else 6
    alphabet = sys.argv[3] if len(sys.argv) > 3 else "PKCR"
    U = int(sys.argv[4]) if len(sys.argv) > 4 else 2
    mode = sys.argv[5] if len(sys.argv) > 5 else "all"      # all：全枚举；random:N:seed：随机 N 段；family：成族
    arms = sys.argv[6].split(",") if len(sys.argv) > 6 else ["G", "RG", "WD", "ALL"]
    if mode == "all":
        histories = ["".join(h) for h in itertools.product(alphabet, repeat=L)]
    elif mode.startswith("random"):
        _, count, seed = mode.split(":")
        rng = random.Random(int(seed))
        histories = ["".join(rng.choice(alphabet) for _ in range(L)) for _ in range(int(count))]
    else:   # family：前缀 alphabet[0] 重复 a 次，再 alphabet[1] 重复 b 次（a, b 各取 0..L）
        histories = [alphabet[0] * a + alphabet[1] * b for a in range(L + 1) for b in range(L + 1)]
    # 合法：K（挂载崩在写行）之后只能再 K 或 C（再挂），不能接着在进程里发布
    histories = [h for h in histories if all(h[i + 1] in "KC" for i in range(len(h) - 1) if h[i] == "K")]
    for arm in arms:
        tally = {"states": 0}
        n = 0
        for h in histories:
            run_history(S, U, arm, h, tally)
            n += 1
        keys = ["states", "fake", "lone0", "lone1", "newest", "states_with_valid_root_outside_window", "max_held"]
        print(f"S={S} L={L} alphabet={alphabet} U={U} mode={mode} arm={arm} histories={n} " + " ".join(f"{k}={tally.get(k, 0)}" for k in keys))
        for k in ("fake", "lone0", "lone1", "newest"):
            if k + "_sample" in tally:
                print(f"    {k}_sample={tally[k + '_sample']}")


if __name__ == "__main__":
    main()
