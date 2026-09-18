# 背景材料：里程碑「第二个事务」第二波代码改动的代码三方对抗第二轮（2026-09-18）

<!-- doc-lint:not-numbers V1 V2 V3 V4 V5 V6 Z1 Z2 Z3 Z4 Z5 Z6 -->

## 一、被判的对象

第一轮判决 `research/prompts/m2-wave1-code-r1-main-verification.md` 判的三处改法，加上它之后叠上去的四批改动。工作区没有提交点，按「文件::项名」给（门禁 56 号按路径认）。**开工快照**：`research/prompts/m2-wave2-code-r1-start-snapshot.sha256`（17 个文件，2026-09-18 01:5x UTC 记；`.claude/rules/implementation-workflow.md`「代码轮派腿之前记一份开工快照」）。

| 文件 | 项 |
|---|---|
| `crates/singlefs-core/src/mount.rs` | `MountError::RowPublishAdmissionRefusedBeforeAcquisition`、`MountError::WarmUpAdmissionRefusedBeforeAcquisition`、`refuse_publishes_before_acquisition_that_do_not_pass_admission`、`warm_up_publish_txgs`、`establish_instance`（取号之前算准入、暖机按计划推） |
| `crates/singlefs-core/src/transaction.rs` | `PublishShape`（含 `ROW_PUBLISH`、`EMPTY_PUBLISH`）、`publish_admission`、`admission_of_one_publish`、`publish_sequence_admission`、`PublishSequenceRefusal`、`PoolWriter::writes_of_failed_publishes` 与 `count_failed_publish`、两处 `persist` 闭包、`publish_first_file` 的 `change_count` |
| `crates/singlefs-core/src/allocator.rs` | `cluster_segments` 与它的访问器、开段登记、回落只清 `open_segment`、`lowest_user_data_slot` 排除全部聚簇段 |
| `crates/singlefs-checker/src/walk.rs` | `judge_release_generation_and_tree_table_birth`、`references_of_root`、`candidate_indexes` |
| `crates/singlefs-checker/src/image.rs` | `IMPLEMENTED_INVARIANTS`（26 → 28） |
| `crates/singlefs-harness/src/` | 新模块 `first_transaction_regions.rs`（`FIRST_TRANSACTION_REGIONS` 21 行）、`sha256.rs`、`hexadecimal.rs`、`bin/first_transaction_region_bytes.rs`；`lib.rs` 三行模块声明 |
| 用例 | `second_transaction_supplement_two_row_publish_admission.rs`（暖机两条）、`second_transaction_supplement_one_write_accounting.rs`（失败重试那条）、`checker_known_bad_images.rs`（`each_c374_bad_image_reddens_only_its_own_invariant_after_the_overwrite`）、`first_transaction_region_bytes.rs`、`first_transaction_step_five_publish.rs` 与 `second_transaction_step_five_reuse.rs` 改过的断言 |
| `crates/mutations.tsv` | 第 101–120 行 |

**实现今天的样子**（主 agent 读过的 `crates/` 路径与看到的事实，都是观测；方案按 `crates/` 今天的实现来谈）：

