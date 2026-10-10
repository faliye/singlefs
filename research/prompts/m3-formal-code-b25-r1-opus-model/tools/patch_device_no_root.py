#!/usr/bin/env python3
"""攻方副本：OPUS_NO_ROOT=1 时 Q5 那一形（对拍那一形与主取样点同一个函数造）的 judged_root_index 取 usize::MAX，
模拟「截出来的写流里没有可判的根槽写」那一种情形，只看控制流走到哪一个出口。用法：patch_device_no_root.py <副本根>"""
import sys, pathlib
p = pathlib.Path(sys.argv[1]) / "crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs"
s = p.read_text()
a = """        writes_and_segments_with_stream_indexes_and_entries(&kept, &[], &common::geometry());
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .unwrap_or(usize::MAX);"""
b = """        writes_and_segments_with_stream_indexes_and_entries(&kept, &[], &common::geometry());
    let judged_root_index = if std::env::var("OPUS_NO_ROOT").is_ok() {
        usize::MAX
    } else {
        writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .unwrap_or(usize::MAX)
    };"""
assert s.count(a) == 1, s.count(a); s = s.replace(a, b)
tmp = pathlib.Path(str(p) + ".tmp"); tmp.write_text(s); tmp.replace(p)
print("patched")
