#!/usr/bin/env python3
"""在仓副本里给池级 checker 的 I-9.4 第三句加一个开关：设了环境变量 R3_I94_THIRD_OFF 时这一句判成恒真（= 删掉第三句）。
用法：python3 patch_checker.py <仓副本里的 crates/singlefs-checker/src/walk.rs>。原样串不是恰好一处就退 3（只改副本）。"""
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text()
old = """            self.judgements
                .judge("I-9.4", left.largest_inode < right.container, || {"""
new = """            self.judgements
                .judge("I-9.4", std::env::var_os("R3_I94_THIRD_OFF").is_some() || left.largest_inode < right.container, || {"""
if text.count(old) != 1:
    sys.exit(3)
path.write_text(text.replace(old, new))
print("patched", path)
