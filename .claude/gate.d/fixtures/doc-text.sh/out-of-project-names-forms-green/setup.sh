#!/usr/bin/env bash
# 放行表四条 XDG 下的 singlefs 路径，基准里只由脚本的 XDG 回退写法与 python 拼接引到；新加的行：第 4 行 ../ 接仓根下也有的名字（链接、include_str!、crates/../），
# 第 5、6 行四种 XDG 回退，第 7 行 python 拼接，第 8 行 singlefs-<后缀>，第 9 行 /home/<别的用户>/，第 10 行反引号里的小写词，第 11 行干净，判绿。
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
cat >> records/2026-10-21-样本路径.md <<'Y'
- 仓里的相对路径：[kb](../.sample-tool/kb/x.md)、include_str!("../../../.sample-tool/heavy")、crates/../.sample-tool/kb/x.md。
- XDG 回退：${XDG_CACHE_HOME:-$HOME/.cache}/singlefs/model 与 ${XDG_CONFIG_HOME:-${HOME}/.config}/singlefs/cards.env。
- XDG 回退：${XDG_DATA_HOME:-$HOME/.local/share}/singlefs/peaks.tsv 与 ${XDG_STATE_HOME:-$HOME/.local/state}/singlefs/runs。
- python：os.path.join(os.environ.get("XDG_DATA_HOME") or os.path.expanduser("~/.local/share"), "singlefs", "peaks.tsv")。
- 后缀：${SINGLEFS_GOLDEN_DIR:-$HOME/.cache/singlefs-golden} 与 ~/.local/share/singlefs-runs。
- 别的用户：/home/u/.local/state/singlefs/runs 与 /home/u/.cache/singlefs。
- 小写词：`toolbox` 与 `datasets` 只是普通词。
- 根分区其余不归 singlefs。
Y
