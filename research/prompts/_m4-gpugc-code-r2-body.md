# m4-gpugc-code-r2：里程碑四七·3 GPU 垃圾回收，代码轮第二轮正文（2026-11-01）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

## 一、题面

代码轮第一轮（判决 `research/prompts/m4-gpugc-code-r1-main-verification.md`）打中的第三节第 1–8、10 条，由第 54 批（`~/.local/share/singlefs/work/m4-gpugc-code-r1-fix-writer/patch/`）与第 55 批（`~/.local/share/singlefs/work/m4-gpugc-alarm-every-wake-writer/patch/`）两份补丁改完，都已打进主工作区；第 9 条与第五节用户定的几处由第 56 批写回 `.claude/kb/decisions/26-后台整理与放置回收.md` D26（后台整理与放置回收） 已定项 13、`.claude/kb/mount-parameters.md`、`.claude/kb/checks-owed.md` C921（只供测试的开关在发布构建里够得着）。这一轮照 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 2 步攻这两批合入的产品代码，分工照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」的第二轮：第一轮的修补本身、产生结论的方法、从未被看过的地方；其中一条是辩方腿，复核第一轮判决够不够得着、是不是同样打中所有替代写法。不许重复第一轮已攻过的角度（第一轮的六格 T1–T6 见第一轮正文 `research/prompts/_m4-gpugc-code-r1-body.md` 第三节）：第一轮攻过、这两批没改的那几处不再攻。

改动全文：`research/prompts/_m4-gpugc-code-r2-diff.md`（两份补丁里产品代码的块按打进主工作区的次序原样抄录，5848 行、15 份文件块；测试、变异表的块不在里面，整份补丁在各自的目录里）。第 55 批以第 54 批为底叠在同几处，两批改同一份文件的，以冻结副本里的现状为准。

## 二、实现今天的样子（主 agent 的观测，2026-11-01）

- 冻结副本 `~/.local/share/singlefs/work/m4-gpugc-code-r2/repo/`（此刻主工作区，含这两批补丁与别的会话这期间合入、没提交的改动；去掉了 `.git`、编译目录、私有配置 `multi-host.env` 与 `.env`）。被判的 9 份产品代码文件、`crates/mutations.tsv` 与 4 份 kb 的 sha256 在 `research/prompts/m4-gpugc-code-r2-snapshot/sha256sums.txt`，冻结副本逐份对上。
- 被判的产品代码：`crates/singlefs-background-gpu/src/` 的 `adapter_inventory.rs`、`background_gpu.rs`、`background_gpu_compute.rs`、`gpus_of_this_host.rs`、`memory_limit_and_chunks.rs`、`region_record_kernels.rs`、`waiting_for_the_gpu.rs`；`crates/singlefs-core/src/` 的 `background_compute.rs`、`mounted_session.rs`。`mounted_session.rs` 里别的会话的块（第 5209–5320 行一带）当上下文读、不攻；腿只攻 `_m4-gpugc-code-r2-diff.md` 里有的块。
- 冻结副本里的落点（第一轮判决第三节的条号在括注里）：
  - 超时按「一段的一次调用」累计（第 1 条；用户 2026-10-31 定「一段一次调用」）：`waiting_for_the_gpu.rs` 第 120 行 `RESUBMISSIONS_AFTER_A_TIMEOUT = 3`，第 185–231 行在同一张卡上交、等、重交，计数记在 `TimeoutsOfTheBatchOnOneGpu`（第 197、206、210 行），交活之前先等那一等第 224–231 行也记进去；出错成员 `background_compute.rs` 第 221 行 `GpuUnusableAfterResubmissions`。
  - 卡清单（第 2 条）：`adapter_inventory.rs`（没点名的卡缺标识不拒、切换时现查清单）。
  - 停着表现推（第 3 条）：`background_compute.rs` 第 41 行 `enum BackgroundTask`、第 49 行 `EVERY_MEMBER`、第 68–72 行编译期核位置一一对应、第 1233 行 `tasks_with_a_gpu_path_stopped_by` 按 `has_a_gpu_path` 现挑。
  - 切不出这一批不分给它（第 4 条）与空闲显存不足（第 5 条、用户 2026-10-31 定「这一批把它那份活交给别的卡」）：`background_compute.rs` 第 230 行 `EveryParticipatingGpuLeftOutUnderItsMemoryLimit`、第 237 行 `EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`、第 280–291 行两种「不用」的说明；`background_gpu.rs` 第 1830 行建缓冲显存不足时把这张卡这一段的显存那一项压到建不成那一段的一半重切。
  - 进程里泄漏的账（第 6 条）：`gpus_of_this_host.rs` 第 656 行 `GpuMemoryLeakedInThisProcess`、第 74 行开卡时扣掉此前泄漏的、第 831 行 `gpus_dropped_leaking_those_still_running`；由调用方持有、开卡时传进来，产品里持有它的挂载工具今天没有调用方。
  - 切到一张都用不上（第 7 条）：`mounted_session.rs` 第 3170 行 `compaction_stays_stopped_pausing_the_live_intent`，第 2460 行（后台入口）与第 3282 行（`run_compaction_on`）在任何写之前调它。
  - 可观测（第 8 条）：`background_compute.rs` 第 412 行 `take_the_submissions_of_the_batch`、第 574 行 `GpuWorkOfTheBatch`（记每段交了几片、重交几次、用了哪几张卡）；卸载泄漏交到 `BackgroundGpuOverTheCards::gpus_leaked_when_dropped`（`background_gpu.rs` 文件头第 36 行）。
  - 第 55 批的报警：`background_compute.rs` 第 1211 行 `#[must_use = …]` 的 `AlarmOfABackgroundTaskStoppedAfterAComputeError`（第 1220–1221 行 `batches_compensated_on_the_cpu`），会话里 `mounted_session.rs` 第 223 行 `CompactionRun`、第 242 行报警字段、第 277–323 行 `BackgroundCompactionCall` 与它取报警的方法、第 2374 行与第 2398 行后台入口 `run_compaction_in_the_background`。
  - 只供测试的开关：`background_compute.rs` 第 1348 行 `judge_the_gc_cpu_compensation_as_not_above_the_threshold_for_tests`（C921 照已有的延后，只补登记）。
