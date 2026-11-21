#!/usr/bin/env bash
# git-hooks-installed 格的红样本（没装）：钩子目录与五个桩钩子都在、有执行位、进仓的模式 100755，只是 core.hooksPath 没设（git 只读 .git/hooks，进仓的钩子不起作用）。
set -e
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p scripts/gate/git-hooks
for hook in pre-commit pre-merge-commit prepare-commit-msg pre-applypatch pre-push; do
  printf '#!/usr/bin/env bash\nexit 0\n' > "scripts/gate/git-hooks/$hook"
  chmod +x "scripts/gate/git-hooks/$hook"
done
git add scripts
git commit -qm '钩子目录'
