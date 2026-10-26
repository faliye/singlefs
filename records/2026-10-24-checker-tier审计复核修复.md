# checker 档审计复核两项的修复与审计目录搬迁：调度记录

## 课题与出口

课题：`records/2026-10-24-singlefs-checker-tier-审计复核.md` 确认的 T01、T02 两项，以及这份报告与同批两份审计报告本身的问题（坏链接、审计证据放在 `crates/` 下）。C01、C02 归 `records/2026-10-24-checker审计复核修复.md` 那一轮，这一轮不碰。

出口：

| 条 | 验收 |
|---|---|
| T01 对拍字只存 txg 低 32 位 | 用户弹窗定「存完整 u64」：每遍落到的根在对拍字、恢复字（`RECOVERY_WORDS_PER_STATE` 8 → 10）、判定库存的恢复字、GPU 结论字里都带 txg 高 32 位；`landed_root_words_of`、`recovery_passes_differ`、`cross_check_words_agree_in_the_cross_check` 比完整身份。用例：同实例、低位相同高位不同的两份报告，对拍字不同、比较器判不同、`recovery_passes_differ` 为真；`2^32-1` 与 `2^32` 判不同；无根与有根判不同；恢复字存进判定库再读回逐字相同（含高位）；GPU 结论字的高位那一字由 `effective_txg.y` 写出（CPU 与 GPU 对拍用例里有一格高位非 0 的取样点，或写明这一格只能在提交时的快档带 gpu 特性那一趟验）；txg7 与 txg8 的负对照保留 |
| T02 覆盖统计读了缓存不存的逐状态判到位 | `tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs` 的「判成立过」改从整张表的 `count_vector`（逐条不变量判到数大于违例数）取，不读逐状态的 `pool_checker_judged_bits`；用例：同一张表冷判与从库里复用，「判成立过」集合相同；某条不变量在所有状态都违例或不适用的表，修法不把它记成成立过 |
| 报告本身 | 三份审计记录里的本机绝对路径链接改成相对路径；三份审计目录 `crates/*/audits/recovery-verification/` 搬进 `research/prompts/`，全仓引用改新路径（这几份文件从没进过提交，按 `.claude/rules/path-moves.md` 不登记 `.claude/gate.d/path-moves.tsv`：基准里旧前缀下没有文件，登记了反而判红） |

## 依赖表

| 件 | 关的条 | 要动的文件 | 依赖 | 批次 |
|---|---|---|---|---|
| 主 agent | 报告本身：checker-tier、checker 两个审计目录搬家与引用、两份记录与初稿的绝对链接 | `records/` 下四份、`research/prompts/recovery-audit-checker-tier-2026-10-24/`、`research/prompts/recovery-audit-checker-2026-10-24/` | 无 | 1（已做） |
| 主 agent | harness 审计目录搬家与那份记录的 6 处绝对链接 | `crates/singlefs-harness/audits/`、`records/2026-10-24-singlefs-harness-恢复验证审计.md` | 那个目录还在被别的会话写，等它停下 | 2（已做：搬到 `research/prompts/recovery-audit-harness-2026-10-24/`，两份脚本的 `ROOT` 随目录深度改成 `HERE.parents[2]`，日志里抄下的旧路径照留；搬之前那 15 个文件已被别的会话全部暂存，暂存状态原样带过去，已给在跑的会话发消息告知新路径） |
| 实现员 | T01、T02（可能同改 `crates/singlefs-checker-tier/src/crash_given_states.rs`，聚成一簇） | `crates/singlefs-checker-tier/src/crash_judge_conclusions.rs`、`crash_judge_gpu.rs`、`crash_judge_recovery.wgsl`、`verdict_store.rs`、`crash_given_states.rs` 与引用恢复字的 checker 档测试、`crates/mutations.tsv` | 无 | 1 |

| 实现员（第二件） | T01 没改全的两处 4 字摘要：`crates/singlefs-checker-tier/src/crash_amplification.rs` 的 `recovery_words_of` 与 `crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 的 `gpu_recovery_words` 仍截 txg 低 32 位（第一件实现员报告「要你定的」第 2 条） | 那两份源文件、`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`tests/gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core.rs`、`tests/record_checker_judges_absence_by_the_persisted_set.rs`、`crates/mutations.tsv` | 第一件的补丁（已打） | 2 |

派发上限 2 件实现员，截止批次 2。

## 第 1 批交回

