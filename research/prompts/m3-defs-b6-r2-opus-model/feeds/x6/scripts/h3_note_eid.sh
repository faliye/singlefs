#!/usr/bin/env bash
# long-run-shards: 按行切，一行一片
# long-run-resume: 同一条命令再起只补还没有结果文件的行
# long-run-hosts: 台数从 layer0-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量；编排照 E171 修订 11 那一套
echo run
