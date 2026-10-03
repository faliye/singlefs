#!/usr/bin/env python3
"""B7 抽象模型：I-3.9 守卫收窄成「比 L 晚的候选根都走得全才判」够不够稳。
每个落点 p：真实引用它的根集合 S（候选根 txg 1..N 的子集）；合法的释放代 g ∈ (L_true, T_true]（L_true = S 里最大的 txg，
T_true = L_true 之后第一条候选根的 txg，没有这样的根就不许带已释放标志，这一格不造）。每条根「走得全」或「走不全」，
走不全的根看得见的引用是真引用的任一子集（读不出的节点下面藏着的引用数不到）。
三种判法：今天（任一根走不全就整条不判）、不设守卫（拿看得见的算 L、T）、收窄（比看得见的 L 晚的根都走得全才判）。
数：合法 g 上判红的次数（误红），与非法 g（落在区间之外）上判红的次数（抓到）。
"""
import itertools, sys
sys.dont_write_bytecode = True

def judge(roots, complete, visible, g, mode):
    # roots: txg 升序列表；visible: 每条根看得见的引用（布尔）；返回 None 不判 / True 成立 / False 判红
    if mode == "today" and not all(complete):
        return None
    seen = [t for t, v in zip(roots, visible) if v]
    if not seen:
        return None
    L = max(seen)
    if mode == "narrow" and any((not c) for t, c in zip(roots, complete) if t > L):
        return None
    later = [t for t in roots if t > L]
    T = min(later) if later else None
    return T is not None and L < g <= T

def main():
    N = 6
    roots = list(range(1, N + 1))
    stats = {m: {"legal_red": 0, "legal_judged": 0, "illegal_red": 0, "illegal_judged": 0} for m in ("today", "none", "narrow")}
    cases = 0
    for S_bits in itertools.product([0, 1], repeat=N):
        if not any(S_bits):
            continue
        S = [t for t, b in zip(roots, S_bits) if b]
        L_true = max(S)
        later = [t for t in roots if t > L_true]
        if not later:
            continue
        T_true = min(later)
        for complete in itertools.product([True, False], repeat=N):
            # 走不全的根：真引用它的可以看不见；走得全的根看得见全部真引用
            hidden_choices = []
            for t, c in zip(roots, complete):
                truly = t in S
                hidden_choices.append([truly] if (c or not truly) else [True, False])
            for visible in itertools.product(*hidden_choices):
                for g in range(1, N + 2):
                    legal = L_true < g <= T_true
                    cases += 1
                    for mode in stats:
                        verdict = judge(roots, list(complete), list(visible), g, "narrow" if mode == "narrow" else ("today" if mode == "today" else "none"))
                        if verdict is None:
                            continue
                        key = "legal" if legal else "illegal"
                        stats[mode][key + "_judged"] += 1
                        if verdict is False:
                            stats[mode][key + "_red"] += 1
    print("情形数:", cases)
    for mode, s in stats.items():
        name = {"today": "今天（任一根走不全整条不判）", "none": "不设守卫", "narrow": "收窄（比 L 晚的根都走得全才判）"}[mode]
        print(f"{name}: 合法 g 判了 {s['legal_judged']} 次、误红 {s['legal_red']}；非法 g 判了 {s['illegal_judged']} 次、抓到 {s['illegal_red']}")

main()
