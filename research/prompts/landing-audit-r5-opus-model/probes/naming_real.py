"""K3 交回闸 ⑤：真会话记录里起过命名检查的每一条 Bash，改前（第十批之前）认不认、改后（只认真跑完的）认不认，按写法分类数；
再把三种要跑命名检查的类型逐份整段判一次（改了 .rs 之后两条都认到没有），数改前放、改后拒的份数。
用法：python3 -I naming_real.py <语料.jsonl> <改后钩子目录> <改前钩子目录> <仓根> <agent 号清单>
数法：一条 Bash 调用一次（「次」）；整段判一份会话记录一份。改前的认法只看命令里有没有，改后的看位置与退出码（退出码取工具结果，语料里存了开头 300 字）。
树有没有 .git 按此刻的文件系统判（当时的 git worktree 副本现在多半删了，那几份按 rsync 副本不判）。只读。"""
import collections, json, os, re, shlex, sys
sys.dont_write_bytecode = True
os.chdir("/")   # 相对路径一律按 / 解（新库与几处认法用 os.path.abspath，结果不随起它的目录变）
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import hookload
corpus, cur_hooks, orig_hooks, root, id_list = sys.argv[1:6]
wanted = set(open(id_list, encoding="utf-8").read().split())
cur = hookload.load(os.path.join(cur_hooks, "handback-guard.sh"), cur_hooks, "cur_handback")
orig = hookload.load(os.path.join(orig_hooks, "handback-guard.sh"), orig_hooks, "orig_handback")
saved = list(sys.argv)
sys.argv = [saved[0], cur_hooks]; heavy_cur = cur.heavy_tests_library()
sys.argv = [saved[0], orig_hooks]; heavy_orig = orig.heavy_tests_library()
sys.argv = saved
assert heavy_cur is not None and heavy_orig is not None

FEATURES = [
    ("接管道到 tail", re.compile(r"\|\s*tail\b")),
    ("接管道到 head（head 先退、命名检查可能被 SIGPIPE 截断）", re.compile(r"\|\s*head\b")),
    ("接管道到 grep", re.compile(r"\|\s*grep\b")),
    ("后面接「; echo」打退出码", re.compile(r";\s*echo\b[^\n]*\$\?|;\s*echo\s+\"?(?:rc|exit|退出)")),
    ("带 || true / || :", re.compile(r"\|\|\s*(?:true|:)\b")),
    ("cd … && 起头", re.compile(r"^\s*cd\s+\S+\s*&&")),
    ("timeout 包着", re.compile(r"\btimeout\s+\d")),
    ("two-logs.sh 包着", re.compile(r"two-logs\.sh")),
    ("同一条里两条用 ; 或换行连着", re.compile(r"code-source-discipline\.sh[^\n;&|]*(?:;|\n)[\s\S]*naming-lint\.sh|naming-lint\.sh[^\n;&|]*(?:;|\n)[\s\S]*code-source-discipline\.sh")),
    ("同一条里两条用 && 连着", re.compile(r"code-source-discipline\.sh[^\n;]*&&[^\n;]*naming-lint\.sh|naming-lint\.sh[^\n;]*&&[^\n;]*code-source-discipline\.sh")),
    ("第一轮 K3-1 的形状：false && / true || / if / 函数定义", re.compile(r"\bfalse\s*&&|\btrue\s*\|\||^\s*if\b|\bfunction\s+\w+|\w+\s*\(\)\s*\{", re.M)),
    ("set -e / set -o pipefail", re.compile(r"\bset\s+-[a-z]*e|pipefail")),
]
def drop_reason(command, exit_code):
    """改后 sequential_naming_text 不收一条命名检查的头一个原因（按它判的次序复述，只用来分类；收不收以它本身为准）→ {命令词串: 原因或 None}。"""
    commands, last_top_list = cur.naming_sequence(command, heavy_cur)
    last_index = len(commands) - 1
    last = commands[last_index] if commands else None
    red_is_naming = (last is not None and last["class"] == "naming" and not last["in_group"] and not last["in_pipeline"] and not last["negated"]
                     and all(other["class"] in ("trivial", "cd", "naming") for other in commands))
    reasons = {}
    for position, item in enumerate(commands):
        if item["class"] != "naming":
            continue
        status_carried = position == last_index and not item["in_group"] and not item["in_pipeline"] and not item["negated"]
        fallible = any(kind in ("cd", "naming") for kind in item["predecessors"])
        if item["blocked"]:
            reason = "在 if / while / until / for / case 或函数体里"
        elif item["after_or"]:
            reason = "在 || 右侧"
        elif item["backgrounded"]:
            reason = "放后台"
        elif item["terminator_before"]:
            reason = "前面有 exit / return / exec"
        elif any(kind not in cur.NAMING_ALLOWED_PREDECESSORS for kind in item["predecessors"]):
            reason = "同一段里前面用 && 接了 cd、赋值、命名检查之外的命令"
        elif exit_code is None:
            reason = "退出码认不出、又包 timeout 或前面接 &&" if (item["timeout"] or fallible) else None
        elif exit_code == 0:
            reason = ("包着 timeout、又不是整条最后一条（后面接 ; echo $?、管道这类，退出码不是它的）" if item["timeout"] and not status_carried else
                      "前面 && 接 cd 或命名检查、又不在顶层最后一段" if fallible and (item["in_group"] or item["list_id"] != last_top_list) else None)
        elif exit_code == 1:
            reason = ("退 1 而整条里另有命令（或最后一条不是命名检查）" if not red_is_naming else
                      "退 1 而同一段前面用 && 接了 cd 或命名检查" if any(kind != "trivial" for kind in item["predecessors"]) else
                      "退 1、包 timeout 又不是最后一条" if item["timeout"] and not status_carried else None)
        else:
            reason = f"退 {exit_code}"
        reasons[shlex.join(item["words"])] = reason
    return reasons

