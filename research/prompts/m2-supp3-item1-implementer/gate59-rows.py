# 门禁 59 号同一个读法（.claude/gate.d/59-crates-mutation-replay.sh）：先对整张表做锚点预扫（六段、原文恰好命中一次），
# 再把仓拷到临时目录、只复跑 sys.argv[2:] 点名的行，`cargo test --offline <参数>`，点名的测试要 FAILED。
import os, re, shutil, subprocess, sys, tempfile
root = sys.argv[1]
wanted = {int(number) for number in sys.argv[2:]}
table = "crates/mutations.tsv"
def unescape(text): return text.replace("\\n", "\n")
rows = []
with open(os.path.join(root, table), encoding="utf-8") as handle:
    for line_number, line in enumerate(handle, 1):
        line = line.rstrip("\n")
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 6 or any(field == "" for field in fields):
            print(f"  ✗ {table}:{line_number} 不是六段"); sys.exit(1)
        rows.append((line_number, fields[0], fields[1], unescape(fields[2]), unescape(fields[3]), fields[4], fields[5]))
stale = []
for line_number, name, path, old, new, _args, _expected in rows:
    full = os.path.join(root, path)
    if not os.path.isfile(full):
        stale.append(f"{table}:{line_number} {name}：文件 {path} 不存在"); continue
    count = open(full, encoding="utf-8").read().count(old)
    if count != 1:
        stale.append(f"{table}:{line_number} {name}：原文在 {path} 里命中 {count} 次，要恰好 1 次")
if stale:
    print("  ✗ 变异表的锚点腐化：")
    for item in stale: print(f"      {item}")
    sys.exit(1)
print(f"  ✓ 锚点预扫：{len(rows)} 条变异，原文都恰好命中一次")
work = tempfile.mkdtemp(prefix="singlefs-mutation-replay-", dir="/tmp/claude-1000/m2-supp3-item1")
target = "/tmp/claude-1000/m2-supp3-item1/gate59-target"
try:
    for item in ["crates", "Cargo.toml", "Cargo.lock", "litmus", "research/results"]:
        source = os.path.join(root, item)
        if not os.path.exists(source): continue
        destination = os.path.join(work, item)
        if os.path.isdir(source):
            shutil.copytree(source, destination, ignore=shutil.ignore_patterns("target"))
        else:
            os.makedirs(os.path.dirname(destination), exist_ok=True); shutil.copy2(source, destination)
    environment = dict(os.environ); environment["CARGO_TARGET_DIR"] = target
    failures = []
    for line_number, name, path, old, new, args, expected in rows:
        if line_number not in wanted: continue
        full = os.path.join(work, path)
        pristine = open(full, encoding="utf-8").read()
        assert pristine.count(old) == 1, (path, name)
        open(full, "w", encoding="utf-8").write(pristine.replace(old, new))
        try:
            run = subprocess.run(["nice", "-n", "19", "cargo", "test", "--offline"] + args.split(), cwd=work, env=environment, capture_output=True, text=True)
        finally:
            open(full, "w", encoding="utf-8").write(pristine)
        output = run.stdout + run.stderr
        with open(f"/tmp/claude-1000/m2-supp3-item1/gate59-row-{line_number}.log", "w", encoding="utf-8") as log:
            log.write(output)
        red_pattern = re.compile(r"^test (\S+::)?" + re.escape(expected) + r" \.\.\. FAILED$", re.M)
        ran_pattern = re.compile(r"^test (\S+::)?" + re.escape(expected) + r" \.\.\. ", re.M)
        if run.returncode != 0 and red_pattern.search(output):
            print(f"  ✓ 第 {line_number} 行 {name}：{expected} 红了")
        elif ran_pattern.search(output):
            failures.append(f"{table}:{line_number} {name}：{expected} 没红（退出码 {run.returncode}）")
        else:
            failures.append(f"{table}:{line_number} {name}：点名的测试 {expected} 没跑到（退出码 {run.returncode}）")
finally:
    shutil.rmtree(work, ignore_errors=True)
for item in failures: print(f"  ✗ {item}")
sys.exit(1 if failures else 0)
