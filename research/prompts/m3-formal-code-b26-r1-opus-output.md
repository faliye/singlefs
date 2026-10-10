# 第 26 批代码轮第一轮：云端攻方（Opus）报告（2026-10-05）

<!-- doc-lint:not-numbers J1 J2 J3 F3 A2 B3 C2 C3 B4 M7a M7d M7f G1 G2 G3 G4 G5 G6 B1 Y2 Y3 -->

## 复跑命令与模型文件

在主仓根：

```
REPO=$PWD FROZEN=/tmp/claude-1000/m3-formal-code-b26-r1/tree WORK=<草稿目录> \
  bash research/prompts/m3-formal-code-b26-r1-opus-model/tools/rerun.sh
```

段名可以单给（`build f3 frac seglen quick r1 r3 r4a r4b r4c r5 r6 r7 r8 r9 mut mutb compare`），`compare` 去掉耗时与 `S2EX` 例子行之后逐份比 `out/`。单段挂钟：r7 约 231 秒、r8 约 200 秒、r4a 约 188 秒，贴着前台 240 秒，复跑放后台起。我在 `/tmp/claude-1000/m3-formal-code-b26-r1-opus/rerun-test/` 用这份脚本从头复跑了 build f3 frac quick r3 mut mutb，compare 10 份 SAME、0 份 DIFF；r1、r4–r9、seglen 只跑过攻方那一趟。

模型目录 `research/prompts/m3-formal-code-b26-r1-opus-model/` 每个文件的 sha256：

```
00f5358d92544a2018587da1d08236118185dc9654a1da7668584f2089f8efd7  out/f3.txt
fdfc1454e0277acfd62c32f89f49e9bf301a1c5b695175c33fe8054ed1b2967d  out/frac.txt
d11ec33b3948ac1e1d3869bb06b5ca2323320403d7804140272bc746c90eb6f5  out/mutations.txt
4cab2a2e50dcbc62ae0d0235d285c46fcbe6fd9e778b0c39e849d8d4072d56d1  out/mutB1-mkfs-mount-pwu-mount-pwu.txt
8f7542e641c9c2f2129a22cce672a1c923c9e1e920762751002bc94b093c481b  out/quick-4gib-cmp.txt
70c6fa29cbb72d2d20e2b56a88b6006da285cbcf45fbf44cef35a3cb594449c6  out/quick-4gib.txt
6db99f67954c3e73442b2b238f647d4e8c05de1ebb99379b36c1199804f77f0b  out/r1-file-zeros.txt
ff5604a7f20368b471ddc21f4c809c3998b16bf415d3fcd2800fc2b5ab900d25  out/r3-mkfs-mount_pwu_mount_pwu_mount.txt
769a5336e93f2a3a24f30a4d8fec297681a6587fa9a7790d73dac3302591d346  out/r3-mkfs-mount_pwu_mount_pwu.txt
51cfa5e25a37ffe92c8621905d9eacc5cd4141a28495ead4a9a2b6913ea143bc  out/r3-mkfs-mount_pwu_zeros_pwu.txt
30b31e319b11bf405ace2e9301cf5fd44fcaefaeac0be9d6937022db752d2853  out/r3-mkfs-mount_zeros.txt
11c2a130c7dc023eaeb36fbab9b3e3a5894fc609c2116256b756b02eaf5f2913  out/r4-file-p1-ow_s5_rf_1_mount.txt
26c526f66d465bcdf6788ecc7b53aefa4973f0fb96d3c08cc6481e54a3d37219  out/r4-file-p1-unlink_mount.txt
9714148392483959c617217a1f184641ae0d88f5dcd2f8a52b2c0e9265d13a99  out/r4-file-p2-ow_s5_ow_s6_rbb_0_mount.txt
8e2c1fadd0c2525c442b08f1dba60e5ebd04a6462cc664d06e8dd4ac6c9b73e9  out/r5-file-p0-zeros_zeros.txt
3f8c40e01bb04b959f9f12235f3ef99e5706a520ebc3cc122d56b9794f5bebaf  out/r6-file-p0-mount_mount.txt
8d3707882b7cf3480221f801ad55c4a16263f2d135e2c496e42dbd068d44c1e6  out/r7-file-p2-rb1-mount.txt
fd38817eb983a1b5eff7b19ca53351b6137f99271175f162d2f1c3b78038faeb  out/r8-file-p1-unlink-inode-mount.txt
39866b9d23fd93cc9ccffc23801cd0f623fe3af62536ad65b162954eb1291c17  out/r9-unmount-ow1.txt
c58265a9a443dfb67240f90f22f1ccbb376cf02704648f6740551113315d5e35  out/seglen.txt
46d291b96bc8e6b06ce56294f79fd8fc606216f608959c6d65116c567d77f0fe  probe/Cargo.toml
f15e600ac9c9730dd337056dc1f0b59f90bc321c1b37fcd095d62c2e38685850  probe/src/common.rs
2cdecddd8b93d7c9101e6a7056d5f8ac4f8cbc80572fa8e45f94406c59a01033  probe/src/custom.rs
6e9ad5232eb382e3e81f7615f4fbb45f053e662f7e8a06af71c1a1258f487cc1  probe/src/f3.rs
018acf8c1748f8713d81c23403de4bf1d6904aa820b563ff39b386ac84cd8a31  probe/src/l0.rs
58bb23c89c2ca9d70db1a35c3a4a34dc019486b42bad9cd85ccb0856994d5ad7  probe/src/main.rs
b16f849efcd5348768c0ee7dae48f96312976df527479a17405123d3cf3b5ce7  probe/src/stage2.rs
435a86e58bda37e74dfdae8a89dcefa39f183d588a77488586fef7b768657cd7  probe/src/unmount.rs
447a3091ff7f3194143e019725d67a223215de678104b708b8ebbad024399fac  tools/apply_mut_b.py
640ec0ba2e30d4e18ce5b147daafc0fd06df8ad00076c2ffe206021af20d65a4  tools/cmp_quick.py
86351d7974ea48a25af8a0782ea8849dde78a8da6a05931b645366cb32cb4ca7  tools/mutate_rows.py
75a7ea9ec517696c1e1af3b199477637bad3d712e84e167297efa4f3e94235fb  tools/rerun.sh
fbf6e5ee4a9ead0ed2e1cc50aee7ab854cbf8056ef9742b455e9803f23ef1245  tools/rows-c664-judge.tsv
62c99d9085a08ddd83135a97f9fd780b92083127c34f51052981e51bab49eac7  tools/rows-replaced-and-mutA.tsv
```

