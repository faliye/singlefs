# 照 .claude/gate.d/59-crates-mutation-replay.sh 的预扫：逐行解析六段、\n 当换行，原文在第 2 段那个文件里数命中；只数，不跑变异。
import collections, os, sys
root = sys.argv[1]
table = "crates/mutations.tsv"
def unescape(text): return text.replace("\\n", "\n")
counts = []
comment_lines = 0
with open(os.path.join(root, table), encoding="utf-8") as handle:
    for line_number, line in enumerate(handle, 1):
        line = line.rstrip("\n")
        if not line or line.startswith("#"):
            comment_lines += 1
            continue
        fields = line.split("\t")
        assert len(fields) == 6 and all(fields), f"第 {line_number} 行不是六段"
        full = os.path.join(root, fields[1])
        count = open(full, encoding="utf-8").read().count(unescape(fields[2])) if os.path.isfile(full) else "文件不存在"
        counts.append((line_number, fields[0], count))
for line_number, name, count in counts:
    print(f"{line_number}\t{count}\t{name}")
print(f"表共 {line_number} 行：注释行 {comment_lines}、变异 {len(counts)} 条；命中次数分布 {dict(collections.Counter(count for _, _, count in counts))}")
bad = [(line_number, name, count) for line_number, name, count in counts if count != 1]
print("命中不是 1 次的：", bad if bad else "无")
