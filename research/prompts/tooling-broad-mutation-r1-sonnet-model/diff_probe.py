import importlib.util, os, sys, subprocess
def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path); m = importlib.util.module_from_spec(spec)
    sys.modules[name] = m; spec.loader.exec_module(m); return m
root = "/home/fy5090/code/singlefs"
D = "/tmp/claude-1000/tooling-broad-mutation-r1-sonnet"
os.makedirs(D + "/old", exist_ok=True)
open(D + "/old/lib_heavy_tests.py", "w").write(subprocess.run(["git", "-C", root, "show", "HEAD:.claude/hooks/lib_heavy_tests.py"], capture_output=True, text=True, check=True).stdout)
open(D + "/old/lib_shell_words.py", "w").write(open(root + "/.claude/hooks/lib_shell_words.py").read())
old = load("lib_heavy_tests", D + "/old/lib_heavy_tests.py")
oldfn = old.heavy_tests_in_command_text
new = load("lib_heavy_tests_new", root + "/.claude/hooks/lib_heavy_tests.py")
newfn = new.heavy_tests_in_command_text
cmds = []
# 语料：研究脚本 --selftest（全部 SELFTEST_RUNNERS）、门禁阶段、mutate.sh、cargo 各形、shard 驱动、e152、gate.sh
import re
src = open(root + "/.claude/gate.d/code-tooling.sh", encoding="utf-8").read()
block = src[src.index("SELFTEST_RUNNERS=("): src.index("SELFTEST_RUNNERS_STARTED_ALONE=")]
cmds += re.findall(r'"([^"]+)"', block)
for stage in sorted(os.listdir(root + "/.claude/gate.d")):
    if stage.endswith(".sh"):
        cmds += [f"bash .claude/gate.d/{stage}", f"bash .claude/gate.d/{stage} --list-items", f"bash .claude/gate.d/{stage} --check x"]
cmds += [
 "cargo test", "cargo test -p singlefs-core", "cargo test -p singlefs-checker-tier", "cargo test --workspace", "cargo nextest run -p singlefs-harness",
 "cargo mutants -p singlefs-core", "cargo build --release", "cargo run --release --bin e152-file-system-benchmark",
 "bash research/scripts/mutate.sh crates/mutations.tsv", "bash research/scripts/mutate.sh --selftest",
 "bash research/scripts/mutation-shard-run.sh", "bash research/scripts/mutation-shard-run.sh --selftest",
 "bash research/scripts/layer0-shard-run.sh crash-case:x .", "bash research/scripts/gate-staged.sh", "bash .claude/scripts/check.sh",
 "python3 research/scripts/crash-amplification.sh --selftest", "bash research/scripts/crash-amplification.sh --list-items",
 "bash research/scripts/e152-run.sh", "bash research/scripts/vm-bench.sh --selftest", "herd7 x.litmus", "qemu-system-x86_64 -m 1G",
 "python3 -m py_compile research/scripts/broad-mutation.py", "cat research/scripts/broad-mutation.py", "grep -n broad-mutation research/scripts/broad-mutation.py",
 "git add research/scripts/broad-mutation.py", "bash -n research/scripts/broad-mutation.py", "python3 - research/scripts/broad-mutation.py",
 "wc -l research/scripts/broad-mutation.py", "sha256sum research/scripts/broad-mutation.py", "chmod +x research/scripts/broad-mutation.py",
 "python3 research/scripts/crates-mutation-rows.py --selftest", "python3 research/scripts/cite-check.py --selftest", "python3 .claude/hooks/lib_heavy_tests.py --selftest",
]
diffs = 0
for c in cmds:
    a = [t.kind for t in oldfn(c, root)]; b = [t.kind for t in newfn(c, root)]
    if a != b:
        diffs += 1; print("DIFF", c, a, b)
print("语料条数", len(cmds), "新旧不同", diffs)
