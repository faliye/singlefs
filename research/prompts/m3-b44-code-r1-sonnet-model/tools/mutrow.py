#!/usr/bin/env python3
"""按变异表第 N 行在副本里改坏一处、跑点名的 cargo test、还原，打一行确定性的汇总。
用法：mutrow.py <变异表> <副本根> <行号> <日志文件>
汇总行：row= name= anchor_hits= cargo_rc= red_tests=<排序后用 | 连> result=<test result 行去掉用时>
副本里被改的那份文件在 finally 里还原；收到 SIGTERM 也走 finally。"""
import os, re, signal, subprocess, sys

table, copy, row, log = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4]
signal.signal(signal.SIGTERM, lambda *_: sys.exit(143))
line = open(table, encoding="utf-8").read().split("\n")[row - 1]
name, path, orig, repl, args, named_red = line.split("\t")
orig = orig.replace("\\n", "\n")
repl = repl.replace("\\n", "\n")
target = os.path.join(copy, path)
text = open(target, encoding="utf-8").read()
hits = text.count(orig)
if hits != 1:
    print(f"row={row} name={name} anchor_hits={hits} cargo_rc=- red_tests=- result=-")
    sys.exit(3)
open(target, "w", encoding="utf-8").write(text.replace(orig, repl))
try:
    env = dict(os.environ)
    env["SINGLEFS_RUN_SCOPE"] = ("种类=target; 跑什么=变异行 %d 点名的 cargo test 目标; 范围=1 个测试目标; "
                                 "为什么=核 m3-b44 代码轮正推腿各格的用例在改坏一处时是否红在点名的测试上" % row)
    r = subprocess.run(["cargo", "test", "--offline"] + args.split(), cwd=copy, env=env,
                       capture_output=True, text=True)
    open(log, "w").write(r.stdout + "\n--- stderr ---\n" + r.stderr)
    red = sorted(set(m.group(1) for m in re.finditer(r"^test (\S+)(?: - should panic)? \.\.\. FAILED$", r.stdout, re.M)))
    results = [re.sub(r"; finished in [0-9.]+s", "", l) for l in r.stdout.split("\n") if l.startswith("test result")]
    named_hit = any(t == named_red or t.endswith("::" + named_red) for t in red)
    print(f"row={row} name={name} anchor_hits=1 cargo_rc={r.returncode} named_test_red={named_hit} "
          f"red_tests={'|'.join(red)} result={' / '.join(results)}")
finally:
    open(target, "w", encoding="utf-8").write(text)
