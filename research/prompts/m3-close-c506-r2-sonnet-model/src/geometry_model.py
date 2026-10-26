#!/usr/bin/env python3
"""m3-close-c506-r2 正推腿的纯算模型（标准库，无随机）。

只从条款与 layout 字节表推「应该是什么」，不 import 也不抄 crates/ 的代码：
  区域 r 起点 = 基址 1 MiB + r × P × chunk（P = 3、chunk = 1 MiB）；区域 r 在盘 [0, 1, 0][r] 上；
  区域长 = S × 槽距；系统配置槽 0、1 在 0、槽距，各宽 4096；journal 环从 16 MiB 起，两块盘互为镜像。
被判的一判：S × 槽距 <= P × chunk。
输出每行 name=... 键值，逐行可 diff。
"""
import sys

MIB = 1 << 20
KIB = 1 << 10
BASE = 1 * MIB
P = 3
CHUNK = 1 * MIB
STRIDE = P * CHUNK
JOURNAL_START = 16 * MIB
JOURNAL_RING_DEFAULT = 768 * MIB
SC_WIDTH = 4096
ASSIGN = [0, 1, 0]
PBS = 512
S_MIN, S_MAX = 4, 16
SPACING_MIN = 4096
SPACING_MAX = BASE - SC_WIDTH  # 条款：槽 1 整槽落在根环基址之前


def region_start(r, stride=STRIDE, base=BASE):
    return base + r * stride


def extents_per_disk(S, d, stride=STRIDE, byte_exact=False):
    """{盘: [(名字, 起, 止)]}。byte_exact=True 时区域只算真写过的字节 (S-1)*d + pbs。"""
    length = (S - 1) * d + PBS if byte_exact else S * d
    disks = {0: [], 1: []}
    for disk in (0, 1):
        disks[disk].append(("sc0", 0, SC_WIDTH))
        disks[disk].append(("sc1", d, d + SC_WIDTH))
        disks[disk].append(("journal", JOURNAL_START, JOURNAL_START + JOURNAL_RING_DEFAULT))
    for r in range(3):
        s = region_start(r, stride)
        disks[ASSIGN[r]].append((f"R{r}", s, s + length))
    return disks


def overlap(a, b):
    return a[1] < b[2] and b[1] < a[2]


def physical_overlaps(S, d, stride=STRIDE, byte_exact=False):
    out = []
    for disk, items in extents_per_disk(S, d, stride, byte_exact).items():
        for i in range(len(items)):
            for j in range(i + 1, len(items)):
                if overlap(items[i], items[j]):
                    out.append((disk, items[i][0], items[j][0]))
    return out


def mkfs_today_overlaps(S, d):
    """入库 mkfs 的判法（读 make_filesystem.rs fixed_structure_extents 之后照式子写）：六段不分盘两两比。"""
    items = [("sc0", 0, SC_WIDTH), ("sc1", d, d + SC_WIDTH)]
    items += [(f"R{r}", region_start(r), region_start(r) + S * d) for r in range(3)]
    items.append(("journal", JOURNAL_START, JOURNAL_START + JOURNAL_RING_DEFAULT))
    return [(a[0], b[0]) for i, a in enumerate(items) for b in items[i + 1:] if overlap(a, b)]


def spacing_in_reader_range(d):
    return SPACING_MIN <= d <= SPACING_MAX


def rule_accepts(S, d, stride=STRIDE):
    """被判的形态：S × 槽距 <= P × chunk，叠在已有的槽距区间判上。"""
    return spacing_in_reader_range(d) and S * d <= stride


def spacings():
    return range(SPACING_MIN, SPACING_MAX + 1, 512)


def shapes():
    return [(S, d) for S in range(S_MIN, S_MAX + 1) for d in spacings()]


def emit(**kv):
    print("name=" + kv.pop("name") + "".join(f" {k}={v}" for k, v in kv.items()))


