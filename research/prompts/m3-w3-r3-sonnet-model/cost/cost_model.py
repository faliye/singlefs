#!/usr/bin/env python3
"""m3-w3-r3 正推腿的代价小模型：纯算术加一个对着真规划器对过账的 Python 复刻（planner.py）。不依赖 crates/ 的编译；
格式常量的出处与冻结树里的行由 check_constants.py 逐个对。输出每行以 COST 开头，rerun.sh cost 把它落进 out/cost-model.out。
口径：
- 「步行」= 台阶行 (代, 水位)，34 字节一条，记账条目（C2aL）。
- 「发号记录」= C3rL 的 (号, 出生代) 16 字节一条，住谱系树同形的容器树（根恒为码 2、孩子是码 3 容器，根满长高）。
- 「谱系记录」= 今天 16 字节的 (T_rb, T_old)；三个形态都多带 8 字节（C4g 带 W，C2aL 带 H_cur，C3rL 带 count）→ 24 字节。
- 单元大小：码 2 节点 16384 字节，码 3 容器 32768 字节；两份副本各写一次（今天的池是两盘）。
"""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from planner import Tree
from probe_compare import rows, run as replica_run   # 复用同一套 15 行的造法

NODE = 16384; DATA_UNIT = 32768; REPLICAS = 2
HDR = lambda key_width: 137 + key_width          # 码 2 头（含预留）；22→159，8→145（unit.rs:143 的实现，冻结树里 key 宽 22 对 159）
LEAF_CAP = (NODE - HDR(22)) // 34                 # 477
INT_CAP = (NODE - HDR(22)) // 108                 # 150
CONT_ROOT_FANOUT = (NODE - HDR(8)) // 120         # 135
LIN_RECORD, LIN_RECORD_NEW = 16, 24
CONT_PACKED_HDR = 136
def cont_records(rec): return (DATA_UNIT - CONT_PACKED_HDR) // rec
assert (LEAF_CAP, INT_CAP, CONT_ROOT_FANOUT) == (477, 150, 135)
assert cont_records(16) == 2039 and cont_records(24) == 1359 and cont_records(40) == 815

def p(tag, **kw):
    print("COST", tag, " ".join(f"{k}={v}" for k, v in kw.items()))

# ---------- 一、记账树（C2aL 的台阶行住这里）：规划器复刻对着真规划器逐行对过账（probe_compare.py）----------
p("constants", leaf_entries=LEAF_CAP, internal_entries=INT_CAP, container_root_fanout=CONT_ROOT_FANOUT,
  lineage_container_records_16B=cont_records(16), lineage_container_records_24B=cont_records(24))

def right_edge_levels(grows, leaf_cap=LEAF_CAP, internal_cap=INT_CAP, start=15):
    """只模拟最右那条路径的占用：返回每次 grow 之后的 (高度, 是不是这次分裂了)。start = 一代 15 行（14 行每发布换代的在最左叶，水位行在最右）。"""
    levels = [start]; out = []
    for _ in range(grows):
        i = 0; split = False; levels[0] += 1
        while True:
            cap = leaf_cap if i == 0 else internal_cap
            if levels[i] > cap:
                split = True
                n = levels[i]; levels[i] = n - (n + 1) // 2        # 右半留在最右路径上，左半 ⌈n/2⌉ 条不再碰
                if i + 1 == len(levels): levels.append(2)
                else: levels[i + 1] += 1
                i += 1; continue
            break
        out.append((len(levels), split))
    return out

def first_growth_to(height, **kw):
    n = 0; levels = [kw.get("start", 15)]
    lc = kw.get("leaf_cap", LEAF_CAP); ic = kw.get("internal_cap", INT_CAP)
    while len(levels) < height:
        n += 1; i = 0; levels[0] += 1
        while True:
            cap = lc if i == 0 else ic
            if levels[i] > cap:
                m = levels[i]; levels[i] = m - (m + 1) // 2
                if i + 1 == len(levels): levels.append(2)
                else: levels[i + 1] += 1
                i += 1; continue
            break
    return n

