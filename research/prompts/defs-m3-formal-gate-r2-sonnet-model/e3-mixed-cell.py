# 探针（只读、不起 Verus、不起真包装）：空真格（check_vacuity）用真仓的登记与核，Verus 与包装都换成桩：奇数次调用退 252，偶数次打一行「验得过」的 JSON。
# 看同一格里同时出现「没跑成」与真的红（照样验得过）时，输出长什么样。用法：python3 e3-mixed-cell.py <仓根> <草稿目录>
import importlib.util, io, os, sys, contextlib
sys.dont_write_bytecode = True
root, scratch = sys.argv[1], sys.argv[2]
sys.path.insert(0, os.path.join(root, "research/scripts"))
spec = importlib.util.spec_from_file_location("verus_proof_check", os.path.join(root, "research/scripts/verus-proof-check.py"))
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
install = os.path.join(scratch, "fake-verus-install"); os.makedirs(install, exist_ok=True)
with open(os.path.join(install, "verus"), "w") as handle: handle.write("#!/bin/sh\nexit 0\n")
os.chmod(os.path.join(install, "verus"), 0o755)
with open(os.path.join(install, "version.json"), "w") as handle: handle.write('{"version": "x", "commit": "y"}')
counter = os.path.join(scratch, "mixed-counter"); open(counter, "w").write("0")
fake = os.path.join(scratch, "fake-wrapper-mixed.sh")
with open(fake, "w") as handle:
    handle.write(f'#!/usr/bin/env bash\nn=$(( $(cat {counter}) + 1 )); echo $n > {counter}\n'
                 'if (( n % 2 == 1 )); then exit 252; fi\n'
                 'echo \'{"verification-results":{"success":true,"errors":0,"verified":3,"encountered-vir-error":false}}\'\nexit 0\n')
module.MEMORY_CAP_RUNNER = fake
options = {"verus_root": install, "cargo_home": None, "launcher": None, "memory_cap": "4G", "timeout": 600, "threads": 4, "jobs": 1}
buffer = io.StringIO()
with contextlib.redirect_stdout(buffer):
    status = module.check_vacuity(root, options)
lines = buffer.getvalue().split("\n")
red_lines = [line for line in lines if line.startswith("  ✗")]
print(f"check_vacuity 退 {status}；✗ 行 {len(red_lines)} 条：没跑成 {sum('没跑成' in l for l in red_lines)} 条，照样验得过（空真，真红）{sum('照样验得过' in l for l in red_lines)} 条")
for kind in ("没跑成", "照样验得过"):
    for line in red_lines:
        if kind in line:
            print("  样一条：", line[:150]); break