- 取号之前的准入（`mount.rs` 的 `establish_instance`）：先按 `PublishShape::ROW_PUBLISH` 与 `EMPTY_PUBLISH` 算写行那一次加 1–3 次暖机空发布要用的记账行数与分配记录条数，算不过返回两个新错误成员之一，一个写都不发；`warm_up_publish_txgs` 让准入与之后真推发布共用同一份计划。⚠️ 实现员量出：这套算术是上界（每次按「角色数 × 盘数」加、连乘几次），而真发起来重开之后释放掉的落点是改写不是追加——它量的那个池按上界在暖机第一次就被拒（准入按 814 算），真实条数 796 → 798 → 798、容量 812。
- 失败路径的写量账（`transaction.rs`）：`PoolWriter::writes_of_failed_publishes` 一次失败存一份、不并进成功发布的账，两条落盘路径收进闭包、失败先记账再原样交回错；调用方要主动取。真设备二进制与 harness 别的调用点在发布报错时整程序退出、不取。
- 回落之后那一段（`allocator.rs`）：`cluster_segments` 记这次挂载开过的每个聚簇段，回落只清 bump 目标、登记留着，`lowest_user_data_slot` 排除全部聚簇段。这张表只住内存，重开之后从分配记录重建时是空的。
- checker 新判两条（`walk.rs`）：按候选集里每条根各走一遍 `references_of_root` 取引用集合、树表条目与分配记录；I-3.9（释放代落在停止引用它的那一格区间里） 判区间（不是「等于」——主 agent 原措辞被残留记录那条流的 12 个合法状态证伪）、I-9.14（树表条目的诞生 txg 跨根不变） 按树 ID 归并比；候选集少于两条根、或条目只出现在一个树表单元里时报「不适用」。层 0 fast 那条用例 6.17s → 8.01s。
- 区域字节导出（`crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs`）：只读，跑 `scenario::run_first_transaction` 之后按 21 行区域表打结果行，另有一行把表与录制流里第一个事务那一段按 (设备, 偏移, 长度) 多重集配对，对不上退出码 1。16 个单元那几行只打 sha256 与前后 32 字节。
- 改动计数：`publish_first_file` 写 `FIRST_TRANSACTION_TXG`（3），E142（第一个事务的干跑） 第十一次跑量出盘上共 67 段字节跟着变、全部归因，另有 `journal_record` 与 `mapping_root` 两个结构也变。
- `crates/` 里没有的：回卷已烧掉的实例代号；失败账的自动回收（要调用方取）；`cluster_segments` 的盘上表示；`first_transaction_region_bytes` 的整段十六进制模式（16 行只有摘要，触发失配时定位不了第一处差异）。

## 二、改法（每条都是可以被攻的推论）

| 编号 | 改法 | 压着的条款（原文在附录一） |
|---|---|---|
| V1 | 写行与暖机那几次发布要用的准入都在取号之前算，算不过一个写都不发 | D18（块里携带什么信息） 已定项 11（取号全或无）；D16（发布语义） 已定项 8 / 已定项 9；D28（挂载期承诺量） 已定项 4 |
| V2 | 失败的发布把这次已记的写单独交出去，与成功那次分开 | 里程碑「增补 1」验收第一条；D17（实现分层与第三方管道） 已定项 2 / 已定项 5 |
| V3 | 本次挂载开过的聚簇段一律不给用户数据 | D3（空间分配） 已定项 8 第 2 条、已定项 10 |
| V4 | I-3.9（释放代落在停止引用它的那一格区间里） 判区间、I-9.14（树表条目的诞生 txg 跨根不变） 按树 ID 归并，两条都在候选集不足时报不适用 | I-3.9（释放代落在停止引用它的那一格区间里）、I-9.14（树表条目的诞生 txg 跨根不变） 两行；D3（空间分配） 已定项 7；D8（核心索引结构） 已定项 8 |
| V5 | 第一个事务的改动计数写发布时的 checkpoint_txg（3） | D8（核心索引结构） 已定项 6 |
| V6 | `crates/` 侧按 21 行区域表导出字节，另有一行把表与录制流配对 | `.claude/kb/layout/01-first-txn.md` 零那一节；`.claude/rules/implementation-first.md` 第 4 条 |

## 三、判据（跑前写死；每条写「什么观测会让它触发」）

