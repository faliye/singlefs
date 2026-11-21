#!/usr/bin/env python3
"""把设计三方 m4-gpu-wait-outside-the-lock-design-r3 攻方腿的探针拼进仓副本的一份拷贝（只动拷贝，不动仓副本与仓）。

用法：patch_the_copy.py <拷贝的仓根> <模型目录>
做的事（核心层、checker 与验证包一行不改，探针只调它们的公开入口）：
  1. Cargo.toml：工作区成员加 crates/gpu-wait-outside-the-lock-r3-probe（原文要在拷贝里恰好命中一次）。
  2. 把 <模型目录>/probe/Cargo.toml 与 probe/src/ 拷成 crates/gpu-wait-outside-the-lock-r3-probe/。
退出码：0 拼好；1 原文命中不是一次或拷贝不像仓副本；2 用法错。
"""
import pathlib
import shutil
import sys

MEMBER_ORIGINAL = '    "crates/singlefs-mount-runtime",\n'
MEMBER_PATCHED = MEMBER_ORIGINAL + '    "crates/gpu-wait-outside-the-lock-r3-probe",\n'
CRATE_NAME = "gpu-wait-outside-the-lock-r3-probe"


def main():
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    copy = pathlib.Path(sys.argv[1])
    model = pathlib.Path(sys.argv[2])
    manifest_path = copy / "Cargo.toml"
    if not manifest_path.is_file() or not (copy / "crates/singlefs-core/src/mounted_session.rs").is_file() or not (copy / "crates/singlefs-checker/src/walk.rs").is_file():
        print(f"  ✗ {copy} 不像仓副本的拷贝（缺 Cargo.toml、mounted_session.rs 或 checker 的 walk.rs）", file=sys.stderr)
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
    print(f"name=patched member={CRATE_NAME} probe_files={sum(1 for _ in (destination / 'src').iterdir()) + 1}")


if __name__ == "__main__":
    main()
