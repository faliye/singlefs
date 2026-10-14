# D1-1 / A1 / A4：capped.sh 设的变量、SINGLEFS_THREAD_CAP 与显式数的关系、闸有没有要求子 agent 经 capped.sh 起。
# 用法：python3 -I r2_capped.py <改前树> <改后树> <工作目录>
import json, os, subprocess, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as c
trees, W = c.parse_trees(sys.argv)

def capped(tree, args, env=None):
    e = dict(os.environ); e.pop("SINGLEFS_THREAD_CAP", None)
    if env: e.update(env)
    r = subprocess.run(["bash", f"{tree}/scripts/hosts/capped.sh"] + args, capture_output=True, text=True, env=e, cwd=tree)
    return r.returncode, r.stdout.strip().replace("\n", ",")[:4000]

def show(cid, args, env=None, grep=None):
    def fn(tree):
        rc, out = capped(tree, args, env)
        if grep: out = ",".join(l for l in out.split(",") if l.split("=")[0] in grep)
        return rc, out or "-"
    c.both("R2", cid, fn, trees)

show("S1-E161-printenv-with-capped-8", ["8", "printenv", "E161_THREADS"])
show("S5-default-no-cap-env", ["--print", "default"], grep={"RUST_TEST_THREADS", "E161_THREADS", "SINGLEFS_RANDOM_HISTORY_THREADS"})
show("S5-default-cap-env-5", ["--print", "default"], env={"SINGLEFS_THREAD_CAP": "5"}, grep={"RUST_TEST_THREADS", "E161_THREADS"})
show("A1-explicit-31-cap-env-8", ["--print", "31"], env={"SINGLEFS_THREAD_CAP": "8"}, grep={"RUST_TEST_THREADS", "E161_THREADS"})
show("A1-explicit-8-cap-env-unset", ["--print", "8"], grep={"RUST_TEST_THREADS", "E161_THREADS"})
show("A1-nested-default-under-N", ["8", "bash", "scripts/hosts/capped.sh", "--print", "default"], grep={"RUST_TEST_THREADS", "E161_THREADS"})
show("X-cpu-judge-var-in-capped-print", ["--print", "8"], grep={"SINGLEFS_CRASH_JUDGE_CPU_THREADS", "SINGLEFS_CRASH_AMPLIFICATION_THREADS"})

def guard(tree, cmd):
    payload = {"tool_name": "Bash", "cwd": tree, "tool_input": {"command": cmd}, "session_id": "s", "agent_type": "implementation-writer"}
    e = dict(os.environ, CLAUDE_PROJECT_DIR=tree, AGENT_HOOK_DETECTIONS=f"{W}/detections.jsonl"); e.pop("SINGLEFS_HEAVY_TESTS", None)
    r = subprocess.run(["bash", f"{tree}/.claude/hooks/heavy-test-guard.sh"], input=json.dumps(payload), capture_output=True, text=True, env=e)
    return r.returncode, "-"
c.both("R2", "S4-sub-agent-harness-test-without-capped", lambda t: guard(t, "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --lib"), trees)
c.both("R2", "S4-sub-agent-harness-test-with-capped-8", lambda t: guard(t, "bash scripts/hosts/capped.sh 8 bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --lib"), trees)
c.both("R2", "S4-sub-agent-harness-test-with-capped-default", lambda t: guard(t, "bash scripts/hosts/capped.sh default bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --lib"), trees)
