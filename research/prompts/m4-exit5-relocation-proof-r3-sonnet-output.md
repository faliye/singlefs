# m4-exit5-relocation-proof-r3　云端辩方腿报告（Sonnet）

轮：m4-exit5-relocation-proof-r3（最后一轮）。立场：云端辩方，格 Z1–Z5；被复核的判决是 `research/prompts/m4-exit5-relocation-proof-r2-main-verification.md` 第二节。
冻结副本指 `~/.local/share/singlefs/work/m4-close-rounds-frozen/`（只读；下面写 `crates/…:行号` 的都是它的，写主工作区的另注）。没读禁读清单里攻方腿这一轮的产出。
我量的数都在草稿目录的副本上，不是入库装置上的数；判决要引的，照 `.claude/rules/three-way-inference.md` 在入库装置上重做。

## 复跑

模型目录 `research/prompts/m4-exit5-relocation-proof-r3-sonnet-model/`，`tools/rerun.sh` 照 `scripts/agents/three-way-rerun-template.sh`，只改了 `rerun_steps` 那一段。
整份连跑超过 240 秒，分五段各一次（`RERUN_STEPS=z1` 到 `z5`），每段一个新的 `--work`：

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-exit5-relocation-proof-r3-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --work <新的草稿目录>
```

前面加 `RERUN_STEPS=z1`（或 z2 … z5）。每段末行 `name=rerun_result mismatches=0` 才算复出。第二台没有 Verus，五段都在本机跑。

| 段 | 内容 | 本机用时（此刻负载 200 上下） |
|---|---|---|
| z1 | 提升到区间的根环协议、原协议探针、桥探针、三个必红世界、G7 窗口移格穷举 | 2 分 12 秒 |
| z2 | 门禁漏写法的 Verus 与产品构建、`verus --compile`、std 规格差分测试 | 2 分 25 秒 |
| z3 | 十二条闭包、回收门槛调用点与改盘面字节的写点在不在三个闭包里、主工作区闭包 | 约 4 分 40 秒 |
| z4 | 证明/幽灵代码影响执行的四种写法、判器键去掉规格与证明块的原型、已验核里改一条引理的键 | 约 2 分 |
| z5 | 闭包并集、A–D 分类、std 调用键、证明行对代码行的比例 | 28 秒 |

模型目录里每个文件的 sha256（`find . -type f | sort | xargs sha256sum`，相对模型目录）：

```
d46414ff83cc9dc4628bcf0fcc92ce7b228273aeb78b01b58ab110354afc5908  out/z1.txt
cfd295e9b7e961c616d340bf9563bc4a7a137690d71fee4fe68663b86de25b9f  out/z2.txt
f7895ad4a2ede88569bd7bb3165e6cceeda70903ecca9a2fe601873bbc2c5cc9  out/z3.txt
dff83c6d6d77fbf68d40747cf1da93f019a6cd41d6b4b2e394c2c6f81c1e1c7c  out/z4.txt
8b53377d6a355af83806dbc6e9f2eab79a6ef2a59faf19ee43d1faac7b7246bc  out/z5.txt
e0a8729de85c206d62081596f3f71dd6a8e88dc5146a8e6b95e2e85e86bcbb76  std-diff/std_spec_differential.rs
35009b2eac205e834bccb3e6efe1f8fdf0abf46ccba308dba2f95c6b30b3bebc  tools/erased_lines.py
a58c0355af2bd135844626ecf6d41fce928142302586b5ce3cd4163d4b8c5e34  tools/g7_named_roots.py
609dc256121d2bfdccd3304324ade2067bb773cd226b98153f686deb7bd62f78  tools/g7_window_shift.py
1c1f28cbb52aa5b654bc5bc3f7f5ac20cf1404727eae709546c7f0a2962f3885  tools/key_lemma_edit.sh
669106a856ede9935452406352cb473a8325d8c0cc1ca96ca9b14fc44640bf21  tools/key_variants_stripped.sh
2ebe665f42c83f0a02bd06fb0784a98f96f9ed4c7953f885aad7d3f707911833  tools/mutate.py
3ecd78f610ed2f9db3893108c7c7ffe5d8f0e562216c0f87fc96c46cee37285d  tools/proof_ratio.py
776fe6a33da995f81503d34e371d1ccce122604405aabec1f219ebfa126902a1  tools/rerun.sh
7a1898c83bbf858f20f5a942c2e1e8fc7bf4b54e9ecbd9f77687caf948b635af  tools/strip_verus.py
bc3a3090a5fd71d258f457a1fcc29bdc320bd18bbbec975170c29d14b3e94f30  tools/z3_membership.py
be9964cbb6cc54df7e3a2233f42dad38149ac609416152cf76ef3bdcf4301d0a  tools/z3_writers.py
d24f30422102a7ce055a48b85417d6b6116402dec9862e71820ab86dd72e2cb4  tools/z5_union.py
55a68e82c36a6f93912716fe5de5cfd5e88b28a7c076bc11c49685073cf11773  verus/gc_ring_runs.rs
9e0d10624f3225fc1b4c44542a21eec64ffc75f862929b81e731e19970669fae  verus/z1_bridge_probe.rs
e011556bec409eb239bb548ce6cbc1784e7615ca432eb6d6dfd765a1a7ebaeec  verus/z1_original_probe.rs
47b240ab250f843ffc374ba43fb087c667e86f725bb315f1d392c7041191c24a  verus/z2_f1b_debug_assertions_fork.rs
153ddd1f99d006e5ccd0722f4e8f6e5d935f43e20c1a04dfbff42966f881f217  verus/z2_f1_feature_fork.rs
af8a6fe1b0741d7c264b006a28301a3d48c06f773a2a021d423a1d3e394aef05  verus/z2_f2_trait_impl_outside.rs
4e398b3542f4ab20449fc5754ce3ed7f27d0c5eb63b019cb2248ee71d70bd638  verus/z2_f3_unsafe.rs
a800f3b107e3804b194d5c59811701eabc6a1c68e91e828144c7edc2d52b4e1f  verus/z2_f4_default_method.rs
176b8b4008594b3a3f094625fc32f7f64843ee200c3760ac1297334b905abaf8  verus/z2_f5_macro_outside.rs
d44a5d52769429828600a2fa610ec8582d7b0d1fdca4c0c0513e966a43576111  verus/z2_f5_main.rs
0fe19bdf5b1ce5e4fb84dd5d0b9ad09e504dc97c6553b6055462d12072e7d4f8  verus/z2_g1_trait_spec.rs
b10f1cc05c6aed6e94c4217ea6da7dd5f57b94efaa770a4d7121427584295554  verus/z2_g2_external_attr.rs
e05f69df8929738d89419ebbe5a84aae2a412130428139ce1fec6225b1bb1c61  verus/z4_h1_proof_calls_exec.rs
07bf87cd50347afc9ed2f0800b4013fe44a91f4b78b1186eb16881b4d6b871fb  verus/z4_h2_ghost_in_exec_branch.rs
efa9a0786926960c95bc0cd1c25d8e2739e8b4f29545599d874f7bb8d5a3b4e8  verus/z4_h3_ghost_param_selects_exec.rs
68a9947324909e6cf0e31b97925b424f8637776c4dc7f4578f94fa3ba8a9eb7d  verus/z4_h4_proof_block_assign_exec_var.rs
8451c13ecc668e444015beb5e8f9d48f94467572b87b0f1bdb8aa4ac26e3a3b5  z2crate/Cargo.toml
47b240ab250f843ffc374ba43fb087c667e86f725bb315f1d392c7041191c24a  z2crate/src/f1b_debug_assertions_fork.rs
153ddd1f99d006e5ccd0722f4e8f6e5d935f43e20c1a04dfbff42966f881f217  z2crate/src/f1_feature_fork.rs
af8a6fe1b0741d7c264b006a28301a3d48c06f773a2a021d423a1d3e394aef05  z2crate/src/f2_trait_impl_outside.rs
176b8b4008594b3a3f094625fc32f7f64843ee200c3760ac1297334b905abaf8  z2crate/src/f5_macro_outside.rs
1d7bdb5f967c93c48f9b423d2984da9a9f2b60ecee59d200b6c76fe836e19aff  z2crate/src/lib.rs
838124d0286d708551ccf93b7bb6ce828636eaf9c5de81721c3cfed95dd0c1f5  z2crate/src/main.rs
```

## 〇　判定一览（辩方判定三种：站得住 / 够不着 / 同样打中别的臂；一条可以带两种）

「别的臂」指第一轮被否的「核外提」那一叠：条目判定与目的地判定放进 `singlefs-replay-prefix` 的可执行核由 core 调，加写序前缀定理，胶水归 I-1.14、崩溃点枚举、模型对拍（`research/prompts/m4-exit5-relocation-proof-r1-main-verification.md` 第二节 X3 与 X4）。

| 格 | 判决条目（第二节） | 辩方判定 | 证据 |
|---|---|---|---|
| Z1 | Y5：采改后协议；变体一（搬迁写进回退目标还在读的槽） | 站得住，但要靠协议的门，不靠区间 P4 的前提：区间 P4 单独放过变体一 | 第一节 1.1、1.2 |
| Z1 | Y5：变体二（两槽单元的第二槽照样可写），「排除了才算这一格关上」 | 对原协议够不着：原协议排除不了、也说不出；提升到区间的协议排除（零轮形态） | 第一节 1.2、1.3 |
| Z1 | Y5：G7 改成「回退候选窗口移一格」 | 站得住，措辞要改：字面「逐项相同」不成立，有界穷举下成立的是「现行内容相同、用户点名的候选是不整理那份的后缀、至多少 k 个」 | 第一节 1.4 |
| Z1 | 改后协议的前提在 core 今天成立吗 | 一条不成立（`ForcedToZero`，只供测试）；一条靠 core 里协议没有的扣住机制满足；一条没人证（回收门槛蕴涵未被引用） | 第一节 1.5 |
| Z2 | Y2：去掉 `--no-cheating`，四类标记进信任清单，`axiom fn` 一律不许 | 站得住；但清单只管 Verus 自己看得见的四类，下面三类写法 Verus 验过、产品违 ensures、清单与门禁都不红 | 第二节 2.1 |
| Z2 | Y1：按 `verus_keep_ghost` 分叉的函数体判红 | 够不着：只挡一个 cfg 原子；feature 原子、`debug_assertions`、verus! 外面的宏里的 cfg 三种同形写法照过。同样打中核外提：今天门禁对 `verus_keep_ghost` 零命中，replay-prefix 同样暴露 | 第二节 2.1、2.4 |
| Z2 | Y1：产品构建与 Verus 的 MIR 逐函数相同进门禁 | 够不着：按第二轮正推 Y1 的比法（原版对 `verus!` 包后两次都是产品编译）看不见 cfg 分叉；要比 Verus 自己编出来的那一份 | 第二节 2.2 |
| Z2 | Y2：每条配差分测试，判得出写错的 std 规格 | 站得住（给一例：`binary_search` 写成「第一个下标」，360 例里 168 例红，真规格 0 例红）；局限在有界域与没有 std 对照的设备类 | 第二节 2.3 |
| Z2 | Y2：vstd 自带的按版本整体登一行 | 站得住；同样打中核外提：replay-prefix 今天的证明已信任 vstd 的 `Vec` / `Seq` / `Map` 规格 | 第二节 2.5 |
| Z3 | Y3：射程 = 发布、回退、崩溃后读映射三个闭包的并集 | 够不着：含回收门槛调用的七个非测试 fn 里三个在三个闭包外（算 floor 的两个：挂载重建分配器后的回收、抬 F 序列里的回收；外加它们调的带隔离的包装）；另有丢弃、实例表扫尾两条改盘面字节的路也在外 | 第三节 3.1、3.2 |
| Z3 | Y3：选段、规划、区域扫描、GPU 后台计算 | 站得住（不必进证明）：它们只选不改，条目核 `relocation_checked_against_the_version` 在发布闭包里；同样适用核外提 | 第三节 3.3 |
| Z3 | Y3：测试开关 `set_reuse_window` 进第三轮必攻 | 站得住（pub、无产品调用者、闭包外）；协议里的对应世界必红；同样打中核外提：被产品执行的回收门槛核同样不管 `floor` 的来历 | 第三节 3.4 |
| Z4 | Y4：判器键去掉 Verus 规格与证明块 | 站得住（原型上五种只改证明的写法键不变，三种改执行代码的键变；Verus 的模式检查拒四种「证明影响执行」的写法）；判决漏写一条：形式化判定的键必须保留规格；原型文本级，对 `lib.rs` 排版失败，要换 Verus 感知的解析 | 第四节 4.1–4.3 |
| Z4 | Y4：做不到时整库重判的代价写进交用户表 | 站得住（数见 4.4）；同样打中核外提：判器闭包里今天已有三个 Verus 核 fn，改它们的证明已经让整库作废，只是 3 个对 549 个 | 第四节 4.4 |
| Z5 | 闭包 1073 个 fn、24970 行；攻方 1071 个 | 站得住，复出：差在闭包里两个 `singlefs-replay-prefix` 核 fn，core 自己 1071 个、24958 行 | 第五节 5.1 |
| Z5 | A 472 / B 190 / C 291 / D 120；std 调用键 456、配上 49% | 站得住，命令照写原样复出；A 是上界（185 个没配上名字的单元归了 A，只看直接调用） | 第五节 5.2 |
| Z5 | 「约 11 万–12.5 万行」 | 够不着：4.5–5 倍取自第二轮正推自己新写的两个原型，不是现成核；同一份 `relocation_judgement.rs` 现数 144 → 57 行（2.5 倍），`compaction_plan_proofs.rs` 8.6 倍，按现数全验约 6 万–21 万行 | 第五节 5.3 |

## 一　Z1　改后协议与区间 P4 撑不撑得住

判决原文：「排除了才算这一格关上」（`research/prompts/m4-exit5-relocation-proof-r2-main-verification.md:23`）。下面逐个变体看「排除」排在哪一件上。

### 1.1　变体一：搬迁写进回退目标还在读的槽

- 原协议（第二轮正推 `gc_ring_proofs.rs`，模型目录里我把它整份拷进 `verus/z1_original_probe.rs` 后追加一条证明 `variant_one_is_refused_and_variant_two_is_not_in_the_original_protocol`，`z1_original_probe.rs:486`）：现行只有 key 7 在槽 10，根环两个根 [回退目标 {7→10, 4→20}, 现行]，窗口 2、F 为 0。往槽 20 写（搬迁的目的写）`ring_step_enabled` 为假。原样：

```
name=verus case=original-protocol-probe rc=0 verification results:: 14 verified, 0 errors|
```

  14 = 第二轮的 13 加这一条。所以变体一在原协议上被排除，靠的是 `Write` 的门 `ring_slot_is_unreferenced`（现行、每个候选、每个读者都要避开）。
- 区间版 P4 单独不排除：`verus/z1_bridge_probe.rs:8` 的 `the_interval_premise_alone_does_not_see_the_rollback_candidate` 证出：现行映射只有 key 7 在 [10,11)，计划搬到 [20,21)，`relocation_plan_is_well_formed`（第二轮正推 `relocation_interval_proofs.rs`）成立，而槽 20 是回退候选的。并入 `singlefs-replay-prefix` 副本整 crate：

```
name=z1_crate label=round-two-modules-only rc=0 verification results:: 110 verified, 0 errors
name=z1_crate label=with-runs-and-bridge rc=0 verification results:: 115 verified, 0 errors
```

  110 与第二轮正推报的相同；115 = 110 + 提升版协议 4 + 桥探针 1。
- 读法：判决说两处修补「采正推的改后协议」。变体一在两件拼起来的形态里排除，但排除发生在环协议的门上；区间 P4 的前提（目的区间避开现行映射）不含回退视图，两件之间没有一条引理接起来。攻方第二轮「打中一」说的缺桥，字面上还在；补法见 1.3（把区间放进环协议的门里，一件证完）。
- 推翻条件：入库装置上，环协议的 `Write` 门之外另有一条搬迁路径能写盘而不经 `ring_slot_is_unreferenced`（则变体一在原协议上也排除不了）。

### 1.2　变体二：两槽单元的第二槽照样可写

- 原协议的映射值是一个槽号（`gc_ring_proofs.rs` 的 `RootView.mapping: Map<u64, u64>`），避让只比这个槽号。同一条证明 `z1_original_probe.rs:486` 里：回退目标里 key 4 是两槽单元（真实占槽 20、21，协议里只能写成 4→20），往槽 21 写 `ring_step_enabled` 为真（上面同一行 `14 verified`）。原协议排除不了变体二，也说不出「两槽」。
- 提升到区间（`verus/gc_ring_runs.rs`，映射值改 `SlotRun { start, span }`，避让改成「没有任何区间罩住这个槽」`run_slot_is_unreferenced`，`gc_ring_runs.rs:87`；步骤、窗口、F、读者、Crash 其余照原文）：同一个状态里往 20、21 写和回收 21 都不合法，往 30 写合法（防空真），见 `variants_one_and_two_are_refused_by_the_run_protocol`（`gc_ring_runs.rs:276`）。原样：

```
name=verus case=runs-protocol rc=0 verification results:: 4 verified, 0 errors|
```

  4 条是：写与回收保持被保护视图、一步保持、任意有限历史保持（`theorem_run_ring_every_finite_history_keeps_every_view`，`gc_ring_runs.rs:253`）、两个变体的状态证明。
- 读法：判决 Y5 的条件「排除了才算这一格关上」对变体二：原协议上没关上；提升版关上。提升版只被我的模型量过，被攻过零轮，交用户时要写「零轮」。槽号与跨度用 int（同第二轮正推 `relocation_interval_proofs`），接 core 的 u64 另证，没做。
- 推翻条件：提升版里有一个 enabled 的步骤序列让某个视图读错（Verus 的 `theorem_run_ring_every_finite_history_keeps_every_view` 就是这条；若在入库装置上换了求解环境后变红即推翻）。

### 1.3　提升版的必红世界（三条都红）

把门改弱一处，Verus 必须红；落点是被改坏的那一条证明。原样：

```
name=verus case=world-start_only rc=1 error: assertion failed|verification results:: 2 verified, 2 errors|
name=verus case=world-no_candidates rc=1 error: postcondition not satisfied|error: precondition not satisfied|verification results:: 2 verified, 2 errors|
name=verus case=world-forced_to_zero rc=1 error: postcondition not satisfied|error: precondition not satisfied|verification results:: 2 verified, 2 errors|
```

- `start_only`：避让退回只比起点槽号（变体二的病根）。
- `no_candidates`：门里去掉回退候选（只剩现行与读者）。
- `forced_to_zero`：`Reclaim` 的门恒真，这是 `ReuseWindow::ForcedToZero`（回收无视 F）在模型里的样子。

### 1.4　G7 改写成「回退候选窗口移一格」之后，「带 GC 与不跑 GC 逐项相同」成立吗

用第二轮正推协议的逻辑层写了有界穷举（`tools/g7_window_shift.py` 与 `tools/g7_named_roots.py`）：用户步 UP / CP / RB / RF，整理步 GC 只在「现行内容与最后一个根相同」时插入（等于第二轮正推 `lemma_a_gc_checkpoint_only_shifts_the_window` 的前提），同一串用户步在不插 GC 和任意位置插 GC 两边各跑一遍，窗口 1 到 3，用户步长度到 5 全穷举，到 9 随机抽样。

- 第一版按内容点名（RB c：回退到内容为 c 的候选）：第二轮正推 5.4 的字面读法「整理那份比不整理那份多移 k 格」按条目比，不成立；去掉相邻重复的整理根之后比也有 8 例对不上，例：窗口 3，用户步 UP、CP、RB 0、CP、RF 0，整理插在 RB 之前，不整理窗口 [0,1,0]，整理窗口 [0]。病根是点名方式：整理多放了一个内容相同的根，把更早的那个同内容根挤出窗口，「抬 F 到内容为 0 的根」在两边指的不是同一个根。
- 第二版按版本号点名（用户只能点自己发布的根；整理的根没有用户点得到的版本号），原样：

```
name=g7_named_roots len=5 sample=None runs=18870233 b_accepts_a_rejects=0 a_accepts_b_rejects=6975104 head_differs=0 not_suffix=0 loss_gt_k=0 ceiling_diff_ne_k=0
name=g7_named_roots len=9 sample=rand:20000:7 runs=4133543 b_accepts_a_rejects=0 a_accepts_b_rejects=1497181 head_differs=0 not_suffix=0 loss_gt_k=0 ceiling_diff_ne_k=0
```

  读法（只对接受情况一致的运行比）：现行内容两边永远相同（`head_differs=0`）；整理那份窗口里用户点名的根是不整理那份的后缀（`not_suffix=0`）、至多少 k 个（`loss_gt_k=0`，k 是整理发布数）；整理那份从不接受不整理那份拒绝的回退（`b_accepts_a_rejects=0`）；抬 F 的上限恰多 k（`ceiling_diff_ne_k=0`）。`a_accepts_b_rejects` 那一大串是「窗口移出去的那些回退，整理那份拒绝了」，就是窗口移格本身。
- 判定：G7 的新说法要写成「现行内容相同；用户能回退到的版本集合是不整理那份的后缀，至多少 k 个；抬 F 上限多 k」，不是「逐项相同」，也不是「恰多移 k 格」。这条在有界域上没有反例，不是证明；对次数的归纳还没写成 Verus 引理（第二轮正推 5.4 自己写了「推的」）。
- 推翻条件：长度超过 9 或窗口超过 3 的历史上出现 `not_suffix` 或 `loss_gt_k` 非 0；或入库的第二台实现里整理发布会在 `head` 与最后一个根内容不同时发生（则 `lemma_a_gc_checkpoint_only_shifts_the_window` 的前提不成立）。

### 1.5　改后协议的前提，在 core 今天的代码里成立吗

协议里放行 `Write` 与 `Reclaim` 的门是「现行、每个回退候选、每个读者都没引用这个槽」。core 里对应的是一个按代数比的门槛函数 `released_record_can_be_reclaimed(released, generation, floor)`（`crates/singlefs-replay-prefix/src/gc_crash_proofs.rs:246`），由 `reclaim_released_records_up_to` 调（`crates/singlefs-core/src/allocator.rs:2561`）。三条前提逐条看（行号是冻结副本的）：

1. **门槛 ⇒ 没被引用：没人证。** 核自己写着「真实分配器必须另证其回收条件蕴涵这一条件；本模型没有从 floor 推出它。」（`crates/singlefs-replay-prefix/src/gc_crash_proofs.rs:70`）。第二轮正推的新协议与我的提升版都以「没被引用」为门，没有一条把 `generation <= floor` 接上它；分配器那一阶段的义务（第二轮正推 3.4 的表）还是空的。
2. **`floor` 的来历：`ForcedToZero` 让它不成立，只供测试。** `ReuseWindow::ForcedToZero`（`allocator.rs:1103`）在 `release_leaving_the_record_allocated_on`（`allocator.rs:2206`）的 `ForcedToZero` 分支（`allocator.rs:2241`）里把释放代本身当 `floor` 调同一个回收函数，门槛函数对任何输入都把刚释放的记录放行，F 不起作用；协议里对应的是 `Reclaim` 门恒真，上面 1.3 的 `world-forced_to_zero` 红。拨它的 `set_reuse_window`（`allocator.rs:2048`）是 pub，非测试代码里没有调用者（`grep -rn set_reuse_window crates --include=*.rs`，滤掉 `/tests/` 之后只剩这一行定义；主工作区与冻结副本相同）。所以今天产品路径上前提成立，缺的是类型层面的保证：pub 方法、`PoolAllocator` 在挂载结果里是 pub 字段，任何调用方都能拨。判决 Y3 把这条「进第三轮必攻」，这里的结论是：它要么挪进默认关的 cargo 特性，要么写进前提并列进「未连接前提」，二选一不影响协议。
3. **抬 F 分两段，协议是一步。** 协议的 `RaiseFloor` 是原子的，`Crash` 不回退 `floor`（第二轮正推 `lemma_a_crash_keeps_the_rollback_candidates_protecting_their_slots`）。core 里 F 先在内存抬、回收出来的槽扣住（`hold_until_floor_takes_effect`，`allocator.rs:2584`；带新 F 的根落满每块盘之后 `release_reclaim_holds` 放开，`allocator.rs:2593`），能立即复用的只到「每块盘上最新有效根」之下（抬 F 序列里 `floor_of_the_reclaim.min(newest_valid_root_on_every_device)`，`mount.rs:4972`；挂载重建分配器时同形，`mount.rs:3426`）。这套机制保证崩溃之后 F 回到持久那一值时被回收的槽还没被覆写，是协议「`floor` 一步持久」这条白拿的前提的实现。它在 `raise_rollback_floor`（`mount.rs:4200`）与 `reclaim_the_released_slots_of_the_rebuilt_allocator`（`mount.rs:3385`）里，两个都在三个闭包外（第三节）。我只读了代码，没建「内存 F / 持久 F」两层的模型（推的）。

推翻条件：(1) 若第三轮之后的实现把 `generation <= floor` 的蕴涵证出来，第 1 条消失；(2) 产品代码里出现 `set_reuse_window` 的调用者，第 2 条变成真违反；(3) 两层 F 的模型里门只看内存 F 时历史仍保持读对，则第 3 条那套机制是多余的。

## 二　Z2　信任清单与门禁的判法撑不撑得住

判决原文：「去掉 `--no-cheating`」「`axiom fn` 一律不许」「vstd 自带的那批按 vstd 版本整体登一行」（`research/prompts/m4-exit5-relocation-proof-r2-main-verification.md:20`）；「被验区域里不许出现按 `verus_keep_ghost` 分叉的函数体」「产品构建与 Verus 看到的 MIR 逐函数相同那一格进门禁」（同文件 `:19`）。

### 2.1　还有哪些写法让 Verus 验过、产品违 ensures、门禁与清单都不红

每一行是一个小 Verus 文件（`verus/z2_*.rs`）加产品构建（`z2crate/`，用与第二轮攻方同样的 crates.io 六个包擦 `verus!`）。Verus 都带 `--no-cheating --rlimit 10`；产品构建两趟：debug 默认特性，release 加 `--features wide`。原样：

```
name=verus case=f1_feature_fork rc=0 verification results:: 1 verified, 0 errors|
name=verus case=f1_feature_fork-with-cfg-feature-wide rc=1 error: possible arithmetic underflow/overflow|error: postcondition not satisfied|verification results:: 0 verified, 1 errors|
name=verus case=f1b_debug_assertions_fork rc=0 verification results:: 1 verified, 0 errors|
name=verus case=f5_macro_outside rc=0 verification results:: 1 verified, 0 errors|
name=verus case=f2_trait_impl_outside rc=0 verification results:: 1 verified, 0 errors|
name=verus case=f3_unsafe rc=0 verification results:: 1 verified, 0 errors|
name=verus case=f4_default_method rc=1 error: postcondition not satisfied|verification results:: 1 verified, 1 errors|
name=z2_product_run profile=debug features=none fn=f5_macro_outside::pick returned=11 ensures_holds=false
name=z2_product_run profile=debug features=none fn=f2_trait_impl_outside::read_checked returned=99 ensures_holds=false
name=z2_product_run profile=release features=wide fn=f1_feature_fork::pick returned=11 ensures_holds=false
name=z2_product_run profile=release features=wide fn=f1b_debug_assertions_fork::pick returned=11 ensures_holds=false
```

| 写法 | Verus | 产品 | 判决的检查红吗 |
|---|---|---|---|
| `#[cfg(feature = "wide")]` 两份函数体（`z2_f1_feature_fork.rs`） | 默认 cfg 验过；带 `--cfg feature="wide"` 红 | `--features wide` 返回 11，ensures 要 12 | 不红：判决只扫 `verus_keep_ghost` |
| `if cfg!(debug_assertions)` 分叉（`z2_f1b_debug_assertions_fork.rs:8`） | 验过（Verus 取 debug_assertions 为真的那支） | release 返回 11 | 不红 |
| 宏在 `verus!` 外面定义、展开里带 `cfg!(verus_keep_ghost)`（`z2_f5_macro_outside.rs:3`） | 验过 | 默认构建返回 11 | 不红：被验文本里没有 cfg 字样 |
| trait 的 ensures 只在 `verus!` 里的 impl 上核，`impl Device for BadDevice`（`z2_f2_trait_impl_outside.rs:15`）写在 `verus!` 外面，泛型函数照 trait 的 ensures 证过 | 验过 | 返回 99，ensures 要 `<= 10` | 不红：清单四类都不涉及 |
| `verus!` 里的 `unsafe { 1 }`（`z2_f3_unsafe.rs`） | 验过 | — | core 的 `#![deny(unsafe_code)]`（`crates/singlefs-core/src/lib.rs:12`，主工作区）挡；冻结副本里放开的是两处：`crates/singlefs-core/src/block_device.rs:565` 的打洞函数与 `crates/singlefs-core/src/checksum.rs:108` 的硬件 CRC 指令路径，都归 D 类 |
| 默认方法被 `verus!` 里的 impl 覆盖而违 ensures（`z2_f4_default_method.rs`） | 红 | — | 站得住：Verus 自己核 |
| `#[verifier::external]` 的函数被验函数调用（`z2_g2_external_attr.rs`） | 拒：「cannot use function … which is ignored」 | — | 站得住 |
| `external_trait_specification` 给 std 特质方法写假 ensures（`z2_g1_trait_spec.rs`） | 声明处不报错；我的调用点探针撞上 `ParseIntError` 不支持，没做通干净的一例 | — | 清单四类不含它；线索，不算打中 |

