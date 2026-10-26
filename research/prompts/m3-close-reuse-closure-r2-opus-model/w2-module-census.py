"""W2：一个入口的闭包里每一份有单元进来的 python 文件（这一份在判法跑时被 import、按路径 exec_module 或当程序起，模块级语句都执行），
数它模块级的 import、assign、class 单元里不在闭包的个数，按「这一份怎么进来的」（import / load / program / inline 之外的别的）分开报。
用法：w2-module-census.py <closure --explain 原样输出> <dump --language python 原样输出> <标签>"""
import collections
import sys

closure_path, dump_path, label = sys.argv[1:4]
members, vias = set(), collections.defaultdict(set)
for line in open(closure_path, encoding="utf-8"):
    fields = line.rstrip("\n").split("\t")
    if fields[0] == "via" and len(fields) >= 4:
        vias[fields[1]].add(fields[3].split(" ")[0])
    elif len(fields) >= 4 and fields[0] not in ("refused", "key"):
        members.add((fields[0], fields[1], fields[2].rsplit(":", 1)[0]))
files = sorted({file for _kind, _name, file in members if file.endswith(".py")})
dumped = collections.defaultdict(list)
for line in open(dump_path, encoding="utf-8"):
    fields = line.rstrip("\n").split("\t")
    if len(fields) >= 4 and fields[0] in ("import", "assign", "class"):
        dumped[fields[2].rsplit(":", 1)[0]].append((fields[0], fields[1], fields[2].rsplit(":", 1)[0]))
missing_total, by_way = 0, collections.Counter()
for file in files:
    missing = [row for row in dumped.get(file, []) if row not in members]
    missing_total += len(missing)
    how = ",".join(sorted(rule for rule in vias.get(f"{file}:<module-body>", set()) | vias.get(f"{file}:<main-guard>", set()))) or "by-name"
    by_way[how] += len(missing)
print(f"name=module_census entry={label} python_files_in_closure={len(files)} module_level_units_not_in_closure={missing_total} by_how_file_entered={dict(sorted(by_way.items()))}")
