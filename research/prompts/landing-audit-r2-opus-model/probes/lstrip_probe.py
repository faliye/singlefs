"""并成 lib-lag-baseline.py 之前四处切表行的写法（原样抄自开工快照 judged-vs-index.diff 里删掉的行）与 content_rows 逐行比。
用法：python3 lstrip_probe.py <仓根>"""
import importlib.util, os, sys

root = sys.argv[1]
spec = importlib.util.spec_from_file_location("lag_baseline", os.path.join(root, ".claude/gate.d/lib-lag-baseline.py"))
library = importlib.util.module_from_spec(spec)
spec.loader.exec_module(library)

def multipath_lag_rows_before(text):            # doc-experiments.sh multipath-registry 格改前的 lag_rows
    rows = []
    for line_number, line in enumerate(text.split('\n'), 1):
        if line.strip() and not line.startswith('#'):
            rows.append((line_number, line.rstrip('\r').split('\t')))
    return rows

def units_rows_of_before(text):                 # doc-experiments.sh replay-comparison-units 格改前的 rows_of
    rows = []
    for number, line in enumerate(text.split("\n"), 1):
        line = line.rstrip("\r")
        if line.strip() and not line.startswith("#"):
            rows.append((number, line.split("\t")))
    return rows

def mutation_parse_rows_before(text):           # code-source-discipline.sh mutation-tables 格改前 parse_rows 的切行
    rows = []
    for line_number, line in enumerate(text.split("\n"), 1):
        if not line.strip() or line.startswith("#"):
            continue
        rows.append((line_number, line.split("\t")))
    return rows

def reading_read_lag_before(text):              # code-source-discipline.sh reading-discipline 格改前 read_lag 的切行
    rows = []
    for line_number, line in enumerate(text.split("\n"), 1):
        if not line.strip() or line.startswith("#"):
            continue
        rows.append((line_number, line.split("\t")))
    return rows

def records_rows_of_before(text):               # doc-process-records.sh 挂账表改前的 rows_of（本来就 lstrip）
    return [(number, raw.split("\t")) for number, raw in enumerate(text.split("\n"), 1) if raw.strip() and not raw.lstrip().startswith("#")]

table = "甲\t理由一\n  # 缩进的注释\n\t#开头的第二列\t理由二\n乙\t理由三\n"
after = library.content_rows(table)
print(f"name=content_rows rows={[number for number, _ in after]}")
for name, before in (("multipath-registry", multipath_lag_rows_before), ("replay-comparison-units", units_rows_of_before),
                     ("mutation-tables", mutation_parse_rows_before), ("reading-discipline", reading_read_lag_before),
                     ("doc-process-records", records_rows_of_before)):
    rows = before(table)
    print(f"name={name} before_rows={[number for number, _ in rows]} after_rows={[number for number, _ in after]} same={[n for n, _ in rows] == [n for n, _ in after]}")
