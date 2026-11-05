#!/usr/bin/env python3
"""H12 / D1 / D2 现喂：feed-dispatch.py <钩子树> <假准入脚本目录> <草稿目录>
把派发闸拷贝的准入脚本指到各种假的（样本拷自 d137-r2fix-dispatch/samples/），经 stdin 喂一份设计员写重跑登记的提示，打退出码、拒不拒、记不记检出。
每次起钩子放进自己的进程组，超时对整组发 SIGKILL。不派任何 agent。"""
import json, os, signal, subprocess, sys

tree, samples, scratch = sys.argv[1:4]
hook = os.path.join(tree, ".claude", "hooks", "runner-dispatch-guard.sh")
prompt = f"# 派发：experiment-designer\n\n重型测试：不跑\n草稿目录：{scratch}\n线程上限：31\n\n写重跑登记 research/prompts/e142-r19-prereg.md。\n"
data = json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "experiment-designer", "prompt": prompt},
                   "session_id": "s-d137-gov-r3-defense", "transcript_path": "/nonexistent"})
cases = [("nonexistent.py", {}), ("usage-exit-2.py", {}), ("silent-exit-2.py", {}), ("unregistered.py", {}),
         ("admission-break.py", {}), ("crashed.py", {}), ("key-refused.py", {}), ("force-rerun-reason.py", {}),
         ("force-rerun-reason.py", {"SINGLEFS_EXPERIMENT_RERUN_REASON": "hook-environment-leftover"})]
base = {k: v for k, v in os.environ.items() if not k.startswith(("SINGLEFS_", "CLAUDE_PROJECT_DIR"))}
base["AGENT_HOOK_DETECTIONS"] = os.path.join(scratch, "dispatch-detections.jsonl")
for script, extra in cases:
    env = dict(base, RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT=os.path.join(samples, script), **extra)
    process = subprocess.Popen(["bash", hook], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               text=True, env=env, cwd=scratch, start_new_session=True)
    try:
        out, err = process.communicate(data, timeout=150)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL); process.communicate(); print(f"name=dispatch script={script} timeout"); continue
    refused = "不写它的重跑登记" in err
    noted = any(line.startswith("!") and ("admission" in line or "准入" in line) for line in err.splitlines())
    first = next((line.strip() for line in err.splitlines() if "准入" in line or "admission" in line), "")
    print(f"name=dispatch script={script} env_reason={'yes' if extra else 'no'} rc={process.returncode} refused={refused} noted={noted}")
    if first:
        print(f"  {first[:300]}")
