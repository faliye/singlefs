#!/usr/bin/env bash
# 放行表（基准里就有）：第 2 行写对了、有脚本引到；第 3 行没有一份 git 跟踪的脚本引到；第 4 行理由不到 8 个字；第 5 行没写 # 与理由；第 6 行路径不以 ~/ 或 / 开头。
# 第 4、5、6 行的路径脚本里都引到了，只红在写法上。新加的一行干净。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/.out-of-project-*' '/setup.sh' '/expect' > .gitignore   # 替身与样本自己的文件不进被判的改动，也不算 git 跟踪的脚本
mkdir -p records scripts .claude
printf '%s\n' '#!/usr/bin/env bash' 'ls "$HOME/sample-kbuild/tree" "$HOME/sample-kbuild/tree2" "$HOME/sample-kbuild/tree3"' > scripts/fetch.sh
printf '%s\n' '# 样本放行表' \
  '~/sample-kbuild/tree  # 样本：scripts/fetch.sh 读这棵假的内核树' \
  '~/sample-unused/tree  # 样本：没有一份脚本读这一条路径' \
  '~/sample-kbuild/tree2  # 太短' \
  '~/sample-kbuild/tree3' \
  'sample-kbuild/tree  # 样本：路径没写成 ~/ 或 / 开头的' > .claude/out-of-project-paths
printf '# 样本空间\n\n- 本项目的目录占用照常写。\n' > records/2026-10-21-样本空间.md
git add -A && git commit -qm base
printf '%s\n' '- 第二台跑了一半。' >> records/2026-10-21-样本空间.md
