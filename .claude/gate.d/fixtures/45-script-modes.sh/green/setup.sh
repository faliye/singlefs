#!/usr/bin/env bash
# 同一个脚本，暂存区里是 100755、工作区可执行 ⇒ 本该判绿。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p research/scripts
printf '#!/usr/bin/env bash\necho x\n' > research/scripts/x.sh
chmod +x research/scripts/x.sh
git add research/scripts/x.sh
git commit -qm base
