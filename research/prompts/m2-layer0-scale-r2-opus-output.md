# m2-layer0-scale-r2 云端攻方腿（Opus）报告

<!-- doc-lint:not-numbers L1 L2 L6 L7 H1 H2 R1 R2 R3 R4 R5 M1 M2 M3 M4 -->

写于 2026-09-26。攻击面 M1、M2、M3（背景材料第四节）。前几轮判决：`research/prompts/m2-layer0-scale-r1-main-verification.md`。全部量出来的数出自冻结副本 `/tmp/claude-1000/l0scale-r1-frozen/` 的拷贝（草稿目录里的 `repo`、`repo-l7` 两份，交回前删），不是入库装置；要引须在入库装置上重做。

复跑（在仓根下）：

```
bash research/prompts/m2-layer0-scale-r2-opus-model/rerun.sh /tmp/claude-1000/l0scale-r1-frozen /tmp/claude-1000/m2-layer0-scale-r2-opus-rerun
```

它把冻结副本拷两份（原样放探针 `opus_r2_probe.rs`；打第一轮的续跑补丁 `m2-layer0-scale-r1-opus-model/l7-resume-prototype-crash-rs.patch` 再放 `opus_r2_resume_probe.rs`），经 `run-with-memory-cap.sh 8G` 与 `capped.sh ${THREADS:-10}` 逐格跑，最后 grep 汇总行。两个探针名字都不带 `layer0`。全跑约 50 分钟（各格挂钟写在 `rerun.sh` 文件头）。`opus_r2_probe.rs` 是第一轮探针 `opus_scale_probe.rs` 原样加上这一轮的追加（文件里「m2-layer0-scale-r2 云端攻方（Opus）追加」以下）。

模型目录每个文件的 sha256（`SHA256SUMS` 原样）：

```
f8bf652a54ef73a760718fb1cc3762813032ce721827ec2e0d9d008e9fb70297  ./opus_r2_probe.rs
a9e9ff9c432c6b586b3ae332c6c66a3ab689aef46a8ab9096e3c85f9e87accdf  ./opus_r2_resume_probe.rs
54aed70d15624cdfca2ce644a6a6a6acd6dc9f80ce5bb8842ec258243c6ef05f  ./outputs/m3-common-rs-2.log
b64ad0740e65c9960cb29598a1a67ccece632b9180b370abfb9a7ac5a434c1c2  ./outputs/q10.log
c9e96cf0195925e3e2b673c6390ed37d9cc37c372f2674a3146ab5417727c723  ./outputs/q1-mirror.log
b163a37bcbf3ff6248083062c589095d65ab41391b381f71113f508b8ac50e6f  ./outputs/q1-sweep120.log
72bd5d54bf343b4c8b35cfdaad043498d239c7c54497ebc4438d74cee1664230  ./outputs/q3-sweep1.log
57aa0afc4dcb1b449deb0715fc7740490c6868b95da15ffd566d9f90180806e4  ./outputs/q4-1.log
f75a64a16ee8c30f6995ff0c393f2903a571ea0297c25d33c6c168708367b8a4  ./outputs/q5-confirm21.log
fb5226e3dd65a86b3a807977117512cff3948f9392d0c457cf63461daf0f181b  ./outputs/q5-shrink.log
630c8f85811b52135f8712da93e62742d2b5756c82b14afd666e54c88ccd829b  ./outputs/q6-1.log
013788ce4f1d65670d3be277a8f34c7447f0dd3889b81f49795590e0d3c622c2  ./outputs/q7_exemption_chains_on_fixed_scripts.log
4c204b6f4a48c53b2eff1f2faf98ae3ec4fcf41add61547fc8ff593e664cbe68  ./outputs/q8_four_write_truth_table.log
5bb15bda5cc60881ee0ec0d008b411bd3c349e51f81fcc1c4aa11c8228c264e0  ./outputs/q9.log
1f5a445eb2e8f5f080a923fab65a4ff23208e6363b8d9120de58714bc150bea1  ./outputs/r2a_one_file_per_input_fingerprint_lets_the_first_stream_wipe_the_second.log
6dd9a4a05808b5e29464e1e52c3047780b8e7581e2ef09676b146baf14f8f1f0  ./outputs/r2b_observer_assertion_is_lost_on_resume.log
2e68a9962f33538e8c1953a7f5a506fc82ddac7be3c6ffa43dee42845f39fcac  ./rerun.sh
```

## 各格判定一览

