#!/usr/bin/env python3
"""第 9 问的模型：check.sh 那张 clippy lint 表换几种 bash 认的写法，被判那棵树的 .claude/hooks/lib_code_discipline_lints.py（检出钩子 ㉓ 与项目代码闸 ⑤ 共用）
读到的，与 bash 自己展开出来的（整轮门禁真传给 clippy 的）逐个比。

用法：python3 lints_model.py --frozen <被判的那棵树> --work <新的空目录>
每种写法一行 `LINTS name=<写法> …`：
  guard_judges        ⑤ 判不判（共用模块读不到表时 ⑤ 不判）
  misreject           照门禁真传的整段写 clippy，⑤ 还说缺的名字（误拒）
  fail_open           门禁真当错误判的 lint（-D、--deny、-F、--forbid 点名的），⑤ 不要求的（照抄出路或少写这几条也过得了 ⑤）
  outlet_equals_gate  ㉓ 出路给的 `--` 之后整段与门禁真传的是否逐词相同
末行 `LINTS name=done emitted=<行数>`。退出码：0 跑完；1 有一种写法装置出错；2 用法错。
"""
import argparse
import importlib.util
import json
import os
import re
import shlex
import subprocess
import sys

EMITTED = 0
ERROR_FLAGS = {"-D": "deny", "--deny": "deny", "-F": "forbid", "--forbid": "forbid"}


def emit(name, **fields):
    global EMITTED
    EMITTED += 1
    parts = [f"name={name}"]
    for key, value in fields.items():
        text = str(value)
        if not re.fullmatch(r"[\w.+\-/=,%:]+", text):
            text = json.dumps(text, ensure_ascii=False)
        parts.append(f"{key}={text}")
    print("LINTS " + " ".join(parts), flush=True)


def error_level_names(words):
    """独立的认法：-D / --deny / -F / --forbid 后面的名字（分开写、贴着写、= 写）都是门禁当错误判的 lint。"""
    names, position = [], 0
    while position < len(words):
        word = words[position]
        if word in ERROR_FLAGS and position + 1 < len(words):
            names.append(words[position + 1])
            position += 2
            continue
        for flag in ("--deny=", "--forbid="):
            if word.startswith(flag):
                names.append(word[len(flag):])
        if re.fullmatch(r"-[DF].+", word):
            names.append(word[2:])
        position += 1
    return names


def bash_words(text, work, name):
    path = os.path.join(work, name + ".sh")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    listed = subprocess.run(["bash", "-c", 'source "$1" && printf "%s\\0" "${CODE_DISCIPLINE_LINTS[@]}"', "table", path], capture_output=True, text=True)
    if listed.returncode != 0:
        raise RuntimeError(f"bash 展开不了 {name}：{listed.stderr.strip()[:200]}")
    return path, [word for word in listed.stdout.split("\0") if word]


def variants(today):
    lines = today.splitlines()
    items = lines[1:-1]
    yield "today", today, {}
    yield "append-with-plus-equals", today + "\nCODE_DISCIPLINE_LINTS+=( -D clippy::needless_return )\n", {}
    yield "reassigned-later", today + "\nCODE_DISCIPLINE_LINTS=( -D clippy::needless_return -D clippy::todo )\n", {}
    continued = "\n".join([lines[0], items[0].split("#")[0].rstrip() + " \\", *items[1:], lines[-1]]) + "\n"
    yield "backslash-continuation", continued, {}
    braced = [line for line in items if "cast_possible_truncation" not in line and "cast_sign_loss" not in line]
    yield "brace-expansion", "\n".join([lines[0], "  -Dclippy::cast_{possible_truncation,sign_loss}", *braced, lines[-1]]) + "\n", {}
    yield "variable-element", "BASE_LINTS=( -D clippy::todo -D clippy::dbg_macro )\n" + "\n".join([lines[0], '  "${BASE_LINTS[@]}"', *items, lines[-1]]) + "\n", {}
    yield "forbid-entries", "\n".join([lines[0], *items, "  -F clippy::todo", "  --forbid clippy::dbg_macro", lines[-1]]) + "\n", {}
    yield "today-with-break-switch-set", today, {"LIB_CODE_DISCIPLINE_LINTS_BREAK": "first-word-per-line"}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    work = os.path.realpath(arguments.work)
    if os.path.exists(work) and os.listdir(work):
        print(f"  ✗ --work {work} 已存在且非空", file=sys.stderr)
        return 2
    os.makedirs(work, exist_ok=True)
    specification = importlib.util.spec_from_file_location("lib_code_discipline_lints", os.path.join(arguments.frozen, ".claude", "hooks", "lib_code_discipline_lints.py"))
    library = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(library)
    with open(os.path.join(arguments.frozen, ".claude", "singlefs-ai-sop", "scripts", "check.sh"), encoding="utf-8") as handle:
        today = re.search(r"^CODE_DISCIPLINE_LINTS=\(.*?^\)", handle.read(), re.M | re.S).group(0)
    failed = False
    for name, text, environment in variants(today):
        try:
            path, gate = bash_words(text, work, name)
            saved = {key: os.environ.get(key) for key in environment}
            os.environ.update(environment)
            try:
                names = library.table_names(path)
                tail = library.table_tail(path)
                read = library.table_words(path)
                gate_tail = ["-D", "warnings", *gate]
                required = None if names is None else ["warnings", *names]
                misreject = [] if required is None else [item for item in required if item not in library.denied_names(gate_tail)]
                fail_open = [item for item in error_level_names(gate) if required is None or item not in required]
                outlet = shlex.split(tail)[1:] if tail else []
            finally:
                for key, value in saved.items():
                    if value is None:
                        os.environ.pop(key, None)
                    else:
                        os.environ[key] = value
            emit(name, gate_words=len(gate), library_words="unreadable" if read is None else len(read), guard_judges="yes" if required is not None else "no",
                 misreject=",".join(misreject) or "none", fail_open=",".join(fail_open) or "none", outlet_equals_gate="yes" if outlet == gate_tail else "no",
                 outlet=" ".join(outlet)[:160] or "empty")
        except Exception as error:   # 一种写法装置出错：写一行接着跑，整轮退 1
            emit(f"error-{name}", error=repr(error)[:200])
            failed = True
    gate = bash_words(today, work, "command-side")[1]
    for name, extra in (("command-allow-after-deny", ["-A", "clippy::cast_sign_loss"]), ("command-cap-lints-allow", ["--cap-lints", "allow"])):
        arguments_after = ["-D", "warnings", *gate, *extra]
        names = library.table_names(os.path.join(work, "command-side.sh"))
        missing = [item for item in ["warnings", *names] if item not in library.denied_names(arguments_after)]
        emit(name, guard_missing=",".join(missing) or "none", guard_passes="yes" if not missing else "no", extra=" ".join(extra))
    print(f"LINTS name=done emitted={EMITTED}", flush=True)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
