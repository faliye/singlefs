"""真派发首条提示上，⑰ 文件名判在「紧跟否定词的不判」开着与关着（弄坏开关 report-name-negation-judged）两种下判定不同的提示，逐个打那一行。
用法：negation_corpus.py <派发闸路径> <会话目录根>"""
import glob, json, os, re, sys
guard, sessions = sys.argv[1:3]
text = open(guard).read().splitlines()
start = next(i for i, l in enumerate(text) if l.startswith("python3 /dev/fd/3")) + 1
end = max(i for i, l in enumerate(text) if l.strip() == "sys.exit(main())")
sys.argv = [sys.argv[0], os.path.dirname(os.path.abspath(guard))]
g = {"__name__": "rdg_probe"}
exec(compile("\n".join(text[start:end]), "rdg_probe", "exec"), g)
count = differ = 0
for meta in sorted(glob.glob(os.path.join(sessions, "*", "subagents", "agent-*.meta.json"))):
    transcript = meta.replace(".meta.json", ".jsonl")
    if not os.path.exists(transcript): continue
    kind = json.load(open(meta)).get("agentType") or "?"
    prompt = None
    for line in open(transcript, errors="replace"):
        if '"type":"user"' in line.replace(" ", ""):
            r = json.loads(line); c = r["message"]["content"]
            prompt = c if isinstance(c, str) else "".join(b.get("text", "") for b in c if b.get("type") == "text"); break
    if not prompt: continue
    count += 1
    os.environ.pop("RUNNER_DISPATCH_GUARD_BREAK", None)
    today = g["report_file_name_verdict"](kind, prompt)
    os.environ["RUNNER_DISPATCH_GUARD_BREAK"] = "report-name-negation-judged"
    judged = g["report_file_name_verdict"](kind, prompt)
    os.environ.pop("RUNNER_DISPATCH_GUARD_BREAK", None)
    if (today is None) != (judged is None):
        differ += 1
        lines = [l.strip()[:200] for l in prompt.splitlines() if "report" in l.lower() and "报告" in l]
        print(os.path.basename(transcript)[6:-6], kind, "今天", "拒" if today else "放行", "| 否定词也判", "拒" if judged else "放行", "|", lines[:2])
print(f"提示 {count} 份，两种判法不同 {differ} 份")
