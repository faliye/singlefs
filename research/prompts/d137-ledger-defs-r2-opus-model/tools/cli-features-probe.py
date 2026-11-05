#!/usr/bin/env python3
"""d137-ledger-defs-r2 攻方腿：compile-then-swap.py 命令行上 --features 的几种写法，各自交给假 cargo 什么；可选在进程里调它的 selftest()。

用法：python3 cli-features-probe.py --script <compile-then-swap.py 的路径> --work <不在或空的目录> [--selftest]
做法：照那份脚本自己的 SelftestFixture 搭临时仓与假 cargo（PATH 最前面放假 cargo，不起真编译），对每种写法调一次它的 main()（经命令行解析，
  不像它的自证那样直接调 swap()），读假 cargo 的调用记录。
输出：每种写法一行
  name=cli_features spelling=<标签> rc=<main 的退出码> calls=<假 cargo 被调几次> gpu_in_every_call=<yes|no|none> features_words=<各次调用里 --features 后面的词，| 隔开>
  --selftest 再打一行 name=selftest_in_process rc=<selftest() 的返回值>（自证的输出照它自己的写法打在前面）。
退出码：0 跑完（不判对错，期望在 ../out/expected.txt）；2 用法错或 --work 非空。
"""
import argparse
import contextlib
import importlib.util
import io
import os
import sys

SPELLINGS = (
    ("features-gpu", ["--features", "gpu"]),
    ("features-equals-gpu", ["--features=gpu"]),
    ("short-flag-gpu", ["-F", "gpu"]),
    ("all-features", ["--all-features"]),
    ("features-comma", ["--features", "extra,gpu"]),
    ("features-twice-gpu-first", ["--features", "gpu", "--features", "extra"]),
    ("features-twice-gpu-last", ["--features", "extra", "--features", "gpu"]),
)


def load(path):
    specification = importlib.util.spec_from_file_location("compile_then_swap_under_probe", path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def features_words(arguments):
    words = arguments.split()
    return [following for word, following in zip(words, words[1:]) if word == "--features"]


def probe_one(module, work, label, extra):
    case_directory = os.path.join(work, label)
    os.makedirs(case_directory)
    fixture = module.SelftestFixture(case_directory)
    saved = {name: os.environ.get(name) for name in ("PATH", module.FAKE_CARGO_LOG_VARIABLE)}
    try:
        os.environ["PATH"] = fixture.fake_bin + os.pathsep + (saved["PATH"] or "")
        os.environ[module.FAKE_CARGO_LOG_VARIABLE] = fixture.cargo_log
        draft = fixture.draft(module.GOOD_BIN_CHANGED)
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code = module.main([draft, fixture.target, "--scratch", fixture.scratch, "--root", fixture.root, *extra])
        calls = fixture.cargo_calls()
        per_call = [features_words(arguments) for _, _, arguments in calls]
        if not calls:
            gpu = "none"
        else:
            gpu = "yes" if all(any("gpu" in value.split(",") for value in words) for words in per_call) else "no"
        shown = "|".join(",".join(words) or "-" for words in per_call) or "-"
        print(f"name=cli_features spelling={label} rc={code} calls={len(calls)} gpu_in_every_call={gpu} features_words={shown}")
    finally:
        for name, value in saved.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--script", required=True)
    parser.add_argument("--work", required=True)
    parser.add_argument("--selftest", action="store_true")
    options = parser.parse_args()
    if os.environ.get("COMPILE_THEN_SWAP_BREAK"):
        print("  ✗ 环境里设着 COMPILE_THEN_SWAP_BREAK：去掉再跑", file=sys.stderr)
        return 2
    if os.path.exists(options.work) and os.listdir(options.work):
        print(f"  ✗ --work {options.work} 已存在且非空：给一个新的", file=sys.stderr)
        return 2
    os.makedirs(options.work, exist_ok=True)
    work = os.path.realpath(options.work)
    module = load(os.path.realpath(options.script))
    for label, extra in SPELLINGS:
        probe_one(module, work, label, extra)
    if options.selftest:
        print(f"name=selftest_in_process rc={module.selftest()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