| 格 | 判定 | 一句话 | 节 |
|---|---|---|---|
| M1 甲二的等价（字面） | **打中，分辨臂；形态是「判据自己写错」** | 可达历史 `UOOUOMSU`（A、B 之后：卸载重挂、覆盖写两次、卸载重挂、覆盖写、进程重开重挂、写一个小文件、卸载重挂）：txg 25 那一段（2 个原地 + 16 个 COW）2^18 = 262144 个状态里 32768 个被记录核对器第二条判红，原地写取 ∅ 的状态里也有（`q1-mirror.log` 的 `imask=0b0`），COW 取 ∅ 与取全集时一个都不红；甲二在整条流 334 个状态上 0 个红。红的是记录核对器复用豁免的漏洞（同一段里 oracle 两遍与 checker 违例都是 0），不是实现的错 | 1.1–1.4 |
| M1 不经根读单元区的另三处（I-7.7、I-7.8、I-1.8） | 没打中 | 随机 120 条历史的全部相交 COW 组逐组全枚举 51720 次判定，不单调的点 0 个；回收窗口置 0（C22 类变异替身）7 条历史，甲二每条都红（32–63 个） | 1.5、第五节 |
| M1 今天 54 号的固定脚本 | 不受影响 | B、C、D、E、E18（与第一轮 H1、H2）上豁免链候选 0 个 | 1.4 |
| M2 R1 键取整份输入指纹 | **打中两格** | ① 一份指纹一个进度文件：两条流（与同一二进制里的几条用例）共用一个指纹，一条流的头行把另一条流的进度整份作废（量过：43 片里杀前跑完的 21 片全丢）——只丢进度、不出错；② 指纹里没有 `.cargo/config.toml`、`RUSTFLAGS`、`CARGO_PROFILE_*`：一半片在一种编译配置下跑、另一半在另一种下跑，合成一份「全量」（推的） | 2.1 |
| M2 R3 严格读法 | 没打中 | 读代码没找到「丢一片而不知道」的读法：合并前核「文件里的片 + 这次跑的片 = 总片数」，不齐就不出结果 | 2.2 |
| M2 R4 观察者 | **打中** | 观察者里的断言：第一趟某个状态让断言 panic，进度行已经写满；第二趟续跑全部片从文件读回，观察者一个状态都没看到，断言不再跑，这一趟绿（量过：第一趟 panicked=true，续跑 panicked=false observer_saw=0） | 2.3 |
| M2 R5 报续跑片数、强制从头跑 | 打中一格（推的） | 判红的那一趟不删进度文件，下一趟续跑就把 2.3 那一类红「续」成绿；R5 只要求报片数，报了也挡不住 | 2.4 |
| M3 乙按流拆的清单 | **打中**（1 格量过，4 格推的） | 清单漏了加在 `src/` 之外、两条流都读的东西：`tests/common.rs`（与 `tests/common/mod.rs` 并存即编不过，量过 E0761）、新加的 `build.rs`、新加的 crate、`.cargo/config.toml`；另有一格今天的整份 `crates/` 也漏：`.gitignore` 的 `target/` 不分层级，`src/` 下叫 `target` 的模块目录不进输入清单 | 第三节 |

背景材料给 M1 的改法是甲二本身；它在打中的那一段不红（量过）。我提的改法（记录核对器第二条的豁免按扇区判）在同一段 262144 个状态上红 0 个（量过，只在这一段上），被攻过零轮，见第四节。

## 一、M1：甲二的等价

### 1.1 打中：记录核对器第二条在「一对重叠的 COW 写只落一个」时判红

历史（前缀照第一轮：mkfs → 取号 1 → 暖机 → A（txg 3）→ B（txg 4），同一进程；之后的字母是用户动作，每步都是冻结副本里公开的入口）：

| 步 | 动作 | 入口 |
|---|---|---|
| U | 正常卸载再可写挂载 | `singlefs_core::mount::unmount`、`mount_writable` |
| O、O | 覆盖写两次（内容约 3 KiB，一个数据单元） | `publish_overwrite` |
| U | 同上 | |
| O | 覆盖写 | |
| M | 进程重开、可写挂载 | `mount_writable` |
| S | 覆盖写一个 16 字节的小文件 | `publish_overwrite` |
| U | 同上 | |

写数与段（`q6-1.log` 原样）：

```
Q6 seq=UOOUOMSU applied=UOOUOMSU writes=589 segments=83 closed_form_full=3438215360 arm_a2_closed_form=334 sigma=79 sigma_len=18 sigma_kinds={"system_configuration_slot": 2, "unit_write": 16} sigma_txg=Some(25)
  CHAIN u=#59(dev0 slot50262+1 txgSome(4)) l=#362(dev0 slot50262+2 txgSome(17)) x=#574(dev0 slot50263+1 txgSome(25)) y=#572(dev0 slot50262+1 txgSome(25)) why=record:claimed_state_missing_unit
  ARM_A2 seq=UOOUOMSU states=334 red=0 elapsed_seconds=0.7
  SIGMA_FULL seq=UOOUOMSU states=262144 expected=262144 record_claimed_state_missing_unit=32768 violations=0 ignored=0 root_without_record=0 checker_violations=0 elapsed_seconds=598.9
```

（`SIGMA_FULL` 用的是层 0 自己的枚举器 `enumerate_layer0_selecting_versions`，`expand` 只展开第 79 段；`ARM_A2` 是甲二的全部状态，每个状态经层 0 的 `evaluate_state_for_versions` 评，红 = 计数里任一项违例非 0。）

发生了什么（槽号是 16 KiB 槽）：

- B（txg 4）在槽 50262、50263 各写了一个 16 KiB 节点（两盘各一份；`u` 是 50262 那一份）。
- 卸载之后这两个槽被回收；txg 17 的一个 32 KiB 数据单元 `l` 落在 50262–50263，盖住 B 的两个节点。
- 又一次卸载之后 `l` 也被回收；txg 25 的两个 16 KiB 节点 `y`（50262）、`x`（50263）分别盖回去，与第 79 段里的另外 14 个单元写、2 个系统配置槽写并在一段。
- 恢复实际走的根 txg ≥ 4（记录核对器判了这一条就说明如此），记录核对器第二条就要 B 的每个单元「在盘上，或被更晚、已持久、而且整份还在位的写盖过」。

判定的真值表（`q8_four_write_truth_table.log`，段里原地写全落、另外 12 个 COW 写取 ∅；取全集时 16 行逐行相同）里红的只有两行：

