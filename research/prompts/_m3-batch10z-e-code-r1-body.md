# 根环实现第 10z 批第五块：代码轮第一轮正文（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 G2 -->

## 一、题面

这一轮攻第 10z 批第五块（显存账与第 ① 段派活）。它照第 10z 批第三块代码轮判决 `research/prompts/m3-batch10z-c-code-r1-main-verification.md` 第三节那三条改。

GPU 用在崩溃放量与对拍上的总口径在 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md`（D24（后台重活能不能卸给 GPU））。

用户原话：「全局的任务仅仅管会不会对指定的电脑造成cpu或者gpu的oom」「不压并发」；2026-10-07 定 G1 二（`research/prompts/m3-batch10t-code-r1-forks.md`）；2026-10-08 定 G2「先回甲、同时量乙」（`research/prompts/m3-batch10z-a-code-r1-forks.md`，乙由 E174（借据丢掉时等待与显存回还） 在量，不在这一轮）。

改动全文：`research/prompts/_m3-batch10z-e-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-r1/batch10z-e/patch/report.md`。

| # | 改动 | 落点 |
|---|---|---|
| A | K1-5：叫停时卡只领判完的块在等的那几块（被交回的块），判完的块照块序全部入库、没判的留给下一趟；轮间状态抽成 harness 档 `RoundsOfTheCards`，原第 3081 行两条断言挪进去；`cards_used` 按卡序号去重 | `crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs` |
| B | K1-4 (a)：设备丢失回调把 wgpu 给的那一句与 `wgpu::hal::DeviceError::OutOfMemory` 比对，相同即记显存不够；(b)：活交回那一刻记设备丢没丢，那一刻设备还在的不因之后同块别段的 Out of Memory 说成显存不够 | `gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs` |
| C | K3：`notify_one` 挂住（60 秒超时）、范围里接到的先记再报到、先问叫停再领块三条，正推 `killers.rs` 五条入库；外层循环的报过卡过滤、下一轮只判没领走的、`cards_used` 抽出来测；第 D 条接线加一条 checker 档用例；变异追加 24、整行替换 5 | 三份 src、`crates/mutations.tsv` |

## 二、实现今天的样子（主 agent 的观测，2026-10-08）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/`，取在第四、五块都合入之后；被判文件 sha256 在 `research/prompts/m3-batch10z-e-code-r1-snapshot/crates-sha256.txt`。
- 合后验证（CPU 部分）：见调度记录第 843 行（与第四块同一趟）；clippy 带 gpu / verdict-store 特性 `--all-targets` 退 0、三种特性 `--no-run` 退 0。碰卡的 checker 档 lib 与 GPU 测试排在 E174 之后。
- 实现员证红：两次 prove-red `✓ 点名 8 条…都抓到了`、`✓ 点名 10 条…都抓到了`。
- 实现员自定、被攻过零轮的四处：
  1. 「判完的块在等的」按「比判完的最大块号小、还在队里」来判；
  2. 叫停、这一轮有卡报了显存不够、判完的块都已入库时，结局由「本次未跑」改成「照叫停收尾」（推翻上一块实现员钉的那一行）；
  3. 丢失缘由靠比对 wgpu 那一句的原文；
  4. 交回那一刻的记法留下两段窄窗口（读码推的）。
- 已知没做、不在这一轮被判范围的：G2 甲认的少给；设备丢了、账没被问到那一下读数多给（借之前必经重起判）；真卡上的 Out of Memory 与丢设备造不出。
- 用户已定、报出来不算打中的：两进程同读一份空闲的竞态，一份 `nvidia-smi -q` 报告里约 1.3 毫秒的窗口。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　判决第三节三条做到没有
1. K1-5：叫停落在任一轮任一时点，判完的块是不是都入库、不判红、不判两遍、不漏判？「在等的」那条判法有没有一形把不在等的块当在等、或反过来？`cards_used` 对不对？
2. K1-4 (a)：比对 wgpu 原文——原文换了措辞、或别的错误恰好同文时会怎样？(b)：交回那一刻的两段窄窗口里，真 panic 会不会仍被说成显存不够，真的显存不够会不会判红？
3. 实现员第 2 处推翻上一块的结局，「照叫停收尾」时没判的块是不是都留给下一趟、库里没有缺块？

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
