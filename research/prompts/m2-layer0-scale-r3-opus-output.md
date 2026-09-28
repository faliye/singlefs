# m2-layer0-scale-r3 云端攻方腿（Opus）报告

<!-- doc-lint:not-numbers N1 N2 N3 R1 R2 R3 R4 R5 M1 S0 S1 S2 S3 S4 S5 S6 S7 S8 -->

写于 2026-09-26。攻击面：背景材料 `research/prompts/_m2-layer0-scale-r3-background.md` 第一节 N2 的三样零轮形态。前几轮判决：`research/prompts/m2-layer0-scale-r1-main-verification.md`、`research/prompts/m2-layer0-scale-r2-main-verification.md`。全部数出自冻结副本 `/tmp/claude-1000/l0scale-r1-frozen/`（开工时 `sha256sum -c` 133 个 OK）的拷贝，是副本上的数、不是入库装置；要引须在入库装置上重做。

复跑（在仓根下）：

```
bash research/prompts/m2-layer0-scale-r3-opus-model/rerun.sh /tmp/claude-1000/l0scale-r1-frozen /tmp/claude-1000/m2-layer0-scale-r3-opus-rerun
```

它把冻结副本拷两份：一份打 `crash-rs-pub-helpers.patch`（只把 `crash.rs` 里两个私有函数 `checkpoint_txg_of_the_publish_that_made_the_write`、`reuse_is_not_proven_illegal_by_the_reclaim_predicate` 改成 `pub`，逻辑不动）再放探针 `opus_r3_probe.rs`（第二轮攻方的 `opus_r2_probe.rs` 原样，加这一轮的追加，文件里「m2-layer0-scale-r3 云端攻方（Opus）追加」以下）；另一份只在 `crates/singlefs-format/tests/` 下放 `opus_r3_build_env_probe.rs`，由 `build-env-scenarios.sh` 逐个场景编跑。都经 `run-with-memory-cap.sh 8G` 与 `capped.sh ${THREADS:-6}`；两个测试目标名字都不带 `layer0`；没跑 54 号。全跑约 3 分钟。`outputs/` 里的五份日志就是用这条命令在 `/tmp/claude-1000/m2-layer0-scale-r3-opus/rerun` 上跑出来的（退出 0），与开发时的首跑逐行一致。

模型目录每个文件的 sha256（`SHA256SUMS` 原样）：

```
5e4536a7756d13603ad64f515a6bfc9c88db51bbeca541b10e035c0abfba397f  ./build-env-scenarios.sh
1be6d50d8bb31053ec6fb788e481fecb21e595496c3551c9654dc8e4e43089cc  ./crash-rs-pub-helpers.patch
4939cb18dc5f936b0c120ce96b7ba4e1dc88cc465496adde689d8a00caab570e  ./opus_r3_build_env_probe.rs
ac82f73cd5a256bf2e62204bdc56a44447601897f2aefdc4d6499e602f9f6b99  ./opus_r3_probe.rs
8ca05cddf2cfe541d5c4894d1cc1c8c3595542039e7246dbd0cffdd01076dc0c  ./outputs/build-env-scenarios.log
9c8c6dbfcf95204863da2c985acaac52739abdd141080637b2a54f3f6513553d  ./outputs/r3a-sweep23.log
5a2fca3b92343b5262a3fad29b6137544770b0321682f2032fdf0e6499e83b4f  ./outputs/r3a-uooouomsu.log
b6c0142fa7122d560e91341c8d1d8b020df1df12b836139eca28374feb595796  ./outputs/r3b.log
12999a25294f952e185d529fe6fa839a911e1c994700cafd315dc544d9dd8ab0  ./outputs/rerun-summary.log
ffbdd917f6a4bd567e9e5072780b76b513d919ae293c76733c3bd576198e1ce9  ./rerun.sh
21f4b4981d33b02a47dd11526424ae701a86ff9ccec4c1a7059efcac899dce40  ./runner.sh
fc63a7121dfb484af5786fed223320a44413f6d72eaffd290077774c0a8acb45  ./rustc-as-RUSTC.sh
9773940db93a512f4d38be53c8fe5edbc796ca81692cfb85bae129554ee9818d  ./rustc-wrapper.sh
```

## 各格判定一览

| 格 | 判定 | 一句话 | 节 |
|---|---|---|---|
| N2 ① 按扇区判，C507 那一格的错位变体 | **打中（量过），形态是「判别子观测不到」** | M1 那一形（节点 → 数据单元 → 节点）里，「只落 y」那个假红状态 X，与「B 在 x 那一槽的节点 u2、连同盖住它的数据单元 l 一个字节都没落盘」的真洞 Y（C507 那一格：后写没落盘），两份崩溃镜像**逐字节相同**（21 条历史里 21 条，差异扇区 0）。记录核对器按 D13（验证路线） 已定项 7 只拿（崩溃前镜像，记录流，崩溃后镜像）三样入参，所以判 X 与判 Y 必然一样：今天两个都红（X 是假红），按盘上字节的按扇区判两个都不红（Y 这个真洞放过了）。Y 上 oracle 与 checker 都是 0，只有记录核对器看得见它。只有把枚举器手里的持久集合也交给核对器（第二轮探针就是这么量的，已经越出 D13 的入参），才分得开这两个状态 | 1.1–1.3 |
| N2 ① C507 那一格（同偏移，钉住的那种） | 没打中 | 同一偏移的后写没落盘，七种判法都红（21/21）：C561 要求钉住的这一格分不出「按持久集合判」与「按盘上字节判」，钉住它也挡不住 1.1 那一格 | 1.2 |
| N2 ① C513 那一道 | **打中字面（量过）**，整体判定不变 | C561 那一行的字面判法（`.claude/kb/checks-owed.md` 第 480 行）没有回收谓词：C513 的三个状态上，按扇区判不带谓词就不红，带上谓词（逐个检查解释扇区的那次后写）就红。这三个状态上 checker 另有 6–7 条违例，整份层 0 照样判红 | 1.4 |
| N2 ① 反方向：今天判法自己的洞 | 顺带量到，零轮 | l 的写没落盘、只有盖住它一半的 y 落了：今天的豁免只要有一次整份在位、区间相交的后写就开脱整份副本，所以判绿（21/21，整份层 0 也绿）；七种按扇区判法都红。按扇区判在这一格比今天严 | 1.5 |
| N2 ② 收严后的 R1–R5 | 没打中「只跑一部分却报全量」（没量；续跑的实现还不存在，按原理与原型读） | 找到四处规格缺口（都是推的）：键按「流名」取、片头只记片长与片数，都认不出「同一条流、同一组状态、换了版本表」（仓里真有这样一对：`every_crash_state_outside…` 与 `one_state_slices…`），今天这一对只会互相假红；读回时一行缺字段照收（字段取 0）；判决收严后的 R4 没有第一轮 R4 的「观察者累计进进度文件」，第一条流的全量续跑后必红；续跑合进来的片是在哪份环境下跑的，标记里不记，与 ③ 叠起来就是「一半片在另一份编译配置下跑」 | 第二节 |
| N2 ③ 54 号输入指纹补的那几样之外 | **打中（量过 7 个会改行为的场景，外加 4 个对照）** | 源码与 `cargo -V && rustc -V` 两行都不变，测试二进制的行为却变了，且不进补后的指纹：仓根上层目录的 `.cargo/config.toml`（量过：关掉 `overflow-checks`、注入编译期与运行期环境变量；54 号 `--full` 的 worktree 建在 `mktemp -d` 下，往上一层就是 `/tmp`）、`CARGO_HOME` 下的 `config.toml`、`CARGO_TARGET_<三元组>_RUSTFLAGS`、`CARGO_TARGET_<三元组>_RUNNER`、`RUSTC_WRAPPER`、`RUSTC_WORKSPACE_WRAPPER`、`RUSTC`。另有 `crates/` 下指向目录的符号链接（推的） | 第三节 |

