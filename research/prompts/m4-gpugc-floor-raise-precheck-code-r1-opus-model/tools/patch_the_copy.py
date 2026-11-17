#!/usr/bin/env python3
"""把抬 F 先算的探针拼进冻结副本的一份拷贝（只动拷贝，不动冻结副本与仓）。

用法：patch_the_copy.py <拷贝的仓根> <模型目录里的 probe 目录>
做的事（每一处原文都要在拷贝里恰好命中一次，命中不是一次就退 1、一个文件都不写）：
  1. crates/singlefs-core/src/mounted_session.rs：
     式子那一判的上界读探针的「上界不算 defer」开关（只给自证用）；
     先算判出放不下时先记一笔，探针旁路开着就照推；每一串推成前后各读一次；
     文件末尾拼上 probe/core_probe_module.rs。
  2. Cargo.toml：工作区成员加 crates/floor-raise-precheck-probe。
  3. 把 probe/Cargo.toml 与 probe/src/ 拷成 crates/floor-raise-precheck-probe/。
退出码：0 拼好；1 原文命中不是一次或拷贝不像冻结副本；2 用法错。
"""
import pathlib
import shutil
import sys

BOUND_ORIGINAL = (
    "                short.available.0 + bytes_of_slots(slots.deferred) + bytes_of_slots(slots.isolated),\n"
)
BOUND_PATCHED = (
    "                short.available.0\n"
    "                    + bytes_of_slots(if floor_raise_precheck_probe::bound_without_deferred() {\n"
    "                        0\n"
    "                    } else {\n"
    "                        slots.deferred\n"
    "                    })\n"
    "                    + bytes_of_slots(slots.isolated),\n"
)
PRECHECK_ORIGINAL = (
    "    if !devices_short.is_empty() {\n"
    "        return Err(no_space_even_with_everything_a_floor_raise_could_release(\n"
)
PRECHECK_PATCHED = (
    "    floor_raise_precheck_probe::note_precheck(&refusal, allocator, !devices_short.is_empty());\n"
    "    if !devices_short.is_empty() && !floor_raise_precheck_probe::bypass() {\n"
    "        return Err(no_space_even_with_everything_a_floor_raise_could_release(\n"
)
RAISE_ORIGINAL = (
    "    match push_one_floor_raise_within_the_admission_budget(\n"
    "        parameters,\n"
    "        devices,\n"
    "        allocator,\n"
    "        Arc::make_mut(current),\n"
    "        shadow_ledger,\n"
    "        publishes_pushed_in_this_admission,\n"
    "    ) {\n"
    "        Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => {\n"
    "            let publishes_of_this_raise = raised.publishes.len();\n"
)
RAISE_PATCHED = (
    "    let observed_before_the_raise = floor_raise_precheck_probe::observe(allocator, &**current);\n"
    + RAISE_ORIGINAL
    + "            floor_raise_precheck_probe::note_raise(\n"
    "                observed_before_the_raise,\n"
    "                allocator,\n"
    "                &**current,\n"
    "                publishes_of_this_raise,\n"
    "            );\n"
)
MEMBER_ORIGINAL = '    "crates/singlefs-mount-runtime",\n'
MEMBER_PATCHED = MEMBER_ORIGINAL + '    "crates/floor-raise-precheck-probe",\n'


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
    session = replace_once(session, BOUND_ORIGINAL, BOUND_PATCHED, "式子那一判的上界")
    session = replace_once(session, PRECHECK_ORIGINAL, PRECHECK_PATCHED, "先算判出放不下")
    session = replace_once(session, RAISE_ORIGINAL, RAISE_PATCHED, "推一串抬 F")
    session += (probe / "core_probe_module.rs").read_text(encoding="utf-8")
    manifest = replace_once(
        manifest_path.read_text(encoding="utf-8"), MEMBER_ORIGINAL, MEMBER_PATCHED, "工作区成员"
    )
    crate_directory = copy / "crates/floor-raise-precheck-probe"
    if crate_directory.exists():
        print(f"  ✗ {crate_directory} 已在：给一份新拷贝", file=sys.stderr)
        sys.exit(1)
    session_path.write_text(session, encoding="utf-8")
    manifest_path.write_text(manifest, encoding="utf-8")
    (crate_directory / "src").mkdir(parents=True)
    shutil.copyfile(probe / "Cargo.toml", crate_directory / "Cargo.toml")
    for source in sorted((probe / "src").glob("*.rs")):
        shutil.copyfile(source, crate_directory / "src" / source.name)
    print(f"PATCH_THE_COPY copy={copy} probe_sources={len(list((probe / 'src').glob('*.rs')))}")


if __name__ == "__main__":
    main()
