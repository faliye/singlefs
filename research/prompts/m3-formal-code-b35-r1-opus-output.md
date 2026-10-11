# 第 35 批代码轮第一轮攻方报告（Opus，2026-10-06）

轮名 m3-formal-code-b35-r1。立场：假设第 35 批的改法是错的，造合法历史让新判法误红、造坏实现让它放过。重点 P1、P2、P3；P4–P7 只顺带看。
代码引文的行号取冻结副本 `/tmp/claude-1000/m3-formal-code-b35-r1/tree/`（清单 316 行，开工时 `sha256sum -c` 全过）；写报告时主工作区的 `crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 已被别的会话改过（行号与冻结副本不同），以冻结副本为准：`cite-check.py --root /tmp/claude-1000/m3-formal-code-b35-r1/tree --root <仓根>` 23 处全对上（kb 两处只在仓根里）；不给 `--root` 时这三份文件里的 10 处行号对不上（原文都在，只是行号移了），加 `--unchanged-since` 开工时刻则列成没判。
副本上量出的数只在攻方原型上量过，不算入库装置上的数。

## 复跑

```
bash research/prompts/m3-formal-code-b35-r1-opus-model/tools/rerun.sh <草稿目录>            # 全部段，10 线程约 22 分钟
bash research/prompts/m3-formal-code-b35-r1-opus-model/tools/rerun.sh <草稿目录> build crafted s256   # 只跑一部分
```

`build` 段：把冻结副本拷进 `<草稿目录>/tree`（先 `sha256sum -c` 清单），拷出五份 checker（`b35-checker-pre35` = walk.rs 反打改动全文里 35-1 的 walk.rs 那一段、加只记不判的探针；`b35-checker-probe` = 今天的 walk.rs 加探针；`mutr` / `mutn` / `mutz` = 三处 CPU 变异），经内存包装 `cargo build --release`，核日志里有 `Compiling b35-opus-proto`。每段经 `scripts/run-with-memory-cap.sh 8G` 与 `capped.sh 10`；8G 超过峰值表两倍，脚本里带 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`（峰值表记的是 208M）。结果与 `out/*.txt` 逐行比（`seconds=` 是挂钟不比）。

模型目录 `research/prompts/m3-formal-code-b35-r1-opus-model/` 每个文件的 sha256：

```
f146807fc23200e724b6de9ab48ad50277b2e99c5168fc2f56d9cb33f75a761d  ./cargo/b35-checker-mutn.toml
b3191060a52bc8437618a784db86b2d9b1a9c30ca2d07c58cecd623664ad0b84  ./cargo/b35-checker-mutr.toml
1c0dd173a2cb53ebe619a7e76719bfcf192ef4a64d8b4d84df3169998ab739d4  ./cargo/b35-checker-mutz.toml
de54487d2242d3b9b2f9ad165c1ed73dcc5a376d8b4e68e60431e14ae515db23  ./cargo/b35-checker-pre35.toml
0644b780ab4343665d95cd6762e822f76c41aad34aefd36f639088fb0bbd1534  ./cargo/b35-checker-probe.toml
1f438e188877f16de198dc02321f5cc606cd177a44fad643d3a010063e411285  ./cargo/b35-opus-proto.toml
f6aa61643637eea2281be592f5f6a55964240ae048210b60f75a78556c5ca830  ./out/crafted.txt
2c8b7e86d54ce63307c7222d4f7941aed2e427a8e5828bc11b863b6337a73d14  ./out/l18.txt
0cf604574b47a66e1d0f2ed67323b099498cf92d47eccb286a3ccd334d18ba80  ./out/l43.txt
8656903fd035808c3ca1690680b14d3460f61764a691a79f408d07cfadbae958  ./out/l45.txt
7e415f0ac849e1c8113dea1e8c9c4661dc38cd88bf9c6d8774e142e0a3735f66  ./out/layer0-seed-counts.txt
9ee1e5451a703f11a4f8edeb000e367871c15e5fd89344c00a852a6c15582085  ./out/r1.txt
61aca081641c31e529ea2c7eb587a2b0f7f863c06016b6c25efdc4e27e9be20b  ./out/r2.txt
0d51ed867cbb6cb62e22c77cb2b406c23078ce95b9896457b63d09c2c4d375ad  ./out/r3.txt
69d8595cd6e196988441a61e0a2c5869dc46de5cf7bcf196128d3b7e792d5d2a  ./out/smallring-1536.txt
a530408bd6f80654a5b0a933b44ee8fbbcefa1f00a8d33be79214e1cc4ab335a  ./out/smallring-256.txt
2a5adfb71dba111b6b450b7172c56e01663e51260a5b30c6c47adb48baa2bc1f  ./src/core.rs
fb4eeaf32a8553929b421cad0d16c0a63065846a7d93c1e790c96a2867af339a  ./src/crafted.rs
1499610b4e69cf5f430125d5d73e1bb5070bf9783a24e9d424b7ec334aba2da1  ./src/craft.rs
020f3ffa188a1c08036b610e9018a8dba77ad222ef816bae9885b2417c038c2a  ./src/drive.rs
f7002caeae4baa3a5c97b39581f8cdd8ff56b08888507db30aacda97fcd1ba56  ./src/judge.rs
0766b75f070f01404a84fd7fff312182e2426e84268a52c456635af2da68b8d7  ./src/layer0.rs
0ab9bc3c87d86cf4d4d3b53ab5a05fca5baef905f325412f1424df4ec0fdafab  ./src/main.rs
a40a1969828b6101c63528634ddb16c4d24265506db69be58412e2545777e3cb  ./src/random.rs
6eca2726afcf1aab30475a90a8a01dfe20d874ecbf3708b59ee36590e47b5ac9  ./src/smallring.rs
a74f124a7c11ab4c8953185d74a3f7dcc462cbbdc8cf672a032a004e27676397  ./src/tally.rs
280989f702a8eea7826795244f411dc9049dee4554f1a4782bcd4f7393f5a12c  ./tools/add_probes.py
0539bc9ea5c3cc3014364ef13306e843bd6700d94f78d7c58b8ec21cd50f9e2c  ./tools/make_mutant.py
ec1ef9c344a2c78092ea88905cf7996cd7c7eb6d447e65fbe9d2fd3340bfa2ea  ./tools/rerun.sh
c79b89675434f8a4a3b351737de89ccb275a5ee84f95594cdc8c325e5a1b45e2  ./tools/setup.sh
```

