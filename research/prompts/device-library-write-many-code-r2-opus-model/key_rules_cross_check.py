#!/usr/bin/env python3
"""攻方腿模型（device-library-write-many-code-r2）：装置一侧的相对键规则（lib.rs 的 is_relative_key_the_relay_accepts，由攻方测试
listing_of_what_the_key_rule_accepts 跑出一份清单）与转交一侧的 RELATIVE_KEY_PATTERN（device-code-key.py 原样载入）逐个比：
全部 Unicode 标量（单个字符的键）与清单里的几条边界键。两份实现各跑各的，比的是跑出来的结果，不是读代码。
用法：python3 -I key_rules_cross_check.py --repo <仓根> --listing <Rust 那一侧写的清单>
打 name=single_characters rust_accepted=<个> python_accepted=<个> differ=<个> [first_differ=<码位>]、每条边界键一行 name=sample、末行 name=summary differ=<总数>。"""
import argparse
import hashlib
import importlib.util
import os
import sys


def load_pattern(repository):
    path = os.path.join(repository, "scripts", "experiment-apparatus", "device-code-key.py")
    spec = importlib.util.spec_from_file_location("device_code_key_under_attack", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.RELATIVE_KEY_PATTERN, path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--listing", required=True)
    options = parser.parse_args()
    pattern, path = load_pattern(options.repo)
    with open(path, "rb") as handle:
        print(f"name=relay_source path={path} sha256={hashlib.sha256(handle.read()).hexdigest()} pattern={pattern.pattern}")
    rust_singles, samples = set(), []
    with open(options.listing, encoding="utf-8") as handle:
        for line in handle.read().split("\n"):
            if line.startswith("sample "):
                _word, hexadecimal, accepted = line.split(" ")
                samples.append(("" if hexadecimal == "-" else bytes.fromhex(hexadecimal).decode("utf-8"), accepted == "true"))
            elif line:
                rust_singles.add(int(line, 16))
    python_singles = {code for code in range(0x110000) if not 0xD800 <= code <= 0xDFFF and pattern.fullmatch(chr(code))}
    differ = sorted(rust_singles ^ python_singles)
    print(f"name=single_characters rust_accepted={len(rust_singles)} python_accepted={len(python_singles)} differ={len(differ)}"
          + (f" first_differ={differ[0]:x}" if differ else ""))
    total = len(differ)
    for key, rust_accepted in samples:
        python_accepted = pattern.fullmatch(key) is not None
        total += rust_accepted != python_accepted
        shown = key if len(key) <= 16 else f"{key[:8]}…（{len(key)} 字符）"
        print(f"name=sample key={shown!r} rust={rust_accepted} python={python_accepted} same={rust_accepted == python_accepted}")
    print(f"name=summary differ={total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
