#!/usr/bin/env python3
"""在自己的副本里拼出两个探针测试文件（不进仓）：
 zz_probe_all_zero_tree_table.rs = 根记录 34-2 新用例文件的前半（造盘助手）+ probes/probe_all_zero_tail.rs
 zz_probe_q1.rs                 = checker_known_bad_images.rs 整份 + probes/probe_q1_tail.rs
用法 build_probes.py <副本根> <模型目录>"""
import sys
root, model = sys.argv[1], sys.argv[2]
tests = f"{root}/crates/singlefs-harness/tests"
src = open(f"{tests}/root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts.rs", encoding="utf-8").read()
head = src[:src.index("/// 场景：第一个文件那条根的树表指针算法类型写 1")]
head = head.replace("use singlefs_harness::memory_pool::SparseBlockDevice;",
                    "use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};\nuse singlefs_checker::walk::check_pool_image;\nuse singlefs_checker::image::InvariantVerdict;")
open(f"{tests}/zz_probe_all_zero_tree_table.rs", "w", encoding="utf-8").write(head + open(f"{model}/probes/probe_all_zero_tail.rs", encoding="utf-8").read())
open(f"{tests}/zz_probe_q1.rs", "w", encoding="utf-8").write(open(f"{tests}/checker_known_bad_images.rs", encoding="utf-8").read() + open(f"{model}/probes/probe_q1_tail.rs", encoding="utf-8").read())
print("probes built")