这些落在 core 里是什么量级：闭包里有 feature 分叉的 fn 函数体 3 个、`not(feature)` 的 2 个（用第二轮攻方的 `cfg-in-closure.py` 在我自己的闭包文件上复出：`feature="lookup-work-counters"` 3 个、`not(feature="lookup-work-counters")` 2 个，共 1071 个 core fn；谓词里带 `target_arch` 的另有 2 个，是上面硬件 CRC 那一路，我在自己的闭包文件上数 cfg 谓词出现在哪些 fn 里：feature 3、target_arch 2，合计 5 个 fn），harness 与 checker 档都打开这个特性（`crates/singlefs-harness/Cargo.toml:18`、`crates/singlefs-checker-tier/Cargo.toml:44`，冻结副本），Verus 按默认特性验。`BlockDevice` 的实现冻结副本里 core 自己 7 个（其中 3 个在测试模块里），harness 34 个，checker 档 12 个；`PoolReader` 的实现 core 10、harness 9、checker 档 9（`grep -rn 'impl[^{]* BlockDevice for'` 的计数）。故障注入的设备就是故意违反设备契约的实现，给 `BlockDevice` 加带 ghost 日志的 ensures 之后，这些实现只要在 `verus!` 外面就不被核。

判定：Y2 的信任清单站得住，Y1 的 cfg 判红够不着——它挡的是一个原子，同形写法至少还有 feature、`debug_assertions`、宏里的 cfg 三种，外加 verus! 外面的 trait impl 一整类。出路（推的，没实现）：被验区域里的 cfg 谓词除 `verus_keep_ghost` 外一律判红或要求进信任清单并列出「产品构建与 Verus 同值」；被验 trait 的所有 impl 必须在被验区域里或登记为信任边界；扫描范围包括被验 crate 里定义的宏。
推翻条件：入库的门禁已扫 feature / debug_assertions / 宏体里的 cfg；或 Verus 对 `verus!` 外面的 trait impl 报错（这一版不报）。

