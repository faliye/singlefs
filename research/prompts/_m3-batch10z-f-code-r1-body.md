# 根环实现第 10z 批第六块：代码轮第一轮正文（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 K4 G2 F2 F4 F5 M05 M10 M13 M14 M17 M22 M23 -->

## 一、题面

这一轮攻第 10z 批第六块（G2 乙，加第五块判决第三节三条）。

GPU 用在崩溃放量与对拍上的总口径在 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md`（D24（后台重活能不能卸给 GPU））。

用户原话：
- 「全局的任务仅仅管会不会对指定的电脑造成cpu或者gpu的oom」；
- 「不压并发」；
- 2026-10-08 照 E174（借据丢掉时等待与显存回还） 两段的数定 G2「改用乙」（岔路单 `research/prompts/m3-batch10z-a-code-r1-forks.md` G2 行）：借据丢掉时先等设备做完再销账，「销了账、卡上可能还没还」那一份整个拿掉。E174 页 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md`。

第五块判决：`research/prompts/m3-batch10z-e-code-r1-main-verification.md` 第三节。

改动全文：`research/prompts/_m3-batch10z-f-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-r1/batch10z-f/patch/report.md`。

| # | 改动 | 落点 |
|---|---|---|
| A | G2 乙：`GpuCardLease` 丢掉时 `Poll` 改成 `Wait { submission_index: None, timeout: 30 秒 }`，等完才销账，poll 包在 `catch_unwind` 里；「销了账、卡上可能还没还」字段与按读数往下压那一步删掉；等失败那一笔不销账、到这张卡的设备丢了、账照重起记那一刻销；丢了的设备在 `DeviceLostNotStartedAgain` 与新加的 `DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain` 两种状态下照读不到这个进程占着的算 | `crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-harness/src/gpu_memory_ledger.rs` |
| B | 第五块判决第三节第 1 条：`StopRequestRememberedInThisRun`，一趟里见过一次叫停就记住，外层循环、卡线程、一轮收尾、整趟收尾都问它 | `gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs` |
| C | 第三节第 2 条：这一趟有卡报过 Out of Memory 而照叫停收尾时，头一句 OOM 照本次未跑那一行的形态打出来 | `crash_amplification.rs` |
| D | 第三节第 3 条：正推七条会红用例（M05、M10、M13、M14、M17、M22、M23）并进测试段；交回那一刻记设备丢没丢、起下一张卡看不看叫停抽到 harness 档；丢失缘由用例写死「Out of memory」，加一条扫 wgpu-core 源码的不要卡用例；新碰卡用例 `crates/singlefs-checker-tier/tests/gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs`、新 harness 档用例 `crates/singlefs-harness/tests/gpu_card_rounds_remember_a_stop_request_removed_mid_run.rs`；变异追加 27、整行替换 9、删 7 | 三份 src、两份新测试、`crates/mutations.tsv` |

## 二、实现今天的样子（主 agent 的观测，2026-10-08）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-f-code-r1/tree/`，取在第六块合入之后（底含删除线第 45 批）；被判文件 sha256 在 `research/prompts/m3-batch10z-f-code-r1-snapshot/crates-sha256.txt`。
- 合后验证（`~/.local/share/singlefs/work/m3-r1/merge-10z-f/summary.txt`，调度记录第 856 行）：
  - harness 档 186 个测试二进制全绿，fmt、clippy 过；
  - checker 档 `--no-run` 与 `build --all-targets` 退 0，checker 档 lib rc=0；
  - 新碰卡用例在真卡上 rc=0；
  - 挂 gpu 特性的 12 份测试 11 份 rc=0。
- 已知、在修、报出来不算打中（第 10z 批第七块）：
  - `next_card_to_start(…, work.stop_of_the_cards())` 不管有没有下一张卡都先问叫停，每起一张卡多问一次。`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 的 `the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest` 因此红：「叫停之前恰好领了五块 left: 4 right: 5」。
  - 库单测 `a_device_short_of_gpu_memory_on_the_card_lent_is_reported_not_run_like_no_room_now` 经 `print_the_not_run_line_or_refuse` 打出 2 行 `NOT_RUN_NO_GPU_ADAPTER`，门禁会记成库那一段的本次未跑。
- 实现员自定、被攻过零轮的：
  1. 等失败那一笔不销账、到设备丢了记重起那一刻销（另一支「记成设备丢了」实现员判为借与重起之间会多给）；
  2. 丢了的设备两种状态下照读不到这个进程占着的算，直到设备丢了那一刻活着的借据都还了，方向只会少借；条款没写这一格；
  3. 「此刻空闲减这个进程账上借着的」读成「容量（空闲加占着的里属于许出去的）减借着的」；
  4. 叫停标记只在 `run_from_environment` 建一个；
  5. 本次未跑那一行的打印条件取「这一趟有卡报过」，比判决写的「这一轮」宽；
  6. 交给 `after_the_round` 的其余参数与 `cards_used` 抽不出（理由在实现员报告第六节）。
- 没造、要卡的：真卡上等超时或等的时候设备出错；真丢设备。
- 用户已定、报出来不算打中的：两进程同读一份空闲的竞态，一份 `nvidia-smi -q` 报告里约 1.3 毫秒的窗口。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　G2 乙做到没有
1. 把这张卡上的缓冲交回驱动、然后销账的每一条路都先等了吗？有没有一条路（重起、起设备失败、panic 展开、不走卡池的设备）不等就销账？
2. 等失败那一支：有没有一形账给的量大于此刻真能再借的？「一直少借」最坏少到多少，会不会整趟弃卡或报本次未跑？
3. 丢了的设备两种状态：有没有一形多给？少给的那一段会不会永远出不来（活着的借据一直不还）？
4. 「卡上可能还没还」那一份拿干净没有？还有没有别处按旧口径往下压或往上加？

### K2　第五块判决第三节三条做到没有
1. 叫停记住：一轮收尾、整趟收尾、卡线程读到的是不是同一个？叫停文件在任一时点被删，还会不会判红或没记成提早收尾？
2. 头一句 OOM 那一行：该打的时候打了吗？会不会在不该打时打、让门禁多数一条本次未跑？
3. 七条会红用例与抽到 harness 档的两处：改回去是不是都红？

### K3　只防 OOM、不压并发两头（新代码引进来的）
1. 有没有一形账给的量大于此刻真能用的？
2. 除用户已认的之外，有没有一形卡放得下却报本次未跑、判红、整趟弃卡，或因为等设备把并发压下去？

### K4　测试判别力
新加的用例与变异行，能不能拦住这一批每一处被改回去？有没有一形「改回去」照绿？

## 四、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
