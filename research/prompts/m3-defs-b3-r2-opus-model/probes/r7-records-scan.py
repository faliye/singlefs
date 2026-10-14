# R7 计数：冻结副本 records/ 下每一行，按派发闸 ⑥ 的认法（轮名前后不连着 A-Za-z0-9_-、同一行有「」括的去掉空白后至少 6 个字的话）能当出处的有几行、几个轮名；
# 其中这一行里没有「叫停」「停掉」「停了」「撂下」「不判」这几个词的又有几行、几个轮名。只打计数，不打行文。
# 调度记录 records/2026-10-12-里程碑三工具与治理批.md 在这一轮的禁读清单里，跳过不读。
# 用法：python3 -I r7-records-scan.py <冻结副本>
import os, re, sys
root = sys.argv[1]
SKIPPED = {"2026-10-12-里程碑三工具与治理批.md"}
ROUND = re.compile(r"(?<![A-Za-z0-9_\-])([a-z0-9][a-z0-9\-]*-r[0-9]+)(?![A-Za-z0-9_\-])")
QUOTE = re.compile(r"「([^「」\n]+)」")
HALT = re.compile(r"叫停|停掉|停了|撂下|不判")
lines = names = 0
lines_no_halt = 0
names_all, names_no_halt_only = set(), set()
names_with_halt = set()
files = 0
for name in sorted(os.listdir(os.path.join(root, "records"))):
    if not name.endswith(".md") or name in SKIPPED:
        continue
    files += 1
    for text in open(os.path.join(root, "records", name), encoding="utf-8", errors="replace").read().split("\n"):
        rounds = set(ROUND.findall(text))
        if not rounds or not any(len(re.sub(r"\s+", "", q)) >= 6 for q in QUOTE.findall(text)):
            continue
        lines += 1
        names_all |= rounds
        if HALT.search(text):
            names_with_halt |= rounds
        else:
            lines_no_halt += 1
            names_no_halt_only |= rounds
print(f"name=r7-scan records_files={files} source_lines={lines} round_names={len(names_all)}")
print(f"name=r7-scan source_lines_without_halt_words={lines_no_halt} round_names_only_in_lines_without_halt_words={len(names_no_halt_only - names_with_halt)}")