### 2.2　「产品构建与 Verus 的 MIR 逐函数相同进门禁」能不能看见 2.1 的分叉

第二轮正推的比法（`research/prompts/m4-exit5-relocation-proof-r2-sonnet-output.md:129`）是原版与接法各出一份产品构建的 MIR 再比：两份都不带 `verus_keep_ghost`，所以比得出「`verus!` 包装有没有改产品代码」，比不出「Verus 验的那份是不是产品编的那份」。2.1 的宏写法就是反例：产品构建（含 `verus!` 包与不包）都返回 11；让 Verus 自己编可执行代码（`verus --compile`，`verus_keep_ghost` 为真）返回 12。原样：

```
name=z2_verus_compile rc=0 verification results:: 1 verified, 0 errors
name=verus-compiled-run pick=12
name=z2_product_run profile=debug features=none fn=f5_macro_outside::pick returned=11 ensures_holds=false
```

判定：「MIR 比对」按第二轮正推的做法够不着 cfg 分叉；要比的两侧是「产品构建」与「`verus --compile` 编出的东西」，这一版里后者能产出可执行文件（上面 12），我没试它能不能吐 MIR，所以比的单位是 MIR 还是跑出来的值，这里不判。推翻条件：`verus --compile` 编出的可执行代码里 `verus_keep_ghost` 为假（则 12 变成 11，这条比法也看不见分叉）；这一版量到的是真。

