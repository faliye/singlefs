import sys
table = "/home/fy5090/code/singlefs/.claude/gate.d/e165-pinned-red-examples.tsv"
rows = []
for raw in open(table, encoding="utf-8").read().split("\n"):
    if raw.strip() and not raw.startswith("#"):
        f = raw.split("\t"); rows.append((f[1], f[2], f[6]))
base = rows[0][2]
adv = [
 "E7RESULT name=state domain=X n=2 counters=a",
 "E7RESULT name=state domain=X n=2 counters=a state=",
 "E7RESULT name=state domain=X n=2 counters=a state=x y",
 "E7RESULT name=state domain=X state=a state=b",
 "E7RESULT name=state domain=X state=a b state=c",
 "E7RESULT name=state domain=X state=;first=r",
 "E7RESULT name=state domain=X state=s;first=r;first=q",
 "E7RESULT name=state domain=X state=s;expected=e",
 "state=s",
 "E7RESULT name=state domain=X state=s\\r",
 "E7RESULT name=state domain=X state=s\\tz",
 "E7RESULT name=state domain=X state=s ",
 "E7RESULT name=state domain=X state= s",
 "E7RESULT name=state domain=X state=s　",
 "E7RESULT name=state domain=X state=一二三;first=四",
 "E7RESULT name=done emitted=3",
 "E7RESULT name=state domain=X state=a;first=b state=c;first=d",
]
out = [f"{d}\t{n}\t{r}" for d, n, r in rows] + [f"X\t2\t{a}" for a in adv]
open("/tmp/claude-1000/m3-verus-land-code-r2-sonnet/h6/rows.tsv", "w", encoding="utf-8").write("\n".join(out) + "\n")
print(len(rows), len(adv))
