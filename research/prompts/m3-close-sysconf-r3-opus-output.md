# m3-close-sysconf-r3 攻方腿（Opus）报告（2026-10-20）

立场：照正文第四节分工表「云端攻方」那一行，攻用户 2026-10-19 选定的甲'、α'（加扫单元写序）、乙'、丙'，与 S3、S6。代码读第一轮那份树副本 `/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree`，引代码写那份副本的绝对路径与行号。凡写「量过」的都是攻方副本上的数（树副本拷贝 + 补丁），不是入库装置上的数。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-close-sysconf-r3-opus-model/`。复跑（整条经内存包装；`PROBE_SEED_TARGET` 可选，指到一个用同一份 `Cargo.lock` 编过的 release 编译目录，只拷第三方产物）：

```
PROBE_SEED_TARGET=<一个 release 编译目录的上一层> bash scripts/hosts/run-with-memory-cap.sh 12G bash research/prompts/m3-close-sysconf-r3-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs \
  --frozen /home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree \
  --work <新的草稿目录>
```

它拷冻结副本，补仓根的 `Cargo.toml`、`Cargo.lock`、`.cargo`、`litmus/`、`.claude/heavy-thread-default`；打 `two-rounds.patch`（第二轮攻方那一份原样：`SINGLEFS_PROBE_ACQUISITION=two_rounds` 打开取号写两轮，`SINGLEFS_PROBE_ROLLBACK=per_device` 回卷每块盘一次，两样都设就是甲'；另把 `crash.rs` 里逐状态走的那个函数改成 `pub`）；跑 `tools/patch_behind_arms.py`（第一轮攻方那份加一臂：`SINGLEFS_PROBE_BEHIND=record_units` 是第一轮的读法「缺单元那一半只看本盘有没有那一版末条记录」，`record_or_tail_units` 是放宽读法「tail 不落后或有末条记录都不算落后」）；加探针 crate `opus-r3-probe`；把 `release-profile-overrides.toml` 接到工作区清单末尾（几个大包 release 下 opt-level 1、不带调试信息）；编探针（`cargo build`，编出来的二进制直接执行）与 harness 档测试二进制（`--no-run`）；跑探针四份、与 `out/` 逐份比打 `same` / `differs`；harness 红名单两臂各跑一次、落后支那份用例两种乙' 读法各跑一次；`RERUN_HARNESS_ALL=1` 时 243 个目标全跑。

`sha256sum`（在模型目录里跑）：

```
80c2cbcb270fc0ea2b60c12900c5be0cd7f134336a5cc3f0d4a3839023b2effc  ./out/alpha-prime-and-rollback.txt
65e19aebe706b12fcc71c69aae5dc57ce31b655d5110a11b94a2e6b9fb835e3d  ./out/counts-and-snapshot.txt
1a72a8385e620d9cf7066975cb9cb9f872c56b9a3e2bdbbd6058ca46bba2283c  ./out/harness-crosschecks.txt
75b24d0e163155ebea641f091bf33cb05a8cb06f83575ae57d2cf2474d54783a  ./out/harness-red-assertions.txt
5683d6ca987338bc381ef985a8df2515ad2d35658499decae25f6d5330d11f63  ./out/harness-two_rounds-per-target.txt
99eb44be6f25fd55a07668e1504257e94f25f88edae550c57b7d2602e481d253  ./out/s3-one_round-and-scan.txt
23d11f29b5fd118eaa610c3f985376cdf2aefb573c8c1432dfeefe7b1091b1f9  ./out/s3-two_rounds.txt
b93206fb5ace302e1f846848aedf4ac50a75c9cf5f2f2c919feafaa1a7704ff2  ./out/snapshot-publish-writes.txt
b8d0c80d7427895ed1bc0a004e9bdeb7aabf57582cff101e756af26228b1f91f  ./probe-crate/Cargo.toml
2ed0d65fad9e8923a8d48b6323dfc6a8375d0b93077d2e61c5a38e0cec047d16  ./probe-crate/src/main.rs
b4f3b04b074495697f357ec67652d34f6769f0a61083b5ab2ee65dd188d3e87f  ./probe-crate/src/probe_alpha_prime.rs
ea85624c3393cf50da4916fde000086d41e5859cd72aa9cc24508dd48631b7cb  ./probe-crate/src/probe_alpha.rs
8cd266a07808697a701243d051137b67c1d47af12a515b57d6216fa62c2fbc86  ./probe-crate/src/probe_body.rs
2f90e733a353dcf5cb858f059270700b594739f5e6ac9ee18c0a5eb73bc1cf50  ./probe-crate/src/probe_r3.rs
260f8e26d0ef8c66635a977e2dd8eff08b4ba03853ffc79b072d4b93c901c781  ./release-profile-overrides.toml
b027180c3b08200bf0f372c6361a5a095a0094d442d3bc1e87c5a0f8b9778a03  ./tools/harness-slice.sh
3426e99985af362e2ca25bb17691b457c0b344ae04676c6399e10bd695996404  ./tools/patch_behind_arms.py
33277b8834dbd68c1cfd6eb3fc528ee4fc393076c58e7a6628db2ab8170bd323  ./tools/rerun.sh
c5539556402a59f891238022c923d473a4938be8492e9b29f64c3d0677d7ce89  ./two-rounds.patch
```

`out/` 是我在草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r3-opus/` 里用同一份源码跑出来的原样行（`grep -o 'probe=.*'` 抽出）。`rerun.sh` 没有端到端跑过一次（见「这条腿自己的限度」）。

## 各格判定一览

