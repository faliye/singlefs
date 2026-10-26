#!/usr/bin/env python3
"""module_census.py <树> <dump 输出> <集合文件> <标签>：闭包里有单元的每份 .py，dump 里它的 import / assign / class / module-body / main-guard 单元不在闭包的个数；
另用 ast 自己数这些文件模块级 import 绑定名与 assign 目标名（独立于 lib_function_units 的切法）。"""
import ast, collections, os, sys
tree, dump, setfile, label = sys.argv[1:5]
members = set()
for l in open(setfile, encoding="utf-8"):
    k, n, f, fp = l.rstrip("\n").split("|", 3)
    members.add((k, n, f))
files = sorted({f for _k, _n, f in members if f.endswith(".py")})
by_kind = collections.Counter(); by_file = {}
dumped = collections.defaultdict(list)
for l in open(dump, encoding="utf-8"):
    f = l.rstrip("\n").split("\t")
    if len(f) >= 4: dumped[f[2].rsplit(":", 1)[0]].append((f[0], f[1], f[2].rsplit(":", 1)[0]))
for file in files:
    miss = [r for r in dumped.get(file, []) if r[0] in ("import", "assign", "class", "module-body", "main-guard") and r not in members]
    c = collections.Counter(r[0] for r in miss); by_file[file] = c; by_kind.update(c)
print(f"{label} py文件={len(files)} 单元缺口(按 dump) " + " ".join(f"{k}={v}" for k, v in sorted(by_kind.items())) + f" 合计(import+assign+class)={by_kind['import']+by_kind['assign']+by_kind['class']} 合计(import+assign)={by_kind['import']+by_kind['assign']}")
# ast 独立数：模块级（顶层语句，不进 def/class 体）import 绑定名 + assign 目标名
ast_imp = ast_asg = ast_cls = 0
for file in files:
    src = open(os.path.join(tree, file), encoding="utf-8").read()
    mod = ast.parse(src)
    imp = asg = cls = 0
    for node in mod.body:
        if isinstance(node, ast.Import): imp += len(node.names)
        elif isinstance(node, ast.ImportFrom): imp += len(node.names)
        elif isinstance(node, ast.Assign): asg += sum(len([x for x in ast.walk(t) if isinstance(x, ast.Name) and isinstance(x.ctx, ast.Store)]) for t in node.targets)
        elif isinstance(node, (ast.AnnAssign, ast.AugAssign)): asg += 1
        elif isinstance(node, ast.ClassDef): cls += 1
    ast_imp += imp; ast_asg += asg; ast_cls += cls
    print(f"  {file}: ast 顶层 import 名 {imp} assign 名 {asg} class {cls} | dump 缺口 {dict(by_file[file])}")
print(f"{label} ast 顶层合计 import={ast_imp} assign={ast_asg} class={ast_cls}（含已在闭包里的）")
