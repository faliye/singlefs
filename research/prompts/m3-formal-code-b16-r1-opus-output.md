# m3-formal-code-b16-r1 云端攻方（Opus）报告（2026-10-04）

攻击面：K2（16-3 判官）、K4（15 行变异的判别力），兼 K1（卸载的答，树表 0 条那一格）。代码引文读冻结副本 `/tmp/claude-1000/m3-formal-code-b16-r1/tree/`；
量的数全在攻方副本上（`rsync` 冻结副本到草稿目录、补工作区清单后编译），不是入库装置上的数。checker 档测试目标一个没跑；原型 `c670-probe`
只把 `singlefs-checker-tier` 当库编（取它的撕裂镜像与可撕裂判定），跑的是原型自己的二进制。

## 复跑

```
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复核攻方原型，峰值表里那一次不代表这一次"
bash research/prompts/m3-formal-code-b16-r1-opus-model/tools/rerun.sh /tmp/claude-1000/<核查草稿目录>
```

环境变量：`FROZEN`（冻结副本根，默认上面那一棵）、`REPO`（默认 `/home/fy5090/code/singlefs`）、`CAP`（默认 8G）、`THREADS`（默认 8）。
脚本末尾打印各段的汇总行、变异行结局与 K1 那一行，与下文和 `outputs/` 逐行比。本机 8 线程下估约 10 分钟（各段挂钟见第七节表）。

模型目录各文件 sha256（`find . -type f | sort | xargs sha256sum` 原样）：

```
f12d086f56f02fb8256c7634393a1bfaf80cf9b1b5da15f18c5df9762c86b72d  c670-probe/Cargo.toml
fde010c94e244e8b855bfcd007bf2b45e327b4536047a13c851da6a01396567e  c670-probe/src/main.rs
274f6c2f108534fd175ce137bce83f894b518951e9773d37318a65867b36c824  harness-tests/attack_unmount_without_trees.rs
897f6867b39f49f350761fa7958d3083629edf8314ed639b7054c0ec78325a67  outputs/abandon-broad.txt
a0b62d40cc582136a611dd0dc2b54b163c46e00b46e012bb4428a7813e4ea538  outputs/abandon-deleting.txt
77a36ce38dd4668f4a5b7abde6e30dffdfe2c8d0baf4a00f9a985b3ccdc34209  outputs/abandon-reuse.txt
2954f30be4c4d05c8bc4ad18753eec641b4f6b3796b254120593f8f494a5f72c  outputs/abandon-rollback.txt
02a0cc67476bd7fe652996003d1d6261ca56d543b92a004a9ab5b54721bc9d30  outputs/base-broad6-s0-8-short.txt
97348ac3ba63449bd3492e3737f1fc8b473bb024c34d1585513228a124271db0  outputs/base-broad6-s8-108.txt
95ce3f549210f206e5e2c13ed4ddb82238270ac2b2c8c381cae4d2d8924895dd  outputs/base-deleting-10-s0-32.txt
48266da2862385daedc49eb7f63ca29e296e93b145eba3e4499214e75f749d25  outputs/base-named-short-nosecond.txt
daa4a63b0c38d8a8f6967049c1ce15ca7fcf18a25cc7193813fdd0d9030a5f92  outputs/base-named-short.txt
1493a6cd1c568696ee560b31d047eea8eb68f6a2b37a606c3c2720648cbae08d  outputs/base-ops-a-nosecond.txt
2c673f43de7bb18eec2a512a1a30eed9248042cc1f6c283aa8f2c5ea6fa07d91  outputs/base-reuse-10-s0-40.txt
2726869f15d5a47d1005b72d67122533cf38149af21419f96513b503514a6a88  outputs/base-rollback-10-s0-40.txt
fbe9052cb9e4dcf09575878f62fb7ad9625ff2f9652fc3bb2acea94d4c74abc7  outputs/base-unitwall-10-s0-32.txt
1209f324e87eea64b1e38a46d93435f5d11f0ae330215cb9579ac3db8d02b989  outputs/harness-base.txt
6f1627059cfacc68f1b207a343a4dcfaa52a7dd6bd3f924b824c3f396eecb4c6  outputs/k1.log
c088dd77c9db2819b6851245448edd6ad748b134550f16a0b4afca560d9aaba5  outputs/m2227-ops-overwrite.txt
8f84f159b57c55ef415a0c552853657a7eabae7865fcb6a32c8f0106ef789b30  outputs/m2228-ops-overwrite.txt
1e990de0b4817146b749d7cf837695cd762e7cb5935ed9093da15efe749812a5  outputs/m242-named-short.txt
fffb2fde31bb7af840af4908bf7358d7874c638dfc4d87ed53bd4c79c65893c0  outputs/m269-named-short.txt
c171b19bc8b8b130e2b0651bd02fa91fd1003ad11165e3efafbb16b9f2410053  outputs/rows-panics.txt
14fa383e42f182171e092fe1355876d4f3b74aebff5c3be17233d29b81cb5bc4  outputs/rows.txt
a036dbca86a72110003932135dd0bd85101cca9341e13f847e7669c61e13c640  outputs/v242u-named.txt
e694d4c5ed84874e69e889ce9116549d8076a22e6027344f31a2a1aa088c3c8b  outputs/v242u-ops_AfterFirstFile_PublishOverwrite.txt
e79e07524f658067e4fde80e3b439acbf50f97b0fffee7f22eafe9fce6f31b28  tools/apply-mutation.py
e41da68c5bc3a335d167bc74194e34f830cda136ad408b5ec387f0df2fa95a94  tools/apply-variant-v242u.py
b2b17b21a64d3c18949c364cdcf4579d60df0bf82ff9a0c2d956f756751e93bd  tools/rerun.sh
5a72e31b781f5994201d6595fbe44d77fda1174d64ab0280001c485aa485149d  tools/run-mutation-rows.sh
```

