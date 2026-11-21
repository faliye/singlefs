#!/usr/bin/env bash
# 基准里一份干净的记录、一份判决、一份三方材料；这一轮往记录里加一行干净的话，往判决里加带任务号与批号的行，
# 往三方材料里加带批号的行，另放一份未跟踪的三方材料带任务号：research/prompts/ 下是冻结证据，这一格不判，判绿；
# 判了的只有那份干净的记录（成功句报判了 1 份、冻结证据 3 份）。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p records research/prompts
cat > records/2026-09-24-样本调度.md <<'X'
# 样本调度（2026-09-24）

- 一句干净的话。
X
printf '%s\n' '# 判决' '' '- 干净的一行。' > research/prompts/x-main-verification.md
printf '%s\n' '# 材料' '' '- 干净的一行。' > research/prompts/_x-body.md
git add -A && git commit -qm base
cat >> records/2026-09-24-样本调度.md <<'X'
- 一句干净的新话。
X
printf '%s\n' '- A/B 读数：任务 12400，结局 task=12400 type=succeeded exit=0' '- 调度那一条跑完（任务 21036）。' >> research/prompts/x-main-verification.md
printf '%s\n' '- 那一路第 7 批交回。' >> research/prompts/_x-body.md
printf '%s\n' '# 腿的报告' '' '- 它的批 105 在修，同 20461、20814。' > research/prompts/x-leg-output.md
