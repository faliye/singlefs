#!/usr/bin/env python3
# E183 第三段 H5 的独立核对（登记 7.2 A34、A35 与 5.3 PC-不扫）：对产物里 24 格阈值历史每一格，用锚点脚本 anchors_e183.py 的 h5() 现算
# 挂载次数、两次清扫在第几次挂载、各删几行、清扫后剩几行、行数峰值、链重写片数峰值、终了 txg，再与产物里逐次挂载行（name=h5_mount）和
# 格汇总行（name=h5_cell_summary）里读出的同一批量比；「交给后台」对照那一格逐次核挂载次数、rows0 = 第几次挂载、没有清扫。
# 不 import 装置、不读 crates/。写成 E7RESULT 行，末行是 name=done。
# 用法：sweep_history_anchor_check.py <锚点脚本 anchors_e183.py 的路径> <产物 .out 的路径>
import hashlib, importlib.util, sys

anchors_path, product_path = sys.argv[1], sys.argv[2]
specification = importlib.util.spec_from_file_location("anchors_e183", anchors_path)
anchors = importlib.util.module_from_spec(specification)
specification.loader.exec_module(anchors)

def fields_of(line):
    return dict(token.split("=", 1) for token in line.split()[1:] if "=" in token)

configurations, summaries, mounts = {}, {}, {}
for line in open(product_path, encoding="utf-8"):
    line = line.rstrip("\n")
    if not line.startswith("E7RESULT "):
        continue
    fields = fields_of(line)
    kind, shard = fields.get("name"), fields.get("shard")
    if kind == "h5_config":
        configurations[shard] = fields
    elif kind == "h5_cell_summary":
        summaries[shard] = fields
    elif kind == "h5_mount":
        mounts.setdefault(shard, []).append(fields)

lines = []
def emit(text):
    lines.append("E7RESULT " + text)

emit("name=anchor_check_script anchors_script_sha256=%s product_sha256=%s" % (
    hashlib.sha256(open(anchors_path, "rb").read()).hexdigest(),
    hashlib.sha256(open(product_path, "rb").read()).hexdigest()))
mismatches, cells_checked, control_checked = 0, 0, 0
for shard in sorted(configurations):
    configuration = configurations[shard]
    if shard.startswith("h5-repeat-of-") or configuration["threshold_rows"] == "none":
        continue
    slots_per_region, threshold = int(configuration["slots_per_region"]), int(configuration["threshold_rows"])
    mode = "unmount" if configuration["unmount_kind"] == "normal_unmount" else "crash"
    model = anchors.h5(slots_per_region, threshold, mode, 2 * threshold + 3 * slots_per_region + 20)
    rows = mounts[shard]
    swept = [row for row in rows if row["rows_deleted"] != "none"]
    product = dict(
        mounts=len(rows),
        sweeps=[int(row["rows_deleted"]) for row in swept],
        residuals=[int(row["rows_left"]) for row in swept],
        mounts_at_sweep=[int(row["mount"]) for row in swept],
        peak_rows0=int(summaries[shard]["peak_rows0"]),
        peak_pages=int(summaries[shard]["peak_chain_pages"]),
        final_txg=int(rows[-1]["checkpoint_txg"]),
    )
    equal = all(product[key] == model[key] for key in product)
    mismatches += int(not equal)
    cells_checked += 1
    emit("name=anchor_check shard=%s mounts=%d/%d sweeps=%s/%s residuals=%s/%s mounts_at_sweep=%s/%s peak_rows0=%d/%d peak_pages=%d/%d final_txg=%d/%d equal=%s" % (
        shard, product["mounts"], model["mounts"],
        ":".join(map(str, product["sweeps"])), ":".join(map(str, model["sweeps"])),
        ":".join(map(str, product["residuals"])), ":".join(map(str, model["residuals"])),
        ":".join(map(str, product["mounts_at_sweep"])), ":".join(map(str, model["mounts_at_sweep"])),
        product["peak_rows0"], model["peak_rows0"], product["peak_pages"], model["peak_pages"],
        product["final_txg"], model["final_txg"], equal))
for shard in sorted(configurations):
    if configurations[shard]["threshold_rows"] != "none":
        continue
    rows = mounts[shard]
    rows0_is_the_mount_number = all(int(row["rows0"]) == int(row["mount"]) for row in rows)
    never_swept = all(row["rows_deleted"] == "none" for row in rows)
    equal = len(rows) == 400 and rows0_is_the_mount_number and never_swept
    mismatches += int(not equal)
    control_checked += 1
    emit("name=anchor_check_control shard=%s mounts=%d rows0_is_the_mount_number=%s never_swept=%s equal=%s" % (
        shard, len(rows), rows0_is_the_mount_number, never_swept, equal))
emit("name=verdict item=h5_anchor_check cells_checked=%d control_cells_checked=%d mismatches=%d" % (cells_checked, control_checked, mismatches))
for line in lines:
    print(line)
print("E7RESULT name=done emitted=%d" % (len(lines) + 1))
sys.exit(1 if mismatches else 0)
