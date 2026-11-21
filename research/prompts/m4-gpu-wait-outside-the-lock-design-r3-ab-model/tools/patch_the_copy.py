#!/usr/bin/env python3
"""把 m4-gpu-wait-outside-the-lock 第三轮判决「三、改法」第 5、7 条的 A/B 探针拼进仓副本的一份拷贝（只动拷贝，不动仓副本与仓）。

用法：patch_the_copy.py <拷贝的仓根> <模型目录>
做的事：
  1. Cargo.toml：工作区成员加 crates/gpu-wait-outside-the-lock-r3-ab-probe（原文要在拷贝里恰好命中一次）。
  2. crates/singlefs-core/src/mounted_session.rs 末尾追加 <模型目录>/core-patch/probe_replan_after_the_window.rs（一个 pub mod，
     经 `use super::*` 调同文件里不公开的几步；文件里原有的代码一个字节不改）。追加之前核它要调的那几个定义在拷贝里各恰好一处、
     这个模块名还不在。
  3. 把 <模型目录>/probe/Cargo.toml 与 probe/src/ 拷成 crates/gpu-wait-outside-the-lock-r3-ab-probe/。
退出码：0 拼好；1 原文命中不是一次、要调的定义缺或多、拷贝不像仓副本；2 用法错。
"""
import hashlib
import pathlib
import shutil
import sys

MEMBER_ORIGINAL = '    "crates/singlefs-mount-runtime",\n'
CRATE_NAME = "gpu-wait-outside-the-lock-r3-ab-probe"
MEMBER_PATCHED = MEMBER_ORIGINAL + f'    "crates/{CRATE_NAME}",\n'
MODULE_NAME = "probe_replan_after_the_window"
# 追加的模块经 `use super::*` 调的、mounted_session.rs 里不公开的定义：每一样在拷贝里要恰好一处。
DEFINITIONS_THE_PATCH_CALLS = [
    "fn advance_compaction_by_one_publish_computing_on<",
    "fn compaction_publish_planned_with_the_records_uncertain_this_round(",
    "fn publish_the_planned_compaction_step_falling_back_on_the_region_check<",
    "fn pause_by_dropping_the_live_intent_or_hand_up<",
    "fn publish_the_compaction_step_once<",
    "fn stop_of_the_rollback_gate<",
    "fn version_as_the_compaction_reads_it(",
    "enum CompactionComputeOfTheBatch<",
    "enum CompactionBatchNotCompleted {",
    "enum CompactionStep {",
    "enum CompactionPublishKind {",
    "enum CompactionRunReach {",
]


def fail(message):
    print(f"  ✗ {message}", file=sys.stderr)
    sys.exit(1)


def main():
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    copy = pathlib.Path(sys.argv[1])
    model = pathlib.Path(sys.argv[2])
    manifest_path = copy / "Cargo.toml"
    session_path = copy / "crates/singlefs-core/src/mounted_session.rs"
    core_patch_path = model / "core-patch" / f"{MODULE_NAME}.rs"
    if not manifest_path.is_file() or not session_path.is_file() or not (copy / "crates/singlefs-checker/src/walk.rs").is_file():
        fail(f"{copy} 不像仓副本的拷贝（缺 Cargo.toml、mounted_session.rs 或 checker 的 walk.rs）")
    if not core_patch_path.is_file() or not (model / "probe" / "Cargo.toml").is_file():
        fail(f"{model} 不像模型目录（缺 core-patch/{MODULE_NAME}.rs 或 probe/Cargo.toml）")
    manifest = manifest_path.read_text(encoding="utf-8")
    count = manifest.count(MEMBER_ORIGINAL)
    if count != 1:
        fail(f"Cargo.toml：工作区成员那一行命中 {count} 次，要恰好 1 次")
    session = session_path.read_text(encoding="utf-8")
    for definition in DEFINITIONS_THE_PATCH_CALLS:
        hits = session.count(definition)
        if hits != 1:
            fail(f"mounted_session.rs：「{definition}」命中 {hits} 次，要恰好 1 次（核心层改过，core-patch 要跟着改）")
    if f"mod {MODULE_NAME}" in session:
        fail(f"mounted_session.rs 里已有 mod {MODULE_NAME}：拷贝不是新拷的")
    destination = copy / "crates" / CRATE_NAME
    if destination.exists():
        fail(f"{destination} 已在：拷贝不是新拷的")
    original_sha256 = hashlib.sha256(session.encode("utf-8")).hexdigest()
    core_patch = core_patch_path.read_text(encoding="utf-8")
    manifest_path.write_text(manifest.replace(MEMBER_ORIGINAL, MEMBER_PATCHED), encoding="utf-8")
    session_path.write_text(session + core_patch, encoding="utf-8")
    destination.mkdir(parents=True)
    shutil.copy2(model / "probe" / "Cargo.toml", destination / "Cargo.toml")
    shutil.copytree(model / "probe" / "src", destination / "src")
    probe_files = sum(1 for _ in (destination / "src").iterdir()) + 1
    print(
        f"name=patched member={CRATE_NAME} probe_files={probe_files} "
        f"mounted_session_original_sha256={original_sha256} core_patch_lines={core_patch.count(chr(10))}"
    )


if __name__ == "__main__":
    main()