我提的改法与各修哪一格见第四节，都是「只在我的模型上量过、被攻过零轮」。

## 一、N2 ①：按扇区判的复用豁免

### 1.0 被判的对象与四种判法

- 被判的：`.claude/kb/checks-owed.md` 第 480 行 C561 那一行给的改法（「豁免改成按扇区判：一份副本缺席 ⟺ 它区间里有一个扇区上的字节不是它的，而这个扇区上没有一次更晚、已持久、字节还在的写」），即第二轮攻方探针的 `variant_claimed_state_missing_unit`；「已持久」在那个探针里取的是枚举器的 `persisted` 标志（`research/prompts/m2-layer0-scale-r2-opus-model/opus_r2_probe.rs` 第 1280 行 `persisted[later] && l.device == w.device && …`）。
- 记录核对器的入参是三样：`.claude/kb/decisions/13-验证路线.md` 第 131 行「**需要第二个输入的核对归记录核对器**，入参 `(崩溃前镜像, 记录流, 崩溃后镜像)`」；冻结副本 `crates/singlefs-harness/src/crash.rs` 第 656–657 行的文档注释也是这么写的，「在盘上」的定义是崩溃后镜像上的字节与记录流逐字节相同。持久集合不在入参里：今天的判法用第 706 行 `                && in_place(later_index)`（整份在位）代替「已持久」。
- 探针里的七种判法（`opus_r3_probe.rs` 的 `r3_missing_groups`，逐组报「发布 txg，16K 槽号」）：
  - `Today`：把第 706、714 行那一版逐份重写一遍，每个状态都与 `check_records` 的总判对账，对不上就 panic，全部跑完没有 panic；
  - `SecPers`：C561 的判法，「已持久」取持久集合，不带 C513；
  - `SecDisk`：C561 的判法，只用 D13 给的三样入参，某次后写在那个扇区上的字节与盘上相同，就算这个扇区被解释了；
  - `SecDiskRec`：只用三样入参，但「后写已持久」递归地判：后写自己的每个扇区，要么是它自己的字节，要么被更晚、同样判成已持久的写解释；
  - 各自带 `513` 的：解释那个扇区的后写还要过 `reuse_is_not_proven_illegal_by_the_reclaim_predicate`。
- 做法：只评手搭的单个崩溃状态，与 C507、C513 两条用例是同一种做法，**不做层 0 的崩溃状态枚举**。这里每条历史的闭式都远超 10^6：`UOOUOMSU` 是 3438215360；23 条历史最小 1330774424、最大 7730036949；C513 那条流 201326619（`outputs/` 里 R3A、R3B 行）。所以没有一条历史整条枚举过，第六节写了这条限度。

### 1.1 打中：错位的 C507 格与 M1 的假红，镜像逐字节相同

历史 `UOOUOMSU`，与第二轮 M1 是同一段（前缀 mkfs → 取号 → 暖机 → A → B；U 卸载重挂，O 覆盖写，M 进程重开，S 写小文件）。角色（`outputs/r3a-uooouomsu.log` 原样）：

```
  R3A roles sigma=79 u=#59(dev0 slot50262+1 txgSome(4)) u2=#61(dev0 slot50263+1 txgSome(4)) l=#362(dev0 slot50262+2 txgSome(17)) x=#574(dev0 slot50263+1 txgSome(25)) y=#572(dev0 slot50262+1 txgSome(25))
  R3A l_bytes_over_u2_range=16384 l_nonzero_over_u2_range=0 disk_before_u2_equals_l_over_that_range=true
  R3A seq=UOOUOMSU images_identical(X,Y)=true differing_sectors=0 images_identical(X2,Y2)=false differing_sectors=2
```

- **X**：前 79 段整段持久，σ 段的原地写全落，σ 里的 COW 写只落 y（两盘）。这是第二轮 q8 真值表里「只落 y」那一行，今天判红，是假红。
- **Y**：在 X 的基础上，B 的节点 u2（槽 50263，两盘）与 txg 17 的数据单元 l（槽 50262–50263，两盘）都摘成没落盘。这是 C507 那一格：u2 唯一的后写 l 与 x 都没落盘，u2 真的缺席；恢复自称走到 txg 24（≥ 4），B 的这一组两份都不在。
- l 落在 u2 那一段（x 那一槽）上的 16384 个字节全是 0（数据单元的内容约 3 KiB，其余是零填充），u2 落盘之前那一段盘上也全是 0。所以 X 里槽 50263 是 l 的零尾，Y 里是从没写过的零，两份镜像在写表碰过的每个扇区上都相同（差异扇区 0）。
- 判定（同一份日志原样，只摘四种不带 513 的判法；带 513 的与各自不带的逐行相同）：

