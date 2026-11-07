# m4-gpugc-shared-bytes-code-r1：攻方腿（Opus）报告（2026-11-06）

立场：攻。假设「第 86 批只换了持有方式、语义照旧」与「第 88 批去重改名不改行为」是错的，造历史去打穿它。第一轮分工：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱；T2、T3 主攻。
代码引文写冻结副本里的路径与行号（`crates/…`）；其中十三份此刻与主工作区逐字节相同、三份开工之后被别的会话改过（那三份引冻结副本的绝对路径，见「这条腿自己的限度」）。kb 引文写 kb 文件自己的行号。

## 复跑命令与模型文件

模型目录：`research/prompts/m4-gpugc-shared-bytes-code-r1-opus-model/`。复跑整条（主 agent 交，命令原样在「要主 agent 跑的命令」第 1 条）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-shared-bytes-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1/repo --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-opus/rerun-1
```

| 文件 | sha256 | 是什么 |
|---|---|---|
| `t4_token_alignment.py` | ff5b7f997d2c84d03864f4d9089ce4efce1943d9b57e5763773b19bfba48e710 | T4 文本模型：第 88 批七个文件块按词法单元对齐删去行与加进行，列出换名与残余，再数冻结副本里旧名还剩几处；末行比两份 `low_and_high_words_of_the_slot` 的词法单元 |
| `probes.patch` | 1da9ef797072ea7917d2bb997bf496e61cacd4951ad8a939ef6bc6fd536ca711 | T1、T2 探针：五处 `Arc::make_mut` 之前各打一行 `OPUS_PROBE site=<处> strong_count=<数>`；`transaction.rs` 末尾加一条编译期断言（`TransactionOutput`、`PublishedUnit`、`PoolVersion` 是 `Send + Sync`） |
| `tools/rerun.sh` | 1db82fd2065e692fd3cccb56513d3f10ea6532c0c5bb490c2222dd89ca11102c | 从 `scripts/agents/three-way-rerun-template.sh` 拷，只改 `rerun_steps`：拷冻结副本、打探针、跑 T4 文本模型与六个测试目标（加 `--nocapture`，每条 `cargo test` 自带 target 范围声明），汇总每处的引用数 |
| `out/t4-token-alignment.txt` | c4fa7f98abd5181edc93385a35b4f8a9c6a295c3c961c668b28a2ea8e4801a7c | T4 文本模型这条腿自己跑出的输出（只读两份文本，见 T4 一节），复跑时与 `$WORK/out/t4-token-alignment.txt` 逐行比 |

探针那几样（引用数）这条腿没跑，读数全是「推的，待主 agent 跑」。

## 各格判定一览

| 格 | 判定 | 一句话 | 依据的性质 |
|---|---|---|---|
| T2 冻结重发那一刻引用数 | **打中声明，没打中语义** | 「那一刻只有一个引用，就地改」在一段测试历史上不成立：冻结的那一份被 `FrozenPublish::clone` 多拿一份，重发时 `Arc::make_mut` 先拷一整版。拷与不拷交回的内容、重发写出的字节都一样，也没有一方看到半截改动；产品代码里重发没有调用方，这条历史只在测试里 | 调用序列（读码，推的；引用数待主 agent 跑探针） |
| T2 与 D16（发布语义）已定项 17、C381 | 没打中这一批 | 拷一份再改只改 `writes` 一个字段，重发的字节取自冻结的 `PublishWrites`，与版本拷没拷无关：这一批没把重发变成「照旧复用同一份内存态重试」，也没碰 C381 那一格。另有一条与这一批无关的条款张力，作线索交主 agent（T2 一节末） | 读码，推的 |
| T3 共享与深拷并存时内容分叉 | 没打中 | 排除了十一条路径（T3 一节表）：共享与深拷混用的每一处，之后谁 `make_mut` 都只改自己那一份，按值时代本来也是各拿各的拷贝；第 89、90 批改坏字节的每一处改的都是本地 `Vec` 或换掉自己那一个 `Arc`，不碰别人指着的那一份 | 读码，推的 |
| T3 过度外推（测试注释） | 打中一句注释 | `publish_lookup_by_identity.rs:1021` 说「就地清零会连 version 一起改坏」：安全 Rust 里改不到共享着的 `Arc<[u8]>`（`make_mut` 先拷、`get_mut` 交 `None`），这句说的危险造不出来；代码本身对 | 读码，推的 |
| T1 读指针身份的地方 | 没打中 | 产品代码里没有一处对 `TransactionOutput` / `PublishedUnit.bytes` 读身份（`ptr_eq`、`strong_count`、`as_ptr`、`Weak`、`get_mut`、`try_unwrap` 逐个 grep）；读身份的只有 `mount.rs:14214` 起那一组测试，判的正是「共享」本身 | grep，量的（命令与输出在 T1 一节） |
| T1 `make_mut` 时序 | 没打中 | 五处每一处拷之前的别的持有者列在 T1 一节表里；`make_mut` 引用数多于一时先拷再改、等于一时就地改，两种情形下别的持有者都看不到改动，与按值时代逐格同义。这条等价靠三条前提，最脆弱的是「产品代码永不读身份」没有任何检查钉着（「哪一条最脆弱」一节） | 读码，推的；探针待主 agent 跑 |
| T4 去重与改名 | 没打中 | 七个文件块按词法单元对齐：换名全是一对一、旧名在测试模块里一处不剩；残余只有 rustfmt 的尾逗号与闭包体去花括号、`pub(crate)`、GIBIBYTE 搬家与一条 use；两份 `low_and_high_words_of_the_slot` 去注释后逐个词法单元相同；WGSL 源串、入口名常量、`include_str!` 一处没动 | T4 文本模型，腿自己跑的（主 agent 未复跑） |

## T2　冻结重发那一刻的引用数（主攻）

被攻的那句：第 86 批实现员报告（`~/.local/share/singlefs/work/m4-gpugc-shared-unit-bytes-writer/result.md` 第 22 行，仓外文件，`grep -nF` 现查过）写 `PoolVersion::with_writes`「冻结重发那一支走它：那一刻只有一个引用，就地改」。

### 打中的调用序列（测试历史，产品代码里走不到）

历史取冻结副本里现成的一条用例，`publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish`（「pub fn publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish(」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1/repo/crates/singlefs-verification/tests/common_images_for_the_given_states_engine/publish_failure_resent_unchanged.rs:153`）），测试目标 `publish_failure_resent_unchanged`。每一步指到许可它的那一句：