reason_counts = collections.Counter(); reason_examples = collections.defaultdict(list)
period_agents = collections.Counter()
calls = collections.Counter(); examples = collections.defaultdict(list)
agents = collections.Counter(); agent_examples = collections.defaultdict(list)
total_calls = 0
for line in open(corpus, encoding="utf-8"):
    item = json.loads(line)
    if item["agent"] not in wanted:
        continue
    records = item["records"]
    for name, arguments, cwd, ran, exit_code in cur.tool_calls(records):
        if name != "Bash" or not ran:
            continue
        command = str(arguments.get("command") or "")
        if "code-source-discipline.sh" not in command and "naming-lint.sh" not in command:
            continue
        before = orig.naming_check_runs(command, cwd, heavy_orig)
        if not before:
            continue
        total_calls += 1
        completed = cur.completed_naming_text(command, exit_code, heavy_cur)
        after = cur.naming_check_runs(completed, cwd, heavy_cur) if completed else set()
        verdict = "改后照认" if after == before else ("改后一条都不认" if not after else "改后少认一条")
        for words, reason in drop_reason(command, exit_code).items():
            key = reason or "收"
            reason_counts[key] += 1
            if reason and len(reason_examples[key]) < 4:
                reason_examples[key].append(f"{item['agent']}（{item['type']}，{(item['first_time'] or '')[:10]}）：{command[:200]!r}")
        tags = [label for label, pattern in FEATURES if pattern.search(command)] or ["单独一条、没有上面这些写法"]
        for tag in tags:
            key = (verdict, f"退 {exit_code}", tag)
            calls[key] += 1
            if len(examples[key]) < 4:
                examples[key].append(f"{item['agent']}（{item['type']}）：{command[:160]!r}")
    if item["type"] in orig.NAMING_CHECKED_AGENTS:
        hook_input = {"agent_id": item["agent"], "transcript_path": "/dev/null"}
        cur.own_transcript_records = lambda _input, records=records: records
        orig.own_transcript_records = lambda _input, records=records: records
        before = orig.naming_problems(hook_input, root, heavy_orig)
        after = cur.naming_problems(hook_input, root, heavy_cur)
        key = ("改前放" if not before else "改前拒") + "、" + ("改后放" if not after else "改后拒")
        edited = any(name in cur.FILE_EDITING_TOOLS and ran and (cur.naming_source_tree(arguments.get("file_path"), root) or (None, False))[1]
                     for name, arguments, _cwd, ran, _code in cur.tool_calls(records))
        if edited:
            day = (item["first_time"] or "")[:10]
            period_agents[(("2026-10-22 起" if day >= "2026-10-22" else "2026-10-22 之前"), key)] += 1
        agents[key] += 1
        if len(agent_examples[key]) < 8 and key != "改前放、改后放":
            agent_examples[key].append(f"{item['agent']}（{item['type']}，{(item['first_time'] or '')[:10]}）")
print(f"起过命名检查、改前认得出的 Bash 调用：{total_calls} 次")
for key in sorted(calls):
    print(f"{'；'.join(key)}：{calls[key]} 次；例：{' ｜ '.join(examples[key])}")
print("改后逐条命名检查收不收、不收的头一个原因（一条 Bash 里每条命名检查各数一次）：")
for key in sorted(reason_counts):
    print(f"  {key}：{reason_counts[key]} 条；例：{' ｜ '.join(reason_examples[key])}")
print("改了要判的 .rs（主工作区或此刻还带 .git 的树）的那几份，按派发日期：")
for key in sorted(period_agents):
    print(f"  {'：'.join(key)}：{period_agents[key]} 份")
print("三种要跑命名检查的类型逐份整段判（改了 .rs 之后两条都认到没有）：")
for key in sorted(agents):
    print(f"  {key}：{agents[key]} 份；例：{'、'.join(agent_examples[key])}")
