# E142 重跑登记（第 26 次）：取号写两轮追平、挂钟挪出 exact 产物、守门与旧值扫法按第 25 次的两处输法改正之后，装置与 `crates/` 在主点上对不对得上；产物一换，页上关系为「支撑」的 21 行还写不写得「不受影响」

<!-- doc-lint:not-numbers N26R N26 P8G G1B G11 G21 G19 G12 T0 -->

写于 2026-11-10，这一次的任何产物之前。判据、门槛、作废与停机条款在这里写死；跑出数之后要改，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」三步走，不在这里回改。

**重跑登记，文件名取 `r26`，不占号。** 上一次是第 25 次：登记 `research/prompts/e142-r25-prereg.md`（已按每次提交归档，不在工作区；取法 `git show ceac9b0fc:research/prompts/e142-r25-prereg.md`，1366 行，sha256 `82e6b5ffc22ceca64529dfd8e853a25efefaf43d94d2fea3aea8b0899313fb3b`），问题单 `research/prompts/e142-r25-questions.md`（同一提交，9 行）。两份与第 25 次的产物冻结不动。**这一次不是差分登记**：装置要改（取号写两轮、挂钟挪出），臂是三个（N26R 参照、N26 真实基线，见第五节），守门脚本和旧值扫法要改（C774 点名的两格）；第二节新抄被测条款；别的格写「同第 25 次登记同名格」时，指上面那份第 25 次登记里同名的那一格（它再往前指的，照它的指法，取法同上，提交 `c84047263` 里有第 21–24 次的登记）。编号接第 25 次：读法 R92 起，格 Q142.146 起，变异 M246 起（变异表现有 M245 为止，`cut -f1 scripts/mutation/tables/e142_new_pool_file_creation_dry_run.tsv` 量的）；失败 / 作废 / 停机条款用 F26*、V26*、S26-*。

**变了哪些输入（一句话一项）**：

| 输入 | 改了什么 | 谁改 |
|---|---|---|
| 装置 `scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs` | 今天 11066 行、sha256 `f1fb548daae1378be29e87cc09ee996b157d4bb743162a564904400077a5317a`（git blob `82d3c789fd78931bcb80a090a52854cdbd5fed43`，与 `HEAD` 相同，`git status` 无改动）。**它不是第 25 次跑的那一份**：第 25 次之后装置源码已有 132 行非注释改动（第三节 3.2 列了：层 0 主臂整轮不再被环境变量挡住、feature bits 的读者判定与 `feature_bits` 行少一个字段、判决字段不再写 `not_run`）。这一次要改两处：取号写两轮（A①–A⑧），挂钟挪出（B①–B③）；改法在第五节 5.1 | 这一次的执行员 |
| 变异表 `scripts/mutation/tables/e142_new_pool_file_creation_dry_run.tsv` | 今天 221 行、sha256 `64caabe99c1f37d69462e2e9952292e0d23723508d4010610ea0967692d1c679`。这一次要重锚不再恰好命中一次的行、加 M246–M251（第九节） | 这一次的执行员 |
| `crates/` | 不改。比对侧导出 bin `crates/singlefs-implementation-experiments/src/bin/e142_new_pool_file_creation_write_dump.rs`（`driver_e142` 与 `driver_e142_p8g_crates_dump` 跑它）只跑、不读、不改；取号写两轮已在 `crates/` 里（`ACQUISITION_WRITE_ROUNDS`，用户 2026-10-19 定） | 别的线 |
| 被测条款 | `layout/01` 一·a1 行、整条流一句、八「第一次可写挂载取号」行、D22 已定项 16 的定案句；全文在第二节 | 已写进 kb |
| `scripts/experiment-apparatus/replay.sh` 第 2662、2663 行 | 两份新产物判完之后改指（第二段末步）；`strip_timing`（第 2824 行）**不加字段** | 这一次的执行员 |
| 新草稿脚本 | `anchors_r26.py`（第七节 7.2 的独立算术，设计时已跑）、`residual_r26.sh`（旧值扫法，设计时已在 `HEAD` 装置与六份合成输入上跑过）；全文与 sha256 在第十三节 13.3。**守门脚本 `r26_guard.py` 由执行员写**（第十三节 13.4 给出它的底本与必须的改法，规则表在第六节 Q142.151 里写死），写完在第十二节修订里记它的 sha256 | 这份登记；执行员 |
| 不变 | 独立 bin `scripts/experiment-apparatus/e142_region_diff_independent.rs`（第 25 次登记记 766 行，今天 809 行，sha256 `2f41d6e588d8a2fd06c141db8b99024f89d0ed36bb1ee3c6456b20d283b49fb7`）与它的变异表；`label6_anchor.py`、`p8g_region_compare.py`（第 22 次登记第十三节全文）；准入键 `E142`（`.claude/gate.d/stage-inputs.tsv` 第 46 行，输入是两个装置的 `main` 与驱动闭包；**这份登记不是准入输入**，`grep -c 'e142-r2' .claude/gate.d/stage-inputs.tsv` 为 0） | — |

**射程（写死）**：这一次要答的是「取号写两轮追平后装置还跟 `crates/` 字节相同吗」「产物一换，页上 21 行能不能写不受影响」。它**不**重判第 25 次已判的 mkfs 行与退役树两项（装置里的那两处改动原样在 N26R 里，守门把它们当零差异组），也不改任何门禁怎么比 mkfs 行。

**装置写在哪（写死）**：独立手写模型 `scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs`，不与 `crates/` 共用代码，**不是入库装置**。改法照第二节条款原文写、**不照 `crates/` 写**，装置里不引 `crates/` 的任何常量，执行员改装置时不读 `crates/singlefs-core/src/transaction.rs`、`make_filesystem.rs`、`admission.rs`、`crates/singlefs-format/src/lib.rs`（第三节 3.1 是设计员读的）。**装置不走崩溃判段入口**（`grep -n 'run_from_environment\|GivenStatesEngine\|judge_not_run_in_this_binary\|crash_state_enumeration' <装置>` 零命中，第十三节命令二）：起它的 `cargo run` / `cargo test` 不带 `--features gpu`，也不写 `task-server-gpu-arguments` 头；判据里的 `NOT_RUN` 一格照写（第六节 Q142.162），命中 0 是预期。

## 一、问题

英文名：new_pool_file_creation_dry_run（照抄第 25 次登记第一节；源文件、变异表、bin 名都沿用它）

**问题单（一份，不是为岔路建的实验，一行一个要回答的判断；起因取自 `.claude/kb/checks-owed.md` C1001 ①（第二节抄了整行）与 `/home/fy5090/.local/share/singlefs/work/older-debts-fact-table/result.md` 第 24 行 E142 那一行（第二节抄了整行）；主 agent 批次 `records/2026-11-10-门禁与实验欠账收尾调度.md#6`）**：

主 agent 给的问法（原话）：E142 第 26 次重跑登记怎么写。要点：装置没跟上取号写两轮；exact 产物里带挂钟（`e142_new_pool_file_creation_dry_run.rs:7862` 打 `name=layer0_main_arm_elapsed seconds=…`）；第 25 次跑的守门两格要这一次改（守门把 S25-drift 的回落写成预期组、旧值扫法排除变异名，C774 原文现读）；产物一换，实验页「影响的决策」里关系为支撑的 21 行要逐行回看。写第 26 次登记：臂、对照、判据、失败条款、产物名，跑前写死；挂钟挪出 exact 产物（不往 `strip_timing` 加字段）；装置模型追平取号写两轮；写明「29 个区域重新全等就把 21 行写不受影响，有归不了因的差异逐条交用户」这一判据。

| # | 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
| 1 | 装置按取号写两轮追平之后，取号路径、整条流的段形、操作数、态数、系统配置槽的世代号与槽、窗口之前的操作数，与条款（`layout/01` 一·a1 行、整条流一句、八·取号行，D22 已定项 16）和 `crates/` 导出侧的窗口之前计数相同吗 | 相同 / 不同（哪一项、两边的值） | 第七节 7.2 A-Q1–A-Q7 任一值不符；`crates/` 导出的 `before_window_summary` 与装置的 `operations=` `writes=` 任一不等 | 第二段 N26 主产物对各行一次 | 开着 |
| 2 | 追平之后主点上装置与 `crates/` 的 29 个区域逐字节、字段表逐行、标签 6 两侧，以及 P8G 上同三样，还全等吗 | 全等 / 哪一处不等 | 区域比对任一 `sha256=` 不等或有 `unmatched`；`name=layout_field … equal=false`；标签 6 两侧不等；P8G 同 | 第二段 N26 主产物三处比对各一次，P8G 一次 | 开着 |
| 3 | N26 主产物对第 25 次主产物，有没有归不了因的差异？29 个区域重新全等、且差异全归得了因，E142 页「影响的决策」里关系为「支撑」的 21 行写「不受影响」；有归不了因的差异逐条交用户 | 无归不了因的差异（21 行写不受影响）/ 有（逐条列出） | 守门 `pass=false`；归因表有一组「未归因」；第 2 行不全等 | 第一段 N26R 对第 25 次主产物（漂移那一半）；第二段 N26 对 N26R 与对第 25 次主产物（取号那一半与总账） | 开着 |
| 4 | 挂钟挪出 exact 产物之后，stdout 产物里还有挂钟字段吗？同一份装置、同一份 `crates/` 再跑一趟，stdout 去掉 `E7INPUT` 起头的行之后逐字节相同吗 | 无挂钟且两趟相同 / 还有挂钟 / 两趟不同（哪一行） | 产物里 `grep -c 'seconds='` 非 0（或任何 `elapsed` 起头的挂钟字段）；两趟 `cmp` 不同 | 第二段 N26 主产物与第二趟；第二趟的条件见 Q142.164 | 开着 |
| 5 | 层 0 主臂整轮（装置默认现跑）在取号写两轮的流上：状态数是不是闭式 16777243、是不是穷尽、违例是不是 0、判决字段是什么 | 成立 / 不成立（哪一格） | `layer0 … exhaustive=false` 或 `states` ≠ 16777243；`layer0_violations` ≠ 0；`verdict` 里 `layer0_states_ok` / `layer0_violations` 与登记不符 | 第二段 N26 主产物一次（第一段 N26R 那一份作旁证：取号一轮的流上同样的读数） | 开着 |
| 6 | 第 25 次留下的两格改正之后再判：(a) 装置不剩旧值（旧值扫法排除变异名）；(b) 守门把 `p8g_control_main_point` 的变化写成预期组之后，N26R→N26 与 N25 主产物→N26R 两步守门过不过；(c) 整张变异表还全抓吗 | 都成立 / 哪一样不成立 | `residual_r26.sh` 总数非 0；守门 `pass=false`；变异表有一条没红或无效 | 第二段产物与变异表各跑一次（(c) 由主 agent 跑整表补判） | 开着 |

**问法与分母（写死，R92–R99 在第一节末）**：第 3 行的「差异」= 两份产物在第 1 行 `E7RESULT name=done ` 之前、去掉 `E7INPUT` 起头的行与空行、照第 22 次登记 R71 归一化与剔除（第 25 次登记第十三节 `r25_guard.py` 文件头的那一套）之后，按行组配对的逐行比；比之前先把第 25 次主产物放在 `git show 841b3beabb98:research/results/e142-new-pool-file-creation-dry-run-r25-main-2026-10-13.out`（1213 行，sha256 `a64b55e9c6acb46a57c92e8ef486a065e2cb9cb4a07a40c8e3b233886d96f2bb`；P8G 导出 `…/e142-r25-p8g-crates-dump-2026-10-13.out`，39 行，sha256 `6cf0735e387114072ec4a4c86c847fca6ee185b4b2087e7d344dea70e61fe6ea`）。

**读法写死**（跑之后不改）：

| # | 名字 | 写死成 |
|---|---|---|
| R92 | 「取号写两轮追平」 | 臂 N26（第五节 5.1）：N26R 上只改 A①–A⑧，取号两轮、每轮逐盘一写、第一轮之后一道池屏障、第二轮之后那道屏障与暖机第一次开头那道并成一道（`layout/01` 第 441、450 行）；世代号 mkfs 1、取号 2、3，暖机 4、5，新池新建文件 6，槽 = 世代号 mod 2（D22 已定项 16；`layout/01` 第 119、399 行） |
| R93 | 「挂钟挪出 exact 产物」 | 臂 N26 的 B①–B③：被测进程自己用 `Instant` 计的那一个数（`layer0_main_arm_elapsed` 的 `seconds=` 和 `threads=`）改成往 **stderr** 打一行 `E7TIMING name=layer0_main_arm_elapsed seconds=… states=… threads=…`，不再经 `emit`（也就不进 stdout 产物、不进文本数字自检的 `EMITTED_LINES`）。`replay.sh` 把 stdout 存成产物、stderr 存成 `$OUT_DIR/$tag.err`（第 4908、4911 行），所以 exact 比的是不带挂钟的 stdout；`strip_timing` 不加字段 |
| R94 | 「29 个区域」 | 主点（两块 4 GiB 盘）产物 `name=impl_bytes_equal_summary` 的 `matched=29`；P8G 另判（Q142.153），不并进 |
| R95 | 「归得了因」 | 差异组落进第六节 Q142.151 规则表登记的三类来源之一：D（装置源码自第 25 次产物以来已有的改动，N26R 对第 25 次主产物）、A（取号写两轮，N26 对 N26R）、T（挂钟挪出，N26 对 N26R），并且执行员在归因表里给出装置源码的行号，说明那一行怎样产出那个变了的字段。规则表之外变了的组、或表里的组变成了表外的形态，一律「未归因」 |
| R96 | 「21 行」 | E142 页「### 影响的决策」表里关系列为「支撑」的 21 行（`awk` 数过，页上共 42 行：支撑 21、备料 17、不影响 4；清单在第二节末）；备料、不影响的行这一次不回看 |
| R97 | 「写不受影响」 | 第六节 J1 全部满足时，写回员在这 21 行的「回看」格各写一句「2026-11-10 不受影响：第 26 次跑（取号写两轮追平、挂钟挪出）主点 29 个区域重新逐字节全等，差异全归因」并带产物名；J1 有一条不满足，**一行都不写「不受影响」**，逐条列出差异交用户 |
| R98 | 「第 3 行 (b) 守门」 | 两步：N25 主产物 → N26R（档 `drift`），N26R → N26（档 `acq2`），都由 `r26_guard.py` 判；规则表在 Q142.151 |
| R99 | 「主点」「P8G」 | 同第 25 次登记 R91：`driver_e142` 默认的两块 4 GiB 盘；P8G = 两块 8 GiB 盘（`driver_e142_p8g_crates_dump` 与产物里 `name=p8g_` 起头的行）；G1B、G11 是第八节的几何敏感性取样点 |

**未定项**：

1. **层 0 主臂整轮（Q142.150）的单趟挂钟 T0 没有读数**。装置今天默认就跑它（`e142_new_pool_file_creation_dry_run.rs` 第 7810–7862 行，没有环境变量挡；第 25 次跑的产物是被挡着的那一版，`layer0 … states=not_run`）。缺：16 777 243 个状态各自的恢复判定耗时（装置里没有可读的单态耗时；单测 `layer0_state_count_is_16777240_with_zero_violations` 第 8662 行在默认 `cargo test` 里整轮枚举同一量级的状态，但这次设计没有量它的挂钟）。谁来定：主 agent 经服务量一次（命令在报告「要主 agent 跑的命令」第 1 条），读数由承接的设计员填进第十四节。T0 若大于 2 小时，第十四节「续跑入口」缺 `enumerate_layer0_grouped`（第 4972 行）的分片与续跑，那是改装置的臂，不是这份登记能定的，交主 agent。
2. **整张变异表（Q142.157 (c)）的挂钟**：表 221 行加 6 行，每条变异的测试范围要看 `mutate.sh` 对 `layer0_state_count…` 那条整轮单测是否每条都跑。谁来定：主 agent 量一条变异的单条挂钟（命令在报告第 2 条）。

## 二、被测条款与它引的定义

下面一整块由 `python3 scripts/kb/quote-kb.py` 机械抄出（出口 `quote-r26.md`，71 行，sha256 `7ab1673edb865507eb4fc2cea908eb9fb5bc7175e685e7fca1d412c52d302495`，回读逐字节一致后原样追加）：先是问题单的两处起因（C1001 整行、事实表第 24 行整行），再是取号写两轮的条款（`layout/01` 第 38、62、85、119、399、436、441、450 行，D22 的 8 号分项行与已定项 16 的定案句）。**条款读出的几个数就是第七节 7.1 的 K 值**：取号 `2+2`、6 次操作、7 个崩溃状态；整条流 `2+2+2+1+2+2+1+2+24+2+1+2`、61 次操作、两态闭式 16777243、三态 16777268；世代号 mkfs 1、取号 2、3，暖机 4、5，新池新建文件 6；槽 = 世代号 mod 2。

**出处 `.claude/kb/checks-owed.md:640-640`（整段抄，未转述）**

```markdown
| C1001 | E142 二十一条支撑分项与 E164 一行拒算 | ① E142（新池新建文件的干跑） 产物一换，实验页「影响的决策」里关系为支撑的 21 行要逐行回看，装置追平之前判不了；② 复跑表 272 行里 E164（崩溃放量新设计的小范围功能测试） 一行拒算（`e164_crash_amplification_design_function_test.rs:45` 用 `CARGO_MANIFEST_DIR` 在运行时拼路径），这一行每趟照跑、不复用 | ① 修装置走第 26 次登记，29 个区域重新全等就写不受影响，有归不了因的差异逐条交用户；② 路径改成编进二进制或测试里现造，`admission.py replay-row-inputs .` 拒算 0 行 | 无；去向：门禁之后下一批 | 任务 11736；`records/2026-11-10-门禁与实验欠账收尾调度.md`「拒算」一节 |
```

**出处 `/home/fy5090/.local/share/singlefs/work/older-debts-fact-table/result.md:24-24`（整段抄，未转述）**

