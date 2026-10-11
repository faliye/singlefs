#!/usr/bin/env python3
"""草稿：把一批变异行逐条套在草稿副本上跑指定用例，报红还是绿（改的是草稿副本里的文件，跑完还原）。
用法：mutate-harness.py <行文件> <从第几条> <到第几条（含）>；行文件每行：标签\t文件\t旧串\t新串\t用例参数\t用例名（旧新串里 \\n 是换行）"""
import os, subprocess, sys, shutil
W = os.path.expanduser("~/.local/share/singlefs/work/m3-batch10y-code-r1-sonnet")
REPO = W + "/repo"
rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1], encoding="utf-8") if l.strip()]
lo, hi = int(sys.argv[2]), int(sys.argv[3])
env = dict(os.environ, CARGO_TARGET_DIR=W + "/target")
for index, row in enumerate(rows, 1):
    if not (lo <= index <= hi):
        continue
    label, file, old, new, command = row[0], row[1], row[2], row[3], row[4]
    old = old.replace("\\n", "\n")
    new = new.replace("\\n", "\n")
    path = REPO + "/" + file
    original = open(path, encoding="utf-8").read()
    count = original.count(old)
    if count != 1:
        print(f"[{index}] {label}: 旧串命中 {count} 次，跳过", flush=True)
        continue
    open(path + ".mut", "w", encoding="utf-8").write(original.replace(old, new, 1))
    os.replace(path + ".mut", path)
    try:
        script = W + "/mut-run.sh"
        open(script, "w").write(
            "#!/usr/bin/env bash\nset -uo pipefail\ncd \"%s\"\nexport CARGO_TARGET_DIR=\"%s\"\ncargo test %s\n" % (REPO, W + "/target", command)
        )
        result = subprocess.run(
            ["nice", "-n", "19", "bash", "/home/fy5090/code/singlefs/scripts/hosts/capped.sh", "4",
             "bash", "/home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh", "4G", "bash", script],
            capture_output=True, text=True, env=env)
        out = result.stdout + result.stderr
        verdict = "红（被抓）" if result.returncode == 101 else ("绿（没抓）" if result.returncode == 0 else f"退出码 {result.returncode}")
        summary = [l for l in out.splitlines() if l.startswith("test result") or "error[" in l][:2] + [l.split(" ... ")[0][5:] for l in out.splitlines() if l.startswith("test ") and l.endswith("FAILED")][:6]
        print(f"[{index}] {label} => {verdict} rc={result.returncode} {summary}", flush=True)
    finally:
        open(path + ".mut", "w", encoding="utf-8").write(original)
        os.replace(path + ".mut", path)
