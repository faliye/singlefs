# tooling-broad-mutation-prune-r1 攻方腿（Opus）报告：Q2、Q3

日期 2026-10-01。被判对象：`research/prompts/_tooling-broad-mutation-prune-r1-body.md` 第三节 P1–P5。前几轮判决：无。
真 crates 上的数都是我副本上的（HEAD 导出，218 个文件逐个 sha256 与库目录这一版 manifest 相同），不是入库装置上的数；模型上的改法与判定只在我的模型上量过、被攻过零轮。

## 各格判定一览

| 格 | 形状 | 判定 | 在哪量的 | 节 |
|---|---|---|---|---|
| Q2-常量-测试文件引用 | 只放常量的文件被测试文件引用、被改文件不引用；第二版收窄取样常量 | 打中：P1–P3 两种读法都判「复用抓到」，重跑没红 | 模型 S1（量过） | 2.1 |
| Q2-常量-两跳 | 被改文件引用的常量由另一文件的常量算出；第二版改第二跳 | 打中：P3 只并进第一跳，判「复用抓到」，重跑没红 | 模型 S2（量过） | 2.2 |
| Q2-执行集合取哪一版 | 另一个文件加 / 删一个与 trait 同名的固有方法 | 打中：读法 A（第一版覆盖）被 S4a 打穿，读法 B（第二版覆盖）被 S4b 打穿；并集两格都不中 | 模型 S4a、S4b（量过；并集按输出的集合算） | 2.3 |
| Q2-测试读的数据文件 | checker 档快档测试运行时读 `.claude/gate.d/stage-inputs.tsv`，不在导出树、不在任何键里 | 打中（这类文件今天就有）；没在副本上造「改它而翻面」，checker 档不跑 | 库目录产物 | 2.4 |
| Q2-`#[path]` 共享的 common | llvm-cov 给的名字带 `..` 与绝对前缀 | 实现层的坑：不归一就查不到 manifest | 模型 S6（量过） | 2.5 |
| Q2-环境与驱动参数 | 负载、线程数、超时倍数不在键里 | 打中不分辨臂（凡按文件定键的设计一起中）；第 556 条副本重跑见 2.6a | 库目录产物、副本 | 2.6 |
| Q2-宏 | 宏定义文件不进执行集合 | 量过盲区；crates 今天零处 `macro_rules!`，碰不到 | 模型 S5 | 2.7 |
| Q2-derive | derive 出来的实现执行了也不记 | 量过盲区；没造出基线绿的翻面（推的） | 真 crates 副本 | 2.7 |
| Q2-泛型、`#[inline]`、闭包 | 记在哪个文件 | 没打中：记回定义它们的文件 | 模型 S8 | 2.7 |
| Q2-共享状态、执行次序、时间、随机数、panic 路径 | 见第五节 | 没打中（一次取样） | 读代码 | 六 |
| Q3-常量初始化式 | 覆盖没有这一行的区域，P2「零覆盖不跑就记红」 | 打中：170 条（3.3%）落在这里；这一版判过的 4 条里 3 条抓到 | 真 crates 副本 | 3.1 |
| Q3-编译期 const fn | 行有区域、计数 0；真结局是无效 | 打中：第 2602 条编不过，P2 记红 | 真 crates 副本 | 3.2 |
| Q3-死循环记超时 | 确定挂住的变异按 P5 永远红 | 打中：第 102 条单跑 300 秒不结束，不改时 0.00 秒 | 真 crates 副本 | 3.3 |
| Q3-负载超时 | 超时阈值随负载 | 打中（库目录产物）；副本重跑见 2.6a | 库目录、副本 | 3.4 |
| Q3-只过第一段 | 第二段基线本身红，15 条一条也补不完 | 打中：红在基线，不在变异 | 库目录产物 | 3.5 |
| Q3-文档测试覆盖 | 稳定版量不了 | 今天碰不到（只有一条 compile_fail） | 副本 | 3.6 |
| Q3-覆盖弄脏专用树 | 插桩二进制在当前目录写 profraw | 实现层：驱动核树退 5 | 模型第一次跑 | 3.7 |

## 一、复跑命令与文件 sha256

模型（`research/prompts/tooling-broad-mutation-prune-r1-opus-model/`，说明在它的 `README.md`）：

```
bash research/scripts/run-with-memory-cap.sh 1G bash research/scripts/capped.sh 12 nice -n 19 python3 research/prompts/tooling-broad-mutation-prune-r1-opus-model/run.py <工作目录> <llvm-tools 1.98.0 的 bin 目录>
```

llvm 工具：`curl -sSfO https://static.rust-lang.org/dist/2026-08-20/llvm-tools-1.98.0-x86_64-unknown-linux-gnu.tar.xz` 解到草稿目录（下载加解压 10 秒），版本行「LLVM version 22.1.8-rust-1.98.0-stable」，与 `rustc -Vv` 的 LLVM 22.1.8 一致；没装 cargo-llvm-cov，没动 `~/.rustup`。

真 crates 副本：`git archive HEAD crates Cargo.toml Cargo.lock litmus .cargo | tar -x -C <副本>`；施加一条变异 `python3 research/prompts/tooling-broad-mutation-prune-r1-opus-model/apply.py <副本> <库目录版本目录>/list.json <序号>`，拷回 `apply.py restore <副本>`；插桩用 `RUSTFLAGS="-C instrument-coverage"` 另开编译目录，跑法与原样输出在 `crates-copy-evidence/` 各文件第一行。

