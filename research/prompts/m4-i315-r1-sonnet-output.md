# m4-i315-r1 云端正推腿报告（2026-10-21）

腿：云端正推（Sonnet），格 K1–K5。立场：从条款原文与 `crates/` 今天的实现推「应该是什么」，逐格与实现比；推荐一种具体组合；要交用户的岔路写成候选、翻面观测、够判条件三列，不带倾向。

## 0　开工核对与复跑

- 开工快照核对：`sha256sum -c research/prompts/m4-i315-r1-snapshot/tree-sha256.txt` 六份被判文件逐份 OK（`walk.rs`、`fault_injection.rs`、`transaction.rs`、`crash_judge_checker_scan.wgsl`、`checker_known_bad_images.rs`、`05-空间记账.md`）；`kb-sha256.txt` 16 行全 OK。没有对不上的。
- 下文行号都是主工作区此刻的 `crates/` 与 kb 现查的（与背景材料给的行号有出入处，以下文为准：例如 `.filter(|header| …)` 在 `walk.rs` 第 16380 行，`continue` 在第 16384 行；写者留记录的 `release_leaving_the_record_allocated_on` 在 `transaction.rs` 第 12585 行）。
- 量的东西都在我自己的副本里（草稿目录 `~/.local/share/singlefs/work/m4-i315-r1-forward/tree`，从主工作区拷 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`，打上调查员的 `instrumentation.patch`），只跑了 harness 档（`singlefs-harness` 的测试目标）与 `singlefs-checker --lib`，checker 档包没编没跑。开关是调查员补丁加在 `walk.rs` 里的环境变量：今天读法不设；乙一 `PROBE_FIX_GROUPING`；乙二 `PROBE_FIX_GROUPING_WHEN_NO_KEY`。
- 模型目录 `research/prompts/m4-i315-r1-sonnet-model/`：`probes/`（调查员的补丁与探针原样拷入，加我写的 `probe_i315_scenario_images.rs`）、`tools/rerun.sh`（模板，只改 `rerun_steps`）、`out/summary.txt`（报告里抄的量的汇总行）。复跑命令与每个文件的 sha256 如下。

复跑（整条经内存包装；模型目录先拷进新的草稿目录再跑，不在原目录里跑；`--frozen` 我自己跑时给的是主工作区本身，判决复跑请给冻结副本，里面要有 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`；线程默认 31，要换设 `RERUN_THREADS`）：

```
bash scripts/hosts/run-with-memory-cap.sh 6G bash <模型目录副本>/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <冻结副本> --work <新的草稿目录> --model <模型目录副本>
```

跑完看最后一行 `name=rerun_compare result=same`（与 `out/summary.txt` 的 TALLY / SCEN / WORLD / test result 行排序后逐行比；`finished in` 的秒数已去掉）。我自己在同一棵树上连跑两次：第二次 `result=same`（第一次没有基准文件，用它生成了 `out/summary.txt`）；每次墙钟 2 到 4 分钟，另加内存包装排队。

| 文件 | sha256 |
|---|---|
| `out/manual-runs.txt` | `677dfdcc5dc5dec36275374d1497c1e32ed5f0219081da68469c66d0b466efe3` |
| `out/summary.txt` | `c07d2cdfb4847f9686ce9548ce7323210f3ed8f0463ca9f49ab231bfbb58399b` |
| `probes/instrumentation.patch`（调查员的补丁，原样拷入） | `cc1b2c2bfa58e2c5ced32ad2abd4b6ac4227dc50e881d77f6dc0df45fda0ff0e` |
| `probes/probe_i315_scenario_images.rs`（我写的） | `09d71f7d6a318ce73a866157fba9352693042606bb31be913df2f7853f2b400a` |
| `probes/probe_swallowed_write_and_i315.rs`（调查员的探针，原样拷入） | `2e6072271c2b88e1abea3ae47c1dcddcba1d94f3a0d117c7036b0008c0c388e3` |
| `tools/rerun.sh`（模板，只改 `rerun_steps`；`--selftest` 5 格过） | `6f416319a968cf09ddfe2373945eb18ab44d4f0f141368df48a5abb6af28d718` |

