#!/bin/sh
# target.<triple>.runner：cargo 用它起测试二进制；这里给测试进程加一个环境变量。
OPUS_R3_RUN_ENV=from-runner exec "$@"
