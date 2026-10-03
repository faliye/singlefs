#!/usr/bin/env bash
# 探针 1：GC 两条用例整读的活文件集合有多大（在副本里临时加一行 eprintln，跑完还原并核 sha256）。
# 环境：MUT_W、CARGO_TARGET_DIR；整条脚本经 run-with-memory-cap.sh 跑。
set -u
F="$MUT_W/crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs"
cp "$F" "$F.bak"
python3 - "$F" <<'PY'
import sys
p = sys.argv[1]
t = open(p, encoding="utf-8").read()
o = "    Ok(WholeReadOfTheLiveFiles {\n        root_txg: read_only.effective_root.checkpoint_txg,"
assert t.count(o) == 1
t = t.replace(o, '    eprintln!("PROBE live_files={} sizes={:?} root={:?}", live_files.len(), live_files.iter().map(|(i, b)| (*i, b.len())).collect::<Vec<_>>(), read_only.effective_root.checkpoint_txg);\n' + o)
open(p, "w", encoding="utf-8").write(t)
PY
cd "$MUT_W" && cargo test --offline -p singlefs-harness --test a_lone_surviving_device_reads_back_what_it_read_before_the_unmount -- --nocapture --test-threads=1 2>&1 | grep -E 'PROBE|test result|^test '
mv "$F.bak" "$F"; sha256sum "$F"