- 第 54、55 批实现员替条款做的选择，报告原文（腿当被攻对象，逐项判它与 D26（后台整理与放置回收） 已定项 13 今天的条款对不对得上）：第 54 批 `~/.local/share/singlefs/work/m4-gpugc-code-r1-fix-writer/result.md`「我替条款做了的实现选择」一节七项（主 agent 照收第 2、4、5、6、7 项；第 1 项「一段一次调用」与第 3 项空闲显存，用户 2026-10-31 另定，见 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 629 行）；第 55 批 `~/.local/share/singlefs/work/m4-gpugc-alarm-every-wake-writer/result.md`「我替条款做了的选择」一节七项（没判，并进这一轮）。第 55 批报告「没做什么」一节自报：外壳乙段、丙段空闲显存那一臂没有用例也没有变异行；`bytes_of_the_last_chunk_not_created` 没写变异行。
- 验证段的数（量的，`records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 655 行起各行）：合入后验证任务 343 测试全绿（333 个测试二进制、3194 条、慢档 77 条，三个包），fmt、clippy 的红都在别的会话的文件；`cargo build --all-targets` 任务 385 退 0、0 条 warning；形式化轻型任务 554 三格全绿（判定没入库，拒算是门禁那一组的事）；checker 档快档任务 1713 crash-case-replay 绿，feature-gated-tests 的红是先前就有的、不是这两批；变异轻型任务 2170「计数：抓到 330 条、没红 0 条、无效 0 条、锚点失效 0 行（共 370 条）」，这两批的 68 项里 66 项抓到，「七·3 装不下自适应分片：丙段外壳不按此刻剩下的显存切记录表」「……开卡时不判显存上限装不装得下最小的一片」两行因卡被占、清场没过而基线没判成。真卡用例与要真卡的变异行这两批还没跑。
- 压着的条款：`.claude/kb/decisions/26-后台整理与放置回收.md` D26（后台整理与放置回收） 已定项 13 八条（第 245–252 行，第 56 批写回之后的现状）与它的射程段；`.claude/kb/milestone/04-fourth-txn.md` 七·3 那一行（第 149 行）的验收；`.claude/kb/mount-parameters.md` 挂载参数表；`.claude/kb/checks-owed.md` C921（只供测试的开关在发布构建里够得着）那一行；`.claude/rules/fs-design.md`「五条硬要求」。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　第一轮的修补本身
第一轮判决第三节第 1–8、10 条逐条：两批的改法做的是不是那一条验收说的那件事，有没有改出新的错。重点：
- 第 1 条：同一张卡、一段一次调用里的超时跨片累计到第 4 次才报；交活前先等那一等算进去之后，等前一次超时提交那一等本身又超时时计一次还是两次；三段里一段报卡用不了之后，同一批另外两段还交不交给这张卡。
- 第 4、5 条：一张卡这一批因显存上限切不出、另一张因空闲显存建不成时，转给别的卡的活怎么分（按挂载时量的代价，还是平摊）；第 1830 行压一半重切的循环一定收敛吗（最小一片之下停）；压低只管这一次调用，下一段、下一批照设置重新起，同一批里第二段会不会再撞一次同样的建不成。
- 第 6 条：泄漏的账在同一进程第二次挂载时扣掉，扣掉之后余量为负或为零时照哪一支走（切不出 / 不用 / 拒挂）；`GpuMemoryLeakedInThisProcess` 没有产品调用方时，这一条的验收在产品路径上成不成立。
- 第 7 条：两处入口（第 2460、3282 行）都在任何写之前判；`run_compaction_on` 被可写挂载续做那一趟调到时，停着的判法与后台入口是否同一份。
- 第 8、10 条：记账字段钉绝对值的用例与补的取样点（补跑途中读数降到不超过、最满的是第二块盘、造非空的删除队列）是否红在点名的断言上。

### T2　第 54、55 批替条款做的选择
两份报告「我替条款做的选择」共十四项，逐项对 D26（后台整理与放置回收） 已定项 13 第 56 批写回之后的正文（第 245–252 行）：条款写了的，代码是不是照写；条款没写的，这一项是不是只是读法、还是改了行为、该交用户。重点：第 55 批第 1 项「停了多久数叫起次数」（出错那一趟就报第 1 次）与条款第 4 条「带原来那一次的错与停了多久」；第 2 项入口在任何读写之前就拒、排空待发没发成的那一次交回 `Err`、不带报警、不计叫起次数，与条款「每次被叫起都报」；第 3 项可写挂载续做那一趟不算叫起；第 4 项 `#[must_use]` 只到「整份结果丢掉就警告」，接住之后不读报警字段编译器不管；第 6 项一张因空闲显存不用、剩下的又因显存上限不用而一张不剩时报哪个成员。

