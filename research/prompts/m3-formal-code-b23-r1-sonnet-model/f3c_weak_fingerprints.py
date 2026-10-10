#!/usr/bin/env python3
"""F3-c 的 4095 个种子两两不同的断言能拦住哪些弱指纹（纯算术，不碰 crates）。

种子 = 历史种子 ^ 盐 ^ 段号 ^ 指纹（crash_injection.rs 第 109、2821 行起）。内联用例固定历史种子 0、段号 16、长 12，
对 mask 0..4095 各算一次、看 BTreeSet 里剩几个。这里把「指纹」换成各种弱版本，同样的输入各算一遍：
- len12: 用例的断言（剩下几个，要 4095 才绿）；
- cross: 段号 0..40 × 长 12 的全部 4095 个子集 = 163800 个 (段号, 子集)，种子两两不同的个数（用例不看，因为它只固定段号 16）；
- len16: 长 16 的段、全部 65535 个子集，指纹的种子两两不同的个数（用例只到 12）。
"""
SALT = 0x73_65_63_6f_6e_64_63_72
OFF = 0xcbf2_9ce4_8422_2325
PRIME = 0x0000_0100_0000_01b3
M = (1 << 64) - 1

def fnv(bits):
    f = OFF
    for b in bits:
        f = ((f ^ int(b)) * PRIME) & M
    return f

variants = {
    "入库 FNV-1a 全折": fnv,
    "只折第一个写(.take(1))": lambda bits: fnv(bits[:1]),
    "少折最后一个写(.take(len-1))": lambda bits: fnv(bits[:-1]),
    "跳过第一个写(.skip(1))": lambda bits: fnv(bits[1:]),
    "只折前 11 个写(.take(11))": lambda bits: fnv(bits[:11]),
    "只折前 12 个写(.take(12))": lambda bits: fnv(bits[:12]),
    "只折前 13 个写(.take(13))": lambda bits: fnv(bits[:13]),
    "只数持久的个数(popcount)": lambda bits: sum(bits),
    "只取第一个被扣下的写的下标": lambda bits: next((i for i, b in enumerate(bits) if not b), len(bits)),
    "掩码当整数直接异或(低位 mask)": lambda bits: sum(1 << i for i, b in enumerate(bits) if b),
    "FNV 截成低 32 位": lambda bits: fnv(bits) & 0xffffffff,
    "FNV 只留低 12 位": lambda bits: fnv(bits) & 0xfff,
    "FNV 只留低 16 位": lambda bits: fnv(bits) & 0xffff,
    "常数 0": lambda bits: 0,
    "只折最后 12 个写": lambda bits: fnv(bits[-12:]),
}

def bits_of(mask, n):
    return [(mask >> k) & 1 == 1 for k in range(n)]

def seed(fp, hist, seg, bits):
    return hist ^ SALT ^ seg ^ fp(bits)

masks12 = [bits_of(m, 12) for m in range(4095)]
masks16 = [bits_of(m, 16) for m in range(65535)]
print(f"{'指纹变体':<34}{'len12(用例)':>12}{'cross(段0..40×12写)':>22}{'len16(65535子集)':>18}  用例结论")
for name, fp in variants.items():
    len12 = len({seed(fp, 0, 16, b) for b in masks12})
    cross = len({seed(fp, 0, s, b) for s in range(40) for b in masks12})
    len16 = len({seed(fp, 0, 16, b) for b in masks16})
    verdict = "绿(拦不住)" if len12 == 4095 else "红"
    print(f"{name:<34}{len12:>12}{cross:>22}{len16:>18}  {verdict}")
print(f"期望：len12 = 4095，cross = {40*4095}，len16 = 65535")