原型怎么判一张镜像（`src/tally.rs` 的 `note_image`）：新 checker（冻结副本的 `singlefs-checker`；层 0 里直接取 `judge_crash_image` 那一份）、probe 拷贝（核它的判定与新 checker 逐条相同，不同记 `SANITY`）、pre35 拷贝各判一遍；core 那一侧照恢复的公开入口（`choose_system_configuration`、`choose_root`、`scan_journal`、`replay_journal`）算施加之后的根，拼成 364 字节新根段（`src/core.rs`），与 checker 认的「下一次挂载先施加的那一版」「I-4.1 重放施加出来的那一版」逐字节比；「由记录施加出来的那一版」与历史里每次挂载之前照恢复算出来、记进账本的那一份逐字节比。

## 各格判定一览

| 格 | 判定 | 依据（节） |
|---|---|---|
| P1-a 两类版本取末条之后与 core 施加的逐字节同一份 | 没打中：26 269 张随机历史镜像、53 352 个层 0 状态，两边都认出的版本 0 处字节不同 | P1 第 1 条 |
| P1-b 「末条没落就不认」改了哪些状态、该不该改 | 打中（不分辨臂，说明与行为不一致）：末条落了而前面有一条没落的发布，core 断号不施加，新旧两份 checker 都仍认它（新 1 122 张、旧 2 251 张）；FL 只去掉「末条没落」那一半 | P1 第 2 条 |
| P1-c 「有末条取末条、没有退回先到的」退路走不走得到 | 走得到（合法历史，量过）：journal 环 6 MiB 与 1 MiB、根环 S = 8 时，环转一圈盖住那次发布的末条而首条还在，64 个记录段子集里 15 个走退路；取的那一份与 core 当初施加的逐字节相同 | P1 第 3 条 |
| P1-d GPU 两处与 CPU 同一组版本、同一取法 | 没打中（读码，推的） | P1 第 4 条 |
| P1 越格：journal 环转过「由记录施加出来的那一版」的记录之后 I-3.1 误红 | 打中（合法历史，量过；新旧 checker 同红，不是第 35 批引入） | P1 第 5 条 |
| P2-a 全零树表指针在合法历史上误红 | 没打中：79 621 张合法镜像上全零的根与记录 0 条 | P2 第 1 条 |
| P2-b I-4.1 的射程与更贴的不变量 | 站住；一处措辞：红的是「重放施加出来的那一版」，重放施加两次发布、第一次全零时不红（坏镜像上 core 照常挂上，两边一致） | P2 第 2 条 |
| P2-c 由记录施加出来的那一版全零之后各处处置自洽 | 没打中；kb I-9.28 那一行的旧说法待写回（判决已列） | P2 第 3 条 |
| P3-a 全零分支红位与 CPU 逐位相同 | 没打中（读码，推的） | P3 第 1 条 |
| P3-b 四条新对拍流判不判得出它名字说的那一形 | 没打中：四条流对应的 CPU 变异在流上都有判定不同的状态（量过 CPU 一侧，GPU 推的） | P3 第 2 条 |
| P3-c 「拿掉根槽写」的构造对不对 | 打中（构造与合法历史逐字节不同，测试注释那句不成立；判定不受影响）：实例 1 那一行的 W 与系统配置槽两处 | P3 第 3 条 |
| P4 Q4 新理由 | 顺带：合法历史上走不到（79 621 张里 0 次），只有坏镜像走得到 | P4 |
| P5、P6、P7 | 没攻 | 没做什么 |

## P1　FL

### 取样

| 段 | 历史 | 镜像 / 状态 | 挂钟 |
|---|---|---|---|
| r1 | 种子 0–119，每段 20 步，三种盘宽轮换（单元区 256 / 384 槽小盘与 4 GiB） | 9 396 张 | 164 s |
| r2 | 种子 120–239，每段 20 步 | 9 448 张 | 180 s |
| r3 | 种子 240–289，每段 40 步 | 7 425 张 | 192 s |
| l43 / l18 / l45 | 层 0：随机前缀 7 步之后接 1 步（抬 F / 两次挂载），`judge_layer0_state_range` + `full_expansion` 全展开，含撕裂的原地覆盖写 | 12 330 + 20 511 + 20 511 个状态 | 111 / 157 / 164 s |