### 2.3　差分测试能不能判出一条写错了的 std 规格（给一例）

被试规格取第二轮攻方的 `<[T]>::binary_search`：写成「`Ok(i)` 时 i 是第一个等于 x 的下标」，看着对，std 只保证「任一个」。做法照第二轮正推 2.4：规格谓词写成可执行孪生，在有界域（字母表 {0,1,2}、长度 0 到 7 的全部有序切片、x 取字母表全部）上逐值喂给真 std，核孪生（`std-diff/std_spec_differential.rs`）。原样：

```
name=std_spec_differential domain=sorted slices over {0,1,2} len<=7 cases=360 wrong_spec_failures=168 weaker_true_spec_failures=0
name=std_spec_differential first_counterexample slice=[0, 0] x=0 std_returned=Ok(1)
```

写错的规格 168 例红，只保证「任一个」的真规格 0 例红：测试判得出这一条，也不误红正确的规格。必红世界（第二轮攻方的 `first_index_of` 那一份）证的是「规格被用到了」，抓不到它——两条是不同的把关。局限（沿用第二轮正推 2.4）：有界域；含量词、`Seq`、`Map` 的规格写不成孪生；`external_body` 的设备读写与校验和没有 std 对照，要另立契约测试。推翻条件：入库装置上写错的规格在差分域内有反例而测试不红，或真规格被误红。

### 2.4　同样打中核外提的部分

- cfg 分叉：今天门禁对 `verus_keep_ghost` 零命中（`name=z2_gate_scan verus_keep_ghost_hits_in_verus_proof_check_py=0`），`singlefs-replay-prefix` 的 `Cargo.toml` 自己登记了这个 cfg（`crates/singlefs-replay-prefix/Cargo.toml:15`）：今天已验的 88 条证明的核同样可以写这种分叉。判决 Y1 的修补是两个臂共用的，不是 core 直验独有的成本。
- rustc 版本差（Verus 的 1.98.1 对产品的 1.98.0，判决 Y1）：产品编 replay-prefix 用同一个产品工具链，同样适用。
- 2.1 的 feature 原子、`debug_assertions`、宏里的 cfg：replay-prefix 里同样写得出；区别在量：core 的闭包里有 5 个 fn 带 cfg 谓词，replay-prefix 小得多，我没数。
- trait impl 在被验区域外：核外提的核是纯函数（没有带 ensures 的 trait），这一类是 core 直验独有的成本（`BlockDevice` 在 core / harness / checker 档共 53 个实现）。
- 差分测试：核外提不需要它（纯函数不 `assume_specification` std），core 直验才需要；判决 Y2 的成本是 core 直验独有的。

