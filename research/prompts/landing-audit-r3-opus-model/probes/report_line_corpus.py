"""把派发闸内嵌的 python 抽出来当模块载入（不跑 main），在真派发首条提示上比报告行三种认法下 ④ 写范围、⑰ 文件名、⑰ 三方形态三样判定。
认法：today＝今天的 REPORT_LINE（行首或句首）；line_only＝去掉句首那一支（第二轮判决 K2-3 写的「只在行首认」）；unanchored＝第五批的句中也认。
用法：report_line_corpus.py <派发闸路径> <仓根> <会话目录根> <输出前缀>"""
import glob, json, os, re, sys, collections
guard, repo, sessions, out = sys.argv[1:5]
text = open(guard).read().splitlines()
start = next(i for i, l in enumerate(text) if l.startswith("python3 /dev/fd/3")) + 1
end = max(i for i, l in enumerate(text) if l.strip() == "sys.exit(main())")
hook_dir = os.path.dirname(os.path.abspath(guard))
sys.argv = [sys.argv[0], hook_dir]
namespace = {"__name__": "rdg_probe"}
exec(compile("\n".join(text[start:end]), "rdg_probe", "exec"), namespace)
g = namespace
TODAY = g["REPORT_LINE"]
LINE_ONLY = re.compile(r"(?P<lead>^[ \t>*_\-]*(?:\d+[.)][ \t]+[ \t*_]*)?)" + g["REPORT_LABEL"] + r"(?P<rest>[^\n]*)", re.M)
UNANCHORED = g["REPORT_LINE_UNANCHORED"]
def verdicts(kind, prompt, pattern):
    g["REPORT_LINE"] = pattern
    name = g["report_file_name_verdict"](kind, prompt) is not None
    form = g["three_way_report_verdict"](kind, prompt, repo) is not None
    try:
        scope = bool(g["out_of_scope_mentions"](repo, kind, prompt))
    except Exception as error:
        scope = "err:" + type(error).__name__
    mentions = [m["named"] for m in g["report_line_mentions"](prompt)]
    g["REPORT_LINE"] = TODAY
    return (name, form, scope), mentions
def first_prompt(path):
    for line in open(path, errors="replace"):
        try: record = json.loads(line)
        except ValueError: continue
        if record.get("type") == "user":
            content = record["message"]["content"]
            return content if isinstance(content, str) else "".join(b.get("text", "") for b in content if b.get("type") == "text")
    return None
rows, diff = 0, collections.Counter()
examples = collections.defaultdict(list)
for meta in sorted(glob.glob(os.path.join(sessions, "*", "subagents", "agent-*.meta.json"))):
    transcript = meta.replace(".meta.json", ".jsonl")
    if not os.path.exists(transcript): continue
    try: kind = json.load(open(meta)).get("agentType") or "?"
    except ValueError: continue
    prompt = first_prompt(transcript)
    if not prompt: continue
    rows += 1
    today, m_today = verdicts(kind, prompt, TODAY)
    line_only, m_line = verdicts(kind, prompt, LINE_ONLY)
    unanchored, m_un = verdicts(kind, prompt, UNANCHORED)
    agent = os.path.basename(transcript)[6:-6]
    for label, other, m_other in (("today_vs_line_only", line_only, m_line), ("today_vs_unanchored", unanchored, m_un)):
        for index, cell in enumerate(("name", "form", "scope")):
            if today[index] != other[index]:
                key = f"{label}:{cell}:today={today[index]}:other={other[index]}"
                diff[key] += 1
                examples[key].append((agent, kind, m_today, m_other))
    if m_today != m_line:
        diff["mentions_differ_today_vs_line_only"] += 1
        examples["mentions_differ_today_vs_line_only"].append((agent, kind, m_today, m_line))
with open(out + ".txt", "w") as handle:
    handle.write(f"prompts {rows}\n")
    for key, count in sorted(diff.items()):
        handle.write(f"{key} {count}\n")
        for agent, kind, a, b in examples[key][:8]:
            handle.write(f"    {agent} {kind} today={a} other={b}\n")
print(open(out + ".txt").read())