```
Q8 seq=UOOUOMSU others_all=false x_dev0=1 x_dev1=1 y_dev0=0 y_dev1=0 red=true record:claimed_state_missing_unit
Q8 seq=UOOUOMSU others_all=false x_dev0=0 x_dev1=0 y_dev0=1 y_dev1=1 red=true record:claimed_state_missing_unit
```

- 只落 `x`（两盘）：`l` 被 `x` 盖掉一半，不再「整份在位」；`y` 没落，槽 50262 上是 `l` 的前半——B 在 50262 的节点「不在、也没被整份在位的写盖过」，判缺席。
- 只落 `y`：对称地，B 在 50263 的节点判缺席。
- ∅：`l` 整份在位，两个节点都豁免；全集：`x`、`y` 各自整份在位，各盖一个，都豁免。

规则出处（冻结副本 `crates/singlefs-harness/src/crash.rs`）：第 706 行 `                && in_place(later_index)`（盖过它的后写要整份在位），第 714 行

```
    let copy_is_missing = |copy: usize| !in_place(copy) && !written_over_later(copy);
```

这个状态本身是合法的：B 的节点在 txg 17 之前已按回收谓词回收，`l` 在 txg 25 之前同样；恢复走的根不引用 B 的节点，也不引用 `l`（同一段 262144 个状态里 oracle 两遍违例 0、checker 违例 0）。

### 1.2 四句

- **分不分辨臂：分辨。** 全量层 0（今天的枚举域）在这条流上判红（第 79 段 32768 个状态）；甲二 334 个状态 0 个红；甲是甲二的子集，同样 0 个。
- **被判的系统当时看不看得到判别它的东西：看得到。** 记录核对器手里就有写表与崩溃镜像，扇区级的「这几个字节是谁写的」它算得出来；枚举器切段时也看得到——探针的 `exemption_chain_candidates` 只读写表就把这 21 条历史全部挑出来（1.3），不用跑恢复。
- **满足判据字面哪一句：** M1 那一行「造一个只在『原地写的某个子集没落 ∧ COW 写只落一部分』时才红的错（变异或可达历史），重点是不经根读单元区的四处判据（…记录核对器第二条）」——可达历史、红只出现在 COW 只落一部分时、原地写取 ∅ 的子集里也有（`q1-mirror.log` 里 `imask=0b0` 那两行）。**但红的是判据自己，不是实现**：按证据纪律的四种形态，这是「判据自己也会写错」那一类——记录核对器的复用豁免只认「整份在位」的后写，跨两代、跨度不对齐的复用（两个 16 KiB 节点 → 一个 32 KiB 数据单元 → 两个 16 KiB 节点）在它眼里成了洞。所以这一格对 M1 共用问句的答案是：甲二放过的是一个**假红**，今天的全量会把它报出来（逼人去修记录核对器），甲二之下它一直不露面；它不是实现的错被放过。
- **改法在打中的那几格上还中不中：** 甲二：不红（量过）。全量：红（量过）。第一轮 L2 的现判：引用集合按 D16 候选集取时 `u`、`l` 都已被回收、不在集合里，不退回全量，漏（推的）；按「写表里已持久的写落过的扇区」取时 `x`、`y` 都落在旧扇区上，退回全量，中（推的）。记录核对器的豁免改成按扇区判（第四节）：这一段 0 个红（量过）——假红没了，全量与甲二在这一段上判得一样。

### 1.3 这个形状有多常见

- 静态扫（`q3-sweep1.log`，只读写表、不跑恢复）：3000 条随机历史，动作取 O、U、M、R、S，长 1–14，种子 7：

```
Q3_TOTAL histories=3000 with_candidates=21 elapsed_seconds=339.2
```

- 21 条逐条动态坐实（`q5-confirm21.log`：σ 段原地全落，COW 取 ∅、只落 `x` 两盘、全集三个状态，用层 0 的评估函数判）：21 条都是「∅ 不红、只落 `x` 红、全集不红」。
- 缩小（`q5-shrink.log`，逐个删动作、仍坐实就留下）：

```
  SHRUNK seq=RUSUSMUSOUS minimal=USUSUSOUS
  SHRUNK seq=UOOUSUSOMMSU minimal=UOOUOMSU
  SHRUNK seq=SOMSMOUSSSSOMU minimal=SOMSMOUSSSSOMU
  SHRUNK seq=MMUSOUSMUSU minimal=USOUSMSU
```

每条最小的都要两次卸载（U）与一次写小文件（S）或更多次覆盖写：要的是「节点槽 → 数据单元 → 节点槽」这样跨度不对齐的两代复用，只有卸载抬 F 之后回收才放得出这么多旧槽。

### 1.4 今天 54 号的固定脚本不受影响

```
Q7 stream=B writes=70 segments=13 chain_candidates=0 confirmed=false
Q7 stream=C writes=166 segments=26 chain_candidates=0 confirmed=false
Q7 stream=D writes=216 segments=33 chain_candidates=0 confirmed=false
Q7 stream=E writes=425 segments=55 chain_candidates=0 confirmed=false
Q7 stream=E18 writes=458 segments=58 chain_candidates=0 confirmed=false
Q7 stream=H1 writes=166 segments=26 chain_candidates=0 confirmed=false
Q7 stream=H2 writes=466 segments=65 chain_candidates=0 confirmed=false
```

冻结副本的固定脚本里没有卸载，出不了这个形状。实三要把正常卸载与挂着时回退加进固定脚本；新形状上有没有，要等实三交回后拿 `exemption_chain_candidates` 那一段（只读写表）在新流上扫一遍（没量）。

