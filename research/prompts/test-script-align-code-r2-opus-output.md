# test-script-align-code-r2 攻方腿（Opus）报告：J1、J2 为主，J3（2026-10-02）

被判的是冻结副本 `/tmp/claude-1000/test-script-align-code-r2/tree/` 里三份补丁与收尾合入之后的代码。下文引代码写冻结副本的绝对路径与行号；量到的数都是草稿副本（`/tmp/claude-1000/test-script-align-code-r2-opus/tree`，加了两处只给原型用的口子）上原型跑的，不是入库装置的数。checker 档包的测试一条没跑；两流用例那几条是在原型里照它们的造法与断言重算。

## 复跑

```
bash research/prompts/test-script-align-code-r2-opus-model/rerun.sh <空的工作目录> [/tmp/claude-1000/test-script-align-code-r2/tree]
```

它从冻结副本拷一份仓、打 `patch_tree.py`（看判器输入原文的口子，不改摘要）与 `patch_tree_fix.py`（我提的 B0，环境变量 `R2_FIX_BOUND_BY_THE_BASE_ROOT` 开，默认关），照模型目录的源码建原型包 `proto`（依赖那份仓的四个包，开 `verdict-store,gpu`），编完一条一条跑六个二进制，原样输出落进 `<工作目录>/runs/`，末尾 grep 出各行。都经 `research/scripts/run-with-memory-cap.sh`、`capped.sh 12`。展开 16 那几批要一个多小时，只复跑展开 6 的设 `R2_RERUN_QUICK=1`。本腿实跑的原样输出在模型目录 `outputs/` 下（下文 `outputs/…` 都指那里）。L1 一节读的主仓入口脚本此刻的 sha256：`research/scripts/crash-amplification.sh` = `7d9e1d88f9a2354e83e74bfb564ea11832bff143506f2af5ee90c88399a47cf1`。

模型目录各文件 sha256（`SHA256SUMS` 原样）：

```
5ea12728d70d0e562abf439a4e4e5777625a8fcb5c153c02d6211dd2934e118c  ./k3check.py
4dfc161ad863541caf51aa17fc0327aa727f26d75e4734a342a78c389856c634  ./make_common.py
d41ad25e44f392fbf2409dcabcc78520897dd4d9bbc9e0695f31c2f0ae5c4922  ./outputs/g1_outcomes-icd-missing.out
ca98e701fd999a164f506214652de563bbc93982f6627044fe20dac7aa6a21be  ./outputs/g1_outcomes-live.out
9fad8512c29d259fd501752b71a025e47f8b7b37173c717a97dc2d32f05493b6  ./outputs/inscope.tsv
94f42d1ee5dc4a1fa17342891923fba8ce1da6e856ba1796f3e7e8f3140a36a7  ./outputs/j1_diff-16-A2.out
24e63c4c935c470634a3976f3864be3fddc5833ff4d0aa305ba2566bc554138d  ./outputs/j1_diff-16-A2.time
22dbbdd01dd5a9d6722b44d72ee611926b0b929eed97f378406e8e6e495e6153  ./outputs/j1_diff-16-A.out
68c6f60c6dddb18db44b36f7592d2ef261a0cc64a0d00d1fe7cc23c38f4c7e8d  ./outputs/j1_diff-16-B.out
7506e68f2839d47ec41dee39a574a0b8d514b0384c71d7c5ac29ddb71a6e577e  ./outputs/j1_diff-16-B.time
d08d1f89fd9d6428a5b7a34ca0b79507059ac0262492524bedaadeb166f56912  ./outputs/j1_diff-16-C.out
f9b6ee8a3ab1d2c8312bda060751e2665ba09621d8ec99a7c815dec58ff9a8f7  ./outputs/j1_diff-16-rollback.out
246ff651b3ed45d99f2eaec89133b9e7c7782585559bfb2450f524b64a67784e  ./outputs/j1_diff-16-rollback.time
a9dbb8882e31a9c0ad79a8c1464529a4853fa29a2bdf266fcf929b0cb1e28e79  ./outputs/j1_diff-6.out
4f58ca764ad114e1419d636245075c3cd933440b7203c513e1e3614314b2f7bd  ./outputs/j1_flows-6.out
8a59e2789d1659dd1fa578df384c5786f16a4a42aa134797e53afbd14972c90f  ./outputs/k3check.out
19f32d5ddfeb31074676c1e1a0e78d129ff30481914a4403db76c8ed56785580  ./outputs/m3_candidates.out
47f0c339c5399073ec891a1aa07377f60e4cac5c99d4cb780dfc76f0f56691ac  ./outputs/m3_twin.out
8f3c91ebe2088ae0c84ba4314400cbadae0f30207d97acb29c8dd617a640b16b  ./outputs/r1_tests-as-merged.out
254f30a3f4c34edca3198dd2a3057c26808b6daa6f3a6c6e1853f3e225f651a8  ./outputs/r1_tests-fix-base-root-bound.out
88b764a1f1128af1f7f4130fae9fbc3a1339498632c10de7ad7c3983b871c5ba  ./patch_tree_fix.py
493633b26ac9ec20da275cc895c2583fba4215ce10a1b61a2d4e790fac8e2395  ./patch_tree.py
4715babca699c6eec47d2a9f9281d5b65f21b43bef476aa4363879fa68aaa896  ./proto/Cargo.toml
a3d6068f83c32b6412655a906db11d6dfeddd3aad6328f3fbb190e4dcb81c1a1  ./proto/src/bin/g1_outcomes.rs
5c620c89224d9de56dfed660e26289b6c035f3b0502910c6a3113daf638873a4  ./proto/src/bin/j1_diff.rs
1e0aa70de8b6d7c2b7a6f731972b55c8b176a76b8b700872b41879cdd4ed1013  ./proto/src/bin/j1_flows.rs
dbe842003c8bd97858a9c6283fa67480362466ccc5690d0dfd225460f1735a4b  ./proto/src/bin/m3_candidates.rs
5d74d77d8024bcdb1ca809efa6dd88260bb3f9556443e581ad7ea759b949d088  ./proto/src/bin/m3_twin.rs
ff51b381eaef34e19e04315010d68fc51c9169ce28b9062dd7ad8fa78c59b9a7  ./proto/src/bin/r1_tests.rs
4fcb3583653aa542c8b90632a00216664afa0b75463103d7868f2966f8055c7b  ./proto/src/common_crash_points/mod.rs
a3393a949cd5aa04792eb286e0dfd09be78b28e863d82b9e40db3151ed8ec4fd  ./proto/src/common/mod.rs
4b3dc56332354645d83296023743817f5d445914ceab6c0884db45c48b6a5d19  ./proto/src/flows.rs
04ed73525268e2c6414acfd2f3acda84aeb01bf8d1a09102dc1d86430ea26bdc  ./proto/src/lib.rs
790175d921a1697715551ccf10dfd7f6c14ede9346af14d7313fe8617c332ca2  ./rerun.sh
```

