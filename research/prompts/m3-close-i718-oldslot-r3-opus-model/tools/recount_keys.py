"""独立重数第二轮辩方的「33 个键分类不变」：从 out/raw 的 ROW 行自己解析，不用辩方的 summary.py。
键 = (格, 镜像名把 n<数字> 换成 n*, 种类)；每个 S、每个候选把这个键下的判定归成四类之一：全成立 H、全判红 V、全不适用 N、混合 M。
打印：各 S 的键数、四个 S 都有的键数、其中有没有键在 S 之间变类（逐候选比），变类的逐个列出。"""
import collections, re, sys, glob
rows = collections.defaultdict(lambda: collections.defaultdict(list))
for path in sorted(glob.glob(sys.argv[1] + "/*.out")):
    for line in open(path, encoding="utf-8", errors="replace"):
        if not line.startswith("ROW "):
            continue
        fields = dict(re.findall(r"(\w+)=(\S+)", line))
        m = re.search(r" all=(\S+)", line)
        if not m or m.group(1) == "none":
            continue
        values = tuple(int(x) for x in m.group(1).split(","))
        key = (fields["cell"], re.sub(r"n\d+", "n*", fields["image"]), fields["kind"])
        rows[key][int(fields["S"])].append(values)
def klass(vectors, column):
    seen = {v[column] for v in vectors}
    if seen == {1}: return "H"
    if seen == {0}: return "V"
    if seen == {-1}: return "N"
    return "M"
per_s = collections.Counter(s for key in rows for s in rows[key])
print("keys_per_S", dict(sorted(per_s.items())), "keys_total", len(rows))
summary_like = [k for k in rows if any(w in k[1] for w in ("zero", "healthy_n", "before_root", "oldest", "middle", "bad_p_255", "bad_t_as_floor"))]
print("keys_matching_the_summary_filter", len(summary_like))
all_four = [k for k in rows if set(rows[k]) >= {4, 5, 8, 16}]
print("keys_present_at_S_4_5_8_16", len(all_four), "of_which_matching_summary_filter", len([k for k in all_four if k in summary_like]))
changed = []
for key in sorted(rows):
    sizes = sorted(rows[key])
    if len(sizes) < 2:
        continue
    for column in range(11):
        classes = {s: klass(rows[key][s], column) for s in sizes}
        if len(set(classes.values())) > 1:
            changed.append((key, column, classes))
print("key_candidate_pairs_changing_class_across_S", len(changed))
for key, column, classes in changed:
    print("  changed", key, "column", column, classes)
print("total_ROW_lines", sum(len(v) for k in rows for v in rows[k].values()))