| 判据 | 问 | 触发的观测 |
|---|---|---|
| Z1 上界准入 | V1 的上界算术会不会拒掉真发得起来的池（实现员已量出一例）；抵扣可复用的已释放记录之后还会不会拒错；反过来，有没有池按上界放行、真发起来却装不下（漏判） | 一段可达历史按上界被拒而真跑得完，或按上界放行而真跑起来撞断言 |
| Z2 失败账 | V2 的账在「失败之后不重试、直接换一条路」「同一次挂载里连着失败两次」「失败发生在最后一步超级块槽写」几种走法上还对不对；调用方不取时那份账去哪了 | 一段历史让按种类的合计与设备一层不等，或失败账被下一次成功发布吞进去 |
| Z3 聚簇段登记 | V3 那张只住内存的表：重开之后为空，用户数据可以落进上次回落留下的段——这一格今天可不可达；同一次挂载里回落两次、或回落之后又开新段时，表会不会把整个单元区都登记成聚簇段而用户数据无处可落 | 一段历史让用户数据落进装着提交内生块的段，或让用户数据分配不出来 |
| Z4 两条新不变量 | V4 判区间之后还漏不漏：区间在洞很长时会松到判不出错写的释放代（C380（环上有洞时释放代判不出唯一值） 已立）；「不适用」那两格会不会把本该判的状态放过；`references_of_root` 与主走读的 `visited_units` 去重口径不同，会不会让同一个单元在两处算出不同的引用集合 | 一份坏镜像让两条新不变量全绿，或一个合法镜像让它们红 |
| Z5 改动计数与区域表 | V5 之后 E142（第一个事务的干跑） 报的 67 段差异有没有漏（`journal_record`、`mapping_root` 之外还有没有）；V6 那张 21 行表与 `.claude/kb/layout/01-first-txn.md` 零那一节逐行对不对得上，换几何时 `region_table_against_writes` 拦不拦得住 | 一处结构变了而 67 段里没有它，或表与字节表对不上而检查不红 |
| Z6 写回 | 收口表第 ②、11、12、19、20a–20d、21、22、23、23′、23″、28、34′ 各行与代码、用例、变异表、`.claude/kb/invariants.md` 的 I-3.9（释放代落在停止引用它的那一格区间里） / I-9.14（树表条目的诞生 txg 跨根不变） 两行对不对得上 | 一句与代码不符 |

**反向接受条款**：Z1 / Z2 / Z3 / Z4 打中 ⇒ 改代码、补会红的用例与变异、记进收口表；本轮不再攻第三轮（`.claude/rules/three-way-inference.md`「三轮之后停」按同一条精神，这一批第一轮已攻过一次）。Z5 打中且落在条款没写的地方 ⇒ 记进收口表、标预想交用户。Z6 打中 ⇒ 改里程碑文字。
**失败条款**：一条腿引的条款与附录原文不符 ⇒ 那一格作废；「没打中」一次不算（本地腿抽两次）；腿在副本上量出的数不进 kb。

## 四、腿的分工（三条推论腿，攻击面不重叠）

| 腿 | 立场 | 分到的格 | 不许碰的格 |
|---|---|---|---|
| Opus 攻方（`three-way-attack`） | 造可达的历史打穿 V1–V3 | Z1、Z2、Z3 | Z4 |
| 本地攻方（`three-way-local-attack`，英文，抽两次） | 按事实表逐格算准入的上界与真实条数、区间判据的边界 | Z4 的算术那一半（区间端点、不适用的两格各在什么条件下出现） | Z1、Z2、Z3 |
| Sonnet 正推（`three-way-forward`） | 核 V4–V6 的代码与用例做的是不是条款说的 | Z4 的条款那一半、Z5、Z6 | 不替攻方找反例 |

## 五、禁读清单（各条腿）

这一轮别的腿的输出（`research/prompts/m2-wave2-code-r1-*-output*.md`）与主 agent 的核实（`research/prompts/m2-wave2-code-r1-main-verification.md`）。前几轮判决可读：`m2-wave1-code-r1-main-verification.md`、`m2-step6-checker-r1-main-verification.md`、`m2-step45-code-r3-main-verification.md`。本地腿的提示避开 Rust 路径 `::`、不写汉字变量名。
