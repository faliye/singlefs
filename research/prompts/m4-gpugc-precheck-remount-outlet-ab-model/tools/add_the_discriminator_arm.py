#!/usr/bin/env python3
"""在已经由一族自己的 patch_the_copy.py 拼好的拷贝上，再拼卸载重挂出路的判别子那一臂（只动拷贝，不动冻结副本与仓）。

用法：add_the_discriminator_arm.py <拷贝的仓根> <模型目录>
做的事（原文要在拷贝里恰好命中一次，命中不是一次就退 1、一个文件都不写）：
  1. crates/singlefs-core/src/mounted_session.rs：
     judge_before_pushing_a_floor_raise_for_a_refusal 改名成 ..._as_in_the_frozen_copy，原名换成一层外壳：
     照调改名的那一份，交回 NoSpaceEvenWithEverythingAFloorRaiseCouldRelease 时调
     remount_outlet_probe::note_the_discriminators_at_the_refusal(allocator, …) 记一条，结局原样交回；
     文件末尾拼上 <模型目录>/core/remount_outlet_core_module.rs。
  2. 把 <模型目录>/common/discriminator.rs 拷进 crates/refusal-precheck-oracle-probe/src/。
退出码：0 拼好；1 原文命中不是一次、拷贝里还没有探针 crate 或已经拼过；2 用法错。
"""
import pathlib
import shutil
import sys

JUDGE_ORIGINAL = (
    "fn judge_before_pushing_a_floor_raise_for_a_refusal(\n"
    "    allocator: &PoolAllocator,\n"
    "    refused_publish: RefusedPublishUnderTheEmptyClusterSegmentsConjunct,\n"
    "    published_in_this_change: PublishedInThisChange,\n"
    "    refusal: PublishError,\n"
    ") -> Result<(PublishedInThisChange, PublishError), UserChangeRefused> {\n"
    "    let devices_short =\n"
)
JUDGE_PATCHED = (
    "fn judge_before_pushing_a_floor_raise_for_a_refusal(\n"
    "    allocator: &PoolAllocator,\n"
    "    refused_publish: RefusedPublishUnderTheEmptyClusterSegmentsConjunct,\n"
    "    published_in_this_change: PublishedInThisChange,\n"
    "    refusal: PublishError,\n"
    ") -> Result<(PublishedInThisChange, PublishError), UserChangeRefused> {\n"
    "    let judged = judge_before_pushing_a_floor_raise_for_a_refusal_as_in_the_frozen_copy(\n"
    "        allocator,\n"
    "        refused_publish,\n"
    "        published_in_this_change,\n"
    "        refusal,\n"
    "    );\n"
    "    if let Err(UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space)) =\n"
    "        &judged\n"
    "    {\n"
    "        remount_outlet_probe::note_the_discriminators_at_the_refusal(allocator, no_space);\n"
    "    }\n"
    "    judged\n"
    "}\n"
    "\n"
    "fn judge_before_pushing_a_floor_raise_for_a_refusal_as_in_the_frozen_copy(\n"
    "    allocator: &PoolAllocator,\n"
    "    refused_publish: RefusedPublishUnderTheEmptyClusterSegmentsConjunct,\n"
    "    published_in_this_change: PublishedInThisChange,\n"
    "    refusal: PublishError,\n"
    ") -> Result<(PublishedInThisChange, PublishError), UserChangeRefused> {\n"
    "    let devices_short =\n"
)
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
    session = replace_once(session, JUDGE_ORIGINAL, JUDGE_PATCHED, "抬 F 之前先算的入口")
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
