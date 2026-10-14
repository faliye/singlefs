#!/usr/bin/env python3
"""Z6：脚本文件头 # long-run-hosts: 的各种写法，经 detached-run.sh long-run-check 判。python3 z6_header.py <树> <输出目录>"""
import os, subprocess, sys, importlib.util
tree, out = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
spec = importlib.util.spec_from_file_location("lib_long_run", os.path.join(tree, ".claude/hooks/lib_long_run.py"))
lr = importlib.util.module_from_spec(spec); sys.dont_write_bytecode = True; spec.loader.exec_module(lr)
C = lr.CANONICAL_HOSTS_SENTENCE
os.makedirs(out, exist_ok=True)
H = "#!/usr/bin/env bash\n# long-run-shards: 按行号 mod 片数切\n# long-run-resume: 结果表里已有的行跳过，重起同一条命令只补没做完的\n"
cases = {
 "canon": f"# long-run-hosts: {C}\n",
 "canon-period-zh": f"# long-run-hosts: {C}。\n",
 "canon-period-en": f"# long-run-hosts: {C}.\n",
 "canon-semicolon-note": f"# long-run-hosts: {C}；双机时两台各取这一个数\n",
 "canon-then-plain-comment-live": f"# long-run-hosts: {C}\n# 说明：每台线程按此刻空闲现算，机器忙时按负载降\n",
 "canon-then-second-hosts-key": f"# long-run-hosts: {C}\n# long-run-hosts: 每台线程按此刻空闲现算\n",
 "canon-then-second-hosts-key-same": f"# long-run-hosts: {C}\n# long-run-hosts: {C}\n",
 "split-canon-over-two-keys": f"# long-run-hosts: {C[:20]}\n# long-run-hosts: {C[20:]}\n",
 "canon-then-blank-then-comment": f"# long-run-hosts: {C}\n\n# 说明：现算\n",
 "canon-dup-shards-key": f"# long-run-hosts: {C}\n# long-run-shards: 再按文件名切一层\n",
}
for name, hosts in cases.items():
    path = os.path.join(out, name + ".sh")
    with open(path, "w") as handle:
        handle.write(H + hosts + 'touch "$1"\n')
    r = subprocess.run(["bash", os.path.join(tree, "scripts/hosts/detached-run.sh"), "long-run-check", "--", "bash", path, "x"], capture_output=True, text=True, cwd=out)
    last = [l for l in (r.stdout + r.stderr).splitlines() if l.strip()]
    print(f"{name}\t{r.returncode}\t{(last[0] if last else '')[:150]}")
