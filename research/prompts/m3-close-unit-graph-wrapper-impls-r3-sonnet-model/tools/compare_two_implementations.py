#!/usr/bin/env python3
"""三方正推腿（m3-close-unit-graph-wrapper-impls-r3）的判法：每个夹具单独放进一个根（根里只有这一个 crate），Rust 一份（unit-graph 命令行）与
Python 一份（scripts/gate/lib_function_units.py 的 closure 子命令）各收一次闭包，比两份闭包逐行是否相同，并数「被看的 impl 方法」进没进各自的闭包。
被看的 impl 方法是「等包装类型被点名」的那一类 impl（impl<Inner> Trait for Wrapper<Inner>，Wrapper 在入口的闭包里没人点名）的方法体：
进了闭包 = 改它键会变；没进 = 值若经别的路到了这里、改它键不变。

用法：compare_two_implementations.py --binary <unit-graph> --python-lib <lib_function_units.py> --model <本模型目录> --work <运行目录（不在或是空目录）>
每个格交一行：name=<格名> rust_members=<个数> python_members=<个数> closures_equal=<yes|no>；两份闭包不同时另打「  diff」行（不进与 expected 的比较）。
结果写进 <运行目录>/result.txt，逐行与 <模型目录>/out/expected.txt 比。
退出码：0 每一行与 expected 相同；1 有行不同；2 用法错、夹具对不上、某一步跑不成。
起的每个子进程放进自己的进程组，超时与收尾时对整组发 SIGKILL，跑完核这一组没有留下进程。"""
import argparse
import os
import re
import shutil
import signal
import subprocess
import sys

SECONDS = 300
CASE_FIELDS = 5


def stop(message):
    print(f"  ✗ {message}", file=sys.stderr)
    raise SystemExit(2)


def run_in_its_own_group(argv, seconds):
    environment = {name: value for name, value in os.environ.items() if name not in ("FUNCTION_UNITS_BREAK", "SINGLEFS_UNIT_GRAPH_BREAK")}
    process = subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=environment, start_new_session=True)
    try:
        output, errors = process.communicate(timeout=seconds)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.communicate()
        stop(f"超时（{seconds} 秒）：{' '.join(argv)}")
    try:
        os.killpg(process.pid, 0)
    except ProcessLookupError:
        pass
    else:
        os.killpg(process.pid, signal.SIGKILL)
        stop(f"跑完之后进程组里还有进程，已整组 SIGKILL：{' '.join(argv)}")
    return process.returncode, output, errors


def read_cases(model):
    cases = []
    with open(os.path.join(model, "cases.tsv"), encoding="utf-8") as table:
        for line in table:
            line = line.rstrip("\n")
            if not line or line.startswith("#"):
                continue
            fields = line.split("\t")
            if len(fields) != CASE_FIELDS:
                stop(f"cases.tsv 一行要五段：{fields}")
            name, crate, rust_entry, python_entry, original = fields
            cases.append(dict(name=name, crate=crate, rust_entry=rust_entry, python_entry=python_entry, original=original))
    return cases


def materialize(model, root, crate):
    source = os.path.join(model, "fixture", "crates", crate)
    if not os.path.isdir(source):
        stop(f"模型目录里没有夹具 crate {crate}")
    for directory, _, names in os.walk(source):
        for name in names:
            if name.endswith(".fixture"):
                target = os.path.join(root, "crates", crate, os.path.relpath(os.path.join(directory, name), source)[: -len(".fixture")])
                os.makedirs(os.path.dirname(target), exist_ok=True)
                shutil.copyfile(os.path.join(directory, name), target)
    with open(os.path.join(root, "Cargo.lock"), "w", encoding="utf-8") as lock:
        lock.write("# 夹具不解析依赖\n")


def closure_lines(argv):
    code, output, _errors = run_in_its_own_group(argv, SECONDS)
    if code != 0:
        return code, None
    return 0, [line for line in output.splitlines() if line and not line.startswith("key\t")]


