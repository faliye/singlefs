#!/usr/bin/env python3
# 样本：认会话记录里写的路径文字（排除表放行这一行）
def is_work_mention(path):
    return path.startswith("~/.local/share/singlefs/work/")
