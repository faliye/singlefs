# tooling-broad-mutation-prune-r3 攻方腿（Opus）报告：U2、U3

日期 2026-10-02。被判对象：`research/prompts/_tooling-broad-mutation-prune-r3-body.md` 第三节 C1–C6。前几轮判决：`research/prompts/tooling-broad-mutation-prune-r1-main-verification.md`、`research/prompts/tooling-broad-mutation-prune-r2-main-verification.md`；前两轮攻过的（常量闭包、跨版本复用、FA–FH、整段超时、墙钟与分时）这里不再攻。
模型上的改法与判定只在我的模型上量过、被攻过零轮；真 crates 上的数都是我副本上的（HEAD 导出，218 个文件逐个 sha256 与库目录这一版 `manifest.tsv` 相同；C4 用的是工作树副本），不是入库装置上的数。库目录只读。

## 各格判定一览

| 格 | 形状 | 判定 | 在哪量的 | 节 |
|---|---|---|---|---|
| U2② X1b | 父测试起子进程（`current_exe`），子进程打印完 `abort`；只有父测试抓得到 | 打中：挑出来的没红、全部测试抓到；三种覆盖取法都中 | 模型（量过） | 2.1 |
| U2② X3 | `const fn` 运行时被弱测试叫，又在常量项里被强测试用 | 打中：正文只例外「只在常量上下文被叫的」；crates 有一处同形（`to_device_offset`），今天不翻 | 模型（量过）；crates 读代码 | 2.1、2.2 |
| U2② X7 | 只有文档测试抓得到 | 打中：覆盖里没有文档测试，零覆盖记没红；今天 crates 0 处 | 模型（量过） | 2.1、2.2 |
| U2② 内联、函数指针、dyn、正常退出的子进程、bin | X1、X1c、X2、X2b、X4–X6 | 没打中：这一版工具链上计数都在 | 模型（量过） | 2.3 |
| U2② 真 crates 第一段 | 241 条抓到，抓它的测试在不在挑出来的集合里 | 没打中：238 条在，3 条在常量初始化式里（正文已例外） | 副本（量过，一次取样） | 2.4 |
| U2① Y1–Y3 | 串字面量里的空白、`/*`、`//` | 打中「正则去注释与空白」的读法；按词法单元比修住 | 模型（量过） | 三 |
| U2① Y5 | 约束挪进 `where` 并换 trait，函数体不变 | 打中「只比函数体」的读法；比整个 fn 项修住 | 模型（量过） | 三 |
| U2① Y4、Y8 | 宏展开、`include_str!` 读进被改函数 | 打中三种读法；今天被变异的两个包 0 处 | 模型（量过） | 三 |
| U2① Y6、Y7、Y9 | 覆盖 trait 默认方法、`overflow-checks`、给类型加 `Default` 让「无效」翻成编得过 | 三种读法都沿用、重跑翻面；属于用户定的「别处改动」那一类；Y9 说明沿用的「无效」也会翻 | 模型（量过） | 三 |
| U2① 这一类有多常见 | 最近 25 个动了导出范围的提交 | 9 个只改别的 `.rs` 或数据，C1 一条不算 | git 历史 | 三 |
| U2③ C4 | SMT 兄弟被占、线程数 | CPU 时间不随分时涨，随 SMT 兄弟被占涨 1.6–2.0 倍，随线程数涨到 1.26 倍 | 副本（量过） | 四 |
| U2④ C6 | 两种修法 | 红的只有一条 lib 测试；这一版 checker 档 33 个测试目标一个没跑过，修完绿不绿不知道 | 库目录日志、读代码 | 五 |
| U3 | 每条变异挑多少、每次提交多久 | 挑出来的测试耗时中位占全部的 33%（core 73%）；最近 15 个提交中位 1 865 条、10.6–82.0 小时，全量 28.8–233.9 小时（推的，只算第一段） | 副本量选择；小时数推的 | 六 |

## 一、复跑命令与文件 sha256

模型 `research/prompts/tooling-broad-mutation-prune-r3-opus-model/`（说明在它的 `README.md`）：

```
bash research/scripts/capped.sh 12 bash research/scripts/run-with-memory-cap.sh 8G python3 -B research/prompts/tooling-broad-mutation-prune-r3-opus-model/run.py <工作目录> <llvm bin 目录>
```

- llvm 工具：`curl -sSfO https://static.rust-lang.org/dist/2026-08-20/llvm-tools-1.98.0-x86_64-unknown-linux-gnu.tar.xz`，解到草稿目录（下载加解压 8 秒，236M），版本行「LLVM version 22.1.8-rust-1.98.0-stable」；没装 cargo-llvm-cov，没动 `~/.rustup`、`~/.cargo/bin`。
- 模型一趟 49 秒（C2 加 C1，负载 5–12）。
- `evidence/` 各脚本的用法写在文件头；真 crates 副本：`git archive HEAD crates Cargo.toml Cargo.lock litmus .cargo | tar -x -C <副本>`，覆盖一遍 `crates_c2_selection.py cover <副本> <插桩编译目录> <llvm bin> <输出目录> 6`（6 条并行 1 129 秒），再 `select`。
- C4：`c4_cpu_time.py <多线程测试二进制> <单线程测试二进制> <crates/singlefs-harness 目录> 3`，两条测试二进制由 `cargo test --no-run -p singlefs-harness --test entries_after_mount_refuse_swapped_or_behind_devices`（与 `--test accounting_tree_node_full`）编出，整条约 15 分钟。