### 2.5　vstd 自带那批

vstd 0.0.0-2026-09-20-0158 源码里 `assume_specification` 487 行、`axiom fn` 184 行、`verifier::external_body` 283 行（`grep -rE` 数行，不是条目数）。判决「按 vstd 版本整体登一行」站得住；第二轮攻方说它不审，这是真的，也是今天 88 条证明已经在信任的东西（核里 `Vec` / `Seq` / `Map` 的用法，`grep -cE` 在 `crates/singlefs-replay-prefix/src/lib.rs` 上 406 行），所以同样打中核外提，不是 core 直验新增的洞。推翻条件：vstd 换版本而清单没跟着重登。

## 三　Z3　射程的完整性

判决原文：「射程加崩溃后读映射那一路」（`research/prompts/m4-exit5-relocation-proof-r2-main-verification.md:21`）。下面把三个闭包记作 U2 = 发布 ∪ 回退，U3 = U2 ∪ 恢复（`recovery::recover`）。闭包都是 `python3 -I scripts/gate/lib_function_units.py closure <冻结副本> <入口>` 现收（第二轮攻方用的就是这一份，它与 `crates/singlefs-unit-graph` 逐单元对拍），入口写在 `tools/rerun.sh` 的 `step_z3`，原样输出在 `out/z3.txt`。

### 3.1　闭包与闭包之外（命令的原样输出）

```
name=z3_closure entry=publish rc=0 fn=836 units=1903 refused=0
name=z3_closure entry=roll_back_by_a_forward_publish rc=0 fn=1057 units=2292 refused=0
name=z3_closure entry=recover rc=0 fn=654 units=1558 refused=0
name=z3_union U2_fn=1073 U3_fn=1185 recover_only_fn=112 recover_only_lines=4837
name=z3_entry entry=run_compaction closure_fn=952 outside_U3_fn=115 outside_U3_lines=2984
name=z3_entry entry=discard_segments closure_fn=198 outside_U3_fn=13 outside_U3_lines=259
name=z3_entry entry=raise_rollback_floor closure_fn=1061 outside_U3_fn=69 outside_U3_lines=2142
name=z3_entry entry=raise_floor_to_ceiling closure_fn=1063 outside_U3_fn=71 outside_U3_lines=2192
name=z3_entry entry=sweep_end_of_mount closure_fn=895 outside_U3_fn=22 outside_U3_lines=728
name=z3_entry entry=scan_region closure_fn=137 outside_U3_fn=13 outside_U3_lines=321
name=z3_entry entry=mount_writable closure_fn=1469 outside_U3_fn=315 outside_U3_lines=10289
```

恢复只比 U2 多 112 个 fn，其中 30 个是 core 的（1521 行，第二轮攻方的数，我复出），82 个是 `singlefs-replay-prefix` 里已验的核 fn（3316 行）。所以判决 Y3「30 个 fn、1521 行」说的是 core 新增，复出。

### 3.2　回收门槛的调用点：七个里三个在三个闭包外

门槛函数 `released_record_can_be_reclaimed`（`crates/singlefs-replay-prefix/src/gc_crash_proofs.rs:246`）是被产品执行的那一个验过的核函数。扫非测试 fn 里对它和它的包装 `reclaim_released_up_to*` / `reclaim_released_records_up_to` 的调用（`tools/z3_membership.py`，原样）：

```
name=z3_gate_caller where=U2 fn=allocator::<impl PoolAllocator>::release_leaving_the_record_allocated_on file=allocator.rs:2206-2248
name=z3_gate_caller where=U2 fn=allocator::<impl PoolAllocator>::reclaim_released_up_to file=allocator.rs:2504-2518
name=z3_gate_caller where=outside fn=allocator::<impl PoolAllocator>::reclaim_released_up_to_isolating_what_the_isolation_still_protects file=allocator.rs:2524-2548
name=z3_gate_caller where=U2 fn=allocator::<impl PoolAllocator>::reclaim_released_records_up_to file=allocator.rs:2551-2590
name=z3_gate_caller where=U2 fn=allocator::<impl PoolAllocator>::record_root_written_by_this_process file=allocator.rs:2731-2810
name=z3_gate_caller where=outside fn=mount::reclaim_the_released_slots_of_the_rebuilt_allocator file=mount.rs:3385-3432
name=z3_gate_caller where=outside fn=mount::reclaim_released_in_the_sequence_raising_the_floor_up_to file=mount.rs:5174-5185
```

闭包外的两个 `mount.rs` 函数恰好是**算出 `floor` 的地方**：挂载重建分配器之后第一次回收（崩溃之后的第一次 `Reclaim`），和抬 F 序列里的回收。判决的射程把「门槛函数被谁调」留在闭包内，把「`floor` 怎么来」留在闭包外：门槛函数被证到什么程度都没用，它的 `floor` 实参不受证。这是 1.5 第 3 条的位置，也是判决 Y3 没点到的最重的一块。

### 3.3　改盘面字节的路：机械清单

判法（`tools/z3_writers.py`）：非测试 fn 里直接调 `BlockDevice` 的 `write_at` / `write_zeroes_at` / `discard_at`（trait 定义在 `crates/singlefs-core/src/block_device.rs:160`），看它在不在 U2 / U3。原样：

```
name=z3_writer where=outside fn=block_device::<trait BlockDevice>::discard_at file=block_device.rs:201-217 prims=discard_at
name=z3_writer where=outside fn=instance_table_sweep::erase_the_heads_and_publish_the_sweep file=instance_table_sweep.rs:532-693 prims=write_at
name=z3_writer where=outside fn=mounted_session::discard_the_segments_reusable_as_a_whole file=mounted_session.rs:2679-2753 prims=discard_at
name=z3_writer_total fn=18 U2=15 U3_only=0 outside=3
```

18 个写点，15 个在 U2 里，恢复闭包比 U2 多出来的 fn 里没有写点（`U3_only=0`），三个在闭包外：`discard_at` 的 trait 默认实现、`discard_the_segments_reusable_as_a_whole`（发布之后丢弃整段可再分配的槽）、`erase_the_heads_and_publish_the_sweep`（实例表清扫抹孤儿头并发清扫发布）。各自的门：丢弃用 `segment_is_reusable_as_a_whole`（`crates/singlefs-core/src/allocator.rs:864`，分配器空闲图状态）；清扫用实例表一行判「未发布」（`UnitHeadReadBySweep::is_unpublished_by_the_row`）。两个门都要另一条引理才接得上协议的「没被候选、读者引用」，它们现在不在射程里。丢弃自己的文档写着结局是三态之一、不能当擦除（`crates/singlefs-core/src/mounted_session.rs:2679` 前的注释），协议里 `Reclaim` 把字节整个删掉比丢弃更坏，所以协议对丢弃的字节后果是保守的，缺的只是门。

### 3.4　不必进证明的：选段、规划、区域扫描、GPU 判定、开关切换

- 整理主循环 `run_compaction`（`mounted_session.rs:2015`）闭包 952 个 fn，其中 115 个（2984 行）在 U3 外：`choose_region`、`plan_batch`、`advance_compaction_by_one_publish_on_the_cpu`、意图的续做与撤回（`compaction_plan.rs` 里 `withdraw_the_live_intent` 等）、写带宽闸。区域扫描 `scan_region` 13 个（321 行）。它们决定搬什么，不决定搬得对不对：条目进发布闭包之后由 `relocation_checked_against_the_version` 对着上一版映射核（`crates/singlefs-core/src/transaction.rs:10142` 冻结副本，主工作区此刻 `:10624`）。只要这个核对任何输入成立（区间 P4 的前提就从这里来），选段的错只让整理搬得不划算，不改恒等。同样适用核外提。
- `BackgroundGpuCompute`（只在主工作区，`crates/singlefs-core/src/background_compute.rs:239`）的三个方法交回的是判定（单元校验和、区域记录分类、每段活槽数），不写盘。主工作区闭包此刻（别的会话在改，数会漂：我前一次量是发布 844 个，这一次 854 个）：

```
name=z3_main_tree_closure entry=publish fn=854 units_naming_a_gpu_item=0
name=z3_main_tree_closure entry=rollback fn=1077 units_naming_a_gpu_item=0
name=z3_main_tree_closure entry=run_compaction_in_the_background fn=992 units_naming_a_gpu_item=14
name=z3_main_tree_closure entry=recover fn=654 units_naming_a_gpu_item=0
```

  发布与回退闭包里没有 GPU 单元，整理（后台）里 14 个，都在选段那一侧。挂载续做整理意图（`mount.rs` 里 `live_intent_to_continue`）与整理开关的运行期切换（`allocator.rs` 的 `set_compaction_switches`、`compaction_mut`）同属这一侧，只改整理自己的状态，不改抽象文件系统状态。
- 推翻条件：这一侧的某个 fn 能让一个条目绕过 `relocation_checked_against_the_version` 进发布（则它也要进射程）；或 GPU 判定会被 core 直接当作「已验过」写进映射而不经 CPU 核。

