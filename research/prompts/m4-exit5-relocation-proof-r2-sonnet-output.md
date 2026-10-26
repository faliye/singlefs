# m4-exit5-relocation-proof-r2　云端正推腿报告（Sonnet）

轮：m4-exit5-relocation-proof-r2。立场：云端正推，格 Y1–Y6，层按用户 2026-10-26 定的「core 直验，连胶水一起证」。
冻结副本 `~/.local/share/singlefs/work/m4-close-rounds-frozen/`（只读；下面引的 `crates/` 行号都是它的，引主工作区的另注）。`mount.rs`、`mounted_session.rs`、`compaction_plan.rs` 三份主工作区与冻结副本不同：
mount.rs 差 34 行、全在 GPU 计算准入那一段，回退那三个函数在主工作区整体后移 4 行（`diff` 现比，`grep -n` 现取：冻结副本 9509 / 9628 / 9665，主工作区 9513 / 9632 / 9669）。
所有数都在草稿目录的副本上量，不是入库装置上的数，判决要引的得照 `.claude/rules/three-way-inference.md` 在入库装置上重做。

## 复跑

模型目录 `research/prompts/m4-exit5-relocation-proof-r2-sonnet-model/`（`tools/rerun.sh` 照 `scripts/agents/three-way-rerun-template.sh`，只改了 `rerun_steps` 那一段）。整份复跑：

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-exit5-relocation-proof-r2-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --work <新的草稿目录>
```

整份连跑约 12 分钟（本机，此刻负载 30 上下，下面四段合计 730 秒，队列不算）。前台单条命令上限 240 秒，我没法在一次前台调用里整份跑完，所以**整份连跑没做**，改成分四段各跑一次（`RERUN_STEPS=<段名>`，每段在新的工作目录里；每段的末行都是 `name=rerun_result mismatches=0`，退出码 0）：

| 段 | 命令前缀 | 内容 | 秒 |
|---|---|---|---|
| A | `RERUN_STEPS=setup,y1,y1p` | Y1：五个变体的产品构建、Verus、MIR 比对；七个单文件探针 | 193 |
| B | `RERUN_STEPS=setup,y2` | Y2：MIR 调用抽取、std 规格按名字配、四批共 96 个探针、登记式 assume_specification 演示 | 171 |
| C | `RERUN_STEPS=setup,y3` | Y3：两个闭包规模与分类、接上第一阶段原型之后的闭包、七个必红世界 | 219 |
| D | `RERUN_STEPS=setup,y4,y5` | Y4：编译目录缓存与逐函数 JSON；Y5：新协议与区间版 P4、十二个必红世界 | 147 |

每一步的输出去掉路径与耗时后与 `out/` 下的参照逐行比，对不上的件数就是退出码。参照是我第一趟跑出来写下的（`RERUN_WRITE_REFERENCE=1`）；中途改了探针、分类脚本、`step_y2` / `step_y3` / `step_y4`，相应段的参照重写过，然后四段**全部**在新目录里又各比了一次（上表的秒数就是这最后一趟的），全同。
`setup` 段是每段都带的前置（建工作区变体）；`y1p`（单文件探针）从 `y1` 拆出来，是因为整段在负载高时超过前台上限。

要主 agent 在后台整份连跑的命令放在文末「要主 agent 跑的命令」。

模型目录里每个文件的 sha256（`find . -type f | sort | xargs sha256sum`，相对模型目录）：

```
fb7aeaaad197c459f5b50375b8991d0158eda6df7a70dcf0ddc3f5f8f0924c8a  assume-demo/Cargo.toml
06ad76a6282e3fec7ba40b793908da12e5186a3957f34910f6bd852bc4926453  assume-demo/registry.tsv
d4c0902758f1b0480424db5f423ee12d629a543f1c11e3019620dd950438bbf8  assume-demo/src/lib.rs
d604f2f51c4a47cd3be90ceaa03ecb5800eb301217298a3b7aaf6e9d0a86790a  out/first-attempts/y1-full-workspace-cargo-verus.txt
206fc1de2b7a19fda3f19e558cd0eb9c4836fb40df48e6655915ebfa8f1b1b8a  out/first-attempts/y1-m3-verify-2.txt
65587ee9ff590af2a970d72945a238c0fc87fd209b35fe01c1da105527a67cf2  out/first-attempts/y1-m3-verify-3.txt
c79f8a3ea4ea512ceb13918442b35f2829e63c6624954b5c5ae2632eed3ab839  out/first-attempts/y1-m3-verify.txt
e525e5323191e1878be0883cbf6c571a57d3f7b154d5ef36b4c92f7e6a7150b3  out/y1-probes.txt
8e45dfca447efe7529e3c9eb349bd384b19ef69560c99b7f6da86d39203b7efd  out/y1.txt
14721618eb635d90af7e0d0b593e2a2406aaae24d8ed7630c0b28dd975e1f5ba  out/y2-assume.txt
dc23d84fcf165c9d22c4a7850dd1f2c8884cab763433388dc6c4e4bdbb98b282  out/y2-mir.txt
bb95be941d9599560e14602dc67dd94b26da5a14b840e19a3481281173ecd5f2  out/y2-probes.txt
0f62e28f549c00f5eec3adf9aad012de5f012334734f60c7dd72e2a52ecdba20  out/y3-closures.txt
596dc26cafc612576b6e7fd2b2f3593aa0bbdc246a9dcf88f0e8ec1bf8644bc5  out/y3-red-worlds.txt
c786574b19cd17a6629de1fb49e22c30465aa8a82afdbbadf25556eefa2e65f9  out/y4.txt
74bef6f18bbb63d7fbc0378e039f3a7221ec2eb2538eb6306329b6ac741cf735  out/y5.txt
5d926173036cb3b23552e589ca30923cc8bae9b032df6530f8d02c6f64a3bc35  patches/y1-m1.patch
b24206ac0fa6d0c362b6c0b8cd069efaa04a246ce4e9a621644ca2465adb33a2  patches/y1-m2.patch
a7c4ee9efe1fbbb862810600ef26aad6b4da759d7504e8d556e3d27f9ae0a054  patches/y3-m4.patch
9cce92bd0b02516dc5ecc658867aa863f480907c6590af88b0f3bd2217db8bbe  patches/y3-s1.patch
5669bb7cd2c7c3a65d0da283ea23e12b7c54d5336a8631b6a3ea7b68970051c1  probes/y1_a_external_type_in_verus_fn.rs
300b4a1e1e2fb9cd133e005715c0097ddfc702afa55c3748fc37940b0c098933  probes/y1_b_attribute_mode.rs
27d54a1c9f35d888a441d50cffc3a6cd970a66f1df3ea1f00717c2888035ca34  probes/y1_c_external_type_specification.rs
d6a5fe61742980e2f8e8c008a425113fb34d848086a3f3a9ce24eb067fc4a6cd  probes/y1_d_assume_specification.rs
27eaebb3d469eacb8413a0033c78c1be21eaa610a76169352bdc2fc64f248d62  probes/y1_e_external_body.rs
4e5cb429615d596c1470a4506ca56c32bcd2ea836a578c78959226519f7a4cd0  probes/y1_f_unverified_callers_are_not_checked.rs
cdf33e105d5c1c9888ea73ee45d01eb6d3b5430f55191fb6d400b8e9c9381ae4  probes/y1_g_assume_specification_signature_must_match_std.rs
2f98a71fea6635eace88345465b1708e1b619bbb6177bb4c08a6a4a9f08912b3  probes/y2_constructs_a.rs
4a717705bc335d3daa7375aec680f279d1d80ca4c4d28709da76c7e839f5dd68  probes/y2_constructs_b.rs
ed69703102b727eab5bc1d06e9c5d2f4f93851845725b03bce028f67ed7b985f  probes/y2_constructs_d.rs
6b2f02157a301f3214f723e0af07b0888600c8f879c671a1e473ae82b6802437  probes/y2_constructs_e.rs
f75092aaa6ace96083cc6c0d9a380d6d20b7574af369c4f1f28604915b67988e  probes/y2c-split/s07_match_enum.rs
dd11f97067351a0c89b7cfcbb60cb85aea4046c93f40e2664538f3ee8a381ae0  probes/y2c-split/s11_generic_device.rs
b2705c1b9ba70232341b4345097980ac33c22d5f2a7ae470ad60a375c77b27c8  probes/y2c-split/s12_dyn_trait.rs
47bee25e0e1b7346384cb2ae0cb817b50a274a3a49412fd157dfa7afa5414433  probes/y2c-split/s13_impl_for_real_device.rs
09a4f07f08a752233e4542c629327efcdaf6c2d7fa57358073c640f459ea9511  src/gc_ring_proofs.rs
d7a3a32662a63f8c68f0ccacbb9d20a8722bfb283a399dd43492c43251ff8e4d  src/relocation_interval_proofs.rs
881fcb737bf30f83498ab125586b9e233a9ede2ea1a2a135866a46f696e7e410  src/relocation_judgement.rs
0c3f886ae377af6b23c1739f62256315ac08d4cf168204f24a3a9bfcfc55ff2d  tools/assume-registry-check.py
de4db2943160ee6197f986c21f8acf5c7b149fcd77f1b435fd731e219305fa8d  tools/closure_classes.py
9c515a32c402ae19b6d1b8c64087f7154dd4d2444aeabaa08e1b19e961f30a57  tools/closures.sh
54422dd63a6d9f5b178035d9c64b820780e94c37613fc7d083793b48326ef9f9  tools/closure_stats.py
a14b3cdcce370dc5f2f0948dd23137f8ae1f7b3b93c82fbc3585082dfe81612f  tools/mir_callees.py
ab9d6d107c2664ec00e4d85c2e3350c93b7311f079ece65ff5e10c1d77088ffe  tools/probe-split.py
149113770ecfc5beabf8468f01def847c7da68ee7b8dc74a7963239ae74608ef  tools/red-worlds-core-m4.tsv
520324e68844028b9ea4ba65debafb04410ec08fa9606de36b86cc8b194b3989  tools/red-worlds-core.py
5a2b44e6bf2fb088cc6a872d164a8421e569e8a4c34eab5e3c30c77a233d6857  tools/red-worlds-core-s1.tsv
e9c103ab394f6f9d7775990216e70a475bb80ffd9087b690aabe46b5dcb11f29  tools/red-worlds.py
fe027800912fb7a53fc1b3ee08d16423e507565a36c4a2e9046bf9f4093af11d  tools/red-worlds-y5.tsv
e21bf246f83d86cf7acb9ea2c56da0749f7c2a294218c9c33be68c4919f1f959  tools/rerun.sh
5926269b5cc41a724ec180545faa9ea65bfba595492ccd05de9a86cb851be04f  tools/std_coverage.py
b8fdebffa9a74acf7ff65204b22ea7ad617d9f09b818adff9a9b8a10b051999f  tools/workspace-Cargo.toml
```

## 〇　结论摘要（正推腿的推荐，不是判决）

- **Y1**：`verus!` 宏包类型与被验函数是主做法，属性模式 `#[verus_verify]` / `#[verus_spec]` 备用于「函数体源码要保持纯 Rust 写法」的几处。两种都不改产品构建：最小工作区里 5557 个 MIR 函数，原版对两种接法逐行比，规范化后 0 行不同（未规范化有 3338 行，全是路径印法与行号）。vstd 本来就在 core 的产品依赖闭包里（经 `singlefs-replay-prefix`）。真正的代价不在接法，在**类型波及**与**私有字段不变量**：包一个 9 行的函数要连搬 4 个类型。
- **Y2**：闭包里 456 个不同的 std 调用键，按名字配得上 vstd 规格的 224 个（49%），调用点 4628 处里落在配上的键上的 3813 处（82%）；96 个探针里 27 个撞上「std 函数没有规格」（Verus 原话 `is not supported`），它们正是按名字没配上的高频键；按名字配上的键（`Vec::push`、`Option::expect`、`ok_or`、`checked_add`、`try_from`、`BTreeMap::get` 等）的探针没有一个因为「没有规格」失败。`assume_specification` 与 `external_body` 在 `--no-cheating` 下一律被 Verus 自己拒（不只是门禁的正则）。登记式（规格 + 差分测试 + 必红世界）在演示 crate 上判得出「写错的规格」：Verus 接受错规格和一条只在错规格下才证得出的假命题（2 verified, 0 errors），差分测试红。
- **Y3**：两个闭包并起来 1073 个 fn 单元、24970 行。按「直接要什么才进得了 Verus」分类：A 直接可验 472、B 只差 panic/format 机器 190、C 要改写 291（行数占 41%）、D 信任边界 120（设备读写、校验和）。第一阶段原型在 core 里直接验过：13 + 13 条 verified，七个必红世界全红。总代价是推的：被验版本约 11 万–12.5 万行（只验 A+B 约 3.9 万–4.3 万行），比原来多约 9 万行（A+B 多约 3.1 万–3.5 万行），登记数从 88 涨到约 2100–2800（全验；只验 A+B 两类约 1300–1700），机器时间不是瓶颈（88 条整 crate 3.4 秒）。
- **Y4**：`verus-proof-check.py` 与登记表要改十一条（最硬的：crate 只认一行、依赖只许 vstd 所以要换 `cargo verus focus`、作弊标记一概禁所以要换成信任清单、`verus_functions` 不收 impl 里的方法与跨文件同名、必红世界逐函数一行、cargo 缓存不看 `--verify-*`），`verification.md` 两句要改，D13 已定项 14 加一行。
- **Y5**：协议补了（持久回退视图、多逻辑版本、根环窗口与 F）。新协议 13 条 verified、区间版 P4 9 条 verified，并入 `singlefs-replay-prefix` 副本整 crate 88 → 110，十二个必红世界全红。第一轮攻方的三个反例都被排除。G7 在带整理的历史上**不是恒等**：整理的 checkpoint 把回退候选窗口移一格（窗口满时丢最旧的一个），这一条被证成引理而不是被回避。
- **Y6**：罩不到的清单十五项，每项写了由什么罩。

