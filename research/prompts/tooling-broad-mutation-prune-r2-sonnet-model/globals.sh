#!/bin/bash
# 只读 git：14 个动了清单文件的提交（commits.json，剔除首个整体导入 53c9f384）里，全局项与读数据文件的那几样变没变；format 的 lib.rs 变没变。从本目录跑。
cd "$(dirname "$0")"; R=/home/fy5090/code/singlefs
cs=$(python3 -c "import json;print(' '.join(x['c'] for x in json.load(open('commits.json')) if x['c']!='53c9f384'))")
n=0; for c in $cs; do n=$((n+1)); done; echo "commits: $n"
fm=0; st=0; li=0; anyg=0
for c in $cs; do
  g=""
  for p in Cargo.lock Cargo.toml .cargo litmus .claude/gate.d/stage-inputs.tsv crates/singlefs-core/Cargo.toml crates/singlefs-checker/Cargo.toml crates/singlefs-format/Cargo.toml crates/singlefs-harness/Cargo.toml; do
    git -C $R diff --quiet $c^ $c -- $p || g="$g $p"
  done
  git -C $R diff --quiet $c^ $c -- crates/singlefs-format/src/lib.rs || { fm=$((fm+1)); g="$g format/lib.rs"; }
  case "$g" in *stage-inputs*) st=$((st+1));; esac
  case "$g" in *litmus*) li=$((li+1));; esac
  [ -n "$g" ] && anyg=$((anyg+1))
  echo "$c:$g"
done
echo "format lib.rs changed in $fm of $n; stage-inputs.tsv in $st; litmus in $li; any global/data file changed in $anyg"