`outputs/` 里的 `base-*`、`m242-*`、`m269-*`、`v242u-named.txt` 是加 `ops` 模式与「抛弃根那一步」计数之前那一版原型跑的（判法相同，少一行 `abandoning_steps=`）；
`rows.txt` 的 `compile_errors=` 列在我那一次跑时数的是 `^error` 开头的行（把 `error: test failed` 也数进去了），脚本已改成只数 `^error[E`，
重数 10 行逐行是 0（`rows-panics.txt` 每段第一行）。

## 各格判定一览

| 格 | 判定 | 依据 | 量过 / 推的 |
|---|---|---|---|
| K2-a 合法崩溃态上误红 | 没打中 | 今天的 core 上 33 975 个首次崩溃态、365 228 个二次崩溃态，可写挂载以判据 N-配置 被拒 0 次（新判官无从红） | 量过（副本，取样域见第二节；不是层 0 全域） |
| K2-b 第 242、269 行那两类坏实现会不会漏 | 打中（射程） | 第二截之外的「二次崩溃之后再挂」被拒只计数、不判 C670；242 类的变体 V242u（只在带单元的发布上先轮换后根）在不含重挂的历史上第二截 0 个、二次崩溃之后 108 个 C670 为真的态 | C670 那一截量过（副本）；崩溃注入其余判官在这些态上红不红是推的 |
| K2-c 读故障的认定漏没漏别的步 | 没打中；认定那一支在今天的生成器里是死的 | 只有「崩溃恢复抛弃根」一步带读故障；1600 段历史里这一步出现 158 次、一次都没往录制流发东西 ⇒ 崩溃注入里 `Injected` 从不出现 | 量过（副本，harness 档执行器） |
| K2-d 撕裂那一支分不分得干净 | 没打中 | 层 0 撕裂态（原地覆写取第三态，含在上面 33 975 个里）14 800 个，可写挂载一次都没拒；两个变异下 C670 为真的撕裂态各 4、9 个，全是坏实现写出的见证值造成的 | 量过（副本） |
| K2-e 接线错了有没有东西红 | 打中（部分） | 没有一条用例在基线上断言崩溃注入报出 C670 不合法；调用点改成恒交 `Injected` 或换回旧判官时基线全绿，只靠第 2227、2228 行变异「活下来」才看得到；恒交 `NoneInjected`、窗口末尾下标取错这两种连变异表都看不到（`Injected` 从不出现） | 推的（读码 + K2-c 的计数） |
| K4 第 2215–2224 行（harness 档 10 行） | 10 行都在点名用例上红、红在点名的那一处 | 副本上逐行施加、跑那一行第 5 列的 cargo 参数 | 量过（副本） |
| K4 第 2225、2226 行 | 推得出红 | 两条点名的 checker 档单元用例用合成的流登记，读码逐步推 | 推的 |
| K4 第 2227、2228 行（E171 第 242、269 行） | 推得出「基线绿、变异红」 | 命名用例那段历史的短段域上，C670 为真的态基线 0、第 2227 行 10（非撕裂 6）、第 2228 行 25（非撕裂 16） | C670 那一截量过（副本）；整条 checker 档用例没跑，`new_findings.is_empty()` 在基线上成不成立是推的 |
| K4 第 2229 行（GPU） | 不归这条腿量 | 没卡时点名用例打 `NOT_RUN_NO_GPU_ADAPTER` 返回成功，门禁记本次未跑 | 推的 |
| K1 树表 0 条那一格 | 打中：「答不了」不是唯一读法 | 副本上放开那一格，模型照 D16 字面答推 2 条（txg 3、4，F = 2），实现一个字节不写，对拍报 `PublishCount` | 量过（副本） |

## 一、原型怎么量（K2、K4 共用）

`c670-probe`（`c670-probe/src/main.rs`）对一段历史：用 harness 的执行器跑、录流（小盘 `UnitAreaOf384Slots`，准入 `JudgedByTheFormula`，与 16-3 那条崩溃注入用例同一套），
按录制流切段，在候选段（整段落在 mkfs 之后、最后一个跑完的操作之前，同 `draw_crash_points` 的候选）上摆崩溃态：段内每个写取它的几态、任意组合、去掉全落那一个，
原地覆写的写（`TearableInPlaceOverwrites::of`）取三态、撕裂镜像用 `torn_image_of_in_place_overwrite`。每个崩溃态上：

1. 只读恢复（`read_recovery_fields(…, TheRecoveredVersion)`）取 `effective_root`，即崩溃注入交给判官的 `version_landed_on_after_the_crash`；
2. 在副本盘上起 `mount_writable_with_space_admission`（录下挂载发的流）；
3. 拒因在 D23 单子上而是判据 N-配置 的，调 16-3 新判官 `judge_writable_mount_refusal_of_a_crash_state`（`NoneInjected`），数交回的不满足条件里有没有
   `WitnessCriterionTrueOnACrashImageWithoutReadFaults`，有就记一个「C670 为真」的态（崩溃注入第二截在这个态上会判拒得不合法）；