```
db9fd43c2c11686bacc2fde864558b1184a6bf139ba1fb7a8c71ef8e0bd9a7bb  apply.py
fb03e58df764038c1f89b099422a25c630786eaa4a0003c7d2629a3d42867f9f  crates-copy-evidence/address-rs-coverage-one-test.txt
49e847a15b97cd548f7c6667ebefe45e52c52aa19e131f1f4a7c65796532c466  crates-copy-evidence/const-initializer-mutations.txt
a0458817b2585d57b2968445f3ffacf297163f99e6203ede338241b3abb38829  crates-copy-evidence/doctest-coverage-on-stable.txt
cd9f215a78251d0bf8f6fea311b745ccfd181991b64d9941d3b0afa4be14b4d2  crates-copy-evidence/mutation-102-single-test.txt
e43e2ddc1cefefe17447757859f5282b04131f9254d224ed59286c7d00a15c42  crates-copy-evidence/mutation-2602-build.txt
90426422c68a5ccbfd2bf65b882553ff5fadef95a73b16305595c27d2a8dddb0  crates-copy-evidence/mutation-5-run.txt
79c18544a1da4fce8f5c3d246c0fab1177f327216ccb2419600a0868c3337fe2  crates-copy-evidence/root-ring-rs-coverage-all-core-lib-tests.txt
c14b19e5baa84841cb13b515ca50bf79cab08faa8c0edba1bd810770042a83d7  outside_fn.py
858456b169e83d8b7a18b78cd48ede6c71c3db704c3c31893af57c8f0348aad8  README.md
feadf4951b7ad4f6bd1c3f384ac49a249459f506ddf7b6bd6a9f004374ad8268  run-output-first-attempt-profraw-left-in-tree.txt
46a685e36518d39e9fbae68a41f05195bbc4f0f28d09daab74da2f33e0ebc59c  run-output.txt
ed81ab238307960a09f6ca98c9b4367fd04b53400497f9cd43fe064a94644615  run.py
85a9ca09fc3bd1ab2e8dc7a0a6d0e237da84d80df99a27982499c7dfbfb03cf3  scenarios.tsv
7001f3fed53ea9bc07a92f2fabac4a05fff2106502f9721a3639425b0b8c57b2  v2/S1-fmtc-lib.rs
15d420ddda5e02cb58b0a51c6c0046ea48cd7d030251cb674d6aba29cd9675fb  v2/S2-fmtc-lib.rs
14f42bfa0a5de28e4fd17dc974e91f8f381a9738ce2f0cf0a5aee6f06869b7c2  v2/S4a-core-extra.rs
d6e4f23202c241469bbae40873c98ac4af7ca1d3c39f90aa3250425b9031c26b  v2/S4b-core-extra.rs
4440dd38c33760e81dec5a2b4faa098267274aba7ef958216205aa40aa6a0ea8  ws/core/Cargo.toml
ea8201dacba8263701fe60d958ed029521262fc2e5978b51f7770621a69f4291  ws/core/src/constfn.rs
f4aea21671f71326783ba639284303f35c90c08778c13fe3b3c96000c75afd1b  ws/core/src/extra.rs
78bfbd79d68cbb8d7b5e21c8307493f25b174c411b84e718b7cea1680ad25d87  ws/core/src/generic.rs
646e03c192794d7cfcc5d02ed3908f0f8bb7964f06ceed49d048d61558a14c34  ws/core/src/layout.rs
0e837a87be5511defcf1aff090a08460b7b7455f68a98f49effa2151e113cb59  ws/core/src/lib.rs
b1e220004e07a7e62fbf6ab1a25103e43d232ef84856de125f8b9098123aeac1  ws/core/src/macros_def.rs
86d56c7cccce1f4d519071f14f557d43d6f535002f5ecdbad3a24bf78c3fff59  ws/core/src/macro_user.rs
0c70e15f093104d26cb87998957c271b5a1dfad9d88ffbd1c5596e523334d117  ws/core/src/render.rs
c937155afd03cd35e5282dc3ab343cdf45bcf635c081b803071678e48fc37f33  ws/core/src/ring.rs
30da92961fb4e0ab67ef7404cc0089621530bb5e86cfb07bb377d80872ae0f10  ws/core/src/split.rs
ce0b5a949ad598a3e18fb71412c9788af4502b9e013705b55e2d6cd01cee7615  ws/core/src/thing.rs
38d6a93b04c9a5fe341da3e208e360442497515be7dd326461047b93f8723570  ws/fmtc/Cargo.toml
f3098b058b2b3bdfaf305f82399c411a6cc20b67b25105d2202fca19204e9372  ws/fmtc/src/lib.rs
997c189883b7cfd9b15ff07be9c3603ffbcf7884956680643a1a21b7ba3ac1a6  ws/harness/Cargo.toml
ebd1098b61b9eb87a1588c0b2b2a6441c75eae69d2e6a8082e6d652bed2be0ea  ws/harness/src/lib.rs
f61e47e84db65c400c0ad8df98a9cce47fadf8c58763af102673f646fe322bab  ws/harness/tests/common/mod.rs
e0bc20e4f314efd74614c43a7e61d0a69fc02516540aa23b604b62d124a3ade4  ws/harness/tests/const_values.rs
d2bb2c1eb361d8a030257987e8fffbe1c718fcb34010e2e8d5ad5115050ae16c  ws/harness/tests/generic_inline_closure.rs
216449ec1a34ea04a0c531ba9ade3e1beb629b344f76f6fbc83a7c3700b6af9a  ws/harness/tests/macro_wrap.rs
774768309cd749c39fe48ca15fb08740660945fd8d85a444e023333c85ee6e0d  ws/harness/tests/ring_slots.rs
da708d2f75d66e5beca124057d8fd853a93fa047c19486cfa6a02864ba03fa5b  ws/harness/tests/split_units.rs
5de68fa5acb2bddfc71a1bad84a7005b8baa5252800aa05467edb261b0d0df94  ws/harness/tests/thing_describe.rs
a95c56103df76d2f11d16b9794d31ac128de9ad2123df802bc3768d7883f8e8e  ws/tier/Cargo.toml
25bd332ef4543ab5644aca1da0566c8bc3b2a0ecfd31d121b9f217971e58e2e7  ws/tier/src/lib.rs
8b1c5d1058aad8c4b4de779784e4c7833594bd5c3df34f5e48d2bd3c4af72178  ws/tier/tests/uses_harness_common.rs
```

