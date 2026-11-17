#!/usr/bin/env bash
# 改了 task-server/ 下的三份文件（无后缀的入口 task、一份 .py、tests/ 下的一份测试），另改了同目录里的说明（.md）与 __pycache__ 里的一份字节码（都不归这一格管），没有判决
# ⇒ 三份没人点名，判红报 3 份。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p task-server/tests task-server/__pycache__ .claude/gate.d
printf '#!/usr/bin/env bash\necho 1\n' > task-server/task
printf 'VALUE = 1\n' > task-server/task_demo.py
printf 'def test_value():\n    assert 1 == 1\n' > task-server/tests/test_demo.py
printf '# 说明\n\n一版。\n' > task-server/README.md
printf 'bytecode 1\n' > task-server/__pycache__/task_demo.pyc
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf '#!/usr/bin/env bash\necho 2\n' > task-server/task
printf 'VALUE = 2\n' > task-server/task_demo.py
printf 'def test_value():\n    assert 2 == 2\n' > task-server/tests/test_demo.py
printf '# 说明\n\n二版。\n' > task-server/README.md
printf 'bytecode 2\n' > task-server/__pycache__/task_demo.pyc