## 各格判定一览

| 格 | 子项 | 判定 | 一句话 |
|---|---|---|---|
| J1 | 第一个根槽写之前的点 | **打中** | 这一点判器输入里整张版本表都进，版本表因流而异，前缀第一个点块键分开（8 态、两边判定逐态相同）；两流用例文件里五条用例照原型重算推的会红（`left=9 right=10`、第 0 个点判器输入不同） |
| J1 | 块键相同的点判定是不是逐态相同 | 没打中 | 两流、到 D 对到卸载、到卸载对七种「只改点之后」的变体，展开 6 与 16 两档，块键相同的点判定与第 ② 段核对位 0 态不同 |
| J1 | 两个推的前提在哪条流上不成立 | 没打中（线索） | 今天四种流根槽写 txg 都严格递增、基镜像都是 mkfs 之后；退成整段写表那一支走不到，也没有用例或变异罩着 |
| J1 | 别的白丢的共享 | 不另算打中 | 被判的根槽写落在点里与点后（每对流 1 点）、没做完那次发布的根槽写只差实例表指针（2 点 3 态），判定都相同；前者按字面判器输入确实不同 |
| J2 | G1 没有 `/sys/bus/pci` 的无卡机器、或一个设备读不出 class | **打中**（主 agent 若只认判据括号里那三种，归可议） | 判 `PciListingUnreadableWhichC620LeavesUndecided`、每条 GPU 用例红；上游要的是本次未跑 |
| J2 | G1 容器、vGPU、多卡、非 NVIDIA 独显 | 没打中 | 照 C620 的设计；多卡只起来一张时照跑、另一张不报，记线索 |
| J2 | L1 按行相加重复计 | 没打中 | 主仓工作区的入口脚本已按流分组、每组只取一次，试的五种行序列都对 |
| J2 | S2 复现这一段 | 没打中（读代码，没跑） | 出红的用例与大档入口都只调同一个判定函数，复现这一段只在有新发现时打、判的就是新发现 |
| J2 | H9 几种组合 | 没打中（读代码） | 登记了而没卡、没编特性、没登记都报错不退回；第 ③ 段照这一趟的核对代码摘要读 |
| J2 | 线程数设 0 panic | 没打中 | 没有脚本设这三个变量；线索：`capped.sh` 罩不住它们 |
| J3 | 第 1587 行与 M3 新用例 | **打中** | 管根槽写的那个点只有 1 态（根槽写没落），两边都绿；这条用例在基线上就红，变异下只红在直接比键那条断言上；145 种单字节改坏 0 种能在那一点判得不同 |
| J3 | 第 1432–1434、1588、1680–1682 行 | **打中**（与 J1 第一行同一个病根） | 点名的用例推的在基线上就红，证红没有意义；B0 改法下回绿 |
| J3 | 53 行静态核 | 没打中 | 锚点、替换文、点名的测试、目标、筛选词、特性、`#[ignore]` 都对 |
| J3 | C604 没留变异的理由 | 站得住一半 | 本机有卡确实分不出；但该写成「换不掉的那一处」，取卡入口今天换不成假的 |

几个打中的改法我都只在自己的模型上量过或只是推的，被攻过零轮（各节的表里逐格标了）。
## J1 第一节：第一个根槽写之前的点把整张版本表写进判器输入（打中）

### 打中的历史

1. 判器输入里够得着的版本按「这个点之前与没做完的那次发布的根槽写」截；一条都没有时整张表都进：「///   够得着的版本是（txg，实例）不超过这几条根里最大那一条的行；这个点之前一条根槽写都没有、或没做完的那次发布在写表里没有根槽写，」（`/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:395`），下一行「///   上界取不出，整张表都进。」（`/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:396`）；代码是「None => text.push_str("versions_up_to=every_version\n"),」（`/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:537`）。
2. 每条流的第一个点 `acquire_instance_and_warm_up.system_configuration_slot.1` 只有系统配置槽写：它之前没有根槽写，它里面没有单元写与记录写（不算没做完的发布），走的正是这一支。这个点的状态只能落回基镜像里的根（同一份文件自己的说法：「///   才核「根在而记录不在」。根槽写的 txg 沿写表严格递增时这两样对这个点的状态都不成立：落到的根至多是没做完的那一版，基镜像里是 mkfs 的」，`/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:401`），oracle 只找 (txg，实例) 不超过落到的根的那几版，版本表里哪一行都够不着。
3. 于是两条流只要版本表有一行不同（哪怕是最后一次发布那一版），第一个点的计划哈希就不同、块键就不同、这一块不复用。版本表本来就因流而异：两流用例的 A、B 覆盖写内容不同，txg 4 那一版就不同；到 D 与到卸载两条固定脚本（`research/scripts/crash-amplification.sh` 登记表里排在第一、第二的两项，同一份库）后者多了 txg 10–19 十版。

可达的历史：两流用例照原样跑；或固定脚本两项照全量的次序在同一份库上跑。不用造任何故障。

### 量到的（草稿副本上的原型，流照用例的造法拷来、盘换成内存稀疏盘；展开上限 6，与两流用例的 `LARGEST_EXPANDED_SEGMENT_WRITES` 相同）

