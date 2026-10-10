#!/usr/bin/env bash
# 把 expected/（或 rerun 出来的 summary/）里的表投影成报告里贴的那几张小表。用法：views.sh <视图> <表所在目录>
# 视图：safety（自己的读法在四个形状上的 inode 名字空间 r / m）、cost（n1new 借法下相对今天多写的角色数）、structure（零条目叶的结构代价）、
#       loss（单点证据丢失）、clone（克隆头有界穷举）、window（unlink 途中的版本占比）、i918、k3cost、k3third、mirrors、stubs
set -euo pipefail
view=${1:?视图}; dir=${2:?表所在目录}
case "$view" in
    safety)
        awk '($1=="cap233"||$1=="cap6"||$1=="ord233"||$1=="rnd233") && $4=="inode" && (($3=="S" && $2=="today") || ($3 ~ /^own_/ && $3 !~ /uniform/)) {print $1,$2,$3,$5,$7,$8,$9,$10,$11,$12}' "$dir/safety.txt" \
            | sed 's/histories_r_pos/r_hist/;s/histories_m_pos/m_hist/;s/m_scans_in_window/m_win/;s/m_scans_outside_window/m_out/;s/r_peak_max/r_max/;s/m_peak_max/m_max/' | column -t ;;
    cost)
        awk '{for(i=4;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} if($3=="n1new" && $1 ~ /^(cap233|cap6|ord233|rnd233)$/ && ($2=="ret_a"||$2=="persist"||$2=="tsz"||$2=="fsz")) printf "%s %s pub=%d events=%d extra_roles=%d (%.2f%% of today) per1000pub=%.2f per_event=%.2f stub=%d ret_units=%d\n",$1,$2,v["publishes"],v["structural_events"],v["roles_vs_today"],100*v["roles_vs_today"]/(v["roles_total"]-v["roles_vs_today"]),1000*v["roles_vs_today"]/v["publishes"],(v["structural_events"]?v["roles_vs_today"]/v["structural_events"]:0),v["stub_versions"],v["units_retirement"]}' "$dir/cost.txt" | column -t ;;
    structure)
        awk '{for(i=4;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} if($3=="n1new" && $1 ~ /^(cap233|cap6|ord233|rnd233)$/ && ($2=="today"||$2=="persist"||$2=="persist_x")) printf "%s %s leaves_avg=%.2f peak=%d I-9.13_false=%.1f%% S3_false=%.1f%% I-9.12_false=%.1f%% misrouted=%.1f%% sep_not_from_leaf=%.1f%% stub_sep_without_room=%d/%d\n",$1,$2,v["leaf_count_sum"]/v["versions"],v["leaf_count_peak"],100*v["versions_i913_false"]/v["versions"],100*v["versions_i94_third_false"]/v["versions"],100*v["versions_i912_false"]/v["versions"],100*v["versions_misrouted_false"]/v["versions"],100*v["versions_sep_not_derivable"]/v["versions"],v["stub_sep_without_room"],v["stub_sep_without_room"]+v["stub_sep_raised_with_room"]}' "$dir/structure.txt" | column -t ;;
    window)
        awk '{for(i=4;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} if($3=="n1new" && $1 ~ /^(cap233|cap6|ord233|rnd233)$/ && $2=="tsz") printf "%s %s versions=%d in_unlink_window=%d (%.1f%%)\n",$1,$2,v["versions"],v["versions_in_unlink_window"],100*v["versions_in_unlink_window"]/v["versions"]}' "$dir/structure.txt" | column -t ;;
    loss)
        awk '$4=="inode" && ($1=="cap233"||$1=="cap6"||$1=="ord233"||$1=="rnd233") {printf "%s %s %s %s %s %s %s\n",$1,$3,$5,$7,$8,$9,$10}' "$dir/loss.txt" | sed 's/histories_r_pos/r_hist/;s/histories_m_pos/m_hist/;s/r_peak_max/r_max/;s/m_peak_max/m_max/' | column -t ;;
    clone)
        awk '/name=clone /{for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} printf "%s %s sequences=%s r(origin/clone)=%s/%s m(origin/clone)=%s/%s seq_with_r=%s seq_with_m_out=%s\n",v["cand"],v["ablation"],v["sequences"],v["r_scans_origin"],v["r_scans_clone"],v["m_scans_origin"],v["m_scans_clone"],v["sequences_with_r"],v["sequences_with_m_outside_window"]}' "$dir/clone.txt" | column -t ;;
    i918)
        awk '{for(i=4;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} if($3=="n1new" && $2=="fsz" && $1 ~ /^(cap233|cap6|ord233|rnd233)$/) printf "%s fsz versions_i918_false=%d of %d versions_in_unlink_window (%.1f%%)\n",$1,v["versions_i918_false"],v["versions_in_unlink_window"],100*v["versions_i918_false"]/v["versions_in_unlink_window"]}' "$dir/structure.txt" | column -t ;;
    k3cost)
        # 同一段历史上，三种借法下各候选的借次数、退役记录数、零条目版本数、相对今天（同一借法）多写的角色数
        awk '{for(i=4;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} if($1 ~ /^(cap233|cap6|ord233|rnd233)$/ && ($2=="today"||$2=="ret_a"||$2=="persist"||$2=="tsz") && $3 != "n1raise") printf "%s %s %s borrows=%d absorbs=%d drops=%d retire_records=%d stub_versions=%d extra_roles_vs_today=%d\n",$1,$3,$2,v["borrows"],v["absorbs"],v["drops"],v["units_retirement"],v["stub_versions"],v["roles_vs_today"]}' "$dir/cost.txt" | sort -k1,1 -k2,2 -k3,3 | column -t ;;
    k3third)
        # 借完之后 I-9.4 第三句为假的版本占比（N1 新形下这是常态，N2 下为 0）与 N1 旧形（借时抬、之后不跟着变）下分隔 key 两种重建都还原不出的占比
        awk '{for(i=4;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} if($2=="today" && $1 ~ /^(cap233|cap6|ord233|rnd233)$/) printf "%s %s %s S3_false=%.1f%% sep_not_derivable_from_leaf=%.1f%% sep_matches_neither_rule=%.1f%% I-9.12_false=%.1f%% misrouted=%.1f%%\n",$1,$3,$2,100*v["versions_i94_third_false"]/v["versions"],100*v["versions_sep_not_derivable"]/v["versions"],100*v["versions_sep_matches_neither_rule"]/v["versions"],100*v["versions_i912_false"]/v["versions"],100*v["versions_misrouted_false"]/v["versions"]}' "$dir/structure.txt" | sort -k1,1 -k2,2 | column -t ;;
    mirrors)
        # 真 checker 对手改镜像的判定：两种 checker（今天的 / 把 I-9.4 第三句判成恒真的），两种分隔 key 约定
        python3 - "$dir/copy.txt" <<'PY'
