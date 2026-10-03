# test-script-align-code-r3 攻方腿（Opus）报告：L1、L4，兼 L2（2026-10-02）

被判的是冻结副本 `/tmp/claude-1000/test-script-align-code-r3/tree/` 里的代码。下文引代码写冻结副本的绝对路径与行号。量到的数都是草稿副本（`/tmp/claude-1000/test-script-align-code-r3-opus/tree`，加了 r2 攻方的看判器输入原文的口子与本腿的几个环境变量开关，默认关、与冻结副本同义）上原型跑的，不是入库装置的数。checker 档包的测试一条没跑：被点名的用例是在原型里照它们的造法与断言（按原次序）重算的。

## 复跑

```
bash research/prompts/test-script-align-code-r3-opus-model/rerun.sh <空的工作目录> [/tmp/claude-1000/test-script-align-code-r3/tree]
```

它从冻结副本拷一份仓、依次打 `research/prompts/test-script-align-code-r2-opus-model/patch_tree.py`（看判器输入原文）、本目录的 `patch_mutations.py`（变异表第 1587、1731、1732、1733 行各一个环境变量开关 `R3_MUT_<行号>`）、`patch_fix.py`（本腿提的改法 B0R，开关 `R3_FIX_B0_RECOVER_THE_BASE`），按变异表第 1741、1729、1730 行造只改原文的副本（`make_text_variants.py`），编原型包 `proto`，一条一条跑，原样输出落进 `<工作目录>/runs/`。全部经 `research/scripts/run-with-memory-cap.sh`、`capped.sh 12`。展开 16 那一趟约 4.5 分钟，`R3_RERUN_QUICK=1` 跳过。harness 档那五行变异的证红另起，命令写在 `rerun.sh` 末尾的注释里。本腿实跑的原样输出在模型目录 `outputs/` 下（下文 `outputs/…` 都指那里）。

模型目录各文件 sha256（`SHA256SUMS` 原样）：

```
4dfc161ad863541caf51aa17fc0327aa727f26d75e4734a342a78c389856c634  ./make_common.py
871263025a04ca4326e5bbb190c55c9409a0f069b15a20823476a638f12d898e  ./make_text_variants.py
4de0cbef5285b4a1e3b8ccdf5cc6d0a6274ef3144c2ca77103e6870652d19258  ./outputs/b0_crashed_base-6-again.out
af511cac342c4bb06d3458e4ff61920777019f144ff2d006ca48c92c19430979  ./outputs/b0_crashed_base-6-fixB0R.out
ddd69dabe370fee2f8fc4ad14094f991efd6b26b61a2602daaa3952b307b19e4  ./outputs/b0_crashed_base-6-mut1731.out
4de0cbef5285b4a1e3b8ccdf5cc6d0a6274ef3144c2ca77103e6870652d19258  ./outputs/b0_crashed_base-6.out
5c573f6132655fa311728d68afab3d0f71695c27fab6b2681119f6013d3077c0  ./outputs/dep_check.out
546b39f8668d15a66b98d9bf0def33a54ba6c4fa36bacbabbdeec348af72a98e  ./outputs/inscope.tsv
2e94a4d9d4aa688bc589482ded38ee871d4ae1095365f5c3ec5247e217bdc178  ./outputs/j1_diff-16-rollback.out
f6b9cf4e2726b69b14260d93b31cbd3518547706d28bc93e6b7a476640e3a636  ./outputs/j1_diff-16-rollback.time
a272b9c7eea377ddb0336d5259383bd0caacabf92e14966e1642e649fe266008  ./outputs/j1_diff-6.out
cf541ded6e4a4b359f26365d85dde9d71cdb43768c89b3961cbaaddaf4f0632d  ./outputs/k3check.out
20f41b5beaa3322e64b967f7f64e0cad67d50d7aa6f35274d72e69bb62a6e0ea  ./outputs/l2_matrix.out
1f6f477502508251e76bcaefed407fd296164c1267cef5f696ed7aa6ed0f56c8  ./outputs/named_tests-baseline.out
1f6f477502508251e76bcaefed407fd296164c1267cef5f696ed7aa6ed0f56c8  ./outputs/named_tests-fixB0R.out
71204e18bed219c2fa05638295e4ee532f515a855f7be46962bdec8379dbbf4e  ./outputs/named_tests-mut1587.out
3033e5bfbc9c0d2bca07360f0132497c3e796b03bd8ca03ddd7e0bc5c51e7e73  ./outputs/named_tests-mut1731.out
616072a34b0613fb3d673f9ef299a34161e321bb82c70618b6660c8587568bab  ./outputs/named_tests-mut1732.out
85bff6e222a5c5b0d0aca484f41723c2eeaac18ae847421c3f0dd73b44fbea9c  ./outputs/named_tests-mut1733.out
7b3bca5372f760a0bfd01936145a396446a5c1b4e5b4267790408ffb0b1edbb6  ./outputs/prove-red.out
f19e98cd0958c9793ef22d47e754e3f1392f9568ee6c06ee381d309654377daa  ./outputs/scan_check.out
31c597c0e8a0852fd20a757b1d2775eecbd94d7332976240e81dfa788ac5a9f8  ./patch_fix.py
4ec8f16433da7bc1eed77da4f59f069a319dd317b47926f854163a8cf72df402  ./patch_mutations.py
7b21ec68e8ef4503de64b027bcdb9f24bb50b00e62d6a79f120908d927411aa9  ./proto/Cargo.toml
71ced84c71f8c1e43f0cb773752e44939a0aae990f0fac27c30c69c6a632a957  ./proto/src/bin/b0_crashed_base.rs
25adf096b904f827bcd7eaa76e4bbba1bafd90bfb714d1fd28135558d98d74d5  ./proto/src/bin/dep_check.rs
faeac74f2de760506fa997d2d7edda199ead136440380f4459395d9d761f1aea  ./proto/src/bin/j1_diff.rs
3514b9e0c582cc9154c5bda475aa0e33e45cdea8bc7198b7d3a51492989a77c1  ./proto/src/bin/l2_matrix.rs
64bead9b9ad45505e87825c957d3fbd6b31407f45db1c950cc01560d66ca3b85  ./proto/src/bin/named_tests.rs
f9c5a5f47bca469210298b5d3f50b520d11b576ccb3fe1008c67ce6474bcc338  ./proto/src/bin/scan_check.rs
37618da6ddd17397429db6b34bf3222d91090b1737a53f3108d8558b2b083293  ./proto/src/flows.rs
04ed73525268e2c6414acfd2f3acda84aeb01bf8d1a09102dc1d86430ea26bdc  ./proto/src/lib.rs
1fc4f4dabafc90c48730494c58ec68dc690eace16fc79e28ac45affbadfe9356  ./rerun.sh
```