## Y1　怎么接

### 1.1　三种接法各在副本上编过 core 今天的一个小函数

被试函数：`holes_admit_the_issue`（冻结副本 `crates/singlefs-core/src/compaction_plan.rs:942`，三行的 `const fn`）；更大的被试函数 `region_is_empty_by_the_records`（`compaction_plan.rs:948`，闭包加 `iter().any`）。
最小工作区 = 冻结副本里只留 `singlefs-format`、`singlefs-core`、`singlefs-replay-prefix` 三个包（`tools/workspace-Cargo.toml`）：整个工作区的 `cargo verus` 在离线机器上过不了，`cargo metadata` 要下载别的平台的包（原样：`Error: `cargo metadata` exited with an error: error: failed to download `android_system_properties v0.1.6``，模型目录 `out/first-attempts/y1-full-workspace-cargo-verus.txt`），所以要么门禁联网要么像现在的 `copy_crate`（`scripts/gate/verus-proof-check.py:684`）那样拷出核与它的依赖。

| 接法 | 改动（补丁在模型目录 `patches/`） | 产品构建 `cargo build -p singlefs-core` | `cargo verus focus -p singlefs-core` | MIR |
|---|---|---|---|---|
| 宏包一个函数（`verusbang`，`y1-m1.patch`） | core 的 `Cargo.toml` 加 `vstd` 一行、`[lints.rust] unexpected_cfgs`、`[package.metadata.verus]`；`compaction_plan.rs` 把函数放进 `verus!` 加一句 `ensures` | rc=0，0 警告 | `verification results:: 1 verified, 0 errors` | 5557 个 MIR 函数，规范化后与原版差 0 行 |
| 属性模式（`attribute`，`y1-m2.patch`） | 同上依赖三处；函数体原样，只加 `#[verus_verify]` 与 `#[verus_spec(admit => ensures ...)]` | rc=0，0 警告 | `1 verified, 0 errors` | 差 0 行 |
| 宏包类型与函数、在 core 里直验（`inplace`，`y3-m4.patch`） | 4 个类型进 `verus!`、`CompactionRegion` 加类型不变量、构造函数与 `region_is_empty_by_the_records` 进 `verus!`（循环改写）；补丁加 78 行删 17 行 | rc=0，0 警告 | `13 verified, 0 errors` | 没比（改了函数体） |
| 新模块、在 core 里直验（`kernelmodule`，`y3-s1.patch`） | 新文件 `relocation_judgement.rs`（175 行）+ `Placement`、`SlotNumber` 两个类型进 `verus!` + `transaction.rs` 里两处调用；补丁加 179 行删 5 行 | rc=0，0 警告 | `13 verified, 0 errors` | 没比 |
| 只在 `cfg(verus_keep_ghost)` 下展开的规格 | 没单做 | — | — | — |

最后一行：`verus!` 与 `verus_spec` 在普通 rustc 下**本来就**擦掉 ghost 部分（`.claude/gate.d/verus-pins.tsv:16` 写着「编译用 crates.io 这六个包擦 verus!、验证用这份 Verus 自带的宏与 vstd，两边是同一份源码」），手写 cfg 展开规格没有额外好处，只多一条「Verus 与 rustc 看到不同代码」的口子，所以没做。

`out/y1.txt` 原样（模型目录；末两行是 MIR）：

```
name=y1_build variant=base rc=0 warnings=0
name=y1_build variant=verusbang rc=0 warnings=0
name=y1_verify variant=verusbang rc=0 verification results:: 1 verified, 0 errors|
name=y1_build variant=attribute rc=0 warnings=0
name=y1_verify variant=attribute rc=0 verification results:: 1 verified, 0 errors|
name=y1_build variant=inplace rc=0 warnings=0
name=y1_verify variant=inplace rc=0 verification results:: 13 verified, 0 errors|
name=y1_build variant=kernelmodule rc=0 warnings=0
name=y1_verify variant=kernelmodule rc=0 verification results:: 13 verified, 0 errors|
name=y1_mir_fn_count base=5557 verusbang=5557 attribute=5557
name=y1_mir_diff_lines verusbang=0 attribute=0
```

MIR 比法：`cargo rustc -p singlefs-core --lib -- --emit=mir`（稳定的 `--emit=mir`），原版与接法各出一份全 crate 的 MIR，各 361620 行；规范化 = 去掉 `std::` 前缀与源码行列号（`rerun.sh` 的 `mir_normalized`）。不规范化有 3338 行不同：vstd 的 `Map` 进了作用域，原版里 `Map<…>` 印成 `std::iter::Map<…>`，多出的 `use` 行让 impl 的行号后移——全是印法（`out/y1-mir-diff-m1.txt` 在草稿里是规范化之后的空 diff，规范化之前的行数 `diff … | wc -l` 得 3338 我在第一趟量过、没存成参照）。
被验的 `holes_admit_the_issue` 两个阶段的 MIR 逐字节相同（`awk '/^fn holes_admit_the_issue/,/^}/'` 的 md5 都是 `e932e10e864f66b8df34c09cfaa58b38`，草稿里量的）。

### 1.2　对产品构建（不跑 Verus 的 `cargo build`）的影响

- vstd 本来就在 core 的产品依赖闭包里：`cargo tree --offline -p singlefs-core -i vstd` 原样输出
  ```
  vstd v0.0.0-2026-09-20-0158
  └── singlefs-replay-prefix v0.1.0 (…/tree-b/crates/singlefs-replay-prefix)
      └── singlefs-core v0.1.0 (…/tree-b/crates/singlefs-core)
  ```
  （冻结副本 `Cargo.toml` 的 `vstd` 钉死版本在 `crates/singlefs-replay-prefix/Cargo.toml`，`Cargo.lock:1024` 是 vstd 的包条目。）core 直接依赖 vstd：`Cargo.toml` 多 3 处，`Cargo.lock` 的 core 条目多 `"vstd"` 一行（`diff tree-b/Cargo.lock tree-m1/Cargo.lock` 原样：`63a64 >  "vstd",`）。
- 产品构建时间：最小工作区冷编译加上 vstd 一次约 12 秒（`out/y1-m1-build.txt`，草稿）；vstd 今天就在产品构建里编，所以新增的是 core 里 `verus!` 展开的宏时间，没量。
- 产品代码形状：`verus!` 块里的 doc 注释写在 `verus! {` 之前会触发 `unused_doc_comments` 警告（`verus! {` 要放在文档注释之上）；只在 ghost 里用的 `use` 会触发 `unused_imports`（用全路径或把 `use` 放进 ghost 用的函数里）；原先的 `const` 被字面量替换后原常量变成死代码（`SIX_BYTE_SLOT_NUMBER_LIMIT never used`）。三样我都碰到了、前两样修了，第三样的修法是在 `verus!` 里声明同值常量并在块外 `const _: () = assert!(…== singlefs_format::…)`（没试）。
- core 的 `#![deny(unsafe_code)]`（冻结副本 `crates/singlefs-core/src/lib.rs:10`）与 `verus!` 展开不冲突（产品构建 0 警告 0 错）。

### 1.3　「Verus 与 rustc 看到的不是同一份代码」的口子

按源码看：`verus!` 块与属性模式都是同一份 `.rs`，rustc 擦掉 ghost，MIR 一致（上表）。真正的口子有三个，探针 `y1_f_unverified_callers_are_not_checked.rs` 量了后两个：

