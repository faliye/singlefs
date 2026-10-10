#!/usr/bin/env python3
"""在副本里打开关、挂探针（m3-formal-code-b24-r1 opus）。不设环境变量时副本行为与冻结副本相同。
用法：patch_copy.py <副本根> <模型目录>。每处原文要恰好命中一次，否则退 3。
- OPUS_ARM=count-zero：第二截「带文件而第一个文件不是活记录」那一支建 0 个（与 crates/mutations.tsv 23-1 改法 B1 那一行同义）；
- OPUS_K2B_SOME=1：判法只看 inode_record 是不是 Some（与 mutations.tsv 改法 b 那一行同义）；
- crash_injection 末尾挂子模块 opus_b24_probe（用私有项摆真历史的第一次崩溃）；工作区加两个探针包。"""
import os, shutil, sys
root, model = sys.argv[1], sys.argv[2]

def edit(path, pairs):
    p = f"{root}/{path}"
    s = open(p, encoding="utf-8").read()
    for old, new in pairs:
        n = s.count(old)
        if n != 1:
            print(f"{path}: 原文命中 {n} 次：{old[:80]!r}", file=sys.stderr)
            sys.exit(3)
        s = s.replace(old, new)
    open(p + ".opus-new", "w", encoding="utf-8").write(s)
    os.replace(p + ".opus-new", p)

CI = "crates/singlefs-checker-tier/src/crash_injection.rs"
edit(CI, [
    ("                PoolVersion::WithFile(current) => publish_new_inodes(\n                    &mut writer,\n                    &mut allocator,\n                    current,\n                    1,",
     "                PoolVersion::WithFile(current) => publish_new_inodes(\n                    &mut writer,\n                    &mut allocator,\n                    current,\n                    if std::env::var(\"OPUS_ARM\").as_deref() == Ok(\"count-zero\") { 0 } else { 1 },"),
    ("fn first_file_is_a_live_record(version: &TransactionOutput) -> bool {\n",
     "fn first_file_is_a_live_record(version: &TransactionOutput) -> bool {\n    if std::env::var(\"OPUS_K2B_SOME\").is_ok() {\n        return version.inode_record.is_some();\n    }\n"),
])
with open(f"{root}/{CI}", "a", encoding="utf-8") as f:
    f.write("\n#[path = \"opus_b24_probe.rs\"]\n#[allow(clippy::all, missing_docs)]\npub mod opus_b24_probe;\n")
shutil.copy(f"{model}/probe/opus_b24_probe.rs", f"{root}/crates/singlefs-checker-tier/src/opus_b24_probe.rs")
for crate in ("opus-b24-probe", "opus-b24-harness-probe"):
    shutil.copytree(f"{model}/probe/{crate}", f"{root}/crates/{crate}", dirs_exist_ok=True)
cargo = f"{root}/Cargo.toml"
s = open(cargo, encoding="utf-8").read()
old = '    "crates/singlefs-reference-filesystem",\n]'
if s.count(old) != 1:
    print("Cargo.toml 成员表原文没命中一次", file=sys.stderr); sys.exit(3)
s = s.replace(old, '    "crates/singlefs-reference-filesystem",\n    "crates/opus-b24-probe",\n    "crates/opus-b24-harness-probe",\n]')
open(cargo + ".opus-new", "w", encoding="utf-8").write(s)
os.replace(cargo + ".opus-new", cargo)
print("patched")