`out/manual-runs.txt` 里的四个 harness 目标、`singlefs-checker --lib`、回退复活世界的逐版 key 数是手跑的，不在 `rerun.sh` 里（`rerun.sh` 只复跑快档归类与场景夹具两组）；它们的命令在第 9 节。
- 开跑前 `ps` 看到别的会话的 `cargo clippy`、`cargo test -p singlefs-checker-tier …`、变异驱动在跑；没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`。内存包装排队过（等别的会话的 slice 放出空），没有绕开。

## 1　结论摘要（推荐的组合，岔路见第 7 节）

1. I-3.15 要改，改成「乙二」：把头用不了的那一份的记录**算进隔离判据的组**（它自己仍没有对象、不判红），但只在「这一版中央映射里没有 key 指着这一槽」时这样归组；有 key 指着的照今天读法。依据：条款第二句「隔离的单元」按单元，且 D19 已定项 5 把隔离写成「映射条目去掉」，所以「没有 key 指着」是隔离的必要条件，这条件 checker 自己看得见。
2. 乙一（不加 key 限定）与条款字面更贴，但会让 `checker_known_bad_images.rs` 第 15657 行那条必红用例的 I-3.15 消失（量的：四个 harness 目标里恰好这一条红，别的 9 个不变量照红）；那一条的 I-3.15 是这次 N2 落地时在未提交改动里加进期望集合的（`git blame` 显示 Not Committed Yet），不是独立从条款推出来的。要不要保它是岔路 F1。
3. 归类器（甲）：I-3.15 改好之后，快档 24 段的新发现 7 条变 0（量的）；插回去那张盘上仍红 I-3.11、I-3.15（7 个吞写点，量的）这件事不再撞名字。**只取 I-3.15 这一半，今天的取样里就够**；甲留着的是名字相撞的隐患，不是现有的红。是否现在补，取决于更大档里有没有撞名字的新发现，写成岔路 F2，不在这里替用户定。
4. 一个磁盘上看不出的缺口：隔离的单元两份其实都好（瞬时坏读、强制隔离开关、插回去那张盘）时，I-3.1、I-3.11、I-3.15 三条都红，任何只看盘的读法都一样（量的，「单元两份都好」场景）。要分出来只能让实现把隔离清单交给判别的一方（甲细、甲三）或让盘上留标记（F3）。

## 2　K1　读法的定义

读法的名字沿用调查员的（乙一、乙二），今天的叫「今天读法」；我另加的归类器读法接着甲往下编号（甲二、甲三）。

| 读法 | 组里取哪几份 | 头用不了的那一份算什么 | 输入从哪来 | 实现位置 |
|---|---|---|---|---|
| 今天读法 | 只取「头能用、且对不上这一版映射」的那几条记录所在的盘 | 没有对象：记录先 `continue`，不进组，也不进隔离判据 | 这一版分配记录树与中央映射、盘上位置项 | 「.filter(|header| birth_txg_in_a_usable_unit_header(header).is_some())」（`crates/singlefs-checker/src/walk.rs:16380`）、「let devices_of_the_unit: Vec<u32> = records_of_the_unit」（`crates/singlefs-checker/src/walk.rs:16466`） |
| 乙一 | 这一版分配记录树里这个（起点槽、跨度）上每块盘的全部未释放记录，不看头 | 自己不判红（没有对象）；但它那一份的字节要拿去对位置项，对不上算「这个单元有一份对不上」→ 整组不判。头用不了的那一份读出来基本对不上任何位置项（调查员量的：7 条里的整单元 CRC 都不等于位置项校验和，我没复跑这一项；固定盘面 W1、W3 里它对不上是我量的），退回自证时 `check_unit` 也不过 | 同上 | 改 `walk.rs` 第 16465 行那个 for 的 `devices_of_the_unit` 来源 |
| 乙二 | 同乙一，但只在「这一版映射里没有 key 点名这个槽（任一块盘）」时；有 key 点名的照今天读法 | 同乙一（在没有 key 的分支里），有 key 的分支里同今天读法（没有对象） | 同上，另用 `keys_at_each_placement`（`walk.rs:16386` 一带已算好） | 同上，多一个判断 |
| 腿加：丙（只记，不推荐） | 乙一，但对位置项校验和改成「这一版映射里点名这个槽的那条位置项」而不是「所有走过的版本里指着这一格的校验和集合」 | 同乙一 | 要把 `LocationEntryChecksumsForQuarantine`（`walk.rs:8192`）改成按版本取 | 改动面大于乙二，且没有量到今天的集合读法在某个形上放过（见 K3 末尾的读码疑点），不推荐 |

| 归类器读法 | 判据 | 输入从哪来 | 现状 |
|---|---|---|---|
| 今天 | 「插回去那张盘上判红的不变量名」与「注入之后判红的名」有交集就不算说谎设备留下的：「.invariants_still_violated_if_the_swallowed_writes_had_landed」（`crates/singlefs-harness/src/fault_injection.rs:2578`） | checker 档对插回去那张盘的判定，只取不变量名 | 已实现 |
| 甲（调查员的粗版） | 插回去那张盘上 I-3.15 的第一处违例说明点名的（盘、槽）与某次被吞的写的字节范围重叠，则这一条不算「仍红」 | 违例说明字符串 + 已有的 `SwallowedWriteByteRange::overlaps`（`fault_injection.rs:2541`） | 调查员量的（报告「判断材料」一节，我没复跑这个开关）：7 条归说谎设备；但池级 checker 每条不变量只留第一处违例：「self.first_violation.entry(invariant).or_insert_with(detail);」（`crates/singlefs-checker/src/image.rs:494`），且 checker 档 GPU 判的结论不带细节：「pub const POOL_CHECKER_VIOLATION_WITHOUT_ITS_DETAIL: &str =」（`crates/singlefs-checker-tier/src/crash_given_states.rs:148`），这一读法在 checker 档的路径上拿不到字符串 |
| 甲二（单元落点 + 实现的隔离清单） | 插回去那张盘上 I-3.15 的每一处违例的（盘、槽、跨度）都在「实现在这段历史里核出过对不上」的清单里；I-3.11 的逐盘差额等于清单里该盘上的槽数之和（推的，没量） | 清单：`TransactionOutput` 上的「pub quarantined_after_release_checksum_mismatch:」（`crates/singlefs-core/src/transaction.rs:3235`），harness 一侧没有任何地方读它（`grep -rn quarantin crates/singlefs-harness/src` 零命中，见下）；每处违例的落点要 checker 交出结构化列表 | 都没有，要新接 |
| 甲三（带清单重判） | 判插回去那张盘时把清单交给 checker 当豁免输入（只用于这一次归类判定，不进生产判定），剩下的红才算「仍红」；名字比对不动 | 清单同甲二；checker 要多一个带豁免输入的入口 | 没有；checker 档里插回去那张盘现在经 GPU 判定库判，要在这张盘上改走 CPU 重判 |

命令：`grep -rn 'quarantin' /home/fy5090/code/singlefs/crates/singlefs-harness/src | wc -l` 输出 `0`（现查）。

## 3　K2　合法盘与说谎设备的盘上会不会误红

「量」= 我在副本里跑出来的（原样行在模型目录 `out/summary.txt`、`out/manual-runs.txt`）；「读码」= 只读代码推的；「复核不了」写原因。每格的红 / 成立指 I-3.15。

| 形 | 今天读法 | 乙一 | 乙二 | 依据 |
|---|---|---|---|---|
| 写被吞后隔离：被吞那份头用不了、另一份好、这一版映射没有 key（调查员那 7 条的形） | 红 | 成立 | 成立 | 量：固定盘面 W1、W1b、W3、W3b（盘 0 / 盘 1 头写零，或整单元写 0xa5，覆盖写一次隔离）今天读法红、乙一乙二成立（`out/summary.txt` 的 SCEN 行）；随机快档 7 条同形，乙一乙二各 24 条说谎设备、新发现 0 |
| 被吞那份是复用槽上的完整旧单元（头能用、位置项校验和对不上） | 成立 | 成立 | 成立 | 量：场景「完整的别的单元」三种读法都成立（`out/summary.txt`）；这一形今天读法本来就进组 |
| 写被吞但实现没核出对不上（12 个吞写点里 5 个） | 不红 | 不红 | 不红 | 量：这 5 点插回去那张盘违例集合是空（`PROBE_DETAIL_ALL` 跑里 `VIOLATIONS 插回去: []` 10 行 = 5 点 × 2） |
| 两份头都用不了 | 成立 | 成立 | 成立 | 量：W2。今天读法也不红：两条记录都 `continue`，组是空的 |
| 隔离的单元两份其实都好（瞬时坏读两次、强制隔离开关、插回去那张盘） | 红 | 红 | 红 | 量：场景「单元两份都好」「那两份改回对得上」三条不变量 I-3.1、I-3.11、I-3.15 全红（与 I-3.1、I-3.11 同步）；这是盘上看不出隔离，任何只看盘的读法都红，见第 1 节第 4 点 |
| 读盘核两次读结果不同类（先读不出、重读对不上，或反过来） | 同上 | 同上 | 同上 | 条款：「两次读结果不同类（先读不出、重读对不上，或反过来）按两次都没对上处置，报重读那一次的样子」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:115`）。处置结局都是隔离；盘上是否可见取决于那份是不是真坏：真坏同第一行，瞬时同上一行（读码） |
| 隔离之后又被整理（搬迁）碰到那一槽 | 同第一行 / 上一行 | 同 | 同 | 读码：隔离的记录留在已分配、槽不回空闲池（用例「fn quarantined_copy_stays_allocated_across_the_remount_and_is_never_handed_out_again」（`crates/singlefs-harness/tests/release_checksum_quarantine.rs:681`）），不会被当目的地；整理签发之后区域里出现隔离槽就丢意图、不搬：「隔离的那几份记进这次挂载的常驻隔离集（整理签发之后 R 里出现隔离槽就丢意图」（`crates/singlefs-core/src/transaction.rs:12596`，判的函数是 `compaction_plan.rs:795` 的 `region_holds_a_quarantined_slot`）；常驻隔离集只住内存（「发布路径记下一次释放前核出对不上而隔离的落点」，`crates/singlefs-core/src/compaction_plan.rs:584`），重挂后丢，丢了之后区域扫描靠盘上读出来的类分（G9 行见第 5 节）。搬迁的**源**也走释放前读盘核（「// 搬走的源落点释放之前同样按映射位置项读盘核」，`crates/singlefs-core/src/transaction.rs:10817`）：源槽的 key 已随搬迁换到目的地，源槽在后续各版里没有 key，乙二的无 key 分支适用；强制开关下源两份都好，三种读法都红。没有用例在这张镜像上断言 I-3.15：用强制开关的用例 `crates/singlefs-harness/tests/compaction_runs_stop_and_discard_after_the_publish.rs` 里 `check_pool_image` 命中 0 次（上面 awk 现数），只断言实现一侧；checker 档的 `crates/singlefs-checker-tier/tests/crash_judge_gpu_secondary_reads_after_the_source_segment_is_discarded.rs` 也用强制开关并判池级 checker，比的是 CPU 与 GPU 逐位一致，没有断言 I-3.15 的值（读码，checker 档包没编没跑）|
| 快照或回退候选集里还有一版指着隔离单元（该版有 key） | 成立 | 成立 | 成立 | 量：W1 里 A（txg 3）仍是候选、有 key，三种读法在 A 那一版都不红（红的只有 B 那一版，才有今天读法的红）。该版里好的那份头与 key 相符，被吞那份 `continue` |
| 两份都好但有记录没人引用（G9「已死」） | 红 | 红 | 红 | 量：四个 harness 目标（含 `checker_compaction_intent_and_relocation_must_red_worlds`）今天读法与乙二全绿，乙一只红一条且不是这一形（见 K3） |