4. 挂载做成、首次崩溃态不带撕裂写的，把挂载那一段写流再切段、每段取全部真子集（二次崩溃，不撕裂），在二次崩溃镜像上再起可写挂载，同样数 C670 为真的态
   （崩溃注入第三截在这些态上只计数「再挂被拒」，见第三节）。

取样域（这一条腿自己缩了什么）：命名用例那段历史（起点到第一个文件、再关掉重挂）层 0 全域首次崩溃态 1 069 129 个（撕裂态含在内；不撕裂 1 069 094 个），
第一个文件那次发布的单元写自成一段 20 个写，单这一段就 2^20 − 1 个。按定义 3b「一段历史自己就超过约 10^6 的，那段不跑」，这段历史的全域没跑，
交主 agent（第八节）。我跑的是「只展开写数不超过 N 的候选段」（`--segments-up-to N`）：命名用例那段历史与 16-3 用例自己的确定部分同一个域（N = 4，
用例对更长的段只按种子抽 2 个），随机历史取 N = 6；二次崩溃同样只展开挂载写流里写数不超过 4 或 6 的段（`--second-segments-up-to`）。
系统配置槽、根槽、journal 记录写都落在这些短段里（单元写那几段只有单元写），但单元写那几段的崩溃态一个没摆——这些数支持不了「层 0 全域上没打中」。

## 二、K2-a：合法崩溃态上会不会误红——没打中

今天的 core（冻结副本原样）上，C670 为真的态与判据 N-配置 的拒全是 0。各段输出整行抄：

| 历史 | 取样域 | 输出文件与原样行 |
|---|---|---|
| 命名用例那段（起点 + 关掉重挂） | 短段 ≤ 4，二次 ≤ 4 | `base-named-short.txt`：`first_states=76 with_torn_write=35 mounts_succeeded=76 n_config_refusals=0`；`second_states=795 second_n_config_refusals=0 second_c670=0` |
| BROAD 6 步，种子 0..8 | 短段 ≤ 4，二次 ≤ 4 | `base-broad6-s0-8-short.txt`：`first_states=692 with_torn_write=305 mounts_succeeded=692 n_config_refusals=0`；`second_states=7545 second_n_config_refusals=0 second_c670=0` |
| BROAD 6 步，种子 8..108 | ≤ 6，二次 ≤ 6 | `base-broad6-s8-108.txt`：`first_states=9476 with_torn_write=4215 mounts_succeeded=9476 n_config_refusals=0`；`second_states=103304 second_n_config_refusals=0 second_c670=0` |
| 回退偏重 10 步，种子 0..40 | ≤ 6，二次 ≤ 6 | `base-rollback-10-s0-40.txt`：`first_states=5568 with_torn_write=2470 mounts_succeeded=5568 n_config_refusals=0`；`second_states=60254 second_n_config_refusals=0 second_c670=0` |
| 抬 F 后复用偏重 10 步，种子 0..40 | ≤ 6，二次 ≤ 6 | `base-reuse-10-s0-40.txt`：`first_states=7804 with_torn_write=3480 mounts_succeeded=7804 n_config_refusals=0`；`second_states=84582 second_n_config_refusals=0 second_c670=0` |
| 删除多片下段叶偏重 10 步，种子 0..32 | ≤ 6，二次 ≤ 6 | `base-deleting-10-s0-32.txt`：`first_states=4729 with_torn_write=2070 mounts_succeeded=4729 n_config_refusals=0`；`second_states=53715 second_n_config_refusals=0 second_c670=0` |
| 朝单元区墙 10 步，种子 0..32 | ≤ 6，二次 ≤ 6 | `base-unitwall-10-s0-32.txt`：`first_states=5000 with_torn_write=2170 mounts_succeeded=5000 n_config_refusals=0`；`second_states=55033 second_n_config_refusals=0 second_c670=0` |
| 写死：mkfs 后挂、零单元发布、重挂、零单元发布、冷启动、重挂 | 层 0 全域（630 个，撕裂含在内），不做二次 | `base-ops-a-nosecond.txt`：`first_states=630 with_torn_write=55 mounts_succeeded=630 n_config_refusals=0` |

合计首次崩溃态 33 975 个（撕裂 14 800 个）、二次崩溃态 365 228 个，可写挂载一次都没拒。按机制推（推的）：判据 N-配置 要「系统配置见证的 tail 大于所选那一版末条记录」，
而今天的持久顺序是记录 → 屏障 → 根槽 FUA → 轮换 → 屏障，轮换落了根必落；同一个实例里根没落、记录落了的那一版由只读恢复按 journal 前缀重放成所选那一版，
c_见证 不大于它；取号写的 tail 取的是盘上自证过的槽的最大 tail（`highest_journal_tail_of`），也不大于所选那一版。层 0 的撕裂只撕得坏一块盘两槽里正被覆写的那一槽
（轮换与取号写前面都有一道屏障），另一槽自证得过，R2 下判得出，所以撕裂态上也不拒。

