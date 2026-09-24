#!/usr/bin/env bash
# 红样本：列路径的 git 调用没带 quotepath，中文路径会被转义
git ls-files --others --exclude-standard -- .claude/kb
