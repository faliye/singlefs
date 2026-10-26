#!/usr/bin/env python3
# E183 第三段 H5 的读产物小工具（只读产物，不 import 装置、不读 crates/）：
#   sweep_cell_table.py <产物 .out 的路径> table            24 格阈值历史各一行的 markdown 表（读 name=h5_cell_summary，片名带 repeat、control 的不算）
#   sweep_cell_table.py <产物 .out 的路径> write-per-mount  每个（S，卸载方式）里四个 θ 的「整段历史设备写字节 ÷ 可写挂载次数」，和相对最小的千分比
# 第二种是 summary 行里 total_write_bytes ÷ writable_mounts 的除法，不是登记 R5c 的结果（V35 让 R5c 不出）。
import sys

product_path, mode = sys.argv[1], sys.argv[2]
cells = []
for number, line in enumerate(open(product_path, encoding="utf-8"), 1):
    if not line.startswith("E7RESULT name=h5_cell_summary ") or "shard=h5-repeat" in line or "shard=h5-control" in line:
        continue
    fields = dict(token.split("=", 1) for token in line.split()[1:] if "=" in token)
    cells.append((int(fields["slots_per_region"]), fields["unmount_kind"], int(fields["threshold_rows"]), number, fields))
cells.sort()
if mode == "table":
    print("| S | 卸载 | θ | 摘要行 | 挂载次数 | 行数峰值 | 片数峰值 | R5b 甲 | 清扫次数 | 期末行数 | 两次清扫的写字节 | 两次清扫的读字节 |")
    print("|---|---|---|---|---|---|---|---|---|---|---|---|")
    for slots, kind, threshold, number, fields in cells:
        crossing = "不越过" if int(fields["peak_chain_pages"]) == 1 else "越过"
        print(f"| {slots} | {'正常卸载' if kind == 'normal_unmount' else '崩溃'} | {threshold} | {number} | {fields['writable_mounts']} | {fields['peak_rows0']} | "
              f"{fields['peak_chain_pages']} | {crossing} | {fields['sweeps_done']} | {fields['final_rows']} | "
              f"{fields['sweep_write_bytes'].replace(',', ' / ')} | {fields['sweep_read_bytes'].replace(',', ' / ')} |")
elif mode == "write-per-mount":
    points = {}
    for slots, kind, threshold, number, fields in cells:
        points.setdefault((slots, kind), {})[threshold] = int(fields["total_write_bytes"]) / int(fields["writable_mounts"])
    for (slots, kind), per_threshold in points.items():
        smallest = min(per_threshold.values())
        print(f"S={slots} {kind}: " + " ".join(
            f"θ={threshold}:每次挂载写{per_threshold[threshold]:.0f}字节:相对最小{per_threshold[threshold] / smallest * 1000:.0f}千分之一"
            for threshold in sorted(per_threshold)))
else:
    sys.exit("用法：sweep_cell_table.py <产物> table|write-per-mount")
