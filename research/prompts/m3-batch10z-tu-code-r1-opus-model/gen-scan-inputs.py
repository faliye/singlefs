#!/usr/bin/env python3
"""K4：从冻结副本的两份 checker 档源文件造扫源码的输入：gen-scan-inputs.py <冻结副本根> <输出目录>。"""
import sys
from pathlib import Path

frozen, out = Path(sys.argv[1]), Path(sys.argv[2])
out.mkdir(parents=True, exist_ok=True)
crash = (frozen / "crates/singlefs-checker-tier/src/crash_amplification.rs").read_text()
gpu = (frozen / "crates/singlefs-checker-tier/src/gpu_unit_checks.rs").read_text()
opening = "#[cfg(test)]\nmod tests {"
assert crash.count(opening) == 1
layer_one_line = "pub fn an_extra_layer_never_asking(first_input: u8, stop_is_requested: &(dyn Fn() -> bool + Sync)) -> bool { let _ = stop_is_requested; first_input == 0 }\n"
layer_multi = "pub fn an_extra_layer_never_asking(\n    first_input: u8,\n    stop_is_requested: &(dyn Fn() -> bool + Sync),\n) -> bool {\n    let _ = stop_is_requested;\n    first_input == 0\n}\n"
fake = "/*\n#[cfg(test)]\nmod tests {\n*/\n"
inputs = {
    "v0-frozen.rs": crash,
    "v1-new-layer-plain.rs": crash.replace(opening, layer_one_line + "\n" + opening, 1),
    "v2-new-layer-behind-a-commented-opening.rs": crash.replace(opening, fake + layer_one_line + "\n" + opening, 1),
    "v3-multiline-new-layer-behind-a-commented-opening.rs": crash.replace(opening, fake + layer_multi + "\n" + opening, 1),
}
signature_tail = "    unit_checksums: &UnitChecksumsOn,\n    version: &JudgeVersion,\n    stop_is_requested: &(dyn Fn() -> bool + Sync),\n) -> Result<CheckingTally, VerdictStoreError> {"
start = crash.index("pub fn check_blocks_where_until(\n")
at = crash.index(signature_tail, start)
inputs["v4-trailing-comment-with-fn-in-a-signature.rs"] = crash[:at] + signature_tail.replace("    version: &JudgeVersion,\n", "    version: &JudgeVersion, // 判法版本，fn 外面定\n") + crash[at + len(signature_tail):]
inputs["v5-attribute-line-before-a-layer.rs"] = crash.replace("pub fn check_blocks_where_until(\n", "#[allow(clippy::too_many_arguments)] // 参数各是一样东西\npub fn check_blocks_where_until(\n", 1)
anchor = "    mod card_pool {\n"
assert gpu.count(anchor) == 1
inputs["g0-frozen.rs"] = gpu
inputs["g1-use-wgpu-itself.rs"] = gpu.replace(anchor, anchor + "        use wgpu;\n", 1)
inputs["g2-block-scoped-rename-in-a-helper.rs"] = gpu.replace(anchor, anchor + "        fn atk_helper() -> u8 {\n            use core as std;\n            std::u8::MAX\n        }\n", 1)
inputs["g3-commented-tests-opening-inside-the-pool.rs"] = gpu.replace(anchor, anchor + "        /*\n#[cfg(test)]\nmod tests {\n        */\n", 1)
for name, text in inputs.items():
    (out / name).write_text(text)
    print(name, "测试模块头两行出现次数", text.count(opening))
