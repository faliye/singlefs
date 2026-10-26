import json, glob, os, re, sys, collections, datetime
ROOT = os.environ.get("SESSIONS_ROOT") or os.path.expanduser("~/.claude/projects/-home-fy5090-code-singlefs")
since = datetime.datetime(2026, 10, 1).timestamp()
NEW = re.compile(r"修(?!订)|重跑|重派|再派|重做|返工|续做|续派|接续|接手|接着做|（续）|补写|补做|改名|改号|改钉值|第\s*[2-9]\d*\s*次|redispatch|re-?run|renumber|retry|\bfix", re.I)
OLD = re.compile(r"修|重跑|重派|再派|重做|返工|续做|续派|接续|接手|接着做|（续）|补写|补做|改名|改号|改钉值|第\s*[2-9]\d*\s*次|redispatch|re-?run|renumber|retry|\bfix", re.I)
rows = []
for p in glob.glob(ROOT + "/*/subagents/*.meta.json"):
    try:
        t = os.path.getmtime(p)
        if t < since: continue
        d = json.load(open(p)).get("description", "")
    except Exception:
        continue
    rows.append(d)
print("总数", len(rows))
o = [d for d in rows if OLD.search(d)]
n = [d for d in rows if NEW.search(d)]
print("旧(含修订)命中", len(o), "新命中", len(n), "差", len(o) - len(n))
tok = collections.Counter()
weak_only = []
for d in n:
    ms = [m.group(0).lower() for m in NEW.finditer(d)]
    for m in set(ms): tok[m] += 1
    # 弱词：单字修后接改/复/正/补 之外？ 这里先列 fix* 词形
print(tok.most_common(30))
# 以 fixture / fixes / fixed / fixing 等命中的
fx = [d for d in n if re.search(r"\bfix", d, re.I) and not re.search(r"\bfix(?:es|ed|ing)?\b", d, re.I)]
print("仅靠 fix 词头(非 fix/fixes/fixed/fixing 独立词，如 fixture)的", len(fx))
for d in fx[:15]: print("  ", d)
# 只靠「修」且后字不是 补/红/复/正/掉 的
only_xiu = [d for d in n if NEW.search(d) and all(m.group(0) == "修" for m in NEW.finditer(d))]
print("只靠单字「修」命中", len(only_xiu))
c = collections.Counter(re.search(r"修(.?)", d).group(1) for d in only_xiu)
print(c.most_common(20))
for d in only_xiu[:40]: print("  ", d)
print("=====")
strong = [d for d in n if any(m.group(0).lower() not in ("修", "fix") for m in NEW.finditer(d))]
weak_only = [d for d in n if all(m.group(0).lower() in ("修", "fix") for m in NEW.finditer(d))]
print("新词表命中", len(n), "其中含至少一个非「修」非 fix 的词", len(strong), "只靠「修」或 fix", len(weak_only))
print("占比", f"{len(weak_only)/len(n):.1%}", " 总数", len(rows), f"新命中占总 {len(n)/len(rows):.1%} 强词命中占总 {len(strong)/len(rows):.1%}")