### 1.5 另三处（I-7.7、I-7.8、I-1.8）：搜过的形状

甲二漏判要一个判据在同一原地子集下「COW 取 ∅ 不红、取全集不红、取某个真子集红」。先按代码分两类 COW 写：

- **落在从没被写过的扇区上的 COW 写**（绝大多数）：四处判据对它们都是单调的——I-7.7 ① ② 多一个带高实例代号的单元头只会多红；I-7.8 多一个码 2 节点只会抬高「出现过的最大树 ID」；I-1.8 ① 同一份内容的两盘副本归并键相同、载荷相同，多一个成员不会让一组变红，不同内容要归并键（含诞生代号与写序，冻结副本 `crates/singlefs-checker/src/walk.rs:3447` 的 `merge_key_before_payload_checksum: 42..101`）撞上才会同组；记录核对器第二条只管根已持久的发布，这一段的单元属于根在后面段里的那次发布。所以这类写的红随 COW 子集单调，全集那一个红，甲二就红（推的：读代码）。
- **落在之前写过的扇区上的 COW 写**：部分落时露出的是旧字节，不单调只可能出在这里。探针对每一段、每个原地子集，把段里与之前单元写相交的 COW 写按「经相交或同槽镜像连通」分组，每组全枚举子集（组外的 COW 写取 ∅、全集各一遍），用记录核对器两条 + 池级 checker 判（`search_non_monotone`；恢复只在每个原地子集上跑一次，这一段没有 journal 记录与根槽写时它不随 COW 变）。

| 形状 | 取样 | 不单调的点 |
|---|---|---|
| 随机历史（O、U、M、R、S，长 1–10，种子 11），每段全部原地子集 | 120 条，51720 次判定，没有一组因超过 14 个写被跳过 | 0 |
| 1.1 那条历史（对照：搜得出来） | 1 条 | 8 个，全是 `record` |
| 回收窗口置 0（`ReuseWindow::ForcedToZero`，C22「刚释放的块立即重分配」那一类的变异替身）：`ZOOOO`、`OZOOO`、`UZOOOSO`、`ZSOSOS`、`MZOOO`、`UOZOSOS`、`ZOUZOO` | 7 条，每段全部原地子集 | 0；甲二每条都红 |
| 第一轮的固定脚本 B、C、D、E18 与 H1、H2 | 第一轮 p2、p6、p7（6330 个取样状态 + 786423 个穷举） | 0（第一轮） |

汇总行原样（`q1-sweep120.log`、`q1-mirror.log`）：

```
Q1_TOTAL streams=120 streams_with_non_monotone=0 cheap_checked=51720 elapsed_seconds=948.9
  NM_KINDS seq=UOOUOMSU {"record": 8} is_exemption_chain_stream=true
```

回收窗口置 0 的三臂（`q10.log`，每条的 `ARMS` 行；甲 = COW 取 ∅，甲二 = ∅ 或全集，全量抽样 = 每段每个原地子集 × 全集、单点、去单点、8 个随机）：

```
ARMS ZOOOO arm_a_states=66 arm_a_bad=31 arm_a2_states=84 arm_a2_bad=43 full_sampled_cow_states=1368 full_sampled_bad_beyond_arm_a=132
ARMS OZOOO arm_a_states=66 arm_a_bad=23 arm_a2_states=84 arm_a2_bad=32 full_sampled_cow_states=1368 full_sampled_bad_beyond_arm_a=128
ARMS UZOOOSO arm_a_states=146 arm_a_bad=39 arm_a2_states=179 arm_a2_bad=54 full_sampled_cow_states=2434 full_sampled_bad_beyond_arm_a=140
ARMS ZSOSOS arm_a_states=74 arm_a_bad=39 arm_a2_states=95 arm_a2_bad=54 full_sampled_cow_states=1596 full_sampled_bad_beyond_arm_a=132
ARMS MZOOO arm_a_states=94 arm_a_bad=23 arm_a2_states=115 arm_a2_bad=32 full_sampled_cow_states=1577 full_sampled_bad_beyond_arm_a=136
ARMS UOZOSOS arm_a_states=146 arm_a_bad=31 arm_a2_states=179 arm_a2_bad=43 full_sampled_cow_states=2434 full_sampled_bad_beyond_arm_a=148
ARMS ZOUZOO arm_a_states=122 arm_a_bad=51 arm_a2_states=146 arm_a2_bad=63 full_sampled_cow_states=1750 full_sampled_bad_beyond_arm_a=272
```

- 120 条里 0 条与静态扫的比例对得上：3000 条里 21 条（0.7%），120 条的期望约 0.84 条。所以「120 条 0 个」不说明别的形状不存在，只说明这 120 条里没有；静态扫只认 1.1 那一种链，别的不单调形状靠的是这 120 条与 7 条变异替身。
- 这一轮之前两次小跑（`q1-try1`、`q1-chains1`，6 条历史）的分组没把同槽的两盘镜像连起来，一对镜像只落一盘时记录核对器按「两份都不在」判不出，那两次的 0 不算数；上面的 120 条是改过之后跑的。

### 1.6 没做的 M1 取样

- 没有在一条流上对甲二做全量对照（每段每个原地子集 × 全部 COW 子集）：1.1 那一段是 2^18 个状态，整条流 34 亿个。做了的是 1.1 那一段的 2^18 全枚举（262144 个，闭式先算过，≤ 10⁶），USUSUSOUS 那一条的 σ 段 30 个写（2^30 − 1 个）按上限没跑，只跑了真值表那一类三个状态。
- 变异只用了回收窗口置 0 这一个替身；没有造「同一实例同一事务号写两版不同内容」让 I-1.8 ① 的归并键相撞的变异（读代码：要诞生代号、写序、五元组全相同而载荷不同，今天的事务号不重号，C464 那一条相关），没量。

