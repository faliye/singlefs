# m3-unlink-code-r2 云端正推腿报告（Sonnet，格 K1、K3；2026-10-03）

正推：从第一轮两份判决行与 kb 条款推到冻结副本里的代码行、测试与变异行。「冻」指冻结副本 `/tmp/claude-1000/m3-unlink-code-r2/tree/`，引到的代码行号都是冻里的；kb 行号是今天仓里那份 kb 文件自己的（现查）。模型与脚本在 `research/prompts/m3-unlink-code-r2-sonnet-model/`（README.md 逐个列了）。

## 0 底座与复跑

- 冻里 21 份与快照对得上：`sha256sum -c research/prompts/m3-unlink-code-r2-snapshot/crates-sha256.txt | grep -c ': OK'`（在冻根下跑）输出 `21`。
- 草稿副本 `/tmp/claude-1000/m3-unlink-r2-forward/tree/`（rsync 自冻），只跑 harness 档单个测试二进制，每条经 `run-with-memory-cap.sh` 与 `capped.sh 6`；checker 档没跑。未改动副本上的基线（日志副本在 `research/prompts/m3-unlink-code-r2-sonnet-model/logs/runs/` 的 `run-*.log`、`base-*.log`；同名被后来的探针重跑盖掉的，以下表贴的结果行为准）：