```
  R3STATE X_only_y_landed(M1_false_red) eff=Some((5, 24)) layer0: oracle=0 oracle_ignored=0 checker_violated=0 record_root_without_record=0 record_claimed_state_missing_unit=1 red=true why=record:claimed_state_missing_unit
    R3RULE X_only_y_landed(M1_false_red) Today missing_groups(txg,slot16k)=[(4, 50263)]
    R3RULE X_only_y_landed(M1_false_red) SecPers missing_groups(txg,slot16k)=[]
    R3RULE X_only_y_landed(M1_false_red) SecDisk missing_groups(txg,slot16k)=[]
    R3RULE X_only_y_landed(M1_false_red) SecDiskRec missing_groups(txg,slot16k)=[]
  R3STATE Y_u2_and_l_never_landed(C507_cell_misaligned) eff=Some((5, 24)) layer0: oracle=0 oracle_ignored=0 checker_violated=0 record_root_without_record=0 record_claimed_state_missing_unit=1 red=true why=record:claimed_state_missing_unit
    R3RULE Y_u2_and_l_never_landed(C507_cell_misaligned) Today missing_groups(txg,slot16k)=[(4, 50263)]
    R3RULE Y_u2_and_l_never_landed(C507_cell_misaligned) SecPers missing_groups(txg,slot16k)=[(4, 50263)]
    R3RULE Y_u2_and_l_never_landed(C507_cell_misaligned) SecDisk missing_groups(txg,slot16k)=[]
    R3RULE Y_u2_and_l_never_landed(C507_cell_misaligned) SecDiskRec missing_groups(txg,slot16k)=[]
```

- 23 条历史的扫描：第二轮 `q3-sweep1.log` 里带豁免链候选的 21 条，加第二轮缩出来的 `USUSUSOUS`、`USOUSMSU`；其中 21 条找得到 u2，另 2 条（`SSUURMSMUOOOU`、`MSSSUMOOSRMORO`）里 l 盖住的另一半在 l 之前没有被认领的单元写过，没有 u2（`outputs/r3a-sweep23.log`）。21 条逐格计数（`outputs/rerun-summary.log` 原样）：

```
X_only_y:Today=21 X_only_y:SecPers=0 X_only_y:SecPers513=0 X_only_y:SecDisk=0 X_only_y:SecDisk513=0 X_only_y:SecDiskRec=0 X_only_y:SecDiskRec513=0 X_only_y:layer0_red=21
Y_u2:Today=21 Y_u2:SecPers=21 Y_u2:SecPers513=21 Y_u2:SecDisk=0 Y_u2:SecDisk513=0 Y_u2:SecDiskRec=0 Y_u2:SecDiskRec513=0 Y_u2:layer0_red=21
Yp_only:Today=0 Yp_only:SecPers=21 Yp_only:SecPers513=21 Yp_only:SecDisk=21 Yp_only:SecDisk513=21 Yp_only:SecDiskRec=21 Yp_only:SecDiskRec513=21 Yp_only:layer0_red=0
C_u_and:Today=21 C_u_and:SecPers=21 C_u_and:SecPers513=21 C_u_and:SecDisk=21 C_u_and:SecDisk513=21 C_u_and:SecDiskRec=21 C_u_and:SecDiskRec513=21 C_u_and:layer0_red=21
images_identical(X,Y)=true in 21 of 21 histories with a u2
```

  （`layer0_red` 是层 0 的评估函数 `evaluate_state_for_versions` 在那个状态上的整份判定，用的是今天的记录核对器。Y 与 X 的 21 个红全来自记录核对器：R3STATE 行里 oracle、oracle_ignored、checker_violated 都是 0。）
- M1 的另一行红（只落 x，X2）没有对应的真洞：l 的头在槽 50262，不是零，从没写过的那一片盘对不上（差异扇区 2）。所以这个打中只落在「后写只剩零尾露在外面」的那一半。

### 1.2 四句（1.1），以及同偏移那一格

- **分不分辨臂：分辨。** 这里的「臂」是记录核对器豁免的两种写法：今天的写法与 C561 的按扇区判。在 Y 上，今天判红，只看盘上字节的按扇区判（`SecDisk`、`SecDiskRec`）不红。这正是 N2 ① 要的「按扇区判放过、今天的判法抓得到的错」。`SecPers` 在 Y 上照红，但它用的持久集合不在记录核对器的入参里。甲二、全量都不涉及：X、Y 是手搭的单个状态，不是枚举出来的。
- **系统当时看不看得到判别它的东西：看不到（判别子观测不到）。** 记录核对器按定义只看三样入参，X、Y 在这三样上逐字节相同（记录流是同一张写表，崩溃前镜像同一份，崩溃后镜像差异扇区 0）。所以「X 不红（修掉 C561 的假红）」与「Y 红（C507 那一格仍红）」这两条要求，在三样入参下互相矛盾，任何只用这三样的豁免写法都只能满足其中一条。要两条都满足，得给核对器第四样入参：枚举器的持久集合。层 0（`CrashImage.persisted`）与崩溃注入两条路手里都有它，冻结副本的 `CrashImage::candidate_unit_slots`（`crash.rs` 第 590 行 `            if *is_persisted && write.device.0 == device && write.kind == StepKind::UnitWrite {`）已经在用它。但这一改就改了 D13（验证路线） 已定项 7 给核对器定的入参，要走决策改写。
- **满足判据字面哪一句：** 背景材料 N2 ① 的「造一个按扇区判放过而今天的判法抓得到的错」：Y 今天红、按盘上字节的按扇区判不红，而且 Y 上只有记录核对器红（oracle 与 checker 都是 0），所以这个错整份层 0 只靠它抓。对 C561 那一行的验收句「C507（记录核对器的复用豁免比登记的候选宽，把真洞变哑） 那一格（后写没落盘的真洞）…在按扇区判之下仍红」，Y 属于那一格（后写 l、x 都没落盘），但偏移与 u2 不同；字面钉住的那条 C507 用例是同偏移的，在 1.2 末尾那一格上七种判法都红（下面）。**也就是说，验收里点名钉住的那条用例，分不出这一格。**
- **改法在打中的格上还中不中：** 见第四节的表。简单说：`SecPers`（加上 513）在 X 不红、Y 红、同偏移那一格红、C513 那一格红，是七种判法里唯一四格都对的；它要的持久集合要作为入参交给核对器。`SecDisk` 与 `SecDiskRec` 在 Y 上不中。今天的判法在 Y 上中，但在 X 上假红。
- **Y 是不是层 0 枚举走得到的状态：走不到。** 层 0 把前面的段整段持久，u2 在第 5 段之前，摘不掉；C507 那条用例同样是手搭的（冻结副本 `second_transaction_step_zero_layer0.rs` 第 1289 行的注释写的是「手搭的状态（层 0 段枚举产生不出它…」）。在今天层 0 的枚举域里，按扇区判比今天判法松的地方只剩两处：C513（1.4）和巧合的字节相等。后者要「当前段里一个被认领的单元没落、同段里一次更晚的写也没落、而那次后写的零尾正好罩住它的非零扇区」，要同一段里有覆盖同一槽的两代写，今天的段形状上我没造出来（推的，没穷举）。

