#!/usr/bin/env python3
"""K1：挂载时读不出的槽，根环表记「它在 (N-3S, first_txg-1] 里能住的最大 txg」h。
引理：进程里每一步（窗口锚 M >= first_txg - 1，窗口 = txg > M - 3S）窗口内、与这一槽同槽位、txg <= first_txg - 1 的 txg 至多一个，且它就是 h。
所以记下的 h 不会高于这一槽里真住着的、落在窗口里的那条根（那条根的 txg <= first_txg-1，同槽位，必是 h）。
逐点枚举（不是抽样）：环长 L = 3S，S = 1..8；N、first_txg、M、槽位 c 全扫。"""
import sys
checked = 0
violations = []
for S in range(1, 9):
    L = 3 * S
    for N in range(0, 4 * L):
        for first_txg in range(N + 1, N + 1 + 2 * L + 1):
            lo = N - L + 1 if N >= L else 0          # 挂载时窗口下沿（含）：txg > N - L
            hi = first_txg - 1                       # 上沿
            for c in range(L):
                members = [t for t in range(lo, hi + 1) if t % L == c]
                if not members:
                    continue
                h = max(members)
                for M in range(first_txg - 1, first_txg - 1 + 3 * L + 1):
                    in_window = [t for t in range(0, hi + 1) if t % L == c and t + L > M]
                    # 进程窗口里同槽位、txg <= first_txg - 1 的 txg
                    checked += 1
                    if len(in_window) > 1 or (in_window and in_window[0] != h):
                        violations.append((S, N, first_txg, c, M, h, in_window))
print(f"checked={checked} violations={len(violations)}")
for v in violations[:5]:
    print("VIOLATION", v)
sys.exit(1 if violations else 0)