## 二、M2：续跑 R1–R5

R1–R5 原文在第一轮判决 `research/prompts/m2-layer0-scale-r1-main-verification.md` 第 31–35 行。原型用第一轮的续跑补丁（严格读法、按片号并、片号去重、头行逐字比、对不上整份删掉从头跑），这一轮只加用例 `opus_r2_resume_probe.rs`。

### 2.1 R1：键取整份输入指纹

换了什么，各是什么结局：

| 换了 | 进不进键 | 结局 |
|---|---|---|
| 线程数 | 不进（R2 让片方案不随线程数变） | 续跑得上、结果与一口气跑完相同（第一轮 p8、p9 量过） |
| 工具链（`cargo -V`、`rustc -V`） | 进 | 整份作废、从头跑，对 |
| 机器 | 键里的工具链行是 `rustc -V` 原样，不带宿主三元组（本机 `rustc -V` 输出 `rustc 1.98.0 (88d9e12ae 2026-08-18)`，三元组只在 `rustc -vV` 的 `host:` 行） | 进度文件若随 git common-dir 搬到另一种宿主上，键对得上、照续；代码里没有随宿主变的分支时结果照样对（推的，没量） |
| `.cargo/config.toml`、`RUSTFLAGS`、`CARGO_PROFILE_RELEASE_*` | **不进**：54 号的输入是 `.claude/gate.d/stage-inputs.tsv` 第 20 行登记的 `crates/ Cargo.toml Cargo.lock` 加 54 号与工具链两行 | 一半片在 `Cargo.toml` 第 17 行 `overflow-checks = true` 下跑、剩下的在一份把它关掉的配置下跑，并成一份 `exhaustive=true` 的全量（推的，没量；今天的全绿标记本来就有这个缺口，续跑把它从「整趟一种配置」放宽成「一趟里混两种」） |

**打中一格（量过）：一份指纹一个进度文件会互相作废。** R1 写的是「键取整份输入指纹（与 54 号全绿标记同一份清单…）」，54 号一趟 `--full` 先跑第一条流、再跑第二条流，两条流（连同同一个二进制里的几条全量用例）指纹相同。只按指纹找进度文件时，下一趟先跑的第一条流读到第二条流的头行（片方案不同）按 R2 整份作废，第二条流杀之前跑完的那一半全丢（`r2a_…log` 原样）：

```
R2A first_run long_slices_total=43 long_slices_evaluated_before_kill=21 second_run short_header_rejected=true short_slices_from_file=0 long_header_rejected=true long_slices_from_file=0 long_slices_evaluated=43 result_equal=true
R2A per_stream_files long_slices_from_file=21 long_slices_evaluated=22 result_equal=true
```

四句：不分辨甲、乙；系统看得到（调用方知道自己是哪条流、哪条用例）；满足的是 M2「R1 的键取整份输入指纹之后…各是什么结局」；结局是只丢进度、不出错（`result_equal=true`），但 2.3 叠上来时丢的是对错。改法：进度文件名按「指纹 + 测试二进制 + 用例名 + 计划哈希」分（第一轮攻方提的计划哈希），指纹只当文件里的第一行核（推的，只在原型上量过按流分文件那一半）。

### 2.2 R3：严格读法有没有丢一片而不知道的读法

读原型与 R3 字面，没找到：

- 末行半截、中间一行校验和不对：从那一行起截掉，截掉的片重跑（第一轮 107386 个截点 0 错）。截掉的不只是坏那一片、还有它后面的好行，只多跑、不少跑。
- 合并前核「文件里读回的片 + 这一次跑的片 = 计划的总片数」，不齐就不出结果（原型 `research/prompts/m2-layer0-scale-r1-opus-model/l7-resume-prototype-crash-rs.patch:228` 的 `enumerate_layer0_resumable` 里 `if from_file.len() + new_slices.len() < slices.len() { return (None, stats); }`）。R3 字面没写这一道；实六照 R3 实现时漏了它，才会有「少一片、状态数少、而全绿标记照写」——那时要靠 54 号的「状态数 = 闭式」兜住（54 号今天判 `exhaustive=true`，推的）。
- 两个进程往同一份文件追加：按片号去重只收第一次，结果对（第一轮量过）。

### 2.3 R4：观察者里的断言，续跑之后不再跑

R4 给了两条路：「观察者（逐状态回调）的累计要么进进度文件、要么限定成按片可重算的量；续跑之后观察者看到的状态数必须等于闭式」。今天的观察者不只累计，里面还有断言：

- 冻结副本 `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:977` 起的观察者里有 `assert!`（第 991 行提示串 `"展开的状态里 B 的记录与根槽都已持久"`）；
- `second_transaction_parallel_line_one_layer0.rs:198` 起的 `observe_the_publish_boundary` 里有 `assert_eq!`（第 212 行提示串 `"第 {} 次多记录发布的末条没落：…"`），第 261 行那条用例把它当观察者。

原型上照它造（观察者在第 40 个状态上断言失败；片行在工作线程里评完一片就写，观察者在调用线程上后看）（`r2b_…log` 原样）：

