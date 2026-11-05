"""Q1：写者今天的实现上，造不造得出「u 带提交标记的记录在 journal、u 与 r 同一实例或 u 那个实例的行 T > u，
而抬 F 那一刻 u 的根不在它那一槽」（推的模型：照快照代码手写，不是编出来的二进制）。

写者（快照 crates/singlefs-core/src/transaction.rs）：一次发布 = 记录落盘（1488–1494，记录之后一道屏障）→ 根槽 FUA（1637）
→ 轮换系统配置 → 屏障（1645）。落盘几步里失败：经分配器的发布冻结、下一次发布之前原样重发（1545、1563、1594）；
挂载里的写行与暖机（零单元或经分配器）失败整个挂载报错（mount.rs 8103、8125），实例随之结束。
恢复（mount.rs）：所选根 = 环里 (txg, 实例) 最大的根；施加同实例、txg 接着的提交记录；上一实例的行 T = 施加之后那一版的 txg
（9230–9233），中间实例补 (j, 0)；新实例第一个 txg = max(环里根的 txg, 记录的 txg) + 1（1403）；新实例代号 = 见过的最大 + 1。
判法（快照 crates/singlefs-checker/src/walk.rs 8986–9052）：对镜像上每条根 r，逐槽认抬 F 那一刻的住户，与模型记下的
「r 的根落盘前一刻那一槽的真住户」比。认出的不是真住户 = 打中（mismatch）。
弄坏开关（证模型会红）：BREAK=no-T-check | first-txg-as-row | zero-unit-continues | no-freeze
用法：python3 q1_model.py R S 最多发布次数 最多故障次数 最多挂载次数
"""
import os
import sys

R, S, MAX_ATTEMPTS, MAX_FAULTS, MAX_MOUNTS = (int(x) for x in sys.argv[1:6])
L = R * S
BREAK = os.environ.get("BREAK", "")
OUTCOMES = ["ok", "root_fail_noland", "root_fail_landed", "record_fail_noland", "record_fail_landed",
            "crash_pre", "crash_mid", "crash_post"]


