#!/usr/bin/env python3
"""S5 试改（辩方腿提的收严，只在本腿模型上量过）：原话找不到而取行号提示那一行时，再要求提示那一行与原话共有一段连续文字，长度不短于原话的六成。
在 <树> 的拷贝上改 lib_abandoned_rounds.py（不碰冻结副本），再用 s5_drift_matrix.py 的全部例子对比改前改后，只列结论变了的。
用法：s5_similarity_trial.py <b10 树> <工作目录> <s5_drift_matrix.py>"""
import os, shutil, subprocess, sys
tree, work, matrix = (os.path.abspath(p) for p in sys.argv[1:4])
patched = os.path.join(work, "patched"); shutil.rmtree(patched, ignore_errors=True); os.makedirs(patched)
shutil.copytree(os.path.join(tree, ".claude"), os.path.join(patched, ".claude"))
path = os.path.join(patched, ".claude/hooks/lib_abandoned_rounds.py")
text = open(path, encoding="utf-8").read()
old = '''        if not hits and hint_usable and name_in(cited_lines[line_number - 1], round_name):
            hits, hint_reason = [line_number], "原话找不到，行号提示那一行还有这一轮的名字，取那一行"'''
new = '''        if not hits and hint_usable and name_in(cited_lines[line_number - 1], round_name):
            import difflib
            _hint_line = cited_lines[line_number - 1]
            _match = difflib.SequenceMatcher(None, excerpt, _hint_line, autojunk=False).find_longest_match(0, len(excerpt), 0, len(_hint_line))
            if _match.size >= 0.6 * len(excerpt):   # 试改：提示那一行要和原话共有一段不短于原话六成的连续文字，才照提示取
                hits, hint_reason = [line_number], "原话找不到，行号提示那一行还有这一轮的名字，取那一行"'''
assert text.count(old) == 1
open(path, "w", encoding="utf-8").write(text.replace(old, new))
done = subprocess.run([sys.executable, "-B", matrix, f"b10={tree}", f"b10-similarity={patched}", "--work", os.path.join(work, "m")], capture_output=True, text=True)
rows = [l.split("\t", 3) for l in done.stdout.strip().split("\n")]
by_case = {}
for case, label, code, first in rows:
    by_case.setdefault(case, {})[label] = (code, first.split("\t")[-1][:70])
changed = [c for c, v in by_case.items() if v["b10"][0] != v["b10-similarity"][0]]
print(f"{len(by_case)} 个例子里结论变了的 {len(changed)} 个（其余 {len(by_case) - len(changed)} 个改前改后相同）：")
for c in changed:
    print(f"  {c}\tb10 {by_case[c]['b10'][0]}\t试改 {by_case[c]['b10-similarity'][0]}")
unchanged_accept = [c for c, v in by_case.items() if v["b10"][0] == v["b10-similarity"][0] == "退出=0"]
print(f"改前改后都收的 {len(unchanged_accept)} 个：" + "；".join(unchanged_accept))
