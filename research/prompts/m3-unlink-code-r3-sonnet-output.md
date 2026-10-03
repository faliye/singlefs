# 删除批第 12 批合入的代码：代码轮第三轮 云端正推腿报告（2026-10-03）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 N7 N8 K1 K2 K3 V1 V2 V3 V4 V5 V6 U4 U8 U13 LR1 MU1 MU2 -->

立场：正推。从判决行、用户定案与条款原文推到冻结副本里的代码行，再推到钉它的测试与变异行。格 K1（N1–N8）、K3（15 份文件各一格）。
引代码一律写冻结副本 `/tmp/claude-1000/m3-unlink-code-r3/tree/` 里的文件:行号（路径从 `crates/` 起）；引 kb 与记录写仓里那份文件自己的行号（现查）。

## 一、材料、命令与口径

- 冻结副本对快照：下一行是这一次真跑的原样输出（15 份）。
- 主 agent 中途消息（读 diff 之后、第一次编译之前收到）：线程上限由 15 改 10。之后每条 cargo 命令都是 `capped.sh 10`，外面套 `run-with-memory-cap.sh 8G`；包装拒过一次（同会话已有一条在跑，攻方腿同时在编），之后设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 照理由并行，没绕开包装。开跑前 `ps` 看到的只有别的 cargo，没有性能测量。
- 我的副本：`/tmp/claude-1000/m3-unlink-code-r3-sonnet/tree/`（拷自冻结副本，编译目录 `/tmp/claude-1000/m3-unlink-code-r3-sonnet/target`）。副本里只多了一条探针用例（追加在 `admission_raises_the_floor_before_refusing.rs` 末尾）；变异驱动每条改完都从冻结副本还原。模型目录 `research/prompts/m3-unlink-code-r3-sonnet-model/`：探针源码、变异驱动、全部日志。
- 背景材料正文第二节与 N3 行的行号有一处对不上冻结副本：N3 写的 `transaction.rs` 第 5734、5795 行，冻结副本里改动计数那两处是 「change_count: change_count_of_the_intent,」（`crates/singlefs-core/src/transaction.rs:5873`） 与 「change_count: change_count_of_the_batch,」（`crates/singlefs-core/src/transaction.rs:5935`）。N1、N2、N4–N7 点名的行号与冻结副本对得上。

```
$ cd /tmp/claude-1000/m3-unlink-code-r3/tree && sha256sum -c /home/fy5090/code/singlefs/research/prompts/m3-unlink-code-r3-snapshot/crates-sha256.txt
crates/mutations.tsv: OK
crates/singlefs-checker-tier/src/crash_injection.rs: OK
crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs: OK
crates/singlefs-core/src/mount.rs: OK
crates/singlefs-core/src/transaction.rs: OK
crates/singlefs-harness/src/history.rs: OK
crates/singlefs-harness/src/model_comparison.rs: OK
crates/singlefs-harness/src/model.rs: OK
crates/singlefs-harness/src/mount_transparency.rs: OK
crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs: OK
crates/singlefs-harness/tests/a_writable_mount_with_system_configuration_slot_zero_unreadable.rs: OK
crates/singlefs-harness/tests/fault_injection_fast_tier.rs: OK
crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs: OK
crates/singlefs-harness/tests/random_histories.rs: OK
crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs: OK
```

我跑过的 harness 档命令（全部在我的副本上；日志在模型目录）：

| 目标 | 结果 | 日志 |
|---|---|---|
| `-p singlefs-harness --test admission_raises_the_floor_before_refusing` | 「test result: ok. 21 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out」（`research/prompts/m3-unlink-code-r3-sonnet-model/adm-base.log:28`） | `adm-base.log` |
| `--test unlink_mount_continuation_and_the_emptied_version` | 「test result: ok. 13 passed; 0 failed; 0 ignored」（`research/prompts/m3-unlink-code-r3-sonnet-model/base-unlink_mount_continuation_and_the_emptied_version.log:20`） | `base-unlink_…log` |
| `--test mount_transparency_judged_field_by_field_after_a_writable_mount` | 「test result: ok. 33 passed; 0 failed; 0 ignored」（`research/prompts/m3-unlink-code-r3-sonnet-model/base-mount_transparency_judged_field_by_field_after_a_writable_mount.log:40`） | `base-mount_…log` |
| `--test a_writable_mount_with_system_configuration_slot_zero_unreadable` | 「test result: ok. 4 passed; 0 failed; 0 ignored」（`research/prompts/m3-unlink-code-r3-sonnet-model/base-a_writable_mount_with_system_configuration_slot_zero_unreadable.log:11`） | `base-a_writable_…log` |
| `--test fault_injection_fast_tier` | 「test result: ok. 14 passed; 0 failed; 6 ignored」（`research/prompts/m3-unlink-code-r3-sonnet-model/base-fault_injection_fast_tier.log:116`） | `base-fault_…log` |
| `--test random_histories` | 「test result: ok. 23 passed; 0 failed; 18 ignored」（`research/prompts/m3-unlink-code-r3-sonnet-model/base-random_histories.log:48`） | `base-random_histories.log` |
| `--lib -- model:: model_comparison:: history::` | 「test result: ok. 40 passed; 0 failed; 1 ignored; 0 measured; 38 filtered out」（`research/prompts/m3-unlink-code-r3-sonnet-model/base-lib.log:48`） | `base-lib.log` |

变异：把 `crates/mutations.tsv` 里这一批新加与改过的行里 harness 档能跑的 38 行，加第 185 行（它点名的用例被重钉过），逐条在我的副本上应用、跑点名用例、还原（`mut-driver.py`，不是 `mutate.sh`，也不是整表）。日志 `mut-run.log`。数出来的：

```
$ grep -c '^ROW' mut-run.log
39
$ grep '^ROW' mut-run.log | grep -vc 'named_test_red=True'
0
```

（`named_test_red=True` = 输出里有「点名用例 … FAILED」一行。39 行全红在点名用例上，没有一行编译错。）

没跑：checker 档一条不跑——`crates/singlefs-checker-tier` 的库测与 `tests/crash_injection_writable_mount_after_the_crash.rs`、变异表里点它们的 7 行（2013、2194–2199）；慢档（`#[ignore]`）用例；第 1873、1876 两行（正文说别的会话改的）与第 2210 行（根环线）。这些在第六节列成「要主 agent 跑的命令」。

## 二、K1：条款对实现（N1–N8）

### N1　丙1：卸载那一串回收的槽不扣到生效