## 各格判定一览

| 格 | 子项 | 判定 | 一句话 |
|---|---|---|---|
| L1 | B0 在基镜像是崩溃状态（记录落了、根没落）的流上 | **打中** | 重开之后第一个点的块键三样都相同，8 态里 8 态判定不同（健康的绿、版本表写错的 `journal_consulted_oracle:wrong_content`）；B0 按基镜像里的根把恢复重建出的那一代截掉了。撤掉 B0（第 1731 行变异）这一对就分得开 |
| L1 | 今天的四种流（mkfs 起头）上块键相同的点 | 没打中 | B0 之后两流第一个点并进同键，展开 6 两流 10 点 44 态、到 D 对到卸载 27 点 115 态，展开 16 到 D 对到卸载 29 点 131185 态，判定与核对位都是 0 态不同 |
| L1 | 判器输入真相同而块键不同 | 没打中（剩下的都是前两轮记过的） | B0 之后两流的「同身份同复用键而计划哈希不同」从 1 点降到 0 点；到 D 对到卸载只剩被判根槽写那一点 |
| L1 | txg 不严格递增那一支的新用例 | 没打中 | 真走到那一支：第 1733 行变异（`<` 换 `<=`）下重算，第 0 个点起「判器输入跟着变」那一条就红 |
| L1 | M3 拆成两条断言 | 没打中 | 第 1587 行变异只红在键那一条（读回那条照样 PASS），第 1732 行只红在读回那一条（读回第 48–56 态不同）；基线全绿 |
| L4 | 42 行（新增 31、改写 11）静态核 | 没打中 | 锚点都恰好命中一次、替换文与原文不同、点名的测试恰好一处定义、筛选词罩得住、特性罩得住、没挂 `#[ignore]`（工具报的几处是它自己认模块路径与多行属性的毛病，逐个对过原文） |
| L4 | 实测会不会红 | 没打中 | 原型重算 1587、1731、1732、1733、1741、1729、1730 七行，harness 档 prove-red 1742、1714、1715、1716、1717 五行，全红在点名的那一条上 |
| L4 | 第 1736 行 | 线索（不算打中） | 只在本机并行度大于 1 时红；推的 |
| L2 | 崩溃注入重放入口里同一种写法的结局 | **打中**（口径两处不一） | 同一个入口：种子、种子数、崩溃点数、短段写数前后带空白照收；每段步数、断言组、盘宽、比重、checker、空间准入带空白就停下；短段写数写 ` none` 停下、报「invalid digit」 |
| L2 | 写 0 | **打中**（可议：主 agent 若认「规模写 0 不算写错」，归可议） | 线程数写 0 一律停下；种子数写 0 照收，那一批跑 0 个崩溃状态，只判清单外失败的那一组判绿 |
| L2 | 非 UTF-8 | 小（措辞不一） | 结局都是停下；故障注入那一份的消息不经共用读法，写法不同 |
| L2 | 依赖判法在新模块下 | 没打中（线索一条） | 第 1741 行改法下 8 条登记用例里 3 条红；入口测试文件自己引 `history` 的那几条看不见库模块也引它（理由只记第一个） |

打中的那两格的改法都只在我的模型上量过或只是推的，被攻过零轮（各节的表里逐格标了）。

## L1 第一节：B0 在基镜像是崩溃状态的流上把恢复重建出的那一代截掉（打中）

### 打中的历史

每一步指到许可它的那一句：