import re, sys
rows = {}; order = []
for line in open(sys.argv[1]):
    m = re.match(r'SONR3 image=(\S+)\s+(separator_convention=(\S+) )?leaves=(.*?) checker=(\S+) violated=(.*)', line.strip())
    if not m: continue
    img, _, conv, leaves, chk, viol = m.groups()
    k = ((conv or "-")[:1], img)
    if k not in rows: order.append(k)
    rows.setdefault(k, {})[chk] = viol
    rows[k]["leaves"] = leaves
def num(k):
    return int(k[1].replace("right_half_number_", "")) if k[1].startswith("right_half_number_") else -1
for k in sorted(order, key=lambda k: (k[0], num(k))):
    v = rows[k]
    if k[1].startswith("right_half_number_"):
        print(f"{k[0]} n={num(k)} today={v['today']} third_clause_deleted={v['third_clause_deleted']}")
PY
        ;;
    stubs)
        python3 - "$dir/copy.txt" <<'PY'
import re, sys
rows = {}; order = []
for line in open(sys.argv[1]):
    m = re.match(r'SONR3 image=(\S+)\s+leaves=(.*?) checker=(\S+) violated=(.*)', line.strip())
    if not m: continue
    img, leaves, chk, viol = m.groups()
    if img not in rows: order.append(img)
    rows.setdefault(img, {})[chk] = viol; rows[img]["leaves"] = leaves
want = ["prefix_left_100_middle_17", "drop_leaves_a_zero_entry_leaf", "absorb_leaves_a_zero_entry_leaf_separator_kept_234",
        "absorb_leaves_a_zero_entry_leaf_separator_raised_to_251_with_room", "absorb_leaves_a_zero_entry_leaf_no_room_for_a_separator"]
for img in want:
    v = rows[img]
    print(f"{img}\n    {v['leaves']}\n    today={v['today']} third_clause_deleted={v['third_clause_deleted']}")
PY
        ;;
    prem)
        # 前提族：同一个形状（叶容量 233、起始 1400、2000 步）一次换一个前提；每千次主线发布的删空 / 吸收 / 借 / 退役记录 / 零条目版本，与零条目叶存在的版本占比
        python3 - "$dir" <<'PY'
