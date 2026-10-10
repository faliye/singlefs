#!/usr/bin/env python3
"""全零 16 KiB / 32 KiB 的 CRC-32C（Castagnoli，反射多项式 0x82F63B78），纯 Python 独立实现；与探针 probe_of_the_sonnet_leg_for_batch32_crc 打的 core 的值逐位比。"""
def table():
    out = []
    for i in range(256):
        c = i
        for _ in range(8):
            c = (c >> 1) ^ 0x82F63B78 if c & 1 else c >> 1
        out.append(c)
    return out
T = table()
def crc32c(data):
    c = 0xFFFFFFFF
    for b in data:
        c = T[(c ^ b) & 0xFF] ^ (c >> 8)
    return c ^ 0xFFFFFFFF
assert crc32c(b"123456789") == 0xE3069283   # 标准检验值
print(f"MODEL crc zeros16384={crc32c(bytes(16384)):08x} zeros32768={crc32c(bytes(32768)):08x}")
