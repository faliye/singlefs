#!/usr/bin/env python3
"""d137-ledger-defs-r3 攻方腿：compile-then-swap.py 在执行员定义写死的用法（--release 加特性）下，命令行上的参数各自交给假 cargo 什么。

用法：python3 release-features-probe.py --script <compile-then-swap.py 的路径> --work <不在或空的目录> [--selftest]
做法：照脚本自己的 SelftestFixture 建临时仓与假 cargo，从 main()（最外层，连 refuse_break_outside_selftest 一起）起，不起真编译；
  --selftest 再在进程里调一次脚本自己的 selftest()（它照样打自己的行），看自证分不分得出被探的这几种用法。
输出：每种写法一行
  name=cli_release_features spelling=<标签> rc=<main 的退出码> calls=<假 cargo 被调几次> gpu_in_every_call=<yes|no|none> release_in_every_call=<yes|no|none>
  --selftest 再打一行 name=selftest_in_process rc=<selftest() 的返回值>。
退出码：0 跑完（不判对错，期望在模型目录 out/expected.txt）；2 用法错、--work 非空或环境里设着弄坏开关。
"""
import argparse
import contextlib
import importlib.util
import io
import os
import sys

SPELLINGS = (
    ("release-features-gpu", ["--release", "--features", "gpu"]),
    ("release-features-equals-gpu", ["--release", "--features=gpu"]),
    ("release-short-flag-gpu", ["--release", "-F", "gpu"]),
    ("features-gpu-without-release", ["--features", "gpu"]),
)


def load(path):
    specification = importlib.util.spec_from_file_location("compile_then_swap_under_release_probe", path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def summary(per_call, wanted):
    if not per_call:
        return "none"
    return "yes" if all(wanted(words) for words in per_call) else "no"


def carries_gpu(words):
    if "--all-features" in words:
        return True
    return any(word == "--features" and "gpu" in following.split(",") for word, following in zip(words, words[1:]))


def probe_one(module, work, label, extra):
    case_directory = os.path.join(work, label)
    os.makedirs(case_directory)
    fixture = module.SelftestFixture(case_directory)
    saved = {name: os.environ.get(name) for name in ("PATH", module.FAKE_CARGO_LOG_VARIABLE)}
    try:
        os.environ["PATH"] = fixture.fake_bin + os.pathsep + (saved["PATH"] or "")
        os.environ[module.FAKE_CARGO_LOG_VARIABLE] = fixture.cargo_log
        draft = fixture.draft(module.GOOD_BIN_CHANGED)
        with contextlib.redirect_stdout(io.StringIO()):
            code = module.main([draft, fixture.target, "--scratch", fixture.scratch, "--root", fixture.root, *extra])
        per_call = [arguments.split() for _, _, arguments in fixture.cargo_calls()]
        print(f"name=cli_release_features spelling={label} rc={code} calls={len(per_call)} "
              f"gpu_in_every_call={summary(per_call, carries_gpu)} release_in_every_call={summary(per_call, lambda words: '--release' in words)}")
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