同偏移那一格（C 状态：u 与 l 两盘都没落，σ 的 COW 取 ∅，所以 u 那一槽的后写 l、y 都没落）：

```
  R3STATE C_u_and_l_never_landed_cow_empty(C507_cell_same_offset) eff=Some((5, 24)) layer0: oracle=0 oracle_ignored=0 checker_violated=0 record_root_without_record=0 record_claimed_state_missing_unit=1 red=true why=record:claimed_state_missing_unit
    R3RULE C_u_and_l_never_landed_cow_empty(C507_cell_same_offset) Today missing_groups(txg,slot16k)=[(4, 50262), (17, 50262)]
    R3RULE C_u_and_l_never_landed_cow_empty(C507_cell_same_offset) SecPers missing_groups(txg,slot16k)=[(4, 50262), (17, 50262)]
    R3RULE C_u_and_l_never_landed_cow_empty(C507_cell_same_offset) SecDisk missing_groups(txg,slot16k)=[(4, 50262), (17, 50262)]
    R3RULE C_u_and_l_never_landed_cow_empty(C507_cell_same_offset) SecDiskRec missing_groups(txg,slot16k)=[(4, 50262), (17, 50262)]
```

同偏移时，后写的头正好压在前写的头上，两个头都不是零，巧合相等不成立，所以哪种判法都红（21/21）。

### 1.3 这一格有多常见

- 用到的只有一样东西：后写露在外面的那一段是零填充，正好罩在前写那一槽上，而那一槽落盘之前也是零（稀疏盘、没写过）。数据单元的内容比槽短，这在今天的写路径上是常态：21 条里 21 条 `l_nonzero_over_u2_range=0`，数的是 `r3a-sweep23.log` 里每条的 R3A 行。
- 不止 M1 那一形：只要「节点槽 → 数据单元 → 节点槽」这种错位复用走到一半、而后写的零尾正好罩住旧节点，都是这一格。第二轮已经量过这种链在随机历史里不罕见（3000 条里 21 条）。

### 1.4 C513 那一道（`outputs/r3b.log`）

照 `second_transaction_supplement_two_record_checker_reuse_legality.rs` 的形状（它的名字不带 `layer0`）在内存盘上重建：回收窗口置 0，覆盖写两次，txg 5 的数据单元落回 txg 4 的那一对槽 50178。

```
R3B writes=99 segments=16 closed_form_full=201326619 txg=(4, 5) data_slots=(50178, 50178)
```

三个状态上各判法认定缺席的组（`outputs/r3b.log` 的 R3RULE 行，这里按行归并）：

| 状态 | Today | SecPers | SecPers513 | SecDisk | SecDisk513 | SecDiskRec | SecDiskRec513 | 层 0 的 oracle / checker 违例条数 |
|---|---|---|---|---|---|---|---|---|
| txg 5 的单元落了、记录与根没落（C513 用例那个状态） | (4, 50178) | 无 | (4, 50178) | 无 | (4, 50178) | 无 | (4, 50178) | 1 / 7 |
| txg 5 除根以外都落了 | (4, 50178) | 无 | (4, 50178) | 无 | (4, 50178) | 无 | (4, 50178) | 0 / 7 |
| 全部持久 | (4, 50178) | 无 | (4, 50178) | 无 | (4, 50178) | 无 | (4, 50178) | 0 / 6 |

- 四句：只分辨「带不带回收谓词」，不分辨按扇区判与今天的判法。核对器看得到（谓词只读写表）。满足 N2 ① 的「C513 那一道在按扇区判之下还红不红」：照 C561 的字面判法不红，把谓词带上、逐个检查解释那个扇区的后写，就红。改法：C561 那一行的验收句里已经写了「C513…那一道在按扇区判之下仍红」，但判法字面没写谓词，第二轮的探针也没搬（它的注释写着「C513 那一道…这里没搬」）。实现时要把谓词接在「解释这个扇区的那次后写」上（`SecPers513`）。
- 整份层 0：这三个状态上 checker 另有 6–7 条违例（I-2.1、I-3.1、I-3.10、I-4.8、I-5.1、I-7.4，第一个状态还有 I-7.2），所以丢了 C513，这里的整份判定也照样红，丢的只是记录核对器这第二个见证。C22 那一类违规复用里，有没有只剩记录核对器一个见证的状态，我没搜（第六节）。

### 1.5 反方向：今天判法自己的洞（零轮）

- Y′：只把 l（两盘）摘成没落盘，u2 照落。l 属于 txg 17，恢复自称走到 txg 24，所以 l 被认领。y 盖住了 l 的前一半且整份在位，今天的 `written_over_later` 只要有一次整份在位、区间相交的后写就开脱整份副本，所以 l 的后一半（槽 50263 上是 u2 的字节）没人问。结果今天判绿、整份层 0 判绿，七种按扇区判法都判缺席（21/21，1.1 的计数表 `Yp_only` 那一行）。
- 这一格与 C507 同类（被认领的单元写没落盘，唯一罩住它那一部分的后写也没落盘），今天判不出来。按扇区判在这一格上比今天严，这是它额外买到的东西。它同样是手搭的状态，层 0 的枚举走不到。