（第 556 条副本重跑的输出另落一份，sha256 写在 2.6a。）

## 二、Q2：「关联文件都没变、而结论变了」

判法照正文第三节：「抓到的（被测试 T 抓到）：被改文件、T 所在的测试文件、T 执行到的全部源文件，都没变才复用。」（`research/prompts/_tooling-broad-mutation-prune-r1-body.md:46`），P3 只加「被改文件里引用到的常量，它的定义文件并进关联文件。」（`research/prompts/_tooling-broad-mutation-prune-r1-body.md:51`）。
模型里每个场景第二版只换一个文件；「判可复用」按 run.py 算出的关联集与改动文件的交集为空判；「重跑」是第二版加同一处变异真跑全部测试。

### 2.1 常量的定义文件不被被改文件引用（S1，量过，模型）

- 形状：取样上沿 `SAMPLE_LIMIT` 放在只有常量的 crate（照 `crates/singlefs-format/src/lib.rs`，那份文件非测试部分只有常量）；引用它的是测试文件，不是被改文件 `split.rs`。第二版把上沿从 25 收到 8（「取样缩一点」）。
- 输出（`research/prompts/tooling-broad-mutation-prune-r1-opus-model/run-output.txt`）：
  - `research/prompts/tooling-broad-mutation-prune-r1-opus-model/run-output.txt:12`：「changed=fmtc/src/lib.rs P1-P3=reuse caught」（读法 A）；第 13 行读法 B 同样「reuse caught」。
  - `research/prompts/tooling-broad-mutation-prune-r1-opus-model/run-output.txt:14`：「v2-baseline=green v2+M failing=none => rerun-verdict=missed」。
- 每一步许可它的那一句：P2 只认「T 执行到的全部源文件」；只有常量的文件没有覆盖区域（2.6 节真 crates 上量过常量初始化式那一行没有计数）；P3 只看「被改文件里引用到的」常量。

### 2.2 常量链两跳（S2，量过，模型）

- 形状：被改文件 `ring.rs` 引用 `layout.rs` 的 `REGION_SLOTS`（P3 把 layout.rs 并进来），`REGION_SLOTS = fmtc::SLOTS_AT_MKFS * 3`，fmtc 在第二跳。第二版把 8 改成 16（照 `crates/singlefs-format/src/lib.rs:256`「pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;」那条 4..16 之间可取的常量）。
- 输出：`research/prompts/tooling-broad-mutation-prune-r1-opus-model/run-output.txt:16`：「associated=core/src/layout.rs,core/src/ring.rs,harness/tests/ring_slots.rs changed=fmtc/src/lib.rs P1-P3=reuse caught」；第 18 行「v2-baseline=green v2+M failing=none => rerun-verdict=missed」。
- 真 crates 上的同形：`crates/singlefs-core/src/admission.rs:167`「pub const INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE: u64 = INSTANCE_TABLE_PAGE_RECORDS - 1;」就是一个经别的文件常量算出来的常量；引用它的文件要是不直接引 singlefs-format，format 那份文件就在第二跳。

### 2.3 「T 执行到的文件」取哪一版的覆盖，正文没写；两种读法各被一个场景打穿（S4a、S4b，量过，模型）

- 形状：Rust 方法解析先选固有方法、后选 trait 方法。固有 `impl` 块可以写在同 crate 的任何文件里。
  - S4a：第二版在另一个文件 `extra.rs` 里给 `Thing` 加一个与 trait 同名、结果相同的固有方法。T 改叫它，被改文件（trait 实现）里的变异不再被 T 叫到。
  - S4b：第一版 `Gadget` 的固有方法在 `extra.rs`，经 `render.rs`（被改文件）出结果；第二版删掉这个固有方法，T 落回 `thing.rs` 里 trait 的实现，不再经过 `render.rs`。
- 输出：
  - `research/prompts/tooling-broad-mutation-prune-r1-opus-model/run-output.txt:20`：「associated=core/src/thing.rs,harness/tests/thing_describe.rs changed=core/src/extra.rs P1-P3=reuse caught」（S4a 读法 A，用第一版覆盖）；第 21 行读法 B「rerun」；第 22 行重跑「missed」。
  - `research/prompts/tooling-broad-mutation-prune-r1-opus-model/run-output.txt:25`：「associated=core/src/render.rs,core/src/thing.rs,harness/tests/thing_describe.rs changed=core/src/extra.rs P1-P3=reuse caught」（S4b 读法 B，用第二版覆盖）；第 24 行读法 A「rerun」；第 26 行重跑「missed」。