1. 判器输入里「够得着的版本」按三样根里最大的那一条截，其中一样是基镜像里带根 magic 的扇区：「///   落到的根要么是基镜像里本来就有的根（基镜像里带根 magic 的扇区），要么是写表里到这个点为止写过的根槽写，要么是 journal 重放出来的」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:396`），下一行「///   那一版——没做完的那次发布，它的根槽写在这个点之后、是下一条。够得着的版本是（txg，实例）不超过这几样根里最大那一条的行：」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:397`）；代码是「                    .chain(self.newest_root_in_the_base_image)」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:556`）。「journal 重放出来的那一版」只认写表里这个点之后的下一条根槽写；基镜像自己的 journal 里落了而根没落的那次发布，三样里哪一样都不是。
2. 流水线收什么基镜像，类型上不限：「    pub base: &'flow MemoryPool,」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:128`）。实现员报告把 B0 写成对非 mkfs 的基镜像也对：「   不用 B1（版本一行都不进）：攻方说「基镜像只有第 0 代根」这一条没有代码核，B0 按基镜像里真有的根截，基镜像不是 mkfs 的流上也对。」（`/tmp/claude-1000/impl-code-r2-crash-amp/report.md:35`）。崩溃之后照崩溃镜像重开、录进一条新流，是仓里现成的起法：「pub fn crash_state_devices(」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-harness/tests/common/mod.rs:259`）；层 0 的用例里也有不从 mkfs 起的基镜像：「    let base = pool.memory_pool();」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/tests/crash_enumeration_record_spill_over_stream.rs:147`）。
3. 造法（`proto/src/flows.rs` 的 `crashed_base_then_mount`）：两流用例那条流（mkfs → 取号加暖机 → 新池新建文件 txg 3 → 覆盖写 txg 4）崩在覆盖写的 journal 记录落盘之后、根槽写落盘之前，这个崩溃状态当基镜像；照它可写挂载（经 `crash_state_devices` 录进新流），再发一次覆盖写（实例 2）。版本表照实写：(1,3)、(1,4)、重开写出的 (2,5)(2,6)(2,7) 与最后那次 (2,8)。一对流只差 (1,4) 那一版的内容改坏一个字节（R1 那一对的造法）。不用造任何别的故障。
4. 重开之后第一个点只有两次系统配置槽写（之前没有根槽写，也没有单元写与记录写），B0 取基镜像里最大的根 (1,3) 当上界，(1,4) 那一版不进判器输入；可这个点的每个状态里，照 journal 恢复会把基镜像里落了的那条记录施加上去，落到 (1,4)，oracle 按 (1,4) 那一版比内容。

### 量到的（草稿副本上的原型，展开上限 6，盘是内存稀疏盘）

`outputs/b0_crashed_base-6.out` 里的基镜像根那一行、第 0 个点那一行与它下面两行、末行，原样整行：

```
B0CB base_roots=[(0, 0), (1, 3), (1, 2), (1, 1)] cut_root=(1, 4)
B0CBPOINT index=0 name=mount_writable_after_the_crash.system_configuration_slot.1 write_kinds=["system_configuration_slot", "system_configuration_slot"] states=8 same_identity=true same_reuse_key=true same_plan_hash=true versions_up_to_txg=3 instance=1 unfinished_publish_root=none states_judged_differently=8 healthy_red_states=0
B0CBSTATE index=0 offset=0 healthy=[] twin=["journal_consulted_oracle:wrong_content"]
B0CBSTATE index=0 offset=1 healthy=[] twin=["journal_consulted_oracle:wrong_content"]
B0CB_TOTAL same_block_key_points=1 states_compared=8 judge_states_differing_under_the_same_block_key=8
```

对照：同一对流在第 1731 行变异下（基镜像的根不进上界，即撤掉 B0），`outputs/b0_crashed_base-6-mut1731.out` 原样：

```
B0CBPOINT index=0 name=mount_writable_after_the_crash.system_configuration_slot.1 write_kinds=["system_configuration_slot", "system_configuration_slot"] states=8 same_identity=true same_reuse_key=true same_plan_hash=false versions_up_to=every_version unfinished_publish_root=none states_judged_differently=8 healthy_red_states=0
B0CB_TOTAL same_block_key_points=0 states_compared=0 judge_states_differing_under_the_same_block_key=0
```

第 0 个点的身份、复用键、计划哈希三样都相同，块键就相同：块键由这三样与块起点组成，「            input_fingerprint: InputFingerprint(*self.reuse_key.as_bytes()),」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:167`）、「            stream_or_node_name: StreamOrNodeName::new(&self.identity)」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:168`）、「            enumeration_plan_hash: EnumerationPlanHash(self.plan_hash),」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:170`）。同一判法版本下健康的那一条先进库，版本表写错的那一条在这个点上读回健康那一条的 8 态绿，而它现判是 8 态红（推的：这一对没进库跑读回，读回只看块键这一点在 M3 那一节的重算里量过）。第 2、3 个点（重开写下第一条根槽写前后）两条流也判得不同，但那里计划哈希已经分开（`same_plan_hash=false`），不复用，对。健康的那一条 17 个点全绿（每行 `healthy_red_states=0`）。

状态数：这一对每条 57 态（展开 6），全展开的闭式数 17170460；这一趟两次跑合起来不到 500 态。

### 四句

- 分不分辨：分得开。同一对流、同一个点，B0 下块键相同而 8 态判定不同；撤掉 B0（第 1731 行变异）这个点块键分开（`same_plan_hash=false`、`versions_up_to=every_version`），这一格不中。今天登记的四种流都从 mkfs 起、基镜像的 journal 是空的，那几条流上 B0 与撤掉它判得一样（下一节）。
- 系统看不看得到判别它的东西：看得到。基镜像在 `CrashFlow::base` 里，算键那一刻对它照 journal 恢复一遍就知道会落到 (1,4)（下面改法 B0R 就是这么取的）。
- 满足判据字面的哪一句：本轮正文第五节第一条前半「造出一对点块键相同而判定不同 ⇒ 打中 L1」。上游条款：「- 剪掉的只能是两种：键相同、已经判过的；与已判状态等价的。等价关系写成用例钉住。」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:61`）——这一对键相同而不等价。
- 跑前条款给的改法在这一格上还中不中：本轮正文没给改法；第二轮判决给的是「够不着任何根的点按基镜像里的根截版本表（攻方改法 B0，零轮），或别的能让两流共享前缀而判器输入真不同时不共享的截法」，B0 正是打中的那一个，后半句的「别的截法」见下表。