```
264d58d2e032fd6e5e5345bc617cdf447142eea7d4279884b542dd98af4dc4f7  evidence/c1_commit_classes.py
8e2a7c462113c4bc9bebe00c17eca10c6922ecb5e9637e6cd57f08dffdd35227  evidence/c1-commit-classes.txt
ba37fb9e9c8223416510f9913337a7eff06f505274ff577d709eeb4d6f07087d  evidence/c1_literals_in_crates.py
c742cb535e8f0e6e579cd06f95851f13ee76aa4e16a182c0167c07ee00a91f7d  evidence/c1-literals-in-crates.txt
05c656a09da2ead9a56ef33b670fc1d440ff0de6defc877baf13b371cf051eec  evidence/c2_const_fn_contexts.py
97c7030e70e97d728d994bd9b2400073029059c313ed55bb346ab8feac5e39a8  evidence/c2-const-fn-contexts.txt
6219ab170c696f48d313d57015cf6fe3ce70e273f1e5eb01d4b30fe94d63edab  evidence/c4_cpu_time.py
cbf4dd9f464f3b2687cd3813e1023b6b0b624d47eb3d7a027c69bac7405a218e  evidence/c4-cpu-time.txt
ebd8d8e60e793404289a51718e0b69791a74a9ec87f1e857ddfaf561ed78bbcd  evidence/crates_c2_selection.py
03e88e4f690f9b6f975da96ebfdb58bd76b865233db6428e34e08060465aa5a6  evidence/crates-c2-selection.txt
d24d32d759c22ddd8d48e0f201d57eee3615aef16ef3e5e57968baa5b15faca4  evidence/crates-cover.txt
74ed12eef807754a868d4b51ffd478e22ef2f1d15e746f2b6cd468bba71150cb  evidence/crates-selection.tsv
39594bec7a485ef61568a26bc348ed97521c2ef6cf8f2aaefb0e68430f581f73  evidence/crates_zero_coverage_split.py
b647acde107dd590beb8506cc4b8c35754537c018894a9b2868828458d4fd7cf  evidence/crates-zero-coverage-split.txt
4f7559ba1beb652f1d26a0a86e75f47c7f0aba47ad754a553a606c726b9a3a37  evidence/u3_commit_cost.py
e5cbfab69d7494dec94f699926f37c5924325ccc58d2826e2910ef9fa5a06bb2  evidence/u3-commit-cost.txt
0f451f8f9142fc6d32507c3e34bb5aabba99c9a659be31c1cb59afa016009372  README.md
01815f837cfa1f5866584102d8470f688e54d2b5813c8869d8a017db2b8f6e57  run-output.txt
e1615f56a095862d519f29a8a3014b69c427dc8d6dea1cb346140a3387a32dbe  run.py
28570daba4c138745e13ed75aec8a672866238d2e3778e3d1acd5211a48f49ba  ws/Cargo.toml
b1fd1fedd39ab77984edcb341b0f34e7c7b335e95f5cbefb520f46af68433eaa  ws/mcore/Cargo.toml
b48bf1ccea0f2ac476c7010831a7859c59dc0013b9a6c0bbc51a78440699d899  ws/mcore/src/doc.rs
ca6daaad0e602a8054d390a63515256de69594e3e20b26dbfe55965834b114d4  ws/mcore/src/dynd.rs
00154a32fef5b77050c9e19464d7b2375915eafb50b27d5be102cff53408ca03  ws/mcore/src/fptr.rs
c1ab1fb70ca30840129122a3fd33458eb5966360444dc16a8ae1ee2840422cb0  ws/mcore/src/inl.rs
8bce8bc6e289d758bb9fae1a7479dad46d2beabde4d39d87915017a711db8dd6  ws/mcore/src/lib.rs
34ca525b4a0681e003558e094a89d1e994ba97bbc1293268c9d1d54040695448  ws/mcore/src/mixed.rs
781173ed6b9ab086393283c9233335592f8fa932decf5e649fd710a9438a6818  ws/mcore/src/sub.rs
cab7c631f25a1367d1367d4ae6598b5f3a849eab729499b79a128004544c4319  ws/mcore/src/tool.rs
4941ce4f6f9b18e90d78d58cf3a8a731bdfdbada46f4fc4d01b002ccdf1bce55  ws/mcore/src/y1.rs
ea283599476df1c966f8aeae8e72c9a76040db64561f5120bc9b0d459ec30716  ws/mcore/src/y2.rs
8ffe43ece95fbd873a4997122b1479f2febe16cb87f079ea62614c986cfd13c2  ws/mcore/src/y3.rs
8be1005d7296f5e2bd76b0ccb4a52b0f28329489edc14603d1fe701304965890  ws/mcore/src/y4_macros.rs
e664c4a6ac781116b31bec9ee2502e996738ce3c9e302bb81aa43bfb4f977350  ws/mcore/src/y4.rs
9f609c4980eb5571fc251d14187b47e7996005b5cfaa8829a596ff8b25142cd1  ws/mcore/src/y5.rs
422dfe4b314b676483223ae917978d6c23956d7d34d2d1f96f4e607c835d6747  ws/mcore/src/y6_impl.rs
3b4500f988b703201cf69ef5d8c334c5c1d6f35b066122572e1b09792f1b6d65  ws/mcore/src/y6_trait.rs
b8553d0d3e7a46d66fb00920d750fe32d9b89dfee8e27ad220fe86397473755f  ws/mcore/src/y7.rs
1121cfccd5913f0a63fec40a6ffd44ea64f9dc135c66634ba001d10bcf4302a2  ws/mcore/src/y8_k.txt
4f1803f1ca6fecfcfb1d6de8d7fa17f20e6e3ebe29996b1f20e987d00a48fcdc  ws/mcore/src/y8.rs
9c4f930deb8255a6705c0e2e6dd5023843c8473b82b9a7d64446cf7584f85fe4  ws/mcore/src/y9.rs
16a2d0b4560e891b859c4fa9be935752347b36db7446ccc55e97589f9e58a378  ws/mcore/src/y9_type.rs
f36af846d0ccae804c24533439dfcfde35ddc6b99bda9d305b38574e63dc4dfa  ws/mharness/Cargo.toml
f3075005ad2b70068ebdaa4cc90886d849f275da5abefb2fae1b826b28d5de29  ws/mharness/src/bin/model-tool.rs
0630791530c911bcfe4f6e25a1d387930e129660296a73e80e435c4474d6d29b  ws/mharness/src/lib.rs
5af0a5280d72f70207c76331178396d3c9733bb6b7916258be5231c6bb414efd  ws/mharness/tests/dispatch.rs
088fef15a2655660e331aed3e90c829ca8eb15b8aeabbba6c8e5b0fc2616564f  ws/mharness/tests/golden.rs
59c02164924fcde6c151c069f77f58dd234f90f9670ae170813501b37c152681  ws/mharness/tests/mixed.rs
e4d1ec46fd4d75a71466bf870b4db744783f7a9a8a4d71e0f891f64b58a6c928  ws/mharness/tests/tool_bin.rs
8d9b55eabb309fb91e09b2a82b54a0b87d8e6bb5ba253d5a312d7fbdd6c78021  ws/mharness/tests/ys.rs
```

## 二、U2②：C2「只跑覆盖到被改函数的测试」挑漏

被判的句子：`research/prompts/_tooling-broad-mutation-prune-r3-body.md:36`「  - 每一版在没改过的代码上跑一遍插桩的全部测试，得到「函数 → 覆盖到它的测试」；」、第 37 行「  - 每条变异先编（编不过记无效），再只跑覆盖到被改函数的测试，遇第一个失败即停；」、第 38 行「  - 零覆盖的记没红（即红）；」、第 39 行「  - 常量初始化式里的变异、只在常量上下文被叫的 const fn 跑全部测试。」。
它依据的证明：`research/prompts/tooling-broad-mutation-prune-r2-main-verification.md:34`「- 测试是确定的。一条测试在没改过的代码上一次都没执行到被改的函数，那么在变异版上，它执行到被改函数之前的每一步都与原版相同，所以永远到不了被改函数，结论也与原版相同（照样过）。」

