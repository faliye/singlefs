#!/usr/bin/env bash
# 登记了搬迁、而豁免表那一行的脚本自己没搬家：建 git 仓，基准里 scripts/old-rig/run.sh 与 scripts/live.sh（文件头按现算读法写齐，正文起 scripts/old-rig/run.sh），
# 表登记 scripts/live.sh 与它的 sha256，提交成基准；再 git mv scripts/old-rig scripts/new-rig、登记这次搬迁，scripts/live.sh 原地只把正文里的路径换成 scripts/new-rig，
# 表里重钉成新文件的 sha256。只起 git、printf、sed、sha256sum。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p scripts/old-rig .claude/gate.d
printf '#!/usr/bin/env bash\necho 干活\n' > scripts/old-rig/run.sh
printf '%s\n' '#!/usr/bin/env bash' \
  '# 样本：文件头按现算读法写齐（点名 --emit-hosts 与 host-resources.py，没点名 heavy-thread-default），登记在豁免表里；它自己不搬家' \
  '# long-run-shards: 按用例切片' \
  '# long-run-resume: 重起同一条命令只补没做完的片' \
  '# long-run-hosts: 台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台的线程与内存取 host-resources.py --json 现算' \
  'worker=scripts/old-rig/run.sh' \
  'bash "$worker"' > scripts/live.sh
printf '%s\n' '# 样本：基准那一版' "scripts/live.sh  $(sha256sum scripts/live.sh | cut -d' ' -f1)  # 样本：文件头按现算写、不能改" > .claude/long-run-live-hosts-exempt
git add -A .claude scripts records
git commit -qm base
git mv scripts/old-rig scripts/new-rig
sed -i 's#scripts/old-rig/#scripts/new-rig/#' scripts/live.sh
printf '%s\n' '# 样本：工作区这一版（重钉）' "scripts/live.sh  $(sha256sum scripts/live.sh | cut -d' ' -f1)  # 样本：文件头按现算写、不能改" > .claude/long-run-live-hosts-exempt
printf '%s\n' '# 样本：搬迁登记' 'scripts/old-rig	scripts/new-rig	2026-10-01	records/样本搬迁.md' > .claude/gate.d/path-moves.tsv
