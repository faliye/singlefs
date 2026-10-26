# m4-exit5-relocation-proof-r2 攻方腿（Opus）报告

日期 2026-10-26。攻击面：正文第三节 Y1–Y6，第二轮角度（第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方）。正推腿这一轮的产出没读（禁读），攻的是正文第四节点名的那一形：core 直验连胶水，加第一轮判决挂起的三处修补。代码行号除另注明的外都是冻结副本 `~/.local/share/singlefs/work/m4-close-rounds-frozen/` 的；`scripts/`、`.claude/` 下的文件冻结副本里没有，引的是主工作区。副本上量的数都标「副本」，不是入库装置上的数。

## 各格判定一览

| 格 | 判定 | 一句话 | 分不分辨臂 |
|---|---|---|---|
| Y1 | 打中 | 同一个 `verus!` 里按 `cfg(verus_keep_ghost)` 分两份函数体：Verus 带 `--no-cheating` 验过，产品构建编的是另一份，跑出来 ensures 不成立；门禁的作弊正则一个都不认 | 分辨：两种接法（`verus!` 包、只在 cfg 下展开规格）都中；另加「被验文件不许出现 `verus_keep_ghost`」一格的不中 |
| Y1 | 事实（副本，量的） | core 今天已经经 `singlefs-replay-prefix` 依赖 vstd；Verus 用 rustc 1.98.1，产品用 1.98.0 | — |
| Y2 | 打中 | `assume_specification` 本身被 `--no-cheating` 拒；放开它就要去掉这个开关，去掉之后 `axiom fn` 证出 false 也过，门禁正则不认 `axiom`；一条看着对、与 std 文档不符的 `binary_search` 规格，必红世界照红，产品跑出来 ensures 不成立 | 分辨：只打「登记 + 必红世界」那条改法 |
| Y3 | 打中 | 阶段按「发布闭包 ∪ 回退闭包」（1071 个 core fn）划时，崩溃后读映射那一路有 30 个 fn、1521 行在外面；闭包里有 5 种只供测试的运行时开关类型，pub 的 `set_reuse_window` 在闭包外 | 分辨：只打「射程 = 写侧两条闭包」那一读 |
| Y4 | 打中 | 今天判法的逐项办法对 core 走不通：1071 个里 598 个在 impl/trait 里，82 个名字被 274 个 fn 共用；在 core 函数上加 ensures 或 proof 块会改崩溃判器闭包键（量的），549 个 fn 同在判器闭包里 | 分辨：打「沿用逐项按名验与现行键」两样 |
| Y5 | 打中 | 两处修补合起来仍放过第一轮反例的变体：区间版 P4 前提只对现行映射，搬迁能写进回退目标还在读的槽（Verus 证出）；补了回退视图的协议按起点槽号避让，两槽单元的第二槽照样可写 | 分辨：只打「P4 区间收严 + 协议补视图」各自单改 |
| Y6 | 清单 | 在上面几格的形下仍罩不到的东西（见 Y6 一节） | — |

我提的改法一样都没实现，只在我的模型上量过反例，改法被攻过零轮。

## 复跑

