#!/usr/bin/env python3
# 用法：compare_r2opus.py <r2 攻方腿的 expected 目录> <这一轮的输出目录>
# 把 r2 攻方腿（Opus）模型的每个输出文件，按头部那一行的 (臂, 变异, 环, 清扫阈值, 长度) 找到这一轮同一组的输出，
# 对着名字表逐候选比 false_positive / event_missed / residual_missed / mutated_history_undetected / mutated_images 五格，
# 再比头部的 images、events、residual_images、reissues_across_abandoned_roots 与几个步数总数。全相等退 0。
import sys, glob, os
NAMES = {  # r2 攻方腿的候选名 -> 这一轮的名字
    "I-9.29": "I-9.29", "I930/C2a-create": "I930-C2a-create", "I930/C2a-all": "I930-C2a-all",
    "C2aL-all:pair-cover": "C2aL-pair-cover", "C2aL-all:pair-eq": "C2aL-pair-eq", "C2aL-create:pair-cover": "C2aL-create-pair-cover",
    "C2aL-all:pair(Tcur,Hnew)": "C2aL-pair-TcurHnew", "C3r": "C3r", "C3rs": "C3rs", "C3r:I930-3": "C3r-I930-3",
    "C3rL:count-cover": "C3rL-count-cover", "C3rL:count-eq": "C3rL-count-eq", "C2aL-strongest": "C2aL", "C3rL-strongest": "C3rL",
}
FIELDS = ["false_positive", "event_missed", "residual_missed", "mutated_history_undetected", "mutated_images"]
def label(h, opus=False):
    mutation = h["mutation"]
    if opus and mutation == "HistoryAndLineagePairFromTheTarget": mutation = "HistoryFromTheTarget"  # 攻方腿的 A + H 就是这一轮（和 r2 辩方腿）的 A
    alphabet = h.get("alphabet", "all12")
    return (h["arm"], mutation, h["ring"], h["sweep_threshold"], h["longest"], alphabet)
def parse(path, prefix):
    head = None; extra = {}; rows = {}
    for line in open(path, encoding="utf-8"):
        t = line.split()
        if not t: continue
        if t[0] == prefix + "MODEL":
            head = dict(kv.split("=", 1) for kv in t[1:])
        elif t[0] == "W3EXTRA":
            extra = dict(kv.split("=", 1) for kv in t[1:])
        elif t[0] == prefix + "COUNT":
            d = dict(kv.split("=", 1) for kv in t[1:]); rows[d["candidate"]] = d
    return head, extra, rows
mine = {}
for p in glob.glob(os.path.join(sys.argv[2], "model-*.out")):
    h, e, r = parse(p, "W3")
    if h: mine.setdefault(label(h), (p, h, e, r))
bad = 0; cells = 0; files = 0
for p in sorted(glob.glob(os.path.join(sys.argv[1], "model-*.out"))):
    h, _, r = parse(p, "W3R2")
    key = label(h, True) + ("all12",)[:0]
    if key not in mine:
        print(f"这一轮没有同一组：{os.path.basename(p)} {key}"); bad += 1; continue
    files += 1
    mp, mh, me, mr = mine[key]
    heads = [("images", "images"), ("events", "events"), ("residual_images", "residual_images"), ("reissues_across_abandoned_roots", "reissues_across_abandoned_roots")]
    if h["arm"] == "Product" and h["mutation"] == "None":
        heads.append(("legal_images", "legal_images"))
    fp_meaningful = h["arm"] == "Product"
    for ok, nk in heads:
        cells += 1
        if h[ok] != mh[nk]: print(f"{os.path.basename(p)} 头 {ok}: 攻方={h[ok]} 这一轮={mh[nk]}"); bad += 1
    for ok, nk in [("rollback_jumps_from_abandoned_roots_only",) * 2, ("rollback_jumps_from_a_root_on_the_timeline",) * 2, ("sweeps",) * 2, ("switches",) * 2, ("rollbacks_back_to_a_former_cur",) * 2]:
        cells += 1
        if h[ok] != me[nk]: print(f"{os.path.basename(p)} 头 {ok}: 攻方={h[ok]} 这一轮={me[nk]}"); bad += 1
    for oc, nc in NAMES.items():
        for f in FIELDS:
            if f == "false_positive" and not fp_meaningful:
                continue  # 非 P 臂上「合法镜像」的定义两边不同（攻方腿只在 P 臂数合法镜像），不比
            cells += 1
            if r[oc][f] != mr[nc][f]:
                print(f"{os.path.basename(p)} {oc} {f}: 攻方={r[oc][f]} 这一轮={mr[nc][f]}"); bad += 1
print(f"compare-r2opus 比了 {files} 个文件 {cells} 格，不同 {bad} 格")
sys.exit(1 if bad else 0)