## 二、N2 ②：收严后的续跑 R1–R5

被判的是 `research/prompts/m2-layer0-scale-r2-main-verification.md` 第 31 行「**断点续跑收严**（实六实现，被攻过零轮）」那一条：R1 的键带流的名字、指纹补 `.cargo/` 等几样；R3 合并前核片数；R4 片行在观察者看完那一片之后才写，任何一片判红就删掉这一趟的进度文件；R5 照旧。实六的实现还不存在，这一节全部按条款字面与第一轮原型（`research/prompts/m2-layer0-scale-r1-opus-model/l7-resume-prototype-crash-rs.patch`）读，**没有跑**。原因有二：续跑要在一条流上枚举崩溃状态，而这个 Sim 里最短的历史（只到 A）闭式就是 67108885（第一轮判决 L5 那一行「第一条流 41 写 / 10 段、全量 67108885」），第二轮与这一轮的历史都在 1e8 以上，按取样规则不跑；第二轮已经在原型上量过续跑机制本身。

| 读法 | 会不会「只跑了一部分却报全量」 | 依据 | 标记 |
|---|---|---|---|
| 同一条流、同一组状态、换了**版本表**（或基镜像、写表内容、被判的根）：键 = 指纹 + 流名，片头 = 片长 + 片数，两者都认不出来 | 仓里今天只会互相假红，不会假绿；以后加一条「与已存的片同流同组状态、预期也是 0 违例、但判法不同」的用例，就会拿别人跑出来的绿片报成自己的全量 | 冻结副本 `second_transaction_step_zero_layer0.rs` 里，第 456 行 `    let prepared = prepare("layer0-e-fast", Script::ReuseAfterRaisingFloor);` 与第 526 行 `    let prepared = prepare("layer0-slices-merge", Script::ReuseAfterRaisingFloor);` 是同一条流，展开谓词逐字相同（第 457、535 行 `    let expand = \|_segment_index: usize, segment: &[usize]\| segment.len() < 10;`），都是 108 个状态；后者第 534 行把 B 的内容换掉（`        .content = third_content();`），预期违例 ≥ 2，前者预期 0。片长 ≥ 108 时两边都是 1 片、区间都是 [0,108)，第一轮原型的严格读法（逐行核片区间）照收 | 推的 |
| 读回的一行少了字段 | 会，悄悄的：原型读一行时从 `Layer0Tally::default()` 起步，缺的字段取 0，认不出的键跳过（patch 第 79 行 `+    let mut t = Layer0Tally::default();`，第 82 行 `+        let number = value.parse::<u64>().unwrap_or(0);`），校验和只保证这一行没被截断或改坏，保证不了序列化器写全了字段。合并函数把每个字段拆开写全，新加字段没并就编不过（冻结副本 `crash.rs` 第 955 行 `absorb_following_slice`）；但序列化器 `slice_line`（patch 第 56 行）是逐字段取值写出来的，没有这道编译期的保证。以后给 `Layer0Tally` 加一个违例计数、却忘了进 `slice_line`，续跑读回来的片上这一项恒为 0 | 推的（今天原型的 19 个字段都写全了） |
| 判决收严后的 R4 只管片行写在观察者之后、判红就删文件，没有第一轮 R4 的「观察者累计进进度文件…续跑之后观察者看到的状态数必须等于闭式」（第一轮判决第 34 行） | 不假绿，是假红：第一条流的全量用例拿观察者的累计当期望值（`first_transaction_step_seven_layer0.rs` 第 466 行 `        read_set_counts.states_judging_allocation_generations(),`），续跑之后观察者只看到新跑的片，这一条对不上，每次续跑都红，续跑白做 | 推的 |
| 「任何一片判红」按片判，而有的用例红在合并之后的断言上（精确计数、按发布分），这时进度文件不删 | 不假绿：片存的是完整计数，再跑一遍合出同一个数，照红 | 推的 |
| 被杀的那一趟没走到「跑完再算一次输入哈希」那一步（54 号第 9 行起那一道），它写下的片被下一趟接着用 | 在从 HEAD 加暂存区新建的 worktree 里，跑的过程中几乎没人改输入，现实里碰不到；但条款字面上，续跑用的片没经过它自己那一趟的收尾核对 | 推的 |
| 与 ③ 叠起来：第一趟在一份编译配置下跑了一半片，被杀；第二趟换了一份指纹认不出的配置（第三节那 7 个场景指纹都认不出）接着跑 | **会**：合起来的片数等于闭式，`exhaustive=true`，标记照写；R5 只报「续跑了几片、从头跑了几片」，不记那些片是在哪份环境下跑的 | 行为变化量过（第三节），合并是推的 |

四句（最后一行，它是这一节唯一「会」的）：只分辨续跑的写法，不涉及约简的臂。系统看得到：cargo 知道读了哪些配置，测试二进制可以现算自己可执行文件的哈希。满足的是 N2 ②「一次只跑了一部分却报全量」：这一趟只在当前配置下跑了一部分片，却报成全量。改法见第四节：把第三节补的指纹写进每一片的片行，合并时要求全部片相同。

## 三、N2 ③：54 号输入指纹补的那几样之外

指纹今天怎么算：`.claude/gate.d/stage-inputs.tsv` 第 20 行给 54 号登记 `crates/ Cargo.toml Cargo.lock`；`research/scripts/admission.py` 第 271 行用 `git ls-files -z --cached --others --exclude-standard` 列文件，第 276 行只留 `os.path.isfile` 为真的，第 287 行加 `cargo -V`、`rustc -V` 两行（cwd 是仓根）。第二轮判决第 29 行要补的是 `.cargo/`、`RUSTFLAGS`、`CARGO_PROFILE_*`、`CARGO_BUILD_*`；这一轮背景材料另列了 `rust-toolchain*` 与 `CARGO_ENCODED_RUSTFLAGS`。

