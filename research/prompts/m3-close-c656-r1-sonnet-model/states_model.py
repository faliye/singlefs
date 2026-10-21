#!/usr/bin/env python3
"""m3-close-c656-r1 正推腿的模型：mkfs 三种写序（今天的、乙、甲拆槽）的崩溃态计数与结局分类。

只用标准库；不编译 Rust、不读盘镜像。计数公式照 crates/singlefs-checker-tier/src/crash.rs 的
state_count_of_the_segments 与 CrashPointReplaySegmentExpansion::checked_state_count（全量与快档甲二）；
三态的认法照同文件 is_tearable_in_place_overwrite（单元写两态；其余写长于一个扇区且基镜像那一段有非零字节取三态）。

用法：python3 states_model.py [--repo <仓根>]   （--repo 给了就多核几处 crates 里的源码事实）
退出码 0 全部自检过；1 有格红。
"""
import itertools, re, sys, os

# ---------- 写表与计数 ----------
class W:
    def __init__(self, name, device, kind, tearable=False):
        self.name, self.device, self.kind, self.tearable = name, device, kind, tearable
    def choices(self):
        return 3 if self.tearable else 2

def full_states(segments):
    total = 1
    for seg in segments:
        prod = 1
        for w in seg:
            prod *= w.choices()
        total += prod - 1
    return total

def quick_states(segments):
    """甲二：段内非单元写各取其几态任意组合 x 单元写只取全不落或全落。"""
    total = 1
    for seg in segments:
        prod = 1
        for w in seg:
            if w.kind != 'unit_write':
                prod *= w.choices()
        if any(w.kind == 'unit_write' for w in seg):
            prod *= 2
        total += prod - 1
    return total

# ---------- 基镜像：哪些范围有旧内容 ----------
# base[dev] = 该盘上有非零字节的范围集合；键：z0 根环基址前、r0 r1 r2 三个根环区域、j journal 环、s0 系统配置槽 0、s1 系统配置槽 1(新槽距处)
def base_both_reused_same_io_min():
    # 旧池两盘都在用、同槽距、用过一段历史：盘 0 背区域 0 和 2，盘 1 背区域 1；journal 环两盘镜像
    return {0: {'z0','r0','r2','j','s0','s1'}, 1: {'z0','r1','j','s0','s1'}}

def base_empty():
    return {0: set(), 1: set()}

def base_disk0_only_old_io_min_65536():
    # 旧池 io_min 65536：槽 1 在 65536 处，新槽距 4096 处槽 1 下没有旧内容；根环区域 0 槽 0 在区域起点有旧根
    return {0: {'z0','r0','r2','j','s0'}, 1: set()}

def base_both_reused_old_io_min_65536():
    return {0: {'z0','r0','r2','j','s0'}, 1: {'z0','r1','j','s0'}}

def base_both_reused_never_published_since_mkfs():
    # 旧池只做过 mkfs（u=0）：journal 环全 0（没发过记录），根环区域只有创世根
    return {0: {'z0','r0','r2','s0','s1'}, 1: {'z0','r1','s0','s1'}}

REGION_OF = {'r0': 0, 'r1': 1, 'r2': 2}

def zf(base, dev, key):
    return W(f'zero_{key}@d{dev}', dev, 'zero_fill', key in base[dev])

def unit(dev, which):
    return W(f'unit_{which}@d{dev}', dev, 'unit_write', False)

def root(region, dev):
    return W(f'root_fua_r{region}@d{dev}', dev, 'root_record_fua', False)

def sysconf(base, dev, slot):
    return W(f'sysconf_s{slot}@d{dev}', dev, 'system_configuration_slot', f's{slot}' in base[dev])

ROOT_DEV = {0: 0, 1: 1, 2: 0}

def tail_segments(base):
    return [[root(0, 0)], [root(1, 1)], [root(2, 0)],
            [sysconf(base, d, s) for d in (0, 1) for s in (0, 1)]]

def today(base):
    seg0 = []
    for d in (0, 1):
        seg0 += [zf(base, d, 'z0'), zf(base, d, 'r0'), zf(base, d, 'r1'), zf(base, d, 'r2'), zf(base, d, 'j')]
    seg0 += [unit(0, 'inst'), unit(0, 'tree'), unit(1, 'inst'), unit(1, 'tree')]
    return [seg0] + tail_segments(base)