装置：模型 `run.py` 的 C2 那半。插桩编一次，每条测试单独一个进程跑两遍（`LLVM_PROFILE_FILE` 不带 `%p` 的一个文件 / 每条测试一个目录带 `%p`），按三种取法导出入口计数大于 0 的函数（不带 %p 只交测试二进制、带 %p 只交测试二进制、带 %p 再加上全部 bin 当 `-object`）；挑「执行到被改函数」的测试，在不插桩的树上施加变异，挑出来的逐条跑、遇第一个失败即停，再 `cargo test --no-fail-fast` 全部跑一遍（含文档测试）。

### 2.1 打中的三格（模型上量过，三种取法都中）

| 格 | 形状 | 挑出来的 | 挑出来的结局 | 全部测试 | 原样输出 |
|---|---|---|---|---|---|
| X1b | 父测试起本测试二进制（`current_exe`）跑子测试，子进程打印完 `abort`（模拟崩溃的子进程）；父测试比打印的每一行 | 只有子测试自己（单跑只核行数） | 没红 | 抓到（父测试红） | `run-output.txt` 第 31–33 行 |
| X3 | `const fn` 运行时被一条弱测试叫，同时在测试文件的 `const` 项里被叫，强测试核那个常量 | 只有运行时那条弱测试 | 没红 | 抓到（核常量那条红） | 第 47–49 行 |
| X7 | 只有文档测试抓得到 | 无（nextest 不跑文档测试，覆盖里没有它） | 零覆盖 ⇒ 记没红 | 抓到（文档测试红） | 第 63–65 行 |

- `research/prompts/tooling-broad-mutation-prune-r3-opus-model/run-output.txt:33`「C2 X1b shape=子进程（current_exe 跑子测试，打印完 abort） policy=pid+all-binaries mutated=mcore/src/sub.rs:8 selected=['golden:test::child_prints_scores_then_crashes'] selected-verdict=missed first-failing=None all-tests-verdict=caught => HOLE」
- `research/prompts/tooling-broad-mutation-prune-r3-opus-model/run-output.txt:49`「C2 X3 shape=const fn 运行时与常量上下文都被叫 policy=pid+all-binaries mutated=mcore/src/mixed.rs:3 selected=['mixed:test::square_of_one_is_positive'] selected-verdict=missed first-failing=None all-tests-verdict=caught => HOLE」
- `research/prompts/tooling-broad-mutation-prune-r3-opus-model/run-output.txt:65`「C2 X7 shape=只有文档测试抓得到 policy=pid+all-binaries mutated=mcore/src/doc.rs:7 selected=[] selected-verdict=zero-coverage=>missed first-failing=None all-tests-verdict=caught => HOLE」
- 证明在哪一步断：X1b 是「插桩看得到执行」这个前提断——子进程 `abort` 不走退出处理，计数写不出来，被改函数确实执行了而覆盖说没有；X3 是「不在运行时执行」那条例外的判法断——正文只例外「只在常量上下文被叫的」，运行时也被叫的照覆盖挑；X7 是「全部测试」与「插桩跑的测试」不是同一批——文档测试在稳定版上量不了覆盖（第一轮攻方 3.6），第二段却跑它。
- 方向：三格都是「挑出来的没红、全部测试抓到」，按 C6 是误红（挡提交、要人去看），不是误绿；但按「等价变异写测试留档后排除」，人看到一条全部测试其实抓得到的「没红」，可能把它当等价变异排掉。

### 2.2 今天的 crates 里碰不碰得到（读代码；checker 档不跑）

