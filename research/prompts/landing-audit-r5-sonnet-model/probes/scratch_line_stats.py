"""「草稿目录：」一行收成行首一种认法之后，真派发提示里哪些丢了、哪些没丢。
用法：python3 -I scratch_line_stats.py <抽出来的目录> <冻结副本>   （读 <冻结副本>/.claude/hooks 下的 lib_prompt_scratch_line.py 与 lib_work_root.py）
对照的旧认法是第九批的两处正则（逐字抄自 inputs/pre-b10-hooks/runner-dispatch-guard.sh 的 WRITER_LOCK_DIRECTORY_LINE 与 REPORT_SCRATCH_MENTION）。"""
import importlib.util, json, os, re, sys, collections
out, frozen = sys.argv[1], sys.argv[2]
def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module
hooks = frozen + "/.claude/hooks/"
lps, wr = load("lps", hooks + "lib_prompt_scratch_line.py"), load("wr", hooks + "lib_work_root.py")
roots = [r.rstrip("/") for r in wr.scratch_roots()]
PREFIX = wr.mention_prefix_pattern()
PATH = r"[^\s`'\"，。；：、（）()「」『』“”,;*<>]*"
OLD16 = re.compile(r"^[ \t>*_\-]*草稿目录[* \t]*[：:][* \t]*`?(" + PREFIX + PATH + ")", re.M)
OLD17 = re.compile(r"(?<![^\W_])草稿目录[* \t]*[：:]?[* \t]*`?(" + PREFIX + PATH + ")")
MARK = re.compile(r"^[ \t]*(?:[>*_\-]+[ \t]*|\d+[.)][ \t]*)\**草稿目录\**[ \t]*[：:]", re.M)
def under(value):
    path = os.path.normpath(os.path.abspath(wr.expand_written_path(value)))
    return path if any(path.startswith(root + "/") for root in roots) else None
def found(pattern, text):
    result = []
    for match in pattern.finditer(text):
        path = under(match.group(1)) if match.group(1) else None
        if path and path not in result:
            result.append(path)
    return result
def without_fences(text):
    lines = text.split("\n")
    covered = lps.fenced_line_numbers(lines)
    return "\n".join(line for number, line in enumerate(lines) if number not in covered)
subagents = json.load(open(out + "/subagent_prompts.json"))
main = json.load(open(out + "/main_prompts.json"))
def day(item): return item["ts"][:10]
for start, label in (("2026-10-14", "⑯ 进派发闸之后"), ("2026-10-15", "骨架普及之后")):
    sel = [r for r in subagents if day(r) >= start]
    new_none = [r for r in sel if not lps.scratch_directories(r["prompt"])]
    lost16 = [r for r in sel if set(found(OLD16, r["prompt"])) - set(lps.scratch_directories(r["prompt"]))]
    lost17 = [r for r in sel if set(found(OLD17, without_fences(r["prompt"]))) - set(lps.scratch_directories(r["prompt"]))]
    marked = [r for r in sel if MARK.search(r["prompt"])]
    print(f"name=subagent_first_messages {label}（{start} 起） 共={len(sel)} 新认法认不出草稿目录={len(new_none)} ⑯丢锁={len(lost16)} ⑰旧认法有而新认法没有={len(lost17)} 行首带记号的草稿目录行={len(marked)}")
every_marked = [r for r in subagents if MARK.search(r["prompt"])]
print(f"name=marked_label_lines 全部={len(subagents)} 行首带列表/引用/粗体记号的「草稿目录：」行={len(every_marked)} 最晚一份的日期={max((r['ts'][:10] for r in every_marked), default='无')}")
real = [r for r in main if r["ts"][:10] >= "2026-10-15" and r["type"] not in (None, "general-purpose", "Explore")]
nolabel = [r for r in real if not lps.scratch_directories(r["prompt"] or "")]
print(f"name=main_session_dispatches 2026-10-15 起 共={len(real)} 认出规范行={len(real) - len(nolabel)} 认不出={len(nolabel)}（其中提示里连「草稿目录」四个字都没有={sum(1 for r in nolabel if '草稿目录' not in (r['prompt'] or ''))}）")
bad = [r for r in real if MARK.search(r["prompt"] or "")]
print(f"name=main_session_marked 行首带记号的草稿目录行={len(bad)}")
skeleton = [r for r in subagents if r["prompt"].startswith("# 派发：")]
print(f"name=skeleton_prompts 骨架头「# 派发：」起头的={len(skeleton)} 其中新认法认不出={sum(1 for r in skeleton if not lps.scratch_directories(r['prompt']))}")