- 实现员报告 `~/.local/share/singlefs/work/checker-tier-audit-txg-coverage/result.md`（sha256 `8e670e96…d1ad93`），补丁照 `scripts/gate/apply-writer-patch.py` 打进工作树，只打不验。
- 恢复字是 12 个字（每遍：落没落到根的标记、实例代号、txg 低 32 位、txg 高 32 位、结局、失败点），不是出口表写的 10 个：实例代号与 txg 都从盘上读，坏镜像能取到 `u32::MAX` / `u64::MAX`，没有有根标记就和「没落到根」分不开；主 agent 照收。GPU 结论字 26 → 30，新字接在末尾，池级 checker 那几个内核的字号不挪。
- 打完报 code-source-discipline 退 1（`--check mutation-tables`），这一格只在提交流程里跑，留到验证段与提交时看（写段第 4 步）。
- WGSL 没经 naga 校验，内核编不编得过等提交时快档带 gpu 特性那一趟。

## 第 2 批交回

- 实现员报告 `~/.local/share/singlefs/work/checker-tier-audit-txg-summaries/result.md`（sha256 `ff928e5d…a377d598`），补丁照 `scripts/gate/apply-writer-patch.py` 打进工作树。两处摘要改成同一个类型 `RecoveryRootAndOutcomesOfState`（6 字：看 journal 那一遍落到的根四字、两遍结局码），放在 `crates/singlefs-checker-tier/src/crash_amplification.rs`；`crash_judge_gpu.rs`（挂 `gpu`）因此引 `crash_amplification`（挂 `verdict-store`），`verdict-store` 在默认特性里、关掉整包 `compile_error`，主 agent 照收，不另挪。
- 打完同样报 code-source-discipline 退 1（mutation-tables），与第 1 批同一处待查。
- 主 agent 自己补了一处：`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 测试 `every_verdict_word_index_is_distinct_and_the_old_ones_stay_put` 的字号清单没收第 1 批新加的四个字号（标记与 txg 高 32 位，两遍各两个），`VERDICT_WORDS` 变 30 之后「每个字都有用途、不越界」那一句会红；补进清单，改了过时的注释与断言消息。只改测试代码，不开代码轮。别处字数都走 `VERDICT_WORDS` 常量，全仓 grep 没有按旧字数写死的。

## 什么算写完（逐条）

- 出口表 T01、T02 两行的条款都有实现：T01 对拍字、恢复字、判定库、GPU 结论字、两处 4 字摘要；T02 「判成立过」改读计数向量。没有「下一批再补」的行。
- 每条新行为都有测试（第 1 批 8 条、第 2 批 3 条，都在 checker 档包，交回前没跑），每处改法都有变异行（第 1 批追加 8 行改写 1 行，第 2 批追加 5 行，全是 checker 档的重型行，证红留给提交时照 `.claude/skills/crash-test/commit-stages.md:1` 跑）。
- 剩下只有跑出来才知道的：编译、测试、WGSL 内核、mutation-tables 那一格。进验证段。批内冒出的新发现进下一轮的清单。checker 档是测试代码，不开代码轮三方（`.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」）。

## 不碰（别的会话在改）

`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl`（`records/2026-10-24-checker审计复核修复.md` 依赖表实现员甲、乙）。

## 延后与不做

| 项 | 去向 |
|---|---|
| R01 撕裂模型的三态抽象与逐扇区持久集合是否判器观察等价 | 待证风险，登记 `.claude/kb/checks-owed.md` C898（撕裂三态与逐扇区持久集合没证过判器观察等价），三语三份同插 |
| B01 给定状态字不比内容 | 不做：给定状态构造函数的契约本就不含期望版本表，内容由 oracle 另判，审计的正反对照已证 |
| B02 失败码只编 enum 大类 | 不做：`gpu_failure_point_code` 文档注释写明成员细节不进码，没有调用方要更细 |
| B03 Verus 证明范围 | 不做：范围说明，证明重跑通过 |
| B04 GPU 判表同源预处理、B05 记录核对器复用豁免是宽松判据 | 不做：信任边界，源码已写明，审计没找到穿透的反例 |
| 内存包装排不上时打的「挪第二台」命令带本机绝对路径，照抄在第二台退 127 | 协作工具的欠账，登记 `records/2026-10-23-subagent拆分提案.md` 第一百一十六节（索引表先占号） |
| 审计脚本 `verify_audit.py` 把 `CARGO_TARGET_DIR` 指到仓根 `target` | 不做：冻结证据不改内容，要复跑的照 `.claude/main-agent.md`「一轮怎么开、怎么收」第 11 条另给仓外编译目录 |

