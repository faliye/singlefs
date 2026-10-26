"""K3：派发闸 ⑰ 两判（报告文件名、三方腿报告形态）在真派发提示上改前改后翻了几份。
用法：python3 -I report_verdicts.py <语料.jsonl> <改后钩子目录> <改前钩子目录> <仓根（只读：读定义 frontmatter 认三方腿类型）> <agent 号清单>
数法同 scratch_lines.py：一份会话记录一份；按派发日期（第一条记录的 UTC 日期）分三段。只读。"""
import collections, json, os, re, sys
sys.dont_write_bytecode = True
os.chdir("/")   # 相对路径一律按 / 解（新库与几处认法用 os.path.abspath，结果不随起它的目录变）
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import hookload

corpus, cur_hooks, orig_hooks, root, id_list = sys.argv[1:6]
wanted = set(open(id_list, encoding="utf-8").read().split())
cur = hookload.load(os.path.join(cur_hooks, "runner-dispatch-guard.sh"), cur_hooks, "cur_dispatch")
orig = hookload.load(os.path.join(orig_hooks, "runner-dispatch-guard.sh"), orig_hooks, "orig_dispatch")

def mention_shape(prompt, directories):
    """改前 ⑰ 认出的那个目录在提示里是怎么写的：取第一处「草稿目录」后面接着它的那一行。"""
    for text in prompt.split("\n"):
        if "草稿目录" not in text:
            continue
        if re.match(r"^[ \t]*[-*+>][ \t]*[*_]*草稿目录", text) or re.match(r"^[ \t]*\*\*草稿目录", text):
            return "列表、引用或粗体起头的「草稿目录：」行"
        if re.search(r"\S.*?(?<![^\W_])草稿目录[* \t]*[：:]", text):
            return "句中带冒号的「…；草稿目录：<路径>」"
        if re.search(r"(?<![^\W_])草稿目录[ \t]+`?(?:~|/|\$)", text):
            return "句中不带冒号的「草稿目录 <路径>」"
    return "别的写法"


def period(stamp):
    day = (stamp or "")[:10]
    return "2026-10-14 之前" if day < "2026-10-14" else ("2026-10-14 到 10-21" if day < "2026-10-22" else "2026-10-22 起")

flips, examples, totals = collections.Counter(), collections.defaultdict(list), collections.Counter()
for line in open(corpus, encoding="utf-8"):
    item = json.loads(line)
    if item["agent"] not in wanted or not item["prompt"]:
        continue
    kind, prompt, when = item["type"] or "", item["prompt"], period(item["first_time"])
    totals[when] += 1
    for name, judge in (("报告文件名", lambda module: module.report_file_name_verdict(kind, prompt)),
                        ("三方腿报告形态", lambda module: module.three_way_report_verdict(kind, prompt, root))):
        before, after = judge(orig) is not None, judge(cur) is not None
        if before != after:
            canonical = bool(cur.prompt_scratch_directories(prompt))
            shape = "提示里有行首「草稿目录：」一行（改前认的是转述里的别处）" if canonical else (
                "没有行首那一行：本提示自己的草稿目录写在" + mention_shape(prompt, orig.report_scratch_directories(prompt)))
            key = (name, "改前拒、改后放" if before else "改前放、改后拒", when, shape)
            flips[key] += 1
            if len(examples[key]) < 8:
                examples[key].append(f"{item['agent']}（{kind}）")
print("语料按日期：" + "、".join(f"{when} {count} 份" for when, count in sorted(totals.items())))
for key in sorted(flips):
    print(f"{'：'.join(key)}：{flips[key]} 份；例：{'、'.join(examples[key])}")
if not flips:
    print("两判改前改后一份都没翻")
