#!/usr/bin/env bash
# 样本：被 source 的共用库，只在函数头后面的行尾注释里提到 --selftest（照 scripts/hosts/peer-host-lib.sh 那一种写法），代码里没有实现它，不算声称有
sample_inside_selftest() { # 调用方的 --selftest 起的：标记文件在、第一行对得上
  [[ -n "${SAMPLE_SELFTEST_MARK:-}" && -f "$SAMPLE_SELFTEST_MARK" ]]
}