随机历史的用户动作按会话状态抽（`src/random.rs` 的 `draw_op`，顺序写比第 34 批的表多抽两格，多条记录的发布要它），不写死后缀；每步四分之一的机会崩在那一步某次根槽写之前的一个切口上，再由下一步（同样随机）起新进程。每次根槽写之前判这几张：记录全落；记录那一段只落第 1..j 条（j < k）；只落第 2..k 条；只落第 k 条；只有一条记录的发布只落一份。几条记录同一段（单元 → 屏障 → 记录 → 屏障 → 根槽），段内任何子集都在层 0 枚举域里。
层 0 的种子从 60 个里按状态数挑（`out/layer0-seed-counts.txt`）：≤ 约 2.5 万个状态、带挂载或抬 F 的后缀。顺序写那一步的单元段有 20–30 次写（2²⁰ 起），全展开超过约 10⁶，没跑；多条记录的发布在层 0 里只由随机历史那几个切口代表（缩了：历史条数从计划的 600 段缩到 290 段，层 0 从 5 段缩到 3 段，按先跑的 40 段 63 s、2 000 个状态 14 s 估的挂钟，见「这条腿自己的限度」）。

### 1. 取末条之后与 core 逐字节（P1-a，没打中）

三段随机历史合计（`out/r1.txt`–`out/r3.txt` 的 `B35R` 行相加）：

```
images = 26269
new next-mount vs core: same-bytes = 16890      new next-mount vs core: checker-only = 1122      new next-mount vs core: none = 8257
new I-4.1 replayed vs core: same-bytes = 16890  new I-4.1 replayed vs core: none = 9379
new version-applied-only-by-records: same-bytes-as-core-applied = 19300   new version-applied-only-by-records: unledgered = 169
pre35 next-mount vs core: same-bytes = 16890    pre35 next-mount vs core: checker-only = 2251    pre35 next-mount vs core: none = 7128
new!=pre35 I-5.4 P>H = 12    new!=pre35 I-9.28 P>P = 11457    new-red I-3.1 = 17
```

层 0 三段合计（`out/l43.txt`、`out/l18.txt`、`out/l45.txt`）：53 352 个状态，next-mount 与 I-4.1 重放都是 `same-bytes = 32805`、`none = 20547`，由记录施加出来的版本 `same-bytes-as-core-applied = 49254`，新旧判定除 I-9.28 部分没判的措辞外一条不差，新 checker 一条不红。`same-version-different-bytes`、`DIFFERENT-bytes-from-core-applied`、`core-only` 三类在全部样本里 0 次；`SANITY probe!=new` 0 次。
`unledgered`（169 次）是 checker 认出、账本里没有的版本：账本只记恢复施加出来的那几版；读 seed=188022814 那一段（`B35H` 行）推的是根曾经落过、后来被根环转掉的那一版，没逐字节比——推的。
合法写者每条记录写同一份新根段（`crates/singlefs-core/src/transaction.rs:8711`「前 N − 1 条一条一个事务」，同一个循环里七段都取同一个 `root`），取首条与取末条字节相同，这一格本来就分不出取法；FL 的判别力只在坏写者（第 34 批 s3 / s4）上，见 P3 第 2 条。

### 2. 「末条没落就不认」改的是哪一半（P1-b，打中：不分辨臂）

**历史**（`out/r1.txt` 第一处 `checker-only`）：seed=188022785，`SequentialWrite` 那一步切成 2 条记录，切口 `records-2..2-of-2`（首条没落、末条落了），新 checker 认下一次挂载先施加的那一版 (2, 6)，core 所选根 (2, 5)、一条都不施加：

```
B35R-FIRST new next-mount vs core: checker-only :: seed=188022785 step=3 op=SequentialWrite(210111) cut=records-2..2-of-2@36 :: checker=Some((2, 6)) core=Some(((2, 5), None, None))
```

每一步对到的被判语句：
- 一次发布的每条记录都带提交标记，只有最后一个事务的末条之外的每条各成一个事务（`crates/singlefs-core/src/transaction.rs:8711`「前 N − 1 条一条一个事务」）。所以旧的 `.find` 只看提交标记时首条就合条件；FL 加上末条条件（`crates/singlefs-checker/src/walk.rs:5614`「record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0」）之后，首条单独落的那一半不再认。
- core 要一次发布从首条起连续走到末条才施加（`crates/singlefs-core/src/recovery.rs:3517`「才整体施加；前缀停在一次发布中间（末条没到、断号」）。首条没落是断号，core 不施加。
- checker 认这一版只判末条那一条自己的反向链，前一条不在盘上就放行（`crates/singlefs-checker/src/walk.rs:7211`「.is_none_or(|previous| record.back_chain == previous.chain_value_the_next_record_must_carry)」）；断号与发布边界本来就不复刻（`crates/singlefs-checker/src/walk.rs:5573`「断号即止、点名单元验证、发布边界不复刻」）。
- FL 那句说明只说了一半（`crates/singlefs-checker/src/walk.rs:5569`「末条还没落的那次发布」）：末条落了、前面断号的那次发布 core 同样不施加，这里照认。