## 验证段

- 合入后验证（两份补丁都已打，一条命令带 `--skip-apply`）第一趟：这一趟 ✓ 4 ✗ 1，✗ 的是：`cargo build --all-targets` 撞了内存上限 8G（`run-with-memory-cap` 报 oom-kill，退出码 143，日志 `~/.local/share/singlefs/work/merge-and-verify/2026-10-24-3760894/build.log` 里被杀前没有 rustc 报错）；fmt、clippy（含编码纪律 7 条）过。是上限给小了，不是代码红，照加大的上限重跑 build。
- 看门狗报 checker 档快档那一步（进程 205201）在用核数过少：那一步在 `run-with-memory-cap.sh` 排内存（slice 已占 24 到 31 GiB，这一步要 36 GiB），不是装置串行，`--ack` 接着排。
- 合入后验证第一趟收尾：这一趟 ✓ 4 ✗ 2 本次未跑 1，✗ 的是：`cargo build --all-targets` 撞内存上限 8G；checker 档快档本机内存排不上（包装退 252，一行没编）。本次未跑：测试（补丁只改 checker 档包）。照脚本的出路交第二台重跑，`--memory` 从 8G 提到 24G。
- 合入后验证第二趟（`--peer --memory 24G`，第二台编）：这一趟 ✓ 3 ✗ 0 本次未跑 2，✓ 的是 fmt、clippy（含编码纪律 7 条）、build；本次未跑：测试（补丁只改 checker 档包）、checker 档快档（`--peer` 不交判定库）。照脚本打出的命令在本机原样起 `checker-tier-crash-point-replay.sh --check crash-case-replay,feature-gated-tests`。
- 看门狗报本机 checker 档快档（进程 1754561）在用核数过少：同样是在 `run-with-memory-cap.sh` 排内存（slice 已占 29 到 35 GiB、要 36 GiB），`--ack` 接着排。
- 本机 checker 档快档第二次排内存也满 60 分钟退 252（heavy slice 被别的会话占 34 GiB、总上限 40.1 GiB）。照包装给的出路经 `multi-host-run.sh --peer --memory-cap 36G` 挪第二台跑那两格；第二台没有本机的判定库，两格全部现判、不写回本机库；提交时快档照常在本机按键复用再跑。另一个会话那处 `crash_judge_checker_common.wgsl` 的 `const_assert` 写法（naga 解析不过）已由它改好，不归这一轮。
- 第二台快档那一趟：这一趟 ✓ 0 ✗ 1，✗ 的是：第二台在用 2.6 核（登记 31，近 606 秒）。现查第二台进程：在跑 `scripts/gate/admission.py test-plan`（逐个测试函数算单元键，单线程 python，已 5 分钟），还没到编译与测试；是装置没声明的串行段，不是卡住，`--ack` 接着跑。
- 第二台快档那一趟收尾：这一趟 ✓ 1 ✗ 2，✓ 的是不带特性的 checker 档快档（现判 1750、复用 0、拒算 71，没有红）；✗ 的是 crash-case-replay（`make-filesystem-over-a-used-pool-stream`、`random-histories-and-fault-injection-given-states` 两条退 78：第二台上没照配置清场，命令没起）与 feature-gated-tests（带 gpu 特性那一趟同样因没清场没起）。两样都不是代码红，算本次未跑；本机 5090 上有别的会话的计算进程，本机清场回读过不了，GPU 那一侧留给提交时快档带 gpu 特性那一趟（`.claude/skills/crash-test/commit-stages.md:1`）。
- 轻型三条：形式化 ✓（3 格绿：Verus 69 项 78 verified 0 errors，钉表 360 行）；崩溃（点名 4 项）第一趟编译撞内存上限 8G 被杀（退 250），加到 24G 重跑；变异（点名 14 行：两批追加的 13 行与改写的 E2 J8 一行）在跑。
- 命名检查：`naming-lint.sh` 全仓 3489 处是排除表里的旧账，checker 档包 887 处命中逐行对两份补丁新加的行，落在新加行上的 0 处。
- 变异轻型（点名 14 行加核心部分）：这一趟 ✓ 275 ✗ 0 本次未跑 5，抓到 275、没红 0、无效 0；这一批的 14 行里 9 行抓到（T01、T02 的 CPU 一侧 8 行与改写的 E2 J8 一行），5 行带 gpu 特性的（`crates/mutations.tsv` 第 5494、5495、5506、5507、5508 行：GPU 结论、GPU 恢复内核、GPU 恢复摘要三行）照多机配置清场回读没过（卡上有别的会话的计算进程），基线没起、变异没跑，算本次未跑，留给提交时 checker-tier-crates-mutation-replay（`.claude/skills/crash-test/commit-stages.md:1`）。
- 崩溃轻型（24G，点名 4 项加核心部分）：这一趟 ✓ 0 ✗ 0 本次未跑 全部，入口末句报「30 项红」，逐行数：78 行 ✗ 里 36 行是 `cargo test 退 78、红 0 条`、42 行是「照多机配置清场之后回读没过」，另 29 项报本次未跑（放量入口与双机驱动退 78）；没有一行是测试判红（红条数全是 0）。原因是卡上有别的会话的计算进程，清场回读过不了，用 GPU 的项都没起。