量法：在冻结副本的拷贝里，`crates/singlefs-format/tests/opus_r3_build_env_probe.rs` 打一行 `OPUS_R3_ENV`，内容是溢出检查开没开、`debug_assertions`、`cfg!(opus_probe)`、编译期的 `option_env!("OPUS_R3_BUILD_ENV")` 与运行期的 `OPUS_R3_RUN_ENV`。源码不动，每个场景只改一样补过的清单里没有的东西，同时打出这一刻的 `cargo -V && rustc -V`。`outputs/build-env-scenarios.log` 原样：

```
SCENARIO S0_baseline toolchain=[cargo 1.98.0 (797e8a9bc 2026-08-05);rustc 1.98.0 (88d9e12ae 2026-08-18);] toolchain_sha256=d6562b4791e169cb compiled=1 OPUS_R3_ENV overflow_checks=true debug_assertions=false cfg_opus_probe=false build_env=None run_env=None
SCENARIO S1_ancestor_dot_cargo_config toolchain=[cargo 1.98.0 (797e8a9bc 2026-08-05);rustc 1.98.0 (88d9e12ae 2026-08-18);] toolchain_sha256=d6562b4791e169cb compiled=1 OPUS_R3_ENV overflow_checks=false debug_assertions=false cfg_opus_probe=false build_env=Some("from-ancestor-config") run_env=Some("from-ancestor-config")
SCENARIO S2_CARGO_HOME_config toolchain=[cargo 1.98.0 (797e8a9bc 2026-08-05);rustc 1.98.0 (88d9e12ae 2026-08-18);] toolchain_sha256=d6562b4791e169cb compiled=1 OPUS_R3_ENV overflow_checks=true debug_assertions=false cfg_opus_probe=true build_env=None run_env=None
SCENARIO S3_CARGO_TARGET_triple_RUSTFLAGS toolchain=[cargo 1.98.0 (797e8a9bc 2026-08-05);rustc 1.98.0 (88d9e12ae 2026-08-18);] toolchain_sha256=d6562b4791e169cb compiled=0 OPUS_R3_ENV overflow_checks=true debug_assertions=false cfg_opus_probe=true build_env=None run_env=None
SCENARIO S6_CARGO_TARGET_triple_RUNNER toolchain=[cargo 1.98.0 (797e8a9bc 2026-08-05);rustc 1.98.0 (88d9e12ae 2026-08-18);] toolchain_sha256=d6562b4791e169cb compiled=0 OPUS_R3_ENV overflow_checks=true debug_assertions=false cfg_opus_probe=false build_env=None run_env=Some("from-runner")
SCENARIO S7_RUSTC_WORKSPACE_WRAPPER toolchain=[cargo 1.98.0 (797e8a9bc 2026-08-05);rustc 1.98.0 (88d9e12ae 2026-08-18);] toolchain_sha256=d6562b4791e169cb compiled=1 OPUS_R3_ENV overflow_checks=true debug_assertions=false cfg_opus_probe=true build_env=None run_env=None
SCENARIO S0b_baseline_fresh_target toolchain=[cargo 1.98.0 (797e8a9bc 2026-08-05);rustc 1.98.0 (88d9e12ae 2026-08-18);] toolchain_sha256=d6562b4791e169cb compiled=1 OPUS_R3_ENV overflow_checks=true debug_assertions=false cfg_opus_probe=false build_env=None run_env=None
SCENARIO S4b_RUSTC_WRAPPER_fresh_target toolchain=[cargo 1.98.0 (797e8a9bc 2026-08-05);rustc 1.98.0 (88d9e12ae 2026-08-18);] toolchain_sha256=d6562b4791e169cb compiled=1 OPUS_R3_ENV overflow_checks=true debug_assertions=false cfg_opus_probe=true build_env=None run_env=None
SCENARIO S5b_RUSTC_fresh_target toolchain=[cargo 1.98.0 (797e8a9bc 2026-08-05);rustc 1.98.0 (88d9e12ae 2026-08-18);] toolchain_sha256=d6562b4791e169cb compiled=1 OPUS_R3_ENV overflow_checks=true debug_assertions=false cfg_opus_probe=true build_env=None run_env=None
SCENARIO S4c_RUSTC_WRAPPER_old_target toolchain=[cargo 1.98.0 (797e8a9bc 2026-08-05);rustc 1.98.0 (88d9e12ae 2026-08-18);] toolchain_sha256=d6562b4791e169cb compiled=0 OPUS_R3_ENV overflow_checks=true debug_assertions=false cfg_opus_probe=false build_env=None run_env=None
SCENARIO S8_back_to_baseline toolchain=[cargo 1.98.0 (797e8a9bc 2026-08-05);rustc 1.98.0 (88d9e12ae 2026-08-18);] toolchain_sha256=d6562b4791e169cb compiled=0 OPUS_R3_ENV overflow_checks=true debug_assertions=false cfg_opus_probe=false build_env=None run_env=None
```

| 场景 | 改了什么 | 二进制变了什么 | 进不进补后的指纹 |
|---|---|---|---|
| S1 | 仓根上一层目录的 `.cargo/config.toml`：`[profile.release] overflow-checks = false` 与 `[env]` | 溢出检查关了（仓根 `Cargo.toml` 第 17 行 `overflow-checks = true` 被压过去）；编译期与运行期都多了一个环境变量 | 不进：补的 `.cargo/` 是仓里那一份，git 列不到仓外。54 号 `--full` 的出路建议 `layer0_full_base="$(mktemp -d)"`（`.claude/gate.d/54-layer0-replay.sh` 第 224 行），worktree 往上一层就是 `/tmp`，任何本机用户都能建 `/tmp/.cargo/config.toml` |
| S2 | `CARGO_HOME` 指到另一个目录，里面的 `config.toml` 写 `[build] rustflags`（默认的 `~/.cargo/config.toml` 同理，今天本机没有这个文件，现查过） | `cfg!(opus_probe)` 变真 | 不进：`CARGO_HOME` 不属于补的那几族，家目录下的配置也不在仓里 |
| S3 | `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS` | 同上 | 不进（`CARGO_TARGET_*` 不在 `CARGO_BUILD_*` 与 `CARGO_PROFILE_*` 里） |
| S6 | `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER` | cargo 经它起测试二进制，能改运行环境、参数，也能根本不跑 | 不进 |
| S7 | `RUSTC_WORKSPACE_WRAPPER` | `cfg` 变真 | 不进 |
| S4b | `RUSTC_WRAPPER`，target 目录是空的 | `cfg` 变真 | 不进（`RUSTC_WRAPPER` 不带 `CARGO_` 前缀；配置里的 `build.rustc-wrapper` 走 `CARGO_BUILD_RUSTC_WRAPPER` 才进） |
| S4c | 同上，但在已经编过的 target 目录上 | **cargo 不重编，行为不变** | — |
| S5b | `RUSTC` 指到另一个编译器，target 目录是空的 | `cfg` 变真 | 不进：指纹里的 `rustc -V` 跑的是 PATH 里的 `rustc`，不是 `$RUSTC` |