1. **verus! 外面的代码 Verus 不看**。同一个文件里，块外的函数可以 `Region { start: u64::MAX }` 绕过类型不变量构造值，也可以不满足 `requires` 就调被验函数；`verus --no-cheating` 说 `verification results:: 3 verified, 0 errors`（`out/y1-probes.txt` 里 `y1_f_…` 两行）。所以带类型不变量的类型，**所有构造点都得在被验区域里**；带 `requires` 的被验函数，**调用点也得在被验区域里**——门禁要 grep 判这两条（见 Y4 的「被验区域外的调用点」一条）。
2. **类型波及**：`verus!` 里的函数用到的每个类型和方法都得在 Verus 的视野里。把 `region_is_empty_by_the_records` 一个函数放进 `verus!`，第一趟报 6 个错（`out/first-attempts/y1-m3-verify.txt`）：`AllocationRecord`、`CompactionRegion`、`DeviceIdentity`、`SlotNumber` 四个类型和 `CompactionRegion::device`、`contains_slot` 两个方法「declared outside the verus! macro」。逐个搬进去后又冒出 `CheckpointTxg`（`y1-m3-verify-2.txt`），再冒出一个**真的证明义务**：`contains_slot` 的 `start_slot + 64` 报 `possible arithmetic underflow/overflow`（`y1-m3-verify-3.txt`，`9 verified, 1 errors`）——因为 `start_slot` 小于 2^48 这个事实只由私有构造函数 `of_one_cluster_segment` 保证。补上 `#[verifier::type_invariant]` 并把构造函数也搬进 `verus!` 才过（`inplace` 变体，13 verified）。闭包里要搬的类型规模：`unit-graph` 两个入口并起来 289 个 struct、157 个 enum、283 个 const、199 个 impl（`out/closures/closure-*.txt` 按 `awk '{print $1}' | sort -u` 数）。`external_type_specification` 不是出路的省法：它在 `--no-cheating` 下可过（探针 `y1_c` `1 verified, 0 errors`），但把类型当透明结构交给 Verus、字段全公开，私有字段的不变量要另靠 `type_invariant`，等于没省。
3. **cargo 的编译指纹不看 `--verify-*`**（见 Y4 的「缓存」一条）：这条是接法的口子，不是 Verus 的。

### 1.4　推荐与会推翻它的现象

推荐：类型与被验函数用 `verus!` 宏包；函数体要保持纯 Rust 源码写法的少数几处（例如跨 crate 常量参与的算术）用属性模式；两种都放进 core 的同一棵源码树，不开第二个 crate。依据：上表（产品构建 0 警告、MIR 0 行差）。
推翻它的现象：(i) 入库装置上同一棵树的产品 MIR 与原版有真实差异（不是印法）；(ii) 产品构建时间在 `verus!` 展开之后明显涨（我没量，只量了 12 秒冷编译）；(iii) `verus!` 展开对 `#[must_use]`、`const fn`、derive 之外的属性（`#[cfg]`、`#[inline]`）有我没试到的改动——我试过 `#[must_use]`、`const fn`、`#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]`，没试 `#[cfg(test)]` 块内的类型。

## Y2　std 规格与「作弊标记」

### 2.1　vstd 的规格罩到 core 用到的 std 的多少（现数）

做法：`cargo rustc -p singlefs-core --lib -- --emit=mir` 出全 crate 的 MIR（5557 个函数、22902 处调用），`tools/mir_callees.py` 抽出每处调用的被调用者并归一成「类型::方法」或「特质::方法@自身类型」，`tools/std_coverage.py` 把 vstd 源码里每个 `assume_specification[ … ]` 方括号里那一项（整数宏占位 `$uN` / `$iN` 展开成各整数类型）、`external_trait_specification` / `external_trait_extension` 里的特质方法，按同样的归一法配上去。
**按名字配，不按类型配**：是估数，下面用探针校准。命令在 `tools/rerun.sh` 的 `step_y2`，原样输出在 `out/y2-mir.txt`。

整个 core（`out/y2-mir.txt` 前半）：

```
范围：整个 core；MIR 调用方函数 3368；vstd 规格键 758，特质方法 42，external_type_specification 类型 54
  类别 std / core / alloc: 10394 处调用
  类别 core 自己的类型与函数（含 derive 出来的特质方法）: 5363 处调用
  类别 drop 胶水: 4192 处调用
  类别 Verus 原生支持（? 与下标；探针 q01 q02 r05 r17 过）: 1547 处调用
  类别 格式化与 panic 机器（format! / panic! / Debug）: 1398 处调用
  类别 函数指针或闭包调用: 8 处调用
std 相关的不同调用键 800；按名字配上 vstd 规格 403；没配上 397
调用点 10394；落在配上规格的键上 8647；落在没配上的键上 1747
```

两个闭包并起来（发布 `publish_relocation_batch` 加回退 `roll_back_by_a_forward_publish`，`out/y2-mir.txt` 后半）：

```
范围：闭包（1073 个 fn 单元）；MIR 调用方函数 951；vstd 规格键 758，特质方法 42，external_type_specification 类型 54
  类别 std / core / alloc: 4628 处调用
  类别 core 自己的类型与函数（含 derive 出来的特质方法）: 1930 处调用
  类别 drop 胶水: 1846 处调用
  类别 Verus 原生支持（? 与下标；探针 q01 q02 r05 r17 过）: 690 处调用
  类别 格式化与 panic 机器（format! / panic! / Debug）: 146 处调用
  类别 函数指针或闭包调用: 5 处调用
std 相关的不同调用键 456；按名字配上 vstd 规格 224；没配上 232
调用点 4628；落在配上规格的键上 3813；落在没配上的键上 815
```

没配上的高频键（闭包内，调用点数 / 涉及的函数数）：`Extend::extend@Vec` 27/16、`slice::to_vec` 23/18、`Iterator::enumerate@Iter` 21/13、`Option::is_some_and` 18、`Option::copied`、`Option::map_or`、`slice::contains`、迭代器的 `chain` / `filter_map` / `flat_map` / `copied`、`num::to_le_bytes` / `from_le_bytes`、`num::div_ceil`、`BTreeMap::entry`、`bool::then_some`、`Option::filter`（完整前 70 行在 `std_coverage.py` 的输出里，`out/y2-mir.txt` 只留头 12 行以保参照稳定）。
闭包那一半只匹配上 951 个 MIR 函数（`closure_classes.py` 另算：1073 个单元里 185 个在 MIR 里没配上名字，原因我没逐个查，推测多是 trait 方法的默认体与宏生成的单元），所以闭包那一半是估数。

### 2.2　探针：96 个小函数直接起 `verus --no-cheating`

`probes/y2_constructs_{a,b,d,e}.rs` 与 `probes/y2c-split/` 四批，按 `pub fn` 拆开各起一次（`tools/probe-split.py`），原样结果在 `out/y2-probes.txt`（名字、退出码、首行）。分布（`awk` 现数）：

```
total 96 pass 54 fail 42 unsupported 27 postcond 4 verifier-no-support 2 decreases(my-error) 3 precond/overflow 3 other 3
```

- 过的 54 个：`?`（`Result` 同型与 `From` 转换）、`Vec` 下标 / `push` / `new` / `with_capacity` / `clone` / `==` / `extend_from_slice` / `truncate` / `resize`、切片 `for` 与 `Range` `for`、`Option::{expect, ok_or, take}`、`usize::try_from`、`u64::from`、`checked_add`、`wrapping_*`、`BTreeMap::{get, remove}`、`BTreeSet::insert`、枚举 `match`、`let … else`、泛型与 `dyn` 单 trait 的设备、`assert!` 带消息、`format!`、位移与掩码、数组字面量与下标。
- 撞上「std 函数没有规格」的 27 个（原话 `is not supported (note: you may be able to add a Verus specification to this function with assume_specification)`）：`Option::{copied, is_some_and, map_or, map_or_else, filter, is_none_or}`、`div_ceil`、`to_le_bytes` / `from_le_bytes`、`slice::{to_vec, contains, windows}`、`sort`、`Vec::{extend, retain}`、`Iterator::{enumerate, copied, chain, filter_map, find_map, max, sum, count@Filter}`、`Chunks`、`BTreeMap::entry` 的 `or_insert`、`bool::then_some`、`mem::take`。**这 27 个正是按名字没配上的高频键**；按名字配上的键没有一个因为「没有规格」失败。
- 证不出后置条件的 4 个：`Iterator::any` 带 `ensures`（`r02`，没有 `any` 的规格，说不出结果）、`Option::map` 的结果规格（`q06_option_map`，闭包的后置条件没传，我的 ensures 写强了）、`Fn` 约束的闭包参数（`r10`）、切片 `==` 与 `Seq` 的 `=~=`（`d23`，我的 ensures 写法问题）。其中只有 `any` 一条是 core 里真有的写法（43 处 `Iterator::any@Iter`）。
- Verus 自己说「还不支持」的 2 个：`Box<dyn Fn(u64) -> u64>`（`dyn with more that one trait`）与切片模式 `[a, ..]`。
- 要证的义务（不是不支持）：`panic!` 可达时报 `precondition not satisfied`（`e01`），不可达才过（`e02`、`e03`、`e04` `Result::expect` 要 `requires r is Ok`）。**core 全 crate 里 `Result::expect` 516 处、`Option::expect` 230 处、`panic_fmt` 79 处（`rerun.sh` 的 `step_y2` 把这三个键的调用点数写进 `out/y2-mir.txt` 末尾）每一处都是一条「走不到」的证明义务**。
- 实现 trait 给真设备要证的：`s13`（把一个字节数组盘实现成带 `ensures` 的 trait）报 `possible arithmetic underflow/overflow`：真设备的实现体自己就要证下标不溢出；产品里 `BlockDevice` 的几个实现（文件后端、内存盘）要么这样证、要么登记成可信声明（Y6）。
- 我自己写错的 3 个（`while` 缺 `decreases`）与拆分器把枚举 / trait 定义拆丢的 3 个（`r07`、`r11`、`r12`，用 `y2c-split/s07 s11 s12` 补过，全过）也算在 42 个失败里；校准按「失败原因」读，不按失败个数读。

按名字配的判断与探针对账：27 个按名字没配上的键各有一个探针，27 个都是 `is not supported`；按名字配上的键的探针里没有一个是「没有规格」。名字键没覆盖到的是**规格强弱**（`any`、`map`），那一类只有写了 `ensures` 才暴露。

### 2.3　`assume_specification` 算不算作弊；改成「登记 + 必红世界」行不行

**Verus 自己怎么看**（`out/y1-probes.txt`，直接起 `verus --no-cheating`）：

```
y1_d_assume_specification [--no-cheating] rc=1 error: external_body/assume_specification not allowed with --no-cheating
y1_d_assume_specification [允许作弊标记] rc=0 verification results:: 1 verified, 0 errors
y1_e_external_body [--no-cheating] rc=1 error: external_body/assume_specification not allowed with --no-cheating
y1_e_external_body [允许作弊标记] rc=0 verification results:: 1 verified, 0 errors
```

- `--no-cheating` 拒的不只是门禁的正则（`scripts/gate/verus-proof-check.py:473` 的 `CHEATING_MARKER`，含 `assume\w*`、`external_body`、`external_fn_specification`、`admit`）：Verus 自己拒 `assume_specification`、`external_body`，**连 `uninterp spec fn` 都拒**（`y1_e` 的第一处报错落在 `pub uninterp spec fn crc_spec` 那一行，见 `probes/y1_e_external_body.rs:4`）。D13 已定项 14 说哈希是 uninterpreted function（`.claude/kb/decisions/13-验证路线.md:387`）——在 `--no-cheating` 下这句**写不出来**；调用设备读写、校验和这类没有 Verus 规格的可执行函数，也只有 `external_body` 或 `assume_specification` 两条路。结论：core 直验**必须**对 core 这一个 crate 放弃 `--no-cheating`，换成别的把关。
- `assume_specification` 的**签名**Verus 自己核：把 `T` 钉成 `u64` 写一条，Verus 拒「assume_specification requires function type signature to match … exactly」（`out/y1-probes.txt` 末两行，`probes/y1_g_assume_specification_signature_must_match_std.rs`）。签名对了、`ensures` 写错，Verus 照单全收。
- vstd 自己就是靠这个：758 个 `assume_specification` 键（外加 42 个特质方法）不被审（vstd 不在 `--no-cheating` 管的 crate 里）。今天的 88 条证明已经信任 vstd 对 `Vec` / `Seq` / `Map` 的规格；把 vstd 的版本钉死（`.claude/gate.d/verus-pins.tsv:19` 的 `package vstd …` 那一行）是今天唯一的把关。

