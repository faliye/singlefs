"""G2：E171 两份 frozen_o3.py 里读搬迁登记的函数正文是不是逐字相同（ast 取源码段）。用法：python3 compare_frozen_o3.py --repo <仓根>"""
import argparse
import ast
import os

NAMES = ["RegisteredMoves", "path_moves_library", "registered_moves", "sha256_before_moves", "matches_the_freeze", "seed_from_directory", "freeze_differences"]
PATHS = ["scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py", "scripts/experiment-apparatus/e171_three_oracle_collusion/frozen_o3.py"]


def segments(path):
    source = open(path, encoding="utf-8").read()
    return {node.name: ast.get_source_segment(source, node) for node in ast.walk(ast.parse(source)) if isinstance(node, (ast.FunctionDef, ast.ClassDef))}


parser = argparse.ArgumentParser()
parser.add_argument("--repo", required=True)
repo = os.path.abspath(parser.parse_args().repo)
first, second = (segments(os.path.join(repo, path)) for path in PATHS)
for name in NAMES:
    print(f"name=frozen-o3-same-body function={name} identical={first.get(name) == second.get(name)}")
