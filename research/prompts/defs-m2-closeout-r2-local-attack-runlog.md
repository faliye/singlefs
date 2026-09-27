# 运行记录：defs-m2-closeout-r2-local-attack（2026-09-26 UTC）

提示文件：`research/prompts/defs-m2-closeout-r2-local-attack.md`（英文，E1 的
F14 攻击面：把 `implementation-workflow.md` 改后重型清单拆成 28 个写法条目
C1.1–C12.1，与探针日志 K1–K29 逐格核对 MATCH / EXITCODE / VERDICT，
再用 Q15/Q16 反向核「清单写了而 classify/cargo_use 源码里没有对应分支」
「源码里的 kind 而清单没提」两类情形，Q1–Q16 共 16 个标签）。
转述核对表：`research/prompts/defs-m2-closeout-r2-local-attack-translation-audit.md`。

调用方式：`nice -n 19 bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-sN.md`，
每次前台起，未用 `setsid` / `&` / `disown`。跑之前 `ps -o pid,args -u "$(id -u)"`
查过 `qemu-system` / `vm-bench.sh` / `e152-file-system-benchmark` / `fio`：两次都零命中，
不是性能测量在跑。网关取 key 用的 `~/code/ai-center/.env.tenants` 存在（`curl` 探测
`:8200/v1/chat/completions` 有响应）。

每份样本另跑 `python3 research/scripts/oov-check.py <样本>` 与
`python3 research/scripts/corruption-check.py <样本>`，两个都判绿之后仍逐份通读全文。

## 逐次调用

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 通读复核 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `defs-m2-closeout-r2-local-attack-output-s1.md` | 0 | 607 | 绿，生词=1（`CONTRADICTED`，提示自定义的判词术语，不是粘连词）拼接=0 | 绿，全部计数为 0 | 通读 92 行：Q1–Q16 全部齐全，Q15 逐条 28 个 C 码答全，Q16 逐条 17 个 kind 名答全；无缺词、无断句、无孤立标点、无粘连词、无拼写变形；答复未写任何代码行号或文件行号 | 干净 |
| 2 | `defs-m2-closeout-r2-local-attack-output-s2.md` | 0 | 571 | 绿，生词=1（`CONTRADICTED`，同上）拼接=0 | 绿，全部计数为 0 | 通读 136 行：Q1–Q16 全部齐全，Q15、Q16 同样逐条答全（28 个 C 码、17 个 kind 名）；无缺词、无断句、无孤立标点、无粘连词、无拼写变形；答复未写任何代码行号或文件行号 | 干净 |

## 干净样本清点

- 干净：s1、s2，共 2 份，达到「至少两份干净样本」的门槛，到此停止抽样。
- 带损坏：无。
- 作废：无。两次调用退出码均为 0，没有出现退出码 5（判红作废）、没有出现非 0 非 5 的退出码，
  没有网关不通；两次调用远未达到「连续五次调用」的上限。
- 目录核对：`ls research/prompts/ | grep defs-m2-closeout-r2-local-attack` 显示提示文件 1 份、
  核对表 1 份、运行记录 1 份（本文件）、`-output-s1.md` 与 `-output-s2.md` 共 2 份，
  没有 `-output-void*.md`，与本记录逐行一致。

## 没做什么

- 不解读、不总结、不采纳两份样本里 Q1 至 Q16 对每个 C 码、每个 K 码、每个 kind 名给出的
  具体判词（MATCH / EXITCODE / VERDICT、NOT IN SOURCE、NOT IN CHECKLIST 与推翻句），
  不判 s1 与 s2 两份干净样本之间的判词方向是否一致——这两件事都是主 agent 的事，不归这份运行记录。
- 未跑云端攻方腿（Opus，E1、E2）、云端辩方腿（Sonnet，E3）或核实员，不在这条腿的任务范围内。
- 未读这一轮别的腿的提示与产出（`research/prompts/defs-m2-closeout-r2-sonnet-*`、
  `research/prompts/defs-m2-closeout-r2-opus-*`）——按主 agent 给的禁读清单，全程未读。
- 未使用 `/tmp/claude-1000/defs-m2-closeout-r2-local-attack/` 草稿目录——这一轮没有需要落草稿的
  中间产物，提示、核对表与样本都直接定稿写进 `research/prompts/`，目录留空。
