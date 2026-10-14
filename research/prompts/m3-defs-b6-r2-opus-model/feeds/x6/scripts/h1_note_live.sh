#!/usr/bin/env bash
# long-run-shards: 按行切，一行一片
# long-run-resume: 同一条命令再起只补还没有结果文件的行
# long-run-hosts: 台数从 layer0-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量；按此刻空闲现算每台槽数与线程（编排照旧）
echo run
