# 变体与原 hook 在附录三 L1–L14 上判定是否逐条相同（只为说明变体改了哪几格；timeout）
import json, os, subprocess, sys, tempfile
L = {
 "L1": 'nohup nice -n 19 bash .claude/scripts/gate.sh --staged > gate-run.log 2>&1 & echo "started pid $!"; disown',
 "L2": 'nohup nice -n 19 bash research/scripts/cache-keepalive.sh > keepalive.log 2>&1 &',
 "L3": 'bash research/scripts/cache-keepalive.sh',
 "L4": 'until ! ps -p 2296232 > /dev/null 2>&1; do sleep 5; done; echo "gate.sh finished"',
 "L5": 'cargo build 2>&1 | tail -3 && echo ok',
 "L6": 'bash a.sh > a.log 2>&1 & bash b.sh > b.log 2>&1 & wait',
 "L7": 'wait; bash late.sh &',
 "L8": 'make |& tee out.log',
 "L9": 'curl -s "https://example.invalid/?a=1&b=2" > page.html',
 "L10": 'setsid bash long.sh > long.log 2>&1 < /dev/null',
 "L11": '(sleep 300; echo done) &> timer.log',
 "L12": 'bash x.sh & pid=$!; wait "$pid"',
 "L13": 'echo "remember: nohup is not needed here"',
 "L14": "bash -c 'sleep 10 &'",
}
def third(hook, text):
    d = tempfile.mkdtemp(dir=os.environ["BGN_DRAFT"])
    det = os.path.join(d, "det.jsonl")
    env = {k: v for k, v in os.environ.items() if not k.startswith("BASH_COMMAND_DETECTOR_")}
    env["AGENT_HOOK_DETECTIONS"] = det
    subprocess.run(["bash", hook], input=json.dumps({"tool_name": "Bash", "session_id": "x", "tool_input": {"command": text, "run_in_background": True}}), text=True, capture_output=True, env=env)
    if not os.path.exists(det):
        return False
    return any(f.startswith("run_in_background 里又自己放后台") for line in open(det) for f in json.loads(line)["findings"])
orig, variant = sys.argv[1], sys.argv[2]
diff = [(k, third(orig, v), third(variant, v)) for k, v in L.items()]
changed = [(k, a, b) for k, a, b in diff if a != b]
print(f"L1–L14 共 {len(L)} 条，变体与原 hook 第三种判定不同的 {len(changed)} 条：" + "、".join(f"{k}（原 {'记' if a else '不记'} → 变体 {'记' if b else '不记'}）" for k, a, b in changed))