- 十一个场景的 `toolchain_sha256` 都是 `d6562b4791e169cb`：指纹里仅有的两个非文件输入一个字都没变，而源码文件也没动，所以补后的指纹在这十一个场景下全都相同（文件清单那一半是按代码推的，`git ls-files` 列不出仓外的东西）。
- S4c 与 S4b 对照：旧 target 目录上 cargo 不认 `RUSTC_WRAPPER` 的变化（cargo 自己的重编判定不含包装器），所以主工作区的快档与 `--full` 的新 worktree 可能编出行为不同的二进制，而两边的指纹相同。
- 符号链接（推的）：`crates/` 下一个指向目录的符号链接，git 只把链接本身当一个条目，`admission.py` 第 276 行的 `os.path.isfile` 对指向目录的链接给 False（在草稿目录里现量过：`crates/linked isfile= False islink= True`），于是链接指向的那个目录里的内容一个字节都不进指纹。今天仓里没有符号链接（`git ls-files -s` 里模式 120000 的条目 0 个）。
- 查过、不算打中的：`*.img`、`*.qcow2` 被 `.gitignore` 第 4、5 行挡住（`git check-ignore -v --no-index` 现查命中）。但 `--full` 的 worktree 只含跟踪的文件，没跟踪的夹具在那里根本不存在，编译会报错，不会假绿。`RUSTUP_TOOLCHAIN` 与 `rustup override` 会改 `rustc -V` 的输出，所以指纹认得出。运行期的 `SINGLEFS_LAYER0_THREADS` 在 R2 之后不影响结果。`TMPDIR` 只决定镜像文件放在哪（`tests/common/mod.rs` 第 40 行 `std::env::temp_dir()`），推的：不改判定。

四句：不涉及约简的臂（这一格对今天的全绿标记与收严后的续跑同样成立）。系统看得到：54 号自己就能把这些路径与变量读出来。满足 N2 ③ 的「补的那几样之外，还有什么改了会改测试二进制的行为而不进指纹」。改法：补的清单改成按前缀全收，外加上层目录的配置文件（第四节）；只补第二轮点名的那几样，碰不到这 7 个场景。

## 四、我提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 在打中的格上 | 标记 |
|---|---|---|---|
| 记录核对器第二条的豁免按扇区判，「已持久」取枚举器的持久集合（作为第四样入参交给核对器），解释某个扇区的那次后写还要过回收谓词（探针里的 `SecPers513`） | 1.1 的 X（假红）与 Y（真洞）；1.2 同偏移那一格；1.4 C513；1.5 的 Y′ | X 不红，Y 红，C 红，C513 三个状态红，Y′ 红（21 条历史逐格：`X_only_y:SecPers513=0`、`Y_u2:SecPers513=21`、`C_u_and:SecPers513=21`、`Yp_only:SecPers513=21`；C513 三行都是 (4, 50178)） | 量过（副本，`outputs/rerun-summary.log`、`outputs/r3b.log`）。它改了 D13（验证路线） 已定项 7 给记录核对器定的入参，要走决策改写：推的 |
| 只用 D13 的三样入参做按扇区判（`SecDisk` 或递归的 `SecDiskRec`） | X、Y′、C513（带谓词时） | **Y 不红**（21/21 放过） | 量过；在三样入参之下，任何写法都同时满足不了「X 不红」与「Y 红」（镜像逐字节相同，是证明，不是取样） |
| 保留今天的判法 | Y、C | X 假红（21/21），Y′ 放过（21/21） | 量过 |
| C561 的验收里，除了同偏移的 C507 用例，另钉一条错位的：u2 与 l 都没落、y 落了（本报告的 Y），断言记录核对器第二条判它缺席；判别力自证：把「已持久」换成只看盘上字节，这条由红转绿 | 1.2 末尾说的「钉住的那条分不出来」 | — | 推的（用例没写；Y 这个状态本身量过） |
| 续跑的键或每一片的片行，带上这次枚举的身份哈希：基镜像、写表（含内容）、段、展开的段位图、版本表、被判的根，加上评估函数的名字 | 第二节第一行 | — | 推的 |
| 片行的序列化器像 `absorb_following_slice` 那样把字段拆开写全；读回时要求每个键恰好出现一次；另写一条往返用例（每个字段都非零的计数写出再读回，要逐项相等） | 第二节第二行 | — | 推的 |
| 片行带上第三节那份补全后的环境指纹，合并时要求全部片相同；标记里记下每片是哪一趟、哪份指纹跑的 | 第二节末行 | — | 推的 |
| 把第一轮 R4 的「观察者累计进进度文件」写回收严后的 R4，或者带观察者的用例不开续跑 | 第二节第三行 | — | 推的 |
| 54 号的指纹补成按前缀全收：环境变量里所有 `CARGO_` 开头的与所有 `RUST` 开头的（含 `RUSTC`、`RUSTC_WRAPPER`、`RUSTC_WORKSPACE_WRAPPER`、`RUST_MIN_STACK`，除掉明知不影响行为的几个，另开名单），外加从仓根逐层往上到 `/` 的每个 `.cargo/config{,.toml}` 与 `$CARGO_HOME/config{,.toml}` 的内容；或者 `--full` 用 `env -i` 起，只给 PATH、HOME 等几样，并把上层目录配置的有无写进标记 | 第三节七个场景 | — | 推的（行为变化量过，改法没实现） |
| `admission.py` 第 276 行对符号链接报错，不再悄悄丢掉；或者跟进链接把目录里的文件列出来 | 第三节的符号链接 | — | 推的 |

