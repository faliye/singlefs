#!/usr/bin/env python3
"""三方攻方腿（m3-close-unit-graph-wrapper-impls-r2）的判法：在夹具上比四种规则收不收「被改的那个方法」、改它之后闭包的键变不变，
再把夹具编出来看入口（驱动程序调的那个函数）交回的值变不变。

用法：judge.py --today-binary <主工作区编的 unit-graph> --verify-copy-binary <verify-copy 编的 unit-graph>
               --today-library <主工作区的 scripts/gate> --verify-copy-library <verify-copy 的 scripts/gate> --model <模型目录> --work <运行目录>
  四种规则：today、verify_copy（两份 unit-graph closure，--toolchain-line host: x86_64-unknown-linux-gnu）；
            python_today、python_verify_copy（两份 lib_function_units.py，经 tools/python_closure.py）。
  每个夹具交五行：name=<crate> rule=<规则> impl_in_closure=<个数> key_changes_with_the_edit=<yes|no>（四种规则各一行）；
                  name=<crate> value_before=<值> value_after=<值>。某一步跑不成的那一格写 error=<原因>，照接着跑别的格。
  结果写进 <运行目录>/result.txt，逐行与 <模型目录>/out/expected.txt 比。
退出码：0 每一行与 expected 相同；1 有行不同；2 用法错、夹具对不上、有格跑不成（单元图退非 0、拒算、编不过、跑不成、超时）。
起的每个子进程放进自己的进程组，超时与收尾时对整组发 SIGKILL，跑完核这一组没有留下进程。"""
import argparse
import os
import re
import shutil
import signal
import subprocess
import sys

TOOLCHAIN_LINE = "host: x86_64-unknown-linux-gnu"
UNIT_GRAPH_SECONDS = 300
PYTHON_SECONDS = 600
RUSTC_SECONDS = 600
DRIVER_SECONDS = 60


class StepFailed(Exception):
    pass


def run_in_its_own_group(argv, seconds):
    process = subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True)
    try:
        output, errors = process.communicate(timeout=seconds)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.communicate()
        raise StepFailed(f"超时（{seconds} 秒）：{' '.join(argv)}")
    try:
        os.killpg(process.pid, 0)
    except ProcessLookupError:
        pass
    else:
        os.killpg(process.pid, signal.SIGKILL)
        raise StepFailed(f"跑完之后进程组里还有进程，已整组 SIGKILL：{' '.join(argv)}")
    return process.returncode, output, errors


def read_edits(model):
    rows = []
    with open(os.path.join(model, "edits.tsv"), encoding="utf-8") as table:
        for line in table:
            line = line.rstrip("\n")
            if not line or line.startswith("#"):
                continue
            fields = line.split("\t")
            if len(fields) != 5:
                print(f"  ✗ edits.tsv 一行要五段：{line}", file=sys.stderr)
                raise SystemExit(2)
            rows.append(dict(crate=fields[0], entry=fields[1], call=fields[2], original=fields[3], replacement=fields[4]))
    return rows


def materialize(model, destination, edits, edited):
    source_root = os.path.join(model, "fixture")
    for directory, _, names in os.walk(source_root):
        for name in names:
            if not name.endswith(".fixture"):
                continue
            relative = os.path.relpath(os.path.join(directory, name), source_root)
            target = os.path.join(destination, relative[: -len(".fixture")])
            os.makedirs(os.path.dirname(target), exist_ok=True)
            shutil.copyfile(os.path.join(directory, name), target)
    edit_lines = {}
    for row in edits:
        path = os.path.join(destination, "crates", row["crate"], "src", "lib.rs")
        with open(path, encoding="utf-8") as source:
            text = source.read()
        if text.count(row["original"]) != 1:
            print(f"  ✗ {row['crate']} 的原文要恰好一次，实际 {text.count(row['original'])} 次", file=sys.stderr)
            raise SystemExit(2)
        edit_lines[row["crate"]] = text[: text.index(row["original"])].count("\n") + 1
        if edited:
            with open(path, "w", encoding="utf-8") as source:
                source.write(text.replace(row["original"], row["replacement"]))
    return edit_lines


def rust_reading(binary, root, row, edit_line):
    argv = [binary, "closure", root, f"{row['crate']}::{row['entry']}", "--toolchain-line", TOOLCHAIN_LINE]
    code, output, errors = run_in_its_own_group(argv, UNIT_GRAPH_SECONDS)
    if code != 0:
        raise StepFailed(f"单元图退 {code}（3 是拒算）：{output[-600:]} {errors[-600:]}")
    key = None
    holding = 0
    location = re.compile(r"^crates/" + re.escape(row["crate"]) + r"/src/lib\.rs:(\d+)-(\d+)$")
    for line in output.splitlines():
        fields = line.split("\t")
        if fields[0] == "key" and len(fields) >= 2:
            key = fields[1]
        elif fields[0] == "fn" and len(fields) >= 3:
            found = location.match(fields[2])
            if found and int(found.group(1)) <= edit_line <= int(found.group(2)):
                holding += 1
    if key is None:
        raise StepFailed("单元图输出里没有 key 行")
    return key, holding


