#!/usr/bin/env python3
"""探针：钩子引导「先读进并编过每份部分文件、全编成了才 exec」放过哪些部分文件状态。
用法：probe_hook_part_states.py --frozen <冻结副本> --work <输出目录>
每条钩子、每一臂在 <输出目录>/roots/<钩子>-<臂>/ 建一棵树：.claude/hooks 是冻结副本里那一份的拷贝、按臂改一份部分文件，
仓根与 .claude 下别的条目符号链接到冻结副本；喂一条输入（完好的钩子该拒的，完工清理闸喂 stop_hook_active 为真的 Stop），
记退出码、stderr 第一行、检出记录行数。只读冻结副本，写只写 <输出目录>。
臂：
  intact             不改
  main-part-empty    带 main 的最后一份截成 0 字节（编得过、什么都不定义）
  first-part-empty   第一份截成 0 字节
  main-part-missing  最后一份删掉（修补要拒的那一种，对照）
每臂一行：PROBE_HOOK_PART hook=<钩子> arm=<臂> exit=<退出码> detections=<检出行数> stderr_first=<stderr 第一行>
main-part-empty 那一臂另跑一次钩子的 --selftest：PROBE_HOOK_PART_SELFTEST hook=<钩子> arm=main-part-empty exit=<退出码> output_lines=<输出行数>
"""
import argparse, json, os, shutil, signal, subprocess, sys

HOOKS = {
    "write-guard": ("write-guard.sh", ("write-guard-decisions.py", "write-guard-selftest.py")),
    "project-code-guard": ("project-code-guard.sh", ("project-code-guard-decisions.py", "project-code-guard-selftest.py")),
    "runner-dispatch-guard": ("runner-dispatch-guard.sh", ("runner-dispatch-guard-prompt-checks.py", "runner-dispatch-guard-locks-and-verdicts.py",
                                                           "runner-dispatch-guard-selftest.py")),
    "completion-cleanup-guard": ("completion-cleanup-guard.sh", ("completion-cleanup-guard-decisions.py", "completion-cleanup-guard-selftest.py")),
    "bash-command-detector": ("bash-command-detector.sh", ("bash-command-detector-scope-and-index.py", "bash-command-detector-hooks-and-overwrites.py",
                                                           "bash-command-detector-literals-and-signals.py", "bash-command-detector-selftest-waits-and-scope.py",
                                                           "bash-command-detector-selftest-entries-and-main.py")),
}
ARMS = ("intact", "main-part-empty", "first-part-empty", "main-part-missing")
SESSION = "probe-hook-part-states"


def hook_input(hook, root, work_root):
    transcript = os.path.join(root, "no-such-transcript.jsonl")
    if hook == "write-guard":
        return {"hook_event_name": "PreToolUse", "tool_name": "Write", "session_id": SESSION, "cwd": root, "transcript_path": transcript,
                "tool_input": {"file_path": os.path.join(root, "research", "probe-hook-part-states.md"), "content": "臂 A2 的说明\n"}}
    if hook == "project-code-guard":
        return {"hook_event_name": "PreToolUse", "tool_name": "Bash", "session_id": SESSION, "cwd": root, "transcript_path": transcript,
                "tool_input": {"command": "cargo build -p singlefs-core", "description": "编一个包"}}
    if hook == "runner-dispatch-guard":
        return {"hook_event_name": "PreToolUse", "tool_name": "Agent", "session_id": SESSION, "cwd": root, "transcript_path": transcript,
                "tool_input": {"subagent_type": "general-purpose", "description": "查文件", "prompt": "查一个文件。"}}
    if hook == "completion-cleanup-guard":
        return {"hook_event_name": "Stop", "session_id": SESSION, "cwd": root, "transcript_path": transcript, "stop_hook_active": True}
    return {"hook_event_name": "PreToolUse", "tool_name": "Bash", "session_id": SESSION, "cwd": root, "transcript_path": transcript,
            "tool_input": {"command": "rm -r " + work_root, "description": "删目录"}}


