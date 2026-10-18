#!/usr/bin/env python3
"""在草稿目录里从 copy-base 拷出几份变体树，每份只改一处，供 unit-graph key 比键。"""
import pathlib, shutil, subprocess, sys

root = pathlib.Path(sys.argv[1])
base = root / "copy-base"
amp = "crates/singlefs-checker-tier/src/crash_amplification.rs"
store = "crates/singlefs-checker-tier/src/verdict_store.rs"

def replace_once(text, old, new, label):
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: 旧串命中 {count} 次")
    return text.replace(old, new)

variants = {
    # 线程默认登记值从 31 改成 30（资源上限）
    "heavy-thread-default": (".claude/heavy-thread-default", None),
    # 同一份登记文件只改理由（# 之后的说明），数不变
    "heavy-thread-default-reason-only": (".claude/heavy-thread-default", (
        "机器忙时也固定 31、不看忙闲", "机器忙时也固定 31、不看忙闲。")),
    # 甲：停下请求那一行 use 多引一个名字（领块前沿）
    "use-line-of-the-stop-request": (amp, (
        "use singlefs_harness::gpu_memory_ledger::{CardMemoryQueue, StopRequestRememberedInThisRun};",
        "use singlefs_harness::gpu_memory_ledger::{CardMemoryQueue, FrontOfTheOtherHost, StopRequestRememberedInThisRun};")),
    # 录入段驱动：观察者录入循环里换一句判断
    "observer-recording-loop": (amp, (
        "                if !is_observed || !is_mine(this_block) {\n                    continue;\n                }\n                if stop_is_requested() {\n                    tally.stopped_early = true;",
        "                if !is_observed || !is_mine(this_block) {\n                    continue;\n                }\n                if stop_is_requested() || is_mine(u64::MAX) {\n                    tally.stopped_early = true;")),
    # 分工：平滑轮转的取模换一种写法
    "weighted-shares": (amp, (
        "        let position = block_sequence % total;",
        "        let position = block_sequence.rem_euclid(total);")),
    # 核对线程数的读法（判段与录入段共用）
    "checking-threads": (amp, None),
    # 导入别的机器的块
    "import-of-the-other-hosts": (amp, (
        "            if is_mine(this_block) {\n                continue;\n            }\n            let key = point.block_key(&local);\n            for (kind, conclusion_version) in &conclusions_of_the_point {",
        "            if is_mine(this_block) || this_block == u64::MAX {\n                continue;\n            }\n            let key = point.block_key(&local);\n            for (kind, conclusion_version) in &conclusions_of_the_point {")),
    # 库的按计划导入
    "planned-import-in-the-store": (store, (
        "            if !conclusion_keys_seen.insert(conclusion_key_bytes.0.clone()) {\n                continue;\n            }",
        "            if !conclusion_keys_seen.insert(conclusion_key_bytes.0.clone()) || planned.paths.is_empty() && false {\n                continue;\n            }")),
}

def checking_threads_edit(text):
    start = text.index("fn checking_threads() -> usize")
    end = text.index("\n}\n", start)
    body = text[start:end]
    new_body = body.replace("{", "{\n    let _split_between_recording_and_judging = 2usize;", 1)
    return text[:start] + new_body + text[end:]

for name, (relative, edit) in variants.items():
    target = root / f"variant-{name}"
    if target.exists():
        shutil.rmtree(target)
    shutil.copytree(base, target, symlinks=True)
    path = target / relative
    text = path.read_text()
    if name == "heavy-thread-default":
        lines = text.splitlines(keepends=True)
        changed = [line.replace("31", "30", 1) if line.strip() and not line.lstrip().startswith("#") else line for line in lines]
        new = "".join(changed)
        if new == text:
            raise SystemExit("heavy-thread-default: 没改动")
    elif name == "checking-threads":
        new = checking_threads_edit(text)
    else:
        new = replace_once(text, edit[0], edit[1], name)
    path.unlink()
    path.write_text(new)
    print(f"{name}: 改 {relative}")