### 3.5　列清单（判法 + 要加进射程的）

要加进射程：(1) 抬 F 一路（`mount::raise_rollback_floor` 起，含 `reclaim_released_in_the_sequence_raising_the_floor_up_to`、`write_the_raised_floor_into_every_system_configuration`，闭包外 69 个 fn、2142 行，要加的是算 `floor` 与持久化 F 的那几个，不是整条）；(2) 挂载重建分配器的回收（`rebuilt_allocator`、`reclaim_the_released_slots_of_the_rebuilt_allocator`）；(3) 丢弃一路（13 个 fn、259 行）；(4) 实例表清扫抹头一路（22 个 fn、728 行）。不必加：3.4。
判法：写点清单（`z3_writers.py`）里 U3 外的写点都要在清单上有一行「由谁罩」；门槛函数的调用点（`z3_membership.py`）都要在射程里或登记「floor 实参来自 X」。
这两条机械判法只覆盖直接调写原语的 fn；经 trait 对象或闭包间接写的我没扫（`lib_function_units.py` 对动态分派本身会拒算，本次闭包 `refused=0`）。
推翻条件：入库后重跑 `z3_writers.py` 写点总数不是 18 或闭包外的不是这三个（代码变了，清单随之要改）。

### 3.6　测试开关 `set_reuse_window`

结论在 1.5 第 2 条：站得住，必攻已攻（协议里 `forced_to_zero` 世界红）。同样打中核外提：`released_record_can_be_reclaimed` 今天就被产品执行，`ForcedToZero` 的分支已经在调它（`crates/singlefs-core/src/allocator.rs:2241`），所以这条前提缺口在核外提那一叠里同样存在。

## 四　Z4　判器键与重判代价

判决原文：「判器闭包键在算指纹时去掉 Verus 的规格与证明块」「否则整库重判的代价写进交用户表」（`research/prompts/m4-exit5-relocation-proof-r2-main-verification.md:22`）。键的口径是 D13 已定项 17：「函数单元从这份判定的入口沿调用与引用收，连同引用的常量、类型、静态项与宏展开一起算」（`.claude/kb/decisions/13-验证路线.md:432`）。

### 4.1　只改证明，键变不变（在副本上改一处量）

被改的是 `allocation_record_tree::key_bytes_width`（发布、回退、判器三条闭包都含它），量 `singlefs_checker_tier::crash::judge_crash_image` 的闭包键（`lib_function_units.py key`，与第二轮攻方的 `key-variants.sh` 同一个入口）。`raw` 是现行算法，`stripped` 是先对源文件跑 `tools/strip_verus.py`（我写的去规格与证明块的原型）再算。原样：

```
name=judge-key variant=frozen raw=7e76314e8552a53efb6ffba49fdd23179799ac94d07f0eab7f0e05a88be4900a stripped=7e76314e8552a53efb6ffba49fdd23179799ac94d07f0eab7f0e05a88be4900a
name=judge-key variant=comment-only raw=7e76314e8552a53efb6ffba49fdd23179799ac94d07f0eab7f0e05a88be4900a stripped=7e76314e8552a53efb6ffba49fdd23179799ac94d07f0eab7f0e05a88be4900a
name=judge-key variant=wrapped-in-verus-without-ghost raw=7e76314e8552a53efb6ffba49fdd23179799ac94d07f0eab7f0e05a88be4900a stripped=7e76314e8552a53efb6ffba49fdd23179799ac94d07f0eab7f0e05a88be4900a
name=judge-key variant=wrapped-with-an-ensures raw=070444cae357beefb8aaf8667e20de83084a1de14ee7af30c5daebddaf82378d stripped=7e76314e8552a53efb6ffba49fdd23179799ac94d07f0eab7f0e05a88be4900a
name=judge-key variant=wrapped-with-an-ensures-and-a-proof-block raw=2efa80a4899459c24451b5d1c05353d418c1258c163cd60528268806bb3fb314 stripped=7e76314e8552a53efb6ffba49fdd23179799ac94d07f0eab7f0e05a88be4900a
name=judge-key variant=wrapped-with-spec-fn-proof-fn-ghost-let-and-assert raw=4b80affbf10b7d969189c25d0f7eaad2d8dc650093b2815504c9624036621da0 stripped=7e76314e8552a53efb6ffba49fdd23179799ac94d07f0eab7f0e05a88be4900a
name=judge-key variant=must-red-exec-string-changed raw=1872d8f173816490b4451d28baf683ba5f29771ee150fd9e8e39900ba12bdf47 stripped=9a0201172e5e259cd417a89639ff586b2164df05e42dd77bff215d8a748377f3
name=judge-key variant=must-red-exec-expression-changed raw=11c344b1da7afced9ef6fe3581eb152fbb419e45f8a19a0df8c75cfb67169b53 stripped=9a346171f4a2338aea7bc0553c27b12f774c100499e458ca919d833e963a0fc3
name=judge-key variant=must-red-exec-attribute-added raw=214b513868c65e943fa5de44ea8092b6bf0b07c1208a6bbb8cd81e2b6c38a794 stripped=3827ac033b5d217cafb674fc41bfcb26a22ec720c3ada776c59bcebd83345218
```

读法：`raw` 一列复出了第二轮攻方的三把键（7e76314e、070444ca、2efa80a4）；`stripped` 一列里只加注释、用 `verus!` 包一下、只加 ensures、加 ensures 与 proof 块、加 spec fn / proof fn / `let ghost` / `assert` 五种改法键都回到原键，三种改执行代码（字符串、表达式、`#[inline(never)]` 属性）的键都变。判决 Y4 的做法在这个原型上站得住。

### 4.2　会不会让真改了产品行为的改动被当成只改证明

- Verus 自己的模式检查拒绝「证明代码影响执行」的四种写法，原样（`verus/z4_h1…h4`，带 `--no-cheating`）：

```
name=verus case=h1_proof_calls_exec rc=1 error: cannot call function `z4_h1_proof_calls_exec::bump` with mode exec|
name=verus case=h2_ghost_in_exec_branch rc=1 error: cannot access spec-mode place in executable context|
name=verus case=h3_ghost_param_selects_exec rc=1 error: cannot perform operation with mode spec, Unary(CoerceMode { op_mode: Spec, from_mode: Spec, to_mode: Spec, kind: Field }, SpannedType|
name=verus case=h4_proof_block_assign_exec_var rc=1 error: cannot mutate exec-mode place in proof-code|
```

  即 proof 块里调有副作用的 exec 函数、`ghost` 变量参与 exec 分支、`Ghost<T>` 参数的值带进 exec 返回值、proof 块里给 exec 变量赋值，Verus 都拒。所以判决问的两个例子在被验通过的代码里造不出来；它们成立要 Verus 的模式系统本身有洞，那属于可信部分（工具链钉死）。`Ghost<T>` 类型的参数仍然是产品签名的一部分（零大小类型），原型不把它剥掉，加一个这样的参数键会变，偏保守。
- 局限一：我的 `strip_verus.py` 是文本级，覆盖 4.1 那几种写法；对 `crates/singlefs-replay-prefix/src/lib.rs` 整份 `verus!` 块剥完后 `rustfmt` 报错（`name=z5_erased file=lib.rs verus_block_code_lines=3824 erased_rust_lines_rustfmt=-1 ratio=3824.00 rustfmt_rc=1`，`out/z5.txt`），原因是 spec fn 头里的 `decreases` 与结构体字段把条目切分带歪。所以「去掉规格与证明块」要换成 Verus 语法树（`verus_syn`，登记表里已钉）做的，不能是文本级；判决要写这一条。
- 局限二：键有两份实现要一起改，`scripts/gate/lib_function_units.py` 与 `crates/singlefs-unit-graph`，门禁 code-tooling 的 function-units-two-implementations-agree 格逐单元对拍两边（`lib_function_units.py` 文件头）。
- 判决漏写一条：形式化判定自己的键必须保留规格与证明。`scripts/crash/lib_verdict_library.py` 文件头说「门禁 checker-tier-formal-verification 的两份判法」用同一套函数单元闭包键（`scripts/crash/lib_verdict_library.py:7`）；若剥规格是整套键的全局行为，改一条 ensures 键不变、复用旧的形式化判定，是假绿。要写成「剥规格只用于崩溃与 QEMU 等判产品行为的判定，形式化判定的键保留规格与证明」，并配一格必红：改一条 ensures 形式化键必须变。

推翻条件：Verus 对 4.2 四种写法之一放行；或 `verus_syn` 做的剥离在 core 上让某个只改执行代码的改动键不变（必红格：改一处执行代码键必须变，4.1 后三行就是这个形状的原型）。

### 4.3　今天的核已经有这个代价（同样打中核外提）

第二轮攻方说判器闭包里有三个 Verus 核 fn。现数不止：

