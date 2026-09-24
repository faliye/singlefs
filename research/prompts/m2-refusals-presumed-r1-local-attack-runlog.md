# 运行记录：m2-refusals-presumed-r1-local-attack（2026-09-22 UTC / 2026-09-23 JST）

提示文件：`research/prompts/m2-refusals-presumed-r1-local-attack.md`（英文；分工表 R5–R8 四格，
20 条 Fact、4 道 Judgment，每道 Judgment 四个子部分对应四列①②③④）。不用任何 markdown 强调，
未用反引号、星号、竖线表格；答案按 Judgment 编号，指路只用 Fact 编号，明令不写文件行号或代码
行号（需要指代码时按函数名或 Fact 编号指）。
转述核对表：`research/prompts/m2-refusals-presumed-r1-local-attack-translation-audit.md`。

调用方式：`nice -n 19 bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-sN.md`，
前台跑，未用 `setsid` / `&` / `disown`（第二次调用超过 Bash 工具默认 120s 超时，被工具自身移入
后台等待完成通知，非本地攻方腿主动起后台）。跑之前 `ps -o pid,etimes,pcpu,args -u "$(id -u)"`
查过 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`：全程零命中，不是性能测量
在跑；另有别的会话在跑 `cargo test --all`（两个进程），按共用约束这类负载不挡本地腿，未等锁。

## 逐次调用

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 结论 |
|---|---|---|---|---|---|---|
| 1 | `m2-refusals-presumed-r1-local-attack-output-s1.md` | 0 | 365 | 绿，生词=4（histories、defensible 等，正常英语词，非拼接） | 绿，`cjk=0 words=349 fffd=0` 各类复读/粘连/落单计数全部为 0 | 干净 |
| 2 | `m2-refusals-presumed-r1-local-attack-output-s2.md` | 0 | 313 | 绿，生词=3（defensible、histories），拼接=0 | 绿，`cjk=0 words=279 fffd=0` 各类复读/粘连/落单计数全部为 0 | 干净 |

通读两份全文：两份都以「判定编号 + 每部分一行短答复」的极简格式作答（不是提示里要求的完整段落，
只给判词 + fact 编号 + 一句 refuted-by），但每一行本身完整、无缺词、无断句、无孤立标点、无列表项
整词缺失、无缺头的粘连词（例如 `achievesceives` 那一类）。两份干净样本达标，到此为止，不再抽样。

## 干净样本清点

- 2 份干净（s1、s2），首两次调用即达标，未撞第 5 次的停机线。
- 没有判红作废（退出码 5）的调用，没有 void 副本
  （`ls research/prompts/ | grep m2-refusals-presumed-r1-local-attack` 核过，
  目录下只有一份提示、两份 output-sN、一份核对表，没有 output-void 文件）。
- 没有退出码非 0 非 5、也没有网关不通的情况，本地腿全程未缺席；两次调用全部退出码 0，
  一次重跑都没用上。
- 本轮 R5–R8 四格全部拿到了答复（每格四部分），没有因为提示预算或调用失败而缺格。

## 没做什么

- 不解读、不总结、不采纳两份样本对 R5–R8 各格①②③④的具体判词，不判两次抽样之间方向是否一致
  ——那是主 agent 的事。
- 未跑云端攻方腿（Opus，R1–R4、Q1）、云端正推腿（Sonnet，P1–P7），不在这条腿的任务范围内，
  也未读它们已交回的产出（`m2-refusals-presumed-r1-sonnet-output.md`、
  `m2-refusals-presumed-r1-opus-output.md`）——按派发提示的禁读要求未读。
- 未碰 R1–R4、P1–P7、Q1 任何一格，按分工表只归本地攻方腿的 R5–R8 四格。
- 未读这一轮别的腿的提示与产出、主 agent 这一轮的核实、同时在飞的别的轮次
  （禁读清单在派发提示里已列明：本轮云端两条腿的产出）。
- 未验证提示里 Fact 4/5/7/8/9/10/11/14/15/16/18/19/20 所依据的源码与 kb 条款是否在两次调用
  之间发生了变化（两次调用间隔很短，未重新现查；若主 agent 需要更晚时刻的现查结果，需另行核实）。