**判**：对 std 已知函数的 `assume_specification` 在 D13 已定项 14 的口径下不是「作弊」，是**可信部分扩大**（和 vstd 的 758 条同类）；今天的门禁把它与 `admit` 同判，是因为核的 crate 只依赖 vstd、从来用不上。改成「只许对 std 已知函数、逐条登记、每条配必红世界」可行，但要加的不止「登记」。

### 2.4　门禁怎么判得出「写的规格与 std 的行为不符」：演示

`assume-demo/`（模型目录，一个只依赖 vstd 的小 crate）+ `tools/assume-registry-check.py` 做四件事：① 源码里的 `assume_specification` 对象集合 == `registry.tsv` 的集合；② 好世界：`cargo verus focus` 过，差分测试（真实 `Option::copied` 在有界域 `{None, Some(0), Some(1), Some(u64::MAX), Some(1<<40)}` 上逐值过规格谓词的可执行孪生）过；③ 必红世界：把规格谓词（泛型版与可执行孪生一起）改成「`Some(x)` 的 `copied` 返回 `None`」，再往 `// CLAIMS-MARKER` 处放一个只在错规格下才证得出的假命题 `claims_none_for_a_nonempty_vector`；④ 打每步结果。原样（`out/y2-assume.txt`）：

```
name=assume_set source=['Option::copied'] registry=['Option::copied'] same=yes
name=good_verus rc=0 verification results:: 2 verified, 0 errors
name=good_test rc=0 test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
name=bad_verus world=copied-returns-none-for-some rc=0 verification results:: 2 verified, 0 errors
name=bad_test world=copied-returns-none-for-some rc=101 test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**读法**：错规格下 Verus 照样 `2 verified, 0 errors`（它验过一个假命题），只有差分测试红——所以登记式的把关**只靠差分测试**，必红世界证明差分测试会红。可执行孪生由 Verus 证它等于规格谓词（`copied_post_exec` 的 `ensures b == copied_post(…)`），这样被测的和被假设的是同一个谓词。
**局限**（逐条）：(i) 差分只在有界域上，`u64` 全宽取不满，只能抽样；(ii) 规格谓词要有可执行孪生，含量词、`Seq`、`Map` 的规格写不成孪生，这类规格不能登记；(iii) `external_body` 的设备读写、校验和**没有 std 对应物**，差分测试没有参照，要另立一种登记（契约测试：拿内存盘 / 文件盘跑 harness，校验和拿 `checksum.rs` 的向量），这一种我没做；(iv) 我试过 `exec_spec_verified!` 自动编孪生，编出的函数在 `Some(0)` 上与规格对不上，原因没追，改手写 + Verus 证等价；(v) 这是 `Option::copied` 一条的演示，不是门禁实现。
**什么现象会推翻这一节**：(a) 入库装置上，登记式把关放过一条写错的规格（差分域内有反例而测试不红）；(b) `assume_specification` 的签名核在更新的 Verus 里放宽，Verus 不再核泛型一致。

### 2.5　另一条路：不登记，改写 core

对按名字没配上的 232 个键，逐个改写成 Verus 有规格的等价写法（循环替迭代器链、`match` 替 `map_or` / `is_some_and`、自写 `to_vec`），没有新增可信部分，但**改的是产品代码**，要靠 harness 档用例与模型对拍确认行为不变。C 类函数 291 个（行数 41%）大半落在这里（Y3 的分类表）。两条路不互斥：设备与校验和只能登记，std 迭代器可以改写。

## Y3　射程与分阶段

### 3.1　闭包的规模与分类（现数）

命令：`tools/closures.sh`（每个入口一次 `unit-graph closure`）与 `tools/closure_classes.py`，原样输出在 `out/y3-closures.txt`。两个入口的 fn 数与第一轮正推腿量的相同（836 / 18057 行，1057 / 24494 行）：

```
singlefs_core::transaction::publish_relocation_batch rc=0
unit-graph：入口 singlefs_core::transaction::publish_relocation_batch：闭包 1903 个单元（Rust 1903、WGSL 0、数据 0），拒算 0 处
fn=836 fn_lines=18057 kernel_fn=2 kernel_fn_lines=12 kernel_share_of_lines=0.07%
singlefs_core::mount::roll_back_by_a_forward_publish rc=0
unit-graph：入口 singlefs_core::mount::roll_back_by_a_forward_publish：闭包 2292 个单元（Rust 2292、WGSL 0、数据 0），拒算 0 处
fn=1057 fn_lines=24494 kernel_fn=2 kernel_fn_lines=12 kernel_share_of_lines=0.05%
```

两个闭包并起来去重 **1073 个 fn 单元、24970 行**（发布与回退共用 820 个 fn；回退那边多 237 个，发布那边只多 16 个）。回退闭包里有 `relocation_checked_against_the_version` 之外的全部发布装配，回退闭包包含了发布闭包的 820/836，所以按回退入口划界，发布装配自然在内；反过来发布闭包里没有回退那 237 个（1057 − 820，与第一轮攻方量的 237 一致）。
按「一个 fn 直接要什么才进得了 Verus」分类（只看直接调用，没沿调用图传递；归类规则在 `closure_classes.py` 文件头）：

```
闭包 fn 单元（去重后）1073 个；MIR 里没配上的单元 185 个（归 A：空函数、trait 方法的默认体、或名字没配上）
  类 A: fn 单元 472，行数 5238
  类 B: fn 单元 190，行数 3340
  类 C: fn 单元 291，行数 10160
  类 D: fn 单元 120，行数 6232
  合计 fn 单元 1073，行数 24970