**合法历史：隔离的单元之后某一版仍有 key 指着这一槽（乙二的依据「之后各版没人指着」不成立的那一段）**——有，而且量到了：

1. A（txg 3）的数据单元，盘 1 那份头用不了；覆盖写 B（txg 4）换下它，释放前读盘核出对不上，两块盘的记录留在已分配，B 的映射里没有 key 指着这一槽。
2. 管理员把盘 1 那份写回原样，之后回退到 A。条款与用例都允许：「盘 1 那一份写回原样（两份都对得上）：回退照做，冷启动读回 A 的内容」（`crates/singlefs-harness/tests/rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs:9`），回退之前逐盘验隔离单元的每一份是用户 2026-10-12 的定案（同文件第 3 行「用户 2026-10-12 定「隔离单元也进逐盘验」」）。
3. 回退发出的新根（txg 5）的映射又有 key 指着这一槽，那槽的记录仍是隔离留下的未释放。量：`PROBE_I315_SLOT=50180` 逐版打印，txg 3 的 key 数（盘 0、盘 1）是 (1,1)，txg 4 是 (0,0)，txg 5 是 (1,1)（`out/manual-runs.txt` 最后一段）。
4. 对读法的影响：txg 5 那一版有 key、头相符，记录成立，三种读法都不红；乙二在这一版走「有 key」分支（=今天读法），照样成立。所以乙二的依据那句话（「隔离的单元之后各版没人指着」）字面上不对，但**结论不受影响**：乙二真正用的是「这一版的映射里有没有 key 点名这一槽」这个当版事实，不是跨版的全称断言。txg 4 那一版在回退后仍在候选集里、两份已修好，三种读法都红（量：探针输出里标「写回原样后回退到 A」的那张盘（模型目录里的标签是「丙」，与 K1 里的读法丙不是一回事），I-3.15 的第一处违例说明点名「根（实例 1、txg 4）」）。

## 4　K3　坏盘上会不会放过

