# 根环实现第 10z 批第三块：代码轮第一轮正文（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 G2 S6 P2 -->

## 一、题面

这一轮攻第 10z 批第三块（显存账）。它照第 10z 批第一块代码轮判决 `research/prompts/m3-batch10z-a-code-r1-main-verification.md` 第三节「不随 G2 变」那几条与第四节用户定案改。

GPU 用在崩溃放量与对拍上的总口径在 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md`（D24（后台重活能不能卸给 GPU））。

用户原话：
- 「全局的任务仅仅管会不会对指定的电脑造成cpu或者gpu的oom」；
- 「不压并发」；
- 2026-10-07 定 G1 二（岔路单 `research/prompts/m3-batch10t-code-r1-forks.md`）；
- 2026-10-08 定 G2「先回甲、同时量乙」（岔路单 `research/prompts/m3-batch10z-a-code-r1-forks.md`）：销了账、卡上可能还没还的那一份只压到「这个进程此刻占着的」为止，拿掉「见到的那一份」。乙由 E174（借据丢掉时等待与显存回还） 上卡量，不在这一轮。

改动全文：`research/prompts/_m3-batch10z-c-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-r1/batch10z-c/patch/report.md`。

| # | 改动 | 落点 |
|---|---|---|
| A | G2 甲：那一份每次读数只压到这个进程此刻占着的，「见到的那一份」连同三处用法拿掉 | `crates/singlefs-harness/src/gpu_memory_ledger.rs` |
| B | S6：设备丢了重起时记成 `DeviceLostNotStartedAgain`，新设备起好之前每次再来照此刻空闲再判 | `gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs` |
| C | 对拍用例读 `stopped_because_the_gpu_memory_was_short`：Some 时打本次未跑那一行；None 时 `states_checked` 钉成这条流的总状态数 | `crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs` |
| D | 范围里接到的 Out of Memory：一块的几段各自报到，先记进设备、等同块几段都报到再认（`end_the_work_and_explain_its_failure`） | `gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs` |
| E | P2：卡线程那一圈、块队、按块序写库、要不要再起一轮挪成 harness 档纯函数；第 ① 段按轮跑，别的卡都收工、还有块没判时收尾再起一轮，只用没报过 Out of Memory 的卡 | `gpu_memory_ledger.rs`、`crash_amplification.rs` |
| F | K3 补用例与变异；删 `lease_card_waiting_for_room` | 三份 src、`crates/mutations.tsv` |

## 二、实现今天的样子（主 agent 的观测，2026-10-08）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-c-code-r1/tree/`，取在 10z 第三块合入之后；被判文件 sha256 在 `research/prompts/m3-batch10z-c-code-r1-snapshot/crates-sha256.txt`。
- 合后验证与变异证红：见调度记录「10z-c 合后验证」那一行。
- 实现员在两腿模型上跑的（改后树）：多给一律 0（攻方 `mode=` 四档 overgive 0、丁 difference=0、重起扫描无放行而放不下；正推 S3 账 18550 等于真 18550、FUZZ 12 组 over_count 0）；少给照实（攻方甲十轮 8950 对 19550，随机四档放得下借不到 146 / 4969 / 5222 / 5067 次；S10 第 k 笔之后 19550 − 1000k）。这些是甲认的代价，报出来不算打中。
- 实现员自定、被攻过零轮的：第 D 条等同块几段都收完再认；第 E 条收尾再起一轮（轮数不超过卡数）、再起的一轮重新借草稿区。
- 已知没做、不在这一轮被判范围的（腿报出来不算打中）：
  - 重起之后少给 D、压不掉（正推 S4、S9）：甲之下照旧；
  - 第 D 条在 checker 档的接线、第 E 条按轮跑的循环本身没有不要卡的用例；
  - 真卡上的 Out of Memory、真丢设备造不出。
- 用户已定、报出来不算打中的：两进程同读一份空闲的竞态，一份 `nvidia-smi -q` 报告里约 1.3 毫秒的窗口，由运行时兜底。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　判决第三节各条与 G2 甲做到没有
1. G2 甲：账给的量在任何一段历史上是不是都不大于此刻真能再借的？有没有别的路径（重起、乙附、不走卡池的设备）还按「见到的」或别的猜法往下压？
2. S6：被拒之后再来的每一次都判了吗？新设备起好之后账回到正常没有？
3. 对拍用例：Some 时本次未跑、None 时钉总状态数，各有没有一形照绿？
4. 第 D 条：同块几段都报到再认，有没有一形等不齐（一段永远不报到）、真 panic 被说成显存不够、或真的显存不够判红？
5. 第 E 条：再起一轮有没有一形判两遍、漏判、判完不入库、或无限再起？

### K2　只防 OOM、不压并发两头（新代码引进来的）
1. 有没有一形让账给的量大于此刻真能用的？
2. 除甲认的少给之外，有没有一形卡放得下却报本次未跑、判红、或整趟弃卡？

### K3　测试判别力
新加的用例与变异行，能不能拦住这一批每一处被改回去？有没有一形「改回去」照绿？

## 四、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