### T3　产生结论的方法（辩方腿主攻）
复核第一轮判决：第二节各格判「打中」的，打中的是不是这两批改掉的那一种写法，换一种同样合条款的写法会不会同样被打中（例：第 1 条「超时按片计」打中，那么按「一段一次调用」计是不是同样被它的反例打中）；判「没打中」的，第一轮只抽了一次样的那几处（第一轮判决第四节点名：`region_scan.rs`、`checksum.rs` 两处快写法，真卡与 WGSL 只读码推）这一轮还是不是只有一份样本；这两批的测试与变异行判的是不是那一条验收（替身罩得到、真卡才罩得到的各是哪几条）。

### T4　从未被看过的地方
第一轮两条腿都没碰、这两批又新加的：报警叫起计数在整理入口拆成「判起跑」「跑」两段之后，哪一处加一、哪一处读，两段之间被别的叫起插进来会不会重计或漏计；`#[must_use]` 加到 `CompactionRun`、`BackgroundCompactionCall`、`BackgroundCompactionAfterThePendingChanges` 之后，产品里每个调用点是不是都读了报警（第 55 批报告说当场抓到四处旧用例）；`GpuWorkOfTheBatch` 不再 `Copy` 之后，记账在一批放弃时是交回还是丢掉；`adapter_inventory.rs` 切换时现查清单的枚举等超过开卡时等的上限，当一张都不在，与条款第 4 条「GPU 不在」拒切换是不是同一件事。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | T1–T4，T3 主攻 | 替两批的写法与第一轮判决的判定辩护：逐条复核第一轮判决第三节第 1–8、10 条，判那一条的打中够不够得着、改法是不是唯一合条款的写法、同一个反例是不是同样打中替代写法；T2 十四项逐项判「只是读法 / 改了行为该交用户 / 与条款相左」，每条写文件:行号与 kb 行号 |
| 云端攻方（Opus） | T1–T4，T1、T4 主攻 | 在冻结副本的拷贝上造反例：T1 用替身造一组超时时序让一段一次调用里第 4 次超时没报或第 3 次就报、造一张卡切不出一张建不成而活丢失或重做的批、造一个压一半重切不收敛或同一批第二段重撞的输入、造泄漏扣完余量为零的第二次挂载；T2 造一个让第 55 批第 1–4 项的读法与条款字面结果不同的叫起序列；T4 造一个让叫起计数重计或漏计的入口序列、找一个丢掉报警编译不警告的产品调用点 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号；引补丁块写 `_m4-gpugc-code-r2-diff.md` 那一节的补丁目录名，不写那份文件的行号。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的数，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit --cwd <自己草稿目录里的副本> -- <命令>` 的整条，写进报告「要主 agent 跑的命令」一节，主 agent 交、跑完新派同立场的腿读输出；量法拷冻结副本，不在冻结副本里改。读码推出来、没量过的写「推的，没量过」。
- 不起 GPU：真卡用例（特性 `tests-on-a-real-gpu`）不写进命令；不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不写进命令），不跑 checker 档测试。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