import sys, collections
d = sys.argv[1]
def rows(f):
    out = {}
    for line in open(f"{d}/{f}"):
        p = line.split()
        out[tuple(p[:3])] = {k: int(v) for k, v in (x.split("=") for x in p[3:])}
    return out
cost, st = rows("cost.txt"), rows("structure.txt")
shapes = sorted({k[0] for k in cost if k[0].startswith("prem_")})
print("前提(谁有数据_怎么删_故障) 每千发布: 删空 吸收 借 退役记录 | persist 零条目版本 零条目叶在的版本占比")
for sh in shapes:
    a = cost[(sh, "ret_a", "n1new")]; p = cost[(sh, "persist", "n1new")]; ps = st[(sh, "persist", "n1new")]
    pub = a["publishes"]
    f = lambda x, n=pub: 1000 * x / n
    print(f"{sh[5:]:36s} {f(a['drops']):5.2f} {f(a['absorbs']):5.2f} {f(a['borrows']):5.2f} {f(a['units_retirement']):5.2f} | {f(p['stub_versions'], p['publishes']):6.2f} {100*ps['versions_i913_false']/ps['versions']:5.1f}%")
PY
        ;;
    len)
        # 历史长度族：E177 式前提（只有 1 号有数据、起始 1400 个）下动作数 100 / 300 / 1000 / 2000（2000 = ord233 与 rnd233 的同类主表行），每段平均发布数、退役记录 / 零条目版本每千次发布、零条目叶在的版本占比
        python3 - "$dir" <<'PY'
import sys
d = sys.argv[1]
def rows(f):
    out = {}
    for line in open(f"{d}/{f}"):
        p = line.split()
        out[tuple(p[:3])] = {k: int(v) for k, v in (x.split("=") for x in p[3:])}
    return out
cost, st = rows("cost.txt"), rows("structure.txt")
print("删序_动作数            发布/段  退役记录/千  零条目版本/千  零条目叶在的版本占比")
for sh in sorted((k[0] for k in cost if k[0].startswith("len_") and k[1] == "ret_a"), key=lambda x: (x.split("_")[1], int(x.split("_")[-1]))):
    a = cost[(sh, "ret_a", "n1new")]; p = cost[(sh, "persist", "n1new")]; ps = st[(sh, "persist", "n1new")]
    print(f"{sh[4:]:22s} {a['publishes']/a['histories']:7.0f} {1000*a['units_retirement']/a['publishes']:11.2f} {1000*p['stub_versions']/p['publishes']:13.2f} {100*ps['versions_i913_false']/ps['versions']:15.1f}%")
PY
        ;;
    sweepfam)
        # 清扫族（cap6，24 段/候选）：自己的读法在 inode / units 上的 r、m 峰值，与崩溃点上在飞的证据单元、清扫抹了多少 / 漏了多少
        python3 - "$dir" <<'PY'
import sys
d = sys.argv[1]
cells = {}
for line in open(f"{d}/safety.txt"):
    p = line.split()
    if p[0].startswith("sweep_") and p[2].startswith("own_") and p[3] in ("inode", "units"):
        kv = dict(x.split("=") for x in p[4:])
        cells[(p[0], p[1], p[3])] = kv
meta = {}
for line in open(f"{d}/sweep.txt"):
    p = line.split()
    meta[(p[0], p[1])] = dict(x.split("=") for x in p[2:])
print("清扫方式                    候选     崩溃点 在飞:退役/墓碑/零条目叶 清扫抹/漏 | inode r_max m_max | units r_max (r>0 的段/24)")
for sh in ("sweep_perfect_cap6", "sweep_misses_evidence_cap6"):
    for cand in ("today", "ret_a", "persist", "tsz"):
        m = meta[(sh, cand)]
        ci = cells.get((sh, cand, "inode")); cu = cells.get((sh, cand, "units"))
        i = f"{ci['r_peak_max']:>3} {ci['m_peak_max']:>3}" if ci else "  -   -"
        u = f"{cu['r_peak_max']:>3} ({cu['histories_r_pos']})" if cu else "  -"
        print(f"{sh:27s} {cand:8s} {m['crash_points']:>4}      {m['ret_in_flight']:>3}/{m['tomb_in_flight']:>3}/{m['zero_entry_leaf_in_flight']:>3}          {m['sweep_erased']:>5}/{m['sweep_evidence_missed']:>3}   | {i}       | {u}")
PY
        ;;
    *) echo "不认的视图：$view" >&2; exit 2 ;;
esac
