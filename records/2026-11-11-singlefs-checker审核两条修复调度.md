# singlefs-checker 审核两条修复调度

课题：关掉 `records/2026-11-11-singlefs-checker严格审核.md` 的 K01、K02。用户原话「修复吧」。

出口：

- K01：池级 checker 的环长判据带上「在飞上限装得进系统配置那 4 字节」一格，这一形归 I-7.13；同表测试多一行罩住它；每条新测试有变异行并证红；I-7.13 正文环长那一格写上这一句。
- K02：`crates/singlefs-checker/invariants/01-块自描述.md` 的 I-1.7 把类型 5（谱系）的登记宽写成 24。

派发上限 2 次（实现员一次、承接一次），截止批次 1。批内冒出的发现只进下一轮清单。

| 批次 | 任务 | 谁做 | 依赖 | 输入 |
| --- | --- | --- | --- | --- |
| 1 | K01 的 checker 判据、同表测试、变异行 | implementation-writer，交补丁 | 无 | 齐 |
| 1 | K02 正文、I-7.13 正文环长那一格 | 主 agent 定点替换 | I-7.13 那一句等 K01 补丁合入后写 | 齐 |

只改测试代码一侧（池级 checker 与验证包），不开代码轮。别的会话在 `crates/singlefs-checker/src/walk.rs`、`crates/mutations.tsv`、`crates/singlefs-checker/invariants/01-块自描述.md` 上有暂存的改动：实现员不碰 `walk.rs`，`mutations.tsv` 只追加，`01-块自描述.md` 只定点替换那一句。

### 第一批进展

- K02：`crates/singlefs-checker/invariants/01-块自描述.md` I-1.7 的谱系登记宽已定点改成 24，回读命中一次。
- K01 写段：实现员交补丁（`~/.local/share/singlefs/work/checker-k01-fix/patch`），任务 13204 打进主工作区，退 0。副本 182M 已删。
- K01 验证段：
  - 任务 13156（不带 `--skip-apply`）撞内存上限 6144M，快照里跑的，主工作区没被写。
  - 任务 13224（带 `--skip-apply`）前五次撞上限（4096M、6144M、9216M 起逐次放大），第 6 次退 1：checker 库新单测现判 1 条绿、复用 184 条；验证包 lib 编不过（9 条 E0308，落在 `crash_amplification.rs`、`crash_state_enumeration.rs`），同表测试与给定状态用例没跑到。
  - 编不过的几份文件不在这一批：管 prune-server 的会话回复是它的实现员在主工作区改到一半（判定库的锁改成每次调用才拿），编过之后它发消息过来。发了、它认了。
- 待做：收到消息后重交带 `--skip-apply` 的合入后验证；绿了交三行新变异的点名复跑（crates-mutation-replay 带 `--item`）与 code-source-discipline 的 mutation-tables 格；补 I-7.13 正文环长那一格「装得进 4 字节」与实现说明里的格数。
- 批内冒出、进下一轮清单的：
  - GPU 判器走读内核 `crates/singlefs-verification/src/crash_judge_checker.wgsl` 环长那一判没有「装得进 4 字节」一格，建表那一处（`crash_judge_common.wgsl`）有；带 gpu 特性的给定状态用例跑出来再判要不要补。那份文件上有别的会话暂存的改动。
  - 合入后验证入口整条占卡：任务 13224 第 5 次尝试持卡 605 秒、平均利用率 14%（task-server 的 card-underused 提醒）。归入口脚本，不在这一批改。
  - 补丁动到 `tests/common_images_for_the_given_states_engine/` 下一份场景文件，合入后验证就挑 89 个验证包测试目标一起编。

### 第二批

用户原话「处理吧」，开第一批记下的三件。派发上限 2 次（实现员一次、承接一次）。

| 批次 | 任务 | 谁做 | 依赖 | 输入 |
| --- | --- | --- | --- | --- |
| 2 | GPU 走读内核 `crash_judge_checker.wgsl` 环长判据补「在飞上限高 32 位为 0」一格，加一行带 gpu 特性的变异 | implementation-writer，主工作区直接改 | 验证要等验证包 lib 编过 | 齐 |
| 2 | 合入后验证挑测试目标过宽、按需领卡的持卡窗口里含编译 | 发给改 `scripts/gate/merge-and-verify.sh` 的那个会话（who-touched 报最近的 Edit/Write 在它那里） | 无 | 齐 |