两流用例里与 R1、M3 相关的七条用例，照各自的造法与断言在原型里重算（`r1_tests`，checker 档的测试本身没跑）。冻结副本原样（`outputs/r1_tests-as-merged.out`）里 FAIL 的行原样：

```
R1TEST test=two_flows_sharing_their_prefix_reuse_its_verdicts_and_fork_only_at_the_tail assertion=recording_b.blocks_already_judged==prefix_blocks FAIL left=9 right=10
R1TEST test=two_flows_sharing_their_prefix_reuse_its_verdicts_and_fork_only_at_the_tail assertion=prefix_lines_end_with_reused_or_not_expanded FAIL offending=["acquire_instance_and_warm_up.system_configuration_slot.1 src/common_crash_points/mod.rs::acquire_instance_and_warm_up.system_configuration_slot.1::16a20ec3dd4d343d0f867072854703c8125492d018b1e537ffe873aa18542175 checked"]
R1TEST test=a_healthy_flow_judged_after_its_twin_with_a_miswritten_version_table_reads_back_its_own_verdicts assertion=recording.blocks_already_judged==prefix_blocks FAIL left=9 right=10
R1TEST test=a_flow_with_a_miswritten_version_table_judged_after_its_healthy_twin_reads_back_its_own_verdicts assertion=recording.blocks_already_judged==prefix_blocks FAIL left=9 right=10
R1TEST test=a_version_the_prefix_cannot_reach_leaves_the_judge_inputs_of_the_prefix_alone assertion=prefix_judge_inputs_equal FAIL prefix_points_with_unequal_inputs=[0] first_name=Some("acquire_instance_and_warm_up.system_configuration_slot.1")
R1TEST test=reused_verdicts_equal_state_by_state_the_verdicts_judged_without_the_library_or_the_prefix_split assertion=recording_b.blocks_already_judged==prefix_blocks FAIL left=9 right=10
```

对应用例里的断言：「"前缀的块全部复用：{:?}",」（`/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:325`）、「"前缀的点要么复用，要么段大过展开上限没有状态可复用：{line}"」（同文件 `/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:336`）、「"前缀的块照样复用、尾巴一块都不复用"」（`/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:760`，第 797 行同一句）、「"开着复用时 B 的前缀全部复用、尾巴一块都不复用：{:?}",」（`/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:601`）、「"前缀第 {index} 个点够不着覆盖写那一版，判器输入相同"」（`/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:813`）。也就是说合入后这份测试文件里有五条用例推的会红（三条这一轮新加的、两条原有的），不等提交时的变异整表。实现员的报告原话是「前缀共享：两流用例里前缀的点够不着覆盖写那一版（txg 4），被判的根槽写对它们都在「之后」，前缀写表相同 ⇒ 摘要相同、照样复用。原用例「前缀的块全部复用」那条断言照实不变。」（`/tmp/claude-1000/impl-code-r1-crash-amp/report.md:51`），第一个点不在这句的射程里。

判器真读到的相同：第一个点身份、复用键相同而计划哈希不同时，两边逐态现判（`j1_diff` 的 `J1SPLIT` 行，`outputs/j1_diff-6.out`）：

```
J1SPLIT pair=two_flows same_identity_and_reuse_key_but_different_plan_hash_points=1 states_compared=8 judge_states_differing=0 points=["acquire_instance_and_warm_up.system_configuration_slot.1"]
```

到 D 与到卸载两条（同一份库里先后跑的两项）展开 16 时同样（`outputs/j1_diff-16-rollback.out`，这一行里另一个点是 D 的根槽写那一点，见第三节）：

```
J1SPLIT pair=rollback_vs_unmount same_identity_and_reuse_key_but_different_plan_hash_points=2 states_compared=9 judge_states_differing=0 points=["acquire_instance_and_warm_up.system_configuration_slot.1", "roll_back_by_a_forward_publish.root_record_fua.1", "roll_back_by_a_forward_publish.system_configuration_slot.1(states 8 vs 9)"]
```

代价：每条流的第一个点在别的流已判过之后还要重判；这一点管一段 2 次系统配置槽写，两次都取三态，8 态，展开到多深都是 8 态，代价小；要紧的是用例红、以及判决「前缀共享在判器输入真相同时保留」那一句对第一个点不成立。

### 四句

