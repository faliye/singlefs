"""三方腿：结局账认到的（报告, 判决）对，与这条腿自己派发提示报告行点名的 -output.md 比。
自己的报告取首条提示里第一个 research/prompts/<名>-output.md 写在「报告」标签后面的；判决按 <名> 去掉模型尾巴（-opus、-sonnet、-opus-s2 这类）取。
用法：leg_own_report.py <today agents.jsonl> <会话目录根> <仓根>"""
import collections, glob, json, os, re, sys, subprocess
rows = [json.loads(l) for l in open(sys.argv[1])]
sessions, repo = sys.argv[2], sys.argv[3]
GUARD = os.path.join(repo if False else sys.argv[3], ".claude/hooks/runner-dispatch-guard.sh")
_text = open(GUARD).read().splitlines()
_start = next(i for i, l in enumerate(_text) if l.startswith("python3 /dev/fd/3")) + 1
_end = max(i for i, l in enumerate(_text) if l.strip() == "sys.exit(main())")
_saved = sys.argv; sys.argv = [sys.argv[0], os.path.dirname(GUARD)]
g = {"__name__": "rdg_probe"}; exec(compile("\n".join(_text[_start:_end]), "rdg_probe", "exec"), g); sys.argv = _saved
FORM = re.compile(r"research/prompts/([^/\s`'\"]+)-output\.md")
def own_report(prompt):
    for pattern in (g["REPORT_LINE"], g["REPORT_LINE_UNANCHORED"]):
        g_saved = g["REPORT_LINE"]; g["REPORT_LINE"] = pattern
        try:
            for m in g["report_line_mentions"](prompt):
                named = (m["named"] or "").split("/home/fy5090/code/singlefs/")[-1]
                f = FORM.fullmatch(named)
                if f: return named, f.group(1)
        finally:
            g["REPORT_LINE"] = g_saved
    return None
MODEL_TAIL = re.compile(r"-(?:opus|sonnet|haiku)(?:-s\d+)?$")
known = set(subprocess.run(["git", "-C", repo, "log", "--branches", "HEAD", "--no-renames", "--name-only", "--format=", "--", "research/prompts"], capture_output=True, text=True).stdout.split())
known |= set(subprocess.run(["git", "-C", repo, "ls-files", "-co", "--exclude-standard", "--", "research/prompts"], capture_output=True, text=True).stdout.split())
stats = collections.Counter(); odd = []
for r in rows:
    if not (r["type"].startswith("three-way") and r["handed_back"]): continue
    path = glob.glob(os.path.join(sessions, "*", "subagents", f"agent-{r['agent']}.jsonl"))[0]
    prompt = ""
    for line in open(path, errors="replace"):
        if '"type":"user"' in line.replace(" ", ""):
            c = json.loads(line)["message"]["content"]; prompt = c if isinstance(c, str) else "".join(b.get("text", "") for b in c if b.get("type") == "text"); break
    own = own_report(prompt)
    confirmed = [tuple(x) for x in (r.get("leg_verdicts") or [])]
    stats["legs"] += 1
    if not own:
        stats["own_report_not_found_in_prompt"] += 1; odd.append(("no-own", r["agent"], r["type"], r["named_outputs"], confirmed)); continue
    own_path, stem = own
    own_round = MODEL_TAIL.sub("", stem.lstrip("_"))
    own_verdict = f"research/prompts/{own_round}-main-verification.md"
    hit = [c for c in confirmed if c[0] == own_path]
    if hit and hit[0][1] == own_verdict: stats["own_report_and_own_round_verdict"] += 1
    elif hit: stats["own_report_but_other_verdict"] += 1; odd.append(("other-verdict", r["agent"], r["type"], own_path, hit))
    elif confirmed: stats["credited_by_another_named_report"] += 1; odd.append(("credited-by-other", r["agent"], r["type"], own_path, confirmed))
    else: stats["not_credited"] += 1
    stats["own_report_known"] += own_path in known
    stats["own_round_verdict_known"] += own_verdict in known
    if len(r["named_outputs"] or []) > 1: stats["handback_names_more_than_one_output"] += 1
print(dict(stats))
for item in odd[:20]: print("  ", item)
