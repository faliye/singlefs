#!/usr/bin/env bash
# 本该判红：触发文件没登记。取改动路径的 git 本身成功、却往标准错误写了一条警告：暂存区里删了两个文件、加了两个内容不同的文件，
# git 配置里的 diff.renameLimit 写成 1，重命名探测超上限，git 写「you may want to set your diff.renameLimit variable …」。
# 取改动路径是阶段在父进程里预取的，取到时 git 写到标准错误的警告要原样打出来，输出里要有它（警告里的 diff.renameLimit 是配置项名字，不随 git 的界面语言变）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/gate.d scripts
cat > .claude/gate.d/knowledge-sync-triggers.tsv <<'EOF'
# 子集：够跑这份夹具的几条
^scripts/	# 研究与协作脚本
EOF
printf 'print("a")\n' > scripts/a.py
printf 'print("b")\n' > scripts/b.py
git add -A && git commit -qm base
git rm -q scripts/a.py scripts/b.py
mkdir -p scripts   # git rm 删光 scripts/ 里的文件后目录跟着没了
printf 'value = 1\nprint(value)\n' > scripts/c.py
printf 'for number in range(3):\n    print(number)\n' > scripts/d.py
git add -A
git config diff.renameLimit 1
