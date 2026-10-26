"""几个现查的事实：比对对象表与登记表逐行对（删掉文件头五行会怎样）、看门狗峰值表有没有峰值为 0 的条目、工作区成员是不是都在 checker 档包的依赖闭包里、
scripts/ 下有几份脚本读 .claude/gate.d/ 里的 tsv。用法：python3 -I misc_facts.py <仓根> <峰值表>"""
import glob, os, re, sys
repo, peaks = sys.argv[1], sys.argv[2]
lines = open(repo + "/research/prompts/e165-r5-prereg.md", encoding="utf-8").read().split("\n")
header = next(i for i, l in enumerate(lines) if l.startswith("| # | 比对对象 |"))
cols = [c.strip() for c in lines[header].strip().strip("|").split("|")]
rows, i = [], header + 2
while i < len(lines) and lines[i].startswith("|"):
    rows.append([c.strip() for c in lines[i].strip().strip("|").split("|")]); i += 1
object_index, rerun_index = cols.index("比对对象"), next(k for k, c in enumerate(cols) if "重跑产物名" in c)
products = [l.split("\t")[4] for l in open(repo + "/.claude/gate.d/replay-comparison-units.tsv", encoding="utf-8").read().split("\n") if l.strip() and not l.startswith("#")]
unmatched = [r[0] for r in rows if not any(p in (r[object_index].strip("`"), r[rerun_index].strip("`")) or p.endswith("-" + r[rerun_index].strip("`")) for p in products)]
named = re.findall(r"^# 不登记的比对对象：research/prompts/e165-r5-prereg\.md #(\d+)：", open(repo + "/.claude/gate.d/replay-comparison-units.tsv", encoding="utf-8").read(), re.M)
print(f"name=comparison_table 表行={len(rows)} 登记行={len(products)} 表里对不上登记的行号={unmatched} 文件头点名不登记的行号={named}")
values = [int(l.split("\t")[0]) for l in open(peaks, encoding="utf-8", errors="replace") if l.strip() and not l.startswith("#") and l.split("\t")[0].isdigit()]
print(f"name=memory_peaks 最小={min(values)}（字节） 峰值为0的条数={sum(1 for v in values if v == 0)}（表在长，条数不打，免得复跑对不上）")
manifest = open(repo + "/Cargo.toml", encoding="utf-8").read()
members = re.findall(r'"(crates/[^"]+)"', manifest.split("members = [")[1].split("]")[0])
dependencies = set(re.findall(r'path = "\.\./([^"]+)"', open(repo + "/crates/singlefs-checker-tier/Cargo.toml", encoding="utf-8").read()))
outside = [m for m in members if m.split("/")[-1] not in dependencies and m.split("/")[-1] != "singlefs-checker-tier"]
print(f"name=workspace_closure 工作区成员={len(members)} 依赖闭包外的成员={outside}")
readers = {}
for path in glob.glob(repo + "/scripts/**/*", recursive=True):
    if os.path.isfile(path) and path.endswith((".py", ".sh")):
        for table in re.findall(r"\.claude/gate\.d/[A-Za-z0-9_.-]+\.tsv", open(path, encoding="utf-8", errors="replace").read()):
            readers.setdefault(table, set()).add(path)
print(f"name=gate_tsv_readers scripts/ 下读 .claude/gate.d/*.tsv 的脚本={len(set().union(*readers.values()))} 涉及的表={len(readers)}")
