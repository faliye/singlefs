#!/usr/bin/env bash
# git-hooks-installed 格的红样本（没有执行位）：core.hooksPath 是绝对路径，五个桩钩子提交进仓之后 pre-applypatch 在工作区里去掉了执行位、进仓的模式仍是 100755
# （弄坏开关 git-hooks-exec-bit-unchecked 不看执行位、git-hooks-added-unchecked 不核这一份，这一份在它们下面判绿）。
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
chmod -x scripts/gate/git-hooks/pre-applypatch
