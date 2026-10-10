#!/usr/bin/env python3
# 用法：python3 -I crc_identity.py —— 不依赖仓、不编译：独立实现 CRC-32C（Castagnoli，init 0xFFFFFFFF、xorout 0xFFFFFFFF），随机 16 KiB 段验两条：
# ① crc(A‖B) == (crc(A‖0₁₆ₖ) ^ crc(0₁₆ₖ)) ^ crc(B)（V3 的版本表第 8 字加拼读的公式）；② 前一半全零时 A = 0 同样成立。末行打 OK 或 FAIL。
import random
POLY = 0x82F63B78
TABLE = []
for i in range(256):
    c = i
    for _ in range(8):
        c = (c >> 1) ^ POLY if c & 1 else c >> 1
    TABLE.append(c)
def crc32c(data: bytes) -> int:
    c = 0xFFFFFFFF
    for b in data:
        c = TABLE[(c ^ b) & 0xFF] ^ (c >> 8)
    return c ^ 0xFFFFFFFF
assert crc32c(b'123456789') == 0xE3069283, 'CRC-32C 标准检验值'
random.seed(7)
K = 16384
zeros = bytes(K)
ok = True
for trial in range(6):
    a = zeros if trial == 0 else bytes(random.getrandbits(8) for _ in range(K))
    b = bytes(random.getrandbits(8) for _ in range(K))
    word8 = crc32c(a + zeros) ^ crc32c(zeros)
    ok &= (word8 ^ crc32c(b)) == crc32c(a + b)
print('CRC_IDENTITY', 'OK' if ok else 'FAIL', 'trials=6')
