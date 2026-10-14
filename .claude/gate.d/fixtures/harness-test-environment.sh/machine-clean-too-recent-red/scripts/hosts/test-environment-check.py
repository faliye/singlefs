#!/usr/bin/env python3
# 门禁 harness-test-environment 的样本 machine-clean-too-recent-red 里的替身，不是真脚本的拷贝：
# clean 预演照真脚本的写法报一项「太新、这一趟清不掉」（逐项报还要等多少秒）、退 4；
# 门禁这一格见到就该拒绝开跑、退 1，不往下跑 check。要是它没拒绝、跑到了 check，这里报判红 0 类、退 0，样本的 exit=1 对不上，样本判红。
import sys

arguments = sys.argv[1:]
if arguments[:1] == ['record-run-start']:
    sys.exit(0)
if arguments[:1] == ['clean']:
    print('test-environment-check：clean（替身）')
    print('  没动任何东西；不碰 1 项，其中 1 项只是太新（不到 10 分钟没改），这一次清不掉、下一次门禁会判它们红：singlefs-stub-too-recent.Xyz')
    print('    · 太新项：/tmp/singlefs-stub-too-recent.Xyz，还要等 137 秒才满 10 分钟没改')
    print('  ✗ 有 1 项太新、这一趟清不掉（clean 不碰，下一趟门禁会判它们红）')
    sys.exit(4)
print('  ✓ 临时目录残留：替身的 check（门禁该在预演退 4 时就拒绝开跑，不该走到这里）')
print('  汇总：查了 1 类；✓ 1 类，判红 0 类，没查成 0 类，没跑 0 类')