## 各格判定一览

| 格 | 攻的是 | 判定 | 一句话 |
|---|---|---|---|
| J1-a | I-9.16 ③ ④、I-2.4 根槽 428..457、I-5.4 新格、I-5.5 部分没判：合法历史误红 | 没打中（一次抽样） | 14 段历史 354,278 个层 0 状态（每段全量展开），四条 0 违例 |
| J1-b | 挂载透明新加的几行在合法历史（多实例、回退、抬 F、unlink、正常卸载后再挂、注入读故障）误红 | 没打中（一次抽样） | 139,513 次第一次崩溃挂载、7,531,673 个二次崩溃镜像，0 行判红、0 次拒可写 |
| J1-c | 运行配置「R2 = 内置默认」放过坏实现 | **打中** | 判官的期望值取自被测 core 的私有常量；副本上把 T_time 默认 5 改 4，挂载透明那份用例 45 条全绿（同一改坏由系统配置字节钉的用例红） |
| J1-d | 字节区间字段表的归类与区间 | 区间没打中；**表外有洞**（推的） | 三张表与 kb 字段表逐行对上；根槽 [457, 根槽宽)、系统配置槽 [489, 4096) 不在任何表里、也没人判 |
| J1-e | journal_tail、journal 实例代号两行放过坏实现 | 推的，没造 | 二次崩溃只判区间：取号写的 tail 写成 R1 所选那块盘的值而不是全池见证值，落在区间里不红 |
| J1-f | I-5.5「部分没判」与 I-5.4 挂法改层 0 计数 | 没打中 | 层 0 把部分没判记进不适用；模型上新池新建文件快档 46 个状态 69 条计数逐条等于 26-2 #8 的钉值 |
| J2-a | 注入读故障的态上新拒因复算为假不算不合法 | **打中** | 放宽挂在「历史里有一步在读故障下发过写」上，而崩溃注入第二、三截被判的那次挂载本身不带故障，两遍读之间没有瞬时读错可言：这条放宽在崩溃注入里只会遮；zeros 那组比重 29% 的段落在它的射程里 |
| J2-b | 新拒因各自的合法判法 | **打中** | 没有 C670 同形的那一条：无故障的崩溃态上某块盘两槽一份都不自证，拒得「合法」、谁也不问这个盘面怎么来的；MUT-B1 下 90 次拒全判合法、checker 0 违例 |
| J2-c | N-配置 去掉「有盘一份都没有就按真」那一支放过不合法的拒 | 没打中 | 复算只变严；那一形改由新拒因判 |
| J2-d | 二次崩溃镜像在注入态下照旧判挂载透明会不会误红 | 没打中误红（一次抽样） | 34,772 次注入态挂载、1,848,852 个注入态二次崩溃镜像 0 行判红；但与 D13 射程字面「带故障的状态不在射程」不一致 |
| J2-e | 新步与新比重改已有取样 | 没打中 | 新步只进新比重；`GenerationWeights::NAMED` 只给复现入口与单测用 |
| J2-f | F3-e 三条在更长的段、别的种子基上 | **打中** | 两类有害改坏 F3-c/d/e 七条全绿：指纹只折段内前（或末）32 个写；历史种子只留第 0 位与第 63 位、或对 3 取余 |
| J2-g | 替换的 5 行与删掉的 RR8 | 没打中 | 副本上 5 行各自点名的用例都红；接替 RR8 的 C664 判法 5 行也都红 |
| J3-a | #8 那 69 条钉值的出处 | 没打中 | 合入 26-3、26-1r 之后的冻结副本上，模型复现 69/69 |
| J3-b | #7 选择器 `<= 5`、计数 9 | 没打中（读码） | 只多展开了那一段 5 个写，24 个写那一段照旧不展开；3² = 9 推得出 |
| J3-c | #2–4 按盘放行清零 | 没打中（读码） | 放行只在那块盘还没发过系统配置槽写时，基线是空白镜像，清零不改见证值 |
| J3-d | C686 断言的落点、层级、出生序号 | 没打中 | 与 D3 已定项 10 ⑤ 表 t5–t9 逐格相同 |
| J3-e | #6 门槛、#9 打印行哈希 | 没打中（推的） | #9 的进度文件哈希是占位字串，checker 档跑到那一核必红（已知、记录里写了待重录） |

「一次抽样」的格照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」办：只算这一次没打中。

## 一、装置与取样

探针 `probe/`（Rust，路径依赖冻结副本里的 crates，只读导入、不改它们）：

- 层 0：`crates/singlefs-checker-tier/src/crash.rs` 的 `enumerate_layer0_selecting_versions_observing_each_state`，展开条件恒真（每一段都 `EveryProperSubset`，原地覆写取三态），每个状态都跑池级 checker 与两遍恢复。固定前缀（mkfs、起点、前 k 步）施加进基线、不枚举，k 取「起点之后的状态数不超过上限」的最小值；状态数先用 `layer0_state_count_with_torn_in_place_overwrites` 算闭式，超过 10⁶ 的不跑。
- 第二截：观察者记下每个状态的持久集合，枚举完之后按「每 n 个取一个」交给 `crash_injection::mount_and_publish_once_on_the_crash_image`（与崩溃注入同一个入口），做成的判挂载透明「只挂载」「挂载加一次发布」两种；被拒的照 `judge_writable_mount_refusal_of_a_crash_state` 判，注没注入两种标记各判一次。
- 二次崩溃：挂载那一段写表逐段摆，段内真子集不超过上限（16，r3 是 64）的全摆，多的按种子抽；每个二次崩溃镜像 `read_recovery_fields` 之后判挂载透明。挂载末尾清扫删了行的不判（我没抄崩溃注入里把删的行放回去那一步）。
- 注入标记照 `StreamMarks::read_faults_in_a_crash_state` 的算法：崩溃窗口末尾那个写的录制流下标不小于某个「读故障下发过东西」的步的起点，就是注入过。