| 坏盘上的形 | 今天读法 | 乙一 | 乙二 | 依据与兜底 |
|---|---|---|---|---|
| 必红用例那一形：一份改坏成 0xa5、另一份是别一版的叶、映射里有 key 指着这一槽、对不上头 | 红（该用例通过） | **放过**：该用例红，期望集合里的 I-3.15 没了，别的 9 条照红 | 红（该用例通过） | 量：四个 harness 目标，今天读法 12 / 14 / 270（2 ignored）/ 20 全过；乙一 269 过 1 红，红的是 `a_shared_leaf_container_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device`，`left`（实际）= {I-1.11, I-1.2, I-2.1, I-3.10, I-3.5, I-4.2, I-4.8, I-4.9, I-7.4}，`right`（期望）多一个 I-3.15；乙二全过（`out/manual-runs.txt`）。乙一放过后兜住它的是 I-1.11、I-2.1 等 9 条（量，同一条用例） |
| 一份头被改坏、映射已去掉 key、记录没留在已分配之外的别的状态（泄漏加一份坏，与合法隔离是同一份字节） | 红 | 放过 | 放过 | 量：W3（A 已出候选集）今天读法红，乙一乙二的违例集合是空，没有任何别的不变量兜住；I-3.1、I-3.11 在这张盘上本来就按单元豁免（成立）。W3b（A 仍是候选）I-2.1、I-4.8、I-7.4 红，那是 A 引用着坏的那份，不是这个泄漏。**这个缺口不能靠读盘消除**：同一份字节也是合法隔离（W1、W3 就是），只有实现的隔离清单或盘上标记分得开（第 1 节第 4 点、岔路 F2、F3）。兜不兜住的比较：I-3.1、I-3.11 今天已经是这个口径（用户 2026-09-25 定「核出坏的才豁免」，`crates/singlefs-checker/invariants/05-空间记账.md:8`），改后 I-3.15 与它们一致，不是新增缺口 |
| 两份都坏（头都用不了） | 成立（放过） | 成立 | 成立 | 量：W2 今天读法就是成立（两条记录都 `continue`），三种读法没有差别；这一形靠 I-2.1 等读到它的不变量兜（W2 里 I-2.1、I-4.8、I-7.4 红，量） |
| 只有一块盘有记录 | 单块盘的组：头能用、无 key 时按这一份对位置项判 | 同 | 同 | 读码：乙二的 `account.records` 取同一（起点槽、跨度）上未释放的记录，只有一块盘的记录时组里只有一块盘；头用不了就是 `continue` 的那条记录自己，组空，不判；没量 |
| 位置项缺（走过的版本都不指着这一份） | 退回 `check_unit` 自证 | 同 | 同 | 读码：「Some(checksums) => checksums.contains(&crc32_castagnoli_table(&unit))」（`crates/singlefs-checker/src/walk.rs:16238`）与它的 None 分支，三种读法共用同一个判据函数，没改 |

**归类器甲在哪一形上会把真缺陷放成说谎设备留下的（判别力那一半）**

| 读法 | 会放过的真缺陷 | 方向 |
|---|---|---|
| 甲（粗版，只看第一处违例说明） | ① 同一张插回去的盘上 I-3.15 有不止一处违例，第一处点名被吞的槽、第二处是别的槽上的真缺陷：看不见第二处（`image.rs:494` 只留第一处）；② I-3.11 的违例说明是逐盘的差额数，不带槽，粗版对它没有判据 | 放过真缺陷（漏报） |
| 甲二（单元落点 + 实现的隔离清单） | 实现把一个被吞的槽泄漏（该释放没释放）：落点在清单里的那个槽上，与合法隔离同形，放过。落点不在清单里的真缺陷不放过 | 只放过「清单里的槽」那一类；清单拿不全（重挂后 `TransactionOutput` 不带：「从盘上重建的版本不是这个进程发布的，是空的」，`crates/singlefs-core/src/transaction.rs:3234`）时方向是假警报（把说谎设备留下的报成新发现），不是漏报 |
| 甲三（带清单重判） | 同甲二，且判的是 checker 加了豁免输入之后的结论 | 同甲二；代价是这一次归类里 checker 不再是只读盘的独立判者（转述 D13 与 `.claude/rules/implementation-first.md` 对独立验证的区分） |

读码疑点（没量、没造出例子，记在这里供攻方腿用）：隔离判据比的是「走过的各版位置项里指着这一格的校验和**集合**」，不是「这一版映射里点名这个槽的那一条」：`LocationEntryChecksumsForQuarantine` 的 `known` 是 `BTreeMap<(u32, u64), BTreeSet<u32>>`（`crates/singlefs-checker/src/walk.rs:8192`），判的是 `contains`。被吞那份若恰是复用槽上更早一版的完整单元、而那一版的位置项也在走过的版本里（候选集里更早的根仍引用过这个槽），`contains` 成立，这一份会被当成「对得上」，隔离不成立，I-3.15 红。我没造出这个形，读回收门槛（`max(F_生效, 环里最旧有效根)`）推它很难出现，所以不进推荐，记作丙的动机。

## 5　K4　条款文意

**5.1　两句谁先谁后，是否已经蕴含一种读法。** I-3.15 定义行里是一个分号连着的两句：「头用不了的那一格没有对象（读不出本身归 I-1.1、I-2.1）；隔离的单元（有一份副本读盘核出与位置项对不上」（`crates/singlefs-checker/invariants/05-空间记账.md:22`，后半句括号以下抄到此为止，原行更长）。第一句说**判的对象**：头用不了的那条记录自己不判红。第二句说**豁免的单位**：「单元」，条件是「有一份副本读盘核出与位置项对不上」。文字没排先后。实现是先套第一句（把记录过滤掉）再对剩下的记录做第二句：「.filter(|header| birth_txg_in_a_usable_unit_header(header).is_some())」（`crates/singlefs-checker/src/walk.rs:16380`）→「let devices_of_the_unit: Vec<u32> = records_of_the_unit」（`walk.rs:16466`）。

