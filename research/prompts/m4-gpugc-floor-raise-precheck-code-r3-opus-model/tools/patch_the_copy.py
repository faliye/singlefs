#!/usr/bin/env python3
"""把抬 F 先算代码轮第三轮攻方腿的探针拼进冻结副本的一份拷贝（只动拷贝，不动冻结副本与仓）。

用法：patch_the_copy.py <拷贝的仓根> <模型目录里的 probe 目录>
做的事（每一处原文都要在拷贝里恰好命中一次，命中不是一次就退 1、一个文件都不写）：
  1. crates/singlefs-core/src/mounted_session.rs：
     先算每进一次记一笔（floor_raise_precheck_r3_probe::note_refusal，记那一次拒绝的成员名）；
     读一块被点名的盘那一处改调 floor_raise_precheck_r3_probe::judged（照代码判一次、记下读数与上界，再按开关交结局；判法本身不动）；
     文件末尾拼上 probe/core_probe_module.rs。
  2. Cargo.toml：工作区成员加 crates/floor-raise-precheck-r3-probe。
  3. 把 probe/Cargo.toml 与 probe/src/ 拷成 crates/floor-raise-precheck-r3-probe/。
退出码：0 拼好；1 原文命中不是一次或拷贝不像冻结副本；2 用法错。
"""
import pathlib
import shutil
import sys

REFUSAL_ORIGINAL = (
    "    let PublishError::SpaceAdmissionRefused(refused) = refusal else {\n"
    "        return Vec::new();\n"
    "    };\n"
)
REFUSAL_PATCHED = "    floor_raise_precheck_r3_probe::note_refusal(refusal);\n" + REFUSAL_ORIGINAL
JUDGED_ORIGINAL = (
    "            device_short_even_with_everything_a_floor_raise_could_release(\n"
    "                short,\n"
    "                SlotsAFloorRaiseCouldRelease::of_the_device(device_map),\n"
    "            )\n"
)
JUDGED_PATCHED = (
    "            floor_raise_precheck_r3_probe::judged(\n"
    "                short,\n"
    "                SlotsAFloorRaiseCouldRelease::of_the_device(device_map),\n"
    "            )\n"
)
MEMBER_ORIGINAL = '    "crates/singlefs-mount-runtime",\n'
MEMBER_PATCHED = MEMBER_ORIGINAL + '    "crates/floor-raise-precheck-r3-probe",\n'
CRATE_NAME = "floor-raise-precheck-r3-probe"


def replace_once(text, original, patched, where):
    count = text.count(original)
    if count != 1:
        print(f"  ✗ {where}：原文命中 {count} 次，要恰好 1 次", file=sys.stderr)
        sys.exit(1)
    return text.replace(original, patched)


def main():
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    copy = pathlib.Path(sys.argv[1])
    probe = pathlib.Path(sys.argv[2])
    session_path = copy / "crates/singlefs-core/src/mounted_session.rs"
    manifest_path = copy / "Cargo.toml"
    if not session_path.is_file() or not manifest_path.is_file():
        print(f"  ✗ {copy} 不像冻结副本的拷贝（缺 mounted_session.rs 或 Cargo.toml）", file=sys.stderr)
        sys.exit(1)
    session = session_path.read_text(encoding="utf-8")
    session = replace_once(session, REFUSAL_ORIGINAL, REFUSAL_PATCHED, "先算每进一次")
    session = replace_once(session, JUDGED_ORIGINAL, JUDGED_PATCHED, "读一块被点名的盘")
    session += (probe / "core_probe_module.rs").read_text(encoding="utf-8")
    manifest = replace_once(
        manifest_path.read_text(encoding="utf-8"), MEMBER_ORIGINAL, MEMBER_PATCHED, "工作区成员"
    )
    crate_directory = copy / "crates" / CRATE_NAME
    if crate_directory.exists():
        print(f"  ✗ {crate_directory} 已在：给一份新拷贝", file=sys.stderr)
        sys.exit(1)
    session_path.write_text(session, encoding="utf-8")
    manifest_path.write_text(manifest, encoding="utf-8")
    (crate_directory / "src").mkdir(parents=True)
    shutil.copyfile(probe / "Cargo.toml", crate_directory / "Cargo.toml")
    sources = sorted((probe / "src").glob("*.rs"))
    for source in sources:
        shutil.copyfile(source, crate_directory / "src" / source.name)
    print(f"PATCH_THE_COPY copy={copy} probe_sources={len(sources)}")


if __name__ == "__main__":
    main()
