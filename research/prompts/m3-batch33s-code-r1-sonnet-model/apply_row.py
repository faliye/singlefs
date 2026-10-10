# usage: apply_row.py <table> <row-no (1-based)> <repo-root> <pristine-root> apply|restore
import sys, shutil, os
table, rowno, repo, pristine, mode = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4], sys.argv[5]
rows = open(table, encoding='utf-8').read().split('\n')
row = rows[rowno-1].split('\t')
name, path, old, new = row[0], row[1], row[2].replace('\\n', '\n'), row[3].replace('\\n', '\n')
target = os.path.join(repo, path)
if mode == 'restore':
    shutil.copyfile(os.path.join(pristine, path), target)
    print("restored", path)
    sys.exit(0)
src = open(target, encoding='utf-8').read()
n = src.count(old)
if n != 1:
    print("ANCHOR HITS", n); sys.exit(3)
open(target, 'w', encoding='utf-8').write(src.replace(old, new))
print("applied", name[:50], "->", path)
print("CARGO", row[4]); print("MUST", row[5])
