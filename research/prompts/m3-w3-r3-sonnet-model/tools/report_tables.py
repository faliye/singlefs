#!/usr/bin/env python3
"""用法：report_tables.py <输出目录>   报告里的几张汇总表从 <输出目录> 里的 model-*.out 现算，一格一格对得上原始行（W3MODEL / W3EXTRA / W3COUNT）。"""
import sys, os
D = sys.argv[1]
def load(name):
    head = None; rows = {}; extra = {}
    for line in open(os.path.join(D, name), encoding="utf-8"):
        if line.startswith("W3MODEL"): head = dict(kv.split("=", 1) for kv in line.split()[1:])
        elif line.startswith("W3EXTRA"): extra = dict(kv.split("=", 1) for kv in line.split()[1:])
        elif line.startswith("W3COUNT"):
            d = dict(kv.split("=", 1) for kv in line.split()[1:]); rows[d["candidate"]] = d
    return head, extra, rows
FORMS = ["C2aL", "C3rL", "C4g", "C2i", "C2ig"]
print("== 表一：V1 / V3 / V2 臂上「现有 I-9.29t、I-9.6t 都不红」的事件与持久镜像，候选不红的个数 / 分母 ==")
print("文件 | 臂 环 长 字母表 | 洞事件分母 | 持久洞分母 | " + " | ".join(FORMS))
for name in ["model-defense-0.out", "model-defense-1.out", "model-defense-2.out", "model-defense-3.out", "model-deep-2.out", "model-ring6-1.out", "model-defense-4.out", "model-main-4.out"]:
    h, e, r = load(name)
    cells = [f"{r[c]['event_missed_where_existing_checks_silent']}/{h['events_where_existing_checks_silent']} + {r[c]['residual_missed_where_existing_checks_silent']}/{h['residual_where_existing_checks_silent']}" for c in FORMS]
    print(f"{name} | {h['arm']} ring={h['ring']} len={h['longest']} {h['alphabet']} | {h['events_where_existing_checks_silent']} | {h['residual_where_existing_checks_silent']} | " + " | ".join(cells))
print()
print("== 表二：P 臂合法镜像上的误报（只在 P 臂有意义）：候选红的张数 / 合法镜像数；括号里是合法的跨被抛弃时间线重发次数 ==")
print("文件 | 臂 环 阈值 长 字母表 | 合法镜像 | 跨抛弃重发 | " + " | ".join(FORMS))
tot = 0
for name in ["model-main-0.out", "model-main-1.out", "model-defense-5.out", "model-deep-0.out", "model-k2-0.out", "model-k2-1.out", "model-k2-2.out", "model-blind-0.out", "model-blind-1.out"]:
    h, e, r = load(name)
    cells = [f"{r[c]['false_positive']}" for c in FORMS]
    print(f"{name} | {h['arm']} ring={h['ring']} sweep={h['sweep_threshold']} len={h['longest']} {h['alphabet']} | {h['legal_images']} | {h['reissues_across_abandoned_roots']} | " + " | ".join(cells))
    if h["arm"] == "Product" and name != "model-blind-0.out": tot += int(h["legal_images"])
