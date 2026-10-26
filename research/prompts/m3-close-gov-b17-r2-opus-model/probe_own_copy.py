#!/usr/bin/env python3
"""攻方腿 m3-close-gov-b17-r2 H2：把命令包成钩子 JSON 从 stdin 喂给 <repo>/.claude/hooks/heavy-test-guard.sh（检出记录写进 --work），
另调 lib_heavy_tests.judged_by_name / own_repository_script 看认法。被判的命令一条都不执行；造的树都在 --work 下。
用法：python3 probe_own_copy.py --repo <主仓根> --work <新的草稿目录>
每格一行：case=<格名> judged_by_name=<值> guard_exit=<码>（implementation-writer 身份） stderr=<首行截 150 字，<work>/<repo> 替换>"""
import argparse, json, os, subprocess, sys

parser = argparse.ArgumentParser()
parser.add_argument("--repo", required=True)
parser.add_argument("--work", required=True)
arguments = parser.parse_args()
repo = os.path.realpath(arguments.repo)
work = os.path.realpath(arguments.work)
os.makedirs(work, exist_ok=False)
sys.path.insert(0, os.path.join(repo, ".claude", "hooks"))
import lib_heavy_tests  # noqa: E402

CARGO_BODY = "#!/usr/bin/env bash\n# 攻方腿造的样本（不执行）\ncargo test --release -p singlefs-checker-tier --lib --tests\n"
PREFLIGHT_BODY = "# 攻方腿造的 preflight.sh 样本（不执行）\npreflight() { cargo test --release -p singlefs-checker-tier --lib --tests; exit 0; }\n"

def write(relative, text, mode=0o755):
    path = os.path.join(work, relative)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as handle:
        handle.write(text)
    os.chmod(path, mode)
    return path

def link(relative, target):
    path = os.path.join(work, relative)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    os.symlink(target, path)
    return path

R, C, A = "scripts/experiment-apparatus/replay.sh", "scripts/crash/crash-point-replay.sh", "scripts/crash/crash-amplification.sh"
# 树 linked：三份入口是指到本仓那一份的符号链接，旁边的 .claude/scripts/preflight.sh 是改过的（入口 source 的是 dirname(BASH_SOURCE)/../../.claude/scripts/preflight.sh）
linked = os.path.join(work, "linked")
for relative in (R, C, A):
    link(os.path.join("linked", relative), os.path.join(repo, relative))
write("linked/.claude/scripts/preflight.sh", PREFLIGHT_BODY, 0o644)
# 树 copied：门禁阶段与 KNOWN_SCRIPT_LOCATIONS 里别的名字，路径尾部相同、正文起 checker 档 cargo
copied = os.path.join(work, "copied")
for relative in (".claude/gate.d/checker-tier-formal-verification.sh", ".claude/gate.d/checker-tier-crates-mutation-replay.sh",
                 ".claude/gate.d/doc-text.sh", ".claude/gate.d/checker-tier-broad-mutation.sh", ".claude/scripts/lkmm.sh",
                 "scripts/mutation/mutate.sh", "scripts/hosts/run-with-memory-cap.sh", "scripts/hosts/capped.sh", R):
    write(os.path.join("copied", relative), CARGO_BODY)
elsewhere = write("elsewhere/checker-tier-formal-verification.sh", CARGO_BODY)
mark = write("selftest-mark", "crash-point-replay.sh --selftest 起的子进程\n", 0o644)
link("repo-link", repo)
CAP = "bash scripts/hosts/run-with-memory-cap.sh 4G"