```

- A 直接可验：只调 core 自己的函数、`?`、下标、vstd 有规格的 std。B 只差 panic / format 机器：`expect`、`unwrap`、`panic!`、`format!` 的义务（证明「走不到」）。C 要改写：用了 vstd 没规格的 std（Y2）、迭代器适配、闭包调用。D 信任边界：调设备读写（`BlockDevice`、`PoolReader`）、`crc32_castagnoli`、哈希。
- 185 个没配上的单元归了 A，所以 A 是上界；传递没算：一个 A 类函数调了 C 类函数，实际上是 C 的依赖，所以「直接可验」的真实数比 472 小。
- 按文件（`out/y3-closures.txt` 后半，按 fn 数降序前几行）：`transaction.rs` 224（A117 B34 C60 D13，8947 行）、`recovery.rs` 170（66/17/48/39，4196 行）、`allocator.rs` 95（50/18/16/11，1475 行）、`extent_tree.rs` 47、`code_two_tree.rs` 45、`admission.rs` 44、`mount.rs` 44（13/0/17/14，1512 行）、`block_device.rs` 36、`allocation_record_tree.rs` 35。

### 3.2　哪些函数单元必须进证明才算「罩到」，哪些可以当未解释

**必须进证明**（罩到抽象状态的改变与保护；位置是冻结副本的行号）：

1. 条目判定与目的地：`relocation_checked_against_the_version`（`crates/singlefs-core/src/transaction.rs:10142`，133 行）、`allocate_relocation_destination`（`:11523`）与它调的分配器 `try_allocate_*`（`allocator.rs`）。
2. 发布装配：`prepare_the_version_publish_relocating_units`（`:11228`，267 行）、`placements_released_by_this_publish`（`:11570`）、`settle_the_allocation_record_tree`（`:11707`）；发布入口本身 `publish_relocation_batch`（`:9680`）。
3. 写序：`persist_publish_writes`（`:1440`，27 行；循环 `for unit in &writes.units` 在 `:1444`）。
4. 回收门槛与保护集：`reclaim_released_records_up_to`（`crates/singlefs-core/src/allocator.rs:2551`）及其两处调用 `released_record_can_be_reclaimed`（`allocator.rs:2561`、`:4466`；这个函数是 `gc_crash_proofs.rs` 里唯一被产品执行的核函数），根环占用与 F 的计算。
5. 回退：`roll_back_by_a_forward_publish`（`crates/singlefs-core/src/mount.rs:9665`）、`version_to_publish_the_rollback_after`（`:9509`）、`mapping_keys_whose_target_locations_differ_from_the_current_mapping`（`:9628`）、`rollback_candidate`（`:9168`）、`user_visible_units_through_the_mapping`（`:9131`）、`is_a_unit_taken_from_the_rollback_target`（`:9091`）、`central_mapping_locations_in_the_version`（`transaction.rs:4813`）。
6. 它们依赖的中央映射树、记账树、分配记录树的重写（`allocation_record_tree.rs`、`code_two_tree.rs`、`extent_tree.rs`、`inode_tree.rs`、`container_tree.rs`、`records.rs`）：重写改的是抽象状态里的映射与账本，不进证明就谈不上「映射是唯一权威」。

**可以当未解释函数**（只证调用方只依赖它的契约；契约登记成可信声明，Y2 的登记式）：

- `BlockDevice` 的读、写、屏障、FUA、写零（`block_device.rs:160` 起的 trait）与它的实现；文件后端打洞那个 `unsafe` 函数（`block_device.rs:568`，`lib.rs:10` 的 `deny(unsafe_code)` 在那里放开）。D13 已定项 4 的射程把真实设备的 FLUSH / FUA 归 C6（`.claude/kb/decisions/13-验证路线.md:142`）。
- `crc32_castagnoli` 与 `crc32_castagnoli_on_route`（`checksum.rs:100`、`:111`）：D13 已定项 14（`.claude/kb/decisions/13-验证路线.md:387`）「哈希是 uninterpreted function」。
- 字节编解码（`bytes.rs` 的 `ByteWriter` / `ByteReader` 20 个 fn、各 `to_bytes` / `from_bytes`）：先当未解释函数加往返契约，后期可证；
- 格式化、`Debug`、panic 文案。

### 3.3　第一阶段的最小原型（在 core 里直验，副本上验过）

两个原型都是**真在 core 里**：

- **kernelmodule（`patches/y3-s1.patch`）**：core 里新增 `relocation_judgement.rs`（模型目录 `src/relocation_judgement.rs`，175 行），`Placement`、`SlotNumber` 的定义搬进 `verus!`（字段与派生不变）；内容：`placements_overlap`（带 `起点 + 跨度` 不溢出的前提）、`disjoint_from_every`、`runs_are_pairwise_disjoint`、`relocation_destinations_are_acceptable`（ensures 与区间版 P4 的前提逐字对应）、`copies_sit_at_the_planned_source`、`unit_bytes_are_available`、一个防空真见证。`transaction.rs` 的 `relocation_checked_against_the_version` 两处调用它们（补丁里 `transaction.rs` 改了两处：源槽那个 `.filter(…)` 闭包与 `bytes.is_empty()`）。`cargo verus focus -p singlefs-core`：**13 verified, 0 errors**；产品构建 0 警告。
- **inplace（`patches/y3-m4.patch`）**：core 里**原地**验一个现有函数 `region_is_empty_by_the_records`（`compaction_plan.rs:948`）和它依赖的 `CompactionRegion`（`logged_operations.rs:71`，私有字段加类型不变量、构造函数一起进 `verus!`）、`AllocationRecord`、`DeviceIdentity`、`SlotNumber`、`CheckpointTxg`。循环改写（`iter().any` 没规格，改 `while`）。**13 verified, 0 errors**；产品构建 0 警告；补丁加 78 行删 17 行。
- 必红世界（`tools/red-worlds-core-s1.tsv`、`-m4.tsv`，`out/y3-red-worlds.txt`）七行，每行改坏一处，`cargo verus focus` 必须红：

```
s1：目的地与受保护落点比重叠时漏判	rc=101	verification results:: 12 verified, 1 errors	落点=crates/singlefs-core/src/relocation_judgement.rs:63:13
s1：两两不重叠只比相邻一项之外的	rc=101	verification results:: 12 verified, 1 errors	落点=crates/singlefs-core/src/relocation_judgement.rs:88:28
s1：源槽判定漏掉第二份副本	rc=101	verification results:: 12 verified, 1 errors	落点=crates/singlefs-core/src/relocation_judgement.rs:149:9
s1：字节可用判成长度大于等于 0	rc=101	verification results:: 12 verified, 1 errors	落点=crates/singlefs-core/src/relocation_judgement.rs:159:9
m4：contains_slot 不引类型不变量	rc=101	verification results:: 12 verified, 1 errors	落点=crates/singlefs-core/src/logged_operations.rs:142:57
m4：region_is_empty 把已释放的也算占着	rc=101	verification results:: 12 verified, 1 errors	落点=crates/singlefs-core/src/compaction_plan.rs:959:9
m4：构造函数漏掉 6 字节上限	rc=101	verification results:: 12 verified, 1 errors	落点=crates/singlefs-core/src/logged_operations.rs:117:12
```

（第二行是改成 `i + 2` 之后先撞上 `i + 2` 的溢出义务，不是撞在「漏比一对」上；仍然红，但不是我标的那个原因。）
- 核内占比：接上之后发布闭包 `fn=839 fn_lines=18083`，被验的 fn（核里的加 core 新模块里被调用的）5 个 35 行，占 0.19%，对第一轮正推腿的核外提原型 0.28%（50/18110）更低——因为这一版把**受保护集的目的地判定还没接**（`relocation_destinations_are_acceptable` 在 core 里没人调，不在闭包里）。**这个数说明：第一阶段只让「被验的行」涨一点点，射程要靠后面几阶段**。
- **没做**：没跑 core 的单元测试或 harness 用例确认接线后行为等价（只 `cargo build` 过）；`relocation_destinations_are_acceptable` 要的受保护集（分配器里未回收记录的区间）在 core 里没有现成的 `Vec<Placement>` 形态，接线要多写一个把分配器状态投影成区间表的函数，这一步没做。

### 3.4　分阶段计划（每阶段验收；义务数怎么现数）

现数方法（每阶段都用同样三样）：① `unit-graph closure` 后数「被验的 fn 单元占闭包的行数」（照第一轮 1.6 的闭包量法，`tools/closures.sh` + 一行 `awk`，上一节的 0.07% → 0.19% 就是这一样）；② `cargo verus focus -p singlefs-core` 读 `verification results:: N verified`（登记数；见 Y4）；③ 必红世界行数 ≥ 带 `ensures` 的被验函数数（门禁今天就是这么判的，`scripts/gate/verus-proof-check.py:2017` 的 `negative_coverage_gaps`）。

| 阶段 | 内容（函数范围） | 本闭包里的 fn 数（A/B/C/D，行数） | 验收 | 状态 |
|---|---|---|---|---|
| 基础设施 | core 加 vstd 依赖三处；类型垫片（地址 newtype、`Placement`、`AllocationRecord`……搬进 `verus!`）；门禁改造（Y4）；信任清单骨架 | — | 产品 MIR 与原版差 0 行；`cargo verus focus -p singlefs-core` 0 errors；登记表有 core 这一行 | 垫片的做法已在两个原型里走通 |
| 判定核 | 条目判定、目的地判定、`region_is_empty_by_the_records`、区间版 P4 | 29（20/3/5/1，317 行） | 本节两个原型：13 + 13 verified；七行必红世界全红；`relocation_destinations_are_acceptable` 接上受保护集 | 原型有，接线没做完 |
| 发布装配与写序 | `transaction.rs` 整份：`prepare_the_version_publish_relocating_units`、`persist_publish_writes`、`placements_released_by_this_publish`、`settle_the_allocation_record_tree` | 224（117/34/60/13，8947 行） | 引理：设备操作序列 = 复制 → 屏障 → journal → 屏障 → 根槽 FUA，对上 `gc_ring_proofs` 的 `Write` / `Relocate` / `Checkpoint` 序列；`BlockDevice` 的 trait 带 ghost 操作日志 | 没做 |
| 分配器 | `allocator.rs` 回收门槛与保护集、`admission.rs`、根环 | 199（91/53/43/12，3354 行） | 引理：回收条件蕴涵 `ring_slot_is_unreferenced`（`gc_crash_proofs.rs:70` 自己写的「本模型没有从 floor 推出它」那一条） | 没做 |
| 回退与挂载 | `mount.rs` 回退一路、`mounted_session.rs` | 101（34/6/31/30，2705 行） | 引理：`roll_back_by_a_forward_publish` 对上 `RingStep::Rollback`；回退后的现行视图读得对 | 没做 |
| 树重写 | 映射树、记账树、分配记录树、extent / inode 树、records | 228（89/56/76/7，3754 行） | 每棵树的重写保持「key → 字节」；`BTreeMap` / `Vec` 的 C 类改写 | 没做 |
| 恢复与编解码 | `recovery.rs`、`journal.rs`、`unit.rs`、`bytes.rs`、`checksum.rs`、`block_device.rs` | 291（120/38/76/57，5887 行） | journal 重放对上 `Relocate` / `Checkpoint`；编解码往返 | 没做；部分已在 `singlefs-replay-prefix`（`replay_journal` 核内占比 69%，第一轮量的） |

### 3.5　总代价（推的）

- 倍数取自两个原型：`relocation_judgement.rs` 175 行对应纯 Rust 约 35 行（约 5 倍），`region_is_empty_by_the_records` 7 行变 32 行（约 4.6 倍），都是 A 类。C、D 类的倍数我没量，只会更高。
- **只验 A + B 两类**（662 个 fn、8578 行）：被验版本约 3.9 万到 4.3 万行（取 4.5–5 倍）。
- **全验**（1073 个 fn、24970 行）：被验版本约 11 万到 12.5 万行，比原来多约 9 万行；C 类的改写另算（对 C 类 291 个 fn 里按名字没配上的 232 个键逐个出路：改写或登记）；D 类设备与校验和只出一份信任清单（预计十几条，每条 5–15 行加差分 / 契约测试）。
- 登记数：两个原型 verified 数与 fn 数之比约 2–2.6（因为 spec fn 与 derive 的 `clone` 也各算一条）：A+B 约 1300–1700，全验约 2100–2800，对今天登记的 88（`.claude/gate.d/verus-pins.tsv:29`）。
- 机器时间不是瓶颈：整 crate 88 条 3.4 秒，110 条 4.0 秒（草稿里 `date` 量的，31 线程）；core 加 13 条 `cargo verus focus` 约 3.6–5 秒。线性外推 2800 条约 2 分钟。瓶颈是写证明的人力和改写 C 类带来的产品代码风险。
- 这个总数**是推的**：没有一个阶段做完过，发布装配以后没原型。**什么现象会推翻它**：做完判定核接线后，实际行数倍数明显高于 5（说明 A 类也不是 5 倍），或发布装配阶段的写序引理发现 `BlockDevice` trait 带 ghost 日志要改动的实现数远大于 2（文件后端、内存盘、稀疏盘……）。

## Y4　判法与门禁

`scripts/gate/verus-proof-check.py` 与 `.claude/gate.d/verus-pins.tsv` 在 core 进来之后要改的地方，逐条（行号是主工作区此刻的；证据是模型目录 `out/y4.txt`、`out/y1-probes.txt`、`out/y2-assume.txt`；门禁一概没起）：

1. **登记表只认一个 crate。** `read_registry` 里 `crate` 行宽度 3（`scripts/gate/verus-proof-check.py:565`），多于一行判红（`:582`），登记表今天是 `crate	crates/singlefs-replay-prefix	88`（`.claude/gate.d/verus-pins.tsv:29`）与三行核（`:30`–`:32`）。改法：`crate` 行允许多条，每条自己一个登记数；core 一条，`singlefs-replay-prefix` 一条（并入第五节的新模块后 88 → 110）。核行不再手登：`kernel_files`（`:613`）已经「有 `verus!` 的文件必须登记」（`:626` 的 `未登记的 Verus 核`），core 里文件数多、改动频，改成**自动收集**（有 `verus!` 或 `#[verus_verify]` 的文件）更不易漏。**注意它今天的检测只认 `verus ! {` 的词序**（`:625` `words[index:index + 2] == ["verus", "!"]`），属性模式的文件会被当成没有核，要补。
2. **直接起 `verus` 编单文件入口，依赖只许 vstd。** `verus_arguments`（`:716`）拼的命令以 `entry` 文件结尾，`check_dependencies`（`:675`）核的 crate 除 vstd 外有依赖就在开跑之前判红（`:10` 的文件头同写）。core 依赖 `libc`、`singlefs-format`、`singlefs-replay-prefix`（冻结副本 `crates/singlefs-core/Cargo.toml:14`–`:17`）。改法：对 core 用 `cargo verus focus -p singlefs-core`（最小工作区里实测 `13 verified, 0 errors`，`out/y1.txt`）；`copy_crate`（`:684`）改成拷「core + 它的路径依赖 + 只含这三个成员的 `Cargo.toml`」：整个工作区的 `cargo verus` 在离线机器上过不了（Y1 的 `cargo metadata` 报错）。`:681` 写着老的「用 Verus 钉的工具链编依赖、交给 `--extern`」的路线（`git log -S prepare_externs` 找得到），是另一条出路。
3. **cargo 的指纹不看 `--verify-*`，同一个编译目录里换参数再验拿到的是上一趟的结果。** `out/y4.txt` 原样：

   ```
   name=y4_first rc=0 note: verifying module compaction_plan (selected functions)|verification results:: 2 verified, 0 errors (partial verification with `--verify-*`)|
   name=y4_second_same_target rc=0 note: verifying module compaction_plan (selected functions)|
   name=y4_second_fresh_target rc=0 note: verifying module logged_operations (selected functions)|verification results:: 1 verified, 0 errors (partial verification with `--verify-*`)|
   ```

   第二行点名的是 `logged_operations`、`CompactionRegion::contains_slot`，输出的却是上一趟的模块、没有 `verification results` 行。`touch lib.rs` 不够（我试过，原样 `Finished … in 0.21s`，没重验）；`cargo clean -p singlefs-core` 要指到 `<编译目录>/verus-partial` 才删得掉（`cargo verus focus` 的产物在那里：删 28 个文件 96.4 MiB，下一趟只重编 core，约 7 秒，草稿里 `out/y4-clean2.txt`、`y4-touch-d.txt`）。出路：每次判定前清 core 的产物，或每个判定自己的编译目录（冷编 vstd 约 20–28 秒）。今天的 `copy_crate` 每次拷一份不带 `target` 的副本（`:684`），天然是后一种。
