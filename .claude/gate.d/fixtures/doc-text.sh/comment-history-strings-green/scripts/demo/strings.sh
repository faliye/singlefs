#!/usr/bin/env bash
# 字符串里的井号与参数展开里的井号不是注释。
name="a # 此前"
count=${#name}
trimmed=${name#a}
echo 'b # 原先' "c # 曾经"
set -- a b
echo $#
