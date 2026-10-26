# m4-validate-code-r1 判决：里程碑四截断落地、inode 截断字段与合回解冲突的代码轮（主 agent，2026-10-22，第一轮）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 K1 K2 K3 K5 K6 K8 K12 R2 R3 R6 S1 -->

被判的产品代码（冻结副本 `~/.local/share/singlefs/work/m4-validate-code-r1/repo/`，HEAD 841b3beab 导出）：`crates/singlefs-core/src/truncate.rs`、`crates/singlefs-core/src/extent_tree.rs`、`crates/singlefs-core/src/write_request_split.rs`、`crates/singlefs-core/src/mounted_read.rs`、`crates/singlefs-core/src/records.rs`、`crates/singlefs-core/src/system_configuration.rs`、`crates/singlefs-core/src/make_filesystem.rs`、`crates/singlefs-core/src/inode_tree.rs`、`crates/singlefs-core/src/lib.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/logged_operations.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-format/src/lib.rs`、`crates/singlefs-reference-filesystem/src/lib.rs`。

## 一、核查

- 开工快照：冻结副本里 `sha256sum -c research/prompts/m4-validate-code-r1-snapshot/crates-sha256.txt` 18 行全 OK；仓根 `sha256sum -c research/prompts/m4-validate-code-r1-snapshot/kb-sha256.txt` 全 OK。腿跑着的时候被判文件没被改过，不用倒推。
- cite-check：`--root` 取冻结副本，「核了 237 处引文、4 处裸行号，对上 190 处，没判 51 处」，对不上 0 处；没判的 51 处都是「路径认不出」（kb 与 `research/` 下的文件不在冻结副本里）。再以仓根为 `--root` 跑一遍（kb 与快照一致，见上一条），「对上 237 处，没判 4 处」，对不上 0 处。剩下 4 处我在冻结副本里逐行看：`system_configuration.rs:31` 是「/// incompat 位 2 = inode 记录带截断字段……」、`:35` 是「pub const INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT: u8 = 0x04;」、`truncate.rs:122` 是「if data_units_existing_before_the_intent > data_units_a_size_needs(target_size_in_bytes) {」、`crates/singlefs-core/src/transaction.rs:8412` 是「truncation_field: InodeRecordTruncationField::NOT_WRITTEN,」，与腿写的相符。
- 复跑：攻方腿的模型拷进 `~/.local/share/singlefs/work/m4-validate-code-r1-verify/opus-model/` 经内存包装跑，七组 `same_as_model_out=yes`（main、cold-walk、cold-recovery、reference-filesystem 四组退 101，other-changes、slots、rollback-histories 三组退 0），与报告「要主 agent 跑的命令」一节写的退出码相同。正推腿的模型拷进同目录 `sonnet-model/` 复跑（`RERUN_STAGES` 默认两段），三行 `name=probe_pure same_as_model_out=yes`、`name=model_check same_as_model_out=yes`、`name=probe_intent_slots same_as_model_out=yes`，槽数探针两行 `slots_outside_allocation_record_tree=11`（3 个单元）与 `=12`（150 个单元）与报告相同。
- 打中的行现查（冻结副本）：`truncate.rs:145`「.expect("第一个文件至少一个数据单元"),」、`truncate.rs:241`「(1..=data_units_before).contains(&data_units_after),」、`recovery.rs:6087`「content.resize(」、`crates/singlefs-reference-filesystem/src/lib.rs:642-645` 是 `truncated.resize(` 后接「.expect("64 位上 usize 与 u64 同宽：截到的长度装得进"),」、`crates/singlefs-harness/src/history.rs:676-677` 写「都装得进内存（至多现在的 size / 加 4 个净荷）」。
- 位 2 登记现查：`.claude/kb/decisions/15-格式冻结政策.md:82`「| incompat | 2..255 | 未分配；…」，`.claude/kb/feature-bits.md:19`「incompat 分了位 0（退役）与位 1，别的位一位都没分」；代码里 `system_configuration.rs:35` 与池级 checker `crates/singlefs-checker/src/lib.rs:280` 都用 0x04。
- 背景材料第二节有一处写错：位 2 常量写成在 `crates/singlefs-format/src/lib.rs`，实际在 `crates/singlefs-core/src/system_configuration.rs:35`。两条腿都发现了，都按实际位置判，不影响判定。

## 二、逐格判定

| 格 | 正推腿 | 攻方腿 | 判定 |
|---|---|---|---|
| T1 截断意图条目与 logged_ops | 一致 | 没打中 | 一致 |
| T2 先落目标与分批 | 一致 | 分批没打中，改 size 入口的 panic 归 T5 甲 | 一致（panic 那一处按 T5 判） |
| T3 尾单元与 extent 计划 | 一致（1140 行重写判定与独立模型逐行相同） | 没打中 | 一致 |
| T4 快照下截断 | 一致（快照测试过） | 没打中（只读码） | 一致；攻方「没打中」只有一次抽样、没造快照用例，这一格的「没打中」不单独支撑结论，靠正推腿跑过的快照测试 |
| T5 变长、洞与读 | 一致 | 打中两处 | **打中**。甲：标签 0 配 size > 0 的第一个文件，条款认它合法（K5、S1），三处读者都收，可写挂载之后改 size，三种 N 都在 `truncate.rs:241` 或 `:145` panic，不是类型化拒绝。乙：变长到 2⁴⁰ 字节（K12 上限之内）后，挂载态读者读得出，冷走读与冷恢复在 `recovery.rs:6087` 按 size 整份分配，进程中止。正推腿没有造这两种输入，它的「一致」与攻方的打中不冲突 |
| T6 截断字段与位 2 | 字段与机制一致，位 2 登记不一致 | 打中一处（登记） | **打中（登记）**。字段、五处构造与读者收 0x02、0x06 都与 R2 相同；位 2 在 D15（格式冻结政策） 已定项 4 登记表与 `feature-bits.md` 都还写没分配。判决 `m4-trunc-r2` 第六节写的是「位号与登记归阶段同步」，是没做完的写回，不是代码越界 |
| T7 恢复与扫描重建 | 与 R6 分派一致，格的前提不成立 | 判不了，前提不成立 | 格的前提不成立：核心层 `rebuild_version_*` 是沿树重建，不按单元头认领；读法 Z 的四条只在池级 checker `crates/singlefs-checker/src/rebuild.rs`，不在这一轮的 diff 里。正文 T7 那一格写错了对象，这一格不判 |
| T8 回退 | 一致 | 没打中（量过两种目标、两种变长） | 一致 |
| T9 准入 | 一致（两个形状 11、12 槽） | 没打中（六种几何 8–14 槽） | 一致；深树上界正推腿推出 25、比 24 多 1，没量，交回的说法是极限形状在真池里不可达，推的，没量过，记进第四节 |
| T10 RefFS | 一致 | 打中一处 | **打中**。RefFS 把截断施加成整份 `resize`，截到 1 TiB 中止；随机历史的截断目标至多「现在的 size 加 4 个净荷」，三方对拍碰不到大变长。和 T5 乙同一类病：合法的大 size 让参照一侧中止 |
| T11 合回解冲突 | 一致 | 没打中 | 一致 |

## 三、正推腿报的「判决没写、代码做了」六处

正推腿第 12 节只报现状。我逐条判去向：

| 处 | 判 | 去向 |
|---|---|---|
| 可写挂载把不带位 2 的旧池就地升位（`transaction.rs:626` 的注释，有测试钉住） | 判决没写，做法与「mkfs 起就置上」同向；格式在第一个外部用户之前是软的 | 写回时进 D15（格式冻结政策） 已定项 4 位 2 那一行的含义：「可写挂载取号那一写起带上」 |
| 几个默认包装（`mounted_read.rs:366`、`recovery.rs:3110` 等）按 `WRITTEN_BY_THIS_WRITER` 收截断字段，不看池的系统配置 | 线索：不带位 2 的池在这几条路上会被当成带位 2 读，偏移 120–135 非零的记录本该判不可用而被收下。没量过，也不知道这几条包装是不是只在测试里用 | 下一批先查调用点与能不能造出这样的镜像，再定改不改 |
| 第一个文件之外的 inode 改 size（N ≠ S）一律拒 | 条款只定了第一个文件；拒成类型化错误，没越界 | 不改；留作里程碑规划的现状 |
| 要重写的尾单元在上一版读不出字节就拒 | 在任何写之前拒，类型化 | 不改 |
| 续做时条目点名的不是现行记录就拒 | 只在坏镜像或外来镜像上可达，类型化 | 不改 |
| 标签 3、4 没绑位 2；预演只走 `deleted_inodes` | 推的，没造过镜像 | 记进下一批的调查清单，和第二处一起看 |

## 四、交用户

| 岔路 | 候选 | 我推荐 | 被攻过几轮 |
|---|---|---|---|
| T5 甲：标签 0 配 size > 0 的第一个文件怎么改 size | 子：`publish_size_change` 在取 txg 之前交类型化拒绝（新的 `TruncationRefusal` 成员），这一形只能覆盖写；丑：让 0 个单元的文件走通（变长与 N = S 只改记录、标签留 0，截短只改记录），要改 K5 的「变长」一句与 `truncate.rs` 两处断言的前提；寅：读者不再收这一形 | 子：只消掉 panic，不动条款，写者今天也写不出这一形 | 零轮（三个都是攻方腿提的） |
| T5 乙与 T10：合法的大 size 让冷读与 RefFS 按 size 整份分配 | 子：冷走读、冷恢复与 RefFS 不再交整份字节，交「单元内容加尾部洞长」；丑：把 K12 的上限收窄到装得下的数；寅：冷读遇到超界的 size 交类型化拒绝 | 子：K12 的上限不动，读的一侧改表示；checker 档调用 `recover` 的装置跟着改 | 零轮 |

两条代价都没量：改动范围是推的，子方案的 API 改动波及 checker 档调用方几处，靠 `grep -rn 'recover(&'` 数，没逐处核。

随机历史的截断目标至多「现在的 size 加 4 个净荷」：不管选哪条，修的那一批都要给随机历史加一档大变长（跨过冷读与 RefFS 会分配的量级），三方对拍才碰得到这一格。

## 五、写回与后续

- 位 2 登记：D15（格式冻结政策） 已定项 4 登记表与 `.claude/kb/feature-bits.md` 各加一行，含义对上 `system_configuration.rs:31`「incompat 位 2 = inode 记录带截断字段」，并进写回批。
- T5、T10 两个岔路由用户定案后并进下一批实现，改完照「写与验分两段」整体再验，再开这一轮的第二轮，攻改法本身。
- 第三节第二、第六处并进下一批调查。
- T9 深树上界（推出 25 比 24 多 1）记进延后项，等有深树装置时量。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D15（格式冻结政策） 已定项 4 | 支撑 | 2026-10-22 不受影响：位 2 用户 2026-10-17 已定，登记行没写是写回欠着，并进写回批 |
| D8（核心索引结构） 已定项 6 | 不影响 | 2026-10-22 不受影响：偏移 120–135 的截断字段与五处构造都与 R2 相同 |
| D18（块里携带什么信息） 已定项 10 | 不影响 | 2026-10-22 不受影响：读法 Z 第四条只在池级 checker，这一轮不判 |

## 用户定案

- 第四节第二行（合法的大 size 让冷读与 RefFS 按 size 整份分配）：用户 2026-10-22 定这一版不改，照现在的方式读，放到后面的优化里程碑。原话「这个没必要管就按照现在的方式读。 之后会优化 没有必要现在优化。 肯定不可能现在这种方式冷读」「现在这个样子冷读肯定是不行的 放到后面的优化里程碑里面」。欠账记 C878（冷读与 RefFS 按 size 整份分配内存）。现查：核心层 `mount.rs`、`mounted_session.rs` 里零处调 `recover(&` 与 `walk_to_file(`，冷读的调用方都在 checker 档与 `recovery.rs` 自己里；攻方腿量到的可写重挂截回做成，挂载那一路不受这一格影响。
- 第四节第一行（标签 0 配 size > 0 改 size panic）：用户 2026-10-22 弹窗选「丑：写者接得住这一形 (Recommended)」。调查员（报告 `~/.local/share/singlefs/work/m4-investigate-t5-tag-zero/report.md`，sha256 f0f43483…）量到：今天写路径写不出这一形（约 5800 次发布里 0 次）；今天的代码上三种 N 都 panic（`truncate.rs:240`、`:145`）；这张盘上重写上段叶的发布会丢掉标签 0 条目（`crates/singlefs-core/src/transaction.rs` 里第一个文件零单元那一臂 `(None, None) => None`），写出的那一版三处读者都拒。丑的改法因此连带那一臂改成照写标签 0。被攻过零轮，改完开这一轮的第二轮。
