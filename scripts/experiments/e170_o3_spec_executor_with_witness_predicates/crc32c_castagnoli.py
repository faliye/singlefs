"""CRC-32C（Castagnoli），E170（O3规约执行器第一版） 执行器自己的一份。

参数只取自 kb 文字：
.claude/kb/decisions/18-块里携带什么信息.md@已定项 17：32 字节头校验和字段放 CRC32C 4 字节 + 28 字节零；自证结构的校验和覆盖整个槽含补齐
——「Castagnoli 多项式 0x1EDC6F41，反射实现（输入输出都反转），初值 0xFFFFFFFF，输出取反；结果按小端写进 32 字节字段的前 4 字节」。

两条路子：本文件的按表实现（逐字节查表，独立写的）与本机 `google_crc32c` 库（有就用来提速）。
模块载入时两条路子在 A8–A10 三个钉死的值与一批随机输入上互比，对不上就拒绝载入（登记第十一节 V3）。
"""

from __future__ import annotations

import os

CASTAGNOLI_POLYNOMIAL_NORMAL_FORM = 0x1EDC6F41
CRC_INITIAL_VALUE = 0xFFFFFFFF
CRC_FINAL_XOR = 0xFFFFFFFF


def reflect_bits(value: int, bit_count: int) -> int:
    reflected = 0
    for bit_position in range(bit_count):
        if value & (1 << bit_position):
            reflected |= 1 << (bit_count - 1 - bit_position)
    return reflected


CASTAGNOLI_POLYNOMIAL_REFLECTED_FORM = reflect_bits(CASTAGNOLI_POLYNOMIAL_NORMAL_FORM, 32)
assert CASTAGNOLI_POLYNOMIAL_REFLECTED_FORM == 0x82F63B78, "反射多项式由 0x1EDC6F41 逐位反转得出"


def build_reflected_lookup_table() -> tuple[int, ...]:
    table_entries = []
    for byte_value in range(256):
        register = byte_value
        for _bit_step in range(8):
            if register & 1:
                register = (register >> 1) ^ CASTAGNOLI_POLYNOMIAL_REFLECTED_FORM
            else:
                register >>= 1
        table_entries.append(register)
    return tuple(table_entries)


REFLECTED_LOOKUP_TABLE = build_reflected_lookup_table()


def crc32c_by_table(data: bytes) -> int:
    register = CRC_INITIAL_VALUE
    lookup_table = REFLECTED_LOOKUP_TABLE
    for byte_value in data:
        register = (register >> 8) ^ lookup_table[(register ^ byte_value) & 0xFF]
    return register ^ CRC_FINAL_XOR


def crc32c_bit_by_bit(data: bytes) -> int:
    """第二份写法：逐位移位，不查表；只在载入自检里用，与按表实现互比。"""
    register = CRC_INITIAL_VALUE
    for byte_value in data:
        register ^= byte_value
        for _bit_step in range(8):
            if register & 1:
                register = (register >> 1) ^ CASTAGNOLI_POLYNOMIAL_REFLECTED_FORM
            else:
                register >>= 1
    return register ^ CRC_FINAL_XOR


try:
    import google_crc32c as google_crc32c_library  # 本机库，与 crates/ 无关

    def crc32c_by_library(data: bytes) -> int:
        return google_crc32c_library.value(bytes(data))

    LIBRARY_IMPLEMENTATION_NAME = "google_crc32c:" + str(google_crc32c_library.implementation)
except ImportError:  # 没装库时整条退回按表实现
    crc32c_by_library = None
    LIBRARY_IMPLEMENTATION_NAME = "absent"

# 登记第七节 A8–A10：设计时用两条路子各算一遍得出的绝对值。
PINNED_CRC32C_OF_ASCII_123456789 = 0xE3069283
PINNED_CRC32C_OF_4096_ZERO_BYTES = 0x98F94189
PINNED_CRC32C_OF_512_ZERO_BYTES = 0x30FCEDC0


def self_check_against_pinned_values() -> list[str]:
    """返回对不上的条目；空表 = A8–A10 三条全过、两条（或三条）路子在随机输入上逐个相等。"""
    disagreements = []
    pinned_cases = [
        ("A8", b"123456789", PINNED_CRC32C_OF_ASCII_123456789),
        ("A9", bytes(4096), PINNED_CRC32C_OF_4096_ZERO_BYTES),
        ("A10", bytes(512), PINNED_CRC32C_OF_512_ZERO_BYTES),
    ]
    for assertion_name, data, expected in pinned_cases:
        by_table = crc32c_by_table(data)
        by_bits = crc32c_bit_by_bit(data)
        if by_table != expected or by_bits != expected:
            disagreements.append(f"{assertion_name} table={by_table:#010x} bits={by_bits:#010x} expected={expected:#010x}")
        if crc32c_by_library is not None and crc32c_by_library(data) != expected:
            disagreements.append(f"{assertion_name} library={crc32c_by_library(data):#010x} expected={expected:#010x}")
    random_source = os.urandom(4096 + 37)
    for slice_end in (0, 1, 7, 64, 511, 4133):
        data = random_source[:slice_end]
        by_table = crc32c_by_table(data)
        if crc32c_bit_by_bit(data) != by_table:
            disagreements.append(f"random length {slice_end}: bit-by-bit differs from table")
        if crc32c_by_library is not None and crc32c_by_library(data) != by_table:
            disagreements.append(f"random length {slice_end}: library differs from table")
    return disagreements


LOAD_TIME_DISAGREEMENTS = self_check_against_pinned_values()
if LOAD_TIME_DISAGREEMENTS:
    raise RuntimeError("CRC-32C 自检不过，这一趟作废（登记第十一节 V3）：" + "; ".join(LOAD_TIME_DISAGREEMENTS))


def crc32c(data: bytes) -> int:
    if crc32c_by_library is not None:
        return crc32c_by_library(data)
    return crc32c_by_table(data)