为什么不用随机历史：BROAD 比重 3 步的历史在 384 槽小盘上已经 1,048,620～3,145,762 个层 0 状态（第一次覆盖写那一段 20 个写，2²⁰ − 1 个）。所以用户动作改成写死步序逐种扫，每段历史的状态数压在 10⁶ 以内，一个不落。

| 段 | 起点 / 固定前缀 / 枚举的步序 | 层 0 状态 | 第二截取样 | 第一次崩溃挂载 | 二次崩溃镜像 |
|---|---|---|---|---|---|
| r1 | 第一个文件之后 / 无 / zeros | 20,511 | 每个 | 20,511（全注入） | 1,099,500 |
| r3 ×4 | mkfs 之后 / 无 / mount,zeros；mount,pwu,zeros,pwu；mount,pwu,mount,pwu；mount,pwu,mount,pwu,mount | 320、344、344、631 | 每个 | 1,639（588 注入） | 132,491 |
| r4a | 第一个文件之后 / ow / rf:1,mount | 24,618 | 每个 | 24,618 | 1,583,574 |
| r4b | 第一个文件之后 / unlink / mount | 20,511 | 每个 | 20,511 | 1,005,315 |
| r4c | 第一个文件之后 / ow,ow / rbb:0,mount | 20,511 | 每个 | 20,511 | 1,005,315 |
| r5 | 第一个文件之后 / 无 / zeros,zeros | 41,021 | 每 3 个 | 13,673（全注入） | 701,556 |
| r6 | 第一个文件之后 / 无 / mount,mount | 41,021 | 每 3 个 | 13,673 | 701,556 |
| r7 | 第一个文件之后 / ow,ow / rb:1,mount | 86,058 | 每 10 个 | 8,605 | 431,098 |
| r8 | 第一个文件之后 / unlink / inode,mount | 86,058 | 每 25 个 | 3,442 | 172,430 |
| r9 | 自建流：mkfs、挂载、第一个文件、覆盖写 1 次 / 这些 / `unmount` | 12,330 | 每个 | 12,330 | 698,838 |

zeros 是新步「每块盘最新那一槽系统配置读回全 0 再可写挂载」，rb:1 是回退到根环第二新的根，rbb:0 是回退到根环之外（被拒、没写），rf:1 抬 F。r9 的最后一个状态就是「正常卸载做完、再挂」的盘。层 0 状态合计 354,278 个（另有 J3-a 的快档 46 个、J2-b 的 MUT-B1 488 个、复跑核对 2,173 个），每段都不到 10⁶，全部原型合计不到 10⁷。挂钟按先跑一小段估（r1 采样 1/50 那一趟 48 秒），每段控制在 240 秒以内；缩的只有第二截的取样（r5–r8）与每段二次崩溃的上限，第一次崩溃的状态一个不落。

## 二、J1　26-3

### J1-a、J1-b　合法历史上的误红：没打中

上表 14 段，每段的 checker 一栏都是 `checker_violated={}`，四条目标的 评估/违例/不适用（`out/` 各份 `L0` 行的 `targets=`）里违例恒 0；第二截一栏没有一行 `MT-VIOLATED`，也没有一次拒可写（各份的 `S2` 计数）。r4b、r8 的 `oracle_violations` 不是 0，那是我给 oracle 的版本表只收带文件的版本、unlink 之后的版本进不去，与 checker、挂载透明无关。

覆盖到的盘面形状：树表 0 条而写过行的一版（分配记录树根非零）与实例 2、3 的取号（r3）；带树一版上回退、抬 F、unlink 续做、两次连续可写挂载（r4、r6–r8）；卸载抬 F 与带卸载记号的空发布及卸载做完的盘（r9）；zeros 那一步取号写落在被藏的那一槽上之后的全部崩溃态（r1、r3、r5）。

推翻条件：在这些步序之外的一段合法历史上（尤其树表条目宽、第一个文件在实例 ≥ 2 的一版上发、清扫删行的那一次挂载），这几条有一条判红而 core 照常挂载。

### J1-c　运行配置「R2 = 内置默认」：打中（判官与被测实现共用默认值）

判官的期望值不是从条款来的，是从被测的 core 取的：`crates/singlefs-harness/src/mount_transparency.rs:1359`：「let defaults = SystemRuntimeConfiguration;」，判的那一句 `crates/singlefs-harness/src/mount_transparency.rs:1919`：「observed == Some(built_in.as_slice()),」；而 `SystemRuntimeConfiguration` 的三个值是 core 自己的私有常量 `crates/singlefs-core/src/system_configuration.rs:85`：「const TIME_THRESHOLD_SECONDS: u32 = 5;」。条款里的值在 `.claude/kb/decisions/16-发布语义.md:140`：「`T_time` 默认值 5 s」与 `.claude/kb/layout/01-first-txn.md:149`：「| 可调值 | T_time | 4 | 5 秒 |」。

副本上量过（`tools/mutate_rows.py` 跑 `tools/rows-replaced-and-mutA.tsv` 末两行，`out/mutations.txt` 原样）：

```
MUT opus MUT-A：T_time 内置默认 5 → 4（挂载透明「R2 = 内置默认」那一行） => 绿（退出码 0；test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s）
MUT opus MUT-A2：同一改坏，系统配置字节钉 => 红（退出码 101；test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s）
```