```markdown
| 2 | E142 装置（第 26 次登记、C1001 ①） | 装置没跟上取号写两轮，exact 产物里又带挂钟 `seconds=`；第 25 次跑的守门两格要第 26 次登记改；产物一换，页上关系为「支撑」的 21 行要逐行回看 | 没做。`scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs:7862` 仍打 `name=layer0_main_arm_elapsed seconds=…`；表行 `replay.sh:2662`、`:2663`；`research/prompts/` 里 e142 只剩 `e142-r19-runner-evidence`，第 26 次登记文件查不到；页「影响的决策」支撑 21、备料 17、不影响 4 行（`grep -c` 量的） | 写第 26 次登记（守门把 S25-drift 的回落写成预期组、旧值扫法排除变异名，C774 原文）；装置模型追平取号写两轮、挂钟挪出 exact 产物（不往 `strip_timing` 加字段）；跑第 26 次、新产物两行入表；29 个区域重新全等就把 21 行写「不受影响」，有归不了因的差异逐条交用户 | 无前置；21 行回看要等第 26 次的产物 | 实验设计员（登记）→ 执行员（改装置、出产物）→ 写回员（21 行与三语页） |
```

**出处 `.claude/kb/layout/01-first-txn.md:38-38`（整段抄，未转述）**

```markdown
这一节的槽号与写清单取自 E142（新池新建文件的干跑） 第二十五次跑（2026-10-13）的主产物：`git show 841b3beabb98:research/results/e142-new-pool-file-creation-dry-run-r25-main-2026-10-13.out`（`scripts/experiment-apparatus/replay.sh` 里 `@driver_e142` 那一行登记的那份）里 `name=width` 30 行 expected == actual，`name=write_list` 29 条写、12 个单元，窗口里 29 个写与 `crates/` 的导出按 (设备, 偏移, 长度) 配对、逐字节全等（`name=impl_bytes_equal_summary … matched=29 equal=29 unequal=0`，记账树根第 10 条已承诺预留两边都是 196 760）；分配记录树与 extent 树按 D8（核心索引结构） 已定项 14 按位置寻址。**那一份产物是取号写一轮时的历史产物，不是现行实测**：它的 `name=instances` 行是取号世代号 2、事务写系统配置槽 1 世代号 5（产物第 690 行），E142（新池新建文件的干跑） 装置没按取号写两轮重跑；现行是取号两轮（`crates/singlefs-core/src/transaction.rs` 的 `ACQUISITION_WRITE_ROUNDS`），系统配置那几行（a1、w3、w6、t15）照现行代码写，由 `crates/singlefs-verification/tests/new_pool_file_creation_publish.rs` 的 `root_slots_system_configurations_and_journal_ring_hold_the_published_state` 钉住两盘槽 0 世代号 6、槽 1 世代号 5。
```

**出处 `.claude/kb/layout/01-first-txn.md:62-62`（整段抄，未转述）**

```markdown
| a1 | 第一次可写挂载取号 | 系统配置，取号两轮（`crates/singlefs-core/src/transaction.rs` 的 `ACQUISITION_WRITE_ROUNDS`）：第一轮槽 0、世代号 2，第二轮槽 1、世代号 3，实例代号都写 1，每轮之后一道池屏障，两盘共 4 次槽写（D23（journal 的角色与格式） 已定项 16：取号先写进每一份系统配置、全或无，之后才动单元） | 18（自证） | 489 / 4096 槽 | 0 | 两盘各一份 | 一 |
```

**出处 `.claude/kb/layout/01-first-txn.md:85-85`（整段抄，未转述）**

```markdown
⇒ **写请求数**：事务本身单元 12 × 2 盘 = 24（t1..t12），记录 1 × 2 盘 = 2，根槽 1，系统配置 2 ⇒ **29 条**，屏障 3 道（单元之后、记录之后、系统配置轮换之后；按设备记 6 步），FUA 1 道；它前面是取号两轮的系统配置写 4 条（a1，每轮两盘各一条）与暖机两次空发布各 5 条（记录 2 + 根 1 + 系统配置 2）⇒ **取号 4 条 + 暖机 10 条**，屏障与 FUA 的道数以八那一节登记的段序列为准。
```

**出处 `.claude/kb/layout/01-first-txn.md:119-119`（整段抄，未转述）**

```markdown
| 自举头 | 槽世代号 | 8 | mkfs 1，取号 2、3，暖机 4、5，新池新建文件发布后 6 | D22（单元原子性怎么合成） 已定项 21 / 已定项 16 |
```

**出处 `.claude/kb/layout/01-first-txn.md:399-399`（整段抄，未转述）**

```markdown
| 系统配置槽轮换写 | 489 / 4096 | 根槽之后写槽 0，世代号 6，tail = 3（取号两轮：槽 0 世代 2、槽 1 世代 3；暖机两次各写一次：槽 0 世代 4、槽 1 世代 5） | D22（单元原子性怎么合成） 已定项 8 / 已定项 21 / 已定项 16；D23（journal 的角色与格式） 已定项 3 | 已定 |
```

**出处 `.claude/kb/layout/01-first-txn.md:436-436`（整段抄，未转述）**

```markdown
⚠️ **表里取 E142（新池新建文件的干跑） 产物的各行，对的是第二十五次跑的主产物** `e142-new-pool-file-creation-dry-run-r25-main-2026-10-13.out`（`scripts/experiment-apparatus/replay.sh` 里 `@driver_e142` 那一行登记的那份；第 36–39 行与第二十一次、第二十四次跑的主产物逐字相同）：暖机、普通发布两行同产物第 37、38 行逐字相同；取号那一行与整条流那一句是取号写两轮之后的形状（取号 `2+2`、6 次操作、7 个崩溃状态；整条流 `2+2+2+1+2+2+1+2+24+2+1+2`、61 次操作、16777243），由 `crates/singlefs-verification/tests/new_pool_file_creation_publish.rs` 的 `recorded_paths_match_the_registered_segment_sequences` 钉住，产物第 36、39 行仍是取号写一轮的 `2`、`2+2+1+2+2+1+2+24+2+1+2`、16777240（E142（新池新建文件的干跑） 装置没按取号写两轮重跑）；mkfs 那一行（产物第 35 行，装置补上每块盘 [0, 根环基址) 清零之后是 `14+1+1+1+4`、25 次操作）是两趟写序之前的段形，比钉它的用例（`2+12+1+1+1+4`、27 次操作）少作废那道屏障；装置是独立手写模型，不按产物比（第二十四次及之前的产物是 `12+1+1+1+4`、23 次操作，差每块盘 [0, 根环基址) 那一步清零）；普通发布与整条流里的 12 个单元是分配记录树、extent 树按位置寻址（D8（核心索引结构） 已定项 14）之后的；每次发布末尾有系统配置轮换之后那道屏障（D16（发布语义） 已定项 7），同一块盘上连着的屏障并成一道（D13（验证路线） 已定项 4）；`name=width` 30 行 expected == actual。那一份产物没有跑崩溃点重放整轮枚举（`name=layer0 … states=not_run closed_form=16777240 skipped=true`，够判之后不跑）；整条流那一句的 16777243 是 `crates/` 测试钉的两态闭式，取三态的全量 16777268 由 `crates/singlefs-verification/tests/crash_enumeration_new_pool_file_creation_stream.rs` 在门禁 crash-point-replay 里枚举。
```

**出处 `.claude/kb/layout/01-first-txn.md:441-441`（整段抄，未转述）**

```markdown
| 第一次可写挂载取号（实例代号 0 → 1，不是根槽写路径） | [a1 系统配置槽 × 2 盘，世代号 2] 屏障 [系统配置槽 × 2 盘，世代号 3] ⇒ `2+2`，6 次操作、7 个崩溃状态，种类 `[system_configuration_slot×2,barrier×2]\|[system_configuration_slot×2]`；取号写两轮、每轮逐盘写一槽，每轮之后一道池屏障（`research/prompts/m3-close-sysconf-r3-main-verification.md` 第四节「落地」），第二轮之后那道是取号写完之后的屏障（D23（journal 的角色与格式） 已定项 16 要取号两写之后、本实例第一个非系统配置写之前一道完成了的屏障；2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）：`crates/singlefs-core/src/transaction.rs` 的 `write_the_witnessed_acquisition` 每轮写完自己发一道（`CommitStep::Barrier`），首次挂载上第二轮那道与暖机第一次空发布开头那道背靠背、并成一道；第二次以后的挂载靠它把取号与写行那次发布的单元写隔开（2026-09-17 按代码改写）；世代号按 D22（单元原子性怎么合成） 已定项 16 逐盘 + 1，首次挂载两轮写出的是 2、3 | 一（a1）；D23（journal 的角色与格式） 已定项 16；两轮的 `2+2` 由 `crates/singlefs-verification/tests/new_pool_file_creation_publish.rs` 的 `recorded_paths_match_the_registered_segment_sequences` 钉住；产物 `name=segments path=instance_acquisition`（`research/results/e142-new-pool-file-creation-dry-run-2026-09-25-r16-arm-n15.out`）是取号写一轮时的，`segments=2` | 在（第六次跑起，整条流开头那一段） |
```

**出处 `.claude/kb/layout/01-first-txn.md:450-450`（整段抄，未转述）**

```markdown
⚠️ 发布与下一次发布之间有发布末尾那道屏障（系统配置轮换之后、发布返回之前，D16（发布语义） 已定项 7，C577（系统配置没见证到的最新根，乙罩不到） 加的）：上一次发布的 2 个系统配置槽写自成一段，不与下一次发布的单元写同段；同一块盘上连着的屏障并成一道（D13（验证路线） 已定项 4），取号第二轮之后那道与暖机第一次开头那道、暖机第一次末尾那道与第二次开头那道在录制流里各只记一道。新池新建文件那条流是整条流 `2+2+2+1+2+2+1+2+24+2+1+2`、16777243 个状态，种类 `[system_configuration_slot×2,barrier×2]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]`；61 次操作，16777243 是每次写只取两态的闭式，系统配置槽写取三态时是 16777268（`crates/singlefs-verification/tests/crash_enumeration_new_pool_file_creation_stream.rs` 的 `FULL_STATES`；产物 `name=segments_three_state path=post_mkfs_stream` 那一行是取号写一轮时的 16777260，E142（新池新建文件的干跑） 装置没按取号写两轮重跑）。mkfs 末尾有屏障，mkfs 与取号之间的接缝是切开的。段序列按整条录制流切，不按路径切，所以登记表按路径给的段序列是「孤立看」的形状，整条流那一句才是崩溃点重放枚举吃的。
```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:19-19`（整段抄，未转述）**

```markdown
| 8 | **固定结构的放置** | 系统配置每盘一份、槽轮换、取号写两轮；journal 环两盘镜像；mkfs 先把每盘 [0, 根环基址) 写 0、过屏障，再把第 0 代根种进全部根环区域并把区域整段写 0 **状态：已定。** **进度：已实现。** |
```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:351-351`（整段抄，未转述）**

```markdown
**定案**：系统配置槽：每盘恒 2 个槽，槽 i 的设备内偏移 = i × 固定结构槽距（D2（RAID 条带策略） 已定项 19），槽世代号从 1 起（mkfs 写下的两槽世代号都是 1）、每写一次 +1，下一次写的槽 = 世代号 mod 2，择槽取校验和过且世代号最大的（逐盘计：取号、发布末尾的轮换、取号全或无失败时的回卷三种系统配置写都写这块盘上自证过的槽里最大的世代号 + 1）。读槽 1 的偏移按这个次序取：本盘槽 0 可用（自证过、几何字段都在这一版读者收的范围里）时取它自述的槽距；否则取池里按设备次序第一块可用槽 0 自述的槽距；全池没有可用的系统配置槽 0 时，读者在格式允许的每个槽距上各读一槽（不小于 4096、512 的整数倍、不越过同一读者收的根环起点减 4096），只收自证过、且自述的固定结构槽距等于所在偏移的槽，几档都收得到时取世代号最大的，世代号相同取偏移小的；逐档读到自证过而池级字段越界的槽整池拒（第 5 句「越界即拒」），incompat 位不认得的槽不可择、不核偏移。逐档读到自证过的槽先判它带不带这一版读者不收之值（含 I-8.1（环几何够大） 三个 journal 环几何字段），带的整池拒、不再核偏移；不带的再核自述槽距等于所在偏移。根环：`r × P × chunk` 是设备内偏移，基址 = 系统配置「根环起点」字段（第一版 1 MiB），区域 r 的起点 = 基址 + r × P × chunk；区域内槽位 = `(txg div R) mod S`，无状态、只由 txg 与 (R, S) 决定，升格进已定项 2 的参数表。五句写死：
```

**E142 页「影响的决策」里关系为「支撑」的 21 行（现读页面 `.claude/kb/experiments/142-新池新建文件的干跑.md` 第 323–369 行，`awk` 按关系列数得 21；逐行列出，次序照页）**：

1. D5（快照 / 空间记账机制） 已定项 8
2. D5（快照 / 空间记账机制） 已定项 7
3. D5（快照 / 空间记账机制） 已定项 9
4. D5（快照 / 空间记账机制） 已定项 10
5. D5（快照 / 空间记账机制） 已定项 11
6. D18（块里携带什么信息） 已定项 2
7. D22（单元原子性怎么合成） 已定项 16
8. D15（格式冻结政策） 已定项 4
9. D15（格式冻结政策） 已定项 2
10. D15（格式冻结政策） 已定项 3
11. D15（格式冻结政策） 已定项 6
12. D3（空间分配） 已定项 10
13. D3（空间分配） 已定项 11
14. D23（journal 的角色与格式） 已定项 14
15. D17（实现分层与第三方管道） 已定项 2
16. D17（实现分层与第三方管道） 已定项 5
17. D23（journal 的角色与格式） 已定项 15
18. D16（发布语义） 已定项 8
19. D16（发布语义） 已定项 9
20. D13（验证路线） 已定项 4
21. D8（核心索引结构） 已定项 14

## 三、实现今天的样子

### 3.1 `crates/`（2026-11-10 现查；行号只是那一刻的；执行员不读这几份，比对侧只跑导出 bin）

1. **取号写两轮**：`crates/singlefs-core/src/transaction.rs` 第 1312 行 `const ACQUISITION_WRITE_ROUNDS: [AcquisitionWriteRound; 2]`，第 1323 行 `write_the_witnessed_acquisition`（第 1333 行 `for _round in ACQUISITION_WRITE_ROUNDS` 每轮逐盘一写、轮后一道池屏障），第 774 行 `roll_back_acquisition`（全或无失败时每块盘回卷一次）。
2. **段序列的钉值**：`crates/singlefs-verification/tests/new_pool_file_creation_publish.rs` 第 327 行 `recorded_paths_match_the_registered_segment_sequences`：第 359 行注释「段 [2] → [2+2]、闭式 4 → 7；mkfs 之后整条 57 → 61 步……闭式 16777240 → 16777243」，第 378 行钉 `(6, "2+2", 7)`，第 414 行钉 `(61, "2+2+2+1+2+2+1+2+24+2+1+2", 16_777_243)`；`crates/singlefs-verification/tests/crash_enumeration_new_pool_file_creation_stream.rs` 第 824 行 `FULL_STATES_WITH_TWO_STATES_PER_WRITE = 16_777_243`、第 828 行 `FULL_STATES = 16_777_268`（系统配置槽写取三态）。
3. **导出 bin**：`crates/singlefs-implementation-experiments/src/bin/e142_new_pool_file_creation_write_dump.rs`（504 行）第 221–248 行取「新池新建文件之前的全部操作」作窗口之前（取号两轮与暖机都在窗口之前），第 288 行起打 `name=before_window_summary operations= writes= last_five=`。只跑、不读、不改。
4. **没读的**：`transaction.rs` 取号函数的函数体、`admission.rs`、`make_filesystem.rs`、`singlefs-format` 常量（这一次不涉及）。

### 3.2 装置今天的样子（第十三节命令一、二现核）

