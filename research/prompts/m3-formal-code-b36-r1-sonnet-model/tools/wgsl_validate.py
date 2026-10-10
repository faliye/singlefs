#!/usr/bin/env python3
"""把冻结副本的两个池级 checker 内核源码照 crash_judge_gpu.rs 的 checker_walk_kernel_source / checker_scan_kernel_source 拼起来，
交 naga（单独的小程序 naga-check，与仓里单测 parse_and_validate 同一套调用）解析与校验；再把 mutations.tsv 里第 36 批 GPU 四行（目标是 wgsl 的）
逐行施加到拼好的源码上再校验一遍（变异要能编，不能是编译失败的假红）。不跑 GPU。
用法 wgsl_validate.py <树根> <输出目录> <naga-check 可执行文件>"""
import json, os, subprocess, sys
root, out, tool = sys.argv[1], sys.argv[2], sys.argv[3]
os.makedirs(out, exist_ok=True)
def rd(name): return open(f"{root}/crates/singlefs-checker-tier/src/{name}", encoding="utf-8").read()
header = ("const VERDICT_WORDS: u32 = 14u;\nconst POOL_CHECKER_VIOLATED_FIRST_WORD: u32 = 8u;\nconst POOL_CHECKER_VIOLATED_WORDS: u32 = 4u;\n"
          "const POOL_CHECKER_REACHED_WORD: u32 = 12u;\nconst POOL_CHECKER_UNDECIDABLE_WORD: u32 = 13u;\n")
common, ccommon = rd("crash_judge_common.wgsl"), rd("crash_judge_checker_common.wgsl")
walk_src = f"{header}\n{common}\n{ccommon}\n{rd('crash_judge_checker.wgsl')}"
scan_src = f"{header}\n{common}\n{ccommon}\n{rd('crash_judge_checker_scan.wgsl')}"
def run(label, text):
    p = f"{out}/{label}.wgsl"; open(p, "w", encoding="utf-8").write(text)
    r = subprocess.run([tool, p], capture_output=True, text=True)
    print(label, "rc=%d" % r.returncode, (r.stdout.strip().splitlines() or [""])[0][:160]); return r.returncode
rc = run("walk-unmutated", walk_src) + run("scan-unmutated", scan_src)
tsv = [l.split("\t") for l in open(f"{root}/crates/mutations.tsv", encoding="utf-8").read().split("\n") if l.count("\t") == 5]
rows = json.load(open(sys.argv[4])) if len(sys.argv) > 4 else []
for r in rows:
    if not r["source"].endswith(".wgsl"): continue
    name, source, find, repl, args, test = next(t for t in tsv if t[0] == r["name"])
    f, g = find.replace("\\n", "\n"), repl.replace("\\n", "\n")
    which = "walk" if source.endswith("crash_judge_checker.wgsl") else "scan"
    base = walk_src if which == "walk" else scan_src
    assert base.count(f) == 1, (name, base.count(f))
    rc += run(f"mut-{r['k'] if 'k' in r else name[:20]}-{which}", base.replace(f, g))
print("total rc", rc)
