#!/usr/bin/env python3
"""攻方副本：理想模型每写出一条根时，txg 是 100 的倍数或大于 1270 就往 stderr 打一行 OPUSBOUND
（中央映射树节点数上界、每盘占槽上界），只读不改判法。用法：patch_model_bound_print.py <副本根>"""
import sys, pathlib
p = pathlib.Path(sys.argv[1]) / "crates/singlefs-harness/src/model.rs"
s = p.read_text()
a = '''            occupied_slots_upper_bound_per_device: previous.occupied_slots_upper_bound_per_device
                + slots_written,
            central_mapping_tree_nodes_upper_bound,'''
b = '''            occupied_slots_upper_bound_per_device: {
                if checkpoint_txg.0 % 100 == 0 || checkpoint_txg.0 > 1270 {
                    eprintln!("OPUSBOUND txg={} mapping_nodes_upper_bound={} occupied_upper_bound={}", checkpoint_txg.0, central_mapping_tree_nodes_upper_bound, previous.occupied_slots_upper_bound_per_device + slots_written);
                }
                previous.occupied_slots_upper_bound_per_device + slots_written
            },
            central_mapping_tree_nodes_upper_bound,'''
assert s.count(a) == 1
tmp = pathlib.Path(str(p) + ".tmp"); tmp.write_text(s.replace(a, b)); tmp.replace(p)
print("patched")
