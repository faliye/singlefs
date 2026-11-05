#!/usr/bin/env bash
# 基准里钩子、脚本、门禁、规则、定义、task-server、prune-server 各一份都干净（钩子里另有一行旧的解析限额通知的正则，基准里就有，不报）；
# 这一轮往钩子里加两行（解析限额通知的正则一行、通知原样一行），再往另外八份里各加一行写了账号轮换或用量限额的话，判红：只点名这一轮新加的行。
# 射程之外的两份未跟踪文件（crates/ 下的源码、门禁样本目录下的文件）也带同样的话，不报。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude/hooks .claude/gate.d .claude/rules .claude/agents scripts task-server prune-server/src crates/sample/src .claude/gate.d/fixtures/sample/red
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
reset_pattern='resets (\w+) (\d+), (\d+)(am|pm) \((\w+)\)'
X
printf '%s\n' 'echo "limit hit, resets Nov 5, 3''pm (Asia/Tokyo)"' >> .claude/hooks/sample-guard.sh
printf '%s\n' 'echo ok' >> .claude/hooks/sample-guard.sh
printf '%s\n' '# 撞了周额度就换模型' >> scripts/sample.py
printf '%s\n' '# 另一个账号接着跑' >> .claude/gate.d/sample-stage.sh
printf '%s\n' '- 会话限额到了就停。' >> .claude/rules/sample.md
printf '%s\n' '- 换了账号再跑。' >> .claude/agents/sample.md
printf '%s\n' '- 限额窗口还没过。' >> .claude/main-agent.md
printf '%s\n' '- 周上限用完了。' >> .claude/agent-common.md
printf '%s\n' '# switched the account' >> task-server/sample.py
printf '%s\n' '// 额度窗口过了再来' >> prune-server/src/sample.rs
printf '%s\n' '// 射程之外的源码：撞了周额度' > crates/sample/src/lib.rs
printf '%s\n' '射程之外的样本：撞了周额度' > .claude/gate.d/fixtures/sample/red/expect
