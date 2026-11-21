#!/usr/bin/env bash
# 基准里一份干净的记录；这一轮往里加调度记录里拿「那一路会话」指别的会话的三句原句（头一句两处），另放一份未跟踪的 kb 页写「那一路的 agent」：四行都报，判红。
# 同一份记录里按出场次序的 mainagent(A) 与反引号里的旧写法一行、kb 页围栏里的旧写法不报（报的行数是 4）。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p records .claude/kb
cat > records/2026-09-24-样本调度.md <<'X'
# 样本调度（2026-09-24）

- 一句干净的话。
X
git add -A && git commit -qm base
cat >> records/2026-09-24-样本调度.md <<'X'
- 先发给「梳理 subagent」那一路会话，它答不归它、指到它的记录 `records/2026-11-17-subagent账本清欠调度记录.md:383`；再发给改检出钩子那一路会话，它认领了，在草稿里把三行点名挪到 `allow-bash-scope-write` 下，等换上工具空出来就换、复跑后发结局行。
- 检出钩子在腿跑着时可能被改检出钩子那一路会话换上 ⑫ 的头注改动，写判决前照快照倒推。
- 检出钩子换上前问过改 ㉒ 那一路会话，换完把新 sha256 发给它。
- mainagent(A) 在改检出钩子，旧写法 `改检出钩子那一路会话` 不再用。
X
printf '%s\n' '# kb 页' '' '- 判据库那一路的 agent 在跑。' '' '```' '改检出钩子那一路会话' '```' > .claude/kb/probe.md
