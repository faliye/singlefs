#!/usr/bin/env bash
# 同一套替身与放行表；新加的行：第 4 行 ../ 接仓根下没有的名字，第 5 行 ~/<XDG 基目录> 带容量，第 6 行反引号里的 <XDG 基目录> 带容量，第 7 行停在 XDG 基目录的路径，
# 第 8 行 XDG 回退与 python 拼接接到别的目录，第 9 行 singlefs 后面多一个字，第 10 行反引号里的小写词带容量，第 11 行反引号里带 - 的名字，判红 8 行；第 12 行干净。
# 带 XDG 基目录名的红行由 printf 拼出来：这份 setup.sh 自己进仓，真门禁判它的新加行时，本机家目录里也有这几个名字。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/.out-of-project-*' '/setup.sh' '/expect' > .gitignore   # 替身与样本自己的文件不进被判的改动，也不算 git 跟踪的脚本
mkdir -p scripts .claude records .sample-tool/kb
cat > scripts/fetch.sh <<'Y'
#!/usr/bin/env bash
cache="${XDG_CACHE_HOME:-$HOME/.cache}/singlefs"
config="${XDG_CONFIG_HOME:-$HOME/.config}/singlefs/cards.env"
state="${XDG_STATE_HOME:-$HOME/.local/state}/singlefs/runs"
Y
cat > scripts/peaks.py <<'Y'
import os
path = os.path.join(os.environ.get("XDG_DATA_HOME") or os.path.expanduser("~/.local/share"), "singlefs", "peaks.tsv")
Y
cat > .claude/out-of-project-paths <<'Y'
# 样本放行表
~/.cache/singlefs  # 样本：scripts/fetch.sh 把取来的模型文件缓存在这里
~/.config/singlefs  # 样本：scripts/fetch.sh 从这里读本机的显卡配置
~/.local/share/singlefs  # 样本：scripts/peaks.py 把峰值表写在这里
~/.local/state/singlefs  # 样本：scripts/fetch.sh 把长活记录写在这里
Y
printf '%s\n' '仓里自己的工具目录。' > .sample-tool/kb/x.md
printf '# 样本路径\n\n- 旧行。\n' > records/2026-10-21-样本路径.md
git add -A && git commit -qm base
cache=.cache; state=.local/state; share=.local/share
{
  printf '%s\n' '- 仓外的相对路径：../.sample-peer-cache/data。'
  printf -- '- 家目录的缓存 ~/%s 38G。\n' "$cache"
  printf -- '- 缓存 `%s` 38G。\n' "$cache"
  printf -- '- 停在基目录：没设时是 ~/%s。\n' "$state"
  printf -- '- 接到别的目录：${XDG_CACHE_HOME:-$HOME/%s}/sample-other 与 os.path.expanduser("~/%s"), "sample-other"。\n' "$cache" "$share"
  printf -- '- 多一个字：$HOME/%s/singlefsx。\n' "$cache"
  printf '%s\n' '- 小写词带容量：`toolbox` 80G。'
  printf '%s\n' '- 带 - 的名字：`sample-workspace`。'
  printf '%s\n' '- 本项目的 research/results 占 3G。'
} >> records/2026-10-21-样本路径.md