## 验证段小结

- 绿：fmt、clippy（含编码纪律 7 条）、build（第二台 24G）、checker 档快档不带特性那一趟（第二台现判 1750、没有红）、形式化轻型 3 格、变异轻型 275 抓到 0 没红 0 无效（这一批 14 行里 9 行抓到）、命名检查（新加行 0 处命中）。
- 本次未跑（都因 GPU 清场过不了或第二台没清场，不是代码红）：checker 档快档带 gpu 特性那一趟与两条 `red-blocks=judged-by-test` 崩溃枚举用例、变异轻型里 5 行 GPU 变异、崩溃轻型全部项。等卡空出来在本机重跑崩溃轻型与那 5 行变异；没等到的照提交时六道重阶段跑（`.claude/skills/crash-test/commit-stages.md`）。
- 没开代码轮三方：两批只改 checker 档包（测试代码，`.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」）。
- 待查：`apply-writer-patch.py` 打两批时报的 code-source-discipline mutation-tables 格退 1，这一格只在提交流程里跑，提交时整轮门禁判。
- 崩溃轻型补跑（卡空了）：看门狗报第二台 `read-faulted-states-never-hold-the-witness-criterion`（核心部分老项，不是这一批改的）在用 4.6 核：现查第二台只有那条测试二进制一个进程、单线程约 95% CPU，没声明串行段。停它这一轮验证就跑不完、改装置超出这一轮范围，`--ack` 接着跑；跑完照它的用时判要不要登欠账。
- 同一趟：第二台「给定状态表」那一项（进程 3453088）0 核，现查是在第二台 `run-with-memory-cap.sh 24G` 排内存（那边 heavy slice 被别的会话的 E185 装置占着），不是装置串行，`--ack` 接着排。
- 崩溃轻型补跑结果：这一趟 ✓ 8 ✗ 10 本次未跑 22。绿的 8 项含点名的 random-histories-and-fault-injection-given-states、crash-injection-fast-tier、make-filesystem-over-a-used-pool；判红的 10 项（instance-table-sweep、mounts-with-read-faults、first-file-version-after-falling-back-to-a-warm-up-root、one-file-overwritten-across-two-writable-remounts、several-inodes-overwritten-in-turn、large-file-overwritten-across-lower-leaves、unlink-of-two-files-holding-data、truncate-to-zero-on-a-nearly-full-pool、both-tiers:gpu-and-cpu-agree-on-small-inputs、both-tiers:pruning-self-proof）每一项都是 `cargo test 退 78、红 0 条`：跑到中途清场回读又没过（`GPU_QUIESCE step=quiesce result=read-back-failed`），没有一条测试判红；22 项退 78 / 77 本次未跑，含点名的 sigma-of-the-misaligned-reuse。
- 5 行 GPU 变异补跑：第一次清场回读报「还有单元是 active」（清场停下的服务里有一个又起来了，那一刻两台都查不到别的计算进程），照包装出路手动跑配置里的清场命令、回读退 0 后重起；刚起 11 秒，里程碑四会话（E185 第 3 次重跑第一段要在本机卡 0、1 上计时，跑前登记要求别的会话这期间不往这两张卡派活）来消息，主 agent 照它停掉（`proc.py stop`，退 137），回了消息，等它通知跑完再补这 5 行与崩溃轻型里本次未跑的那几项。
- 停那一趟时 `gpu-quiesce-run.sh` 被 KILL、里面的变异门禁成了孤儿接着跑，主 agent 对它再 `proc.py stop` 一次才停干净；工具欠账登 `records/2026-10-23-subagent拆分提案.md` 第一百二十三节。
- GPU 变异补跑（卡让回来之后）：要现跑 32 条（GPU 那几份文件改了，核心部分几行的键也跟着变），外层 8G ÷ 每条 4G 只开 2 个工作进程；开头先编依赖声明了串行 708 秒，`--ack`。看门狗报进程 930699 常驻 6.73 GiB 过阈值 3.6 GiB：现查 RssAnon 1.6G（涨得慢）、RssFile 3.2G、RssShmem 3.1G（显卡驱动映射），scope 的 memory.current 1.8G、上限 4G，不是失控，`--ack`。
- GPU 变异补跑结果：这一趟 ✓ 270 ✗ 1，抓到 270、没红 0、无效 0、内存撞顶 0（共 271 条，现判 32、复用 239）；这一批 5 行 GPU 变异都抓到。✗ 的是第 5069 行（C656 mkfs 崩溃枚举，核心部分老行）基线不绿：点名测试的闭包随 T01 变了要现判，基线在每条 4G 上限里撞顶没判成；设 `GATE_MUTATION_MEMORY_MAX=16G` 只重跑这一行。
- 第 5069 行单重跑：每条 16G、外层 20G，只开得出 1 个工作进程，核心部分 266 格基线串着跑，看门狗报在用 1.7 核；本机可用内存约 20G 加不了外层，不是装置串行，`--ack` 让它跑完。
- 第 5069 行单重跑变慢的原因（量的，那一趟输出开头的计划行）：别的会话改了 `scripts/gate/lib_function_units.py`，变异门禁的判法单元指纹 c63379ff285e… → c7bc2303eee3…，核心部分 266 行一条都复用不了（复用 0、现跑基线 266 批、变异至多 266 条）；这一趟每条 16G 只开 1 个工作进程，第一个小时判 189 格、第二个小时 33 格，还要好几个小时（推的）。不是这一批的改动，接着跑。
- 第 5069 行单重跑停下（已判 296 格，共约 533 格）：里程碑四会话要在本机卡 0、1 上给 E185 第 4 次重跑的控漂移段计时、请别的会话缓 CPU 重活，主 agent 从最里层的门禁起逐层 `proc.py stop`（门禁、内存包装、清场包装），进程链核过 0 个残留，回了消息。已判的格子在判定库里，重起按键复用。
- 变异轻型收尾：第 5069 行接着补（外层 32G、每条 16G、两个工作进程）：这一趟 ✓ 266 ✗ 0，抓到 266、没红 0、无效 0、基线不绿 0（现判 1、复用 265），C656 那一行抓到（停下前那一趟已判，复用）。变异轻型到此全绿：这一批 14 行全部抓到。
- 崩溃轻型收尾（卡空了补跑，24G）：这一趟 ✓ 39 ✗ 0 本次未跑 1，39 项都不红（点名的 random-histories-and-fault-injection-given-states、crash-injection-fast-tier、make-filesystem-over-a-used-pool、sigma-of-the-misaligned-reuse 都绿）；本次未跑的是 both-tiers:pinned-red-examples（崩溃点重放还没有钉表，与改前相同）。
- `read-faulted-states-never-hold-the-witness-criterion` 这一趟跑 1217 秒、看门狗量到同时在算约 1.0 条线程、没声明串行段：不归这一批，登 `.claude/kb/checks-owed.md` C907（读错崩溃注入那条用例单线程跑二十分钟），三语三份同插。

## 验证段收尾

- 轻型三条全绿：形式化 3 格；变异轻型核心部分 266 条加这一批 14 行全部抓到（没红 0、无效 0、基线不绿 0）；崩溃轻型 39 项不红。
- 合入后验证：fmt、clippy、build 绿；checker 档快档不带特性那一趟绿（第二台现判 1750、没有红）。
- 还没验到的：checker 档快档带 gpu 特性那一趟与两条 `red-blocks=judged-by-test` 崩溃枚举用例（第二台那一趟没清场退 78，本机内存排不上），交由提交时 `checker-tier-crash-point-replay` 快档；code-source-discipline 的 mutation-tables 格（只在提交流程里跑）。
- 延后项去向：R01 → C898；协作工具两条 → `records/2026-10-23-subagent拆分提案.md` 一百一十六、一百二十三；单线程用例 → C907。

## 第 3 批：延后项（用户 2026-10-25 弹窗选四项、允许多开子 agent）

课题与出口：

| 条 | 出口 |
|---|---|
| 拆分提案一百一十六 | `scripts/hosts/run-with-memory-cap.sh` 排不上时打的「挪第二台」命令里，以本机仓根起头的参数改写成树内相对路径（仓根本身写 `.`）；`scripts/hosts/multi-host-run.sh --peer` 见命令里有本机仓根绝对路径就拒、退 2、出路写改成相对路径；各一格自证与弄坏开关 |
| 拆分提案一百二十三 | `scripts/crash/gpu-quiesce-run.sh` 收到 TERM / INT 把信号转给它起的命令（进程组），限时等它退出再收尾；被 KILL 时命令也跟着停（跑在自己的进程组或 scope 里）；自证一格：停包装之后假命令不在；配弄坏开关 |
| C907 | `crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs` 里互相独立的单位按区间切片并行，线程数取登记的每台默认重活线程数，合并确定；不能并行的写明理由 |
| C898 前置 | 只回答：登记的崩溃流（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行与崩溃放量分项）里有没有同一段上重叠、都走撕裂三态的两次原地覆写；没有就写明 R01 只是枚举器层的字节差异 |
| mutation-tables 格的红 | 主 agent 带 `SINGLEFS_HEAVY_TESTS=user-request` 单跑那一格（量的）：红在 `crates/mutations.tsv` 第 2891、4272、4300、4500、4589 行锚点失效，第 4321/4590、4831/4832/4842 等五组重复行，第 4594 行点名 `#[ignore]` 测试没带 `--include-ignored`，以及 `scripts/mutation/tables/` 那一段；这一批的 14 行（T01、T02、E2 J8）一行都不在里面，不派调查员、不改，归各行的会话 |

