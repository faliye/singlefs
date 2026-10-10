#!/usr/bin/env python3
"""攻方零轮改法 F2（只在攻方副本上量过、被攻过零轮）：池级 checker 判走读跟随的指针头部时，MAC / nonce 之外把算法类型、
压缩算法码、压后长度（头部偏移 28..32）一起判 0（core 的 PointerHeadAsRead::judged 判的那一组）。用法：pointer_head_fix.py <树根>。"""
import sys
path = sys.argv[1] + "/crates/singlefs-checker/src/image.rs"
text = open(path, encoding="utf-8").read()
old = "    pointer_bytes[..POINTER_MAC_AND_NONCE_BYTES]\n        .iter()\n        .all(|byte| *byte == 0)\n"
new = "    pointer_bytes[..POINTER_MAC_AND_NONCE_BYTES + 4]\n        .iter()\n        .all(|byte| *byte == 0)\n"
if text.count(old) != 1:
    sys.exit(f"anchor hits {text.count(old)}")
open(path, "w", encoding="utf-8").write(text.replace(old, new))
print("applied F2")
