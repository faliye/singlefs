#!/usr/bin/env bash
# invariant-anchors 那一格要 git 仓：基准是一个空提交，样本里的文件全都未跟踪，invariants.md 整份都算这次新写，
# I-99.1、I-99.3 点了名、I-99.2 没点名，只红 I-99.2 那一行；不把未跟踪算进改动范围的取法会报「这次改动没有新写或改写不变量行」。
# 读不了的 kb 文件那一种红放在 red-unreadable 样本里：悬空的符号链接会让 invariant-count-elsewhere 那一格读文件时出错，盖掉这里要看的两行。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
git commit -q --allow-empty -m '基准：一份文件都还没有'
