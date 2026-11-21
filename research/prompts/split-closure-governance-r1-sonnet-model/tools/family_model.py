#!/usr/bin/env python3
"""拆分族认法的独立小模型：只按 scripts/gate/lib_lag_split_family.py 文件头写的三种拆法与比法重写一遍，不 import 仓里任何代码；
把 family_probe.py 的前八组输入喂给它，打每组交出的涨了的路径，供与仓里那份的 actual 逐行比（不一致就是两边有一边读错）。
用法：python3 -I family_model.py"""


def recipes(owner):
    suffix = owner[-3:]
    directory, _, name = owner.rpartition("/")
    own = directory + "/" if directory else ""
    stem = name[:-3]
    result = [(own, stem.replace("-", "_") + "_", (".py",))]
    if suffix == ".sh":
        result.append((own, stem + "-", (".py", ".sh")))
        if directory == ".claude/gate.d":
            result.append((directory + "/lib/", stem + "-", (".py", ".sh")))
    return result


def prefix_length(owner, path):
    if owner == path:
        return len(path)
    if owner.endswith(".rs"):
        name = owner.rpartition("/")[2]
        prefix = owner[:len(owner) - len(name)] if name in ("main.rs", "mod.rs") else owner[:-3] + "/"
        return len(prefix) if path.endswith(".rs") and path.startswith(prefix) else None
    if owner[-3:] not in (".py", ".sh"):
        return None
    best = None
    for directory, prefix, suffixes in recipes(owner):
        if not path.startswith(directory) or not path.endswith(suffixes):
            continue
        rest = path[len(directory):]
        if "/" not in rest and rest.startswith(prefix) and rest[len(prefix):-3]:
            best = max(best or 0, len(directory) + len(prefix))
    return best


def owner_of(path, base):
    if path in base:
        return path
    lengths = {o: prefix_length(o, path) for o in base}
    lengths = {o: n for o, n in lengths.items() if n is not None}
    return max(lengths, key=lambda o: (lengths[o], len(o), o)) if lengths else None


def growth(base, current):
    rows = {}
    for path, count in current.items():
        rows.setdefault(owner_of(path, base), []).append((path, count))
    red = []
    for owner, members in rows.items():
        if owner is None or sum(count for _p, count in members) > base[owner]:
            red += [path for path, _c in members]
    return sorted(red)


CASES = [
    ({"x/foo.py": 5}, {"x/foo.py": 5, "x/foo_part.py": 2}),
    ({"x/foo.py": 5}, {"x/foo.py": 3, "x/foo_part.py": 2}),
    ({"scripts/gate/lib_function.py": 5}, {"scripts/gate/lib_function.py": 2, "scripts/gate/lib_function_units.py": 3}),
    ({"scripts/gate/lib_function.py": 5, "scripts/gate/lib_function_units.py": 1}, {"scripts/gate/lib_function.py": 2, "scripts/gate/lib_function_units.py": 3}),
    ({"a/src/main.rs": 3}, {"a/src/main.rs": 1, "a/src/other_module.rs": 2}),
    ({"scripts/gate/gate.sh": 4}, {"scripts/gate/gate.sh": 1, "scripts/gate/gate-staged.sh": 3}),
    ({"x/foo.py": 5}, {"y/foo_part.py": 1}),
    ({"x/foo.py": 5}, {"x/foobar.py": 1}),
]
for number, (base, current) in enumerate(CASES, 1):
    print(f"model-case={number} red={growth(base, current)}")
