#!/usr/bin/env bash
# 本该判红，一份样本同时放四种坏法：
#   ① 一个触发文件（research/scripts/sneak.py）一个字都没登记——「顺手修」就长这样；
#   ② 登记了 .claude/agents/gone.md，而这一批根本没碰它（上一批提交完没清表，下一批就白放行）；
#   ③ 一行登记只写路径、# 后面空着，没写为什么这一批非碰它不可；
#   ④ 一行登记写成 ./ 开头，不是从仓库根起；
#   ⑤ 造一份不再读触发文件清单的 68 号（清单被人抄回脚本里的那种），两道就会对「什么算触发文件」各说各话。
# 另外正常登记了一个真碰了的触发文件，证明它不是整表判红。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/gate.d .claude/agents research/scripts
cat > .claude/gate.d/knowledge-sync-triggers.tsv <<'EOF'
# 子集：够跑这份夹具的几条
^\.claude/agents/	# agent 定义
^research/scripts/	# 研究与协作脚本
EOF
printf '# demo\n\n第一版\n' > .claude/agents/demo.md
printf 'print("old")\n' > research/scripts/kept.py
git add -A && git commit -qm base
printf '# demo\n\n第二版\n' > .claude/agents/demo.md
printf 'print("new")\n' > research/scripts/kept.py
printf 'print("sneak")\n' > research/scripts/sneak.py
# ⑤：68 号被改成不读那份清单了（清单抄回了脚本里）
printf '#!/usr/bin/env bash\n# 把清单抄回脚本里的那一版\nexit 0\n' > .claude/gate.d/68-knowledge-sync.sh
cat > .claude/batch-scope <<'EOF'
# 这一批要碰的触发文件

.claude/agents/demo.md	# 这一批改的就是它的定义
.claude/agents/gone.md	# 上一批留下的，这一批根本没碰它
research/scripts/kept.py	#
./research/scripts/kept.py	# 路径写歪了，不是从仓库根起
EOF