- GPU 那一件判为要改的依据：走读内核是池级 checker 在 GPU 上的那一份，第 146–148 行的环长判与 CPU 改前同形（只判在飞上限不是 0），这一槽收成可用之后 `journal_ring_geometry_fields_agree` 判 I-8.1；CPU 一侧补了这一格之后两边在这一形上判得不同。读码推的，带 gpu 特性的那条给定状态用例还没跑过。
- 入口脚本两件不归这一轮：发了消息，等它认。

### 第二批进展

- GPU 走读内核：实现员在主工作区改了 `crates/singlefs-verification/src/crash_judge_checker.wgsl` 环长那一判（在飞上限高 32 位不为 0 时这一槽收成带拒收值），`crates/mutations.tsv` 末尾追加一行带 gpu 特性的变异；报告 `~/.local/share/singlefs/work/checker-k01-gpu-kernel/result.md`。没跑过。
- 写段冒出、接在这一步之后做：`crates/singlefs-verification/src/crash_judge_dispatch.rs` 钉走读内核展开大小的那条单测会跟着红（实现员推的多 12 个表达式左右，没量过）；等验证包 lib 编过，主 agent 交那条单测取实测数，再派实现员重钉。
- 入口脚本两件：改那份脚本的会话认了，由它这一批改（挑目标只按 mod 与 `#[path]` 实际走得到的落；带 gpu 特性那一趟先编好再领卡），改完它回消息。
- 管任务服务的会话通知服务与各门禁入口换上了新一批，crates 变异判法单元指纹变了、已有行要重判。

### 验证段进展

- 合入后验证带 `--skip-apply` 走不通：补丁打上之后别的会话把 checker 库那条新单测改了名（去掉开头的 `a_`），补丁倒不掉（任务 13354 退 1）。改成直接点名测试目标与用例。
- 绿的三条：checker 库新单测（任务 13224 那一趟，`test result: ok. 1 passed`）；同表测试整个目标（任务 13510，`test result: ok. 6 passed; 0 failed`）；给定状态目标里点名新用例（任务 13511，`test result: ok. 1 passed; 0 failed`）。
- 走读内核展开大小：任务 13384 量到 `GPU_KERNEL_INLINED_PIN kernel=checker-walk expressions=370463 pinned_expressions=370442 margin=29537 pinned_margin=29558`，主 agent 照实测数重钉 `crates/singlefs-verification/src/crash_judge_dispatch.rs` 两个数，并把 `crates/mutations.tsv` 里拿旧钉值当锚点的那一行改到新值。重钉之后的复跑（任务 13457）还在等卡。
- 变异复跑（任务 13606，三行）：
  - 抓到两行：「checker 环长判据去掉「在飞上限装进 4 字节」一格（库单测）」「core 读者的环长判据「在飞上限装进 4 字节」一格恒成立（core 与 checker 收同一张表那一格）」，分片日志各一行「红了（现判：行新加）」。
  - 没红一行：「checker 环长判据「在飞上限装进 4 字节」一格恒成立（给定状态用例）」，分片日志「没红（退出码 0）」。门禁给这一行起的 cargo 带 `--features gpu --release`，给定状态用例由 GPU 判器判，改坏 CPU checker 的这一格它不红。这一行删掉：CPU 那一格由库单测那一行证红，给定状态用例带 GPU 时由走读内核那一行变异证红（还没跑，等卡）。
  - 合并那一步因没红的那一行在库里读不到而退 1。
- 发给别的会话、它们认了的：
  - 验证包 release 编译每条命令从 4096M 起步连撞三四次：管任务服务的会话定了同包同特性同档归同一类命令键，登记在拆分提案第二百二十四节。
  - 变异分片没领到卡却有进程上了第二台的卡（wrong-card）：同一个会话认了，改成领卡之前给不要卡的格把显卡藏掉。
  - 五张卡被起跑就持卡的实验分片占着、利用率 2% 到 21%：同一个会话说服务一侧已改好、开关等验证包卡池改完再开。
  - 任务 13455 第 6 次测试进程收到 SIGTERM：它核了不是服务发的，来源没查到；结局的原因句写出信号名，登记在拆分提案第二百二十四节。

### 等卡那三件的结局（两件绿，一件已交）

