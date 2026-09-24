# 运行记录：m2-checker-supp-code-r1-local-attack（2026-09-21 UTC / 2026-09-22 JST）

提示文件：`research/prompts/m2-checker-supp-code-r1-local-attack.md`（英文；K4 全面 + K3 里
「实现与判据原文逐字对不对得上」那几格；16 条 Fact，5 道 Judgment，各 3 项子答复，合计 15 格，
在「一份不超过 16 格」之内）。不用任何 markdown 强调，答案按 Judgment 编号，指路只用 Fact 编号，
明令不写文件行号或代码行号（需要指代码时按函数名或 Fact 编号）。
转述核对表：`research/prompts/m2-checker-supp-code-r1-local-attack-translation-audit.md`。

调用方式：`nice -n 19 bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-sN.md`，
前台跑，未用 `setsid` / `&` / `disown`。跑之前 `ps -o pid,etimes,pcpu,args -u "$(id -u)"` 查过
`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`：全程零命中，不是性能测量在跑；
另有别的会话在跑 `cargo test --all`、`cargo build --all-targets` 与两个 `gate.sh --staged`，
按共用约束这类负载不挡本地腿，未等锁。

## 逐次调用

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 结论 |
|---|---|---|---|---|---|---|
| 1 | `m2-checker-supp-code-r1-local-attack-output-s1.md` | 0 | 407 | 绿，生词=2（checker's 及其复数形式，规则表外的正常英语所有格派生词），拼接=0 | 绿，`cjk=0 words=387 fffd=0` 各类复读/粘连/落单计数全部为 0 | 干净 |
| 2 | `m2-checker-supp-code-r1-local-attack-output-s2.md` | 0 | 188 | 绿，生词=1（checker's），拼接=0 | 绿，`cjk=0 words=183 fffd=0` 各类复读/粘连/落单计数全部为 0 | 干净 |

通读两份全文：5 道 Judgment 各给出 verdict / justification / this would be refuted by 三项，
无缺词、无断句、无孤立标点、无列表项整词缺失；s2 的行文比 s1 简练（三项合写成一句），
但每一项都完整、无残缺。两份干净样本达标，到此为止，不再抽样。

## 干净样本清点

- 2 份干净（s1、s2），首两次调用即达标，未撞第 5 次的停机线。
- 提示未撞输出预算（15 格，未拆分）。
- 没有判红作废（退出码 5）的调用，没有 void 副本
  （`ls research/prompts/ | grep m2-checker-supp-code-r1-local-attack` 核过，
  目录下只有一份提示、两份 output-sN、一份核对表，没有 output-void 文件）。
- 没有退出码非 0 非 5、也没有网关不通的情况，本地腿全程未缺席；两次调用全部退出码 0，
  一次重跑都没用上。

## 没做什么

- 不解读、不总结、不采纳两份样本对 Judgment 1 至 5 的具体判词，不判两次抽样之间方向是否一致
  ——那是主 agent 的事。
- 未跑云端攻方腿（Opus，K1/K2）、云端正推腿（Sonnet，K3 里要构造镜像的那几格），不在这条腿的
  任务范围内。
- 未碰 K1、K2 两格（事务号），也未碰 K3 里要构造让判据误判或判不出的合法/坏镜像的那几格
  ——按分工表这两类都不归本地攻方腿。
- 未读这一轮别的腿的提示与产出、主 agent 这一轮的核实、同时在飞的别的轮次
  （禁读清单未在派发提示里单列，主 agent 也未点名要读别的腿的产出，未主动去读）。