```
R2B first_run panicked=true observer_saw=40 progress_lines=43
R2B resumed_run panicked=false observer_saw=0 slices_from_file=43 slices_evaluated=0 tally_states=Some(127) progress_lines=43
```

第一趟红，进度文件已经写满 43 片；第二趟同一批输入续跑，一片都不用跑，观察者一个状态都没看到，这一趟绿。

四句：
- 分辨臂：只分辨续跑的写法，不涉及甲、乙。
- 系统看得到：看得到——哪几片观察者看完了、看的时候有没有 panic，调用线程知道。
- 满足哪句：M2「R4 观察者按片可重算这一限定，今天层 0 的观察者（逐状态回调）有哪些做不到」——带断言的观察者做不到：断言不是累计量，进不了进度文件，也「重算」不出来。
- 改法还中不中：R4 第一条路（累计进进度文件）不中（断言没有可存的累计）；R4 第二条路的尾句「续跑之后观察者看到的状态数必须等于闭式」照字面当一道判红，中（这里 0 ≠ 127，推的：原型没实现这道判）；我提的改法见第四节。

今天真在 54 号 `--full` 里的观察者只有第一条流那一个（`first_transaction_step_seven_layer0.rs:360`，只累计分配代读集的计数，R4 第一条路够用）；带断言的那两处在快用例里。R4 是否只对全量用例开续跑，决定这一格今天碰不碰得到（推的）。

### 2.4 R5：报续跑片数、留强制从头跑的开关

- R5 让 54 号报「这一次续跑了几片、从头跑了几片」。报了挡不住 2.3：那一趟的计数行、状态数、`exhaustive=true` 与从头跑的一模一样，全绿标记照写。
- 54 号今天在判红或被打断时删这批输入那一格标记；R5 没写判红那一趟要不要删进度文件。不删，下一趟就从一趟红过的进度续上（2.3 就是这样绿的）。改法：判红的那一趟连进度文件一起删，或者进度行只在那一片的观察者看完、没有 panic 之后才写（推的，没实现）。
- 「一次只跑了一半的全量被当成全量」：原型不会（2.2 那一道核），54 号的「状态数 = 闭式」也会判出来；会被当成全量的是「全部片都有、其中一部分是在一趟红过的运行里跑出来的」，即 2.3。

## 三、M3：乙按流拆的清单

被攻的清单是第一轮正推腿的提案（`research/prompts/m2-layer0-scale-r1-sonnet-output.md` 第 135 行起两行）：每条流一行，路径是自己那一份 `tests/*_layer0.rs`、`tests/common/`、4 个 crate 的 `src/` 与 `Cargo.toml`、根 `Cargo.toml`、`Cargo.lock`，加 `command=cargo command=rustc`。清单按路径前缀登记，算指纹的是 `research/scripts/admission.py:271` 的 `git ls-files -z --cached --others --exclude-standard -- <登记路径>`。

### 3.1 只进了一条流（或两条都没进）、而两条流都读的改动

| 改动 | 两条流读不读 | 进不进乙的清单 | 进不进今天的整份 `crates/` | 标记 |
|---|---|---|---|---|
| 新加 `crates/singlefs-harness/tests/common.rs`（与 `tests/common/mod.rs` 并存） | 读：两条流的测试文件都写 `mod common;`，两处都在就编不过 | **不进**（清单只有 `tests/common/` 目录与各自的用例文件） | 进 | 量过（下面原样） |
| 新加任一 crate 的 `build.rs`（能设 `cfg`、环境变量、改链接） | 读：cargo 编库与测试前先跑它 | **不进**（清单只有 `src/` 与 `Cargo.toml`；加 `build.rs` 不必改 `Cargo.toml`，cargo 自动认） | 进 | 推的 |
| 把一块代码拆进新 crate（`crates/singlefs-xxx/`），之后只改新 crate 的 `src/` | 读 | 拆的那一次改了 `Cargo.toml` 会重跑；**之后**改新 crate 不进 | 进 | 推的 |
| `src/` 里 `#[path = "../x.rs"]` 或 `include!("../x.rs")` 指到 `src/` 之外 | 读 | 不进 | 在 `crates/` 下就进 | 推的（冻结副本 4 个 crate 的 `src/` 里现查没有这类用法，只有 `model_comparison.rs:513` 的 `include_str!("model.rs")`，在 `src/` 里） |
| 仓根 `.cargo/config.toml`（`[build] rustflags`、`[profile.release]`、`[env]`） | 读：编出来的二进制随它变 | 不进 | **也不进**（登记的只有 `crates/ Cargo.toml Cargo.lock`） | 推的 |
| `src/` 下名叫 `target` 的目录（例如 `crates/singlefs-core/src/target/mod.rs`） | 读：`mod target;` 照编 | **不进** | **也不进**：仓根 `.gitignore` 第 3 行 `target/` 不分层级，`git ls-files --exclude-standard` 列不出它 | 量过 `git check-ignore`（下面原样）；编不编进去是推的 |

`tests/common.rs` 那一格（在我的副本上加一个空的 `tests/common.rs`，编自己的探针 `opus_r2_probe.rs`，它与两条流的测试文件一样写 `mod common;`；`m3-common-rs-2.log` 原样节选）：

```
error[E0761]: file for module `common` found at both "crates/singlefs-harness/tests/common.rs" and "crates/singlefs-harness/tests/common/mod.rs"
 --> crates/singlefs-harness/tests/opus_r2_probe.rs:5:1
```