def second_phase_writes(base):
    out = []
    for d in (0, 1):
        out += [zf(base, d, 'r0'), zf(base, d, 'r1'), zf(base, d, 'r2'), zf(base, d, 'j')]
    out += [unit(0, 'inst'), unit(0, 'tree'), unit(1, 'inst'), unit(1, 'tree')]
    return out

def option_yi(base):
    """乙：每盘 [0,根环基址) 清零 -> 屏障 -> 其余清零与单元 -> 屏障 -> 根 FUA x3 -> 系统配置 x4 -> 屏障。"""
    return [[zf(base, 0, 'z0'), zf(base, 1, 'z0')], second_phase_writes(base)] + tail_segments(base)

def option_jia_prime(base):
    """甲拆槽：作废拆成四步单槽清零，盘 1 先于盘 0、槽 1 先于槽 0，每步一道该盘屏障。"""
    def part(dev, key, tear):
        return W(f'zero_{key}@d{dev}', dev, 'zero_fill', tear)
    steps = []
    for d in (1, 0):
        steps.append([part(d, 'z0_slot1_range', 's1' in base[d])])
        steps.append([part(d, 'z0_slot0_range', 's0' in base[d])])
    return steps + [second_phase_writes(base)] + tail_segments(base)

def op_counts(segments, barrier_steps):
    writes = sum(len(s) for s in segments)
    return writes, writes + barrier_steps

# ---------- 结局分类（照 recovery.rs / mount.rs 的规则写成可枚举的小模型） ----------
# 一块盘的一个系统配置槽：'old'（旧池自证过）、'new'（新池自证过）、'dead'（零或撕裂）
def mount_outcome(slots_by_disk):
    """slots_by_disk = {0: (s0, s1), 1: (s0, s1)}；返回 (读挂载结局, 可写挂载结局)。
    规则：choose_system_configuration_naming_devices_without_a_mountable_slot 逐盘取最大世代号槽，两槽都不可择的盘跳过，
    一块盘都交不出整池拒；可写挂载在 devices_without_the_selected_version 里任一盘一份自证过的槽都没有就拒。
    fsid 筛（用户 2026-10-19 定 S3）在这里不起作用：下面每个状态里只有一种 fsid 的活槽，枚举时断言。"""
    fsids = {v for pair in slots_by_disk.values() for v in pair if v != 'dead'}
    assert len(fsids) <= 1, f'一个状态里新旧两种活槽并存：{slots_by_disk}'
    alive_disks = [d for d, pair in slots_by_disk.items() if any(v != 'dead' for v in pair)]
    if not alive_disks:
        return 'refused:NoValidSystemConfiguration', 'refused'
    kind = next(iter(fsids))
    pool = 'old' if kind == 'old' else 'new'
    writable = pool if len(alive_disks) == len(slots_by_disk) else f'refused:SystemConfigurationUnreadableAfterTheReadStage({pool})'
    return f'{pool}', writable

def landings(tearable):
    return ('N', 'T', 'F') if tearable else ('N', 'F')

def slot_after(landing, old_state, new_state):
    # 一次对槽的清零：N 没落 => 原样；T 撕裂 => 槽自证不过；F 落了 => 清成零
    return old_state if landing == 'N' else 'dead'

def slot_after_write(landing, prev_state, new_state):
    # 一次对槽的新写：N 没落 => 原样；T 撕裂 => 不过；F => 新槽
    return prev_state if landing == 'N' else ('dead' if landing == 'T' else new_state)

def enumerate_yi_phase1():
    """乙作废段：两盘 [0,根环基址) 清零各取 N/T/F；旧池每盘两槽都是旧槽。T：撕裂镜像前一半清零后一半留旧，
    新旧不同的那一截是槽 0 与槽 1 之间到槽 1 末尾，截的中点落在槽 0 之后、槽 1 之前（槽距 4096，槽内容 489 字节），
    所以模型里 T = 槽 0 清了、槽 1 留着；这是 crash.rs torn_image_of_in_place_overwrite 的读法，真设备上撕法更多（见报告 V2）。"""
    rows = []
    for l0, l1 in itertools.product('NTF', repeat=2):
        if (l0, l1) == ('F', 'F'):
            continue   # 段尾全落那一个归下一段
        slots = {}
        for d, l in ((0, l0), (1, l1)):
            if l == 'N':
                slots[d] = ('old', 'old')
            elif l == 'T':
                slots[d] = ('dead', 'old')
            else:
                slots[d] = ('dead', 'dead')
        rows.append(((l0, l1), slots, mount_outcome(slots)))
    return rows

