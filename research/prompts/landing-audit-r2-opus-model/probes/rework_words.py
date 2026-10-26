import json, re, sys, collections
agents = [json.loads(l) for l in open(sys.argv[1])]
total = sum(a["cost"] for a in agents)
W = re.compile(r"修(?!订)|重跑|重派|再派|重做|返工|续做|续派|接续|接手|接着做|（续）|补写|补做|改名|改号|改钉值|第\s*[2-9]\d*\s*次|redispatch|re-?run|renumber|retry|\bfix", re.I)
hit = [a for a in agents if a["rework"]]
print("返工字样", len(hit), f"{sum(a['cost'] for a in hit)/total:.1%}")
first = collections.Counter()
for a in hit:
    first[W.search(a["description"]).group(0).lower()] += 1
print("命中词（每个派发取第一个命中）:", first.most_common())
# 只因「修」命中、且「修」后接的字
xiu = collections.Counter()
xiu_only = []
for a in hit:
    words = {m.group(0).lower() for m in W.finditer(a["description"])}
    if words == {"修"}:
        xiu_only.append(a)
        for m in re.finditer(r"修(?!订)(.)?", a["description"]):
            xiu[(m.group(1) or "（行尾）")] += 1
print("只因单字「修」命中:", len(xiu_only), f"{sum(a['cost'] for a in xiu_only)/total:.1%}", "后接的字:", xiu.most_common(15))
# 没命中而带别的接续字样的
OTHER = re.compile(r"续|接着|继续|再跑|重新|重试|补跑|补完|again|二次|第二段|下一段|收尾|接力|接上")
miss = [a for a in agents if not a["rework"] and OTHER.search(a["description"])]
mc = collections.Counter(OTHER.search(a["description"]).group(0) for a in miss)
print("没命中而带接续类字样:", len(miss), f"{sum(a['cost'] for a in miss)/total:.1%}", mc.most_common())
for a in miss[:12]: print("   漏?", a["type"], a["description"][:60])
for a in xiu_only[:12]: print("   误?", a["type"], a["description"][:60])