- 分不分辨：分得开。同一对流，第一个点判器真读到的逐态相同（8 态，0 态不同），块键不同；其余前缀点块键相同。
- 系统看不看得到判别它的东西：看得到。基镜像在 `CrashFlow::base` 里，基镜像里的根（带根 magic 的扇区）在算键那一刻读得到；`crash_facts.rs` 的 `base_roots` 就是这么读的。
- 满足判据字面的哪一句：本轮正文第五节第一条后半「判器输入相同而块键不同（前缀共享丢了而判决说保留）⇒ 打中 J1」。上游条款：「- 状态空间按共享前缀组织：几条流、几个项共有的前缀只判一次，后面的接着它判。」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:60`）。
- 跑前条款给的改法在这几格上还中不中：本轮正文没给改法。第一轮判决给的「判器读到的每一样输入都进那一块的键…前缀共享在判器输入真相同时保留」两半里，后一半在这一格不成立。

### 我提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修「第一个点块键不同」那一格 | 修「五条用例红」那一格 | 量过还是推的 |
|---|---|---|---|
| B0：点之前没有根槽写、又没有没做完的发布时，够得着的版本按基镜像里的根截（基镜像里带根 magic 的扇区，取 (txg，实例) 最大的那一条），不再整张表都进（`model/patch_tree_fix.py`，仓副本里按环境变量开） | 修：七条用例的断言全部 PASS（`outputs/r1_tests-fix-base-root-bound.out`，M3 那一条最后一个断言除外，见 J3 节） | 修：同上，`left=10 right=10` | 量过（草稿副本，展开 6） |
| B1：同一情形下版本一行都不进（写成 `versions_up_to=the_base_image_root`），依据第 401 行那句前提 | 推的修 | 推的修 | 推的；前提「基镜像只有 mkfs 第 0 代根」没有代码核，基镜像不是 mkfs 的流（今天只有单点流 `one_crash_point_over_every_write` 那几项）上会错 |

B0 下展开 6 的 `j1_diff` 没有重跑（判器输入只在第一个点变，第一个点逐态相同已量）；B0 让第一个点在只差版本表的两条流间重新共键，块键相同的点判定逐态相同这一判我只在冻结副本原样上量了（下一节），B0 下没量。

推翻条件：在入库装置上跑两流用例那五条，第 325、336、601、760、797、813 行的断言都过，这一格就不成立。
## J1 第二节：块键相同的点判定是不是逐态相同（没打中）

判据第一条前半要的是「判器输入不同而块键相同」。我没去逐样核判器读了什么，直接量结局：两条流里身份、复用键、计划哈希都相同的点（块键相同、同一判法版本下会互相复用），各自逐态现判，比 `red_items`；第 ② 段核对位也比（`crash_facts::verify_states_from_facts`，同一张事实表的 CPU 参照）。只要有一态不同就是打中。

先看两个推的前提今天有没有流不满足（`j1_flows`，`outputs/j1_flows-6.out`）：两流用例的 A、B 与 M3 那一对、到 D、到卸载四种流，根槽写 txg 都沿写表严格递增（到卸载那条 19 条根槽写，txg 1–19，两块盘轮着落），`MayAllBeRead` 那一支在今天的流上一次都走不到；基镜像都是 mkfs 之后那一份。原样：

```
J1FLOW name=unmount writes=477 segments=78 points=78 root_writes=19 txgs_strictly_increase=true expand_up_to=6 states=261 judged_root_index=474
J1FLOW name=rollback writes=191 segments=36 points=36 root_writes=9 txgs_strictly_increase=true expand_up_to=6 states=125 judged_root_index=188
```

对（左边那条判过的点存着、右边逐点比）：两流 A 对 B；到卸载对到 D（登记表里同一份库先后跑的两项，前 36 个点同身份）；到卸载对它自己的七种「只改点之后」的变体——D 之后第一次单元写、journal 记录写、根槽写（实例表指针里一个字节，根身份不动）、系统配置槽写各改一个字节，最后一条根槽写的 txg 加 100（仍严格递增），txg 10 起的各版内容改坏，被判的根槽写挪到 txg 18 那一条。

| 展开 | 对（`pair=`） | 同键且有状态的点 | 比的态数 | 判定不同 | 核对位不同 | 输出 |
|---|---|---|---|---|---|---|
| 6 | two_flows | 9 | 36 | 0 | 0 | `outputs/j1_diff-6.out` |
| 6 | rollback_vs_unmount | 26 | 107 | 0 | 0 | `outputs/j1_diff-6.out` |
| 6 | later_unit | 29 | 124 | 0 | 0 | `outputs/j1_diff-6.out` |
| 6 | later_journal | 29 | 124 | 0 | 0 | `outputs/j1_diff-6.out` |
| 6 | later_root_payload | 29 | 124 | 0 | 0 | `outputs/j1_diff-6.out` |
| 6 | later_sysconfig | 31 | 128 | 0 | 0 | `outputs/j1_diff-6.out` |
| 6 | last_root_txg_jump | 58 | 248 | 0 | 0 | `outputs/j1_diff-6.out` |
| 6 | versions_after_d | 28 | 116 | 0 | 0 | `outputs/j1_diff-6.out` |
| 6 | judged_root_moved | 57 | 248 | 0 | 0 | `outputs/j1_diff-6.out` |
| 16 | rollback_vs_unmount | 28 | 131177 | 0 | 0 | `outputs/j1_diff-16-rollback.out` |
| 16 | later_unit | 31 | 131194 | 0 | 0 | `outputs/j1_diff-16-A.out` |
| 16 | later_journal | 31 | 131194 | 0 | 0 | `outputs/j1_diff-16-A.out` |
| 16 | later_root_payload | 31 | 131194 | 0 | 0 | `outputs/j1_diff-16-A.out` |
| 16 | later_sysconfig | 33 | 131198 | 0 | 0 | `outputs/j1_diff-16-A2.out` |
| 16 | last_root_txg_jump | 63 | 327923 | 0 | 0 | `outputs/j1_diff-16-B.out` |

取样范围：展开 6 时各对的全部同键点；展开 16 时到 D 对到卸载与五种变体（上表），每批不超过约 10⁶ 态。展开 16 的「txg 10 起各版内容改坏」「被判的根槽写挪到 txg 18」两种（第 C 批）没跑完：后台时限到了被停，只有展开 6 的数；批次与挂钟见「这条腿自己的限度」。展开 16 把可写挂载暖机里两段 16 次写的索引节点写、抬 F 与卸载里各两段 16 次写展开了，「一段写只落一部分」那一类状态在里面；18、20、24、28 次写的单元写段仍不展开（展开 24 一段就是 2²⁴ 个态，超过这一腿的量）。

结论：块键相同的点，判定与核对位逐态相同，0 态不同。两个推的前提在今天的流上都成立，严格递增那一支是真走的那一支；「不严格递增退成整段写表」那一支在今天的流上走不到，也没有用例或变异罩着它（`crates/singlefs-checker-tier/tests/` 里搜 `strictly_increase`、`MayAllBeRead` 0 份文件；`crates/mutations.tsv` 里提到「严格递增」的两行在 `crates/singlefs-checker/src/walk.rs`，与它无关）——把「严格递增」那一判改成恒真（`let txgs_of_the_root_writes_strictly_increase = root_writes`，`/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:451` 那一句的值），推的没有一条用例会红；这是一处会决定复用对错、而没有自证的分支，我只记线索，没造一条不严格递增的流去证它非要不可。

推翻条件：在入库装置上找到一对块键相同、某个状态判定不同的点。

## J1 第三节：别的「判器真读到的相同而块键不同」（量了，代价小，不另算打中）

同一批 `J1SPLIT` 行（身份、复用键相同而计划哈希不同的点，两边逐态现判）：

- 被判的根槽写落在点里与落在点后：到 D 那条把 D 的根槽写当被判的那一条，到卸载那条把卸载最后一条当，D 的根槽写那一点两边计划哈希不同（判器输入只差「judged_root_slot_write=inside_the_point_at_write_0」与「never_persisted」），1 态、判定相同。代码自己写明被判的根槽写「不进逐项判定；照样进键」，所以按字面判器输入确实不同，不算打中；代价每对流 1 个点。
- 没做完的那次发布的根槽写只改了实例表指针里一个字节：管它前面两个点（单元写那一点与 journal 记录那一点）块键分开，3 态判定相同。这一行罩的是「恢复落到由记录重建的那一版时读它的实例表」，在这两个点的状态里那一版的单元都已落、被抛弃的判不到，判定碰巧相同，不说明这一行多余。
- 改坏 txg 10 起的各版内容：分开的点里有 136 态判定确实不同，键分开是对的。
## J2：改法本身的新缺口

### G1：PCI 判定在几种机器上的结局（打中一格，余下照设计）

判定纯函数与 PCI 读法原样从冻结副本的拷贝调（`g1_outcomes`）：PCI 设备表喂假的 sysfs 目录，Vulkan 枚举与起好的卡数按场景给。`outputs/g1_outcomes-live.out`、`outputs/g1_outcomes-icd-missing.out` 逐行摘成下表（原因串太长没抄，判定名原样）：

| 场景（`G1 scenario=` / `G1LIVE`） | PCI 读法的结果 | 起好的卡 | 判定 |
|---|---|---|---|
| vm_or_container_without_sys_bus_pci_and_no_gpu | ListingUnreadable | 0 | PciListingUnreadableWhichC620LeavesUndecided |
| container_whose_sysfs_lists_no_pci_device | 数到 0 个 | 0 | NotRunNoGpuAdapter |
| vm_with_virtio_gpu_only | 数到 0 个 | 0 | NotRunNoGpuAdapter |
| no_gpu_host_with_one_pci_device_whose_class_is_unreadable | ListingUnreadable | 0 | PciListingUnreadableWhichC620LeavesUndecided |
| container_on_a_gpu_host_without_the_nvidia_icd | 数到 2 个 | 0 | NvidiaCardOnPciButVulkanEnumeratesNone |
| vm_guest_whose_nvidia_vgpu_vulkan_reports_virtual | 数到 1 个 | 0 | NvidiaCardOnPciButVulkanEnumeratesNone |
| datacenter_card_class_3d_controller_driver_missing | 数到 1 个 | 0 | NvidiaCardOnPciButVulkanEnumeratesNone |
| two_cards_on_pci_one_in_vulkan_one_started | 数到 2 个 | 1 | RunOnTheUsableCards |
| two_cards_on_pci_two_in_vulkan_one_started | 数到 2 个 | 1 | RunOnTheUsableCards |
| one_card_healthy | 数到 1 个 | 1 | RunOnTheUsableCards |
| 本机真枚举，VK_ICD_FILENAMES=None | 数到 2 个 | 1 | RunOnTheUsableCards |
| 本机真枚举，VK_ICD_FILENAMES=Some("/nonexistent/nvidia_icd.json") | 数到 2 个 | 0 | NvidiaCardOnPciButVulkanEnumeratesNone |

打中的一格：没有 `/sys/bus/pci` 的虚机或容器（微虚机只用 virtio-mmio、gVisor 一类沙箱）、本来就没有显卡，判 `PciListingUnreadableWhichC620LeavesUndecided`，取卡入口对它 panic：每条 GPU 用例判红。实现员把这一支照「C620 没定」先判红：「/// Vulkan 枚举里没有 NVIDIA 独显，PCI 设备表又读不出：C620 没定这时算不算「没卡」，第一版不判本次未跑、判红。」（`/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:595`），并在报告里交主 agent 定（`/tmp/claude-1000/impl-code-r1-gpu/report.md:24`「PCI 设备表读不出时怎么判，C620 没写」）。上游的条款是「要特定硬件的那几条，在没有那种硬件的机器上报「本次未跑」。」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:53`）。同一支还罩着「PCI 设备表里只要一个设备的 `class` 读不出」：「(Err(what), Ok(_) | Err(_)) | (Ok(_), Err(what)) => return unreadable(what),」（`/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:721`）——没卡的机器上一个热插拔中途消失的设备（目录列出来、读 `class` 时已经没了）也让每条 GPU 用例判红（表里 `no_gpu_host_with_one_pci_device_whose_class_is_unreadable` 那一行）。