4. **逐函数项可以不逐函数起 Verus。** `cargo verus focus … -- --output-json` 一趟整 crate 的 JSON 里每个函数一项（`func-details`），实测 core 原型里 19 条 core 条目、`verified: 13`（`out/y4.txt` 末行）。`--verify-only-module <模块> --verify-function <类型::方法>` 点名单个函数也行，但只认**全名**：裸 `contains_slot` 报 `more than one match`（`out/y4.txt` 倒数第二行），要写 `CompactionRegion::contains_slot`。逐函数起 2000 个 Verus 约 7 秒 × 2000 ≈ 4 小时，不现实；从整 crate 的 JSON 读逐函数结果是出路（推的：我没把门禁的逐项判定改成读 JSON）。
5. **作弊标记一概禁，而 core 直验必须有。** `CHEATING_MARKER`（`:473`）含 `external_body`、`external_fn_specification`、`assume\w*`、`admit`、`verifier::external`；Verus 自己在 `--no-cheating` 下拒 `assume_specification`、`external_body`、`uninterp spec fn`（Y2）。改法：对 core 去掉 `--no-cheating`；`cheating_markers`（`:1070`）改成「标记集合 == 信任清单」：新表 `.claude/gate.d/verus-trusted.tsv`，每行：种类（`assume_specification` / `external_body`）、对象（std 路径或 core 函数）、所在文件、差分或契约测试名、必红世界名、理由；`admit`、裸 `assume(`、`#[verifier::external]`（让整个函数对 Verus 不可见）**仍一概禁**。信任清单每行的三道判在 Y2 2.4 的演示里：集合相等、测试过、必红世界里 Verus 照过而测试红。
6. **必红世界「每个带 `ensures` 的函数一行」。** `negative_coverage_gaps`（`:2017`）要每个带 `ensures` 的函数、每个 `invariant` 块都有一行；core 里是上千个，手写不现实。改法：自动世界——对每条 `ensures` 子句机械生成一个取反变体、Verus 必红；手写行只留给有语义意图的几十行。自动世界我没试（推的）。我给两个原型手写了七行（`tools/red-worlds-core-*.tsv`），七行都红。
7. **空真筛查每处 `requires`。** `requires_insertion_points`（`:1577`）要求 `requires` 独立成行、函数体的 `{` 独立成行（verusfmt 风格）；core 的 `verus!` 块要按这个格式写。`verus_functions`（`:1151`）拒 `verus!` 里 impl / trait 里的函数（`:1169`）与跨核同名函数（`:1175`）；core 里方法同名是常态（`new`、`device`、`start_slot`……），要改成「模块 + `类型::方法`」全名索引，函数项名带模块（`kernel_module`，`:694`，core 的模块就是 `src/*.rs`，平的）。
8. **被验区域之外的调用点。** Verus 不看 `verus!` 外面的代码（Y1：绕过类型不变量构造值、不满足 `requires` 调被验函数，`3 verified, 0 errors`，`probes/y1_f_unverified_callers_are_not_checked.rs`）。新增一格：① 带 `#[verifier::type_invariant]` 的类型，struct 字面量只许在被验区域里；② 带 `requires` 的被验函数，调用点只许在被验区域里（测试模块除外）或登记为信任边界入口。判法用 `unit-graph` 的调用边加 grep，没写。
9. **单元键与复用（D13 已定项 17，`.claude/kb/decisions/13-验证路线.md:429`）。** 键只由「函数单元的闭包指纹、cfg 原子、工具链、`Cargo.lock` 解析出的依赖版本」组成（`:432`），`--verify-*` 参数、编译目录、路径不进键（`:433`）。对 core 里被验的函数：键 = 它的闭包（fn、类型、常量、宏展开，含 `verus!` 里的 spec fn 与 proof fn）；入口 `publish_relocation_batch` 的闭包 1903 个单元，叶子变了它作废——这是对的（它依赖叶子），代价是顶层项作废频繁、叶子项不受影响。Verus 与 vstd 版本进键（今天 `tool-versions` 已有）。信任清单本身与它的差分测试也要进键。
10. **`.claude/rules/verification.md` 两句。** 冻结副本与主工作区同：`:13` 那一行里「形式化验证（checker-tier-formal-verification：Verus 验 `crates/singlefs-replay-prefix`，E165 红例钉表两格）」改成「Verus 验 `crates/singlefs-replay-prefix` 与 `crates/singlefs-core` 里 `verus!` 或 `#[verus_verify]` 标出的函数」；`:126` 那一行里「四格用 Verus 验 `crates/singlefs-replay-prefix`（登记表 `.claude/gate.d/verus-pins.tsv`）」改成「四格用 Verus 验登记表点名的各 crate（`crates/singlefs-replay-prefix`、`crates/singlefs-core`），core 一侧的信任清单 `.claude/gate.d/verus-trusted.tsv` 另有三道判（集合相等、差分或契约测试过、必红世界里测试红）」。这是改写草稿，措辞由主 agent 定。
11. **D13 已定项 5、13、14 的影响。** 已定项 5：`checker-implementation-disjoint` 格判 crate 依赖闭包交集（`.claude/gate.d/checker-independence-and-sync.sh:16`），`singlefs-checker` 只依赖 `singlefs-format`（`crates/singlefs-checker/Cargo.toml:9`）；vstd 已在 core 闭包里，checker 的闭包里没有，所以 core 加 vstd 不影响它。**但别让 `singlefs-format` 进 Verus 视野**（给它加 vstd 依赖会让 checker 的闭包里出现 vstd，碰到 `.claude/kb/decisions/13-验证路线.md:162` 的「`singlefs-checker` 只依赖 `singlefs-format`」）；core 里对 format 的常量用「`verus!` 里同值常量加块外 `const _: () = assert!(… == singlefs_format::…)`」的写法（没试）。已定项 13：新 spec（`gc_ring_proofs`、区间 P4）并入同源性登记的一对 `refinement-spec-o1-model`（`.claude/gate.d/divergence-events.tsv:15`），事件仍靠人手记。已定项 14：在表里加一行「std 规格与设备 / 校验和契约：信任清单」，哈希仍当未解释函数、碰撞抗性在验证范围之外。

**什么现象会推翻这一节**：(a) 入库的 `verus-proof-check.py` 里 `cargo verus focus` 在门禁的离线环境下也过不了（最小工作区之外还缺包）；(b) `--output-json` 的 `func-details` 在入库装置上缺函数（不是每个函数一项）；(c) 自动世界（取反 `ensures`）对一批函数不红（平凡的 `ensures`）。

## Y5　第一轮挂起的三处

两个新模块都在模型目录 `src/`，放进 `singlefs-replay-prefix` 的副本、加两行 `pub mod` 之后整 crate 直接起 `verus --no-cheating --crate-type=lib --edition=2021 …`（参数同 `scripts/gate/verus-proof-check.py:716` 的 `verus_arguments`，没起门禁）。`out/y5.txt` 原样：

```
name=y5_crate label=冻结副本原样 rc=0 verified=88 errors=0
name=y5_crate label=加两份新证明 rc=0 verified=110 errors=0
name=y5_single_file file=relocation_interval_proofs rc=0 verification results:: 9 verified, 0 errors
name=y5_single_file file=gc_ring_proofs rc=0 verification results:: 13 verified, 0 errors
```

88 与 `.claude/gate.d/verus-pins.tsv:29` 登记数相等；110 = 88 + 9 + 13。必红世界十二行全红（`tools/red-worlds-y5.tsv`，同一份 `out/y5.txt` 后半），例：

```
区间：目的区间不与映射区间重叠的前提去掉	rc=1	errors=3	verified=6	落点=relocation_interval_proofs.rs:175:20
环：回收只避开现行与读者、不避开回退候选	rc=1	errors=2	verified=11	落点=gc_ring_proofs.rs:157:17
环：回退可以退到 floor 之下	rc=1	errors=2	verified=11	落点=gc_ring_proofs.rs:193:20
环：Checkpoint 从不挤窗口	rc=1	errors=3	verified=10	落点=gc_ring_proofs.rs:149:9
```

### 5.1　新协议 `gc_ring_proofs.rs`（483 行）

对比 `gc_crash_proofs.rs`（冻结副本 `crates/singlefs-replay-prefix/src/gc_crash_proofs.rs`）：状态只有 `disk`、`current`、`durable`、`readers` 四样，所有读者共享同一个 `logical`（该文件 `:9` 的原话「所有读者共享同一个 `logical`，连 key 的定义域也相同」），`AcquireReader` 不是持久快照、`Crash` 丢掉它（`:10`），回收只避开当前根、持久根、留存读者（`:70` 的注释自己写着「本模型没有从 floor 推出它」）。新协议：

- **每个视图带自己的逻辑内容**：`RootView { mapping, logical }`（`gc_ring_proofs.rs:18`）；状态 `GcRingState`（`:23`）有 `disk`、`roots`（根环里的持久根，旧到新，**Crash 不清**）、`floor`（回退候选是 `roots[floor..]`，只增不减）、`head`（现行版本，可领先最后一个持久根）、`readers`（各带自己的逻辑版本）、`window`（根环能容的候选数）。
- 步骤 `RingStep`（`:37`）十种：`Write`、`Reclaim`、`Relocate`、`UserPublish`（用户发布新逻辑版本）、`Checkpoint`（现行版本落成新根；候选数超窗口就把最旧的挤出去，`floor_after_a_checkpoint` `:106`）、`Rollback`（现行版本换成某个候选根的视图）、`RaiseFloor`（只许往上）、`AcquireReader`、`ReleaseReader`、`Crash`（现行版本回到最后一个持久根，读者清空）。
- **回收与覆写要避开三类视图引用的槽**：现行、每个回退候选、每个读者（`ring_slot_is_unreferenced`，`:82`）。不变量 `ring_all_views_read_right`（`:70`）：现行、每个候选、每个读者各自读到自己的逻辑内容。
- 一步保持（`lemma_ring_step_preserves_every_view`，`:144`）与任意有限历史（`theorem_ring_every_finite_history_keeps_every_view`，`:217`）：历史里可以有崩溃、回退、抬 F、挤窗口、不同逻辑版本的读者与根。

### 5.2　三个反例怎么被排除

