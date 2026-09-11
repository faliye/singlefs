#!/usr/bin/env bash
# 一个脚本：工作区可执行，暂存区里被写成 100644（手工暂存写死模式的形态）⇒ 本该判红。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p research/scripts
printf '#!/usr/bin/env bash\necho x\n' > research/scripts/x.sh
chmod +x research/scripts/x.sh
git add research/scripts/x.sh
git update-index --chmod=-x research/scripts/x.sh
git commit -qm base
