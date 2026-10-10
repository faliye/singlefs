#!/usr/bin/env python3
"""在仓副本里给池级 checker 的 I-9.4 第三句加一个开关：设了 OPUS_K3_I94_N1 时改判「左容器最大 key < 右容器最小 key」。
用法：python3 patch_checker.py <仓副本里的 crates/singlefs-checker/src/walk.rs>（只改副本，原样串不是恰好一处就退 3）。"""
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text()
old = """            self.judgements
                .judge("I-9.4", left.largest_inode < right.container, || {"""
new = """            // 攻方副本（m3-leaf-death-r1 opus K3）：设了 OPUS_K3_I94_N1 时第三句换成「左容器最大 key < 右容器最小 key」。
            let right_bound = if std::env::var_os("OPUS_K3_I94_N1").is_some() {
                right.smallest_inode
            } else {
                right.container
            };
            self.judgements
                .judge("I-9.4", left.largest_inode < right_bound, || {"""
if text.count(old) != 1:
    sys.exit(3)
path.write_text(text.replace(old, new))
print("patched", path)