- 任务 13457（重钉后的内核大小单测，带 gpu 特性）：绿。判定行「test crash_judge_dispatch::tests::each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget ... ok」「test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 946 filtered out」；走读内核那一行「GPU_KERNEL_INLINED_PIN kernel=checker-walk expressions=370463 pinned_expressions=370463 margin=29537 pinned_margin=29537 budget=400000」。
- 任务 13512（带 gpu 特性的给定状态用例，核 GPU 走读内核与 CPU 判得一致）：第 1 到 3 次在编译阶段撞内存上限（4096M、6144M、9216M），第 4 次绿。判定行「test journal_ring_whose_in_flight_record_limit_overflows_its_four_byte_field_is_refused_by_core_and_reddens_the_checker_table ... ok」「test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 363 filtered out」；给定状态那一行写 states_judged_by_the_gpu_here=1、states_judged_by_the_cpu_here=0。
- 任务 13903（门禁 crates-mutation-replay 点名「GPU 判器 走读内核环长判据去掉「在飞上限装进 4 字节」一格（给定状态用例带 GPU）」一行）：交了、等结局。
- I-7.13 正文两处已补（环长一格的「装得进 4 字节」与「六格各一条用例」）；「…一格恒成立（给定状态用例）」那一行变异删了：门禁给这一行带 gpu 特性，给定状态由 GPU 判，不经 CPU 那一格，不红。
- 排队期间发给管任务服务的会话、还没见回的三问：崩溃点重放分片领到卡之后又编 4 分 02 秒（任务 13697 第 4 次的日志第 148 到 182 行）；任务 13878 持第二台卡 1、服务读到进程在卡 0；起跑就分卡的任务与按需领卡的任务之间卡放出来先给谁。

### 走读内核那一行变异抓到

- 任务 13903（门禁 crates-mutation-replay 点名 1 行）：退 0。分片那一步（子任务 13905）现判：「✓ GPU 判器 走读内核环长判据去掉「在飞上限装进 4 字节」一格（给定状态用例带 GPU）：journal_ring_whose_in_flight_record_limit_overflows_its_four_byte_field_is_refused_by_core_and_reddens_the_checker_table 红了（现判：行新加）」；合并那一步（子任务 13906）核账：「计数：抓到 1 条、没红 0 条、无效 0 条、锚点失效 0 行（共 1 条）」「基线：没改坏的副本上点名的测试恰好对上一个用例、结果是 ok 的 1 行，基线不绿 0 行」。只判了选中的 1 行，不是整道结论。
- 到此 K01 三处（checker、core 与 checker 收同一张表、GPU 走读内核）的变异行各抓到一次；整表复跑、clippy、build 与整轮门禁归提交时。

### 四样检查交了，命名四格里 data-names 红、不是这一批的

- 交了五条：14406 变异行静态核、14407 命名四格、14408 naming-lint、14409 clippy、14410 build；clippy 与 build 不点名包的被服务拒，改成 `-p singlefs-checker -p singlefs-verification` 重交。
- 14407 结局 command-failed 退 1：vague-names、test-file-names、ordinal-names 三格绿，data-names 红，红行是「.claude/data-name-lint-exclude:53  .claude/kb/checks-owed.md:m1_ 已经一个违规都没有了」与第 54、55、59 行同一句，汇总「0 处违规、排除表 4 处不合规（扫了 1127 份文件、44491 个串）」。这两份文件这一批没碰，不改。
- 发消息问了「里程碑3未完成事项与欠账 收尾」与管命名回扫的会话，是不是它们的、知不知道；等回话。其余四条还没出结局。

- 14408 naming-lint 绿：「✓ 命名纪律通过：查了 1008 个 .rs 文件、296304 个声明的名字」。
- data-names 那四行排除：管命名回扫的会话认领、已删，它交了 14420 复核那一格，结局没出。
- 14409 clippy 退 101，红在这一批没碰的两份文件：`crates/singlefs-verification/src/crash_amplification.rs:23224:11` type_complexity 一处，`crates/singlefs-verification/src/crash_library.rs:3406:9`、`3413:9`、`3414:9` 三处 unused `StageOutcome`；验证包 lib 没编过，这一批改的测试文件这一趟没被 clippy 查到，checker 包只到 Checking 一行、没有报错行。发消息问了 prune-server 会话是不是它的；它改好之后重交 clippy。
