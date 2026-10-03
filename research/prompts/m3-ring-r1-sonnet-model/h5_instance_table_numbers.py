#!/usr/bin/env python3
"""H5：实例表行数随挂载次数涨时的片数、槽数（式子照 crates/singlefs-core/src/admission.rs:194-222 与 instance_table.rs:122-127）。
行数 rows0 取挂载次数 M（每次可写挂载写一行，rows0 = 挂载时读到的行数加这次要写的行数，近似 M）。
每片 32768 字节 = 2 个 16 KiB 槽；N_switch = 3，切换预留 = (N_switch + 1) × (32768 × 片数 + R × c_max × 16384)，这里只算与片数有关的那一半。
"""
import math

ROWS_PER_PAGE = 370 - 1
PAGE_BYTES = 32768
SLOT_BYTES = 16384
N_SWITCH = 3
JOURNAL_RECORD_BYTES = 4096
HEADER = 311
NAMED = 56
print(f"点名项容量 现行 (4096-311)//56 = {(JOURNAL_RECORD_BYTES - HEADER) // NAMED}；新根段加 86+86+4 字节后 (4096-311-176)//56 = {(JOURNAL_RECORD_BYTES - HEADER - 176) // NAMED}")
print(f"跨记录起点：现行 67 片 = {67 * ROWS_PER_PAGE} 行；加宽后 64 片 = {64 * ROWS_PER_PAGE} 行")
for mounts in (369, 370, 10**3, 10**4, 10**6):
    pages = max(1, math.ceil((mounts + N_SWITCH) / ROWS_PER_PAGE))
    reserve_bytes = (N_SWITCH + 1) * PAGE_BYTES * pages
    print(f"M={mounts:>8}: 片数={pages:>5} 一片 2 槽 ⇒ 链 {pages * 2:>6} 槽/每盘；"
          f"每次挂载重写整条链 {pages * PAGE_BYTES / 2**20:9.3f} MiB/每盘（两盘各一份）；"
          f"切换预留里链那一半 {reserve_bytes // SLOT_BYTES:>6} 槽 = {reserve_bytes / 2**20:9.3f} MiB/每盘")