- 11066 行，sha256 `f1fb548d…317a`，`git status` 无改动，与 `HEAD` 同一个 blob。
- **与 `git show 841b3beabb98:scripts/experiments/e142_new_pool_file_creation_dry_run.rs`（11092 行，`1655e36e…ee07a`，第 25 次产物所在提交里的那一份）的非注释差异共 132 行**（命令二，`diff` 滤掉注释行）：(i) 层 0 主臂整轮不再被 `E142_LAYER0_MAIN` 挡住，`main()` 第 7810–7862 行现跑，`layer0`、`layer0_violations`、`journal_effect`、`journal_header_reader_branch_counts`、`layer0_outcome_matrix*`、`verdict_layer0`、`layer0_main_arm_elapsed`、`done_layer0_main` 各行有值，`name=verdict` 里 `layer0_states_ok`、`layer0_violations`、`journal_differing_states` 不再是 `not_run`；准入键固定 `E142`；(ii) feature bits：删了 `INCOMPAT_RETIRED_FIRST_SSD_LINE_BIT` 与 `refuses_retired_bit0` 字段，`incompat_bits_are_mountable` 多判 `has_truncation_fields`（位 2 必带），几条单测断言跟着变，`name=gap id=G12` 的文字变；(iii) 注释里的日期删除。这一组改动是第 3 行守门「漂移」那一档（`drift`）要归因的 D 类来源。
- **取号**：`acquire_instance`（第 2375 行）只写一次系统配置槽（槽 = `SYSTEM_CONFIGURATION_GENERATION_AT_INSTANCE_ACQUISITION % 2`，常量 330 行 = 2），段收尾靠暖机第一次空发布开头那道屏障；`system_configuration_write_for_publish`（第 2811–2815 行）世代号 = txg + 2、槽 = 世代号 mod 2；`name=instances` 行（第 7979 行）打 `acquisition_system_configuration_generation=` 与 `transaction_system_configuration_generation/slot=`；`name=gap id=G21`（第 5660 行）文字写「取号那一步……不另加屏障……段序列独占一行[system_configuration_slot×2]」。
- **用到取号的地方**：`built_pool`（第 8243 行起）与 `RecordedNewPoolFileCreationStream`（第 4464 行起）记 `acquisition_operation_count`；`new_pool_file_creation_stream_paths`（第 4488 行起）按它切 `instance_acquisition`、`warm_up` 两条路径；产物 `name=segments` 五行（含 `path=instance_acquisition`、`path=post_mkfs_stream`）、`name=segments_three_state`、`name=segments_sensitivity`（G11）、`name=before_window_summary side=model`、`name=journal_header_write_digest`（每次写一行）、`name=gap id=G21`、`name=instances`、窗口里系统配置槽那两次写的 `name=device_region_bytes` / `name=write_list_row` 与 `name=r18_clause_fields region=system_configuration`（第 6093 行起，按窗口里那一槽读）。
- **取号相关的钉旧值单测与旧字面**（第五节 A⑥ 要跟着改的）：第 8688–8745 行（整条流 `2+12…`、取号 2 次操作 `vec![2]`、57 次操作等）、第 8845 行（`("instance_acquisition", 2, "2", 4, 9, "2", "[system_configuration_slot×2]")`）、第 8981 行 `(47, 33)`、第 9061 行世代号阶梯、第 8651–8668 行与第 8831 行的 16777240 / 16777260；`residual_r26.sh` 在今天的装置上逐行列出这些命中（命令四，共 32 处）。
- **挂钟**：第 7810 行 `Instant::now()`，第 7862 行 `emit(… "name=layer0_main_arm_elapsed seconds={:.3} states={} threads={…}" …)`，是装置里唯一的挂钟字段（`grep -n 'seconds=\|Instant\|SystemTime\|elapsed()'` 现查，命令二）；`emit`（第 7209 行）打 stdout 并推进 `EMITTED_LINES`。
- **`name=p8g_control_main_point`**（第 8226 行）：把 P8G 一段里重算出的判决字段与主点已打印的 `name=verdict` 那一行逐字段比（`compare_stream_verdict_fields_to_printed_verdict`），打 `fields= equal= unequal= unequal_fields=`；第 25 次的守门把它放在零差异组里，N25M → N25 一步里它的 `equal=` 回落到 12，是守门输的一格（C774 原文）。
- `grep -c 'ckpt_cost\|checkpoint_cost' <装置>` 为 0（同第 25 次，不在射程）。
- 独立 bin 809 行，`2f41d6e5…49fb7`，文件头两种调用同第 25 次。

### 3.3 装置要改的地方

见第五节 5.1 臂 N26 的 A①–A⑧、B①–B③。

## 四、跑之前已经存在的数

写判据之前已经读到或算出的数，照实列。**这一次的门槛不盲**：第 1 行要比的几个值在条款里，`crates/` 的测试把同样的值钉成了数（下面第 2 条），第 2 行比的是装置与 `crates/` 的字节；第 3 行比的是第 25 次的产物本身。哪一格用到了哪个数、怎么用，逐条写。

1. **`.claude/kb/checks-owed.md` C774 与事实表第 24 行**（第二节整行抄了）：第 25 次跑（2026-10-13）装置追平之后主点 29 个区域逐字节相同、字段表 180 行全等（`git show 841b3beabb98:research/results/…r25-main-2026-10-13.out` 第 414、907 行，是 C774 记的）；守门 Q142.138 (1) 按登记字面记 F25d 一次输，`p8g_control_main_point` 回落到 `equal=12 unequal=0`；旧值扫法把变异名 M233 里的「233」当旧偏移，F25j。**影响**：第 2、3 行要的「全等」，上一次的读数不能搬过来：第 25 次产物是取号写一轮的时候出的（取号写两轮是用户 2026-10-19 定的，晚于产物 2026-10-13，`layout/01` 第 436、450 行写明那份产物「是取号写一轮时的历史产物」）；今天 `crates/` 已是两轮，装置没跟上，所以这一次装置侧要改、`crates/` 侧不改，29 个区域能不能重新全等，登记时不知道，是这一次要量的。守门与扫法的两处改法只改判法，不看读数。
2. **`crates/` 一侧已经钉着的数**（第三节 3.1 第 2 条，设计时读了那几行注释与断言）：取号 `(6, "2+2", 7)`；整条流 `(61, "2+2+2+1+2+2+1+2+24+2+1+2", 16_777_243)`；`FULL_STATES = 16_777_268`；注释「57 → 61 步」。**影响**：它们与条款（第二节）和第七节 7.2 的独立算术（`anchors_r26.py` 的输出）三方一致，7.1 的 K 值按条款原文当期望；`crates/` 的测试是旁证，不当装置的期望来源（装置照条款写，执行员不读那几份）。
3. **自己算的**（第十三节命令三、四、五）：(a) `anchors_r26.py`：条款的两态闭式式子（每段 2^k − 1 个非满子集、加终态 1）在 mkfs `14+1+1+1+4` 上给出 16402、在取号一轮的整条流上给出 16777240（与旧产物值相同），在取号两轮的流上给出 7（取号）、16777243（整条流）、三态 16777268 / 旧三态 16777260，窗口之前 (51, 35)（旧 (47, 33)），整条流写数 43（61 次操作里 18 步是 9 道池屏障按设备记）。(b) `residual_r26.sh` 在今天的装置上 `O_` 样式 0、`A_` 八个样式共 32 处命中（逐样式 18、1、3、1、1、2、1、5，加上 `O_` 0 合计 32）；六份合成输入（变异名、裸 233、同行变异名与 310、`header311`、表第 2 列裸 233、表第 2 列变异名）上依次 0、1、1、1、1、0，**已经在设计时跑出来**。**影响**：旧值扫法排除变异名这一处改正有对照物（PC-sweep 两端都见过）；`A_` 样式「每个样式 ≥ 1」是 PC-old26 的期望。
4. **分辨率与翻面差**（`experiment-designer` 3a ②）：这一次的判据全是计数与逐字节比较（sha256、行数、`cmp`），没有时钟或采样分辨率问题；唯一的时钟量（`layer0_main_arm_elapsed`）不进任何判据，只作报告里的耗时。翻面观测的预期差：取号一轮 → 两轮，装置侧 `before_window_summary` 从 (47, 33) 变 (51, 35)、取号路径 6 次操作 / 7 个态（旧 2 / 4）、整条流 59 → 61 次操作、16777240 → 16777243；这些在 N26R（取号一轮）与 N26 上符号相反，**判据没有在某一臂基线上退化的相对量**（都是绝对值与零差异）。
5. **触发事件**（3a ③）：没有「搬空发布、回收、淘汰、写满」这类要触发的事件；层 0 枚举的态数由两态闭式 16777243 定（A-Q4），不靠某事件发生。不适用。
6. **没读的**：`research/results/` 下第 25 次产物的内容（只对它的文件名、行数、sha256 算过；执行员在第一段步 ④ 才读）、`.claude/kb/experiments/` 下 E142 页除「影响的决策」表（第 323–369 行）之外的任何小节、实验变更史、第 25 次执行员报告。第 25 次登记（1366 行）读了第 1–40、40–77、446–560、595–630、678–765、985–1031 行与 `r25_guard.py` 全文；第 21–24 次登记只读了与旧值扫法、`label6_anchor.py`、`p8g_region_compare.py` 取法有关的行。

## 五、臂、阳性对照、真实基线

### 5.1 臂

问题单的第 1–3 行不是在两种设计之间选，是「照条款改完的装置，与条款、与 `crates/`、与上一次产物对不对得上」。臂是装置的两个样子，每一条对面的臂都是装置真有过的样子，不另造。

| 臂 | 怎么做 | 做完会怎样（待证命题，由第六节的格核，不当前提） | 谁认它 |
|---|---|---|---|
| **N26R（参照，第一段先跑）** | 今天仓里的装置一字不改（`f1fb548d…317a`），冻结一份 `n26r.rs`；照 `driver_e142` 两条命令跑 | 取号一轮：取号路径 `2`、2 次操作、4 个态；整条流 11 段 `2+2+1+2+2+1+2+24+2+1+2`、59 次操作、两态 16777240、三态 16777260；窗口之前 (47, 33)；世代号取号 2、新池新建文件 5（槽 1）；对 `crates/` 导出（两轮）的窗口里系统配置槽那次写落点与内容不同，区域比对不全等（PC-acq）；对第 25 次主产物的差异全在 D 类来源（Q142.151 的 `drift` 档）；层 0 整轮 16777240 个状态、违例 0 | 取号写一轮：D23 已定项 16「取号先写进每一份系统配置（一次系统配置槽写，世代号 +1）之后才动单元」的原写法，用户 2026-10-19 定取号写两轮之前的样子 |
| **N26（真实基线，第二段改完）** | 在 N26R 上只改 A①–A⑧、B①–B③（下表），照 `layout/01` 第 441、450 行与 D22 已定项 16 原文写 | 取号 `2+2`、6 次操作、7 个态；整条流 12 段、61 次操作、16777243 / 16777268；窗口之前 (51, 35)；世代号 2、3、4、5、6 与槽 0、1、0、1、0；主点 29 个区域对 `crates/` 逐字节相同（Q142.147）；对 N26R 只有 `acq2` 档登记的那几组行变，另 B 类：`layer0_main_arm_elapsed` 离开 stdout；层 0 整轮 16777243 个状态、违例 0 | 取号写两轮：D22 已定项 8、已定项 16，`layout/01` 一·a1 行与八·取号行 |

**臂 N26 的改法**（每处照第二节条款原文写，不照 `crates/` 写）：

