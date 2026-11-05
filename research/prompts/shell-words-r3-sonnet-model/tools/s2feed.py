"""s2feed.py <输出.json> [并行数] : 第二轮正推腿乙的 1943 个案例（案例表 cases_*.py 里的 CASES，只取数据）喂此刻的两份钩子，交回 [{id, rc_h, rc_d, exp_h, exp_d}]。
喂法自己写：钩子取草稿目录 s2work/repo/.claude/hooks 下的拷贝（build-skeleton.sh 搭的，与冻结副本逐字节相同）；cwd 按案例的 cwd 项选 repo / copy / stagedrepo / emptyrepo / 草稿目录；
{D} 换成草稿目录或工作根下的 scratch（案例带 wr 时）。"""
import sys, os, json, subprocess, importlib
from concurrent.futures import ThreadPoolExecutor
sys.dont_write_bytecode = True
S2 = os.environ.get("REPO_ROOT", "/home/fy5090/code/singlefs") + "/research/prompts/shell-words-r2-sonnet-s2-model/tools"
sys.path.insert(0, S2)
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__))); D = ROOT + "/s2work"
cwd_of = {"draft": D, "repo": D + "/repo", "copy": D + "/copy", "staged": D + "/stagedrepo", "empty": D + "/emptyrepo"}
mods = sorted(f[:-3] for f in os.listdir(S2) if f.startswith("cases_") and f.endswith(".py"))
cases = []
for m in mods:
    mod = importlib.import_module(m)
    for c in mod.CASES: cases.append(dict(c, id=m[len("cases_"):] + "." + c["id"]))
def feed(c, which):
    opt = c["opt"]; who = c["ident"]
    cwd = cwd_of.get(opt.get("cwd") or ("repo" if who == "main" else "copy"), opt.get("cwd"))
    scratch = D + "/wr/scratch" if opt.get("wr") else D
    ti = {"command": c["cmd"].replace("{D}", scratch)}
    if opt.get("bg"): ti["run_in_background"] = True
    if opt.get("timeout"): ti["timeout"] = opt["timeout"]
    inp = {"tool_name": "Bash", "cwd": cwd, "session_id": "r3-s2-recheck", "tool_input": ti}
    if who != "main": inp["agent_type"] = who; inp["agent_id"] = "agent-r3-s2-recheck"
    env = {k: v for k, v in os.environ.items() if k not in ("SINGLEFS_HEAVY_TESTS", "CLAUDE_PROJECT_DIR")}
    env["AGENT_HOOK_DETECTIONS"] = D + "/detections.jsonl"
    if opt.get("wr"): env["SINGLEFS_WORK_ROOT"] = D + "/wr"
    f = {"H": "heavy-test-guard.sh", "D": "bash-command-detector.sh"}[which]
    r = subprocess.run(["bash", f"{D}/repo/.claude/hooks/{f}"], input=json.dumps(inp), text=True, capture_output=True, env=env, timeout=300)
    return r.returncode
def one(c): return dict(id=c["id"], ident=c["ident"], cmd=c["cmd"], rc_h=feed(c, "H"), rc_d=feed(c, "D"), exp_h=c["h"], exp_d=c["d"])
with ThreadPoolExecutor(int(sys.argv[2]) if len(sys.argv) > 2 else 12) as ex: res = list(ex.map(one, cases))
json.dump(res, open(sys.argv[1], "w"), ensure_ascii=False)
print("cases", len(res))
