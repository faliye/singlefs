#!/usr/bin/env python3
"""m3-close-c506-r3 辩方腿：独立复核前两轮报出的数。纯标准库，不编译 Rust。
用法：python3 model.py --repo <仓根>
式子都从 kb 条款与 crates 源码读来另写一份，不抄前两轮模型；源码常量先回读对账，对不上退 1。"""
import os, re, sys, pathlib

REPO = pathlib.Path(sys.argv[sys.argv.index("--repo") + 1]) if "--repo" in sys.argv else pathlib.Path(".")
MIB = 1 << 20

def source_const(path, name):
    text = (REPO / path).read_text(encoding="utf-8")
    m = re.search(r"\bconst\s+" + re.escape(name) + r"\s*:\s*\w+\s*=\s*([^;]+);", text)
    if not m:
        raise SystemExit(f"name=source_check result=MISSING const={name} file={path}")
    expr = m.group(1).strip().replace("_", "")
    if not re.fullmatch(r"[0-9+*() ]+", expr):
        raise SystemExit(f"name=source_check result=NOT_LITERAL const={name} expr={expr}")
    return eval(expr)

FMT = "crates/singlefs-format/src/lib.rs"
S_MIN = source_const(FMT, "ROOT_RING_SLOTS_PER_REGION_MINIMUM")
S_MAX = source_const(FMT, "ROOT_RING_SLOTS_PER_REGION_MAXIMUM")
S_MKFS = source_const(FMT, "ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM")
P = source_const(FMT, "ROOT_RING_PRIME_STEP")
CHUNK = source_const(FMT, "ROOT_RING_CHUNK_BYTES")
R = source_const(FMT, "ROOT_RING_REGIONS")
SLOT_BYTES = source_const(FMT, "SLOT_BYTES")
BASE_SLOT = source_const(FMT, "ROOT_RING_BASE_SLOT")
JOURNAL_SLOT = source_const(FMT, "JOURNAL_RING_START_SLOT")
SYSCFG = source_const(FMT, "SYSTEM_CONFIGURATION_SLOT_BYTES")
SENTINEL = source_const(FMT, "ROOT_RECORD_NO_RING_SLOT_PINNED")
SP_MIN = source_const(FMT, "FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES")
BASE = BASE_SLOT * SLOT_BYTES
JOURNAL = JOURNAL_SLOT * SLOT_BYTES
STRIDE = P * CHUNK
DEV = [0, 1, 0]  # 区域归属，D22 已定项 16 第 4 句
print(f"name=source_check result=ok S_min={S_MIN} S_max={S_MAX} S_mkfs={S_MKFS} P={P} chunk={CHUNK} R={R} base={BASE} journal={JOURNAL} syscfg={SYSCFG} sentinel={SENTINEL} sp_min={SP_MIN} stride={STRIDE}")
assert (S_MIN, S_MAX, S_MKFS, P, CHUNK, R, BASE, JOURNAL, STRIDE) == (4, 16, 8, 3, MIB, 3, MIB, 16 * MIB, 3 * MIB)

def region_start(r):
    return BASE + r * STRIDE

def overlap(a, b):
    return a[0] < b[1] and b[0] < a[1]

