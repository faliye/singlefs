"""挂特性的测试目标怎么列：门禁 crash-point-replay 的 feature-gated-tests 格与 product-gpu-real-card-tests 格各要一张表，认法只在这里一份。

测试目标的源文件怎么认（平铺的 tests/<名>.rs；目录式的 tests/<名>/main.rs 连同它 mod 声明到的子模块）在 lib_test_target_sources.py。
一个测试目标的「根文件名」：平铺的是 <名>.rs，目录式的是 <名>/main.rs（相对 tests/）。

用法（调它的只有门禁 crash-point-replay.sh 的 list_feature_gated_test_files 与 product_gpu_target_rows）：
  lib_feature_gated_targets.py feature-gated <tests 目录> [--top-level-only]   （同一份判法的函数形态：feature_gated_targets）
      每个测试目标里 #[cfg(…)] / #![cfg(…)] 点到的特性，一个测试目标一行「根文件名<TAB>whole-file|items<TAB>特性,特性」（根文件里有 #![cfg(…)] 点到特性的记 whole-file；
      子模块里的 #![cfg(…)] 只管那个子模块，不算），照根文件名排序；挂特性的一个都没有时不打。读不了的抛异常（退 1，traceback 在标准错误）。
  lib_feature_gated_targets.py product-gpu <包目录> <特性> [--top-level-only]
      这个包里开了这个特性才编的测试目标，一个一行「目标名<TAB>根文件（包目录起）」，照目标名排序：Cargo.toml 的 [[test]] 里 required-features 有这个特性的
      （根文件取它的 path，没写取 tests/<名>.rs 或 tests/<名>/main.rs），加 tests/ 里 cfg 写了 feature = "<特性>" 的测试目标。
      退出码：0 有（逐行打）、3 包不在（没有 Cargo.toml）、4 包的 [features] 里没有这个特性、5 包有这个特性而没有测试目标挂它、6 读不了（原因打在标准错误）。
  --top-level-only：只认 tests/ 顶层的 .rs，目录式测试目标当不存在（门禁 crash-point-replay 的弄坏开关 directory-target-listing-unseen）。
  // 起头的行（注释与文档注释）不读；括号按层数配对（rustfmt 把长的 cfg 折成几行也认）。
  退出码 2：用法不对。
"""
import os
import pathlib
import re
import sys
import tomllib

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import lib_test_target_sources as target_sources  # noqa: E402

ATTRIBUTE_START = re.compile(r"#(!?)\[\s*cfg\s*\(")
FEATURE_NAME = re.compile(r"\bfeature\s*=\s*\"([^\"]+)\"")


def root_file_names(tests_directory, top_level_only):
    """tests_directory 里每个测试目标的根文件名，照名排序：平铺的 <名>.rs，目录式的 <名>/main.rs。"""
    names = [path.name for path in tests_directory.glob("*.rs")]
    if not top_level_only:
        names += [f"{path.parent.name}/main.rs" for path in tests_directory.glob("*/main.rs")]
    return sorted(names)


def target_texts(tests_directory, root_file_name):
    """这个测试目标每份源文件里不以 // 起头的行连成的文本，次序照 target_source_files（根文件排头）。"""
    target = root_file_name.removesuffix("/main.rs").removesuffix(".rs")
    return ["\n".join(line for line in pathlib.Path(source).read_text(encoding="utf-8").splitlines() if not line.lstrip().startswith("//"))
            for source in target_sources.target_source_files(str(tests_directory), target)]


def cfg_features(text):
    """text 里 #[cfg(…)] / #![cfg(…)] 点到的特性，与其中有没有点到特性的 #![cfg(…)]。→ (特性集合, 有没有)"""
    features, inner = set(), False
    for match in ATTRIBUTE_START.finditer(text):
        depth, end = 1, match.end()
        while end < len(text) and depth > 0:
            depth += {"(": 1, ")": -1}.get(text[end], 0)
            end += 1
        named = FEATURE_NAME.findall(text[match.end():end])
        features.update(named)
        inner = inner or bool(named and match.group(1) == "!")
    return features, inner


def feature_gated_targets(tests_directory, top_level_only=False):
    """tests_directory（pathlib.Path）里挂特性的测试目标：[(根文件名, "whole-file" 或 "items", [特性…])]，照根文件名排序；一个都没有交空表。"""
    found = []
    for name in root_file_names(tests_directory, top_level_only):
        features, whole_file = set(), False
        for position, text in enumerate(target_texts(tests_directory, name)):
            named, inner = cfg_features(text)
            features |= named
            whole_file = whole_file or (inner and position == 0)
        if features:
            found.append((name, "whole-file" if whole_file else "items", sorted(features)))
    return found


def product_gpu_rows(package_directory, feature, top_level_only=False):
    """product-gpu 子命令：→ (退出码, 行)。"""
    manifest = package_directory / "Cargo.toml"
    if not manifest.is_file():
        return 3, []
    try:
        data = tomllib.loads(manifest.read_text(encoding="utf-8"))
        if feature not in (data.get("features") or {}):
            return 4, []
        tests_directory, targets = package_directory / "tests", {}
        for entry in data.get("test") or []:
            if isinstance(entry, dict) and entry.get("name") and feature in (entry.get("required-features") or []):
                root_file = target_sources.target_root_file(str(tests_directory), entry["name"])
                targets[entry["name"]] = entry.get("path") or (os.path.relpath(root_file, package_directory) if root_file else f"tests/{entry['name']}.rs")
        attribute = re.compile(r'\bfeature\s*=\s*"' + re.escape(feature) + '"')
        for name in root_file_names(tests_directory, top_level_only):
            if attribute.search("\n".join(target_texts(tests_directory, name))):
                targets.setdefault(name.removesuffix("/main.rs").removesuffix(".rs"), f"tests/{name}")
    except (OSError, ValueError) as error:
        print(f"{type(error).__name__}: {error}", file=sys.stderr)
        return 6, []
    return (0, [f"{name}\t{source}" for name, source in sorted(targets.items())]) if targets else (5, [])


def main(arguments):
    top_level_only = "--top-level-only" in arguments
    words = [word for word in arguments if word != "--top-level-only"]
    if words[:1] == ["feature-gated"] and len(words) == 2:
        code, rows = 0, [f"{name}\t{scope}\t{','.join(features)}" for name, scope, features in feature_gated_targets(pathlib.Path(words[1]), top_level_only)]
    elif words[:1] == ["product-gpu"] and len(words) == 3:
        code, rows = product_gpu_rows(pathlib.Path(words[1]), words[2], top_level_only)
    else:
        print("  ✗ 用法不对\n  → 怎么办：lib_feature_gated_targets.py feature-gated <tests 目录> [--top-level-only] | product-gpu <包目录> <特性> [--top-level-only]", file=sys.stderr)
        return 2
    for row in rows:
        print(row)
    return code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