print(f"P 臂合法镜像合计（main-0、main-1、defense-5、deep-0）= {sum(int(load(n)[0]['legal_images']) for n in ['model-main-0.out','model-main-1.out','model-defense-5.out','model-deep-0.out'])}")
print()
print("== 表三：历史写错 / 不写（K3 的变异）：P 臂 = 历史被改过的镜像里不红 / 被改过的；V1 臂 = 现有检查都不红的事件里候选不红的个数 ==")
cols = []
for i, n in enumerate(["A", "B", "C"]): cols.append((f"P+{n}", "P", f"model-mutation-{i}.out"))
for i, n in enumerate(["A", "B", "C"]): cols.append((f"V1+{n}", "V1", f"model-mutation-{3+i}.out"))
for i, n in enumerate(["H", "Z", "R", "D", "Mh"]): cols.append((f"P+{n}", "P", f"model-mutation2-{i}.out"))
for i, n in enumerate(["H", "Z", "R", "D", "Mh"]): cols.append((f"V1+{n}", "V1", f"model-mutation2-{5+i}.out"))
data = [(n, arm, *load(p)) for n, arm, p in cols]
print("V1 臂洞事件分母：" + " ".join(f"{n}={h['events_where_existing_checks_silent']}" for n, arm, h, e, r in data if arm == "V1"))
print("候选 | " + " | ".join(n for n, *_ in data))
for c in FORMS + ["I930-C2a-create", "I930-C2a-all", "I930-C2a-create-tol", "I930-C2a-all-tol", "C2aL-tol"]:
    cells = []
    for n, arm, h, e, r in data:
        d = r[c]
        cells.append(f"{d['mutated_history_undetected']}/{d['mutated_images']}" if arm == "P" else d["event_missed_where_existing_checks_silent"])
    print(c + " | " + " | ".join(cells))
print()
print("== 表四：Y1：P 臂上字面读法与 L 形的误报（取法一 = main-0/1，取法二 = k2-0/1），后两列是误报里「跳升之后」的数 ==")
LIT = ["I930-C2a-create", "C3r-I930-3", "C2aL-create-pair-cover", "C2aL-pair-TcurHnew", "I930-C2a-create-tol", "I930-C2a-all", "C2aL", "C3rL", "C4g"]
print("候选 | " + " | ".join(["取法一 环4 从不", "取法一 环4 阈值0", "取法二 环4 从不", "取法二 环4 阈值0", "取法一 环8 阈值0", "取法二 环8 阈值0"]))
files = ["model-main-0.out", "model-main-1.out", "model-k2-0.out", "model-k2-1.out", "model-defense-5.out", "model-k2-2.out"]
loaded = [load(f) for f in files]
for c in LIT:
    print(c + " | " + " | ".join(f"{r[c]['false_positive']} (跳升后 {r[c]['false_positive_after_a_jump_from_abandoned_roots']})" for h, e, r in loaded))
print("jumps_from_abandoned_roots_only / images_after_a_jump / 跨抛弃重发 / 回退水位退回的镜像 / 时间线重发的镜像：")
for f, (h, e, r) in zip(files, loaded):
    print(f"  {f} {h['arm']}: {e['rollback_jumps_from_abandoned_roots_only']} / {e['images_after_a_jump_from_abandoned_roots']} / {h['reissues_across_abandoned_roots']} / {e['watermark_regressed_images']} / {e['timeline_reissue_images']}")
print()
print("== 表五：C143 那一格：含 rollback-blind 的字母表（长 7、环 4）上三臂 ==")
for f in ["model-blind-0.out", "model-blind-1.out", "model-blind-2.out"]:
    h, e, r = load(f)
    print(f"  {f} {h['arm']}: images={h['images']} 水位退回的镜像={e['watermark_regressed_images']} 时间线上重发的镜像={e['timeline_reissue_images']} 事件={h['events']} 洞事件={h['events_where_existing_checks_silent']} 持久镜像={h['residual_images']} 洞持久={h['residual_where_existing_checks_silent']}")
print()
print("== 表六：「容许跳升」的读法在历史长过环（环 8、长 11、字母表 4 个）时：P 臂合法镜像上的误报 ==")
h, e, r = load("model-tolring-0.out")
print(f"  {h['alphabet']} ring={h['ring']} len={h['longest']} 合法镜像={h['legal_images']} 取法一的跳升={e['rollback_jumps_from_abandoned_roots_only']} 跳升之后的镜像={e['images_after_a_jump_from_abandoned_roots']}")
for c in ["I930-C2a-create", "I930-C2a-create-tol", "I930-C2a-all", "I930-C2a-all-tol", "C2aL", "C2aL-tol", "C3rL", "C4g"]:
    print(f"  {c} 误报={r[c]['false_positive']}")