def enumerate_final_sysconf_segment():
    """乙与今天共有的新系统配置段：作废已持久（两盘旧槽全死），四次新系统配置写各取 N/T/F。"""
    rows = []
    for ls in itertools.product('NTF', repeat=4):
        if ls == ('F', 'F', 'F', 'F'):
            continue
        slots = {0: tuple(slot_after_write(ls[i], 'dead', 'new') for i in (0, 1)),
                 1: tuple(slot_after_write(ls[i], 'dead', 'new') for i in (2, 3))}
        rows.append((ls, slots, mount_outcome(slots)))
    return rows

def enumerate_jia_prime_phase1():
    """甲拆槽：盘 1 槽 1 -> 盘 1 槽 0 -> 盘 0 槽 1 -> 盘 0 槽 0，每步 N/T/F，前面的步全持久、后面的步全没落。"""
    rows = []
    order = [(1, 1), (1, 0), (0, 1), (0, 0)]   # (盘, 槽)
    for step_index, (d, s) in enumerate(order):
        for l in 'NT':   # F 归下一步的起点（最后一个槽的 F 与 T 是同一个盘面）
            slots = {0: ['old', 'old'], 1: ['old', 'old']}
            for (dd, ss) in order[:step_index]:
                slots[dd][ss] = 'dead'
            if l == 'T':
                slots[d][s] = 'dead'
            frozen = {k: tuple(v) for k, v in slots.items()}
            rows.append((step_index, l, frozen, mount_outcome(frozen)))
    return rows

def enumerate_any_sector_subset_of_the_invalidation():
    """不靠崩溃点重放的撕裂模型：作废那一步只写零，真设备上落下的扇区可以是任意子集；对系统配置槽的效果只有"这一槽还自证得过 / 不过"，
    所以每盘两槽各取活 / 死，共 16 种盘面（含一个都没改的起点）。"""
    rows = []
    for bits in itertools.product(('old', 'dead'), repeat=4):
        slots = {0: bits[0:2], 1: bits[2:4]}
        rows.append((bits, slots, mount_outcome(slots)))
    return rows

def split_into_segments(ops):
    """D13 已定项 4 的切段：屏障只放行它那块盘；当前段里每块有写的盘都被自己的屏障放行时这一段才关上。ops 元素：('w', 盘, 名) 或 ('b', 盘)。"""
    segments, current, released = [], [], set()
    for op in ops:
        if op[0] == 'w':
            current.append(op)
            released.discard(op[1])
        else:
            released.add(op[1])
            if all(d in released for d in {w[1] for w in current}) and current:
                segments.append([w[2] for w in current])
                current, released = [], set()
    if current:
        segments.append([w[2] for w in current])
    return segments

def order_of_two_passes():
    # 乙的写序（要求）：先两块盘各清 [0,根环基址)，再两块盘各过屏障，之后才写别的
    return [('w', 0, 'Z0@d0'), ('w', 1, 'Z0@d1'), ('b', 0), ('b', 1),
            ('w', 0, 'ZR@d0'), ('w', 1, 'ZR@d1'), ('b', 0), ('b', 1)]

def order_per_device():
    # 反例写序：一块盘一块盘做完（清零、屏障、其余清零），再做下一块盘
    return [('w', 0, 'Z0@d0'), ('b', 0), ('w', 0, 'ZR@d0'), ('b', 0),
            ('w', 1, 'Z0@d1'), ('b', 1), ('w', 1, 'ZR@d1'), ('b', 1)]

def tally(rows, key):
    c = {}
    for r in rows:
        k = key(r)
        c[k] = c.get(k, 0) + 1
    return dict(sorted(c.items()))

# ---------- 序号：今天写序下"只落了 [0,根环基址) 清零"的 9 个态 ----------
def today_ordinal(landed_digits, radices):
    value, weight = 0, 1
    for digit, radix in zip(landed_digits, radices):
        value += digit * weight
        weight *= radix
    return value

