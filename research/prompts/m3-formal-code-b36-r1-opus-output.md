# 第 36 批代码轮第一轮攻方报告（Opus，2026-10-06）

轮名 m3-formal-code-b36-r1；被判的是冻结副本 `~/.local/share/singlefs/work/m3-formal-code-b36-r1/tree/` 里 36-1b 与 36-2 两份补丁。代码引文写 `crates/…` 相对路径与冻结副本的行号；开工时核过，这一轮引到的 13 份 crates 文件在仓里工作区与冻结副本逐字节相同（sha256 前 12 位逐个比）。

## 复跑

```
bash research/prompts/m3-formal-code-b36-r1-opus-model/tools/rerun.sh <草稿目录>          # 不给段名 = 全部
bash research/prompts/m3-formal-code-b36-r1-opus-model/tools/rerun.sh <草稿目录> setup build crafted crafted2 naga   # 只跑造盘与 naga 那几段（几分钟）
```

`setup` 把冻结副本拷进 `<草稿目录>/tree`、核清单 316 行，造两份 checker 拷贝（`b36-checker-probe` = 今天的 walk.rs 加只记不判的探针；`b36-checker-pre36` = 反打 36-1b 的 walk.rs，即第 35 批的 FL 认法，同样加探针），反打 36-2 的三份 wgsl 得到 36-2 之前的内核；`build` 编两个二进制（`b36probe`、`b36naga`），核日志里四个包各有 `Compiling`。每条 cargo 与每次跑编出来的二进制都经 `scripts/hosts/run-with-memory-cap.sh`，同一时刻只起一条，线程 16。各段输出落 `<草稿目录>/out/<段>.log`，与模型目录 `out/` 里留的同名文件比（挂钟 `seconds=`、`wall=` 不比；`-FIRST` 行的原文是各线程最先遇到的那一张，只比键）。

模型目录每个文件的 sha256（`find . -type f | sort | xargs sha256sum`）：

```
4b537e495fbb1c2085a0f697bf834e660929fa15746239b6cd7d15900999d2ee  cargo/b36-checker-pre36.toml
f019dac97e76d3e92c93a63dc61cf9bd71a857dfe133657085587de1a9f90e22  cargo/b36-checker-probe.toml
6638e8bdea4bed5136c6545fb6974190db14369d74d256d801a752fdfe559d37  cargo/b36-opus-naga.toml
4135f1e6bf7c2885555e0713b48cd86f4927648698e760230da8a490ad0514d7  cargo/b36-opus-proto.toml
2b49a817235437b1418c40c7533cafb725574b8244fa4d92ef33aa72fa229f41  naga/main.rs
80fb7fa973e3f54ab02b59e46fcd5e2f6fa7e4ece7cdf898f6e71254601ee42b  out/crafted2.log
67a7168185f5fa59cb0bbee901bd474196dbf8d891270a2e063412206514797a  out/crafted.log
4a9cf84fa572d5d81e3606c09a5644fc04b4d73ab3f56e855abdc91679b97292  out/l12.log
908025013553fc313c1e697c9141cd4a1bb032e738f31d02efe05ebed5bf42db  out/l14.log
443e626c9524f0507f5e4358307aa1dc1bebed9da30423a7ed56ec5e831f9190  out/l36a.log
73437b790ea373d0e2e73ebd37a860d781f8f2b87b656465e7a9fd8d3c2c23cc  out/l36b.log
760f37ad1853276b77fd90d8c9e6e1902122e8a7c8dcdfd6c6d8c0de66d934c3  out/l36c.log
5842587fdd304c9907233ab3a78b4ac52a1cf33d822ed216f8bd6cf2018b37fb  out/l57.log
162af000ba835b7ad731c189e4056e2a5403c8b2fcae49ca33223c87944bdf36  out/l66.log
f24b10c9d9ab174ed3b6757854283a6d0cbde277b1dd31018ad850af00411187  out/layer0-counts.txt
7f4476f51121b6e4434a32eca4b97f2913361ff6491eff7eab2c780303171291  out/layer0.time
c6e31e3881db9dd650d25d12bdcaf69c6f649c4191d942081f7f19adf2aa89c8  out/mutrows-a.txt
be8384eb75f4852d9f4a7c1ed04b3f21234491f50beb3360b059d5536e116e08  out/mutrows-b.txt
7377c0ecb626bebaea1dd33288973b4cec96047b0832487ca141dc4c20cb98a8  out/naga.log
19e9169f2e3ae357f7bd8944a0c36604477c903e75c65dd580f884123eb5af1a  out/r1.log
94ce0994942a36f06759fef8ae9606b3112a2dd8c6b29ab1ee644180c03b9017  out/r2.log
99e699f4893b0e3c7038b0fc004b4295eea4346695f2956f27fedc76de23feb0  out/r3.log
0e7fd9fd8f4902e382fd93e638dca2cb60d42cbb0b680edeb3c363fbbb3e0184  out/random.time
dfc5cb6351eac79e7d2d4b269a65424a8bfcbc32c23c849d8b29e3a54172fe45  out/seed10.log
8f5026d2b819f9994cc412f2f0f8935bf776c0c3891a8ea18055ec1911c593ed  src/core.rs
e4c791d54e3a078f4182d03151fa8b6e1fd86befec28d032d550e89061daa261  src/crafted.rs
1499610b4e69cf5f430125d5d73e1bb5070bf9783a24e9d424b7ec334aba2da1  src/craft.rs
020f3ffa188a1c08036b610e9018a8dba77ad222ef816bae9885b2417c038c2a  src/drive.rs
71533d2fdaceb9581db3b9e63b95b5e16f118ce6ee17f7ce57aa4aea9d653534  src/judge.rs
7a29622f8df6c63683590bf57dd4adc42adde569ad5e92a84f81d4a6fa2edca8  src/layer0.rs
0620f65224acf5e26a759bd2ede0dca20a3fc7cda76f6aa424f07f2373951b5a  src/main.rs
68e2016cd78b0dafbe07f31d3af38823c30f97852a4bcf3ec336a1d56134c337  src/random.rs
a2749856cca45ba698caf0cab224a3a9e10dd69532f847f4d4ec565810e29ccd  src/tally.rs
9559c34573aae4d3db421b32bf9aa30857886cb10affba262bf1c47664df3f7f  tools/add_probes36.py
d6a4a258bd67622cd8a2d77e07d3af570560bc832d0248404327df0c2b67d8e7  tools/mutrows.sh
35fc490da45773796da9ce7a54d21126abc56820db566c135ef29abed4c2f390  tools/rerun.sh
dc8a9e5e5ebd5b0bf4dcb3475a6592f5db6079bc8f45c67c4e626ff75444bc9c  tools/setup.sh
```