| 格 | 对象 | 判定 | 量过 / 推的 |
|---|---|---|---|
| U1.1 | 甲' 在 harness 档现有用例上的牵连面 | 243 个目标：20 个转红（今天的一臂下同 20 个全绿），218 个绿，5 个在 debug 下 232 秒内没跑完（其中 1 个重跑之后绿）。红的断言全是写死的世代号、槽位奇偶、写数段形、按序号注入故障的落点；没有一条读得出是语义违例（这一句是读断言文字推的） | 量过（副本） |
| U1.1 | `.claude/kb/layout/` 字节表 | `01-first-txn.md` 8 行写死世代号或取号写数（英文版同 5 行）；取号之后最新槽的奇偶翻面：新池新建文件之后最新槽由槽 1 世代 5 变成槽 0 世代 6 | 行号现查；奇偶量过 |
| U1.2 | 登记崩溃流的状态数 | 新池新建文件流：今天 16777260（与 checker 档用例钉的 `FULL_STATES` 逐位相同），甲' 16777268；可写挂载流三个起点 20511 → 20519；挂上再覆盖写一次 1069098 → 1069106。每次取号 +8（带撕裂）/ +3（闭式），两块盘 | 量过 4 条流；其余登记流推的 |
| U1.3 | 回卷每块盘一次 | 没打中：两起点 × 第 1–4 次系统配置写报错 × 只那一次 / 从那一次起，回卷流全部崩溃状态都挂得上，没有一个状态出现一块盘两槽全废；回卷写之间不用再加屏障（每段每块盘至多一写） | 量过 |
| U1.3 | 三块盘几何 | 今天的实现量不了：mkfs 断言盘数是 2 | 量过（panic 原样）；三块盘的结论推的 |
| U2 | α' 断言 T、T−1、T+1 × 三种盘面 | **打中**：断言 T−1、T 的根与记录都读不出时，α' 照写，之后可写挂载停在 txg 4，丢了已确认的 txg 5（两条取号臂、扫与不扫四格都丢）；其余 16 格不丢、不撞 | 量过 |
| U2 | 持续读错的槽 | α' 写了也照旧拒（甲' 之上） | 量过 |
| U2 | 扫单元区挂钟与内存 | 384 槽小盘两块：读 2782 槽、0.002 秒、内存只留一个最大值；默认几何与真盘没量 | 小盘量过；大盘推的 |
| U2 | 扫描漏掉读不出写序的单元 | 今天的一轮 C555 形 20502 个状态：不扫撞 16383、扫 0；把本实例解得开的单元全翻坏再扫，撞 16383，池级 checker 红 0 | 量过 |
| U3 | 乙'（两种读法）整份落后支用例 | 甲' 之上 42 条全过，两种读法都是 | 量过 |
| U3 | 发布单元写到一半取下的快照盘 | 乙' 两种读法与今天同拒；「缺的份数不到 2，只在有末条记录时不算缺」按定义与乙' 等价（有末条记录时乙' 已不读单元），同拒 | 乙' 量过；加项推的 |
| U3 | (实例代号, 计数器) 在甲' 下认错 | 没打中 | 推的 + 第二轮量过的撞号 0 |
| U4 | 丙' 第三件 | 两槽全废的盘的根环、journal 环读得到（位置来自别的盘的系统配置与常量），但这块盘是谁只由调用方的盘表说了算。**打中（推的）**：同 fsid 克隆盘在克隆那边只写了单元就崩、两槽系统配置又全废，四件闸都过，补写之后取号与克隆的残留单元同号 | 推的 |
| U5 | S3「自证不过」 | **打中**：今天择系统配置那一判不按 fsid 筛，别池 fsid 的槽世代号更大时整池报 `SystemConfigurationsDisagree`；单故障（一槽 fsid 不符）在 k2 起点 20519 个状态里 16404 或 4115 个（看坏的是哪一槽）可写挂载拒；丢确认过的写 0 | 量过 |
| U5 | S6 不收 | 没打中：没有读故障的 61549 个崩溃状态（原始镜像与挂上之后）I-8.6 红 0；这几条流造不出那一形 | 量过 0；造不出推的 |

## U1　甲' 撑不撑得住，牵连面与状态数

实现：副本里的甲' 就是 `two-rounds.patch` 的两轮加 `SINGLEFS_PROBE_ROLLBACK=per_device`。回卷对传进来的每一项各写一次：「for &index in written {」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/transaction.rs:680`），每块盘只传一次就是「每块盘只回卷一次」。

### U1.1 harness 档转红清单（量过，副本）

跑法：harness 档 242 个集成测试目标加库内单元测试，逐个目标一条 `cargo test -p singlefs-harness --test <目标>`（每条带 target 范围声明，子 agent 起整包被范围声明闸拒，所以拆成逐目标），debug 编译，经内存包装，甲' 一臂全跑；红了的目标再在今天的一臂下跑一次。逐目标退出码在 `research/prompts/m3-close-sysconf-r3-opus-model/out/harness-two_rounds-per-target.txt`，两臂对照在 `research/prompts/m3-close-sysconf-r3-opus-model/out/harness-crosschecks.txt`。

