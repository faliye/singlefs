#!/usr/bin/env bash
# 排除表只缩不涨按放行的被判行数比：建 git 仓，基准里入口 scripts/new-rig/report_cores.sh 只有一行 $(nproc)、表里一行写它，提交成基准；
# 再放回工作区里多了一行 $(nproc) 的样子（表不变）。只起 git、mv、printf。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mv scripts/new-rig/report_cores.sh .entry-now
printf '%s\n' '#!/usr/bin/env bash' '# 样本：基准那一版的入口，只有一行 $(nproc)' 'cores=$(nproc)' > scripts/new-rig/report_cores.sh
git add -A .claude scripts
git commit -qm base
mv .entry-now scripts/new-rig/report_cores.sh
