"""e171-conclusion-r1 攻方腿 G4：计入分母的 37 行各自被哪条测试抓（crates/mutations.tsv 第 5、6 段），按测试目标分类，与 E171 三分交叉。
只读：冻结副本的 crates/mutations.tsv、两张行表、比对器产物里的 name=row 与 Q1 / Q3 名单。
用法：python3 killer_tests_of_counted_rows.py <冻结树根> <比对器产物>
"""
import re
import sys

root, comparator = sys.argv[1], sys.argv[2]
table_dir = f"{root}/research/e7-index-bench/src/bin/e171_three_oracle_collusion"
names = {}
for path in ("bad-implementations-117.tsv", "bad-implementations-expansion-164.tsv"):
    for line in open(f"{table_dir}/{path}", encoding="utf-8"):
        c = line.rstrip("\n").split("\t")
        if c[0].isdigit():
            names[int(c[0])] = (c[2], c[4])
mutations = {}
for line in open(f"{root}/crates/mutations.tsv", encoding="utf-8"):
    if line.startswith("#") or not line.strip():
        continue
    c = line.rstrip("\n").split("\t")
    mutations.setdefault((c[1], c[0]), []).append((c[4], c[5]))
text = open(comparator, encoding="utf-8").read()
counted = sorted(int(m) for m in re.findall(r"name=row row=(\d+) row_name=\S+ category=counted", text))
quiet = set(int(x) for x in re.search(r"name=q1_gamma .*?row_numbers=([\d,]+)", text).group(1).split(","))
q3 = re.search(r"name=q3_beta caught_rows=\d+ only_O1=([\d,]+) only_O2=([\d,]+)", text)
only_o1 = set(int(x) for x in q3.group(1).split(","))
only_o2 = set(int(x) for x in q3.group(2).split(","))


def kind(args: str) -> str:
    m = re.search(r"--test (\S+)", args)
    target = m.group(1) if m else ("lib:" + args.split()[1] if "--lib" in args else args)
    if target == "checker_known_bad_images":
        return "O2-machinery(checker_known_bad_images)"
    if "random_histories" in target or "model" in target:
        return "O1-machinery(model)"
    if "-p singlefs-checker-tier" in args:
        return "checker-tier:" + target
    if "--lib" in args:
        return "unit:" + args.split()[1]
    return "harness:" + target


rows = []
for r in counted:
    file, name = names[r]
    found = mutations.get((file, name), [])
    k = kind(found[0][0]) if len(found) == 1 else f"lookup={len(found)}"
    verdict = "all_quiet(gamma)" if r in quiet else ("only_O1" if r in only_o1 else ("only_O2" if r in only_o2 else "two_or_more"))
    rows.append((r, verdict, k))
    print(f"row={r}\t{verdict}\t{k}")
from collections import Counter  # noqa: E402
print("name=killer_summary " + " ".join(f"{v}|{k.split(':')[0].split('(')[0]}={n}" for (v, k), n in
                                         sorted(Counter((v, k) for _, v, k in rows).items())))
