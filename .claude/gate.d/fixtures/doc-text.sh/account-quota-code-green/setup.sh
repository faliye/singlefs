#!/usr/bin/env bash
# 与 account-quota-code-red 同一个基准（钩子里基准就有一行旧的解析限额通知的正则，不报）；这一轮往钩子里加的解析限额通知的正则一行带了理由的放行标记，
# 另一行是干净的；脚本里加一行显卡额度与内存上限的话。射程之外的两份未跟踪文件（crates/ 下的源码、门禁样本目录下的文件）带「撞了周额度」，不报，判绿。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude/hooks .claude/gate.d .claude/rules .claude/agents scripts task-server prune-server/src crates/sample/src .claude/gate.d/fixtures/sample/green
cat > .claude/hooks/sample-guard.sh <<'X'
#!/usr/bin/env bash
# 样本钩子
old_pattern='resets (\w{3}) (\d+), (\d+)(am|pm) \((\w+)\)'
X
printf '%s\n' '#!/usr/bin/env bash' > .claude/gate.d/sample-stage.sh
printf '%s\n' 'print("ok")' > scripts/sample.py
printf '%s\n' '# 样本规则' > .claude/rules/sample.md
printf '%s\n' '# 样本定义' > .claude/agents/sample.md
printf '%s\n' '# 样本主 agent' > .claude/main-agent.md
printf '%s\n' '# 样本共用约束' > .claude/agent-common.md
printf '%s\n' 'print("ok")' > task-server/sample.py
printf '%s\n' 'fn main() {}' > prune-server/src/sample.rs
git add -A && git commit -qm base
cat >> .claude/hooks/sample-guard.sh <<'X'
reset_pattern='resets (\w+) (\d+), (\d+)(am|pm) \((\w+)\)'  # account-quota:allow 解析外部通知文本，格式由对方定
echo ok
X
printf '%s\n' '# 显卡额度按优先级分，内存上限 12G' >> scripts/sample.py
printf '%s\n' '// 射程之外的源码：撞了周额度' > crates/sample/src/lib.rs
printf '%s\n' '射程之外的样本：撞了周额度' > .claude/gate.d/fixtures/sample/green/expect
