#!/usr/bin/env python3
"""攻方腿 m3-close-gov-b17-r2 H2 误拒一侧：真正文的拷贝（实现员副本、别的工作树这一类，路径尾部相同、内容与本仓那一份逐字节相同）里起轻用法，喂钩子看拒不拒。
不执行被判的命令。用法：python3 probe_copy_light.py --repo <主仓根> --copy <只读用的仓副本根> --work <新的草稿目录>"""
import argparse, json, os, subprocess, sys, hashlib
parser = argparse.ArgumentParser()
parser.add_argument("--repo", required=True); parser.add_argument("--copy", required=True); parser.add_argument("--work", required=True)
arguments = parser.parse_args()
repo, copy, work = (os.path.realpath(path) for path in (arguments.repo, arguments.copy, arguments.work))
os.makedirs(work, exist_ok=False)
sys.path.insert(0, os.path.join(repo, ".claude", "hooks"))
import lib_heavy_tests  # noqa: E402
R, C, A = "scripts/experiment-apparatus/replay.sh", "scripts/crash/crash-point-replay.sh", "scripts/crash/crash-amplification.sh"
def digest(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()[:16]
for relative in (R, C, A):
    print(f"name=same_bytes file={relative} repo={digest(os.path.join(repo, relative))} copy={digest(os.path.join(copy, relative))}")
SCOPE = "SINGLEFS_RUN_SCOPE='种类=quick; 跑什么=崩溃点重放轻型; 范围=轻型全部项; 为什么=攻方腿喂钩子的样本'"
CASES = [
    ("copy-replay-list-items", f"bash {R} --list-items", copy, R),
    ("copy-crash-point-replay-list-items", f"bash {C} --list-items", copy, C),
    ("copy-crash-point-replay-quick", f"{SCOPE} bash scripts/hosts/run-with-memory-cap.sh 8G bash {C} --quick", copy, C),
    ("copy-crash-amplification-list-items", f"bash {A} --list-items", copy, A),
    ("repo-crash-point-replay-quick", f"{SCOPE} bash scripts/hosts/run-with-memory-cap.sh 8G bash {C} --quick", repo, C),
]
guard = os.path.join(repo, ".claude", "hooks", "heavy-test-guard.sh")
environment = {key: value for key, value in os.environ.items() if not key.startswith("SINGLEFS_HEAVY_TESTS") and not key.endswith("_BREAK")}
environment["AGENT_HOOK_DETECTIONS"] = os.path.join(work, "detections.jsonl")
environment["CLAUDE_PROJECT_DIR"] = repo
def shown(text):
    first = (text.strip().splitlines() or [""])[0]
    return first.replace(work, "<work>").replace(copy, "<copy>").replace(repo, "<repo>")[:170]
for role in ("main-agent", "implementation-writer"):
    for name, command, directory, word in CASES:
        hook_input = {"tool_name": "Bash", "tool_input": {"command": command}, "cwd": directory, "session_id": "probe-m3-close-gov-b17-r2-opus"}
        if role != "main-agent":
            hook_input["agent_type"] = role
        done = subprocess.run(["bash", guard], input=json.dumps(hook_input), capture_output=True, text=True, env=environment, cwd=directory)
        print(f"role={role}\tcase={name}\tjudged_by_name={lib_heavy_tests.judged_by_name(word, directory)}\tguard_exit={done.returncode}\tstderr={shown(done.stderr)}")