量出来的分布：新 checker `checker-only` 1 122 张，旧 2 251 张；差的 1 129 张就是「末条没落」那一半。新 checker 在这 1 122 张上判的是一版 core 不施加的版本；样本里这些镜像上没有因此多出的红（新旧判定只差 `I-5.4 P>H` 12 张：旧 checker 认了一版树表读不出的「下一次挂载先施加的那一版」，报部分没判；新 checker 不认它（推为那次发布没落末条、它的单元已被后来的挂载盖掉，没逐槽核），报成立，例 `seed=188023071 step=14 op=Mount`）。
定点再造一次（`out/crafted.txt` 的 `B35E` 段）：首条没落、末条落了之后挂载、再写两次，挂载那几次发布的每个切口上，那次没施加的发布自己写的单元一个都没被盖掉（`unapplied-publish units overwritten=[]`），I-3.10、I-5.4 都成立——这条路上 I-3.10 误红要等新实例复用那几个槽，而认它的窗口在新实例第一条根落盘时就关了，样本里没走到。

四问：
1. 分不分辨臂：不分辨。FL 与取先到的那条（pre35）在这些状态上都认、都判同一份字节；病根在两臂共用的「只看一条记录、不复刻断号」那个认法上。
2. 判别它的东西 checker 看不看得到：看得到。首条在不在盘上、同一次发布的序号是不是从 1 连到末条，记录头里都有；`replay_the_next_mount_would_do` 已经把这几关都复刻了（I-4.1 用它）。
3. 满足的是哪一句的字面：不是哪条不变量判错，是 35-1 那段说明「I-3.10 与 I-5.4 都经这一个函数认这一版，同一个条件」与「core 不施加，这里也不认」只在末条没落那一半成立；背景第三节 P1 问的「checker 认的那一版与 core 恢复真施加的那一版」在这一半上是「core 不施加、checker 认」，不是字节不同。
4. 改法碰不碰得到：FL 碰不到这一半（量过：新 1 122 张仍在）。改法 R1（我提的，见文末改法表）：「下一次挂载先施加的那一版」改取 I-4.1 那份重放施加的第一次发布——推的，没实现。

### 3. 退路走不走得到（P1-c，走得到；取的那一份与 core 相同）