`drive.rs`、`craft.rs` 是第 35 批攻方模型同名文件原样拷来的。`out/` 里的产物不是同一个二进制一次跑出来的：`crafted.log` 在加序号区间参数之前编的那一版上跑，`r2.log`、`r3.log`、`seed10.log` 在加 `B36_DEBUG` 打印之后，`crafted2.log` 在加 `crafted2` 那一段之后；这三处改动都没碰已有几段的逻辑（序号区间不给时就是全段，打印只在设了 `B36_DEBUG=1` 时出），所以今天的源码能把每一份都复跑出来。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| P1 R1 认的那一版与 core 施加的第一次发布是不是同一份 | 没打中（这一次抽样） | 合法历史随机取样 24 971 张镜像、层 0 全展开 123 040 个状态：R1 认出的 15 440 + 16 419 张，与 core 施加的第一次发布逐字节相同，`core-only` 0、`checker-only` 0 |
| P1 断号 / 点名单元停在第一次或后面某次 / 锚点认不定 / 所选根 txg 没有后继 | 没打中 | 合法历史上 FL 认而 core 不施加的 1 591 张（种子 10 逐张看全是前面某条没落的断号），R1 一张不认；后四支只在造的坏盘上走得到，R1 逐张与 core 同 |
| P1 `Stopped` 带第一次那一支在什么历史上走到 | 观察（不是打中） | 合法历史上 0 次：要两次以上没落根槽的发布连着进重放，今天的写侧每次发布的根槽 FUA 在下一次发布之前；只在撕掉根槽的坏盘上走到（75 张，R1 与 core 同） |
| P1 多实例 / 回退之后 | 没打中 | 随机历史里重挂换实例、管理员回退、抬 F 都在抽的动作里；层 0 那一条回退流 65 548 个状态全展开 |
| P1 I-3.10 与 I-5.4 是否仍同一个认法 | 站住（读码） | 两处都读同一个 `Option<VersionTheNextMountAppliesFirst>` |
| P1 合法历史上多报少判 | 没打中；R1 修掉 FL 的 14 次多报部分没判 | 新旧两份逐条比：只有 I-5.4 P→H 14 张，全是 FL 认了一份 core 不施加的断号发布、它的分配记录树已被下一次挂载盖掉 |
| P1 越格：I-3.1 在合法历史上红 | 不归这一轮 | 新旧同红 5 张（种子 `188088733`，管理员回退之后），交主 agent |
| P2 写第 31 个字的路径先于扫描内核开工、所有置 `H_VALID == 1` 的路径都写了它 | 站住（读码，推的） | 置 1 只有一处，前提是走读循环走到 `WALK_PHASE_DONE`，而进 DONE 的两处都在写第 31 个字那一句之后 |
| P2 走读内核 I-4.1 行为不变 | 站住（读码，推的） | 原来交回 NONE 的四处，新的 `last_record_of_the_last_publish_applied` 照旧 NONE |
| P2 扫描内核读到的那一行与 CPU 认的是同一条记录 | 站住（读码，推的） | 同一份按 (实例, 计数器) 先到先得的环、同一个下标；草稿区那一段扫描内核不写 |
| P2 观察：扫描内核超预算 | 事实更正 | naga 数：冻结副本的扫描内核展开 692 201 个表达式；背景材料写的 700 783 是反打 36-2 之后的数。36-2 让它少了 8 582 个，前后都超 400 000，36-2 的两条对拍流与两行 GPU 变异今天都编不到卡上 |
| P3 R2 两条流的新造法 | 站住（读码） | 崩在 txg 4 根槽 FUA 之前、从全落的崩溃镜像起新进程录挂载与覆盖写：每个枚举出来的状态都是合法历史 |
| P4 三处说明与 I-5.5 理由 | 站住；另记一处 kb 写回措辞 | 代码说明与判定一致；I-3.10、I-5.4 两行 kb 里「txg 比最新根大 1」那一条 R1 不再要，造盘上 R1 与 core 同认 txg + 2 那一版 |
| P5 变异行 | 站住（CPU 四行副本上量过；GPU 四行推的） | 八行锚点各命中 1 次；CPU 四行副本上基线绿、改坏红在断言上、还原后绿；GPU 四行改过的内核 naga 都校验过，但扫描内核超预算，跑不到卡上 |

