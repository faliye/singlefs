"""把这一轮复跑的汇总和第二轮入库的汇总逐行比，只比这一轮跑过的格。
用法：python3 compare.py opus <这一轮汇总> <第二轮汇总>…
      python3 compare.py sonnet <这一轮汇总> <第二轮汇总>   （只比候选 pointer_slots_fixed；第二轮的例子序号是先到先得、不确定，两边都去掉）
打出：name=r3_compare <腿> same|differs rows_mine=N rows_ref=N；不同的行逐行列出（只在一边有的）。退出码 0 全同，1 有不同。"""
import re, sys

kind, mine_path, *ref_paths = sys.argv[1:]
mine = [line.rstrip("\n") for line in open(mine_path, encoding="utf-8") if line.startswith("name=c834_")]
ref = [line.rstrip("\n") for path in ref_paths for line in open(path, encoding="utf-8") if line.startswith("name=c834_")]

def strip_example(line):
    return re.sub(r" example_ordinal=\d+", "", line)

if kind == "opus":
    keys = set()
    for line in mine:
        if line.startswith("name=c834_coverage"):
            fields = dict(re.findall(r"(\w+)=(\S+)", line))
            keys.add((fields["history"], fields["faults"]))
    def wanted(line):
        fields = dict(re.findall(r"(\w+)=(\S+)", line))
        if line.startswith("name=c834_coverage"):
            return (fields["history"], fields["faults"]) in keys
        history, _, faults = fields["history"].partition("/")
        return (history, faults) in keys
    ref = [line for line in ref if wanted(line)]
else:
    prefixes = set()
    for line in mine:
        fields = dict(re.findall(r"(\w+)=(\S+)", line))
        if fields.get("candidate") == "pointer_slots_fixed":
            prefixes.add(fields["prefix"])
    def wanted(line):
        fields = dict(re.findall(r"(\w+)=(\S+)", line))
        return fields.get("candidate") == "pointer_slots_fixed" and fields["prefix"] in prefixes
    ref = [strip_example(line) for line in ref if wanted(line)]
    mine = [strip_example(line) for line in mine if "candidate=pointer_slots_fixed" in line]
only_mine = sorted(set(mine) - set(ref))
only_ref = sorted(set(ref) - set(mine))
same = not only_mine and not only_ref and len(mine) == len(ref)
print(f"name=r3_compare {kind} {'same' if same else 'differs'} rows_mine={len(mine)} rows_ref={len(ref)}")
for line in only_mine:
    print("只在这一轮：" + line)
for line in only_ref:
    print("只在第二轮：" + line)
sys.exit(0 if same else 1)
