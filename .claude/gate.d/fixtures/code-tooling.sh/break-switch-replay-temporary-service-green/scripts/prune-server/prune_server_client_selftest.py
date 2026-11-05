#!/usr/bin/env python3
# 样本的假包装：照真包装 --with-temporary-service [--server-binary <二进制>] -- <命令…> 的写法收参数，不起服务，
# 只设 SAMPLE_TEMPORARY_SERVICE=1 再起那条命令、交它的退出码
import os
import subprocess
import sys

arguments = sys.argv[1:]
if arguments[:1] != ["--with-temporary-service"] or "--" not in arguments:
    print("  ✗ 样本的假包装认不出用法", file=sys.stderr)
    sys.exit(2)
command = arguments[arguments.index("--") + 1:]
sys.exit(subprocess.run(command, env=dict(os.environ, SAMPLE_TEMPORARY_SERVICE="1")).returncode)
