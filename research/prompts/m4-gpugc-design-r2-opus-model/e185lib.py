"""读 E185 第 3 次重跑产物的公用解析。"""
import re

def rows(path, name):
    out = []
    with open(path) as handle:
        for line in handle:
            if f" name={name} " not in line and not line.rstrip().endswith(f"name={name}"):
                continue
            fields = dict(m.groups() for m in re.finditer(r"(\w+)=(\S+)", line))
            if fields.get("name") == name:
                out.append(fields)
    return out

def floats(text):
    return [None if v == "none" else float(v) for v in text.split(",")]