def python_reading(library, model, root, row, edit_line):
    argv = [sys.executable, os.path.join(model, "tools", "python_closure.py"), "--library", library, "--root", root,
            "--file", f"crates/{row['crate']}/src/lib.rs", "--path", row["entry"], "--edit-line", str(edit_line)]
    code, output, errors = run_in_its_own_group(argv, PYTHON_SECONDS)
    if code != 0:
        raise StepFailed(f"python 那一份退 {code}（3 是拒算）：{output[-600:]} {errors[-600:]}")
    values = dict(line.split("=", 1) for line in output.splitlines() if "=" in line and not line.startswith("refused"))
    return values["key"], int(values["holding"])


def value_of_the_entry(root, row, build_directory):
    os.makedirs(build_directory, exist_ok=True)
    library = os.path.join(root, "crates", row["crate"], "src", "lib.rs")
    code, _output, errors = run_in_its_own_group(
        ["rustc", "--edition", "2021", "--crate-type", "rlib", "--crate-name", row["crate"], library,
         "--out-dir", build_directory], RUSTC_SECONDS)
    if code != 0:
        raise StepFailed(f"{library} 编不过：{errors[-1500:]}")
    driver = os.path.join(build_directory, f"drive_{row['crate']}.rs")
    with open(driver, "w", encoding="utf-8") as text:
        text.write(f"fn main() {{\n    println!(\"{{}}\", {row['crate']}::{row['call']}());\n}}\n")
    program = os.path.join(build_directory, f"drive_{row['crate']}")
    code, _output, errors = run_in_its_own_group(
        ["rustc", "--edition", "2021", "--extern", f"{row['crate']}={build_directory}/lib{row['crate']}.rlib",
         driver, "-o", program], RUSTC_SECONDS)
    if code != 0:
        raise StepFailed(f"{driver} 编不过：{errors[-1500:]}")
    code, output, errors = run_in_its_own_group([program], DRIVER_SECONDS)
    if code != 0:
        raise StepFailed(f"{program} 退 {code}：{errors[-600:]}")
    return output.strip()


def one_line(text):
    return " ".join(text.split())[:400]


def main():
    parser = argparse.ArgumentParser()
    for option in ("--today-binary", "--verify-copy-binary", "--today-library", "--verify-copy-library", "--model", "--work"):
        parser.add_argument(option, required=True)
    arguments = parser.parse_args()
    rules = (
        ("today", "rust", arguments.today_binary),
        ("verify_copy", "rust", arguments.verify_copy_binary),
        ("python_today", "python", arguments.today_library),
        ("python_verify_copy", "python", arguments.verify_copy_library),
    )
    edits = read_edits(arguments.model)
    original_root = os.path.join(arguments.work, "fixture-original")
    edited_root = os.path.join(arguments.work, "fixture-edited")
    edit_lines = materialize(arguments.model, original_root, edits, edited=False)
    materialize(arguments.model, edited_root, edits, edited=True)
    lines = []
    failures = 0
    for row in edits:
        edit_line = edit_lines[row["crate"]]
        for rule_name, language, tool in rules:
            try:
                if language == "rust":
                    key_before, holding = rust_reading(tool, original_root, row, edit_line)
                    key_after, _ = rust_reading(tool, edited_root, row, edit_line)
                else:
                    key_before, holding = python_reading(tool, arguments.model, original_root, row, edit_line)
                    key_after, _ = python_reading(tool, arguments.model, edited_root, row, edit_line)
            except StepFailed as failure:
                failures += 1
                lines.append(f"name={row['crate']} rule={rule_name} error={one_line(str(failure))}")
                continue
            changes = "yes" if key_before != key_after else "no"
            lines.append(f"name={row['crate']} rule={rule_name} impl_in_closure={holding} key_changes_with_the_edit={changes}")
        try:
            before = value_of_the_entry(original_root, row, os.path.join(arguments.work, "build-original", row["crate"]))
            after = value_of_the_entry(edited_root, row, os.path.join(arguments.work, "build-edited", row["crate"]))
            lines.append(f"name={row['crate']} value_before={before} value_after={after}")
        except StepFailed as failure:
            failures += 1
            lines.append(f"name={row['crate']} value error={one_line(str(failure))}")
    with open(os.path.join(arguments.work, "result.txt"), "w", encoding="utf-8") as result:
        result.write("\n".join(lines) + "\n")
    with open(os.path.join(arguments.model, "out", "expected.txt"), encoding="utf-8") as expected_file:
        expected = [line.rstrip("\n") for line in expected_file if line.strip() and not line.startswith("#")]
    mismatches = 0
    for line in lines:
        print(line)
    for position in range(max(len(lines), len(expected))):
        got = lines[position] if position < len(lines) else "（没有这一行）"
        want = expected[position] if position < len(expected) else "（没有这一行）"
        if got != want:
            mismatches += 1
            print(f"  ✗ 与 expected 不同：得到「{got}」，expected「{want}」")
    print(f"name=summary lines={len(lines)} mismatches={mismatches} failed_steps={failures}")
    if failures:
        return 2
    return 1 if mismatches else 0


if __name__ == "__main__":
    sys.exit(main())
