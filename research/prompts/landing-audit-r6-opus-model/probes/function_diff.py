"""K2 前两轮站住的判法有没有被第十一批改到：两版钩子里点名的几个函数，函数体逐字比（按 def 起、到下一个顶格 def / 顶格赋值为止）。
用法：python3 -I function_diff.py <改前文件> <改后文件> <函数名>…"""
import re, sys
before, after, names = sys.argv[1], sys.argv[2], sys.argv[3:]
def bodies(path):
    text = open(path, encoding="utf-8").read().split("\n")
    found, current, name = {}, [], None
    for line in text:
        top = re.match(r"^(def |class |[A-Z_]+ = |PY$|@)", line)
        if top:
            if name:
                found[name] = "\n".join(current).rstrip()
            name, current = None, []
            match = re.match(r"^def (\w+)", line)
            if match:
                name = match.group(1)
        if name:
            current.append(line)
    if name:
        found[name] = "\n".join(current).rstrip()
    return found
left, right = bodies(before), bodies(after)
for name in names:
    if name not in left or name not in right:
        print(f"{name}：改前{'有' if name in left else '没有'}、改后{'有' if name in right else '没有'}")
    else:
        print(f"{name}：{'逐字相同' if left[name] == right[name] else '不同'}（改前 {left[name].count(chr(10)) + 1} 行、改后 {right[name].count(chr(10)) + 1} 行）")
