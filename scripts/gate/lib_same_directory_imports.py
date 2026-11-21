#!/usr/bin/env python3
# admission: always 读的是调用方这一次给的目录里此刻在盘上的 .py，不存结论
# run-condition: none 只用 python 标准库，读文件、不起子进程
"""同目录 import 闭包：一份 python 文件在同目录里 import 的各份模块，递归收全。

自证要把一份脚本连同它 import 的同目录模块拷进样本仓，拷的清单照这一份现算，不按文件名前缀列：
被 import 的模块名字不一定与入口同前缀（lib_function_units_python_module.py 在模块级 import lib_lag_split_family），按前缀列会漏。

认法（same_directory_imports）：用 ast 读一份文件里的每一处 import（模块级、函数里的都算，行前缀、字符串与注释里的不算）：
  import <模块>[.<子模块>…]、from <模块>[.<子模块>…] import …：取点号前面那一段的模块名；
  from . import <名>、from .<模块> import …（相对 import）：取点出来的同目录模块名。
  名字加 .py 在目录里有这份文件的才算同目录模块；标准库与别的目录的模块（目录里没有同名 .py）不算。
  不认按拼出来的名字载入（importlib、__import__、按路径 exec）：那样载入的文件不在清单里，调用方把它们另列。
闭包（same_directory_import_closure）：入口自己在内，被收进来的每一份再读它的 import，直到不再多出新的为止；交文件名（不带目录），排序。
入口文件读不了、不是合法的 python 抛异常（拷出来的清单不全比拷不出来更难查）。

运行期读的数据文件（runtime_data_files）：有的模块在运行期从仓根往下读数据文件（例 lib_function_units_registry.py 从 LIBRARY_ROOT 读 scripts/gate/test-service-code-paths.tsv），
只拷 .py 的样本仓里读不到。这样的模块在模块级写一个字面的字符串元组 RUNTIME_DATA_FILES，每项是从仓根起的相对路径（写法 ("scripts/gate/test-service-code-paths.tsv",)）；
这里用 ast 读闭包里每份模块的模块级这个赋值（不 import 它们），取并集、排序，交仓根起的路径：
  没有这个常量的模块算零份；
  只认赋值语句的右边是字面的字符串元组（元组里每一项都是字符串字面量，空元组也行）；列表、拼出来的、引用别的名字的、同一份里赋了不止一处的抛 RuntimeDataError，不悄悄跳过：
  悄悄跳过的拷出来的样本仓缺文件，红在用到它的那一格里，比在这里报更难查。

调它的（各自留自己的弄坏开关，认法只在这一份）：
  scripts/gate/admission_selftest_support.py 的 function_units_library_names 与 function_units_runtime_data_names（准入自证拷算键库与它运行期读的数据文件）。
"""
import ast
import os

RUNTIME_DATA_CONSTANT = "RUNTIME_DATA_FILES"


class RuntimeDataError(ValueError):
    """模块级的 RUNTIME_DATA_FILES 不是字面的字符串元组，或赋了不止一处。"""


def imported_module_names(source, path):
    """source（文件 path 的源码）里每一处 import 点到的模块名（点号前面那一段）的集合；不是合法的 python 抛 SyntaxError。"""
    names = set()
    for node in ast.walk(ast.parse(source, filename=path)):
        if isinstance(node, ast.Import):
            names.update(alias.name.split(".")[0] for alias in node.names)
        elif isinstance(node, ast.ImportFrom):
            if node.module:
                names.add(node.module.split(".")[0])
            else:   # from . import <名>：点出来的就是同目录模块
                names.update(alias.name for alias in node.names)
    return names


def same_directory_imports(directory, file_name):
    """directory 里 file_name 这一份直接 import 的同目录模块的文件名（.py）的集合；读不了抛 OSError。"""
    path = os.path.join(directory, file_name)
    with open(path, encoding="utf-8") as handle:
        source = handle.read()
    return {f"{name}.py" for name in imported_module_names(source, path) if os.path.isfile(os.path.join(directory, f"{name}.py"))}


def same_directory_import_closure(directory, entry_name):
    """directory 里 entry_name（带 .py）连同它递归 import 的同目录模块的文件名，排序。"""
    found, pending = {entry_name}, [entry_name]
    while pending:
        for name in same_directory_imports(directory, pending.pop()):
            if name not in found:
                found.add(name)
                pending.append(name)
    return sorted(found)


def assignments_to_runtime_data_constant(tree):
    """tree（一份模块的语法树）里对 RUNTIME_DATA_CONSTANT 这个名字的赋值语句（Assign、AnnAssign、AugAssign；def 与 class 与 lambda 里的不算）。"""
    found, pending = [], list(tree.body)
    while pending:
        node = pending.pop()
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Lambda)):
            continue
        if isinstance(node, (ast.Assign, ast.AnnAssign, ast.AugAssign)):
            targets = node.targets if isinstance(node, ast.Assign) else [node.target]
            if any(isinstance(inner, ast.Name) and inner.id == RUNTIME_DATA_CONSTANT for target in targets for inner in ast.walk(target)):
                found.append(node)
        pending.extend(ast.iter_child_nodes(node))
    return found


def declared_runtime_data_files(source, path):
    """source（文件 path 的源码）模块级写的 RUNTIME_DATA_FILES：字面的字符串元组交元组；没有这个常量交空元组；
    不是字面的字符串元组（列表、拼出来的、引用别的名字的、+=、写在 if 与 try 里的、赋了不止一处的）抛 RuntimeDataError；不是合法的 python 抛 SyntaxError。"""
    tree = ast.parse(source, filename=path)
    assignments = assignments_to_runtime_data_constant(tree)
    if not assignments:
        return ()
    if len(assignments) > 1 or assignments[0] not in tree.body or not isinstance(assignments[0], (ast.Assign, ast.AnnAssign)):
        raise RuntimeDataError(f"{path}：{RUNTIME_DATA_CONSTANT} 要在模块级只赋一次（不用 +=、不写在 if 与 try 里）")
    value = assignments[0].value
    if not (isinstance(value, ast.Tuple) and all(isinstance(item, ast.Constant) and isinstance(item.value, str) for item in value.elts)):
        raise RuntimeDataError(f"{path}:{assignments[0].lineno} {RUNTIME_DATA_CONSTANT} 要写成字面的字符串元组（每项一个字符串字面量，例 (\"scripts/gate/x.tsv\",)）")
    return tuple(item.value for item in value.elts)


def runtime_data_files(directory, file_names, nonliteral_skipped=False):
    """directory 里 file_names（文件名，带 .py，一般是 same_directory_import_closure 的结果）每份模块的 RUNTIME_DATA_FILES 的并集，排序，从仓根起的路径；
    读不了抛 OSError，写得不对抛 RuntimeDataError。nonliteral_skipped 只给调用方的弄坏开关用：写得不对的那一份算零份、不抛。"""
    found = set()
    for name in file_names:
        path = os.path.join(directory, name)
        with open(path, encoding="utf-8") as handle:
            source = handle.read()
        try:
            found.update(declared_runtime_data_files(source, path))
        except RuntimeDataError:
            if not nonliteral_skipped:
                raise
    return sorted(found)