| 测试二进制（过滤） | 结果行 |
|---|---|
| admission_raises_the_floor_before_refusing（全部） | `test result: ok. 14 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 53.21s` |
| unlink_mount_continuation_and_the_emptied_version（全部） | `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 58.84s` |
| unlink_in_histories_judged_by_the_model（rolling_back_to_the_intent，三条新用例） | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 21.27s` |
| unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.06s` |
| a_floor_raise_refused_for_space_counts_as_short_of_space（U3 一条） | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.06s` |
| an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping（LR3、LR2 两条） | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 164.14s` |
| writable_mount_of_a_formatted_pool（LR6 一条） | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 77.78s` |
| rollback_by_a_forward_publish（D 段序列一条） | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 72.48s` |
| named_unit_verification_stands_in_for_the_barrier_before_the_record（全部三条） | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 42.20s` |

- 变异表锚点：`python3 research/prompts/m3-unlink-code-r2-sonnet-model/check_anchors.py /tmp/claude-1000/m3-unlink-code-r2/tree` 的首行 `检了 2158 行，有问题 16 行`。16 行全是第 2030 行及以前的旧行（我的校验器认不得多目标写法、别处的 litmus 文件与同文件外的测试名）；本批动到的第 2124–2163 行逐行过：文件在、原文恰好命中一次、替换与原文不同、点名的测试函数在对应目标文件里有定义。变异表差的账：`+` 38 行、`-` 11 行，按名字配对 替换 7、追加 31、删 4，与背景材料第一节 N6 行的数相同（草稿 `muts-diff.txt` 数的）。
- 分辨力探针（`research/prompts/m3-unlink-code-r2-sonnet-model/probes/`）：在草稿副本改一处、跑点名的测试二进制、还原。下文每处引「探针 Pn」都指这里。

## 1 N1（K1）：挂载续做推满仍放不下

条款原文（整条抄在 `m3-unlink-code-r2-sonnet-model/kb-quotes.md`）：

- 「推满仍放不下时挂载照样交回可写、队列留着，之后会话里每次用户改动先走空、走不动报空间不够，正常卸载照常」（`.claude/kb/decisions/08-核心索引结构.md:463`）
- 「挂载处推满仍不够时挂载照样做成可写，之后的发布照发布准入判、不够报 ENOSPC，正常卸载照常」（`.claude/kb/decisions/16-发布语义.md:43`）
- 「挂载续做的删除类发布被落点那一判以空间拒时同样推抬 F 再试，推满仍放不下挂载照样做成可写」（`.claude/kb/decisions/16-发布语义.md:43`）
- 用户定案（转述，判决 `m3-unlink-code-r1-main-verification.md` 末段原样引的）：弹窗选「照 D16 做成可写」。判决 U14 行要求把「下一次挂载接着走」那句文档改成这个结局。

逐句推到代码：

| 条款句 | 代码 | 钉它的 | 判 |
|---|---|---|---|
| 挂载照样交回可写 | 类型：「pub enum DeletionQueueContinuationOutcome {」（`crates/singlefs-core/src/mount.rs:915`），字段「pub deletion_queue_continuation_outcome: DeletionQueueContinuationOutcome,」（`crates/singlefs-core/src/mount.rs:909`）；推不动时「Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(stop)) => {」（`crates/singlefs-core/src/mount.rs:4465`）里给 `continuation.outcome` 赋 `StillShortAfterTheFloorRaises` 后 `return Ok(continuation)` | 用例 `admission_raises_the_floor_before_refusing.rs:440`（挂载 `unwrap_or_else` 要 Ok）；变异行 `crates/mutations.tsv:2133`「三·2 删除批代码轮 U14」把这一臂换回 `Err(MountError::Publish…)`，点名这条用例 | 一致 |
| 队列留着、`Mounted::current` 是续做最后落盘的那一版 | 同一臂先「*current = PoolVersion::of_a_version_of_trees(walked);」（`crates/singlefs-core/src/mount.rs:4466`）；文档「挂载照样交回可写，`Mounted::current` 是续做最后落盘的那一版」（`crates/singlefs-core/src/mount.rs:919`） | 用例断言队列「队列留着第一个文件」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:465`）。**探针 P3**：删掉 4466 那句赋值，`cargo test --test admission_raises_the_floor_before_refusing -- continuation` 得 `test result: FAILED. 3 passed; 1 failed`，红的正是这条用例（panic 在该文件 478 行）。无变异行，但有用例钉 | 一致 |
| 续做发成的步与推的抬 F 仍记进 `MountOutput` | 「continuation.steps_published.extend(steps_published);」（`crates/singlefs-core/src/mount.rs:4427`）在判拒绝之前，推成的串进 `continuation.floor_raises`（4458 那一臂）；Stopped 臂直接交回这个值 | 同一用例读「现行那一版」与盘上 13 次发布；对 `deletion_queue_continuation` 条数没有断言 | 一致（条数无钉） |
| 之后会话每次用户改动先走空、走不动报空间不够 | 先走空：「if !current_version_of_trees.deleted_inodes.is_empty() {」（`crates/singlefs-core/src/mounted_session.rs:380`），走不动交 `NoSpaceAfterRaisingTheFloor`（该文件 540 行） | 同一用例 477 行起：`let Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) = &change else {`，断言 `("SpaceAdmissionRefused", 0, 0, true)`（一版没发成、抬 F 0 串、停因是抬 F 那串自己被空间拒） | 一致 |
| 正常卸载照常 | 代码文档三处写它成立：「（`mounted_session::UserChangeRefused::NoSpaceAfterRaisingTheFloor`），正常卸载照常可走。」（`crates/singlefs-core/src/mount.rs:913`）、「走不动报空间不够，正常卸载照常可走。」（`crates/singlefs-core/src/mount.rs:4373`）、「正常卸载照常可走（它抬 F 到现行那一版、不判上限）；交主 agent 定。」（`crates/singlefs-core/src/mount.rs:4508`） | 同一批的用例自己断言相反：`Err(MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. })`（`admission_raises_the_floor_before_refusing.rs:505`），注释（整行，中间的「卸载成」是原文自带的引号，用省略号带过）「那一串在任何写之前的预演里就取不到落点（攻方 E8b 量到的那一形；派发的验收写…这块盘上做不到，交主 agent）。」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:494`） | **冲突**（两边整行如左；这一格归另开的设计轮，条款句「正常卸载照常」在窄盘上实现做不到，代码文档却写成立） |

U14 的文档改写：判决点名的「下一次挂载接着走」今天在 `crates/singlefs-core/src/*.rs` 里 0 命中（`grep -n '下一次挂载接着走' crates/singlefs-core/src/*.rs` 无输出）；`# Errors` 段改成「走空里的一步没发成、不是空间不够：错连同这次挂载已落盘各次发布的写」（`crates/singlefs-core/src/mount.rs:4378`），`mount_writable` 文档加「挂载续做里推满仍放不下同样不报错：挂载照样做成、队列留着」（`crates/singlefs-core/src/mount.rs:4603`）。改写本身对得上定案；残留是上表最后一行那三句。

挂载之后的下一次挂载（A6 那一形）：用例 `after_a_mount_whose_continuation_stays_short_the_next_mount_is_refused_by_the_placement_before_acquisition_writing_nothing`（`admission_raises_the_floor_before_refusing.rs:541`）钉：下一次挂载在取号前被落点拒（`instance_to_acquire: InstanceGeneration(3)`，录制流 0 步、两盘逐字节不变）。变异行 `crates/mutations.tsv:476` 与 479 已改点名这条用例（判决 U5 的要求）。也就是「做成可写」之后，这块窄盘重挂不了、卸载也做不成——两件都是用例自己断言出来的。

什么会推翻：(a) 若 `Mounted::current` 的行为靠别的测试已钉、P3 的红是别处假红——看 P3 日志 `probe-p3-admission_raises_the_floor_before_refusing.log` 的 panic 位置；(b) 若「正常卸载照常」在条款里只管宽盘（条款 D16 第 43 行没写盘多窄），则最后一行不算冲突而是「规则没说」。

## 2 N2（K1）：续做里不是空间不够的错带写账

契约原文：「失败之前这一串里已经落盘的发布各自的写，按先后（可写挂载写行那次在最前）；第一次就失败时是空的。」（`crates/singlefs-core/src/mount.rs:728`）；该结构上方的除外句「不是发布的写不在其内：可写挂载取号那两次系统配置槽写」（`crates/singlefs-core/src/mount.rs:721`）；同一份文件里宣布的用途「增补 2 收口表第 58 行：与 `Publish` 同一个形态」，整句「不交出来设备一层数到的写与程序交得出的账对不上」（`crates/singlefs-core/src/mount.rs:292`）。判决 U9 行验收：错里的写账等于落盘次数。

代码推演：错的组成 = 挂载续做之前的账 ++ 续做里已落盘的账。

- 续做之前：「let writes_of_the_publishes_before_the_continuation: Vec<WritesByStructureKind> =」（`crates/singlefs-core/src/mount.rs:4273`），链 `once(&row_publish)`、`chain(&warm_up_publishes)`、再接 `space_admission.floor_raises()` 的各次空发布（4274–4284）。写行在最前、暖机、准入推的串，次序同发布先后。
- 续做里：「.chain(continuation.writes_of_the_persisted_publishes_in_txg_order())」（`crates/singlefs-core/src/mount.rs:4441`）；那个函数把发成的步与推的各串的各次空发布合起来「persisted.sort_by_key(|publish| publish.root.checkpoint_txg);」（`crates/singlefs-core/src/mount.rs:4353`）。
- 判拒绝之前先「continuation.steps_published.extend(steps_published);」（`crates/singlefs-core/src/mount.rs:4427`），所以出错那趟走空里已落盘的步在账里；失败那一次的账走 `writes_of_failed_publishes`（4445 起的构造）。

量（草稿副本上，探针测试 `research/prompts/m3-unlink-code-r2-sonnet-model/probe_tests_*.rs.txt`，输出原样）：

1. 无抬 F 的形（三个单元文件发完意图就崩、续做第二步的第一个写报块设备错）：`PROBE writes_before_the_failing_write=75 sum_persisted=73 sum_failed=0 persisted_len=3 failed_len=1 before_first_publish=0`。账里 3 份（写行、暖机、续做第一步）合计 73 次写，加取号两次系统配置槽写正好 75 = 失败前设备数到的写。用例钉份数：`(1 + warm_up_publishes + 1, publishes_landed,),`（`unlink_mount_continuation_and_the_emptied_version.rs:641`）。
2. 有抬 F 的形（窄盘上续做推过一串两次空发布之后，第 12 步的第一个写报错）：`PROBE failing_write=391 device_writes_issued_before_failure=390 account_persisted_sum=386 acquisition=2 persisted_len=16 before_first_publish=0 gap=2`。账 16 份，设备多出 2 次写 = 那一串先写系统配置的两次写（`raise_sysconf_writes=2`）。
3. 次序：同一形把失败挪到推完那串之后又发成一步（txg 24）的下一步，`PROBE2 … expected_by_txg=[19, 17, 17, 25, 25, 25, 25, 25, 29, 29, 29, 29, 29, 25, 17, 21, 29] actual=[19, 17, 17, 25, 25, 25, 25, 25, 29, 29, 29, 29, 29, 25, 17, 21, 29] equal=true`（每项是一份账的写调用数；期望值按 txg 排）。

判：

| 子格 | 判定 | 依据 |
|---|---|---|
| 条数 = 落盘次数、写行在最前 | 一致 | 量 1；用例+**探针 P1**：把续做那一截从链里拿掉（`probes/p1.orig`→`p1.repl`），`a_mount_whose_continuation_step_fails_…` 红（`panicked … unlink_mount_continuation_and_the_emptied_version.rs:636`）；变异行 `crates/mutations.tsv:2134` 同形 |
| 按先后（续做内步与推的串交错） | 一致，但没有入库用例钉 | 量 3 在入库用例里没有对应形；**探针 P2**：删掉 `sort_by_key`（4353）后 `unlink_mount_continuation…` 的 `a_mount…` 与 `admission_raises_the_floor_before_refusing` 的 4 条 continuation 用例全绿（`test result: ok. 1 passed`、`ok. 4 passed`），只有我的量 3 探针转红（`equal=false`，`left … 29, 17, 21 right … 17, 21, 29`） |
| 准入推的串在账里（`space_admission.floor_raises()` 那一段） | 一致，没有用例钉 | **探针 P4**：把 4277–4283 那段链删掉，同两个二进制全绿（`ok. 4 passed`、`ok. 1 passed`）；没有变异行 |
| A 自报缺口①：推过抬 F 之后再出错，每一串先写系统配置那一步的写不在错里 | **冲突**（口径） | 字面上 `writes_of_persisted_publishes` 说的是「发布各自的写」，系统配置那两次不是发布，不违反 728 行；但 721 行的除外句只除外了取号那两次，292 行宣布的目的是设备数到的写与账对得上，量 2 差 2。代码自己在「推的那几串先写系统配置那一步（`RaisedFloor::system_configuration_writes_before_the_first_publish`）不是发布，不在这里。」（`crates/singlefs-core/src/mount.rs:4342`）承认了 |
| A 自报缺口②：抬 F 自己报错那一支原样交回 | 规则没说 | 「Err(cause) => {」（`crates/singlefs-core/src/mount.rs:4474`）交回抬 F 自己的错，不带续做之前与续做里已落盘的账。契约字面只管 `PublishSequenceFailed`；同类的写行阶段推抬 F 报错有专门的「pub struct FloorRaiseFailedAfterTheMountsPublishes {」（`crates/singlefs-core/src/mount.rs:500`），带「报错那一串之前已经推成的那几串抬 F，按先后」（`crates/singlefs-core/src/mount.rs:507`），续做阶段没有对应的 |

什么会推翻：量 2 的 gap=2 若换成 0（比如设备计数把系统配置写算进别处）则缺口①不成立；若有入库用例在「续做推过抬 F 之后再出错」上断言账的次序，P2/P4 的「全绿」就不是未钉。

## 3 N3（K1）：执行器不再跳过删除中的版本，模型与实现逐种改动

条款：D8 已定项 16 第 5 条的末句（拒成已有的那一种拒绝，括注是 `UserChangeRefused::NoFileVersionToChange`）：「删空的一版上覆盖写、顺序写、unlink 拒成已有的」（`.claude/kb/decisions/08-核心索引结构.md:462`）；第 6 条「会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的 `deleted_inodes` 队列走空，再做它自己」（`.claude/kb/decisions/08-核心索引结构.md:463`）。

实现一侧：会话走空（「if !current_version_of_trees.deleted_inodes.is_empty() {」`crates/singlefs-core/src/mounted_session.rs:380`）后按成员分流：建 inode 放行，「UserChange::Overwrite(_)」（`crates/singlefs-core/src/mounted_session.rs:401`）、「| UserChange::SequentialWrite(_)」（`crates/singlefs-core/src/mounted_session.rs:402`）、「| UserChange::Unlink { .. } => {」（`crates/singlefs-core/src/mounted_session.rs:403`）同一臂拒成「fn no_file_version_to_change(」（`crates/singlefs-core/src/mounted_session.rs:467`），带走空发成的版与推的抬 F（U10）。
模型一侧：执行器不再早退，覆盖写「let queue_is_walked_first = !current_file_version.deleted_inodes.is_empty();」（`crates/singlefs-harness/src/history.rs:2931`）后改走逐版比对的收尾；建 inode 的前提改成「if version.inode_record.is_some() && version.deleted_inodes.is_empty() {」（`crates/singlefs-harness/src/history.rs:3523`）；拒绝映射到新理由「ModelRefusalReason::OnlyNewInodesOnAVersionWithTreesAndNoFile,」（`crates/singlefs-harness/src/history.rs:3205`），模型里的拒绝分支在 `crates/singlefs-harness/src/model.rs:1547`（`answer_refusing_a_change_on_a_version_with_trees_and_no_file`），覆盖写与 unlink 的答法开头调它。

| 用户改动（删除中的版本上） | 实现结局 | 模型结局 | 钉它的 | 判 |
|---|---|---|---|---|
| 覆盖写 | 走空 → `NoFileVersionToChange{三样}` | 走空那几步按「走空的一步」比，再答拒绝 `OnlyNewInodes…` | 用例 `unlink_in_histories_judged_by_the_model.rs:376`；变异行 `crates/mutations.tsv:2136`（执行器照旧跳过）；会话一侧 `…continuation_and_the_emptied_version.rs:659` + 变异行 2135（不带发成的版）；**探针 P6b**：把理由映射成 `Unexplained`，两条（覆盖写、unlink）红 | 一致 |
| unlink | 同上 | 同上 | 用例 `unlink_in_histories_judged_by_the_model.rs:400`；变异行 2137；**探针 P6a**：删掉模型 `answer_unlink_intent` 开头的拒绝分支，恰这一条红（panic 在该文件 403 行） | 一致 |
| 建 inode | 走空 → 建，做成 | 走空那几步 + `answer_create_inode` | 用例 `unlink_in_histories_judged_by_the_model.rs:420`；变异行 2138 | 一致 |
| 顺序写 | 与覆盖写同一臂（402 行） | 模型没有顺序写这个操作，执行器也不生成 | 无 | 规则没说（不在随机历史里） |
| 走空被空间拒（`NoSpaceAfterRaisingTheFloor`） | 经 `member_and_reason_of_a_refused_user_change` 映成已有理由 | 走空那一步模型答「走空的一步」 | 本批没有新用例 | 复核不了（K2 的格，我没造） |
| 走空多于一步 | 三条用例的文件都只有 1 个数据单元，走空 1 步 | — | 用例 `…continuation_and_the_emptied_version.rs:659` 是 3 步，但那一条不过模型 | 模型罩多单元文件归 U13，不在本轮 |

这一格还要说的两件：(1) 模型新理由与执行器收尾那段没有任何变异行指向 `model.rs`（`crates/mutations.tsv` 第 2124–2163 行的文件列里没有 `model.rs`），P6a 是我加的探针，入库的钉全靠那三条用例；(2) `NoFileVersionToChange` 的理由映射对所有成员一视同仁，会话里「现行那一版根本没有树」那一臂（`PoolVersion::WithoutFile`，`mounted_session.rs:333`）也会映到「有树而文件集合为空」这条理由，但执行器在 `file_version()` 为 None 时先报前提不满足，走不到。U11 验收「512 段那 4 次被比到」没跑（random_histories 的 512 段在 debug 下太长，我只跑了三条单点用例），写「复核不了」。

什么会推翻：P6a/P6b 若在入库表里已有等价变异行，则「无变异行」一条失效；若执行器生成器能产出顺序写，则「规则没说」一格要补用例。

## 4 N5（K1）：抬 F 的界按位置取；U12 按写账划续做段

条款：形式化接入那条线定的口径，调度记录里：「F 的界按挂载交回的抬 F 那几串在写表末尾的位置取，取不出交 NotLocatedInTheMountWrites、整段 F 记不判」（`records/2026-10-01-删除与并发事务.md:376`），同一行续「不许改回按内容认」。判决 U12 行（`research/prompts/m3-unlink-code-r1-main-verification.md` 的 U12 一行，转述）：崩溃注入按 U9 交回的写账划续做段，不并进暖机。

### 4.1 界是不是按位置取、没退回按内容认：一致

- 界 = 写表长度减去「从第一串抬 F 起」的写数：「let Some(first_write) = usize::try_from(writes_from_the_first_floor_raise_on)」（`crates/singlefs-checker-tier/src/crash_injection.rs:1893`），第一串与写数由「fn writes_from_the_first_floor_raise_on(」（`crates/singlefs-checker-tier/src/crash_injection.rs:1826`）按挂载交回的账（准入推的串、续做各步、续做推的串）算。
- 内容只拿来核界，不拿来找界：界上那个写要是系统配置槽写「.is_some_and(|write| write.kind == StepKind::SystemConfigurationSlot);」（`crates/singlefs-checker-tier/src/crash_injection.rs:1901`），且「let every_floor_raise_root_is_written_from_the_boundary = floor_raise_roots」（`crates/singlefs-checker-tier/src/crash_injection.rs:1908`）；不满足交 `NotLocatedInTheMountWrites`、不 panic。这与被推广的旧函数同形（旧版 `crates/singlefs-harness/src/mount_transparency.rs:288` 也是「长度减写数，再核两样」），不是按 F 值或字节内容找。
- 生产路径调新函数：「floor_raises_in_the_mount_writes_counting_the_continuation(&mounted.output, &mount_writes)」（`crates/singlefs-checker-tier/src/crash_injection.rs:1666`）。旧函数今天只剩 `crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs` 里 5 处调用（343、855、872、905、921 行；`grep -rnF 'FloorRaisesInTheMountWrites::of_the_mount_output' crates` 在冻里共 6 行，另 1 行是 `crash_injection.rs:1870` 的注释），于是同一个判法有两份实现、各有自己的用例（规则没说要不要并成一份；漂移风险记在这里）。

### 4.2 准入与续做都推过时界取得对不对

- 准入推了：界 = 准入串写数 + 续做全部步写数 + 续做串写数（「+ continuation_steps_after(None)」`crates/singlefs-checker-tier/src/crash_injection.rs:1853`；根身份并上续做串「roots.extend(roots_of(&self.pushed_by_the_continuation));」`crates/singlefs-checker-tier/src/crash_injection.rs:1850`）。准入没推、续做推了：第一串是续做推的第一串，界之后是续做各串加 txg 大于它的续做步（「let first_pushed_by_the_continuation = self.pushed_by_the_continuation.first()?;」`crates/singlefs-checker-tier/src/crash_injection.rs:1858`）。写表末尾的真次序是「准入串… 续做各步与续做串按 txg 交错」，求和不看交错，所以两支的和是对的。
- 我在 harness 档照真实挂载量过 txg：续做推串夹在两步之间的那一形，步 10 是 txg 21，推的串两次空发布是 22、23，步 11 是 24（探针输出 `STEP 10 txg=21`、`RAISEPUB 0.0 txg=22`、`RAISEPUB 0.1 txg=23`、`STEP 11 txg=24`），与单测 `a_floor_raise_pushed_inside_the_continuation_is_located_between_the_continuation_steps`（`crates/singlefs-checker-tier/src/crash_injection.rs:3881`）造的写表同形。
- 钉：两条单测各钉一支（3881 钉「只续做推」，3938 `floor_raises_pushed_by_the_mount_admission_are_located_before_the_continuation_that_follows_them` 钉「只准入推」）；变异行 `crates/mutations.tsv:2145`、`crates/mutations.tsv:2146`。**没钉：两样都推的混合形**——两条单测里 `pushed_by_the_continuation` 与 `pushed_by_the_admission` 总有一个是空，1850 行那句 `roots.extend`、1851–1854 的 `writes_of(&self.pushed_by_the_continuation)` 去掉都不会让这两条单测红（读代码推的；checker 档不许我跑，没量）。
- 半截的串：续做里推抬 F 那一串自己被空间拒时，`Stopped(FloorRaiseRefusedForSpace)` 里可能已有半截：「在落盘之前被拒，先写系统配置那一步与前面 n − 1 次已落盘、在调用方的现行版本里」（`crates/singlefs-core/src/mount.rs:2323`），这些写在写表末尾、不在 `MountOutput` 的账里，界会向后偏；偏到的位置恰是系统配置槽写时 `FirstWriteAt` 照样交出一个偏的界（推的，没量；窄盘 + 续做 + 抬 F 半截失败三样叠起来才走得到）。

### 4.3 挂载透明「F」那一判仍只认准入推的（本批没动、但与界同一条线）

「let floor_raised_by_the_mount = mounted」（`crates/singlefs-checker-tier/src/crash_injection.rs:1630`）仍调「.map(|mounted| FloorRaisedByTheMount::of_the_mount_output(&mounted.output));」（`crates/singlefs-checker-tier/src/crash_injection.rs:1633`），它只看「match output.space_admission.floor_raises().last() {」（`crates/singlefs-harness/src/mount_transparency.rs:241`）。续做推的串不算，F 的预期是「FloorRaisedByTheMount::NotRaised => FloorRelation::Unchanged,」（`crates/singlefs-harness/src/mount_transparency.rs:792`）。harness 档量（探针 3，`probe_mount_transparency_floor_when_only_the_continuation_pushed_a_floor_raise`）：`PROBE3 floor_relation_input=NotRaised admission_raises=0 continuation_raises=1 effective_floor_before=Some(CheckpointTxg(0)) effective_floor_after=Some(CheckpointTxg(18))`，`judge_mount_transparency` 的 `RollbackFloor` 判 `Violated`（期望「F 生效值是 R1 的 Some(CheckpointTxg(0))」，观察「R2 的 F 生效值 Some(CheckpointTxg(18))」）。同一次判里 `UserVisibleTreeRootPointers` 也判 `Violated`——任何真发了删除步的续做都会让这一格红，与 F 无关；所以今天没有独立的 F 假红，但「挂载续做进挂载透明怎么判」没有人定（规则没说；不在 21 份里，`mount_transparency.rs` 没改）。

### 4.4 U12：「只有续做才写的结构」五种取法

代码：「fn writes_a_structure_only_the_continuation_changes(writes: &WritesByStructureKind) -> bool {」（`crates/singlefs-checker-tier/src/crash_injection.rs:1923`）把 `DataUnit`、`ExtentTreeNode`、`InodeTreeLeafContainer`、`InodeTreeRoot`、`DeletedInodesTreeNode` 五种记为只有续做写（末一种在 1930 行），其余十种记 false；`match` 穷举，种类少列编译不过。划段「.skip(1)」（`crates/singlefs-checker-tier/src/crash_injection.rs:1959`）跳过写行那份账，找第一个写了这五种之一的已落盘发布；都没写而失败那一次写了，续做从失败那次起；否则 None（写行之后都算暖机）。

按真挂载量一遍每次发布写了哪些种类（探针 1，窄盘上挂载续做 14 步 + 续做里推 1 串两次空发布，输出原样节选）：

```
KINDS row: AllocationRecordTreeNode=6 AccountingTreeNode=2 CentralMappingTreeNode=2 TreeTableUnit=2 InstanceTableUnit=2 JournalRecord=2 RootSlot=1 SystemConfigurationSlot=2
KINDS warm0: AllocationRecordTreeNode=6 AccountingTreeNode=2 CentralMappingTreeNode=2 TreeTableUnit=2 JournalRecord=2 RootSlot=1 SystemConfigurationSlot=2
KINDS contraise0-pub1: AllocationRecordTreeNode=10 AccountingTreeNode=2 CentralMappingTreeNode=2 TreeTableUnit=2 JournalRecord=2 RootSlot=1 SystemConfigurationSlot=2
KINDS step0: ExtentTreeNode=4 InodeTreeLeafContainer=2 InodeTreeRoot=2 AllocationRecordTreeNode=6 AccountingTreeNode=2 CentralMappingTreeNode=2 TreeTableUnit=2 JournalRecord=2 RootSlot=1 SystemConfigurationSlot=2
KINDS step13: ExtentTreeNode=2 InodeTreeRoot=2 AllocationRecordTreeNode=6 AccountingTreeNode=2 CentralMappingTreeNode=2 TreeTableUnit=2 JournalRecord=2 RootSlot=1 SystemConfigurationSlot=2
```

写行、两次暖机与续做推的两次空发布一份都不含那五种；14 步每一步都含 `ExtentTreeNode` 与 `InodeTreeRoot`（最后一步没有 `InodeTreeLeafContainer`）。所以取法对：有效的是 `ExtentTreeNode`、`InodeTreeRoot`、`InodeTreeLeafContainer` 三种，`DataUnit` 与 `DeletedInodesTreeNode` 这次挂载里一次都没出现（续做从不写数据单元；`deleted_inodes` 根只在意图那次写），列着无害但是空转。钉：单测 `a_mount_that_handed_back_an_error_inside_the_continuation_marks_the_continuation_by_the_persisted_publishes`（`crates/singlefs-checker-tier/src/crash_injection.rs:3996`）、变异行 `crates/mutations.tsv:2147`（`false &&`）；接线「Err(MountError::Publish(failed)) => {」（`crates/singlefs-checker-tier/src/crash_injection.rs:1647`）没有端到端用例。B 自报③「续做开头先被空间拒、先推的那一串抬 F 归暖机」就写在「续做开头先被空间拒、先推的那一串抬 F 归暖机」（`crates/singlefs-checker-tier/src/crash_injection.rs:1951`）：属实，是账里分不出准入推还是续做推造成的，已写进注释；没有用例钉。

| 子格 | 判 |
|---|---|
| 界按位置取、未退回按内容认 | 一致 |
| 准入、续做各自单推 | 一致，各有单测与变异行 |
| 准入与续做都推 | 一致（读代码推的），没有用例钉 |
| 挂载透明 F 判不认续做推的串 | 规则没说（探针 3 量到 F 与根指针两格红，后者与 F 无关） |
| U12 五种取法 | 一致（量到：写行/暖机/空发布 0 次，续做每步 ≥2 种）；两种空转 |

什么会推翻：4.2 若 checker 档战役里有一条同时有准入推与续做推的历史且它的二次崩溃 F 判过——混合形就有钉；4.4 若有一种别的发布（比如卸载那一串）在挂载里写 `InodeTreeRoot`，则取法偏宽。

## 5 K3：21 份文件各一格

变异行复跑（草稿副本上按表里一行原样改、跑它第 5 列点名的 harness 档用例、还原；脚本 `research/prompts/m3-unlink-code-r2-sonnet-model/run_row.py`）：行 2133、2134、2135、2136、2137、2138、2139、2140、2141、2143、2151、2161、2162、2163 共 14 行，每行都是 `rc=101`、点名的那条用例 `FAILED`，没有一行放过；行 2144（LR2）两次都在 225 秒线上超时（第一次含 6 分钟冷编译，第二次是用例本身，基线两条合计 164 秒），写复核不了。checker 档的行（2145–2150、2152–2160）不许我跑，下表标「checker 档，没跑」。

| # | 文件 | 这一批动了什么 | 判 | 钉它的与复核 |
|---|---|---|---|---|
| 1 | `crates/mutations.tsv` | `+`38 `-`11：替换 7、追加 31、删 4（N6） | 一致 | 本批 40 行锚点全中（第 0 节）。删的 4 行：3 行点名的 `only_allocated_statistic_above_walked`、`allocation_statistic_mechanism_of`、`raise_after_rollback_leaves_allocated_statistic_above_walked` 三个函数今天在 `crates/` 下 0 命中（`grep -rn` 无输出），表里也没有残留的 `known_red_forms_are_matched` 名字（`grep -c` 为 0）；第 4 行「挂载续做推满仍不够时当作走完」换成 U14 行 2133。行 2151、2157、2163 是同一处改动（`transaction.rs` 单元写与记录之间的屏障）点名三条不同用例，不是重复 |
| 2 | `crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` | 一行：把 `NoFileVersionToChange { floor_raises, .. }` 的抬 F 记进穷举臂 | 一致 | 「Err(UserChangeRefused::NoFileVersionToChange { floor_raises, .. }) => floor_raises,」（`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:19103`）与 `history.rs` 的同形臂「| UserChangeRefused::NoFileVersionToChange { floor_raises, .. },」（`crates/singlefs-harness/src/history.rs:3093`）相同；类型变了，旧臂编译不过，所以不会悄悄漏。没有用例（bin），没有变异行 |
| 3 | `crates/singlefs-checker-tier/src/crash_injection.rs` | N5 的界、U12 划段、`ExactlyOneCrashState`、`withheld_write_kinds` 改按「段首 + k」 | 一致（读代码 + 复核数）；checker 档没跑 | 界与划段见第 4 节。`ExactlyOneCrashState`：段号/写数/扣下下标不成立时 `assert!`（「stream_indexes[writes_of_the_state.end - 1] < marks.after_the_last_finished_step,」`crates/singlefs-checker-tier/src/crash_injection.rs:2983`），拿去跑战役或写重放行就 panic（「!matches!(draw, CrashPointDraw::ExactlyOneCrashState { .. }),」`crates/singlefs-checker-tier/src/crash_injection.rs:3578`、`draw_environment_values` 里 3367 行）。「段内第 k 个写 = 段首 + k」成立：`split_into_segments` 按操作次序切、一段是连续的一截（`crates/singlefs-harness/src/segments.rs` 159–183 行，我读的），旧写法 `segment[k]` 与新写法逐项相同。RefFS 的崩溃时刻改取持久窗口最后一个写所在段（「let segment_the_crash_closes = segment_containing_the_write(」`crates/singlefs-checker-tier/src/crash_injection.rs:1140`），对段内崩溃状态与旧值相同（窗口末尾就是 `segment_index` 那一段）。变异行 2145–2147、2152–2160 全在 checker 档 |
| 4 | `crates/singlefs-checker-tier/src/device_log.rs` | LR7：两条单测、三行变异 | 一致；checker 档，没跑 | 长度 0 的守卫在源码里「if declared_length == 0 {」（`crates/singlefs-checker-tier/src/device_log.rs:295`），越过段尾的守卫在「let continues_at_cursor = piece.start == covered_up_to && piece.end <= fill_end;」（`crates/singlefs-checker-tier/src/device_log.rs:320`）。两条新单测（847、866 行）按代码手推：长度 0 时守卫缺了返回「用掉 0 条」、`assert_eq!(…, None)` 先红；接续写伸出段尾时缺了第二个条件，已铺满末端越过段尾，函数末尾的 `assert_eq!` panic。变异行 2148–2150 的原文/替换文各自命中一次。判决 LR7 要的「长度 0 守卫、接续写越过段尾、两个 assert」三处各有一行，对上 |
| 5 | `crates/singlefs-checker-tier/tests/crash_injection_campaign.rs` | `histories_without_any_crash_point == 0` 两处、零单元/首文件各摆几个崩溃状态、三条靶向用例 | 一致（数字在 harness 档复核过）；checker 档，没跑 | 数字：同一段历史在 harness 档的录制流上（探针 `probe_campaign_segments.rs.txt`）量到——历史 A（首个文件 + 覆盖写 5、7）24 段；第 21 段头 3 个写 `[JournalRecord, JournalRecord, RootRecordFua]`，第三个是 `(1, 5)`；第 12 段头两个写同偏移、两块盘；从第 12 段起 56 个写里根槽写在下标 26（`(1, 3)`）与 55（`(1, 4)`），第 56 个就是 `(1, 4)`。历史 B（24 次覆盖写）总写数 826、从第 12 段起 793（「writes_from_the_segment_to_the_end, 793,」`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:1667`），`(1, 3)` 的根槽被后来的根盖掉，被扣的两处之后无写。历史 C 的段序列里 `24,2,1,2` 恰一处、`2,1,2` 多处（零单元与暖机共用同形，具体哪三段归零单元那一步要看标记表，checker 档才取得出）。7 与 8 的算术（段写数 ≤2 整段枚举 2^n−1，更长抽 1）对：3+1+3、1+3+1+3。文件自己写明「这一条在 checker 档里没在交回之前跑过」（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:609`）。变异行 2152–2160 全 checker 档 |
| 6 | `crates/singlefs-core/src/mounted_session.rs` | U10：`NoFileVersionToChange` 改成带三样的结构体变体；`no_file_version_to_change` | 一致 | 三个构造点：入口 `WithoutFile` 臂（333 行起，三样全空）、建 inode 以外的改动「return Err(no_file_version_to_change(published_in_this_change));」（`crates/singlefs-core/src/mounted_session.rs:404`）、第二个 `WithoutFile` 臂（407）；`UserChange::SequentialWrite` 与 `Overwrite`、`Unlink` 同臂（401–403）。钉：用例 `…continuation_and_the_emptied_version.rs:659`（三版 `(1,2,1),(2,1,1),(3,0,0)`、盘上正好落 3 次）；变异行 2135 复跑红；空队列那一形 `…:314` 断言三样都空 |
| 7 | `crates/singlefs-core/src/mount.rs` | N1：`DeletionQueueContinuationOutcome`、`MountOutput` 新字段、续做推满仍放不下交 `Ok`；N2：续做错带写账 | 部分一致：N1 的结局、会话、U14 改写一致；两处**冲突**、一处规则没说 | 见第 1、2 节：冲突 ①「正常卸载照常可走」三处文档（913、4373、4508）对同批用例断言的 `Err(MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. })`（`admission_raises_the_floor_before_refusing.rs:505`）；② 292 行宣布的「设备一层数到的写与程序交得出的账对得上」对续做推过抬 F 后出错差 2 次写（量 2）；规则没说：续做阶段抬 F 自己报错那一支（4474）不带已落盘的账。钉：行 2133（复跑红）、2134（复跑红）、探针 P1/P3 红；P2（次序）、P4（准入推的串）无钉 |
| 8 | `crates/singlefs-core/src/transaction.rs` | 只改 `publish_sequential_write` 的文档：不再按单元数拒 | 一致 | 文档句「切出的单元多于一片下段叶装得下的 144 个时不拒」（`crates/singlefs-core/src/transaction.rs:5636`）；旧错误成员 `ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided` 在 `crates/` 下 0 命中（`grep -rn` 无输出），说明代码里没有这条拒绝；U2 用例走的就是这条路径（`sequential_write_in_process_as_instance_one_and_advance_the_pool` 调 `publish_sequential_write`，`crates/singlefs-harness/tests/common/mod.rs:410`）并写出 287 个单元，基线 `ok. 1 passed`。「内部扇出 147」那一半我没核（没去读 extent_tree.rs 的常量），写复核不了 |
| 9 | `crates/singlefs-harness/src/history.rs` | N3 执行器（覆盖写、unlink、建 inode 不再跳过删除中的版本）；N4 清单删空；收尾函数换成 `&dyn Fn` | 一致 | N3 见第 3 节。清单：「pub const KNOWN_RED_FORMS: [KnownRedForm; 0] = [];」（`crates/singlefs-harness/src/history.rs:1851`）；被它挡着的三个函数在 `crates/` 下 0 命中；单测改成「with_the_known_red_list_empty_an_allocated_statistic_above_walked_is_a_new_finding_whatever_its_mechanism」（`crates/singlefs-harness/src/history.rs:6383`），断言 `KNOWN_RED_FORMS.is_empty()` 且三种观察都归 `NewFinding { signature: FailureSignature::CheckerViolations { .. } }`。「F 抬进空档只有 I-3.1 红」那一形今天由丁-defer 修好这一点靠 `tests/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs`（冻里在）与 `random_histories.rs:1408` 的复现用例；这一批的验证日志说它们都绿，我没跑。`.claude/kb/milestone/02-second-txn.md` 收口表第 43 行仍写「登记成生成器的已知红第 1 条」（现在态），与清单已空不符，属 kb 写回批 |
| 10 | `crates/singlefs-harness/src/model_comparison.rs` | 只改一句注释 | 一致 | 注释「队列不空时那两种改动先走空、再拒成带着走空那几步的 `NoFileVersionToChange`」（`crates/singlefs-harness/src/model_comparison.rs:676`）与 mounted_session 的 U10 行为、history.rs 的执行器相符 |
| 11 | `crates/singlefs-harness/src/model.rs` | 新拒绝理由 `OnlyNewInodesOnAVersionWithTreesAndNoFile`；覆盖写与 unlink 开头调拒绝分支 | 一致 | 「OnlyNewInodesOnAVersionWithTreesAndNoFile,」（`crates/singlefs-harness/src/model.rs:473`）、分支 1547 行起；三处 `match` 穷举（499、519、2062）都加了成员。钉：三条 U11 用例；**无变异行指向 model.rs**，探针 P6a 补量：删掉 unlink 的分支，恰那一条用例红 |
| 12 | `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs` | N1 用例（改写）+ U5 用例 | 一致；其中「卸载」那一段是冲突的另一边 | 见第 1 节；基线 `ok. 14 passed`；行 476、479 已改点名 U5 用例。窄盘造法与断言都是真盘面（录制流 0 步、两盘逐字节不变） |
| 13 | `crates/singlefs-harness/tests/a_floor_raise_refused_for_space_counts_as_short_of_space.rs` | U3：一条用例 | 一致 | 「fn a_deleted_inodes_queue_longer_than_one_root_holds_is_not_short_of_space_and_pushes_no_floor_raise(」（`crates/singlefs-harness/tests/a_floor_raise_refused_for_space_counts_as_short_of_space.rs:169`）：拒绝取自 `deleted_inodes_root_entries` 对 2032 个号真交回的错；行 2139 复跑红。它钉的是分类函数 `refusal_is_short_of_space`，会话与挂载续做都走这一个函数，所以一处钉两处 |
| 14 | `crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` | LR3、LR2：两条用例 | 一致（LR2 的变异红复核不了） | 基线两条 `ok. 2 passed … 164.14s`；LR3「fn a_floor_raise_whose_shadow_ledger_meets_an_abandoned_root_unreadable_by_account_and_mapping_is_refused_and_restores_the_allocator(」（`crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs:590`）行 2143 复跑红；LR2「fn an_abandoned_root_with_a_two_page_instance_table_and_an_unreadable_account_isolates_both_pages_from_the_mapping(」（`crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs:685`）行 2144 两次复跑都超过 225 秒线，没有结果 |
| 15 | `crates/singlefs-harness/tests/common_admission_pool/mod.rs` | 共用池助手的 `NoFileVersionToChange` 臂记抬 F | 一致 | 「Err(UserChangeRefused::NoFileVersionToChange { floor_raises, .. }) => {」（`crates/singlefs-harness/tests/common_admission_pool/mod.rs:303`）与旁边 `Publish`、`FloorRaiseFailedWhilePushingForSpace` 臂同形，没有它旧臂在类型变了之后编译不过 |
| 16 | `crates/singlefs-harness/tests/named_unit_verification_stands_in_for_the_barrier_before_the_record.rs` | K3-⑨ 两条 + C592 一条（B 在今天底座上重做） | 一致 | 基线 `ok. 3 passed … 42.20s`；行 2161、2162、2163 复跑各红对应那一条。C592 那条与 2151、2157 是同一处屏障改动 |
| 17 | `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` | D 的段序列钉 `24+2+1+2` | 一致 | 「"24+2+1+2",」（`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:493`）；整条流 36 段之和 195 = 到 C 的 166 + D 的 29（我用 python 把那串 36 个数加了：`36 195 29 166`，分别是段数、总和、末四段和、前面的和）；行 2151 复跑红。用例注释写「debug 下单跑几秒，是轻用例」（`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:466`），基线实测 72.48 秒，是在机器负载 80 以上时量的（`uptime` 显示 `load average: 82.41`），同一用例变异后在第一句断言上 2.37 秒红，所以注释的「几秒」不能凭这次判错 |
| 18 | `crates/singlefs-harness/tests/unlink_in_histories_judged_by_the_model.rs` | U11 三条用例 | 一致 | 基线 `ok. 3 passed`；行 2136、2137、2138 复跑各红对应那一条；探针 P6a、P6b 红（第 3 节） |
| 19 | `crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` | U9（块设备错）、U10（拒绝带三版）、空队列形断言三样为空 | 一致 | 基线 `ok. 11 passed`；行 2134、2135 复跑红；探针 P1 红 |
| 20 | `crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs` | U2 新文件：287 个单元、产品批大小 | 一致 | 「const DATA_UNITS_OF_THE_FILE: usize = 287;」（`crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs:32`），断言每版从盘上重建与发布逐项相同、池级 checker 无红、单元数序列 `[287, 143, 0]`；行 2141（`<` 改 `<=`，源码「last_data_unit < data_units_kept」`crates/singlefs-core/src/unlink.rs:178`）复跑红 |
| 21 | `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs` | LR6：孤记录盘面改用 `WriteIsSwallowed` | 一致 | 判决 LR6 行要的「改用 `WriteIsSwallowed` 吞取号写造盘面」照做：「fault: InjectedFault::WriteIsSwallowed,」（`crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs:236`），断言吞了两次「swallowing_the_acquisition.fired().len(),」（`crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs:260`）；基线 `ok. 1 passed … 77.78s`，后面「零单元发布 txg 2、暖机推到 txg 4」的断言仍成立，说明盘面还在孤记录那一支上。没有变异行（判决没要） |

## 6 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1·N1 结局：续做推满仍放不下挂载交回可写、队列留着 | 一致 | `Stopped` 臂交 `Ok`，`current` 是最后落盘的一版（探针 P3 删赋值即红）；行 2133 复跑红 |
| K1·N1 之后会话每次改动先走空、走不动报空间不够 | 一致 | 用例断言 `NoSpaceAfterRaisingTheFloor`，一版没发成 |
| K1·N1 「正常卸载照常」 | 冲突 | mount.rs 913、4373、4508 三处文档写成立；同批用例 505 行断言卸载 `Err(…RehearsalBeforeAnyWrite)`；归另开的设计轮 |
| K1·N1 U14 文档改写 | 一致 | 「下一次挂载接着走」0 命中；残留只有上一行三句 |
| K1·N2 条数 = 落盘次数、写行在最前 | 一致 | 量 1：75 = 73 + 取号 2；探针 P1 与行 2134 红 |
| K1·N2 按 txg 先后、准入推的串在账里 | 一致，未钉 | 量 3 次序对；探针 P2、P4 删掉对应代码后入库用例全绿 |
| K1·N2 A 缺口①：推的串先写系统配置那步不在错里 | 冲突（口径） | 字面不违反 728 行；违反 292 行宣布的「账对得上设备数」，量 2 差 2 次写 |
| K1·N2 A 缺口②：抬 F 自己报错那一支不带已落盘的账 | 规则没说 | 契约只管 `PublishSequenceFailed`；与写行阶段的 `FloorRaiseFailedAfterTheMountsPublishes` 口径不齐 |
| K1·N3 删除中的版本上覆盖写 / unlink / 建 inode | 一致 | 模型逐版比走空的步，再答拒绝或建；三条用例 + 行 2136–2138 复跑红 + 探针 P6a/P6b 红 |
| K1·N3 顺序写 | 规则没说 | 实现与覆盖写同臂，执行器不生成这个操作，模型没有 |
| K1·N3 走空被空间拒、走空多步 | 复核不了 | 本批没有这两形的用例（文件只有 1 个单元，U13 归下一批） |
| K1·N5 界按位置取 | 一致 | 长度减写数，再核系统配置槽写与根槽写；没退回按内容认 |
| K1·N5 准入与续做各自单推 | 一致 | 两条单测 + 行 2145、2146（checker 档，没跑） |
| K1·N5 准入与续做都推 | 一致（读代码推的），未钉 | 两条单测里总有一边空 |
| K1·N5 挂载透明 F 判不认续做推的串 | 规则没说 | 探针 3：`RollbackFloor` 判 Violated，`UserVisibleTreeRootPointers` 也 Violated（与 F 无关） |
| K1·N5 U12 五种取法 | 一致 | 量到写行/暖机/空发布 0 次、续做每步 ≥2 种；`DataUnit`、`DeletedInodesTreeNode` 空转 |
| K3 #1 `crates/mutations.tsv` | 一致 | 本批 40 行锚点全中；14 行复跑全红；删的 4 行守的代码已不在 |
| K3 #2 `e158_root_choice_repair.rs` | 一致 | 一行穷举臂，与 history.rs 同形 |
| K3 #3 `crash_injection.rs` | 一致（checker 档没跑） | 界/划段/靶向状态/段首加 k 都推过；两处没钉的见第 4 节 |
| K3 #4 `device_log.rs` | 一致（checker 档没跑） | 两条单测 + 三行变异，手推对得上 |
| K3 #5 `crash_injection_campaign.rs` | 一致（数在 harness 档复核，checker 档没跑） | 段数、写数、根槽下标与 56、793 都量到相同 |
| K3 #6 `mounted_session.rs` | 一致 | 三个构造点；用例 + 行 2135 红 |
| K3 #7 `mount.rs` | 部分一致，两冲突一规则没说 | 见 K1·N1、N2 |
| K3 #8 `transaction.rs` | 一致 | 注释，旧错误成员 0 命中，U2 用例走这条路径；「扇出 147」没核 |
| K3 #9 `history.rs` | 一致 | N3 执行器、清单删空；kb 收口表第 43 行现在态未改，归写回批 |
| K3 #10 `model_comparison.rs` | 一致 | 只改注释，与实现一致 |
| K3 #11 `model.rs` | 一致 | 新理由三处穷举都加；无变异行，探针 P6a 补红 |
| K3 #12 `admission_raises_the_floor_before_refusing.rs` | 一致 | 14 条绿；是 N1 冲突的另一边 |
| K3 #13 `a_floor_raise_refused_for_space_counts_as_short_of_space.rs` | 一致 | 行 2139 复跑红 |
| K3 #14 `an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` | 一致（LR2 变异红复核不了） | 两条绿；LR3 行 2143 红；LR2 行 2144 超 225 秒 |
| K3 #15 `common_admission_pool/mod.rs` | 一致 | 助手臂跟着类型变 |
| K3 #16 `named_unit_verification_stands_in_for_the_barrier_before_the_record.rs` | 一致 | 三条绿；行 2161–2163 红 |
| K3 #17 `rollback_by_a_forward_publish.rs` | 一致 | 段和 195 = 166 + 29；行 2151 红 |
| K3 #18 `unlink_in_histories_judged_by_the_model.rs` | 一致 | 三条绿；行 2136–2138 红 |
| K3 #19 `unlink_mount_continuation_and_the_emptied_version.rs` | 一致 | 11 条绿；行 2134、2135 红 |
| K3 #20 `unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs` | 一致 | 一条绿；行 2141 红 |
| K3 #21 `writable_mount_of_a_formatted_pool.rs` | 一致 | LR6 照判决做，一条绿 |

## 7 没做什么

- 不判别的腿的格：K2（攻方）整格不做，也不替主 agent 采纳或出判决；上面「冲突」「规则没说」只是两边并排。
- 没跑 checker 档（`singlefs-checker-tier` 的测试、`crash_injection.rs` 的单测、`device_log.rs` 的单测、战役用例与变异行 2145–2150、2152–2160）；这些格的判定是读代码加 harness 档复核数字，不是跑出来的。
- 没跑 `random_histories` 的 512 段（判决 U11 验收「4 次被比到」）与全量 `cargo test`；只跑了单点用例与探针。
- 没核：`transaction.rs` 文档里的「内部扇出 147」；LR2 变异行 2144 的红（用例在 debug 下超过 225 秒线）；行 2142、2145–2150、2152–2160（后者 checker 档）。
- 没读禁读清单里的文件（`m3-unlink-code-r2-opus-output.md` 与它的模型目录、`/tmp/claude-1000/m3-unlink-r2-attack/`、`m3-unmount-after-short-continuation-r1-*-output.md`）。
- 机器负载在 80 到 96（`uptime` 里的 `load average: 82.41`、`96.39`），本报告里所有耗时只当参考。

### 清理

草稿副本 `/tmp/claude-1000/m3-unlink-r2-forward/tree`（15M）与编译目录 `/tmp/claude-1000/m3-unlink-r2-forward/target`（2.7G）已删；探针与脚本、日志副本留在 `research/prompts/m3-unlink-code-r2-sonnet-model/`（`logs/` 下 `probe-*.log`、`logs/runs/`）。我自己起的测试进程里有两个因 `run_row.py` 超时没被带走（pid 1295136、1299121，内容是行 2144 变异下的 LR2 用例），已按 `proc.py stop` 逐个停掉，再查一遍没有剩下的。