| # | 改什么 | 依据 |
|---|---|---|
| A① | `acquire_instance`（第 2375 行）改成两轮：第 r 轮（r = 1、2）逐盘写一次系统配置槽，`slot_generation` = 1 + r（2、3），槽 = 世代号 mod 2（0、1），`journal_instance` 两轮都写取到的新实例号；**第一轮之后记一道池屏障**（每盘一步，写法同装置里别处的池屏障），**第二轮之后不另记屏障**（它与暖机第一次空发布开头那道背靠背、并成一道，装置现有的「同一块盘上连着的屏障并成一道」切段规则照用；若装置现有切段不并，则第二轮之后不记，靠暖机开头那道） | `layout/01` 第 441 行（「第二轮那道与暖机第一次空发布开头那道背靠背、并成一道」）、第 450 行（「取号第二轮之后那道与暖机第一次开头那道……在录制流里各只记一道」）、第 62 行（「第一轮槽 0、世代号 2，第二轮槽 1、世代号 3，实例代号都写 1，每轮之后一道池屏障」）；D22 已定项 8 |
| A② | 世代号常量拆成「取号第一轮 = 2」「取号第二轮 = 3」两个，旧常量 `SYSTEM_CONFIGURATION_GENERATION_AT_INSTANCE_ACQUISITION` 不留；`system_configuration_write_for_publish`（第 2811 行）世代号 = txg + 3，槽 = 世代号 mod 2（暖机 txg 1、2 得 4、5；新池新建文件 txg 3 得 6、槽 0） | `layout/01` 第 119 行（「mkfs 1，取号 2、3，暖机 4、5，新池新建文件发布后 6」）、第 399 行；D22 已定项 16 |
| A③ | `name=instances` 行：`acquisition_system_configuration_generation` 拆成 `acquisition_first_round_system_configuration_generation`（2）与 `acquisition_second_round_system_configuration_generation`（3）；`transaction_system_configuration_generation` = 6、`transaction_system_configuration_slot` = 0；别的字段不动 | 同 A② |
| A④ | 路径切点：`acquisition_operation_count` 现在是 mkfs 之后多 6 次操作（4 写加一道池屏障按设备记 2 步）；`new_pool_file_creation_stream_paths` 按它切，不另改；暖机、新池新建文件两截的写不变 | `layout/01` 第 441 行（`2+2`、6 次操作、7 个崩溃状态） |
| A⑤ | 新加一行产物 `name=segments_geometry point=G1B path=instance_acquisition operations=… segments=… closed_form=… kinds=…`，用 `PoolParameters::one_device_with_barriers()` 跑 mkfs、取号，与 `path=mkfs` 那一行同一种切法与写法，放在 mkfs 那一行之后 | 第八节几何敏感性（一块盘；取号每轮每盘一写） |
| A⑥ | 单测跟上：第三节 3.2「取号相关的钉旧值单测」那几处改成 7.2 A-Q1–A-Q7 的值；`layer0_state_count_is_16777240_with_zero_violations` 改名 `layer0_state_count_is_16777243_with_zero_violations`、钉 16_777_243；加四条：(i) 取号恰四次系统配置槽写，槽 0 世代 2、槽 1 世代 3，两轮之间恰一道池屏障、第二轮之后没有；(ii) 世代号阶梯 1、2、3、4、5、6 与槽 1、0、1、0、1、0 逐个对（mkfs 两槽同写 1）；(iii) `name=segments_geometry … path=instance_acquisition` 的四个字段等于 7.2 A-Q7；(iv) G11 点上取号路径等于主几何那一行 | 第七节；`test-discipline.md`「只让多条臂互相比」 |
| A⑦ | 注释与文字跟上：`name=gap id=G21`（第 5660 行）的文字改成取号两轮、每轮之后一道池屏障、第二轮之后那道并进暖机开头、取号路径段序列 `2+2`；第 2371–2374 行、第 2811 行、第 4464 行、第 8243–8244 行、第 8688 行的注释同改。**G19 与别的 gap 一字不改** | 同 A① |
| A⑧ | 变异表：锚点不再恰好命中一次的行重锚（预计与取号函数体、世代号常量有关，别的照 `mutate.sh` 的锚点核对逐条列），替换文的语义不变；加 M246–M251（第九节） | `mutation-sampling.md` 第七类 |
| B① | 第 7862 行那个 `emit(…"name=layer0_main_arm_elapsed seconds=… states=… threads=…")` 改成 `eprintln!` 一行，前缀 `E7TIMING `（不是 `E7RESULT `），字段同前；行由一个小函数 `layer0_main_arm_timing_line(seconds, states, threads) -> String` 拼，便于单测。计时仍在被测进程里、用它自己的 `Instant`（第 7810 行），不挪到转发输出的循环里 | R93；`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」 |
| B② | 单测：`layer0_main_arm_timing_line` 的返回值以 `E7TIMING ` 起头、不以 `E7RESULT ` 起头、含 `seconds=`；加一条：往一个 `Emitter` 里 `emit` 这一行之外的所有结果行，没有任何一行含 `seconds=` | R93 |
| B③ | `strip_timing`、`replay.sh` 的比对、别的结果行一处不改；`name=done_layer0_main` 仍在 stdout（无时钟） | 用户批次要求「不往 `strip_timing` 加字段」 |

**「怎么做」推得出「做完会怎样」**：A①–A④ 只在取号那一段多一轮写和一道屏障、把后面每次系统配置槽写的世代号 +1、槽翻一位；所以整条流多一个 2 写段（两态闭式 +3，三态 +5）、操作数 +4（多 2 写，加第一轮之后那一道池屏障按设备记的 2 步；第二轮之后那道并进暖机开头，不多记）、窗口之前 +4 操作 +2 写、窗口里系统配置槽那次写的槽与世代号变（5/槽 1 → 6/槽 0）；单元、记录、根槽的字节不变。「窗口里别的 28 个区域与 `crates/` 仍逐字节相同」**推不出来**，那是这一次要量的（对 `crates/` 侧：取号两轮早已在里面）。B 只改一行的去向，不改 stdout 里任何别的字节。

**没改的（写死，不在这一次的射程里）**：ckpt_cost、mkfs 中途崩溃的语义、一盘几何上的逐字节比、墓碑树与退役树非零根时 H 的读法（同第 25 次）；取号两轮中途崩溃的语义（第一轮落盘、第二轮没落盘那些态）靠层 0 整轮枚举里的恢复判定间接走到，不是这一次要答的格；全或无失败时每块盘回卷一次（`roll_back_acquisition`）装置不模型。

### 5.2 阳性对照（每一条被判的比对都有；每一条跑出来的臂上都跑；按段钉）

| # | 对哪条比对 | 怎么做 | 该看到什么 | 落在哪一跑 | 看不到时 |
|---|---|---|---|---|---|
| **PC-acq 区域比对分得出取号一轮与两轮** | Q142.147、Q142.148 | N26R 产物的 `impl_bytes_equal_summary`、`impl_bytes_unmatched` 各行与 `before_window_summary` 两侧 | `unequal` + 两个 `unmatched` 之和 ≥ 1，且每个不等或未配上的区域 `region=` 都是 `system_configuration`；两侧 `before_window_summary` 的 `operations=` 不等（装置 47，`crates/` 51）、`writes=` 不等（33 对 35） | 第一段 N26R | 作废 V26d |
| **PC-g26 守门脚本分得出每一种违例** | Q142.151 | `r26_guard.py --selftest` 一次、`R26_GUARD_BREAK=1` 一次（合成输入的集合见 Q142.151） | 前一次每个样本 `ok=True`、`all_ok=True`；后一次点名的那个样本 `ok=False`、`all_ok=False` | 两段各跑一次 | 作废 V26d |
| **PC-g26b 守门看得见真差异** | Q142.151 | 第一段 `drift` 档的 `R26 changed_lines_total=`；第二段 `acq2` 档的同一个数 | 都 ≥ 1（N25 主产物与 N26R 在层 0 各行上必不同；N26R 与 N26 在 `segments path=instance_acquisition` 上必不同） | 两段各一 | 为 0 ⇒ 守门没看见，作废 V26d |
| **PC-sweep 旧值扫法分得出旧值、也认得变异名** | Q142.157 (a) | `residual_r26.sh` 在草稿目录的六份合成输入上跑（第十三节命令四，设计时已跑出 0、1、1、1、1、0） | 同这六个数 | 第一段、第二段各一次 | 作废 V26d |
| **PC-old26 旧值样式分得出旧值** | Q142.157 (a) | `residual_r26.sh` 在 N26R 的装置源码与第一段开头那张变异表上跑 | `O_` 样式 0；`A_` 八个样式每个 `count` ≥ 1（设计时是 18、1、3、1、1、2、1、5） | 第一段 | 作废 V26d |
| **PC-timing 挂钟检查分得出挂钟** | Q142.149 | N26R 产物 `grep -c 'seconds='`；N26 `.err` 文件 `grep -c '^E7TIMING name=layer0_main_arm_elapsed seconds='` | N26R 产物恰 1 行含 `seconds=`（它的 `layer0_main_arm_elapsed` 行）；N26 `.err` 恰 1 行 | N26R 在第一段，N26 在第二段 | 作废 V26d |
| **PC-indep 独立 bin 分得出** | Q142.148 | 独立 bin 第一种调用 `<N26R 产物> <N26R 产物的 crates/ 那一半>` | `name=independent_region_diff_summary` 的 `unequal` ≥ 1，或有 `name=independent_region_unmatched` 行 | 第一段 | 作废 V26d |
| **P2（装置自己的翻位对照）** | Q142.147 | 产物四行 `name=positive_control_p2` | 四行都是 `regions_flagged=1`、`matches_injection=true` | N26R、N26 两份产物各自 | 作废 V26d |
| **PC-e 字段表分得出** | Q142.147 | 产物 `name=layout_field_positive_control`（与 `p8g_` 那一行） | `injected=4 detected=4 exact=true` | 两份产物各自 | 作废 V26d |
| **PC-k P8G 逐区域比对分得出一个字节** | Q142.153 | 第 22 次登记命令五：`p8g_region_compare.py --selftest`，再设 `P8G_COMPARE_BREAK=1` 一次 | `True`；`False` | 第二段 | 作废 V26d |
| **PC-j 标签 6 解码分得出** | Q142.147 | 第 22 次登记命令四：`label6_anchor.py` 不带输入一次、`LABEL6_ANCHOR_BREAK=1` 一次 | 合成自检的 `DECODE` 行照第 22 次命令四的输出；弄坏开关下那一格判错 | 第二段 | 作废 V26d |
| **PC-layer0 层 0 判别力** | Q142.150 | 产物 `name=verdict` 的 `control_violations_ok`、`positive_control_main_geometry_ok`，以及 `pc2 caught`（`verdict` 里 `positive_control_main_geometry_ok` 已含 `pc2.caught == 12`） | 都是 `true` | N26R、N26 两份产物各自 | 作废 V26d |
| **PC-a 锚点脚本没被改** | 第七节 7.2 | 第十三节命令三重跑 `anchors_r26.py` | 输出的 sha256 是 13.3 登记的值 | 两段各一次 | 作废 V26b |

**对照逐格落在哪一跑**：第一段判 Q142.146（N26R 的基线读数）、Q142.151 `drift` 档、Q142.152 的漂移半边与 PC-acq；对照全在第一段 N26R 产物与第一段命令里。第二段判 Q142.147–Q142.150、Q142.153–Q142.157，对照在第二段 N26 主产物、第二段自己重跑的 PC 里；第一段的 N26R 产物只作 `acq2` 档的参照物，不替第二段当对照。

### 5.3 真实基线

**真实基线是臂 N26**。第 1 行「相同」与「不同」、第 2 行「全等」与「哪一处不等」、第 3 行「无归不了因的差异」与「有」都是正当结果（第十节），不许写成「装置没测出差别所以坏了」。⚠️ 两份实现照同一份 kb 改，一起错时由第七节盯：取号 `2+2` / 7 / 61 / 16777243 / 16777268 出自条款原文（`layout/01` 第 441、450 行）、`crates/` 的测试钉了同一批数（3.1 第 2 条），世代号阶梯出自 `layout/01` 第 119 行；**这些绝对值是独立于装置的**，装置与 `crates/` 区域全等而这些值不符 ⇒ F26a「两侧一起错」。

### 5.4 实现量、分段、步骤与挂钟

**量（估计）**：没有历史族、崩溃支线、坏镜像、新子系统。装置改 A①–A⑧ 约 40–70 行（一个取号循环、三个常量、一行产物、约十五处钉值、四条新单测、注释），B①–B③ 约 10–15 行（一个小函数、一行 `eprintln!`、两条单测）；变异表重锚约 2–6 行、加 6 行；守门脚本 `r26_guard.py` 在 `r25_guard.py` 的 320 行上改约 80–120 行（两档规则表、`p8g_control_main_point` 与层 0 一族的预期组、合成自检样本）。跑：`driver_e142` 每段 1 次（第二段另加第二趟 1 次）、P8G 导出第二段 1 次；装置每段编两次、单测各 1 次；独立 bin 两次；草稿脚本约 20 次；变异子集第二段 6–8 条；整张变异表 227 条左右（主 agent 跑）。

**分段**（每段交回之后主 agent 对着问题单判续不续；一行开着的都不剩就停）：

- **第一段**：补问题单**第 3 行的漂移那一半**（N25 主产物 → N26R，档 `drift`）和 PC-acq，并给出 T0（`E7TIMING` 行的 `seconds=`）。补完之后第 3 行还差取号那一半，所以第 1–6 行这一段都不够判；价值是：先看装置源码自第 25 次以来的 132 行非注释改动到底带来哪些行变，再定要不要往下改（有「未归因」就停，不必白改装置）。
- **第二段**：补问题单**第 1、2、3（取号那一半与总账）、4、5、6 行**。补完之后第 1–5 行够判，第 6 行 (a)(b) 够判、(c) 等主 agent 跑完整表补判。第一段打中 F26c（漂移里有未归因）时第二段照样能跑，由主 agent 定。

**第一段的步骤**（写死；「之前不许读」一列指那一步做完之前不许读的东西）：

| 步 | 做什么 | 之前不许读 |
|---|---|---|
| ⓪ | (1) 起跑前提 P1–P4（第十一节）。(2) `ps -o pid,args -u "$(id -u)"` 看负载，照共用约束办。(3) 快照 A1：`find crates -type f -not -path '*/target/*' -print0 \| sort -z \| xargs -0 sha256sum` 存草稿目录；随后做步 ③ 的 B1、C1。(4) 重建草稿脚本、核 sha256（13.3 的表）。(5) 取出第 25 次主产物与 P8G 导出（`git show 841b3beabb98:research/results/<名>`）进草稿目录，核行数与 sha256（1213、39；`a64b55e9…`、`6cf0735e…`）；**这时只核 sha256 与行数，不读内容** | 第 25 次主产物内容 |
| ① | 把装置原样拷进草稿目录 `n26r.rs`，核 sha256 `f1fb548d…317a`、11066 行；变异表拷一份 `n26r.tsv` 核 `64caabe9…`。命令二的 `diff`（`HEAD` 版对 841b3 版，滤掉注释）原样进报告 | 第 25 次主产物内容 |
| ② | 臂 N26R：装置一字不改，在仓里编（release），照 `driver_e142` 两条命令原样跑（装置 stdout 存产物，stderr 存 `.err`），产物存 `research/results/e142-new-pool-file-creation-dry-run-r26-arm-n26r-2026-11-10.out`；准入放行才算（退 0；退 77 ⇒ 停机 S26-admission，stderr 原样交回）。同一步照 `driver_e142_p8g_crates_dump` 跑 P8G 导出，存 `research/results/e142-r26-p8g-crates-dump-n26r-2026-11-10.out`。`.err` 存 `research/results/e142-r26-timing-n26r-2026-11-10.err`；`E7TIMING` 的数不适用（N26R 的挂钟还在 stdout 里，`layer0_main_arm_elapsed` 行），T0 取那一行 | — |
| ③ | 快照 B1，与 A1 `cmp`（V26c）。PC-acq（N26R 产物的区域比对与 `before_window_summary` 两侧）、PC-indep、PC-timing（N26R 那一半）、P2、PC-e、PC-layer0。读 N26R 的第 1–6 行读数，作 Q142.146 | 第 25 次主产物内容 |
| ④ | **从这一格起才读第 25 次主产物**。`r26_guard.py` 的 `drift` 档（`python3 r26_guard.py <第 25 次主产物> <N26R 产物> drift`），再逐组写归因表（组、变几行、归到 D 类的哪一条、装置行号）；归因用 `diff <(HEAD 版) <(841b3 版)` 的非注释行和装置源码行号，不凭印象；`PC-g26`、`PC-sweep`、`PC-old26` | — |
| ⑤ | 快照 C1，与 B1 `cmp`。交回前做第十一节「交回之前」。报告里写 T0 | — |

**第二段的步骤**：

| 步 | 做什么 | 之前不许读 |
|---|---|---|
| ⓪ | (1) 起跑前提 P1–P4，其中 P1 的装置与表是第一段交回时的值。(2) 快照 A2：同 A1，与第一段 C1 `cmp`（`crates/` 变了 ⇒ 记下变了哪些文件，只要不是 `crates/singlefs-core/src/transaction.rs`、`singlefs-format`、`make_filesystem.rs`、`admission.rs`、导出 bin 就不停机；是就 S26-open）。(3) 重建草稿脚本，核 sha256。 | — |
| ① | **读第二节条款与第七节，改装置**：A①–A⑧、B①–B③；每处改完写进草稿目录的改动清单。单测：`cargo test --release -p e7-index-bench --bin e142-new-pool-file-creation-dry-run`（经任务服务，命令由主 agent 交；执行员不自己起）必须 0 败；红了 ⇒ 改法照登记写不绿时 S26-scope。写 `r26_guard.py`（底本与必须的改法在 13.4）与它的合成自检，核 PC-g26，**sha256 与装置 sha256 记进第十二节修订** | — |
| ② | `residual_r26.sh` 在 N26 的装置与变异表上：`total` 必须 0 ⇒ Q142.158；再跑第 22 次登记命令六（标签 6 四种旧写法）、第 23 次登记命令十二（四个计数 0、0、1、0；路径改 `scripts/experiment-apparatus/`）。变异表重锚、加 M246–M251，小表对改后源码跑一次（第九节），抓到 / 无效 / 没红三数进第十二节 | — |
| ③ | 快照 B2。臂 N26：编（release），照 `driver_e142` 两条命令原样跑，stdout 存 `research/results/e142-new-pool-file-creation-dry-run-r26-main-2026-11-10.out`，stderr 存 `research/results/e142-r26-timing-2026-11-10.err`（准入同第一段）；照 `driver_e142_p8g_crates_dump` 跑 P8G 导出存 `research/results/e142-r26-p8g-crates-dump-2026-11-10.out`；快照 C2，与 B2 `cmp`（V26c）。立即跑 `python3 scripts/gate/admission.py experiment <仓根> E142`（Q142.163） | — |
| ④ | Q142.147（`label6_anchor.py --decode` 四行）、Q142.148（独立 bin 第一种调用，后一半用 `awk` 切进草稿目录）、Q142.149、Q142.150、Q142.153（`p8g_region_compare.py`、`label6_anchor.py --decode <导出> p8g_device_region_bytes`）、Q142.154–Q142.156、Q142.158、PC 全部；守门 `acq2` 档（N26R → N26）与总账（第 25 次主产物 → N26，档 `same` 只打印、不判）；归因表 | — |
| ⑤ | **条件步**：第二趟 `driver_e142`（只装置那一条，stdout 存草稿目录），与 N26 主产物各去掉 `E7INPUT` 起头的行 `cmp`（Q142.164）。条件见 Q142.164 | — |
| ⑥ | `scripts/experiment-apparatus/replay.sh` 第 2662、2663 行改指两份新产物名（`E142\|@driver_e142\|\|e142-new-pool-file-creation-dry-run-r26-main-2026-11-10.out\|exact`、`E142\|@driver_e142_p8g_crates_dump\|\|e142-r26-p8g-crates-dump-2026-11-10.out\|exact`）；`.claude/gate.d/replay-comparison-units.tsv` 为这两个比对对象各加一行（单元表出处等照该文件头）；机读登记与第十一节表对得上。**只有 Q142.147–Q142.154 都够判之后才改 `replay.sh`** | — |
| ⑦ | 交回前做第十一节「交回之前」；整表变异命令（第九节）原样写进报告交主 agent | — |

## 六、报哪些量与各自的判据

编号 Q142.146 起（Q142.161 号空着不用）。每格各报各的判定，不合取；合取（某一行「够判」要哪几格都过）由主 agent 对着问题单做，唯一的例外是 J1（本节末），它是用户要的那条判据。臂的定义并排放：N26 =「取号两轮（每轮逐盘一写，第一轮之后一道池屏障），后面每次系统配置槽写的世代号 +1；层 0 挂钟行改去 stderr」。能从定义直接推出的（装置源码里多了一轮；世代号 = txg + 3）不算判据；判别力在「装置的段形、闭式、世代号与槽跟条款同不同」「两份独立实现写出的字节同不同」「改了这一处之后别的组、别的行、别的列动没动」「值本身是不是条款给的数」。

| 格 | 怎么算（产物里的行） | 门槛 / 判定 | 对应问题单哪一行 · 取什么值让它翻面 |
|---|---|---|---|
| **Q142.146 N26R 基线读数（第一段起点，只报）** | N26R 产物里 `name=segments`（五行）、`segments_three_state`、`before_window_summary side=model`、`instances`、`layer0`、`verdict`、`layer0_main_arm_elapsed` 整行 | 只报，不判（都能从「取号写一轮」推出）；T0 取 `layer0_main_arm_elapsed` 的 `seconds=` | 不对应翻面。附带，够判后不跑第二遍 |
| **Q142.147 主点三处（R94、R88 的口径同第 25 次）** | (1) N26 主产物 `name=impl_bytes_equal_summary`，不同时取 `equal=false` 与 `impl_bytes_unmatched` 各行；(2) 全部 `name=layout_field` 与 `name=layout_field_summary`；(3) `label6_anchor.py --decode <主产物>` 四行 `committed_reservation=` 与装置 `name=accounting` 的那个 | (1) `matched=29 equal=29 unequal=0`、两个 `unmatched` 0 ⇒「全等」；(2) `rows=180 equal=180 unequal=0` ⇒「全等」；(3) 五个数同一个值 ⇒「两侧相等」。否则按 `region=` / 行 / 数列出 | **第 2 行**。(1) 任一区域不等或有 `unmatched` ⇒「哪一处不等」并停机 S26-crates；(2)(3) 不等 ⇒「哪一处不等」（F26i） |
| **Q142.148 取号写两轮后的另一条路径与窗口之前计数** | (1) 独立 bin 第一种调用 `<N26 主产物> <主产物里第一行 name=done 之后那一半>` 的 `name=independent_region_diff_summary`；(2) 两侧 `name=before_window_summary`（`side=model` 对 `crates/` 那一半）的 `operations=`、`writes=`、`last_five=`；(3) `crates/` 那一半 `name=impl_config` 的 `mkfs_operations=` 对装置 mkfs 行的 `operations=` | (1) `model_regions=29 crates_regions=29 equal=29 unequal=0` 且没有 `name=independent_region_unmatched` 行；(2) `operations=` 两侧都是 51、`writes=` 都是 35、`last_five=` 逐字相等；(3) 相等（25）。三项各报「相等 / 不等（两边的值）」 | **第 1、2 行**。(1) 与 Q142.147 (1) 一边全等一边不等 ⇒ 停机 S26-crates，两条路径都查；(2) `operations=` / `writes=` 不等 ⇒ 第 1 行记「与 `crates/` 对不上」并停机 S26-crates；`last_five=` 不等 ⇒ 窗口切点不在同一处，作废 V26e |
| **Q142.149 挂钟离开 exact 产物（R93）** | (1) N26 主产物 `grep -c 'seconds='`，另 `grep -c -i 'elapsed'`；(2) N26 `.err` 文件 `grep -c '^E7TIMING name=layer0_main_arm_elapsed seconds='`；(3) 产物里 `name=done_layer0_main` 恰 1 行 | (1) 两个计数都是 0；(2) 恰 1；(3) 恰 1 ⇒「挂钟已挪出」 | **第 4 行**。(1) ≥ 1 ⇒「还有挂钟」（F26k）；(2) 为 0 ⇒ 挂钟量丢了，作废 V26f |
| **Q142.150 层 0 主臂整轮（第 5 行）** | N26 主产物 `name=layer0`、`name=layer0_violations`、`name=verdict_layer0`、`name=verdict` 里 `layer0_states_ok`、`layer0_violations`；N26R 同样几行作旁证 | N26：`segments=` 是 `2+2+2+1+2+2+1+2+24+2+1+2`，`states=16777243`，`closed_form=16777243`，`exhaustive=true`；`layer0_violations` 的 `violations=0`；`verdict` 里 `layer0_states_ok=true`、`layer0_violations=0`（该字段按装置写法，值为「零违例」即成立）；`name=journal_effect` 的 `differing_states`、`name=layer0_outcome_matrix_summary` 的 `off_diagonal`、`name=journal_header_reader_branch_counts` 的 `total` 只报（N26R 同报），不设门槛 | **第 5 行**。`states` ≠ 16777243、`exhaustive=false`、`violations` ≥ 1 ⇒「不成立（哪一格）」（F26m）；不影响 J1，但不成立时第 3 行交用户时一并列出 |
| **Q142.151 守门（R98）** | `r26_guard.py <第 25 次主产物> <N26R 产物> drift`（第一段）；`r26_guard.py <N26R 产物> <N26 主产物> acq2`（第二段）；`r26_guard.py <第 25 次主产物> <N26 主产物> same`（总账，只打印，不判）。规则表见本表下面「守门规则表」；执行员逐组写归因表（`groups_to_attribute=` 列出的每一组都要归到 D、A、T 之一，连同装置行号和一句话说明那一行怎样产出这个字段） | 两档 `R26_VERDICT … pass=true`，且归因表里没有一组「未归因」⇒「守门过」 | **第 3、6 行**。`pass=false` ⇒ F26d；归因表有「未归因」⇒ F26e |
| **Q142.152 总账与 J1（第 3 行）** | 对第 25 次主产物，N26 的每个变了的组：来源（D、A、T 或「变了又变回，净零」）、归因表里的行号；加 Q142.147 (1) | 执行员列出 **J1** 的五个条件各自成立与否（本节末）；全部成立 ⇒「无归不了因的差异」；否则逐条列出归不了因的差异（组、第 25 次的值、N26R 的值、N26 的值、尝试过的归因） | **第 3 行**。J1 任一不成立 ⇒「有」，逐条交用户（F26e、F26c 或 F26g 的触发） |
| **Q142.153 第 2 行的几何敏感性：P8G** | `p8g_region_compare.py <主产物> <第 26 次 P8G 导出>` 的 `P8G_SUMMARY`；`label6_anchor.py --decode <主产物> p8g_device_region_bytes` 与 `--decode <P8G 导出>` 各两行；主产物 `name=p8g_layout_field_summary` | 三项各判：`matched=29 equal=29 unequal=0`、两个 `unmatched` 0；四行解码同一个值；`rows=180 unequal=0` | 第八节那一格。与主点一边全等一边不等 ⇒ F26g「不稳定」，所在那一行照主点答、另记这一格；P8G 的任一不等同样是一条差异，进 Q142.152 的归因 |
| **Q142.154 取号与整条流对条款（第 1 行）** | N26 主产物：(1) `name=segments path=instance_acquisition`；(2) `name=segments path=post_mkfs_stream`；(3) `name=segments_three_state path=instance_acquisition` 与 `post_mkfs_stream`；(4) `name=before_window_summary side=model`；(5) `name=instances`；(6) 窗口里系统配置槽两次写的 `name=device_region_bytes`（`region=system_configuration`）的 `offset=` 对应的槽（槽 0 在设备内偏移 0、槽 1 在 4096，`SYSTEM_CONFIGURATION_SLOT_OFFSETS` 第 129 行；D22 已定项 16「槽 i 的设备内偏移 = i × 固定结构槽距」） | 逐项对第七节：(1) `operations=6 segments=2+2 closed_form=7`、`kinds=[system_configuration_slot×2,barrier×2]\|[system_configuration_slot×2]`（K-Q1、A-Q1）；(2) `operations=61`、12 段、`closed_form=16777243`、种类串逐字等于 `layout/01` 第 450 行那一串（K-Q2、A-Q2）；(3) 取号路径 `writes=4`，`closed_form_two_state=7`，三态 17（A-Q8 同一式子：8 + 8 + 1）；整条流 `writes=43`、两态 16777243、三态 16777268（A-Q3、A-Q6）；(4) `operations=51`、`writes=35`（A-Q4）；(5) 取号两轮的世代号 2 与 3、新池新建文件 6 与槽 0（K-Q4、A-Q5）；(6) 窗口里那次写是槽 0（`offset=0`），不是槽 1。每项各报「相同 / 不同（装置值、条款值）」 | **第 1 行**。任一不同 ⇒ 第 1 行记「不同（哪一项）」，同时停机 S26-clause（条款值是独立算出并用命令核过的，先查装置）|
| **Q142.155 绝对值（第 1、2 行的旁证）** | 7.1 K-Q1–K-Q6 逐个对产物；字段表里与系统配置槽有关的行（若有）对条款 | 全符 ⇒「是条款的数」 | 第 1、2 行「相同 / 全等」的绝对值旁证：两侧相等而不是条款的数 ⇒ F26a「两侧一起错」，所在行照答、另记这一格 |
| **Q142.156 第 1 行的几何敏感性** | N26 产物：(1) `name=segments_geometry point=G1B path=instance_acquisition` 对 7.2 A-Q7；(2) `name=segments_sensitivity point=G11 … path=instance_acquisition` 与 `path=post_mkfs_stream` 对主几何的对应行 | (1) 四个字段等于 A-Q7（`operations=3 segments=1+1 closed_form=3`、`kinds=[system_configuration_slot×1,barrier×1]\|[system_configuration_slot×1]`）；(2) G11 点上的 `path=instance_acquisition` 四个字段与主几何那一行相等（发布末尾那道屏障不影响取号），`path=post_mkfs_stream` 的变化量按 `acq2` 档规则（比 N26R 多 4 操作、两态 +3、三态 +5） | 第八节那一格：(1) 不等 ⇒ F26g（几何上不稳定，A-Q7 是推的，先查推法），不让第 1 行本身翻面；(2) 不等 ⇒ F26d |
| **Q142.157 第 6 行三样（R89 的口径，扫法与守门按 C774 改）** | (a) `residual_r26.sh` 在 N26 的装置与变异表上；第 22 次命令六；第 23 次命令十二；第 22 次命令七里 `287` / `295` 逐处归类（`O_` 样式命中为 0 时不需要归类）；(b) Q142.151 的两档；(c) 整张变异表（第九节那条命令） | (a) `residual_r26 total=0`，**且**命令十二四个计数 0、0、1、0 ⇒ 成立；命令六四个计数另报；(b) 两档 `pass=true` ⇒ 成立；(c) 几栏之和 = 表的行数、收尾「已还原，基线仍全绿」、`✅` = 行数 ⇒ 全抓 | **第 6 行**。(a) `total` ≥ 1 ⇒「不成立」（F26j）；(b) 同 Q142.151；(c) `❌` 或 `⏭` ≥ 1 ⇒ 不成立（F26l）；`⚠️`、`⏱`、`🧱` ≥ 1 ⇒ 那一轮失败，重跑，不判 |
| **Q142.158 第二段前提：装置照条款补上了** | `residual_r26.sh` 的 `A_` 八个样式；N26 单测 | 八个 `A_` 样式的 `count` 都是 0，且单测 0 败 ⇒「补上了」 | 第 1、2 行的前提（R92）。有一个 ≥ 1 ⇒ F26j，第 1、2 行不够判 |
| **Q142.159 21 行回看清单（只报，J2）** | 对 Q142.152 里每个来源为 A 或 T 或 D 的组名，在 E142 页「判决」「影响的决策」以及 21 行分项的页上引文里 `grep` 该组的行名（如 `name=instances`、`name=segments`、`layer0`），列出命中页内的位置 | 只报，不判；给写回员逐行写回看时用 | 不对应翻面。**附带**：写回员需要，不参与 J1 |
| **Q142.160 变异（对 Q142.157 (c) 的补充，只报）** | 第九节小表的抓到 / 无效 / 没红三数 | 只报 | 附带，够判后不跑 |
| **Q142.162 `NOT_RUN`** | 装置的标准输出与它写出的每份产物（含 `.err`）里 `grep -c NOT_RUN` 合计（`.gz` 用 `zgrep -c`） | 合计 0；不满足的那一份按本次未跑报 | 不对应翻面。装置不调崩溃判段入口，预期 0 |
| **Q142.163 指纹** | 主产物落盘之后立即跑 `python3 scripts/gate/admission.py experiment <仓根> E142` | 退 77 ⇒「指纹对上」 | 不对应问题单的行。退 0 ⇒ 停机 S26-fingerprint |
| **Q142.164 第二趟逐字节（第 4 行后半）** | 第二段步 ⑤：同一份装置、同一份 `crates/`、同一台机器再跑一趟 `driver_e142`（装置那一条），stdout 与 N26 主产物各去掉 `E7INPUT` 起头的行后 `cmp` | **条件**：第一段交回的 T0 ≤ 7200 秒才跑；跑了 `cmp` 相同 ⇒「两趟相同」，不同 ⇒ 列第一处不同的行（F26k）。T0 > 7200 秒 ⇒ 这一格标「够判后未跑」，第 4 行后半由主 agent 经服务跑 `replay.sh` 的 `driver_e142` 那一趟（复跑表第 2662 行，kind `exact`，结果「字节一致」）补判 | **第 4 行**。两趟不同 ⇒ F26k |
| Q142.165 附带：P8G 导出与第 25 次同不同 | `cmp <第 25 次 P8G 导出> <第 26 次 P8G 导出>` | 只报「相同 / 不同（`cmp` 第一处偏移）」，不判 | **附带**：不对应问题单的行；够判后不另跑 |

**守门规则表（Q142.151，写死；`r26_guard.py` 照它写，规则之外的一律不许，宁严勿宽）**。归一化同 `r25_guard.py` 文件头（第 22 次 R71）：取第一行 `E7RESULT name=done ` 之前、删 `E7INPUT ` 起头的行与空行，`hexadecimal=` 换成 `hexadecimal_sha256=`，剔除 `impl_bytes_equal`、`impl_bytes_unmatched`、`impl_bytes_equal_summary`、`window_segments side=crates`、`window_segments_summary`、`done`、`self_check_text_number_unclassified`（只在两份每一行的 `row=` 之后都含 `name=impl_bytes_equal` 时）。之后按行组（`name=` 的值）分组、组内按出现次序配对（`layout_field` 一类按 (id, record_or_unit, field) 配对、`journal_header_write_digest` 去掉 `index=` 之后按序列比）。

| 档 | 组 | 许可的形态 | 来源标签 |
|---|---|---|---|
| `drift`（第 25 次主产物 → N26R） | `layer0`、`journal_effect`、`journal_header_reader_branch_counts`（旧产物是 `skipped=true` 一行，新产物是有值的一行） | 各恰一行变；新行的字段值按 Q142.150 在 N26R 上读（`states=16777240`、`exhaustive=true`、`violations=0`） | D（层 0 不再被环境变量挡住） |
| | `layer0_violations`、`layer0_violations_by_publish`、`layer0_outcome_matrix`（0 行或多行）、`layer0_outcome_matrix_summary`、`verdict_layer0`、`layer0_main_arm_elapsed`、`done_layer0_main` | 新产物多出来的组（旧产物没有）；`layer0_main_arm_elapsed` 的 `seconds=`、`threads=` 不比 | D |
| | `verdict` | 一行变；只许 `layer0_states_ok`、`layer0_violations`、`journal_differing_states` 三个字段从 `not_run` 变成有值，其余字段逐字相同 | D |
| | `feature_bits` | 一行变；只许少一个字段 `refuses_retired_bit0`，别的字段逐字相同 | D（feature bits 位 0 退役） |
| | `gap`（只 `id=G12`） | 一行变，只变 `text=` | D |
| | `p8g_verdict`、`p8g_control_main_point` | 一行变，只许 `fields=`、`equal=`、`unequal=`、`unequal_fields=` 与上面 `verdict` 三个字段对应的值变；**预期组**：新产物里 `unequal=0` 且 `equal=` = `fields=`（第 25 次守门输在这一格上，C774） | D（回落是层 0 字段有了值） |
| | `self_check_*`、`name=verdict` 里 `text_numbers_all_classified` | 按 `r25_guard.py` 现有处理 | D |
| | 其余每一组 | 零差异（含全部 `device_region_bytes`、`write_list*`、`layout_field*`、`accounting`、`root_record`、`back_chain`、`width`、`segments*`、`before_window_summary`、`journal_header_write_digest*`、`instances`、`positive_control_p2`） | — |
| `acq2`（N26R → N26） | `segments` | 恰两行变（`path=instance_acquisition`、`path=post_mkfs_stream`），只许 `operations`、`segments`、`closed_form`、`kinds` 四列；值按 Q142.154 (1)(2)；`path=mkfs`、`warm_up`、`transaction` 三行零差异 | A |
| | `segments_three_state` | 恰两行变（同两条路径），只许 `writes`、`closed_form_two_state`、`closed_form_three_state` 三列；值按 Q142.154 (3) | A |
| | `segments_sensitivity` | 变的行只许是 `path=instance_acquisition` 与 `path=post_mkfs_stream`；G11 点上取号路径与主几何那一行相等；`path=post_mkfs_stream` 的变化 = 多 4 操作、两态 +3、三态 +5；别的点（G10a、G10b、G12a、G12b 只打 `transaction` 与 `post_mkfs_stream`）里 `post_mkfs_stream` 同样按这个增量 | A |
| | `segments_geometry` | 新多一行 `path=instance_acquisition`（A-Q7）；mkfs 那一行零差异 | A |
| | `before_window_summary side=model` | 一行变，只许 `operations`（47 → 51）、`writes`（33 → 35）；`last_five=` 零差异 | A |
| | `journal_header_write_digest` | 去掉 `index=` 之后：新序列 = 旧序列插入 2 行（`kind=system_configuration_slot`、设备 0 与 1、槽 1 的偏移、世代号 3 的内容），并且旧序列里 6 行 `kind=system_configuration_slot` 的写（暖机两次、新池新建文件各两盘）只变 `offset=`（落点翻槽）与 `sha256=`（内容里世代号 +1）；其余行逐字相同；`index=` 单调递增 | A |
| | `journal_header_write_digest_summary` | 一行变，`writes=` 恰 + 2 | A |
| | `instances` | 一行变；字段按 A③ 登记（拆成两个取号世代号，`transaction_*` 变成 6 与槽 0）；别的字段零差异 | A |
| | `gap`（只 `id=G21`） | 一行变，只变 `text=` | A |
| | `device_region_bytes`、`write_list_row`、`p8g_device_region_bytes`、`p8g_write_list_row`、`r18_clause_fields` | 变的行只许是窗口里系统配置槽那次写的两盘各一行（`region=system_configuration`）；只变 `offset=`、`sha256=` / `hexadecimal_sha256=` 与同一行里的世代号字段；其余区域行零差异 | A |
| | `layer0`、`layer0_violations_by_publish`、`journal_effect`、`layer0_outcome_matrix*` | 各恰一行（矩阵按键）变；`layer0` 的 `segments=`、`states=`、`closed_form=`、`states_by_publish=` 变；`exhaustive` 在两份里都必须 `true`、`layer0_violations` 的 `violations=` 都必须 0 | A |
| | `layer0_main_arm_elapsed` | N26R 有、N26 没有（stdout 里）；N26 的 `.err` 里有 | T |
| | `p8g_verdict`、`p8g_control_main_point` | 同 `drift` 档的预期组：`unequal=0`、`equal=` = `fields=` | A |
| | `self_check_*`、`verdict` 里 `text_numbers_all_classified` | 按 `r25_guard.py` 现有处理；`verdict` 其余字段零差异（`layer0_*` 在两份里都必须是「成立」的值） | T |
| | 其余每一组 | 零差异（含 `accounting`、`root_record`、`back_chain`、`width`、`layout_field*`、`positive_control_p2`、`feature_bits`、`config`、`p8g_` 里不涉及系统配置槽的行、`gap` 里除 G21） | — |

规则里没列的组在新产物里变了 ⇒ `groups_bad`（不是 `groups_to_attribute`）；规则里列了但形态不符（多变一行、变了表外的列、值不对）⇒ `groups_bad`。`groups_to_attribute` 只列规则明说要逐行归因的组（`drift` 的 `layer0` 一族、`journal_header_write_digest`、`p8g_control_main_point`、`segments_sensitivity`）。**这两处是 C774 点名的改正**：(1) `p8g_control_main_point` 在两档里都写成「预期组」，不再放进零差异清单；(2) 旧值扫法的改正在 `residual_r26.sh`（删变异名再匹配），不在守门。

### 6.1 J1：用户要的那条判据（写死）

**J1「29 个区域重新全等就把 21 行写不受影响，有归不了因的差异逐条交用户」** 的条件，五条全部成立才写：

1. **29 个区域全等**：Q142.147 (1) 与 Q142.148 (1) 都是「全等」（主点 `matched=29 equal=29 unequal=0`、两个 `unmatched` 0；独立路径同）。
2. **没有归不了因的差异**：Q142.151 两档都 `pass=true`，Q142.152 的归因表里没有一组「未归因」；Q142.147 (2)(3)、Q142.153 若有不等，每一处也要归得了因，归不了因的算差异。
3. **没有作废、没有停机**：没有打中 V26*、S26-*；前提 Q142.158 过。
4. **阳性对照全过**：第 5.2 节每一条「该看到什么」都看到。
5. **条件格交代清楚**：Q142.164 跑了且相同，或者标明「够判后未跑」并给出 T0。

**五条全成立** ⇒ 写回员在 R96 列的 21 行的「回看」格各写 R97 那句话（不受影响），不另判每一行；**有一条不成立** ⇒ 21 行一行都不写「不受影响」，执行员把每一处差异逐条（组、第 25 次的值、N26R 的值、N26 的值、尝试过的归因、对应 Q 号）列进报告交主 agent 转用户；Q142.159 列出哪些行引到了这些组，帮用户逐条看。**J1 不包括层 0 一格（Q142.150）**：它读数不成立也不改 J1，但第 3 行交用户时一并列出，因为 21 行里的 D16 已定项 8 / 9、D13 已定项 4、D23 已定项 14 引过层 0 的读数。

## 七、钉绝对值的断言

### 7.1 出自被测条款本身的（不符 ⇒ F26a「条款可能错」，两边都查，不作废）

| # | 值 | 出处 |
|---|---|---|
| K-Q1 | 取号路径：段序列 `2+2`、6 次操作、7 个崩溃状态、种类 `[system_configuration_slot×2,barrier×2]\|[system_configuration_slot×2]` | `layout/01` 第 441 行 |
| K-Q2 | 整条流（mkfs 之后）：`2+2+2+1+2+2+1+2+24+2+1+2`、61 次操作、两态闭式 16777243，种类串逐字等于 `layout/01` 第 450 行那一串 | `layout/01` 第 436、450 行 |
| K-Q3 | 系统配置槽写取三态的全量 16777268 | `layout/01` 第 450 行 |
| K-Q4 | 世代号：mkfs 1（两槽同写）、取号 2、3，暖机 4、5，新池新建文件发布后 6；第 r 次写的槽 = 世代号 mod 2；取号两轮的实例代号都写 1 | `layout/01` 第 62、119、399 行；D22 已定项 16 |
| K-Q5 | 窗口里那次系统配置槽写：槽 0、世代号 6；末态两盘槽 0 世代 6、槽 1 世代 5（`layout/01` 第 38 行：`root_slots_system_configurations_and_journal_ring_hold_the_published_state` 钉的「两盘槽 0 世代号 6、槽 1 世代号 5」） | `layout/01` 第 38、399 行 |
| K-Q6 | mkfs 行不变：`operations=25 segments=14+1+1+1+4 closed_form=16402` | 第 25 次登记 7.1 K1；`layout/01` 八「mkfs 种根」（`python3 scripts/crash/check-segment-registry.py --print-mkfs-segments-line` 现取，P4 的命令） |

### 7.2 独立算出、用命令核过的（不符 ⇒ 作废 V26b）

`anchors_r26.py`（不读装置、不读 `crates/`，只用条款里的每段 2^k − 1 加终态 1 的闭式；设计时已跑，输出整份在 13.3，sha256 `c5fac27110827db9858bfff99a8c78edc5962be931231b899af707c4b9f273ea`）：

| # | 值 | 算式 |
|---|---|---|
| A-Q0 | mkfs `14+1+1+1+4` 两态 16402（公式的校验：与条款一致） | Σ(2^k − 1) + 1 |
| A-Q1 | 取号 `2+2`：两态 7，6 次操作（4 写 + 第一轮之后一道池屏障按设备记 2 步） | 3 + 3 + 1 |
| A-Q2 | 整条流 12 段：两态 16777243，61 次操作 = 取号 6 + 暖机 20 + 新池新建文件 35 | 8×3 + 3×1 + (2^24 − 1) + 1（8 个 2 写段、3 个 1 写段、1 个 24 写段） |
| A-Q3 | 整条流三态（系统配置槽的段取 3^k − 1）：16777268 | 脚本 |
| A-Q4 | 窗口之前 (operations, writes) = (51, 35)；N26R 的 (47, 33) | 25 + 6 + 20；21 + 4 + 10 |
| A-Q5 | 世代号与槽：(1,1) (2,0) (3,1) (4,0) (5,1) (6,0) | 世代号 mod 2 |
| A-Q6 | 整条流写数 43，屏障 18 步（9 道池屏障 × 2 盘），43 + 18 = 61 | 脚本 |
| A-Q7 | G1B 一块盘：取号 `1+1`、两态 3、3 次操作（2 写 + 1 道屏障） | **推的**，条款只给了两盘；不符先查推法（F26g），不作废 |
| A-Q8 | N26R（取号一轮）：取号 `2`，两态 4、三态 9、2 次操作；整条流 11 段两态 16777240、三态 16777260、59 次操作；窗口之前 (47, 33) | 脚本 |

A-Q7 的分类：它是推的、条款没给一块盘的值，所以它**不属于**独立算出并核过的那一类；它出现在 7.2 只因为同一个脚本算它；对不上走 F26g，不走 V26b。

## 八、轨迹与几何敏感性

### 8.1 轨迹

这是确定性的单趟模型，没有「轮」。被谓词消费的量是计数：区域 `equal` / `unequal` / `unmatched` 数、字段表 `rows` / `unequal`、层 0 的 `states` 与 `violations`、守门的 `changed_lines_total`。各报「峰值 / 为正轮数 / 期末值」不适用（没有多轮）；对应的轨迹量是**层 0 的 `states_by_publish` 五桶**（`txg1`、`txg2`、`txg3`、`after_last_root`、`every_write_persisted`），N26R 与 N26 各整行抄进报告，不设门槛。

### 8.2 几何敏感性（`mutation-sampling.md` 第六类）

- **两块 4 GiB 盘（主点）与两块 8 GiB 盘（P8G）**：Q142.153，同一批判据（29 区域、180 行字段表、标签 6）。取号写两轮不依赖盘容量，P8G 的区域比对主要盯字节随盘容量变的那几处；预期同主点。
- **一块盘带屏障（G1B）——方向相反的取样点**：Q142.156 (1)，取号路径每轮只有一盘一写，A-Q7 推 `1+1`、3 次操作；这是两块盘（每轮两写）的另一侧，若装置的取号循环把「逐盘一写」写成了「固定两写」，在这一点上才暴露（M249 在这一点上改变输出）。
- **发布末尾不放那道屏障（G11）**：Q142.156 (2)，取号路径应与主几何相等；`post_mkfs_stream` 的变化量应与主几何相同。
- **取样点的覆盖**：第 1 行的取号形状在两盘（主点、P8G）与一盘（G1B）三个点上各量；第 2 行的区域比对只在两盘两个容量上量（`crates/` 只造两块盘，`make_filesystem` 开头断言两块盘），一盘几何上的逐字节比同第 25 次不在射程，实验页记「够判后未跑」。

## 九、变异

**整张表**：改完之后的那一张（221 行加 6，重锚的不改条数）。锚点不再恰好命中一次的（第七类）由执行员重锚，每条写「旧锚点 → 新锚点、语义为什么没变」；替换编不过的（第八类）换一条会改行为的；没红的按「没红的三类，判据不同」三分。独立 bin 的变异表不动、不重跑。

**新加的六条**（每条都写在哪个取样点上改变输出、哪条单测抓它）：

| 名 | 改什么 | 在哪个取样点上改变输出 | 抓它的单测 |
|---|---|---|---|
| M246_acquisition_writes_only_one_round | 删掉 A① 的第二轮 | 主点：取号路径 `2`、2 次操作；整条流 11 段 59 次操作 | A⑥ (i) 与 Q142.154 (1)(2) 的钉值 |
| M247_acquisition_second_round_repeats_generation_two | 第二轮的世代号也写 2 | 主点：`name=instances` 的第二轮世代号 2；取号第二轮写出的槽字节变；其后各次世代号阶梯断了 | A⑥ (ii) 世代号阶梯 |
| M248_publish_generation_ignores_second_round | `system_configuration_write_for_publish` 回到 txg + 2 | 主点：窗口里系统配置槽那次写变成世代 5、槽 1：`name=instances` 的 `transaction_*`、窗口里 `region=system_configuration` 的 `offset=` 与 `sha256=`；与 `crates/` 比有区域不等 | A⑥ (ii) 与 Q142.147 |
| M249_acquisition_loops_over_the_pool_not_per_device | 取号每轮写成固定两次（不随盘数） | **主点上与原式同值**（两块盘）；敏感点是 G1B（一块盘）：`path=instance_acquisition` 4 次操作 | A⑥ (iii) 的 G1B 钉值（A-Q7） |
| M250_barrier_between_acquisition_rounds_dropped | 去掉第一轮之后那道池屏障 | 主点：取号路径 `4`（一段）、4 次操作；整条流 11 段 | A⑥ (i) |
| M251_layer0_timing_line_back_on_stdout | B① 的 `eprintln!` 改回 `emit`（`E7RESULT` 前缀） | 主点产物：多一行含 `seconds=` | B② 的前缀单测与 Q142.149 (1) |

**等价的那一种，写明**：第二轮的 `journal_instance` 写成与第一轮不同的值，在 `crates/` 只比到的窗口里和整条流的字节上有可观测差别（槽字节），不等价，不进「等价」；把取号第一轮和第二轮的槽互换（第一轮槽 1 世代 2、第二轮槽 0 世代 3）会让槽 = 世代号 mod 2 断掉，不是等价，而是 K-Q4 的反例，归 M247 一类（由 A⑥ (ii) 抓）。无等价变异需要留档。

**要主 agent 跑的命令（整张变异表，Q142.157 (c)）**：执行员在第二段步 ⑦ 把下面这条原样写进报告，主 agent 经任务服务起；放在第二段主产物之后跑（装置源码在两者之间不许被别人改，`mutate.sh` 退 5 的那一轮不算数）。工作目录 `/home/fy5090/code/singlefs/scripts/experiment-apparatus`（`mutate.sh` 在 `scripts/mutation/` 下，参数是 bin 名、源文件、变异表；执行员核一次与 `mutate.sh` 文件头的用法一致，不一致照文件头改并写进报告）：

```
cd /home/fy5090/code/singlefs/scripts/experiment-apparatus && MUTATE_JOBS=4 MUTATE_TARGET_DIR=<执行员草稿目录>/mtarget bash ../mutation/mutate.sh e142-new-pool-file-creation-dry-run e142_new_pool_file_creation_dry_run.rs ../mutation/tables/e142_new_pool_file_creation_dry_run.tsv > ../../research/results/e142-mutate-r26-<日期>.log 2>&1
```

跑完看（仓根）：`grep -E '^(✅|⏭|❌|💥|⚠️|⏱|🧱)' research/results/e142-mutate-r26-<日期>.log | awk '{print $1}' | sort | uniq -c`（几栏之和等于表的行数）、收尾的「计数：」行、末行「已还原，基线仍全绿」；没有那一行或退出码是 5 ⇒ 这一轮不算数。第二段步 ② 的子集用同一条命令，表换成草稿目录里只含 M246–M251 与重锚的几行的一份，日志名换成 `-r26-s2-`。

## 十、失败条款

每条都是正当结果，照实记，不作废；每条后面紧跟「什么观测会让它触发」。F26b、F26f、F26h 这一次没有对象，号空着不用。

| # | 条款 | 什么观测会让它触发 | 触发之后 |
|---|---|---|---|
| F26a | 条款可能错：装置照条款写，写出的数与 7.1 不同；或两侧一起偏离条款的数 | Q142.155 有一格不符而 Q142.147 (1) 全等（两侧一起错）；Q142.154 不同而装置的取号循环确实是两轮（A⑥ (i) 绿）；7.1 K-Q2 与 7.2 A-Q2 不符 | 两边都查：写明条款原文哪一行、装置哪一行、`crates/` 的测试钉在哪（3.1 第 2 条，这时执行员可以读 3.1 点名的 `crates/` 测试文件，只读）；装置不为了相等改，查出装置错的记下、交主 agent，改了就是又一次跑 |
| F26c | 取号那一半或漂移那一半碰了不该碰的区域 | Q142.151 `acq2` 档 `region` 类组里变的行不是窗口里系统配置槽那两盘各一行（Q142.147 (1) 里 `region=` 不是 `system_configuration` 的区域不等或未配上） | 照 `evidence-discipline.md` 三步：先按最弱读法记一次输（点名哪几个区域），第 1–3 行不够判，停下交回 |
| F26d | 守门打中：改动之外有别的行变了，或变的行、列不是登记的形态 | Q142.151 任一档 `R26_VERDICT … pass=false`（`groups_bad=` 点名的组）；Q142.156 (2) 不等 | 同 F26c 三步：按最弱读法记一次输（「臂 N26R / N26 的『做完会怎样』在登记的形态下不成立」），照实列 `R26_GROUP` 各行；那一档对应的行不够判。**不许**把形态放宽后在这一份产物上重判：要放宽就是又一次跑 |
| F26e | 有归不了因的差异组 | Q142.151 的归因表里有一组记「未归因」；J1 第 2 条不成立 | 照实列那一组的行名、变几行、第一行与最后一行（归一化之后）、两份产物的值；第 3 行记「有归不了因的差异」，21 行一行都不写「不受影响」，逐条交用户 |
| F26g | 几何上不稳定 | Q142.156 (1) G1B 那一行有一个字段不等于 A-Q7；Q142.153 与主点一边全等一边不等 | 记「不稳定」，两边都查（A-Q7 是推的，先查推法）；所在那一行照主点答，另记这一格；P8G 的不等同时进 Q142.152 的归因 |
| F26i | 字段表或标签 6 两侧不等 | Q142.147 (2) `unequal` ≥ 1 或 `rows` ≠ 180；(3) 五个数不是同一个值 | 列出那几行（期望、实际、`layout_line`）；第 2 行照答「哪一处不等」；该处进 Q142.152 的归因 |
| F26j | 装置没照条款补全，或还剩旧值 | Q142.158 有一个 `A_` 样式 ≥ 1；Q142.157 (a) `residual_r26 total` ≥ 1，或命令十二前两个计数有一个 ≥ 1 | Q142.158 的：第 1、2 行不够判，停下交回（不在这一份产物上补改）；Q142.157 (a) 的：第 6 行 (a) 照答「不成立」 |
| F26k | 挂钟还在 exact 产物里，或两趟不相同 | Q142.149 (1) 任一计数 ≥ 1；Q142.164 跑了而 `cmp` 不同 | 列出含挂钟的行 / 第一处不同的行；第 4 行照答「有 / 不同」；两趟不同时不许改比对去抹它（不加 `strip_timing` 字段），交主 agent |
| F26l | 变异没抓全 | Q142.157 (c) `❌` 或 `⏭` ≥ 1；第二段小表里有一条没红或无效 | 逐条按「没红的三类，判据不同」三分；第 6 行 (c) 照答「不成立」 |
| F26m | 层 0 整轮不成立 | Q142.150 `states` ≠ 16777243、`exhaustive=false`、`violations` ≥ 1 | 列出哪一格；第 5 行照答「不成立（哪一格）」；J1 不因它改判，第 3 行交用户时一并列出 |

## 十一、作废条款与停机条款

**重跑登记的比对对象（这一次要与之比的上一次产物；门禁 doc-experiments 的 replay-comparison-units 格读这张表）**：

| # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |
|---|---|---|---|---|---|---|
| 1 | `e142-new-pool-file-creation-dry-run-r25-main-2026-10-13.out` | `a64b55e9c6acb46a57c92e8ef486a065e2cb9cb4a07a40c8e3b233886d96f2bb` | 1213 | `scripts/experiment-apparatus/replay.sh` 第 2662 行 `E142\|@driver_e142\|\|e142-new-pool-file-creation-dry-run-r25-main-2026-10-13.out\|exact`；`driver_e142` 两条命令（导出 bin，再装置） | `e142-new-pool-file-creation-dry-run-r26-main-2026-11-10.out` | 取号写一轮时的产物，在版本库历史里（`git show 841b3beabb98:research/results/<名>`）；跑的日子不是 2026-11-10 时产物名里的日期照实际日期，机读登记与本表跟着改，不算判据变更 |
| 2 | `e142-r25-p8g-crates-dump-2026-10-13.out` | `6cf0735e387114072ec4a4c86c847fca6ee185b4b2087e7d344dea70e61fe6ea` | 39 | `scripts/experiment-apparatus/replay.sh` 第 2663 行 `E142\|@driver_e142_p8g_crates_dump\|\|e142-r25-p8g-crates-dump-2026-10-13.out\|exact`；`driver_e142_p8g_crates_dump` | `e142-r26-p8g-crates-dump-2026-11-10.out` | 同上；`crates/` 一侧的导出，装置不参与 |

**起跑前提（每一段的步 ⓪ 逐条核，有一条不齐 ⇒ 停机，那一段一步都不跑）**：

| # | 前提 | 怎么核 | 不齐 ⇒ |
|---|---|---|---|
| P1 | 装置、表、参照产物是登记时那一份 | 装置 `f1fb548d…317a` 11066 行；变异表 `64caabe9…c679` 221 行；独立 bin `2f41d6e5…49fb7`；臂 N15 参照 `research/results/e142-new-pool-file-creation-dry-run-2026-09-25-r16-arm-n15.out` `f0e1515a…126d`；第 25 次主产物与 P8G 导出的 sha256 与行数同第十一节表。第二段：装置与变异表是第一段交回时记的（第一段不改它们，所以与第一段相同） | S26-sha |
| P2 | 条款没在设计之后变 | 照第十三节命令六，用同一条 `quote-kb.py` 命令重抄第二节，sha256 必须是 `7ab1673edb865507eb4fc2cea908eb9fb5bc7175e685e7fca1d412c52d302495`；不同就 `diff` 两份出口、原样进报告 | S26-clause |
| P3 | `crates/` 里取号两轮还在、导出 bin 还在 | `grep -c 'const ACQUISITION_WRITE_ROUNDS: \[AcquisitionWriteRound; 2\]' crates/singlefs-core/src/transaction.rs` 为 1；`ls crates/singlefs-verification/src/bin/e142_new_pool_file_creation_write_dump.rs` 在 | S26-open |
| P4 | mkfs 登记行没变 | `python3 scripts/crash/check-segment-registry.py --print-mkfs-segments-line` 退 0、恰一行，且含 `operations=25`、`segments=14+1+1+1+4`、`closed_form=16402`（K-Q6）。**设计员没有跑它**（子 agent 不起项目脚本），命令交主 agent 或由执行员的派发提示附上 | S26-clause |

**准入（写死）**：装置 `main()` 开头的准入调用，键 `E142`；stderr 原样存进执行员草稿目录、贴进报告；退 0 才往下跑那一次；退 3、2 ⇒ 停机 S26-admission；退 77 ⇒ 这一份输入已有一份带同一指纹的产物，停机 S26-admission 交回（N26R 若退 77，说明有人已按今天的装置出过产物，交主 agent 定用不用那一份）。不设 `SINGLEFS_EXPERIMENT_RERUN_REASON`、不设 `ADMISSION_BREAK`。内存包装报 245、250–254 的，照它自己打的出路改，不绕开包装；那一次的输出不算产物。文件名里的日期取跑的那一天（`TZ=Asia/Tokyo date +%F`）；已存在的同名文件不覆盖，按日期另存。

### 作废（这一次不出结论，产物改名 `-void.out` 留在执行员草稿目录、不入库，修好重跑；已经入库的上一次产物照留）

V26a 号空着不用（本次没有「同一份装置与同一份 `crates/` 跑出不同装置那一侧」作废：两趟不同是 Q142.164 / F26k 的正当结果，不作废）。

| # | 条件 | 什么观测会让它触发 |
|---|---|---|
| V26b | 锚点的输入或算法坏了 | PC-a 的输出 sha256 不是登记的值；7.2 A-Q0–A-Q6、A-Q8 与产物无关地自己不符（脚本输出与 13.3 登记的整份不同）；P4 的命令退非 0 或不是恰一行 |
| V26c | 跑的途中 `crates/` 变了 | 同一段里快照 A 与 B、或 B 与 C `cmp` 不同 |
| V26d | 阳性对照不过：测量分不出差别 | 第 5.2 节任一条没看到「该看到什么」 |
| V26e | 产物缺行或配不上 | 产物里 `name=segments path=instance_acquisition` 或 `path=post_mkfs_stream` 不是恰一行（第一行 `name=done` 之前）；N26 的 `segments_geometry … path=instance_acquisition` 不是恰一行；`impl_bytes_equal_summary`、`layout_field_summary`、`accounting`、`root_record`、`verdict`、`instances`、`layer0` 任一不是恰一行；`label6_anchor.py --decode` 主产物不是恰两行装置一侧、两行 `crates/` 一侧；Q142.148 (2) `last_five=` 不等 |
| V26f | 读数读不到，或内存包装自己的结局 | 任一格的量读出来是空的（读不到 ≠ 读到 0，脚本少打一行也算，`.err` 文件不在或没有 `E7TIMING` 行也算）；或经 `scripts/hosts/run-with-memory-cap.sh` 跑出退出码 245、250–254 |

### 停机（既不作废，也不当结果；两边都查，写进报告交回主 agent）

| # | 条件 | 什么观测会让它触发 | 怎么办 |
|---|---|---|---|
| S26-open | 开跑条件不齐 | P3 不过；第二段 A2 与第一段 C1 `cmp` 不同且变的文件含取号、mkfs、format 常量、记账或导出 bin | 一步都不跑，交回缺哪一样 |
| S26-sha | 装置、表、参照产物或草稿脚本不是登记时那一份 | P1 有一个不等；草稿脚本照 13.3 全文重建两次 sha256 仍不等 | 不改源码、不跑；交回现值 |
| S26-admission | 准入不放行 | 装置二进制退 3 或 2；该放行的那几次（N26R、N26 主产物）退 77 | 原样交回 stderr；不设强制与破闸变量，不改登记表 |
| S26-clause | 条款或登记行在设计之后变了，或装置与条款对不上 | P2 或 P4 不过；Q142.154 任一项不同 | 停在那一步；交回 `diff` 或不符的项；臂的定义要不要改由主 agent 按 `evidence-discipline.md` 三步定 |
| S26-scope | 单测不绿、或要改别处才做得成 | 参照臂的单测有败；改完的臂单测有败而改法照登记写不绿；或为过编译要动 5.1 之外的任何一处 | 停在那一步；交回哪一条单测、为什么；那一段的行不够判 |
| S26-dump | 导出 bin 要改才跑得了，或被重型闸拒 | `driver_e142` 第一条命令或 `driver_e142_p8g_crates_dump` 编不过、退非 0，或被 `.claude/hooks/heavy-test-guard.sh` 拒 | 不改 `crates/`、不换写法绕过；交回原样输出 |
| S26-crates | 装置与 `crates/` 对不上（`implementation-first.md` 第 4 条） | Q142.147 (1) 有不同区域，或 Q142.147 (1) 与 Q142.148 (1) 两条路径不一致；Q142.148 (2) `operations=` / `writes=` 不等；Q142.147 (3) 两侧不等 | 两边都查：写明条款原文哪一行、装置哪一行、`crates/` 哪个文件哪一行（这时执行员可以读 3.1 点名的 `crates/` 文件，只读、不改）；不为了相等改装置、锚点或登记行；那一行照答「不同 / 哪一处不等」 |
| S26-fingerprint | 产物头的指纹已不是今天的输入 | Q142.163 准入不退 77 | 原样交回；列跑的途中谁改了哪份输入（`git status`、准入 stderr）；重不重跑由主 agent 定 |
| S26-time | 层 0 整轮单趟挂钟超过两小时，登记里没有分片与续跑入口 | 第一段交回的 T0 > 7200 秒 | 第二段整段不跑；交回 T0 与 `enumerate_layer0_grouped` 没有分片 / 续跑入口这一条；要不要给装置加环境变量门（把层 0 挡回去，等于改臂）或加分片，由主 agent 定 |

### 交回之前（每一段交回都做）

1. 那一段的产物里 `name=segments`（五行）、`name=segments_three_state`、`name=segments_sensitivity`（G11 的取号与整条流两行）、`name=segments_geometry`、`name=before_window_summary`（两侧）、`name=impl_config`、`name=instances`、`name=journal_header_write_digest_summary`、`name=impl_bytes_equal_summary`、`name=root_record`、`name=accounting`、`name=layout_field_summary`、`name=layout_field_positive_control`、四行 `name=positive_control_p2`、第一行 `name=verdict`、`name=layer0`、`name=layer0_violations`、`name=verdict_layer0`、`name=p8g_verdict`、`name=p8g_control_main_point`、`name=p8g_layout_field_summary`；`label6_anchor.py` 的全部 `DECODE` 行；`p8g_region_compare.py` 的 `P8G_SUMMARY`；独立 bin 的两种汇总行；`r26_guard.py` 的 `R26 `、`R26_GROUP`、`R26_VERDICT` 各行；`residual_r26.sh` 的各 `residual_r26` 行；快照 `cmp` 的结果；`E7TIMING` 行；`grep -c NOT_RUN` 的合计——原样整行抄进报告（`evidence-discipline.md`「引产物就整行抄」）；归因表、翻面表、归类表放进报告文件。
2. 问题单逐行写「已够判 / 还差什么 / 剩下的量能不能让它翻面」。问题单文件执行员不改，状态由主 agent 改。
3. **交主 agent 的清单**（执行员不改 kb 与 `crates/`）：
   - **够判之后要主 agent 改的 kb 句子**：`.claude/kb/layout/01-first-txn.md` 第 436 行（「第 36、39 行仍是取号写一轮的 `2`……（E142 装置没按取号写两轮重跑）」那一句、「表里取 E142 产物的各行，对的是第二十五次跑的主产物」那一句）、第 441 行出处列里「产物 `name=segments path=instance_acquisition`（`…r16-arm-n15.out`）是取号写一轮时的」那一句、第 450 行「产物 `name=segments_three_state path=post_mkfs_stream` 那一行是取号写一轮时的 16777260」那一句、第 38 行「那一份产物是取号写一轮时的历史产物」那一句；`checks-owed.md` C774 与 C1001 ① 两行；`.claude/kb/experiments/142-新池新建文件的干跑.md` 加第二十六次跑一节，「它答不了的」第 9 条（`crates/` 里的流取号写两轮、装置不枚举）改成现值；D22 已定项 16 依据里「按 21 个区域比对全等」那一句核一遍。
   - **写回员的活**：J1 成立 ⇒ 21 行（第二节末的清单）的「回看」格各写 R97 那句话，带产物名；J1 不成立 ⇒ 一行都不写，逐条把差异交用户。备料 17 行、不影响 4 行这一次不回看。
   - 实验页、实验索引与变更史照执行员定义写；三语页由写回员同步。
4. `scripts/experiment-apparatus/replay.sh` 第 2662、2663 行与 `.claude/gate.d/replay-comparison-units.tsv` 两行（第二段步 ⑥，带条件）。

### 够判停机

问题单 6 行都够判（第六节）就停。Q142.159、Q142.160、Q142.165 只报不判；第五节 5.1 末段「没改的」那几样不在这一次跑，实验页记「够判后未跑」。第一段交回之后主 agent 判第二段不续的（打中 F26e 且第 3 行已能交用户、或 S26-time），第二段整段不跑，记「够判后未跑」。

## 十二、修订

留空；装置写之后、产物之前由执行员写，只许收严或补臂。第二段必须在这里记：改完的装置 sha256 与行数、变异表 sha256 与行数、`r26_guard.py` 的 sha256 与行数、重锚清单、小表三数、单测条数与结果。

## 十三、读过的文件与跑过的命令

### 13.1 读过的文件（行区间；grep 命中行也算读过）

- `.claude/agent-common.md`（开头约 150 行与规则怎么读一节）、`.claude/agents/experiment-designer.md`（已在派发里给出）。规则类文件按定义点名读：`test-discipline.md`、`three-way-inference.md`、`evidence-discipline.md`、`mutation-sampling.md`、`implementation-first.md`、`test-script-discipline.md`（只对点名的小节依各自的判据措辞使用；没有读出已有结论）。
- `.claude/kb/checks-owed.md`：`grep -n 'C1001'`（第 640 行，整行）、`grep -n 'C774'`（第 477 行，整行）。`/home/fy5090/.local/share/singlefs/work/older-debts-fact-table/result.md` 第 24 行整行。
- `.claude/kb/experiments/142-新池新建文件的干跑.md`：第 1 行（标题）、`grep -n '^#'` 的小节标题、第 323–369 行（「影响的决策」整表，按关系列数 21）、`grep -n 'seconds=\|第二十五次\|影响的决策'` 的命中行（第 33、587 行等命中行）。**没有读**其余小节的内容（判决、结论、历史版本）。
- `.claude/kb/experiments.md` 与 `.claude/kb/experiments-history.md` 里 `grep -rn '取号写两轮'` 的命中行（第 185 行、第 4647、5330、5332、5960、5981、5986 行，E161、E183、E184 的行，别的实验的行；读到了「取号写两轮之后新池新建文件流 12 段、43 次写」「16777243、16777268」这类别的实验页上的数，已在第四节第 2 条与 7.2 里作为旁证列出，**没有**读 E142 的结果行）。
- `.claude/kb/layout/01-first-txn.md` 第 38、62、85、119、399、434–452 行（整行）；`.claude/kb/decisions/22-单元原子性怎么合成.md` 第 17–19、185、197–202、349–372 行；`.claude/kb/layout/…` 的 `grep -n '取号'` 命中行。
- `.claude/gate.d/stage-inputs.tsv` 第 46–48 行与 `grep -c 'e142-r2'`（0）；`.claude/gate.d/replay-comparison-units.tsv` 文件头与 `grep` 两个 E 号的行；`.claude/gate.d/crash-point-replay-items.tsv` 命中行（取号屏障）。
- `scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs`：文件头 1–12 行、第 129–130、329–330、2358–2400、2805–2820、4464–4500、5658–5660、6093–6096、6959–7040、7205–7214、7250–7300、7768–7870、7976–7990、8006–8007、8190–8232、8243–8263、8658–8745、8825–8860、8981、9045–9061、10501–10530 行（及所列 grep 命中行）。**注意**：这是装置，不是被测条款；设计员读它是为了写守门规则表、旧值样式和单测钉值清单，执行员改装置时读它是正常的。
- `scripts/experiment-apparatus/replay.sh`：第 2655–2668、2815–2850、3040–3085、4780–4812、4908–4911 行。`scripts/kb/claim-experiment.sh`（文件头）、`scripts/kb/quote-kb.py`（文件头用法）。
- `crates/singlefs-implementation-experiments/src/bin/e142_new_pool_file_creation_write_dump.rs`：`grep -n -i 'acquisition\|window\|system_config'` 命中行（第 91、221–250、252–300、326–327、485 行）。`crates/singlefs-core/src/transaction.rs`：`grep -n 'ACQUISITION_WRITE_ROUNDS\|fn write_the_witnessed_acquisition\|fn roll_back_acquisition'` 命中行（第 774、1312、1323、1333 行）。`crates/singlefs-verification/tests/new_pool_file_creation_publish.rs` 第 359、378、414 行（命中行）；`crash_enumeration_new_pool_file_creation_stream.rs` 第 824、828、892 行。
- 第 25 次登记（`git show ceac9b0fc:research/prompts/e142-r25-prereg.md`）第 1–77、446–560、595–765、985–1363 行；第 25 次问题单全文；第 22 次登记（`git show c84047263:…e142-r22-prereg.md`）第 1318–1538 行（脚本全文）与 `grep` 命中行；第 21 次登记第 1526–1548 行；第 23 次登记第 490–500 行。
- `git show 841b3beabb98:scripts/experiments/e142_new_pool_file_creation_dry_run.rs`（整份，只做 `diff` 与计数，滤掉注释后 132 行逐行扫过）。
- **没读**：`research/results/` 下任何一份产物的内容（`e142-new-pool-file-creation-dry-run-r25-main-2026-10-13.out` 与 P8G 导出只对 `git show … | sha256sum`、`wc -l` 算过；`arm-n15` 参照只算了 sha256）；E142 页除「影响的决策」表之外的小节；E142 的实验变更史；第 25 次执行员报告。

### 13.2 跑过的命令（原样；输出是 2026-11-10 在仓根或草稿目录跑的原样输出；`D=` 是设计员的草稿目录 `/home/fy5090/.local/share/singlefs/work/e142-r26-registration`，执行员重跑时换成自己的）

**命令一、二**（各 sha256 与行数、与第 25 次产物所在提交里装置的差异、判段入口零命中、挂钟字段、准入表）：

```
cd /home/fy5090/code/singlefs && A=scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs; { echo '命令一'; sha256sum $A scripts/mutation/tables/e142_new_pool_file_creation_dry_run.tsv scripts/experiment-apparatus/e142_region_diff_independent.rs research/results/e142-new-pool-file-creation-dry-run-2026-09-25-r16-arm-n15.out; wc -l $A scripts/mutation/tables/e142_new_pool_file_creation_dry_run.tsv scripts/experiment-apparatus/e142_region_diff_independent.rs | head -3; git hash-object $A; git rev-parse HEAD:$A; git status --short $A | wc -l; for f in e142-new-pool-file-creation-dry-run-r25-main-2026-10-13.out e142-r25-p8g-crates-dump-2026-10-13.out; do git show 841b3beabb98:research/results/$f | sha256sum; git show 841b3beabb98:research/results/$f | wc -l; done; echo '命令二'; git show 841b3beabb98:scripts/experiments/e142_new_pool_file_creation_dry_run.rs | sha256sum; git show 841b3beabb98:scripts/experiments/e142_new_pool_file_creation_dry_run.rs | wc -l; diff <(git show 841b3beabb98:scripts/experiments/e142_new_pool_file_creation_dry_run.rs) $A | grep '^[<>]' | grep -vE '^[<>] *//' | wc -l; grep -c 'run_from_environment\|GivenStatesEngine\|judge_not_run_in_this_binary\|crash_state_enumeration' $A; grep -n 'seconds=\|Instant\|SystemTime\|elapsed()' $A | cut -c1-120; grep -c 'e142-r2' .claude/gate.d/stage-inputs.tsv; sed -n 2662,2663p scripts/experiment-apparatus/replay.sh; } > $D/cmd12.txt 2>&1; cat $D/cmd12.txt
命令一
f1fb548daae1378be29e87cc09ee996b157d4bb743162a564904400077a5317a  scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs
64caabe99c1f37d69462e2e9952292e0d23723508d4010610ea0967692d1c679  scripts/mutation/tables/e142_new_pool_file_creation_dry_run.tsv
2f41d6e588d8a2fd06c141db8b99024f89d0ed36bb1ee3c6456b20d283b49fb7  scripts/experiment-apparatus/e142_region_diff_independent.rs
f0e1515a0bea80800acfc5eb5f2ff77f975ca096ba90167200a30e59f437126d  research/results/e142-new-pool-file-creation-dry-run-2026-09-25-r16-arm-n15.out
 11066 scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs
   221 scripts/mutation/tables/e142_new_pool_file_creation_dry_run.tsv
   809 scripts/experiment-apparatus/e142_region_diff_independent.rs
