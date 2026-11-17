#!/usr/bin/env bash
# 改了 scripts/ 下的三份脚本（.sh、.py、.rs 各一），另改了同目录里的一份登记表（.tsv）与一份说明（.md，都不归这一格管），没有判决
# ⇒ 三份脚本没人点名，判红报 3 份。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p scripts/demo .claude/gate.d
printf '#!/usr/bin/env bash\necho 1\n' > scripts/demo/run.sh
printf 'VALUE = 1\n' > scripts/demo/lib_demo.py
printf 'fn main() { println!("1"); }\n' > scripts/demo/engine.rs
printf 'demo\t1\n' > scripts/demo/table.tsv
printf '# 说明\n\n一版。\n' > scripts/demo/README.md
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf '#!/usr/bin/env bash\necho 2\n' > scripts/demo/run.sh
printf 'VALUE = 2\n' > scripts/demo/lib_demo.py
printf 'fn main() { println!("2"); }\n' > scripts/demo/engine.rs
printf 'demo\t2\n' > scripts/demo/table.tsv
printf '# 说明\n\n二版。\n' > scripts/demo/README.md