- 甲' 下红 21 个，其中 `publish_order_matches_litmus` 是副本里缺 `litmus/` 目录（两臂都报读不了 `.litmus` 文件）；补上目录之后它在甲' 下绿：「publish_order_matches_litmus with_litmus_dir arm=two_rounds rc=0」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-crosschecks.txt:25`）。同样补了目录之后 `a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error` 今天绿、甲' 红：「a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error with_litmus_dir arm=two_rounds rc=101」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-crosschecks.txt:24`）。
- 所以甲' 造成的红是 20 个目标；这 20 个在今天的一臂下全绿（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-crosschecks.txt` 第 1–21 行的 `baseline_rc=0`，那两个 litmus 目标看第 22–25 行）。
- 没跑完的 5 个（debug 下单个目标超过 232 秒）：`accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf`、`an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping`、`checker_known_bad_images`、`inode_leaf_absorb_and_borrow_on_the_unlink_path`、`inode_leaf_merge_and_borrow_against_an_independent_reference`。重跑一次之后 `an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping` 绿：「an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping arm=two_rounds retry rc=0 test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 215.27s」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-crosschecks.txt:29`）；另 4 个没判，写进「要主 agent 跑的命令」。

20 个红目标、每个的首条断言（整份在 `research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt`）与归类。归类是我读断言文字推的，没有逐条改测试复核：

| 目标 | 首条断言（原样摘自产物那一行） | 归类（推的） |
|---|---|---|
| `a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is` | 「该交回 FloorRaiseFailedAfterTheMountsPublishes，实际 Publish(PublishSequenceFailed」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:2`） | 按序号注入的写错落点前移两写 |
| `acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish`（8 条全红） | 「DeviceIdentity(0)：取号写世代 2、实例代号 1、tail 0」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:4`） | 写死世代号 |
| `admission_raises_the_floor_before_refusing`（8 条） | 「已落盘的发布一次一份，份数等于设备一层落下的根槽写数」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:13`） | 注入落点前移、写账合计写死「取号那两次」 |
| `a_writable_mount_with_system_configuration_slot_zero_unreadable`（3 条） | 「槽 0 是最新的世代 8、槽 1 是世代 7」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:22`） | 写死世代号与哪一槽最新 |
| `a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error` | 「取号与写行那次零单元发布发到盘上的每一步，止于盘 1 那道报错的屏障之前」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:26`） | 写死步序列 |
| `core_review_unit_area_start_and_publish_limits`（2 条） | 「该交回 FloorRaiseFailedAfterTheMountsPublishes，实际 Publish(PublishSequenceFailed」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:28`） | 注入落点前移 |
| `fault_injection_fast_tier` | 「取号之后第 3 次写报错：重开的盘上系统配置还是新号 2、不回卷成 1」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:31`） | 注入落点前移：第 3 次写在甲' 下是第二轮取号写，报错走回卷 |
| `instance_acquisition`（6 条全红） | 「取号写世代 6 带新号 2 留着，世代 5 那一槽没被回卷写覆写」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:33`） | 写死世代号、屏障数 |
| `new_pool_file_creation_publish`（9 条） | 「取号两写 + 取号之后那道屏障，两块盘各记一步」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:40`） | 写死步序列 |
| `new_pool_file_creation_write_regions` | 「表里每一行都要有一条写落在它上面」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:50`） | 写区表按槽奇偶写死（没核） |
| `overwrite_in_one_instance` | 「mkfs 1、取号 2、暖机 3 / 4、A 5、B 6」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:52`） | 写死世代号 |
| `recovery_reads_the_created_file_after_reopen` | 「择回的系统配置槽带的是上一次发布的 tail：它比环里最新的记录旧一格」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:54`） | 用例翻坏偏移 4096 那一槽、假定它是最新槽；甲' 下最新槽在槽 0 |
| `rollback_by_a_forward_publish` | 「整条流到 D 为止的段序列」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:56`） | 写死段形 |
| `row_publish_record_without_its_root` | 「根槽写之前是取号两写、六个单元各两写、记录两写」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:58`） | 写死写数 |
| `unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over` | 「默认环下 mkfs 到第一个文件写出的两块盘与钉值那一版（mkfs 写 incompat 0x06 之后）逐字节相同」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:60`） | 整盘字节钉值 |
| `warm_up_journal_counter` | 「取号两份 + 三次发布各两份」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:62`） | 写死写数 |
| `writable_mount_acquisition_write_reads_the_system_configuration_slots_from_the_disk` | 「前提：盘 0 槽 0 是这块盘最新的一槽」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:64`） | 槽奇偶 |
| `writable_mount_of_a_formatted_pool`（2 条） | 「要取的号 1」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:66`） | 用例让说谎的设备吞掉「取号那两次」槽 0 写，甲' 的第二轮落在槽 1、号 1 留在盘上，下一次取 2；另一条按读序号注入（没核） |
| `writable_mount_rehearsal_and_the_real_run_read_each_block_once` | 「这次挂载写的字节数与改之前相同」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:69`） | 写死字节数 |
| `write_accounting_per_structure` | 「取号两盘各一次系统配置槽 + 写行发布 + 各次暖机按种类的合计 == 录制器在挂载这段里记下的写」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-red-assertions.txt:71`） | 写账「取号各一次」写死 |

`writable_mount_of_a_formatted_pool` 那一条值得单看：用例的前提是「取号写被吞、号 1 不在盘上、下一次重取 1 无害」；甲' 下第二轮那一写没被吞，号 1 留在槽 1，下一次取 2。结局更保守（不重发号），但用例的意图（重发号无害）要换一个造法才测得到。

### U1.1 字节表里写死世代号与取号写数的行（行号现查）

甲' 之后新池新建文件这一段的世代号是：mkfs 1，取号 2（槽 0）、3（槽 1），暖机 4（槽 0）、5（槽 1），新池新建文件发布后 6（槽 0）。量过：同一条流挂上之后再覆盖写一次，盘上是「sysconf=d0[g6i1t3,g7i1t4]」（`research/prompts/m3-close-sysconf-r3-opus-model/out/counts-and-snapshot.txt:19`）那一形，新池新建文件那一版的系统配置是槽 0 世代 6、tail 3。今天是槽 1 世代 5。要改的行：

- 「| a1 | 第一次可写挂载取号 | 系统配置槽 0，世代号 2，实例代号 1」（`.claude/kb/layout/01-first-txn.md:62`）：取号要多一行 a2（槽 1，世代号 3）。
- 「| w3 | 暖机 | 系统配置槽 1，世代号 3，tail = 1」（`.claude/kb/layout/01-first-txn.md:65`）：变成槽 0 世代 4。
- 「| w6 | 暖机 | 系统配置槽 0，世代号 4，tail = 2」（`.claude/kb/layout/01-first-txn.md:68`）：变成槽 1 世代 5。
- 「| t15 | 事务 | 系统配置槽 1，世代号 5，tail = 3」（`.claude/kb/layout/01-first-txn.md:83`）：变成槽 0 世代 6。
- 「| 自举头 | 槽世代号 | 8 | mkfs 1，取号 2，暖机 3、4，新池新建文件发布后 5」（`.claude/kb/layout/01-first-txn.md:119`）。
- 「根槽之后写槽 1，世代号 5，tail = 3（取号写槽 0 世代 2；暖机两次各写一次：槽 1 世代 3、槽 0 世代 4）」（`.claude/kb/layout/01-first-txn.md:396`）。
- 「2 次操作、4 个崩溃状态」（`.claude/kb/layout/01-first-txn.md:411`）：取号一段变两段，状态数见 U1.2。
- 「取号系统配置槽 × 2 盘，世代号 3」（`.claude/kb/layout/01-first-txn.md:415`）。

英文版同样 5 行：`.claude/kb/layout/01-first-txn.en.md` 第 62、83、118、395、410 行（`grep -n -i 'generation' 01-first-txn.en.md` 现查）。`02-second-txn`、`03-third-txn` 三种语言里「世代」都不指系统配置槽世代号。

checker 档用例里写死的状态数与段数（没跑，推的：每条含取号的流状态数变，下面这些断言跟着红）。现查命令 `grep -n -E 'const [A-Z_]*(STATES|SEGMENTS)[A-Z_]*|closed_form_state_count\(&prepared.segments\), [0-9]|segments.len\(\), [0-9]' crates/singlefs-checker-tier/tests/*.rs`，例：

- 「const FULL_STATES: u64 = 16_777_260;」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:752`）与「const FULL_STATES_WITH_TWO_STATES_PER_WRITE: u64 = 16_777_240;」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:748`）。
- 「assert_eq!(closed_form_state_count(&prepared.segments), 50_724_921);」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:671`）。
- 「const FULL_STATES: u64 = 33;」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-checker-tier/tests/crash_enumeration_writable_mount_of_an_io_min_8192_pool.rs:50`）。
- 「assert_eq!(tally.states, 1 + (3 * 3 - 1) + 262_143);」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-checker-tier/tests/crash_enumeration_acquisition_barrier.rs:236`）。

### U1.2 登记崩溃流的状态数：今天与甲'（量过 4 条，其余推的）

数法：探针录写流，`writes_and_segments` 切段，`closed_form_state_count` 与 `crash_state_count_with_torn_in_place_overwrites`（展开取 `EveryProperSubset`）各算一次，不枚举、不跑 checker 档测试。

| 流 | 今天（闭式 / 带撕裂态） | 甲'（闭式 / 带撕裂态） | 出处 |
|---|---|---|---|
| 新池新建文件流（登记 `crash-case:new-pool-file-creation-stream`，由 harness 库 `run_new_pool_file_creation` 在两块 4 GiB 内存稀疏盘上走） | 16777240 / 16777260，11 段 | 16777243 / 16777268，12 段 | `research/prompts/m3-close-sysconf-r3-opus-model/out/counts-and-snapshot.txt` 第 1、10 行 |
| 可写挂载流，起点 k0、k2、r1（384 槽小盘） | 20496 / 20511 | 20499 / 20519 | 第 2–7、11–16 行 |
| 可写挂载之后会话再覆盖写一次（k0_pub） | 1069078 / 1069098 | 1069081 / 1069106 | 第 8、17 行 |

原样：「probe=counts stream=new_pool_file_creation devices=2 acquisition=one_round rollback=per_written writes=41 segments=11 closed_form_states=16777240 states_with_torn=16777260 segment_sizes=[2,2,1,2,2,1,2,24,2,1,2]」（`research/prompts/m3-close-sysconf-r3-opus-model/out/counts-and-snapshot.txt:1`），「probe=counts stream=new_pool_file_creation devices=2 acquisition=two_rounds rollback=per_device writes=43 segments=12 closed_form_states=16777243 states_with_torn=16777268 segment_sizes=[2,2,2,1,2,2,1,2,24,2,1,2]」（`research/prompts/m3-close-sysconf-r3-opus-model/out/counts-and-snapshot.txt:10`）。

今天那一列与 checker 档用例钉的 `FULL_STATES`、`FULL_STATES_WITH_TWO_STATES_PER_WRITE` 逐位相同，所以这条自己录的流与登记用例录的是同一条（那份用例我只读了常量与段形断言，流由 harness 库函数造，没从用例拷代码）。

其余 30 余条登记流没录（每条要在原型里重造历史，挂钟不够），推的：每次可写挂载取号多一段「每块盘一次原地覆写」，两块盘闭式 +3、带撕裂态 +8；流里有几次取号就加几倍。登记了按段展开（甲二 ∪ 展开 6）的流，新段只有 2 写，落在全展开那一档里，同样 +8。

### U1.3 回卷每块盘一次（量过，副本，两块盘）

起点 k0、r1，整池第 n 次系统配置写注入写报错（n = 1–4，只那一次 / 从那一次起每次），录失败那次挂载的写流，枚举它全部崩溃状态，每个状态不加故障可写挂载一次，另数「有一块盘两槽都解不出」的状态。十六种情形全部挂得上，例：「probe=rollback prefix=k0_fail4_once acquisition=two_rounds rollback=per_device fault=next_mount_no_fault outcomes={"mounted": 19}」（`research/prompts/m3-close-sysconf-r3-opus-model/out/alpha-prime-and-rollback.txt:84`）。「两槽都解不出」那一项一行都没打出来（`grep -c device_with_both_slots_unreadable research/prompts/m3-close-sysconf-r3-opus-model/out/alpha-prime-and-rollback.txt` 输出 0），即这 16 种情形的全部崩溃状态里 0 个。

回卷写之间要不要加屏障：回卷段里每块盘至多一写（例：「segment_shape=2s2,1s1,2s2」，k0_fail4_once 的写流），同一块盘两槽不会在同一段里被覆写；第二轮攻方打中逐份回卷的那一形在这里不出现。不用再加（量过的就这 16 种情形）。

三块盘：今天的实现量不了。mkfs 断言盘数：「"第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）"」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/make_filesystem.rs:479`），探针在三块盘上 panic 在同一处（草稿目录 `meas/quick-runs.log`）。推的：甲' 的取号段宽随盘数 D 涨，闭式每次 +(2^D − 1)、带撕裂态 +(3^D − 1)，D = 3 时 +7 / +26；每块盘回卷一次时回卷段里每块盘仍至多一写，两槽全废照样要两个故障。这一格对今天的代码不成立，也不构成打中。

## U2　α'（甲' 之上，另带扫单元写序）

实现（`probe-crate/src/probe_alpha_prime.rs` 文件头写了取法）：查断言那一版「根槽里是它」或「journal 环里有它的末条记录」，两样都不成一个字节不写；实例代号取各盘自证槽、根环、journal 环（两块盘整环逐条解）的最大值，扫的一臂再并进单元区每个槽解出的写序实例代号；tail 取那一版末条记录的计数器；只重写本盘不自证的槽。盘面起点 k2（最新版 T = txg 5，根在盘 0），每盘最新那一槽读错、写之后读得出（R2 判不出的那一形）；三种盘面是 T 的根槽读错 / T 的根槽与 T 的记录（两块盘各一份）都读错 / 都不读错。之后带着故障可写挂载一次，再撤掉全部故障挂一次。今天的一轮与甲' 各 18 格，结局逐格相同。

| 盘面 | 断言 T | 断言 T−1 | 断言 T+1 |
|---|---|---|---|
| T 的根读得出 | 写，挂在 txg 5 | 写，挂在 txg 5 | 不写，照旧拒 |
| 根读不出、记录读得出 | 写（凭记录），挂在 txg 5 | 写，挂在 txg 5（记录重建出 T） | 不写 |
| 两样都读不出 | 不写，照旧拒 | **写，挂在 txg 4，丢 txg 5** | 不写 |

撞号：18 格里挂上的都取 2，盘上最大实例是 1，0 格撞。checker：每格末尾池级 checker 红 0（`checker_red_end=[]`）。

### 打中：断言旧一版、新一版的根与记录都读不出时，α' 把可恢复的盘面写成丢了已确认的那一版（量过，副本）

原样（甲'）：「face=neither_readable assertion=T_minus_1 scan=true before=refused NewerStateStillUnreadableAfterOneReread alpha_prime=wrote_2_slots instance=1 tail=4 by=root mount_with_faults=mounted instance=2 effective_txg=4 lost_confirmed_txg_5=true」（`research/prompts/m3-close-sysconf-r3-opus-model/out/alpha-prime-and-rollback.txt:37`）；不扫那一格同样丢（`research/prompts/m3-close-sysconf-r3-opus-model/out/alpha-prime-and-rollback.txt:36`），今天的一轮两格同（第 16、17 行）。对照：同一盘面不让 α' 写（断言 T+1 那一格），撤掉故障之后挂在 txg 5：「face=neither_readable assertion=T_plus_1 scan=true」那一行末尾「mount_after_faults_cleared=mounted instance=2 effective_txg=5」（`research/prompts/m3-close-sysconf-r3-opus-model/out/alpha-prime-and-rollback.txt:39`）。

历史：k2 两次覆盖写都返回成功（txg 4、5 已确认）；之后每盘最新系统配置槽读错，T 的根槽与 T 的两份记录读错（三处读故障，都是读不出、不是改写）；调用方先断言真实的 T，α' 拒（两样都读不出）；改断言 T−1，α' 查到 T−1 的根，写；可写挂载取 T−1、接着暖机，txg 5 的根槽之后被新实例的发布按 txg 覆写，读故障撤掉也回不来。

四句：
- 分不分辨臂：分。只中「α' 写了」；同一盘面上 α' 不写（只读）时撤掉故障就回到 txg 5。扫与不扫两臂一起中（扫只喂实例代号）。
- 系统当时看不看得到判别子：看得到一部分。T 的根与记录读不出，但 T 写下的单元还在单元区里、读得出，单元头带 `birth_txg`；α' 加的那一趟扫描每个单元都解了头，只取了写序实例代号。
- 满足的是哪一句：U2「每格报 α' 写不写、之后挂不挂得上、撞不撞号、丢不丢确认过的写」的「丢不丢确认过的写」；确认的判据照 U5 那一句「一次发布返回成功就算确认，那一版的 txg」。
- 跑前条款给的改法：第二轮判决给 α' 的改法是「查断言那一版的根或记录读得出、且是它」，这一格上它查过了、照写，不起作用。我提的改法（只在我的模型上量过零次、被攻过零轮，推的）：扫单元区时一并取 `birth_txg` 的最大值，大于断言那一版就一个字节不写。代价（推的）：发布途中崩、只落了单元的孤单元也会让真实断言被拒，留在只读。

### 持续读错的槽、扫描的挂钟与内存、扫描漏掉读不出写序的单元

- 持续读错（写之后照样读错）：甲' 之上 α' 照写两槽，之后照旧拒：「behavior=persistent before_alpha=refused NewerStateStillUnreadableAfterOneReread」（`research/prompts/m3-close-sysconf-r3-opus-model/out/alpha-prime-and-rollback.txt:43`），同一行「after_alpha=refused」。写不进去的槽：「alpha=write_failed_after_0_slots」（`research/prompts/m3-close-sysconf-r3-opus-model/out/alpha-prime-and-rollback.txt:44`）。结局与只读相同，第二轮判决这一句照旧。
- 挂钟与内存（量过，小盘）：「geometry=384_slot_small device_bytes=45858816 highest_instance=1 parsed_units=24 slots_read=2782 seconds=0.002」（`research/prompts/m3-close-sysconf-r3-opus-model/out/alpha-prime-and-rollback.txt:40`）。扫描只留一个最大值，内存与盘大小无关（读码）。默认几何没量：单元区从「默认环下是 784 MiB（槽 50176）」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/journal.rs:188`）起到盘尾，每槽 16 KiB，1 TiB 一块盘约 6.7 × 10⁷ 个槽、整盘读一遍；按 1 GB/s 顺序读约 1100 秒一块盘（推的，读速是拍的，没量）。
- 扫描漏掉读不出写序的单元（量过）：今天的一轮 k0 起点可写挂载流的 20511 个崩溃状态，落了本实例非系统配置写的 20502 个，每盘最新槽读错，α' 断言盘上最新的根：「"no_scan/COLLIDE": 16383」「"scan/mounted_above": 20492」「"scan_after_flipping_every_unit_of_the_instance/COLLIDE": 16383」「"scan_after_flipping_every_unit_of_the_instance/COLLIDE/checker_red=[]": 16383」（`research/prompts/m3-close-sysconf-r3-opus-model/out/s3-one_round-and-scan.txt:27`）。扫描把撞号从 16383 压到 0；本实例写下的单元一个都解不开时扫描漏掉、照样撞 16383 个，挂上之后池级 checker 红 0：撞上的残留谁都读不出。甲' 之上取号流撞号 0（第二轮量过、这一轮 U5 那几趟又是 0），扫描只在 α' 不落在甲' 之上时有用。

## U3　乙'

今天缺单元那一半只比 tail：「if newest_system_configuration_journal_tail >= version_journal_position {」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/mount.rs:5802`）。副本两种读法（`tools/patch_behind_arms.py`）：`record_units` 是第一轮攻方那一臂（缺单元那一半只看本盘环槽里有没有那一版末条记录，tail 不看），第二轮判决「整份 42 条两臂全过」量的就是它；`record_or_tail_units` 是正文那句的放宽读法（tail 不落后、或本盘有末条记录，都不算落后）。

- 整份落后支用例，不带 `--skip`，甲' 之上：「arm=two_rounds+record_units rc=0 test result: ok. 42 passed; 0 failed」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-crosschecks.txt:26`），「arm=two_rounds+record_or_tail_units rc=0 test result: ok. 42 passed; 0 failed」（`research/prompts/m3-close-sysconf-r3-opus-model/out/harness-crosschecks.txt:27`），今天的落后判法 + 甲' 同样 42 条过（第 28 行）。
- 两种读法只在「tail 不落后而本盘那一版末条记录读不出」时不同：记录写每块盘一份（`CommitStep::WriteJournalRecordToEveryDevice`），要那一格落进去得记录那一份坏、同时缺一份单元，两个故障；`record_units` 那一读法在这一格上新拒（今天放行），推的、没造。

### 发布单元写到一半时取下的快照盘（量过，副本）

造法（`probe_r3.rs` 的 snapshot）：起点 k0 或 k1，最新那次发布 P（一次会话覆盖写，每块盘 10 次单元写，之后记录两份、根一份、系统配置两份）；盘 1 的快照取在它的单元写落了 9 次（或 10 次）、记录与根都没落的时刻；池在两块盘上做完 P；再把盘 1 换成快照，可写挂载。三条臂（今天、`record_units`、`record_or_tail_units`）逐格相同：

| P 的根在哪块盘 | 快照落了几次单元写 | 结局（三臂相同） |
|---|---|---|
| 盘 0（k1 起点，P = txg 5） | 9 / 10 | 拒，`WritableMountRefusedByDevicesWithoutTheSelectedVersion` |
| 盘 0 | 10 / 10 | 挂上，txg 5 |
| 盘 1（k0 起点，P = txg 4） | 9 / 10 | 拒，`NewerStateStillUnreadableAfterOneReread` |
| 盘 1 | 10 / 10 | 挂上，txg 4 |

原样（`record_or_tail_units` 一臂，根在盘 0、落了 9 次）：「overwrites_before=1 landed_unit_writes=9_of_10 sysconf=d0[g8i1t5,g7i1t4] d1[g6i1t3,g7i1t4] outcome=refused WritableMountRefusedByDevicesWithoutTheSelectedVersion」（`research/prompts/m3-close-sysconf-r3-opus-model/out/counts-and-snapshot.txt:29`）；P 的写序与根落在盘 0：「d0:JournalRecord d1:JournalRecord d0:RootRecordFua」（`research/prompts/m3-close-sysconf-r3-opus-model/out/snapshot-publish-writes.txt:2`）。打出的行不带落后臂的名字，三臂按跑的次序在第 19–22、23–26、27–30 行。

「乙' 加『缺的份数不到 2，只在有末条记录时不算缺』」：本盘有那一版末条记录时乙' 已经不算落后、不读单元，这一项只在乙' 本来就放行的盘上生效，两个候选在每一格上结局相同（读定义推的）；快照盘在单元写到一半时取下，末条记录必定还没写，这一格两个候选都拒（乙' 那一半量过）。

### (实例代号, 计数器) 在甲' 下还会不会认错（没打中，推的）

认错要同一个 (实例代号, 计数器) 写出两条不同的记录。实例代号重发：甲' 之上取号流两个起点 × 24 条故障臂 × 20519 个状态，式子全是 `formula_above_acquired`，撞号 0（`research/prompts/m3-close-sysconf-r3-opus-model/out/s3-two_rounds.txt`，`grep -c COLLIDE` 输出 0）；回卷之后被回卷的号仍留在另一槽（例「sysconf=d0[g8i2t3,g9i1t3] d1[g8i1t3,g7i2t3]」，k0 第 4 次系统配置写报错之后），下一次取号跳过它。同一实例内计数器从「读得出的最大计数器 + 1」起：「let next_counter = records」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/mount.rs:8074`）。剩下的路是第二轮攻方说的「同一实例两段历史，前一段写过记录又全读不出」，甲' 之下实例不重发，这条路关了。

## U4　丙'（没实现，读码推的）

- 两槽全废时读不读得到这块盘的根环与 journal 环：读得到位置。环的位置是写死的常量，系统配置里只是抄一份：「journal_ring_start_slot: JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION,」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/transaction.rs:606`）、「root_ring_base_slot: ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/transaction.rs:607`）；环长等几何从别的盘择到的系统配置来（各盘之间 fsid 与设备数要对得上）。设备表不来自这块盘：每块盘是谁由调用方交进来的 (身份, 设备) 对说了算，「对两块设备做 mkfs。`devices` 里每块盘的身份由调用方给」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/make_filesystem.rs:471`）；两槽全废的盘自己记的 `this_device` 读不到，挂载核不了它是不是它自称的那块。
- 读不到时第三件怎么判：条文没写。读错（设备报错）与解不开（撕裂、翻坏）都可能藏着比所选那一版新的根或记录，照「没有更新的」放行就是把看不见当成没有。建议写成：这块盘根环或 journal 环里有读错的槽就不补写（推的，零轮）；解不开的槽没法与「从来没写过」分开，只能接受。
- 四种盘在闸下（推的）：
  - 空盘：第一件读不到单元，不补写。
  - 别池的盘：单元头里的 fsid 不同，字节比不上，不补写。
  - 对调过的盘（两块盘池）：只有一块两槽全废时，另一块自证得过、记的本盘设备号与盘表对不上，挂载在别处拒（第二轮攻方推过）；两块都全废是两个故障，挂不上。
  - 同 fsid 克隆盘：克隆那边的根与记录读得出时第三件拒，关掉了第二轮打中的那一形。**打中（推的）**：克隆那边可写挂载取号 N+1、写了单元、在记录与根落盘之前崩，这块克隆盘的两槽系统配置又全废（克隆那边的取号写在甲' 下两槽都带 N+1，要全废得再坏两槽，或管理员当坏盘清掉那一段），换进本池。四件闸：第一件（所选那一版 V 的单元都在，COW 没覆写）过，第二件看根槽、过，第三件根环与 journal 环里没有比 V 新的根与记录、过，第四件不涉及。补写之后本池取号 = max(别的盘的系统配置, 根环) + 1 = N+1，与克隆盘单元区里实例 N+1 的残留单元同号——正是 C555 那一形，挂在克隆盘上。
  - 四句：分辨臂（只中「补写」那一臂；今天这块盘两槽全废，取号之前那一判就拒）；系统看得到判别子（克隆盘单元区里写序实例 N+1 的单元，前提是扫这块盘的单元区）；判据是 U4「同 fsid 的克隆盘……在闸下各是什么结局」；跑前条款的改法（第三件「没有更新的根与记录」）在这一格上不起作用。我提的改法（推的、零轮）：第三件再加「这块盘单元区里没有写序实例大于所选那一版实例的单元」，代价是补写前整盘扫一遍单元区（挂钟见 U2）。
- 补写之后到第一次轮换之间池级 checker：没量（丙' 没实现）。推的：补写写的是取号那一份内容，甲' 下两轮各写一槽，这块盘两槽都带新号；I-7.16 只看自证槽之间世代号相差不超过 1，两轮写下的世代号相差 1。

## U5　S3、S6

### S3「自证不过」：判据与量法

「哪一版已确认」：前缀里最后一次返回成功的发布的 txg（k0 起点是 3，新池新建文件；k2 是 5，第二次覆盖写）；被录的那次可写挂载还没返回，它里面的暖机与写行发布不算。单故障：一块盘的一槽换成 fsid 改了一个字节、重封了校验和的系统配置（读得出、整槽校验和过、fsid 不是本池），四个位置；一次断电：被录的可写挂载写流的每个崩溃状态。每格都真做一次可写挂载（第二轮攻方只在取号式子不大于本实例号时才挂，fsid 臂一次都没挂），比所选那一版的 txg 与已确认的 txg。

### 打中：今天择系统配置那一判不是「自证不过」，单故障就整池拒（量过，副本）

原样（甲'，k2）：「fault=fsid/d0s0/confirmed_txg_5 outcomes={"mounted_not_below_confirmed": 4115, "refused_Recovery": 16404}」（`research/prompts/m3-close-sysconf-r3-opus-model/out/s3-two_rounds.txt:8`），拒因：「error=Recovery(SystemConfigurationsDisagree)」（`research/prompts/m3-close-sysconf-r3-opus-model/out/s3-two_rounds.txt:22`）。今天的一轮（k2）：「fault=fsid/d0s0/confirmed_txg_5 outcomes={"mounted_not_below_confirmed": 4113, "refused_Recovery": 16398}」（`research/prompts/m3-close-sysconf-r3-opus-model/out/s3-one_round-and-scan.txt:8`）。另三个位置是同样两个数对调（坏的那一槽恰好是这块盘最新的一槽时拒）。k0 起点同（`research/prompts/m3-close-sysconf-r3-opus-model/out/s3-two_rounds.txt` 第 33–39 行）。丢确认过的写：4 个位置 × 2 个起点 × 20519 个状态，`LOST_CONFIRMED` 0。

代码：取号那一边按 fsid 筛：「if system_configuration.immutable.filesystem_identifier」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/recovery.rs:4270`）；择系统配置那一边只按校验和与世代号挑，各盘挑到的 fsid 对不上就整池拒：「return Err(RecoveryFailure::SystemConfigurationsDisagree);」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/recovery.rs:1448`），注释自己写了这一形：「与本池的槽 1 分不出。它的世代号更大时择到的就是它：fsid 不同时各盘择到的对不上、整池报 [`RecoveryFailure::SystemConfigurationsDisagree`]；」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/recovery.rs:1282`）。ordinal 0 是「被录的挂载一个写都没落」的状态，即不断电也拒：单故障就够。

四句：
- 分不分辨臂：分 S3 的候选，不分取号臂。「自证不过」照字面会把那一槽当不存在、择另一槽、挂上；今天的择系统配置是「整池拒」那一候选的行为。甲' 与今天的一轮数一样（16404 / 16398 只差新增的状态）。
- 系统当时看不看得到判别子：看得到。可写挂载手里有调用方给的本池 fsid（参数里的 `filesystem_identifier`），择系统配置那一判没拿它比。
- 满足的是哪一句：U5「单故障加一次断电的历史上量丢不丢确认过的写」——丢写那一分句没中（0）；中的是前两轮的前提「今天的实现就是自证不过」与「撞号没打中」只在取号式子上量、没挂载过。按 evidence-discipline「打中归错了判据」那一行，这一格要记成「S3 的现状描述错了」，不是「自证不过丢写」。
- 跑前条款给的改法：两轮判决只有「落自证不过」一句，没有改法。我提的（推的、零轮）：择系统配置时先按参数里的 fsid 筛，筛掉的当自证不过。只读挂载是不是同样拒没量（推的：两条挂载都先走恢复）。

### S6 不收：没有读故障的崩溃状态里 I-8.6 红几个（量过 0，造不出那一形是推的）

三趟（甲' 的 k2、k0，今天的 k2）共 61549 个崩溃状态，原始镜像与不加故障挂上之后的镜像各跑池级 checker，红 0：「raw_checker_red_states_by_invariant={}」（`research/prompts/m3-close-sysconf-r3-opus-model/out/s3-two_rounds.txt:20`）、「post_mount_checker_red_states_by_invariant={}」（`research/prompts/m3-close-sysconf-r3-opus-model/out/s3-two_rounds.txt:21`），k0 第 45、46 行，今天的一轮 `research/prompts/m3-close-sysconf-r3-opus-model/out/s3-one_round-and-scan.txt` 第 20、21 行。这几条流里新实例的第一条记录前一格坐的都是同一池前一实例读得出的记录，射程句说的那一形（前一格是读不出过的残留）没出现；第二轮攻方的推理（没有读故障时它读不出只有撕裂与没写两种）我没找到反例。不分辨候选：同一批状态在今天与甲' 下都是 0。

## 改法各修哪一格

全部只在我的模型上量过或推过，被攻过零轮。「量过」指副本上贴了原样输出的；「推的」指按代码推、没实现没跑。

| 改法 | U2 断言 T−1 丢 T | U4 克隆盘只写了单元 | U5 S3 单故障整池拒 | U4 环槽读错 |
|---|---|---|---|---|
| α' 扫单元区时取 `birth_txg` 最大值，大于断言那一版就不写 | 修（推的） | 不涉及 | 不涉及 | 不涉及 |
| 丙' 第三件加「单元区里没有写序实例大于所选那一版的单元」 | 不涉及 | 修（推的） | 不涉及 | 不涉及 |
| 丙' 第三件「环槽读错就不补写」 | 不涉及 | 不修（克隆的环里本来就没有新东西）（推的） | 不涉及 | 修（推的） |
| 择系统配置先按参数里的 fsid 筛 | 不涉及 | 不涉及 | 修（推的） | 不涉及 |
| 跑前已有的「α' 查根或记录」 | 不修（量过：查过了照写） | 不涉及 | 不涉及 | 不涉及 |
| 跑前已有的「丙' 第三件：没有更新的根与记录」 | 不涉及 | 不修（推的） | 不涉及 | 条文没写怎么判 |

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| U1.3 | 回卷每块盘一次：取号写第 1–4 次报错，只那一次 / 从那一次起；回卷流每个崩溃状态再可写挂载 | 起点 k0、r1，16 种情形的全部崩溃状态（每种 1–19 个） |
| U1 | 甲' 取号流的崩溃状态 × 每块盘再坏一槽（翻坏 / 持续读错，4 种组合）× 单块盘一槽换别池 fsid | 起点 k0、k2，各 20519 个状态 × 12 条臂；撞号 0，原始与挂后池级 checker 红 0 |
| U2 | α' 断言 T、T−1、T+1 × 三种盘面 × 扫与不扫 × 今天与甲' | 起点 k2 一个，36 格；撞号 0 |
| U2 | 扫描漏写序 | 今天的一轮 k0 的 20511 个状态 × 三臂 |
| U3 | 乙' 两种读法的整份用例；快照盘 | 42 条 × 2；快照 2 个起点 × 2 种落法 × 3 臂 |
| U3 | (实例代号, 计数器) 认错 | 推的 + 上面那 2 × 20519 × 24 条臂的撞号 0 |
| U4 | 空盘、别池的盘、对调的盘 | 推的 |
| U5 | S6 的那一形 | 61549 个没有读故障的状态，I-8.6 红 0 |

放开的用户步骤：挂载之前的历史（直接崩、再覆盖写两次、崩后挂一次再写一次）；挂上之后用户再写一次（k0_pub）只数了状态数（1069106 个，超过一段约 10⁶），没枚举。α' 那一格的用户动作（先断言 T 被拒、再断言 T−1）是我写死的次序；断言 T−1 一步就够打中，次序不改结局（推的）。

## 这条腿自己的限度

- 主机负载高（看到 load average 约 100、内存包装的总池排过队），单条前台命令不超过 240 秒：几个大包在 release 下降到 opt-level 1、不带调试信息（`release-profile-overrides.toml`）；harness 档用 debug 跑，4 个目标单个超过 232 秒没判。
- harness 档的 20 个红只归了类，没逐条改用例去证「改了期望值就绿、没有语义违例」。
- 状态数只录了 4 条流（新池新建文件流、可写挂载流三起点与挂后写一次）；其余登记流是推的。
- 三块盘几何今天的代码不支持，量不了。
- α' 是我照判决文字实现的读法：查根环一槽与整环记录，tail 取那一版末条记录；条文没写的地方都是我选的。
- 丙' 没实现，U4 全是推的。
- `rerun.sh` 没有端到端跑过一次：要从冻结副本现编探针（release，带 RocksDB）与 harness 档测试二进制，单条前台命令装不下。`out/` 的数是草稿目录里同一份源码跑出来的。
- S3 只量了可写挂载；只读挂载走不走同一判没量。

## 要主 agent 跑的命令

1. harness 档没判的 4 个目标，在 `rerun.sh` 建好的副本里（工作目录 `<草稿目录>/tree`，`CARGO_TARGET_DIR=<草稿目录>/target`），甲' 一臂，逐个：
   `SINGLEFS_PROBE_ACQUISITION=two_rounds SINGLEFS_PROBE_ROLLBACK=per_device SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 <目标>; 范围=1 个测试目标; 为什么=sysconf 第三轮攻方 debug 下没跑完的目标，判取号写两轮的牵连面' bash scripts/hosts/run-with-memory-cap.sh 12G cargo test --release -p singlefs-harness --test <目标>`
   目标：`accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf`、`checker_known_bad_images`、`inode_leaf_absorb_and_borrow_on_the_unlink_path`、`inode_leaf_merge_and_borrow_against_an_independent_reference`。线程 31，看 `test result:` 那一行；红的话再不带两个环境变量跑一次比。
2. 模型复跑：报告开头那条 `rerun.sh`，排在后台（编译超过 4 分钟），看末尾 `run=<名> same` 四行与 `harness=` 各行。

## 没做什么

- 不判辩方那几格（U6）；不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没跑 checker 档包的任何测试、没跑重型测试；探针依赖 checker 档包，只 `cargo build`、经内存包装直接执行编出来的二进制。
- 探针里的流都是自己造的（harness 库函数 `run_new_pool_file_creation` 与 harness 测试辅助模块 `common_admission_pool`），没从崩溃枚举用例拷代码；新池新建文件流的状态数只与那份用例钉的常量对了一下。全部原型枚举的状态合计 88392 个（取号流 2 × 20519 + 20511、扫描那趟 20511、估时两段各 3100、回卷流 132；只数不枚举的不算），低于约 10⁷。
- 没读禁读清单（这一轮无）。
