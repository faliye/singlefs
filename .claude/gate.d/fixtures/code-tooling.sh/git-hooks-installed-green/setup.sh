#!/usr/bin/env bash
# git-hooks-installed 格的绿样本：小 git 仓里有进仓的钩子目录 scripts/gate/git-hooks/（pre-commit、pre-merge-commit、prepare-commit-msg、pre-applypatch、pre-push 五个桩，有执行位、进仓的模式 100755），
# core.hooksPath 写成这棵树里钩子目录的绝对路径。
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
git config core.hooksPath "$(pwd -P)/scripts/gate/git-hooks"