我的读法（推）：两句管的是两件不同的事，可以都成立——头用不了的那条记录不判红（第一句），但它所在的单元里这一份读出来对不上位置项，所以这个单元是「隔离的单元」（第二句），同一单元别的盘上的记录也不判。头用不了的字节拿去对位置项必然对不上（除全零槽恰好配着全零单元校验和那一种特例（读码，`contains` 判的是 CRC 相等）；退回自证时 `check_unit` 也不过），这一点调查员那 7 条逐条量到了（CRC 不等于位置项校验和）。所以字面上第二句已经蕴含了乙一的归组（组按单元，不按过滤后的记录）；今天的实现把两句的先后弄成了「先过滤再归组」，结果是**越坏越不红**：一份头用不了红（W1），两份头都用不了反而不红（W2，量）。这是一个不单调的读法，不像是条款要的。

**5.2　「隔离那一格照 I-3.1 那一份隔离判据」今天的实现做到没有。** 条款原句：「隔离那一格照 I-3.1 那一份隔离判据（`walk::some_copy_of_the_unit_fails_its_location_entries`），实现员 2026-10-15 推的读法，交主 agent 认」（`crates/singlefs-checker/invariants/05-空间记账.md:22`）。I-3.1 那一份在 `quarantined_slots_exempted_per_device`（`walk.rs:8395`），它的归组不看头：把树表里种类 3 那棵分配记录树的**全部**未释放记录，剔掉被引用的槽，按（起点槽、跨度）归组——剔的那一句是「if record.is_released || referenced_start_slots.contains(&(record.device, record.slot))」（`walk.rs:8439`）。它的文档把理由写明了：「按单元而不按份判：写者只要一份核出对不上就把两块盘的记录一起留在已分配」（`walk.rs:8388`）。I-3.15 复用了它的校验函数（`some_copy_of_the_unit_fails_its_location_entries`，`walk.rs:16223`），没有复用它的归组。结论：校验做到了，归组没做到；量：W1、W3 里同一张盘 I-3.1、I-3.11 成立、I-3.15 红。I-3.1 的「没有根引用」在 I-3.15 里对应的是「这一版映射里没有 key 点名这个槽」，这就是乙二那条限定的来历：I-3.1 的分支不会豁免被引用的槽。

**5.3　必红用例期望里的 I-3.15 是条款要的，还是写用例时按实现写的。** 两头都有：
- 按实现写的证据：这条用例 `fn a_shared_leaf_container_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device(`（`crates/singlefs-harness/tests/checker_known_bad_images.rs:15657`）里期望集合那行 I-3.15 和文档那句「A 那一版的账里叶容器那一槽的记录未释放、那一槽却装着诞生代号 9 的另一版叶」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:15655`）都是未提交改动：现查 `git blame -L15655,15655` 与 `-L15667,15667` 都是 `Not Committed Yet 2026-10-21`；`git show HEAD:` 里这条用例的期望集合里 I-3.15 命中 0 次；`git diff HEAD -U0` 里文档那句是新增行。即：I-3.15 随 N2 落地时按实现输出加进了期望集合。
- 条款也要它红的证据：N2 的原句「新判：N2 反查（每条未释放记录罩住的槽，头里的 key 在映射里的位置含这个槽；豁免类要被根或父指针直接指着）」（`research/prompts/m4-gc-r1-main-verification.md:34`），这条用例的槽里没有一份头与 key 相符，也不被根直指，N2 照字面要红。
- 它们在这条用例上字面冲突：同一个单元有一份 0xa5 改坏（第二句「有一份副本读盘核出与位置项对不上」成立）。冲突的根在「隔离」二字：D19 的隔离是**写者在释放时的处置**，「隔离并计数报出」之前的动作是「逻辑上照样释放（映射条目去掉）」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:109` 的「**核出对不上时隔离那个槽，发布照成**：逻辑上照样释放（映射条目去掉），物理槽不还回空闲池，记进隔离并计数报出。」），所以被隔离的单元在那一版里不该还有 key 点名；这条用例是 key 还点名着、却有一份坏，不是写者隔离出来的状态，而是活着的单元坏了。乙二把「当版没有 key」当作隔离的必要条件，就是把这个区分写进读法里；乙一没有。这是我推的读法，D19 与 I-3.15 定义行都没有把它写成一句话，要主 agent 或用户认（岔路 F1）。

**5.4　G9 的「不一致」与「头用不了没有对象」的张力（条款自己的）。** I-3.15 定义行写着「G9 的」…「两类判红」（原行：`第三节 G3 的 N2 反查、G9 的「已死」「不一致」两类判红`，引成「第三节 G3 的 N2 反查、G9 的…两类判红」，`crates/singlefs-checker/invariants/05-空间记账.md:22`）；G9 把「不一致」定义成「已死（有记录、映射里没人指向它——第一版只报告不动，N2 会红）、不一致（头读不出或两来源对不上）」（`research/prompts/m4-gc-r1-main-verification.md:40`）。按 G9 的字面，「头读不出」属于「不一致」，该判红；同一行定义的第一句又说头用不了没有对象。实现取了后者（头读不出归 I-1.1、I-2.1），G9 那半句没有对应的判。我读的是：N2 判红的「不一致」是「头读得出、两来源对不上」这一半，「头读不出」那一半归 I-1.1 / I-2.1，G9 是整理区域扫描的分类，不是不变量的判词（推）。这一句建议在定义行补一个「G9 的『不一致』里头读不出那一半不归这一条」。

**5.5　归类器的文档「插回去仍红照报新发现」与甲是否冲突。** 文档原句：「一条不是注入点造成的违例（插回去仍红）、一次读坏的槽不是被吞那次写的、一行与重算的对不上，照报新发现——这是判别力那一半。」（`crates/singlefs-harness/src/fault_injection.rs:2560`）。这句的前提是「插回去仍红 ⇒ 不是注入点造成」。在隔离这一形上前提不成立：注入点（被吞的写）→ 实现释放前核出对不上 → 两块盘的记录留在已分配 → 插回去之后两份都好、盘上看不出隔离、仍红。量：关掉实现的隔离（调查员的 `PROBE_NO_QUARANTINE`，只为造推翻条件）24 条全归说谎设备（`out/manual-runs.txt` 最后一行）。所以甲不是和那句冲突，是那句缺一个例外：「插回去仍红，但那是实现在被吞的单元上核出对不上、按 D19 留下的记录」。取甲二或甲三时这句文档要改；只取 I-3.15 那一半时不用改。