## P1　R1（CPU）

### 装置

- 盘是内存稀疏盘（`SparseBlockDevice`，经第 35 批攻方的 `drive.rs`），每段历史从 mkfs 起在内存里走，崩溃 = 那一步之前的镜像叠上那一步录制流的一段前缀，新进程从那张镜像起。
- 每张镜像跑四样：`singlefs-checker`（今天的 R1）、探针拷贝（判定与它逐条相同，原型每张核一遍，`SANITY probe!=new` 一次都没出）、`pre36`（反打 36-1b 的 FL）、core 那一侧（`src/core.rs`：`choose_system_configuration`、`choose_root`、`scan_journal`、`replay_journal`）。
- core 不单独交回「第一次发布」，原型这样取：`replay_journal` 报 `prefix_applied` 条，施加的是水位之上（同实例、txg 大于所选根）按 (实例, 计数器) 键序的前 `prefix_applied` 条——取前缀的核里「水位之上」就是「record.instance.0 == root_instance.0 && record.checkpoint_txg.0 > root_checkpoint_txg.0」（`crates/singlefs-replay-prefix/src/lib.rs:3311`），核的循环从水位之上第一条起、遇到第一条不收的就停；其中第一条带末条标志的就是第一次发布的末条，照 `replay_journal` 拼新根的那一段（「NewRootSource::NewRootSegmentOfTheRecordAtKeyOrderIndex(index_in_key_order) => {」，`crates/singlefs-core/src/recovery.rs:3656`）拼成根、摊成 364 字节，与 checker 认的那一版的新根段逐字节比。施加了记录而找不到第一次发布的末条时原型断言失败，一次都没出。
- 分类：`same-bytes`（同一版、新根段逐字节同）、`checker-only`（checker 认、core 没施加）、`core-only`、`different-version`、`same-version-different-bytes`。

### 合法历史：随机取样（不是层 0 全展开）

用户动作按会话状态从一张表里抽（挂载、第一个文件、覆盖写、顺序写、删、建 inode、管理员回退、抬 F、卸载），每步四分之一的机会崩在它某次根槽写之前的一个切口上、下一步起新进程。每一步之后的镜像，与每次根槽写之前各切口的镜像都判：全落、前 j 条、缺第一条、只剩最后一条、中间缺一条（第 35 批的切口加了「中间缺一条」）；只有一条记录时只落第一份。种子底数 `0xB36_0000`，与第 35 批的 `0xB35_0000` 不重。

| 批 | 种子 | 步数 | 镜像 | R1 对 core 第一次发布 | FL 对 core | 新旧判定不同 |
|---|---|---|---|---|---|---|
| r1 | 0..111 | 20 | 8 992 | same-bytes 5 495、none 3 497 | checker-only 661 | I-5.4 P→H 4 |
| r2 | 112..223 | 20 | 8 792 | same-bytes 5 513、none 3 279 | checker-only 517 | I-5.4 P→H 6 |
| r3 | 400..447 | 40 | 7 187 | same-bytes 4 432、none 2 755 | checker-only 413 | I-5.4 P→H 4 |

原样输出：「B36R new next-mount vs core-first: same-bytes = 5495」（`research/prompts/m3-formal-code-b36-r1-opus-model/out/r1.log:118`）、「B36R pre36 next-mount vs core-first: checker-only = 661」（`research/prompts/m3-formal-code-b36-r1-opus-model/out/r1.log:125`）、「B36R new!=pre36 I-5.4 P>H = 4」（`research/prompts/m3-formal-code-b36-r1-opus-model/out/r1.log:122`）；另两批同形（`out/r2.log` 第 118、122、125 行，`out/r3.log` 第 54、58、62 行）。三批里 `core-only`、`different-version`、`same-version-different-bytes` 都是 0 次（键没出现）；R1 认出的每一张都是「与最新根同实例、txg 比最新根大 1」：「B36R new next-mount: same-instance-as-newest=true txg-minus-newest=1 = 5495」（`research/prompts/m3-formal-code-b36-r1-opus-model/out/r1.log:119`）；重放的结局只有两种：施加一次发布（与 R1 认出的张数相同）、一次都不施加——`applied publishes=2`、`stopped`、`anchor-undecided`、`no-successor` 都是 0。

FL 多认的 1 591 张逐张是什么：种子 10（`out/seed10.log`，`B36_DEBUG=1` 打的每一张 R1 与 FL 认的不同的镜像）里全是「前面某条没落、末条落了」的断号：`records-2..k`、`record-k-only`、`records-without-2`，以及崩在「只剩最后一条」之后再挂载那几步——那条孤零零的末条一直留在环里，FL 每一张都认，core 每一张都不施加。I-5.4 P→H 那 14 张都在后一种上：下一次挂载重建分配器时那次断号发布的单元没人占着，被新实例的发布盖掉，FL 认它、它的分配记录树读不全，报部分没判；R1 不认，报成立。例：「B36R-FIRST new!=pre36 I-5.4 P>H :: seed=188088330 step=11 op=Mount cut=before-root@34」（`research/prompts/m3-formal-code-b36-r1-opus-model/out/r1.log` 的 `-FIRST` 段）。这是 R1 修掉的一处多报，不是打中。

### 合法历史：层 0 全展开

前缀 7 步（同一张表随机抽，可带崩溃），后缀 1 步不崩，后缀那条录制流按层 0 的枚举域全展开（`judge_layer0_state_range` 加 `full_expansion`，与 `enumerate_layer0` 同一份 `Layer0StatePlan`、同一个 `judge_crash_image`，段内整写子集逐个枚举、含撕裂的原地覆盖写），每个状态跑上面四样。

| 种子 | 后缀 | 段长 | 层 0 状态 | R1 认出 | R1 对 core |
|---|---|---|---|---|---|
| 57 | 挂载 | [2, 12, 2, 1, 2, 2, 1, 2, 2, 1, 2] | 4 140 | 6 | 全 same-bytes |
| 66 | 抬 F | [2, 12, 2, 1, 2, 12, 2, 1, 2] | 8 223 | 6 | 全 same-bytes |
| 14 | 挂载 | [2, 14, 2, 1, 2, 12, 2, 1, 2] | 20 511 | 3 | 全 same-bytes |
| 12 | 挂载 | [2, 14, 2, 1, 2, 12, 2, 1, 2, 12, 2, 1, 2] | 24 618 | 16 401 | 全 same-bytes |
| 36 | 管理员回退 | [16, 2, 1, 2] | 65 548（分三段跑） | 3 | 全 same-bytes |

合计 123 040 个状态，`judged` 与状态数逐段相等：「B36L0-DONE seed=12 judged=24618 range=0..24618 of=24618 full=true seconds=111.6」（`research/prompts/m3-formal-code-b36-r1-opus-model/out/l12.log:3`）、「B36L0T seed=12 new next-mount vs core-first: same-bytes = 16401」（`research/prompts/m3-formal-code-b36-r1-opus-model/out/l12.log:8`）；种子 36 三段的 `range=` 接成 0..65548（`out/l36a.log`、`out/l36b.log`、`out/l36c.log` 第 3 行）。五条流上新旧两份的判定逐条相同（`new!=pre36` 键没出现），没有红。层 0 这几条流的记录段都是单条记录的发布（段长 2 = 一条记录两份），断号在这里造不出；多条记录的发布（顺序写）单元段 26 次写起，层 0 状态数过 10⁶，见「这条腿自己的限度」。

### 造的坏盘（合法历史走不到的几支）

| 形 | 张数 | R1 对 core 第一次发布 | FL 对 core | 重放结局 |
|---|---|---|---|---|
| A 之后两步用户动作 B、C（7 × 7 种组合 × 3 种宽），撕掉 B、C 写的每条根槽 | 75 | same-bytes | same-bytes | 施加 2 次 60 张、3 次 15 张 |
| 同上，最后一次发布的一个点名单元盘 0 那份翻一个字节 | 75 | same-bytes | same-bytes | stopped first=some |
| 同上，第一次发布的点名单元翻一个字节 | 75 | none / none | checker-only | stopped first=none |
| 同上，第一次发布两条以上记录时抹掉它的第一条两份 | 36 | none / none | checker-only | nothing |
| 记录全落、根槽没落，把那次发布的记录 txg 从 T+1 改成 T+2（锚点在） | 6 | same-bytes（R1 与 core 都施加 T+2） | core-only | 施加 1 次 |
| 锚点那条抄到计数器 + 1 的槽上（所选根那次发布两条带末条标志） | 3 | none / core 拒 | none / core 拒 | anchor-undecided |
| 最新根 txg 改成 u64 最大值 | 3 | none / core 拒 | none / core 拒 | no-successor |
| journal 环 256 条（在飞上限 85），一次大顺序写 84、85 个数据单元，崩在根槽之前 | 2 | same-bytes | same-bytes | 施加 1 次 |

原样输出见 `out/crafted.log` 第 553–561 行与 `out/crafted2.log` 第 13–14 行，例：「B36XT multi-stop-last new-vs-core-first=same-bytes pre36-vs-core-first=same-bytes replay=stopped first=some = 75」（`research/prompts/m3-formal-code-b36-r1-opus-model/out/crafted.log:559`）。B、C 的 49 种组合里有 72 个（三种宽合计）因 B 或 C 被拒（删在没文件时、回退目标不是候选等）没造。在飞上限那一行：86、89 个数据单元的大写被写侧拒（「B36X inflight ring=256 units=86 BigWrite not ok」，`research/prompts/m3-formal-code-b36-r1-opus-model/out/crafted.log:551`），写侧在发任何写之前按同一个上限拒（「pub fn journal_record_limit_of_one_publish(journal_ring_bytes: u64) -> u64 {」，`crates/singlefs-core/src/transaction.rs:6767`），一次发布越过上限这一形合法历史上没有。

### 读码的几格

- I-3.10 与 I-5.4 同一个认法：`check_pool_image` 算一次 `version_the_next_mount_applies_first`，I-5.4 按版本那一层直接用，I-3.10 经「version_the_next_mount_applies_first.as_ref(),」（`crates/singlefs-checker/src/walk.rs:11333`）取树表指针。两处的所选根与 I-4.1 同一个入参（「&roots[newest_index].2,」，`crates/singlefs-checker/src/walk.rs:11208` 与 `crates/singlefs-checker/src/walk.rs:11281`）。
- 新说明「/// core 照施加，第一次那一版照认（I-4.1 对这种镜像报不适用，那是 I-4.1 自己的射程）。这一版的 txg 比最新根大，根环里没有它的根。」（`crates/singlefs-checker/src/walk.rs:5871`）与上面造盘的 stop-last 一形对得上；第一次发布的末条在 `ends_its_publish` 那一支第一次收到末条时记下（「last_record_of_the_first_publish_applied.get_or_insert(record);」，`crates/singlefs-checker/src/walk.rs:8718`），停在点名单元时原样交回（「return ReplayTheNextMountWouldDo::StoppedAtANamedUnitThatDoesNotVerify {」，`crates/singlefs-checker/src/walk.rs:8703`）。
- `Stopped` 带第一次那一支在合法历史上走不到：要第二次发布也进同一次重放，第一次发布的根槽就得没落，而今天的写侧每次发布写完根槽、轮换系统配置之后才起下一次发布。上面合法历史的 24 971 张与 123 040 个状态里 `applied publishes=2` 与 `stopped` 都是 0 次，core 那一侧「施加了不止一次发布」的键也一次没出。36-1b 钉这一支的坏镜像用例与 36-2 的对拍流都是撕了根槽的坏盘，与此一致。

### 越格：I-3.1 在合法历史上红（新旧同红，不归这一轮）

r3 批里 5 张，新旧两份同红：「B36R new-red I-3.1 = 5」（`research/prompts/m3-formal-code-b36-r1-opus-model/out/r3.log:59`）与「B36R pre36-red I-3.1 = 5」（`research/prompts/m3-formal-code-b36-r1-opus-model/out/r3.log:65`）。第一张是种子 `188088733` 第 17 步 `Rollback(2)` 之后（4 GiB 宽、默认环，历史在 `out/r3.log` 那一行 `B36H seed=188088733`）。不是第 36 批的改动造成的（反打 36-1b 照红），没查它是不是 C697（那一条写的是 1 MiB 短环）那一族，交主 agent。四句：不判这一轮的臂；判别子不涉及；满足的是 I-3.1 的等式那一句；这一轮没有改法碰它。

## P2　R1（GPU，读码与 naga，都是推的）

- 置 `H_VALID == 1` 只有一处：「if (reached == WALK_REACHED_THE_HAND_OFF && undecidable == 0u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4602`）之下的「sws(H_BASE + H_VALID, 1u);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4603`），每个状态开头先「sws(H_BASE + H_VALID, 0u);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4619`）。`walk_and_hand_off` 交回 `WALK_REACHED_THE_HAND_OFF` 的两处：第 4297 行前一行刚置了 `undecidable`（「undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (18u << 8u);」，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4296`），不会置 1；另一处在主循环之后（「return WALK_REACHED_THE_HAND_OFF;」，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4307`），主循环只在「if (phase == WALK_PHASE_DONE) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3829`）那一处 `break`（第 3916、3919 行两个 `break` 在里层的 `while` 里）。进 DONE 只有两处：「phase = WALK_PHASE_DONE;」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4183`）与（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4274`）那一行，前者紧跟在「sws(H_BASE + H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED, replay.last_record_of_the_first_publish_applied);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4180`）之后，后者是 I-4.1 那次走读走完之后，而能走那次走读的也只有经过第 4180 行的那一支。所以凡 `H_VALID == 1` 的状态，第 31 个字都是这个状态这一趟写的；扫描内核在入口先看 `H_VALID`（「if (sw(H_BASE + H_VALID) == 0u) {」，`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3151`）。
- 第 31 个字原来空着：「const H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED: u32 = 31u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:734`），下一张表从第 32 个字起（「const H_BELOW_FLOOR_SLOTS: u32 = 235504u;   // 盘、槽 lo/hi」，`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:736`，与「const H_BASE: u32 = 235472u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:698`）差 32）。全仓 wgsl 里别处不写 `H_BASE + 31`。
- 扫描内核读的那一行：重放读的那一份环在「const C_REPLAY_RECORDS: u32 = 253936u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:750`），每行 4 个字，下一张表「const C_APPEARANCE_PLACEMENTS: u32 = 255984u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:764`），中间 2 048 个字 = 512 行，正好是「const MAX_RECORDS: u32 = 512u;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:149`），收第 513 条之前就置 `undecidable`（第 3604 行），越不了界。扫描内核引用到的草稿区表，除 `C_REPLAY_RECORDS`（只在第 1103 行算行地址）之外，起点都在 17 504..227 280（`C_` 开头，交接段之前）或 290 544 起（`D_` 开头五张），加交接段本身，没有一张落在 253 936..255 984 里（逐个常量查的）；走读内核在第 4180 行之后只走 I-4.1 那一版（上下文 1 的表都在 159 200 之下），也不再写这一段。扫描内核把下标换成行地址（「return C_REPLAY_RECORDS + replay_index * C_REPLAY_RECORD_WORDS;」，`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1103`），行里第 0 个字是 `collect_the_records_the_replay_reads` 写的版本号（「sws(row, s.version);」，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3615`），`checker_record_of_row` 照它取字节（「let version = sw(row);」，`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:278`）。环按盘的次序先到先得，与 CPU `records_the_replay_reads` 同一个取法，所以拿到的是同一块盘上的同一份。
- 走读内核 I-4.1 那一处：原来交回 NONE 的四处（读环判不了、锚点认不定、txg 没有后继、停在点名单元上）新的「最后一次」字段照旧 NONE，停在点名单元那一处只多交第一次的下标（「return ReplayTheNextMountWouldDo(NONE, last_record_of_the_first_publish_applied);」，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3764`）；I-4.1 那一阶段只看最后一次那个字段（第 4181 行），行为不变。
- naga：两个内核 36-2 前后都过 naga 的解析与校验（与 `crash_judge_gpu.rs` 里那道校验同一组 `ValidationFlags::all()`、`Capabilities::all()`），展开后的表达式：

```
B36G kernel=walk-before-36-2 naga=valid inlined_expressions=319119 budget=400000
B36G kernel=walk-after-36-2 naga=valid inlined_expressions=319139 budget=400000
B36G kernel=scan-before-36-2 naga=valid inlined_expressions=700783 budget=400000
B36G kernel=scan-after-36-2 naga=valid inlined_expressions=692201 budget=400000
```

（`research/prompts/m3-formal-code-b36-r1-opus-model/out/naga.log` 第 1–4 行；「after」是冻结副本原样，「before」是冻结副本的三份 wgsl 反打 36-2。）背景材料「根环线报：各线合入后扫描内核 `crash-judge-checker-scan` 展开 700 783 个表达式、超预算 400 000，GPU 对拍编之前就拒」（`research/prompts/_m3-formal-code-b36-r1-background.md:19`）里的 700 783 与这里反打 36-2 之后的数逐位相同，冻结副本自己的是 692 201：36-2 删掉扫描内核那两圈找记录的循环，少了 8 582 个；前后都超过「pub const LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL: u64 = 400_000;」（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:208`）。推的：那个数多半是在 36-2 合入之前的树上量的；谁推过线归根环线，这里只给数。后果：36-2 两条对拍流、两行 GPU 变异与 34-1、35-1 两行被替换的 GPU 变异，今天都编不到卡上，GPU 那一侧的结论只能是推的。

## P3　R2

读码：新造法录到 (1, 4) 那次根槽写之前停，从这一段写全落的崩溃镜像（`crash_state_devices` 带 `every_write_landed`）起新进程录可写挂载与实例 2 的覆盖写，两段接成一条流。`writes_and_segments_with_stream_indexes` 把末尾没被屏障收口的那一段也留下（`crates/singlefs-harness/src/memory_pool.rs` 的 `if !current.is_empty()` 那一处），所以 txg 4 的三条记录在流里自成一段；层 0 枚举出来的状态要么是第一段的一个前缀加记录段的一个子集（崩在根槽 FUA 之前），要么是第一段全落再加挂载那一段的一部分（崩溃之后的新进程），都是合法历史，挂载读到的是流里那三条记录。原造法里录制时挂载读到 (1, 4) 的根、W 写错、根槽之后的系统配置轮换多出来的那三处，新造法里都不在：根槽写与它之后的轮换都在被截掉的那一截里。流的断言钉着挂载择 (1, 3)、施加到 (1, 4)、前缀 3 条，录错就当场红（只是这条用例在 checker 档、而且扫描内核超预算，今天跑不到）。全零树表指针那条改成挂载之后在流尾另起一段改记录：名字说的「由记录施加出来的那一版全零」在这一段全落的状态上在。没在副本上逐字节比（第 35 批量过的 W 与系统配置槽两处），这一格是读码。

## P4　R3

- FL 那段说明（第 5865–5874 行）与 R1 的实现逐句对得上，造盘的 stop-last、stop-first、gap-first 三形各自印证一句。
- I-5.5 的新理由「或数据指针的落点全在池里没有的盘上」：判法在数据指针收集之后只对「for device in &devices {」（`crates/singlefs-checker/src/walk.rs:5139`）逐盘判，落点全在池外时这一版一次 `judge` 都不调，走到这一句；但它前面还要过分配记录树那两关（种类 3 条目恰好一条、树读得全），过不了的那一版先记进部分没判。所以「三样都走到这一句」只在那两关都过时成立；整条判定的优先次序（违例、部分没判压过不适用）让报出来的仍是部分没判，不会把它说成「没有数据单元」。措辞可收，判定不受影响。
- kb 写回（不是代码）：I-3.10 射程 ④（`.claude/kb/invariants.md` 第 152 行）与 I-5.4（同文件第 199 行）写的认法带「与最新根同实例、txg 比最新根大 1、带提交标记、根环里没有它的根」几条。R1 不再看「大 1」：锚点在时链首只要计数器接得上，造盘 txggap 上 R1 与 core 都施加 T+2 那一版，FL 与 kb 那几条字面都不认（「B36XT txggap new-vs-core-first=same-bytes pre36-vs-core-first=core-only replay=applied publishes=1 = 6」，`research/prompts/m3-formal-code-b36-r1-opus-model/out/crafted.log:560`）。合法历史上 R1 认出的 31 859 张（随机 15 440 + 层 0 16 419）都是「大 1」，两种写法今天分不开。第 35 批判决第三节第 3 条已把这两行列进写回，照 R1 写时把「大 1」那一条改成「照 I-4.1 那份重放施加的第一次发布」。四句：不分辨臂（R1 与 core 同）；判别子观测得到（记录头的 txg）；满足的是 kb 措辞与实现不一致，不是判据打中；这一轮没有跑前条款给的改法。

## P5　变异行

`crates/mutations.tsv` 里点名「下一次挂载先施加的那一版」的八行：36-1b 追加的第 3020、3021 行与替换的第 963、2927 行（CPU），36-2 追加的第 3022、3023 行与替换的第 2855、2934 行（GPU）。

- 锚点：八行在冻结副本对应文件里各恰好命中 1 次（CPU 四行用 python 数，GPU 四行由 `b36naga` 数：「B36G row=3022 anchor_hits_in_the_file=1」，`research/prompts/m3-formal-code-b36-r1-opus-model/out/naga.log:9`）。
- CPU 四行（副本上量过，副本是冻结副本的拷贝 `~/.local/share/singlefs/work/m3-formal-code-b36-r1-opus/mut`，自己的编译目录）：点名的测试先在没改的副本上跑、全过；逐行改坏、只跑那一条，四行都红在断言上（`panicked at crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `assertion` 那一行），日志里 `singlefs-checker` 都重编了；每行改完照原文件还原、sha256 对上；最后再跑一遍全过。原样：「B36M tag=row963 rc=101 compiled_checker=1 result=test result: FAILED. 0 passed; 1 failed;」（`research/prompts/m3-formal-code-b36-r1-opus-model/out/mutrows-a.txt:2`），第 2927 行同形（`out/mutrows-a.txt:4`），第 3020、3021 行见 `out/mutrows-b.txt` 第 2、4 行；基线与还原后见各文件第 1、6 行。第 963 行的锚点今天落在 I-4.1 与下一次挂载那一版共用的重放里，改坏它 I-4.1 也一起少了反向链那一关；点名的那条用例照红在它自己的断言上（「B 那条记录进不了前缀：只该红 I-8.6」那一句），不受影响。
- GPU 四行（推的）：改过的内核各经 naga 解析与校验都过（`out/naga.log` 第 6、8、10、12 行），展开后扫描内核那两行是 692 206 与 700 779、走读内核那两行是 319 143 与 319 139；扫描内核超预算，对拍用例在编内核之前就拒，四行今天证不了红。读码：第 3022 行的替换文交回的是行地址（`return R_RECORDS + i * RECORD_WORDS;`），与新调用方 `checker_record_of_row` 要的同一种，改坏之后确是「只看带末条标志那一条」的 FL 认法；第 2934 行的 `i + 1u - open_publish_records` 写在 `open_publish_records` 已为这一条加过 1 之后，取到的是这次发布第一条的下标，与它名字说的「取第一条合条件的」一致。

## 没打中的形状

| 形 | 取样 | 结果 |
|---|---|---|
| R1 认的那一版对 core 施加的第一次发布（合法历史，随机取样） | 272 段历史（20 步 224 段、40 步 48 段），24 971 张镜像，三种宽 | 认出 15 440 张全 same-bytes；core-only、different-version 0 |
| 同上（层 0 全展开） | 5 条流 123 040 个状态（后缀挂载 3、抬 F 1、管理员回退 1） | 认出 16 419 个全 same-bytes；新旧判定逐条相同 |
| 断号（前面某条没落、中间缺一条、只剩末条） | 随机取样里的切口 | FL 认 1 591 张、R1 0 张、core 0 张 |
| 两次以上没落根槽的发布连着重放、停在第一次或后面某次的点名单元上、第一次断号 | 造盘 261 张（B、C 放开扫 7 × 7 种用户动作 × 3 种宽） | R1 与 core 逐张同 |
| 锚点认不定、所选根 txg 没有后继 | 造盘各 3 张 | R1 不认、core 拒挂 |
| 锚点在、链首 txg 跳一格 | 造盘 6 张 | R1 与 core 同认（kb 字面不认，见 P4） |
| 一次发布记录数在在飞上限上下 | 环 256 条，84、85、86、89 个数据单元 | 84、85 两张 R1 与 core 同；86、89 写侧拒 |
| 合法历史上 R1 多报 / 少判 | 上面全部合法镜像 | 新旧判定只差 I-5.4 P→H 14 张（R1 修掉 FL 的多报），没有一张多出或少了红 |
| GPU：置 `H_VALID == 1` 的路径、第 31 个字、草稿区重叠、I-4.1 行为 | 读码 + naga | 都站住（推的） |

## 这条腿自己的限度

- 层 0 只跑到单条记录的发布。80 个种子（前缀 7 步、后缀 1 步）先只数状态（`out/layer0-counts.txt`）：39 个过 10⁶ 不跑（多条记录的顺序写都在这里，单元段 26 次写起），13 个只有 1 个状态，9 个后缀是卸载、`writes_and_segments` 要录制入口区间才收带卸载记号的根槽写（C557 那道检查）、原型没录入口而拒了，19 个在 10⁶ 之内、合计 1 708 681 个状态。按负载下量到的每秒约 130–170 个状态（种子 14：20 511 个 158.9 秒），全跑约 3 小时，超过 40 分钟，缩到 5 个：挂载 3 条、抬 F 1 条、管理员回退 1 条，共 123 040 个状态；缩前缀后的状态数与估时就是上面两个数。留下的每条流崩溃状态一个不落（`full=true`，种子 36 分三段、`range=` 接满）。全腿原型跑的量合计约 15 万个状态与镜像（层 0 123 040、随机 24 971、造盘 281、试跑 2 632，另有种子 10 的细看重跑），远在 10⁷ 之内。
- 多条记录发布上的断号只在随机取样的切口上量，不是层 0 全展开：切口取「前 j 条、缺第一条、只剩最后一条、中间缺一条」，没取任意子集，也没取「只落一块盘那一份」与撕裂的原地覆写。
- 随机批的大小按试跑定：32 段 × 20 步用 49.45 秒（16 线程），据此每批 112 段 × 20 步（实跑 112.6、149.7 秒）与 48 段 × 40 步（169.6 秒）。批都没设限时。
- core 的「第一次发布」是原型从 `prefix_applied` 与键序推回去的（理由与出处见 P1「装置」），core 没有直接交回它；推法错了的话 same-bytes 那一列就不作数。旁证：随机与层 0 里 R1 认出的每一张都是施加一次发布，那时第一次就是最后一次，与 core 交回的根逐字节同（`I-4.1 replayed vs core-last` 那一行同数）。
- 一次抽样：P1、P2 的「没打中」只算这一次。
- GPU 全是读码与 naga，没上卡；扫描内核超预算，卡上也跑不了。
- P3 只读码，没在副本上把新造法的流与合法历史逐扇区比。

## 没做什么

- 没跑重型测试：checker 档测试、GPU 对拍、`crates/mutations.tsv` 整表都没跑；GPU 四行变异只做了锚点与 naga。
- 没判正推腿那几格，没读禁读清单里的文件，没替主 agent 采纳。副本上量的数（CPU 四行变异、原型的全部计数）不是入库装置上的数。
- 没提代码改法；P4 那一条 kb 措辞（I-3.10、I-5.4 两行的「大 1」）是写回时的建议，只在我的模型上量过、被攻过零轮。
- I-3.1 那 5 张没往下查。
- 草稿目录 `~/.local/share/singlefs/work/m3-formal-code-b36-r1-opus/` 里留了 `out/`（原始日志）与 `mutrows.sh`、`walk36-1b.patch`、`wgsl36-2.patch`、`walk.rs.orig`；两份仓副本（`tree/`、`mut/`）、两个编译目录（`target/`、`target-mut/`）与反打出来的 `wgsl-pre36-2/` 交回前删掉，`tools/rerun.sh setup build` 会重建。
