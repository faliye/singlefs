#!/usr/bin/env python3
"""正推腿对拍参照模型（只用标准库、不读 crates/，与产品代码不共用任何一行）。三段，每段打 name=<名> 起头的行，末行 name=summary。

甲 CRC-32C：两种写法各一份（反射 + 查表；非反射 + 逐位、最高位先），各对已知答案向量与随机输入逐个比。
   已知答案向量取自 CRC-32C 的公开检验值与 RFC 3720 附录 B.4 的四条（记忆写下，由本段断言核实：对不上就是我记错了，打 FAIL）。
乙 活槽数：照 tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs 里 records_on 的形状搭一块单元区，
   三种读法算每段活槽数：逐记录逐槽累加（产品那一份的读法）、区间重叠求和、槽集合去重；看哪几段两种读法给不同的数。
丙 取样格：用一族「长度相关」与「长度余数相关」的假设故障，数现有长度格与加密的长度格各抓到几族（假设族是本模型自己造的，不是产品的错）。
用法：python3 -I model/independent_reference_model.py
"""
import random
import sys

POLYNOMIAL_FORWARD = 0x1EDC6F41
POLYNOMIAL_REFLECTED = 0x82F63B78
checks = 0
failures = []


def check(name, condition):
    global checks
    checks += 1
    if not condition:
        failures.append(name)
        print(f"name=FAIL case={name}")


def crc32c_reflected_table(data):
    table = []
    for value in range(256):
        remainder = value
        for _ in range(8):
            remainder = (remainder >> 1) ^ POLYNOMIAL_REFLECTED if remainder & 1 else remainder >> 1
        table.append(remainder)
    register = 0xFFFFFFFF
    for byte in data:
        register = table[(register ^ byte) & 0xFF] ^ (register >> 8)
    return register ^ 0xFFFFFFFF


def reversed_bits(value, width):
    return int(format(value, f"0{width}b")[::-1], 2)


def crc32c_forward_bit_serial(data):
    register = 0xFFFFFFFF
    for byte in data:
        reflected_byte = reversed_bits(byte, 8)
        for bit_position in range(7, -1, -1):
            input_bit = (reflected_byte >> bit_position) & 1
            top_bit = (register >> 31) & 1
            register = (register << 1) & 0xFFFFFFFF
            if top_bit ^ input_bit:
                register ^= POLYNOMIAL_FORWARD
    return reversed_bits(register, 32) ^ 0xFFFFFFFF


def crc32c_section():
    known_answers = [
        (b"123456789", 0xE3069283),
        (bytes(32), 0x8A9136AA),
        (bytes([0xFF] * 32), 0x62A8AB43),
        (bytes(range(32)), 0x46DD794E),
        (bytes(range(31, -1, -1)), 0x113FDB5C),
    ]
    for position, (data, expected) in enumerate(known_answers):
        for writing, function in (("reflected_table", crc32c_reflected_table), ("forward_bit_serial", crc32c_forward_bit_serial)):
            got = function(data)
            check(f"known_answer_{position}_{writing}", got == expected)
            print(f"name=crc_known_answer vector={position} writing={writing} expected=0x{expected:08X} got=0x{got:08X}")
    generator = random.Random(20261119)
    disagreements = 0
    lengths = [0, 1, 2, 3, 4, 7, 8, 9, 15, 16, 17, 63, 64, 65, 255, 256, 257, 4095, 4096, 4097]
    for length in lengths:
        data = bytes(generator.randrange(256) for _ in range(length))
        if crc32c_reflected_table(data) != crc32c_forward_bit_serial(data):
            disagreements += 1
    check("random_agreement", disagreements == 0)
    print(f"name=crc_random_agreement lengths={len(lengths)} disagreements={disagreements}")


SEGMENT_SLOTS = 64
FULL_SEGMENTS = 6
TRAILING_SLOTS = 10
UNIT_AREA_SLOTS = FULL_SEGMENTS * SEGMENT_SLOTS + TRAILING_SLOTS
UNIT_AREA_STARTS = [50176, (1 << 32) - 2 * SEGMENT_SLOTS]