### 改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修「崩溃基镜像上第 0 个点同键异判」那一格 | 保不保两流的前缀共享（今天的流） | 量过还是推的 |
|---|---|---|---|
| B0R：基镜像的上界再并上「对基镜像本身照 journal 恢复（`JournalPolicy::Consult`）实际走的那条根」（`patch_fix.py`，开关 `R3_FIX_B0_RECOVER_THE_BASE`） | 修：第 0 个点上界变成 txg 4，块键分开，`B0CB_TOTAL same_block_key_points=0`（`outputs/b0_crashed_base-6-fixB0R.out`） | 保：三条被点名的用例重算全绿，其中「前缀第 0 个点判器输入相同」那一条照过（`outputs/named_tests-fixB0R.out`，与基线逐字节相同） | 量过（草稿副本，展开 6）；第 0 个点的状态里系统配置槽写落一半会不会让恢复走到比「恢复基镜像」更新的根，我没证，推的不会（系统配置槽不带根，也不加记录） |
| B0J：基镜像的 journal 里有比基镜像最新的根更新的记录时，上界取不出、整张表都进 | 推的修 | 推的保（mkfs 之后 journal 是空的） | 推的；要一个「记录比根新」的判法，没实现 |
| B1（版本一行都不进，第二轮攻方提过） | 不修：(1,4) 照样不进 | 保 | 推的 |

还要一条能红的用例：两流测试文件里加这一对（崩溃基镜像、只差 (1,4) 那一版），两个次序读回与现判逐态相同；配一行变异「上界不并恢复基镜像走到的根」。推的，没实现。

推翻条件：在入库装置上照这一对的造法跑，第 0 个点身份、复用键、计划哈希有一样不同，或两条流在那 8 态上判得一样，这一格就不成立。

## L1 第二节：今天的流上 B0 之后块键相同的点（没打中）

照第二轮攻方的 `j1_diff`（原样拷来，只换包名）在 B0 之后的代码上重量：块键三样都相同的点逐态现判比 `red_items` 与第 ② 段核对位；身份与复用键相同而计划哈希不同的点另比（`J1SPLIT`）。B0 之后两流的第一个点并进了同键那一组（第二轮是 9 点、这一轮 10 点），到 D 对到卸载从 26 点到 27 点（展开 6）。`outputs/j1_diff-6.out` 里两对的原样：

```
J1SPLIT pair=two_flows same_identity_and_reuse_key_but_different_plan_hash_points=0 states_compared=0 judge_states_differing=0 points=[]
J1DIFF pair=two_flows left=two_flows_a right=two_flows_b equal_block_key_points_with_states=10 states_compared=44 judge_states_differing=0 verifier_states_differing=0 first_difference=None
J1SPLIT pair=rollback_vs_unmount same_identity_and_reuse_key_but_different_plan_hash_points=1 states_compared=1 judge_states_differing=0 points=["roll_back_by_a_forward_publish.root_record_fua.1", "roll_back_by_a_forward_publish.system_configuration_slot.1(states 8 vs 9)"]
J1DIFF pair=rollback_vs_unmount left=unmount right=rollback equal_block_key_points_with_states=27 states_compared=115 judge_states_differing=0 verifier_states_differing=0 first_difference=None
```

展开 16（`outputs/j1_diff-16-rollback.out`，264 秒，`outputs/j1_diff-16-rollback.time`）：

```
J1SPLIT pair=two_flows same_identity_and_reuse_key_but_different_plan_hash_points=0 states_compared=0 judge_states_differing=0 points=[]
J1DIFF pair=two_flows left=two_flows_a right=two_flows_b equal_block_key_points_with_states=10 states_compared=44 judge_states_differing=0 verifier_states_differing=0 first_difference=None
J1SPLIT pair=rollback_vs_unmount same_identity_and_reuse_key_but_different_plan_hash_points=1 states_compared=1 judge_states_differing=0 points=["roll_back_by_a_forward_publish.root_record_fua.1", "roll_back_by_a_forward_publish.system_configuration_slot.1(states 8 vs 9)"]
J1DIFF pair=rollback_vs_unmount left=unmount right=rollback equal_block_key_points_with_states=29 states_compared=131185 judge_states_differing=0 verifier_states_differing=0 first_difference=None
J1DIFF_TOTAL judged_states=262460 threads=12
```

展开 6 那一趟另有第二轮的七种「只改点之后」的变体，块键相同的点判定都 0 态不同（`outputs/j1_diff-6.out` 的 `J1DIFF pair=later_*`、`last_root_txg_jump`、`versions_after_d`、`judged_root_moved` 行）。展开 16 只重跑了两流与到 D 对到卸载两对，变体没重跑。

结论：今天的四种流上 B0 只把每条流的第一个点并进同键，判定与核对位逐态相同；两流里「判器输入真相同而块键不同」的点从 1 降到 0，到 D 对到卸载只剩被判根槽写那一点（第二轮已记，按字面判器输入不同）。推翻条件：在入库装置上找到一对从 mkfs 起的流，块键相同而某态判得不同。

## L1 第三节：txg 不严格递增那一支与 M3 那两条断言（没打中）

照用例原次序重算（`named_tests`），变异由仓副本里的开关打开，每条用例打出第一条 FAIL 的断言（cargo test 里用例就红在那一条）。四个文件的 `NTEND` 行原样：

