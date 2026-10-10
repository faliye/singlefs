#!/usr/bin/env python3
"""E142 导出 bin 的拒绝判据用整数算术独立复算（不起 bin：直接执行 src/bin 下的装置属 checker 档，子 agent 不跑）。
按 e142_new_pool_file_creation_write_dump.rs 的 free_maps_of_the_two_devices_stay_within_the_bound 逐步复写：
  槽数 = floor(每盘字节 / 16384) − 50176（单元区起点槽，768 MiB 默认环）；负数（盘末在单元区起点之前）⇒ 不在这里判（放过，交 mkfs）；
  空闲图合起来 = 槽数 × 4 × 2，checked u64；> 2^30 拒。
再复写两条内联单测的断言，并对 mutations 第 18、19 行（`> u64::MAX`、`>=`）各算一遍，看两条单测的断言会不会翻。"""
SLOT = 16384
START = 50176
DEVICES = 2
PER_SLOT = 4
BOUND = 1 << 30
U64 = (1 << 64) - 1


def judge(device_bytes, compare):
    slots = device_bytes // SLOT - START
    if slots < 0:
        return "ok(mkfs-decides)"
    total = slots * PER_SLOT * DEVICES
    if total > U64:
        return "refuse(overflow)"
    return "refuse(over-bound)" if compare(total) else "ok"


original = lambda total: total > BOUND
mutant_never = lambda total: total > U64  # 变异第 18 行
mutant_ge = lambda total: total >= BOUND  # 变异第 19 行

slots_at_bound = BOUND // (PER_SLOT * DEVICES)
device_bytes_at_bound = (START + slots_at_bound) * SLOT
print(f"SLOTS_AT_BOUND {slots_at_bound} (= 2^27 {slots_at_bound == 1 << 27}) DEVICE_BYTES_AT_BOUND {device_bytes_at_bound}")
for label, arguments in [("4GiB", 4294967296), ("8GiB", 8589934592), ("0", 0), ("U64MAX", U64), ("2^43", 1 << 43), ("at-bound", device_bytes_at_bound), ("at-bound+slot", device_bytes_at_bound + SLOT)]:
    print(f"ARITH {label:14} bytes={arguments:>20} orig={judge(arguments, original):18} mutant18={judge(arguments, mutant_never):18} mutant19={judge(arguments, mutant_ge)}")

# 内联单测一：U64MAX 拒（原因含「超过上界」）、4GiB / 8GiB / 0 放过
t1 = lambda f: judge(U64, f).startswith("refuse") and all(judge(b, f).startswith("ok") for b in (4294967296, 8589934592, 0))
# 内联单测二：恰在上界放过、多一槽拒
t2 = lambda f: judge(device_bytes_at_bound, f) == "ok" and judge(device_bytes_at_bound + SLOT, f).startswith("refuse")
for name, f in [("original", original), ("mutant18(> u64::MAX)", mutant_never), ("mutant19(>=)", mutant_ge)]:
    print(f"INLINE_TESTS {name:22} largest_device_bytes_test={'pass' if t1(f) else 'RED'} at_the_bound_test={'pass' if t2(f) else 'RED'}")
print("MAX_SLOTS_OF_AN_U64_DEVICE_TIMES_8", ((U64 // SLOT) - START) * 8, "fits u64:", ((U64 // SLOT) - START) * 8 <= U64)
