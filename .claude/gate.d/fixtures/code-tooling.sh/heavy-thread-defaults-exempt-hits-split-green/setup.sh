#!/usr/bin/env bash
# 排除表只缩不涨按放行的被判行数比：建 git 仓，基准里两行 $(nproc) 都在入口 scripts/new-rig/report_cores.sh 里、表里一行写入口的路径，提交成基准；
# 再放回工作区里拆好的样子（两行各挪进一份部分文件、表里两行）。只起 git、mv、printf。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mv scripts/new-rig/report_cores.sh .entry-now
mv scripts/new-rig/report_cores-count.sh .count-now
mv scripts/new-rig/report_cores-show.sh .show-now
mv .claude/heavy-thread-exempt .table-now
printf '%s\n' '#!/usr/bin/env bash' '# 样本：基准那一版的入口，两行 $(nproc) 都在这一份里' 'cores=$(nproc)' 'shown=$(nproc)' > scripts/new-rig/report_cores.sh
printf '%s\n' '# 样本：基准那一版的表' 'scripts/new-rig/report_cores.sh:$(nproc)  # 样本：只把核数打出来，不起线程' > .claude/heavy-thread-exempt
git add -A .claude scripts
git commit -qm base
mv .entry-now scripts/new-rig/report_cores.sh
mv .count-now scripts/new-rig/report_cores-count.sh
mv .show-now scripts/new-rig/report_cores-show.sh
mv .table-now .claude/heavy-thread-exempt
