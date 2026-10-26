"""今天（行首或句首）认、只行首不认的报告行里点名带 report 的文件：逐个打开工日期、类型、那一句、点名的文件与提示里「草稿目录」是否同一棵。
用法：sentence_start_extra.py <派发闸路径> <会话目录根> <最早东京日期>"""
import datetime, glob, json, os, re, sys, zoneinfo, collections
guard, sessions, since = sys.argv[1:4]
text = open(guard).read().splitlines()
start = next(i for i, l in enumerate(text) if l.startswith("python3 /dev/fd/3")) + 1
end = max(i for i, l in enumerate(text) if l.strip() == "sys.exit(main())")
sys.argv = [sys.argv[0], os.path.dirname(os.path.abspath(guard))]
g = {"__name__": "rdg_probe"}
exec(compile("\n".join(text[start:end]), "rdg_probe", "exec"), g)
LINE_ONLY = re.compile(r"(?P<lead>^[ \t>*_\-]*(?:\d+[.)][ \t]+[ \t*_]*)?)" + g["REPORT_LABEL"] + r"(?P<rest>[^\n]*)", re.M)
TOKYO = zoneinfo.ZoneInfo("Asia/Tokyo")
rows = []
for meta in sorted(glob.glob(os.path.join(sessions, "*", "subagents", "agent-*.meta.json"))):
    transcript = meta.replace(".meta.json", ".jsonl")
    if not os.path.exists(transcript): continue
    kind = json.load(open(meta)).get("agentType") or "?"
    stamp = prompt = None
    for line in open(transcript, errors="replace"):
        r = json.loads(line) if line.strip().startswith("{") else {}
        if stamp is None and r.get("timestamp"): stamp = datetime.datetime.fromisoformat(r["timestamp"].replace("Z", "+00:00"))
        if r.get("type") == "user":
            c = r["message"]["content"]; prompt = c if isinstance(c, str) else "".join(b.get("text", "") for b in c if b.get("type") == "text"); break
    if not prompt or not stamp or stamp.astimezone(TOKYO).date().isoformat() < since: continue
    body = g["without_code_blocks"](prompt)
    line_starts = {m.start("rest") for m in LINE_ONLY.finditer(body)}
    draft = re.search(r"草稿目录[：:]\s*`?([^\s`，。]+)", prompt)
    for m in g["REPORT_LINE"].finditer(body):
        if m.start("rest") in line_starts: continue
        names = [t.group(0) for t in g["REPORT_NAME_TOKEN"].finditer(m.group("rest")) if re.search(r"report[^/]*\.\w+$", t.group(0).rstrip("/."), re.I)]
        if names:
            rows.append((stamp.astimezone(TOKYO).date().isoformat(), os.path.basename(transcript)[6:-6], kind, names[0], draft.group(1) if draft else None,
                         body[max(0, m.start() - 30):m.end("rest")][:150].replace("\n", "⏎")))
print(f"句首多认、点名带 report 的文件：{len(rows)} 处（东京 {since} 起开工的提示）")
for row in rows:
    same = bool(row[4]) and row[3].startswith(row[4].rstrip("/"))
    print(f"  {row[0]} {row[1]} {row[2]} 与草稿目录同棵={same} | {row[5]}")
