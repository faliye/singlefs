#!/usr/bin/env python3
"""建 agent-write-scope 格的五份探针样本仓：从冻结副本的两份样本（agent-write-scope-name-like-red、agent-write-scope-green）各拷一份再改一处。
用法：build_write_scope_samples.py --frozen <冻结副本> --out <样本根>（<样本根> 不在或为空）
  name-like-red-unchanged          原样（对照：该红、点名 lib_shell_words_shadow.py:4）
  name-like-red-quoted-comment     被判仓的入口 lib_shell_words.py 末尾加一行注释，引号里写影子文件名
  name-like-red-docstring-import   被判仓的入口末尾加一个字符串常量，里面一行以 import lib_shell_words_shadow 起头
  green-core-by-import-module      green 加一份 lib_shell_words_core.py（定义 brace_expand），入口用 importlib.import_module("lib_shell_words_core") 载入它
  green-core-by-from-import        同上，入口用 from lib_shell_words_core import brace_expand 载入（对照）
每份样本打一行 SAMPLE name=<名> path=<路径> entry_sha256=<被判仓入口的 sha256>。
"""
import argparse, hashlib, os, shutil, sys

FIXTURES = os.path.join(".claude", "gate.d", "fixtures", "code-tooling.sh")
CORE = "def brace_expand(word, mask):\n    return [word]\n"
EDITS = {
    "name-like-red-unchanged": ("agent-write-scope-name-like-red", "", None),
    "name-like-red-quoted-comment": ("agent-write-scope-name-like-red", "# 'lib_shell_words_shadow.py' 是别的钩子留下的影子，不归这一份。\n", None),
    "name-like-red-docstring-import": ("agent-write-scope-name-like-red", 'NOTE = """不要这样写：\n    import lib_shell_words_shadow\n"""\n', None),
    "green-core-by-import-module": ("agent-write-scope-green", 'import importlib\nbrace_expand = importlib.import_module("lib_shell_words_core").brace_expand\n', CORE),
    "green-core-by-from-import": ("agent-write-scope-green", "from lib_shell_words_core import brace_expand\n", CORE),
}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--out", required=True)
    arguments = parser.parse_args()
    out = os.path.realpath(arguments.out)
    if os.path.exists(out) and os.listdir(out):
        print(f"  ✗ 样本根 {out} 已存在且非空", file=sys.stderr)
        return 2
    for name, (fixture, appended, core) in EDITS.items():
        target = os.path.join(out, name)
        shutil.copytree(os.path.join(os.path.realpath(arguments.frozen), FIXTURES, fixture), target)
        entry = os.path.join(target, ".claude", "hooks", "lib_shell_words.py")
        with open(entry, "a", encoding="utf-8") as handle:
            handle.write(("\n" + appended) if appended else "")
        if core is not None:
            with open(os.path.join(target, ".claude", "hooks", "lib_shell_words_core.py"), "w", encoding="utf-8") as handle:
                handle.write(core)
        digest = hashlib.sha256(open(entry, "rb").read()).hexdigest()
        print(f"SAMPLE name={name} path={target} entry_sha256={digest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