推翻条件：主 agent 在第八节那条全域命令里，命名用例那段历史 1 069 094 个不撕裂态上有一个 `first_c670` 不为 0；或在含 `Unlink`、`RaiseRollbackFloor` 的历史上
展开单元写那几段之后出现判据 N-配置 的拒。

## 三、K2-b：第 242、269 行那两类坏实现会不会漏——打中（射程）

C670 的题面是「没有任何读故障的崩溃态，不许让 D23（journal 的角色与格式） 已定项 14 可写挂载那条见证判据为真」（`.claude/kb/checks-owed.md:529`），
改法是「崩溃注入第二截在没注入读故障的崩溃镜像上复算见证判据，为真即判拒得不合法」（同一行）。16-3 照改法只接了第二截
`crates/singlefs-checker-tier/src/crash_injection.rs:2110`「match judge_writable_mount_refusal_of_a_crash_state(」。崩溃注入的第三截（第二截那次可写挂载途中再崩、再挂）
被拒时只计数：`crates/singlefs-checker-tier/src/crash_injection.rs:2771`「WritableMountAlone::Refused { .. } => {」，
下一行起记 `crates/singlefs-checker-tier/src/crash_injection.rs:2774`「.entry(RemountIdempotenceNotJudged::TheRemountWasRefused)」然后返回，不判拒得合不合法。
二次崩溃镜像同样是没注入读故障的崩溃态（第二截的挂载起在无故障的崩溃镜像上），题面管得到它，改法没接。

**造的历史与坏实现**：坏实现 V242u（`tools/apply-variant-v242u.py`，攻方自造，不在 `crates/mutations.tsv` 里）是第 242 行那一类（系统配置轮换先于根槽 FUA），
只在带单元的发布上错，零单元发布照旧先根后轮换；它改的是 `persist_publish_writes` 末尾那一处调用
`crates/singlefs-core/src/transaction.rs:1151`「persist_the_root_then_rotate_the_system_configuration(」。今天的持久顺序由一处函数供三条发布路径共用，
所以「只在一条路径上错」要改出这一处的分支——它是一种可达的坏写法（同一条持久顺序抄成两份、一份写错），不是第 2227 行原样。

| 历史 | 坏实现 | 第二截 C670 为真的态 | 二次崩溃之后 C670 为真的态（第三截不判） | 输出 |
|---|---|---|---|---|
| 起点 + 一次覆盖写（不重挂） | V242u | 0 | 108 | `v242u-ops_AfterFirstFile_PublishOverwrite.txt`：`first_states=72 with_torn_write=35 mounts_succeeded=72 n_config_refusals=0`；`second_states=933 second_n_config_refusals=108 second_c670=108` |
| 命名用例那段（起点 + 关掉重挂） | V242u | 5（撕裂 2） | 138 | `v242u-named.txt`：`first_c670=5 in_short_segments(<=4)=5 in_torn_states=2`；`second_states=1143 second_n_config_refusals=138 second_c670=138` |
| 起点 + 一次覆盖写 | 第 2227 行（E171 第 242 行） | 5（撕裂 2） | 120 | `m2227-ops-overwrite.txt`：`first_c670=5 in_short_segments(<=4)=5 in_torn_states=2`；`second_states=1080 second_n_config_refusals=120 second_c670=120` |
| 起点 + 一次覆盖写 | 第 2228 行（E171 第 269 行） | 8（撕裂 2） | 194 | `m2228-ops-overwrite.txt`：`first_c670=8 in_short_segments(<=4)=8 in_torn_states=2`；`second_states=495 second_n_config_refusals=194 second_c670=194` |

为什么 V242u 在不重挂的历史上第二截看不见：起点里第一次挂载的两次暖机是零单元发布（不受 V242u 影响）；第一个文件与覆盖写受影响，但「轮换落了、根没落」的态里
同一实例的记录已落，只读恢复按 journal 重放落到那一版，c_见证 不大于它，可写挂载不拒。只有新实例的第一次带单元发布（带树的一版上重挂的暖机）记录不跨实例重放，
所选那一版停在上一个实例，见证判据为真——这一步在这段历史里只出现在第二截那次挂载里，它的「轮换落了、根没落」只能经二次崩溃摆出来，正落在第三截。
第 2227、2228 行原样在起点那一次挂载上就红，所以这条射程缺口今天不让两行变异活下来；随机战役里含「带文件的一版上关掉重挂」的历史也会在第二截抓到 V242u。
漏的是：不含这一步的那几段历史，与只在「带树那一版上的挂载」这条路径上错的坏实现（V242u 是一例）。

**打中之后四句**：
1. 分不分辨臂：分辨。「只接第二截」与「第二截加第三截」在 V242u、不重挂的那段历史上 0 对 108；在第 2227、2228 行上两臂都红（5 / 120、8 / 194）。
2. 被判的系统看不看得到：看得到。第三截手里有二次崩溃镜像（`materialized_crash_image(&image)`）与再挂的结局；今天 `WritableMountAlone::Refused` 只留了
   `member` 字符串与映射后的理由、没留 `MountError` 本身，接新判官要把错误带出来（改一处枚举），读故障沿用第一次崩溃那个态的认定。
3. 满足的是判据字面的哪一分句：C670 题面「没有任何读故障的崩溃态」那一句；不是改法那一句（改法只点名第二截，16-3 照改法做了）。即打中的是改法的射程比题面窄，
   不是实现没照改法做。
4. 跑前条款给的改法在打中的格上还中不中：本轮正文第五节的改法去向是「并进下一批实现员」，C670 原改法（第二截）在这几格上不中；第六节改法 P1 中（只在我的模型上量过）。

## 四、K2-c：读故障的认定——没打中，认定那一支今天是死的

只认一类步：`crates/singlefs-checker-tier/src/crash_injection.rs:1016`「HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => true,」。
我在 harness 执行器里核了别的步：读故障只由 `DeviceReadingZerosOverHiddenRanges` 造，只有 `apply_crash_recovery_abandoning_the_newest_root` 用它；
`execute_history_with` 交的注入计划是 `SharedFaultPlan::unarmed`。所以认定没漏别的步（读码）。

这一步今天一个写都不发：`crates/singlefs-harness/src/history.rs:487`「C554 乙之后读阶段重读一次仍读不出、取号之前拒可写」。量过（`abandon-*.txt`，12 步历史、
每组 400 段，只跑执行器不摆崩溃态）：

```
abandon-broad.txt:    abandoning_steps=116 abandoning_steps_that_sent_something=0
abandon-deleting.txt: abandoning_steps=42 abandoning_steps_that_sent_something=0
abandon-rollback.txt: abandoning_steps=0 abandoning_steps_that_sent_something=0
abandon-reuse.txt:    abandoning_steps=0 abandoning_steps_that_sent_something=0
```

（上面四行是各文件里那一行原样，前面加了文件名；`abandoning_steps` 数的是观察到的这一类步，含前提不满足的。）1600 段历史里 158 次，没有一次往录制流发东西，
所以 `read_faults_in_a_crash_state` 在崩溃注入里恒交 `NoneInjected`，`Injected` 那一支（不判 C670）只在第 2225、2226 行点名的合成流单元用例里走到。
这不是误判，是射程：等这一步哪天真写了（例如乙罩不到的那一形被造出来），这一支才有对象。

## 五、K2-d：撕裂那一支——没打中

层 0 的第三态（原地覆写撕裂）在系统配置槽上摆得出：第二节合计 14 800 个撕裂态，可写挂载一次都没拒（R2：一槽撕裂、另一槽自证过，判得出）。
16-3 的 harness 用例拿「盘 0 两槽都撕裂」那一份手造盘面钉合法；这种盘面在层 0 里摆不出（取号写与轮换写前面各有一道屏障，同一块盘两槽不同在一段里）。
两个变异下 C670 为真的态里撕裂的各 4、9 个（`m242-named-short.txt`：`first_c670=10 in_short_segments(<=4)=10 in_torn_states=4`；
`m269-named-short.txt`：`first_c670=25 in_short_segments(<=4)=25 in_torn_states=9`），挑出的例子里撕的是一块盘的那一槽、另一块盘落了坏实现写的见证值——
红是坏实现造成的，不是撕裂被误当成见证。崩溃注入本身不摆撕裂态（`CrashImage` 的 `persisted` 只有落与不落），这一格对第二截没有对象。

## 六、K2-e：接线错了有没有东西红——打中（部分，推的）

接线是两处：调用点把认定交给结构体 `crates/singlefs-checker-tier/src/crash_injection.rs:1311`「read_faults: read_faults_in_the_crash_state,」，
判官那一处取出来交给新判官 `crates/singlefs-checker-tier/src/crash_injection.rs:2116`「refusal_on_the_crash_image.read_faults,」。
全仓断言崩溃注入里不合法拒数的只有新用例那一句 `crates/singlefs-checker-tier/src/crash_injection.rs:4907`「tally.writable_mount_refusals_on_the_d23_list_illegal, 0,」
（`grep -rn writable_mount_refusals_on_the_d23_list_illegal crates/` 别处只有计数字段本身），没有一条用例在基线上要它大于 0。逐种接错：

| 接错的样子 | 基线用例 | crates 变异整表 | 判定 |
|---|---|---|---|
| 调用点恒交 `Injected`，或换回旧判官 `judge_writable_mount_refusal` | 全绿（新用例断言的是 0） | 第 2227、2228 行由「抓到」变「没红」，整表判失败 | 只有变异整表看得到（提交时跑）；推的 |
| 调用点恒交 `NoneInjected` | 全绿 | 全绿 | 看不到；今天 `Injected` 不出现（第四节），行为也不变 |
| 窗口末尾下标取错（例如取段首那个写） | 全绿 | 全绿 | 看不到；同上 |

改法 P2（第九节）给第一行补一条基线上就要红的用例。

## 七、K4：15 行变异

harness 档 10 行在副本上逐行跑（`tools/run-mutation-rows.sh`，施加 → 那一行第 5 列的 cargo 参数 → 从冻结副本还原），结局 `outputs/rows.txt` 原样：

```
line=2215 rc=101 named=FAILED compile_errors=1
line=2216 rc=101 named=FAILED compile_errors=1
line=2217 rc=101 named=FAILED compile_errors=1
line=2218 rc=101 named=FAILED compile_errors=1
line=2219 rc=101 named=FAILED compile_errors=1
line=2220 rc=101 named=FAILED compile_errors=1
line=2221 rc=101 named=FAILED compile_errors=1
line=2222 rc=101 named=FAILED compile_errors=1
line=2223 rc=101 named=FAILED compile_errors=1
line=2224 rc=101 named=FAILED compile_errors=1
```

（`compile_errors=1` 是那一次的数法数进了 `error: test failed`，真的编译错逐行 0，见第一段说明。）基线两份新用例文件 7 条全绿（`outputs/harness-base.txt`，从副本日志里摘的 `test` 行：
`test result: ok. 4 passed` 与 `test result: ok. 3 passed`）。15 行的锚点在冻结副本里逐行恰好命中一次（`apply-mutation.py` 的核）。各行红在哪（`rows-panics.txt`）：

| 行 | 点名要证的那一处 | 点名用例红在哪 | 判定 |
|---|---|---|---|
| 2215 | 模型卸载那一串带的 F | 对拍报根带的 F：模型 0、实现 3 | 量过，对 |
| 2216 | 实现卸载带的 F（core） | 同一格：覆盖写到 txg 4 那条用例里 txg 5 那条根，模型带 F 4、实现带 F 0；txg 3 那条用例同样红 | 量过，对 |
| 2217 | 树表 0 条不答 | 第 329 行断言：模型答了 | 量过，对（这一格本身见第八节） |
| 2218 | 对拍比根带的 F | 第 315 行断言：`left: Ok(())`、`right: Err(RollbackFloor)` | 量过，对 |
| 2219 | 卸载之后会话关 | 第 246 行断言（`has_writable_session`） | 量过，对 |
| 2220 | 胶水交全部根 | 对拍报写出的根的条数 `PublishCount` | 量过，对 |
| 2221 | 至多 R 次 | 对拍报 `PublishCount`：模型 2 条 | 量过，对 |
| 2222 | C670 那一条丢了 | 交回 `Legal`、要 `Illegal` | 量过，对 |
| 2223 | 撕裂那一支没分开 | 撕裂那条交回 `Illegal(…Neither…)`、要 `Legal`；同时只落轮换那条也红（支名变成 Neither） | 量过，对；第二条红是连带 |
| 2224 | 注入过读故障也判 | 交回 `Illegal`、要 `Legal` | 量过，对；崩溃注入里不出现 `Injected`，这一行只证判官函数 |
| 2225 | 认定一律没注入 | 合成流，窗口末尾 20、35 两处要 `Injected` | 推的（checker 档单元用例没跑） |
| 2226 | 一个都没发也算注入 | 合成流里那一步 20 → 20，窗口末尾 20、35 两处交回 `Injected` | 推的（逐步推：`operations_before_the_step <= 20` 成立） |
| 2227 | E171 第 242 行 | 见下 | C670 那一截量过；整条用例推的 |
| 2228 | E171 第 269 行 | 见下 | 同上 |
| 2229 | GPU 最新根水位一格 | 有卡才跑；没卡打 `NOT_RUN_NO_GPU_ADAPTER`，门禁 `checker-tier-crates-mutation-replay` 记本次未跑 | 推的，不归这条腿 |

第 2227、2228 行改指的用例是 `no_crash_state_without_read_faults_after_a_remount_is_refused_writable_for_a_witness_newer_than_the_selected_version`，
它摆崩溃态用 `crates/singlefs-checker-tier/src/crash_injection.rs:4871`「segment_writes: 4,」（更长的段按种子抽 2 个）。原型在同一段历史、同一个短段域上量（撕裂态含在内，
崩溃注入不撕裂，括号里是不撕裂的个数）：

| core | 第二截 C670 为真的态 | 落在哪一步 | 输出原样行 |
|---|---|---|---|
| 今天 | 0 | — | `base-named-short.txt`：`first_c670=0 in_short_segments(<=4)=0 in_torn_states=0` |
| 第 2227 行 | 10（6） | 起点 5、关掉重挂 5 | `m242-named-short.txt`：`first_c670=10 in_short_segments(<=4)=10 in_torn_states=4` |
| 第 2228 行 | 25（16） | 起点 8、关掉重挂 17 | `m269-named-short.txt`：`first_c670=25 in_short_segments(<=4)=25 in_torn_states=9` |

第 2227 行那几态是「轮换落了、根没落」（例：`landing=["SystemConfigurationSlot@d0:2", "SystemConfigurationSlot@d1:0", "RootRecordFua@d1:0"]`，段 = 轮换两写加根槽写共 3 个，
不超过 4，整段展开）；第 2228 行那几态是取号写落了、暖机根没落（起点那一次落到 mkfs 那一版，c_见证 是槽世代号，记录读不出，走「两条都读不出按真」那一支）。
据此推（推的）：用例断言 `illegal == 0` 在两行变异下都不成立 ⇒ 红；基线上第二截不拒 ⇒ 这一句绿。基线整条用例绿还要 `new_findings.is_empty()` 等别的判官都绿，
那一部分我没跑、也没核过别的崩溃注入用例在同类起点上的结局，留给主 agent 提交时那一趟。
用例对长段只抽 2 个、我没摆，那几段只有单元写，按第二节的机制推不会出见证判据为真的态。

## 八、K1（兼）：树表 0 条那一格——「答不了」不是唯一读法

模型在这一格报 `NotModeled`，理由写的是条款没判：`crates/singlefs-harness/src/model.rs:2474`「"模型里现行版本 {:?} 树表 0 条：卸载推不推空发布条款没判，模型不答",」。
条款字面判了：D16 已定项 1 正常卸载那一行「推一串空发布把 F 抬到现行那一版的 txg」（`.claude/kb/decisions/16-发布语义.md:42`），那一行没有把树表 0 条的一版划出去；
已定项 9 又写了这种版本上的空发布长什么样，「第一次可写挂载的暖机时树表 0 条 ⇒ 零单元」（`.claude/kb/decisions/16-发布语义.md:221`）。
不推的是实现：`crates/singlefs-core/src/mount.rs:2344`「return Ok(Unmounted::NothingWrittenOnAVersionWithoutFile {」，依据是
`crates/singlefs-core/src/mount.rs:2216`「抬 F 买不到东西（主 agent 2026-09-26 定，实二交回第一问；推的，被攻过零轮）。」——这条定案没进 D16（正文第二节也这样记）。

所以这一格不是「条款沉默」，是「条款字面与一条没进条款的定案说反话」，读法至少三种：

| 读法 | 模型答什么 | 对拍在这一格上 | 量过 / 推的 |
|---|---|---|---|
| 甲：照 D16 字面 | 推空发布、F = 现行 txg | 对实现红 | 量过（副本）：放开那一格（第 2217 行的改法）后 `k1.log`：`ATTACK-K1 model_roots(txg,F)=[(3, 2), (4, 2)] implementation=一个字节都不写 verdict=Err(PublishCount)` |
| 乙：照 2026-09-26 那条定案 | 零条根、会话关、F 不动 | 对实现绿 | 推的（`observed_unmount` 对这一支交零条根） |
| 丙：今天的写法，不答 | — | 不比 | 量过（基线用例绿） |

丙把甲乙之间的冲突藏进「模型答不了」：哪天卸载进了随机历史，这一格在对拍里一次都不会红。甲里模型推了 2 次（txg 3、4，F 都是 2），对拍在条数那一格就红了。这一格是条款与定案之间的岔路，归用户；攻方只报「不是唯一读法、理由那句与 D16 字面不符」。

四句：分辨臂（甲红、乙绿、丙不比）；系统看得到（`contents.has_trees()`）；满足的是 K1 题面「是不是唯一的读法」那一问；跑前条款没给这一格的改法。

## 九、改法（全部只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 基线上 | 坏实现上 | 量过 / 推的 |
|---|---|---|---|---|
| P1：第三截（二次崩溃之后再挂）被拒、理由是判据 N-配置 时，也调 `judge_writable_mount_refusal_of_a_crash_state`（崩溃镜像换成二次崩溃镜像，所选那一版取二次崩溃镜像上只读恢复的 `effective_root`，读故障沿用第一次崩溃那个态的认定），C670 那一条为真就报红；`WritableMountAlone::Refused` 要把 `MountError` 带出来 | K2-b | 第二节 365 228 个二次崩溃态上判据 N-配置 的拒 0 个 ⇒ 不添红 | V242u：不重挂那段 108、命名那段 138；第 2227 行 120、第 2228 行 194（各自那段历史） | C670 那一截量过（副本原型）；接进 `crash_injection.rs` 的那几行、它与 I-8.4 那一判的先后，推的 |
| P2：一条基线上就要红的接线用例——在 harness 档用例 `writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 那份「只落了轮换」的盘面上，经崩溃注入的第二截入口（或把 `judge_the_writable_mount_on_the_crash_image` 拆出可单测的一截）判，断言 `illegal == 1` | K2-e 第一行 | 红（这正是要的） | — | 推的 |
| P3：K1 那一格先定读法（甲改实现或乙把 2026-09-26 定案写进 D16），模型照定下的答；定之前把理由那句改成「条款字面推、实现照未入条款的定案不推，两者冲突」 | K1 | — | — | 推的；岔路归用户 |

## 十、没打中的形状

- 误红：BROAD / 回退偏重 / 抬 F 后复用偏重 / 删除偏重 / 朝单元区墙五组权重，6–10 步，共 252 段随机历史（8 + 100 + 40 + 40 + 32 + 32，见第二节表），
  加写死的两段（命名那段、无文件那段）；短段域 ≤ 4 或 ≤ 6，撕裂态含在内；二次崩溃短段域。没摆单元写那几段的崩溃态。
- 别的读故障步：读码查了执行器全部十类步与注入计划（未挂），量了 1600 段历史里抛弃根那一步发没发东西。
- 撕裂那一支：层 0 第三态 14 800 个；没造「同一块盘两槽同时撕裂」的流（今天的持久顺序摆不出）。
- 第 2227、2228 行在不重挂的历史上会不会漏：不漏（起点那一次挂载就红，`m2227-ops-overwrite.txt`、`m2228-ops-overwrite.txt`）。
- 没试的形状：E171 第 270 行那一类（见证值偏小，判据假而该真）——方向相反，不归 C670；`Unlink`、`CreateInode` 只在删除偏重那 32 段里出现，没单独写死。

## 十一、取样、估时与预算

每次跑之前先跑一小段估时（命名那段短段域 76 个首次崩溃态 0.6 秒、带二次崩溃 7.0 秒）；随机历史按 8 线程、每批一组权重跑，批内段数按小段估时定，不设批限时。
状态数（首次 + 二次，撕裂含在内）都远低于每次约 10^6、全腿约 10^7 的上限：

| 批 | 段数 | 线程 | 首次崩溃态 | 二次崩溃态 | 挂钟（秒，原型自报） |
|---|---|---|---|---|---|
| 命名那段，基线 | 1 | 1 | 76 | 795 | 7.0 |
| BROAD 6 步 0..8 | 8 | 8 | 692 | 7 545 | 9.0 |
| BROAD 6 步 8..108 | 100 | 8 | 9 476 | 103 304 | 120.0 |
| 回退偏重 10 步 0..40 | 40 | 8 | 5 568 | 60 254 | 66.2 |
| 抬 F 后复用偏重 10 步 0..40 | 40 | 8 | 7 804 | 84 582 | 94.9 |
| 删除偏重 10 步 0..32 | 32 | 8 | 4 729 | 53 715 | 75.4 |
| 朝单元区墙 10 步 0..32 | 32 | 8 | 5 000 | 55 033 | 86.2 |
| 写死的无文件那段，层 0 全域 | 1 | 1 | 630 | 0 | 2.4 |
| 变异与 V242u 六次 | 1 × 6 | 1 | 508 | 5 394 | 3.0–12.7 |
| 抛弃根那一步计数（只跑执行器） | 1600 | 8 | 0 | 0 | 每组 1.6–2.3 |

合计约 3.5 万个首次崩溃态、37 万个二次崩溃态。缩了什么：只展开写数不超过 4 或 6 的段（第一节）；命名那段历史的层 0 全域（1 069 129 个）一段就超过 10^6，没跑，交第十二节。

## 十二、要主 agent 跑的命令（可选，核 K2-a 与 K4 基线那一半）

命名用例那段历史的层 0 全域（首次崩溃态 1 069 129 个，不做二次崩溃），8 片并行。先跑完第一段的 `rerun.sh`（它末尾的二进制是 V242u 那一版，下面先按基线重编）：

```
W=/tmp/claude-1000/<核查草稿目录>; REPO=/home/fy5090/code/singlefs
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="攻方原型全域分片，峰值表里那一次不代表这一次"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="攻方原型全域八片，各片独立进程、只读同一个二进制"
(cd "$W/ws" && CARGO_TARGET_DIR="$W/target" nice -n 19 bash "$REPO/research/scripts/run-with-memory-cap.sh" 4G bash "$REPO/research/scripts/capped.sh" 8 cargo build --offline --release -p c670-probe)
rm -f "$W"/full-b1-*.rc
for k in 0 1 2 3 4 5 6 7; do { nice -n 19 bash "$REPO/research/scripts/run-with-memory-cap.sh" 2G "$W/target/release/c670-probe" named --no-second --shard "$k/8" > "$W/full-b1-$k.txt" 2> "$W/full-b1-$k.err"; echo "$?" > "$W/full-b1-$k.rc"; } & done
wait
ls "$W"/full-b1-*.rc | wc -l     # 要是 8，不是就整批作废
for k in 0 1 2 3 4 5 6 7; do cat "$W/full-b1-$k.rc"; grep -E '^first_states|^first_c670=|^elapsed' "$W/full-b1-$k.txt"; done
```

工作目录 `$W/ws`（`rerun.sh` 建的副本）；线程：8 片各 1 线程。估时（推的）：每态约 5–8 毫秒，每片约 13.4 万态 ⇒ 每片 11–18 分钟。
判读：8 份 `first_states` 加起来应是 1 069 129；`first_c670=` 八份都是 0 支持第二节「基线不误红」在这段历史的全域上成立，有一份不为 0 就推翻它（看 `example first` 行）。

## 十三、这条腿自己的限度

- 原型的「C670 为真」只复现崩溃注入第二截里新判官那一判（同一个判官函数、同一个 `effective_root`、`NoneInjected`），没跑崩溃注入的其余判官；`DiskAfterTheRefusal`
  与只读读出那两格我交的是常量（不影响 C670 那一条是否进不满足条件的清单）。
- 二次崩溃只摆挂载写流里短段的全部真子集（不撕裂），崩溃注入自己的二次崩溃按种子抽，两边的态集合不同；P1 的「基线不添红」只在我摆的那一部分上成立。
- V242u 是我造的坏实现，可达性靠「持久顺序抄成两份」这种写法，没有在仓里找到现成的先例。
- 没跑的：checker 档测试目标一个没跑（第 2225–2229 行与 16-3 那条崩溃注入用例的整条结局都是推的）；K3（GPU 那一格）不归这条腿。
- 违规一处：第一次 `cargo build`（原型首编）没经内存包装就跑了，之后的编译都经包装；那一次只编、退 0。

## 十四、没做什么

- 没重数 C670 挡路列要的「今天的 core 上的 W 与 61 个见证拒」（E171 自己那几段历史没跑）；第七节两行变异的数是命名那段历史上的，不是 E171 的取样。
- 没跑命名那段历史的层 0 全域（第十二节交主 agent）；随机历史没展开单元写那几段。
- 没改 `crates/`、kb、门禁；副本上的改动都在草稿目录，已删（见交回）。

## cite-check 末行

`python3 research/scripts/cite-check.py research/prompts/m3-formal-code-b16-r1-opus-output.md --root /tmp/claude-1000/m3-formal-code-b16-r1/tree --root . --background research/prompts/_m3-formal-code-b16-r1-background.md`：

```
  ✓ 核了 16 处引文，对上 16 处，没判 0 处（逐处列在上面）
```