- 两版覆盖取并集（按这几行输出里的关联集算，没另跑）：S4a 的并集含第 21 行的 extra.rs，S4b 的并集含第 24 行的 extra.rs，两格都判重跑。
- 真 crates：前提（trait 实现与用方法语法叫它的测试）处处都有；没在 crates 上造这一份第二版（推的）。

### 2.4 测试读的数据文件：今天 crates 里就有测试读导出树之外的文件（打中，库目录产物）

- `crates/singlefs-checker-tier/src/crash_identity.rs:1324`「std::fs::read_to_string(root.join(".claude/gate.d/stage-inputs.tsv")).expect("登记表");」：checker 档快档里的这条测试在运行时读 `.claude/gate.d/stage-inputs.tsv`。
- 驱动的导出范围（`research/scripts/broad-mutation.py:47`「导出范围取门禁 checker-tier-crates-mutation-replay 登记的拷贝范围」）不含 `.claude/`，这一版第二段的基线因此是红的：
  - `/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/baseline.tsv:8`：「two-test-checker-tier exit-101 47.3 2026-09-30T21:55:29Z」
  - `/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/logs/baseline.two-test-checker-tier.log:62`：「登记表: Os { code: 2, kind: NotFound, message: "No such file or directory" }」
- 对 P1–P3：这份文件不在 crates/，不在 P1 的全局项里，覆盖也看不见「读了哪个文件」。把它放进导出树之后，它的内容一变，这条测试的结局就变，而 P1–P3 的关联集里没有它。
- 对 Q3 见 3.5。

### 2.5 `#[path]` 指回 harness 的 common：覆盖给的文件名带 `..`（量过，模型；实现层的坑）

- `research/prompts/tooling-broad-mutation-prune-r1-opus-model/run-output.txt:10` 里 llvm-cov 原样给的名字是「model-work/v1/tier/tests/../../harness/tests/common/mod.rs」，不是规范路径；名字也带建树时的绝对前缀。
- crates 里这样写的 checker 档测试文件，工作树里数出 29 个（`grep -lF '#[path = "../../singlefs-harness/tests/common/mod.rs"]' -r crates/singlefs-checker-tier/tests | wc -l`，2026-10-01）；仓里已经为同一件事写过归一：`crates/singlefs-checker-tier/src/crash_amplification.rs:339`「编译期 `file!()` 给的路径归一成仓内路径：去掉 `./`，把 `..` 与它前面那一段一起消掉」。
- 照名字直接去 manifest 里查，harness 的 common 查不到；查不到当「不是关联文件」，它改了也复用。这是实现的写法问题，不是 P2 的条款问题；P4 的样本里要有一条。

### 2.6 环境与驱动参数：一个文件都没改，结论翻面

- 指纹只有源码与工具链：`research/scripts/broad-mutation.py:45`「这一版的指纹 = sha256(源码指纹、换行、rustc -Vv 的输出)」。P1 的全局项（`research/prompts/_tooling-broad-mutation-prune-r1-body.md:44`）也只多了 feature 与 build.rs。
- 不在键里而改变结局的：超时倍数与地板（`research/scripts/broad-mutation.py:28`「单步超时 = max(地板, 倍数 × 这一版基线里这一步的秒数)」）、线程数（`research/scripts/broad-mutation.py:1194` 的 `-j threads`）、机器负载、驱动自己的步骤表、cargo-nextest 与 eatmydata 的版本。
- 这一版自己的产物里，同一种「第一段整段跑完」的耗时从 136.6 秒到 1101.8 秒，基线是 278.5 秒：
  - `/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/baseline.tsv:3`：「one-test ok 278.5 2026-09-30T15:16:40Z」
  - `/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/results.tsv:135` 第 250 条 136.6 秒；`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/results.tsv:268` 第 516 条 1101.8 秒，离限时 1392 秒差 290 秒。
  - 15 条只过第一段的整段耗时（命令 `awk -F'\t' '$4=="stage-one-only"{print $8}' results.tsv | sort -n`）：136.6 411.0 457.2 501.6 533.3 538.0 610.4 620.5 641.1 683.5 699.4 701.9 723.2 1095.6 1101.8。
- 第 556 条（`ReuseWindow::name` 换成 `""`）在库里是超时：`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/results.tsv:288`「one-test 超过 1392 秒」。我在副本上照驱动的命令重跑了第一段，结局见 2.6a。

### 2.6a 第 556 条：库里记「超时」，真结局是「抓到」（打中，真 crates 副本量过）

