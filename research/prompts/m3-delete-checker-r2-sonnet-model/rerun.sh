#!/usr/bin/env bash
# 正推腿两个计数模型的复跑：纯 Python，不读盘、不跑 crates，几秒。
# 用法：bash research/prompts/m3-delete-checker-r2-sonnet-model/rerun.sh <还不存在的工作目录>
# 把本目录的 .py 拷进工作目录，跑五个脚本，输出写进工作目录/out/，再与本目录 expected/ 逐个 diff；末尾打印「diff 相同」或列出不同。
set -euo pipefail
export PYTHONDONTWRITEBYTECODE=1
work=$1
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
mkdir "$work"
cp "$here"/*.py "$work/"
mkdir "$work/out"
cd "$work"
nice -n 19 python3 legal_history_model.py 3000 10 > out/legal.out
nice -n 19 python3 bad_history_model.py 3000 10 refuse > out/bad.out
nice -n 19 python3 subsets.py > out/subsets.out
nice -n 19 python3 bing_naive.py > out/bing_naive.out
nice -n 19 python3 t12_newest_only.py > out/t12_newest_only.out
status=0
for name in legal bad subsets bing_naive t12_newest_only; do
    if ! diff -q "$here/expected/$name.out" "out/$name.out" > /dev/null; then
        echo "不同：$name.out"
        status=1
    fi
done
if [ "$status" = 0 ]; then echo "diff 相同（5 个输出与 expected/ 逐字节一致）"; fi
exit "$status"
