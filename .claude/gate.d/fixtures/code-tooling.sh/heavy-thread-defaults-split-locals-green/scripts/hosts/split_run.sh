#!/usr/bin/env bash
# 样本：拆成入口加部分文件的脚本，工作进程数是入口收到的参数（文件自己赋的局部变量），不是从环境读的
SAMPLE_SPLIT_WORKERS="${1:?工作进程数}"
echo split_run