- 照驱动第一段的命令（`eatmydata cargo nextest run --locked` 四个包，`-j 12`）在副本上重跑两次，机器负载 30–38（别的会话在跑）：第一次到后台默认时限被停时 495 条过、0 条失败；第二次到约 2 小时的后台时限被停时 `research/prompts/tooling-broad-mutation-prune-r1-opus-model/crates-copy-evidence/mutation-556-stage-one-reruns.txt:2`「PASS 行数 687；FAIL 行数 0」，同文件第 13 行 `test_only_switches_force_every_branch` 里的一条在 472 秒处被停。两次都没跑完，不是结局。
- 读测试：`crates/singlefs-harness/tests/test_only_switches_force_every_branch.rs:112` 那一条断言比的正是 `ReuseWindow::name()` 报的分支名。单跑这一条（施加第 556 条之后）：`research/prompts/tooling-broad-mutation-prune-r1-opus-model/crates-copy-evidence/mutation-556-single-catching-test.txt:2`「FAIL [ 147.570s] (1/1)」，第 5 行「left: ""」。
- 所以：一个文件都没改，库里这一条的结论是「超时」（按 P5 红），真结局是「抓到」（不红）。抓它的那条测试在整段里排在后面、单条就要 148 秒以上，负载一高整段就过了 1392 秒。这是 2.6 那一格在真 crates 上的实例；也是 3.4 的误红。
- 两份输出的 sha256：`mutation-556-single-catching-test.txt` 9d59fae1aaf0ffcaa14f6a51199997107356dd91a00fc54424787a107b7b0bd1，`mutation-556-stage-one-reruns.txt` 20cf8d1787443396ea614506b3e855aaf882fd808619700fe34fdbef34b11cb7。

### 2.7 宏、泛型、`#[inline]`、闭包、derive：覆盖记在哪个文件（量过；只有宏与 derive 是盲区）

- 宏：`research/prompts/tooling-broad-mutation-prune-r1-opus-model/run-output.txt:5` 只有「files=core/src/macro_user.rs,harness/tests/macro_wrap.rs」，宏定义所在的 `macros_def.rs` 不在集合里：展开出来的代码记在使用处。crates 今天 `macro_rules!` 零处（`grep -rn 'macro_rules!' crates/singlefs-core/src crates/singlefs-checker/src crates/singlefs-format/src crates/singlefs-harness` 无输出），今天碰不到。
- 泛型（在测试 crate 里单态化）、`#[inline]` 跨 crate、闭包：`research/prompts/tooling-broad-mutation-prune-r1-opus-model/run-output.txt:4`「files=core/src/generic.rs,harness/tests/generic_inline_closure.rs」，记回定义它们的文件。没打中。
- derive 出来的实现不记（真 crates 副本量过）：`research/prompts/tooling-broad-mutation-prune-r1-opus-model/crates-copy-evidence/address-rs-coverage-one-test.txt:121`「  121|      1|        assert!(earlier_instance < later_instance);」执行了 derive 出来的 `PartialOrd`，而 `research/prompts/tooling-broad-mutation-prune-r1-opus-model/crates-copy-evidence/address-rs-coverage-one-test.txt:33`「   33|       |#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]」没有计数。一个只放带 derive 的类型的文件，改字段次序就改了 `Ord`，而它不进任何 T 的执行集合。没造出基线绿的第二版（改字段次序在 crates 里会动别的测试），算推的。

### 2.8 共享状态、执行次序、时间与随机数、panic 路径（没打中）

见第六节「没打中的形状」。

## 三、Q3：P5「有任何没红就红」会不会被别的原因误红

### 3.1 常量初始化式里的变异：覆盖没有这一行的区域，P2 不跑就记红，而它被抓到（打中，真 crates 副本量过）

- P2 原文：「一条测试都没覆盖到那一行的变异，直接记没红（即红，见 P5），不跑。」（`research/prompts/_tooling-broad-mutation-prune-r1-body.md:48`）
- 第 5 条变异 `address.rs:90:38: replace << with >>` 改的是 `crates/singlefs-core/src/address.rs:90`「pub const COUNTER_LIMIT: u64 = 1 << 48;」。副本上用插桩跑 core 的库测试，这一行没有计数：`research/prompts/tooling-broad-mutation-prune-r1-opus-model/crates-copy-evidence/address-rs-coverage-one-test.txt:90`「   90|       |    pub const COUNTER_LIMIT: u64 = 1 << 48;」。常量在编译期求值、内联进用的地方，没有运行时代码，任何测试都盖不到它。
- 而它被抓到：库里 `/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/results.tsv:8` 记「caught」；副本上施加后重跑：`research/prompts/tooling-broad-mutation-prune-r1-opus-model/crates-copy-evidence/mutation-5-run.txt:2`「test address::tests::journal_sequence_numbers_order_by_instance_then_counter ... FAILED」。
- 有多少条：清单里不在任何函数里的运算符变异（常量初始化式里的）170 条，占 5 091 条的 3.3%；这一版判过的 4 条里 3 条抓到、1 条只过第一段：`research/prompts/tooling-broad-mutation-prune-r1-opus-model/crates-copy-evidence/const-initializer-mutations.txt:1`「total 5091 operator-mutations-outside-any-fn 170」，第 2 行「outcomes {'caught': 3, 'none': 166, 'stage-one-only': 1}」。命令：`python3 research/prompts/tooling-broad-mutation-prune-r1-opus-model/outside_fn.py <副本> <库目录版本目录>`。
- 模型同形：`research/prompts/tooling-broad-mutation-prune-r1-opus-model/run-output.txt:29`「SCENARIO S7b v1+M failing=['harness/tests/const_values.rs::compile_time_values_are_pinned'] => caught」，第 30 行「tests-whose-coverage-includes-the-mutated-file=none」。

### 3.2 只在常量上下文里被叫到的 const fn：行有区域、计数 0；P2 记红，而它编不过（打中，真 crates 副本量过）

