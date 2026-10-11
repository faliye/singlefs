#!/usr/bin/env python3
"""把补丁里的变异行逐行打进冻结副本的草稿拷贝、跑那一行点名的 harness 档测试、看红绿，再还原那一份文件。
用法：run_rows.py <改动全文 md> <工作树（含 Cargo.toml）> <日志目录> <md 行号>...
只跑 `-p singlefs-harness` 的行（checker 档的行不跑，写「没跑」）。
测试命令经 capped.sh 7 与 run-with-memory-cap.sh 12G；退出码 250-254 是包装自己的结局，原样记。
输出一行一条：<md 行号> <名字前 40 字> <退出码> <红|绿|包装> <测试结果行>
"""
import os, subprocess, sys, shutil
md, work, logdir = sys.argv[1], sys.argv[2], sys.argv[3]
wanted = [int(x) for x in sys.argv[4:]]
os.makedirs(logdir, exist_ok=True)
lines = open(md, encoding="utf-8").read().split("\n")
repo = "/home/fy5090/code/singlefs"
for lineno in wanted:
    cols = lines[lineno - 1].split("\t")
    name, path, old, new, cmd, test = cols
    old = old.replace("\\n", "\n"); new = new.replace("\\n", "\n")
    if not cmd.startswith("-p singlefs-harness"):
        print(lineno, name[:40], "-", "没跑（checker 档）", "", flush=True); continue
    target = os.path.join(work, path)
    original = open(target, encoding="utf-8").read()
    assert original.count(old) == 1, (lineno, original.count(old))
    open(target, "w", encoding="utf-8").write(original.replace(old, new))
    try:
        # cmd 形如 "-p singlefs-harness --test X" 或 "-p singlefs-harness --test X -- filter" 或 "-p singlefs-harness --lib -- filter"
        parts = cmd.split(" -- ")
        head = parts[0].split()
        filters = parts[1].split() if len(parts) > 1 else []
        if test and test not in filters:
            filters.append(test)
        os.environ.setdefault("RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON", "草稿副本里单跑变异行取宽上限")
        os.environ.setdefault("RUN_WITH_MEMORY_CAP_PARALLEL_REASON", "同一会话下正推腿与攻方腿各跑各的")
        argv = ["nice", "-n", "19", "bash", f"{repo}/scripts/capped.sh", "7", "timeout", "220",
                "bash", f"{repo}/scripts/run-with-memory-cap.sh", "12G",
                "cargo", "test", "--offline", "--manifest-path", f"{work}/Cargo.toml", *head, "--", *filters]
        log = os.path.join(logdir, f"row-{lineno}.log")
        with open(log, "w") as out:
            code = subprocess.run(argv, stdout=out, stderr=subprocess.STDOUT, cwd=repo).returncode
        text = open(log, encoding="utf-8", errors="replace").read().split("\n")
        result = next((l for l in text if l.startswith("test result")), "")
        verdict = "包装" if 250 <= code <= 254 else ("红" if code == 101 else ("绿" if code == 0 else f"其他{code}"))
        compile_err = any("error[" in l or l.startswith("error:") and "could not compile" in l for l in text)
        if compile_err: verdict = "编译错"
        print(lineno, name[:40], code, verdict, result, flush=True)
    finally:
        open(target, "w", encoding="utf-8").write(original)
