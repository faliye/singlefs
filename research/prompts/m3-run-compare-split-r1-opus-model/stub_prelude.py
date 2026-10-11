#!/usr/bin/env python3
"""攻方腿模型（m3-run-compare-split-r1）：给判器内核的 WGSL 造一份「生成的常量」替身。
仓里内核前面接的那段常量（crash_judge_gpu.rs 的 kernel_prelude_wgsl_constants）是 Rust 现生成的；这里不编 checker 档，
就把几份 WGSL 里用到、而几份里都没定义的全大写标识符一律写成 `const NAME: u32 = 1u;`。表达式个数只数函数体里的 IR，
常量取什么值不改它（数法见 kernel-size/src/main.rs）；替身对不对，由「三个内核量出来与仓里单测钉的值逐个相同」来证。
用法：stub_prelude.py <输出文件> <WGSL 文件>…
"""
import re
import sys

out_path, sources = sys.argv[1], sys.argv[2:]
text = "\n".join(open(path, encoding="utf-8").read() for path in sources)
# 去掉注释，免得注释里的大写词被当成标识符
text = re.sub(r"//[^\n]*", "", text)
defined = set(re.findall(r"\b(?:const|override|var(?:<[^>]*>)?|let|fn|struct|alias)\s+([A-Za-z_][A-Za-z0-9_]*)", text))
used = set(re.findall(r"\b([A-Z][A-Z0-9_]{2,})\b", text))
missing = sorted(name for name in used - defined)
with open(out_path, "x", encoding="utf-8") as handle:
    for name in missing:
        handle.write(f"const {name}: u32 = 1u;\n")
print(f"STUBBED count={len(missing)}")
