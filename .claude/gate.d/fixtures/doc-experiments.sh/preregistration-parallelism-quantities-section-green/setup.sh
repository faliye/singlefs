#!/usr/bin/env bash
# 绿样本：基准提交里只有一份说明；提交之后新建两份登记。
# e996：第六节写「每段 10000 次发布，共 960 段」，「预计挂钟：」的式子「960 段 × 10000 次 × 0.5 毫秒 ÷ 31 ≈ 155 秒」，两个态数都对上（次与次发布量词互为开头、同值），比了 2 个；
# e997：第六节只写定义、不写数，「预计挂钟：」写「12 个取样点、每个取样点约 3 分钟」，态数在第六节里认不出，报「态数与第六节的比对本次未判 1 份」并点名，不算红。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p research/prompts
printf '# 样本\n' > research/prompts/README.md
git add -A
git commit -qm '基准：一份说明'
cat > research/prompts/e996-preregistration.md <<'REGISTRATION'
# E996 样本登记

## 六、报哪些量与各自的判据

| 量 | 怎么算 | 门槛 |
|---|---|---|
| 发布开销 | 每段 10000 次发布，共 960 段 | 中位数不过门槛 |

## 十四、并行与挂钟

- 独立单位：960 段微历史互相独立，每段各自造池
- 线程：段分到每台默认重活线程数条线程上，取 .claude/heavy-thread-default 的登记值
- 串行：无，段之间没有计时窗口也没有共享状态，都能并行
- 预计挂钟：约 5 分钟（960 段 × 10000 次 × 0.5 毫秒 ÷ 31 ≈ 155 秒，留一倍余量；推的）
- 预计在用核数：并行时约 31 核（段数远多于线程数），推的
REGISTRATION
cat > research/prompts/e997-preregistration.md <<'REGISTRATION'
# E997 样本登记

## 六、报哪些量与各自的判据

| 量 | 怎么算 | 门槛 |
|---|---|---|
| 落点偏离 | 每个取样点上落点偏离政策函数的次数 | 恒为零判过 |

## 十四、并行与挂钟

- 独立单位：取样点互相独立，每个取样点的造池不共享状态
- 线程：取样点分到每台默认重活线程数条线程上，取 .claude/heavy-thread-default 的登记值
- 串行：计时窗口必须单线程跑完（量的是单条流的挂钟），窗口里不起别的线程
- 预计挂钟：12 个取样点、每个取样点约 3 分钟，串行约 36 分钟（推的）
- 预计在用核数：造池阶段约 12 核（取样点只有 12 个），计时窗口里 1 核，推的
REGISTRATION
