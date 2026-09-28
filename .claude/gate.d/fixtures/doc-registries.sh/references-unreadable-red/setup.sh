#!/usr/bin/env bash
# 读不了的 kb 文件：一个悬空的符号链接。grep 读它退 2，本该判红，
# 而不是把那一份里的引用静默丢掉、拿剩下的照判照绿。符号链接在临时目录里现造，不摆进仓里。
set -e
ln -s missing-target.md .claude/kb/dangling-link.md