1. B 落盘中途报块设备错。交给落盘的那一版是这次新建的：「PoolVersion::of_a_version_of_trees(output),」（`crates/singlefs-core/src/transaction.rs:12670`，所在函数「fn publish_version_of_trees_relocating_units_after_a_check<Device: BlockDevice, Refusal, Check>(」（`crates/singlefs-core/src/transaction.rs:12619`）），它「PoolVersion::of_a_shared_version_of_trees(Arc::new(version))」（`crates/singlefs-core/src/transaction.rs:2447`）：此刻引用数 1。
2. 失败分支把这一版连同写表冻结进分配器：「allocator.freeze_publish(FrozenPublish {」（`crates/singlefs-core/src/transaction.rs:1598`）→「self.frozen_publish = Some(Box::new(frozen));」（`crates/singlefs-core/src/allocator.rs:2088`）。引用数仍是 1。
3. 用例把冻结着的那一次整份拷一份留着：「let frozen = pool」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1/repo/crates/singlefs-verification/tests/common_images_for_the_given_states_engine/publish_failure_resent_unchanged.rs:171`）……「.clone();」（同文件 `:175`）。许可它的是「#[derive(Clone, Debug)]」（`crates/singlefs-core/src/transaction.rs:1535`）：`FrozenPublish` 的 `version: PoolVersion` 照派生的 `Clone` 拷，`PoolVersion::WithFile(Arc<…>)` 拷的是引用。此刻引用数 2。`frozen` 这个名字在 `:188` 之后不再出现，但它活到函数末尾（`:279`），Rust 不提前放。
4. 重发：「resend_the_frozen_publish(&mut writer, &mut pool.allocator).expect("重发这一遍不再报错")」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1/repo/crates/singlefs-verification/tests/common_images_for_the_given_states_engine/publish_failure_resent_unchanged.rs:225`）→「let Some(frozen) = allocator.take_frozen_publish() else {」（`crates/singlefs-core/src/transaction.rs:1621`）→ 写完 →「version.with_writes(」（`crates/singlefs-core/src/transaction.rs:1637`）→「Arc::make_mut(version).writes = writes;」（`crates/singlefs-core/src/transaction.rs:2561`）。引用数 2，`make_mut` 先把整版 `TransactionOutput` 拷一份（单元字节是 `Arc<[u8]>`，只多一个引用）再写 `writes`。

于是「那一刻只有一个引用」在这条历史上不成立（推的，读码；引用数由探针 `site=with_writes` 那一行量，待主 agent 跑，预期这个目标里恰好一行 `strong_count=2`）。对照臂：`instance_table_sweep_stops_on_a_block_device_error` 的 `a_write_error_while_landing_the_sweep_publish_deletes_no_row_freezes_the_publish_and_the_next_sweep_deletes_the_row` 重发之前没拷 `FrozenPublish`（只拷了现行那一版的 `TransactionOutput`），预期重发那一行 `strong_count=1`——两条一红一绿，说明探针分得出拷与不拷（推的，待主 agent 跑）。

产品代码里这条序列走不到：`resend_the_frozen_publish` 在 `crates/` 里的调用方只有 `crates/singlefs-verification/tests/` 下的八份测试文件（`grep -rn 'resend_the_frozen_publish(' crates/` 现查，产品源码里只有定义那一行 `:1617`）；`allocator.frozen_publish()` 在 `singlefs-core/src` 里的九处（`mount.rs` 四处、`mounted_session.rs` 四处、`transaction.rs` 一处，逐处读过）都是读 checkpoint_txg 或判有没有，没有一处拷。别的多出引用的办法只有冻结期间整份拷分配器（`PoolAllocator` 也是 `#[derive(Clone, Debug)]`，「pub struct PoolAllocator {」（`crates/singlefs-core/src/allocator.rs:1867`）），产品里冻结期间拷的那几份（预演、试发、只供测试的开关）都在同一个函数里用完就放，活不到下一次重发（推的，读码）。

### 拷一份再改会不会变成已定项 17 禁的那件事，或让一方看到半截改动

不会，三条理由，各自可核：

- 改的只有一个字段。`with_writes` 只写 `writes`（「Arc::make_mut(version).writes = writes;」（`crates/singlefs-core/src/transaction.rs:2561`）），拷出来的那一份其余字段与冻结时逐字段相同；它是交给调用方的那一版，留在 `frozen` 里的那一份照旧是空账。按值时代 `FrozenPublish::clone` 本来就拷一整版，重发交回的是分配器里那一份、加写账——两种写法交回的内容逐字段相同（推的；探针不比内容，用例自己的断言 `:241`–`:254` 比重发交回那一版的 txg 与 jsn）。
- 重发写出去的字节不从版本里取。`persist_publish_writes(pool, &frozen.writes)` 发的是冻结的 `PublishWrites`（「if let Err(cause) = persist_publish_writes(pool, &frozen.writes) {」 这一行在 `crates/singlefs-core/src/transaction.rs:1625`），`with_writes` 在它之后才跑；拷不拷版本碰不到写盘的字节与次序。用例 `:239`–`:240` 判「失败那一遍落到盘上的每一步……都是重发这一遍的前缀、逐字节相同」，判的正是这一点。
- 看不到半截改动。`make_mut` 引用数多于一时改的是新拷的那一份，`frozen` 里那一份一个字节都不动；引用数为一时没有第二个持有者。`TransactionOutput` 里没有内部可变的成员（T1 一节，探针里另加一条 `Send + Sync` 的编译期断言钉它），没有 `Weak`（`grep` 现查，T1 一节）。

已定项 17 的原文：「报错之后不许假设可以照旧复用同一份内存态重试。」（`.claude/kb/decisions/16-发布语义.md:377`），射程：「只在「转只读」与「重做又失败」那几格上生效。」（`.claude/kb/decisions/16-发布语义.md:383`）。这一批让重发多了一次「也许拷一份」，复用的内存态（冻结的写表、冻结的那一版、发布之后那一份分配器）一样也没多、一样也没少，判不到这一批头上。

C381 那一行（「根已落盘之后发布失败，分配器仍退回」（`.claude/kb/checks-owed.md:265`））管的是失败之后分配器退回、再分出去的那一格；这一批在冻结那一支里只改了 `with_writes` 那一个赋值，分配器那三处（`:1596`–`:1597` 换回、`:1598` 冻结、`:1635` 换成发布之后那一份）一个字没动（`_m4-gpugc-shared-bytes-code-r1-diff.md` 第 86 批 `transaction.rs` 一节里冻结与重发只出现 `with_writes` 那一处，现查）。

### 线索（不是这一批的打中，交主 agent 判要不要另立）

重发又失败时「allocator.freeze_publish(frozen);」（`crates/singlefs-core/src/transaction.rs:1627`）把同一份 `FrozenPublish` 原样放回，下一次重发照旧发它——字面上就是「照旧复用同一份内存态重试」，而已定项 17 的射程里正好点了「重做又失败」那一格。这是 D23（journal 的角色与格式）已定项 14「逐字节原样重发」与 D16（发布语义）已定项 17 两条之间的张力，按值时代就在，与 `Arc` 无关；照第一轮分工它属于「内部矛盾」，但出了这一轮的攻击面（只攻补丁块），这里只记、不判。什么现象会推翻「这是张力」：已定项 17 的变更史或 D23 已定项 14 里有一句明写「原样重发不算复用内存态」。这条腿没去读变更史。

## T3　共享与深拷并存时的内容分叉（主攻）

找的形状：同一版在一处共享（`Arc<TransactionOutput>`）、在另一处按值深拷，之后有一方经 `make_mut` 或别的办法改了，另一方仍以为两处相同。要打中，得有一方「以为相同」——读身份，或文档、断言写着「最后一个就是 `current`」而改动发生在这句的射程之内。逐条排除如下（全是读码，推的）：

| 路径 | 共享处 / 深拷处 | 之后谁改、改的是谁的那一份 | 「以为相同」的那句管不管得到 | 结论 |
|---|---|---|---|---|
| 整理那一趟的走空 | 「versions_published.extend(walk.steps_published.into_iter().map(Arc::unwrap_or_clone));」（`crates/singlefs-core/src/mounted_session.rs:3788`）：最后一步与 `current` 共享，交回时拷一份 | 走空之后的整理批发布另起新版，不改 `current` 指着的那一份；`current` 换成新版 | 「这一趟发成的每一版，按先后：先走空的删除续做、再是整理的各批。最后一个就是会话的 `current`。」（`crates/singlefs-core/src/mounted_session.rs:345`）只说内容；没有一处按身份比 | 不分叉 |
| 整理的各批 | 「*current = PoolVersion::of_a_version_of_trees(published.clone());」（`crates/singlefs-core/src/mounted_session.rs:4657`，另一处 `:4778`）与「versions_published.push(*version);」（`crates/singlefs-core/src/mounted_session.rs:3918`）：两处各一份深拷 | 之后改 `current` 的只有下一次发布（换整份）与推抬 F（`make_mut`，拷） | 同上 | 不分叉；这里仍是整版深拷（单元字节只多引用），与「不再各拷一整版」那句（`transaction.rs:2427`）不矛盾，那句点名的是版本列表与挂载交回值，没点名整理 |
| 抬 F 那一串 | 「pub publishes: Vec<TransactionOutput>,」（`crates/singlefs-core/src/mount.rs:4382`）按值 | 抬 F 收的是 `&mut TransactionOutput`，五处 `make_mut` 里四处就是给它拷出自己的那一份（`mount.rs:4610`、`:8335`、`:8708`，`mounted_session.rs:5079`） | 抬 F 的版本列表与 `current` 按值时代本来就是两份 | 不分叉 |
| 挂载续做 | 「let mut walked = Arc::clone(current_version_of_trees);」（`crates/singlefs-core/src/mount.rs:8281`），续做交回的各步与 `walked` 共享 | 被空间拒时「Arc::make_mut(&mut walked),」（`crates/singlefs-core/src/mount.rs:8335`）拷一份再抬 F，交回的那一步不动 | `MountOutput::deletion_queue_continuation` 的文档只在「走到了空时」说最后一个就是 `Mounted::current`；推满仍不够（`Stopped`）与抬 F 报错两条出口上 `walked` 是抬过 F 的、续做最后一步是抬之前的——这两条出口不在那句的射程里 | 不分叉（射程外的不同，按值时代同样不同） |
| 写行与暖机之后推抬 F | `row_publish`、`warm_up_publishes` 与 `current` 共享 | 「Arc::make_mut(current_file_version),」（`crates/singlefs-core/src/mount.rs:8708`）拷一份再抬 F；写行那一版的写账在拷之前取（`:8696`–`:8699`） | 这几版之后只读写账、核隔离，不再与 `current` 比 | 不分叉 |
| 卸载 | `current` 与调用方还留着的版本列表共享 | 「Arc::make_mut(current_file_version),」（`crates/singlefs-core/src/mount.rs:4610`）拷一份再抬 F，调用方的 `current`（`&mut PoolVersion`）跟着看到 | 版本列表本来就是这次改动之前的那几版 | 不分叉 |
| 会话改动之后走空 | 「let mut published = Arc::new(published);」（`crates/singlefs-core/src/mounted_session.rs:3129`）与「.push(Arc::clone(&published));」（`:3132`） | 走空里推抬 F 经「Arc::make_mut(current),」（`crates/singlefs-core/src/mounted_session.rs:5079`）拷 | `VersionsPublishedInThisChange` 的文档写「最后一个是会话的 `current`（之后没再推抬 F 时）」，自己把推抬 F 排在射程外 | 不分叉 |
| 另发一批搬迁、推环 | 「versions_published.push(Arc::clone(&relocated));」（`crates/singlefs-core/src/mounted_session.rs:2135`）、「versions_published.push(Arc::clone(&next));」（`crates/singlefs-core/src/mounted_session.rs:3406`） | 之后只经发布换整份 | 同上 | 不分叉 |
| 走空本身 | 「*current = Arc::new(next);」（`crates/singlefs-core/src/transaction.rs:12319`）、「steps_published.push(Arc::clone(current));」（`:12320`） | 走空里没有就地改 | 「两处指着同一份」只在两次发布之间成立，下一步换整份 | 不分叉 |
| 交回自己的一份 | 「PoolVersion::WithFile(version) => Some(Arc::unwrap_or_clone(version)),」（`crates/singlefs-core/src/transaction.rs:2498`） | 拷出来的那一份归调用方，别人手里的不动 | 没有一方按身份认「我拿着的就是交回的那一份」（T1 一节的 grep） | 不分叉 |
| 单元字节 | 「pub bytes: Arc<[u8]>,」（`crates/singlefs-core/src/transaction.rs:3239`）：照抄的角色与上一版共用、写表的单元与版本的单元共用 | 产品代码里没有一处改 `Arc<[u8]>` 的内容：`make_mut` 五处全在 `Arc<TransactionOutput>` 上，对单元字节的写只有整个换掉（`Arc::from(…)`） | — | 不分叉 |

第 89、90 批改的测试里改坏字节的每一处（冻结副本现查）：

| 处 | 怎么改的 | 改到谁 |
|---|---|---|
| 「let inode_root = pool.output.unit(TransactionUnit::InodeRoot).bytes.to_vec();」（`crates/singlefs-verification/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs:101`） | 拷进本地 `Vec` 再改 | 只有本地那一份，再写进测试自己的镜像 |
| 「let mut unit = leaf.bytes.to_vec();」（`crates/singlefs-verification/tests/checker_narrow_invariants_and_abandoned_roots.rs:996`）、「let mut orphan = data_unit.bytes.to_vec();」（同文件 `:1508`） | 同上 | 同上 |
| 「let mut wrong_width = pool.output.unit(TransactionUnit::TreeTable).bytes.to_vec();」（`crates/singlefs-verification/tests/new_pool_file_creation_publish.rs:1505`） | 同上 | 同上 |
| 「let mut malformed = version.clone();」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1/repo/crates/singlefs-verification/tests/common_images_for_the_given_states_engine/publish_lookup_by_identity.rs:1022`）之后「malformed_mapping_root.bytes = Arc::from(vec![0; malformed_mapping_root.bytes.len()]);」（同文件 `:1028`） | 换掉 `malformed` 那一个单元的 `Arc` | 只有 `malformed`；`version` 那个单元的 `Arc` 不动，其余单元两边共用、都没人改 |
| 「let mut current_of_the_raise = TransactionOutput::clone(current_file_version);」（`crates/singlefs-verification/tests/abandoned_root_whose_account_is_unreadable_passes_when_the_current_account_records_later_units_over_its_slots.rs:153`） | 深拷一整版，抬 F 改这一份 | 只有这一份；会话的那一版不动（按值时代 `.clone()` 同义） |
| 「carried.bytes[..],」（`crates/singlefs-verification/tests/tree_nodes_and_the_central_mapping_on_mounted_read.rs:709`） | 只比，不改 | — |

结论：没打中。什么现象会推翻：在任一条上找到一处产品代码对单元字节或整版做身份比较，或一句文档在它的射程内声称「两处相同」而其中一方此前经过了 `make_mut`。

### 打中一句注释（过度外推）

「// 只换掉 malformed 那一份的根：克隆出来的一版与 version 共用单元字节，就地清零会连 version 一起改坏。」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1/repo/crates/singlefs-verification/tests/common_images_for_the_given_states_engine/publish_lookup_by_identity.rs:1021`）：后半句说的危险在安全 Rust 里造不出来——`Arc<[u8]>` 只解引用成 `&[u8]`，`.fill(0)` 编不过；要就地改只能经 `Arc::make_mut`（引用数多于一时先拷，改的只是 `malformed`）或 `Arc::get_mut`（共享时交 `None`）。代码的做法（换掉那一个 `Arc`）对，理由写错了。按共用约束「代码与脚本的注释、说明只写现状」这一条它是现状句，但现状说错了；改法：后半句改成「`Arc<[u8]>` 不能就地改，换掉这一个单元的字节」。只在我读码上判过、被攻过零轮。

## T1　读身份的地方与 `make_mut` 的时序

### 读身份的地方（量的：grep，冻结副本）

命令（在冻结副本根下跑，`mount.rs` 第 14200–14499 行那一组判共享的测试先滤掉）：

```
grep -rnE 'Arc::(ptr_eq|strong_count|weak_count|downgrade|get_mut|try_unwrap|into_inner|into_raw|as_ptr)' --include=*.rs crates/singlefs-core/src crates/singlefs-checker/src crates/singlefs-verification/src crates/singlefs-background-gpu/src | grep -v 'crates/singlefs-core/src/mount.rs:14[2-4][0-9][0-9]:' | cut -c1-140
```

原样输出：

```
crates/singlefs-verification/src/crash_amplification.rs:15172:            std::sync::Arc::ptr_eq(&held_before_adding, &held_now),
crates/singlefs-verification/src/crash_amplification.rs:15177:            std::sync::Arc::strong_count(&held_before_adding),
crates/singlefs-core/src/background_compute.rs:476:        Arc::ptr_eq(&self.shared, &other.shared)
crates/singlefs-verification/src/crash_library.rs:751:        open_libraries.insert(key.clone(), Arc::downgrade(&open));
crates/singlefs-verification/src/gpu_unit_checks.rs:2667:            !Arc::ptr_eq(&next_device, &timed_out_device),
crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:1072:                    Some(latest) if Arc::ptr_eq(&latest, chunk) =>
crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:1075:                        latest_chunks.insert(chunk_index, Arc::dow
crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:1121:            stored_chunks.insert(Arc::as_ptr(chunk));
crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:8670:        assert!(Arc::ptr_eq(
```

九处里没有一处的参数是 `TransactionOutput`、`PoolVersion` 的成员或 `PublishedUnit.bytes`：判崩溃放量的桶表索引、后台计算句柄的相等（「Arc::ptr_eq(&self.shared, &other.shared)」（`crates/singlefs-core/src/background_compute.rs:476`））、判定库句柄、GPU 设备租约、E202 的内存块。另查了裸指针（`ptr::eq`、`.as_ptr()` 落在名字里带 bytes / unit / version / output 的行）：只有 `gpu_unit_checks.rs:2544` 一处，比的是 Vulkan 实例，测试里。读身份的只剩「mod versions_and_unit_bytes_shared_instead_of_copied_tests {」（`crates/singlefs-core/src/mount.rs:14214`）那一组，它判的正是「指着同一份」（例「Arc::ptr_eq(&unit.bytes, &previous.unit(unit.identity).bytes),」（`crates/singlefs-core/src/mount.rs:14286`）），是钉实现的测试，不是产品分支。

内部可变：`TransactionOutput`（「pub struct TransactionOutput {」（`crates/singlefs-core/src/transaction.rs:3251`））的成员逐个是普通数据；`singlefs-core/src` 与 `singlefs-format/src` 里带 `Cell` / `RefCell` / `OnceCell` 的结构体成员共 16 处（`grep -rnE` 按「成员名: 类型」形状数、去掉 `::new` 那几行，数出 16）（读路径的计数、带生命周期的读者、写入口 `PoolWriter`），没有一处是 `TransactionOutput` 递进去的成员类型（推的：按成员类型名对的，没展开每个嵌套类型；探针里的 `Send + Sync` 编译期断言把这一条变成编得过 / 编不过，待主 agent 跑）。

### 五处 `make_mut` 拷之前的别的持有者（推的，读码；引用数待探针量）

| 处 | 别的持有者可能是谁 | 引用数多于一时 | 等于一时 | 该看到改动的一方 | 不该看到的一方 |
|---|---|---|---|---|---|
| `with_writes`（「Arc::make_mut(version).writes = writes;」（`crates/singlefs-core/src/transaction.rs:2561`）），落盘成功那一支（`:1605`） | 没有：这一版是这次 `Arc::new` 的 | — | 就地写账 | 调用方拿到交回值 | — |
| 同上，重发那一支（`:1637`） | 冻结期间拷出的 `FrozenPublish` 或整份分配器（T2 一节的序列） | 拷一份写账，交回 | 就地 | 调用方 | 拷出来的那份 `FrozenPublish`：留空账，没看到 |
| 卸载（「Arc::make_mut(current_file_version),」（`crates/singlefs-core/src/mount.rs:4610`）） | 调用方还留着的上一次改动的版本列表、挂载交回值 | 拷一份抬 F | 就地 | `current`（调用方给的 `&mut PoolVersion`，返回之后照样有） | 版本列表里那一份 |
| 挂载续做被空间拒（「Arc::make_mut(&mut walked),」（`crates/singlefs-core/src/mount.rs:8335`）） | 续做交回的最后一步；这一轮一步都没走成时是 `*current` 自己与写行、暖机那几版 | 拷一份 | 就地 | `walked`，各出口 `*current = …(walked)` | 续做交回的各步、`*current` 换上之前的那一份 |
| 写行之后推抬 F（「Arc::make_mut(current_file_version),」（`crates/singlefs-core/src/mount.rs:8708`）） | `row_publish` 或最后一次暖机 | 拷一份 | 就地 | `current` | 写行、暖机那几版 |
| 会话为空间推抬 F（「Arc::make_mut(current),」（`crates/singlefs-core/src/mounted_session.rs:5079`）） | 这次改动的版本列表最后一个 | 拷一份 | 就地 | 会话的 `current` | 版本列表 |

每一行的「该看到的一方」拿的是同一个 `&mut Arc`，`make_mut` 之后它指着改过的那一份；「不该看到的一方」拿的是另一个 `Arc`，`make_mut` 在引用数多于一时先拷、不碰它。两种情形下的可观测结果与按值时代逐格相同：按值时代这些持有者本来就各拿一份拷贝。`Arc::unwrap_or_clone` 那几处（`transaction.rs:2498`、`mounted_session.rs:3788`）交出去的一份归拿的一方，没有一方按身份认「我拿着的就是交回的那一份」（上面的 grep）。

结论：没打中。什么现象会推翻：探针跑出来某一处「不该看到的一方」的内容在 `make_mut` 之后变了（探针不直接比内容；要比得在探针里先拷一份内容再比，这条腿没做，写在限度里），或 `Send + Sync` 那条断言编不过。

## T4　第 88 批的去重与改名

### 文本模型（腿自己跑的，只读两份文本；主 agent 未复跑）

`t4_token_alignment.py` 读 `_m4-gpugc-shared-bytes-code-r1-diff.md` 第 88 批的七个文件块：去掉 `//` 起的注释（含文档注释），把删去行与加进行各切成词法单元，`difflib` 对齐；等长、全是标识符的替换记成换名（旧名 -> 新名），其余记成残余；再在冻结副本那份文件里数旧名还剩几处；末行把 `region_record_kernels.rs` 删掉的那份 `low_and_high_words_of_the_slot` 与 `live_slot_kernels.rs` 留下的那份逐个词法单元比。这条腿在草稿目录里起它的拷贝跑了一次（`python3 -I <草稿目录>/t4_token_alignment_draft2.py <改动全文> <冻结副本>`，退 0），输出原样（也在 `out/t4-token-alignment.txt`）：

```
== crates/singlefs-background-gpu/src/adapter_inventory.rs removed_tokens=61 added_tokens=72
rename WAIT_LIMIT_UNDER_TEST -> WAIT_LIMIT_IN_THE_SETTINGS_UNDER_TEST one_to_one=True old_name_left_in_frozen_file=0
residue insert old=[] new=[# [ cfg ( test ) ] pub ( crate )]
== crates/singlefs-background-gpu/src/background_gpu_compute.rs removed_tokens=112 added_tokens=112
rename card -> card_named_in_the_failures one_to_one=True old_name_left_in_frozen_file=0
== crates/singlefs-background-gpu/src/checksum_kernels.rs removed_tokens=5 added_tokens=5
rename the_two_kernels_are_separate_sources_with_their_own_entry_points -> the_table_driven_and_bitwise_checksum_kernels_are_separate_sources_with_their_own_entry_points one_to_one=True old_name_left_in_frozen_file=0
== crates/singlefs-background-gpu/src/gpus_of_this_host.rs removed_tokens=15 added_tokens=8
residue delete old=[const] new=[]
residue replace old=[: u64 = 1 << 30 ;] new=[,]
== crates/singlefs-background-gpu/src/live_slot_kernels.rs removed_tokens=87 added_tokens=93
rename an_outcome_the_kernels_never_wrote_is_an_unrecognized_verdict_word -> live_slot_outcome_the_kernels_never_wrote_is_an_unrecognized_verdict_word one_to_one=True old_name_left_in_frozen_file=0
rename outcome_bytes -> live_slot_outcome_bytes_of_the_words one_to_one=True old_name_left_in_frozen_file=0
rename the_two_kernels_are_separate_sources_with_their_own_entry_points -> the_walking_and_searching_live_slot_kernels_are_separate_sources_with_their_own_entry_points one_to_one=True old_name_left_in_frozen_file=0
residue insert old=[] new=[pub ( crate )]
residue insert old=[] new=[,]
residue insert old=[] new=[,]
== crates/singlefs-background-gpu/src/region_record_kernels.rs removed_tokens=627 added_tokens=581
rename an_outcome_the_kernels_never_wrote_is_an_unrecognized_verdict_word -> region_record_outcome_the_kernels_never_wrote_is_an_unrecognized_verdict_word one_to_one=True old_name_left_in_frozen_file=0
rename entry -> central_mapping_entry one_to_one=True old_name_left_in_frozen_file=15
rename outcome_bytes -> region_record_outcome_bytes_of_the_words one_to_one=True old_name_left_in_frozen_file=0
rename the_two_kernels_are_separate_sources_with_their_own_entry_points -> the_walking_and_searching_region_record_kernels_are_separate_sources_with_their_own_entry_points one_to_one=True old_name_left_in_frozen_file=0
residue insert old=[] new=[use crate :: live_slot_kernels :: low_and_high_words_of_the_slot ;]
residue replace old=[low_and_high_words_of_the_slot ( slot : SlotNumber ) -> [ u32 ; 2 ] { [ u32 :: try_from ( slot . 0 & u64 :: from ( u32 :: MAX ) ) . expect ( "取了低 32 位" ) , u32 :: try_from ( slot . 0 >> 32 ) . expect ( "盘上槽号是 6 字节，高 32 位装得进 u32" ) , ] } fn entry] new=[central_mapping_entry]
residue insert old=[] new=[,]
residue insert old=[] new=[,]
residue insert old=[] new=[,]
== crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs removed_tokens=311 added_tokens=317
rename cpu_reference_outcome -> outcome_the_cpu_reference_gives_at_the_position one_to_one=True old_name_left_in_frozen_file=0
rename entry -> two_kernel_outcomes_of_the_entry one_to_one=True old_name_left_in_frozen_file=9
residue insert old=[] new=[,]
residue insert old=[] new=[,]
residue delete old=[{] new=[]
residue replace old=[}] new=[,]
residue insert old=[] new=[,]
residue insert old=[] new=[,]
residue insert old=[] new=[,]
residue insert old=[] new=[,]
residue insert old=[] new=[,]
residue insert old=[] new=[,]
residue delete old=[{] new=[]
residue replace old=[}] new=[,]
residue delete old=[{] new=[]
residue replace old=[}] new=[,]
residue insert old=[] new=[,]
== low_and_high_words_of_the_slot removed_copy_diff_lines=1543-1549 kept_copy_live_slot_kernels_lines=73-78 identical_after_dropping_pub_crate=True
```

逐项读：

- 换名全是一对一（`one_to_one=True`）。旧名在冻结副本里还剩的只有 `entry`：`region_record_kernels.rs` 剩 15 处、`two_kernel_reconciliation.rs` 剩 9 处，逐处看过全在 `mod tests` 之前的产品代码里，是循环变量 `entry`（例 `region_record_kernels.rs:337` 的 `for (entry_position, entry) in mapping_entries.iter().enumerate() {`），不是改名前那个测试帮手；`mod tests`（`region_record_kernels.rs:1361`、`two_kernel_reconciliation.rs:146`）之后 `entry(` 一处不剩（`awk` 现查）。所以没有一处调用留着旧名、被 `use super::*` 引进来的别的同名条目接走。
- 残余：`# [ cfg ( test ) ] pub ( crate )`（GIBIBYTE 搬到模块级）、`pub ( crate )`（`low_and_high_words_of_the_slot` 放宽可见性）、`use crate :: live_slot_kernels :: low_and_high_words_of_the_slot ;`、`gpus_of_this_host.rs` 删掉本地 `const GIBIBYTE : u64 = 1 << 30 ;` 改成 use 里多一个 `, GIBIBYTE`，其余全是 rustfmt 换行带出的尾逗号与 `|_| { panic!(…) }` 去掉花括号成 `|_| panic!(…)`（闭包体同一个表达式）。`region_record_kernels.rs` 那一条 `replace` 是删掉的整个函数（对齐把紧随其后的 `fn entry` 并进了同一段）。
- 两份 `low_and_high_words_of_the_slot`：删掉的那份（改动全文第 1543–1549 行）与留下的那份（`live_slot_kernels.rs:73`–`:78`）去掉 `pub(crate)` 后逐个词法单元相同（`identical_after_dropping_pub_crate=True`），连两条 `expect` 的串都相同。46 的清单「甲：正文相同」对。
- GIBIBYTE 两份原值都是 `1 << 30` 的 `u64`，留下的那份「pub(crate) const GIBIBYTE: u64 = 1 << 30;」（`crates/singlefs-background-gpu/src/adapter_inventory.rs:337`）挂 `#[cfg(test)]`；包里定义 GIBIBYTE 的只剩这一处（`grep -rn 'const GIBIBYTE'` 现查）。

### 可见性、WGSL 与单元图

- `pub(crate)` 之后用 `low_and_high_words_of_the_slot` 的只有两份文件：`live_slot_kernels.rs` 三处、`region_record_kernels.rs` 三处（`grep -rn` 现查，包外没有，它也出不了包）。两处要的拆法相同（低 32 位、高 32 位），没有别的模块借它做别的事。
- WGSL：第 88 批的加删行里 `wgsl`、`KERNEL_SOURCE`、`ENTRY_POINT`、`include_str` 只出现在三条改名的测试函数名里（「the_two_kernels_are_separate_sources_with_their_own_entry_points」换成三个新名），核源串常量、入口名常量与 `.wgsl` 文件一个字没动（`awk 'NR>=1148' … | grep -iE` 现查）；这个函数是 Rust 一侧打包槽号字的，不进 WGSL 源串。
- 单元图：`region_record_kernels.rs` 里调它的三处（`:261`、`:354`、`:395`）改由 `use` 落到 `live_slot_kernels.rs:73`。按 `scripts/gate/lib_function_units.py` 文件头的 R-裸名 / R-结构，入口闭包经那条 use 收进 `live_slot_kernels.rs` 的那一个单元（推的，读规则）；改前两份同名同目标时裸名照规则可能把两份都收进来（只会多收），改后只收一份——闭包变窄到恰好、不会漏。要坐实，主 agent 跑「要主 agent 跑的命令」第 2 条，看闭包里有没有 `live_slot_kernels.rs:73-` 那一行。
- 旧测试名在冻结副本的仓内登记里（`crates/mutations.tsv`、`.claude/gate.d/` 的表）一处不剩（`grep -rlw` 五个旧名现查，只有 `WAIT_LIMIT_UNDER_TEST` 命中 `crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs`——那是另一个模块测试里自己的常量（`:529`，`Duration::from_secs(5)`），与 `adapter_inventory.rs` 改名前那一份同名同值，撞名就撞在这里；这一批改名的是 `adapter_inventory.rs` 那一份，值照旧 5 秒）。

结论：没打中。什么现象会推翻：复跑 T4 模型时某一条换名 `one_to_one=False`、或某个旧名在 `mod tests` 之后还剩、或末行 `identical_after_dropping_pub_crate=False`；或第 2 条命令打出的闭包里没有 `live_slot_kernels.rs` 那个单元。

## 哪一条最脆弱

「只换持有方式、语义照旧」站得住，靠的是三条前提一起成立：`Arc<T>` 配 `make_mut` 与 `unwrap_or_clone` 在这三条下与按值的 `T` 逐格同义（推的，Rust 标准库的语义；这一节不引别的项目）。

| 前提 | 今天成立吗 | 有什么钉着它 | 一旦破了，哪一格先红 |
|---|---|---|---|
| P1 被共享的三样（`TransactionOutput`、`PublishedUnit`、`PoolVersion`）没有内部可变成员 | 成立（推的，按成员类型名查；探针的编译期断言待主 agent 跑） | **没有**：仓里没有任何断言或测试钉它 | 有人给 `TransactionOutput` 加一个 `Cell` 计数，共享的几方就互相看得到改动；编译照过、`PartialEq` 照过 |
| P2 产品代码不读这几样的身份（`ptr_eq`、`strong_count`、按地址做键） | 成立（量的，T1 一节的 grep） | **没有**：`mount.rs:14214` 那组测试正向钉「共享」，没有一条反向钉「产品不按身份分支」 | 有人写 `if Arc::ptr_eq(&a, &b) { 跳过 }` 一类的省事判断，共享时跳、拷了不跳，同一段历史两种结果 |
| P3 不用 `Weak`、`get_mut`、`try_unwrap` 这类按引用数分支的接口 | 成立（量的，同一条 grep） | 没有 | `get_mut(...).expect(...)` 在有人多拿一份时（T2 那条序列就是一例）当场 panic |

最脆弱的是 P2：它是唯一一条「今天对、明天写一行就错、错了编译器与现有测试都不吭声」的前提，而且第 86 批自己的测试（`mount.rs:14214` 起）把「指着同一份」写成了要守的性质，等于在鼓励后来的代码依赖身份。P1 次之：成员类型今天是普通数据，但同样没人钉。T2 那条序列说明「引用数恰好为一」本身就不是一个能靠的性质——测试里随手一个 `.clone()` 就变成二；凡是把引用数当前提的说法（实现员报告那一句）都该删掉，代码只靠 `make_mut` 的语义、不靠引用数。

## 打中之后先答四句

| 打中 | 分不分辨臂 | 被判的系统当时看不看得到判别它的东西 | 满足的是判据字面的哪一个分句 | 跑前条款给的改法在打中那几格上还中不中 |
|---|---|---|---|---|
| T2「那一刻只有一个引用」不成立 | 不分辨：这一轮没有臂，被判的是一句声明；打中只让那句声明出局，`with_writes` 的代码不出局（它对一与多于一都对） | 看得到：`make_mut` 自己读引用数决定拷不拷；调用方看不到、也不需要看到 | 正文第三节 T2 的「核：冻结重发那一刻是不是真只有一个引用（版本列表、挂载交回值、调用方手里的那一份有没有还指着它）」——这条序列里多出的那一份是「调用方手里的那一份」（测试拷的 `FrozenPublish`），不是版本列表或挂载交回值 | 正文没给跑前改法；我提的改法（删掉那句声明）在这一格上修得到，见下表 |
| T3 测试注释的理由写错 | 不分辨（同上，判的是一句注释） | 不适用（注释，不是运行时判断） | 正文第三节 T3 的「`Arc<[u8]>` 的单元字节被测试或装置当 `Vec<u8>` 就地改坏……改的是不是只有被测的那一份」：改的确实只有那一份，打中的是注释给的理由，不是行为 | 同上 |

## 改法与各修哪一格

全部只在我的读码上判过、被攻过零轮。没有一格量过。

| 改法 | 修哪一格 | 状态 |
|---|---|---|
| 删掉实现员报告里「那一刻只有一个引用，就地改」；`PoolVersion` 文档（`transaction.rs:2427`–`:2428`）那句「一版发出来之后不再改」改成「别人指着的那一份不再改」（`with_writes` 与五处 `make_mut` 在引用数为一时就地改的正是发出来的那一份） | T2 声明那一格；T1 措辞 | 推的，没实现 |
| `publish_lookup_by_identity.rs:1021` 后半句改成「`Arc<[u8]>` 不能就地改，换掉这一个单元的字节」 | T3 注释那一格 | 推的，没实现 |
| 在 `crates/singlefs-core` 里加一条编译期断言：`TransactionOutput`、`PublishedUnit`、`PoolVersion` 是 `Send + Sync`（探针里那一条的样子），钉 P1 | 最脆弱一节 P1 | 推的，待主 agent 跑（探针里有，跑过才算量过） |
| 钉 P2：一格静态检查，`singlefs-core/src` 产品代码（`#[cfg(test)]` 之外）出现 `Arc::ptr_eq` / `strong_count` / `get_mut` 而参数类型是这三样之一的判红。按名字 grep 判不出参数类型，要么接进 `scripts/gate/lib_function_units.py` 的闭包，要么退一步只按「产品代码里出现 `Arc::ptr_eq(` 就要登记理由」判 | 最脆弱一节 P2 | 推的，没实现；落点与形状要主 agent 定 |

## 要主 agent 跑的命令

1. 模型整份复跑（探针 + T4 文本模型）。
   - 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-shared-bytes-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1/repo --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-opus/rerun-1`
   - 在哪起：仓根（脚本只认参数给的路径，工作目录随意；判决复跑照规则拷模型目录到主 agent 自己的草稿目录，`--model` 指过去、`--work` 换成那边的新目录）。脚本里六条 `cargo test` 各带 `SINGLEFS_RUN_SCOPE='种类=target; …; 范围=1 个测试目标; …'`，外面这一条不用声明；不套包装、不写线程。
   - 估时：推的，没量过。冷编 `singlefs-core` 的 lib 测试与 `singlefs-verification` 的四个测试目标，估 15–30 分钟；不起 GPU，不碰崩溃枚举。
   - 跑完看：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`（脚本 `set -euo pipefail`，任一条 `cargo test` 红就非 0、红在哪看 `$WORK/out/<目标>.log`）；`task-server/task log <号>` 里 `== <目标>` 与 `summary <目标>:` 那几行（也在 `$WORK/out/probe-summary.txt`）。预期（推的，跑前写下）：
     - `summary core-send-and-sync:` 有 `site=send_and_sync_compiled` 一次——编得过即 P1 成立；编不过整条退非 0，那就是 P1 被打穿，T1 一节结论翻面。
     - `summary publish-failure-resent-unchanged:` 里 `site=with_writes strong_count=2` 恰好 1 次（T2 那条序列），其余 `with_writes` 都是 `strong_count=1`。若是 `strong_count=1`，T2 的打中撤回。
     - `summary instance-table-sweep-block-device-error:` 里 `site=with_writes` 全是 `strong_count=1`（对照臂）。若这里也出 2，探针分不出拷与不拷，上一条作废。
     - 另三个目标里各处的 `strong_count` 这条腿没预言具体数；任一处出现多于一，都只说明那一处拷了，不改结论（T1 一节：拷与不拷可观测结果相同）。
     - `diff /home/fy5090/code/singlefs/research/prompts/m4-gpugc-shared-bytes-code-r1-opus-model/out/t4-token-alignment.txt <WORK>/out/t4-token-alignment.txt` 无输出。
   - 跑完删：`<WORK>/tree`（仓副本，约 50M 起，带编译目录）由主 agent 删；编译目录经服务给的照 `task-server/task clean <号>`。
2. 单元图闭包：`region_record_kernels.rs` 调的那个函数落没落到 `live_slot_kernels.rs`。
   - 命令：`task-server/task submit -- python3 /home/fy5090/code/singlefs/scripts/gate/lib_function_units.py closure /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1/repo crates/singlefs-background-gpu/src/region_record_kernels.rs::packed_region_record_chunk --judgment-kind product-behavior`
   - 在哪起：仓根。只读冻结副本，不写。
   - 跑完看：`exit=0`；`task-server/task log <号>` 里有一行种类 `fn`、文件段是 `crates/singlefs-background-gpu/src/live_slot_kernels.rs:73-78` 的单元，且没有 `region_record_kernels.rs` 里同名的单元。退 4（拒算）或没有那一行，T4 单元图那一条翻面。
3. 交回前已跑的引文核对（直接跑的文本工具，不经服务）：`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-shared-bytes-code-r1-opus-output.md`，末行贴在文末。

等 1、2 的结果才能定的：T2 打中是不是「量的」（现在是读码推的）、P1 是不是量过、T4 单元图那一条。

## 没打中的形状

- T1：读身份——四个包产品源码里 `Arc::` 的九种身份 / 引用数接口全扫（命中九处，全与这几样无关），加裸指针 `ptr::eq`、`.as_ptr()`；内部可变——core 与 format 里按「成员名: 类型」形状数出的 16 处 `Cell` 类成员，逐个对到所在结构体；时序——五处 `make_mut`，每处列了可能的别的持有者与两种引用数下谁看得到。
- T2：重发的八份测试文件逐个看重发之前有没有拷 `FrozenPublish` 或分配器（只有 `publish_failure_resent_unchanged` 一份拷了）；产品代码里重发的调用方（零处）、读冻结的九处、冻结期间整份拷分配器的几处（都在同一个函数里用完）。没扫：会话、挂载之外别的 crate 里有没有把 `FrozenPublish` 存进长命结构——`grep` 只看了 `resend_the_frozen_publish(` 与 `.frozen_publish()`。
- T3：十一条共享与深拷混用的路径（表），第 89、90 批改到的七处测试代码（表）。没扫：第 86 批改到、但不在 89、90 批的那几十份测试文件与装置里的 `make_mut`（`singlefs-verification` 里 `grep -rn make_mut` 数出 22 处，含 `src/` 的装置与 `history.rs`），读过的三处都是拿会话的 `current` 拷出自己的一份、交给抬 F 或回退：读过三处（`discard_while_a_publish_is_frozen…:200`、`discard_after_the_frozen_publish_is_resent…:207`、`common_compaction_deletion_prefix/mod.rs:397`）形状相同，其余没逐处读。
- T4：七个文件块全量对齐；旧名在冻结副本里还剩几处逐个看；两份函数逐词法单元比；WGSL 相关的加删行全扫；旧测试名在仓内登记里全扫。

## 这条腿自己的限度

- 结论几乎全是读码推的。量过的只有：T1 的 grep、T4 的文本模型（腿自己在草稿目录起的拷贝，只读两份文本，主 agent 未复跑）。探针的引用数、`Send + Sync` 断言一个都没跑。
- 探针只打引用数，不比「不该看到改动的一方」的内容。要直接量「看没看到」，得在每处 `make_mut` 之前把别的持有者的内容拷一份、之后比——这会多出一个持有者、改了引用数本身，所以没做；这一条完全靠 `make_mut` 的语义推。
- 「与按值时代逐格同义」是推的：没把改前的代码编起来、在同一段历史上与改后逐字段对拍。
- 引文核对：交回前把被引的十六份代码文件与冻结副本逐份 `sha256sum` 比过，十三份相同、三份不同——`publish_failure_resent_unchanged.rs`、`publish_lookup_by_identity.rs`、`instance_table_sweep_stops_on_a_block_device_error.rs` 在开工之后被别的会话改过（主工作区里 `publish_lookup_by_identity.rs` 的同一行已挪到第 1020 行），这几份的引文写冻结副本的绝对路径，其余写仓内相对路径；开头一节说「与主工作区逐字节相同」只对那十三份成立。别的被引文件没比，判决照规则用倒推出的树再核。
- 没读 D16（发布语义）已定项 17 的变更史与 D23（journal 的角色与格式）已定项 14 全文；T2 末那条线索只凭两条已定项的现文。
- 这是一份样本的「没打中」，按规则一次不算，T1、T3、T4 的「没打中」要再抽一份才记。

## 没做什么

- 不判正推那几格；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数（探针、闭包）都没复跑，命令在上一节。
- 没起 cargo、没起 GPU、没跑门禁阶段；没用崩溃枚举（这条腿的攻击面里没有崩溃点要枚举，3b 的取样与估时条款不适用）。
- 没改冻结副本；探针补丁是拷三份文件到草稿目录改出来的 `diff -ru`。

## 清理与引文核对

- 删了：草稿目录里探针用的三份源文件拷贝（`probe/`，`du -sh` 4.3M）与 `__pycache__`（24K）；这条腿没建仓副本、没有编译目录。草稿目录里留着的只有 T4 文本模型的拷贝与它的输出（`t4_token_alignment_draft2.py`、`t4-draft2.txt`）和引文核对的输出，都不是副本。
- 「要主 agent 跑的命令」第 1 条会在草稿目录下建 `rerun-1/`（含仓副本与编译输出），由主 agent 跑完删。
- 引文核对：`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-shared-bytes-code-r1-opus-output.md`，退 0，末行：

```
  ✓ 核了 50 处引文、31 处裸行号，对上 50 处，没判 31 处（其中裸行号没有引文可比 23 处；逐处列在上面）
```
