"""K3 与 K2 草稿目录一行：拿真会话记录里每份派发提示，分别喂改前（第十批之前）与改后的几处认法，数形状与认出来的目录不同的份数。
用法：python3 -I scratch_lines.py <语料.jsonl> <改后钩子目录> <改前钩子目录> <scripts/agents 目录> <只收这几个 agent 号的清单，一行一个>
数法：一份子 agent 会话记录算一份（「份记录」）；派发提示文字逐字相同的（sha256 相同）合成一份（「份不同提示」）。只读语料，不写文件。"""
import collections, hashlib, importlib.util, json, os, re, sys
sys.dont_write_bytecode = True
os.chdir("/")   # 相对路径一律按 / 解（新库与几处认法用 os.path.abspath，结果不随起它的目录变）
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import hookload

corpus, cur_hooks, orig_hooks, agents_dir, id_list = sys.argv[1:6]
wanted = set(open(id_list, encoding="utf-8").read().split())

def library(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

cur_dispatch = hookload.load(os.path.join(cur_hooks, "runner-dispatch-guard.sh"), cur_hooks, "cur_dispatch")
orig_dispatch = hookload.load(os.path.join(orig_hooks, "runner-dispatch-guard.sh"), orig_hooks, "orig_dispatch")
cur_handback = hookload.load(os.path.join(cur_hooks, "handback-guard.sh"), cur_hooks, "cur_handback")
orig_handback = hookload.load(os.path.join(orig_hooks, "handback-guard.sh"), orig_hooks, "orig_handback")
orig_cleanup = hookload.load(os.path.join(orig_hooks, "completion-cleanup-guard.sh"), orig_hooks, "orig_cleanup")
lib = library(os.path.join(cur_hooks, "lib_prompt_scratch_line.py"), "lib_prompt_scratch_line_probe")
roots = library(os.path.join(cur_hooks, "lib_work_root.py"), "lib_work_root_probe")
skeleton = library(os.path.join(agents_dir, "dispatch-prompt.py"), "dispatch_prompt_probe")
cost_text = open(os.path.join(agents_dir, "agent-cost.py"), encoding="utf-8").read()
watch_text = open(os.path.join(agents_dir, "agent-watch.py"), encoding="utf-8").read()
# agent-cost.py 与 agent-watch.py 的那一处认法逐字取自源文件（不 import 整份脚本），取不到就停
cost_chars = re.search(r'^PATH_CHARS = r"(.*?)"', cost_text, re.M).group(1)
assert 'draft = re.search(r"草稿目录[：:]\\s*(" + PATH_CHARS + "+)", prompt or "")' in cost_text
assert 'SCRATCH_PATH_MENTION = re.compile(work_roots.mention_prefix_pattern() + r"[^\\s`\'\\"，。；：、（）()「」<>]+")' in watch_text
COST_DRAFT = re.compile(r"草稿目录[：:]\s*(" + cost_chars + "+)")
WATCH_MENTION = re.compile(roots.mention_prefix_pattern() + r"[^\s`'\"，。；：、（）()「」<>]+")
base_roots = [root.rstrip("/") for root in roots.scratch_roots()]

def expand(value):
    return os.path.normpath(os.path.abspath(roots.expand_written_path(value.rstrip("/"))))

def under(path):
    return any(path.startswith(root + "/") for root in base_roots)

SHAPES = collections.OrderedDict([
    ("骨架那一行（第 0 列、全角冒号）", re.compile(r"^草稿目录：")),
    ("行首空白之后、或半角冒号、或冒号前有空白（新库认，骨架不打）", re.compile(r"^(?:[ \t]+草稿目录[ \t]*[：:]|草稿目录[ \t]*:|草稿目录[ \t]+[：:])")),
    ("列表记号起头的「- 草稿目录：」「* 草稿目录：」", re.compile(r"^[ \t]*[-*+][ \t]+草稿目录[* \t]*[：:]")),
    ("编号起头的「1. 草稿目录：」", re.compile(r"^[ \t]*\d+[.)][ \t]+[*_]*草稿目录[* \t]*[：:]")),
    ("引用记号起头的「> 草稿目录：」", re.compile(r"^[ \t]*>[ \t]*[*_]*草稿目录[* \t]*[：:]")),
    ("粗体起头的「**草稿目录**：」「**草稿目录：**」", re.compile(r"^[ \t]*(?:[-*+][ \t]+)?\*\*草稿目录")),
    ("表格里的「| 草稿目录 |」", re.compile(r"^[ \t]*\|.*草稿目录")),
    ("句中带冒号的「…，草稿目录：」（不在行首）", re.compile(r"\S.*?(?<![^\W_])草稿目录[* \t]*[：:]")),
    ("句中不带冒号的「，草稿目录 <路径>」", re.compile(r"(?<![^\W_])草稿目录[ \t]+`?(?:~|/|\$)")),
    ("「草稿目录在 <路径>」", re.compile(r"草稿目录在[ \t]*`?(?:~|/|\$)")),
])

CURRENT_PERIOD = [None]
per_agent, per_prompt = collections.Counter(), collections.defaultdict(set)
examples = collections.defaultdict(list)
total_agents, total_prompts, with_label = 0, set(), set()

periods = collections.defaultdict(collections.Counter)

def period(stamp):
    day = (stamp or "")[:10]
    return "2026-10-14 之前" if day < "2026-10-14" else ("2026-10-14 到 10-21" if day < "2026-10-22" else "2026-10-22 起")

def note(key, agent, digest):
    periods[key][CURRENT_PERIOD[0]] += 1
    per_agent[key] += 1
    per_prompt[key].add(digest)
    if len(examples[key]) < 6 and agent not in examples[key]:
        examples[key].append(agent)

with open(corpus, encoding="utf-8") as handle:
    for line in handle:
        item = json.loads(line)
        if item["agent"] not in wanted:
            continue
        prompt = item["prompt"]
        if not prompt:
            continue
        agent = f"{item['agent']}（{item['type']}）"
        CURRENT_PERIOD[0] = period(item["first_time"])
        digest = hashlib.sha256(prompt.encode("utf-8")).hexdigest()
        total_agents += 1
        total_prompts.add(digest)
        if "草稿目录" not in prompt:
            continue
        with_label.add(digest)
        note("提示里出现「草稿目录」", agent, digest)
        lines = prompt.split("\n")
        fenced = lib.fenced_line_numbers(lines)
        for number, text in enumerate(lines):
            if "草稿目录" not in text:
                continue
            for name, pattern in SHAPES.items():
                if pattern.search(text):
                    note(("行形状", name), agent, digest)
            if number in fenced and re.match(r"^[ \t]*草稿目录[ \t]*[：:]", text):
                note(("行形状", "闭合围栏代码块里的行首「草稿目录：」"), agent, digest)
        values = lib.scratch_line_values(prompt)
        new_dirs = lib.scratch_directories(prompt)
        if len(values) >= 2:
            note("新库认出两个以上的「草稿目录：」值", agent, digest)
        if len(new_dirs) >= 2:
            note("新库认出两个以上工作根下的草稿目录（⑯ 各建一把锁、⑰ 都当本提示的）", agent, digest)
        # ⑯ 改前改后
        old16, new16 = orig_dispatch.scratch_directories_named(prompt), cur_dispatch.scratch_directories_named(prompt)
        if old16 != new16:
            note(("⑯ 建锁的目录改前改后不同", "改前有、改后没有" if set(old16) - set(new16) else "改后多出", ), agent, digest)
        # ⑰ 改前改后
        old17, new17 = orig_dispatch.report_scratch_directories(prompt), cur_dispatch.report_scratch_directories(prompt)
        if old17 != new17:
            note(("⑰ 认的本提示草稿目录改前改后不同", "改前有、改后没有" if set(old17) - set(new17) else "改后多出"), agent, digest)
        # 完工清理闸改前改后（只比展开后落在工作根下的值；运行时还要目录在、归属对）
        old_cc = [expand(match.group(1)) for match in orig_cleanup.SCRATCH_LINE_PATTERN.finditer(prompt)]
        new_cc = [expand(value) for value in values]
        old_cc, new_cc = [p for p in dict.fromkeys(old_cc) if under(p)], [p for p in dict.fromkeys(new_cc) if under(p)]
        if old_cc != new_cc:
            glued = all(any(old.startswith(new) and old != new for new in new_cc) for old in set(old_cc) - set(new_cc))
            note(("完工清理闸认的草稿目录改前改后不同", "改前把路径后面的字粘进路径、改后截在标点（目录本身相同）" if glued and set(new_cc) - set(old_cc) <= set() or glued and new_cc else
                  ("改前有、改后没有" if set(old_cc) - set(new_cc) else "改后多出")), agent, digest)
        # 交回闸 ⑥ 改前改后（只比展开后落在工作根下的）
        old6 = [p for p in dict.fromkeys(expand(v) for v in orig_handback.prompt_label_paths(prompt, "草稿目录")) if under(p)]
        new6 = cur_handback.prompt_scratch_directories(prompt)
        if old6 != new6:
            note(("交回闸 ⑥ 认的草稿目录改前改后不同", "改前有、改后没有" if set(old6) - set(new6) else "改后多出"), agent, digest)
        # 新库与 scripts/ 下另三种认法（只比展开后落在工作根下的第一个；看门狗是一组）
        first = new_dirs[0] if new_dirs else None
        if first is None and any(under(expand(v)) for v in (m.group(1) for m in orig_dispatch.REPORT_SCRATCH_MENTION.finditer(prompt))):
            note(("没有行首「草稿目录：」一行、却在别处写了工作根下的草稿目录（⑯ 不建锁、⑰ 只判裸文件名、完工清理闸不判它）",), agent, digest)
        draft = skeleton.DRAFT_LINE.search(prompt)
        skeleton_dir = expand(draft.group(1).strip("`'\"")) if draft else None
        skeleton_dir = skeleton_dir if skeleton_dir and under(skeleton_dir) else None
        if skeleton_dir != first:
            note(("骨架 --check 的 DRAFT_LINE 与新库认的第一个草稿目录不同", "骨架有、新库没有" if first is None else ("新库有、骨架没有" if skeleton_dir is None else "两边都有、不是同一个")), agent, digest)
        cost = COST_DRAFT.search(prompt)
        cost_dir = expand(cost.group(1)) if cost else None
        cost_dir = cost_dir if cost_dir and under(cost_dir) else None
        if cost_dir != first:
            note(("账表 agent-cost.py 的 draft 与新库认的第一个草稿目录不同", "账表有、新库没有" if first is None else ("新库有、账表没有" if cost_dir is None else "两边都有、不是同一个")), agent, digest)
        watch = {expand(m) for m in WATCH_MENTION.findall(prompt)}
        if first is not None and not any(path == first or path.startswith(first + "/") for path in watch):
            note(("看门狗 progress_file_of 的候选里没有新库认的草稿目录",), agent, digest)
        if first is not None and any(not (path == first or path.startswith(first + "/")) for path in watch):
            note(("看门狗 progress_file_of 的候选里另有新库认的草稿目录之外的路径（取改得最晚的 progress.md）",), agent, digest)

print(f"语料：{total_agents} 份记录、{len(total_prompts)} 份不同提示；提示里出现「草稿目录」的 {len(with_label)} 份不同提示")
for key in sorted(per_agent, key=lambda k: str(k)):
    label = key if isinstance(key, str) else "：".join(key)
    split = "、".join(f"{when} {count}" for when, count in sorted(periods[key].items()))
    print(f"{label}：{per_agent[key]} 份记录、{len(per_prompt[key])} 份不同提示（按派发日期：{split}）；例：{'、'.join(examples[key])}")