```
# outputs/named_tests-baseline.out
NTEND test=a_twin_whose_root_slot_write_alone_differs_reads_back_its_own_verdicts first_failing_assertion=none(green)
NTEND test=a_version_the_prefix_cannot_reach_leaves_the_judge_inputs_of_the_prefix_alone first_failing_assertion=none(green)
NTEND test=root_slot_writes_whose_txgs_do_not_strictly_increase_put_every_later_write_into_the_judge_inputs first_failing_assertion=none(green)
# outputs/named_tests-mut1587.out
NTEND test=a_twin_whose_root_slot_write_alone_differs_reads_back_its_own_verdicts first_failing_assertion=reuse_key_differs
NTEND test=a_version_the_prefix_cannot_reach_leaves_the_judge_inputs_of_the_prefix_alone first_failing_assertion=none(green)
NTEND test=root_slot_writes_whose_txgs_do_not_strictly_increase_put_every_later_write_into_the_judge_inputs first_failing_assertion=none(green)
# outputs/named_tests-mut1731.out
NTEND test=a_twin_whose_root_slot_write_alone_differs_reads_back_its_own_verdicts first_failing_assertion=none(green)
NTEND test=a_version_the_prefix_cannot_reach_leaves_the_judge_inputs_of_the_prefix_alone first_failing_assertion=prefix_point_0_judge_inputs_equal
NTEND test=root_slot_writes_whose_txgs_do_not_strictly_increase_put_every_later_write_into_the_judge_inputs first_failing_assertion=none(green)
# outputs/named_tests-mut1732.out
NTEND test=a_twin_whose_root_slot_write_alone_differs_reads_back_its_own_verdicts first_failing_assertion=read_back==fresh_twin
NTEND test=a_version_the_prefix_cannot_reach_leaves_the_judge_inputs_of_the_prefix_alone first_failing_assertion=none(green)
NTEND test=root_slot_writes_whose_txgs_do_not_strictly_increase_put_every_later_write_into_the_judge_inputs first_failing_assertion=none(green)
# outputs/named_tests-mut1733.out
NTEND test=a_twin_whose_root_slot_write_alone_differs_reads_back_its_own_verdicts first_failing_assertion=none(green)
NTEND test=a_version_the_prefix_cannot_reach_leaves_the_judge_inputs_of_the_prefix_alone first_failing_assertion=none(green)
NTEND test=root_slot_writes_whose_txgs_do_not_strictly_increase_put_every_later_write_into_the_judge_inputs first_failing_assertion=point_0_not_increasing_judge_inputs_differ
```

- txg 不严格递增：用例真走到 `MayAllBeRead` 那一支。第 1733 行变异下第 0 个点起每个点的「判器输入跟着变」都 FAIL（`outputs/named_tests-mut1733.out`），断言是「            "txg 不严格递增：第 {index} 个点（{}）的判器输入罩着之后的整段写表，最后一次写改了它跟着变",」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:1004`）。线索：这条用例只改流里最后一次写，钉得住「之后的写整段进摘要」，钉不住摘要从点之后紧接着的那一次写起（`suffix_digests[last_write]` 写成 `[last_write + 1]` 推的不红）；变异表里没有这一行，不在 L4 的判据里。
- M3：第 1587 行变异下只有「        "这一步的写表不同，复用键不同"」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:907`）那一条 FAIL，读回那一条照样 PASS；第 1732 行变异下只有「        "根槽写改坏的那一条读回的判定与现判的不同：复用了健康那一条的判定"」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:914`）那一条 FAIL，读回第 48–56 态（管根槽写的那个点之后的 9 态）与现判不同。与用例文档注释写的分工一致。线索：1587 那一行只靠直接比键那条断言拦，变异改的就是键的算法，这一拦是结构上的；读回分不出这一种改坏，第二轮已量过、实现员照写了。

## L4：变异表（没打中）

### 范围

冻结副本 `crates/mutations.tsv` 对第二轮冻结副本按名字比：新增 32 行、改写 11 行、删 0 行；新增里第 1722 行（E168）是别的会话的，不判，余 42 行（`outputs/inscope.tsv`，第一列是行号）。

### 静态核（42 行全过）

用第二轮攻方的 `k3check.py`（`K3_TREE` 指冻结副本）逐行核原文恰好命中次数、替换文与原文不同、点名测试的定义处数与所在目标、筛选词、特性、`#[ignore]`（`outputs/k3check.out`）：42 行都是 `hits=1 replacement_differs=True test_defs=1`。它报的 7 处「筛选词不罩」「不是测试」逐个对过原文，都是它自己的毛病：第 1725–1728 行的四条用例在顶格的 `mod tests`（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:822` 起），不在 `gpu_test_start` 里，路径是 `gpu_unit_checks::tests::…`；第 1737、1589 行的两条在顶格的 `mod wall_clock_progress_line_tests`（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/src/crash.rs:4712`）；第 1749 行那条的 `#[should_panic(` 跨了三行（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/src/crash_injection.rs:2991` 起），工具认不出它前面的 `#[test]`。

### 实测（原型重算或 harness 档证红）