def main():
    repo = None
    if '--repo' in sys.argv:
        repo = sys.argv[sys.argv.index('--repo') + 1]
    failures = []
    def check(name, got, want):
        ok = got == want
        print(f"{'ok  ' if ok else 'RED '} {name}: got={got} want={want}")
        if not ok:
            failures.append(name)

    print('## 一、自检：公式对上已登记与已量的四个总数')
    check('空盘 16402 (layout/01-first-txn.md 八 mkfs 行)', full_states(today(base_empty())), 16402)
    check('A 历史 82970 (报告 mkfs-count-full.log)', full_states(today(base_disk0_only_old_io_min_65536())), 82970)
    check('B 历史 280019 (报告 第二节)', full_states(today(base_both_reused_same_io_min())), 280019)
    check('x=65536 两盘都重用 279974 (报告 mkfs-count-full.log 第三行)', full_states(today(base_both_reused_old_io_min_65536())), 279974)
    # 序号：B 历史的 11780 号态（报告最小复现）= 第 0 块盘 z0、r0 与两块盘 journal 环落了
    radices = [w.choices() for w in today(base_both_reused_same_io_min())[0]]
    digits = [0] * 14
    for i in (0, 1, 4, 9):
        digits[i] = radices[i] - 1
    check('序号 11780 号态的混合进制拆法 (报告最小复现)', today_ordinal(digits, radices), 11780)

    print('\n## 二、B 型基镜像（两盘重用、同槽距、journal 环有旧记录）三种写序的写表')
    base = base_both_reused_same_io_min()
    for name, fn, barrier_steps in (('今天', today, 4), ('乙', option_yi, 6), ("甲拆槽", option_jia_prime, 8)):
        segs = fn(base)
        writes, ops = op_counts(segs, barrier_steps)
        sizes = '+'.join(str(len(s)) for s in segs)
        print(f'{name}: 段序列={sizes} 写={writes} 操作={ops} 全量态数={full_states(segs)} 快档甲二态数={quick_states(segs)}')
    check('今天 段序列', '+'.join(str(len(s)) for s in today(base)), '14+1+1+1+4')
    check("乙 段序列", '+'.join(str(len(s)) for s in option_yi(base)), '2+12+1+1+1+4')
    check("甲拆槽 段序列", '+'.join(str(len(s)) for s in option_jia_prime(base)), '1+1+1+1+12+1+1+1+4')
    # 两态口径（登记表那种、不补第三态）：布局表八 mkfs 行 16402 的口径
    def two_state(segs):
        t = 1
        for seg in segs:
            t += 2 ** len(seg) - 1
        return t
    print(f"两态闭式：今天={two_state(today(base))} 乙={two_state(option_yi(base))} 甲拆槽={two_state(option_jia_prime(base))}")

    print('\n## 三、各段拆开的全量态数（B 型基镜像）')
    for name, fn in (('今天', today), ('乙', option_yi), ("甲拆槽", option_jia_prime)):
        segs = fn(base)
        print(name, [full_states([s]) - 1 for s in segs], '加末节点 1')

    print('\n## 四、乙作废段（作废那一步没持久完）的结局，B 型基镜像')
    rows = enumerate_yi_phase1()
    for (l, slots, out) in rows:
        print(f'  盘0={l[0]} 盘1={l[1]} 槽={slots} -> 读挂载={out[0]} 可写挂载={out[1]}')
    print('  汇总(读挂载结局):', tally(rows, lambda r: r[2][0]))
    print('  汇总(可写挂载结局):', tally(rows, lambda r: r[2][1].split('(')[0]))
    check('乙作废段态数 = 3x3-1', len(rows), 8)
    assert all(r[2][0] == 'old' for r in rows)

    print('\n## 五、乙与今天共有的新系统配置段（四次新系统配置写，作废已持久）')
    rows4 = enumerate_final_sysconf_segment()
    print('  汇总(读挂载结局):', tally(rows4, lambda r: r[2][0]))
    print('  汇总(可写挂载结局):', tally(rows4, lambda r: r[2][1].split('(')[0]))
    check('新系统配置段态数 3^4-1', len(rows4), 80)
    assert all(r[2][0] != 'old' for r in rows4), '新系统配置段不许挂上旧池'

    print("\n## 六、甲拆槽 作废段（四步单槽清零）的结局")
    rowsj = enumerate_jia_prime_phase1()
    for (step, l, slots, out) in rowsj:
        print(f'  步序{step} {l} 槽={slots} -> 读={out[0]} 可写={out[1]}')
    print('  汇总(读挂载结局):', tally(rowsj, lambda r: r[3][0]))
    # 清到最后一个槽之前，盘 0 的槽 0 或槽 1 里至少有一个旧槽 => 读挂载=old
    for r in rowsj:
        all_dead = all(v == 'dead' for pair in r[2].values() for v in pair)
        assert (r[3][0] == 'old') == (not all_dead), r
    print('  最后一个槽的清零取 T 的盘面 = 四槽全死 = 整池作废（这一步就是提交点），其余每一格都挂上旧池')

    print('\n## 六之二、任意扇区子集的作废盘面（两槽各活/死，16 种）')
    anyrows = enumerate_any_sector_subset_of_the_invalidation()
    print('  汇总(读挂载结局):', tally(anyrows, lambda r: r[2][0]))
    print('  汇总(可写挂载结局):', tally(anyrows, lambda r: r[2][1].split('(')[0]))
    assert [r for r in anyrows if r[2][0] != 'old'] == [r for r in anyrows if all(v == 'dead' for pair in r[1].values() for v in pair)]

    print('\n## 七、今天写序下"只落了 [0,根环基址) 清零"的 9 个态的序号（B 型基镜像，全量混合进制，写表里最前那次写是最低位）')
    segs = today(base)
    radices = [w.choices() for w in segs[0]]
    names = [w.name for w in segs[0]]
    i0, i5 = names.index('zero_z0@d0'), names.index('zero_z0@d1')
    ords = []
    for a in range(3):
        for b in range(3):
            digits = [0] * 14
            digits[i0], digits[i5] = a, b
            ords.append(today_ordinal(digits, radices))
    print('  ', sorted(ords), '(数位 0 没持久 1 撕裂 2 持久；盘 0 清零下标', i0, '盘 1 清零下标', i5, ')')
    check('9 个序号', sorted(ords), [0, 1, 2, 162, 163, 164, 324, 325, 326])

    print('\n## 八、敏感度：基镜像变了，乙与今天的态数怎么动（全量 / 快档）')
    for label, b in (('B 型（journal 环有旧记录）', base_both_reused_same_io_min()),
                     ('旧池只做过 mkfs（journal 环全 0）', base_both_reused_never_published_since_mkfs()),
                     ('旧池 io_min 65536、两盘都重用', base_both_reused_old_io_min_65536()),
                     ('只重用盘 0（旧池 io_min 65536）', base_disk0_only_old_io_min_65536())):
        t, y, j = today(b), option_yi(b), option_jia_prime(b)
        print(f'{label}: 今天 {full_states(t)} / {quick_states(t)}；乙 {full_states(y)} / {quick_states(y)}；甲\' {full_states(j)} / {quick_states(j)}')

    print('\n## 十、写序的循环结构：先两块盘各清零、再两块盘各过屏障，与逐盘做完的差别')
    two_passes = split_into_segments(order_of_two_passes())
    per_device = split_into_segments(order_per_device())
    print('  两趟写序的段：', two_passes)
    print('  逐盘写序的段：', per_device)
    def index_of(segments, name):
        return next(i for i, s in enumerate(segments) if name in s)
    def sysconf_invalidation_durable_before_anything_else(segments):
        # 安全：所有 Z0 所在的段都严格早于任何别的清零所在的段（别的清零开始时，每盘的旧系统配置已经都持久地作废了）
        return max(index_of(segments, 'Z0@d0'), index_of(segments, 'Z0@d1')) < min(index_of(segments, 'ZR@d0'), index_of(segments, 'ZR@d1'))
    check('两趟写序：两块盘的作废都严格早于别的清零', sysconf_invalidation_durable_before_anything_else(two_passes), True)
    check('逐盘写序：盘 0 的根环清零早于盘 1 的作废（盘 1 的旧系统配置还活着时盘 0 的根环已经清了）', sysconf_invalidation_durable_before_anything_else(per_device), False)

    if repo:
        print('\n## 九、源码事实（--repo）')
        src = open(os.path.join(repo, 'crates/singlefs-core/src/make_filesystem.rs'), encoding='utf-8').read()
        check('make_filesystem.rs 里 write_zeroes_at 的调用点数', len(re.findall(r'write_zeroes_at\(', src)), 3)
        check('make_filesystem.rs 里 barrier() 的调用点数', len(re.findall(r'device\.barrier\(\)', src)), 2)
        check('make_filesystem.rs 里读盘调用点数', len(re.findall(r'\.read_at\(|device\.read\(', src)), 0)
    print()
    if failures:
        print('红：', failures)
        return 1
    print('全部自检过')
    return 0

if __name__ == '__main__':
    sys.exit(main())
