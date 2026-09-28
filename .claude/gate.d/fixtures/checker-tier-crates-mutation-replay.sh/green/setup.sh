#!/usr/bin/env bash
# checker-tier-crates-mutation-replay 算底座指纹要 git 列文件（research/scripts/admission.py 的 listed_input_files），按条记录写在被判那棵树的 git common-dir 里：
# 样本拷进临时目录之后在那里建一个空仓，不提交，未跟踪的文件照样列得出来。
set -e
git init -q .