四句：
- 分不分辨：分得开「没卡」与「有卡驱动坏」两种机器，判定把「没卡而读不出 PCI」判成与「有卡驱动坏」同一个结局（红）。
- 系统看不看得到判别它的东西：一半看得到。`/sys/bus/pci` 整个不在时内核没有 PCI 总线，NVIDIA 独显插不上（推的）；只有一个设备读不出时，其余设备的 vendor、class 读得到，NVIDIA 显示控制器数得出来，不必整份判读不出。
- 满足哪一句：本轮正文第五节第二条「造出一个输入让改后的判法给错结局（该红不红、该绿不绿、该报本次未跑而判绿…）」——这里是反向的「该报本次未跑而判红」，字面列的三种里没有它，按「给错结局」那一句的总括归；主 agent 若只认括号里列的三种，这一格归「可议」。
- 改法（只在推、没实现，被攻过零轮）：`/sys/bus/pci` 不在（`read_dir` 报 NotFound）且 Vulkan 没有 NVIDIA 独显，判本次未跑；目录在而个别设备读不出的，跳过读不出的那几个、只要数到的 NVIDIA 显示控制器有一个就判红、一个都没有就照「读不出」交主 agent 定的那一支。这一改把第 1676 行变异「PCI 设备表里一项读不出就跳过（不算读不出）」想守的那一判改掉了，那一行要跟着改。

