#!/usr/bin/env bash
# 红样本：基准提交里只有一份号小于 186 的旧登记（e100，没有「并行与挂钟」一节）；提交之后改了它（不判），
# 又新建三份号不小于 186 的：e990 缺整节，e991 有节而缺「串行：」一行，e992「预计在用核数：」没有数字。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p research/prompts
printf '# E100 旧登记\n\n## 一、问题\n\n英文名：old_sample\n' > research/prompts/e100-preregistration.md
git add -A
git commit -qm '基准：一份旧登记'
printf '\n## 十二、修订\n\n收严一条门槛。\n' >> research/prompts/e100-preregistration.md
cat > research/prompts/e990-preregistration.md <<'EOF'
# E990 样本登记

## 一、问题

英文名：missing_section_sample

## 十三、读过的文件与跑过的命令

无。
EOF
cat > research/prompts/e991-preregistration.md <<'EOF'
# E991 样本登记

## 十四、并行与挂钟

- 独立单位：十二个取样点互相独立，每个取样点的造池不共享状态
- 线程：取样点分到每台默认重活线程数条线程上，取登记值
- 预计挂钟：第一段约 40 分钟（推的：单个取样点约 3 分钟乘 12 个除以线程数）
- 预计在用核数：造池阶段约 12 核（取样点只有 12 个），推的

## 十五、别的
EOF
cat > research/prompts/e992-preregistration.md <<'EOF'
# E992 样本登记

## 十四、并行与挂钟

**独立单位：** 八个几何格互相独立，各格自己建状态
**线程：** 几何格分到登记的线程数上，格内的计时窗口不分
**串行：** 计时窗口必须单线程跑完，同一台机器上不许有本进程别的活抢核
**预计挂钟：** 两段各约 90 分钟，依据是上一次同形装置的单格用时
**预计在用核数：** 造池时贴着登记数，计时窗口里只有一个核在用
EOF