def build_root(frozen, root):
    os.makedirs(os.path.join(root, ".claude"))
    for name in sorted(os.listdir(frozen)):
        if name != ".claude":
            os.symlink(os.path.join(frozen, name), os.path.join(root, name))
    for name in sorted(os.listdir(os.path.join(frozen, ".claude"))):
        if name != "hooks":
            os.symlink(os.path.join(frozen, ".claude", name), os.path.join(root, ".claude", name))
    shutil.copytree(os.path.join(frozen, ".claude", "hooks"), os.path.join(root, ".claude", "hooks"),
                    symlinks=True, ignore=shutil.ignore_patterns("__pycache__"))


def apply_arm(hooks_directory, parts, arm):
    if arm == "main-part-empty":
        open(os.path.join(hooks_directory, parts[-1]), "w").close()
    elif arm == "first-part-empty":
        open(os.path.join(hooks_directory, parts[0]), "w").close()
    elif arm == "main-part-missing":
        os.remove(os.path.join(hooks_directory, parts[-1]))


def clean_environment(root, detections):
    environment = {name: value for name, value in os.environ.items()
                   if not name.endswith("_BREAK") and "SELFTEST_MARK" not in name and not name.endswith("_DISABLE_SCOPE")}
    environment["CLAUDE_PROJECT_DIR"] = root
    environment["AGENT_HOOK_DETECTIONS"] = detections
    return environment


def run_sample(command, stdin_text, environment, cwd, timeout):
    """一条样本放进它自己的会话（进程组）里起；超时只对这一组发 SIGKILL。交 (退出码或 timeout, stdout, stderr)。"""
    process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
                               env=environment, cwd=cwd, start_new_session=True)
    try:
        stdout, stderr = process.communicate(stdin_text, timeout=timeout)
        return str(process.returncode), stdout, stderr
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        stdout, stderr = process.communicate()
        return "timeout", stdout, stderr


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    parser.add_argument("--timeout", type=int, default=300)
    arguments = parser.parse_args()
    frozen = os.path.realpath(arguments.frozen)
    work = os.path.realpath(arguments.work)
    work_root = os.path.dirname(work)
    output = os.path.join(work, "hook-part-states")
    os.makedirs(output)
    for hook, (entry, parts) in HOOKS.items():
        entry_text = open(os.path.join(frozen, ".claude", "hooks", entry), encoding="utf-8").read()
        unlisted = [part for part in parts if f'"{part}"' not in entry_text]
        if unlisted:
            print(f"PROBE_HOOK_PART_SETUP hook={hook} unlisted_parts={unlisted}（入口的部分文件名单与探针对不上，这一钩子的读数作废）")
        for arm in ARMS:
            root = os.path.join(work, "roots", f"{hook}-{arm}")
            build_root(frozen, root)
            apply_arm(os.path.join(root, ".claude", "hooks"), parts, arm)
            detections = os.path.join(output, f"{hook}-{arm}.detections.jsonl")
            open(detections, "w").close()
            payload = json.dumps(hook_input(hook, root, work_root), ensure_ascii=False)
            code, stdout, stderr = run_sample(["bash", os.path.join(root, ".claude", "hooks", entry)], payload,
                                              clean_environment(root, detections), root, arguments.timeout)
            for suffix, text in (("stdout", stdout), ("stderr", stderr)):
                with open(os.path.join(output, f"{hook}-{arm}.{suffix}"), "w", encoding="utf-8") as handle:
                    handle.write(text)
            detection_lines = sum(1 for line in open(detections, encoding="utf-8") if line.strip())
            first = next((line.strip() for line in stderr.splitlines() if line.strip()), "")[:160]
            print(f"PROBE_HOOK_PART hook={hook} arm={arm} exit={code} detections={detection_lines} stderr_first={first}", flush=True)
            if arm == "main-part-empty":   # 带 main 的那一份空着时，钩子自己的 --selftest 报什么（退出码、输出几行）
                code, stdout, stderr = run_sample(["bash", os.path.join(root, ".claude", "hooks", entry), "--selftest"], "",
                                                  clean_environment(root, detections), root, arguments.timeout)
                lines = len([line for line in (stdout + stderr).splitlines() if line.strip()])
                print(f"PROBE_HOOK_PART_SELFTEST hook={hook} arm={arm} exit={code} output_lines={lines}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
