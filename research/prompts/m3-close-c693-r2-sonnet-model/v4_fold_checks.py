#!/usr/bin/env python3
"""正推腿 m3-close-c693-r2 V4：点删与墓碑——开前端之后「上一代的行离开」落成点删、排空时朴素取最大，今天的检查红不红；换成什么形的检查红。
同时：删一个可写头时它的台阶行怎么离开，三种落法各自在哪些检查下红。

行形、台阶认法、两种落法（rebuild 整树重建 / point_delete 前端点删 + 排空折叠）照第一轮攻方 `research/prompts/m3-close-c693-r1-opus-model/w4_frontend_fold.py`
（本脚本是照它的行形与规则重写的，不 import 它；折叠与台阶规则两份一致，所以「与第一轮攻方模型同一份规则」，不算独立路径）。
今天的检查（照 crates 写）：
  i313_syntactic  checker walk.rs：标签 12、设备段 0xFFFF_FFFF、代段早于这一版 txg 的行跳过，其余行代段 = txg
  count_check     recovery.rs accounting_entry_count_is_three_plus_six_per_device：台阶行（同语法）之外恰 3 + 6d
  i930_one        checker I-9.30 ①：水位（标签 12 代段最大那一行，不分树）不小于每一级台阶的 H
本脚本新加的三个候选检查（都是推出来的形，没进 crates，也没进 checker）：
  steps_strict    每个 (标签 12, 树) 下，代段早于这一版 txg 的行按代段升序，值严格递增（台阶的定义是水位变了的那次发布：相邻两级 H 相等就不是两级台阶）
  live_only       标签 12 的每一行的树 ID 都是这一版树表里还活着的头
  i930_per_tree   I-9.30 ① 按树分：每棵树的现行水位（该树代段最大那一行）不小于该树每一级台阶的 H
模型自带的真值（不是检查，checker 看不到）：true_missing = D5 意义下该有而不在的台阶行数；stale = 既不是这一版的行也不是真台阶的行数。
自证：rebuild 与 point_delete+arrival 在每段历史每一版上逐行相同；弄坏开关 V4_BREAK=arrival_keeps_tomb_target（arrival 折叠不删墓碑指的 key）必须让「两条路逐行相同」红（退 3）。
"""
import os, sys

D = 2
POOL, PER_DEVICE, WM, NODEV = [4, 6], [1, 2, 3, 5, 10, 11], 12, 0xFFFFFFFF
BREAK = os.environ.get("V4_BREAK", "")


def rows_of_this_publish(txg, heads, counter):
    rows = {}
    for tag in POOL:
        rows[(tag, 0, NODEV, txg)] = counter
    for dev in range(D):
        for tag in PER_DEVICE:
            rows[(tag, 0, dev, txg)] = counter + dev
    for head, h in heads.items():
        rows[(WM, head, NODEV, txg)] = h
    return rows


def crates_steps(tree, txg):
    steps = sorted(((k, v) for k, v in tree.items() if k[0] == WM and k[3] < txg), key=lambda kv: kv[0][3])
    this = [(k, v) for k, v in tree.items() if k[0] == WM and k[3] == txg]
    if this and (not steps or steps[-1][1] != this[0][1]):
        steps.append(this[0])
    return dict(steps)


def per_tree_steps(tree, txg, live):
    out = {}
    for head in sorted({k[1] for k in tree if k[0] == WM}):
        if live is not None and head not in live:
            continue
        out.update(crates_steps({k: v for k, v in tree.items() if k[0] == WM and k[1] == head}, txg))
    return out


def steps_by_rule(tree, txg, rule, live):
    if rule == "crates":
        return crates_steps(tree, txg)
    return per_tree_steps(tree, txg, None if rule == "per_tree_all" else live)


def drain(leaf, frontend, fold):
    out = dict(leaf)
    by_key = {}
    for key, value in frontend:
        by_key.setdefault(key, []).append(value)
    for key, values in by_key.items():
        if fold == "arrival":
            if values[-1] is None:
                if BREAK != "arrival_keeps_tomb_target":
                    out.pop(key, None)
            else:
                out[key] = values[-1]
        else:   # naive_max：单调统计量（标签 12）把墓碑当 0、与叶上现值取最大
            if key[0] == WM:
                folded = max([out.get(key, 0)] + [0 if v is None else v for v in values])
                out[key] = folded
            elif values[-1] is None:
                out.pop(key, None)
            else:
                out[key] = values[-1]
    return out