```
name=judge-closure-replay-prefix-units fn=83 lines=3322 proof_fn=23 proof_lines=1536
name=judge-closure-units-in-the-two-proof-only-files count=0
name=judge-key-lib-lemma-edit variant=frozen key=7e76314e8552a53efb6ffba49fdd23179799ac94d07f0eab7f0e05a88be4900a
name=judge-key-lib-lemma-edit variant=one-assert-true-added-in-lemma_applying_a_record_twice_in_a_row_is_applying_it_once key=9383df24031f04c68a91bf61299761eb57760c6973fe96c1f3a16289d9eb78c3
```

判器闭包今天就含 `singlefs-replay-prefix` 的 83 个 fn 单元（3322 行），其中 23 个是 `lemma_` 一类证明 fn（1536 行）；在已验的 `lib.rs` 里给一条引理多写一句 `assert(true);`，判器闭包键就变。两个只放证明的文件（`compaction_plan_proofs.rs`、`gc_crash_proofs.rs`）一个单元都不在判器闭包里，所以放在那里的新证明不触发。结论：这个代价不是 core 直验独有的，今天的核外提那一叠已经付着（83 个单元对 549 个）；判决的「去掉规格与证明块」修的是两个臂共有的问题，对已验的 `lib.rs` 也有好处。

### 4.4　做不到时整库重判的代价（推的数，用现成数）

- 并集 1071 个 core fn 里有 549 个同在判器闭包里，judge 闭包 core fn 589 个（用第二轮攻方的 `closure-analysis.py` 在我自己的闭包文件上复出，没进我的 rerun：`name=closure-overlap core_fn_in_union_and_in_judge_crash_image=549 of=1071 judge_fn_total=1298 judge_core_fn=589`）。
- 键是整个判器闭包一把，改 549 个里任何一个的任何字符（含 ensures、proof），库里按这把键存的判定全部作废；不是按 549 分之几的比例作废。
- 库的体量（`~/.local/share/singlefs/crash-amplification/crash-state-verdicts`，`find … -type f` 数、`du -sh`）：1368 个 SST 文件、21G。库里现有块数我没能现数：读库要 checker 档的二进制 `crash_library_reader`，属重型，不起。
- 一次整库作废要重判多少态，用两个现成的数夹出来：下界取 D13 已定项 17 依据里 2026-10-13 实测的那一次无谓整库作废「整库一把的判法摘要一换累计多判 3 935 242 934 态」（`.claude/kb/decisions/13-验证路线.md:446`），约 3.9×10⁹；上界取出口 10 写的全域「全域约 3.8×10¹⁰ 态」（推的，`.claude/kb/milestone/04-fourth-txn.md:25`）。对比同一行的配额「每个里程碑至少 100 亿态」：一次整库作废约等于 0.4 到 3.8 个里程碑配额。
- 全验按 A、B、C、D 四批派，每批至少改一次射程内 fn 的证明，现行键下至少四次整库作废：约 1.6×10¹⁰ 到 1.5×10¹¹ 态（4 × 上面两个数，推的，没有现成的实测）。
- 判据：只要「剥规格与证明块」的键做成，这个数就不进交用户表；做不成再交。

## 五　Z5　第二轮报的数能不能核

判决原文：「总代价推的约 11 万–12.5 万行」（`research/prompts/m4-exit5-relocation-proof-r2-main-verification.md:21`）。

### 5.1　1073 与 1071 差在哪

两边用的是同一个闭包（正推用 `crates/singlefs-unit-graph`，攻方与我用 `scripts/gate/lib_function_units.py`，两边发布 836、回退 1057 一致）。差在计的对象：1073 是并集里所有 fn 单元，1071 是其中 `singlefs_core::` 开头的；差的两个是 `singlefs-replay-prefix` 里已验的核 fn。原样（`out/z5.txt`）：

```
name=z5_union publish_fn=836 rollback_fn=1057 union_fn=1073 shared=820 only_publish=16 only_rollback=237
name=z5_union_by_crate singlefs_core=1071 singlefs_replay_prefix=2
name=z5_union_lines span_lines=24970 code_lines=24346 core_span_lines=24958
name=z5_union_noncore fn=singlefs_replay_prefix::gc_crash_proofs::released_record_can_be_reclaimed at=crates/singlefs-replay-prefix/src/gc_crash_proofs.rs:246-251
name=z5_union_noncore fn=singlefs_replay_prefix::records_at at=crates/singlefs-replay-prefix/src/lib.rs:506-511
```

这两个核 fn 不是要新验的东西。要新验的是 1071 个 core fn、24958 行（起止行数；去掉空行与 `//` 注释行是 24346 行含那两个核 fn 的）。判定：两边数都对，差别是口径；以后写「core 新验 1071 个 fn」更准。

### 5.2　A–D 分类与 std 调用键：照命令现量能不能复出

命令写在第二轮正推 `tools/rerun.sh` 的 `step_y3`、`step_y2`：`cargo rustc -p singlefs-core --lib -- --emit=mir` 出 MIR，`tools/mir_callees.py` 抽调用，`tools/closure_classes.py` 与 `tools/std_coverage.py` 对闭包文件分类。我用自己的闭包文件（`lib_function_units.py` 收的，不是正推的 `unit-graph`）喂同样的脚本，原样：

```
mir fn=5557 calls=22902
闭包 fn 单元（去重后）1073 个；MIR 里没配上的单元 185 个（归 A：空函数、trait 方法的默认体、或名字没配上）
  类 A: fn 单元 472，行数 5238
  类 B: fn 单元 190，行数 3340
  类 C: fn 单元 291，行数 10160
  类 D: fn 单元 120，行数 6232
  合计 fn 单元 1073，行数 24970
std 相关的不同调用键 456；按名字配上 vstd 规格 224；没配上 232
调用点 4628；落在配上规格的键上 3813；落在没配上的键上 815
```

A 472 / B 190 / C 291 / D 120 与行数、456 个键、配上 224（49%）、调用点 82% 全部复出，也就是闭包文件换了一份实现数出来结果不变。判法本身的性质没有变，要写进用到这些数的地方：按名字配不按类型配；185 个没配上名字的单元归了 A（A 是上界，名字配上的 A 只有 287）；只看直接调用，一个 A 类 fn 调了 C 类 fn 实际是 C 的依赖。是估数，不是 Verus 的判。

### 5.3　「11 万–12.5 万行」的比例取自哪里，按现数重算

推法是 24970 行乘 4.5–5 倍；第二轮正推 3.5 节说倍数「取自两个原型」：`relocation_judgement.rs` 175 行对应纯 Rust 约 35 行、`region_is_empty_by_the_records` 7 行变 32 行。这两个都是它自己新写的，不是现成核；那个「约 35 行」是估的。现成核的现数（`out/z5.txt`，`tools/proof_ratio.py` 与 `tools/erased_lines.py`；前者数「verus! 块里去掉空行与注释的全部行 / 其中 exec fn 的行」，后者数「verus! 块行 / 去掉规格与证明块后 `rustfmt` 排版的纯 Rust 行」）：

```
name=z5_proof_ratio file=compaction_plan_proofs.rs code_lines=944 exec=148 spec=301 proof=454 other=41 ratio_all_over_exec=6.38 ratio_nonexec_over_exec=5.38
name=z5_proof_ratio file=gc_crash_proofs.rs code_lines=224 exec=6 spec=48 proof=153 other=17 ratio_all_over_exec=37.33 ratio_nonexec_over_exec=36.33
name=z5_proof_ratio file=lib.rs code_lines=3826 exec=2362 spec=761 proof=644 other=59 ratio_all_over_exec=1.62 ratio_nonexec_over_exec=0.62
name=z5_proof_ratio file=relocation_judgement.rs code_lines=146 exec=115 spec=20 proof=9 other=2 ratio_all_over_exec=1.27 ratio_nonexec_over_exec=0.27
name=z5_erased file=compaction_plan_proofs.rs verus_block_code_lines=942 erased_rust_lines_rustfmt=110 ratio=8.56 rustfmt_rc=0
name=z5_erased file=relocation_judgement.rs verus_block_code_lines=144 erased_rust_lines_rustfmt=57 ratio=2.53 rustfmt_rc=0
name=z5_erased file=gc_crash_proofs.rs verus_block_code_lines=222 erased_rust_lines_rustfmt=18 ratio=12.33 rustfmt_rc=0
name=z5_erased file=lib.rs verus_block_code_lines=3824 erased_rust_lines_rustfmt=-1 ratio=3824.00 rustfmt_rc=1
```

（`out/z5.txt` 里还有 `gc_ring_proofs.rs`、`relocation_interval_proofs.rs` 两个纯协议文件，exec 为 0，比例没有意义，这里没贴。）