退路是这一句（`crates/singlefs-checker/src/walk.rs:7810`「环里读不到带末条标志的那一条时退回先到的那一条」）。要走到它，那一版要仍被认作「由记录施加出来的那一版」（四条件都成立，第 ④ 条是 `crates/singlefs-checker/src/walk.rs:7848`「let its_units_are_not_reclaimable_yet = oldest_valid_root_txg < record.checkpoint_txg;」），同时它的末条已经读不出、首条还在。记录按计数器落槽（`crates/singlefs-core/src/journal.rs:204`「let slot_in_ring = (counter - 1).checked_rem(ring_slots).expect(」），计数器全池接着走（`crates/singlefs-core/src/mount.rs:6135`「计数器全池接着走」），所以覆盖按计数器次序来、首条先被盖；首条还在而末条读不出，只有一种：环转一圈回来的那次发布在同一段里同时盖首条与末条的槽，层 0 取「盖末条那几次写落了、盖首条那次没落」的子集。默认 768 MiB 环要 196 608 条记录才转一圈，那时根环 24 条根早换过一轮、第 ④ 条已不成立；短环才走得到。

**历史**（`src/smallring.rs`，`out/smallring-1536.txt` 与 `out/smallring-256.txt`；4 GiB 两盘，根环 S 取 mkfs 第一版的 8（`crates/singlefs-format/src/lib.rs:279`「pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;」），journal 环 6 MiB（同 `crates/singlefs-harness/src/history.rs:162`「const SMALL_DEVICE_JOURNAL_RING_BYTES: u64 = 6 << 20;」）或 1 MiB，mkfs 都收）：

1. mkfs、挂载、第一个文件；顺序写三个数据单元（B，计数器 4–6，三条记录），崩在 B 的根槽写之前。
2. 新进程挂载：恢复施加 B（账本记下 core 施加的那一份新根段），实例 2 写行与两次暖机。
3. 一次切出很多条记录的大顺序写（每个数据单元一条记录）三次、再一次凑数，把下一条记录的计数器推到 4 + 环的条数（6 MiB：500、500、500、30 个单元；1 MiB：80、80、80、10）。这时 B 仍是由记录施加出来的那一版（`B35S-BASE ... same-bytes-as-core-applied = 1`）。
4. 再写一个三单元的文件：它的三条记录正好落在 B 的三个槽上。只取这一步的记录段：更早的段全持久、六次记录写（两盘各三次）任取一个整写子集，64 个状态。

```
B35S-RECORDS images = 64
B35S-RECORDS new version taken from a record without the last flag (退路) = 15
B35S-RECORDS new version-applied-only-by-records: same-bytes-as-core-applied = 63
B35S   mask=110000 fallback=[(1, 4)] versions=[(1, 4)] ring=[(1, 4, 4, false), (1, 5, 4, false)]
```

两种环长逐行相同（15 / 64 走退路）。取到的那一份与 core 当初施加的那一份逐字节相同（合法写者每条记录同一份新根段）。这条退路在合法历史上走得到，但走到时取首条与取末条没有字节差；坏写者（首条写旧）落在这些状态上时退回的就是旧的那一份——与第 34 批之前同一个结局，FL 在这一格上不比旧取法差也不比它好。
没跑的：这一步的单元段 30 次写（1 MiB 环）/ 34 次写（6 MiB 环），全展开 1 073 742 561 / 17 179 869 921 个状态，超过约 10⁶，没跑（`B35S-SKIP full layer0 states=…`）；记录段的撕裂变体没造。

### 4. GPU 两处（P1-d，没打中，推的）

- 拼 `R_DATA`：同键已有一条、它不带末条标志而这一条带，就换上这一条（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3833`「if ((existing.flags & 1u) == 0u && (r.flags & 1u) != 0u) {」）；记录数组按计数器升序插入（`checker_insert_record`），同一 (实例, txg) 的几条在数组里与 CPU 的 (实例, 计数器) 键序同序，状态机与 CPU 的 `SegmentTakenForTheVersion` 一样：先占先到的、带标志的一到就换、之后不换。
- 扫描内核取下一次挂载先施加的那一版：多了末条条件（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1086`），取数组里第一条合条件的；CPU `.find` 在 (实例, 计数器) 键序上取第一条，同实例里两边同序。
- 两处都只是认法，P1-b 那一半（末条落了、前面断号）GPU 与 CPU 同样照认（推的）。`checker_insert_record` 按计数器去重、不分实例，与 CPU 按 (实例, 计数器) 建键不同，那一形 kb 记为第一版两盘池上不可达，不是第 35 批引入，没攻。

### 5. 越格：journal 环转过那一版的记录之后 I-3.1 误红（合法历史，量过；不是第 35 批引入）

同一段历史接着走：第 4 步的记录全落（`mask=111111`）之后，B 的三条记录都被盖掉，checker 不再认出 B，B 换下、还在 defer 里的单元没人引用，I-3.1 红；再推一圈（`after-one-more-lap`）照红：

```
B35S-RECORDS-FIRST new-red I-3.1 :: lapping record-subset mask=111111 :: 盘 0：记账的已分配 Some(52641792)，遍历全部有效根得到 52510720（其中隔离豁免 0、F 之下仍在 defer 的 0）；机理：根环槽数 24、最新根 txg 11、环里自证过的根槽 11 个、最老的自证过的根 txg 0、遍历的候选根槽 11 个、并进遍历的由记录施加出来的版本 0 个、被实例
```

差 131 072 字节 = 8 槽。新旧两份 checker 同红（`new!=pre35` 里没有 I-3.1），不分辨 FL。这是欠账表 C697（已分配统计在短环加大文件上误红）那一形：C697 写的是 1 MiB 环；6 MiB 环（单元区 384 / 256 / 240 槽那几档小盘用的环长）加每次 500 个单元的顺序写同样走到，交主 agent 看要不要补进 C697 的参数说明，不在这一轮判。随机历史里另有 17 张 I-3.1 红（新旧同红，都在抬 F 之后的镜像上，例 `seed=188022925 step=15 op=RaiseFloor(26)`），像第 34 批判决「越格线索」那一族，没核是不是同一形。

## P2　Q1 甲（全零树表指针）

### 1. 合法历史上误红（P2-a，没打中）

79 621 张合法镜像（随机历史 26 269 张、层 0 53 352 个状态）上树表指针整条全零的根与记录 0 条（`tally` 的 `all-zero` 键一次都没出现；`grep -c all-zero` 六份输出都是 0）。写全零树表指针的路只在坏镜像上。

### 2. I-4.1 的射程（P2-b，站住；一处措辞）

造盘（`out/crafted.txt`，`src/crafted.rs` 的 C、D 段）：

| 镜像 | 新 checker 红 | pre35 红 | mutz 红 | core 只读恢复 / 可写挂载 |
|---|---|---|---|---|
| C1 前：新池新建文件那次发布的记录树表指针全零，根槽没落 | I-4.1 | 无 | 无 | Failed UnitUnreadable 槽 0 / 拒 |
| C1 后：同上，根槽也落了、根记录里也全零 | I-2.1 I-4.8 I-7.2 I-7.4 | 同新 | 无 | Failed UnitUnreadable 槽 0 / 拒 |
| C2 前：三条记录的发布 B 记录里全零、根槽没落 | I-4.1 | 无 | 无 | Failed UnitUnreadable 槽 0 / 拒 |
| C2 后：B 干净地被挂载施加、实例 2 覆盖写一次，之后把 B 的三条记录改成全零 | I-2.1 I-3.1 I-4.8 I-7.4 | I-3.1 I-3.5 | I-3.1 I-3.5 | FileRead (2, 8) / 挂上 |
| D：抹掉 txg 4 的根槽，txg 4、5 两次发布都只在记录里；txg 4 记录全零、5 完好 | 无 | 无 | 无 | FileRead，施加到 (1, 5) / 挂上 |

- I-4.1 判的是重放之后那一版（`.claude/kb/invariants.md@I-4 事务与崩溃一致性 —— 类`「所选根（重放之后那一版）指向的树全部可完整遍历」），实现取重放施加的最后一次发布的末条拼出那一版，另起走读、判定丢掉（`crates/singlefs-checker/src/walk.rs:8437`「用一份另起的走读走一遍（判定丢掉，那些归别的不变量）」）。只施加一次发布时，那一版就是「下一次挂载先施加的那一版」，C1 前、C2 前两张都只红 I-4.1、两种挂载都拒，结局同向。
- D 那张：先施加的那一版全零、最后那一版完好，新 checker 不红，core 也照常挂上（core 只用最后那次发布的末条拼根，不读中间那一版的树表）。所以背景第二节与 35-1 报告那句「下一次挂载先施加的那一版树表指针全零时红 I-4.1」只在重放施加一次发布时成立，准确的说法是 walk.rs 说明里的「重放施加出来的那一版红 I-4.1」。D 要抹根槽才造得出；合法写者根落了下一次发布才开始写（推的），样本里重放施加了几次发布没有单独计数，这一条只算措辞。
- 更贴的不变量：I-2.1 的「被引用」按回退候选集里的根算，下一次挂载才施加的那一版不在里面；I-7.2 管最新根；I-4.8、I-7.4 管候选集。射程里有这一版的只有 I-4.1。没找到更贴的。

### 3. 由记录施加出来的那一版全零之后各处自洽（P2-c，没打中）

C2 后那张逐条（`out/crafted.txt`「C2-applied-only-by-records-all-zero-end」）：I-2.1、I-4.8、I-7.4 红（主走读照 core 跟过去，与根环里的候选根同一结局）；I-3.1 红（这一版自己还在 defer 里的单元没走到）；I-5.4、I-5.5 部分没判、点名「树表指针全零」；I-9.5、I-4.7、I-4.6、I-3.4、I-9.29 不适用，理由「走读到的某一版树表指针全零（core 照它读盘 0 槽 0、读不出）：那一版的删除态射程里判不了」；I-9.28 成立（只比根环里的根）。处置与根环里的候选根全零那一形同（判损坏的走主走读，按版本重读的几条报部分没判 / 判不了），没找到自相矛盾的。
kb 那一行还写着旧处置（`.claude/kb/invariants.md@I-9 inode 树结构 —— 类`「由记录施加出来那一版里的树表指针全零，主走读仍当这一版没有树表、不记走读失败」，同一行还有「GPU 判器的走读对根记录里全零的树表指针没有跟过去」）；第 34 批判决第三节第 3 条已把它列进写回批，不另算打中。
C2 后那张 core 照常挂上而 checker 红四条：core 不再读这一版（它没有根槽，回退的目标取自根环），这是用户 Q1 选甲「两类都判损坏」的结局，照定案，不攻。

## P3　Q3 甲（GPU 走读内核）与四条对拍流

### 1. 全零分支的红位（P3-a，没打中，推的）

GPU 分支读两条位置条目、记一次走读失败、不走映射根（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3139`「_ = w_read_referenced_unit(tree_table_pointer, NODE_BYTES);」）；`w_read_referenced_unit` 每条位置条目判一格 I-2.1 并在对不上时加 `w_mismatches`，判定只在主走读那一份进违例位（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:793`「if (w_judging && !holds) {」）。逐类对 CPU：最新根 → 主走读 I-2.1、`newest_failures` → I-7.2、I-4.8、I-7.4；别的候选根与 `R_DATA` 那几版 → I-2.1、I-4.8、I-7.4；被抛弃的根（上下文 1、不判）→ 走读失败 → I-7.4；I-4.1 那一版（上下文 1）→ 走读失败 → I-4.1，I-2.1 不进位，CPU 那份另起的走读判定也丢掉。读码看逐位相同，GPU 没跑。

### 2. 四条新对拍流判不判得出它名字那一形（P3-b，没打中；CPU 一侧量过，GPU 推的）

对拍流 GPU 与 CPU 逐位比；对应的 GPU 变异要在至少一个状态上让 GPU 与 CPU 不同才红。我在 CPU 上造了三处同形的变异（`tools/make_mutant.py`：`mutr`、`mutn` 与 `crates/mutations.tsv` 第 35 批两行 FL 变异同一改法，`mutz` 是 GPU Q3 那一行在 CPU 的对应），看同形的状态上新 checker 与变异判红的集合不同没有：

| 流（测试名缩写） | GPU 变异 | CPU 变异 | 状态 | 新 checker | 变异 |
|---|---|---|---|---|---|
| …next_mount_version_carries_the_previous_root… | 扫描内核取第一条 | mutn | B 的六次记录写 64 个子集 | 39 个子集与变异不同；全落时 {I-5.4} | 全落时 {} |
| …applied_only_by_its_records…previous_root | 拼 R_DATA 取先到 | mutr | 挂载与覆盖写之后（两种造法各一张） | {} | {I-3.1, I-3.5} |
| …root_and_of_its_record_is_all_zero | 全零直接返回 | mutz | C1 前 / C1 后 | {I-4.1} / {I-2.1, I-4.8, I-7.2, I-7.4} | {} / {} |
| …records_of_a_version_applied_only_by_them…all_zero… | 全零直接返回 | mutz | C2 后 | {I-2.1, I-3.1, I-4.8, I-7.4} | {I-3.1, I-3.5} |

四条都有判定不同的状态，测试里断言的红（第三条要 I-2.1、I-7.2、I-4.1，第四条要 I-2.1、I-4.8、I-7.4）在新 checker 上都出现。全流展开没跑：挂载加覆盖写那一段 17 170 485 个状态、新池新建文件那一段 16 777 228 个，都超过约 10⁶（`out/crafted.txt` 的 `B35F-SKIP`）。第三、四条我造的是同形的镜像（C1 自己录流、改记录与根槽写；C2 走合法崩溃再干净挂载、挂载之后改记录），不是测试里那条流本身。

### 3. 「拿掉根槽写」的构造（P3-c，打中：构造与合法历史不同；判定不受影响）

被判的话：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:7753`「挂载在录制时读到的是 txg 4 的根；拿掉根槽写之后，恢复施加那三条记录重建出的是同一版」与下一行「挂载写出的行与之后的发布不变。」（7754），构造是录完之后删掉 B 的根槽写（同文件 7818「flow.writes.remove(root_of_the_three_unit_publish);」）。

在原型里两种造法各造一遍（`src/crafted.rs` 的 B 段，三单元顺序写 B = (1, 4)）：合法造法 = 崩在 B 的根槽写之前、新进程挂载、覆盖写；对拍流造法 = B 的根照常落、新进程挂载（读到 B 的根）、覆盖写，之后从流里拿掉 B 的根槽写。末态逐扇区比（`out/crafted.txt`）：

```
B35X B-clean crash-image vs constructed-base differ: {"dev0 system_configuration_slot": 1, "dev1 system_configuration_slot": 1}
B35X B-clean legal-end vs constructed-end differ: {"dev0 journal_record": 5, "dev0 root_record_fua": 3, "dev0 system_configuration_slot": 2, "dev0 unit_write": 1, "dev1 journal_record": 5, "dev1 root_record_fua": 1, "dev1 system_configuration_slot": 2, "dev1 unit_write": 1}
B35D B-clean legal-end(a) vs constructed-end(b) dev0 offset=824180736 kind=unit_write ranges=[(11, 13), (89, 92), (149, 149)] a=[[83, fc, 8d], [b6, 61, bb, 77], [04]] b=[[92, d1, 0d], [16, 46, c8, 27], [00]]
```

- 差的那个单元是实例表那一片：码 3 单元的记录从偏移 136 起（`crates/singlefs-format/src/lib.rs:44`「pub const PACKED_UNIT_RECORDS_OFFSET: u64 = 136;」），第一行的种类 1 + 实例代号 4 + 所选根 txg 8 之后是 W（`crates/singlefs-core/src/instance_table.rs:39`「writer.put_u64(self.applied_transaction_high_water);」），偏移 149 正是第一行 W 的低字节：合法造法 4，对拍流造法 0。W 取恢复施加的最大事务号（`crates/singlefs-core/src/mount.rs:6168`「applied_transaction_high_water: journal.maximum_applied_transaction,」）；录制时挂载读到 B 的根、什么都没施加，所以是 0。这一片的校验和变了，写行那次发布的记录（点名项校验和）、之后各条记录的反向链、引用这一片的根都跟着变——「挂载写出的行与之后的发布不变」不成立。
- 拿掉根槽写的基镜像比合法崩溃镜像多了 B 那次发布在根槽之后轮换的系统配置槽（两盘各一槽，偏移 147、155–158、469 三处不同）：根槽没落而系统配置已轮换，持久顺序里走不到（根槽 FUA 之后才轮换）。
- 判定：合法造法与对拍流造法的末态上，五份 checker 各自判红的集合都相同（干净那一对全是 {}；首条写旧那一对新、mutn、mutz 是 {}，pre35 与 mutr 是 {I-3.1, I-3.5}），core 两种挂载都挂上、读到 (2, 8)。所以对拍那条流仍判得出它要判的 R_DATA 取法（P3-b），只是它的状态不是一段合法历史的状态；挂载之前、记录全落而系统配置已轮换的那批状态上，有没有因此多出 CPU 与 GPU 同红的判定，没逐状态核（全流 17 170 485 个状态，超过约 10⁶，没跑）。
- 第四条流（记录全零）同样是拿掉根槽写造的：挂载之前三条记录已全零，那时 core 两种挂载都拒（`out/crafted.txt`「B35X C2 mount over all-zero records → refused Recovery(UnitUnreadable { slot: SlotNumber(0) })」），流里接着的那次挂载是录制时在干净的记录上做的；挂载之后那批状态没有哪段历史走得到。与 `checker_known_bad_images.rs` 挂载之后改记录的造法同类，作为坏镜像族成立，测试说明里写的形（同文件 7914「挂载之前三条都落了的状态上它是下一次挂载施加的那一版，重放走不完（I-4.1）」）在 C2 前那张上量到了。

四问：不分辨 FL 的哪一臂（是对拍装置的构造问题）；判别它的东西 checker 看得到（W 与系统配置槽都在盘上），但今天没有不变量判 W 与重放施加的事务号对不对得上；满足的是测试说明那一句的字面；改法 R2（合法造法录流）在我的模型上末态判定与原造法相同（量过 CPU 一侧），对 GPU 推的。

## P4　Q4 新理由（顺带）

新理由在「认出来的那几版都没有数据单元」时才出现。合法写者建树只有第一个文件这一条路，空文件也写一个数据单元（`out/crafted.txt`「Q4-empty-first-file」那张 I-5.5 成立）；79 621 张合法镜像上 I-5.5 只出现过成立（随机 7 963 + 层 0 53 352）与「一版都没认出来」那一句（随机 18 306），新理由 0 次。它只在坏镜像（extent 树根指针写零）上走得到，与 35-1 的坏镜像一致。另读码：认出来的那一版的数据单元全落在池里没有的盘上时也走到这一句（说的是「没有数据单元」，实际有）——只有坏镜像造得出，推的，没造。

## 改法表

只在我的模型上量过、被攻过零轮。

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| R1：「下一次挂载先施加的那一版」改取 I-4.1 那份重放（`replay_the_next_mount_would_do`）施加的第一次发布的末条，I-3.10、I-5.4 与 GPU 扫描内核同用 | P1-b：末条落了而前面断号、core 不施加的 1 122 张 | 推的（没实现）；FL 在这几格上不起作用（量过：新 checker 仍 1 122 张） |
| R2：对拍流第二、四条改成合法造法录流（录到 B 的根槽写之前停，从崩溃镜像起新进程录挂载与覆盖写；第四条在挂载之后改记录写） | P3-c | 末态判定与原造法相同（量过，CPU 一侧）；GPU 推的 |
| R3：35-1 说明与背景那句改成「重放施加出来的那一版全零红 I-4.1」，FL 那句补「末条落了而前面断号的那次发布 core 也不施加，这里照认」 | P1-b 说明、P2-b 措辞 | 推的（文字） |
| I-3.1 那一形 | P1 第 5 条 | 不是我提的：C697 已有用户定的 RCW；本轮不改 |

## 没打中的形状

| 形状 | 取样 | 结果 |
|---|---|---|
| 合法历史上 FL 让新 checker 多红或少判（对 pre35） | 随机 290 段（20 步 240 段、40 步 50 段，三种盘宽）26 269 张；层 0 三段 53 352 个状态 | 判定只差 I-9.28 部分没判的措辞（11 457 张）与 I-5.4「部分没判 → 成立」12 张；没有一张多红或少红 |
| checker 认的版本与 core 施加的字节不同 | 同上，加小环 128 个记录段子集 | 0 次 |
| 多条记录的发布只落一部分（前 j 条、后 k−1 条、只落末条），之后崩溃再挂载 | 随机历史的切口与崩溃续跑；定点 E 段（三单元发布只落后两条 / 只落末条 / 只落首条，再挂载、写一到两次，两种盘宽） | 新 checker 不红；没施加的那次发布的单元在新实例第一条根落盘之前没被复用 |
| 坏写者：一次发布首条写成上一版的新根段（第 34 批 s3 / s4） | 造盘 A 段 64 个记录子集、B 段两种造法 | 新 checker 取末条，s3 红 I-5.4、s4 不红，与 core 一致；变异 mutn / mutr 都回到旧结局 |
| 合法历史里全零树表指针 | 79 621 张 | 0 条 |
| 全零的三类版本与 core 同不同向 | C1、C2、D 共 5 张 | 同向；由记录施加出来那一版全零时 checker 红而 core 挂上，是用户定的甲 |
| GPU 全零分支、R_DATA 与扫描内核的取法 | 读码 | 与 CPU 同（推的） |

## 这条腿自己的限度

- 一次抽样：上面「没打中」各格只算这一次观测（`.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算`）。
- 层 0 只跑了三段（12 330、20 511、20 511 个状态），后缀是抬 F 与两次挂载，里面没有多条记录的发布：顺序写那一步的单元段 20–34 次写，全展开 10⁶ 到 10¹⁰ 个状态，按「一段历史自己超过约 10⁶ 就不跑」没跑。多条记录的发布只由随机历史每次根槽写之前的五种切口与 A、小环两处的整写子集代表，不含撕裂的那几种、也不含两盘只落一份的组合。
- 缩了什么：计划随机 600 段、层 0 5 段（含 65 548 个状态的回退那一段与 24 618 个状态的一段），按先跑的 40 段 20 步 63 s、层 0 2 000 个状态 14 s（10 线程）估，全量约 45 分钟；缩到随机 290 段、层 0 3 段，实跑约 16 分钟。每批段数：随机三批 120 / 120 / 50 段，层 0 每段一次跑完。本腿原型跑过的崩溃状态合计约 5.4 万个（层 0 53 352、记录子集 192），镜像约 2.9 万张（含试跑 3 181 张），远低于约 10⁷。
- 四条对拍流的全流展开没跑（1 677 万到 1 717 万个状态）；P3-b 只在同形镜像与末态上比了 CPU 与 CPU 变异，GPU 一侧全是推的。
- `unledgered` 169 次没与 core 逐字节比（推为根曾落过、后来被根环转掉的那几版）。
- 盘宽只轮换三档、journal 环只在小环那段改过；根环 S 只取 8；两盘里一块盘读不出的形状没造。
- core 那一侧用的是恢复的公开入口重算施加之后的根，没走可写挂载的那一份（可写挂载只在造盘段各跑一次看结局）。

## 没做什么

- P5（GPU 恢复内核择根五段）、P6（B4 取样点、新池新建文件用例、829 行注释）、P7（变异行锚点与基线）没攻。
- 没跑任何重型测试：checker 档测试、GPU 对拍、`crates/mutations.tsv` 整表都没跑；harness 档测试也没跑（35-1 的十行 harness 变异没在副本上施加）。
- 没改仓里除报告与模型目录之外的任何文件；冻结副本只读，编译都在草稿目录里那份拷贝上。
- 草稿目录 `/tmp/claude-1000/m3-formal-code-b35-r1-opus/`：交回前删掉了里面的仓副本 `tree/` 与编译目录 `target/`（复跑用 `tools/rerun.sh` 重建）；留下的是 `out/`（各段原始日志，已拷进模型目录 `out/`）与 `model-src/`（模型源码，已拷进模型目录）。
