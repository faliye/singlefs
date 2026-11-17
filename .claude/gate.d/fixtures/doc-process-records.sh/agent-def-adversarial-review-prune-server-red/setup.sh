#!/usr/bin/env bash
# 判定库服务两处：scripts/prune-server/ 下的客户端一份、prune-server/ 下的 src 一份与无后缀的 control 一份改了，prune-server/src/old.rs 删了；
# 另改了同目录里的说明（.md）与 Cargo.lock（都不归这一格管），没有判决 ⇒ 四份没人点名（删掉的也算），判红报 4 份。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p scripts/prune-server prune-server/src .claude/gate.d
printf 'VALUE = 1\n' > scripts/prune-server/client.py
printf 'pub fn serve() -> u32 { 1 }\n' > prune-server/src/lib.rs
printf 'pub fn old() -> u32 { 1 }\n' > prune-server/src/old.rs
printf '#!/usr/bin/env bash\necho 1\n' > prune-server/control
printf '# 说明\n\n一版。\n' > prune-server/README.md
printf 'version = 1\n' > prune-server/Cargo.lock
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf 'VALUE = 2\n' > scripts/prune-server/client.py
printf 'pub fn serve() -> u32 { 2 }\n' > prune-server/src/lib.rs
git rm -q prune-server/src/old.rs
printf '#!/usr/bin/env bash\necho 2\n' > prune-server/control
printf '# 说明\n\n二版。\n' > prune-server/README.md
printf 'version = 2\n' > prune-server/Cargo.lock
