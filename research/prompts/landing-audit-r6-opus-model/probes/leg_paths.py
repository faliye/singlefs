"""K3 交回闸 ② 与派发闸 ⑰ 的三方腿报告认法，三版（第十批 v10、任务 35 之后 v35、此刻 now）在真记录上逐次比。
②：语料里三方腿（此刻 frontmatter 登记的类型）每一次 SubagentHandback 调用的 message，交各版 handback-guard.sh 的 leg_reports_named；
⑰：三方腿每份派发提示交各版 runner-dispatch-guard.sh 的 three_way_report_verdict；另把全部类型的派发提示交 report_file_name_verdict（⑰ 另一判，新的一批输入）。
用法：python3 -I leg_paths.py <语料.jsonl> <仓根（只读：定义 frontmatter、git worktree list）> <树名=树根>…"""
import collections, json, os, re, sys
sys.dont_write_bytecode = True
os.chdir("/")
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import hookload
corpus, root = sys.argv[1:3]
trees = [argument.split("=", 1) for argument in sys.argv[3:]]
handback, dispatch = {}, {}
for name, tree in trees:
    hooks = os.path.join(tree, ".claude", "hooks")
    handback[name] = hookload.load(os.path.join(hooks, "handback-guard.sh"), hooks, f"handback_{name}")
    dispatch[name] = hookload.load(os.path.join(hooks, "runner-dispatch-guard.sh"), hooks, f"dispatch_{name}")
legs = sorted(handback[trees[-1][0]].citation_checked_agents(root))
print("三方腿类型（此刻 frontmatter）：" + "、".join(legs))

def named(module, message):
    try:
        found = module.leg_reports_named(message, root)
    except TypeError:
        found = module.leg_reports_named(message)
    return sorted(item if isinstance(item, str) else item[1] for item in found)

def front(message):
    match = re.search(r"research/prompts/[^\s/`'\"<>]*-output\.md", message)
    if not match:
        return "（没有 research/prompts/…-output.md）"
    head = re.search(r"[^\s`'\"<>：:（(]*\Z", message[:match.start()]).group(0)
    return head or "（不带前缀）"

handbacks, handback_examples, prompts, prompt_examples, names = collections.Counter(), collections.defaultdict(list), collections.Counter(), collections.defaultdict(list), collections.Counter()
for line in open(corpus, encoding="utf-8"):
    item = json.loads(line)
    kind, prompt = item["type"] or "", item["prompt"] or ""
    if prompt:
        verdicts = tuple("拒" if dispatch[name].report_file_name_verdict(kind, prompt) is not None else "放" for name, _ in trees)
        names[verdicts] += 1
    if kind not in legs:
        continue
    if prompt:
        verdicts = tuple("拒" if dispatch[name].three_way_report_verdict(kind, prompt, root) is not None else "放" for name, _ in trees)
        prompts[verdicts] += 1
        if len(prompt_examples[verdicts]) < 3:
            prompt_examples[verdicts].append(item["agent"])
    for record in item["records"]:
        for block in record["message"]["content"]:
            if block.get("type") == "tool_use" and block.get("name") == "SubagentHandback":
                message = str(block.get("input", {}).get("message") or "")
                verdicts = tuple("认出" if named(handback[name], message) else "没认出" for name, _ in trees)
                key = (verdicts, front(message) if len(set(verdicts)) > 1 or verdicts[-1] == "没认出" else "")
                handbacks[key] += 1
                if len(handback_examples[key]) < 2:
                    handback_examples[key].append(f"{item['agent']}（{(item['first_time'] or '')[:10]}）")
order = "→".join(name for name, _ in trees)
print(f"② 三方腿交回（一次调用一次；{order}）：")
for (verdicts, head), count in sorted(handbacks.items(), key=lambda pair: (-pair[1], pair[0])):
    print(f"  {'/'.join(verdicts)}{('；前缀 ' + head) if head else ''}：{count} 次；例：{'、'.join(handback_examples[(verdicts, head)])}")
print(f"  合计 {sum(handbacks.values())} 次")
print(f"⑰ 三方腿派发提示 three_way_report_verdict（一份记录一份；{order}）：")
for verdicts, count in sorted(prompts.items()):
    print(f"  {'/'.join(verdicts)}：{count} 份；例：{'、'.join(prompt_examples[verdicts])}")
print(f"  合计 {sum(prompts.values())} 份")
print(f"⑰ 全部类型派发提示 report_file_name_verdict（{order}）：")
for verdicts, count in sorted(names.items()):
    print(f"  {'/'.join(verdicts)}：{count} 份")