四句：① 它分辨的是「期望从哪来」：按条款写死 36 字节的判法在这一改坏上红，现在这一种不红。② 判官做决定那一刻看得到条款值（5 s、2 GiB、三水位 0 是常量），不是观测不到。③ 满足的是判决 Y2 那一格「运行配置 36 字节照正推『R2 = 内置默认』」里「内置默认」那个词：实现把它读成「core 此刻的默认」，于是这一行只判得出「R2 的 36 字节不是 `to_slot` 写的」，判不出默认值本身写错。④ 改法见第五节 G1：在 harness 里照 D16 已定项 5 与 D26 写死期望字节（推的、没实现）；不改的话，仓里由 `crates/singlefs-harness/tests/system_configuration_mutability_classes.rs:95`：「fn splitting_the_system_configuration_into_four_mutability_classes_changes_no_byte_on_disk() {」那一份的 sha 钉兜着（量过，上面 MUT-A2）。分量：轻，挂载透明这一行不独立，别处有兜底。

### J1-d　字节区间字段表：区间对得上，表外有两段没人判（推的）

三张表逐行对了 kb：根记录 15 行盖满 [0, 457)，系统配置 46 行与 `.claude/kb/layout/01-first-txn.md` 的系统配置字段表逐行同宽同序（magic 4 起到回退下界 F 止，合计 489），树表条目 9 段盖满 [0, 200)。归类上我没找到放过合法历史、或把该比的模掉的一格。

表外的两段：根槽 [457, 根槽宽) 与系统配置槽 [489, 4096)。core 写 0（`ByteWriter` 起始全 0），自证校验和罩着它们，但 `parse_slot` 不看、池级 checker 只拿 `ROOT_RECORD_BYTES` 判槽宽（`grep -n 'SYSTEM_CONFIGURATION_BYTES\|ROOT_RECORD_BYTES' crates/singlefs-checker/src/*.rs` 只命中槽宽那几处）、挂载透明只比 [0, 457) 与 [0, 489)。一个往这两段写非 0（重封校验和）的实现，新加的 I-2.4 根槽那一格与挂载透明都不红。条款这边 `.claude/kb/decisions/22-单元原子性怎么合成.md:154`：「512 槽内余量 55——槽宽是运行时探测值，余量给将来字段。」只说留给将来，没说第一版恒 0，所以这是条款缺口，不是 26-3 的错。没造坏镜像，推的。

### J1-f　I-5.5 部分没判与 I-5.4 挂法对层 0 计数：没打中

读码：层 0 把部分没判与不适用记进同一格，`crates/singlefs-checker-tier/src/crash.rs:1493`：「InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => {」，所以 I-5.5 从不适用改报部分没判不改层 0 的「评估/违例/不适用」三个数；I-5.4 新格只在树表非空的根上判，那些根本来就走得到分配记录、本来就评估过。

量过：探针 `quick` 段用起点「mkfs 同一个进程里取号、暖机、发第一个文件」（与 `build_pool` 同一串）、前缀只有 mkfs、甲二展开，得到 `QUICK 4gib segments=[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2] states=46 root_persisted=9 verification_ran=6`（`out/quick-4gib.txt`），69 条计数与 26-2 #8 钉的值逐条比（`tools/cmp_quick.py`，`out/quick-4gib-cmp.txt` 69 行都是 ok；I-3.10 的期望由用例现算，我只抄出 12）。这一趟跑在合入 26-3、26-1r 之后的冻结副本上，所以 26-3 的 I-5.4 挂法与部分没判没有改这条流上的计数。

GPU：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 I-9.16 那一处（`w_judge_tree_table_entries_ordering`）没有 ③ ④，根槽 428..457 与带树一版分配记录树根全零 GPU 也不判；26-3 交回已写「GPU 没改」、记录第 985 行交 26-5。我读了 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 里重封字段的那些流，没有一条改到种类码、根槽 428..457 或带树一版的根记录分配记录树根，所以这份对拍用例不会因为 26-3 多出 CPU 红（推的，GPU 没跑）。

## 三、J2　26-1r

### J2-a　注入读故障的态上「新拒因复算为假不算不合法」：打中（放宽挂错了判别子）

放宽在 `crates/singlefs-harness/src/mount_transparency.rs:2625`：「) => conditions_not_met.retain(|condition| {」，条件是第 2623–2624 行的（新拒因，`ReadFaultsInTheCrashState::Injected`）。它的理由写在那份用例的文件头 `crates/singlefs-harness/tests/writable_mount_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_judged_apart_from_the_witness_criterion.rs:9`：「崩溃态注入过读故障时，两遍读之间的瞬时读错在崩溃镜像上复算不出」，出处是 C664 的还清条件 `.claude/kb/checks-owed.md:522`：「带读故障的档里造一个取号那一刻瞬时读错的态，合成一个拒因时判不合法、分开后判合法」。那份用例的拒是自己拼的：瞬时读错发生在被判的那次挂载里。

崩溃注入里交进来的标记不是这个意思。第二截的标记是 `crates/singlefs-checker-tier/src/crash_injection.rs:1504`：「let read_faults_in_the_crash_state = marks.read_faults_in_a_crash_state(」，判的是「崩溃窗口末尾之前，历史里有没有一步在读故障下发过写」，交给判官的是 `crates/singlefs-checker-tier/src/crash_injection.rs:1564`：「read_faults: read_faults_in_the_crash_state,」。而被判的那次挂载是在崩溃镜像的拷贝上起的、不带故障层：`crates/singlefs-checker-tier/src/crash_injection.rs:2057`：「let mut device = SparseBlockDevice::new(」。这次挂载读的是一份静止的内存盘，读阶段与取号读两遍，读到的字节相同，「两遍读之间的瞬时读错」在这里不会发生。所以在崩溃注入里，这一拒复算为假只有一个解释：实现拒错了；而只要这段历史更早有过一步 zeros，它就被判合法。

射程量过（`out/frac.txt`，起点之后的段）：zeros 那组比重 24 步、种子 0–79，384 槽 80 段历史里 42 段的 zeros 真往盘上发过写，7,916 个段里 2,292 个（29%）的崩溃态被认成注入过；256 槽 8,672 个里 2,536 个。今天它遮住的是 0 次：r1、r3、r5 的 34,772 次注入态挂载一次都没被拒（`S2 refused` 计数没有）。

四句：① 分辨：照「被判的这次挂载自己在不在读故障下」判，与照「历史有没有过读故障步」判，在 zeros 之后的每个崩溃态上给出不同的判定。② 判官看得到：崩溃注入第二、三截的挂载是它自己起的，带没带故障层是写死的，不是观测不到。③ 满足的分句：D13 的射程句 `.claude/kb/decisions/13-验证路线.md:57`：「带故障的状态不在射程」管的是带故障的状态；C664 的还清条件管的是「取号那一刻瞬时读错」的那次挂载。崩溃注入把前者当成后者用了，是「打中归错了判据」那一型。④ 改法：G3（第五节）。

### J2-b　新拒因的合法判法少一条 C670 同形的：打中（坏实现放过）

N-配置 那一拒有 C670：无读故障的崩溃态上判据为真，说明这份盘面本不该出现，判红。条款写的就是这个缺口：`.claude/kb/decisions/13-验证路线.md:172`：「为真就判合法（挂载透明的射程只管不带故障的状态，已定项 1），没有判官问」。新拒因分出来之后没有对应的那一条：无读故障的崩溃态上「某块盘两槽一份自证过的都没有」复算为真，判合法，谁也不问这块盘怎么两槽一起坏的；池级 checker 只要全池有一块盘有就不红，`crates/singlefs-checker/src/image.rs:1122`：「单盘全废只读恢复照样挂得上，不判违例，」。

副本上量过：MUT-B1（`tools/apply_mut_b.py`：盘 0 每次写系统配置槽之后，在同一段里把另一槽清成 0）。探针在 `mkfs mount,pwu,mount,pwu` 上全量 488 个状态（`out/mutB1-mkfs-mount-pwu-mount-pwu.txt`）：

```
L0 custom/mkfs/unit-area-384/p0/mount,pwu,mount,pwu seed=0 states=488 closed_form=488 segments=21 faulted_steps=[] oracle_violations=0 root_without_record=0 checker_violated={} …
S2 refusal-verdict/clean/judged-as-NoneInjected/Legal(SystemConfigurationUnreadableAfterTheReadStage)=90
S2 refused/clean/Some(SystemConfigurationUnreadableAfterTheReadStage)=90
S2 MT-VIOLATED/clean/second/SystemConfigurationJournalTail=60
S2 MT-VIOLATED/clean/second/SystemConfigurationJournalInstance=23
```

checker 0 违例、oracle 0、90 次拒全判合法；唯一的红是 26-3 新加的二次崩溃 journal_tail、journal 实例代号两行（R2 落到盘 1 那一份较旧的槽上）。崩溃注入每个挂载阶段只按种子摆一个二次崩溃，摆没摆到这几个是运气（推的，崩溃注入没跑）。

四句：① 分辨：有这一条判法时这 90 次是红，没有时合法。② 判官看得到：没注入读故障的标记与复算都在手里。③ 分句：D13 第 172 行那条欠账的同形；这一形在 26-1r 之前并在 N-配置 里，当时 U6 的复算按真、C670 照常比，也不红，所以缺口不是 26-1r 引入的，是 C664 把拒因分开之后才露成单独一格。④ 改法：G4（第五节）。

### J2-c　N-配置 去掉「有盘一份都没有就按真」那一支：没打中

去掉之后 `crates/singlefs-harness/src/mount_transparency.rs:2385`：「if system_configuration_witness_undecidable_on_the_crash_image(&slots_of_every_device) {」只剩判不出那一支按真，复算为真的情形只少不多，判不合法的只会更多；被去掉的那一形改由新拒因的复算接住。core 那边同一形也改报新拒因（判据 N-配置 不再含它）。实现若仍报 N-配置，判官复算为假、判不合法——这正是 C664 那一行要的。

### J2-d　二次崩溃镜像在注入态下照旧判：没打中误红，但与射程字面不一致

第一截按标记跳过（`crates/singlefs-checker-tier/src/crash_injection.rs:1586`：「ReadFaultsInTheCrashState::NoneInjected => judge_mount_transparency_on_the_crash_image(」），二次崩溃那一截不看标记（`crates/singlefs-checker-tier/src/crash_injection.rs:3335`：「violations.extend(mount_transparency_violations_of_a_second_crash(」）。量过：34,772 次注入态挂载、1,848,852 个注入态二次崩溃镜像，挂载透明 0 行判红（r1、r3 两段 zeros、r5）。zeros 之后的盘面形状（最新那一槽被取号写盖掉、tail 是被藏之前的见证值）在不带故障的历史里也摆得出（取号写落了一块盘、轮换没落；推的，没逐形对过），判法在这上面成立不奇怪。这一格的问题只在口径：D13 第 57 行说带故障的状态不在射程，代码在注入态上照判二次崩溃。今天不误红，所以不急；改法 G5。

### J2-e　新步与新比重会不会改已有取样：没打中

新步只出现在新那一组比重的三张表里；`crates/singlefs-harness/src/history.rs:1035`：「pub const NAMED: [GenerationWeights; NamedGenerationWeights::ALL.len()] = {」多了一组，但全仓用它的只有复现入口按名字找比重与 history.rs 自己的单测（`grep -rn 'GenerationWeights::NAMED\|NamedGenerationWeights::ALL' --include=*.rs crates/` 只命中 history.rs），没有哪个战役按下标轮流取比重，所以已有各组的种子不会换到别的比重上。`draw_operation` 的新臂不抽随机数，已有各组的步序不变由指纹用例钉着。

### J2-f　F3-e 三条在更长的段、别的种子基上：打中（两类有害改坏七条全绿）

种子是历史种子、盐、段号、段内子集指纹四样异或（`crates/singlefs-checker-tier/src/crash_injection.rs:3085`：「^ fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment),」），指纹折段内每一个写（`crates/singlefs-checker-tier/src/crash_injection.rs:3062`：「fn fingerprint_of_the_persisted_subset(persisted_within_the_segment: &[bool]) -> u64 {」）。F3-e 的 B3 用长 32 的段（`crates/singlefs-checker-tier/src/crash_injection.rs:4868`：「const WRITES_OF_THE_SEGMENT: usize = 32;」），C2 用三个历史种子（`crates/singlefs-checker-tier/src/crash_injection.rs:4912`：「[HistorySeed(0), HistorySeed(1), HistorySeed(1 << 63)];」）。b24 判决记下的就是这一问：`research/prompts/m3-formal-code-b24-r1-main-verification.md:39`：「它自己的取样（段号 2^k、段长 32、历史种子 2^63）对更长的历史、更多步数、别的种子基罩不罩得住没人攻过」。

真历史里有比 32 长的段（`out/seglen.txt`，起点之后）：

```
SEGLEN several-leaves unit-area-384 steps=24 seeds=0..40 segments_after_the_starting_point=2916 longest=48 (seed 12) over_32=6
SEGLEN several-leaves 4gib steps=24 seeds=0..40 segments_after_the_starting_point=3178 longest=626 (seed 14) over_32=85
```

模型（`probe/src/f3.rs` 照抄两个不公开函数的式子；入库的 F3-c、F3-d 的 A/B/C、F3-e 的 A2/B3/C2 七条逐条算；`out/f3.txt` 原样）：

```
F3 Shipped: F3-c=绿 F3-d-A=绿 F3-d-B=绿 F3-d-C=绿 F3-e-A2=绿 F3-e-B3=绿 F3-e-C2=绿 | 长48扣一两个写 1176/1176 不同 | 长626扣一两个写 196251/196251 不同 | 战役种子基+0..80、段16长12 327600/327600 不同
F3 FoldFirst32: F3-c=绿 F3-d-A=绿 F3-d-B=绿 F3-d-C=绿 F3-e-A2=绿 F3-e-B3=绿 F3-e-C2=绿 | 长48扣一两个写 529/1176 不同 | 长626扣一两个写 529/196251 不同 | 战役种子基+0..80、段16长12 327600/327600 不同
F3 FoldLast32: F3-c=绿 F3-d-A=绿 F3-d-B=绿 F3-d-C=绿 F3-e-A2=绿 F3-e-B3=绿 F3-e-C2=绿 | 长48扣一两个写 529/1176 不同 | 长626扣一两个写 529/196251 不同 | 战役种子基+0..80、段16长12 327600/327600 不同
F3 SeedBitsZeroAndSixtyThree: F3-c=绿 F3-d-A=绿 F3-d-B=绿 F3-d-C=绿 F3-e-A2=绿 F3-e-B3=绿 F3-e-C2=绿 | 长48扣一两个写 1176/1176 不同 | 长626扣一两个写 196251/196251 不同 | 战役种子基+0..80、段16长12 8190/327600 不同
F3 SeedLowByteAndBitSixtyThree: F3-c=绿 F3-d-A=绿 F3-d-B=绿 F3-d-C=绿 F3-e-A2=绿 F3-e-B3=绿 F3-e-C2=绿 | 长48扣一两个写 1176/1176 不同 | 长626扣一两个写 196251/196251 不同 | 战役种子基+0..80、段16长12 327600/327600 不同
F3 SeedModuloThree: F3-c=绿 F3-d-A=绿 F3-d-B=绿 F3-d-C=绿 F3-e-A2=绿 F3-e-B3=绿 F3-e-C2=绿 | 长48扣一两个写 1176/1176 不同 | 长626扣一两个写 196251/196251 不同 | 战役种子基+0..80、段16长12 12285/327600 不同
F3 M7aFoldLast12: F3-c=绿 F3-d-A=绿 F3-d-B=绿 F3-d-C=绿 F3-e-A2=绿 F3-e-B3=红 F3-e-C2=绿 | 长48扣一两个写 79/1176 不同 | 长626扣一两个写 79/196251 不同 | 战役种子基+0..80、段16长12 327600/327600 不同
F3 M7dSegmentLowSixBits: F3-c=绿 F3-d-A=绿 F3-d-B=绿 F3-d-C=绿 F3-e-A2=红 F3-e-B3=绿 F3-e-C2=绿 | 长48扣一两个写 1176/1176 不同 | 长626扣一两个写 196251/196251 不同 | 战役种子基+0..80、段16长12 327600/327600 不同
F3 M7fSeedLowestBit: F3-c=绿 F3-d-A=绿 F3-d-B=绿 F3-d-C=绿 F3-e-A2=绿 F3-e-B3=绿 F3-e-C2=红 | 长48扣一两个写 1176/1176 不同 | 长626扣一两个写 196251/196251 不同 | 战役种子基+0..80、段16长12 8190/327600 不同
```

只折前 32 个或末 32 个写：七条全绿，长 48 的段扣一两个写 1,176 个第一次崩溃态只剩 529 个不同种子，长 626 的 196,251 个只剩 529 个。历史种子只留第 0 位与第 63 位、或对 3 取余：七条全绿，战役种子基加 0..80、段 16 长 12 的 327,600 个态只剩 8,190 个或 12,285 个。入库的三行变异（M7a、M7d、M7f）照旧各被一条抓住。四句：① 分辨入库式子与改坏；② 不涉及判官观测；③ 打中 b24 判决第三节第 3 条那笔账自己写的那一问；④ 改法 G6（第五节），模型上量过：

```
F3PROPOSED Shipped: C3=绿 B4=绿
F3PROPOSED FoldFirst32: C3=绿 B4=红
F3PROPOSED FoldLast32: C3=绿 B4=红
F3PROPOSED SeedBitsZeroAndSixtyThree: C3=红 B4=绿
F3PROPOSED SeedLowByteAndBitSixtyThree: C3=红 B4=绿
F3PROPOSED SeedModuloThree: C3=红 B4=绿
F3PROPOSED M7aFoldLast12: C3=绿 B4=红
F3PROPOSED M7dSegmentLowSixBits: C3=绿 B4=绿
F3PROPOSED M7fSeedLowestBit: C3=红 B4=绿
```

### J2-g　替换的 5 行与删掉的 RR8：没打中

副本上逐行改坏、跑那一行点名的 harness 档用例（`tools/mutate_rows.py`，`out/mutations.txt` 原样）：

```
MUT C554 乙：判不出（两条记录都读不出）按判据为假 => 红（退出码 101；test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 7.54s）
MUT C579（用户 2026-09-27 定：乙判不出按真）：乙把「判不出」按假处置，所选根自己那条记录读不出时可写挂载照做 => 红（退出码 101；test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 2.57s）
MUT 拒绝合法（11-4）：每块盘最新那一槽系统配置都报读错时 N-配置 复算成假（C331 取甲「按真」那一句丢了） => 红（退出码 101；test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s）
MUT 挂载透明（11-4）：两棵树的根指针只看 R1 读不读得出、不比 => 红（退出码 101；test result: FAILED. 42 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s）
MUT C673 ⑥ I-5.5 判法：落点判成恒成立（数据单元搬走的坏镜像不红） => 红（退出码 101；test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 82 filtered out; finished in 7.87s）
MUT 第26批 C664 判法：注入过读故障时读阶段之后系统配置又读不出那一拒照样要判据在镜像上复算为真（瞬时读错判成拒错了） => 红（退出码 101；test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s）
MUT 第26批 C664 判法：没注入读故障时也放过读阶段之后系统配置又读不出那一拒的判据复算 => 红（退出码 101；test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s）
MUT 第26批 C664 判法：读阶段之后系统配置又读不出那一拒当判据 N-配置 的拒判（复算不按各自的判据） => 红（退出码 101；test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s）
MUT 第26批 C664 判法：读阶段之后系统配置又读不出那一拒的判据不认「某块盘一份自证过的都没有」 => 红（退出码 101；test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.02s）
MUT 第26批 C664 判法：复算判据 N-配置 仍把「某块盘一份自证过的系统配置都没有」按真（两种拒因的判据并成一个） => 红（退出码 101；test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.03s）
```

前 5 行是替换的那 5 行，各自点名的用例都红。删掉的 RR8（「复算判据 N-配置 不认某块盘一份自证过的系统配置都没有」）罩的行为在 26-1r 之后本来就该是那样（那一形改由新拒因判），它的反面由后 5 行里的两行接住（「读阶段之后系统配置又读不出那一拒的判据不认…」与「复算判据 N-配置 仍把…按真」），都红。

## 四、J3　26-2

### J3-a　#8 那 69 条钉值：没打中

钉值在 `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` 的 `assert_checker_counts`（例：`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:172`：「let never_comparable_on_this_stream = ["I-9.14", "I-8.7", "I-8.8", "I-7.9", "I-3.12", "I-9.28"];」）。出处是 2026-10-04 主工作区的 checker 档日志，那棵树还没有 26-3。我在合入之后的冻结副本上自己造同形的流复现（J1-f 那一段），69 条逐条等于钉值，所以 26-3 没让它们过时。复现用的是我自己从起点造的流（历史执行器的「第一个文件之后」起点、前缀只有 mkfs），不是那份用例的流；两条流的段长序列（用例自己钉的是 `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:524`：「vec![2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2],」）与 46、9、6 三个数相同，字节不同（E142 参数与小盘参数不同）。

### J3-b　#7 选择器 `<= 3` → `<= 5`、计数 1 → 9：没打中（读码）

`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:847`：「&|_segment_index, segment| segment.len() <= 5,」，`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:853`：「tally.record_root_without_record, 9,」。段序列 `[2, 2, 1, 2, 2, 1, 2, 24, 5]` 里放宽之后多展开的只有末尾那一段 5 个写（两份记录、根槽、两次系统配置轮换），24 个单元写那一段在 `<= 3` 与 `<= 5` 下都不展开。那一段里两份记录与根槽是首次写（各两态），两次轮换是原地覆写（各三态）；根槽落了、两份记录都没落的组合是 1 × 1 × 1 × 3 × 3 = 9，都是真子集。用例要判的「摘掉屏障就摆得出根在案而记录不在」照样摆得出。推翻条件：checker 档跑出的计数不是 9，或那一段里记录写被认成原地覆写。

### J3-c　#2–4 按盘放行清零：没打中（读码）

`crates/singlefs-checker-tier/tests/a_publish_returns_only_after_the_system_configuration_rotation_is_durable.rs:155`：「|| !devices_with_a_system_configuration_slot_write」。见证值从空白镜像起算，只施加系统配置槽写；一块盘还没发过系统配置槽写时，它的两槽在这份镜像上全 0，清零不改它；发过之后（不论那一写放没放行）再清零照旧停下。层 0 的段内子集按发出次序施加，同一段里先发的清零不会盖到后发的槽写上。我想到的放过形状都要求清零落在同一块盘发过槽写之后，那正是停下的那一支。

### J3-d　C686 断言的出处：没打中

`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:1238`：「fn allocation_record_tree_nodes_carry_birth_sequences_zero_to_four_as_the_bump_order_table_lists() {」里五行（落点、层级、出生序号、罩哪块盘）与 `.claude/kb/decisions/03-空间分配.md:216`：「| t5 | 分配记录树叶，设备 0（码 2） | 65538 | 50245 | 0 |」到 `.claude/kb/decisions/03-空间分配.md:220`：「| t9 | 分配记录树根，层级 2（码 2） | 65538 | 50249 | 4 |」逐格相同；层级取表里「层级 1 节点」「层级 2」两处字面，出生序号的发号次序与 bump 次序相同是 ⑤ 那一段的定案。孩子指针那一格按「低一层、同一块盘」从表里推，根罩两块盘，推得出。「两块盘上两份逐字节相同」这一句没在 ⑤ 表里，它出自提交内生块两盘同槽的放置，用例绿着，我没再追条款。

### J3-e　#6 门槛、#9 打印行：没打中（推的）

#6 `crates/singlefs-checker-tier/tests/crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs:587`：「rejudged.red_states_by_signature.len() >= 4 && largest_signature_states >= 8,」：签名来自那条流上故意写错的版本表让 oracle 红，26-3 不改 checker 在这条合法流上的判定（J1-f 量过 69 条 0 违例），所以签名分组不变。#9 的打印行哈希同理不变（GOLDEN_TALLY 那几项就是这 69 条的计数）；进度文件哈希是占位字串 `crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs:851`：「const GOLDEN_PROGRESS_FILE_SHA256: &str = "待重录：进度文件内容的 SHA-256，取法见上";」，checker 档跑到那一核必红，记录里已写待重录。

## 五、改法（都是攻方提的，只在我的模型上量过、被攻过零轮）

| 编号 | 改法 | 修哪一格 | 在打中的格上 | 标 |
|---|---|---|---|---|
| G1 | 挂载透明运行配置那一行的期望字节在 harness 里照 D16 已定项 5、D26 写死（5 s、2 GiB、0 × 3），不经 core 的 `SystemRuntimeConfiguration` | J1-c | MUT-A 下由绿转红 | 推的（没实现；MUT-A 只量了今天的绿） |
| G2 | 定一句「根槽 [457, 根槽宽)、系统配置槽 [489, 4096) 第一版恒 0」并进 I-2.4，或明写不判 | J1-d | 往槽尾写非 0 的实现由不红转红（定成恒 0 时） | 推的，要先定条款 |
| G3 | 崩溃注入第二、三截判新拒因时一律按没注入判；或给判官加一个「被拒的这次挂载自己在读故障下跑」的入参，崩溃注入恒交「否」，harness 那份用例交「是」 | J2-a | 注入态上复算为假的新拒因由合法转不合法 | 射程量过（29% 的段），遮住的次数今天 0；改法推的 |
| G4 | 新拒因补 C670 同形的一条：没注入读故障的崩溃态上新拒因复算为真，报「这份盘面正确实现摆不出」 | J2-b | MUT-B1 那 90 次由合法转红 | 推的（数是 MUT-B1 下数出来的拒数，判法没实现）；不带改坏的各段里新拒因 0 次，不会多出误红 |
| G5 | 二次崩溃的挂载透明照第一截一样按标记跳过；或在 D13 射程句下写明二次崩溃照判的理由 | J2-d | 只是口径，今天不误红 | 推的 |
| G6 | F3 再加两条取样：C3 历史种子 0 与 2^k（k = 0..63）各 4095 个；B4 长 1024 的段扣下一个写 | J2-f | 两类改坏由绿转红 | 量过（`out/f3.txt` 的 `F3PROPOSED` 九行） |

G1 到 G6 各自只修表里那一格；J2-a 的 G3 对 J2-b 不起作用（MUT-B1 那 90 个态都没注入），J2-b 的 G4 对 J2-a 也不起作用（注入态照旧被 G3 之前的放宽遮住）。

## 没打中的形状

- I-9.16 ③ ④ 在合法历史上：core 写的树表恒 11 条、每种一条；由记录施加出来的那一版的树表也是 core 写的单元。上面 14 段 0 违例。没造「种类 9–12 缺一条」的实现：④ 只要求 1–4（判决 Y3 取的就是这个射程），这样的树表今天不红，是设计如此，不算打中。
- I-2.4 根槽 428..457：core 的 `to_slot` 恒写 0（`crates/singlefs-core/src/root_record.rs:112`：「writer.skip(12 + 16); // nonce、MAC：第一版留位全 0」），判的是全部可读根，合法历史上没有非 0 的来源。
- I-5.4 新格：带文件的一版根记录里分配记录树根恒全零（`publish_first_file`、覆盖写、回退的向前发布都写空根；零单元发布照抄上一版，只在暖机走），r3 里树表 0 条、写过行的一版（实例 2、3）之后接 pwu 与 mount 都没红。
- journal_tail 与 journal 实例代号两行在多实例、取号只落一块盘、zeros 之后：R1 与 R2 都择盘 0 那一份（`crates/singlefs-core/src/recovery.rs:1132`：「None => chosen = Some(best_on_device),」），取号写带全池见证值、轮换写带本次末条计数器，区间与取等两句在我想到的二次崩溃组合上都成立，量的也是 0。区间罩不住「取号写把 tail 写成盘 0 自己那一份」这种改坏，但我想不出它在单故障下有害的历史（推的），所以不列为打中。
- 挂载透明运行配置在合法历史上：core 只有一种写法，R2 恒等于默认，不会误红（J1-c 打中的是另一面）。
- 「崩溃恢复抛弃根」那一步：从第一个文件起点跑 `abandon`，起点之后 0 个写（与 C696 那一行「发了写的 0 次」相同），注入态只有 zeros 造得出。
- 正常卸载后再挂：r9 卸载那一段 12,330 个状态（含卸载做完的那一个）全取第二截，0 行判红、0 次拒。

## 这条腿自己的限度

- 崩溃注入本身没跑（它的测试目标在 checker 档）。我的第二截照它公开的入口重写了一份：没有第三截（再挂与 I-8.4、C670），没有 RefFS，清扫删了行的挂载不判，二次崩溃每段至多 16 个（r3 是 64）、多的按种子抽，r5–r8 的第二截按 1/3、1/10、1/25 取。第一次崩溃的状态一个不落。
- 历史是我写死的 14 段，没有随机抽：随机历史一段就超过 10⁶ 个层 0 状态。盘宽几乎只用了 384 槽小盘；4 GiB 只用在 `quick` 与 `seglen`。
- 每个「没打中」都只抽了一次。
- `probe/src/f3.rs` 照抄了 `crash_injection.rs` 里两个不公开函数的式子与三个常量，没法直接调它们；抄错了数就不对。对照：入库的三行变异在我的模型上各被它点名的那一条抓住（M7a 被 B3、M7d 被 A2、M7f 被 C2），与那三行变异的登记相符。
- MUT-A、MUT-B1 是我自己造的坏实现，只在副本上量过；副本上的数不算入库装置上的数。
- GPU 判器一格没跑。

## 没做什么

- checker 档的测试、`crates/mutations.tsv` 整表、崩溃注入战役、GPU 对拍都没跑；J1-f、J3 涉及 checker 档用例的结论是在我的模型上复现或读码推的。
- 没读别轮的腿报告（c655-c687、b24 两轮的腿报告与模型）；只读了它们的判决。
- 正推、辩方那几格不判，不替主 agent 采纳。
- 草稿目录 `/tmp/claude-1000/m3-formal-code-b26-r1-opus/` 里的仓副本、编译目录交回前删掉；`out/`、`edits/`、几份行文件留着（模型目录里有同样的产物）。

## 引文核对

`python3 research/scripts/cite-check.py research/prompts/m3-formal-code-b26-r1-opus-output.md --root /tmp/claude-1000/m3-formal-code-b26-r1/tree --root . --background research/prompts/_m3-formal-code-b26-r1-background.md` 的末行：

```
  ✓ 核了 38 处引文，对上 38 处，没判 0 处（逐处列在上面）
```
