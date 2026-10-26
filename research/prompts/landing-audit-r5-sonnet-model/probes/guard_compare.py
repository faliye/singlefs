"""把真派发提示逐份喂第九批（旧）与第十批（新）的派发闸 --check-prompt，比报告文件名那一条理由。
用法：python3 -I guard_compare.py <抽出来的目录> <树目录> <仓根> <开始日期>
  树目录里要有 tree_old、tree_new 两棵（rerun.sh 建：.claude/hooks 是各自那一版，.claude 其余项与 scripts 符号链接回仓根）
  --check-prompt 不建锁、不读主会话记录；stdin 接 /dev/null（它会先读 hook JSON，接着终端就挂住）"""
import collections, concurrent.futures, json, os, subprocess, sys
out, trees, repo, start = sys.argv[1:5]
rows = [r for r in json.load(open(out + "/subagent_prompts.json")) if r["ts"][:10] >= start and r["type"]]
work = out + "/guard_prompts"
os.makedirs(work, exist_ok=True)
REPORT_NAME = "报告行点名的文件名里带 report"
def judge(job):
    tree, row = job
    path = f"{work}/{row['agent']}.md"
    if not os.path.exists(path):
        open(path, "w", encoding="utf-8").write(row["prompt"])
    done = subprocess.run(["bash", f"{trees}/tree_{tree}/.claude/hooks/runner-dispatch-guard.sh", "--check-prompt", path, "--type", row["type"], "--root", repo],
                          capture_output=True, text=True, stdin=subprocess.DEVNULL, cwd=repo, timeout=120)
    return tree, row["agent"], done.returncode, done.stdout + done.stderr
results = collections.defaultdict(dict)
with concurrent.futures.ThreadPoolExecutor(6) as pool:
    for tree, agent, code, text in pool.map(judge, [(t, r) for t in ("old", "new") for r in rows]):
        results[agent][tree] = (code, text)
byday = {r["agent"]: r["ts"][:10] for r in rows}
types = {r["agent"]: r["type"] for r in rows}
pairs = collections.Counter((results[a]["old"][0], results[a]["new"][0]) for a in results)
print(f"name=guard_compare 提示={len(rows)}（{start} 起） 退出码（旧,新）分布={dict(sorted(pairs.items()))}")
lost = sorted(a for a in results if REPORT_NAME in results[a]["old"][1] and REPORT_NAME not in results[a]["new"][1])
gained = sorted(a for a in results if REPORT_NAME not in results[a]["old"][1] and REPORT_NAME in results[a]["new"][1])
perday = collections.Counter(byday[a] for a in lost)
print(f"name=report_name_reason 旧拒新放={len(lost)} 旧放新拒={len(gained)} 按日={dict(sorted(perday.items()))}")
print(f"name=report_name_reason_since_2026-10-15 旧拒新放={sum(1 for a in lost if byday[a] >= '2026-10-15')}")
print(f"name=only_reason 旧只因这一条拒、新放行={sum(1 for a in lost if results[a]['old'][0] == 2 and results[a]['new'][0] == 0)}")
for a in ("a0a2dfee322e26786", "a789e7091d91430aa", "a8afb2185cfa840cc", "a9717cf07fb6e7c21"):
    if a in results:
        print(f"name=named_in_90_section_20 {a} {types[a]} 旧={'拒' if REPORT_NAME in results[a]['old'][1] else '放'} 新={'拒' if REPORT_NAME in results[a]['new'][1] else '放'}")