照设计、不算打中的几格（结局见上表）：容器挂在有卡的宿主上、没给 NVIDIA 的 Vulkan 驱动（sysfs 不分命名空间，看得到宿主的卡）判红，C620 要的就是「PCI 上有卡而 Vulkan 枚举不出判红」；虚机里 NVIDIA vGPU 若被 Vulkan 报成虚拟设备也判红（同一句；vGPU 实际报哪一种我没核）；两张卡在 PCI 上、只起来一张，照跑在起来的那一张上、另一张不报（本机就是这样：Vulkan 枚举出两张 NVIDIA 独显、起好一张，表里倒数第二行；另一张被本地模型服务占着显存，推的）——上游「配置了几台机器就分到几台」管的是配置里登记的卡，取卡判定今天不读显卡配置，这一格我只记线索。

### L1：按行相加会不会重复计（没打中）

读的是主仓工作区里此刻的 `research/scripts/crash-amplification.sh`（不在冻结副本里；它已经按流分组、每组只取一次）。按组取的三种：有先判着的行只取那一行；没有就取导入行之外各行之和；只排计划的 `CRASH_AMPLIFICATION_PLACEMENT` 行不取。我试的形状：单机叫停那一趟（组里一行，取它录入时的数，对）；双机先判着、两份、导入（取先判着那一行，对）；双机没先判着、两份、导入（两份录入时各数自己那一份，块不相交，相加对；导入行不取）；固定脚本那两项用例跑第二趟只为断言、不打第二行（读 `crash_amplification_fixed_script_through_rollback.rs` 末段，对）；多流的项每条流 `flow=` 名字不同、各自一组。没造出重复计的行序列。读代码，没跑脚本（它是重型入口）。

### S2：「复现这一段」只判新发现那一支照抄还复现不复现（没打中）

读的：重放这一整批的那一行带这一批自己的那一组断言，复现这一段（种子数 1）一律写只判新发现；出红的用例与大档入口都只调同一个 `judge_crash_injection_report`，小快档、快档、回退取样三条用例的函数体在它之外没有别的断言（读 `crash_injection_campaign.rs` 三条用例的函数体）。复现这一段只在有新发现时打，照抄它跑大档判的正是新发现那一条。没跑：要复现一批崩溃注入得起 checker 档的入口或照第一轮的原型重建一份改坏的副本，这一轮没做。

### H9：几种组合下的结局（没打中）

读 `computation_routes` 与 `run_from_environment` 第 ①② 段那两处 `match`：登记了而没卡、开了 GPU 而没编进特性、开了 GPU 而一类都没登记，三处都报错退出，不退回 CPU；GPU 跑法下没登记 `crash_verify_gpu`，第 ② 段在 CPU 上核，核对表按 CPU 参照的代码摘要另存，第 ③ 段照这一趟的核对代码读，不会拿 GPU 那一版的核对块当这一版的。登记了 `crash_judge_gpu` 又登记 `gpu_unit_checks` 时后者不起作用、也不报错（实现员报告「停下」第 5 条已写明）。读代码，没跑。

### 同族环境变量的共用解析：线程数设 0 现在 panic（没打中）

全仓（`research/`、`.claude/`、`scripts/`，不含 `research/prompts/`、`research/results/`）搜 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`、`SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`、`SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`：除了 `crates/` 里那两份测试文件与一份 records，没有脚本设它们，没有调用方拿 0 或空串当「取默认」。线索（不在这一格的射程）：`research/scripts/capped.sh` 设的线程变量里没有 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`（「# SINGLEFS_BAD_DISK_INPUT_WORKER_THREADS、SINGLEFS_RANDOM_HISTORY_THREADS，以及通用的 SINGLEFS_THREAD_CAP」，`research/scripts/capped.sh:16`），而这两份对拍没设时取本机并行度（「.unwrap_or_else(|| std::thread::available_parallelism().unwrap_or(std::num::NonZeroUsize::MIN))」，`/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:117`），派发给的线程上限罩不住它们。

### 线索：K5 那份「一趟读的环境变量表」的自证只扫字面串