def target(txg, S):  # D22 已定项 16：区域 = txg mod R，区域内槽 = (txg div R) mod S
    return txg % R, (txg // R) % S

# ---- 一、R1「24/24 判 I-7.17 红、对照 96/96 干净」：按字节别名重算，不挂载 ----
def aliased_pairs(S, sp, width=512):
    """同一块盘上两个不同(区域, 槽)的字节段相交的对。"""
    slots = [(r, j, region_start(r) + j * sp) for r in range(R) for j in range(S)]
    pairs = []
    for a in range(len(slots)):
        for b in range(a + 1, len(slots)):
            ra, ja, xa = slots[a]
            rb, jb, xb = slots[b]
            if DEV[ra] == DEV[rb] and overlap((xa, xa + width), (xb, xb + width)):
                pairs.append(((ra, ja), (rb, jb)))
    return pairs

def red_states(S, sp, txgs, width=512):
    """逐状态（盘上已发布到 txg=t）：把每个发布写到它的物理槽里，再按两种读法判 I-7.17。"""
    pairs = aliased_pairs(S, sp, width)
    reds = 0
    for t in txgs:
        # 物理字节段 -> 最后写进去的 txg；第 0 代种进三个区域的槽 0
        phys = {}
        for r in range(R):
            phys[(DEV[r], region_start(r))] = 0
        for u in range(1, t + 1):
            r, j = target(u, S)
            phys[(DEV[r], region_start(r) + j * sp)] = u
        red = False
        for r in range(R):
            for j in range(S):
                key = (DEV[r], region_start(r) + j * sp)
                hit = [k for k in phys if k[0] == key[0] and overlap((k[1], k[1] + width), (key[1], key[1] + width))]
                for k in hit:
                    u = phys[k]
                    ok = (j == 0) if u == 0 else (target(u, S) == (r, j))
                    red |= not ok
        reds += red
    return reds, len(txgs), len(pairs)

# 状态数：txg 0（刚 mkfs）..72；r1 攻方报的几次挂载后选到的根 txg 取 2、37、46、70，必须落在这个范围里
TXGS = list(range(0, 73))
arms = [("rewritten_S16_512KiB", 16, 512 * 1024), ("S16_4KiB", 16, 4096), ("S4_512KiB", 4, 512 * 1024),
        ("S16_256KiB", 16, 256 * 1024), ("S8_512KiB", 8, 512 * 1024)]
ctrl_clean = 0
for name, S, sp in arms:
    reds, n, pairs = red_states(S, sp, TXGS)
    reds24, n24, _ = red_states(S, sp, [2, 37, 46, 70] + list(range(3, 23)))
    print(f"name=alias arm={name} S={S} spacing={sp} aliased_pairs={pairs} red_states={reds}_of_{n} subset24_red={reds24}_of_{n24}")
    if name != "rewritten_S16_512KiB":
        ctrl_clean += n24 - reds24
print(f"name=alias_control_clean_states_per_24x4 clean={ctrl_clean} expected_if_96_of_96=96")

# ---- 二、R1「S ≤ 85」：钉值 = 3·槽 + 区域，最大 3S − 1，255 是哨兵 ----
max_pin = lambda S: R * (S - 1) + (R - 1)
s_max_pin = max(S for S in range(1, 256) if max_pin(S) < SENTINEL)
print(f"name=pin_bound max_pin_value_formula=3S-1 largest_S_with_pin_below_{SENTINEL}={s_max_pin} pin_at_85={max_pin(85)} pin_at_86={max_pin(86)}")

# ---- 三、R1「粗粒度 S ≤ 25、散开 S ≥ 10」：块数常量从 E50 源码读来 ----
e50 = (REPO / "scripts/experiment-apparatus/e50_ring_slots.rs").read_text(encoding="utf-8")
def e50_const(name):
    m = re.search(r"const\s+" + name + r"\s*:\s*u64\s*=\s*([^;]+);", e50)
    return eval(m.group(1))
COARSE, SCATTERED, MATTERS = e50_const("COARSE_BLOCKS"), e50_const("SCATTERED_BLOCKS"), e50_const("PINNED_MATTERS")
print(f"name=e50_constants coarse_blocks={COARSE} scattered_blocks={SCATTERED} pinned_matters={MATTERS}")
first_coarse_S = min(S for S in range(1, 400) if R * S * COARSE >= MATTERS)
first_scat_S = min(S for S in range(1, 400) if R * S * SCATTERED >= MATTERS)
first_coarse_N = min(N for N in range(1, 2000) if N * COARSE >= MATTERS)
first_scat_N = min(N for N in range(1, 2000) if N * SCATTERED >= MATTERS)
print(f"name=e50_crossing coarse_first_S_reaching_line={first_coarse_S} (S_le_{first_coarse_S - 1}_does_not) scattered_first_S_reaching_line={first_scat_S} coarse_exact_N={first_coarse_N} scattered_exact_N={first_scat_N}")
print(f"name=e50_values S25_coarse={R*25*COARSE} S26_coarse={R*26*COARSE} S9_scattered={R*9*SCATTERED} S10_scattered={R*10*SCATTERED} S16_scattered={R*16*SCATTERED} S16_coarse={R*16*COARSE}")
sampled = [2, 4, 8, 16, 32, 64, 256, 1024]
print(f"name=e50_page_row first_sampled_N_coarse={min(n for n in sampled if n*COARSE >= MATTERS)} first_sampled_N_scattered={min(n for n in sampled if n*SCATTERED >= MATTERS)} N8_coarse={8*COARSE} N8_scattered={8*SCATTERED} N64_coarse={64*COARSE} N64_scattered={64*SCATTERED}")

# ---- 四、R2「26429 / 29744 格逐形状相等、错拒 0」：mkfs 六段两两比 vs 读者新判，再对盘上字节 ----
SP_MAX_READER = BASE - SYSCFG  # 槽 1 整槽落在根环基址之前：槽距 ≤ 1 MiB − 4096（D22 已定项 2 / 已定项 16）

def mkfs_extents(S, sp):  # make_filesystem.rs 的 fixed_structure_extents：系统配置槽 0、1，区域 0..2，journal 环
    ex = [(0, SYSCFG), (sp, sp + SYSCFG)]
    ex += [(region_start(r), region_start(r) + S * sp) for r in range(R)]
    ex.append((JOURNAL, JOURNAL + (768 << 20)))
    return ex

def mkfs_accepts(S, sp, pbs, io_min):
    if pbs > sp:
        return False
    ex = mkfs_extents(S, sp)
    if any(overlap(ex[i], ex[j]) for i in range(len(ex)) for j in range(i + 1, len(ex))):
        return False
    width = max(pbs, io_min)  # I-7.5：每个槽的起始偏移是判定宽度的整数倍
    starts = [0, sp] + [region_start(r) + j * sp for r in range(R) for j in range(S)]
    return all(x % width == 0 for x in starts)

def rule_accepts(S, sp):  # 被判形态的新判：S × 槽距 ≤ P × chunk（P、chunk 已钉成第一版常量）
    if os.environ.get('C506_R3_BREAK') == 'rule_right_side_doubled':  # 弄坏开关，只给证红用
        return S * sp <= 2 * STRIDE
    return S * sp <= STRIDE

def physical_overlap(S, sp, pbs, level):
    """level='extent'：区域整段（S × 槽距，mkfs 清零的那一段）；level='slot'：只算根槽 pbs 宽。逐盘按 0/1/0 归属。"""
    for d in (0, 1):
        segs = [(0, SYSCFG), (sp, sp + SYSCFG), (JOURNAL, JOURNAL + (768 << 20))]
        for r in range(R):
            if DEV[r] != d:
                continue
            if level == "extent":
                segs.append((region_start(r), region_start(r) + S * sp))
            elif level == "trimmed":  # 区域只算真写过的字节：(S−1) × 槽距 + 一个根槽宽
                segs.append((region_start(r), region_start(r) + (S - 1) * sp + pbs))
            else:
                segs += [(region_start(r) + j * sp, region_start(r) + j * sp + pbs) for j in range(S)]
        if any(overlap(segs[i], segs[j]) for i in range(len(segs)) for j in range(i + 1, len(segs))):
            return True
    return False

total = 0
RES = {}
for pbs in (512, 4096):
    sps = list(range(SP_MIN, SP_MAX_READER + 1, pbs))
    acc_rule = acc_mkfs = disagree = false_reject = uncovered_ext = uncovered_slot = 0
    refused = refused_no_slot_overlap = 0
    ext_overlap_all = slot_overlap_all = trim_overlap_all = shapes = 0
    uncovered_trim = over_reject_extent = over_reject_trim = touching = 0
    for sp in sps:
        for S in range(S_MIN, S_MAX + 1):
            shapes += 1
            a, b = rule_accepts(S, sp), mkfs_accepts(S, sp, pbs, pbs)
            acc_rule += a; acc_mkfs += b
            disagree += a != b
            false_reject += b and not a
            eo, so = physical_overlap(S, sp, pbs, "extent"), physical_overlap(S, sp, pbs, "slot")
            to = physical_overlap(S, sp, pbs, "trimmed")
            ext_overlap_all += eo; slot_overlap_all += so; trim_overlap_all += to
            uncovered_trim += a and to
            over_reject_extent += (not a) and not eo
            over_reject_trim += (not a) and not to
            touching += a and S * sp == STRIDE
            uncovered_ext += a and eo
            uncovered_slot += a and so
            if not a:
                refused += 1
                refused_no_slot_overlap += not so
    total += shapes
    print(f"name=shapes pbs={pbs} spacings={len(sps)} shapes={shapes} rule_accepts={acc_rule} mkfs_accepts={acc_mkfs} rule_xor_mkfs={disagree} false_rejects_vs_mkfs={false_reject} rule_accepts_but_extent_overlap={uncovered_ext} rule_accepts_but_slot_overlap={uncovered_slot}")
    print(f"name=shapes_physical pbs={pbs} extent_overlap_shapes={ext_overlap_all} slot_overlap_shapes={slot_overlap_all} rule_refuses={refused} rule_refuses_and_no_slot_overlap={refused_no_slot_overlap} trimmed_overlap_shapes={trim_overlap_all} rule_accepts_but_trimmed_overlap={uncovered_trim}")
    RES[pbs] = dict(shapes=shapes, acc_rule=acc_rule, acc_mkfs=acc_mkfs, disagree=disagree, false_reject=false_reject, uncovered_ext=uncovered_ext,
                    uncovered_slot=uncovered_slot, uncovered_trim=uncovered_trim, ext=ext_overlap_all, slot=slot_overlap_all, trim=trim_overlap_all,
                    refused_no_slot=refused_no_slot_overlap, over_ext=over_reject_extent, touching=touching)
    print(f"name=shapes_over_rejection pbs={pbs} rule_refuses_but_extent_clean={over_reject_extent} rule_refuses_but_trimmed_clean={over_reject_trim} rule_accepts_exactly_touching={touching}")
print(f"name=shapes_total total={total}")

# ---- 五、R2「槽距 629248..786432 写 4」「有效槽距上界 768 KiB」「min 公式的直方图」 ----
def written_S(sp):  # 被判形态：mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋)
    return min(S_MKFS, STRIDE // sp)

for pbs in (512, 4096):
    sps = list(range(SP_MIN, SP_MAX_READER + 1, pbs))
    hist = {}
    for sp in sps:
        hist.setdefault(written_S(sp), []).append(sp)
    rows = [(k, len(v)) for k, v in sorted(hist.items())]
    four = hist[4]
    print(f"name=min_formula pbs={pbs} histogram={rows} spacing_giving_4={four[0]}..{four[-1]} spacing_giving_below_4={min(hist[3])}..{max(hist[3])} any_16={16 in hist}")
four_exact = [sp for sp in range(1, SP_MAX_READER + 1) if STRIDE // sp == 4]
print(f"name=bound_arithmetic S4_largest_spacing={STRIDE // S_MIN}={STRIDE // S_MIN // 1024}KiB real_4_interval_bytes={four_exact[0]}..{four_exact[-1]} KiB_of_upper={four_exact[-1] / 1024} reader_range_top={SP_MAX_READER}")
# 用阈值「< 4 拒」与「< 5 拒」比：自由槽距上 mkfs 会写出哪些 S
for threshold in (4, 5):
    kept = {}
    for sp in range(SP_MIN, SP_MAX_READER + 1, 512):
        w = written_S(sp)
        if w >= threshold:
            kept[w] = kept.get(w, 0) + 1
    print(f"name=threshold refuse_below={threshold} written_values_over_free_spacing={sorted(kept.items())} touches_edge_4={4 in kept} touches_edge_16={16 in kept}")

# ---- 六、X2：D2 已定项 19 的槽距取法下，门槛 < 5 之后还剩什么取值 ----
def slot_spacing_for(io_min):  # 4096 向上取整到 io_min 的整数倍（system_configuration.rs 的 slot_spacing_for）
    return SP_MIN if io_min == 0 else -(-SP_MIN // io_min) * io_min

def reaches_mkfs(S, sp, pbs, io_min):
    return mkfs_accepts(S, sp, pbs, io_min)

for pbs in (512, 4096):
    outcome = {}
    passing_io = []
    for io_min in [0] + list(range(512, 2 * MIB + 1, 512)):
        if io_min and io_min < pbs:
            pass
        sp = slot_spacing_for(io_min)
        if sp > SP_MAX_READER and False:
            continue
        S = written_S(sp)
        under_threshold_4 = S >= 4
        under_threshold_5 = S >= 5
        ok_mkfs = S >= S_MIN and mkfs_accepts(S, sp, pbs, io_min)
        key = (S if ok_mkfs else "mkfs_refuses_by_overlap_or_alignment")
        outcome.setdefault(key, []).append(io_min)
        if ok_mkfs:
            passing_io.append(io_min)
    pow2 = all(x == 0 or (x & (x - 1)) == 0 for x in passing_io)
    print(f"name=d2_19_conforming pbs={pbs} io_min_range=0..{2*MIB} accepted_io_min_count={len(passing_io)} accepted_all_power_of_two_or_zero={pow2} largest_accepted_io_min={max(passing_io)} written_S_values={sorted(k for k in outcome if k != 'mkfs_refuses_by_overlap_or_alignment')} io_min_giving_6={outcome.get(6)} written_S_equals_4_or_below_among_accepted={any(isinstance(k,int) and k<=4 for k in outcome)}")
# io_min = 1 MiB：被判形态给的是 3，小于 4 与小于 5 都拒；但 mkfs 的「固定结构两两不重叠」在这之前就拒
sp1m = slot_spacing_for(MIB)
print(f"name=io_min_1MiB spacing={sp1m} formula_gives={written_S(sp1m)} refused_by_threshold_4={written_S(sp1m) < 4} refused_by_threshold_5={written_S(sp1m) < 5} mkfs_extent_overlap_at_S4={not mkfs_accepts(4, sp1m, 512, MIB)} reader_range_top={SP_MAX_READER} spacing_over_range={sp1m > SP_MAX_READER}")

# ---- 七、grep 计数：77 处直接调用、读者读不读 P / chunk、GPU 区间字面量 ----
rs_files = sorted((REPO / "crates").rglob("*.rs"))
needle = "RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM"
per_file = {f: f.read_text(encoding="utf-8", errors="replace").count(needle) for f in rs_files}
print(f"name=grep_at_make_filesystem direct_calls={sum(per_file.values())} files={sum(1 for v in per_file.values() if v)}")
def lines_matching(rel, pattern):
    text = (REPO / rel).read_text(encoding="utf-8").splitlines()
    return [(i + 1, l.strip()) for i, l in enumerate(text) if re.search(pattern, l)]
for rel in ("crates/singlefs-core/src/recovery.rs", "crates/singlefs-core/src/system_configuration.rs"):
    hits = lines_matching(rel, r"\b363\b|\b367\b|ROOT_RING_PRIME_STEP|ROOT_RING_CHUNK_BYTES")
    print(f"name=grep_reader_of_P_chunk file={rel} hits={[(n, t[:90]) for n, t in hits]}")
hits = lines_matching("crates/singlefs-checker/src/image.rs", r"read_u32\(slot, 36[37]\)")
print(f"name=grep_reader_of_P_chunk file=crates/singlefs-checker/src/image.rs hits={[(n, t[:70]) for n, t in hits]}")
wgsl_sites = []
for f in sorted((REPO / "crates/singlefs-checker-tier/src").glob("*.wgsl")):
    for i, l in enumerate(f.read_text(encoding="utf-8").splitlines()):
        if re.search(r"slots_per_region\s*(<|>=)\s*4u", l):
            wgsl_sites.append((f.name, i + 1))
        if re.search(r"const (MAX_ROOTS|C_MAX_CANDIDATES): u32 = 48u", l):
            wgsl_sites.append((f.name, i + 1, "48u"))
print(f"name=grep_wgsl_s_interval_and_48 sites={wgsl_sites}")


# ---- 八、前两轮报出的数 vs 这里算出的数（原样给 reported / computed，机器判 match） ----
claims = [
    ("r1_alias_24_of_24_red", "24/24", f"{red_states(16, 512 * 1024, [2, 37, 46, 70] + list(range(3, 23)))[0]}/24"),
    ("r1_controls_96_of_96_clean", "96/96", f"{ctrl_clean}/96"),
    ("r1_S_le_85", "85", str(s_max_pin)),
    ("r1_coarse_S_le_25_not_over_line", "25", str(first_coarse_S - 1)),
    ("r1_scattered_S_ge_10_over_line", "10", str(first_scat_S)),
    ("r2_shapes_pbs512", "26429", str(RES[512]["shapes"])),
    ("r2_shapes_total", "29744", str(total)),
    ("r2_rule_accepts_pbs512", "9411", str(RES[512]["acc_rule"])),
    ("r2_rule_accepts_pbs4096", "1184", str(RES[4096]["acc_rule"])),
    ("r2_rule_xor_mkfs_total", "0", str(RES[512]["disagree"] + RES[4096]["disagree"])),
    ("r2_false_rejects_total", "0", str(RES[512]["false_reject"] + RES[4096]["false_reject"])),
    ("r2_uncovered_overlap_total", "0", str(sum(RES[k][f] for k in RES for f in ("uncovered_ext", "uncovered_slot", "uncovered_trim")))),
    ("r2_extent_overlap_pbs512", "8966", str(RES[512]["ext"])),
    ("r2_refused_no_slot_overlap_pbs512_pbs4096", "13784+1717", f"{RES[512]['refused_no_slot']}+{RES[4096]['refused_no_slot']}"),
    ("r2_over_rejection_extent_pbs512", "8052", str(RES[512]["over_ext"])),
    ("r2_trimmed_overlap_pbs512_is_26429_minus_18733", "7696", str(RES[512]["trim"])),
    ("r2_spacing_giving_4_from", "629248", str(min(sp for sp in range(SP_MIN, SP_MAX_READER + 1, 512) if written_S(sp) == 4))),
    ("r2_spacing_giving_4_to", "786432", str(max(sp for sp in range(SP_MIN, SP_MAX_READER + 1, 512) if written_S(sp) == 4))),
    ("r2_effective_spacing_upper_KiB", "768", str(STRIDE // S_MIN // 1024)),
    ("r2_at_make_filesystem_direct_calls", "77", str(sum(per_file.values()))),
    ("r2_at_make_filesystem_files", "44", str(sum(1 for v in per_file.values() if v))),
]
bad = 0
for name, reported, computed in claims:
    ok = reported == computed
    bad += not ok
    print(f"name=claim {name} reported={reported} computed={computed} match={str(ok).lower()}")
print(f"name=claims_summary claims={len(claims)} mismatched={bad}")
sys.exit(1 if bad else 0)
