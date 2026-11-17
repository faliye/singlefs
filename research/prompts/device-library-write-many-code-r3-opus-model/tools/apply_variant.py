#!/usr/bin/env python3
"""把攻方腿的草稿测试接进 <树> 里判定库口子 lib.rs 的 mod tests 末尾，再照一个变体目录换上一处原文。

用法：apply_variant.py <树> <草稿测试文件> <变体目录或 ->
  变体目录里两份文件：anchor（原文）与 replacement（替换文），原文在 lib.rs 里要恰好命中一次；给 - 只接草稿测试。
退 0 改好了；退 1 lib.rs 不以 mod tests 的收尾 } 结束、或原文不是恰好命中一次（树一个字不改）；退 2 用法错。
"""
import pathlib
import sys

LIBRARY_SOURCE = pathlib.Path("crates/singlefs-device-library/src/lib.rs")


def main() -> int:
    if len(sys.argv) != 4:
        print("用法：apply_variant.py <树> <草稿测试文件> <变体目录或 ->", file=sys.stderr)
        return 2
    tree, drafts, variant = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), sys.argv[3]
    target = tree / LIBRARY_SOURCE
    source = target.read_text(encoding="utf-8")
    if not source.endswith("\n}\n"):
        print(f"✗ {LIBRARY_SOURCE} 不以 mod tests 的收尾 }} 结束", file=sys.stderr)
        return 1
    if variant != "-":
        anchor = (pathlib.Path(variant) / "anchor").read_text(encoding="utf-8")
        replacement = (pathlib.Path(variant) / "replacement").read_text(encoding="utf-8")
        hits = source.count(anchor)
        if hits != 1:
            print(f"✗ {variant} 的原文在 {LIBRARY_SOURCE} 里命中 {hits} 次，要恰好一次", file=sys.stderr)
            return 1
        source = source.replace(anchor, replacement, 1)
    source = source[: -len("}\n")] + drafts.read_text(encoding="utf-8") + "}\n"
    target.write_text(source, encoding="utf-8")
    print(f"name=applied variant={pathlib.Path(variant).name if variant != '-' else 'unmodified'} file={LIBRARY_SOURCE}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
