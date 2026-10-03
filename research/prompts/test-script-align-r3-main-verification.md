# test-script-align-r3 判决（2026-10-01，最后一轮）

正文 `research/prompts/_test-script-align-r3-body.md`；腿：正推 `research/prompts/test-script-align-r3-sonnet-output.md`（V2、V3）、攻方 `research/prompts/test-script-align-r3-opus-output.md`（V1），模型目录各自 `-model/`。前两轮判决 `research/prompts/test-script-align-r1-main-verification.md`、`research/prompts/test-script-align-r2-main-verification.md`。

## 一、核查

- `python3 research/scripts/cite-check.py <两份报告> --root . --background research/prompts/_test-script-align-r3-background.md`：「✓ 核了 54 处引文，对上 54 处，没判 0 处」。
- 开工快照 `research/prompts/test-script-align-r3-snapshot/judged-sha256.txt` 判决前 `sha256sum -c` 全对。
- 攻方 A1、A2 主 agent 现查（`research/prompts/test-script-align-r2-opus-model/hook-probe.py` 喂本仓 `heavy-test-guard.sh`，以 implementation-writer 身份）：`bash research/scripts/mutat[[:alpha:]].sh crates/mutations.tsv` 退 0（A1：POSIX 字符类，钩子的 python glob 不认、零命中落回字面放行）；`bash research/scripts/crash-amplification.sh` 退 0（A2：崩溃放量全量入口不在按名字判的重型表里）；对照 `bash research/scripts/crash-amplification-two-hosts.sh --selftest` 退 2。`bash .claude/gate.d/54-layer0-repla[[:alpha:]].sh <根>` 退 2（这一条另有判法拦住，A1 的形态不是每条都中）。
- 正推 V3 复现了前两轮报的自证格数、种数、开关数、样本判对数、探针退出码，都对得上；对不上的几处它写明了原数、复现数与命令（54 号 12 个开关里 6 个文件头点名的样本与实测判错的不相等；术语计数的原取法没留下）。主 agent 没逐条复跑，照它的模型目录采。

## 二、判定与去向

| 格 | 判定 | 去向 |
|---|---|---|
| A1 POSIX 字符类通配绕过钩子 | 采；零轮 | 交用户表第 1 行 |
| A2 崩溃放量全量入口子 agent 能直接跑 | 采；零轮 | 交用户表第 1 行 |
| A3 hook-swap.py 换上一半与回滚冲突 | 没打中（只抽一次，不作「没打中」用） | 不改 |
| A4 崩溃放量全量没有人能派 | 采 | 新记欠账：崩溃放量全量没有门禁阶段、崩溃验证员定义里没有这一道、派发表里没有 runner；交用户表第 5 行 |
| V2 漏记 L1–L7 | 采 | 新记欠账（L1 checker 档一条用例一个场景、L2 用例函数名带欠账号、L3 门禁弄坏开关没有复跑方并改 C625 描述、L4 四份自证起真重活与上游不合、L5 `replay.sh` 与 `broad-mutation.py` 没按入口脚本格式核、L6 调度记录里「下一批」尾巴进欠账表）；L7 改 C612、C613 两处过期句 |
| V2 两笔矛盾 | 采 | C20 的「判红」改成照 D13（验证路线） 已定项 9「提交照样可以进，记一笔」；C409 引号里改成 D13 现文逐字；C604 前置补全 |
| V2 三处「未定」 | 交用户 | C623（K 常量 / R 现读 / 按项登记）、C625（A / B / C / D 四种文件头写法）、C622（M1 开的人设 / M2 样本根里放 / M3 向上游提）：候选定义与翻面观测在正推报告 1.2 节；交用户表第 2–4 行 |
| V3 前两轮的数 | 复现一致 | 54 号文件头六个开关的点名样本与实测不符，记一笔改文件头（下一批） |

## 三、交用户（零轮与岔路）

| # | 事 | 候选 | 状态 |
|---|---|---|---|
| 1 | A1、A2 两个现成的绕过 | 现在就修（A1：命令词带 `[[:`、`[[=`、`[[.` 的用 bash 自己展开，认不出就拒；A2：`crash-amplification.sh`、`crash-amplification-item.sh` 进按名字判的重型表，`--selftest` 照登记）；或留到下一批 | 零轮，交用户 |
| 2 | C623 棘轮 | K / R（读范围三种）/ 按项登记 | 交用户 |
| 3 | C625 文件头写法 | A / B / C / D | 交用户 |
| 4 | C622 标记谁设 | M1 / M2 / M3 | 交用户 |
| 5 | 崩溃放量全量由谁跑 | 记欠账，下一批在崩溃验证员定义或门禁里加一道 | 记账 |

用户 2026-10-01 弹窗定：第 1 行「现在就修」（交 `tooling-writer`，经 `research/scripts/hook-swap.py` 换上，零轮）；第 2 行 C623 取「按项登记」；第 3 行 C625 取「B mutate.sh 写法」；第 4 行 C622 取「M2 样本根里放」。三处定案交 `kb-scribe` 写回欠账表。

## 四、被判文件里点名的定义与共用约束

`.claude/main-agent.md`「禁止」第一条、`.claude/agents/tooling-writer.md` 第 3 步：这一轮判它们照字面走里程碑收尾（A4），做不了的一步是崩溃放量全量没有 runner，记欠账，两份定义的现文不改。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 9 | 不影响 | 2026-10-01 不受影响：这一轮只核它的现文照字面走不走得通；崩溃放量全量没有 runner 记欠账，不改定案 |
| D13（验证路线） 已定项 15 | 不影响 | 2026-10-01 不受影响 |
