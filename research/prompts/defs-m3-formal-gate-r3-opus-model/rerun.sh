#!/usr/bin/env bash
# 复跑这条腿的全部探针：bash rerun.sh <仓根> <草稿目录（不存在或空）>。挂钟 73 秒（2026-10-01 在本机整条跑一次量的，rerun-output.txt 末行），在 20 分钟以内。
# 不跑门禁、不跑判法脚本的干活用法、不跑 --selftest；钩子只喂 JSON。L2 直接起 Verus 验一份三行的玩具文件 8 次一趟（经内存包装 400M、capped.sh 4，strace 两次），
# 安装拷贝是原安装顶层文件的硬链接（不拷 1.6G），截断的那一份另写新文件、不碰原 inode。git 写只落在仓拷贝自己新建的 .git 里。
# 末尾逐份比：tmp 路径换成 <草稿>，L2 的「秒」那几行（挂钟）不比。
# 已知：l2-K3-output 必报「不同」——K3 把 VERUS_ROOT 的值（草稿目录里的路径）打进那一行，摘要随草稿目录变；2026-10-01 复跑只差这一行（VERUS_ROOT=<草稿>/B 那一行的前 16 位）。
set -uo pipefail
REPO="${1:?仓根}"; W="${2:?草稿目录}"; HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$W"
for copy in repo repo-k repo-l2 repo-l2k; do rsync -a --exclude target --exclude .git "$REPO/" "$W/$copy/"; done
python3 "$HERE/fix-draft-l1.py" "$W/repo-k"
python3 "$HERE/fix-draft-l2.py" "$W/repo-l2k"
python3 "$HERE/l1-hook-cases.py" "$W/repo" "$W/l1" > "$W/l1-output.txt" 2>&1
python3 "$HERE/l1-hook-cases.py" "$W/repo-k" "$W/l1k" > "$W/l1-K-output.txt" 2>&1
( cd "$W/repo" && python3 "$HERE/l1-e165-parse.py" "$W/repo" ) > "$W/l1-e165-parse-output.txt" 2>&1
python3 "$HERE/l1-argv-demo.py" "$W/l1" > "$W/l1-argv-demo-output.txt" 2>&1
bash "$HERE/l2-install.sh" "$W/repo-l2" "$W/l2" > "$W/l2-output.txt" 2>&1
bash "$HERE/l2-install.sh" "$W/repo-l2k" "$W/l2" > "$W/l2-K3-output.txt" 2>&1
for k in all-not-run mixed all-red; do bash "$HERE/l3-stage.sh" "$W/repo" "$k" "$W/l3" > "$W/l3-stage-$k.txt" 2>&1; done
python3 "$HERE/l3-judge.py" "$W"/l3-stage-{all-not-run,mixed,all-red}.txt > "$W/l3-judge-output.txt" 2>&1
python3 "$HERE/l3-judge-k4.py" "$W"/l3-stage-{all-not-run,mixed,all-red}.txt > "$W/l3-K4-output.txt" 2>&1
( cd "$W" && bash "$W/repo/scripts/hosts/run-with-memory-cap.sh" 1Q true > "$W/l3-wrapper-251.txt" 2>&1; echo "rc=$?" >> "$W/l3-wrapper-251.txt" )
for name in l1-output l1-K-output l1-e165-parse-output l1-argv-demo-output l2-output l2-K3-output l3-judge-output l3-K4-output l3-stage-all-not-run l3-stage-mixed l3-stage-all-red l3-wrapper-251; do
  if diff -q <(grep -v '秒）$' "$HERE/$name.txt") <(sed "s#$W#<草稿>#g" "$W/$name.txt" | grep -v '秒）$') >/dev/null; then echo "相同 $name"; else echo "不同 $name"; fi
done
