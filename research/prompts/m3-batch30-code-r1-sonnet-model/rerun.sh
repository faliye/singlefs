#!/usr/bin/env bash
# 正推腿（Sonnet）m3-batch30-code-r1 的模型：在被判的冻结副本上跑 F-W 与 X2、X13、X14 几条坏镜像（带探针，打出每张镜像红了哪几条不变量），
# 再把第 30 批变异表里的 CPU 行逐条改坏、跑点名的测试（GPU 两行要卡，不跑）。
# 环境变量：SINGLEFS_REPO = 被判的树（含 Cargo.toml、Cargo.lock、crates/；本轮是 ~/.local/share/singlefs/work/m3-batch30-code-r1/tree）；
#           SINGLEFS_WRAPPER = 含 scripts/run-with-memory-cap.sh 与 capped.sh 的仓（缺省取本脚本所在仓）；
#           SONNET_WORK = 工作目录（缺省 mktemp 新建；一次调用最多再开新的一步 150 秒，到了退 3，同一个 SONNET_WORK 再调一次接着做）。线程上限 7、编译 6G、跑 4G。
# 同一时刻只起一条 cargo；用自己的编译目录；compare 不依赖 SINGLEFS_REPO。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export SINGLEFS_WRAPPER="${SINGLEFS_WRAPPER:-$(cd "$here/../../.." && pwd)}"
work="${SONNET_WORK:-$(mktemp -d "$HOME/.local/share/singlefs/work/m3-batch30-code-r1-sonnet-rerun.XXXXXX")}"
mkdir -p "$work"
if [ "${1:-}" = compare ]; then
    # 比之前先排序；输出里带「耗时」类字段的行不进比（这里没有）
    for file in baseline.txt mutations.txt; do
        diff <(LC_ALL=C sort "$here/expected-$file") <(LC_ALL=C sort "$work/out/$file") && echo "same: $file"
    done
    exit 0
fi
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO}"
python3 "$here/drive.py" "$work" || { code=$?; echo "work=$work（退出码 $code；3 是预算用完，SONNET_WORK=$work 再调一次接着做）"; exit "$code"; }
echo "work=$work"