# 复刻 vs 右边缘模拟：容量压小的几档（树高到 7 以上）逐发布对高度；产品容量对前 40000 个发布的高度
def check_edge_model_against_replica():
    # 一代 15 行里 14 行每发布换代、都落在最左叶：右边缘模拟只在这 14 行装得进最左叶切开后的左半（叶容量 ≥ 30）时成立，所以压小的几档取叶 40、60
    for (lc, ic, pubs, ce) in [(40, 4, 6000, 1), (60, 3, 6000, 1), (40, 2, 6000, 1), (477, 150, 40000, 1)]:
        t = Tree(lc, ic); kept = False; prev = 0; edge = right_edge_levels(pubs, lc, ic, 15); bad = 0
        heights = []
        for k in range(1, pubs + 1):
            gen = 2 + k
            deleted = [] if k == 1 else [r for r in rows(prev) if not (r == (12, 65537, 0xFFFFFFFF, prev) and kept)]
            t.publish(sorted(deleted), sorted(rows(gen)))
            heights.append(t.root.level + 1); kept = True; prev = gen
        # 右边缘模拟的第 j 次 grow 对应第 j+1 个发布之后（第 1 个发布是起点，自带 15 行）
        for k in range(2, pubs + 1):
            if heights[k - 1] != edge[k - 2][0]: bad += 1
        p("edge-model-vs-replica", leaf_cap=lc, internal_cap=ic, publishes=pubs, heights_compared=pubs - 1, different=bad)
        assert bad == 0
check_edge_model_against_replica()

N2 = first_growth_to(2); N3 = first_growth_to(3); N4 = first_growth_to(4); N5 = first_growth_to(5)
p("accounting-height-thresholds", history_rows_when_height_2=N2, history_rows_when_height_3=N3, history_rows_when_height_4=N4, history_rows_when_height_5=N5,
  note="history_rows = 第一次到这一高度时已经留下的台阶行数（不含当代那一行）；叶从中间切，只往右追加时每片叶只装一半")
# 产品容量下复刻的逐发布重写数（稳态 2h−1，分裂那一次多 1–2 个）
rep_hist = {}
t = Tree(LEAF_CAP, INT_CAP); kept = False; prev = 0
for k in range(1, 40001):
    gen = 2 + k
    deleted = [] if k == 1 else [r for r in rows(prev) if not (r == (12, 65537, 0xFFFFFFFF, prev) and kept)]
    rw = t.publish(sorted(deleted), sorted(rows(gen)))
    h = t.root.level + 1
    rep_hist.setdefault(h, []).append(rw)
    kept = True; prev = gen
for h, v in sorted(rep_hist.items()):
    from collections import Counter
    p("accounting-rewritten-per-publish-replica", height=h, publishes=len(v), rewritten_distribution=dict(sorted(Counter(v).items())))
assert Counter(rep_hist[1]) == Counter({1: len(rep_hist[1])}) and set(rep_hist[2]) <= {3, 4, 5} and max(Counter(rep_hist[2]), key=Counter(rep_hist[2]).get) == 3
steady = {h: 2 * h - 1 if h >= 2 else 1 for h in range(1, 6)}
p("accounting-steady-rewritten", **{f"h{h}": v for h, v in steady.items()})

# 每个台阶行数 N 下：高度、每发布重写的记账节点数、比今天（1 个）多几个单元、多写几次、每发布写的字节
def height_of_history(n):
    levels = right_edge_levels(n)
    return levels[-1][0] if levels else 1
for n in [0, 100, 462, 463, 1000, 5000, 20000, 35850, 100000, 1_000_000, 10_000_000]:
    h = 1 if n < N2 else (2 if n < N3 else (3 if n < N4 else (4 if n < N5 else 5)))
    nodes = steady[h]; extra_units = nodes - 1
    p("accounting-cost-by-history-rows", history_rows=n, height=h, rewritten_nodes_per_publish=nodes, extra_units=extra_units,
      extra_writes=extra_units * REPLICAS, bytes_written_per_publish=nodes * NODE * REPLICAS, bytes_extra=extra_units * NODE * REPLICAS)
# 台阶行在盘上占的字节：叶装 477 条、只往右追加时每片叶只装约一半
half_leaf = (LEAF_CAP + 1) // 2
p("accounting-storage-per-step-row", entry_bytes=34, leaf_rows_when_appending=half_leaf, bytes_per_row_one_copy=round(NODE / half_leaf, 1),
  bytes_per_row_two_copies=round(REPLICAS * NODE / half_leaf, 1))
# 保留 I-3.13 的那一路（每发布把全部台阶行换成这一代的代）：整段台阶行每发布全删全插
def rewrite_all_cost(n):
    t = Tree(LEAF_CAP, INT_CAP); gen = 3; t.touched = set()
    for r in rows(gen): t.insert(r)
    hist = [(12, 65537, 0xFFFFFFFF, g) for g in range(10 ** 6, 10 ** 6 + n)]
    t.touched = set()
    for k in hist: t.insert(k)
    nodes, leaves, h = t.stats()
    # 下一发布：全部台阶行换成新代（删 n+15 个、插 n+15 个）
    gen2 = gen + 1; olds = sorted(list(hist) + rows(gen)); news = sorted([(12, 65537, 0xFFFFFFFF, g + 10 ** 7) for g in range(10 ** 6, 10 ** 6 + n)] + rows(gen2))
    rw = t.publish(olds, news)
    return nodes, h, rw
