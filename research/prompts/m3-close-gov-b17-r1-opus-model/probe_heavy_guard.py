#!/usr/bin/env python3
"""攻方腿 m3-close-gov-b17-r1 G4：拿重型闸自己的判法喂命令，看 replay.sh --list-items 的放行放过什么。
不起任何被判的命令：只调 .claude/hooks/lib_heavy_tests.py 的 heavy_tests_in_command_text / judged_by_name，
再把同一条命令包成钩子的 JSON 从 stdin 喂给 .claude/hooks/heavy-test-guard.sh（检出记录写进 --work 下）。
用法：python3 probe_heavy_guard.py --repo <仓根> --work <新的草稿目录>
每格一行：格名<TAB>lib 判出的类<TAB>钩子退出码（实现员身份）<TAB>钩子 stderr 第一行（截 160 字）。"""
import argparse, json, os, subprocess, sys

parser = argparse.ArgumentParser()
parser.add_argument("--repo", required=True)
parser.add_argument("--work", required=True)
arguments = parser.parse_args()
repo = os.path.realpath(arguments.repo)
work = os.path.realpath(arguments.work)
os.makedirs(work, exist_ok=True)
sys.path.insert(0, os.path.join(repo, ".claude", "hooks"))
import lib_heavy_tests  # noqa: E402

# 一份「改过的拷贝」：--list-items 那一支起 cargo（形状照真 replay.sh 里 GPU 试编那一行与 checker 档测试）；只写不执行
FAKE_BODY = """#!/usr/bin/env bash
# 副本里改过的实验复跑入口（攻方腿造的样本，不执行）
if [[ "${1:-}" == "--list-items" ]]; then
  cargo build --release -p singlefs-checker-tier --features verdict-store,gpu --lib
  cargo test --release -p singlefs-checker-tier --lib --tests
  exit 0
fi
"""
def write(relative, text):
    path = os.path.join(work, relative)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as handle:
        handle.write(text)
    return path

copy_replay = write("copy/scripts/experiment-apparatus/replay.sh", FAKE_BODY)
write("copy/scripts/crash/crash-point-replay.sh", FAKE_BODY)
elsewhere_replay = write("elsewhere/replay.sh", FAKE_BODY)
bash_env = write("bash-env.sh", "cargo test --release -p singlefs-checker-tier --lib --tests\n")
copy_root = os.path.join(work, "copy")
R = "scripts/experiment-apparatus/replay.sh"

CASES = [
    ("main-list-items", f"bash {R} --list-items", repo),
    ("main-list-items-then-experiment", f"bash {R} --list-items E142", repo),
    ("main-list-items-force-after", f"bash {R} --list-items --force", repo),
    ("main-no-arguments", f"bash {R}", repo),
    ("wrapped-memory-cap", f"bash scripts/hosts/run-with-memory-cap.sh 4G bash {R} --list-items", repo),
    ("wrapped-env-nice-capped", f"env FOO=1 nice -n 19 bash scripts/hosts/capped.sh 4 bash {R} --list-items", repo),
    ("joined-and-experiment", f"bash {R} --list-items && bash {R} E142", repo),
    ("joined-semicolon-cargo", f"bash {R} --list-items; cargo test -p singlefs-checker-tier", repo),
    ("copy-absolute-path", f"bash {copy_replay} --list-items", repo),
    ("copy-cd-relative", f"cd {copy_root} && bash {R} --list-items", repo),
    ("copy-cwd-relative", f"bash {R} --list-items", copy_root),
    ("copy-no-arguments-control", f"bash {copy_replay}", repo),
    ("elsewhere-same-body-control", f"bash {elsewhere_replay} --list-items", repo),
    ("copy-crash-point-replay-list-items", f"bash {copy_root}/scripts/crash/crash-point-replay.sh --list-items", repo),
    ("bash-env-replay-list-items", f"BASH_ENV={bash_env} bash {R} --list-items", repo),
    ("bash-env-env-replay-list-items", f"env BASH_ENV={bash_env} bash {R} --list-items", repo),
    ("bash-env-export-replay-list-items", f"export BASH_ENV={bash_env}; bash {R} --list-items", repo),
    ("bash-env-bash-c-true-control", f"BASH_ENV={bash_env} bash -c true", repo),
    ("prefix-assignment-variable", f"A=E142; A=--list-items bash {R} $A", repo),
    ("variable-known-list-items", f"A=--list-items; bash {R} $A", repo),
    ("xargs-appends-experiment", f"echo E142 | xargs bash {R} --list-items", repo),
    ("xargs-bare-entry", f"echo E142 | xargs bash {R}", repo),
    ("source-list-items", f"source {R} --list-items", repo),
    ("sh-list-items", f"sh {R} --list-items", repo),
    ("bash-x-list-items", f"bash -x {R} --list-items", repo),
    ("glob-argument", f"bash {R} --list-item?", repo),
]

guard = os.path.join(repo, ".claude", "hooks", "heavy-test-guard.sh")
environment = {key: value for key, value in os.environ.items() if not key.startswith("SINGLEFS_HEAVY_TESTS")}
environment["AGENT_HOOK_DETECTIONS"] = os.path.join(work, "detections.jsonl")
environment["CLAUDE_PROJECT_DIR"] = repo
print(f"name=judged_by_name copy={lib_heavy_tests.judged_by_name(copy_replay, repo)} elsewhere={lib_heavy_tests.judged_by_name(elsewhere_replay, repo)} main={lib_heavy_tests.judged_by_name(R, repo)}")
for name, command, directory in CASES:
    found = lib_heavy_tests.heavy_tests_in_command_text(command, directory)
    kinds = ",".join(sorted({test.kind for test in found})) or "none"
    hook_input = {"tool_name": "Bash", "tool_input": {"command": command}, "cwd": directory,
                  "agent_type": "implementation-writer", "session_id": "probe-m3-close-gov-b17-r1-opus"}
    done = subprocess.run(["bash", guard], input=json.dumps(hook_input), capture_output=True, text=True, env=environment, timeout=120)
    first = next((line for line in done.stderr.splitlines() if line.strip()), "").replace(work, "<work>").replace(repo, "<repo>")[:160]
    print(f"case={name}\tlib={kinds}\tguard_exit={done.returncode}\tstderr={first}")
