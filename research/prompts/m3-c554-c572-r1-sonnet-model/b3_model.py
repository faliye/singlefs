# 读冻结副本 mount.rs 1591-1695 后写的小模型：三圈共用一个 isolated 集合，按 (盘, 起点槽) 去重（现行）与按槽去重（对照）。
# 不是入库装置，不读盘，只复述那一段的控制流。
def run(placements_in_order, exempt_start_slots, key_of):
    isolated_keys = set()
    slots = set()
    for dev, slot, span in placements_in_order:
        key = key_of(dev, slot, span)
        if (dev, slot) in exempt_start_slots or key in isolated_keys:
            continue
        isolated_keys.add(key)
        for s in range(slot, slot + span):
            slots.add((dev, s))
    return sorted(slots)

start_key = lambda d, s, p: (d, s)                 # 现行：mount.rs 1641 / 1665 / 1686
span_key = lambda d, s, p: (d, s, p)               # 候选 b：键带跨度
def per_slot(placements, exempt, ):                # 候选 a：逐槽去重（先到的变体决定调哪个 isolate*，这里只数覆盖）
    covered = set()
    for dev, slot, span in placements:
        if (dev, slot) in exempt:
            continue
        for s in range(slot, slot + span):
            covered.add((dev, s))
    return sorted(covered)

# 批 10z 判决里的一形（C 表里起点 1544 有两份，窄的先到）
ps = [(0, 1544, 1), (0, 1544, 2)]
print("start-key  :", run(ps, set(), start_key))
print("span-key   :", run(ps, set(), span_key))
print("per-slot   :", per_slot(ps, set()))
# 反向次序（宽的先到）：现行不丢槽
print("start-key (wide first):", run(list(reversed(ps)), set(), start_key))
# 起点槽豁免的对称形：候选根引用 (1544,1)，被抛弃根引用 (1544,2)
print("exempt by start slot  :", run([(0, 1544, 2)], {(0, 1544)}, start_key), "(候选只罩 1544 时 1545 也被一并豁免)")