## 6　K5　改法与代价

行数是估的，「量」只指调查员补丁里能数出来的部分；GPU 与归类器的行数全是推的，没写没编。

| 处 | 乙二（推荐的 I-3.15 一半） | 乙一 | 甲二 / 甲三（归类器一半） |
|---|---|---|---|
| `crates/singlefs-checker/src/walk.rs` 的 `judge_unreleased_allocation_records_against_the_central_mapping`（第 16258 行起；归组在 16465 行起的 for） | for 里先算「这一版映射里有没有 key 点名这一槽（任一块盘）」，没有时 `devices_of_the_unit` 取这一版分配记录里这个（起点槽、跨度）上全部未释放记录的盘（不看头）；有时照旧。调查员补丁里这一块约 20 行（带开关），去开关加文档句约 +22 | 同上去掉「有没有 key」的判断，约 +12 | 不动 |
| `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `g_judge_the_unreleased_records_of_the_version_against_its_central_mapping`（6690 行起）；对拍 | 6726–6740 的标记循环保持（头用不了的仍 `continue`、不标）；6770 起的组循环里 `some_copy_fails` 的来源改成「无 key 时取该（槽，跨度）上全部未释放记录」：要照 `g_placement_is_accounted_for`（6661 行）的桶遍历加一个「有没有 key 点名这个槽」的小函数，推 +30–45 行。对拍：`POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 是空表（`crash_judge_conclusions.rs:438`），下标 95 内核置位、对拍逐位比，所以 CPU 与 GPU 必须同改，否则含这一形的状态上两边分歧（读码） | 同，少「有没有 key」那个小函数，推 +20–30 | 不动 |
| I-3.15 定义行（`crates/singlefs-checker/invariants/05-空间记账.md:22`） | 补三句：① 头用不了的那一份仍算进隔离判据的组，但只在这一版映射没有 key 点名这一槽时；② G9 的「不一致」里头读不出那一半不归这一条；③ 订正状态栏的这半句「GPU 判器下标 95：内核不置位、对拍不比，主 agent 合入时补登」（`crates/singlefs-checker/invariants/05-空间记账.md:22`）——它与「pub const POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS: &[usize] = &[];」（`crates/singlefs-checker-tier/src/crash_judge_conclusions.rs:438`）这张空表对不上 | 补①（去掉限定）与③ | 不动（只取归类器时 I-3.15 行不改） |
| 必红用例 `crates/singlefs-harness/tests/checker_known_bad_images.rs:15657` | 不动（量：乙二下全过） | 期望集合删 I-3.15 一行，文档句「…跟着红」改写；这样改等于认「一份坏就隔离」在 key 点名着的形上也成立 | 不动 |
| 夹具 | 新增一个「头用不了的隔离世界」场景（W1 或 W3 型，覆盖写一次，交 `hand_out` 并断言 I-3.15 / I-3.1 / I-3.11 全成立），放 `crates/singlefs-harness/tests/common_images_for_the_checker_tier/release_checksum_quarantine.rs`，我的探针里造这些世界的函数约 40 行，推 +45–60；该文件现有三个场景只断言 I-3.1、I-3.11（`only_the_named_invariants` 不断言没点名的，`crates/singlefs-harness/tests/common_images_for_the_checker_tier/mod.rs:113`），各补 I-3.15 的 holds / violated，共约 +5 行（量：三种读法下 I-3.15 与 I-3.1、I-3.11 同步，见 `out/summary.txt`） | 同 | 归类夹具：`fault_injection_fast_tier.rs` 一带（`common_images_for_the_checker_tier/` 里同名文件）加一条「吞写 + 隔离」的归类用例，推 +40 |
| 归类器 `crates/singlefs-harness/src/fault_injection.rs` | 不动 | 不动 | `WhatTheSwallowedWritesExplain`（`:2506`）加「实现核出对不上的单元清单」字段，`left_by_the_lying_device`（`:2566`）加分支，推 +40–60；文档句（`:2560`）加例外。清单要从 `history.rs` 把每一步 `versions_published`（`:4761`、`:5945` 等多处各自取）里的 `quarantined_after_release_checksum_mismatch` 累计，且跨重挂累计（`TransactionOutput` 自己不带跨重挂的：「从盘上重建的版本不是这个进程发布的，是空的」（`crates/singlefs-core/src/transaction.rs:3234`）；会话上另有 `CompactionOfThisMount::quarantined_slots()`（`compaction_plan.rs:638`）但也只管这次挂载），推 +40–70 |
| 池级 checker 交违例落点（只甲二要） | 不动 | 不动 | `Judgements`（`image.rs:473` 起）为 I-3.15 加违例落点列表而不是只留第一处（`image.rs:494`），推 +20–30；checker 档对插回去那张盘改 CPU 重判才拿得到细节（GPU 结论「不带细节」，`crash_given_states.rs:148`），推 +40。甲三另需 checker 加带豁免输入的入口，推 +40–70，归类调用处改 CPU 重判，推 +20–40 |
| 变异行与必红世界 | 两条新变异行（推，anchor 文本要到时现取，没按变异表文本替换跑过）：① 拿掉「有没有 key」判断（=乙一）→ 必须被 `crates/singlefs-harness/tests/checker_known_bad_images.rs:15657` 那条红（量：乙一开关下它红）；② 拿掉「头用不了的也进组」（=今天读法）→ 必须被新夹具红（量：今天读法下 W1、W3 红）。`crates/mutations.tsv:602` 那一行（「读得出而对不上不重读；故障注入快档判出（一次坏读隔离一对好槽，I-3.11 红）」）本身靠非说谎设备的读错注入点，`the_device_lies` 为假（`fault_injection.rs:190`）不走插回去那张盘，乙二不动它杀的东西（读码）；它的基线要等 7 条新发现清零才转绿 | ① 的反向：没有「有没有 key」那条，必红用例要改 | 归类变异：拿掉清单比对 → 归类夹具红（推） |

**只取一半会留下什么**