| 行 | 怎么量 | 结局 | 输出 |
|---|---|---|---|
| 1731 | 原型重算点名用例，开关 `R3_MUT_1731` | 红在「前缀第 0 个点判器输入相同」 | `outputs/named_tests-mut1731.out` |
| 1732 | 同上，`R3_MUT_1732` | 红在读回那一条 | `outputs/named_tests-mut1732.out` |
| 1733 | 同上，`R3_MUT_1733` | 红在第 0 个点「判器输入跟着变」 | `outputs/named_tests-mut1733.out` |
| 1741 | 依赖用例的断言照原文重算，在只改了原文的副本上（判法代码按原文收，不编译） | 8 条登记用例里 3 条 FAIL ⇒ 用例红 | `outputs/dep_check.out` |
| 1729、1730 | C604 源码扫描照原扫法重算，在只改了原文的副本上 | 都红在「绕开入口」那一条，点名的行是改的那一行 | `outputs/scan_check.out` |
| 1742、1714、1715、1716、1717 | `prove-red.sh --copy`（harness 档，冻结副本的拷贝） | 五行都抓到 | `outputs/prove-red.out` |

原样（各文件里的判定行）：

```
DEP_TOTAL root=/tmp/claude-1000/test-script-align-code-r3/tree targets=8 failing=0 test_would_be=green
DEP_TOTAL root=variants/row1741 targets=8 failing=3 test_would_be=red
SCAN_VERDICT root=/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier test_would_be=green first_red=None
SCAN_VERDICT root=variants/row1729/crates/singlefs-checker-tier test_would_be=red first_red=Some("not_allowed=[\"tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs:38\"]")
SCAN_VERDICT root=variants/row1730/crates/singlefs-checker-tier test_would_be=red first_red=Some("not_allowed=[\"src/crash_judge_dispatch.rs:997\"]")
数值环境变量 故障注入线程数写错悄悄退回本机并行度（不经共用解析）	抓到	an_unreadable_fault_injection_worker_thread_count_panics_naming_the_variable_and_the_value 红了
数值环境变量 共用判法：设了而认不出悄悄当成没设（C606 同族：写错的规模、额度、线程数退回没设时的那一档）	抓到	a_number_variable_set_to_text_that_is_not_a_number_panics_naming_the_variable_and_the_value 红了
数值环境变量 共用判法：线程数设成 0 也悄悄当成没设（不许为 0 的数认不出时退回本机并行度）	抓到	a_nonzero_number_variable_set_to_zero_panics_naming_the_variable_and_the_value 红了
数值环境变量 共用判法：没设也当成设了 0（调用方没设时的那一档被吞）	抓到	an_unset_number_variable_is_none 红了
数值环境变量 共用判法：不去前后空白（复现行抄出来带换行的值认不出）	抓到	a_set_number_variable_reads_back_without_surrounding_blanks 红了
✓ 点名 5 条：跑了 5 条，跳过 0 条，跑的都抓到了
```

第 1723、1724、1743、1744、1753 行实现员已在 harness 档证过红（各自报告），我没重跑。余下 26 行在 checker 档包里（库内单测或两流、流水线、σ 测试文件），只读代码推：替换文改的都是那一判、点名的测试都调到被改的那几行、推得出会红（多数是 `#[should_panic(expected = …)]` 等不到 panic 或消息对不上）。没量，推的。

