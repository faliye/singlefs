#!/usr/bin/env bash
# 样本：文件头按现算读法写齐（点名 --emit-hosts 与 host-resources.py，没点名 heavy-thread-default）；setup.sh 把它连同所在目录放回搬家之前的位置提交成基准
# long-run-shards: 按用例切片
# long-run-resume: 重起同一条命令只补没做完的片
# long-run-hosts: 台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台的线程与内存取 host-resources.py --json 现算
root="$(cd "$(dirname "$0")/../.." && pwd)"
rig="$root/scripts/new-rig"
slots=9
echo "$rig scripts/new-rig/live.sh $slots"
