#!/usr/bin/env bash
# Y3 样本：在当前目录建 git 仓（假配置同第一轮攻方样本，值都是编的），提交一版干净记录，再造三种这一次改动：
#   case=symlink-dangling  research/results/peer-latest 是指到第二台仓路径的符号链接（目标在本机不存在）；另有一份干净的新日志
#   case=symlink-mount     research/results/peer-e999.out 是指到「以第二台主机名命名的挂载点」下文件的符号链接（目标存在、内容干净；挂载点在样本仓外 $MOUNT_ROOT）
#   case=unreadable        从第二台 tar 过来的产物目录里一份权限 000 的文件（路径带第二台仓路径），另有一份干净的新日志
# 用法：MOUNT_ROOT=<仓外目录> bash setup.sh <case>
set -e
case_name=$1
cp "$(dirname "$0")/multi-host.env" "$(dirname "$0")/.private-values-hostname" . 2>/dev/null || true
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/multi-host.env' '/setup.sh' '/.private-values-hostname' > .gitignore
mkdir -p records research/results
printf '# 样本调度\n\n- 干净的一行。\n' > records/2026-09-24-样本调度.md
git add -A && git commit -qm base
printf 'name=e999 rows=3 clean\n' > research/results/e999-local.log
case "$case_name" in
  symlink-dangling) ln -s /srv/sample-private/peer-tree/research/results/e999 research/results/peer-latest ;;
  symlink-mount)
    mkdir -p "$MOUNT_ROOT/sample-peer-box-17/results"
    printf 'name=e999 rows=3 clean\n' > "$MOUNT_ROOT/sample-peer-box-17/results/e999.out"
    ln -s "$MOUNT_ROOT/sample-peer-box-17/results/e999.out" research/results/peer-e999.out ;;
  unreadable)
    d=research/results/from-peer/srv/sample-private/peer-tree/research/results
    mkdir -p "$d"; printf 'name=e999 rows=3\n' > "$d/e999.out"; chmod 000 "$d/e999.out" ;;
esac