for n in [100, 462, 5000]:
    nodes, h, rw = rewrite_all_cost(n)
    p("accounting-keep-I-3.13-route", history_rows=n, tree_nodes_before=nodes, height_before=h, rewritten_nodes_in_next_publish=rw)

# ---------- 二、容器树（C3rL 的发号记录、C4g 的谱系记录住这里；根恒为码 2，孩子是码 3 容器，末尾分裂、左边的节点一个不动）----------
def container_shape_after(containers, fanout=CONT_ROOT_FANOUT):
    levels = []
    for _ in range(containers):
        i = 0
        while True:
            if i >= len(levels): levels.append([1]); break
            if levels[i][-1] < fanout: levels[i][-1] += 1; break
            levels[i].append(1)
            if i + 1 == len(levels): levels.append([2]); break
            i += 1
    return levels
# 与 container_tree.rs:after_a_container_is_opened_at_the_right_end 的文字逐条对：压小扇出的几个点
assert container_shape_after(1, 2) == [[1]] and container_shape_after(2, 2) == [[2]] and container_shape_after(3, 2) == [[2, 1], [2]]
assert container_shape_after(4, 2) == [[2, 2], [2]] and container_shape_after(5, 2) == [[2, 2, 1], [2, 1], [2]]
# 对着真的 ContainerTreeShape::after_a_container_is_opened_at_the_right_end 的探针输出（W3SHAPE 行）逐行比：每层节点数
if len(sys.argv) > 1 and os.path.exists(sys.argv[1]):
    import re
    seen = bad = 0
    for line in open(sys.argv[1], encoding="utf-8"):
        m = re.match(r"W3SHAPE fanout=(\d+) containers=(\d+) root_level=(\d+) nodes_per_level=\[([\d, ]*)\]", line)
        if not m: continue
        fan, c = int(m[1]), int(m[2]); want = [int(x) for x in m[4].split(",")]
        mine = [len(lv) for lv in container_shape_after(c, fan)]
        seen += 1
        if mine != want or int(m[3]) != len(mine): bad += 1; print("容器树形状不同", fan, c, mine, want)
    p("container-shape-vs-real-probe", lines=seen, different=bad); assert seen > 0 and bad == 0
else:
    p("container-shape-vs-real-probe", note="没给探针输出，跳过这一对账")
