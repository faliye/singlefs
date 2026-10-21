#!/usr/bin/env python3
# E183 第一段 A31 的独立核对（登记 7.2 A31）：对产物里每一份读数，用锚点脚本 anchors_e183.py 的 ring_window_slots / unit_area
# 现算环窗口字节与单元区容量，写成 E7RESULT 行再与产物里装置报的比。不 import 装置、不读 crates/。
# 用法：small_disk_anchor_check.py <锚点脚本 anchors_e183.py 的路径> <产物 .out 的路径>
import hashlib, importlib.util, sys

anchors_path, product_path = sys.argv[1], sys.argv[2]
specification = importlib.util.spec_from_file_location("anchors_e183", anchors_path)
anchors = importlib.util.module_from_spec(specification)
specification.loader.exec_module(anchors)

def fields_of(line):
    return dict(token.split("=", 1) for token in line.split()[1:] if "=" in token)

configuration, headers, devices = {}, {}, []
current = None
for line in open(product_path, encoding="utf-8"):
    line = line.rstrip("\n")
    if not line.startswith("E7RESULT "):
        continue
    fields = fields_of(line)
    kind = fields.get("name")
    if kind == "h4_config":
        configuration[fields["item"]] = fields
    elif kind == "h4_reading" and "entry" in fields:
        current = (fields["item"], fields["state"], fields["entry"])
        headers[current] = fields
    elif kind == "h4_reading_device":
        devices.append((current, fields))

print("E7RESULT name=anchor_check_script anchors_script_sha256=%s product_sha256=%s" % (
    hashlib.sha256(open(anchors_path, "rb").read()).hexdigest(),
    hashlib.sha256(open(product_path, "rb").read()).hexdigest()))
mismatches = 0
for key, fields in devices:
    item, state, entry = key
    header = headers[key]
    config = configuration[item]
    device_bytes = int(header["device_mebibytes"]) << 20
    ring_bytes = int(config["journal_ring_bytes"])
    slots_per_region = int(config["slots_per_region"])
    levels = header["retirement_tree_internal_levels"]
    first, last = anchors.unit_area(device_bytes, ring_bytes)
    script_capacity = (last - first + 1) * anchors.SLOT
    script_window_deducted = anchors.ring_window_slots(
        device_bytes, ring_bytes, slots_per_region, 0 if levels == "none" else int(levels))[0] * anchors.SLOT
    product_window = int(fields["ring_window_bytes"])
    product_capacity = int(fields["capacity_bytes"])
    capacity_equal = script_capacity == product_capacity
    if levels == "none" and entry == "writable_mount":
        expected_window, window_rule = 0, "not_counted_before_the_first_file_version"
    else:
        expected_window, window_rule = script_window_deducted, "deducted"
    window_equal = expected_window == product_window
    mismatches += int(not (capacity_equal and window_equal))
    print("E7RESULT name=anchor_check item=%s state=%s entry=%s device=%s device_mebibytes=%s script_unit_area_bytes=%d product_capacity_bytes=%d "
          "capacity_equal=%s window_rule=%s script_ring_window_bytes=%d product_ring_window_bytes=%d ring_window_equal=%s" % (
              item, state, entry, fields["device"], header["device_mebibytes"], script_capacity, product_capacity,
              capacity_equal, window_rule, expected_window, product_window, window_equal))
print("E7RESULT name=verdict item=anchor_check readings_checked=%d mismatches=%d" % (len(devices), mismatches))
sys.exit(1 if mismatches else 0)
