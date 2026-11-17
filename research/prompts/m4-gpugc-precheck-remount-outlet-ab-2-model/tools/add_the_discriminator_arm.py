#!/usr/bin/env python3
"""在已经由一族自己的 patch_the_copy.py 拼好的拷贝上，再拼卸载重挂出路的判别子那一臂（只动拷贝，不动冻结副本与仓）。

用法：add_the_discriminator_arm.py <拷贝的仓根> <模型目录>
做的事（原文要在拷贝里恰好命中一次，命中不是一次就退 1、一个文件都不写）：
  1. crates/singlefs-core/src/mounted_session.rs：
     push_one_floor_raise_for_a_refusal_for_space 里抬 F 之前先算那一步（judge_before_pushing_a_floor_raise_for_a_refusal）
     先接住结局，交回 NoSpaceEvenWithEverythingAFloorRaiseCouldRelease 时调
     remount_outlet_probe::note_the_discriminators_at_the_refusal(参数, 盘表, 分配器, 现行那一版, 影子账, 剩下的改动, …) 记一条，
     再照原样交回；文件末尾拼上 <模型目录>/core/remount_outlet_core_module.rs。
  2. 把 <模型目录>/common/discriminator.rs 拷进 crates/refusal-precheck-oracle-probe/src/。
退出码：0 拼好；1 原文命中不是一次、拷贝里还没有探针 crate 或已经拼过；2 用法错。
"""
import pathlib
import shutil
import sys

JUDGE_CALL_ORIGINAL = (
    "    let (published_in_this_change, refusal) = judge_before_pushing_a_floor_raise_for_a_refusal(\n"
    "        allocator,\n"
    "        refused_publish,\n"
    "        published_in_this_change,\n"
    "        refusal,\n"
    "    )?;\n"
)
JUDGE_CALL_PATCHED = (
    "    let judged = judge_before_pushing_a_floor_raise_for_a_refusal(\n"
    "        allocator,\n"
    "        refused_publish,\n"
    "        published_in_this_change,\n"
    "        refusal,\n"
    "    );\n"
    "    if let Err(UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space)) =\n"
    "        &judged\n"
    "    {\n"
    "        remount_outlet_probe::note_the_discriminators_at_the_refusal(\n"
    "            parameters,\n"
    "            &devices[..],\n"
    "            allocator,\n"
    "            &**current,\n"
    "            shadow_ledger,\n"
    "            remainder,\n"
    "            no_space,\n"
    "        );\n"
    "    }\n"
    "    let (published_in_this_change, refusal) = judged?;\n"
)
SERIES_PUT_BACK_MARK = "pub fn put_back_series("
CRATE_NAME = "refusal-precheck-oracle-probe"
MODULE_MARK = "pub mod remount_outlet_probe {"


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
    model = pathlib.Path(sys.argv[2])
    session_path = copy / "crates/singlefs-core/src/mounted_session.rs"
    crate_sources = copy / "crates" / CRATE_NAME / "src"
    if not session_path.is_file() or not crate_sources.is_dir():
        print(f"  ✗ {copy} 里没有 mounted_session.rs 或探针 crate：先跑那一族的 patch_the_copy.py", file=sys.stderr)
        sys.exit(1)
    session = session_path.read_text(encoding="utf-8")
    if MODULE_MARK in session:
        print(f"  ✗ {session_path} 已经拼过判别子那一臂：给一份新拷贝", file=sys.stderr)
        sys.exit(1)
    if session.count(SERIES_PUT_BACK_MARK) != 1:
        print(f"  ✗ {session_path} 里推串记录放回的函数命中 {session.count(SERIES_PUT_BACK_MARK)} 次：那一族的 core_probe_module.rs 不是这一份模型的", file=sys.stderr)
        sys.exit(1)
    session = replace_once(session, JUDGE_CALL_ORIGINAL, JUDGE_CALL_PATCHED, "推串那一处抬 F 之前先算")
    session += (model / "core" / "remount_outlet_core_module.rs").read_text(encoding="utf-8")
    discriminator_source = model / "common" / "discriminator.rs"
    if not discriminator_source.is_file():
        print(f"  ✗ {discriminator_source} 不在", file=sys.stderr)
        sys.exit(1)
    session_path.write_text(session, encoding="utf-8")
    shutil.copyfile(discriminator_source, crate_sources / "discriminator.rs")
    print(f"ADD_THE_DISCRIMINATOR_ARM copy={copy} model={model}")


if __name__ == "__main__":
    main()
