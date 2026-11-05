"""K8（C717）一对写死的历史：合法写者 L 与坏写者 B 在 S = 4 的根环上留下逐条相同的根环（txg、实例、F、树状态），
L 在抬 F 那一刻按整环算的上限是 9、抬到 5 合法；B 那一刻的上限是 1、抬到 5 越界。镜像上满足 C717 要红的那一格条件。
用户动作（发布空不空、抬到多少）在 L、B 两边只差 9、10、11 三次发布空不空；推的模型。"""
import sys
sys.path.insert(0, sys.argv[1] if len(sys.argv) > 1 else ".")
from k8_window_model import ceiling, slot_of, judge, k8_condition

S = 4
DEVS = (0, 1, 0)

def build(nonempty_y):
    ring, rows = {}, {}
    st = {}
    state = "A"
    F = 0
    log = []
    def land(t, inst, F, state):
        ring[slot_of(t, S)] = (t, inst, F, state)
    land(1, 1, 0, "A")                      # X：非空
    for t in range(2, 9):
        land(t, 1, 0, "A")                  # 空发布
    for t, s in zip(range(9, 13), nonempty_y):
        state = s
        land(t, 1, 0, state)
    rows[1] = 13                             # 13 的记录落了、根槽没落：由记录施加出 13，新实例写行 (1, 13, ·)
    land(14, 2, 0, state)                   # 实例 2 第一条根（写行，树不变）
    valid = [v for v in ring.values() if v[0] < 15 and v[0] >= 0 and not (v[1] in rows and v[0] > rows[v[1]])]
    c = ceiling(valid, DEVS)
    log.append(("allocator_ceiling_at_raise_txg_15", c))
    F = 5
    land(15, 2, F, state)
    for t in range(16, 25):
        land(t, 2, F, state)
    return ring, rows, log

L_ring, L_rows, L_log = build(["B", "C", "D", "E"])
B_ring, B_rows, B_log = build(["A", "A", "A", "E"])
print("L", L_log, "F_new=5 legal" if L_log[0][1] >= 5 else "F_new=5 ILLEGAL")
print("B", B_log, "F_new=5 legal" if B_log[0][1] >= 5 else "F_new=5 ILLEGAL")
print("images_identical", sorted(L_ring.values()) == sorted(B_ring.values()) and L_rows == B_rows)
print("image_roots", sorted(L_ring.values()))
for name, ring, rows in (("L", L_ring, L_rows), ("B", B_ring, B_rows)):
    print(name, "window_judgement(raise_txg,F,ceiling,red)", judge(ring, rows, S, DEVS, "window"))
    print(name, "whole_ring_judgement(raise_txg,F,ceiling,red)", judge(ring, rows, S, DEVS, "whole"))
    print(name, "c717_must_red_condition(raise_txg,F,X,W)", k8_condition(ring, rows, S, DEVS))
