#!/usr/bin/env python3
"""把设计三方 m4-gpu-wait-outside-the-lock-design-r2 辩方腿的探针拼进冻结副本的一份拷贝（只动拷贝，不动冻结副本与仓）。

用法：patch_the_copy.py <拷贝的仓根> <模型目录>
做的事（核心层与运行库一行不改，探针只调它们的公开入口）：
  1. Cargo.toml：工作区成员加 crates/gpu-wait-outside-the-lock-r2-probe（原文要在拷贝里恰好命中一次）。
  2. 把 <模型目录>/probe/Cargo.toml 与 probe/src/ 拷成 crates/gpu-wait-outside-the-lock-r2-probe/。
  3. 把 <模型目录>/probes/ 下的 .rs 拷进 crates/singlefs-mount-runtime/tests/（运行库的测试目标，和同目录的 common_pool_on_memory_devices 一起编）。
退出码：0 拼好；1 原文命中不是一次或拷贝不像冻结副本；2 用法错。
"""
import pathlib
import shutil
import sys

MEMBER_ORIGINAL = '    "crates/singlefs-mount-runtime",\n'
MEMBER_PATCHED = MEMBER_ORIGINAL + '    "crates/gpu-wait-outside-the-lock-r2-probe",\n'
CRATE_NAME = "gpu-wait-outside-the-lock-r2-probe"


def main():
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    copy = pathlib.Path(sys.argv[1])
    model = pathlib.Path(sys.argv[2])
    manifest_path = copy / "Cargo.toml"
    tests_directory = copy / "crates/singlefs-mount-runtime/tests"
    if not manifest_path.is_file() or not (copy / "crates/singlefs-core/src/mounted_session.rs").is_file() or not tests_directory.is_dir():
        print(f"  ✗ {copy} 不像冻结副本的拷贝（缺 Cargo.toml、mounted_session.rs 或运行库的 tests 目录）", file=sys.stderr)
        sys.exit(1)
    manifest = manifest_path.read_text(encoding="utf-8")
    count = manifest.count(MEMBER_ORIGINAL)
    if count != 1:
        print(f"  ✗ Cargo.toml：工作区成员那一行命中 {count} 次，要恰好 1 次", file=sys.stderr)
        sys.exit(1)
    manifest_path.write_text(manifest.replace(MEMBER_ORIGINAL, MEMBER_PATCHED), encoding="utf-8")
    destination = copy / "crates" / CRATE_NAME
    if destination.exists():
        print(f"  ✗ {destination} 已在：拷贝不是新拷的", file=sys.stderr)
        sys.exit(1)
    destination.mkdir(parents=True)
    shutil.copy2(model / "probe" / "Cargo.toml", destination / "Cargo.toml")
    shutil.copytree(model / "probe" / "src", destination / "src")
    test_files = sorted((model / "probes").glob("*.rs"))
    for test_file in test_files:
        shutil.copy2(test_file, tests_directory / test_file.name)
    print(f"name=patched member={CRATE_NAME} probe_files={sum(1 for _ in (destination / 'src').iterdir()) + 1} runtime_test_files={len(test_files)}")


if __name__ == "__main__":
    main()