顺带量到：同一个副本上不先 `touch` 探针源文件时，cargo 认为测试二进制是新的、不重编，退出 0（`m3-common-rs.log`，没入模型目录）——cargo 的 dep-info 只记读过的 `tests/common/mod.rs`，不记「没读到的 `tests/common.rs`」。54 号若在一个留着旧 `target/` 的工作树里跑，这一格连编译错都不会报（推的）。

`target/` 那一格（主工作区现查）：

```
$ git check-ignore -v --no-index crates/singlefs-core/src/target/mod.rs
.gitignore:3:target/	crates/singlefs-core/src/target/mod.rs
```

四句（对整张 3.1）：
- 分辨臂：`tests/common.rs`、`build.rs`、新 crate 三格分辨乙与今天（今天的整份 `crates/` 罩得到，乙罩不到）；`.cargo/config.toml` 与 `target/` 两格不分辨，今天与乙一起漏，是共用前提（登记路径 + `--exclude-standard`）的缺口，另立一笔账。
- 系统看得到：看得到。编译器知道它读了哪些文件（`cargo` 的 dep-info），但没读到的候选文件（`tests/common.rs`）它不记；清单若由 dep-info 生成也会漏这一格。
- 满足哪句：M3「有没有一处改动只进了一条流的清单、而另一条流也读它（…`build.rs`、`Cargo.lock`、宏）」——这几格是两条流都读、两条流的清单都没进。`Cargo.lock` 两行都有，不漏；过程宏今天 4 个 crate 都没有（`Cargo.toml` 的依赖只有 path 依赖），新加过程宏 crate 就是「新 crate」那一格。
- 改法：清单按「`crates/` 整份减去别的流自己的用例文件」写（排除法，不列举），新加的东西默认两条流都重跑；`.cargo/`、`rust-toolchain*`、`.gitignore` 进两条流的清单；`target/` 那一条要么把 `.gitignore` 改成 `/target/`，要么清单不按 `--exclude-standard` 取（都是推的）。

### 3.2 改了什么，各重跑哪几条（照乙的清单，按上面的排除法改过之后）

| 改的 | 第一条流 | 第二条流 | 依据 |
|---|---|---|---|
| 判它的 54 号 | 重跑 | 重跑 | 54 号的 sha256 进两条流的指纹 |
| 只改第一条流的用例文件 | 重跑 | 不跑 | 只在它自己的清单里 |
| 只改第二条流的用例文件 | 不跑 | 重跑 | 同上 |
| checker（`crates/singlefs-checker/src/`） | 重跑 | 重跑 | 两条流都经 `evaluate_state_for_versions` 调 `check_pool_image` |
| `tests/common/`、`tests/common.rs`、任一 `build.rs`、新 crate | 重跑 | 重跑 | 3.1 |
| 另外 7 份 `*_layer0.rs`、`tests/common_tree_split/` | 不跑 | 不跑 | 两条流的用例都不 `mod` 它们（冻结副本现查：两份只写 `mod common;`） |

乙之下 L6 那 6 条没人跑的全量照旧没人跑：它们不在 54 号里，按流拆碰不到它们（第一轮已报）。

## 四、我提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 在打中的格上 | 标记 |
|---|---|---|---|
| 记录核对器第二条的豁免按扇区判：一份副本缺席 ⟺ 它区间里有一个扇区上的字节不是它的，而这个扇区上没有一次更晚、已持久、在这个扇区上字节还在的写（探针 `variant_claimed_state_missing_unit`；C513 那一道「复用证得出违反回收谓词不开脱」没搬进去） | M1 1.1 | 第 79 段 262144 个状态：今天红 32768 个，改法红 0 个 | 量过（副本，下面原样）；没量它在别的流上是不是仍抓得到 C507（后写没落盘的真洞）那一格——按定义那一格的扇区上字节不是后写的，照判缺席（推的） |
| 甲二之下，段里有「COW 写落在一个已持久的单元写上、而那个单元写又盖过更早一次已被认领的单元写」的，这一段退回全量（静态、只读写表：探针 `exemption_chain_candidates` 的判法） | M1 1.1（在记录核对器改好之前兜底） | 3000 条随机历史里挑出 21 条、21 条都是真的不单调；1.1 那一段退回全量是 2^18 个状态 | 挑得准是量过的；「挑出来的之外没有别的不单调段」只有 120 + 7 条历史的 0（1.5），不是证明 |
| 续跑的进度文件名按「指纹 + 测试二进制 + 用例名 + 计划哈希」分，指纹只当文件里的一行核 | M2 2.1 | 按流分文件：第二条流读回 21 片、只跑 22 片，结果相同 | 量过按流分那一半；按用例名、计划哈希分是推的 |
| 片行只在调用线程上、那一片的观察者看完而没有 panic 之后才写；判红的那一趟删进度文件；续跑之后「观察者看到的状态数 = 闭式」当一道判红 | M2 2.3、2.4 | — | 推的，没实现 |
| `.cargo/`、`rust-toolchain*` 进 54 号（与乙的每条流）的登记路径；`RUSTFLAGS`、`CARGO_PROFILE_*`、`CARGO_ENCODED_RUSTFLAGS` 的值进指纹 | M2 2.1 表末行、M3 3.1 | — | 推的 |
| 乙的清单用排除法（`crates/` 整份减去另一条流自己的用例文件）写，不列举 | M3 3.1 前三格 | — | 推的 |
| `.gitignore` 的 `target/` 改成 `/target/` 与 `crates/*/target/`，或清单不按 `--exclude-standard` 取 | M3 3.1 末格 | — | 推的 |

`q9.log` 原样：