def records_on(start):
    """（盘号, 起点槽, 跨度, 已释放）：与测试里 records_on 一条一条对应，盘号 0 是单元区那块盘，1 是另一块盘。"""
    width = SEGMENT_SLOTS
    return [
        (0, start + 3 * width + 10, 2, False),
        (0, start - 3, 5, False),
        (0, start, 2, False),
        (0, start + 5, 1, False),
        (0, start + 7, 2, True),
        (1, start + 9, 2, False),
        (0, start + width - 1, 2, False),
        (0, start + width + 6, 2, False),
        (0, start + width + 6, 2, False),
        (0, start + width + 36, 200, False),
        (0, (1 << 32) - 1, 2, False),
        (0, start + FULL_SEGMENTS * width + 2, 3, False),
        (0, start + 5 * width + 60, 2, True),
    ]


def live_slots_per_record_per_slot(records, start):
    counts = [0] * FULL_SEGMENTS
    for device, slot, span, released in records:
        if device != 0 or released:
            continue
        for covered in range(slot, slot + span):
            if covered < start:
                continue
            segment = (covered - start) // SEGMENT_SLOTS
            if segment < FULL_SEGMENTS:
                counts[segment] += 1
    return counts


def live_slots_by_interval_overlap(records, start):
    counts = []
    for segment in range(FULL_SEGMENTS):
        low, high = start + segment * SEGMENT_SLOTS, start + (segment + 1) * SEGMENT_SLOTS
        counts.append(sum(max(0, min(slot + span, high) - max(slot, low))
                          for device, slot, span, released in records if device == 0 and not released))
    return counts


def live_slots_by_distinct_slots(records, start):
    covered_slots = set()
    for device, slot, span, released in records:
        if device == 0 and not released:
            covered_slots.update(range(slot, slot + span))
    counts = [0] * FULL_SEGMENTS
    for covered in covered_slots:
        if covered >= start and (covered - start) // SEGMENT_SLOTS < FULL_SEGMENTS:
            counts[(covered - start) // SEGMENT_SLOTS] += 1
    return counts


def live_slots_section():
    for position, start in enumerate(UNIT_AREA_STARTS):
        records = records_on(start)
        per_record = live_slots_per_record_per_slot(records, start)
        by_interval = live_slots_by_interval_overlap(records, start)
        by_distinct = live_slots_by_distinct_slots(records, start)
        differing = [segment for segment in range(FULL_SEGMENTS) if per_record[segment] != by_distinct[segment]]
        check(f"interval_equals_per_record_area_{position}", per_record == by_interval)
        print(f"name=live_slots_area area={position} start={start} per_record={per_record} interval_overlap={by_interval} distinct_slots={by_distinct} segments_where_distinct_differs={differing}")


def current_length_grid():
    """现有对拍输入里出现的单元长度：每种起点余数先垫一个那个长度的单元，再放七种长度；装不下自适应分片那条另用 64 KiB。"""
    lengths = {0, 1, 2, 3, 7, 8, 9, 4095, 4096, 65536}
    return lengths


def stratified_length_grid():
    lengths = set(range(0, 66))
    for edge in (128, 256, 4096, 8192, 16384, 32768, 65536):
        lengths.update((edge - 1, edge, edge + 1))
    lengths.update((127, 255))
    return lengths


def fault_families():
    families = []
    for threshold in (8, 16, 64, 256, 1024, 4097, 8192, 16384, 16385, 32768, 32769, 65536):
        families.append((f"wrong_when_length_at_least_{threshold}", lambda length, t=threshold: length >= t))
    for modulus in (2, 3, 4, 8, 16, 32, 64, 256):
        for residue in range(modulus):
            families.append((f"wrong_when_length_mod_{modulus}_is_{residue}", lambda length, m=modulus, r=residue: length % m == r))
    return families


def length_grid_section():
    families = fault_families()
    for label, grid in (("current", current_length_grid()), ("stratified", stratified_length_grid())):
        missed = [name for name, fires in families if not any(fires(length) for length in grid)]
        print(f"name=length_grid grid={label} lengths={len(grid)} families={len(families)} caught={len(families) - len(missed)} missed={len(missed)} missed_names={missed}")
        check(f"grid_{label}_counts", len(missed) <= len(families))


def main():
    crc32c_section()
    live_slots_section()
    length_grid_section()
    print(f"name=summary checks={checks} failures={len(failures)}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
