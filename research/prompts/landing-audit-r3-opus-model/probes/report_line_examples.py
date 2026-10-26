"""对 report_line_corpus 列出的两类差（今天不认、第五批认）逐个打：开工日期（东京）、类型、报告标签前后原文、
子 agent 有没有用 Write 写那个 report.md 而被工具层拒（工具结果里有「Subagents should return findings as text」）。
用法：report_line_examples.py <派发闸路径> <仓根> <会话目录根>"""
import datetime, glob, json, os, re, sys, zoneinfo, collections
guard, repo, sessions = sys.argv[1:4]
text = open(guard).read().splitlines()
start = next(i for i, l in enumerate(text) if l.startswith("python3 /dev/fd/3")) + 1
end = max(i for i, l in enumerate(text) if l.strip() == "sys.exit(main())")
sys.argv = [sys.argv[0], os.path.dirname(os.path.abspath(guard))]
g = {"__name__": "rdg_probe"}
exec(compile("\n".join(text[start:end]), "rdg_probe", "exec"), g)
TODAY, UNANCHORED = g["REPORT_LINE"], g["REPORT_LINE_UNANCHORED"]
TOKYO = zoneinfo.ZoneInfo("Asia/Tokyo")
REFUSED = "Subagents should return findings as text"
def first(path):
    stamp, prompt, refused = None, None, 0
    for line in open(path, errors="replace"):
        try: r = json.loads(line)
        except ValueError: continue
        if stamp is None and r.get("timestamp"):
            stamp = datetime.datetime.fromisoformat(r["timestamp"].replace("Z", "+00:00"))
        if prompt is None and r.get("type") == "user":
            c = r["message"]["content"]
            prompt = c if isinstance(c, str) else "".join(b.get("text", "") for b in c if b.get("type") == "text")
        if REFUSED in line: refused += 1
    return stamp, prompt, refused
def check(kind, prompt, pattern):
    g["REPORT_LINE"] = pattern
    try:
        return (g["report_file_name_verdict"](kind, prompt) is not None, g["three_way_report_verdict"](kind, prompt, repo) is not None)
    finally:
        g["REPORT_LINE"] = TODAY
groups = collections.defaultdict(list)
for meta in sorted(glob.glob(os.path.join(sessions, "*", "subagents", "agent-*.meta.json"))):
    transcript = meta.replace(".meta.json", ".jsonl")
    if not os.path.exists(transcript): continue
    try: kind = json.load(open(meta)).get("agentType") or "?"
    except ValueError: continue
    stamp, prompt, refused = first(transcript)
    if not prompt: continue
    t_name, t_form = check(kind, prompt, TODAY)
    u_name, u_form = check(kind, prompt, UNANCHORED)
    agent = os.path.basename(transcript)[6:-6]
    contexts = []
    for m in UNANCHORED.finditer(g["without_code_blocks"](prompt)):
        if not any(t.start() <= m.start() < t.end() + 1 for t in TODAY.finditer(g["without_code_blocks"](prompt))):
            s = g["without_code_blocks"](prompt)
            contexts.append(s[max(0, m.start() - 24):m.start() + 60].replace("\n", "⏎"))
    day = stamp.astimezone(TOKYO).date().isoformat() if stamp else "?"
    if u_name and not t_name: groups["name_passes_today"].append((day, agent, kind, refused, contexts))
    if t_form and not u_form: groups["form_refused_today"].append((day, agent, kind, refused, contexts))
for key in ("name_passes_today", "form_refused_today"):
    rows = sorted(groups[key])
    print(f"== {key} {len(rows)}; 开工于东京 2026-10-14 及以后的 {sum(r[0] >= '2026-10-14' for r in rows)}; 其中会话里有工具层拒写的 {sum(r[0] >= '2026-10-14' and r[3] > 0 for r in rows)}")
    print("   按日期：", dict(collections.Counter(r[0] for r in rows)))
    for day, agent, kind, refused, contexts in rows[-12:]:
        print(f"   {day} {agent} {kind} 工具层拒写 {refused} 次 | {contexts[0] if contexts else ''}")
