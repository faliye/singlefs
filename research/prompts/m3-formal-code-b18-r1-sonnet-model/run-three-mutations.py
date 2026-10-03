#!/usr/bin/env python3
"""在副本 WS 上逐条套 18-2 的三行变异（行文取自 _m3-formal-code-b18-r1-diff.md「18-2 变异」），跑各自的 harness 档用例，还原。
用法：python3 run-three-mutations.py <副本工作区根> <仓根> ；输出每条变异跑指定用例的退出码与 test result 行。
只改副本，不碰仓。"""
import os, subprocess, sys

ws, repo = sys.argv[1], sys.argv[2]
target = os.path.join(os.path.dirname(ws), "target")
T = "unmount_on_a_version_without_trees_writes_nothing_and_the_model_answers_zero_roots_and_agrees"
U = "an_unmount_that_pushes_empty_publishes_on_a_version_without_trees_is_reported_at_the_publish_count"
rows = [
    ("M1 model.rs has_trees -> true", "crates/singlefs-harness/src/model.rs",
     "        let roots = if current.contents.has_trees() {", "        let roots = if true {", T),
    ("M2 mount.rs 卸载树表 0 条那一支写 4096 字节", "crates/singlefs-core/src/mount.rs",
     "        PoolVersion::WithoutFile(current_version_without_file) => {\n            return Ok(Unmounted::NothingWrittenOnAVersionWithoutFile {",
     "        PoolVersion::WithoutFile(current_version_without_file) => {\n            devices[0].1.write_at(crate::address::DeviceOffsetInBytes(1 << 30), &[0xFF; 4096], crate::block_device::WriteDurability::Plain).expect(\"变异\");\n            return Ok(Unmounted::NothingWrittenOnAVersionWithoutFile {", T),
    ("M3 model.rs judge_roots 不比条数", "crates/singlefs-harness/src/model.rs",
     "        if expected_roots.len() != observed_roots.len() {", "        if false {", U),
]
env = dict(os.environ, CARGO_TARGET_DIR=target)
for name, rel, old, new, test in rows:
    path = os.path.join(ws, rel)
    original = open(path, encoding="utf-8").read()
    assert original.count(old) == 1, (name, original.count(old))
    open(path, "w", encoding="utf-8").write(original.replace(old, new))
    try:
        cmd = ["nice", "-n", "19", "bash", f"{repo}/research/scripts/capped.sh", "6", "bash",
               f"{repo}/research/scripts/run-with-memory-cap.sh", "1G", "cargo", "test", "--offline", "-p", "singlefs-harness",
               "--test", "model_answers_unmount_with_the_floor_at_the_current_version", "--", test]
        out = subprocess.run(cmd, cwd=ws, env=env, capture_output=True, text=True)
        lines = [l for l in (out.stdout + out.stderr).splitlines() if l.startswith("test result") or l.startswith("test ") or "panicked" in l]
        print(name, "exit=", out.returncode)
        for l in lines:
            print("   ", l)
    finally:
        open(path, "w", encoding="utf-8").write(original)
