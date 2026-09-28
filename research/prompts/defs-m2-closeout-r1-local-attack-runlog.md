# 运行记录：defs-m2-closeout-r1-local-attack（2026-09-26）

提示文件：`research/prompts/defs-m2-closeout-r1-local-attack.md`（英文，D2 攻击面：
按事实表把 `agent-common.md`「执行前拒绝的写法」列的每一种写法与探针日志逐格核对，
Exhibit A 74 个 C 码、Exhibit B 64 个 P 码，Q1–Q12 共 12 个标签）。
转述核对表：`research/prompts/defs-m2-closeout-r1-local-attack-translation-audit.md`。

调用方式：`nice -n 19 bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-sN.md`，
每次前台起（未用 `setsid` / `&` / `disown`）；命令本身超过工具默认 120 秒前台等待时被
调度环境自动转入后台，随后靠任务完成通知得知结果，未另起等待循环轮询。跑之前
`ps -o pid,args -u "$(id -u)"` 查过 `qemu-system` / `vm-bench.sh` / `e152-file-system-benchmark` /
`fio`：零命中，不是性能测量在跑。网关取 key 用的 `~/code/ai-center/.env.tenants` 存在。

每份样本另跑 `python3 research/scripts/oov-check.py <样本>`（单参数）与
`python3 research/scripts/corruption-check.py <样本>`，两个都判绿之后仍逐份通读全文。

## 逐次调用

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 通读复核 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `defs-m2-closeout-r1-local-attack-output-s1.md` | 0 | 1298 | 绿，生词=0 拼接=0 | 绿，全部计数为 0 | 通读 99 行，74 个 C 码逐条应答齐全、Q11/Q12 齐全，无缺词、无断句、无孤立标点、无粘连词、无拼写变形 | 干净 |
| 2 | `defs-m2-closeout-r1-local-attack-output-s2.md` | 0 | 1017 | 绿，生词=0 拼接=0 | 绿，全部计数为 0 | 通读发现系统性损坏：整份文本里凡是判词该是 UNTESTED 的地方（本该出现 27 次），全部拼成 `UNTESFED`（T 被换成 F），一次都没有拼对；`UNTESFED` 不成词、词表里也没有，但既不是两个词表词的拼接、也不是复读或成对标记落单，`oov-check.py`／`corruption-check.py` 的规则都抓不到这种单字母替换型损坏，两者因此都判绿 | 带损坏（不算干净样本） |
| 3 | `defs-m2-closeout-r1-local-attack-output-s3.md` | 0 | 958 | 绿，生词=0 拼接=0 | 绿，全部计数为 0 | 通读 176 行，74 个 C 码逐条应答齐全、Q11/Q12 齐全，UNTESTED 全部拼写正确，无缺词、无断句、无孤立标点、无粘连词、无拼写变形 | 干净 |

## 干净样本清点

- 干净：s1、s3，共 2 份，达到「至少两份干净样本」的门槛，到此停止抽样。
- 带损坏：s2（`UNTESFED` 单字母替换型拼写损坏，出现 27 次，通读发现、两个自动检测器均判绿未抓到），不当干净样本。
- 作废：无。三次调用退出码均为 0，没有出现退出码 5（判红作废）、没有出现非 0 非 5 的退出码，
  没有网关不通；三次调用未超过「连续五次调用」的上限。
- 目录核对：`ls research/prompts/ | grep defs-m2-closeout-r1-local-attack` 显示提示文件 1 份、
  核对表 1 份、运行记录 1 份（本文件）、`-output-s1.md` 至 `-output-s3.md` 共 3 份，
  没有 `-output-void*.md`，与本记录逐行一致。

## 没做什么

- 不解读、不总结、不采纳三份样本（含带损坏的 s2）里 Q1 至 Q12 对每个 C 码给出的具体判词
  （MATCH / EXITCODE / VERDICT 与推翻句），不判 s1 与 s3 两份干净样本之间的判词方向是否一致——
  这两件事都是主 agent 的事，不归这份运行记录。
- 未跑云端攻方腿（Opus，D1/D2/D3/D4）、云端正推腿（Sonnet，D1/D2/D3）或核实员，不在这条腿的任务范围内。
- 未读这一轮别的腿的提示与产出（`research/prompts/defs-m2-closeout-r1-sonnet-*`、
  `research/prompts/defs-m2-closeout-r1-opus-*`）——按主 agent 给的禁读清单，全程未读。
- 未使用 `/tmp/claude-1000/defs-m2-closeout-r1-local-attack/` 草稿目录——这一轮没有需要落草稿的
  中间产物，提示、核对表与样本都直接定稿写进 `research/prompts/`，目录留空。