def slot_of(txg):
    return (txg % R, (txg // R) % S)


def largest_below(region, slot, rt):
    residue = region + R * slot
    if rt - 1 < residue:
        return None
    u = residue + ((rt - 1 - residue) // L) * L
    return u if u > 0 else None


stats = dict(images=0, roots_judged=0, slots_recognised=0, mismatch=0, caught_record_without_root=0,
             caught_new_instance_first_txg_is_u_plus_one=0, caught_row_of_u_instance_at_most_u=0)
examples = []


def evaluate(st):
    stats["images"] += 1
    ring, journal, tables, snaps = st["ring"], st["journal"], st["tables"], st["snaps"]
    for (rg, sl), r in ring.items():
        if r is None or r[0] == 0:
            continue
        rt, ri = r
        rows = tables[ri]
        truth_ring = snaps[r]
        stats["roots_judged"] += 1
        for region in range(R):
            for slot in range(S):
                c = ring.get((region, slot))
                truth = truth_ring.get((region, slot))
                if c is None:
                    got = None
                elif c[0] < rt:
                    got = c
                else:
                    u = largest_below(region, slot, rt)
                    if u is None:
                        continue  # 只可能住第 0 代根：判法按抬之前的 F 报空着或认不出，不在本题
                    versions = [i for (i, x) in journal if x == u]
                    if len(versions) != 1:
                        continue  # 认不出（Err）
                    vi = versions[0]
                    its_rows = [T for (i, T) in rows if i == vi]
                    landed = vi == ri or (its_rows and all(u < T for T in its_rows)) or BREAK == "no-T-check"
                    if not landed:
                        if truth != (u, vi):
                            stats["caught_record_without_root"] += 1
                            stats["caught_new_instance_first_txg_is_u_plus_one"] += st["first_txg"].get(vi + 1) == u + 1
                            stats["caught_row_of_u_instance_at_most_u"] += bool(its_rows) and all(T <= u for T in its_rows)
                        continue
                    got = (u, vi)
                stats["slots_recognised"] += 1
                if got != truth and not (got is None and truth is not None and truth[0] == 0):
                    stats["mismatch"] += 1
                    if len(examples) < 3:
                        examples.append((st["log"], r, (region, slot), got, truth))


def land(st, root):
    if root not in st["snaps"]:
        st["snaps"][root] = dict(st["ring"])
    st["ring"][slot_of(root[0])] = root


def copy(st):
    n = dict(st)
    n["ring"], n["journal"], n["snaps"] = dict(st["ring"]), set(st["journal"]), dict(st["snaps"])
    n["tables"], n["first_txg"], n["log"] = dict(st["tables"]), dict(st["first_txg"]), st["log"]
    return n


def mount(st, warm_ups):
    """恢复 + 取号：交回新状态（实例在挂载阶段，还要发 1 + warm_ups 次挂载里的发布）。"""
    n = copy(st)
    roots = [r for r in n["ring"].values() if r is not None]
    chosen = max(roots)
    eff = chosen[0]
    while (chosen[1], eff + 1) in n["journal"]:
        eff += 1
    inst = n["max_inst"] + 1
    first = max([r[0] for r in roots] + [x for (_, x) in n["journal"]]) + 1
    row_T = first if BREAK == "first-txg-as-row" else eff
    rows = tuple(n["tables"][chosen[1]]) + ((chosen[1], row_T),) + tuple((j, 0) for j in range(chosen[1] + 1, inst))
    n["tables"][inst] = rows
    n["first_txg"][inst] = first
    n.update(max_inst=inst, inst=inst, next_txg=first, mount_left=1 + warm_ups, frozen=None,
             mounts=n["mounts"] + 1, log=n["log"] + (f"mount(inst={inst},first={first},row=({chosen[1]},{row_T}),W={warm_ups})",))
    return n


def attempt(st, txg, outcome, in_mount):
    """一次发布（或重发）的结局：交回新状态。"""
    n = copy(st)
    inst = n["inst"]
    n["log"] = n["log"] + (f"{'mount-publish' if in_mount else ('resend' if n['frozen'] else 'publish')}(txg={txg}):{outcome}",)
    n["attempts"] += 1
    n["faults"] += outcome != "ok"
    if outcome in ("ok", "root_fail_noland", "root_fail_landed", "record_fail_landed", "crash_mid", "crash_post"):
        n["journal"].add((inst, txg))
    if outcome in ("ok", "root_fail_landed", "crash_post"):
        land(n, (txg, inst))
    if outcome.startswith("crash"):
        n["inst"] = None
        return n
    if outcome == "ok":
        n["frozen"] = None
        n["next_txg"] = txg + 1
        if in_mount:
            n["mount_left"] -= 1
        return n
    # 失败：挂载里的整个挂载报错；挂着时冻结等重发（弄坏开关放开这两道）
    if in_mount and BREAK != "zero-unit-continues":
        n["inst"] = None
    elif not in_mount and BREAK != "no-freeze":
        n["frozen"] = txg
    else:
        n["next_txg"] = txg + 1
        if in_mount:
            n["mount_left"] -= 1
    return n


def dfs(st):
    evaluate(st)
    if st["inst"] is None:
        if st["mounts"] < MAX_MOUNTS:
            for w in (1, 2):
                dfs(mount(st, w))
        return
    if st["attempts"] >= MAX_ATTEMPTS:
        return
    in_mount = st["mount_left"] > 0
    txg = st["frozen"] if st["frozen"] is not None else st["next_txg"]
    for outcome in OUTCOMES:
        if outcome != "ok" and st["faults"] >= MAX_FAULTS:
            continue
        dfs(attempt(st, txg, outcome, in_mount))
    if not in_mount:  # 用户可以不再发、进程退出（冻结着也一样）
        q = copy(st)
        q["inst"] = None
        q["log"] = q["log"] + ("quit",)
        dfs(q)


genesis = {"ring": {(rg, sl): None for rg in range(R) for sl in range(S)}, "journal": set(), "snaps": {},
           "tables": {0: ()}, "first_txg": {}, "max_inst": 0, "inst": None, "mounts": 0, "attempts": 0,
           "faults": 0, "frozen": None, "next_txg": 0, "mount_left": 0, "log": ()}
genesis["ring"][(0, 0)] = (0, 0)
genesis["snaps"][(0, 0)] = dict(genesis["ring"])
sys.setrecursionlimit(10000)
dfs(genesis)
print(f"name=q1_model R={R} S={S} attempts<={MAX_ATTEMPTS} faults<={MAX_FAULTS} mounts<={MAX_MOUNTS} break={BREAK or 'none'} "
      + " ".join(f"{k}={v}" for k, v in stats.items()))
for e in examples:
    print("example", e)
