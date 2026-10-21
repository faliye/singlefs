#!/usr/bin/env bash
# 基准里一份记录、一份 kb 页、一份三方判决正文都干净（记录里另有一行旧话，带「撞了限额」，基准里就有，不报）；
# 这一轮往三份里各加写了账号切换、用量限额的行，另放一份未跟踪的新记录，判红：只点名这一轮新加的行。
# 射程之外的两份未跟踪文件（research/prompts 里喂云端腿的 _ 开头附录、.claude/rules 下的规则页）也带同样的话，不报。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p records .claude/kb research/prompts .claude/rules
cat > records/2026-09-24-样本调度.md <<'X'
# 样本调度（2026-09-24）

- 旧行：两条腿撞了限额，基准里就有，这一格只判这一轮新加的行。
X
cat > .claude/kb/01-样本页.md <<'X'
# 样本页

- 一句干净的话。
X
cat > research/prompts/sample-r1-main-verification.md <<'X'
# 样本判决

- 三条腿都交回了。
X
git add -A && git commit -qm base
cat >> records/2026-09-24-样本调度.md <<'X'
- 切换账号后续做。
- 撞了周额度，等重置。
- 另一个账号接着跑。
- 两条腿撞限额，主 agent 收尾。
- 额度恢复了，开始吧。
- 通知原文 You've hit your session limit。
- 限额窗口还没过。
- 用量限额到了就停。
- 额度用完了，等下一轮。
- then we switched the account and went on.
- 用户原话撞了限额。account-quota:allow
- 一行干净的话：中途停下没交回。
X
cat >> .claude/kb/01-样本页.md <<'X'
- 反推腿撞会话限流没跑成。
- 子 agent 被额度打断。
X
cat >> research/prompts/sample-r1-main-verification.md <<'X'
- 核查员撞了会话限额停在半路。
X
cat > records/2026-09-25-新记录.md <<'X'
# 新记录

- 换了账号再跑。
X
cat > research/prompts/_sample-r1-appendix.md <<'X'
- 抄给云端腿读的旧话：撞了周额度。
X
cat > .claude/rules/sample.md <<'X'
- 规则页里的话：撞了周额度。
X