def container_levels_for(records, rec_bytes):
    c = max(1, -(-records // cont_records(rec_bytes)))
    # 容器数 → 码 2 层数：左边满、只在最右端新开 ⇒ 层数 = ⌈log_135 c⌉（c ≤ 135 时 1 层）
    L = 1; cap = CONT_ROOT_FANOUT
    while c > cap: L += 1; cap *= CONT_ROOT_FANOUT
    return c, L
# 闭式对着逐个新开容器的形状模拟再对一遍（扇出压到 5，层数到 4）
for fan in (2, 3, 5):
    shapes = {}
    for c in range(1, 200):
        lv = container_shape_after(c, fan); shapes[c] = len(lv)
        L = 1; cap = fan
        while c > cap: L += 1; cap *= fan
        assert L == len(lv), (fan, c, L, len(lv))
p("container-tree-closed-form-vs-shape-simulation", fanouts="2,3,5", containers_up_to=199, result="层数闭式 ⌈log_fanout 容器数⌉ 与逐个新开容器的形状模拟相等")
for rec_bytes, name in [(16, "issuance-record-16B(C3rL 每个新 inode 一条)"), (24, "issuance-range-record-24B(C3rL 每次建 inode 的发布一条)"), (24, "lineage-record-24B(三个形态的谱系记录)")]:
    for n in [1000, 2039, 100_000, 275_265, 275_266, 1_000_000, 37_000_000, 10 ** 9]:
        c, L = container_levels_for(n, rec_bytes)
        p("container-tree-cost", record=name, records=n, containers=c, code2_levels=L, rewritten_units_per_append=1 + L,
          bytes_written_per_append=(DATA_UNIT + L * NODE) * REPLICAS)
p("container-tree-thresholds", containers_for_height_1=CONT_ROOT_FANOUT, records_for_height_1_16B=CONT_ROOT_FANOUT * cont_records(16),
  records_for_height_2_16B=CONT_ROOT_FANOUT ** 2 * cont_records(16), records_for_height_1_24B=CONT_ROOT_FANOUT * cont_records(24),
  records_for_height_2_24B=CONT_ROOT_FANOUT ** 2 * cont_records(24))
p("container-tree-storage-per-record", bytes_one_copy_16B=round(DATA_UNIT / cont_records(16), 2), bytes_two_copies_16B=round(REPLICAS * DATA_UNIT / cont_records(16), 2),
  bytes_two_copies_24B=round(REPLICAS * DATA_UNIT / cont_records(24), 2))

# ---------- 三、各形态每次发布多写什么（对着今天：记账 1 片叶 + 其余；谱系容器只在回退那次重写）----------
def accounting_extra(history_rows):
    h = 1 if history_rows < N2 else (2 if history_rows < N3 else (3 if history_rows < N4 else 4))
    return steady[h] - 1, h
def c3rl_extra(records):
    c, L = container_levels_for(records, 16)
    return 1 + L, L          # 容器 1 + 码 2 层数 L；每个建 inode 的发布
for creates in [100, 462, 1000, 10000, 100000, 1000000]:
    ae, ah = accounting_extra(creates)
    ce, cl = c3rl_extra(creates)
    p("per-create-publish-extra", creates_so_far=creates,
      C2aL_extra_units=ae, C2aL_accounting_height=ah, C2aL_extra_writes=ae * REPLICAS,
      C3rL_extra_units=ce, C3rL_code2_levels=cl, C3rL_extra_writes=ce * REPLICAS,
      C4g_extra_units=0, C4g_extra_writes=0)
p("per-rollback-publish-extra", C4g="0 单元（谱系容器本来每次回退都重写）；记录 16→24 字节，容器装 2039→1359 条", C2aL="0 单元；记录 +8 字节；跳升那次多一条台阶行（记账树照上表）",
  C3rL="0 单元；记录 +8 字节")

# ---------- 四、崩溃状态数（E77 闭式 1 + Σ(2^|段| − 1)）：基数取 E142 登记的第一个文件事务流 24+2+1+2 ----------
def closed(segs): return 1 + sum((1 << s) - 1 for s in segs)
base = [24, 2, 1, 2]
p("crash-states", base_segments="24+2+1+2", base_closed_form=closed(base))
for m in [0, 1, 2, 3, 4, 6]:
    segs = [24 + 2 * m] + base[1:]
    p("crash-states-with-extra-units", extra_units=m, extra_writes=2 * m, segments="+".join(map(str, segs)), closed_form=closed(segs), ratio_to_base=round(closed(segs) / closed(base), 1))
assert closed(base) == 16777223 and closed([28, 2, 1, 2]) == 268435463

# ---------- 五、盘上增长（两份副本）与一次检查要读的量 ----------
for n in [10 ** 4, 10 ** 6, 10 ** 8]:
    c3_c, c3_L = container_levels_for(n, 16)
    c3_bytes = c3_c * DATA_UNIT * REPLICAS
    c2_leaves = -(-n // half_leaf)
    p("growth-after-n-create-events", n=n,
      C2aL_step_rows_if_one_publish_per_create=n, C2aL_leaf_units=c2_leaves, C2aL_bytes_two_copies=c2_leaves * NODE * REPLICAS,
      C3rL_records=n, C3rL_containers=c3_c, C3rL_bytes_two_copies=c3_bytes,
      C4g_bytes=0, note="C2aL 按每次建 inode 都是一次独立发布（最坏）；一次发布建多个 inode 时 C2aL 少、C3rL 仍每个 inode 一条")
for r in [1000, 100_000]:
    p("growth-after-n-rollbacks", n=r, lineage_records=r, containers_24B=-(-r // cont_records(24)), bytes_two_copies_24B=-(-r // cont_records(24)) * DATA_UNIT * REPLICAS,
      bytes_two_copies_16B_today=-(-r // cont_records(16)) * DATA_UNIT * REPLICAS)
p("checker-read-per-image", C2aL="全部台阶行（34 字节 × 行数，一份副本）", C3rL="全部发号记录（16 字节 × 创建过的 inode 数，一份副本）；还要拿最新树里每条记录去查 (号, 出生代) 在不在", C4g="全部谱系记录（24 字节 × 回退次数）")

# ---------- 六、台阶压不压得下去（Q4）：两行并成一行会让夹在中间的合法对象判红 ----------
def cut_red(rows_, tree): return any(h > n and b > g for (n, b) in tree for (g, h) in rows_)
rows_two = [(10, 5), (20, 9)]                    # (g1,H1)=(10,5)、(g2,H2)=(20,9)：号 5..8 是 g1 之后、g2 之前发的
legal_object = [(6, 15)]                          # 号 6 出生代 15（10 < 15 ≤ 20），合法
assert not cut_red(rows_two, legal_object)
merged = [(10, 9)]                                # 并成 (g1, H2)
assert cut_red(merged, legal_object)
p("step-rows-cannot-be-merged", example="两行 (10,5),(20,9)；合法对象 (号 6, 出生代 15)；并成 (10,9) 后 ② 判红", rows_kept_per_watermark_change=1,
  rows_covered_by_an_earlier_row=0, reason="水位沿时间线严格上升，没有一行被更早的行盖住")
print("COST end")
