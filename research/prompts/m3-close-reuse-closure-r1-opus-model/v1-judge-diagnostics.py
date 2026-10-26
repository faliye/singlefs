"""照 verus-proof-check.py 的 run_verus 拼 outcome（diagnostics = stderr + "\n" + stdout），调它的 must_red_row_problems 判一行必红世界，打判定与一句判定。
只 import 那份脚本（不走 __main__、不起 Verus），登记的落点取 src/kernel.rs 里 12 行所在的那个 fn（这里不读真文件：root 给 None，落点只比「无位置」）。"""
import importlib.util, json, os, sys
root, stdout_path, stderr_path, status = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4])
spec = importlib.util.spec_from_file_location("verus_proof_check", os.path.join(root, "scripts/gate/verus-proof-check.py"))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
stdout, stderr = open(stdout_path, encoding="utf-8").read(), open(stderr_path, encoding="utf-8").read()
outcome = {"exit": status, "timed_out": False, "wrapper": module.WRAPPER_OWN_EXITS.get(status), "json": None, "diagnostics": stderr + "\n" + stdout, "stderr": stderr}
outcome["json"] = json.loads(stdout) if stdout.strip().startswith("{") else None
row = {"number": 1, "name": "attack-row", "proof_errors": 1, "landing_text": None}
class Registry:
    rlimit = 10
    crate_directory = "crates/attack"
problems, summary = module.must_red_row_problems(row, outcome, {"timeout": 60, "memory_cap": "200M"}, Registry(), root=None)
host_lines = [line for line in stderr.splitlines() if "别的机器" in line or "此刻放得下" in line]
print("host_lookup_lines_in_diagnostics=" + str(len(host_lines)))
for line in host_lines:
    print("  host_lookup_line=" + line[:160])
print("problems=" + json.dumps([problem[0][:160] for problem in problems], ensure_ascii=False))
print("summary=" + summary)
print("proof_error_messages=" + json.dumps(module.proof_error_messages(outcome["diagnostics"]), ensure_ascii=False))
