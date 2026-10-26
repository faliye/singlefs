#!/usr/bin/env bash
# 搬迁实据那一支（真搬迁）：建 git 仓，基准里 scripts/old-rig 下三份文件、一份点名它的说明与一张已登记 scripts/ancient-rig → scripts/old-rig 的搬迁登记表；
# 再 git mv scripts/old-rig scripts/new-rig，两份只把路径换成新前缀、一份另改了一行，说明里的路径一起换，登记表加一行 scripts/old-rig → scripts/new-rig。
# 只起 git 与 printf。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p scripts/old-rig docs .claude/gate.d
printf '#!/usr/bin/env bash\n# 在 scripts/old-rig/run.sh\necho 跑\n' > scripts/old-rig/run.sh
printf '看 scripts/old-rig/run.sh\n' > scripts/old-rig/data.txt
printf '笔记\n' > scripts/old-rig/notes.txt
printf '复现：bash scripts/old-rig/run.sh\n' > docs/howto.md
printf '%s\n' '# 样本：搬迁登记' 'scripts/ancient-rig	scripts/old-rig	2026-09-01	records/样本搬迁.md' > .claude/gate.d/path-moves.tsv
git add -A scripts docs records .claude
git commit -qm base
git mv scripts/old-rig scripts/new-rig
printf '#!/usr/bin/env bash\n# 在 scripts/new-rig/run.sh\necho 跑\n' > scripts/new-rig/run.sh
printf '看 scripts/new-rig/run.sh\n' > scripts/new-rig/data.txt
printf '笔记，搬完又改了一行\n' > scripts/new-rig/notes.txt
printf '复现：bash scripts/new-rig/run.sh\n' > docs/howto.md
printf '%s\n' 'scripts/old-rig	scripts/new-rig	2026-10-01	records/样本搬迁.md' >> .claude/gate.d/path-moves.tsv