def members_in(lines, crate, edit_line):
    location = re.compile(r"^crates/" + re.escape(crate) + r"/src/lib\.rs:(\d+)-(\d+)$")
    count = 0
    for line in lines:
        fields = line.split("\t")
        if fields[0] == "fn" and len(fields) >= 3:
            found = location.match(fields[2])
            if found and int(found.group(1)) <= edit_line <= int(found.group(2)):
                count += 1
    return count


def host_triple():
    code, output, _errors = run_in_its_own_group(["rustc", "-vV"], 60)
    found = re.search(r"^host: (\S+)$", output, re.MULTILINE) if code == 0 else None
    if found is None:
        stop("rustc -vV 取不到 host 行")
    return found.group(1)


def main():
    parser = argparse.ArgumentParser()
    for option in ("--binary", "--python-lib", "--model", "--work"):
        parser.add_argument(option, required=True)
    arguments = parser.parse_args()
    for path in (arguments.binary, arguments.python_lib):
        if not os.path.isfile(path):
            stop(f"文件不在：{path}")
    if os.path.exists(arguments.work) and os.listdir(arguments.work):
        stop(f"运行目录 {arguments.work} 已存在且非空：给一个新的")
    host = host_triple()
    lines = []
    diff_lines = []
    for case in read_cases(arguments.model):
        root = os.path.join(arguments.work, "roots", case["name"])
        materialize(arguments.model, root, case["crate"])
        with open(os.path.join(root, "crates", case["crate"], "src", "lib.rs"), encoding="utf-8") as source:
            text = source.read()
        if text.count(case["original"]) != 1:
            stop(f"{case['name']} 的原文要恰好一次，实际 {text.count(case['original'])} 次")
        edit_line = text[: text.index(case["original"])].count("\n") + 1
        rust_code, rust_lines = closure_lines([arguments.binary, "closure", root, case["rust_entry"], "--toolchain-line", f"host: {host}"])
        python_entry = f"crates/{case['crate']}/src/lib.rs::{case['python_entry']}"
        python_code, python_lines = closure_lines([sys.executable, arguments.python_lib, "closure", root, python_entry])
        if rust_lines is None or python_lines is None:
            lines.append(f"name={case['name']} rust_rc={rust_code} python_rc={python_code}")
            continue
        equal = "yes" if sorted(rust_lines) == sorted(python_lines) else "no"
        lines.append(f"name={case['name']} rust_members={members_in(rust_lines, case['crate'], edit_line)} "
                     f"python_members={members_in(python_lines, case['crate'], edit_line)} closures_equal={equal}")
        for line in sorted(set(rust_lines) - set(python_lines)):
            diff_lines.append(f"  diff {case['name']} 只在 Rust：{line}")
        for line in sorted(set(python_lines) - set(rust_lines)):
            diff_lines.append(f"  diff {case['name']} 只在 Python：{line}")
    os.makedirs(arguments.work, exist_ok=True)
    with open(os.path.join(arguments.work, "result.txt"), "w", encoding="utf-8") as result:
        result.write("\n".join(lines) + "\n")
    with open(os.path.join(arguments.model, "out", "expected.txt"), encoding="utf-8") as expected_file:
        expected = [line.rstrip("\n") for line in expected_file if line.strip() and not line.startswith("#")]
    mismatches = 0
    for line in lines + diff_lines:
        print(line)
    for position in range(max(len(lines), len(expected))):
        got = lines[position] if position < len(lines) else "（没有这一行）"
        want = expected[position] if position < len(expected) else "（没有这一行）"
        if got != want:
            mismatches += 1
            print(f"  ✗ 与 expected 不同：得到「{got}」，expected「{want}」")
    print(f"name=summary lines={len(lines)} mismatches={mismatches}")
    return 1 if mismatches else 0


if __name__ == "__main__":
    sys.exit(main())