- X1 这个形状在 checker 档里就有：`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs:870`「    let output = std::process::Command::new(std::env::current_exe().expect("测试二进制的路径"))」，子测试是同文件第 792 行「fn print_the_unsharded_enumerations_for_the_golden_comparison() {」，它单跑时只核行数（第 815 行「        "文件头 + 每片一个状态的 46 行片行"」），父测试核打印行的 sha256（第 904 行「            GOLDEN_PRINTED_LINES_SHA256.to_string()」）。子进程正常退出，模型里同形的 X1（正常退出）三种取法都挑到了父测试（`run-output.txt` 第 27–29 行 `=> same`），所以今天这一处不是洞；子进程改成被杀或 `abort` 就是 X1b。harness 档、core、checker、format 四个包里没有起子进程的测试（`grep -rn 'Command::new\|current_exe\|CARGO_BIN_EXE\|process::abort'` 在这四个包里 0 处）。
- X3 这个形状今天有一处：`crates/singlefs-core/src/address.rs:83`「    pub const fn to_device_offset(self) -> DeviceOffsetInBytes {」在运行时被叫，又在 harness 的常量项里被叫：`crates/singlefs-harness/src/new_pool_file_creation_regions.rs:127`「    SlotNumber(ROOT_RING_BASE_SLOT).to_device_offset().0」、第 132 行「    SlotNumber(JOURNAL_RING_START_SLOT).to_device_offset().0」。它的三条变异里两条（第 3、4 条）库里这一版被 core 自己的运行时测试抓到，挑得到，所以今天不翻；按 `const`/`static` 项里叫 const fn 去数（`evidence/c2_const_fn_contexts.py`，原样输出 `evidence/c2-const-fn-contexts.txt`；只认 `const X: T = …;` 与 `static` 项，没认数组长度、常量泛型、`const {}` 块），被变异的两个包的 const fn 里命中三个：`to_device_offset` 两边都被叫；另两个（`at_make_filesystem_checked_at_compile_time`、`row_publish_rewriting_instance_table_pages`）全仓 grep 只有定义与常量项，是正文已例外的「只在常量上下文被叫」。
- X7 今天 0 处：core、checker 的 src 里文档测试只有 `crates/singlefs-core/src/address.rs` 开头那一条 `compile_fail`，改函数体的变异让它翻不了面。

### 2.3 正文点名的那几种，模型上没打中（三种取法都挑到了抓它的测试）

- 跨 crate `#[inline(always)]`（X4）、`static` 函数指针表（X5）、`dyn` 分派（X6）：`run-output.txt` 第 53 行「C2 X4 shape=跨 crate #[inline(always)] policy=pid+all-binaries mutated=mcore/src/inl.rs:4 selected=['dispatch:test::mix_inlined_across_crates'] selected-verdict=caught first-failing=dispatch:test::mix_inlined_across_crates all-tests-verdict=caught => same」，第 57、61 行同。`-C instrument-coverage` 的计数在内联之前插进 MIR，怎么叫到都记在被叫函数名下。
- 子进程正常退出（X1）、先后两个子进程（X1c，不带 `%p` 的同一个文件）、只在另一个 bin 里执行的函数而 `llvm-cov export` 只交测试二进制（X2b）：都挑到了。例第 35 行「C2 X1c shape=子进程（先后两个子进程，只在第一个里执行） policy=naive+test-binary mutated=mcore/src/sub.rs:13 selected=['golden:test::child_prints_scores_c', 'golden:test::golden_parent_runs_two_children_in_turn'] selected-verdict=caught first-failing=golden:test::golden_parent_runs_two_children_in_turn all-tests-verdict=caught => same」。我开工前以为不带 `%p` 时后起的子进程会把前一个的计数盖掉、只交测试二进制会丢掉 bin 里的函数，这一版工具链（LLVM 22.1.8）上量出来都不丢；没查运行时为什么不丢。
- 所以子进程这一类真正的边界是「子进程有没有正常退出」，不是「有没有子进程」。

### 2.4 真 crates 副本：这一版库里 241 条第一段「抓到」，抓它的测试有没有被 C2 挑到（量过，一次取样）

装置 `evidence/crates_c2_selection.py`：HEAD 导出的副本（`git archive HEAD crates Cargo.toml Cargo.lock litmus .cargo`，218 个文件逐个 sha256 与库目录这一版 `manifest.tsv` 相同），第一段（harness 档四个包、不含 `#[ignore]`）776 条测试每条单独一个进程插桩跑一遍（6 条并行，测试自己的线程变量设 1），按「执行到被改函数」给清单 5 091 条挑测试，与库目录 `results.tsv` 比。
- `research/prompts/tooling-broad-mutation-prune-r3-opus-model/evidence/crates-cover.txt:2`「CRATES-COVER done tests=776 failed=0 wall-seconds=1129 sum-test-wall=6547 tests-with-child-profiles=0」
- `research/prompts/tooling-broad-mutation-prune-r3-opus-model/evidence/crates-c2-selection.txt:2`「CRATES-SELECT caught-in-stage-one=241 catcher-in-selected=238 catcher-not-in-selected=3」
- 挑漏的 3 条（第 3–5 行）都在常量初始化式里（第 5 条 `address.rs:90` 的 `pub const COUNTER_LIMIT: u64 = 1 << 48;`，第 13、14 条 `admission.rs:167` 的 `pub const INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE: u64 = INSTANCE_TABLE_PAGE_RECORDS - 1;`），正是正文第 39 行例外的那一类，按 C2 跑全部测试，不算挑漏。
- 零覆盖 256 条（`evidence/crates_zero_coverage_split.py`，原样输出 `evidence/crates-zero-coverage-split.txt` 第 1–9 行）：落在 fn 外（常量初始化式）170 条、fn 里 86 条（core 62、checker 24）。fn 里这 86 条，库里这一版判过的 3 条结局都是「只过了第一段」（第 6 行「ZERO-COVERAGE inside-fn library-outcome=stage-one-only count=3」），与「零覆盖 ⇒ 第一段没红」相合；其余 83 条没判过。
- 读法：今天的 crates 上，第一段抓到的 241 条里没有一条被 C2 挑漏；这个量够不着 2.1 的三格（第一段没有子进程、没有文档测试，X3 那一处被运行时测试抓到）。只抽了一次样（`.claude/rules/three-way-inference.md:159` 那一行），不支撑「没有」。

### 2.5 一个装置自己的坑，顺手记下

同一个插桩编译目录先后编过两份副本（工作树副本、HEAD 导出），cargo 沿用了没改过的 `singlefs-checker`、`singlefs-format` 的产物，覆盖里这两个包的文件名指着前一份副本的路径；按「文件名以副本根开头」过滤，checker 的 1 765 条变异就全成了零覆盖。改成按 `/crates/` 之后归一才对（第一轮 B7 说的是 `..` 与绝对前缀，这里是另一个来源）。上面的数都是归一之后重跑的。

## 三、U2①：C1 漏算

被判的句子：`research/prompts/_tooling-broad-mutation-prune-r3-body.md:31`「- **C1 哪些变异算「这次改到的」**：落在被改函数里的变异，加上下面两类：」、第 32 行「  - 被改文件里函数外面的部分（`use`、类型定义、`impl` 块头、属性、常量与 `static`）变了，那个文件里的全部变异；」、第 33 行「  - 新加或删掉的函数。」、第 34 行「  函数边界切不开的退回按文件。函数「改没改」比的是它的源码文本（去掉空白差异与注释？——要不要去掉是一问，见 U2）。」

装置：模型 `run.py` 的 C1 那半。每个场景第二版只动一处；被改函数「改没改」按三种读法判：
- R1：整个 `fn` 项（从 `pub`/`fn` 到函数体的 `}`），正则去块注释、行注释，空白折成一个空格（不认串）；
- R2：整个 `fn` 项，按词法单元比（串、原始串、字符各一个单元，注释与可嵌套块注释去掉；与 `crates/singlefs-checker-tier/src/crash_identity.rs:276`「pub fn lexical_tokens(source: &str) -> Vec<String> {」同一套切法，我在模型里另写了一份 Python）；
- R3：同 R2，但只比函数体 `{ … }`，签名与 `where` 不进函数文本（也不进「函数外面的部分」，因为外面的部分是去掉整个 fn 项之后剩下的）。
三种读法都另判「被改文件函数外面的部分」与「新加、删掉的函数」。然后两版各施加同一条变异，真跑全部测试。

| 格 | 第二版改了什么 | R1 | R2 | R3 | 第一版 + 变异 | 第二版 + 变异 | 原样输出 |
|---|---|---|---|---|---|---|---|
| Y1 | 被改函数里一个串字面量的空白（`"a  "` → `"a "`） | 沿用 | 重判 | 重判 | 抓到 | 没红 | `run-output.txt` 第 71–74 行 |
| Y2 | 被改函数里 `"/*"` 与 `"*/"` 两个串之间的一行代码 | 沿用 | 重判 | 重判 | 抓到 | 没红 | 第 79–82 行 |
| Y3 | 被改函数里 `"//"` 同一行后面的代码 | 沿用 | 重判 | 重判 | 抓到 | 没红 | 第 87–90 行 |
| Y4 | 另一个文件里的 `macro_rules!` 的展开式（被改函数调这个宏） | 沿用 | 沿用 | 沿用 | 抓到 | 没红 | 第 95–98 行 |
| Y5 | 泛型约束挪进 `where` 并换成另一个同名方法的 trait，函数体一字不变 | 重判 | 重判 | 沿用 | 抓到 | 没红 | 第 103–106 行 |
| Y6 | 另一个文件的 `impl` 里加一个覆盖 trait 默认方法的同名方法（变异在默认方法里） | 沿用 | 沿用 | 沿用 | 抓到 | 没红 | 第 111–114 行 |
| Y7 | `Cargo.toml` 的 `overflow-checks` | 沿用 | 沿用 | 沿用 | 抓到 | 没红 | 第 119–122 行 |
| Y8 | 被改函数 `include_str!` 读的数据文件 | 沿用 | 沿用 | 沿用 | 抓到 | 没红 | 第 127–130 行 |
| Y9 | 另一个文件给返回类型加 `#[derive(Default)]`（变异是 `replace make -> W with Default::default()`） | 沿用 | 沿用 | 沿用 | 无效 | 没红 | 第 135–138 行 |

- 例：`research/prompts/tooling-broad-mutation-prune-r3-opus-model/run-output.txt:72`「C1 Y1 reading=R1-regex-whole-item c1=reuse v1+M=caught v2+M=missed => HOLE(reused-verdict-flips)」，对照同一格 R2：第 73 行「C1 Y1 reading=R2-lexer-whole-item c1=rerun:function-text-changed v1+M=caught v2+M=missed => ok」。
- `research/prompts/tooling-broad-mutation-prune-r3-opus-model/run-output.txt:97`「C1 Y4 reading=R2-lexer-whole-item c1=reuse v1+M=caught v2+M=missed => HOLE(reused-verdict-flips)」
- `research/prompts/tooling-broad-mutation-prune-r3-opus-model/run-output.txt:106`「C1 Y5 reading=R3-lexer-body-only c1=reuse v1+M=caught v2+M=missed => HOLE(reused-verdict-flips)」，同一格 R2 第 105 行「C1 Y5 reading=R2-lexer-whole-item c1=rerun:function-text-changed v1+M=caught v2+M=missed => ok」。
- `research/prompts/tooling-broad-mutation-prune-r3-opus-model/run-output.txt:137`「C1 Y9 reading=R2-lexer-whole-item c1=reuse v1+M=unviable v2+M=missed => HOLE(reused-verdict-flips)」
- 每一格第二版的基线都是绿的（各格 `v2-baseline-failing=[]` 那一行，例第 71 行）。

### 3.1 按「改没改」的定义分三类

- **实现层（Y1–Y3、Y5）**：R2 修住 Y1–Y3（量过），整个 fn 项（含签名、`where`）进函数文本修住 Y5（量过）。要写进 C1 的是「按词法单元比整个 fn 项」，不是「去掉空白与注释」。今天的 crates 里 R1 会看错的串字面量（`evidence/c1-literals-in-crates.txt` 第 1 行「C1-LITERALS files=31 functions=1237 string-tokens-in-functions=1920 comment-markers-in-string=0 whitespace-run-or-newline-in-string=8」）：函数体里含 `//`、`/*`、`*/` 的串 0 个，含连续空白或换行的串 8 个（都在断言消息里）。今天碰得到的少，规则照样要写死。
- **被改函数自己的代码其实变了，文本没变（Y4、Y8）**：宏展开进函数、`include_str!` / `include_bytes!` / `include!` 读进函数。C1 第二条只把「宏定义所在文件」的变异算进来，用宏的函数在别的文件，算不进来。今天被变异的两个包里 `macro_rules!` 0 处、`include_str!` 等 0 处（`grep -rln` 在 `crates/singlefs-core/src`、`crates/singlefs-checker/src`），碰不到。
- **别处改动（Y6、Y7、Y9，以及测试与测试读的数据）**：用户定这一类靠里程碑末全量兜底（`research/prompts/_tooling-broad-mutation-prune-r3-body.md:12`「- 别处改动让没改的变异翻面这一类，靠里程碑末一次全量兜底。」）。Y6 正文第四节点名了（trait 默认方法被覆盖），在 C1 下它属于这一类：新加的覆盖方法按第 33 行算进来，被覆盖的默认方法里的变异算不进来。Y9 是新的一种：沿用的不只是「抓到」，还有「无效」——库里这一版判过的 303 条里 39 条无效，替换文全是 `Default::default()`（清单 5 091 条里这种替换 598 条）；别的文件给某个类型加 `Default`，这批里落在那个类型上的会从「无效」翻成编得过，可能没红。
- 这一类在今天的提交里有多常见（`evidence/c1-commit-classes.txt` 末行「C1-COMMIT-TOTALS commits=25 src=16 test-only=2 data-only=2 test-or-data-without-src=9」）：最近 25 个动了导出范围的提交，9 个只改了别的 `.rs`（harness、format、checker 档的源码与各包的测试）或数据、没动被变异的两个包的 src——这 9 个提交 C1 一条变异都不算，而它们改的正是判变异的那一方（例 `4e8b0370` 动了 24 个这一类文件，`evidence/c1-commit-classes.txt` 第 2 行）；改 `litmus/` 的 3 个（harness 有两条测试在运行时读它：`crates/singlefs-harness/tests/publish_order_matches_litmus.rs:13`「        .join("../../litmus")」）。

## 四、U2③：C4 按 CPU 时间限时，在多线程与 SMT 上的口径

被判的句子：`research/prompts/_tooling-broad-mutation-prune-r3-body.md:41`「- **C4 单条测试限时**：基线里那条测试的 CPU 时间乘倍数加下限，超了算那条测试没过，即抓到（用户定）；墙钟只作兜底。」
第二轮攻方量过的是「挤在同两个核上分时」：墙钟涨、CPU 不涨。这一轮量的是另两样：同一物理核的 SMT 兄弟被占、线程数变。装置 `evidence/c4_cpu_time.py`（`os.wait4` 取测试进程的用户加系统时间，`taskset` 钉核；本机 k 与 k+16 是同一物理核的两个 SMT 线程），测试二进制是工作树副本上编的两条 harness 测试，各 3 次，原样输出 `evidence/c4-cpu-time.txt`（量的前后负载 5.82 与 11.70，别的会话在跑）。

| 格 | 墙钟（秒） | CPU（秒） | 原样输出 |
|---|---|---|---|
| 单线程测试，核 8，兄弟 24 空 | 3.09–3.11 | 3.09–3.11 | 第 1 行 |
| 同上，兄弟 24 上一个空转进程 | 4.94 | 4.94 | 第 2 行 |
| 同上，另一个物理核 9 上一个空转进程（对照） | 3.20–3.97 | 3.20–3.97 | 第 3 行 |
| 16 格的多线程测试，1 线程，8 个逻辑 CPU | 99.39–100.71 | 99.38–100.70 | 第 4 行 |
| 同上，4 线程 | 33.48–33.53 | 101.15–101.32 | 第 5 行 |
| 同上，12 线程 | 17.53–21.71 | 106.69–133.90 | 第 6 行 |
| 同上，12 线程挤在 2 个逻辑 CPU（8、9） | 50.23–50.54 | 99.92–100.14 | 第 7 行 |
| 同上，6 线程钉 8–13，兄弟 24–29 空 | 25.46–25.64 | 103.17–104.14 | 第 8 行 |
| 同上，6 线程钉 8–13，兄弟 24–29 各一个空转 | 51.43–51.84 | 203.98–205.36 | 第 9 行 |

- `research/prompts/tooling-broad-mutation-prune-r3-opus-model/evidence/c4-cpu-time.txt:2`「C4 single cpu=8 spinner-on-sibling-24 reps=3 wall-min=4.94 wall-max=4.94 cpu-min=4.94 cpu-max=4.94 exit-codes=[0]」
- `research/prompts/tooling-broad-mutation-prune-r3-opus-model/evidence/c4-cpu-time.txt:9`「C4 multi threads=6 cpus=8-13 spinners-on-siblings-24-29 reps=3 wall-min=51.43 wall-max=51.84 cpu-min=203.98 cpu-max=205.36 exit-codes=[0]」
- 读法：
  - 分时（同一逻辑 CPU 上排队）不涨 CPU（第 7 行与第 4 行都约 100 秒），与第二轮一致；
  - SMT 兄弟被占，CPU 时间与墙钟一起涨：单线程 1.6 倍，6 线程 2.0 倍。本机 32 个逻辑 CPU 是 16 个物理核，变异跑在别的会话把机器占满的时候，测试的线程多半有一个忙着的兄弟。「CPU 时间不随负载涨」只在分时这一种负载下成立；
  - 多线程测试的 CPU 时间随线程数涨（1、4、12 线程：约 100、101、107–134 秒），12 线程那一格散到 1.26 倍，是线程多于 8 个逻辑 CPU 时的等待与切换。基线在哪个线程数下量、变异在哪个线程数下跑，要钉成同一个（`SINGLEFS_THREAD_CAP` 由 `capped.sh` 按派发设，两次跑可以不同）。
- 口径上没定的（推的，没造）：
  - 一条测试起子进程时（2.2 那条 checker 档 golden 用例），父进程的 CPU 时间不含子进程；`RLIMIT_CPU` 只管一个进程，按「那条测试」限得把子进程算进去，要按进程组或 cgroup 记；
  - nextest 不报每条测试的 CPU 时间，驱动要自己包一层（`wait4` 或每条测试一个 cgroup 读 `cpu.stat`）；
  - 不耗 CPU 的挂（等锁、等子进程）CPU 限时永远不到，靠墙钟兜底，墙钟兜底的倍数就是第二轮说的「两头都输」那一个。
- 倍数要盖住的散：只算这一轮量到的，SMT 一项就要 ≥ 2.0（6 线程那一格）；与第二轮库目录里长测试的散（墙钟 0.17–30.59）不是同一个量，没法直接比。

## 五、U2④：C6 第二段基线，两种修法的代价

被判的句子：`research/prompts/_tooling-broad-mutation-prune-r3-body.md:45`「  - 第二段基线先修绿：测试读的仓内文件进导出范围，或改测试不读导出树之外的文件。」

现查到的（读代码与库目录日志；checker 档不跑）：
- 红的是一条 lib 测试：`crates/singlefs-checker-tier/src/crash_identity.rs:1324`「            std::fs::read_to_string(root.join(".claude/gate.d/stage-inputs.tsv")).expect("登记表");」（用例名在第 1319 行起）。它只拿登记表里 `crash-case:` 开头的 8 行认出要核的测试目标，再按模块引用算判法代码，核的是「每条登记的崩溃用例的判法代码含恢复与 checker 走树」——读源码文本的结构，不执行被变异的函数，也不随函数体的变异翻面（推的：模块引用由 `mod`/`use`/路径定，变异只改函数体与返回值）。
- 这一版第二段 checker 档那一步只跑到 lib：`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/logs/baseline.two-test-checker-tier.log:2`「     Running unittests src/lib.rs (/home/fy5090/.local/share/singlefs/broad-mutation/target/release/deps/singlefs_checker_tier-17e304590552b086)」，第 68 行「test result: FAILED. 50 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 47.15s」，之后 cargo 就停了（不带 `--no-fail-fast`）。checker 档 `tests/` 下 33 个测试目标在这一版一个都没跑过：修好这一条之后基线绿不绿、整步要多久，今天都不知道。
- 读仓根的别的 checker 档测试（`PipelineJudgingCode::of_the_judges` 那几处、`crash_identity` 的副本比对）读的是 `crates/` 与 `Cargo.lock`，在导出范围里；`grep` 在 checker 档里找到的读 `.claude/` 的只有上面这一处（非注释行）。

| 修法 | 改什么 | 代价 | 量过 / 推的 |
|---|---|---|---|
| 甲：`.claude/gate.d/stage-inputs.tsv` 进导出范围 | 驱动的导出范围加一个文件（44 行） | 今天的驱动按整棵树认版本：这个文件 9 月以来 204 个提交里改了 8 次，每改一次就是新的一版；按第三版的 C1，它不是 `.rs`，改了 C1 什么都不算（同 Y7、Y8），这条测试本来也不抓变异，所以对结论没影响。代价只在「门禁登记表进了变异的导出树」这个耦合 | 次数是 `git log --since=2026-09-01` 数的；其余推的 |
| 乙：改这条测试不读导出树之外 | 一条 lib 测试：要么把要核的用例清单放进 crates（与登记表各一份，再由门禁核两份相同），要么找不到登记表时只核 `tests/` 下按名字认得出的崩溃用例 | 改一条测试；多一份要同步的清单（第一种）或少核一点（第二种） | 推的 |
| 两种都要的一步 | 第二段 checker 档那一步加 `--no-fail-fast` 重量一次基线，才知道 33 个目标里还有没有别的红、整步多久 | 一次 checker 档整步（重型，子 agent 不跑） | 没量 |

## 六、U3：代价（第一段量过的部分与推的部分分开写）

### 6.1 C2 每条变异要跑多少测试（真 crates 副本量过，第一段）

- `research/prompts/tooling-broad-mutation-prune-r3-opus-model/evidence/crates-c2-selection.txt:8`「CRATES-SELECT package=all mutants=5091 zero-coverage=256 selected-median=182 selected-p90=499 selected-max=586 seconds-median=2517.0 seconds-p90=7582.0 seconds-max=7690.0 seconds-sum=19077430」
- 秒数是把挑出来的测试在库目录基线日志 `logs/baseline.one-test.log` 里的耗时加起来（不是墙钟）；全部 776 条加起来 7 691.1 秒（第 1 行），整遍墙钟 278.5 秒（库目录 `baseline.tsv` 第 3 行「one-test	ok	278.5	2026-09-30T15:16:40Z」）。
- 按包分（第 6、7 行）：core 的变异挑出的测试耗时中位 5 607 秒，是全部的 73%；checker 的中位 2 452 秒，32%。core 的函数几乎条条被 harness 那些建池、挂载、发布的测试走过，按覆盖挑在 core 上省得很少。
- 「遇第一个失败即停」才是省时的大头：库里这一版 242 条抓到，测试秒数中位 1.0；第一段没红的 15 条中位 620.5（`results.tsv` 第 8 列按结局分，我现算的）。C2 省的是没红的那批每条从整遍降到挑出来的那部分。

### 6.2 每次提交与里程碑末全量（推的）

`evidence/u3_commit_cost.py`：最近 15 个动了 core / checker src 的提交，按 C1 的 R2 读法（词法单元比整个 fn 项；函数外部分变了算整个文件；新加、删掉的函数算）数「这次改到的」变异（清单用库目录这一版的，按 HEAD 上的函数归属对过去），每条按「编译 20.4 秒（这一版抓到的编译秒数平均）+ 278.5 秒 × 挑出来的测试耗时 ÷ 7 691.1」算上界，零覆盖在 fn 外的算整遍、在 fn 里的不跑。
- `research/prompts/tooling-broad-mutation-prune-r3-opus-model/evidence/u3-commit-cost.txt:16`「U3-TOTALS commits=15 mutants-median=1865 hours-median=82.0 hours-min=0.0 hours-max=188.5 milestone-full-all-5091-mutants-hours-upper=233.9 same-without-c2-selection-hours=422.7」
- 例：第 8 行「U3-COMMIT 8186d5b5 files=1 whole-file-rerun=0 changed-functions=1 mutants=2 hours-upper=0.0」；第 10 行「U3-COMMIT 495bede6 files=12 whole-file-rerun=0 changed-functions=5 mutants=118 hours-upper=3.6」；第 1 行「U3-COMMIT dbcc0d2f files=9 whole-file-rerun=9 changed-functions=42 mutants=3006 hours-upper=123.0」。
- 15 个提交里 10 个有文件的「函数外面的部分」变了（`whole-file-rerun` 不为 0，`awk` 数的），这一条把整个文件拉进来，是每次提交代价的大头；其余 5 个只改函数体，在 0–3.6 小时。
- 下界（每条至少编一次、抓到的测试几乎不花时间）：中位那个提交 1 865 条 × 20.4 秒 ≈ 10.6 小时；里程碑末全量 5 091 条 × 20.4 秒 ≈ 28.8 小时。上界：中位 82.0 小时，全量 233.9 小时（不挑测试 422.7 小时）。真值在两者之间，取决于没红的有多少条。
- 只算了第一段。第二段（耗时 harness 用例 936 秒、文档测试 1.5 秒、checker 档整步没量过，见 5 节）都没进这个数；按覆盖挑的话第二段也要各量一遍覆盖，checker 档要在 release 下插桩。
- 编译 20.4 秒是在别的会话占着机器时量的，按单驱动、一条一条编；每条变异都要编一次，C2 不减编译。

## 七、打中之后的四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md:126` 那一句）

判据：`research/prompts/_tooling-broad-mutation-prune-r3-body.md:76`「- U2 造出「C2 挑出来的测试没红、全部测试抓到」 ⇒ C2 的证明有洞，补规则（例如那一类一律跑全部测试），写进判决、标零轮。」、第 77 行「- U2 造出 C1 漏算 ⇒ 补 C1 的规则，同上。」

| 打中 | 分不分辨臂 | 系统当时看不看得到 | 满足判据哪一句 | 改法在这几格上 |
|---|---|---|---|---|
| X1b 子进程 abort | 分辨「按覆盖挑」与「跑全部」；三种覆盖取法都中（不分辨取法） | 只看覆盖看不到（abort 的子进程不写计数）；看进程看得到：覆盖那一遍记下每条测试起过几个进程（`execve` 计数或 cgroup 的 `pids.peak`），与 profraw 文件数比 | 第 76 行，模型上；今天的 crates 里同形的那一处子进程正常退出，不中 | 「起过子进程而 profraw 少于进程数的测试，算覆盖到一切」修它（推的，没实现）；只按「有没有子进程」判也修，但把 X1 那种不需要的也拉进来 |
| X3 const fn 两边都被叫 | 分辨；不分辨覆盖取法 | 静态看得到：常量上下文里叫了哪些 const fn 是语法 | 第 76 行，模型上；crates 有一处同形（`to_device_offset`），今天被运行时测试抓到，不翻 | 把正文第 39 行的「只在常量上下文被叫的 const fn」改成「在任何常量上下文里被叫过的 const fn」修它（推的；按定义它就跑全部测试，结局与「全部测试」那一列相同） |
| X7 只有文档测试抓得到 | 分辨 | 看得到：文档测试在源码里 | 第 76 行，模型上；今天 0 处 | 被变异的包的文档测试每条变异都跑（第一轮 B8；这一版第二段那一步 1.5 秒）修它（推的） |
| Y1–Y3 串里的空白与注释记号 | 分辨 R1 与 R2 | 看得到 | 第 77 行，模型上；今天 crates 里会看错的串 8 个（都只有空白），注释记号 0 个 | R2 修住（量过，`run-output.txt` 第 73、81、89 行） |
| Y5 约束换了、函数体不变 | 分辨 R3 与 R1、R2 | 看得到 | 第 77 行，模型上 | 函数文本取整个 fn 项修住（量过，第 105 行） |
| Y4、Y8 宏 / include 进函数 | 三种读法一起中（打中不分辨读法，病在「函数文本」这个定义） | 静态看得到（宏名、include 路径在函数体里） | 第 77 行，模型上；今天 0 处 | 「宏定义变了，凡函数体里出现这个宏名的函数都算改了；include 的文件变了，凡 include 它的函数都算改了」修它（推的） |
| Y6、Y7、Y9 别处改动 | 三种读法一起中 | 看得到改了什么文件，看不到它翻了哪条变异 | 字面上满足第 77 行；按第 12 行它属于用户定的里程碑末兜底那一类，不按「漏算」判 | 正文没给改法；第八节的两条都推的 |

## 八、几个改法各修哪一格（都只在我的模型上量过或没量，被攻过零轮）

| 改法 | 修哪几格 | 量过 / 推的 |
|---|---|---|
| C1 的「函数文本」= 整个 fn 项（含属性之外的签名、`where`），按词法单元比（可以直接用 `crates/singlefs-checker-tier/src/crash_identity.rs` 的 `lexical_tokens`） | Y1、Y2、Y3、Y5 | 量过（模型 R2，`run-output.txt` 第 73、81、89、105 行） |
| C1 加一条：宏定义（`macro_rules!`）或被 `include_str!` / `include_bytes!` / `include!` 读的文件变了，凡用到它的函数都算改了 | Y4、Y8 | 推的 |
| C1 加一条：导出范围里的 `Cargo.toml`、`Cargo.lock`、`.cargo/`、工具链变了，全部重判（第一轮 B6 的全局键，第三版里没有了） | Y7 | 推的 |
| C3 加一条：「无效」不沿用，每次提交重编（库里这一版无效的编译秒数中位 0.6；按 303 条里 39 条的比例，全表约 650 条、十分钟上下） | Y9 | 比例与秒数是库目录现算的，规则推的 |
| C2 第 39 行改成「在任何常量上下文里被叫过的 const fn 跑全部测试」，常量上下文含 `const`/`static` 项、数组长度、常量泛型、`const {}` 块 | X3 | 推的 |
| C2 加：被变异的包的文档测试每条都跑 | X7 | 推的（第一轮 B8 已有，第三版丢了） |
| C2 加：覆盖那一遍记每条测试起的进程数，profraw 少于进程数的测试算覆盖到一切 | X1b | 推的 |
| C4：基线与变异跑在同一个线程数下；限时按进程组（含子进程）记 CPU；倍数至少盖住 SMT 的 2.0 倍 | 4 节 | SMT 与线程数的散量过（副本），规则推的 |
| C6：第二段 checker 档那一步加 `--no-fail-fast` 重量一次基线 | 5 节 | 没量（重型） |

## 九、没打中的形状（各只取样一次，不支撑「没有」：`.claude/rules/three-way-inference.md:159` 那一行）

| 形状 | 试了什么、取样多大 | 为什么没打中 |
|---|---|---|
| 跨 crate `#[inline(always)]`、函数指针、`dyn` 分派 | 模型 X4–X6，每格一条变异、三种覆盖取法 | 计数插在内联之前，按被叫函数记 |
| 子进程正常退出、先后两个子进程、只在另一个 bin 里执行 | 模型 X1、X1c、X2、X2b | 这一版工具链上计数都没丢（2.3） |
| 真 crates 第一段：抓到的变异，抓它的测试没被挑到 | 241 条全取（库目录这一版第一段的全部抓到） | 3 条挑漏都在常量初始化式里，正文已例外（2.4） |
| 测试读源码文本（`include_str!` 读被变异文件、按词法摘要比对） | 读代码：`crates/singlefs-harness/src/model_comparison.rs` 里一条测试读 `model.rs` 原文，不在被变异的两个包里；checker 档 `crash_identity` 读源码算摘要的测试比的是同一棵树的两份副本 | 今天被变异的文件没有被测试按原文读的 |
| 环境决定走哪条路（`debug_assertions`、`cfg(feature)`、`HashMap` 次序、`Instant`、线程数） | `grep` core、checker 的 src | 都是 0 处；线程数与计时只在 harness 与测试里，只决定分片与打印 |
| C3 按描述存，格式化（rustfmt）之后函数内相对位置漂了 | 读规则，没造 | 按词法单元比时函数「没改」，而「函数内相对位置」若按行列存就对不上，沿用的结论找不到 ⇒ 按 C6 是「沿用的结论不在」⇒ 红或重判，方向安全，代价是一次格式化提交重判整片（推的）。存成「函数内第几个词法单元」就不漂（推的） |

## 十、这条腿自己的限度

- C1、C2 的打中都只在模型上造出、真跑；真 crates 上只量了 C2 在第一段抓到的 241 条（2.4），没在 crates 上造 X1b、X3、X7 的翻面（今天 crates 里没有同形的翻面点，造就要写新代码）。
- 每个场景只取了一种第二版改法与一条变异；「用户决定的那几步」在这里是第二版怎么改，没放开扫。各读法「不中」只对那一种改法成立。
- 模型里的切函数与词法切法是我写的 Python，不是 crates 里的 `lexical_tokens`；R1 是我挑的一种「去注释与空白」的写法，正文没说要用哪种，别的写法未必在 Y1–Y3 上中。
- C4 的数只量了两条 harness 测试，各 3 次，钉核在 8–15 与它们的 SMT 兄弟上；别的会话在跑（负载 5.82 → 11.70）。12 线程那一格散到 1.26 倍，分不开是线程超订还是别的会话。测试二进制是工作树副本上编的（与 HEAD 有 28 个文件不同，两条测试的文件里只有 `entries_after_mount_refuse_swapped_or_behind_devices.rs` 在其中），只拿它比 CPU 与墙钟，不拿它比结局。
- 2.4、6.1 的覆盖在测试线程变量设 1 时量；基线日志的耗时是库目录在别的负载与线程数下量的。6.2 的小时数是推的上下界，只算第一段。
- 覆盖那一遍跑了三次，进仓的是第三次：第一次在工作树副本上（与库目录这一版 28 个文件不同，变异的行号对不上），作废；第二次在 HEAD 导出上 12 条并行，跑到 727 条撞 8G 上限（包装退 250，那一次的输出不算），4 条并行续完之后又发现 2.5 的路径问题，整遍作废；第三次 6 条并行重跑。作废的输出没进仓。
- 一切都只抽了一次样，没有 s2。

## 十一、收尾

- 删了（草稿目录 `/tmp/claude-1000/tooling-broad-mutation-prune-r3-opus/` 下）：`target-cov` 23G（插桩编译目录）、`target-plain` 848M（不插桩编译目录）、`work` 317M（模型各场景的树与编译目录）、`tools` 236M（llvm-tools 1.98.0）、`crates-cover-head` 61M（每条测试的覆盖结果）、`repo` 12M（工作树副本）、`repo-head` 11M（HEAD 导出）；更早删了 `try-target` 79M 与 `try` 148K（模型试编）。草稿目录里留着日志、草稿与一份 const fn 扫描脚本，共 160K。
- 运行时看到的负载：开工时别的会话有两条 `cargo build --release -p singlefs-checker-tier` 与一条 `e168` 装置在跑，负载 14；之后 3–12。没有性能测量在跑，没等 cargo 的锁。内存包装因为峰值表里这几条命令的实测峰值远低于 8G，要求写理由才放行，设了 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`。
- C4 第一次起的那一趟里有 16 线程的格，超过派发的线程上限 12；量完单线程三格之后停掉（逐个 `proc.py stop`，停完列过一遍不剩），改成上限 12 重跑，进仓的是重跑的输出。
- 没做：没跑 checker 档与第二段、没起广谱驱动与 cargo-mutants；没在真 crates 上造 X1b、X3、X7、Y 各格的翻面；没判正推那几格；没替主 agent 采纳任何改法。
- `python3 research/scripts/cite-check.py research/prompts/tooling-broad-mutation-prune-r3-opus-output.md` 末行：「✓ 核了 24 处引文，对上 24 处，没判 0 处（逐处列在上面）」（加这一节之前跑的）。
