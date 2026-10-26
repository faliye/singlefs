#!/usr/bin/env bash
# 红样本：基准提交里只有一份说明；提交之后新建 e995，第六节写「变异 17 条」，「并行与挂钟」一节的「预计挂钟：」写「变异 24 条、每条…约 1 分钟，串行约 20 分钟」：
# 挂钟算式对得上（24 × 1 分钟与 20 分钟差不到一个数量级），态数 24 条与第六节的 17 条对不上，点名判红。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p research/prompts
printf '# 样本\n' > research/prompts/README.md
git add -A
git commit -qm '基准：一份说明'
cat > research/prompts/e995-preregistration.md <<'REGISTRATION'
# E995 样本登记

## 六、报哪些量与各自的判据

| 量 | 怎么算 | 门槛 |
|---|---|---|
| 变异抓获 | 变异 17 条，每条都要被抓 | 全抓判过 |

## 十四、并行与挂钟

- 独立单位：每条变异各自重编各自跑，互不共享编译目录
- 线程：变异分到每台默认重活线程数条线程上，取 .claude/heavy-thread-default 的登记值
- 串行：无，变异之间没有计时也没有共享状态，都能并行跑
- 预计挂钟：变异 24 条、每条重编加跑测试约 1 分钟，串行约 20 分钟（推的）
- 预计在用核数：并行时约 24 核（每条变异一核），推的
REGISTRATION
