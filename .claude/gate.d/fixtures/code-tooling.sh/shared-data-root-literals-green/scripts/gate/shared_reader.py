#!/usr/bin/env python3
"""样本：文档串里写的 ~/.local/share/singlefs/work 不判。"""
import os
HOWTO = "设 SAMPLE_ROOT，缺省在 ~/.local/share/singlefs/last-runs 下"
PREBUILT = os.path.expanduser("~/.local/share/singlefs/rocksdb-prebuilt")


def selftest():
    sample = "~/.local/share/singlefs/work/x/"
    return sample
