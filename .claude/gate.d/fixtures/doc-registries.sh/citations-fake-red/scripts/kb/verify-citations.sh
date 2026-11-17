#!/usr/bin/env bash
# 样本替身：真 verify-citations.sh 读本机绝对路径下的源码树，装不进样本；这一份只交一个退出码 3 与一行结果，
# 让门禁 doc-registries 的 citations 那一格判「脚本退非 0 就判红」那一半。
echo "  ✗ 样本替身：3 条里 1 条未命中"
exit 3
