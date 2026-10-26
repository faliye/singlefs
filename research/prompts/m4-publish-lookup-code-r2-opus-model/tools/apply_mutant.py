"""apply_mutant.py <变异表> <名字> <树根>：按名字从变异表取一行（名字、文件、旧串、新串，制表符分隔，\\n 记换行），
旧串在 <树根>/<文件> 里恰好命中一次才替换，否则退 1。"""
import pathlib
import sys

table, name, root = sys.argv[1], sys.argv[2], pathlib.Path(sys.argv[3])
rows = [line.rstrip("\n").split("\t") for line in pathlib.Path(table).read_text().splitlines() if line and not line.startswith("#")]
matching = [row for row in rows if row[0] == name]
if len(matching) != 1:
    sys.exit(f"变异表里名字 {name} 命中 {len(matching)} 行")
_, file, old, new = matching[0][:4]
old, new = old.replace("\\n", "\n"), new.replace("\\n", "\n")
path = root / file
text = path.read_text()
count = text.count(old)
if count != 1:
    sys.exit(f"{name}：旧串在 {path} 命中 {count} 次")
path.write_text(text.replace(old, new))
print(f"applied {name} {file}")