线索（不算打中）：第 1736 行「核对线程数没设时取 1」点名的用例比的是「没设时等于本机并行度」：「            std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:2928`），本机并行度是 1 的机器上替换文与原文结局相同，这一行报「没红」。本机 32 核，内存包装不限 CPU（推的，读 `research/scripts/run-with-memory-cap.sh` 只设内存）。

推翻条件：checker-tier-crates-mutation-replay 判这 42 行里有一行没红或无效。

## L2（兼）：数值环境变量解析收成一份之后

原型 `l2_matrix` 直接调库里的几处入口（一次只设一个变量，panic 用 `catch_unwind` 收），78 行 `L2 site=…`（`outputs/l2_matrix.out`）。

### 同一个重放入口里，前后空白的结局两样（打中：口径两处不一）

共用解析先去空白：「    match text.trim().parse::<Number>() {」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-harness/src/numeric_environment_variable.rs:51`）。同一个崩溃注入重放入口里，每段步数仍走 `history` 的 `Result` 版，不去空白：「            text.parse()」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-harness/src/history.rs:4095`）；取名字的几个变量按原文逐字比：「        .find(|choice| environment_value_of(*choice) == value)」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-harness/src/history.rs:4138`）；短段写数先按原文比 `none`，比不上才交共用解析：「    let draw = if short_segment_writes == SAMPLED_WITHOUT_SHORT_SEGMENTS_VALUE {」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/src/crash_injection.rs:2482`）。原样（摘 `outputs/l2_matrix.out` 里 `crash_injection_campaign_from_environment` 的几行，前缀 `L2 site=crash_injection_campaign_from_environment variable=SINGLEFS_CRASH_INJECTION_` 照抄）：

```
L2 site=crash_injection_campaign_from_environment variable=SINGLEFS_CRASH_INJECTION_FIRST_SEED value=" 7" outcome=accepted:"first_seed=7 seeds=2 operations=3 draw=Sampled { crash_points_per_history: 4 } assertions=NewFindingsOnly"
L2 site=crash_injection_campaign_from_environment variable=SINGLEFS_CRASH_INJECTION_SEEDS value=" 5" outcome=accepted:"first_seed=1 seeds=5 operations=3 draw=Sampled { crash_points_per_history: 4 } assertions=NewFindingsOnly"
L2 site=crash_injection_campaign_from_environment variable=SINGLEFS_CRASH_INJECTION_POINTS value=" 4" outcome=accepted:"first_seed=1 seeds=2 operations=3 draw=Sampled { crash_points_per_history: 4 } assertions=NewFindingsOnly"
L2 site=crash_injection_campaign_from_environment variable=SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES value=" 3" outcome=accepted:"first_seed=1 seeds=2 operations=3 draw=EveryProperSubsetOfShortSegments { segment_writes: 3, sampled_in_longer_segments: 4 } a
L2 site=crash_injection_campaign_from_environment variable=SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES value=" none" outcome=panic:SINGLEFS_CRASH_INJECTION_SHORT_SEGMENT_WRITES=" none" 认不出（invalid digit found in string）：要
L2 site=crash_injection_campaign_from_environment variable=SINGLEFS_CRASH_INJECTION_OPERATIONS value=" 40" outcome=panic:SINGLEFS_CRASH_INJECTION_OPERATIONS= 40：只认 非负整数
L2 site=crash_injection_campaign_from_environment variable=SINGLEFS_CRASH_INJECTION_OPERATIONS value="40\n" outcome=panic:SINGLEFS_CRASH_INJECTION_OPERATIONS=40\n：只认 非负整数
L2 site=crash_injection_campaign_from_environment variable=SINGLEFS_CRASH_INJECTION_ASSERTIONS value="fast-tier\n" outcome=panic:SINGLEFS_CRASH_INJECTION_ASSERTIONS=fast-tier\n：只认 new-findings、small-fast-tier、fast-tier、r
L2 site=crash_injection_campaign_from_environment variable=SINGLEFS_CRASH_INJECTION_DEVICES value=" 4gib" outcome=panic:SINGLEFS_CRASH_INJECTION_DEVICES= 4gib：只认 4gib、unit-area-384、unit-area-256、unit-area-240
```

（行尾截在 260 个字符，原行更长；`grep -c '^L2 ' outputs/l2_matrix.out` 是 78。）

- 分不分辨：分得开。同一行复现命令里，带空白的数在种子、崩溃点数上照收，在每段步数上停下；` none` 停下时说「invalid digit」，不说「只认数或 none」。
- 系统看得到：看得到，原文就在手里。
- 满足哪一句：本轮正文第五节第二条括号里的「口径两处不一」。结局都是「照收」或「停下」，没有一处悄悄换成别的值，所以不是「该 panic 不 panic」。
- 分量：复现行由 `environment_assignments_in_line` 按空白切开再喂回，切出来的值不带空白；只有人手设环境变量时撞得到。小。
- 改法（推的，被攻过零轮）：每段步数改经共用解析；取名字的变量与 `none` 也先去空白，或共用解析不去空白、统一照原文判。没实现。

### 写 0：线程数停下，种子数照收、跑 0 个状态判绿（打中，主 agent 若认「规模写 0 不算写错」归可议）

```
L2 site=common::<NonZeroUsize> value="0" outcome=panic:V="0" 认不出（number would be zero for non-zero type）：要写一个 core::num::nonzero::NonZero<usize> 
L2 site=crash_amplification::checking_threads value="0" outcome=panic:SINGLEFS_CRASH_AMPLIFICATION_THREADS="0" 认不出（number would be zero for non-zero type）：要写一个 
L2 site=crash_amplification::expand_up_to value="0" outcome=accepted:0
L2 site=crash_injection_campaign_from_environment variable=SINGLEFS_CRASH_INJECTION_SEEDS value="0" outcome=accepted:"first_seed=1 seeds=0 operations=3 draw=Sampled { crash_points_per_history: 4 } assertions=NewFindingsOnly"
L2 site=crash_injection_campaign_from_environment variable=SINGLEFS_CRASH_INJECTION_POINTS value="0" outcome=accepted:"first_seed=1 seeds=2 operations=3 draw=Sampled { crash_points_per_history: 0 } assertions=NewFindingsOnly"
L2 site=crash_injection_campaign_from_environment variable=SINGLEFS_CRASH_INJECTION_OPERATIONS value="0" outcome=accepted:"first_seed=1 seeds=2 operations=0 draw=Sampled { crash_points_per_history: 4 } assertions=NewFindingsOnly"
L2ZERO seeds=0 histories_run_crash_points=0 new_findings=0 assertions=NewFindingsOnly judged_by_new_findings_only_would_be_green=true
```

`L2ZERO` 那一行是把种子数写 0 的那一批真跑起来（`run_crash_injection_campaign`，0 段）：0 个崩溃状态、0 条新发现；大档入口默认照「只判清单外的失败」判，那一组不看计数：「        CrashInjectionAssertions::NewFindingsOnly => {}」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:187`），用例绿。上游的条款是「- 退出码 0 跑到的全绿（有项报了「本次未跑」也是 0）、1 有红、2 用法错、77 一项都没跑到、78 准入或运行条件不满足（`preflight-discipline.md`），这五个的含义固定；别的拒绝形态可以另用退出码，在文件头逐个列明。」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:32`），种子数写 0 的那一趟一项都没跑到，按 0 绿着过去。展开上限写 0 照收（每段都不展开），那一趟流水线自己报「不是全域」，不算绿着过去（推的，没跑流水线）。

- 分不分辨：分得开，一个跑了、一个没跑。
- 系统看得到：看得到，种子数就在 `campaign.seed_count` 里。
- 满足哪一句：第五节第二条「该 panic 不 panic」——前提是种子数 0 算写错；第二轮判决只把线程数 0 定为全仓一个口径，没说规模 0，所以归哪一句交主 agent。
- 改法（推的，被攻过零轮）：种子数、崩溃点数、每段步数这几个规模变量取 `NonZeroU64` / `NonZeroUsize`；或照收但大档入口在 0 段时报「本次未跑」。没实现。

### 非 UTF-8 与依赖判法（小，线索）

- 非 UTF-8 结局各处都是停下，消息有两种写法：共用读法「            panic!("{variable} 的值不是 UTF-8：{raw:?}；不当成没设")」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-harness/src/numeric_environment_variable.rs:30`），故障注入自己那一份「                panic!("{FAULT_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE} 不是 UTF-8：{raw:?}")」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-harness/src/fault_injection.rs:2615`）。读代码，没跑。
- 依赖判法在新模块下照样成立：冻结副本上 8 条登记用例全 PASS，`numeric_environment_variable.rs` 被收进每一条的判法代码（经 `crash.rs` 或 `crash_amplification.rs`），`history.rs` 只在入口测试文件自己引它的三条里出现、理由都是测试文件（`outputs/dep_check.out` 前 9 行）。线索：理由只记第一个引到它的文件，「                        .or_insert_with(|| format!("referenced by {file}"));」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/src/crash_identity.rs:915`），判的又只是这一个理由，「                    reason.starts_with("referenced by crates/singlefs-checker-tier/tests/"),」（`/tmp/claude-1000/test-script-align-code-r3/tree/crates/singlefs-checker-tier/src/crash_identity.rs:1448`）。第 1741 行改法下 `crash_enumeration_floor_raise_pushed_by_admission` 与 `record_checker_judges_absence_by_the_persisted_set` 两条照样 PASS（入口文件先引了 `history`），靠另外三条红。哪天库模块改成只在这两条的闭包里引 `history`，这一判看不见。今天不影响判法摘要（`history.rs` 反正已经在那两条里），不算打中。

## 没打中的形状

| 攻的地方 | 试过的形状 | 取样范围 |
|---|---|---|
| L1 同键异判 | 两流 A/B；到 D 对到卸载；到卸载对七种只改点之后的变体；崩溃基镜像重开之后的一对（打中，见第一节） | 展开 6 全部同键点；展开 16 只重跑两流与到 D 对到卸载 |
| L1 同输入异键 | 同上各对的「同身份同复用键而计划哈希不同」 | 同上 |
| L1 txg 不严格递增 | 第 1733 行变异下重算那条用例 | 1 条流 |
| L1 M3 | 基线、1587、1732 三种下重算，逐条断言 | 1 对流 |
| L4 | 42 行静态核；12 行实测（见 L4 节表） | 42 行 |
| L2 | 10 种数值写法 × 4 处解析；崩溃注入重放入口 10 个变量 × 2–7 种写法；种子数 0 真跑一批 | 78 行 |
| L2 依赖 | 冻结副本与第 1741 行副本上 8 条登记用例 | 16 次收判法代码 |

## 这条腿自己的限度

- 数都是草稿副本上原型跑的，不是入库装置。流照用例的造法拷来（第二轮攻方的 `flows.rs` 加本腿的 `crashed_base_then_mount`），盘换成内存稀疏盘（`make_common.py`）。被点名的用例是照断言重算，用例本身（checker 档）没跑。
- 崩溃状态没有按层 0 的枚举域全展开：原型都按展开上限跑（6，展开 16 只一趟），与两流用例同一个上限；全展开的闭式数两流 33554462、到 D 51773503、到卸载 1662648449、崩溃基镜像那一条 17170460（各输出的 `closed_form_state_count_fully_expanded=`），都超过约 10⁶，所以没全展开跑。崩溃基镜像那一对在展开 6 下已经打中；更深的展开没量。
- 状态量：这一腿实跑合起来约 26.6 万态（展开 16 那一趟 262460，`outputs/j1_diff-16-rollback.out` 末行；别的几趟合起来几千态），低于 10⁷；每一次跑之前原型按展开上限数状态、超过 10⁶ 就停（`j1_diff` 的断言）。展开 16 那一趟没先跑小段估时，照第二轮同一趟的量（262372 态、4 分 45 秒）估不到 5 分钟，实跑 264 秒。
- 改法 B0R 只在崩溃基镜像那一对与三条用例上量过；没在展开 16 下量，没量它对别的流的计划哈希改了多少。
- 内存包装：编译两次（首编 1 分 27 秒、加改法后增量 6 秒）、原型二进制十几次，都退 0；没撞包装自己的结局。

## 没做什么

- 没跑 checker 档包的任何测试与门禁阶段；42 行变异里 26 行只读代码推，没证红。
- 没在入库装置上重做任何数；B0R 与 L2 的两个改法被攻过零轮，后两个只是推的。
- 崩溃基镜像那一对没进库跑读回（同键就读回同一块，是按块键的组成推的）；没在展开 16 下跑这一对。
- 没判 L3、L5（正推腿的格）；L2 只做了量，没逐个调用方核引入口（正推腿的格）。
- 没读禁读清单里的文件。

## 草稿与删除

- 删了：原型包连同它的编译目录 `/tmp/claude-1000/test-script-align-code-r3-opus/proto`（771M）、仓副本 `/tmp/claude-1000/test-script-align-code-r3-opus/tree`（13M）、证红用的仓副本 `/tmp/claude-1000/test-script-align-code-r3-opus/prtree`（648M，含它的 target）、只改原文的三份副本 `/tmp/claude-1000/test-script-align-code-r3-opus/variants`（37M）。复跑材料都在模型目录里。
- 留着：草稿目录里的 `runs/`（原样输出、`.err`、编译日志、证红日志）、`model/`（模型目录的底稿）、`inscope.tsv`、`k3check.out`、`rows-*.txt`；都不是仓副本或编译目录。
