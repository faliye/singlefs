#!/usr/bin/env python3
"""复跑入口：python3 run_all.py > run_all.out。全是纯 Python，几分钟内跑完（单核）。"""
import sys, os, time
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import birth_floor_v2 as m

def tier(name, depth):
    fut, fault = name == "FUT", name == "FAULT"
    t = time.time()
    stats, cls, ex = m.dfs(m.ALPHABETS[name], depth, fut, fault)
    m.show(f"[穷举] {name} 深度 {depth}" + (" 含故障" if fault else " 无故障"), stats, "plain 红单元按 R_first 归类 " + str(cls))
    for k in ("opusI", "both"):
        if k in ex:
            print(f"   首例 {k}: {ex[k]}")
    print(f"   用时 {time.time() - t:.0f} 秒")

def rnd(name, n, length, seed):
    alpha = m.ALPHABETS[name]
    alpha = alpha + [("W",)] * 6 + [a for a in alpha if a[0] in "RF"] * 2
    s, wrapped, ex = m.random_histories(alpha, n, length, seed, fut=(name == "FUT"), fault=(name == "FAULT"))
    m.show(f"[随机] {name} {n} 段 × 至多 {length} 步，种子 {seed}", s, f"其中走过根环（txg > {m.RING + 3}）的历史 {wrapped} 段")

if __name__ == "__main__":
    for name, depth in (("HARNESS", 7), ("API", 6), ("FUT", 6), ("FAULT", 6)):
        tier(name, depth)
    for name, seed in (("HARNESS", 24), ("API", 21), ("FUT", 22), ("FAULT", 23)):
        rnd(name, 3000, 90, seed)
    alpha = m.ALPHABETS["API"] + [("W",)] * 4 + [a for a in m.ALPHABETS["API"] if a[0] in "RF"]
    hs = m.sample_histories(alpha, 3000, 14, 5)
    print(f"[坏镜像层·盖小] {len(hs)} 段随机合法历史（API 字母表，无故障），种子 5；「天花板外」= 盖到的数不比 R_prev.txg 小，任何下界写法都抓不到")
    m.print_study(m.mutation_study(hs))
    hs2 = m.sample_histories(m.ALPHABETS["API"] + [("W",)] * 4, 1500, 12, 7)
    print("[坏镜像层·复活旧单元（只红 ①）]", m.reappearance_study(hs2))
    fa = m.ALPHABETS["FAULT"]
    allh = []
    def rec(h, d):
        allh.append(h)
        if d == 5:
            return
        for op in fa:
            n = m.step(h, op, False, True)
            if n is not None:
                rec(n, d + 1)
    rec(m.start(), 0)
    print(f"[出现规则 = ② 的 sc_lt ∪ ① 的 holes_lt] FAULT 字母表深度 5 共 {len(allh)} 张镜像，红集不一致的 {m.check_appearance_rule_equals_union(allh)} 张")