**推导链**
1. 用户定案：2026-10-03「丙 1 进下一批」（记录 「乙新、丙1 站住、零轮」（`records/2026-10-01-删除与并发事务.md:390`），原话在那一行末尾）。丙1 的定义（攻方 `m3-unmount-after-short-continuation-r1` 报告）：「改法（只在我的模型上量过、被攻过零轮）：`raise_the_floor_through` 卸载入口回收时不扣住」（`research/prompts/m3-unmount-after-short-continuation-r1-opus-output.md:198`）。同一份报告写明它自己没判的两件事：「但两件事没判：只剩一块盘、那块盘上最新的根早于新 F 时择根与重放怎么走」（`research/prompts/m3-unmount-after-short-continuation-r1-opus-output.md:202`）。岔路单的翻面观测：「崩在写系统配置之后、第一条根之前的状态上 checker 红，或回收的槽被已确认的根引用」（`research/prompts/m3-unmount-after-short-continuation-r1-forks.md:7`）。
2. 条款：D16 已定项 1「抬 F 那一串」行 「预演过了、第一次发布之前，先把新 F 写进每块盘的系统配置、过一道屏障」（`.claude/kb/decisions/16-发布语义.md:41`）；「生效」行 「恢复后生效值 F_生效 = max(各幸存盘最新持久有效根所带 F 的最大值」（`.claude/kb/decisions/16-发布语义.md:39`）；「回退候选集」行 「按现行那一版的实例表判仍然有效 ∧ txg ≥ F_生效 ∧ txg 落在一圈窗口里」（`.claude/kb/decisions/16-发布语义.md:40`）；「正常卸载」行 「推一串空发布把 F 抬到现行那一版的 txg」（`.claude/kb/decisions/16-发布语义.md:42`）。三行没有一句说卸载那一格的回收槽扣不扣；扣住这件事 kb 里只在「抬 F 那一串」行末提了一句 「那一格里扣住的槽怎么放没有条款，见 C546」（`.claude/kb/decisions/16-发布语义.md:41`）。丙1 的条款补句还没写回（`grep -n '丙 1\|丙1' .claude/kb/decisions/16-发布语义.md` 零命中，上面「扣住」的命中不是丙1）。
3. 代码：「RaiseFloorEntry::Admission => ReclaimedReuse::HeldUntilFloorTakesEffect,」（`crates/singlefs-core/src/mount.rs:2258`） 与 「RaiseFloorEntry::Unmount => ReclaimedReuse::Immediately,」（`crates/singlefs-core/src/mount.rs:2259`）；唯一调用点 「entry.reuse_of_the_reclaimed_slots(),」（`crates/singlefs-core/src/mount.rs:2765`）；`raise_the_floor_through` 只有两个入口调（`grep -n 'raise_the_floor_through(' mount.rs mounted_session.rs` 得 `mount.rs:2303`（Admission）与 `mount.rs:2353`（Unmount）两处）。先写系统配置在回收之后、预演之后、第一条根之前：「预演过了、第一次发布之前：先把新 F 写进每块盘的系统配置、过一道屏障」（`crates/singlefs-core/src/mount.rs:2795`）；屏障在 `write_the_raised_floor_into_every_system_configuration` 里是第一个系统配置写之前一道（「第一个系统配置写之前一道池屏障（代码审阅第 19 条）」（`crates/singlefs-core/src/transaction.rs:606`））、全部系统配置写完之后一道（`transaction.rs:634` 的 `perform_commit_step(CommitStep::Barrier)`，在 `for index in 0..pool.devices.len()` 那个循环之后）——次序与丙1 定义的「先写系统配置那一步之后就可以用」相合。
4. 钉它的：用例 `…_stays_short_…_its_unmount_lets_the_next_mount_continue`（「fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue(」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:466`））、`…walks_to_empty_without_raising_the_floor_is_unmounted_on_the_slots_its_unmount_sequence_reclaims`（「fn a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmounted_on_the_slots_its_unmount_sequence_reclaims(」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:684`））、两条掉电用例（「fn an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version(」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:1258`），「fn after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_surviving_device_recovers_its_own_newest_root_below_the_floor(」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:1404`））；变异行 「三·2 删除批第 12 批丙 1（用户 2026-10-03 定「丙 1 进下一批」）」（`crates/mutations.tsv:2177`） 起 2177–2180 四行（同一个替换：卸载臂改回 `HeldUntilFloorTakesEffect`，各点名一条用例），准入臂另有第 46 行（「步 5：抬 F 回收的槽不扣住、生效之前就能发出去」（`crates/mutations.tsv:46`））。
5. 我复跑（副本上，原样）：2177–2180 与 46 都红在点名用例上，红因是卸载那一串的预演取不到落点：「ROW 2180 rc=101 named_test_red=True other_failed=0 compile_err=False」（`research/prompts/m3-unlink-code-r3-sonnet-model/mut-run.log:11`）；第 2178 行那一条的 panic 信息 「refused_publish_in_the_sequence: 2, publishes_in_the_sequence: 3, cause: PlacementRefused { unit: AccountingTree, refusal: NoFreeSlotOnAnyDevice }」（`research/prompts/m3-unlink-code-r3-sonnet-model/mut-run.log:7`）（其余三条同一形：`RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`）。这四行钉的是「丙1 在不在」（拿掉它卸载在预演里被拒），没有一行钉「崩溃安全」。

**判定与并排**
- 改动面：一致。丙1 只改了卸载入口那一臂，准入臂仍扣（第 46 行红）。
- 文档推理（`mount.rs` 第 2251–2255 行）对不上恢复的择根代码，冲突，两边整行：

| 实现文档 | 恢复择根（冻结副本 `recovery.rs`） |
|---|---|
| 「引用它们的那几条根都已低于 F_生效」（`crates/singlefs-core/src/mount.rs:2255`） | 「三个区域全部槽逐个验自证校验和，取 `(checkpoint_txg, 实例代号)` 最大的」（`crates/singlefs-core/src/recovery.rs:1260`） 与 「choose_root_slot(&choice_keys_in_ring_order).map(|chosen_slot| {」（`crates/singlefs-core/src/recovery.rs:1298`） |

  `choose_root(reader, system_configuration)` 的入参里没有 F，择根只看 (txg, 实例代号) 最大的自证根。「根低于 F_生效」只在「回退候选集」那一行（上面第 2 步）里挡根；崩溃恢复择根不看 F_生效。所以文档那句推理只对「被回退择中」成立，对「单盘存活时恢复择到比 F 低的最新根」不成立。这一点不是我推的——X 的用例自己量到了：「剩下的盘 1 系统配置里是新 F 21，恢复择到它上面最新的根 txg 19、journal 没往后施加」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:1492`）。
- 用例的对照不是正文要的对照：「第一次空发布写进回收出来的槽的那几个单元没让池级 checker 多判出一条」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:1514`） 比的是同一份（丙1 之后的）代码的两个崩点——「系统配置 + 第一次空发布的单元」对「只写了系统配置」。它能说「第一次空发布的单元写没让 checker 多红一条」，说不了「丙1 之前那一版在同一刀上红不红」：改之前那一版在这块盘上根本摆不出这一刀（「判别力：卸载回收的槽照旧扣到生效时拷贝上那一次卸载就在预演里被拒（这一刀摆不出来），第一步红。」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:1402`））。要对照得换一块 `HeldUntilFloorTakesEffect` 也能卸成的盘。
- 我对那块盘做了一次单点计算（不是扫；探针 `probe_n1_intersection.rs.txt`，同一造法、同一刀、丢盘 0）：盘 1 上，相对挂载之后镜像，「系统配置 + 第一次空发布单元」那一刀改写的 16 KiB 槽是 「PROBE disk1 changed 16KiB-slots after sysconf+units cut: {0, 1408, 1409, 1410, 1411, 1412, 1413}」（`research/prompts/m3-unlink-code-r3-sonnet-model/probe1.log:14`）（槽 0 是系统配置，1408–1413 是单元）；只写系统配置那一刀只改槽 0。逐条根看这些槽有没有被它引用（该根分配记录里未释放的记录）：txg 19 那条（丢盘 0 后盘 1 上最新的根）「PROBE root txg=19 inst=InstanceGeneration(2) F=0 unmarked_records_on_d1=27 hit_by_unit_writes=[]」（`research/prompts/m3-unlink-code-r3-sonnet-model/probe1.log:35`）——交集为空；被写到的是更老的根 txg 3–8、14（例 「PROBE root txg=14 inst=InstanceGeneration(2) F=0 unmarked_records_on_d1=37 hit_by_unit_writes=[1411]」（`research/prompts/m3-unlink-code-r3-sonnet-model/probe1.log:30`）），它们在盘 1 的根环里排在 19 之后，择根不会选它们，除非 19 自己读不出。
- 控制（比 X 用例的对照更干净）：把「卸载还没开始」的那份镜像（`image_after_the_mount`，系统配置里 F 还是 0）同样丢掉盘 0、只留盘 1，直接跑池级 checker（探针 `probe2_n1_control_before_the_unmount.rs.txt`）：「PROBE2 survivor of the image BEFORE the unmount starts: 7 violations」（`research/prompts/m3-unlink-code-r3-sonnet-model/probe2.log:22`），列的是 I-1.1、I-2.1、I-3.1、I-4.8、I-7.2、I-7.4、I-7.6（例 「PROBE2   before: I-7.4: 最新根（txg 19）引用的单元已被复用或抹头（校验和对不上或头用不了）」（`research/prompts/m3-unlink-code-r3-sonnet-model/probe2.log:28`），I-2.1 点名的就是被丢掉的盘 0：「PROBE2   before: I-2.1: 实例表单元 在盘 0 槽 1600 的那一份与位置条目里的校验和对不上」（`research/prompts/m3-unlink-code-r3-sonnet-model/probe2.log:24`））。系统配置写进新 F 之后剩 6 条（I-3.1 随 F 抬高消失）：「PROBE2 survivor after sysconf-only cut: 6 violations; equal to before-unmount: false」（`research/prompts/m3-unlink-code-r3-sonnet-model/probe2.log:30`）；再加第一次空发布的单元写，仍是同样 6 条：「PROBE2 survivor after sysconf+units cut: 6 violations; equal to sysconf-only: true」（`research/prompts/m3-unlink-code-r3-sonnet-model/probe2.log:31`）。也就是说 X 报的那六类红（I-1.1、I-2.1、I-4.8、I-7.2、I-7.4、I-7.6）在卸载一个字节都没写的时候就已经红了，丙1 前后的卸载序列都从这份镜像起步，所以这六条属于「单盘镜像本身」（checker 在少了一块盘的镜像上判），不是丙1 写出来的。这是这一块盘上的一个点。
- 对 N1 专门判据（正文第五节）的含义：这一块盘上「txg 19 引用的槽」与「卸载写的槽」无交集，且那六类红在卸载开始之前就在——按判据的第二句，这一个点属于「那六条红是单盘镜像本身的事」。但判据要的是 K2 把单元区槽数、覆盖写次数、文件单元数、崩在哪一批、卸载里崩在哪个写之后、丢哪块盘放开扫之后的结果，一个点不够。我不下「站住」。
- 另：两处文档此刻已不真。「准入与卸载共用这一串，两个入口只差这两样」（`crates/singlefs-core/src/mount.rs:2234`） 与 「两个入口只差 [RaiseFloorEntry] 那两样」（`crates/singlefs-core/src/mount.rs:2610`） 说入口只差「判不判上限、带不带卸载记号」，丙1 之后差三样（多了回收槽扣不扣）。

**判定**：部分一致。改动面、用例、变异行与条款不冲突；文档推理对不上择根代码；崩溃安全这一格正推单独坐实不了（只有一个点的计算），交 K2。条款没写丙1 这一句（补句待写回）。
**什么现象会推翻**：K2 或主 agent 在某块盘面上算出「丢盘后存活盘上最新根引用的槽 ∩ 卸载那一串单元写落的槽 ≠ ∅」，而 `HeldUntilFloorTakesEffect` 在同一刀上不红——那就是丙1 在单盘恢复上打穿；反过来，若某类盘面上两个都红同样几条、且卸载没开始的存活盘镜像也红同样几条，那几条归单盘镜像（checker 在少一块盘的镜像上判）而不归丙1。

### N2　V1、V2、V3：续做里推的抬 F 报错与写账

**推导链**
1. 判决行（第二轮）：「续做里推的抬 F 自己报了不是空间不够的错时，照 `MountError::FloorRaiseFailedAfterTheMountsPublishes` 的样子包一层」（`research/prompts/m3-unlink-code-r2-main-verification.md:37`）（V1）；「续做那一格交 `Publish` 时，准入推的与续做推的每一串先写系统配置那一步的写进错里」（`research/prompts/m3-unlink-code-r2-main-verification.md:38`）（V2）；「U9 补两条用例：准入推过抬 F 的起点与续做里推过抬 F 的起点」（`research/prompts/m3-unlink-code-r2-main-verification.md:39`）（V3）。V2 的验收：「「落下的写 − 取号两次 = 写账合计」在准入推过、续做推过、两边都推过三种起点上各一条用例」（`research/prompts/m3-unlink-code-r2-main-verification.md:38`）。
2. 契约文档（两处口径写在结构里）：「这次挂载交给设备的写（取号那两次系统配置槽写除外）= `writes_of_persisted_publishes` 各份 + `floor_raises` 各串先写系统配置那一步」（`crates/singlefs-core/src/mount.rs:509`），后接 「+ 写行之后那一处另加 `floor_raises` 各串的各次空发布（续做那一处它们已在 `writes_of_persisted_publishes` 里，不另加）+ `cause` 里那一串自己的写账。」（`crates/singlefs-core/src/mount.rs:510`）；`Publish` 的契约：「于是这一串交给设备的写 = 这一格 + 已落盘各份 + 失败账。」（`crates/singlefs-core/src/mount.rs:744`）
3. 代码，续做那一处（`continue_the_deletion_queue_until_it_is_empty`，第 4575 行起）：已落盘各份 = 续做之前的 + 续做里按 txg 的，其中「续做之前的」已含准入推的各次空发布（「续做之前这次挂载已落盘的发布（写行那次、暖机每次、写行之后推的抬 F 每一次空发布）各自的写，按先后」（`crates/singlefs-core/src/mount.rs:4457`））；`Publish` 臂：「writes_before_the_first_publish: system_configuration_writes_of_the_floor_raises(」（`crates/singlefs-core/src/mount.rs:4639`）（准入推的接续做推的，各串先写系统配置那一步按种类相加）；抬 F 自己报错臂：「continuation.writes_of_the_persisted_publishes_in_txg_order(),」（`crates/singlefs-core/src/mount.rs:4679`） 与 「.chain(continuation.floor_raises)」（`crates/singlefs-core/src/mount.rs:4685`）。写行之后那一处不同：已落盘各份只有写行与暖机（「`FloorRaiseFailedAfterTheMountsPublishes`，连同 `writes_of_persisted_publishes`（这次挂载写行与暖机那几次已落盘的写）」（`crates/singlefs-core/src/mount.rs:4769`）），推成的几串空发布在 `floor_raises` 里。两处与上面契约文档逐项对得上，口径不同是设计、写在文档里的。
4. 相加（Q3）：`WritesByStructureKind` 只记每种 {写调用数, 字节数}（「counted: BTreeMap<WrittenStructureKind, WriteCallsAndBytes>,」（`crates/singlefs-core/src/write_accounting.rs:142`）），`system_configuration_writes_of_the_floor_raises` 按种类把两样各自加起来再重放成同样多次写调用（「逐种相加的合计与逐串合计之和相同」（`crates/singlefs-core/src/mount.rs:4728`） 断言合计），不丢信息；没有哪一种统计会被「字节全记在第一次」改变。
5. 钉它的：三种起点各一条用例 「fn a_continuation_step_failing_after_the_admission_raised_the_floor_hands_back_every_landed_write()」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:1020`）（准入推过）、「fn a_continuation_step_failing_after_the_continuation_raised_the_floor_hands_back_every_landed_write(」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:1065`）（续做推过）、「fn a_continuation_step_failing_after_both_the_admission_and_the_continuation_raised_the_floor_hands_back_every_landed_write(」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:1109`）（两边都推过），共用断言 「写账合计等于落下的写减去取号那两次（推的每一串先写系统配置那一步在 writes_before_the_first_publish 里）」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:1011`）；V1 一条 「fn a_floor_raise_failing_on_a_device_error_in_the_continuation_hands_back_the_writes_of_the_mounts_publishes(」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:1178`）。变异行 2181（V1）、2182–2184（V2 三种起点）、2185–2186（V3 的 MU1、MU2），我复跑全红在点名用例上：「ROW 2184 rc=101 named_test_red=True other_failed=0 compile_err=False」（`research/prompts/m3-unlink-code-r3-sonnet-model/mut-run.log:23`） 等 6 行（`grep -c` 见第一节）。

**判定**：一致。契约文档、两处代码、三种起点的用例互相对得上；V1–V3 的验收句都有会红的用例。
**钉得住与钉不住的形（给 K2 与主 agent 用）**：
- 用例比的是写调用数（「/// 几份写账的写调用数之和。」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:872`）），没比字节数；
- 注错点只摆在「某次续做发布的第一个写」（「再在另一份上让 txg 为 `failing_publish` 的那一次续做发布的第一个写报块设备错」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:932`））；V4 注释自己写了注错点落在记录写之后时份数比只读挂载少 1（「份数等于这次挂载落在盘上的」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:720`） 那一行起），写账合计是否仍对得上，这三条用例没有扫；
- 续做里推的那一串自己的「先写系统配置那一步报错」成员（`RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot`）装进 `cause` 的形，在续做那一处没有用例（`grep -rn RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot tests/` 命中的 「fn a_block_device_error_in_the_floor_raise_pushed_by_the_mount_fails_the_mount_instead_of_reporting_still_short(」（`crates/singlefs-harness/tests/a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is.rs:66`） 与 `core_review_unit_area_start_and_publish_limits.rs` 里的是写行之后那一处）：读代码，它的写账在 `cause` 里，合计公式不变，但无会红的用例钉续做这一处。
**什么现象会推翻**：任一注错点上「落下的写 − 取号两次 ≠ 写账合计」，或 `floor_raises` 里的串与 `writes_of_persisted_publishes` 里的空发布重复/漏记。

### N3　Q-F：意图那次与每一批的改动计数

**推导链**
1. 用户定案（记录原话在那一行）：「意图与每一批把改动计数写成本次发布的 checkpoint_txg、ctime 跟着改，进下一批 crates」（`records/2026-10-01-删除与并发事务.md:372`）。条款：D8 已定项 6 字段表 「改动计数（最后一次改动所在发布的 checkpoint_txg」（`.claude/kb/decisions/08-核心索引结构.md:189`）；第 16 项第 7 条 「①b 同一份崩溃镜像各自重挂载续做同一批，两份盘逐字节相同」（`.claude/kb/decisions/08-核心索引结构.md:464`），逻辑内容的定义 「不含实例号、根与单元头里的 txg、改动计数、落点」（`.claude/kb/decisions/08-核心索引结构.md:464`）。
2. 代码：意图 「let change_count_of_the_intent = txg.0;」（`crates/singlefs-core/src/transaction.rs:5865`） → 「change_count: change_count_of_the_intent,」（`crates/singlefs-core/src/transaction.rs:5873`）；批 「let change_count_of_the_batch = txg.0;」（`crates/singlefs-core/src/transaction.rs:5917`） → 「change_count: change_count_of_the_batch,」（`crates/singlefs-core/src/transaction.rs:5935`）。`txg` 是 `checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)`，即这次发布的 txg；①b 的两份盘各自续做同一批，txg 由上一版根定，相同。用例 「fn the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_own_publish() {」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:426`） 与既有的 「fn two_copies_of_the_image_between_two_batches_continue_to_byte_identical_disks() {」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:580`）（本轮基线 13 条全绿，含它）；变异 2188、2189 复跑红。
3. 偏离：用户定案写的是「ctime 跟着改」，代码没改任何时间字段：「写入时间不改：记录里没有 ctime 字段，`write_time_seconds` 由调用方给，删除步没有确定的来源，改它会让同一份崩溃镜像各自续做出不同的盘」（`crates/singlefs-core/src/transaction.rs:5836`）。这句理由的前半不成立：盘上记录有 ctime，且三个时钟都由 `write_time_seconds` 写——「for _clock in 0..3 {」（`crates/singlefs-core/src/records.rs:77`）；条款表 「atime / mtime / ctime 秒，各 8」（`.claude/kb/decisions/08-核心索引结构.md:188`）。后半（删除步没有确定的时间来源，改了会破 ①b）站得住：`publish_unlink_intent` / `publish_unlink_step` 没有时间入参。用户定案括注要求「派之前核 ctime 的来源是不是确定的」，仓里没有记录这一核的结果（`grep -n ctime records/2026-10-01-删除与并发事务.md` 只命中用户定案那一行 372）。这句理由不是实现员自己写的：实现员 X 的派发提示（`/tmp/claude-1000/m3-batch12/writer-x-prompt.md` 第 15 行，不在仓里）已经带着「`write_time_seconds` 不动：记录里没有 ctime 字段，……」，把用户定案里的「ctime 跟着改」收窄成了不动，代码注释照抄它；X 的报告第 7 项也只写「写入时间不动」。

**判定**：部分一致。`change_count` 一致且有用例与变异；「ctime 跟着改」没做，代码注释给的理由前半写错，后半成立——是不是用户知情的收窄，仓里没有记录，交主 agent。
**什么现象会推翻**：①b 的两份盘在续做同一批后不逐字节相同（`two_copies…` 红）；或主 agent 查到用户已知情「ctime 不改」，则偏离一格改判一致。

### N4　U8：墓碑预付的层级读码 2 头

**推导链**
1. 判决：第一轮 U8（「墓碑预付的 H 从墓碑树根节点码 2 头现读（D3（空间分配） 已定项 2 字面）」（`research/prompts/m3-unlink-code-r1-main-verification.md:37`））。条款 D3 已定项 2：「H 是现行那一版墓碑树的内部层数（零根与只有根时 0），从墓碑树根节点码 2 头现读」（`.claude/kb/decisions/03-空间分配.md:52`）。
2. 代码：不重写墓碑树根的那些发布（写入类发布、各批）读上一版 `TombstoneRoot` 单元字节的头：「u64::from(root_header.level).saturating_sub(1)」（`crates/singlefs-core/src/transaction.rs:4191`），上一版没有墓碑容器取 0；这次重写墓碑树根的（意图那次往开放容器追加）仍读形状：「return resolved.tombstone_tree.internal_levels();」（`crates/singlefs-core/src/transaction.rs:4184`）（理由写在文档：新根的头由同一份形状编出，按构造相等；`ContainerTreeShape::internal_levels` 是 「u64::from(self.root_level()).saturating_sub(1)」（`crates/singlefs-core/src/container_tree.rs:191`））。两处调用都传了 `previous`：`prepare_the_version_publish` 与 `publish_admitted`。
3. 钉它的：「fn the_tombstone_prepay_of_an_overwrite_reads_the_tombstone_tree_height_from_the_root_node_header()」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:477`）（把内存里那一版墓碑树根单元头的层级改高两层、形状不动，覆盖写的标签 6 多 2 × 16384 × 副本数）；变异 2190（读形状）我复跑红。

**判定**：一致（读头那一支）；部分：重写根那一支条款字面「现读」、代码读形状，文档说按构造相等——条款没写这一支，没有变异行钉它（`return resolved.tombstone_tree.internal_levels();` 换成别的值没有用例红）。X 报告说的「零根那一支交 0」是上一版没有容器那一臂，同样没有用例单独钉。
**什么现象会推翻**：意图那次（重写根）之后记账标签 6 与「从该版真发出去的根头现读」不等。

### N5　挂载透明认续做

**推导链**
1. 用户定案「续做算挂载的一部分」：记录里的落点 「连同 F 那一判认续做推的抬 F，归下一批 crates」（`records/2026-10-01-删除与并发事务.md:385`）；形式化接入那条线定的取界规则：「取不出交 `NotLocatedInTheMountWrites`、整段 F 记不判」（`records/2026-10-01-删除与并发事务.md:376`），并明写 「不许改回按内容认」（`records/2026-10-01-删除与并发事务.md:376`）。
2. 界按位置取：「.and_then(|writes| mount_writes.len().checked_sub(writes))」（`crates/singlefs-harness/src/mount_transparency.rs:474`）（写表长度减去「从第一串抬 F 起到写表末尾的写数」，文档 「界 = 写表长度减去从第一串抬 F 起到写表末尾的写数」（`crates/singlefs-harness/src/mount_transparency.rs:448`））；按内容（根身份）只在取出界之后当核对：「let the_boundary_is_a_system_configuration_slot_write = mount_writes」（`crates/singlefs-harness/src/mount_transparency.rs:478`） 与 `every_floor_raise_root_is_written_from_the_boundary` 两样不满足交 「对不上的交 [FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites]，不 panic」（`crates/singlefs-harness/src/mount_transparency.rs:461`）。没有退回按内容认。
3. F 认续做推的抬 F：「.chain(output.floor_raises_pushed_by_the_continuation.iter())」（`crates/singlefs-harness/src/mount_transparency.rs:251`） 接在准入推的之后取最后一串；用户可见树根指针按续做之后的那一版比：「.filter(|(step, _)| step.0 == version.0 && step.1 <= version.1)」（`crates/singlefs-harness/src/mount_transparency.rs:570`）（同实例里 txg 不大于它的最近一步），判字段处 「Relation::WrittenByTheMount(_) => continuation」（`crates/singlefs-harness/src/mount_transparency.rs:1201`）。
4. 钉它的（harness 档，本轮基线 33 条全绿）：「fn after_a_mount_that_continued_the_deletion_the_two_tree_root_pointers_are_those_of_its_last_continuation_step(」（`crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs:1111`）、「fn a_mount_whose_continuation_raised_the_floor_leaves_the_effective_floor_at_the_last_one_the_continuation_raised_to(」（`crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs:1154`）、取界三条 `a_floor_raise_pushed_inside_the_continuation_is_located_between_the_continuation_steps`（第 1208 行）、`floor_raises_pushed_by_the_mount_admission_are_located_before_the_continuation_that_follows_them`（第 1266 行）、「fn floor_raises_pushed_by_both_the_admission_and_the_continuation_are_located_from_the_first_admission_raise(」（`crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs:1316`）；变异 2138、2139、2191、2192、2193 我复跑全红在点名用例上（取界单测在这一批从 checker 档搬进 harness，所以 2138、2139 的文件列与点名用例列随之改了，我用的是改后的）。

**判定**：一致。界仍按位置取，没有退回按内容认；F 与树根指针都认续做。
**覆盖面的事实**：崩溃注入战役跑的是两块 4 GiB 的盘（「pub const CHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {」（`crates/singlefs-harness/src/history.rs:4784`）），（推的：「两块 4 GiB 的盘：单元区墙在这里够不着，挂载与之后那一次发布都该成」（`crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs:17`），所以空间准入不拒、不推抬 F，战役里 `floor_raises_pushed_by_the_continuation` 恒空；这一条我没量）；「续做里推了抬 F」那几形的取界与划段只由 harness 档的窄盘用例钉，战役摆不到。
**什么现象会推翻**：某个二次崩溃点上，续做中间落在某一步的版本，树根指针既不等该步的、也不等 R1 的而被判绿；或取界单测里 `NotLocated` 在合法的挂载写表上出现。

### N6　崩溃注入：删完那一版、U4、V1 划段、V6

**推导链**
1. 条款：D8 已定项 16 第 5 条 「删完之后只能做 `publish_new_inodes`（size 0）、写行与空发布」（`.claude/kb/decisions/08-核心索引结构.md:462`）；第一轮 U4：「崩溃注入战役不再在 Continuation 段 `continue` 跳过」（`research/prompts/m3-unlink-code-r1-main-verification.md:33`）；第二轮 V1 的划段半句与 V6。
2. 代码（`singlefs-checker-tier`，冻结副本里读，没跑）：删完那一版调 `publish_new_inodes`：「PoolVersion::WithTreesAndNoFile(current) => publish_new_inodes(」（`crates/singlefs-checker-tier/src/crash_injection.rs:1695`）；划段认 `FloorRaiseFailedAfterTheMountsPublishes`：「MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {」（`crates/singlefs-checker-tier/src/crash_injection.rs:1790`），按它的 `writes_of_persisted_publishes` 划（传 `&[]` 作失败账），与 `Publish` 那一臂同一个函数。
3. 与 N2 口径的一致性：续做那一处交的已落盘各份（含准入推的空发布、续做各步、续做推的空发布，按 txg）里，划段取「第一个写了只有续做才写的结构的那一次」起为续做（「写行之后第一个写了只有续做才写的结构」（`crates/singlefs-checker-tier/src/crash_injection.rs:1826`） 那段文档）；写行之后那一处交的已落盘各份只有写行与暖机，推成的空发布在 `floor_raises` 里、不进划段，于是 None、写行之后都算暖机——与 N2 里两处契约的差别正好吻合。成功的挂载则按 `1 + 暖机 + 准入推的各次空发布` 数（「1 + mounted.output.warm_up_publishes.len()」（`crates/singlefs-checker-tier/src/crash_injection.rs:1642`） 起），两种划法在「续做开头先被空间拒、先推一串抬 F」这一形上不同（错的划法把那一串归暖机，成功的划法归续做），文档自己承认：「续做开头先被空间拒、先推的那一串抬 F 归暖机：错里分不出它是准入推的还是续做推的。」（`crates/singlefs-checker-tier/src/crash_injection.rs:1829`）
4. U4：战役用例不再跳过 Continuation 段，并加一段 unlink 历史：「tally.absorb(&unlinking_the_first_file.tally);」（`crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs:59`）、`for phase in WritableMountPhase::ALL {` 里每段断言 ≥ 1 个二次崩溃点（该文件第 83 行起，已无 `Continuation => continue`）。变异行 2194–2199 点的都是 checker 档用例，我没跑。
5. V6 的单测搬进 harness（N5 第 4 步），其中 「fn floor_raises_pushed_by_both_the_admission_and_the_continuation_are_located_from_the_first_admission_raise(」（`crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs:1316`） 就是「准入与续做都推」那一条，2193 复跑红。

**判定**：一致（读代码）；checker 档用例与 2013、2194–2199 共 7 行变异**复核不了**（checker 档不跑）。V1 的划段单测只用合成写账（`writes_of_kinds`）验分类，不是在真挂载失败上验——真失败那一形在战役里摆不到（见 N5 覆盖面）。
**什么现象会推翻**：真挂载在续做里报 `FloorRaiseFailedAfterTheMountsPublishes` 时，划段给出的续做起点与设备一层的根槽写顺序对不上；或删完那一版上挂载之后那一次发布被拒。

### N7　U13：模型的分批、比重与映射树上界

**推导链**
1. 条款 D8 已定项 16 第 2 条：「从尾部按数据单元个数分批，每批释放尾部 K 个单元」（`.claude/kb/decisions/08-核心索引结构.md:459`）；「K 产品取 `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`（144，自由选取）」（`.claude/kb/decisions/08-核心索引结构.md:459`）。第一轮 U13 的验收：「至少一段历史里删除批 ≥ 2 片叶」（`research/prompts/m3-unlink-code-r1-main-verification.md:42`）。
2. 分批同一个批大小、同一个断点：实现 「if data_units_remaining <= data_units_per_batch {」（`crates/singlefs-core/src/unlink.rs:120`）（不多于一批就是最后一批）、「let data_units_kept = data_units_remaining - data_units_per_batch;」（`crates/singlefs-core/src/unlink.rs:125`）；模型 「const MODEL_DELETION_BATCH_DATA_UNITS: u64 = EXTENT_TREE_LOWER_LEAF_DATA_UNITS;」（`crates/singlefs-harness/src/model.rs:651`），「let step = if data_units_remaining > MODEL_DELETION_BATCH_DATA_UNITS {」（`crates/singlefs-harness/src/model.rs:1941`），剩下数 「- MODEL_DELETION_BATCH_DATA_UNITS,」（`crates/singlefs-harness/src/model.rs:1537`）。两边都是 144、都是「剩下 > 144 才是非最后一批」、批后剩下 = 剩下 − 144；「从尾部起」只影响哪些单元被释放，不影响个数，模型只数个数。两边的开关：「模型只罩产品批大小：随机历史的挂载不装只供测试压小批大小的开关」（`crates/singlefs-harness/src/model.rs:650`），所以产品批大小对产品批大小。
3. 比重与内容：「pub const DELETING_FILES_OF_SEVERAL_LOWER_LEAVES: GenerationWeights = GenerationWeights {」（`crates/singlefs-harness/src/history.rs:864`）；覆盖写另抽跨单元内容，单元数 2–16、145–152（「data_units: EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 1 + selector % 8,」（`crates/singlefs-harness/src/history.rs:986`））、289–292（「data_units: 2 * EXTENT_TREE_LOWER_LEAF_DATA_UNITS + 1 + selector % 4,」（`crates/singlefs-harness/src/history.rs:993`））；走顺序写入口（`UserChange::SequentialWrite`），别的几组比重不变（`overwrite_contents: WithinOneDataUnitOrJustBeyond`）。胶水按单元号接内容、下段节点与映射树根之下的节点不对应角色：「TransactionUnit::ExtentLowerNode(_) => return None,」（`crates/singlefs-harness/src/model_comparison.rs:73`）、「TransactionUnit::MappingTreeNodeBelowTheRoot(_) => return None,」（`crates/singlefs-harness/src/model_comparison.rs:101`）（这两类单元的分配代、位置、记录归池级 checker 判，模型比不到它们）。
4. 映射树节点数上界：模型 「pub fn central_mapping_tree_nodes_upper_bound(nodes_before: u64, entries: u64) -> u64 {」（`crates/singlefs-harness/src/model.rs:551`）；它的论证对着实现的切法读：实现从中间切，左半留 ⌈n÷2⌉、右半 ⌊n÷2⌋（「let right_keys = keys.split_off(keys.len().div_ceil(2));」（`crates/singlefs-core/src/code_two_tree.rs:450`）），装不下时 n = 容量 + 1，右半 = ⌈容量÷2⌉，模型取 「fn smallest_half_of_a_split(capacity: u64) -> u64 {」（`crates/singlefs-harness/src/model.rs:529`）；单测钉了容量 294 / 143 与右半 147 / 72（`central_mapping_tree_nodes_grow_by_two_per_147_entries_and_keep_the_previous_bound`，lib 基线绿）。收缩只摘空节点（「children.remove(child_index);」（`crates/singlefs-core/src/code_two_tree.rs:559`），不合并），故先删后插里节点数不增。右半在切之后「这一段只插不删」，所以每次切在最终树里留下一个至少 147 条（内部 72 个孩子）的新节点，彼此不相交：147 s ≤ E、72 t ≤ N − 1，每次切至多多 2 个节点（一个右半加根分裂时的新根）——读下来与代码一致，我没找到缺口。上界只涨不落：「previous.central_mapping_tree_nodes_upper_bound.max(1),」（`crates/singlefs-harness/src/model.rs:1651`） 取上一版的。
5. 钉它的：快档 `random_histories.rs` 里 「fn deleting_a_file_of_two_lower_leaves_publishes_a_batch_keeping_data_units_that_the_model_compares(」（`crates/singlefs-harness/tests/random_histories.rs:2026`）、「fn deleting_a_file_of_two_lower_leaves_then_rolling_back_to_its_intent_walks_two_steps_before_the_next_change(」（`crates/singlefs-harness/tests/random_histories.rs:2068`）、「fn a_file_of_three_lower_leaves_grows_the_central_mapping_tree_below_its_root_and_the_model_compares_every_version(」（`crates/singlefs-harness/tests/random_histories.rs:2122`）（292 → 148 → 4 两个非最后批），都在本轮基线 23 条里；变异 2200–2203、2211–2213 我复跑全红。取样点本身（48 段）是慢档 「fn random_histories_deleting_files_of_several_lower_leaves_compare_every_batch_and_the_walks_before_a_change(」（`crates/singlefs-harness/tests/random_histories.rs:1971`），`#[ignore]`，我没跑（debug 超 240 秒），列在第六节。

**判定**：一致（批大小、断点、次序的计数口径同一个；上界的推导对着实现的切法读得通）。「上界是否在所有合法历史上不低于真节点数」「漏掉该拒的」是 K2 的事，我只有读代码的结论，没有扫。
**什么现象会推翻**：某段合法历史里实现的中央映射树节点数 > 模型带的上界（模型的占槽上界低于真写槽数 ⇒ `ModelDisagreement`），或模型的占槽上界过大使该拒的发布被判「允许拒」。

### N8　其余：LR1、重钉取样点、清单为空、守护变异、变异表

**LR1**（遗留判决 「补坏槽 0（偶数世代）的可写挂载与取号用例；第二块盘那一格、读错形、逐盘核 `Err(device)` 出口各补一条变异行」（`research/prompts/m3-leftover-code-r1-main-verification.md:40`），验收 「「只看槽 1」那一种错实现至少红一条点名用例」（`research/prompts/m3-leftover-code-r1-main-verification.md:40`））：新文件四条用例（第 143、172、211、389 行；基线 4 条绿），变异行 2205、2206（只看槽 1，两条点名用例）、2207（第二块盘）、2208（逐盘核 `Err(device)` 出口）、2209（读错形，点名既有用例），我复跑 5 条全红在点名用例上，验收满足。2205 与 2206 是同一处替换（`for offset in [0, slot_spacing_in_bytes] {` 在 `recovery.rs` 只此一处），名字分「见证读」「取号」两种，红的用例不同。
**重钉 `fault_injection_fast_tier.rs`**：取样点由种子区间改成一段写死的历史加一个注入点（「const STEP_OF_THE_SWALLOWED_ROOT_SLOT_WRITE: usize = 3;」（`crates/singlefs-harness/tests/fault_injection_fast_tier.rs:495`）），去掉 `#[ignore]`，基线 14 条绿；点名它的变异第 185 行（「被吞的写不插回录制流重建「实现以为自己写下的」镜像」（`crates/mutations.tsv:185`））我复跑红在这条用例上。
**A3b-15 换取样点**：第 1323 行改点名 lib 单测，复跑红。
**「清单为空那一形照新发现报」**：第 2204 行（`classify_failure` 查清单之后一律接住第 0 条）复跑红；`KNOWN_RED_FORMS` 在冻结副本里确是空的（用例首行 `assert!(KNOWN_RED_FORMS.is_empty(), "清单是空的")` 钉）。
**续做结局守护**：第 2187 行复跑红，红因 「续做推满仍放不下：WalkedToEmpty」（`research/prompts/m3-unlink-code-r3-sonnet-model/mut-run.log:34`）。
**每条新变异改坏的是不是名字说的那一处**：读 48 行新加/改动行的原文/替换文，逐条对名字，没有对不上的；要注意四点——① 2177–2180 是同一个替换配四条点名用例，2182–2184 是同一个替换配三条，名字里的「起点」是用例的分法不是替换的分法；② 2177–2180 都靠「卸载预演被拒」红，钉的是丙1 在不在，不是崩溃安全（N1）；③ 第 2210 行（`unreadable_ring_slots_inside_the_window_tests`）是根环线的、不是这一批；④ 2013、2194–2199 点 checker 档，没跑。全表锚点检查：2208 条数据行里 2207 条原文在冻结副本里恰好命中一次，余下第 1362 行点 `litmus/…litmus`，在 `crates/` 之外（冻结副本里没有）。
**推翻条件**：任一行在冻结副本上应用后点名用例不红；或某条名字说 A、替换改的是 B。

## 三、K3：被判的 15 份文件，每份一格

判定词：一致 / 部分一致 / 冲突 / 规则没说 / 复核不了。「读」= 只读了冻结副本里的代码；「跑」= 在我的副本上跑过 harness 档。

| # | 文件（冻结副本） | 这一批在里面动了什么 | 判定 | 依据 |
|---|---|---|---|---|
| 1 | `crates/mutations.tsv` | 新加/改动 48 行（46、802、890、1323、1868、1873、1876、2013、2126、2138、2139、2177–2213） | 一致（跑 39 行） | 2208 条数据行里 2207 条原文恰好命中一次，第 1362 行点 `litmus/`、在 `crates/` 外；39 行（本批 38 行加第 185 行）逐条应用都红在点名用例上（`mut-run.log`）；没跑的 10 行：checker 档 7 行（2013、2194–2199）、别的会话改的 2 行（1873、1876）、第 2210 行（根环线，不属于这一批）。名字与替换对得上，见 N8 |
| 2 | `crates/singlefs-checker-tier/src/crash_injection.rs` | 删完那一版改调 `publish_new_inodes`；划段认 `FloorRaiseFailedAfterTheMountsPublishes`；取界类型搬进 harness；挂载透明认续做 | 一致（读）；复核不了（跑） | 「PoolVersion::WithTreesAndNoFile(current) => publish_new_inodes(」（`crates/singlefs-checker-tier/src/crash_injection.rs:1695`），「MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {」（`crates/singlefs-checker-tier/src/crash_injection.rs:1790`）；它的库测和 2013、2194–2199 在 checker 档，没跑（见 N6） |
| 3 | `crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs` | 去掉 Continuation 段的跳过，加一段 unlink 历史 | 一致（读）；复核不了（跑） | 「tally.absorb(&unlinking_the_first_file.tally);」（`crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs:59`）；`grep -n 'Continuation => continue' crates/singlefs-checker-tier/tests/*.rs` 零命中；变异 2195 点它，没跑 |
| 4 | `crates/singlefs-core/src/mount.rs` | 丙1 一臂；V1/V2 续做报错与写账；V5 三处文档 | 部分一致 | 丙1 与 V1/V2 一致（N1、N2）；V5 的「正常卸载照常做成」已改（「正常卸载（抬 F 到现行那一版、不判上限）照常做成，它的预演算上新 F 回收出来的槽」（`crates/singlefs-core/src/mount.rs:961`））；但 2251–2255 的推理对不上恢复择根（N1），2234、2610 两处「两个入口只差这两样」已不真。第 2210 行那一段（根环线）不在这一批、没判 |
| 5 | `crates/singlefs-core/src/transaction.rs` | Q-F 改动计数；U8 墓碑预付读码 2 头 | 部分一致 | N3：`change_count` 一致，用户定案里的「ctime 跟着改」没做，注释理由前半写错；N4：读头那一支一致，重写根那一支读形状（条款没写这一支） |
| 6 | `crates/singlefs-harness/src/history.rs` | 新比重 `DELETING_FILES_OF_SEVERAL_LOWER_LEAVES`、跨单元内容与顺序写入口、第四新的根回退、统计计数、「清单为空」 | 一致（跑） | N7、N8：比重与内容见 「pub const DELETING_FILES_OF_SEVERAL_LOWER_LEAVES: GenerationWeights = GenerationWeights {」（`crates/singlefs-harness/src/history.rs:864`）；`classify_failure` 的第 2204 行复跑红；lib 基线 40 条绿 |
| 7 | `crates/singlefs-harness/src/model_comparison.rs` | 胶水：多单元内容按单元号接；下段节点与映射树根之下的节点不对应角色 | 一致（跑） | 「TransactionUnit::ExtentLowerNode(_) => return None,」（`crates/singlefs-harness/src/model_comparison.rs:73`）；变异 890、2202、2211 复跑红；代价：这两类单元模型比不到（归池级 checker） |
| 8 | `crates/singlefs-harness/src/model.rs` | 批大小 144、走空分批、映射树节点数上界、A3b-15 取样点 | 一致（跑）；上界在所有合法历史上够不够 = K2 | 见 N7；变异 802、1323、2200、2201、2203、2212、2213 复跑红 |
| 9 | `crates/singlefs-harness/src/mount_transparency.rs` | 界按位置取、认续做；树根指针按续做那一版比 | 一致（跑） | N5；变异 1868、2138、2139、2191–2193 复跑红 |
| 10 | `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs` | 丙1 的 4 条用例；V1–V3 的 5 条用例；V4 | 一致，但丙1 的单盘对照不足 | 22 条里 21 绿 1 慢档；它钉「丙1 在不在」和写账合计，不钉崩溃安全：「第一次空发布写进回收出来的槽的那几个单元没让池级 checker 多判出一条」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:1514`） 的对照是同一份代码的另一刀（N1） |
| 11 | `crates/singlefs-harness/tests/a_writable_mount_with_system_configuration_slot_zero_unreadable.rs` | 新文件，LR1 的 4 条用例 | 一致（跑） | LR1 验收「只看槽 1 的错实现至少红一条点名用例」：2205、2206 红；基线 4 条绿 |
| 12 | `crates/singlefs-harness/tests/fault_injection_fast_tier.rs` | 说谎设备那条用例改成写死的一段历史加一个注入点 | 一致（跑） | 基线 14 绿 6 慢档；点名它的第 185 行复跑红 |
| 13 | `crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs` | 续做认；取界单测搬进来 | 一致（跑） | 基线 33 条绿；5 行变异复跑红（N5） |
| 14 | `crates/singlefs-harness/tests/random_histories.rs` | 3 条快档用例、1 条慢档取样点 | 部分一致：快档跑过，慢档复核不了 | 快档 23 条绿；慢档 「fn random_histories_deleting_files_of_several_lower_leaves_compare_every_batch_and_the_walks_before_a_change(」（`crates/singlefs-harness/tests/random_histories.rs:1971`） 是 `#[ignore]`，U13 的「删除批 ≥ 2 片叶的段数不为 0」要靠它，我没跑，列在第六节 |
| 15 | `crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` | Q-F 用例、U8 用例、V4 注释 | 一致，带 ctime 一项 | 基线 13 条绿，3 行变异（2188–2190）红；Q-F 用例断言 「写入时间不动（记录里没有 ctime，删除步没有确定的来源）」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:423`）——钉的就是 N3 里那处与「ctime 跟着改」字面的偏离 |

## 四、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-N1 丙1 | 部分一致 | 只改了卸载那一臂、4 条用例 + 4 行变异都红；`mount.rs` 2251–2255 的「根都低于 F_生效」只对回退候选集成立，恢复择根不看 F；X 盘面上存活盘最新根 txg 19 与卸载写的槽无交集（单点），六类红在卸载开始前的单盘镜像上已红；崩溃安全这格要 K2 放开扫才能定；条款补句未写回 |
| K1-N2 V1–V3 写账 | 一致 | 两处 `writes_of_persisted_publishes` 的口径与契约文档对上；三起点各一条用例、V1 一条；6 行变异红；只比调用数、注错点只在发布第一个写、续做处 sysconf 报错形无用例 |
| K1-N3 Q-F | 部分一致 | `change_count` = 本次 txg，①b 用例仍绿；用户定案的「ctime 跟着改」没做，注释写「记录里没有 ctime 字段」与 `records.rs`、D8 字段表不符，核 ctime 来源的结果无记录 |
| K1-N4 U8 | 一致（读头那支）/部分 | 不重写根的发布读上一版码 2 头；重写根那支读形状，条款没写，无变异行钉 |
| K1-N5 挂载透明 | 一致 | 界仍按位置取，按内容只当核对；F、树根指针认续做；5 行变异红；战役 4 GiB 盘摆不到续做推抬 F |
| K1-N6 崩溃注入 | 一致（读）/复核不了（跑） | `publish_new_inodes` 一臂、U4、V1 划段与 N2 口径吻合；checker 档用例与 7 行变异没跑 |
| K1-N7 U13 | 一致 | 批大小/断点同一个 144 与 `<=`；上界推导对着实现切法读得通；慢档取样点没跑；上界够不够交 K2 |
| K1-N8 其余 | 一致 | LR1 验收满足；重钉、A3b-15、清单为空、守护变异各复跑红；全表锚点 2207/2208 命中一次 |
| K3 15 份 | 见第三节 | 一致 10（#1、#6–#13、#15；其中 #8 的上界、#10 的单盘对照、#15 的 ctime 带限定）、部分一致 3（#4 mount.rs、#5 transaction.rs、#14 random_histories）、读一致/跑复核不了 2（#2、#3，checker 档）、冲突 0 |


## 五、校验与反推（这一腿自己的三步）

- 正推：上面各格。
- 反推：「若 N1 的文档推理是对的，本该看到」恢复择根里有 F 的过滤——`choose_root` 的入参与 `choose_root_slot` 的键里没有 F，没出现；「若丙1 只改卸载入口，本该看到」`reuse_of_the_reclaimed_slots` 是唯一调用点、准入臂变异红——出现了。
- 校验（换一条路）：丙1 的「改动面」除了读代码，还用 39 行变异独立验：卸载臂改回 Held 则 4 条用例红、准入臂改成 Immediately 则第 46 行红；两条路不共用代码。N1 的崩溃安全这格没有第二条独立的路（只有我那一次计算，和 X 用例自己的断言），所以不下「站住」。

## 六、要主 agent 跑的命令（我没跑）

第 1–3 条在冻结副本的拷贝上跑（不在主仓），内存上限 8G，线程 10；checker 档的带 `SINGLEFS_HEAVY_TESTS=commit`（或用户要求时 `=user-request`）。

1. checker 档库测（N6，变异 2013、2194–2199 点它们）：`cd <冻结副本拷贝> && SINGLEFS_HEAVY_TESTS=commit bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 10 cargo test --offline -p singlefs-checker-tier --lib -- crash_injection::tests::`；看 `test result:` 行与下面三条用例名 ok：`a_mount_whose_floor_raise_failed_after_a_continuation_step_marks_the_continuation_by_the_persisted_publishes`、`the_publish_after_a_writable_mount_on_the_emptied_version_creates_an_inode_and_is_published`、`the_remount_after_each_second_crash_reads_the_same_content_as_the_uninterrupted_mount_in_every_mount_phase`。
2. checker 档战役用例（U4）：同上前缀，`cargo test --offline -p singlefs-checker-tier --test crash_injection_writable_mount_after_the_crash`；看 `test result: ok` 与输出里「挂载途中的二次崩溃」每段（含续做）计数 ≥ 1。
3. U13 慢档取样点（N7，harness 档但 debug 下超 240 秒）：`cd <冻结副本拷贝> && bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 10 cargo test --release --offline -p singlefs-harness --test random_histories -- --include-ignored random_histories_deleting_files_of_several_lower_leaves_compare_every_batch_and_the_walks_before_a_change --nocapture`；看 `test result: ok` 与统计行「删除：模型比过不是最后一批的删除批（≥ 2 片下段叶）… 次、在 … 段里」两个「段数」都不为 0。
4. 变异行 2013、2194–2199：用 `.claude/gate.d/checker-tier-crates-mutation-replay.sh` 文件头写的 `--item <变异名>` 只判这 7 行（`SINGLEFS_HEAVY_TESTS=commit`；这一道在主工作区跑、判工作区那一份，工作区里此刻有别的会话的改动，要不要先等它们暂存由主 agent 定）。
5. N1 对照（K2 的事，列在这里供主 agent 核）：选一块 `HeldUntilFloorTakesEffect` 也能卸成的盘面（宽盘或没被续做吃光槽的盘），在「先写系统配置之后、第一次空发布单元落盘之后、根未落」那一刀上丢盘 0，看丙1 前后存活盘上最新根引用的槽与单元写的交集和池级 checker 红的条数。

## 七、没做什么

- 没出判决，没替主 agent 采纳或否决任何一条；攻方立场的 K2（反例搜索、放开扫、注错点扫描、模型上界对拍）一格没做。N1 那一次计算是单点，不是扫。
- 没跑 checker 档任何东西，没跑慢档，没跑变异整表（只在自己的副本上逐行应用了 39 行 harness 档变异）。
- 没读禁读清单里的文件（攻方腿这一轮的提示与产出），没读主仓 `crates/`（引的全是冻结副本）。
- 没改仓里任何 `crates/`、kb、规则；只写了本报告和模型目录。
- 实现员报告只读了 X 的（`/tmp/claude-1000/m3-batch12/x/report.md` 第六节：Q1–Q4 与丙1 两刀的描述、第 7 项 Q-F），Y、Z、改钉值的没读；报告的结论句只当线索，上面的判定都以代码为准。

## 八、交回前删掉的东西

- 删了 `/tmp/claude-1000/m3-unlink-code-r3-sonnet/target`（2.7G，我的 `CARGO_TARGET_DIR`）与 `/tmp/claude-1000/m3-unlink-code-r3-sonnet/tree`（15M，拷自冻结副本、带两条探针用例的仓副本）。
- 没删：`/tmp/claude-1000/m3-unlink-code-r3-sonnet/` 里的其余文件（日志、模板，合计 304K）——模型目录 `research/prompts/m3-unlink-code-r3-sonnet-model/` 里已有探针源码（`probe_n1_intersection.rs.txt`、`probe2_n1_control_before_the_unmount.rs.txt`，追加到冻结副本的 `admission_raises_the_floor_before_refusing.rs` 末尾即可复现）、变异驱动 `mut-driver.py` 与全部日志；副本被删后要复核，从冻结副本重拷即可。