def w1_probe_cells():
    for S, d in [(16, 512 * KIB), (8, 512 * KIB), (16, 256 * KIB), (12, 256 * KIB), (6, 512 * KIB), (4, 512 * KIB), (8, 384 * KIB), (8, 384 * KIB + 512)]:
        emit(name="w1_cell", S=S, spacing=d, region_bytes=S * d,
             physical_full=("overlap:" + "|".join(f"d{x[0]}:{x[1]}/{x[2]}" for x in physical_overlaps(S, d))) if physical_overlaps(S, d) else "clean",
             physical_byte_exact="overlap" if physical_overlaps(S, d, byte_exact=True) else "clean",
             mkfs_today="refuse" if mkfs_today_overlaps(S, d) else "accept",
             rule="accept" if rule_accepts(S, d) else "reject")


def w1_enumerate():
    all_shapes = shapes()
    n = len(all_shapes)
    rule_ok = [(S, d) for S, d in all_shapes if rule_accepts(S, d)]
    phys_ok_full = {(S, d) for S, d in all_shapes if not physical_overlaps(S, d)}
    phys_ok_exact = {(S, d) for S, d in all_shapes if not physical_overlaps(S, d, byte_exact=True)}
    mkfs_ok = {(S, d) for S, d in all_shapes if not mkfs_today_overlaps(S, d)}
    rule_set = set(rule_ok)
    emit(name="w1_counts", shapes=n, rule_accept=len(rule_set), mkfs_today_accept=len(mkfs_ok),
         physical_clean_full=len(phys_ok_full), physical_clean_byte_exact=len(phys_ok_exact))
    emit(name="w1_cover", rule_accepts_but_physically_overlaps=len(rule_set - phys_ok_full),
         rule_accepts_but_byte_exact_overlaps=len(rule_set - phys_ok_exact))
    emit(name="w1_equals_mkfs_today", rule_minus_mkfs=len(rule_set - mkfs_ok), mkfs_minus_rule=len(mkfs_ok - rule_set))
    band = sorted(phys_ok_full - rule_set)
    emit(name="w1_over_rejection_full", count=len(band),
         min_region_bytes=min(S * d for S, d in band), max_region_bytes=max(S * d for S, d in band))
    band_exact = sorted(phys_ok_exact - rule_set)
    emit(name="w1_over_rejection_byte_exact", count=len(band_exact))
    touching = sorted((S, d) for S, d in rule_set if S * d == STRIDE)
    emit(name="w1_touching_shapes_rule_accepts", count=len(touching), first=touching[0], last=touching[-1])
    # 先不分盘、再按盘的两种「最大 S」
    for d in (131072, 196608, 262144, 393216, 524288, 1044480):
        emit(name="w1_largest_S", spacing=d, uniform_rule=STRIDE // d, same_disk_0_and_2=(2 * STRIDE) // d,
             region2_before_journal=(JOURNAL_START - region_start(2)) // d)


def w1_negative_controls():
    all_shapes = shapes()
    # 反例一：不加这一判，只留槽距区间 -> 同盘区域相叠的形状还在
    no_rule = [(S, d) for S, d in all_shapes if spacing_in_reader_range(d)]
    bad = [(S, d) for S, d in no_rule if physical_overlaps(S, d)]
    emit(name="w1_control_no_rule", accepted=len(no_rule), physically_overlapping=len(bad),
         includes_16_512KiB=(16, 512 * KIB) in bad)
    # 反例二：把右边换成同盘上限 6 MiB -> 在 0/1/0 上还罩得住，换归属就不罩
    six = [(S, d) for S, d in all_shapes if spacing_in_reader_range(d) and S * d <= 2 * STRIDE]
    bad6 = [(S, d) for S, d in six if physical_overlaps(S, d)]
    emit(name="w1_control_same_disk_rule_on_0_1_0", accepted=len(six), physically_overlapping=len(bad6))
    global ASSIGN
    saved = ASSIGN
    ASSIGN = [0, 0, 1]
    bad6b = [(S, d) for S, d in six if physical_overlaps(S, d)]
    ASSIGN = saved
    emit(name="w1_control_same_disk_rule_on_0_0_1", accepted=len(six), physically_overlapping=len(bad6b))
    # 反例三：右边 P×chunk 取盘上字段（checker 今天读 363 / 367），盘上字段可以是别的数
    for disk_stride in (3 * MIB, 5 * MIB, 6 * MIB, 9 * MIB):
        ok = [(S, d) for S, d in all_shapes if spacing_in_reader_range(d) and S * d <= disk_stride]
        leak = [(S, d) for S, d in ok if physical_overlaps(S, d, stride=disk_stride)]
        leak_journal = [(S, d) for S, d in leak if any(x[1] == "journal" or x[2] == "journal" for x in physical_overlaps(S, d, stride=disk_stride))]
        emit(name="w1_control_disk_stride", p_times_chunk=disk_stride, rule_accepts=len(ok),
             overlapping_with_that_stride=len(leak), overlapping_journal=len(leak_journal))
    # 反例四：判成 < 而不是 <=，错拒 mkfs 今天收的「首尾相接」形状
    lt = [(S, d) for S, d in all_shapes if spacing_in_reader_range(d) and S * d < STRIDE]
    mkfs_ok = [(S, d) for S, d in all_shapes if not mkfs_today_overlaps(S, d)]
    emit(name="w1_control_strict_less_than", wrongly_rejects_mkfs_accepted=len(set(mkfs_ok) - set(lt)))


def alignment_ok(d, io_min, pbs=PBS, S=8):
    width = max(pbs, io_min)
    offsets = [0, d] + [region_start(r) + j * d for r in range(3) for j in range(S)]
    return all(o % width == 0 for o in offsets)


def conforming_spacing(io_min):
    if io_min == 0:
        return SPACING_MIN
    return -(-SPACING_MIN // io_min) * io_min


def w2_min_formula():
    hits4 = []
    below4 = []
    other = {}
    for d in spacings():
        v = min(8, STRIDE // d)
        if v == 4:
            hits4.append(d)
        elif v < 4:
            below4.append(d)
        other[v] = other.get(v, 0) + 1
    emit(name="w2_min_over_free_spacing", written_S_histogram=sorted(other.items()),
         spacing_giving_exactly_4=f"{min(hits4)}..{max(hits4)}" if hits4 else "none",
         spacing_giving_below_4=f"{min(below4)}..{max(below4)}" if below4 else "none",
         spacing_giving_16="none")
    # 满足 D2 已定项 19 的取法（槽距 = 4096 向上取整到 io_min 的整数倍）且 mkfs 的对齐判过：哪些 io_min 能走到
    rows = []
    for io in range(512, SPACING_MAX + 1, 512):
        d = conforming_spacing(io)
        if not spacing_in_reader_range(d):
            continue
        if not alignment_ok(d, io, S=4):
            continue
        S = min(8, STRIDE // d)
        rows.append((io, d, S))
    emit(name="w2_conforming_io_min_passing_alignment", count=len(rows),
         io_min_values=[r[0] for r in rows if r[2] != 8] , written_S_not_8=[(r[0], r[2]) for r in rows if r[2] != 8],
         any_written_S_is_4_or_below=any(r[2] <= 4 for r in rows))
    emit(name="w2_conforming_io_min_all_powers_of_two", value=all((r[0] & (r[0] - 1)) == 0 for r in rows if r[0] >= 4096) )
    # 非 2 的幂的 io_min（3072 等）：对齐判是否拒
    for io in (3072, 6144, 12288, 786432, 655360):
        d = conforming_spacing(io)
        emit(name="w2_non_power_of_two_io_min", io_min=io, spacing=d, reader_range=spacing_in_reader_range(d),
             alignment_pass=alignment_ok(d, io, S=4), written_S=min(8, STRIDE // d) if d else None)
    # 槽距是自由参数时（library mkfs 不从 io_min 派生）：4096 的整数倍里能走到 S = 4 的
    free4 = [d for d in range(4096, SPACING_MAX + 1, 4096) if min(8, STRIDE // d) == 4 and alignment_ok(d, 4096, S=4)]
    emit(name="w2_free_spacing_reaching_4", count=len(free4), examples=free4[:3])


def w2_dependents():
    for S in (4, 6, 8, 12, 16):
        emit(name="w2_ring_depth", S=S, ring_slots=3 * S, ring_window_factor=3 * S + 1, max_pin_value=3 * S - 1,
             pin_fits_a_byte_below_sentinel=(3 * S - 1) <= 254)
    emit(name="w2_pin_bound", max_S_with_pin_below_255=max(S for S in range(1, 300) if 3 * S - 1 <= 254))
    emit(name="w2_gpu_root_table", format_upper_bound_times_R=S_MAX * 3, gpu_MAX_ROOTS_literal=48)


def slot_aliases(S, d, pbs=PBS, stride=STRIDE):
    """同一块盘上、字节区间相交的两个不同根槽 (区域, 槽)。"""
    by_disk = {}
    for r in range(3):
        for j in range(S):
            by_disk.setdefault(ASSIGN[r], []).append((region_start(r, stride) + j * d, r, j))
    out = []
    for disk, items in by_disk.items():
        items.sort()
        for i in range(len(items)):
            for k in range(i + 1, len(items)):
                if items[k][0] >= items[i][0] + pbs:
                    break
                out.append((disk, items[i][1:], items[k][1:]))
    return out


def w3_alias_enumerate():
    all_shapes = [(S, d) for S, d in shapes() if spacing_in_reader_range(d)]
    for pbs in (512, 4096):
        al = {(S, d) for S, d in all_shapes if d >= pbs and slot_aliases(S, d, pbs)}
        rule_ok = {(S, d) for S, d in all_shapes if rule_accepts(S, d)}
        ext = {(S, d) for S, d in all_shapes if physical_overlaps(S, d)}
        emit(name="w3_slot_alias_vs_rule", pbs=pbs, shapes_with_a_root_slot_aliased=len(al),
             of_them_accepted_by_rule=len(al & rule_ok), extent_overlap_shapes=len(ext),
             extent_overlap_without_slot_alias=len(ext - al),
             alias_spacings_not_power_of_two=sorted({d for S, d in al if d & (d - 1)})[:6])
    cell = slot_aliases(16, 512 * KIB)
    emit(name="w3_cell_16_512KiB_aliases", pairs=";".join(f"d{x[0]}:r{x[1][0]}s{x[1][1]}=r{x[2][0]}s{x[2][1]}" for x in cell))
    seeds = [(r, 0) for r in range(3)]
    for S, d in [(16, 512 * KIB), (13, 512 * KIB), (8, 512 * KIB)]:
        hit = []
        for r, j in seeds:
            off = region_start(r) + j * d
            for r2 in range(3):
                if ASSIGN[r2] != ASSIGN[r]:
                    continue
                if r2 != r:
                    k, rem = divmod(off - region_start(r2), d)
                    if rem == 0 and 0 <= k < S:
                        hit.append(f"seed_r{r}s{j}_is_scanned_as_r{r2}s{k}")
        emit(name="w3_genesis_seed_scanned_elsewhere", S=S, spacing=d, hits=hit if hit else "none")


def selftest():
    # 判别力：弄坏一处，模型必须抓得到
    assert physical_overlaps(16, 512 * KIB), "S=16/512KiB 必须同盘相叠"
    assert not physical_overlaps(8, 512 * KIB), "S=8/512KiB 在 0/1/0 上同盘不叠"
    assert mkfs_today_overlaps(8, 512 * KIB), "入库 mkfs 判法拒 S=8/512KiB"
    assert not rule_accepts(16, 512 * KIB)
    assert rule_accepts(8, 384 * KIB) and not rule_accepts(8, 384 * KIB + 512)
    emit(name="selftest", result="ok")


def main():
    selftest()
    w1_probe_cells()
    w1_enumerate()
    w1_negative_controls()
    w2_min_formula()
    w2_dependents()
    w3_alias_enumerate()


if __name__ == "__main__":
    main()