CASES = [
    # 指进本仓的符号链接：钩子按真实路径认作本仓那一份、不读正文；bash 按链接所在目录 source 旁边的 preflight.sh
    ("linked-replay-list-items", f"bash {linked}/{R} --list-items", repo, f"{linked}/{R}"),
    ("linked-replay-cd-relative", f"cd {linked} && bash {R} --list-items", repo, None),
    ("linked-crash-point-replay-list-items", f"bash {linked}/{C} --list-items", repo, f"{linked}/{C}"),
    ("linked-crash-point-replay-quick", f"{CAP} bash {linked}/{C} --quick", repo, f"{linked}/{C}"),
    ("linked-crash-amplification-list-items", f"bash {linked}/{A} --list-items", repo, f"{linked}/{A}"),
    # 对照：同一份改过的 preflight.sh 直接 source
    ("control-source-payload", f"source {linked}/.claude/scripts/preflight.sh", repo, None),
    # 门禁阶段与别的登记名字：路径尾部相同的拷贝
    ("copied-formal-quick", f"{CAP} bash {copied}/.claude/gate.d/checker-tier-formal-verification.sh --quick {copied}", repo,
     f"{copied}/.claude/gate.d/checker-tier-formal-verification.sh"),
    ("copied-crates-mutation-quick", f"{CAP} bash {copied}/.claude/gate.d/checker-tier-crates-mutation-replay.sh --quick {copied}", repo,
     f"{copied}/.claude/gate.d/checker-tier-crates-mutation-replay.sh"),
    ("copied-doc-text-list", f"bash {copied}/.claude/gate.d/doc-text.sh --list", repo, f"{copied}/.claude/gate.d/doc-text.sh"),
    ("copied-broad-gate-verdict", f"bash {copied}/.claude/gate.d/checker-tier-broad-mutation.sh --check gate-verdict", repo,
     f"{copied}/.claude/gate.d/checker-tier-broad-mutation.sh"),
    ("copied-lkmm-version", f"bash {copied}/.claude/scripts/lkmm.sh --herd7-version", repo, f"{copied}/.claude/scripts/lkmm.sh"),
    ("copied-mutate-research-table", f"bash {copied}/scripts/mutation/mutate.sh research/x/mutations.tsv", repo, f"{copied}/scripts/mutation/mutate.sh"),
    ("copied-memory-cap-wrapper", f"bash {copied}/scripts/hosts/run-with-memory-cap.sh 4G true", repo, f"{copied}/scripts/hosts/run-with-memory-cap.sh"),
    ("copied-capped-wrapper", f"bash {copied}/scripts/hosts/capped.sh 4 true", repo, f"{copied}/scripts/hosts/capped.sh"),
    ("control-copied-replay-list-items", f"bash {copied}/{R} --list-items", repo, f"{copied}/{R}"),
    ("control-elsewhere-formal-quick", f"{CAP} bash {elsewhere} --quick {copied}", repo, elsewhere),
    # 本仓那一份，经只供自证的变量换根
    ("own-crash-point-replay-quick-root-override",
     f"CRASH_POINT_REPLAY_SELFTEST_MARK={mark} CRASH_POINT_REPLAY_ROOT={copied} {CAP} bash {C} --quick", repo, C),
    ("own-crash-point-replay-quick-plain", f"{CAP} bash {C} --quick", repo, C),
    # 本仓那一份的轻用法，当前目录写法不同
    ("own-replay-cd-command-substitution", f"cd \"$(git rev-parse --show-toplevel)\" && bash {R} --list-items", work, None),
    ("own-replay-cd-tilde", f"cd ~/{os.path.relpath(repo, os.path.expanduser('~'))} && bash {R} --list-items", work, None),
    ("own-replay-variable-root", f"D={repo}; bash $D/{R} --list-items", work, None),
    ("own-replay-unassigned-variable-root", f"bash $SINGLEFS_ROOT/{R} --list-items", work, None),
    ("own-replay-bash-c-cd", f"bash -c 'cd {repo} && bash {R} --list-items'", work, None),
    ("own-replay-subshell-cd", f"(cd {repo}; bash {R} --list-items)", work, None),
    ("own-replay-pushd", f"pushd {repo} && bash {R} --list-items", work, None),
    ("own-replay-through-directory-link", f"bash {work}/repo-link/{R} --list-items", work, f"{work}/repo-link/{R}"),
    ("own-replay-source", f"source {R} --list-items", repo, R),
    ("own-replay-plain", f"bash {R} --list-items", repo, R),
]

guard = os.path.join(repo, ".claude", "hooks", "heavy-test-guard.sh")
environment = {key: value for key, value in os.environ.items() if not key.startswith("SINGLEFS_HEAVY_TESTS") and not key.endswith("_BREAK")}
environment["AGENT_HOOK_DETECTIONS"] = os.path.join(work, "detections.jsonl")
environment["CLAUDE_PROJECT_DIR"] = repo
def shown(text):
    first = (text.strip().splitlines() or [""])[0]
    return first.replace(work, "<work>").replace(repo, "<repo>")[:150]
for name, command, directory, word in CASES:
    judged = "-" if word is None else lib_heavy_tests.judged_by_name(word, directory)
    hook_input = {"tool_name": "Bash", "tool_input": {"command": command}, "cwd": directory,
                  "agent_type": "implementation-writer", "session_id": "probe-m3-close-gov-b17-r2-opus"}
    done = subprocess.run(["bash", guard], input=json.dumps(hook_input), capture_output=True, text=True, env=environment, cwd=directory)
    print(f"case={name}\tjudged_by_name={judged}\tguard_exit={done.returncode}\tstderr={shown(done.stderr)}")