- 只取 I-3.15（乙二）：快档 24 段新发现 7 → 0、说谎设备 17 → 24（量，`out/summary.txt` 的 TALLY 行）。留下：① 名字比对的隐患——插回去那张盘因隔离仍红 I-3.11、I-3.15（7 个吞写点，量），若注入之后那张盘因别的原因也红这两条，名字相撞，被报成新发现（假警报，不是漏报）；② 盘上看不出隔离的形（两份都好）三条仍红，是第 1 节第 4 点的天花板，不是这一半能修的；③ 归类器文档句不用改。
- 只取归类器（甲二 / 甲三）：池级 checker 在 W1、W3 这种合法隔离盘上仍红 I-3.15（量，今天读法），I-3.15 与 I-3.1、I-3.11 的「按单元」口径仍不一致、读法不单调（5.1）。这个红只出现在「写者以为落了盘、盘上其实没落」的盘上，只有说谎设备与漏屏障造成（读码：普通掉电之后重开，实现读的是盘上真有的东西，不会再「以为落了盘」），所以别处判池级 checker 的地方（崩溃点重放、放量）遇不到；留下的是 checker 自己的不一致，和归类器多接一条跨重挂的清单。
- 两半都取：最完整，最贵（上表三列相加），多出来的一半只在大档里撞名字时才有用，现有取样里没有量到需要它的实例。

## 7　岔路单（候选、翻面观测、够判条件；不带倾向、不写已有的数）

| 岔路 | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| F1　I-3.15 里头用不了的那一份怎么算 | A 今天读法（先过滤再归组）；B 乙一（按单元归组、不限定，必红用例期望删 I-3.15）；C 乙二（按单元归组，仅当这一版映射没有 key 点名这一槽）；D 丙（C 再加：对位置项校验和改成这一版映射里点名这一槽的那一条，不是各版的集合） | 把必红用例里 I-3.15 去掉之后，该镜像上还有没有某一个只有 I-3.15 能红的世界（逐条关掉别的不变量看红不红）；找到一段合法历史，其中当版 key 点名着隔离的单元、且有一份坏、且记录不该红 | 用户定「N2 在 key 点名着、一份坏的活单元上还要不要红」这一句；加上「只有 I-3.15 能红的世界」的个数量出来（0 个 / 至少 1 个） |
| F2　归类器要不要补 | A 不补，只比不变量名；B 甲（粗，只看第一处说明）；C 甲二（单元落点 + 实现的隔离清单）；D 甲三（带清单重判插回去那张盘） | I-3.15 改后，在更大的取样（大档、更多种子、含漏屏障类注入）里，出现「插回去仍红的名字与注入之后红的名字相同、而红的原因不是同一件事」的新发现；或出现一条真缺陷因名字比对被放成说谎设备的实例 | 大档故障注入判（重型，主 agent 跑）一次：新发现里签名含 I-3.11 / I-3.15 的条数，和这些条里插回去那张盘的违例原因是否都是实现的隔离；条数为 0 则 A 够，非 0 则要在 B / C / D 里挑 |
| F3　盘上看不出隔离的形（两份其实都好）怎么办 | A 接受（I-3.1、I-3.11、I-3.15 在该形上红，写进条款）；B 隔离时把标记落盘（格式改动，哪一位 / 哪个字段另议）；C 只在归类器这一侧用清单（=F2 的 C 或 D）认 | 真实使用里「瞬时坏读两次」让合法盘被永久隔离的出现频率；或用户对「合法盘上池级 checker 红」的容忍线 | 用户定容忍线；若取 B，先要一个格式改动的代价数（规则 `three-way-inference.md`「岔路交用户定之前，每条路要有代价数」，这条路今天没有任何代价数，要先建计数模型） |
| F4　GPU 判器与 CPU 判器同改的批次 | A 同一批改；B CPU 先改、GPU 随后（对拍在两者之间分歧） | 对拍在含「头用不了的隔离」状态上的逐位比结果 | 不是用户岔路，是主 agent 的批次安排；写在这里是因为只改 CPU 一侧 checker 档对拍会分歧（读码，没跑 checker 档） |

## 8　每条结论与什么现象会推翻它

| 结论 | 什么现象会推翻它 |
|---|---|
| 今天读法在合法的隔离盘上误红（K2、K4：W1、W3 的盘面是合法隔离，不是缺陷） | 条款作者（用户）明说「头用不了的那一份就该让 I-3.15 红」，或找出 D19 里一句话说被吞（头用不了）的那一份不属于「有一份副本读盘核出对不上」。我没找到，已读 D19 已定项 5 全节 |
| 今天读法不单调（一份头坏红，两份头坏反而不红，W1 与 W2） | W2 在别的不变量上红了、而 I-3.15 的成立是靠它兜住的（量：W2 里 I-2.1、I-4.8、I-7.4 红，I-3.15 不红；若谁认为「两份都坏该红 I-3.15」，这条不单调论就变成「今天读法在 W2 上放过」，方向相反，读法选择要重议） |
| 乙二比乙一好在不放过必红用例形（K3） | 找到一个合法历史：当版 key 点名着隔离的单元、且有一份坏、且 I-3.15 不该红。我用回退复活造过一段（探针里标「写回原样后回退到 A」的世界），当版有 key、头相符、成立，没推翻；若有一段历史里当版 key 点名着、头也对得上 key、却有另一份坏——这在乙二走今天读法的分支上是成立的，也不推翻。真正会推翻的是「当版 key 点名着、好的那份头与 key 不符、另一份坏、而这是合法的」这一形（位置项过期的提示？）——我没找到，但没穷举 |
| 乙一放过必红用例（K3） | 该用例红这一条是量的：四个 harness 目标里的 `a_shared_leaf_container_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device`，乙一开关下红、乙二与今天读法下过。要推翻需要在同一棵树上同一开关下它不红 |
| 泄漏加一份坏与合法隔离是同一份字节，读盘分不开（K3） | 找到一个只读盘的判别特征：例如隔离留下的记录在盘上有别于泄漏的记录的字段（分配代照旧 3 之类，量到泄漏记录的字段与隔离记录的字段不同）。我读到隔离记录「分配代照旧」（`release_checksum_quarantine` 用例文档），泄漏的记录没有造出来比，未验 |
| 只取 I-3.15 在现有快档取样里够用（K5） | 更大档里出现签名含 I-3.11 / I-3.15 的新发现，且插回去那张盘的红来自实现的隔离（名字相撞）；或出现一条因名字比对被放过的真缺陷 |
| GPU 判器必须与 CPU 同改（K5） | 对拍表里 I-3.15 不比（`POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 非空）；现在是空表，是读码，没跑 checker 档 |
| 「I-3.15 定义行状态栏『GPU 判器下标 95：内核不置位、对拍不比』与实现对不上」 | 在 `crash_judge_conclusions.rs` 的空表之外另有一处排除下标 95 的地方（我查了该文件的说明与表，没找到） |

## 9　命令与原样输出

手跑（在副本里，同一个 `tree/`，已删；`out/manual-runs.txt` 有原样汇总行）：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的四个测试目标…; 范围=4 个测试目标; 为什么=…' <开关> nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G cargo test --release --no-fail-fast -p singlefs-harness --test checker_known_bad_images --test checker_compaction_intent_and_relocation_must_red_worlds --test checker_cross_links_malformed_nodes_and_mapping_entries --test checker_narrow_invariants_and_abandoned_roots
SINGLEFS_RUN_SCOPE='…' <开关> nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-checker --lib
SINGLEFS_RUN_SCOPE='…' <开关> SINGLEFS_PROBE_I315=1 PROBE_I315_SLOT=50180 nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 3G cargo test --release -p singlefs-harness --test probe_i315_scenario_images isolated_unit_repaired -- --nocapture
```

