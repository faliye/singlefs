# 探针（只读、不起 Verus、不起真包装）：空真格（check_vacuity）用真仓的登记与核，包装换成桩，三种格各跑一遍：
#   notrun 全部退 252；mixed 奇数次退 252、偶数次打一行「验得过」；red 全部打「验得过」（每一处都是真红）。
# 打出这一格的全部输出，再按「一格里每一条 ✗ 都写着没跑成：内存包装 …」的字面算一遍（crash-verifier.md 第 3 步）。
# 用法：python3 l3-cell-modes.py <仓根> <草稿目录> <notrun|mixed|red> [--quiet]
import importlib.util, io, os, sys, contextlib
sys.dont_write_bytecode = True
root, scratch, mode = sys.argv[1], sys.argv[2], sys.argv[3]
quiet = "--quiet" in sys.argv
sys.path.insert(0, os.path.join(root, "research/scripts"))
spec = importlib.util.spec_from_file_location("verus_proof_check", os.path.join(root, "research/scripts/verus-proof-check.py"))
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
install = os.path.join(scratch, "fake-verus-install"); os.makedirs(install, exist_ok=True)
with open(os.path.join(install, "verus"), "w") as handle: handle.write("#!/bin/sh\nexit 0\n")
os.chmod(os.path.join(install, "verus"), 0o755)
with open(os.path.join(install, "version.json"), "w") as handle: handle.write('{"version": "x", "commit": "y"}')
counter = os.path.join(scratch, f"counter-{mode}"); open(counter, "w").write("0")
fake = os.path.join(scratch, f"fake-wrapper-{mode}.sh")
cond = {"notrun": "true", "mixed": "(( n % 2 == 1 ))", "red": "false"}[mode]
with open(fake, "w") as handle:
    handle.write(f'#!/usr/bin/env bash\nn=$(( $(cat {counter}) + 1 )); echo $n > {counter}\n'
                 f'if {cond}; then echo "✗ 桩：内存不够排不上：等了 0 秒" >&2; echo "→ 桩：空出来再重跑" >&2; exit 252; fi\n'
                 'echo \'{"verification-results":{"success":true,"errors":0,"verified":3,"encountered-vir-error":false}}\'\nexit 0\n')
module.MEMORY_CAP_RUNNER = fake
options = {"verus_root": install, "cargo_home": None, "launcher": None, "memory_cap": "4G", "timeout": 600, "threads": 4, "jobs": 1}
buffer = io.StringIO()
with contextlib.redirect_stdout(buffer):
    status = module.check_vacuity(root, options)
out = buffer.getvalue()
if not quiet:
    sys.stdout.write(out)
lines = out.split("\n")
reds = [l for l in lines if l.lstrip().startswith("✗")]
not_run = [l for l in reds if "没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 " in l and "按红记" not in l.split("没跑成")[0]]
others = [l for l in reds if l not in not_run]
print(f"## mode={mode} 退 {status}；✗ 行 {len(reds)} 条：写着「没跑成：内存包装…退 <码>」{len(not_run)} 条，不是这样写的 {len(others)} 条")
print(f"## 字面判：每一条 ✗ 都是没跑成 = {len(reds) > 0 and not others}；另有真红 = {bool(others)}")
for l in others[:2]:
    print("## 不是这样写的一条：", l.strip()[:120])
sys.exit(status)
