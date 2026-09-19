import os, shutil, subprocess, sys, time
sys.path.insert(0, "/tmp/claude-1000/m2-supp3-item1/proof")
from mutations import MUTATIONS
copy = "/tmp/claude-1000/m2-supp3-item1/proof/repo"
original = "/tmp/claude-1000/m2-supp3-item1/proof/pristine"
log = open("/tmp/claude-1000/m2-supp3-item1/proof/proofs.log", "a", encoding="utf-8")
def run(binary):
    arguments = ["nice", "-n", "19", "cargo", "test", "-p", "singlefs-harness"]
    arguments += ["--lib"] if binary == "--lib" else ["--test", binary]
    started = time.time()
    result = subprocess.run(arguments, cwd=copy, capture_output=True, text=True)
    lines = (result.stdout + result.stderr).splitlines()
    verdicts = [line for line in lines if line.startswith("test ") and (" ... ok" in line or " ... FAILED" in line or " ... ignored" in line)]
    panics = [line for line in lines if "panicked at" in line or line.startswith("assertion") or "left:" in line or "right:" in line or "要以已知红收尾" in line or "镜像逐字节" in line]
    return result.returncode, verdicts, panics, time.time() - started, lines
for binary in ["second_transaction_supplement_three_random_history"]:
    code, verdicts, panics, seconds, _ = run(binary)
    log.write(f"== 基线 {binary} 退出码 {code}，{seconds:.0f} 秒\n")
    for line in verdicts: log.write(f"  {line}\n")
    log.flush()
for name, path, old, new, binary in MUTATIONS:
    full = os.path.join(copy, path)
    text = open(full, encoding="utf-8").read()
    count = text.count(old)
    if count != 1:
        log.write(f"== {name}：原文命中 {count} 次，跳过\n"); log.flush(); continue
    open(full, "w", encoding="utf-8").write(text.replace(old, new))
    try:
        code, verdicts, panics, seconds, lines = run(binary)
    finally:
        shutil.copy2(os.path.join(original, path), full)
        os.utime(full, None)
    log.write(f"== {name}（{path}；{binary}）退出码 {code}，{seconds:.0f} 秒\n")
    for line in verdicts: log.write(f"  {line}\n")
    for line in panics[:12]: log.write(f"    | {line[:300]}\n")
    log.flush()
log.write("== 全部跑完\n")