模型目录 `research/prompts/m4-exit5-relocation-proof-r2-opus-model/`。复跑命令（仓根起，草稿目录给一个新的；这一次跑了 82 秒，看内存包装排不排队）：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m4-exit5-relocation-proof-r2-opus-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m4-close-rounds-frozen --work <新的草稿目录>
```

它做三件事：一、直接起 Verus 0.2026.09.20（`--rlimit 10`，带不带 `--no-cheating` 每行写明）验作弊标记三份、Y1/Y2 原型 crate 与三份必红变体、Y5 攻击模块（与冻结副本的两份证明一起）与三份必红变体；二、原型 crate 照产品方式编（不带 `verus_keep_ghost`）、直接执行编出来的二进制，逐个核 ensures；三、冻结副本的拷贝上用 `scripts/gate/lib_function_units.py closure` 收四条闭包（发布、回退、崩溃判器、恢复）数 Y3/Y4 的几样，`key` 量判器闭包键在四种改法下变不变，`cargo tree` 看依赖。末了与 `out/summary.txt` 逐行比，不同退 1。

`out/summary.txt` 全文（副本，量的）：

```
name=verus case=axiom-plain-no-cheating rc=1 error: external_body/assume_specification not allowed with --no-cheating 
name=verus case=axiom-plain-without-no-cheating rc=0 verification results:: 1 verified, 0 errors 
name=verus case=axiom-broadcast-no-cheating rc=1 error: external_body/assume_specification not allowed with --no-cheating error: external_body/assume_specification not allowed with --no-cheating 
name=verus case=assume-specification-no-cheating rc=1 error: external_body/assume_specification not allowed with --no-cheating 
name=verus case=assume-specification-without-no-cheating rc=0 verification results:: 1 verified, 0 errors 
name=verus case=divergence-only-no-cheating rc=0 verification results:: 2 verified, 0 errors 
name=verus case=both-modules-without-no-cheating rc=0 verification results:: 3 verified, 0 errors 
name=verus case=divergence-must-red-verified-body rc=1 error: postcondition not satisfied error: possible arithmetic underflow/overflow 
name=verus case=divergence-must-red-cfg-macro rc=1 error: postcondition not satisfied error: possible arithmetic underflow/overflow 
name=verus case=std-spec-must-red-first-index rc=1 error: postcondition not satisfied verification results:: 2 verified, 1 errors 
name=verus case=y5-baseline-two-frozen-proofs rc=0 verification results:: 31 verified, 0 errors 
name=verus case=y5-attack rc=0 verification results:: 33 verified, 0 errors 
name=verus case=y5-must-red-rollback-read-unchanged rc=1 error: assertion failed verification results:: 32 verified, 1 errors 
name=verus case=y5-must-red-protocol-allows-the-write rc=1 error: assertion failed verification results:: 32 verified, 1 errors 
name=verus case=y5-must-red-two-slot-write-refused rc=1 error: assertion failed verification results:: 32 verified, 1 errors 
name=product-build rc=0
name=product-run fn=destination_after start=10 span=2 returned=11 ensures_holds=false
name=product-run fn=destination_after_by_the_cfg_macro start=10 span=2 returned=11 ensures_holds=false
name=product-run fn=first_index_of slice=[2, 2, 2, 2, 2] returned=Some(4) ensures_holds=false
name=closure-union entries=publish_relocation_batch+roll_back_by_a_forward_publish fn=1073 fn_lines=24970 core_fn=1071
name=closure-union-in-impl core_fn_inside_impl_or_trait=598 of=1071
name=closure-union-name-collisions distinct_names_shared_by_two_or_more_fns=82 fns_carrying_a_shared_name=274 top=[('to_bytes', 12), ('parse', 12), ('from', 11), ('new', 9), ('journal_record_offsets_hint', 8), ('device_size_in_bytes', 7)]
name=closure-overlap core_fn_in_union_and_in_judge_crash_image=549 of=1071 judge_fn_total=1298 judge_core_fn=589
name=closure-recover-outside-union core_fn_in_recover_not_in_union=30 recover_core_fn=571
name=closure-union-test-only-switch-types present=['CompactionBatchSize', 'CompactionRunForcing', 'DeletionSwitches', 'ReuseWindow', 'UnlinkBatchSize']
name=membership fn=set_reuse_window in_publish=0 in_rollback=0 in_judge=0
name=membership fn=release_leaving_the_record_allocated_on in_publish=1 in_rollback=1 in_judge=0
name=membership fn=reclaim_released_records_up_to in_publish=1 in_rollback=1 in_judge=0
name=closure-recover-outside-union-lines fns=30 fn_lines=1521 includes_mapping_read=1 includes_read_mapped_tree_root=1 includes_replay_journal=1
name=closure-sizes publish_fn=836 rollback_fn=1057 judge_fn=1298 recover_fn=654
name=cfg-in-closure-fn-bodies cfg=feature="lookup-work-counters" fns=3 first=['count', 'drop', 'entering']
name=cfg-in-closure-fn-bodies cfg=not(feature="lookup-work-counters") fns=2 first=['count', 'entering']
name=cfg-in-closure-fn-bodies total_core_fns=1071
name=judge-key variant=frozen key=7e76314e8552a53efb6ffba49fdd23179799ac94d07f0eab7f0e05a88be4900a
name=judge-key variant=comment-only key=7e76314e8552a53efb6ffba49fdd23179799ac94d07f0eab7f0e05a88be4900a
name=judge-key variant=wrapped-in-verus-without-ghost key=7e76314e8552a53efb6ffba49fdd23179799ac94d07f0eab7f0e05a88be4900a
name=judge-key variant=wrapped-with-an-ensures key=070444cae357beefb8aaf8667e20de83084a1de14ee7af30c5daebddaf82378d
name=judge-key variant=wrapped-with-an-ensures-and-a-proof-block key=2efa80a4899459c24451b5d1c05353d418c1258c163cd60528268806bb3fb314
name=cargo-tree core_already_depends_on_vstd_through_singlefs_replay_prefix=yes
name=cheating-regex text=pub_axiom_fn_a_lemma_that_proves_false matches=0
name=cheating-regex text=pub_broadcast_axiom_fn_axiom_every_slot_ matches=0
name=cheating-regex text=#[cfg matches=0
name=cheating-regex text=if_cfg! matches=0
name=cheating-regex text=pub_assume_specification<T:_Ord>[_<[T]>: matches=1
```

各文件 sha256（模型目录下相对路径）：

```
a19e10b7b5d5cf7da62e19d26df06e1cffdcf57773e649cade6c4620119290bc  ./out/summary.txt
fd672f1975f85ec8551cb6e69256dac973adc54f176c86298d17b0c63928c725  ./tools/cfg-in-closure.py
4e2b07d7a6a41e9d78195b49c7bb0b0a7dece807c0695e79cc9b60790af28e01  ./tools/closure-analysis.py
533efb4e717e348ee2368a4d9b5c405f63b6d822db80ca66a2a4d97632b3cd60  ./tools/key-variants.sh
9755fb76dcfb8b1678966f2b876b336a6a147a5c8e053ef8b9507e137a7b0342  ./tools/rerun.sh
2e2d192f5894b46221f87b7fb5342a9a3a7640e787ae2505c5b2e2a6b079b178  ./verus/assume_spec.rs
c84aed102edbf20f738d641403ccc4f340b3bb1341a157d627c9ba0935fe3f8b  ./verus/attack_patched_p4_and_rollback_views.rs
f08f95c15111fd683fdceda7ff6716327e66922b6a51c2d74a013e81bd09f609  ./verus/axiom_broadcast.rs
6d4d37d061403162b85e8585f70af1aa3b600e47bc0f593153a2c5d400188709  ./verus/axiom_plain.rs
f9f7fea3d2b50389c47c87cb71b7723239ddc1e487792c8219e02190925ea5b6  ./y1crate/Cargo.toml
ee02e4a53f67f0339404aeb8ac60c43a9eb2971189fef28947734e5cdabd6b66  ./y1crate/src/divergence_root.rs
6b4ee67b95c94b16a13d0205ca686271bbb0b713901458573899347a8372cff4  ./y1crate/src/divergence.rs
b8bbf2221d947eaefa277af46f6d857ac1e3cad0a9921b70666ae2a9f276cdbe  ./y1crate/src/lib.rs
e587809e6d2e2ca3b5405ae76279fa0407a80d3ad764044f0d1c153bd8ad1048  ./y1crate/src/main.rs
87dda8108ea3ee23547b8c6944c274cb9c307a886404175d0ca38ecee34eae49  ./y1crate/src/std_spec_probe.rs
```

取样与估时：这一腿没造崩溃历史，没有崩溃状态数可算（理由在「这条腿自己的限度」）。Verus 每份几秒到几十秒，闭包每条约 5 秒。

## Y1　怎么接

### 打中：Verus 验的与产品编的可以不是同一份函数体，`--no-cheating` 与门禁正则都放过

今天的同源担保只说到包：「#   所以编译用 crates.io 这六个包擦 verus!、验证用这份 Verus 自带的宏与 vstd，两边是同一份源码；对齐走「Verus 退到 0.2026.09.20」，不改依赖。」（`.claude/gate.d/verus-pins.tsv:16`，主工作区）。它管宏与 vstd 是同一份，不管被验文件里的代码在两种编法下展开成什么：Verus 编的时候 `verus_keep_ghost` 为真，产品编的时候为假（`crates/singlefs-replay-prefix/Cargo.toml` 的 check-cfg 就登记了这两个 cfg）。

原型 `y1crate/src/divergence.rs`（副本）两种写法：

- 写法一：同名两份 `destination_after`，一份 `#[cfg(verus_keep_ghost)]` 体是 `start + span`，一份 `#[cfg(not(verus_keep_ghost))]` 体是 `start + 1`，两份都写 `ensures r == start + span`。
- 写法二：同一份函数体 `if cfg!(verus_keep_ghost) { start + span } else { start + 1 }`。

量到的（`out/summary.txt` 原样）：

```
name=verus case=divergence-only-no-cheating rc=0 verification results:: 2 verified, 0 errors 
name=verus case=divergence-must-red-verified-body rc=1 error: postcondition not satisfied error: possible arithmetic underflow/overflow 
name=verus case=divergence-must-red-cfg-macro rc=1 error: postcondition not satisfied error: possible arithmetic underflow/overflow 
name=product-run fn=destination_after start=10 span=2 returned=11 ensures_holds=false
name=product-run fn=destination_after_by_the_cfg_macro start=10 span=2 returned=11 ensures_holds=false
name=cheating-regex text=#[cfg matches=0
name=cheating-regex text=if_cfg! matches=0
```

读法：带 `--no-cheating` 两条都验过；把 Verus 看见的那一支改坏，必红世界照样红（所以必红世界那一格也是绿的）；产品构建跑出 11，ensures 要 12。作弊标记取的是判法里那一行「CHEATING_MARKER = re.compile(」（`scripts/gate/verus-proof-check.py:473`，主工作区），两种写法它都不认。

**四句**：① 分辨：`verus!` 包住被验函数、只在 `cfg(verus_keep_ghost)` 下展开规格两种接法都中——两种都靠这个 cfg 区分两种编法，被验代码里只要能写这个 cfg 就能分岔；接法里另加一格「被验文件（去注释后）不许出现 `verus_keep_ghost`、`verus_keep_ghost_body`」的不中（推的，没实现）。② 判别子看得到：两份函数体都在源码里，门禁扫源码那一刻看得到。③ 满足的分句：正文第四节攻方立场「某种接法让产品构建变了或 Verus 与 rustc 看到的不是同一份代码」的后一半。④ 改法：上面那一格是文本判法，挡得住这两种写法；挡不住经宏间接产生的 cfg（宏在别的文件里定义），所以还要配「被验 crate 里定义的宏也在扫描范围」——推的。

**什么现象会推翻它**：Verus 或 crates.io 的擦除宏在 `verus!` 里见到 `verus_keep_ghost` 就报错（这次量的是不报）；或门禁已有一格扫这个 cfg（`grep -n verus_keep_ghost scripts/gate/verus-proof-check.py` 现查 0 处命中）。

### 两条事实（副本，量的）

- 产品构建今天就带着 vstd：`cargo tree -p singlefs-core` 里 vstd 经 `singlefs-replay-prefix` 进来（`name=cargo-tree core_already_depends_on_vstd_through_singlefs_replay_prefix=yes`）。「core 加 vstd 依赖」对产品依赖集合不加新包；变的是 core 自己的源码要经擦除宏展开。
- Verus 起的 rustc 与产品不是一个版本：「    "toolchain": "1.98.1-x86_64-unknown-linux-gnu (overridden by environment variable RUSTUP_TOOLCHAIN)",」（Verus 安装目录 `version.json:9`），产品 `rustc --version` 现查是 1.98.0。vstd 的 std 规格是对 1.98.1 的 std 写的、被它编的，产品链的是 1.98.0 的 std；判法的 toolchain 格核的是 Verus 安装与六个包，不比产品工具链（`grep -n 'rust-toolchain' scripts/gate/verus-proof-check.py` 0 处命中）。补丁版本之间 std 行为多半不变，这一条我没造出行为差，只记成未连接的前提。

## Y2　std 规格与「作弊标记」

### 打中：「只许对 std 已知函数写 `assume_specification`、逐条登记、配必红世界」这一改法，三处站不住

**一、它得先去掉 `--no-cheating`。** 判法每一次起 Verus 都带这个开关（`scripts/gate/verus-proof-check.py:7` 那一行写着「带 --no-cheating」）。量到的（副本，原样）：

```
name=verus case=assume-specification-no-cheating rc=1 error: external_body/assume_specification not allowed with --no-cheating 
name=verus case=assume-specification-without-no-cheating rc=0 verification results:: 1 verified, 0 errors 
```

所以登记表管不了「哪几条放进来」：开关是整次起 Verus 的，开了它一条 `assume_specification` 都过不了，关了它全部放行。

**二、去掉开关之后，正则成了唯一一道，它不认 `axiom`。** `verus/axiom_plain.rs` 写一个不带函数体的 `pub axiom fn` 确保 false，在一个返回 7、ensures 写 `r == 0` 的函数里调它：

```
name=verus case=axiom-plain-no-cheating rc=1 error: external_body/assume_specification not allowed with --no-cheating 
name=verus case=axiom-plain-without-no-cheating rc=0 verification results:: 1 verified, 0 errors 
name=verus case=axiom-broadcast-no-cheating rc=1 error: external_body/assume_specification not allowed with --no-cheating error: external_body/assume_specification not allowed with --no-cheating 
name=cheating-regex text=pub_axiom_fn_a_lemma_that_proves_false matches=0
name=cheating-regex text=pub_broadcast_axiom_fn_axiom_every_slot_ matches=0
```

今天 `axiom fn` 由开关挡着，判法文档列的标记里没有它（「      核里（去掉 // 注释之后）不许出现 external_body、assume…（含 assume_specification）、admit、verifier::external / verifier(external)、」，`scripts/gate/verus-proof-check.py:25`）。这是「从未被看过的地方」：正则与开关重叠时看不出正则缺一样，拿掉开关才露出来。

**三、必红世界判不出「规格与 std 不符」。** `y1crate/src/std_spec_probe.rs` 给 `<[T]>::binary_search` 写的规格是「`Ok(i)` 时 i 是第一个等于 x 的下标」——看着对，std 文档写的是有几个相等元素时返回其中任一个。被验函数 `first_index_of` 照这条规格证 ensures「第一个下标」：

```
name=verus case=both-modules-without-no-cheating rc=0 verification results:: 3 verified, 0 errors 
name=verus case=std-spec-must-red-first-index rc=1 error: postcondition not satisfied verification results:: 2 verified, 1 errors 
name=product-run fn=first_index_of slice=[2, 2, 2, 2, 2] returned=Some(4) ensures_holds=false
```

必红世界（把 `Ok(i) => Some(i)` 改成取 i - 1）照规定红了，登记、必红、正则三样都绿，产品跑出来下标是 4、ensures 要 0。必红世界证的是「这条规格被用到了」，不是「规格对 std 为真」；后一样 Verus 那一侧看不见 std 的函数体。core 用到同类有平局语义的 std 函数：`binary_search` / `binary_search_by_key` 9 处、`max_by_key` 11 处（`grep -F` 现数，冻结副本 `crates/singlefs-core/src/*.rs`），例「            .max_by_key(|candidate| (candidate.checkpoint_txg, candidate.instance));」（`crates/singlefs-core/src/mount.rs:3967`）——`max_by_key` 平局取最后一个，规格写成取第一个同样过必红世界。这几处今天的键大概都互异（没逐处核），打中的是判法，不是 core。

**vstd 自带的那一份同样是假定，而且不登记。** 副本里 vstd 0.0.0-2026-09-20-0158 的源码现数：`assume_specification` 485 行、`axiom fn` 184 行、`verifier::external_body` 283 行（`grep -rE` 数行，不是条目数，一条可能跨行）。它们在 vstd 里，`--no-cheating` 不管，判法也不扫；「只许已知函数、逐条登记」若只管 core 里写的那几条，vstd 那几百条照旧不进账，而且是对 1.98.1 的 std 写的（Y1 第二条事实）。

**四句**：① 分辨：只打「放开 `assume_specification`、靠登记加必红世界判」那一条改法；不放开、把 core 里用到的 std 调用换成自写并证过的 exec 函数（或用 vstd 已有规格的那几个）的不中，代价是改产品代码（推的）。② 判别子观测不到：门禁判「规格与 std 的行为不符」要看 std 的函数体或跑 std，Verus 与必红世界都只看规格本身——这是「判别子观测不到」那一类，改判据就是新一轮。③ 满足的分句：正文 Y2 原问「那一条门禁怎么判得出『写的规格与 std 的行为不符』」——判不出。④ 改法：要判得出，只能另立一条不经 Verus 的路子：每条登记的规格配一个按规格写的运行时对拍（随机输入调真 std，核 ensures；平局、空切片、重复元素这类边角显式枚举），跑在产品工具链上——推的、被攻过零轮；它是测试，不是证明，把那条规格从「证出」降到「测过」。

**什么现象会推翻它**：Verus 有开关能只放行 `assume_specification`、仍拒 `assume`/`admit`/`axiom`/`external_body`（这一版 `--no-cheating` 的报错把 external_body 与 assume_specification 并成一句，我没逐个查 Verus 的命令行选项）；或判法另有一格在运行时对拍登记的规格。

补一句（现查）：`verus --help` 里与此有关的只有一个开关，原文「        --no-cheating   Do not allow assume, admit, verifier::external_body,」接下一行「                        and assume_specification」，没有只放行 `assume_specification` 的选项（Verus 安装目录里的 `verus`，这一版）。

## Y3　射程与分阶段

### 打中一：以写侧两条闭包划阶段，崩溃之后定义「抽象状态是什么」的读路径在外面

量法同第一轮（闭包按调用与引用收），用的是 `scripts/gate/lib_function_units.py closure`（与 `singlefs-unit-graph` 对拍的那一份，发布闭包 836 个 fn 与第一轮量的相同）：

```
name=closure-sizes publish_fn=836 rollback_fn=1057 judge_fn=1298 recover_fn=654
name=closure-union entries=publish_relocation_batch+roll_back_by_a_forward_publish fn=1073 fn_lines=24970 core_fn=1071
name=closure-recover-outside-union core_fn_in_recover_not_in_union=30 recover_core_fn=571
name=closure-recover-outside-union-lines fns=30 fn_lines=1521 includes_mapping_read=1 includes_read_mapped_tree_root=1 includes_replay_journal=1
```

在外面的 30 个里有 `recovery::recover`、`scan_journal`、`replay_journal`、`choose_root`、`read_mapped_tree_root`、`CentralMappingTreeReadOnFirstUse` 的 `tree` / `locations_of_key`、`walk_to_file_judging_the_truncation_fields_as`（清单在草稿里用闭包文件现算，复跑可重出）。整理的结论「每个 key 读到的字节不变」，崩溃之后是由这一路读出来的：映射从哪份副本取、按 key 落到哪个落点，都在这几个函数里。阶段验收若是「两条闭包里的函数都进证明、verified 数达到登记」，这一路改了照样全绿，而用户看见的内容变了。`gc_crash_proofs` 自己也写着这一半没接：「//! journal 完整提交的重放应映射为 Relocate/Checkpoint，解码和映射仍需调用方证明。」（`crates/singlefs-replay-prefix/src/gc_crash_proofs.rs:18`）。

这与第一轮 X2 那一处不同：第一轮说的是回退路径不在发布闭包里，判决已把回退算进射程；这一处是把回退算进来之后，读侧仍不在。

**四句**：① 分辨：只打「射程 = 发布闭包 ∪ 回退闭包」那一读；射程按「GC 写下的状态被谁读」收（加恢复与挂载读路径）的不中，代价是并集再多 30 个 fn、1521 行（量的）。② 判别子看得到：读路径在 core 里、静态可数。③ 满足的分句：正文第四节攻方立场「分阶段里某一阶段的验收在 core 里有一条绕过它而改变抽象状态的路径」。④ 改法：把恢复闭包并进射程，或把抽象状态定义成「恢复函数读出来的东西」、让恢复路径成为阶段一的义务之一——推的。恢复路径里 `replay_journal`、`choose_root` 已有核（D13 已定项 1 M1–M4），读映射那几个没有。

### 打中二：只供测试的运行时开关在产品代码里，pub 的拨法在闭包外

```
name=closure-union-test-only-switch-types present=['CompactionBatchSize', 'CompactionRunForcing', 'DeletionSwitches', 'ReuseWindow', 'UnlinkBatchSize']
name=membership fn=set_reuse_window in_publish=0 in_rollback=0 in_judge=0
name=membership fn=release_leaving_the_record_allocated_on in_publish=1 in_rollback=1 in_judge=0
```

`release_leaving_the_record_allocated_on` 在两条闭包里，它对 `ReuseWindow::ForcedToZero` 那一支当场回收，不看 F；拨这个开关的「    pub fn set_reuse_window(&mut self, reuse_window: ReuseWindow) {」（`crates/singlefs-core/src/allocator.rs:2048`）是 pub、不带 `cfg(test)`，文档写「    /// 装上复用窗口这个只供测试的开关（`ReuseWindow`）。产品路径一处都不调它。」（同文件第 2047 行），后果写在「/// 它造出来的正是 C22（刚释放的块立即重分配） 要的那一格：被复用的槽仍被一条候选根引用着。」（同文件第 1097 行）。另外分配器的空闲图是 pub 字段「    pub devices: Vec<DeviceFreeMap>,」（同文件第 1819 行），挂载结果与会话都把分配器 pub 出来（「    pub allocator: PoolAllocator,」，`crates/singlefs-core/src/mount.rs:1143`、`crates/singlefs-core/src/mounted_session.rs:91`）。

推的（没写 Verus）：要证 `release` 一类函数「F 之上释放的不回收」，Verus 会在 `ForcedToZero` 那一支红，只能加 `requires` 窗口是产品值，或用类型不变式；`requires` 由调用方守，pub 的拨法与 pub 字段都能在验过的函数之外把它破掉；类型不变式要字段私有、改动它的方法都守不变式，而只供测试的开关本身就是为了破它——与 `.claude/rules/fs-design.md@五条硬要求` 那一条「2. **每条分支必须能被测试强制进入。**」（`.claude/rules/fs-design.md:122`）撞。今天产品代码里没有调用（我只 `grep` 了 `set_reuse_window`：调它的都在测试里，6 处），所以这一格打的是「阶段验收」能被绕过，不是今天有错。

**四句**：① 分辨：只打「开关保留成运行时值、按函数 `requires` 验」那一做法；开关改成编译期（`cfg` 或特性，产品构建里没有那一支）的不中，代价是 harness 档与 checker 档的用例要换编法（推的）。② 看得到：开关的值在分配器里，运行时可读（`reuse_window()`）。③ 同打中一的分句。④ 改法：见①；也可以证明里显式带「窗口 = 产品值」当前提，并在报告里把这一前提列进未连接前提——这等于承认那几个测试用例跑的不是被证的代码。

### 产生结论的方法：verified 数当阶段验收

推的、没量：Verus 的 verified 数按函数计，没有 ensures 的 exec 函数验过也算一条（只证了不溢出、不越界、循环终止）；判法的必红世界按「每个带 ensures 的函数一行」配，没有 ensures 的函数一行都不用配。所以「这一阶段 N 个函数进证明、verified 数到 N」可以在一条功能性质都没证的情况下达成。阶段验收要写成「点名的 ensures 被证出、每条配必红世界」，verified 数只当辅助。

## Y4　判法与门禁

### 打中一：逐项按名验的那套办法对 core 的闭包走不通

判法今天两条：「      verus! 里两个 fn 同名、或 fn 在 impl / trait 里判红：--verify-function 按名点（有恰好同名的就取它，名字是别人子串的照常点得中），这两种点不准。」（`scripts/gate/verus-proof-check.py:34`），「      跨核同名也判红：项名、各项之和与判定库按名字计，必须全局唯一。整格再做一次不带过滤的 crate 验证，防止义务遗漏。」（同文件第 35 行）。另有一条「直接起的 Verus 只带它自带的 vstd：crate 的 Cargo.toml 除 vstd 之外还有依赖，proof、vacuity、must-red-worlds 三格开跑之前判红。」（同文件第 10 行）——core 依赖 libc、`singlefs-format`、`singlefs-replay-prefix`，照现行判法三格开跑前就红。

在两条闭包的并集上量（副本）：

```
name=closure-union-in-impl core_fn_inside_impl_or_trait=598 of=1071
name=closure-union-name-collisions distinct_names_shared_by_two_or_more_fns=82 fns_carrying_a_shared_name=274 top=[('to_bytes', 12), ('parse', 12), ('from', 11), ('new', 9), ('journal_record_offsets_hint', 8), ('device_size_in_bytes', 7)]
```

1071 个里 598 个在 impl / trait 里，274 个带着与别的 fn 相同的名字。只把「一行 crate」放宽成几行而保留逐项按名点，这两条会让大半个射程判红；改成按限定名点要先确认 Verus 0.2026.09.20 的 `--verify-function` 认不认 impl 里的方法（这一点我没量）。多 crate 还要换掉「直接起 Verus、只带自带 vstd」：要么改走 `cargo verus`（判法文件头写了不用它的理由：「Verus 每次都直接起（不经 cargo verus：它按内容缓存，内容不变时不出判定行）」，第 7 行），要么自己给 Verus 传依赖 crate 的编译产物——两样都是判法的新路子，不是改几个数。

**四句**：① 分辨：打「沿用现行判法、只改登记数与 crate 行数」那一读；重写成按限定名点、多 crate 的不中（推的，代价没量）。② 看得到：闭包是静态的。③ 满足的分句：正文 Y4「`verus-proof-check.py` 只认一行 crate……这几条在 core 进来之后怎么改」。④ 改法见①。

### 打中二：在 core 函数上写证明，会让崩溃判定库整库作废（从未被看过的地方）

D13 已定项 17 的射程那一行（`.claude/kb/decisions/13-验证路线.md:439`）定了：镜像判定与给定状态判定的版本取的是判定定义那一组（CPU 参照判器 `crash::judge_crash_image` 等）的闭包键；单元指纹取去注释之后的词法单元。我在 core 的 `allocation_record_tree::key_bytes_width`（发布、回退、判器三条闭包都含它）上做四种改法，量 `judge_crash_image` 的闭包键（副本，`lib_function_units.py key`）：

```
name=judge-key variant=frozen key=7e76314e8552a53efb6ffba49fdd23179799ac94d07f0eab7f0e05a88be4900a
name=judge-key variant=comment-only key=7e76314e8552a53efb6ffba49fdd23179799ac94d07f0eab7f0e05a88be4900a
name=judge-key variant=wrapped-in-verus-without-ghost key=7e76314e8552a53efb6ffba49fdd23179799ac94d07f0eab7f0e05a88be4900a
name=judge-key variant=wrapped-with-an-ensures key=070444cae357beefb8aaf8667e20de83084a1de14ee7af30c5daebddaf82378d
name=judge-key variant=wrapped-with-an-ensures-and-a-proof-block key=2efa80a4899459c24451b5d1c05353d418c1258c163cd60528268806bb3fb314
name=closure-overlap core_fn_in_union_and_in_judge_crash_image=549 of=1071 judge_fn_total=1298 judge_core_fn=589
```

只加 ensures 或 proof 块——产品构建里擦掉、编出来的代码不变——判器闭包键就变；并集里 549 个 fn 同在判器闭包里。证明写进 core 之后，每改一处这 549 个 fn 里的证明，崩溃判定库里按这把键存的判定全部现判。今天这件事已经小规模发生：判器闭包里有 `singlefs_replay_prefix::replay_core`、`choose_root_slot`、`first_copy_of_each_record_key` 三个 Verus 核（`lib_function_units.py key` 的输出里有这三行），改它们的证明同样作废那一库；core 直验把这个面从 3 个函数扩到几百个。

**四句**：① 分辨：打所有「证明与可执行代码住在同一个函数单元里」的接法（`verus!` 包与 cfg 注解都是）；证明另住一处（独立的证明 crate 引用 core 的函数、不改 core 的函数单元）的不中，但那一形就不是「core 的函数体上直接写 ensures」了（推的）。② 看得到：键与组成清单都落盘。③ 满足的分句：D13 已定项 17 要的是「沿不沿用只看单元键」——它照字面成立，键变了是对的；打中的是代价，不是正确性。正文 Y4「单元键与复用（D13 已定项 17）对 core 里被验的函数怎么算」。④ 改法：算键之前把 `verus!` 里的 ghost 部分（requires、ensures、decreases、`proof { }`、spec / proof fn）剥掉，另算一把只给形式化判定用的键——推的、被攻过零轮；它本身是 D13 已定项 17「键过窄」那一类风险的新入口（剥错一样，执行代码的改动就被当成 ghost 漏判），要配一格必红：改一处执行代码、键必须变。

**什么现象会推翻它**：崩溃判定库的块键实际不含判器闭包（与已定项 17 射程那一行不符）；或单元算法已经剥 ghost（这次量的是不剥）。

## Y5　第一轮挂起的三处

我没读到正推腿给的改后协议，攻的是第一轮判决写下的两处修补的最直接写法（`verus/attack_patched_p4_and_rollback_views.rs`）：P4 的目的地前提收严成「目的区间两两不交、与映射里每个活单元的区间不交」（`destinations_are_disjoint_intervals_and_free`）；协议的 Write 允许条件在冻结副本那三类之外再避开每个持久回退视图（`patched_slot_is_unreferenced`）。其余照冻结副本的 spec 原样引用，不改冻结副本。

### 打中一：区间版 P4 前提只对现行映射，搬迁能写进回退目标还在读的槽

具体状态：现行那一版只有 key 7 在槽 10；回退目标（用户删掉 key 4 之前那一版）里 key 4 在槽 20，现行映射里已没有 key 4，槽 20 已释放、还被回退窗口护着。把 key 7 搬到槽 20：

- 区间版前提成立，冻结副本那一版（起点互异）也成立；
- 现行那一版读 key 7 不变（P4 的结论照样成立）；
- 回退目标读 key 4 从 `[99]` 变成 `[42]`；
- 补了回退视图的协议里往槽 20 写不允许，冻结副本那一版协议允许。

```
name=verus case=y5-baseline-two-frozen-proofs rc=0 verification results:: 31 verified, 0 errors 
name=verus case=y5-attack rc=0 verification results:: 33 verified, 0 errors 
name=verus case=y5-must-red-rollback-read-unchanged rc=1 error: assertion failed verification results:: 32 verified, 1 errors 
name=verus case=y5-must-red-protocol-allows-the-write rc=1 error: assertion failed verification results:: 32 verified, 1 errors 
```

多出的 2 条就是这份文件的两个 proof fn。读法：两处修补各自都对，合在一起缺一条桥——「P4 前提 ⇒ 协议里那次写是允许的」推不出来，因为 P4 前提只说现行映射，回退视图那一半在 P4 里没有名字。第一轮攻方反例一（回收覆写源槽、旧版映射读坏）的毛病换了一条路回来：不是 Reclaim 覆写，是搬迁的目的写覆写。core 今天靠分配器挡着（已释放、没回收的槽在空闲图上仍占着），这份挡法不在 P4 的前提里。

**四句**：① 分辨：打「P4 区间收严、协议补视图，两份各证各的」那一叠法；P4 的前提里加上「目的区间避开每个持久回退视图」（或改成直接要求协议的 Write 允许条件）的不中。② 看得到：分配器选目的地那一刻手里有空闲图，回退视图也在根环里。③ 满足的分句：正文 Y5「改后协议仍放过第一轮那两个反例的变体」。④ 第一轮判决给的两处改法在这一格上：P4 区间收严，还中；协议补视图，在协议那一侧不中，但 P4 那一侧碰不到——改法碰不到打中的格。

### 打中二：补了视图的协议仍按起点槽号避让，两槽单元的第二槽照样可写

协议的避让是「    forall|key: u64| mapping.contains_key(key) ==> #[trigger] mapping[key] != slot」（`crates/singlefs-replay-prefix/src/gc_crash_proofs.rs:66`），只比起点；协议自己写着没有跨度（「// 没有设备编号、span、根环、写缓存和 journal 编码，不能将其字段直接等同于盘上结构。」，同文件第 25 行）。回退目标里 key 4 是两槽单元、起点 20：

```
name=verus case=y5-must-red-two-slot-write-refused rc=1 error: assertion failed verification results:: 32 verified, 1 errors 
```

`the_patched_protocol_still_avoids_by_start_slot_only` 证出往槽 21 写在补了视图的协议里是允许的，而槽 21 在那个单元的区间里；必红变体（改成「不允许」）红。第一轮判决把「按区间不重叠」只开给 P4，协议的避让条件是另一份 spec，那一处改法碰不到它。

**四句**：① 分辨：打「协议按槽号、只在 P4 那边收区间」；协议的视图与避让也按区间写的不中。② 看得到：分配器手里有跨度（分配记录里的 `span_slots`）。③ 同上一格的分句。④ 改法：协议的 `gc_map_avoids` 改成区间版、映射值带跨度——推的，没写，义务数没量。

### G7 那一句

不判（第二轮的立场不在这里）。只记一条攻方这边的要求：G7 改写句里若说「恒等对回退能退回的每一版都成立」，它的证明要用到上面打中一缺的那条桥；只说现行那一版，就与第一轮 X1 打中一的回退结局分叉照旧冲突。

**什么现象会推翻这一节**：正推腿给的改后 P4 前提里本来就有回退视图那一半，或改后协议的避让按区间写——那样这两格对它不中，只对第一轮判决字面的那两处修补中。

## Y6　罩不到的

在 Y1–Y5 的形下仍罩不到的，按这一腿打出来的与读代码推的分两类；正文列的设备屏障与 FUA、字节编解码、校验和无碰撞、双盘副本、隔离、共享快照我没另攻，照正文列。

| 罩不到的 | 由来 | 由什么罩 | 标 |
|---|---|---|---|
| 被验文件里按 `verus_keep_ghost` 分岔的代码 | Y1 打中 | 门禁加一格扫这个 cfg；产品构建的崩溃点重放照跑真二进制 | 量过（反例） |
| vstd 自带的 485 行 `assume_specification`、184 行 `axiom fn`、283 行 `external_body` 是否对产品那一版 std 为真 | Y1 事实、Y2 | 没有路子罩；最多是产品工具链与 Verus 工具链钉同一版 | 量过（行数） |
| core 自己写的 std 规格是否对 std 为真 | Y2 打中 | 运行时对拍（测，不是证） | 量过（反例） |
| 只供测试的运行时开关那几支（5 种类型在闭包里） | Y3 打中二 | 照旧由 harness 档与 checker 档的用例罩；证明里写成前提 | 推的（Verus 那一半） |
| 恢复与挂载的读映射路径（30 个 fn、1521 行） | Y3 打中一 | 并进射程，或崩溃点重放全量照旧罩 | 量过（行数） |
| 回退视图与 P4 之间的桥、协议的跨度 | Y5 | 补证明；没补之前由模型对拍（要抽回退，第一轮 X1 打中一）罩 | 量过（反例） |
| 证明改动的代价落在崩溃判定库上 | Y4 打中二 | 不是罩不到，是代价：每改一次证明，判器闭包键变一次 | 量过（键） |
| feature 分岔：闭包里 3 个 fn 的函数体带 `cfg(feature = "lookup-work-counters")` | 现数（`name=cfg-in-closure-fn-bodies`） | Verus 只按一组特性验；harness 档打开这个特性编 | 量过（个数） |

## 改法逐格表（我提的改法，只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 在打中的那几格上 | 标 |
|---|---|---|---|
| 被验文件（连同其中定义的宏）不许出现 `verus_keep_ghost`、`verus_keep_ghost_body` | Y1 | 两种写法都会被挡；经别处宏产生的挡不住 | 推的 |
| 产品工具链与 Verus 的 rustc 钉同一版，判法比一次 | Y1 事实 | 消掉 1.98.0 / 1.98.1 的差；不碰规格本身对不对 | 推的 |
| 不放开 `assume_specification`，std 调用换成自写、证过的 exec 函数 | Y2 | 保住 `--no-cheating`；代价是改产品代码 | 推的 |
| 放开时：每条规格配运行时对拍，平局、重复元素、空输入显式枚举 | Y2 | 能抓到 `binary_search` 那一类；抓不到对拍没取到的输入 | 推的 |
| 射程按读者收，恢复闭包并进来 | Y3 打中一 | 那 30 个 fn 进射程 | 推的（数是量的） |
| 只供测试的开关改成编译期 | Y3 打中二 | 产品构建里没有那几支，pub 拨法消失 | 推的 |
| 判法改按限定名点、多 crate | Y4 打中一 | 598 个 impl 方法、274 个同名 fn 点得到 | 推的（Verus 认不认没量） |
| 算键前剥 ghost，形式化另一把键，配「改执行代码键必变」必红 | Y4 打中二 | 判器闭包键不再随证明变 | 推的 |
| P4 前提加「目的区间避开每个持久回退视图」 | Y5 打中一 | 反例里的前提应当证不出 | 推的 |
| 协议避让改按区间、映射值带跨度 | Y5 打中二 | 槽 21 那一写应当不允许 | 推的 |

## 没打中的形状

| 形状 | 试了什么 | 结果 |
|---|---|---|
| `axiom fn` 在 `--no-cheating` 下过 | `axiom_plain.rs`、`axiom_broadcast.rs` | 被拒（`external_body/assume_specification not allowed with --no-cheating`）：开关挡着，只在去掉开关时才是洞（算进 Y2） |
| 产品代码里有人调测试开关或直改空闲图 | `grep` `set_reuse_window(` 与 `.devices` 的可变用法 | 产品路径里只有 `instance_table_sweep.rs` 在一份分配器拷贝上占满空闲槽（只供测试的开关那一支）；没打中今天的产品 |
| 协议补视图之后 Reclaim 覆写回退源槽（第一轮反例一原形） | 只在 `patched_slot_is_unreferenced` 上推 | 补了视图之后不允许：这一原形被排除，变体见 Y5 |
| 环回绕、多盘时回退视图的生灭与崩溃的先后 | 只读了 `mount.rs` 里回收下界那一段（`reclaim_floor`、每块盘都落了的最新有效根） | 没造历史、没枚举崩溃，不判 |

## 这条腿自己的限度

- 崩溃状态没枚举：这一腿的反例都在 Verus 与单进程运行里，不靠崩溃；Y5 最后一格（环回绕与多盘）是崩溃那一维，我没调 `enumerate_crash_states`，交主 agent 判要不要另派。
- 没读正推腿的改后协议，Y5 攻的是第一轮判决字面两处修补的最直接写法；它对正推的那一形中不中，要主 agent 对着正推的文件核。
- Y3 打中二与 Y4 打中一里 Verus 那一半（`requires` 之外怎么守、`--verify-function` 认不认 impl 方法）是推的，没起 Verus 量。
- 闭包用的是 `lib_function_units.py`，不是第一轮的 `unit-graph`；发布闭包 836 个 fn 与第一轮相同，回退闭包 1057 个与第一轮相同，其余两条没有对照。
- 复跑那一趟前台超过 240 秒的风险：这次 82 秒，排队时会更长。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上的数都不是入库装置上的数。
- 没跑任何门禁阶段，没起 `scripts/gate/verus-proof-check.py`（只读了它的正则那一行拿来匹配），没跑 checker 档测试，没起 GPU。
- 没在 core 本身上试编 `verus!`（Y1 的原型是一个只依赖 vstd 的小 crate）；core 直验能不能编过归正推那一格。
- 要主 agent 跑的命令：无。

## 交回前复跑与清理

- 交回前在新的登录 shell 里整份跑了一次 `tools/rerun.sh`（命令同「复跑」一节，`--work` 给的是草稿目录下新的 `rerun-final/`）：包装退 0，末行原样「name=rerun-compare out/summary.txt same=yes」。退非 0 的件数为 0。
- 删了：冻结副本拷贝 `tree/`（39M）、原型 crate 的编译目录（115M）、第一趟复跑 `rerun-first/`（1.6M）、必红变体副本 `mut/`（648K）、`y5/` 与 `y1crate/` 草稿（132K）；复跑脚本自己删掉了它建的树、编译目录与变体副本。
- 草稿目录 `~/.local/share/singlefs/work/m4-exit5-relocation-proof-r2-opus/` 现在约 3.8M：几趟探针的原样输出、闭包清单与 `rerun-final/` 的输出，没有仓副本与编译目录；正文只引模型目录 `out/summary.txt` 与复跑重出的行，草稿不入库。
