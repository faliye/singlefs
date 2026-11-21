#!/usr/bin/env python3
"""cargo metadata --no-deps 的输出 → 每个包的每个目标一行：name=cargo_target case=<样本> package=<包> target=<目标名> kind=<种类> src=<相对样本根的源文件>。
用法：list_targets.py <样本名> <样本根> <metadata 的 JSON 文件>"""
import json
import os
import sys


def main(arguments):
    if len(arguments) != 3:
        print("用法：list_targets.py <样本名> <样本根> <metadata 的 JSON 文件>", file=sys.stderr)
        return 2
    case, root, path = arguments
    root = os.path.realpath(root)
    with open(path, encoding="utf-8") as handle:
        metadata = json.load(handle)
    count = 0
    for package in metadata.get("packages", []):
        for target in package.get("targets", []):
            count += 1
            source = os.path.relpath(os.path.realpath(target.get("src_path", "")), root)
            print(f"name=cargo_target case={case} package={package.get('name')} target={target.get('name')} "
                  f"kind={','.join(target.get('kind', []))} src={source}")
    print(f"name=cargo_targets case={case} packages={len(metadata.get('packages', []))} targets={count}")
    return 0 if count else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
