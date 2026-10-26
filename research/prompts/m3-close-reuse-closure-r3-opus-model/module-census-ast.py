"""模块级语句普查，另一条路数（不读 dump，自己用 ast 切）：闭包里有单元的每份 python 文件，逐条模块级语句按 python 的语义列出 import 时执行的东西——
import / from-import 按绑定的名字、赋值（=、带注解的 =、+=）按目标名、class 按类名、def 按函数名，再看它在不在闭包的单元行里（种类、限定名、文件）。
另数 import 时会执行代码的 def 头：带装饰器、或缺省值不是字面常量的模块级 def，本身不在闭包里的。
用法：module-census-ast.py <根> <闭包输出> <标签>"""
import ast
import collections
import sys

root, closure_path, label = sys.argv[1:4]
members = set()
for line in open(closure_path, encoding="utf-8"):
    fields = line.rstrip("\n").split("\t")
    if fields[0] not in ("via", "refused", "key") and len(fields) >= 4:
        members.add((fields[0], fields[1], fields[2].rsplit(":", 1)[0]))
files = sorted({file for _kind, _name, file in members if file.endswith(".py")})


def bound(node):
    if isinstance(node, ast.Name):
        yield node.id
    elif isinstance(node, (ast.Tuple, ast.List)):
        for element in node.elts:
            yield from bound(element)
    elif isinstance(node, ast.Starred):
        yield from bound(node.value)


missing, executed_def_heads = collections.Counter(), []
for file in files:
    tree = ast.parse(open(f"{root}/{file}", encoding="utf-8").read())
    for statement in tree.body:
        rows = []
        if isinstance(statement, ast.Import):
            rows = [("import", (alias.asname or alias.name).split(".")[0]) for alias in statement.names]
        elif isinstance(statement, ast.ImportFrom):
            rows = [("import", alias.asname or alias.name) for alias in statement.names if alias.name != "*"]
        elif isinstance(statement, ast.Assign):
            rows = [("assign", name) for target in statement.targets for name in bound(target)]
        elif isinstance(statement, (ast.AnnAssign, ast.AugAssign)) and isinstance(statement.target, ast.Name):
            rows = [("assign", statement.target.id)]
        elif isinstance(statement, ast.ClassDef):
            rows = [("class", statement.name)]
        elif isinstance(statement, (ast.FunctionDef, ast.AsyncFunctionDef)):
            defaults = statement.args.defaults + [d for d in statement.args.kw_defaults if d is not None]
            runs = statement.decorator_list or any(not isinstance(d, ast.Constant) for d in defaults)
            if runs and ("def", statement.name, file) not in members:
                executed_def_heads.append(f"{file}:{statement.lineno}:{statement.name}")
        for kind, name in rows:
            if (kind, name, file) not in members:
                missing[kind] += 1
print(f"name=module_census_ast label={label} python_files_in_closure={len(files)} not_in_closure_total={sum(missing.values())} "
      f"by_kind={dict(sorted(missing.items()))} executed_def_heads_not_in_closure={len(executed_def_heads)}")
for item in executed_def_heads[:6]:
    print(f"  def_head={item}")
