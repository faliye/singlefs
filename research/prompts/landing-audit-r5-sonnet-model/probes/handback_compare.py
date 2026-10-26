"""把真子 agent 会话（实现员、工具员、执行员）逐份喂三棵交回闸：旧（第九批）、新（第十批）、只认退 0（新的把 NAMING_COMPLETED_EXIT_CODES 改成 (0,)），比命名检查那一条。
用法：python3 -I handback_compare.py <抽出来的目录> <树目录> <仓根>
  树目录里要有 tree_old、tree_new、tree_only0；钩子读会话记录里的命令，不读报告；hook 输入的 transcript_path 是主会话那份，子 agent 那份在 <会话>/subagents/ 下"""
import collections, concurrent.futures, json, os, re, subprocess, sys
out, trees, repo = sys.argv[1:4]
PHRASE = "没跑过共用约束点名的两条命名检查"
rows = [r for r in json.load(open(out + "/subagent_prompts.json")) if r["type"] in ("implementation-writer", "tooling-writer", "experiment-runner")]
projects = os.path.dirname(os.path.dirname(os.path.dirname(rows[0]["path"])))
def judge(job):
    tree, row = job
    hook_input = {"tool_name": "SubagentHandback", "tool_input": {"message": "交回"}, "agent_type": row["type"], "agent_id": row["agent"],
                  "transcript_path": f"{projects}/{row['session']}.jsonl"}
    done = subprocess.run(["bash", f"{trees}/tree_{tree}/.claude/hooks/handback-guard.sh"], input=json.dumps(hook_input), capture_output=True, text=True,
                          env={**os.environ, "CLAUDE_PROJECT_DIR": repo}, cwd=repo, timeout=120)
    return tree, row["agent"], done.returncode, PHRASE in (done.stdout + done.stderr)
refused = collections.defaultdict(dict)
with concurrent.futures.ThreadPoolExecutor(6) as pool:
    for tree, agent, code, naming in pool.map(judge, [(t, r) for t in ("old", "new", "only0") for r in rows]):
        refused[tree][agent] = naming
agents = [r["agent"] for r in rows]
ran = {}
for r in rows:
    commands = []
    for line in open(r["path"], encoding="utf-8", errors="replace"):
        if "naming-lint.sh" not in line:
            continue
        try:
            content = json.loads(line).get("message", {}).get("content")
        except Exception:
            continue
        for part in content if isinstance(content, list) else []:
            if isinstance(part, dict) and part.get("type") == "tool_use" and part.get("name") == "Bash" and "naming-lint.sh" in part.get("input", {}).get("command", ""):
                commands.append(part["input"]["command"])
    if commands:
        ran[r["agent"]] = commands
count = lambda tree, subset: sum(1 for a in subset if refused[tree][a])
print(f"name=handback_compare 子agent={len(agents)} 命名检查那条拒：旧={count('old', agents)} 新={count('new', agents)} 只认退0={count('only0', agents)}")
print(f"name=only0_vs_new 新与只认退0判得不同的={sum(1 for a in agents if refused['new'][a] != refused['only0'][a])}")
newly = sorted(a for a in agents if refused["new"][a] and not refused["old"][a])
print(f"name=newly_refused 新拒而旧放={len(newly)} 其中调用过 naming-lint.sh 的={sum(1 for a in newly if a in ran)} {newly}")
print(f"name=ran_naming_lint 调用过 naming-lint.sh 的子agent={len(ran)} 其中旧拒={count('old', ran)} 新拒={count('new', ran)}")
for a in newly:
    last = ran[a][-1] if a in ran else ""
    print(f"name=newly_refused_command {a} " + re.sub(r"\s+", " ", last)[:150])

# 这些会话改的 .rs 在哪：主工作区的 crates/ 与 scripts/ 下（交回闸判）、只在别处（副本、草稿目录；rsync 拷的没有 .git，交回闸不判）、没改 .rs
main_prefix = repo.rstrip("/") + "/"
classes = collections.Counter()
for r in rows:
    main_edit = other_edit = False
    seen = set()
    for line in open(r["path"], encoding="utf-8", errors="replace"):
        if '"tool_use"' not in line:
            continue
        try:
            content = json.loads(line).get("message", {}).get("content")
        except Exception:
            continue
        for part in content if isinstance(content, list) else []:
            if isinstance(part, dict) and part.get("type") == "tool_use" and part.get("name") in ("Edit", "Write", "MultiEdit") and part.get("id") not in seen:
                seen.add(part.get("id"))
                path = part.get("input", {}).get("file_path", "")
                if path.endswith(".rs") and re.search(r"/(crates|scripts)/", path):
                    if path.startswith(main_prefix + "crates/") or path.startswith(main_prefix + "scripts/"):
                        main_edit = True
                    else:
                        other_edit = True
    classes["改了主工作区 crates/ 或 scripts/ 下的 .rs" if main_edit else ("只改了别处的 .rs" if other_edit else "没改 .rs")] += 1
print(f"name=edit_classes 子agent={len(rows)} {dict(sorted(classes.items()))}")