| 第一轮攻方的反例 | 在新协议里 | 引理（`gc_ring_proofs.rs` 行） |
|---|---|---|
| 搬迁、checkpoint 之后立刻回收并覆写源槽，每步合法，旧版映射（回退要复活的）读坏 | 旧根还是回退候选时，它引用的源槽回收与覆写**都不合法**；具体重放：第一轮那五步在第四步（回收）处停下 | `lemma_the_source_of_a_rollback_candidate_cannot_be_reclaimed`（`:240`）；`lemma_the_first_round_sequence_stops_at_the_reclaim_when_the_old_root_is_still_a_candidate`（`:262`，`window: 2`，证 `s3.floor == 0` 与 `!ring_step_enabled(s3, Reclaim { slot: 10 })`） |
| 同上，中间插一次 `Crash` 清空读者，回退目标还在盘上的根环里 | 根环与 `floor` 在 `Crash` 之下原样，候选继续保护自己的槽 | `lemma_a_crash_keeps_the_rollback_candidates_protecting_their_slots`（`:254`） |
| 把回退目标当读者塞进去，它是另一个逻辑版本（key 集合不同）时 `gc_all_views_match` 对任何 `logical` 都假，前提空真 | 每个视图各带自己的 `logical`，不变量按视图各自量化；key 集合不同的两个根与一个读者同时满足不变量 | `witness_two_roots_with_different_logical_versions_satisfy_the_invariant`（`:360`） |
| P4：两槽目的地罩住一槽单元，起点互异的前提照样成立 | 前提改成区间不重叠；那个具体状态使前提不成立；整次搬迁的定理在区间前提下证出 | `lemma_the_first_round_two_slot_counterexample_breaks_the_interval_premise`（`relocation_interval_proofs.rs:319`）、`theorem_relocating_runs_in_order_keeps_every_unit_read`（`:217`）、防空真 `witness_a_two_unit_plan_satisfies_the_interval_premise`（`:341`） |

窗口只容一个候选时第一轮那五步的第四步**合法**（`lemma_with_a_window_of_one_the_old_root_leaves_and_its_source_becomes_reclaimable`，`:312`）：`Checkpoint` 把旧根挤出窗口（`floor == 1`），`Reclaim { slot: 10 }` 合法，而同时 `Rollback { target: 0 }` 不再合法——回收源槽读坏不了任何候选，因为那个候选已不是候选。**这就是 G7 的真实内容：整理的发布占根环槽，用户看得见的变化是窗口移一格，不是读坏字节。**

### 5.3　区间版 P4（`relocation_interval_proofs.rs`，378 行）

映射值改成「起点 + 跨度」（`SlotRun`，`:12`），盘是「每个槽一段字节」，读一个 key 是读它区间里的每个槽。前提 `relocation_plan_is_well_formed`（`:98`）：key 两两不同、源区间等于映射里的区间、目的跨度等于源跨度且至少 1、目的区间两两不重叠（`destination_runs_are_disjoint_and_free`，`:82`）且与映射里**任何**一个区间都不重叠、映射里每个区间跨度至少 1。定理（`:217`）：按次序搬前 `count` 项之后每个 key 读到的单元（逐槽字节）不变，被搬的 key 指到各自的目的区间。
四行必红世界各去掉一条前提（去掉哪一条哪一条红，`out/y5.txt`）。**局限**：区间与槽号用 `int`（协议层数学整数），core 的 u64 版是 `relocation_judgement.rs` 的 `placements_overlap` 等可执行函数，它们的 `ensures` 是同一个谓词在 u64 上的写法，但**没有一条引理把 `Placement`（u64）与 `SlotRun`（int）接成一条证明链**；溢出的前提 `起点 + 跨度 ≤ u64::MAX` 写在可执行函数的 `requires` 里（`relocation_judgement.rs` 的 `placement_is_bounded`）。

### 5.4　G7 改写成什么

恒等性质的主语要连「回退能退回的版本集合」与 F 一起说。协议层已证的引理给出的精确说法（`gc_ring_proofs.rs:401` 起）：

用户看得见的状态记作 `(H, W, C)`：`H` 是现行版本的逻辑内容；`W` 是回退候选窗口里各根的逻辑内容按新旧排成的序列（`window_logicals`，`:401`）；`C` 是抬 F 的上限（最后一个根的下标，`raise_floor_ceiling`）。

- 整理里不发布的三步（写目的地、换映射、回收）不改 `(H, W, C)`（`lemma_the_non_publishing_gc_steps_change_nothing_the_user_sees`，`:410`）。
- 任何一步都不改环里已有的根（`lemma_no_step_rewrites_a_root_already_in_the_ring`，`:422`）：整理改不了旧根的逻辑内容。
- 整理的一次发布（现行版本的逻辑内容与最后一个根相同，没有用户的改动在飞）：`H` 不变，`C` 加一，`W` 变成 `W` 末尾追加 `H`、**窗口满时丢最旧的一个**（`lemma_a_gc_checkpoint_only_shifts_the_window`，`:436`）。窗口没满时一个回退落点都不少（`:457`）；窗口满时恰少最旧的那一个（`:471`）。

所以「带 GC 的与不跑 GC 的逐项相同」在含回退与抬 F 的历史上**不成立**，成立的是：对不含回退、不抬 F 的历史逐项相同；含回退与抬 F 的历史，带整理那份的窗口比不带整理的那份多移 `整理发布数` 格，抬 F 的上限多 `整理发布数`（对次数的归纳我没写成引理，推的）。第一轮攻方量到的「回退可达集合 12 格里 10 格不同」与这个说法相容（不是证明它们相等）。
另一条路是改实现让整理的发布不占窗口（D16 已定项 1 的读法：只数改过用户可见状态的根）；那样 `W` 在整理发布下原样、恒等恢复成立。**我没有建模这一条**（协议里 `Checkpoint` 只有一种），要它得先由用户定「搬根算不算改了用户可见状态」（第一轮攻方 X1 打中处，`research/prompts/m4-exit5-relocation-proof-r1-opus-output.md` 记了条款出处）。

### 5.5　这一节没证的与会推翻它的现象

- 协议是协议：槽是原子地址，没有设备编号、span、journal 编码、屏障与 FUA，`Checkpoint` 原子；不变量是**前提**，没有一条证明说 core 的分配器回收条件蕴涵 `ring_slot_is_unreferenced`（分配器一阶段，Y3）。
- `RaiseFloor`、`window` 是抽象：真实的 F 由挂载准入与空间压力推（`.claude/kb/decisions/16-发布语义.md` 的已定项），我只证了「上限随整理的发布加一」，没建模 F 怎么被推。
- 快照共享块（留存的映射条目跟着换位置）、隔离记号、双盘副本不在协议里（Y6）。
- 没证新协议是旧协议的推广（单逻辑版本、无回退时退化成 `gc_crash_proofs`）；只在具体见证里让两者并存（`:360` 的见证含一个读者与两个根）。
- 推翻它的现象：(i) 入库装置上，一份历史里整理的 checkpoint 之后某候选根读到的字节与它的逻辑内容不同而新不变量还成立（说明前提集不全）；(ii) 对拍里「带整理比不带整理多移的格数」不等于整理发布数。

## Y6　罩不到的

在 Y1–Y5 的形下仍然罩不到的，逐项写由什么罩（「罩」= 已在仓里有的判法；括注「新」= 要新立）：

| 项 | 为什么罩不到 | 由什么罩 |
|---|---|---|
| 设备屏障与 FUA 的真实语义 | Verus 只能证 core 发出的设备操作序列（`BlockDevice` 带 ghost 操作日志，Y3 发布装配阶段）；设备是否守序是设备的事 | 崩溃点重放的按设备切段枚举（`.claude/kb/decisions/13-验证路线.md:140`）；真设备 FLUSH / FUA 归 C6，要 QEMU（`:142`） |
| 字节编解码（`bytes.rs`、各记录 `to_bytes` / `from_bytes`） | 先当未解释函数；往返契约是登记的前提不是证明 | 独立解析器与池级 checker（D13 已定项 5、7）；崩溃点重放的 CPU 判器在每个崩溃状态上调 `recover`；往返契约测试（新） |
| 校验和无碰撞（CRC32C） | D13 已定项 14：哈希是未解释函数，碰撞抗性在验证范围之外（`:387`）；已定项 1 射程把「校验和过了」写成布尔（`:69`） | 知情接受（已定项 4 对 CRC32C 碰撞概率）；`checksum.rs` 的向量测试；位置提示读路径要过整单元校验和（`recovery.rs`，第一轮 1.2） |
| 双盘副本（两份副本同槽、一盘丢失） | 协议里一个槽一份字节；`[LocationEntry; 2]` 两份副本判成「都等于计划源槽」（`relocation_judgement.rs`）只是形状，不证两盘同步 | 崩溃点重放的盘数维（已定项 1 射程「槽数与设备数」）；故障注入（harness）；N6（I-1.14：同一映射 key 各版字节相同） |
| 隔离（quarantine 记号与 `quarantined_after_release_checksum_mismatch`） | 协议没有这一类；`gc_crash_proofs` 自己也没列进已连接的前提 | 崩溃点重放全量与池级 checker（具体不变量编号我没逐个核） |
| 共享快照（留存的映射条目跟着换位置，`transaction.rs:9662` 的文档句） | 读者视图只有「一份映射」，没有「映射条目被快照引用」的另一份记账 | O1 模型对拍里的快照操作与池级 checker；协议加快照视图是新工作（新） |
| 并发与会话（挂着时整理与用户写交错、串行提交之外） | D13 已定项 2 只做串行提交（`:91`）；协议是顺序步骤 | 随机历史对拍（`compaction_while_mounted_in_random_histories.rs`）、崩溃点重放里 GC 流的 40 段（`.claude/gate.d/crash-point-replay-items.tsv:24`） |
| 「GC 不把成功变成 ENOSPC，除非 df 也跟着少报」 | 准入与账本的算术不在搬迁协议里 | 对拍的另判（G7 的 H-G7-2，`research/prompts/m4-gc-r1-main-verification.md:38`） |
| 分配器空闲图的数据结构内部 | 只证对外契约（回收条件、保护集），内部 `BTreeMap` 的平衡与迭代不证 | 池级 checker 的「分配记录罩住的槽互不相交」（`allocator.rs` 的类型文档写作 I-5.4） |
| journal 编解码与重放把搬迁还原成 `Relocate` / `Checkpoint` | 要先把 `recovery.rs` 的 170 个函数验过（Y3 恢复编解码阶段），今天只有核（`replay_journal` 核内占比 69%，第一轮量的） | E165、崩溃点重放、`singlefs-replay-prefix` 里已有的择根与取前缀核 |
| 位置提示读路径（父节点里的旧提示先读、读不出回退中央映射，`recovery.rs:761`） | 提示是性能旁路，「读得出」靠整单元校验和 | 崩溃点全量枚举；校验和契约 |
| 抽象状态到 O1 可观察读回（POSIX 内容）的桥 | 解码只经映射读单元，是点名前提；Verus 的 `logical` 是 key 到字节 | O1 对拍；I-1.13（扫描重建）；N6 |
| GPU 后台计算（`background_compute.rs`，计划在 GPU 上算） | GPU 实现另写一份，验不到 core | 快档里 GPU 与 CPU 的对拍（D13 已定项 17 射程，`:439`） |
| 工具链：Verus、z3、vstd 规格本身对不对 | 全是可信部分；vstd 的 758 个 `assume_specification` 不被审 | 版本钉死（`.claude/gate.d/verus-pins.tsv:16`、`:19`）；登记式信任清单只管 core 一侧新增的 |
| 文件后端打洞的 `unsafe`（`block_device.rs:568`） | `deny(unsafe_code)` 在那里放开，Verus 不看 | 文件后端的测试；信任清单（设备类） |
| 被验区域之外的代码对被验类型的构造与对被验函数的调用 | Verus 不看（Y1 的 `y1_f` 探针） | 新格「被验区域外的调用点」（Y4 第 8 条）（新） |

