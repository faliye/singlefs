#!/usr/bin/env bash
# 与 account-quota-red 同一个基准；这一轮加的行只有显卡额度、内存额度、空间配额这类正常用法，欠账号、版本号换号，带理由的放行标记，
# 命中处附近有资源词的裸词命中（撞额度：账上 8G、显卡额度重置、显存用量上限），判绿。基准里的旧行带「撞了限额」，不报；射程之外的未跟踪文件带同样的话，也不报。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p records .claude/kb research/prompts .claude/rules research/prompts/sample-snapshot
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
- 显卡额度按优先级先后分，GPU_CARDS_BY_PRIORITY 的额度写在本机配置里。
- 内存上限 12G，内存额度不够时不再起新的一件。
- 与变异轻型撞额度：账上 8G 加回收不掉的 2G。
- 显卡额度重置后再分。
- 空间配额超了就拒绝写。
- 欠账号 C820 到 C829 留给另一条分支；快照换号不换号全仓没写。
- 子 agent 因此停下的：中途停下没交回。
- 用户原话撞了周额度，原样留。account-quota:allow 用户原话
X
cat >> .claude/kb/01-样本页.md <<'X'
- 账户余额不碍事，显存用量上限 24G。
X
cat >> research/prompts/sample-r1-main-verification.md <<'X'
- 三条腿的显卡额度各占一张卡。
X
cat > research/prompts/_sample-r1-appendix.md <<'X'
- 抄给云端腿读的旧话：撞了周额度。
X
cat > research/prompts/sample-snapshot/copy.md <<'X'
- 快照副本里的旧话：切换账号后续做。
X
cat > .claude/rules/sample.md <<'X'
- 规则页里的话：撞了周额度。
X
