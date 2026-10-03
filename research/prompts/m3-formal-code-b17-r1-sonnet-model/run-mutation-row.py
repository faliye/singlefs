#!/usr/bin/env python3
"""在副本工作区里逐行施加一条 crates 变异表格式的变异、跑点名的 harness 档测试、还原。
用法：run-mutation-row.py <工作区> <变异文件> <行号> <日志目录>
变异文件每行六段（变异名、文件、原文、替换文、cargo test 参数、必须红的测试名），\\n 表示换行。
只在草稿目录的副本上改；cargo test 经内存包装与线程上限。"""
import os, shlex, subprocess, sys, shutil

ws, mutfile, lineno, logdir = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4]
os.makedirs(logdir, exist_ok=True)
lines = open(mutfile, encoding="utf-8").read().split("\n")
row = lines[lineno - 1].split("\t")
assert len(row) == 6, (lineno, len(row))
name, rel, old, new, args, must_red = row
old = old.replace("\\n", "\n")
new = new.replace("\\n", "\n")
path = os.path.join(ws, rel)
src = open(path, encoding="utf-8").read()
hits = src.count(old)
print(f"row {lineno}: {name}\n  file {rel}\n  anchor hits {hits}")
if hits != 1:
    print("  ANCHOR NOT EXACTLY ONCE"); sys.exit(3)
backup = path + ".orig-mut"
shutil.copy2(path, backup)
try:
    open(path, "w", encoding="utf-8").write(src.replace(old, new))
    scripts = "/home/fy5090/code/singlefs/research/scripts"
    cmd = ["nice", "-n", "19", "bash", f"{scripts}/run-with-memory-cap.sh", "4G", "bash", f"{scripts}/capped.sh", "8",
           "cargo", "test", "--offline"] + shlex.split(args)
    log = os.path.join(logdir, f"row{lineno}.log")
    with open(log, "w") as f:
        env = dict(os.environ, RUN_WITH_MEMORY_CAP_PARALLEL_REASON="多个 agent 各跑各的变异副本", RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="副本上施加变异要重新编译，峰值表只记过未变异那一次")
        p = subprocess.run(cmd, cwd=ws, stdout=f, stderr=subprocess.STDOUT, timeout=235, env=env)
    print(f"  exit {p.returncode}")
    text = open(log, encoding="utf-8", errors="replace").read()
    failed = [l for l in text.split("\n") if l.startswith("test ") and l.endswith("FAILED")]
    for l in text.split("\n"):
        if l.startswith("test result"):
            print("  " + l)
    for l in failed:
        print("  " + l)
    print(f"  named test red: {any(must_red in l for l in failed)}")
finally:
    shutil.move(backup, path)
    os.utime(path, None)  # 还原后把修改时间置成现在：cargo 按修改时间判新旧，不置的话会沿用变异那一次的编译产物
