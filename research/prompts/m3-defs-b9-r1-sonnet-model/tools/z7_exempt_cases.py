#!/usr/bin/env python3
"""Z7：lib_long_run.py exempt-check 的各种基准。python3 z7_cases.py <树> <临时根目录>
每例造 <根>/<例名>/{.claude/long-run-live-hosts-exempt, scripts/p.sh} 和基准表文件，跑
  python3 -B <树>/.claude/hooks/lib_long_run.py exempt-check <根>/<例名> [--base-table <基准表>]
打 例名 / 退出码 / 输出首行"""
import hashlib, os, shutil, subprocess, sys
tree, base = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
lib = os.path.join(tree, ".claude/hooks/lib_long_run.py")
HEAD = ("#!/usr/bin/env bash\n# long-run-shards: 按行号 mod 片数切\n# long-run-resume: 结果表里已有的行跳过，重起同一条命令只补没做完的\n"
        "# long-run-hosts: 台数从 layer0-shard-configuration-check.sh --emit-hosts 读，每台线程与内存用 host-resources.py --json 现算\n")
def script(slots): return HEAD + f"slots={slots}\ntouch \"$1\"\n"
def sha(text): return hashlib.sha256(text.encode()).hexdigest()
S7, S8 = script(7), script(8)
def table(path, text, reason="E171 入口，被 sha256 钉住，真按此刻空闲现算"):
    return f"{path}  {sha(text)}  # {reason}\n"
P = "scripts/p.sh"
cases = [
 # 例名, 现在的脚本, 现在的表, 基准表 (None = 不给 --base-table)
 ("不给基准", S7, table(P, S7), None),
 ("基准与现在逐字相同", S7, table(P, S7), table(P, S7)),
 ("改了脚本再重钉", S8, table(P, S8), table(P, S7)),
 ("基准是不带 sha256 的旧写法而现在重钉", S8, table(P, S8), f"{P}  # E171 入口，被 sha256 钉住\n"),
 ("基准是不带 sha256 的旧写法而路径没变", S7, table(P, S7), f"{P}  # E171 入口，被 sha256 钉住\n"),
 ("基准里没有这个路径", S7, table(P, S7), table("scripts/other.sh", S7)),
 ("基准是空文件", S7, table(P, S7), ""),
 ("基准只有注释", S7, table(P, S7), "# 只有注释\n"),
 ("基准里 sha256 写成大写", S8, table(P, S8), f"{P}  {sha(S7).upper()}  # E171 入口，被 sha256 钉住\n"),
 ("基准里同一路径两行不同 sha256 而现在是其中一个", S7, table(P, S7), table(P, S7) + table(P, S8)),
 ("基准里同一路径两行不同 sha256 而现在是第三个", script(9), table(P, script(9)), table(P, S7) + table(P, S8)),
 ("表里 sha256 与脚本不符而基准相同", S8, table(P, S7), table(P, S7)),
 ("表里删了这一行", S8, "", table(P, S7)),
]
os.makedirs(base, exist_ok=True)
for name, script_text, table_text, base_text in cases:
    root = os.path.join(base, name)
    shutil.rmtree(root, ignore_errors=True)
    os.makedirs(os.path.join(root, ".claude")); os.makedirs(os.path.join(root, "scripts"))
    open(os.path.join(root, P), "w").write(script_text)
    open(os.path.join(root, ".claude/long-run-live-hosts-exempt"), "w").write(table_text)
    command = ["python3", "-B", lib, "exempt-check", root]
    if base_text is not None:
        base_file = os.path.join(root, "base-table.txt")
        open(base_file, "w").write(base_text)
        command += ["--base-table", base_file]
    r = subprocess.run(command, capture_output=True, text=True)
    lines = [l.strip() for l in (r.stdout + r.stderr).splitlines() if l.strip()]
    print(f"{name}\t{r.returncode}\t{' | '.join(lines[:3])[:300]}")
