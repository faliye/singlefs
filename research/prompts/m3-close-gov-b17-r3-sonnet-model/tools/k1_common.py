"""K1 各探针共用：按路径载入某一份仓里的 crates-mutation-rows.py（判法所在的仓 = 那份脚本自己所在的仓），读变异表、算每一行的键。"""
import importlib.util
import os
import sys


def load_rows_module(script_repository):
    path = os.path.join(script_repository, "scripts", "mutation", "crates-mutation-rows.py")
    specification = importlib.util.spec_from_file_location("crates_mutation_rows_under_probe", path)
    module = importlib.util.module_from_spec(specification)
    sys.modules[specification.name] = module
    specification.loader.exec_module(module)
    return module


def table_rows(module, root):
    rows, _malformed, memory_max, timeout_seconds = module.read_table(os.path.join(root, module.TABLE), dict(os.environ))
    memory_max, timeout_seconds, problems = module.limits_of(memory_max, timeout_seconds, dict(os.environ))
    assert not problems, problems
    broken = module.anchor_broken_rows(root, rows)
    return [row for row in rows if row.line_number not in broken], memory_max, timeout_seconds


def key_context_for(module, root, rows, memory_max, timeout_seconds):
    environment = dict(os.environ)
    return module.plan_rows(root, rows, environment, memory_max, timeout_seconds)