依赖表：

| 件 | 关的条 | 要动的文件 | 依赖 | 批次 |
|---|---|---|---|---|
| 工具员甲 | 一百一十六 | `scripts/hosts/run-with-memory-cap.sh`、`scripts/hosts/multi-host-run.sh` | 无 | 3 |
| 工具员乙 | 一百二十三 | `scripts/crash/gpu-quiesce-run.sh` | 无 | 3 |
| 实现员 | C907 | 上面那一份测试、`crates/mutations.tsv`（只追加） | 无 | 3 |
| 调查员 | C898 前置 | 只读（草稿副本里可加探针） | 无 | 3 |

派发上限 4 件，截止批次 3；四件互不撞文件，同一条消息并行派。只改测试代码与脚本，不开代码轮三方。

## 第 3 批交回

- 实现员（C907）报告 `~/.local/share/singlefs/work/impl-read-fault-injection-parallel/result.md`（sha256 `42b0dd58…6d0670`），补丁照 `scripts/gate/apply-writer-patch.py` 打进工作树（只打不验，打完照旧报 code-source-discipline 退 1，红在别的会话的行）。两层按区间切片并行，线程数取 `CrashInjectionWorkerThreads::from_the_environment()`，切片复用 `seed_slices`；新快档对拍用例 `read_faulted_states_judged_on_several_worker_threads_and_on_one_report_the_same_states_and_counts`；变异追加 2 行（「C907 」起头）。
- 主 agent 改 `.claude/gate.d/stage-inputs.tsv` 第 71 行注释里「单线程」那一句（`replace-once.py`）；不加 `threads=`：那一项跑完打的 `READ_FAULTED_STATES_UNITS_FINISHED` 没有 `resumed_slices=` / `freshly_run_slices=`，`scripts/gate/admission.py` 按 `threads=` 判会红。
- 延后：走 CPU 判器时第一次崩溃那张表仍在 `judge_the_given_states_on_the_cpu` 里整张串行判，拆它要改表的内容与块键（库里结论重判）或改 `src/`，是设计判断，C907 那一行照旧开着（不再改 `.claude/kb/checks-owed.md`：它在里程碑四两轮未判的开工快照里）；GPU 一路多线程同时交表的显存没量过，同记在 C907 名下。
- 工具员甲（一百一十六）报告 `~/.local/share/singlefs/work/tooling-peer-move-relative-paths/result.md`（sha256 `27de69aa…bfffe80d`），直接改主工作区两份脚本：`run-with-memory-cap.sh --selftest` 133 项绿、`host-lookup` 子集 13 项；`multi-host-run.sh --selftest` 45 格绿；三个弄坏开关（`RUN_WITH_MEMORY_CAP_BREAK=hostlookupabsolutepath`、`MULTI_HOST_RUN_BREAK=peer-absolute-path-accepted`、`MULTI_HOST_RUN_BREAK=local-absolute-path-refused`）各红在点名的那一格（报告里的数，主 agent 没复跑）。拆分提案一百一十六的「已改」一行由主 agent 照报告原文追加（工具员写不了 `records/`）。没罩到的写法：`--root=<仓根>` 这类带前缀的词、`bash -c` 脚本串里的仓根路径；break-switch-headers 等门禁格没核，归提交时整轮门禁。
- 调查员（C898 前置）报告 `~/.local/share/singlefs/work/investigate-overlapping-in-place-overwrites/result.md`（sha256 `d19337b6…87e15c12`）：合表 36 项里 33 项录成 45 条流，同段、同盘、区间重叠、两次都可撕裂的原地覆写对每条都是 0（不分种类的同段重叠上界也是 0）；另 3 项走给定状态表、不按段做三态展开（读代码推的）。主 agent 现查它的产物：`run6.txt` 45 行 FLOW 都是 `same_segment_overlapping_tearable_pairs=0`，`POSITIVE_CONTROLS ok`，`run5.txt` 与 `run6.txt` 的 FLOW 行去掉 run6 新加的一条后逐行相同；6 趟输出里 `SAME_SEGMENT_PAIR` 都是 0。结论：今天登记的流里 R01 没有对象，只是枚举器层的字节差异；C898 照旧开着，管以后新登记的流（不改 `.claude/kb/checks-owed.md`：在里程碑四两轮未判的开工快照里）。
- 看门狗检出（只记）：工具员乙前台命令 timeout 写了 595000 毫秒，超过 `.claude/agent-common.md`「长活交回主 agent」的 240000 毫秒；它的 `gpu-quiesce-run.sh --selftest` 在本机排内存（要 322 MiB），正常排队。
- 工具员乙（一百二十三；当初占的号一百一十八被别的会话的「重型闸认本仓那一份…」先用了，这一节被挪到一百二十三，本记录里的引用已改）报告 `~/.local/share/singlefs/work/tooling-quiesce-forward-stop/result.md`（sha256 `7786fdb0…2957c0d`），只改 `scripts/crash/gpu-quiesce-run.sh`（改后 sha256 `0094a816…88c81`）：命令起在自己的进程组，收到 INT / TERM / HUP 转给整组、限时 8 秒后补 KILL；另起守卫进程，包装被 KILL 时守卫收掉命令；`--selftest` 26 格绿，四个新开关（no-forward-signal、no-kill-after-limit、no-orphan-guard、leave-guardian-running）各红在点名的格（报告里的数）。「已改」一行由主 agent 照报告原文追加进一百二十三节。已知缺口照它文件头：起命令到守卫起来之间的几毫秒、setsid 出去的子孙与内存包装的 scope 进程组信号够不到（scope 那一支靠内存包装自己 stop）。
- 第 3 批验证段：主 agent 在主工作区复跑三份改过的脚本的自证（各经 `run-with-memory-cap.sh 4G`）：`gpu-quiesce-run.sh --selftest` rc=0（开关轮各红在点名的格，含 leave-guardian-running → guardian-stopped）、`run-with-memory-cap.sh --selftest` rc=0（133 项）、`multi-host-run.sh --selftest` rc=0（45 格），与两位工具员报的数相同。接着起崩溃轻型（点名 read-faulted-states-never-hold-the-witness-criterion，32G）与变异轻型（点名 C907 两行）。
- 两条轻型都在本机排内存（slice 已占 36 GiB、各要 8 GiB），`--ack` 接着排；看门狗转抄的「挪第二台」出路里根参数已是 `.`（工具员甲的改法在真场景里起效）。
- 崩溃轻型（32G）排满 60 分钟退 252（slice 被另一会话 24G 与本会话变异 8G 占着）；改 24G 重排。变异轻型（C907 两行）已排上在跑。
- 变异轻型（C907 两行）驱动进程 1047426 已跑 2 小时 15 分、已判 571 / 757 格：核心部分判法单元又变了、整张重判（不是卡住），`--ack` 接着跑；崩溃轻型在放量分项那一段。
- 里程碑四预开发会话来问本机那一趟崩溃轻型（进程 1032576）：是本会话的，放量分项 25 个已跑 22 个，回它约 20 到 40 分钟（推的）；跑完之后这一批不再起带 GPU 的活，空当给它；告知变异轻型那一趟（进程 1047426）判到第 5069 行（带 gpu 特性）时会用卡，它清场回读因此不过就说一声、本会话停那一趟让它先跑。
- 崩溃轻型（C907 那一趟）中途：这一趟 ✓ 46 ✗ 2（pipeline.log 的判定行），✗ 的是放量分项 several-inodes-overwritten-in-turn：双机导入那一趟退 101，用例 panic「观察者 writable_mount_and_continuation 在点 acquire_instance_and_warm_up.system_configuration_slot.1 块 0 的录入段那一块不在库里」（`crates/singlefs-checker-tier/tests/common_crash_points/mod.rs` 的 `.expect("按环境跑得完")`；日志 `~/.local/share/singlefs/crash-amplification/runs/crash-amplification-…-2531097-two-hosts/import.log`），不是判出违例（red=0）。现查：C907 只改读错崩溃注入那一份测试，碰不到这一项；上一趟崩溃轻型里这一项绿；池级 checker 两份源码的修改时刻都早于这一项起跑，两台日志的 key_now 相同（329cd43c…），「跑中途源码被改」的解释不成立。原因没查清：等整趟跑完单独重跑这一项，复现就派调查员二分。
- 崩溃轻型（C907 那一趟）收尾：这一趟 ✓ 37 ✗ 1 本次未跑 1；点名的 read-faulted-states-never-hold-the-witness-criterion 绿（快档 4 条红 0），用时 415 秒（改前 1217 秒，并行起效）；✗ 的是 several-inodes-overwritten-in-turn（上一条记的那一句，原因没查清）；本次未跑 both-tiers:pinned-red-examples（钉表还没有）。答应过里程碑四预开发会话这一趟完后让卡，不当场重跑那一项；它接着跑的崩溃轻型全项会跑到这一项，已请它把结果告诉本会话，复现就派调查员二分，不复现照「单次观测不算数」记不稳定。
- 变异轻型（C907 两行加核心部分）收尾：这一趟 ✓ 266 ✗ 0 本次未跑 2，抓到 266、没红 0、无效 0（共 268 条，现判 261、复用 7；现判的 257 条是点名测试的闭包变了）；C907 两行都抓到。基线不绿 2 行都是环境：第 2944 行（点名 GPU 测试，清场回读没过：卡让给里程碑四预开发会话在用）、第 5070 行（C656，峰值表记撞过顶、每条要 12G，比外层 8G 大，一行没跑）；都是核心部分老行。等里程碑四预开发会话用完卡，外层给够内存单独重跑这两行。
- 里程碑四预开发会话的崩溃轻型带 `--local-only`、走单机路，照不到双机导入那条路径，它只回报那一项在单机路上的结果。主 agent 派调查员（只读：读代码与 `crash-amplification-…-2531097-two-hosts` 那几份日志）回答双机导入那一趟何时报「录入段那一块不在库里」、为什么上一趟绿这一趟红，并写出只跑这一分项的复现命令交主 agent；双机复现等里程碑四预开发会话用完卡由主 agent 跑。
- 调查员（several-inodes 那一处红）报告 `~/.local/share/singlefs/work/investigate-import-missing-observer-block/result.md`（sha256 `5a01347d…1f32f01bd2c`）：这一项录入段与本机判段用旧二进制，导入那一趟前主工作区打进了里程碑四的七·3 骨架补丁（`m4-gpugc-skeleton-writer/patch/crates.patch`，sha256 `c3cf9ea5…`）重编，补丁改到观察者 writable_mount_and_continuation 闭包里的单元，观察者版本变了，导入那一趟按新版本找不到录入块。主 agent 现查：import.log 有 3 行 `Compiling singlefs-`、2m 49s，share-0.log 0 行、0.54s；补丁点名的 5 份文件都在 share-0 与 import 之间改过、`git apply --check --reverse` 退 0；同一窗口另有 5 份测试文件（`tests/` 下）也被别的会话改过，调查员写成「正好是补丁那 5 个」不准，测试文件不进观察者闭包，不推翻结论。结论：不是代码缺陷，是跑的中途树被别的会话改了，与这一批无关；冻住的树上重跑这一项预言绿（推的），卡空出来时随下一趟崩溃轻型核。顺带的工具毛病（版本闸在分项驱动交来的判段那条路上跳过、报成通用红）登拆分提案一百二十八。