这十五项里，「新」标的三项（往返契约测试、快照视图、被验区域外调用点那一格）是本形下才新增的工作；其余十二项今天已有判法在罩，本形不改变它们的责任。

## 冲突两句并排（整行抄）

只列「应该是什么」（kb 与实现）与被判对象（今天的条款与门禁）对不上的。每对的行号现 `awk 'NR==…'` 取过。

**冲突一　规则文字说 Verus 验的是 `singlefs-replay-prefix`，出口 5 要验 core 的搬迁执行。**

`.claude/rules/verification.md:126`：
```
| 形式化验证：恢复重放取前缀、择根、扫环三截的核与择根取前缀的合判推论经 Verus 验过，E165 全量的红例都钉进快档、钉表自己成形、在被判那棵树的实现上逐行重放与原实现相同 | 门禁 checker-tier-formal-verification：verus-toolchain-pinned、verus-proof、verus-vacuity、verus-must-red-worlds 四格用 Verus 验 `crates/singlefs-replay-prefix`（登记表 `.claude/gate.d/verus-pins.tsv`）；红例钉表 `.claude/gate.d/e165-pinned-red-examples.tsv` 两格：pinned-red-examples-table 只判钉表自己（每行 22 段、指纹、抄来的段、档的写法、点名的变异还在，不读红例账、产物与 `research/`），pinned-red-examples-replay 逐行重放；钉全由 E165 第二段驱动 `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` 写完一对红例账与产物时当场判（追加、再 `--check` 同一对，没钉全那一趟判红；它文件头「钉红例」一段） |
```
`.claude/kb/milestone/04-fourth-txn.md:20`：
```
| 5 | 「GC 在抽象文件系统状态上是恒等变换」在里程碑三七·1 选定的工具上证出；带 GC 的模型对拍与不跑 GC 逐项相同。用户 2026-10-26 定：证明要罩到 core 的搬迁执行（`publish_relocation_batch` 那一路），抽象模型的证明加模型对拍不算证出 | 二 |
```

**冲突二　门禁把 `assume*` / `external_body` 一概判作弊，D13 已定项 14 要哈希当未解释函数、core 直验要调设备。**

`scripts/gate/verus-proof-check.py:473`：
```
CHEATING_MARKER = re.compile(r"\b(external_body|external_fn_specification|assume\w*|admit)\b|verifier\s*::\s*external\b|verifier\s*\(\s*external\s*\)")
```
`.claude/kb/decisions/13-验证路线.md:387`：
```
| D4（校验和位置） | 根链差分把 Merkle 从「要付的代价」翻转成验证杠杆；但形式验证里哈希是 uninterpreted function，**碰撞抗性完全在验证范围之外，不许读成「验了 Merkle」** |
```
（Verus 自己在 `--no-cheating` 下连 `uninterp spec fn` 也拒：`out/y1-probes.txt` 里 `y1_e_external_body [--no-cheating]` 一行。）

**冲突三　门禁起的 Verus 只带 vstd，core 的清单有三个别的依赖。**

`scripts/gate/verus-proof-check.py:10`：
```
直接起的 Verus 只带它自带的 vstd：crate 的 Cargo.toml 除 vstd 之外还有依赖，proof、vacuity、must-red-worlds 三格开跑之前判红。
```
冻结副本 `crates/singlefs-core/Cargo.toml:14`–`:17`：
```
libc = "0.2"
singlefs-format = { path = "../singlefs-format" }
# 取 journal 前缀的纯函数核（Verus 验的那一份），recovery::replay_journal 调它；compaction_plan 直接调用整理选段核、计划核，allocator 调用释放代比较核。
singlefs-replay-prefix = { path = "../singlefs-replay-prefix" }
```

**冲突四　回归测试的注释把「整理不改用户看得见的东西」当前提；协议层证出的是窗口移一格。**

`crates/singlefs-harness/tests/compaction_while_mounted_in_random_histories.rs:588`：
```
/// RefFS 翻译整理那一步（判决 G7：整理不改用户看得见的东西，不进操作序列）：起点（第一个文件）→ 可写挂载 → 整理到停 → 覆盖写，
```
`research/prompts/m4-exit5-relocation-proof-r2-sonnet-model/src/gc_ring_proofs.rs:469`–`:471`：
```
// 反过来：窗口满的时候，整理的一次发布使窗口少掉最旧的那个候选（用户看得见的变化：回退少一个落点）。
// 这条是第一轮攻方「整理改了回退能退回的内容集合」的精确表述：不是读到坏字节，是窗口移一格。
pub proof fn lemma_a_gc_checkpoint_with_a_full_window_drops_exactly_the_oldest_rollback_target(s: GcRingState, i: int)
```

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Y1 怎么接 | 规则没说 | `verus!` 宏与属性模式都能把 core 今天的小函数接上，产品构建 0 警告、5557 个 MIR 函数规范化后 0 行差；vstd 本来就在 core 的产品依赖闭包里。口子是类型波及、私有字段不变量、被验区域外的代码 Verus 不看（`y1_f`） |
| Y2 std 规格与作弊标记 | 冲突（冲突二） | 闭包 456 个 std 调用键 49% 按名字配上 vstd 规格（调用点 82%）；`assume_specification` / `external_body` 在 `--no-cheating` 下被 Verus 自己拒，连 `uninterp spec fn` 也拒，core 直验必须去掉这个旗子；登记式（集合相等 + 差分测试 + 必红世界）在演示里判得出写错的规格，Verus 自己判不出 |
| Y3 射程与分阶段 | 规则没说（另见冲突一） | 两个闭包并起来 1073 个 fn、24970 行：A 472 / B 190 / C 291 / D 120；第一阶段原型在 core 里直验两种做法各 13 verified、七行必红世界全红；总代价是推的（被验版本约 3.9 万–4.3 万行（A+B）到 11 万–12.5 万行（全验）），机器时间不是瓶颈 |
| Y4 判法与门禁 | 冲突（冲突一、冲突三，另有冲突二） | `verus-proof-check.py` 与登记表要改十一条，最硬的三处：多 crate 登记、`cargo verus` 替直接起 `verus`、信任清单替作弊标记；cargo 指纹不看 `--verify-*` 会拿到上一趟的结果 |
| Y5 第一轮挂起的三处 | 冲突（冲突四） | 新协议 13 verified、区间版 P4 9 verified，并入 `singlefs-replay-prefix` 副本 88 → 110，十二行必红世界全红；第一轮三个反例都被排除；G7 在含回退与抬 F 的历史上不是恒等，是窗口移一格，这条被证成引理 |
| Y6 罩不到的 | 一致（D13 已定项 14、已定项 4 射程） | 十五项，每项写了由什么罩；三项是本形新增的工作 |
| D13 已定项 5、13、17 | 一致 | core 加 vstd 不碰 checker 的依赖闭包；新 spec 并入同源性登记的一对；单元键按 core 里被验函数的闭包算 |

## 没做什么

- 不判别的腿的格（攻方的 Y1–Y6 作答）、不替主 agent 采纳或出判决；没读禁读清单里的文件（攻方腿这一轮的报告与模型目录）。
- 没起任何门禁阶段、没起 `scripts/gate/verus-proof-check.py`、没跑 checker 档测试、没起 GPU、没跑 `cargo test` 之外的重型；唯一跑过的 `cargo test` 是演示 crate `assume-demo` 的一个 `--exact` 测试，经内存包装。
- core 里接线后（`kernelmodule` 变体）行为等价**没验**：只 `cargo build` 过，没跑 core 的单元测试或 harness 用例。`inplace` 变体改了 `region_is_empty_by_the_records` 的函数体（`iter().any` 改循环）与三个方法的 `const` 写法，同样没跑用例。
- Y3 的发布装配以后各阶段没有原型；`relocation_destinations_are_acceptable` 的受保护集接线没做。
- Y4 的信任清单、自动必红世界、被验区域外调用点那一格只是设计，没写成门禁代码；`--output-json` 逐函数项只读过一次，没改门禁的逐项判定。
- 区间 P4（int）与 core 的可执行版（u64）之间没有接成一条证明链。
- 没试 `exec_spec_verified!` 偏差的原因；没试 `Box<dyn Fn>`、切片模式的绕法。
- 云端攻方的角度没替它想：这一形下「分阶段里某一阶段的验收在 core 里有一条绕过它而改变抽象状态的路径」，我只在 Y1/Y4 里写了被验区域外的调用点这一条，其它路径（比如 `cfg(test)` 里的构造、`unsafe` 打洞）没系统找。
- 冻结副本里 `mount.rs`、`mounted_session.rs`、`compaction_plan.rs` 与主工作区此刻不同：我的数全按冻结副本量（回退函数在主工作区后移 4 行）；GPU 垃圾回收会话后来改的部分没进这些数。

## 要主 agent 跑的命令

1. 整份连跑（我在 240 秒的前台上限内跑不完；本机、约 7–10 分钟）：

   ```
   bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-exit5-relocation-proof-r2-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --work <新的草稿目录>
   ```
   内存上限 8G、线程由脚本里的 `capped.sh 31` 定；看末行 `name=rerun_result mismatches=0`。
2. 判决要引的数照 `.claude/rules/three-way-inference.md` 在入库装置上重做：Y2 的覆盖数（`tools/std_coverage.py`）、Y3 的分类（`tools/closure_classes.py`）、Y5 的 verified 数，都是副本上量的。

## 交回前复核与清理

- 四段复跑全部在新目录里比过一次，全同（上面「复跑」一节的表）；整份连跑没做（前台上限）。
- 交回前跑 `python3 scripts/agents/cite-check.py` 点名本报告，根给主工作区与冻结副本，末行原样：`✓ 核了 0 处引文、65 处裸行号，对上 0 处，没判 65 处（其中裸行号没有引文可比 39 处；逐处列在上面）`，没有 ✗ 行。
- 草稿目录删了（`du -sh` 记的大小）：编译目录 3.2G；冻结副本拷贝 39M；最小工作区拷贝 tree-b / m1 / m2 / m3 / m4 / s1 各 4.4M、tree-y5 412K；演示 crate 的运行目录 539M；复跑用的工作目录 39M + 4.1M + 4.4M + 140K；探针、补丁、辅助脚本的草稿副本（模型目录里都有）。留着 `out/`（4.9M，量的原样输出）与 `progress.md`。
- 内存包装排队碰到过几次（本机此刻有别的会话在跑崩溃放量与编译，包装最长让我排了 109 秒），都等过去了；负载最高时 `uptime` 报 load average 96，那一趟复跑被我自己的 238 秒超时截断过一次，重来的一趟正常。
