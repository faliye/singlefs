# 探针（只读、不起 Verus、不起真包装）：MEMORY_CAP_RUNNER 换成按 FAKE_EXIT 退出的桩，走 verus-proof-check.py 的 run_verus -> outcome_problem -> problem_howto，
# 打每个退出码在「没跑成」「超时红」「Verus 没出判定」三类里落哪一类、给什么出路。用法：python3 e3-verus-exit-codes.py <仓根> <草稿目录>
import importlib.util, os, sys, tempfile
sys.dont_write_bytecode = True
root, scratch = sys.argv[1], sys.argv[2]
sys.path.insert(0, os.path.join(root, "research/scripts"))
spec = importlib.util.spec_from_file_location("verus_proof_check", os.path.join(root, "research/scripts/verus-proof-check.py"))
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
fake = os.path.join(scratch, "fake-wrapper.sh")
with open(fake, "w") as handle:
    handle.write('#!/usr/bin/env bash\nexit "${FAKE_EXIT:-0}"\n')
module.MEMORY_CAP_RUNNER = fake
options = {"memory_cap": "4G", "timeout": 600, "threads": 4, "launcher": None}
howto_own = "看上面 Verus 的报错；超时先看是哪条查询慢（--profile），确要更长的时限设 GATE_VERUS_TIMEOUT_SECONDS 并写明为什么"
with tempfile.TemporaryDirectory(dir=scratch) as crate:
    for code in (0, 1, 137, 250, 251, 252, 253, 254):
        os.environ["FAKE_EXIT"] = str(code)
        outcome = module.run_verus("/nonexistent/verus", crate, ["--x"], options, "probe")
        problem = module.outcome_problem(outcome, options)
        howto = module.problem_howto(outcome, howto_own) if problem else None
        kind = "没跑成" if problem and problem.startswith("没跑成") else ("超时红" if problem and "按红记" in problem else ("没出判定" if problem else "无问题"))
        route = "WRAPPER_HOWTO" if howto == module.WRAPPER_HOWTO else ("格自己的出路" if howto == howto_own else "无")
        print(f"退 {code:>3} | wrapper={outcome['wrapper']!s:<12} timed_out={outcome['timed_out']!s:<5} | 类 {kind:<6} | 出路 {route} | {(problem or '')[:78]}")
