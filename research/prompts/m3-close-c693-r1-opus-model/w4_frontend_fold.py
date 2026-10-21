#!/usr/bin/env python3
"""攻方腿 m3-close-c693-r1 W4：开前端之后整批换代的两种落法，与删一个可写头时台阶行怎么离开，在今天的检查下红不红。

独立手写模型，纯整数，不 import crates/。行形照 crates 今天（池级 3 行：标签 4、6、12；每盘 6 行：1、2、3、5、10、11）。
几样「照 crates 今天」的规则逐条照这几处写（行号在报告里现查）：
  crates_steps     transaction.rs inode_number_watermark_steps：标签 12、代段早于这一版 txg 的行都是台阶行，不按树分；
                   这一版那一行值与最后一级不同才算一级
  i313_syntactic   walk.rs 第 2649 行起：标签 12、设备段 0xFFFF_FFFF、代段早于这一版 txg 的行跳过，其余行判代段 = txg
  count_check      recovery.rs accounting_entry_count_is_three_plus_six_per_device：台阶行（同上语法）之外的行恰 3 + 6d
  i930_one         walk.rs I-9.30 ①：水位（标签 12 代段最大那一行，不分树）不小于每一级台阶的 H
落法：
  rebuild          这一版的树 = 这一版写的行 ∪ 上一版按 crates_steps 认出的台阶行（crates 今天的规划结果，没有墓碑）
  point_delete     前端：上一版有、这一版没有的 key 进一条墓碑，这一版的行进前端；排空到叶时同 key 折叠
                   fold=arrival  D8 已定项 13：按到达序取最后一条，墓碑就删
                   fold=naive_max 单调统计量（标签 12）把墓碑当 0 与叶上的值取最大（C344 写的那种朴素折叠）
台阶照抄规则（step_rule）：crates = 上面 crates_steps（不分树，这一版那一行只认 find 到的第一条）；
  per_tree_all = 按 (统计量, 树) 分开认（D5 已定项 2 射程、transaction.rs 那条 TODO 写的做法），不看那个头还在不在；
  per_tree_live = 按树分开认，只照抄还活着的头的台阶行（删头那次不带过去）。
真值：D5 意义的台阶行 = 那个头水位变了的那次发布留的 (g, H)，按 (统计量, 树) 分。stale = 既不是这一版的行、也不是真台阶行的行数。
自证：rebuild 与 point_delete+arrival 在每段历史每一版上逐行相同（两条路）；弄坏开关 W4_FOLD_BREAK=arrival_keeps_tomb_target
让 arrival 折叠不删墓碑指的 key，「两条路逐行相同」必须红。
"""
import os, sys

D = 2
POOL = [4, 6]
PER_DEVICE = [1, 2, 3, 5, 10, 11]
WM = 12
NODEV = 0xFFFFFFFF
BREAK = os.environ.get("W4_FOLD_BREAK", "")

def rows_of_this_publish(txg, heads_watermark, counter):
    rows = {}
    for tag in POOL:
        rows[(tag, 0, NODEV, txg)] = counter
    for dev in range(D):
        for tag in PER_DEVICE:
            rows[(tag, 0, dev, txg)] = counter + dev
    for head, h in heads_watermark.items():
        rows[(WM, head, NODEV, txg)] = h
    return rows

def crates_steps(tree, txg):
    steps = sorted(((k, v) for k, v in tree.items() if k[0] == WM and k[3] < txg), key=lambda kv: kv[0][3])
    this = [(k, v) for k, v in tree.items() if k[0] == WM and k[3] == txg]
    if this:
        k, v = this[0]
        if not steps or steps[-1][1] != v:
            steps.append((k, v))
    return dict(steps)

def per_tree_steps(tree, txg, live_heads):
    out = {}
    for head in sorted({k[1] for k in tree if k[0] == WM}):
        if live_heads is not None and head not in live_heads:
            continue
        sub = {k: v for k, v in tree.items() if k[0] == WM and k[1] == head}
        out.update(crates_steps(sub, txg))
    return out

def steps_by_rule(tree, txg, rule, live_heads):
    if rule == "crates":
        return crates_steps(tree, txg)
    if rule == "per_tree_all":
        return per_tree_steps(tree, txg, None)
    return per_tree_steps(tree, txg, live_heads)

def drain(leaf, frontend, fold):
    out = dict(leaf)
    by_key = {}
    for key, value in frontend:
        by_key.setdefault(key, []).append(value)
    for key, values in by_key.items():
        if fold == "arrival":
            last = values[-1]
            if last is None:
                if BREAK != "arrival_keeps_tomb_target":
                    out.pop(key, None)
            else:
                out[key] = last
        else:
            if key[0] == WM:
                folded = max([out.get(key, 0)] + [0 if v is None else v for v in values])
                if folded:
                    out[key] = folded
                else:
                    out.pop(key, None)
            else:
                last = values[-1]
                if last is None:
                    out.pop(key, None)
                else:
                    out[key] = last
    return out