## 五、没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| 同偏移的 C507 格（后写的头压在前写的头上） | 21 条历史 × 七种判法 | 七种判法都红，放过 0 个 |
| M1 的另一行（只落 x）有没有逐字节相同的真洞 | 21 条历史 | 没有：l 的头不是零，差异扇区 2 |
| 层 0 枚举域里，按扇区判比今天松的另一种途径（同一段里两代写，靠零尾巧合相等） | 读今天的段形状 | 没造出（推的，没穷举） |
| C513 的违规复用里，记录核对器是不是唯一见证 | C513 那条流上的 3 个状态 | 不是：checker 另有 6–7 条违例 |
| 续跑：两条流、两个进程往同一份文件追加、末行半截、判红之后续跑 | 读条款与原型 | R1 分流、R3 核片数、R4 调换次序之后，没找到假绿（第二节第四、五行） |
| 续跑：仓里今天的用例有没有被别的用例存下的绿片冒充 | 冻结副本 `second_transaction_step_zero_layer0.rs` 里五处枚举调用（第 458、538、589、977、1156 行，第 538 行那一处在一条用例里按两种切法各调一次） | 只有 e-fast 与 slices-merge 那一对撞得上，两个方向都是假红 |
| 指纹：`RUSTUP_TOOLCHAIN`、`rustup override`、被 `.gitignore` 挡住的夹具、`TMPDIR`、`SINGLEFS_LAYER0_THREADS` | 读代码，加 `git check-ignore` 现查 | 都不假绿（第三节末尾那一条） |

## 六、这条腿自己的限度

- 全部数出自冻结副本的拷贝，不是入库装置。历史用的是第二轮探针的内存盘 `Sim`，靠公开入口重建，不是仓里那份 `prepare`。
- **没做任何层 0 的崩溃状态枚举**：这里每条历史的闭式都在 1.3e9 以上，C513 那条流是 201326619，都超过每次约 10^6 的上限，所以一条都没跑，全部结论都来自手搭的单个状态。数一下：`UOOUOMSU` 6 个状态，23 条历史里 21 条各 6 个，C513 那条 3 个。每个状态都跑恢复、checker 与层 0 的评估函数，也就是说，碰过的每个状态都完整判过。
- 「只看盘上字节」的两种按扇区判法是我按 C561 的字面写的，实六实际怎么实现可能不同；1.2 那条「三样入参之下两条要求互相矛盾」不依赖写法，只依赖镜像逐字节相同这一点。
- `Today` 是我把判法逐份重写的一版，每个状态都与 `check_records` 的总判对过账，没有对不上的；不过对的只是总判，逐组的细节没有别的出处可以对。
- 第二节全部是推的：实六没有实现，第一轮的原型也不是实六。
- 第三节只在 `singlefs-format` 下的一个测试上量行为变化，没有在层 0 的测试二进制上量：那两个目标名字带 `layer0`，不能跑。这些变化在 cargo 这一层，与编的是哪个 crate 无关（推的）。
- 估时与批次：先跑 `UOOUOMSU` 一条（0.8 秒），估 23 条约 20–40 秒，实测 39.7 秒；③ 与 C513 都不到 1 分钟。每条命令一批，没为挂钟缩过取样；缩的只有 10^6 的上限，也就是整条历史的枚举一条都没跑。
- 开跑前看了负载：辩方腿的一个 `cargo test`（pid 4144231）在跑，没有性能测量。

## 七、没做什么

- 没跑名字带 `layer0` 的目标，没跑 54 号，没跑 `gate.sh`；没写 kb，没做 git 写操作。
- 没判 N1（辩方）、N3（本地攻方）；没读这一轮其余几条腿的提示与产出。
- 没实现第四节的任何改法，也没有在入库装置上复跑。
- 产物没进 `research/results/`：写范围只给了报告、模型目录与草稿目录。日志原样放在模型目录的 `outputs/`（5 份），`/tmp/claude-1000/m2-layer0-scale-r3-opus/` 下是开发时的首跑日志（`r3a-try1.log`、`r3a-sweep23.log`、`r3b.log`、`cfgprobe/scenarios*.log`、`build1.log`），与 `outputs/` 逐格相同，不支撑额外的结论，没入库。
- 交回之前删掉的仓副本与编译目录，另见交回。

## 八、什么会推翻这些结论

- 1.1：在入库装置上照 `UOOUOMSU` 重建，X、Y 的崩溃镜像有差异扇区（例如实六之后数据单元的尾巴不再补零、改成带校验的填充），或者 Y 上 oracle 或 checker 红了。前者说明 Y 不再与 X 同形；后者说明 Y 不只靠记录核对器看得见。
- 1.2 的「判别子观测不到」：D13（验证路线） 已定项 7 的入参里本来就有持久集合，或者记录核对器另有别的入参能分开 X 与 Y。
- 1.4：实六的按扇区判法带上了回收谓词，C513 那条用例在它之下照红。
- 第二节：实六的续跑键里已经有枚举身份的哈希，片行带了环境指纹，读回时核字段是否写全，第一条流的全量用例续跑之后绿。
- 第三节：54 号 `--full` 用 `env -i` 起、把上层目录的配置读进标记，或者 cargo 的配置查找不再往仓根以上走。

## 九、交回前删掉的草稿（2026-09-26）

两份仓副本（`/tmp/claude-1000/m2-layer0-scale-r3-opus/repo` 800M，`…/rerun/repo` 800M），两份 ③ 的副本（`…/cfgprobe/repo3` 与 `…/rerun/cfgprobe/repo3`，各 34M），六个空的 target 目录（`target-s0`、`target-s4`、`target-s5` 各两份，各 7.1M），两个 `cargohome`（各 64K）与 `symlinkprobe`（20K），都已删掉。上层目录的 `.cargo/config.toml` 每次跑完场景当场就删了（脚本里的 `rm`）。草稿目录里只剩日志与包装脚本。