`<开关>` 取不设（今天读法）、`PROBE_FIX_GROUPING=1`（乙一）、`PROBE_FIX_GROUPING_WHEN_NO_KEY=1`（乙二）。原样结果（取自 `out/manual-runs.txt`）：今天读法 12 / 14 / 270（2 ignored）/ 20 全过；乙二同；乙一 12 / 14 / 269 过 1 红 / 20，红的是 K3 第一行那条用例；`singlefs-checker --lib` 三种开关各 99 过。

## 10　判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1　读法的定义 | 规则没说 | 条款只写了两句，没选读法；今天读法（先过滤再归组）与「隔离那一格照 I-3.1」冲突；乙一、乙二、归类器甲二 / 甲三的定义、组取法、头用不了的算什么、输入来源见第 2 节两张表 |
| K2　合法盘与说谎设备的盘上会不会误红 | 冲突 | 今天读法在合法隔离盘（W1、W3，7 条同形）上误红；乙一、乙二不误红；盘上看不出隔离的形（两份都好）三种读法一律红，读盘解不了；乙二的依据「之后各版没人指着」字面不成立（回退复活造出了反例，量），但乙二用的是当版事实，结论不受影响 |
| K3　坏盘上会不会放过 | 一致（乙二）/ 冲突（乙一） | 乙一让必红用例 `a_shared_leaf_container_born_after_the_rollback_target_with_a_garbled_copy_…` 红（量），放过「key 点名着、头对不上、一份坏」；乙二不放过；泄漏加一份坏与合法隔离是同一份字节，三种读法都放过，别的不变量兜不住（量：W3），与 I-3.1、I-3.11 今天的口径一致；归类器甲粗放过第二处违例，甲二 / 甲三放过「清单里的槽上的泄漏」 |
| K4　条款文意 | 冲突 | 第二句字面已蕴含按单元归组（今天读法读成了不单调）；「照 I-3.1」归组没做到；必红用例的 I-3.15 是 N2 落地时按实现加的，N2 字面也要它红，与第二句在这条用例上冲突；G9 的「不一致」含头读不出，与定义行第一句张力；归类器文档句缺例外而非冲突 |
| K5　改法与代价 | 规则没说 | 乙二：`walk.rs` 约 +22、GPU 推 +30–45、定义行补三句、新夹具推 +50–65、两条变异行（推）；归类器甲二 / 甲三推 150–250 / 110–200 行；只取 I-3.15 的这一半快档新发现 7 → 0（量），名字相撞隐患留着；两半的取舍与四条岔路见第 6、7 节 |

## 11　没做什么

- 没替主 agent 采纳、没出判决；第 1 节的「推荐」是正推腿按立场分工写的组合，岔路 F1–F4 不带倾向。
- 没改主工作区一个字；乙二、甲二、甲三都没有写进 `crates/`，数都在副本里用调查员补丁的环境变量开关量的。
- 没跑 checker 档包（含 GPU 判器与对拍）、没跑 `crates/mutations.tsv` 第 602 行的基线或任何变异、没跑大档故障注入；GPU 一侧的改法与行数全是读码推的；7 条在 checker 档全量用例里是 GPU 还是 CPU 判出的 I-3.15，复核不了（要 checker 档）。
- 没对 `LocationEntryChecksumsForQuarantine` 的「各版校验和集合」疑点（第 4 节末）造出例子；没穷举「当版 key 点名着、头与 key 不符、一份坏」的合法历史。
- 没量甲粗、甲二、甲三任何一个（只量了调查员关隔离的 24 条说谎设备那一次，`out/manual-runs.txt` 最后一行）；I-3.11 的逐盘差额等于隔离清单槽数之和是推的，没量。
- 没量「搬迁的源被隔离」那一形（强制开关下），只读码；没量读盘核两次读结果不同类。
- 没做格式改动方案的代价数（F3 的 B）。
- 没动 `.claude/agents/`、`.claude/rules/`、不碰 `~/.claude/`；没做 git 写操作。

## 12　删掉的东西

副本 `~/.local/share/singlefs/work/m4-i315-r1-forward/tree`（删前 `du -sh` 1.8G，含编译目录）、两次复跑的副本 `rerun1/tree`、`rerun2/tree`（各 963M，含编译目录）都已 `rm -rf`，删前 `ps` 查过没有进程开着。草稿目录剩日志（`*.log`）与 `rerun*/out/`，合计 980K。
