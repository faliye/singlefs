"""在 <树>/.claude/hooks/runner-dispatch-guard.sh 上做攻方提的改法（只在草稿树里，被攻过零轮）：
⑰ 的本提示草稿目录先照新库认行首那一行；一行都没有时退回第九批 ⑰ 那一种（句中、冒号可有可无、不在引号里）。
用法：python3 -I make_fix.py <树>"""
import os, sys
path = os.path.join(sys.argv[1], ".claude", "hooks", "runner-dispatch-guard.sh")
text = open(path, encoding="utf-8").read()
old = '''def report_scratch_directories(prompt):
    """⑰ 认的本提示草稿目录：行首「草稿目录：」一行点名的（prompt_scratch_directories）。"""
    return prompt_scratch_directories(prompt)
'''
new = '''REPORT_SCRATCH_MENTION_FALLBACK = re.compile(r"(?<![^\\W_])草稿目录[* \\t]*[：:]?[* \\t]*`?(" + SCRATCH_MENTION_PREFIX + r"[^\\s`'\\"，。；：、（）()「」『』“”,;*<>]*)")

def report_scratch_directories(prompt):
    """⑰ 认的本提示草稿目录：行首「草稿目录：」一行点名的（prompt_scratch_directories）；一行都没有时退回第九批 ⑰ 的认法（攻方 landing-audit-r5 的改法，被攻过零轮）。"""
    found = prompt_scratch_directories(prompt)
    if found:
        return found
    judged = without_code_blocks(prompt)
    spans = quoted_spans(judged)
    roots = [root.rstrip("/") for root in (work_roots.scratch_roots() if work_roots else ())]
    for match in REPORT_SCRATCH_MENTION_FALLBACK.finditer(judged):
        if inside_spans(match.start(), spans):
            continue
        path = os.path.normpath(expanded_mention(match.group(1)))
        if any(path.startswith(root + "/") for root in roots) and path not in found:
            found.append(path)
    return found
'''
assert text.count(old) == 1
temporary = path + ".new"
open(temporary, "x", encoding="utf-8").write(text.replace(old, new))
os.replace(temporary, path)
print("改好了：", path)