82d3c789fd78931bcb80a090a52854cdbd5fed43
82d3c789fd78931bcb80a090a52854cdbd5fed43
0
a64b55e9c6acb46a57c92e8ef486a065e2cb9cb4a07a40c8e3b233886d96f2bb  -
1213
6cf0735e387114072ec4a4c86c847fca6ee185b4b2087e7d344dea70e61fe6ea  -
39
命令二
1655e36e951ebbf070670fe1dd2471a1d38e863b0b64d86071fc9ed30f5ee07a  -
11092
132
0
7810:    let layer0_main_start = std::time::Instant::now();
7862:    emit(&mut emitter, &format!("name=layer0_main_arm_elapsed seconds={:.3} states={} threads={layer0_main_threads}
0
E142|@driver_e142||e142-new-pool-file-creation-dry-run-r25-main-2026-10-13.out|exact
E142|@driver_e142_p8g_crates_dump||e142-r25-p8g-crates-dump-2026-10-13.out|exact
```

**命令三**（`anchors_r26.py`，第七节 7.2；`python3 -I`）：

```
cd $D && python3 -I anchors_r26.py > anchors_r26.out; cat anchors_r26.out; sha256sum anchors_r26.py anchors_r26.out
A-Q0 mkfs 14+1+1+1+4 two_state = 16402 (条款值 16402，校验公式)
A-Q1 取号 2+2 two_state = 7  operations = 6 (4 写 + 第一轮之后一道池屏障按设备记 2 步)
A-Q2 整条流 12 段 two_state = 16777243  operations(取号 6 + 暖机 20 + 新池新建文件 35) = 61
A-Q3 整条流 12 段 three_state = 16777268
A-Q4 窗口之前 (operations, writes) = ( 51 , 35 )
A-Q5 世代号与槽 = [(1, 1), (2, 0), (3, 1), (4, 0), (5, 1), (6, 0)]
A-Q6 整条流写数 = 43  屏障步数 = 18  (= 9 道池屏障 × 2 盘 = 18 )
A-Q7 G1B 一块盘取号 1+1 two_state = 3  operations = 3 (2 写 + 1 道屏障)
A-Q8 N26R 取号一轮 2 two_state = 4  three_state = 9  operations = 2
A-Q8 N26R 整条流 11 段 two_state = 16777240  three_state = 16777260  operations = 59 窗口之前 ( 47 , 33 )
cf38d3b31249514e17e734742833decbd2e7bab5bbad540a584388183052746c  anchors_r26.py
c5fac27110827db9858bfff99a8c78edc5962be931231b899af707c4b9f273ea  anchors_r26.out
```

**命令四**（`residual_r26.sh` 在 `HEAD` 装置与变异表上，按样式的命中行数；PC-sweep 的六份合成输入）：

```
cd /home/fy5090/code/singlefs && bash $D/residual_r26.sh . | grep -E 'pattern=|total'
residual_r26 pattern=O_header_311_and_old_offsets count=0
residual_r26 pattern=A_two_state_16777240 count=18
residual_r26 pattern=A_three_state_16777260 count=1
residual_r26 pattern=A_eleven_segment_string count=3
residual_r26 pattern=A_before_window_47_33 count=1
residual_r26 pattern=A_post_mkfs_57_operations count=1
residual_r26 pattern=A_acquisition_two_writes_one_round count=2
residual_r26 pattern=A_generation_ladder_one_round count=1
residual_r26 pattern=A_old_constant_name count=5
residual_r26 total=32
```

六份合成输入（草稿目录 `pc/s/*.rs`、`pc/t/*.tsv`，三参数形态 `bash residual_r26.sh . <装置样本> <表样本>`）：变异名一行 + 表第 1 列 `M233_x` → `total=0`；`// 233` → `total=1`；`let x = 310; // M246_foo` → `total=1`；`// header311` → `total=1`；表第 2 列裸 `x 233 y` → `total=1`；表第 2 列 `x M233_q y` → `total=0`。设计时依次跑出 0、1、1、1、1、0。

**命令五**（第 22 次登记命令五、四的自检，脚本取自第 22 次登记全文，`python3 -I`）：

```
cd $D && { python3 -I p8g_region_compare.py --selftest; P8G_COMPARE_BREAK=1 python3 -I p8g_region_compare.py --selftest; python3 -I label6_anchor.py | head -8; LABEL6_ANCHOR_BREAK=1 python3 -I label6_anchor.py | head -2; }
```

输出：`selftest same=(3, 0, 0, 0) flip=(2, 1, 0, 0) drop=(2, 0, 1, 0) True`；弄坏开关下 `… flip=(3, 0, 0, 0) … False`；`label6_anchor.py` 不带输入 `per_object_41 41 expected 41 True`、`A1 label6 main … 98386 expected 98386 True`（A1 是退役树之前的旧式子值，PC-j 只看合成自检那几行、不判 A1）；弄坏开关下 `per_object_41 40 … False`。

**命令六**（占号与读规则时的其余命令，原样）：`bash scripts/hosts/work-directory-ownership.sh claim $D --purpose 'E142 第 26 次重跑登记'`；`git log --all --oneline -- 'research/prompts/e142*'`（找第 25 次登记的位置）；`git ls-tree -r --name-only 84b4ae23f -- research/prompts | grep -E 'e142-(r2[0-9])'`；`python3 scripts/kb/quote-kb.py $D/quote-r26.md '.claude/kb/checks-owed.md~^\| C1001 \|' '<事实表>:24-24' '.claude/kb/layout/01-first-txn.md:38-38' '…:62-62' '…:85-85' '…:119-119' '…:399-399' '…:436-436' '…:441-441' '…:450-450' '.claude/kb/decisions/22-单元原子性怎么合成.md:19-19' '…:351-351'`（输出「✓ 12 段整抄进 … 回读逐字节一致」）。

### 13.3 两份新草稿脚本全文（设计时已跑）

照原样存进执行员草稿目录（文件名照写），核 sha256；`residual_r26.sh` 要带执行位。

`anchors_r26.py`（21 行，sha256 `cf38d3b31249514e17e734742833decbd2e7bab5bbad540a584388183052746c`）：

```python
# E142 第 26 次登记第七节 7.2 的独立算术：不读装置、不读 crates/，只用条款里的数（layout/01 八、D22 已定项 16）。
def two_state(segs):   # 每段 2^k - 1 个非满子集，加终态 1（layout/01 八；mkfs 14+1+1+1+4 -> 16402 校验）
    return sum(2**k - 1 for k in segs) + 1
def three_state(segs, sysconf_segs):   # 系统配置槽写取三态：该段 3^k - 1 代替 2^k - 1
    return sum((3**k - 1 if i in sysconf_segs else 2**k - 1) for i, k in enumerate(segs)) + 1
# 整条流的段种类（layout/01 第 450 行的种类串，新）：sc = 系统配置槽×2，jr = journal 记录×2，root = 根槽 FUA，unit = 24 个单元
kinds_new = ["sc", "sc", "jr", "root", "sc", "jr", "root", "sc", "unit", "jr", "root", "sc"]
new = [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]
kinds_old = ["sc", "jr", "root", "sc", "jr", "root", "sc", "unit", "jr", "root", "sc"]
old = [2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]
sc = lambda kinds: {i for i, k in enumerate(kinds) if k == "sc"}
print("A-Q0 mkfs 14+1+1+1+4 two_state =", two_state([14, 1, 1, 1, 4]), "(条款值 16402，校验公式)")
print("A-Q1 取号 2+2 two_state =", two_state([2, 2]), " operations =", 2 + 2 + 2, "(4 写 + 第一轮之后一道池屏障按设备记 2 步)")
print("A-Q2 整条流 12 段 two_state =", two_state(new), " operations(取号 6 + 暖机 20 + 新池新建文件 35) =", 6 + 20 + 35)
print("A-Q3 整条流 12 段 three_state =", three_state(new, sc(kinds_new)))
print("A-Q4 窗口之前 (operations, writes) = (", 25 + 6 + 20, ",", 21 + 4 + 10, ")")
print("A-Q5 世代号与槽 =", [(g, g % 2) for g in range(1, 7)])
print("A-Q6 整条流写数 =", sum(new), " 屏障步数 =", 61 - sum(new), " (= 9 道池屏障 × 2 盘 =", 9 * 2, ")")
print("A-Q7 G1B 一块盘取号 1+1 two_state =", two_state([1, 1]), " operations =", 1 + 1 + 1, "(2 写 + 1 道屏障)")
print("A-Q8 N26R 取号一轮 2 two_state =", two_state([2]), " three_state =", three_state([2], {0}), " operations = 2")
print("A-Q8 N26R 整条流 11 段 two_state =", two_state(old), " three_state =", three_state(old, sc(kinds_old)), " operations = 59 窗口之前 (", 25 + 2 + 20, ",", 21 + 2 + 10, ")")
```

`residual_r26.sh`（35 行，sha256 `bc372f560103767916eb723a4e4efc1a5d97f025abaeaae1f201d985d9a31fc9`）：

```bash
#!/usr/bin/env bash
# E142 第 26 次登记 Q142.157 / Q142.158：装置与变异表里旧值的残留，逐条列出、按样式各报行数、最后报总数。
# 与第 21 次 residual_old_values.sh（样式 O_*）同一套判法，两处改动：(1) 匹配之前先把每行里的变异名（M 加数字、可带下划线名字，如 M233_area_below_the_root_ring_base）
# 整个删掉，再匹配（变异名里的数字不是旧偏移；删掉的只是变异名这一个记号，同一行里别的数字照判）；白名单 W2（M67、M186–M188）因此不再需要，只留 W1。
# (2) 加取号写两轮之前的旧值样式 A_*（取号一轮的态数、段序列、操作数、世代号与旧常量名）。
# 用法：bash residual_r26.sh <仓根> [<装置文件> <变异表文件>]    缺省取 <仓根>/scripts/experiment-apparatus/ 与 <仓根>/scripts/mutation/tables/ 下那两份。
set -u
ROOT="${1:?仓根}"
F="${2:-$ROOT/scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs}"
T="${3:-$ROOT/scripts/mutation/tables/e142_new_pool_file_creation_dry_run.tsv}"
NAMES='M[0-9]+(_[A-Za-z0-9_]*)?'
W1='0x[0-9a-f]{8}, 0x[0-9a-f]{8}'
total=0
report() { # $1 样式名  $2 扩展正则
  local label="$1" pattern="$2" hits table_hits count
  hits=$(sed -E "s/$NAMES//g" "$F" | grep -nE -- "$pattern" | grep -vE -- "$W1")
  table_hits=$(NAMES="$NAMES" PATTERN="$pattern" WHITELIST="$W1" awk -F'\t' '{s = $2 "\t" $3; gsub(ENVIRON["NAMES"], "", s); if (s ~ ENVIRON["PATTERN"] && s !~ ENVIRON["WHITELIST"]) print NR}' "$T")
  count=$( { printf '%s\n' "$hits"; printf '%s\n' "$table_hits"; } | sed '/^$/d' | wc -l)
  printf '%s\n' "$hits" | sed '/^$/d' | cut -d: -f1 | tr '\n' ' ' | sed "s/^/fixture_lines:$label: /"; echo
  printf '%s\n' "$table_hits" | sed '/^$/d' | tr '\n' ' ' | sed "s/^/table_rows:$label: /"; echo
  echo "residual_r26 pattern=$label count=$count"
  total=$((total + count))
}
# O_*：第 21 次那一组（记录头 311 字节与 fsid / MAC 旧偏移）
report O_header_311_and_old_offsets '(^|[^0-9_x])(311|188|67|3785|310|233)([^0-9_]|$)|_311|311_|287\.\.=294|295\.\.=310|\[287\.\.295\]|\[295\.\.311\]|287 \+ 8|, 287, "fsid|, 295, "MAC|header311'
# A_*：取号写一轮的旧值
report A_two_state_16777240 '(^|[^0-9_])16_?777_?240([^0-9_]|$)'
report A_three_state_16777260 '(^|[^0-9_])16_?777_?260([^0-9_]|$)'
report A_eleven_segment_string '(^|[^0-9+])2\+2\+1\+2\+2\+1\+2\+24\+2\+1\+2([^0-9+]|$)'
report A_before_window_47_33 '\(47, 33\)'
report A_post_mkfs_57_operations 'post_mkfs_operations\.len\(\), 57|取号 2 \+ 暖机 20 \+ 新池新建文件 35'
report A_acquisition_two_writes_one_round '取号：两盘各写一次系统配置槽|只有两次系统配置槽写|acquisition_operations\.len\(\), 2([^0-9]|$)'
report A_generation_ladder_one_round '取号 2（槽 0）、w3 3（槽 1）、w6 4（槽 0）、t11 5（槽 1）'
report A_old_constant_name 'SYSTEM_CONFIGURATION_GENERATION_AT_INSTANCE_ACQUISITION'
echo "residual_r26 total=$total"
```

### 13.4 守门脚本与其余草稿脚本的底本（执行员照这里重建 / 改写）

| 脚本 | 取法 | sha256 | 用在 |
|---|---|---|---|
| `r25_guard.py`（底本，320 行） | `git show ceac9b0fc:research/prompts/e142-r25-prereg.md \| sed -n '1032,1141p;1147,1256p;1262,1361p' > r25_guard.py` | `49f511b7afe37cfa425a348d920a41cf2652179667ff33046121cdd01186cda7` | `r26_guard.py` 的底本；**不直接用** |
| `label6_anchor.py`（81 行） | `git show c84047263:research/prompts/e142-r22-prereg.md \| sed -n '1349,1429p'` | `15c945328a473e21c0e50a604445e54b9b40b98c4996a434a1596883a70725f7` | Q142.147 (3)、Q142.153、PC-j |
| `make_synthetic.py`（12 行） | `… \| sed -n '1435,1446p'` | `f9936271b8c23c7c4fd7868d9b18bb4978ac18c7d9c443e5b25099f83b0fede8` | PC-j 的合成输入 |
| `p8g_region_compare.py`（76 行） | `… \| sed -n '1462,1537p'` | `1090d0a763085ae6eb7a9ae2739b0d1b7bec9bea23f03a6180298792cd7937a8` | Q142.153、PC-k |

（第 22 次登记文件 `git show c84047263:research/prompts/e142-r22-prereg.md`，sha256 `963557f844c10563315fd82d1b0066e631d61706e3b78c93ade1d79231c7bd66`；第 25 次登记文件 sha256 见文件头。）

**`r26_guard.py` 的写法（执行员第二段步 ① 写，写完算 sha256 记进第十二节；设计员没写它，因为它要吃 N26 的真实行形态，装置还没改）**：

1. 以 `r25_guard.py` 为底本，名字与前缀 `R25` → `R26`，弄坏开关 `R26_GUARD_BREAK`；档名集合改成 `drift`、`acq2`、`same`；`first_half`、`normalize`、`groups`、`layout_key`、`changed_keys` 一字不改。
2. `judge_group` 的规则全部按第六节「守门规则表」重写：每个组要么在 `ZERO` 清单里（零差异），要么有一条形态规则（列、值、行数）；**不在任何清单里的组变了 ⇒ `groups_bad`**（`R25` 里默认落进 `attribute` 的那一路改成 `bad`，`attribute` 只给规则表明说的四类组）。
3. `p8g_verdict`、`p8g_control_main_point` 从零差异清单移到「预期组」：新产物里 `unequal=0` 且 `equal=` 等于 `fields=` 才过，`fields=`、`equal=` 数值本身可变（C774：第 25 次守门把它放零差异组、被 `equal=12` 的回落打红，这是登记的字面漏写，不是装置错）。
4. 合成自检 `--selftest`：至少这些样本，期望值在名字里：`same_identical`（过）、`same_one_change`（红）、`drift_good`（过）、`drift_bad_region_row`（`device_region_bytes` 一行变，红）、`drift_bad_unlisted_group`（`accounting` 变，红）、`drift_bad_verdict_extra_field`（`verdict` 里 `width_mismatches` 变，红）、`drift_p8g_control_good`（`equal=14 fields=14 unequal=0`，过）、`drift_p8g_control_unequal`（`unequal=1`，红）、`acq2_good`（过）、`acq2_bad_region_set`（变了非系统配置槽的区域，红）、`acq2_bad_segments_other_path`（`segments path=transaction` 变，红）、`acq2_bad_digest_extra_insert`（多插第三行，红）、`acq2_bad_instances_field`（`instances` 别的字段变，红）、`acq2_bad_layer0_violation`（`violations=1`，红）、`acq2_timing_still_on_stdout`（N26 里还有 `layer0_main_arm_elapsed`，红）。`R26_GUARD_BREAK=1` 时 `acq2` 档不再核「变的区域行恰是窗口里系统配置槽那两盘」，使 `acq2_bad_region_set` 判过、打 `ok=False`，`all_ok=False`。
5. 输出行格式同 `R25`：`R26 profile=…`、`R26_GROUP name=… rows_old= rows_new= changed= added= removed= problems=…`、`R26 changed_lines_total=`、`R26_VERDICT profile=… pass=… groups_bad=… groups_to_attribute=…`；自检行 `PC_G26 sample=… ok=…`、`PC_G26 all_ok=`。
6. 脚本不 import 装置、不读 `crates/`，只吃两份产物文本。

**命令六（标签 6 四种旧写法）与命令十二（T_time 旧写法）在今天的路径上的写法**（沿用第 22、23 次登记，路径 `scripts/experiments/` 换成 `scripts/experiment-apparatus/`）：

```
cd /home/fy5090/code/singlefs && A=scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs; { echo '命令六'; grep -c 'STATISTIC_COMMITTED_RESERVATION_BYTES, TREE_IDENTIFIER_NONE, 0)' $A; grep -c 'for statistic in \[STATISTIC_UNRECLAIMABLE_BYTES, STATISTIC_PENDING_DELETE_BYTES, STATISTIC_COMMITTED_RESERVATION_BYTES\]' $A; grep -c '准入不等式要的四项 day-1 各写一行 0' $A; grep -c '三项 0（读不到行是坏账' $A; echo '命令十二'; grep -c 'writer.put_u32(5); // T_time 秒' $A; grep -c 'T_time 秒' $A; grep -c 'writer.put_u32(5000); // T_time 毫秒' $A; grep -c 'T_time' scripts/mutation/tables/e142_new_pool_file_creation_dry_run.tsv; }
```

设计时在今天的装置（N26R）上跑过（`grep -c`，只读）：命令六 `0 0 0 0`，命令十二 `0 0 1 0`，与登记的门槛一致；执行员在 N26 上重跑，门槛不变：命令十二必须 0、0、1、0，命令六四个计数另报。PC-old 的合成对照（把旧写法追加进拷贝再跑，期望命令十二第一、二个计数各 1、命令六第三个计数 1）照第 25 次登记 5.2「PC-old」的做法，在第二段 N26 上做。

## 十四、并行与挂钟

- 独立单位：每段里互相独立的有三组：装置（N26R 或 N26）的 `driver_e142` 一趟、`crates/` P8G 导出一趟（第二段）、草稿脚本与独立 bin 的比对；N26R 与 N26 是两个不同的二进制与两份产物，互不读对方。同一段内编译、`cargo test`、导出 bin 的编译可并行起；比对必须等产物落盘，是串行的下游。
- 线程：层 0 整轮枚举与编译、单测一律取登记值 `.claude/heavy-thread-default`（31），命令里不写；草稿脚本（守门、扫法、比对）单线程。
- 串行：快照 A → B → C 之间、同一段内「产物落盘 → 准入退 77 检查（Q142.163）→ 比对」因共享工作树与 `crates/` 状态必须串行；第二段步 ③ 的 N26 一趟与步 ⑤ 的第二趟不并行（第二趟要在 N26 产物落盘、快照 C2 之后起，两趟共用同一份二进制与机器，挂钟字段在 stderr 里本来就不进比较，所以并行不会改结果，只是互相挤线程、拖慢 T0 的读数，故排成串行）；变异表整表由主 agent 单独起，不与同一棵树上的 `driver_e142` 并行（`mutate.sh` 退 5 的规则）。
- 预计挂钟：**第一段**约 T0 + 40 分钟（编译约 15 分钟，估，装置十万行级别单文件 release 编译，没量；N26R 一趟 = T0 加非层 0 部分约 5 分钟，估；导出 bin 编译与运行约 10 分钟，估；比对与归因表约 10 分钟）；**第二段**约 2×T0 + 70 分钟（单测里含一次整轮枚举同量级的状态，约 T0；N26 一趟约 T0；第二趟约 T0，仅 T0 ≤ 7200 秒时跑）；T0 是层 0 主臂整轮 16 777 243 个状态的单趟挂钟，**没有读数**（第一节未定项 1），所以这两个数是式子不是结论；整张变异表（主 agent）挂钟同样依赖每条变异是否都跑整轮单测，也没有读数（未定项 2）。超过 2 小时的段：分片单元与续跑入口，装置的 `enumerate_layer0_grouped`（第 4972 行）与 `main()` 里都没有，这一次登记**做不到**，缺的是第二段「层 0 整轮」这一步的分片与续跑；T0 > 7200 秒时按停机 S26-time 交回，不在这份登记里硬凑。
- 预计在用核数：编译与单测 31 个线程（登记值，机器上实际在用核数由服务定）；层 0 整轮 31 个线程（`thread_cap_from_env` 取线程上限，登记值）；`crates/` 导出与草稿脚本各 1 个核；变异表整表 4 个工作进程（`MUTATE_JOBS=4`，每个进程内编译与测试再取 31 线程上限，核数以服务定的为准）。