def publish(previous, prev_txg, txg, heads, counter, design, fold, rule):
    new_rows = rows_of_this_publish(txg, heads, counter)
    steps = steps_by_rule(previous, prev_txg, rule, set(heads))
    if design == "rebuild":
        tree = dict(steps)
        tree.update(new_rows)
        return tree
    keep = set(steps) | set(new_rows)
    frontend = [(k, None) for k in sorted(previous) if k not in keep] + sorted(new_rows.items())
    return drain(previous, frontend, fold)


def checks(tree, txg, live, true_steps):
    i313 = all(r[3] == txg for r in tree if not (r[0] == WM and r[2] == NODEV and r[3] < txg))
    non_steps = sum(1 for r in tree if not (r[0] == WM and r[3] < txg))
    count_ok = non_steps == 3 + 6 * D + (len(live) - 1)
    wm = [(r, v) for r, v in tree.items() if r[0] == WM]
    i930_one = all(max(wm, key=lambda rv: rv[0][3])[1] >= v for r, v in crates_steps(tree, txg).items())
    strict, per_tree_ok = True, True
    for head in {r[1] for r, _ in wm}:
        sub = sorted(((r[3], v) for r, v in wm if r[1] == head and r[3] < txg))
        strict &= all(a[1] < b[1] for a, b in zip(sub, sub[1:]))
        cur = max(((r[3], v) for r, v in wm if r[1] == head), default=(0, 0))[1]
        per_tree_ok &= all(cur >= v for _, v in sub)
    live_only = all(r[1] in live for r, _ in wm)
    stale = sum(1 for r in tree if r[3] != txg and r not in true_steps)
    missing = sum(1 for s in true_steps if s not in tree and s[3] < txg)
    return dict(i313=i313, count=count_ok, i930_one=i930_one, steps_strict=strict, live_only=live_only, i930_per_tree=per_tree_ok), stale, missing


def run(history, design, fold, rule):
    txg, heads = 3, {65537: 2}
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
        flags, stale, missing = checks(tree, txg, set(heads), true_steps)
        trace.append((g, txg, len(tree), flags, stale, missing, dict(tree)))
    return trace


HISTORIES = {
    "single_head_two_inode_publishes": [("create_inode", 65537) if g in (3, 7) else ("empty",) for g in range(1, 21)],
    "single_head_inode_every_other_publish": [("create_inode", 65537) if g % 2 == 0 else ("empty",) for g in range(1, 21)],
    "clone_created_then_deleted": [("create_clone", 65549) if g == 2 else ("create_inode", 65549) if g in (4, 5) else ("delete_head", 65549) if g == 10 else ("empty",) for g in range(1, 21)],
}
CASES = [("rebuild", "none"), ("point_delete", "arrival"), ("point_delete", "naive_max")]
NAMES = ["i313", "count", "i930_one", "steps_strict", "live_only", "i930_per_tree"]
for hname, hist in HISTORIES.items():
    for rule in ("crates", "per_tree_all", "per_tree_live"):
        for design, fold in CASES:
            tr = run(hist, design, fold, rule)
            red = {n: sum(1 for t in tr if not t[3][n]) for n in NAMES}
            first = {n: next((t[0] for t in tr if not t[3][n]), 0) for n in NAMES}
            print(f"E7RESULT name=history history={hname} step_rule={rule} design={design} fold={fold} rows_final={tr[-1][2]} stale_final={tr[-1][4]} true_steps_missing_final={tr[-1][5]} "
                  + " ".join(f"{n}_red={red[n]}" for n in NAMES) + " first_red_publish=" + ",".join(f"{n}:{first[n]}" for n in NAMES))
ok = True
for hname, hist in HISTORIES.items():
    for rule in ("crates", "per_tree_all", "per_tree_live"):
        a, b = run(hist, "rebuild", "none", rule), run(hist, "point_delete", "arrival", rule)
        diff = sum(1 for x, y in zip(a, b) if x[6] != y[6])
        ok &= diff == 0
        print(f"E7RESULT name=two_paths history={hname} step_rule={rule} versions={len(a)} differing_versions={diff}")
print(f"E7RESULT name=done two_paths_ok={int(ok)}")
sys.exit(0 if ok else 3)