- `crates/singlefs-core/src/root_ring.rs:48`「const fn at_make_filesystem_checked_at_compile_time(slots_per_region: u64) -> Self {」只被 `AT_MAKE_FILESYSTEM` 这个常量叫。core 全部 132 条库测试插桩跑一遍，用到这个常量的测试行跑了（`research/prompts/tooling-broad-mutation-prune-r1-opus-model/crates-copy-evidence/root-ring-rs-coverage-all-core-lib-tests.txt:158`「  158|      1|            target_for_publish(CheckpointTxg(3), RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM)」），const fn 那一行是 0：`research/prompts/tooling-broad-mutation-prune-r1-opus-model/crates-copy-evidence/root-ring-rs-coverage-all-core-lib-tests.txt:48`「   48|      0|    const fn at_make_filesystem_checked_at_compile_time(slots_per_region: u64) -> Self {」。
- 清单第 2602 条把它换成 `Default::default()`。副本上编：`research/prompts/tooling-broad-mutation-prune-r1-opus-model/crates-copy-evidence/mutation-2602-build.txt:2`「error[E0277]: the trait bound `root_ring::RootRingSlotsPerRegion: Default` is not satisfied」——真结局是无效（不红），P2 的次序（先看覆盖、零覆盖不跑）会把它记成红。P2 的另一句「无效（编不过）的每次重判」（`research/prompts/_tooling-broad-mutation-prune-r1-body.md:49`）要先编才知道是无效，两句的先后正文没定。
- 模型同形（会抓到的那一种）：`research/prompts/tooling-broad-mutation-prune-r1-opus-model/run-output.txt:27` S7 抓到，第 28 行覆盖里没有被改文件。
- 覆盖分得出 3.1 的「没有区域」与真正的「有区域、计数 0」，分不出 3.2：编译期叫的 const fn 与运行时没走到的死代码同样是 0。

### 3.3 确定的死循环记「超时」，P5 记红，而且永远红（打中，真 crates 副本量过）

- 第 102 条 `admission.rs:890:40: replace - with /`：`crates/singlefs-core/src/admission.rs:885`「while splits_at_the_level_below > 0 {」的循环里，`(splits_at_the_level_below - 1) / internal_additions_per_split_of_a_fresh_node`（同文件第 890 行）换成 `/ 1` 之后，每轮不再减一。
- 库里记超时：`/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/results.tsv:61`「one-test 超过 1392 秒」；那份日志里 774 条过、2 条被 SIGTERM 停在半路（`logs/102.one-test.log`，没整行引）。
- 副本上单跑那条被停的 core 测试：没改过的 0.00 秒（`research/prompts/tooling-broad-mutation-prune-r1-opus-model/crates-copy-evidence/mutation-102-single-test.txt:11`「test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 131 filtered out; finished in 0.00s」），施加变异后 300 秒没跑完（同文件第 5 行「超过限时 300 秒」；退出码 253 是包装的结局，不是这条测试的结局）。
- 这条变异改了行为、测试也看得出来（测试不结束），按 P5 却是红；它不是等价变异，P5 给的出路（写一条等价性测试再排除）用不上。每次提交都红，或者每次都要重付 1392 秒以上去重判。
- 驱动的认失败那一行已经认 nextest 的 `TIMEOUT`：`research/scripts/broad-mutation.py:188` 的 `NEXTEST_FAILURE_LINE` 含「TIMEOUT」。给第一段配上按每条测试的基线时长算的单条终止时限，这一类会落成「抓到」（推的，没跑）。

### 3.4 负载造成的超时（2.6 的另一面）

- 7 条超时里，第 556 条在库里那一趟到 1392 秒时 706/776 跑完、一条失败都没有（`logs/556.one-test.log` 末尾，没整行引）；我在副本上的重跑见 2.6a。
- 超时阈值是「倍数 × 这一版基线」，基线本身在当时的负载下量（`research/scripts/broad-mutation.py:28`）；同一版里第一段整段跑完的耗时散到 136.6–1101.8 秒（2.6）。复用之后这一版的超时行留在结果表里，按 P5 一直红，直到被改文件变。

### 3.5 「只过了第一段」的条，这一版里没有一条能补完：第二段的基线本身是红的（打中，库目录产物）

- 2.4 那条测试读不到导出树外的文件，第二段 checker 档快档的基线退 101。驱动的规矩是基线红就不跑（`research/scripts/broad-mutation.py:73`「基线：每一版每一步先在没改过的树上跑一遍，全绿才开始跑变异（基线红了一律退 6）」）。
- 于是这一版 15 条「只过了第一段」都补不成定论，按 P5 每一条都是红；红的原因是基线，不是这 15 条变异。P5 要把「基线红」单列成另一种红，不摊到变异上。

### 3.6 文档测试的覆盖在稳定版上量不了（今天碰不到）

- `research/prompts/tooling-broad-mutation-prune-r1-opus-model/crates-copy-evidence/doctest-coverage-on-stable.txt:3`「error: the option `Z` is only accepted on the nightly compiler」，第 4 行「error: the option `persist-doctests` is only accepted on the nightly compiler」。
- 今天 core 与 checker 的文档测试只有 `crates/singlefs-core/src/address.rs` 开头那一条 `compile_fail`，不执行任何行，不会因此误红；以后加一条会运行的文档测试，它盖到的行在覆盖里是 0。

### 3.7 覆盖那一遍自己会弄脏专用树（实现层，量过，模型第一次跑）

- 插桩编出来的测试二进制只要被起（连 `--list` 也算）而没设 `LLVM_PROFILE_FILE`，就在当前目录写 `default_*.profraw`。我的模型第一次跑就这样把 12 个 profraw 写进了树（`sed -n 11p … | grep -o 'default_[0-9_]*\.profraw' | sort -u | wc -l`）：`research/prompts/tooling-broad-mutation-prune-r1-opus-model/run-output-first-attempt-profraw-left-in-tree.txt:11` 的 changed 列是一串「default_」开头的文件。
- 驱动核树时多出文件就拒：`research/scripts/broad-mutation.py:70`「树里多出没登记的文件，拒绝跑（退 5）」。覆盖那一遍在专用树上跑，就要给每一次起二进制都设 `LLVM_PROFILE_FILE`。

## 四、打中之后的四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md:126`「打中之后先问四句」）

| 打中 | 分不分辨臂 | 被判的系统当时看不看得到判别它的东西 | 满足判据字面的哪一句 | 跑前条款的改法在这几格上 |
|---|---|---|---|---|
| 2.1、2.2 常量 | 分辨：P3 只看被改文件那一跳的写法中，「常量引用按闭包收」「只放常量的文件一变就整版重判」两种写法不中（推的） | 看得到：常量引用是源码里的静态事实，两版的文件 sha256 也都在；覆盖看不到 | 正文第六节第一条「Q2 造出一条「P1–P3 判可复用、而在这一版上重跑结论翻面」的反例」；但反例在模型上，不在「`crates/` 今天的代码」上 | 「改规则，并把这个反例做进 P4」：P4 已列的「常量定义文件变了」若只造一跳的样本，S1、S2 照样漏 |
| 2.3 执行集合 | 分辨读法 A、读法 B、并集：A 被 S4a 中，B 被 S4b 中，并集两格都不中（按输出的集合算） | 看得到：两版各跑一遍覆盖（P2 本来就每版跑），两份都留着就能取并集 | 同上一句，模型上 | 只改成读法 B（「新版的覆盖」）在 S4b 上照样中 |
| 2.4、3.5 导出树外的数据文件 | 不分辨复用的各条臂：不管复不复用，这一版第二段基线都红；病根在导出范围 | 覆盖看不到「读了哪个文件」；静态看得到（`root.join(".claude/…")`） | Q3 一句「Q3 找到误红的来源 ⇒ 写进 P5 的判法」；P3 已列「测试运行时读的非 `.rs` 文件」这一类，这里补的是「今天就有，而且在导出树外」 | P3 倾向的「并进关联文件」做不到：文件不在树里，哈希不到；要先改导出范围或这条测试 |
| 2.6、3.3、3.4 环境与超时 | 不分辨：凡是只按文件定键的设计一起中；另立一笔账 | 看得到一部分：nextest 给每条测试的时长，基线里有每条测试的时长就分得出「挂住」与「慢」 | Q2 那一句满足（2.6a：零文件改动，超时翻成抓到）；Q3 那一句（超时的阈值）满足 | 正文没给改法；P5「超时就红」在 3.3 上把一条被看出来的变异永远记红 |
| 3.1 常量初始化式 | 分辨：「零覆盖不跑就记红」中，「零覆盖照样跑」不中 | 看得到：llvm-cov 分得出「没有区域」与「计数 0」 | Q3 那一句 | 「写进 P5 的判法」：P5 本身不动也行，动的是 P2 的那一句 |
| 3.2 编译期 const fn | 分辨：同上 | 看不到：计数 0 与运行时没走到的死代码相同（判别子观测不到），只能编、跑 | Q3 那一句 | 同上；只靠「没有区域」那一条判别救不了这一格 |

## 五、几个改法各修哪一格（都只在我的模型上量过或没量，被攻过零轮）

| 改法 | 修哪几格 | 量过 / 推的 |
|---|---|---|
| P3 的常量规则改成：被改文件、T 的测试文件、T 执行到的每个文件引用到的常量，沿常量引用链收闭包 | 2.1、2.2 | 推的 |
| 只放常量的文件（今天是 `crates/singlefs-format/src/lib.rs`）一变就整版重判 | 2.1、2.2 在 crates 上的同形 | 推的 |
| 执行集合取两版覆盖的并集 | 2.3 的 S4a、S4b | 按 `run-output.txt` 第 21、24 行的集合算出，没另跑判定 |
| 关联集改用静态引用闭包（`crates/singlefs-checker-tier/src/crash_identity.rs:7`「顺着 `use` 与路径引用只收用到的模块，外加给这些类型写 `impl` 的同 crate 文件」） | 2.1、2.2、2.3、宏 | 推的；2.3 要靠它收「写 `impl` 的文件」 |
| 零覆盖的变异照样先编、再跑，不直接记红 | 3.1、3.2 | 两格的真结局量过（抓到 / 无效），改法本身没实现 |
| 超时不复用；第一段配按条的终止时限，挂住的落「抓到」 | 3.3、3.4、2.6 的一部分 | 推的；负载照样进不了键 |
| 导出树收进测试运行时读的文件并进 P1 的键；P5 把「基线红」单列 | 2.4、3.5 | 推的 |
| 覆盖给的文件名先归一（去绝对前缀、消 `..`）；对不上 manifest 的名字一律当「变了」 | 2.5 | 推的；模型里归一之后集合对了（`run-output.txt` 第 10 行 files 列） |
| 覆盖那一遍每次起插桩二进制都设 `LLVM_PROFILE_FILE` | 3.7 | 量过（模型）：改后那一次 changed 列只剩被换的文件（`run-output.txt` 第 12 行） |
| 键里加驱动参数：步骤表、超时倍数与地板、线程数、nextest 与 eatmydata 版本 | 2.6 的一部分 | 推的 |

## 六、没打中的形状（各只取样一次，不支撑「没有」：`.claude/rules/three-way-inference.md:159`「**一次不算**」）

| 形状 | 试了什么、取样多大 | 为什么没打中 |
|---|---|---|
| 测试之间的共享状态、执行次序 | 在 core、checker 的 src 与 harness 的 src、tests 里搜 `static`、`OnceLock`、`thread_local!`、`Atomic*`、`LazyLock` | 只有纯函数的查表缓存（`crates/singlefs-core/src/checksum.rs:11`、`crates/singlefs-checker/src/lib.rs:58`）、带进程号的临时镜像名计数器、panic 钩子（线程局部开关）；第一段是 nextest，一条测试一个进程。第二段的 `cargo test`（同进程多线程）只跑文档测试与 checker 档快档，checker 档没逐个看 |
| 时间与随机数 | 搜 `Instant::now`、`SystemTime`、`rand`、`thread_rng`、`RandomState`、`HashMap` | harness 里 `Instant::now` 只用来打印用时；随机历史的种子基是常量 `crates/singlefs-harness/src/history.rs:89`，那份文件被随机历史测试执行，本来就在关联集里；core 与 checker 的 src 里没有 `HashMap` / `HashSet` |
| panic 路径与 `should_panic` | 搜 `should_panic`（2 处）、`process::exit`、`process::abort`、`Command::new`、`CARGO_BIN_EXE` | 后四样零处：测试进程都正常退出，profraw 照写；panic 前走过的行照记 |
| 泛型单态化、`#[inline]` 跨 crate、闭包 | 模型 S8 | 覆盖记回定义它们的文件（2.7） |
| 排除表 `research/scripts/broad-mutation-ignored-exclusions.tsv` 在指纹外 | 读驱动的登记判法 | 它的内容被测试文件钉死：`research/scripts/broad-mutation.py:65` 起把「没登记」「排除表指着树里没有的用例」「两边都在」都判红，排除表只能跟着测试文件里的 `#[ignore]` 理由变 |
| litmus 文件（测试运行时读，在导出树、在指纹里） | 读 `crates/singlefs-harness/tests/publish_order_matches_litmus.rs` | 那条测试判相等：只改 litmus 不改代码，基线就红，造不出基线绿的翻面；没在副本上试 |
| 宏 | 模型 S5 量过盲区 | crates 今天零处 `macro_rules!` |
| 文档测试 | 副本上试插桩 | 今天只有一条 `compile_fail`（3.6） |

## 七、这条腿自己的限度

- 2.1–2.3 的翻面只在模型上造出、真跑；真 crates 上没造出「基线绿、只改一个非关联文件、结论翻面」的第二版：格式常量多处被钉死，改一个就动基线；方法遮蔽要写新代码。真 crates 上满足 2.1 前提的被改文件是不引用 singlefs-format 的 5 个 core 文件（`block_device.rs`、`bytes.rs`、`mounted_session.rs`、`write_accounting.rs`、`write_request_split.rs`，清单里 12+17+18+42+62 = 151 条）；别的被改文件直接引用 format，而 format 是一个文件，P3 一跳就把它整份并进来。
- P3 那一列（被改文件引用到的常量的定义文件）是我读被改文件手填的，不是工具算的。
- 真 crates 的覆盖只量了 core 的 132 条库测试（一条单独量、全部合起来量各一次），没量 harness、checker 与 checker 档；3.1 的「任何测试都盖不到」靠的是「常量初始化式没有区域」这一条性质，不是全量覆盖。
- 第 556 条只在副本上重跑一次，机器当时负载 30 上下（别的会话在编 RocksDB、跑 checker 档）。
- checker 档一条没跑（重型），2.4、3.5 全凭库目录里那一版的基线日志。
- 模型每个场景只跑一次（代码确定，没有随机数）。
- 没有起 cargo-mutants，变异按清单的 `start`、`end`、`replacement` 手工写进副本（`apply.py` 照驱动 `replace_span` 的写法）。

## 八、收尾

- 删了（草稿目录 `/tmp/claude-1000/tooling-broad-mutation-prune-r1-opus/` 下）：`target-plain` 12G、`target-cov` 282M、`repo`（HEAD 导出的副本）11M、`model-work`（模型两版树与编译目录）1.4G、`tools`（llvm-tools 1.98.0）236M。日志与草稿留在草稿目录。
- 运行时看到的负载：开工时有别的会话的 `cargo test`（16 线程）在跑；第 556 条两次重跑期间负载 30–38，有别的会话在编 RocksDB、跑 checker 档。没有等锁。
- 没做：没跑 checker 档、没起广谱驱动与 cargo-mutants；没量 harness 与 checker 的覆盖；没在真 crates 上造 2.1–2.3 的第二版；没判正推、辩方那几格。
- `python3 research/scripts/cite-check.py research/prompts/tooling-broad-mutation-prune-r1-opus-output.md` 末行：「✓ 核了 43 处引文，对上 43 处，没判 0 处（逐处列在上面）」（加这一行之前跑的）。
