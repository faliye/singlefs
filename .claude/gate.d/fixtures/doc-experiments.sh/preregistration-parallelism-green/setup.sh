#!/usr/bin/env bash
# 绿样本：基准提交里只有一份号小于 186 的旧登记（e100，没有「并行与挂钟」一节）；提交之后改了它（不判），
# 又新建一份号不小于 186 的 e993，五样写全（标签行一份用列表记号、一份用加粗，围栏代码块里的假标签行不算）。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p research/prompts
printf '# E100 旧登记\n\n## 一、问题\n\n英文名：old_sample\n' > research/prompts/e100-preregistration.md
git add -A
git commit -qm '基准：一份旧登记'
printf '\n## 十二、修订\n\n收严一条门槛。\n' >> research/prompts/e100-preregistration.md
cat > research/prompts/e993-preregistration.md <<'EOF'
# E993 样本登记

## 十四、并行与挂钟

```
串行：围栏里这一行不算
```

- 独立单位：十二个取样点互相独立，每个取样点的造池不共享状态
- **线程**：取样点分到每台默认重活线程数条线程上，取 .claude/heavy-thread-default 的登记值
- 串行：计时窗口必须单线程跑完（量的是单条流的挂钟），窗口里不起别的线程
- 预计挂钟：第一段约 40 分钟（推的：单个取样点约 3 分钟乘 12 个、除以并行的取样点数）
- 预计在用核数：造池阶段约 12 核（取样点只有 12 个），计时窗口里 1 核，推的

## 十五、别的
EOF