读法：
- 同一个 `relocation_judgement.rs`：144 行 verus 块对擦掉规格与证明后 57 行，是 2.53 倍，不是「175 对 35」的 5 倍；那份纯 Rust 的 35 行是估的，现数 57。
- `compaction_plan_proofs.rs`（选段核与计划核的证明）是 8.56 倍；`lib.rs`（replay_core 一类，exec 为主）用另一种量法（全部行 / exec 行，exec 含子句）是 1.62 倍，文本级剥离器对它失败所以没有 8.56 一类的同口径数。`gc_crash_proofs.rs` 是协议文件，擦掉只剩 18 行，不代表翻译旧代码。
- 现数的倍数从 1.6 到 8.6，不是一个 4.5–5。按闭包的 24346 个代码行乘：约 3.9 万到 20.8 万行。11–12.5 万落在里面，但这个点值不由现成核定出，它由正推自己的两个原型定。口径还有一处不匹配：擦除后的行数是 `rustfmt` 排版的，闭包的 24346 是原文行，两者差几个百分点，我没校。
- 判定：「约 11 万–12.5 万行」当作量级可以留，要写成「3.9 万到 20.8 万行，随 spec 与 proof 占比摆动，11 万–12.5 万是按正推两个新原型的倍数取的点」，并且仍是推的，没有一个阶段做完过（第二轮正推 3.5 节自己写了这一点）。
- 推翻条件：判定核阶段（第二轮正推 3.4 表的第二阶段）接线做完后，实际 verus 行对擦除行的倍数落在 1.6–8.6 之外。

## 六　判决的结论同样打中被否的「核外提」那一叠吗（汇总）

「core 直验」指用户 2026-10-26 定的做法；「核外提」指第一轮被否的那一叠（见〇一节开头）。

| 判决（第二轮第二节）里的条目 | 打中 core 直验 | 打中核外提 | 依据 |
|---|---|---|---|
| cfg 分叉（`verus_keep_ghost`）判红、Verus 与产品 rustc 版本差 | 是 | 是：今天的 88 条证明同样暴露，门禁对它零命中 | 2.4 |
| feature / `debug_assertions` / 宏里的 cfg、verus! 外的 trait impl | 是 | cfg 三种是；trait impl 一类核外提的纯函数核没有 | 2.1、2.4 |
| std 规格覆盖（456 键、49%）、`assume_specification` 信任清单、差分测试 | 是（独有） | 不是：纯函数核不 `assume_specification` std | 2.3、5.2 |
| vstd 自带的 487 行 `assume_specification` 假定 | 是 | 是：今天已在信任 | 2.5 |
| 射程缺 `floor` 的来历（抬 F、挂载重建回收）、丢弃、清扫 | 是 | 是：被产品执行的门槛核同样不管 `floor` 的来历；核外提的射程更小，缺得更多 | 3.2、3.3 |
| 测试开关 `set_reuse_window` | 是 | 是 | 1.5、3.6 |
| 判器闭包键随证明变 | 是（549 个 core fn） | 是：判器闭包今天含 83 个核单元、23 个证明 fn，改一句 `assert(true);` 键就变 | 4.3、4.4 |
| 按全路径逐项验（598 个 impl / trait 方法、82 个共用名字） | 是（独有） | 不是：核是平铺的少量 fn | 第二轮攻方 Y4 打中一，我复出数 |
| 改后协议、区间 P4、G7 窗口移一格 | 是 | 是：协议是被证的对象，与做法无关 | 一 |
| 「约 11 万–12.5 万行」 | 是（独有） | 不是：核外提只验判定核（第一轮量到接上之后核内占比 0.28%），量级小得多 | 5.3 |

## 七　给判决措辞的建议（辩方复核的结果，不替主 agent 撤销或维持）

1. Y5「排除了才算这一格关上」：变体一在原协议上关上（14 verified），变体二在原协议上没关上，在提升到区间的协议上关上（4 verified、三个必红世界红）；区间 P4 与环协议之间的桥，用提升版一件证完比另补一条引理省（推的）。提升版被攻过零轮，交用户表标「零轮」。
2. G7 改写：把「带 GC 与不跑 GC 逐项相同」改成「现行内容相同；用户点名的回退候选是不整理那份的后缀、至多少 k 个；抬 F 上限恰多 k」，并写明按版本号点名（按内容点名会出 8 个假反例）。
3. Y2 信任清单旁加三条：被验区域里的 cfg 谓词除 `verus_keep_ghost` 外一律判红或进清单并要求产品构建与 Verus 同值；被验 trait 的所有 impl 必须在被验区域内或登记为信任边界；扫描范围含被验 crate 里定义的宏。
4. Y1 的 MIR 比对：比的两侧写成「产品构建」对「`verus --compile` 编出来的东西」，不是原版对包装后的两次产品构建。
5. Y3 射程：除三个闭包外，加抬 F 一路（算 `floor` 与持久化 F 的几个 fn）、挂载重建分配器的回收、丢弃、实例表清扫抹头，共四块，各自要一条「门 ⇒ 没被候选与读者引用」的引理；选段、规划、区域扫描、GPU 判定不必进。
6. Y4：剥规格与证明块只用于判产品行为的判定，形式化判定的键保留规格与证明（配必红：改一条 ensures 形式化键必须变）；剥离用 Verus 语法树；两份键实现一起改。
7. 「约 11 万–12.5 万行」改写成区间并写明点值的来历；A 类 472 写成「上界，名字配上的 287」。

## 要主 agent 跑的命令

无（我没有需要重型或后台的命令）。复跑命令在文首，五段各一次，每段末行 `name=rerun_result mismatches=0`。

## 没做什么

- 没起任何门禁阶段、没起 `scripts/gate/verus-proof-check.py`、没跑 checker 档测试（包括读判定库用的 `crash_library_reader`）、没起 GPU；直接起的 `verus` 与 `cargo` 都在本机、经 `scripts/hosts/capped.sh 31` 与内存包装。第二台没有 Verus，没用。
- 没替主 agent 撤销或维持判决。没读禁读清单里的文件。
- Z1：没建「内存 F / 持久 F」两层的协议（1.5 第 3 条是读代码的推的）；提升版协议的槽号与跨度用 int，没接 core 的 u64；G7 的对次数归纳没写成 Verus 引理，只有有界穷举。
- Z2：没实现任何门禁改法；`external_trait_specification` 的调用点探针撞上 `ParseIntError` 不支持，没做通一例干净的；`verus --compile` 能不能吐 MIR 没试（只比了跑出来的值）；经 trait 对象或闭包间接写盘的点没扫。
- Z3：GPU 一路只读了 trait 签名和闭包里的单元名，没读 GPU 实现；主工作区的数随别的会话在改（发布闭包 844 到 854），没冻结。
- Z4：`strip_verus.py` 是文本级原型，对 `lib.rs` 失败；判定库里现有块数没数（需要重型二进制）；整库重判的态数是用两个现成的数夹出来的，不是量的。
- 复出但没进我的 rerun 的两个数：第二轮攻方 `closure-analysis.py` 的 549 / 598 / 82 / 274 / 30（命令在 4.4），闭包里带 cfg 谓词的 fn 数（feature 3、target_arch 2，临时脚本）。
- 所有数都在冻结副本或本机草稿副本上量，不是入库装置上的数；判决要引的，照 `.claude/rules/three-way-inference.md` 在入库装置上重做。

## 交回前核对

- 复跑：终版模型目录（41 个文件，含 `out/` 的五份参照，sha256 见文首）上五段各比对一次，末行都是 `name=rerun_result mismatches=0`（`name=rerun_compare item=z1` 到 `z5` 都是 `result=same`）；`erased_lines.py` 最后改了一行（加 `sys.dont_write_bytecode`，不让它在模型目录里留 `__pycache__`）之后，z5 又单独比对一次，仍是 `same`。
- cite-check：`python3 scripts/agents/cite-check.py research/prompts/m4-exit5-relocation-proof-r3-sonnet-output.md`，没有 ✗ 行，末行原样：

```
  ✓ 核了 9 处引文、55 处裸行号，对上 9 处，没判 55 处（其中裸行号没有引文可比 33 处；逐处列在上面）
```

  没判的另 22 处（含这一节自己列的 6 处）是模型目录里还没入库的文件名（`gc_ring_runs.rs`、`z1_original_probe.rs` 等，行号我用 `grep -n` 现取过：`gc_ring_runs.rs:87`、`:253`、`:276`，`z1_original_probe.rs:486`，`z1_bridge_probe.rs:8`，`z2_f1b_debug_assertions_fork.rs:8`，`z2_f5_macro_outside.rs:3`，`z2_f2_trait_impl_outside.rs:15`）和冻结副本里有多个同名文件的 `mount.rs`、`block_device.rs`（行号同样现取）。
- 清理：草稿目录 `~/.local/share/singlefs/work/m4-exit5-relocation-proof-r3-sonnet/` 删前 `du -sh` 总计 775M，其中编译目录 `z2-target` 196M、`z5`（MIR 编译目录、最小工作区拷贝、证明行比例用的 replay-prefix 拷贝）447M，其余是五段复跑的工作目录与闭包文本（`clos` 16M、`rr-*` / `cmp*` 各 0.2M 到 16M）与 replay-prefix 副本 436K；这些都删了，现 160K，只剩运行日志文本。没有仓副本、编译目录或库拷贝留着。