def publish(previous, prev_txg, txg, heads_watermark, counter, design, fold, rule="crates"):
    new_rows = rows_of_this_publish(txg, heads_watermark, counter)
    steps = steps_by_rule(previous, prev_txg, rule, set(heads_watermark))
    if design == "rebuild":
        tree = dict(steps)
        tree.update(new_rows)
        return tree
    keep = set(steps) | set(new_rows)
    frontend = [(k, None) for k in sorted(previous) if k not in keep]
    frontend += sorted(new_rows.items())
    return drain(previous, frontend, fold)

def checks(tree, txg, true_steps, live_heads):
    i313 = all(r[3] == txg for r in tree if not (r[0] == WM and r[2] == NODEV and r[3] < txg))
    non_steps = sum(1 for r in tree if not (r[0] == WM and r[3] < txg))
    count_ok = non_steps == 3 - 1 + len(live_heads) + 6 * D if len(live_heads) != 1 else non_steps == 3 + 6 * D
    wm_rows = [(r, v) for r, v in tree.items() if r[0] == WM]
    watermark = max(wm_rows, key=lambda rv: rv[0][3])[1]
    i930 = all(watermark >= v for r, v in crates_steps(tree, txg).items())
    stale = sum(1 for r in tree if r[3] != txg and r not in true_steps)
    leftover_of_deleted = sum(1 for r in tree if r[0] == WM and r[1] not in live_heads)
    return i313, count_ok, i930, stale, leftover_of_deleted

def run(history, design, fold, label, rule="crates"):
    txg = 3
    heads = {65537: 2}
    tree = rows_of_this_publish(txg, heads, 0)
    true_steps = {(WM, 65537, NODEV, 3)}
    trace = []
    for g, event in enumerate(history, start=1):
        prev_txg, txg = txg, txg + 1
        if event[0] == "create_inode":
            heads[event[1]] += 1
            true_steps.add((WM, event[1], NODEV, txg))
        elif event[0] == "create_clone":
            heads[event[1]] = heads[65537]
            true_steps.add((WM, event[1], NODEV, txg))
        elif event[0] == "delete_head":
            del heads[event[1]]
            true_steps = {s for s in true_steps if s[1] != event[1]}
        tree = publish(tree, prev_txg, txg, heads, g, design, fold, rule)
        i313, count_ok, i930, stale, leftover = checks(tree, txg, true_steps, set(heads))
        trace.append((g, txg, len(tree), i313, count_ok, i930, stale, leftover))
    return trace

HISTORIES = {
    "single_head_two_inode_publishes": [("create_inode", 65537) if g in (3, 7) else ("empty",) for g in range(1, 21)],
    "clone_created_then_deleted": [("create_clone", 65549) if g == 2 else ("create_inode", 65549) if g in (4, 5) else ("delete_head", 65549) if g == 10 else ("empty",) for g in range(1, 21)],
}
designs = [("rebuild", "none"), ("point_delete", "arrival"), ("point_delete", "naive_max")]
results = {}
for hname, hist in HISTORIES.items():
  for rule in ("crates", "per_tree_all", "per_tree_live"):
    for design, fold in designs:
        tr = run(hist, design, fold, hname, rule)
        results[(hname, design, fold, rule)] = tr
        last = tr[-1]
        red_versions = sum(1 for t in tr if not (t[3] and t[4] and t[5]))
        print(f"E7RESULT name=history history={hname} step_rule={rule} design={design} fold={fold} publishes={len(tr)} rows_final={last[2]} stale_rows_final={last[6]} deleted_head_rows_final={last[7]} versions_with_any_check_red={red_versions} i313_syntactic_red={sum(1 for t in tr if not t[3])} count_check_red={sum(1 for t in tr if not t[4])} i930_one_red={sum(1 for t in tr if not t[5])} rows_trajectory={'/'.join(str(t[2]) for t in tr)}")

# 两条路：rebuild 与 point_delete+arrival 逐版逐行相同
def trees(hname, design, fold):
    txg = 3; heads = {65537: 2}; tree = rows_of_this_publish(txg, heads, 0); out = []
    for g, event in enumerate(HISTORIES[hname], start=1):
        prev_txg, txg = txg, txg + 1
        if event[0] == "create_inode": heads[event[1]] += 1
        elif event[0] == "create_clone": heads[event[1]] = heads[65537]
        elif event[0] == "delete_head": del heads[event[1]]
        tree = publish(tree, prev_txg, txg, heads, g, design, fold); out.append(dict(tree))
    return out
ok = True
for hname in HISTORIES:
    a, b = trees(hname, "rebuild", "none"), trees(hname, "point_delete", "arrival")
    diff = sum(1 for x, y in zip(a, b) if x != y)
    ok &= diff == 0
    print(f"E7RESULT name=two_paths history={hname} versions={len(a)} differing_versions={diff}")
print(f"E7RESULT name=done two_paths_ok={int(ok)}")
sys.exit(0 if ok else 3)