```
Q9 seq=UOOUOMSU sigma=79 sigma_states=262144 today_red=32768 sector_wise_red=0 differ=32768 elapsed_seconds=208.7
```

## 五、没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| 相交 COW 组上的不单调（四处判据 + 池级 checker 其余各条） | 随机历史 120 条（O、U、M、R、S，长 1–10），每段全部原地子集，组内全枚举（≤ 14 个写，0 组被跳过），组外 ∅ / 全集，51720 次判定 | 0 个 |
| 回收窗口置 0（C22 类变异替身） | 7 条历史，三臂 + 不单调搜索 | 不单调 0；甲二每条都红，甲也每条都红 |
| 不相交的 COW 写（落在没写过的扇区上） | 读代码（1.5 第一条） | 四处判据单调，推的 |
| I-1.8 ① 归并键相撞 | 读代码 | 要诞生代号、写序、五元组全同而载荷不同，没造出，没量 |
| R3 丢片 | 读原型与 R3 字面 | 没找到丢而不知道的读法（2.2） |
| 今天 54 号的固定脚本上的豁免链 | B、C、D、E、E18、H1、H2 | 0 个候选 |

## 六、这条腿自己的限度

- 全部数出自冻结副本的拷贝，不是入库装置。固定脚本与历史是我在内存稀疏盘上用公开入口重建的（与第一轮同一套 `Sim`），不是仓里那份落文件的 `prepare`。
- 「红」按层 0 的评估函数 `evaluate_state_for_versions` 的计数判（oracle 两遍、记录核对器两条、checker 任一条）；各条用例在计数之外另写的断言（段形状、闭式、观察者）没算进来。
- 不单调搜索用的是「便宜判定」（记录核对器 + 池级 checker，恢复只按原地子集跑一次）：这一段里有 journal 记录或根槽写时恢复会随 COW 变，这种段被当成不随 COW 变处理了——冻结副本的流里 COW 写只与系统配置槽写同段，没有这种段；若一个变异把 journal 记录并进单元段，这个搜索会漏（推的）。坐实时一律换回层 0 的评估函数。
- 取样：静态扫 3000 条、动态扫 120 条、变异替身 7 条，都是抽样不是穷举；120 条 0 个的期望本来就只有约 0.84 个（1.5）。
- 估时：q1 先跑 3 条（`q1-try1`，40 秒，都是 500–770 写的长流）与 3 条（`q1-mirror`，23.2 秒）估每条约 8–13 秒，120 条估约 20–26 分钟、实测 948.9 秒；q3 先跑 30 条 4.0 秒，估 3000 条约 7 分钟、实测 339.2 秒；σ 全枚举开跑前只用闭式算了状态数（2^18 = 262144，≤ 10⁶），没先估时，实测 598.9 与 556.9 秒。都没超过 40 分钟，没为挂钟缩取样；缩的只有 10⁶ 上限：USUSUSOUS 那条的 σ 段 30 个写（2^30 − 1 个）没全枚举。
- 批：每条命令就是一批，段数（历史条数）按上面的估时定，没设批限时；q6 那一条前台超过 600 秒被工具移到后台，照样跑完、退出 0。
- 续跑的两格是在第一轮的原型上量的；实六的实现会不一样（例如片行写在哪一步），2.3 那一格在实现上要重量。

## 七、没做什么

- 没跑任何名字带 `layer0` 的测试目标，没跑 54 号；两条流的测试二进制在 `tests/common.rs` 那一格上会不会编不过，是拿我自己的探针（同样 `mod common;`）量的。
- 没判 M4（辩方那一格）；没算甲二的逐段状态数（本地攻方那一半）。
- 没实现、没量：续跑的片行写在观察者之后、判红删进度文件；`.cargo/config.toml` 进指纹；乙清单的排除法写法。
- 产物没进 `research/results/`：写范围只给报告、模型目录与草稿目录；日志原样放在模型目录 `outputs/`（14 份），其余草稿日志（`q1-try1`、`q1-chains1`、`q2-try1`、`q3-try1..3`、`q10-try`、`m3-common-rs`、`build*`）留在 `/tmp/claude-1000/m2-layer0-scale-r2-opus/`，没入库：它们是改分组之前的试跑或编译日志，不支撑结论。
- 中途收到的消息：跑满一小时时主 agent 要求写进度，写进了 `/tmp/claude-1000/m2-layer0-scale-r2-opus/progress.md`（当时 M1 在跑回收窗口置 0 那一批）；没改任务。
- 草稿目录里的仓副本 `repo`、`repo-l7` 交回前删掉（见交回）；没做 git 写操作，没写 kb。

## 八、什么会推翻这些结论

- M1 1.1：在入库装置上照 `UOOUOMSU` 重建，第 79 段全枚举记录核对器第二条红 0 个，或 COW 取 ∅ / 全集时也红——前者说明我的重建与仓里不同，后者说明它不是「只在部分落时红」。
- M1 1.1 的「假红」定性：同一段里恢复出的版本与 oracle 不符、或按扇区判的豁免在同一段仍红——那就说明 B 的节点真被引用着，这一格是真洞而不是判据错。
- M1 1.5 的「没打中」：别的随机种子、更长的历史、或一个新变异造出非 `record` 的不单调点。
- M2 2.3：实六的实现把片行写在观察者看完之后，这一格在实现上不复现。
- M3 3.1：`tests/common.rs` 那一格在两条流的测试二进制上编得过（例如它们改成 `#[path]` 指定模块），或乙的清单改成排除法。
