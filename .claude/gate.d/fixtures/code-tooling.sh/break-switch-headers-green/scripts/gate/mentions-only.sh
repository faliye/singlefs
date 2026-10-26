#!/usr/bin/env bash
# 样本脚本：只在注释里提 OTHER_BREAK=x，代码里只把它从子进程环境里摘掉，不算读
env -u OTHER_BREAK bash -c true
