#!/usr/bin/env python3
"""apply-mutation.py <copy-root> <relative-file> <old-file> <new-file>: replace old text (exactly once) with new text; \\n in the
text files is literal (the files hold real text)."""
import sys, pathlib, os
root, rel, old_path, new_path = sys.argv[1:5]
old = pathlib.Path(old_path).read_text(encoding='utf-8')
new = pathlib.Path(new_path).read_text(encoding='utf-8')
p = pathlib.Path(root) / rel
s = p.read_text(encoding='utf-8')
n = s.count(old)
if n != 1:
    sys.exit(f"{rel}: anchor count {n}")
p.write_text(s.replace(old, new), encoding='utf-8')
os.utime(p, None)
print(f"mutated {rel}")
