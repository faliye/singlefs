#!/usr/bin/env python3
"""F-零槽 的 WGSL 算术逐位移植（正推腿 Sonnet，m3-batch32-code-r1）：read_at 与 zero_unit_version_over_a_slot_nothing_wrote 照 u32 / vec2<u32> 的回绕语义写，
落点表第 9–11 字照 crash_judge_tables.rs 的 unit_slots_nothing_wrote_after_each_location 写；再拿一份独立的 CPU 参考（稀疏读者的契约：越过盘尾读不到，
盘内没被任何落点盖住的范围读出全零）逐 (槽, 长度) 对。只算 u32，没有 GPU。输出 MODEL 行，每行一个结论。"""
M32 = 0xFFFFFFFF
SLOT = 16384
NODE_BYTES = 16384
DATA_UNIT_BYTES = 32768
SLOT_SHIFT = 14
NONE = M32

def u32(v): return v & M32

def shl64(a, s):
    ax, ay = a
    return (u32(ax << s), u32((ay << s) | (ax >> (32 - s))))

def lt64(a, b): return a[1] < b[1] or (a[1] == b[1] and a[0] < b[0])
def le64(a, b): return not lt64(b, a)
def eq64(a, b): return a == b

def split(v): return (v & M32, v >> 32)

class Table:
    """locations：按 (盘, 偏移) 升序的 (device, offset, length, kind)；kind 'U' 单元槽、'O' 别的（根槽、系统配置、journal 记录）。"""
    def __init__(self, locations, device_bytes, unit_area_start_slot):
        self.locations = locations
        self.device_bytes = device_bytes
        self.unit_area_start_slot = unit_area_start_slot
        self.gaps = self._gaps()
    def _gaps(self):
        device_end_slot = self.device_bytes // SLOT
        first_whole = (self.device_bytes - DATA_UNIT_BYTES) // SLOT + 1 if self.device_bytes >= DATA_UNIT_BYTES else 0
        out = []
        for index, (device, offset, length, kind) in enumerate(self.locations):
            if kind == 'U':
                first_after = offset // SLOT + 1
            else:
                first_after = -(-(offset + length) // SLOT)
            first = max(first_after, self.unit_area_start_slot)
            if index + 1 < len(self.locations) and self.locations[index + 1][0] == device:
                end = min(self.locations[index + 1][1] // SLOT, device_end_slot)
            else:
                end = device_end_slot
            if not first < end:
                first, end = 0, 0
            assert first <= M32 and end <= M32
            out.append((first, end, first_whole))
        return out
    def word(self, location, k):
        device, offset, length, kind = self.locations[location]
        lo, hi = split(offset)
        if k == 0: return device
        if k == 1: return lo
        if k == 2: return hi
        if k == 3: return length
        if k == 9: return self.gaps[location][0]
        if k == 10: return self.gaps[location][1]
        if k == 11: return self.gaps[location][2]
        raise KeyError(k)

def zero_unit_version(table, before, offset, length):
    first_slot = u32(offset[0] >> SLOT_SHIFT) | u32(offset[1] << (32 - SLOT_SHIFT))
    first_slot = u32(first_slot)
    cond_bad = ((offset[0] & (NODE_BYTES - 1)) | (offset[1] >> (32 - SLOT_SHIFT))) != 0
    if cond_bad or first_slot < table.word(before, 9) or table.word(before, 10) < u32(first_slot + ((length + NODE_BYTES - 1) >> SLOT_SHIFT)):
        return NONE
    return 0 if first_slot < table.word(before, 11) else 1

def read_at(table, device, offset, length):
    """返回 ('NONE',) / ('LOC', 落点下标) / ('ZERO', 0 或 1：32 KiB / 16 KiB 那条全零单元)。"""
    lo, hi = 0, len(table.locations)
    before_on_device = False
    before_starts = False
    while lo < hi:
        mid = (lo + hi) >> 1
        d = table.word(mid, 0)
        o = (table.word(mid, 1), table.word(mid, 2))
        if d < device or (d == device and le64(o, offset)):
            lo = mid + 1
            before_on_device = d == device
            before_starts = before_on_device and eq64(o, offset)
        else:
            hi = mid
    if not before_on_device:
        return ('NONE',)
    location = lo - 1
    if before_starts:
        unit_length = table.word(location, 3)
        return ('LOC', location) if length <= unit_length else ('NONE',)
    version = zero_unit_version(table, location, offset, length)
    unit_length = DATA_UNIT_BYTES >> (version & 1) if version != NONE else 0
    if version == NONE or length > unit_length:
        return ('NONE',)
    return ('ZERO', version)

def cpu_reference(table, device, byte_offset, length):
    """CPU 稀疏读者：越过盘尾读不到（None）；盘内没被任何落点盖住的范围读出全零；盖住的读出落点上的字节（这里不关心是什么）。"""
    if byte_offset + length > table.device_bytes:
        return 'UNREADABLE'
    for d, o, l, kind in table.locations:
        touched = SLOT if kind == 'U' else l      # 单元槽落点只因为那一个 16 KiB 槽被写过才建（enumerate_locations），第二个 16 KiB 没人写就是零
        if d == device and o < byte_offset + length and byte_offset < o + touched:
            return 'OVERLAPS_A_LOCATION'
    return 'ALL_ZERO'

def build_small_table():
    """像层 0 那些流：单元只写在单元区开头那几槽，别的都没写过。"""
    gib4 = 4 << 30
    unit_area = 1024 + 64
    locs = []
    for device in (0, 1):
        locs.append((device, 0, 4096, 'O'))
        locs.append((device, 32768, 4096, 'O'))
        for slot in (unit_area, unit_area + 1, unit_area + 5, unit_area + 40):
            locs.append((device, slot * SLOT, DATA_UNIT_BYTES, 'U'))
    locs.sort()
    return Table(locs, gib4, unit_area)

def build_table():
    gib4 = 4 << 30
    unit_area = 1024 + 64          # 单元区起始槽（任取一个，真值由基线几何定）
    locs = []
    # 两块盘各自：根环 / 系统配置 / journal 记录这些单元区之前的落点，加单元区里几个槽（带 32 KiB 落点，盘尾那个只有 16 KiB）
    for device in (0, 1):
        locs.append((device, 0, 4096, 'O'))
        locs.append((device, 32768, 4096, 'O'))
        for slot in (unit_area, unit_area + 1, unit_area + 5, 90_000, 150_000, 262_142):
            length = DATA_UNIT_BYTES if slot * SLOT + DATA_UNIT_BYTES <= gib4 else NODE_BYTES
            locs.append((device, slot * SLOT, length, 'U'))
        # 盘尾只装得下 16 KiB 的那个槽
        locs.append((device, 262_143 * SLOT, NODE_BYTES, 'U'))
    locs.sort()
    return Table(locs, gib4, unit_area)

def main():
    for name, table in (('wide', build_table()), ('small', build_small_table())):
        run(name, table)

def run(name, table):
    kinds = {}
    def record(kind, key):
        kinds.setdefault(kind, []).append(key)
    scan_slots = list(range(0, 262_150)) + [(1 << 32) - 3, (1 << 32) - 2, (1 << 32) - 1] \
        + [(1 << 32) + s for s in (0, 190_000, 262_143)] + [(1 << 33) + 190_000, (1 << 36) - 1, (1 << 36) + 190_000, (1 << 40) + 190_000]
    for device in (0, 1):
        for slot in scan_slots:
            offset = shl64(split(slot), SLOT_SHIFT)
            byte_offset = slot * SLOT
            for length in (NODE_BYTES, DATA_UNIT_BYTES):
                gpu = read_at(table, device, offset, length)
                cpu = cpu_reference(table, device, byte_offset, length)
                if gpu[0] == 'ZERO' and cpu != 'ALL_ZERO':
                    record(f'GPU 交回全零单元而 CPU {cpu}', (device, slot, length, gpu))
                elif cpu == 'ALL_ZERO' and gpu[0] == 'NONE':
                    area = '单元区之前' if slot < table.unit_area_start_slot else '单元区里'
                    record(f'CPU 读出全零而 GPU 读不到（{area}）', (device, slot, length))
                elif gpu[0] == 'ZERO' and cpu == 'ALL_ZERO':
                    record('一致：都是全零', (device, slot, length, gpu))
    for kind, keys in sorted(kinds.items()):
        print(f"MODEL zero-unit[{name}] {kind}：{len(keys)} 个 (盘, 槽, 长度)")
    # 全部列出「GPU 交回全零单元而 CPU 不是全零」的槽号区间（按长度分）
    bad = [k for kind, keys in kinds.items() if kind.startswith('GPU 交回全零单元而 CPU') for k in keys]
    by_length = {}
    for device, slot, length, gpu in bad:
        by_length.setdefault((device, length), []).append(slot)
    for (device, length), slots in sorted(by_length.items()):
        print(f"MODEL zero-unit[{name}] 越界误认 盘 {device} 长度 {length}：槽 {sorted(slots)}")
    # 实现员列的两形（第一形：32 KiB 读时第一槽没写过、第二槽有落点；第二形：单元区之前）与它们的 16 KiB 对照
    first_location_slots = sorted({o // SLOT for d, o, l, k in table.locations if d == 1 and k == 'U'})
    for s in first_location_slots:
        if s - 1 >= table.unit_area_start_slot and (s - 1) not in first_location_slots:
            for length in (NODE_BYTES, DATA_UNIT_BYTES):
                offset = shl64(split(s - 1), SLOT_SHIFT)
                print(f"MODEL zero-unit[{name}] fork-1 槽 {s - 1}（没写过，下一槽 {s} 是落点）读 {length}：GPU={read_at(table, 1, offset, length)} CPU={cpu_reference(table, 1, (s - 1) * SLOT, length)}")
            break
    for slot in (0, 1, table.unit_area_start_slot - 1):
        offset = shl64(split(slot), SLOT_SHIFT)
        print(f"MODEL zero-unit[{name}] fork-2 单元区之前的槽 {slot} 读 16384：GPU={read_at(table, 1, offset, NODE_BYTES)} CPU={cpu_reference(table, 1, slot * SLOT, NODE_BYTES)}")
    # 单点：偏移 (2^32 + 190000) 个槽
    for slot in ((1 << 32) - 1, (1 << 32) + 190_000, (1 << 33) + 190_000, (1 << 36) + 190_000, 190_000):
        offset = shl64(split(slot), SLOT_SHIFT)
        print(f"MODEL zero-unit[{name}] point slot={slot} offset=({offset[0]},{offset[1]}) 16KiB read_at={read_at(table, 1, offset, NODE_BYTES)} cpu={cpu_reference(table, 1, slot * SLOT, NODE_BYTES)}")

main()
