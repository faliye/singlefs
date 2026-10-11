#!/usr/bin/env python3
# 用法: variant3.py <名字> <文件(相对 probe-tree)> <旧串文件> <新串文件> [-- 额外的 cargo test 参数…]
# 一次 cargo 同时跑 lib 的 inode_tree:: 和探针；替换恰好一次；信号/异常都还原并 touch。
import subprocess, sys, shutil, os, re, signal
name, rel, oldf, newf = sys.argv[1:5]
cargo_args = sys.argv[6:] if len(sys.argv) > 5 and sys.argv[5] == "--" else []
root = os.path.expanduser("~/.local/share/singlefs/work/m3-batch50-code-r1-sonnet")
tree = f"{root}/probe-tree"; path = f"{tree}/{rel}"; bak = path + ".orig"
def restore():
    if os.path.exists(bak):
        shutil.move(bak, path); os.utime(path)
def on_term(signum, frame):
    restore(); sys.exit(143)
signal.signal(signal.SIGTERM, on_term); signal.signal(signal.SIGINT, on_term)
orig = open(path).read()
pairs = [(open(o).read().rstrip("\n"), open(n).read().rstrip("\n")) for o, n in zip(oldf.split(","), newf.split(","))]
text = orig
for old, new in pairs:
    assert text.count(old) == 1, f"旧串命中 {text.count(old)} 次: {old[:40]}"
    text = text.replace(old, new)
shutil.copy(path, bak)
open(path, "w").write(text); os.utime(path)
cmd = ["nice", "-n", "19", "bash", "/home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh", "10G", "bash",
       "/home/fy5090/code/singlefs/scripts/capped.sh", "8", "cargo", "test", "--offline",
       "--target-dir", f"{root}/target-h"] + cargo_args + ["--", "--nocapture", "--test-threads=4"]
try:
    r = subprocess.run(cmd, cwd=tree, capture_output=True, text=True, env=dict(os.environ, PROBE_SEEDS="12", PROBE_STEPS="9000"))
finally:
    restore()
out = r.stdout + r.stderr
open(f"{root}/var-{name}.log", "w").write(out)
failed = re.findall(r"^test (\S+) \.\.\. FAILED", out, re.M)
ok_inode = len(re.findall(r"^test \S+ \.\.\. ok", out, re.M))
summ = re.findall(r"^test result: .*$", out, re.M)
print(f"VARIANT {name} exit={r.returncode} compile_error={'error[' in out or 'could not compile' in out} tests_ok={ok_inode}")
print(f"  red={failed}")
print(f"  {summ}")