`every_environment_variable_this_file_reads_by_name_is_in_the_list_a_run_reads` 只认 `env::var("…")` 这种字面串：「for opening in ["env::var(\"", "env::var_os(\""] {」（`/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:2950`）。经常量读的（展开上限、GPU 计算类）今天恰好都在表里；一趟经别的文件读的不在表里也扫不到，例如 GPU 判器的「std::env::var("SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME").is_ok_and(|value| value == "1");」（`/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:481`）。它只改派活次序、不改判定，所以不算打中；归 J4（辩方的格），我不判。
## J3：变异表

### 第 1587 行与 M3 新用例：「改坏根槽写的 magic 之后那个点有状态判红」不成立（打中）

用例的造法：「let mut twin = record_flow("root-write-twin-flipped", &content_of(11, 4100), true);」（`/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:863`），改坏的是「WrittenContents::Bytes(bytes) => bytes[0] ^= 0x01,」（同文件 `/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:866`）；最后一条断言要这一对在管根槽写的那个点上判得不同：「"这一对在管根槽写的那个点上判得不同：根槽写改坏的那一条有红"」（`/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:911`）。实现员的报告把它记成推的：「8. **M3 用例能不能红取决于一处推断**：改坏根槽写 magic 之后那个点的状态要判红；推断错了，用例的非空断言当场红（不会悄悄绿），看到时把改坏的字节换一处（例如根记录里的树表指针）。」（`/tmp/claude-1000/impl-code-r1-crash-amp/report.md:147`）。

量到的（`m3_twin`，展开 6 同用例）：管根槽写的那个点 `publish_overwrite.root_record_fua.1` 只管一段 1 次写（根槽写本身），真子集只有「没落」那一个状态；那个状态里根槽写的字节不在盘上，改坏哪个字节都一样。`outputs/m3_twin.out` 第 15、16 行原样：

```
M3POINT index=13 name=publish_overwrite.root_record_fua.1 owns_the_flipped_root_write=true states=1 same_identity=false same_reuse_key=false same_judge_inputs=true same_plan_hash=true states_judged_differently=0
M3POINT index=14 name=publish_overwrite.system_configuration_slot.1 owns_the_flipped_root_write=false states=9 same_identity=false same_reuse_key=false same_judge_inputs=false same_plan_hash=false states_judged_differently=9
```

改坏的那一条的红（`journal_ignored_oracle:recovered_to_an_older_root`）全落在下一个点（系统配置槽那一点，根槽写已落）；那个点的判器输入里「这个点之前的写表」已经罩着这次根槽写的内容，复用键之外也分得开。`r1_tests` 照用例的断言重算：「owner_point_judged_differently FAIL healthy=[Some([])] twin=[Some([])]」（`outputs/r1_tests-as-merged.out` 第 20 行，B0 改法下同一行照样 FAIL）。

推出来的两件事：
- 这条用例在冻结副本原样上就红（最后一条断言），第 1587 行的证红没有意义：基线已经红。
- 变异「复用键不罩写表」下这条用例照样红，但红在「"这一步的写表不同，复用键不同"」（`/tmp/claude-1000/test-script-align-code-r2/tree/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:897`）那条直接比键的断言上，不是红在读回与现判不同上：管根槽写的那个点在变异下块键相同，可两边判定本来逐态相同（1 态都是绿），读回照样对。

换一处改坏也不行（`m3_candidates`，同一次覆盖写尾巴里每次写各改 5 个位置的一个字节，共 145 种，`outputs/m3_candidates.out` 末行原样）：

```
M3CAND_TOTAL tried=145 qualifying=0
```

「只剩复用键分得开」的有 145 种中的大多数（`only_reuse_key_separates=true`），但在管那次写的点上判得不同的一种都没有：单元写那一点展开 6 时 0 个状态，journal 记录、根槽写、系统配置槽写那几点的状态里改坏的字节要么不在盘上、要么恢复与 checker 照样判绿。实现员报告给的退路「把改坏的字节换一处（例如根记录里的树表指针）」也在这 145 种里（根槽写偏移 40 落在树表指针里），同样 0 态不同。

四句：
- 分不分辨：分得开「推断成立」与「不成立」：管根槽写的那个点 1 态、两边都绿。
- 系统看不看得到：看得到，段长 1、展开方式在计划里。
- 满足哪一句：本轮正文第五节第三条「点名的测试跑不到或推不出会红」里的「推不出会红」，基线红是同一件事的另一面。
- 跑前条款的改法：没给。我提的（只在我的模型上量过、被攻过零轮）：这一行不点名「读回与现判」的用例，改点名 `crash_identity.rs` 里按属性直接比复用键的库内用例（第一轮报告提过的 `each_attribute_changes_the_reuse_key` 一路），并把 M3 那条用例的最后一条断言删掉或改成核「下一个点上判得不同」；推的，没实现。要一对「只有写表分得开、又判得不同」的流，展开 6 下这条流的尾巴里找不到（145 种 0 种），要么展开到单元写那一段（24 次写），要么换一条段内有两次以上可撕或可落的写、且落一半就判红的流；推的。

### 点名的测试在基线上就红的别的行（打中，与 J1 第一节同一个病根）

照 J1 第一节，下面这几行点名的用例在冻结副本原样上推的就红（原型重算的是同一组断言），证红没有意义；B0 改法下都回绿（`outputs/r1_tests-fix-base-root-bound.out`）：

| 行 | 变异名 | 点名的用例 | 基线上红在哪 |
|---|---|---|---|
| 1432–1434 | 崩溃放量 前缀 记录器丢掉设点的文件 / 路径段带上点在流里的序号 / 块键带上路径 | `two_flows_sharing_their_prefix_reuse_its_verdicts_and_fork_only_at_the_tail` | 第 325 行，`left=9 right=10` |
| 1588 | 判定存储 判定块的键不接判法版本 | `reused_verdicts_equal_state_by_state_the_verdicts_judged_without_the_library_or_the_prefix_split` | 第 601 行，`left=9 right=10` |
| 1680、1681 | 崩溃放量 版本表不进判器输入（健康的后进／写错的后进） | 两个次序各一条 | 第 760、797 行，`left=9 right=10` |
| 1682 | 崩溃放量 版本表不按够得着的截 | `a_version_the_prefix_cannot_reach_leaves_the_judge_inputs_of_the_prefix_alone` | 第 813 行，第 0 个点 |

（行号是冻结副本 `crates/mutations.tsv` 的行号；1432–1434、1588 不是这一轮新加的，它们点名的用例因 R1 的改法在基线上变红。）

### 静态核（53 行，没打中）

这一轮四份报告写明的行（第 1674–1721 行 48 行，加改过的第 1585、1587、1590、1595、1609 行）用 `model/k3check.py` 逐行核：原文在文件里恰好命中一次、替换文与原文不同、点名的测试恰好一处 `fn` 定义、定义落在参数点名的目标里（`--test` / `--lib` / `--bin`）、筛选词是测试路径的子串、参数里的特性罩得住文件头与用例上的 `cfg`、用例没挂 `#[ignore]`。53 行全过；第 1709、1710 行点名的同名用例在两个包里各有一处定义，各自的参数指到自己那一份，不算错（`outputs/k3check.out`）。第 1645–1673 行是别的会话同期加的，正文说不判，没看。

### C604 没留变异那一条的理由

站得住一半：本机两张 NVIDIA 卡、PCI 上读得到（G1 节 `G1LIVE` 行），改回 `.expect` 在本机走同一条「起好卡、照跑」的路，变异整表在本机记「没红」。站不住的一半：上游要「项的驱动、设备、时钟、随机源都能从参数或环境变量换成假的」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:38`），取卡入口今天直接读真的 Vulkan 枚举与 `/sys/bus/pci/devices`，换不成假的，所以才留不了变异；理由该写成「换不掉的那一处」并进文件头，而不是「有卡的机器上走同一条路」。推的，没造假驱动量。
## 没打中的形状

| 攻的地方 | 试过的形状 | 取样范围 |
|---|---|---|
| J1 块键相同而判定不同 | 两流 A/B；到 D 对到卸载；到卸载对七种只改点之后的变体（见 J1 第二节） | 展开 6 全部同键点；展开 16 见 J1 第二节的表 |
| J1 两个推的前提 | 四种流的根槽写 txg 序列与基镜像 | 4 种流，根槽写 4、4、9、19 条 |
| J1 第 ② 段核对块共用块键 | 同上各对，同键点的核对位逐态比 | 同上，0 态不同 |
| J2 L1 汇总 | 单机叫停；双机先判着＋两份＋导入；双机两份＋导入；固定脚本第二趟；多流各自一组 | 读主仓工作区里此刻的脚本，没跑 |
| J2 S2 | 三条小档用例的函数体在判定函数之外有没有别的断言；复现这一段什么时候打、照什么判 | 读代码，没跑 |
| J2 H9 | 没开 GPU、开了没登记、开了没编特性、登记了没卡、GPU 跑法下没登记核对、判器上卡又登记单元校验和 | 读代码，没跑 |
| J2 线程数 0 | 全仓脚本里谁设这三个变量 | grep，0 处 |
| J2 G1 | 见 G1 节的表 | 10 种合成场景、2 种真枚举 |
| J3 静态核 | 53 行 | 全部 |
| J3 M3 的退路 | 覆盖写尾巴里每次写 5 个位置各改一个字节 | 145 种 |

## 这条腿自己的限度

- 所有数都是草稿副本上原型跑的，不是入库装置。流照用例的造法拷来，盘换成内存稀疏盘、重开按录制流在内存里重建（`model/make_common.py`）；崩溃点的文件标签是原型自己的路径（`src/common_crash_points/mod.rs`），与仓里的不同、两条流之间一致。两流用例那七条是在原型里照断言重算，没跑用例本身（checker 档）。
- 仓副本里加了两处口子（`model/patch_tree.py` 只把判器输入原文拿出来、不改摘要；`model/patch_tree_fix.py` 是 B0，环境变量开，默认关）。原样那几趟的数在口子开关关着时跑，判器输入摘要与冻结副本同一段文字的 SHA-256。
- 展开 16 是这一腿量得起的最深一档；18、20、24、28 次写的单元写段没展开，「单元写只落一部分而判器读到点之后的写」那一类只量到 16 次写的索引节点段那几处。B0 下没重跑 `j1_diff`。
- G1 的「没有 `/sys/bus/pci`」「一个设备读不出 class」是假的 sysfs 目录喂同一个读法；没在真的微虚机、沙箱里跑。vGPU 在 Vulkan 里报哪一种设备我没核。
- J2 的 L1、S2、H9 只读代码；L1 读的是主仓工作区里此刻的入口脚本（不在冻结副本里，sha256 见开头「复跑」一节），它在别的会话手里还在改。
- 状态量（`closed_form_state_count` 是全展开的闭式数，这一腿只按展开上限跑）：两流 33554462、到 D 51773503、到卸载 1662648449；这一腿实跑的状态数见各输出的 `J1DIFF_TOTAL` 与「批」那一段，合起来远低于 10⁷。
- 批次与挂钟（`j1_diff`，线程 12，一批一条，经内存包装）：先跑一小段（展开 16、只跑到 D 对到卸载）当估时用：262 372 态、4 分 45 秒，按它估余下约 190 万态要 33 分钟左右，没超过 40 分钟，没缩。正式跑分四批：A（四种「点之后改一个字节」）撞了后台默认时限（30 分钟）被停，停时前三种已打出结果行（`outputs/j1_diff-16-A.out`），时间文件是空的；第四种另起一批 A2，37 分 30 秒、786 878 态；B（最后一条根槽写 txg 加 100）35 分 02 秒、786 922 态；C（版本改坏、被判的根槽写挪位）跑到第 18 分钟左右撞了那一条后台命令的 90 分钟时限被停，没有结果行，没再起。实际速度只有估时的一半上下（同机别的会话在编 checker 档），估时偏乐观。每批状态数都不超过约 10⁶；这一腿实跑合起来约 250 万态（展开 6 那几趟 2 112 态可略），低于 10⁷。
- 内存包装退过几次非 0：编原型第二次用 16G 退 251（超过峰值表里实测峰值两倍），改 `auto` 又因峰值表里只有一次空编而撞上限退 250（oom-kill）；之后编译用 12G 加理由。原型第一次跑 `r1_tests` 因没设 `CARGO` 在判法摘要处 panic（退 101），设上重跑；有两次同会话已有一条在跑退 252，等它跑完再起。退非 0 的那几次输出都不算。

## 没做什么

- 没跑 checker 档包的任何测试，也没跑门禁阶段；两流用例五条推的会红、M3 用例基线红，都是原型照断言重算的，要在入库装置上跑那份测试文件坐实。
- 没在入库装置上重做任何数；B0 改法、M3 的改法、G1 的改法都只在我的模型上量过或只是推的，被攻过零轮。B0 下没重跑 `j1_diff`。
- 展开 16 的第 C 批（版本改坏、被判的根槽写挪位）撞时限被停，没再起；那两种只有展开 6 的数。
- S2、H9、L1 只读代码，没跑；S2 没照第一轮那样造改坏的副本重放一批崩溃注入。
- 没判 J4、J5（辩方的格）；读到的越格线索（K5 环境变量表只扫字面串、`capped.sh` 罩不住两份对拍的线程变量）只记下。
- 第 1645–1673 行变异（别的会话同期加的）、`crash_identity.rs`、`crash_injection.rs`、`crash_judge_dispatch.rs` 里正文说混着别的会话改动的那几处，没判。

## 草稿与删除

- 删了：草稿副本 `/tmp/claude-1000/test-script-align-code-r2-opus/tree`（仓副本）与 `/tmp/claude-1000/test-script-align-code-r2-opus/proto`（原型包连同它的编译目录 `target`），删前 `du -sh` 各是 13M 与 783M；复跑材料都在模型目录里。
- 留着：草稿目录里的 `runs/`（原样输出、日志与 `.err`，`outputs/` 是它的拷贝）、`report/`（这份报告的分段底稿）、`k3/`、`model/`（模型目录的底稿）、`progress.md`、几份 `build*.log`；都不是仓副本或编译目录。
