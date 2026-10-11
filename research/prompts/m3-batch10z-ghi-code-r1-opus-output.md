# m3-batch10z-ghi-code-r1 攻方腿（opus）报告（2026-10-08）

数都是在冻结副本 `~/.local/share/singlefs/work/m3-batch10z-ghi-code-r1/tree/` 的拷贝上量的，是副本上的数，不是入库装置上的数。

## 复跑与模型文件

复跑（仓根下）：`SINGLEFS_TOOLS=$PWD bash research/prompts/m3-batch10z-ghi-code-r1-opus-model/rerun.sh <新工作目录>`，末尾每份汇总一行「<名> 全同 / 不同」。rerun.sh 我没有整条跑过（前台时限 240 秒、子 agent 不起后台），里面每一步都是我在草稿目录里分步跑过的同一条命令；本机约 12 分钟加编译。

模型目录各文件 sha256（`find . -type f | sort | xargs sha256sum`）：

```
b55429f7e6c9800ab7ce4186bd800029a10af34cb75a8d31543499f1e2eae0ab  out/iso-b3.txt
2179321475f931a3aa67af252addfccc4cfe199b4d42b48bfea080a4c4743aff  out/k11-raw.log
aae06ae9c2cb3ef31916c2641a1168882146eafd302a00e337a2ee4f1cf65967  out/k11.txt
6cd6b8f62a6a0123deeb84e55845b9c6fd881f2e0ab23da830dffa76071d8cd6  out/mutants.txt
8a54cd96aa0e1619f730c267d6a6cdeabd20adaec81645e38be17b22c57e2335  out/raw-try1.txt
cb315e376e658c2320bfff1b5c719c241e6ee5d1fee58ea4da2f2caa259f203f  out/raw-try3.txt
3c1b7aa37d123c5a7f54f5a287735423bc2fd070a37e14828541ceb3a2e7cac3  out/raw-try5.txt
58828c31b1e51797136f1061892e9cbd8ef0b2d38f1f91bb81ae2201c456b98d  out/summary-b1.txt
a1be117208e773a81707456b8a929c691d141b7ad6da7b21b9bdbbca3a9c5f73  out/summary-b2.txt
645114254ee567405d2bc705e99e392d59ecc61a2169839debca7375cf591aeb  out/summary-b3.txt
b16106ee719e027bad907b946573f32d3c68882e4e72ea5af7ce4f0df52cb17c  rerun.sh
34147466e10d754d543e4353a49c7a3aecfe634142b2470e202f36c5f0fda78f  scripts/agg.py
a0dbff8e1cf103ecb6d3bb468b26f51519fe8ec6d361d9868e661f3141fcfe61  scripts/iso.py
ffeaa6917f6f77a4508ff771a87a08614a8af08cdec575fd2858f642e2d4b993  scripts/mut.py
0ba235718825ef9a2003fb73574c10f501a8a6c3ce7c2e1347e63d395d3d4e44  src/opus10zghi-arms.patch
95e638f3fbdaf6f7bccf919209c913aadc1eae797c3d839ef75cdd33df44309c  src/opus10zghi-probe/Cargo.toml
b94159a06763f53775b74759c681fc542bd3f74d6a73abff849bce0062cc0798  src/opus10zghi-probe/src/lib.rs
8abccb5273661ec7b23ebbbbeb42457d417258d82a1d29297efdb47c531ac15a  src/opus10zghi-probe/tests/opus_k21_chain.rs
32d717b9639494c922bf102f4a91dd44e87272a547bbfd1f4e76d5c07a830394  src/opus_k11_poll_race.rs
```

`out/summary-b2.txt` 的原始日志里，`OPUS_PERSIST=1` 那几件的臂名是我在草稿目录里用 `sed` 改成 `-persist` 后缀再汇总的；`scripts/agg.py` 现在按文件名里的 `-persist-` 自己加后缀，rerun.sh 不再改日志。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K2-1 丙罩不罩得住 | **打中** | R 与所选那一版 P 之间还有一次发布 Q 时，R 从 Q 继承的单元既不是 R 写的、也不在 P 的账里；它们只记在 C 里 Q 那一槽，Q 那一槽先被这个进程写掉、解开，R 那一槽要晚一个 txg 才写掉。写掉 R 那一槽的那次发布，根落盘之前那个层 0 状态 I-7.4 红，违例的根是 R。丙 60 段里 9 段红、h2 60 段里 8 段红，攻方改法「丙并中间」60 段里 0 段 |
| K2-2 丙多护的那一截什么时候放 | 没打中 | 多护的那一截在写掉 R 那一槽时放（30 步覆写之后隔离回到 `0+0`）；与对照臂（绕开故障读 R 的真账）比，丙收尾时多隔离至多 34 槽（两块盘合计），三臂各 90 段，被拒的各 3 段、都是管理员回退目标不在候选集里，丙没有比对照臂多拒一次 |
| K2-3 K1-1 新口径 | 没试 | 公开函数、`on_device_modes.rs`、e158 只换了取哪一份拒集，挂载自己写行之后才抛弃的根那一槽读坏会不会照拒，我没造（见「没打中的形状」） |
| K2-4 K1-2 空条目来自坏盘记录 | 没试 | 坏盘记录造空条目那一形我没造，第二节末尾写了为什么 |
| K1-1 借之前 Poll 的出口 | **打中（窄，要卡造）** | 借之前那一串先列卡、再 poll、再销账，三步不在一次锁里；poll 交回队列空了之后、销账之前另一笔借据记成「没等成」，这一笔一起被销，它的缓冲还在卡上。模型上多给 1960 MiB（受额度截断）。时间窗是 poll 返回到销账之间，真卡上撞不撞得上是推的 |
| K1-2 丢了的设备判放掉 | 没打中 | 账这一层按 `GpuCard` 克隆都放掉判，造不出多给或判不成放掉；「wgpu 句柄另拿出来活得更久」是正文第二节已列的前提 |
| K1-3 展开中不再抛 | 没打中 | 只在已在展开中时不抛，原来那一下 panic 照样往外走 |
| K3-1、K3-2 第七块 | 没打中 | 卡线程领块那一圈照旧每块看叫停；打印那一半三处产品路径都还经它。扫源码用例有一处推的盲区（第六节） |
| K4 | 同 K1-1 | K1-1 那一形就是「账给的量大于此刻真能用的」；锁把借卡压住的窗口只有起设备与清账之间几步，没造出 |
| K5 判别力 | 部分打中 | K2-1 那一形没有用例；harness 档账的 5 条补充改坏里 2 条红、3 条照绿，照绿的 3 条我判是今天走不到或等价（第六节） |

## 一、K2-1：丙罩不住 R 从中间那次发布继承的单元（打中）

### 历史（只固定前缀与故障，用户动作放开扫）

1. 起 384 槽小盘、第一个文件，覆盖写 3 次，最后一版是 P = (1, 6)；再录 Q = (1, 7)（顺序写三个数据单元，三条记录）、R = (1, 8)（建一个 inode，数据单元与 Q 共用）。崩在 R 的根槽落盘之后、R 那次轮换之前：`crates/singlefs-core/src/transaction.rs:1386`：「/// 发布的最后三步（D16（发布语义） 已定项 7 的持久顺序）：根槽 FUA 写 → 系统配置槽轮换 → 屏障，屏障完成才返回。」Q 的轮换已持久。
2. 可写挂载那一次读回全零（只这一次，写到哪一段就不再藏）：每块盘上 Q 那次轮换写的系统配置槽、Q 的根槽、Q 带末条标志的那条记录、R 自己写的树表 / 中央映射树根 / 分配记录树根。故障模型是用户认过的那一类：`crates/singlefs-core/src/recovery.rs:5442`：「/// 代价（判决同一条，用户认下）：已确认的一版在较新的系统配置槽与它的单元同时读回全零时，c_见证 落回它之前，读回的是上一版。」
3. 择根选到 R、R 走读失败，c_见证 落回 P，F2 退到 [P, R) 里读得出、不被抛弃的最新一条根，就是 P（Q 的根槽读不出）：`crates/singlefs-core/src/recovery.rs:5419`：「/// 第三节第 7 条）：系统配置没见证过所选那一版时，退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根——交回的不一定是」。Q 带末条标志的记录读不出，「越过写齐了记录的发布不退」那一条不拦：`crates/singlefs-core/src/recovery.rs:5427`：「/// `research/prompts/m3-batch37-43-code-r1-opus-output.md` 第二节 Y4，用户 2026-10-08 定「两形都修」）：退过去会越过那次写齐了记录的发布。」
4. C 那一圈：Q 读得出的前两条记录点名的落点与 Q 的账记在 Q 那一槽；R 账读不出、记录齐，走丙，R 那一槽 = R 的记录点名的 + 挂载那一刻现行账（P 的账）里仍分配着的：`crates/singlefs-core/src/mount.rs:2008`：「placements_still_allocated_in_the_account(current_records)」。丙那一支的前提写在 `crates/singlefs-core/src/mount.rs:2004`：「// 这条根建在所选那一版上：它引用的单元要么是它自己写的（记录点名了，C 那一份里有），要么在所选那一版的账里，」——R 建在 Q 上，不建在 P 上，这一句在这段历史上不成立。
5. 挂上之后撤故障，用户覆盖写。这个进程写到 Q 那一槽的 txg 时解开 Q 那一槽：`crates/singlefs-core/src/allocator.rs:2184`：「.remove(&ring_slot)」，再清掉剩下的 C 不罩着的隔离位：`crates/singlefs-core/src/allocator.rs:2217`：「&released_named_by_unapplied_journal_records,」。R 那一槽里没有 Q 的单元，它们这时就发得出去；R 那一槽要到下一个 txg 才被写掉。
6. 写掉 R 那一槽的那次发布（第 19 步，下标从 0 数）取到了 Q 的单元：单元、记录都落了、这一步的根还没落的那个状态（层 0 段边界前缀）R 还在根环里、Q 的单元被写掉了 2 份，池级 checker 判 I-7.4，违例的根是 R（实例 1、txg 8）。

### 量到的（模型目录 `out/`）

挂载那一刻的读数（`out/raw-try1.txt` 第 1 行，丙臂、Q 只藏末条记录那一形）：`selected=(1, 6) c_all=88 c_r=44 q_unit_writes=26 q_in_c_r=0 q_in_c_all=26`——Q 写的 26 份单元写都在 C 里，没有一份在 R 那一槽里。

30 步覆写、每步看「根落之前」那个状态，三条臂各一行（`out/raw-try3.txt` 原样）：

```
OPUS10ZGHI arm=h2 k=3 q=s r=n p=(1, 6) qk=(1, 7) rk=(1, 8) faults=cfgq,qroot,qlast,racct actions=oooooooooooooooooooooooooooooo hit=true secs=6.93 states=0 selected=(1, 6) c_all=66 c_r=22 q_unit_writes=26 q_in_c_r=0 q_in_c_all=26 faulted_reads=54 o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=1)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=2] o=ok(raises=0)[r_in=false,q_over=2] o=ok(raises=0)[r_in=false,q_over=4] o=ok(raises=0)[r_in=false,q_over=6] o=ok(raises=0)[r_in=false,q_over=6] o=ok(raises=0)[r_in=false,q_over=22] o=ok(raises=0)[r_in=false,q_over=26] o=ok(raises=0)[r_in=false,q_over=26] o=ok(raises=1)[r_in=false,q_over=26] o=ok(raises=0)[r_in=false,q_over=26] o=ok(raises=0)[r_in=false,q_over=26] first_red_step=- first_preroot_red=19o:r_in_ring=true:q_units_overwritten=2:I-7.4:被抛弃的根（实例1、txg8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元在盘0槽1420的那一份与位置条目里的校验和对不上；数据单元两份都读不到对得上的；映射条目指的单元 isolated_at_end=0+0 final=- crash=[]
OPUS10ZGHI arm=chain k=3 q=s r=n p=(1, 6) qk=(1, 7) rk=(1, 8) faults=cfgq,qroot,qlast,racct actions=oooooooooooooooooooooooooooooo hit=false secs=7.31 states=0 selected=(1, 6) c_all=132 c_r=88 q_unit_writes=26 q_in_c_r=26 q_in_c_all=26 faulted_reads=54 o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=1)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=false,q_over=0] o=ok(raises=0)[r_in=false,q_over=0] o=ok(raises=0)[r_in=false,q_over=2] o=ok(raises=0)[r_in=false,q_over=4] o=ok(raises=0)[r_in=false,q_over=22] o=ok(raises=0)[r_in=false,q_over=26] o=ok(raises=0)[r_in=false,q_over=26] o=ok(raises=1)[r_in=false,q_over=26] o=ok(raises=0)[r_in=false,q_over=26] o=ok(raises=0)[r_in=false,q_over=26] first_red_step=- first_preroot_red=- isolated_at_end=0+0 final=- crash=[]
OPUS10ZGHI arm=bing k=3 q=s r=n p=(1, 6) qk=(1, 7) rk=(1, 8) faults=cfgq,qroot,qlast,racct actions=oooooooooooooooooooooooooooooo hit=true secs=7.62 states=0 selected=(1, 6) c_all=88 c_r=44 q_unit_writes=26 q_in_c_r=0 q_in_c_all=26 faulted_reads=54 o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=1)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=0] o=ok(raises=0)[r_in=true,q_over=2] o=ok(raises=0)[r_in=false,q_over=2] o=ok(raises=0)[r_in=false,q_over=2] o=ok(raises=0)[r_in=false,q_over=4] o=ok(raises=0)[r_in=false,q_over=6] o=ok(raises=0)[r_in=false,q_over=22] o=ok(raises=0)[r_in=false,q_over=26] o=ok(raises=0)[r_in=false,q_over=26] o=ok(raises=1)[r_in=false,q_over=26] o=ok(raises=0)[r_in=false,q_over=26] o=ok(raises=0)[r_in=false,q_over=26] first_red_step=- first_preroot_red=19o:r_in_ring=true:q_units_overwritten=2:I-7.4:被抛弃的根（实例1、txg8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元在盘0槽1420的那一份与位置条目里的校验和对不上；数据单元两份都读不到对得上的；映射条目指的单元 isolated_at_end=0+0 final=- crash=[]
```

用户动作放开扫（字母表 `onsFb`，每段 20–34 步，每批 20 段、每臂 3 个种子；k=5 与故障不撤各 40 段），`out/summary-b1.txt`、`out/summary-b2.txt` 原样：

```
arm=bing k=3 q=s r=n faults=cfgq,qroot,qlast,racct histories=60 hit=9 preroot_red=9 red_root_txg=8=9
arm=chain k=3 q=s r=n faults=cfgq,qroot,qlast,racct histories=60
arm=h2 k=3 q=s r=n faults=cfgq,qroot,qlast,racct histories=60 hit=8 preroot_red=8 red_root_txg=8=8
arm=bing k=5 q=s r=n faults=cfgq,qroot,qlast,racct histories=40 hit=8 preroot_red=8 red_root_txg=10=8
arm=bing-persist k=3 q=s r=n faults=cfgq,qroot,qlast,racct histories=40 hit=8 preroot_red=8 red_root_txg=8=8
arm=chain k=5 q=s r=n faults=cfgq,qroot,qlast,racct histories=40
arm=chain-persist k=3 q=s r=n faults=cfgq,qroot,qlast,racct histories=40
```

`red_root_txg=8` / `=10` 是 R 的 txg（k=5 时 R = (1, 10)）。红的每一段都是 checker 判 R 违例，Q 那时已离开根环。

### 打中之后的四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分辨。丙（冻结副本原样）与 h2（丙那一截换成空，等于改前）都红，攻方改法「丙并中间」0 红；不是各臂共用的前提出了事 |
| 被判的系统当时看不看得到判别它的东西 | 看得到：挂载那一刻 C 里 Q 那一槽的落点就在内存里，R 的 txg、所选那一版的 txg 都知道，(P, R) 之间有哪几个 txg 的槽记着东西一查就有 |
| 满足的是判据字面的哪一个分句 | 正文 K2-1「R 引用、却既不是 R 自己写的、也不在挂载那一刻现行账里仍分配着的单元有没有？」——有：R 从 Q 继承的单元。违反的是 `.claude/kb/decisions/23-journal的角色与格式.md` 第 390 行「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」那一句与 `.claude/kb/invariants.md` 第 58 行 I-7.4（近 K 代块未被复用）；R 是这次挂载自己写行抛弃的，套第 390 行主语的口径与第四块判决 K2-2 那一行相同 |
| 跑前条款给的改法在打中的格上还中不中 | 判决第三节只给了丙一个改法，丙在这一格上照中（见上表 `arm=bing`） |

另一形（Q 的记录全读不出，或 Q 只有一条记录）：Q 的单元连 Q 那一槽都没记，挂上之后第一次覆写就红，违例的根是 Q（txg 7）（`out/raw-try1.txt` 第 2、4、5 行）。那是 Q 自己在挂载那一刻读不出、没人护它的单元，归 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）那一族，不算丙这一格的打中。

### 改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 标 |
|---|---|---|
| 丙并中间：走丙那一支的根 R，C 里 R 那一槽再并上 (所选那一版, R) 之间每个 txg 那一槽记着的落点（`src/opus10zghi-arms.patch` 里 `OPUS10ZGHI_ARM=chain`） | 上面那一格：60 + 40 + 40 段 0 红 | 量过（`out/summary-b1.txt`、`out/summary-b2.txt` 的 `arm=chain` 行） |
| 解开一槽时，若较晚那一槽的根还在环里、走过丙，把解开的落点并进较晚那一槽 | 同一格 | 推的（没实现没跑） |
| 照旧丙、什么都不加 | 不修 | 量过（`arm=bing`） |

「丙并中间」多护的是 (P, R) 之间那几次发布记着的落点，挂载那一刻 `c_r` 从 44 到 88（`out/raw-try3.txt` 的 `arm=chain` 行），D28（挂载期承诺量） 已定项 1 第九项的计数随之变；别的入库用例红不红是推的。

### 这一格的限度

- 第 19 步那一次发布整段的层 0 全量是 268435475 个状态（`out/raw-try5.txt`：`crash=[c19=skipped(268435475)]`），超过每次约 10⁶ 的上限，没跑。我只把「这一步最后一条根槽写之前的写全部施加」那一个状态交 checker；它是那次发布层 0 枚举域里的一个段边界状态，不是全量。
- 只用了一种 Q（顺序写三个数据单元）、一种 R（建 inode）、一个几何（384 槽小盘、S = 8）。

## 二、K2-2 到 K2-4：丙多护的那一截、新口径、空条目

### K2-2（没打中）

造法同第一节，但挂载那一刻只藏 R 的账（`OPUS_FAULTS=racct`），F2 退到 Q，R 是建在所选那一版上的单层（第四块判决 K2-2 那一形）。三条臂：丙（原样）、h2（丙那一截换空）、exact（对照：丙那一支里绕开读故障读 R 自己的账，是系统那一刻读不到的真值，只为量丙多护了多少）。R 三种（覆盖写、顺序写、建 inode），字母表 `onsWFbu`，每段 3–12 步，每臂每种 30 段。`out/summary-b3.txt`、`out/iso-b3.txt` 原样：

```
arm=bing k=3 q=s r=o faults=racct histories=30 step_refused=1
arm=bing k=3 q=s r=s faults=racct histories=30 step_refused=1
arm=bing k=3 q=s r=n faults=racct histories=30 step_refused=1
arm=exact k=3 q=s r=o faults=racct histories=30 step_refused=1
arm=exact k=3 q=s r=s faults=racct histories=30 step_refused=1
arm=exact k=3 q=s r=n faults=racct histories=30 step_refused=1
arm=h2 k=3 q=s r=o faults=racct histories=30 step_refused=1
arm=h2 k=3 q=s r=s faults=racct histories=30 step_refused=1
arm=h2 k=3 q=s r=n faults=racct histories=30 step_refused=1
r=n paired_histories_with_session=13 bing_isolates_more=11 max_extra_isolated_slots=18
r=o paired_histories_with_session=13 bing_isolates_more=11 max_extra_isolated_slots=34
r=s paired_histories_with_session=13 bing_isolates_more=11 max_extra_isolated_slots=34
```

- 被拒的那 1 段每格都是同一段历史：`b=Err:TargetNotACandidate{…BelowEffecti…}`（管理员回退目标在 F_生效 之下），三臂一样，不是准入。
- 收尾时还有会话的 13 段里，丙比 exact 多隔离的槽至多 34（R 覆盖写、顺序写）、18（建 inode），两块盘合计。第一节 30 步覆写那几段里隔离在写掉 R 那一槽之后回到 `isolated_at_end=0+0`（`out/raw-try3.txt`），多护的那一截按时放掉。
- 「把空间占满、准入白拒」这一形在这个几何上造不出：384 槽的盘上多隔离的几十槽没有一次让准入拒。盘接近满时会不会让准入早拒、早几次，我没造（要把文件写到准入线附近，没做）；即使早拒，准入拒之前先推空发布、推过 R 那一槽就放，这是推的。

### K2-3（没试）

`ring_slots_whose_reread_still_bad_refuses_the_ceiling_by` 只除去根环表里记成被抛弃根的槽；挂载自己写行之后才被抛弃的那条根（第四块 H2 那一形，表里记成有效根）它的槽不在除去之列，挂上之后这一槽读坏时抬 F 那一串会照拒。这算不算违背用户 2026-10-08 定的 H1「被抛弃根的槽不拒」，要逐字读 `.claude/kb/decisions/16-发布语义.md` 第 37 行 H1 那一句的射程，我没读完，也没造这段历史。交主 agent 判要不要另开。

### K2-4（没试）

坏盘记录（种类不认识的点名项）造出 C 那一槽空条目，要在录制流里改 journal 记录的字节、让它过自证而点名项不认识；这一轮我没造。

## 三、K1：第八块显存账

### K1-1 借之前 poll 那一串多给（打中，窄）

被判的那一串在 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs`：先列卡 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1334`：「for card_position in self.ledger.cards_with_leases_kept_after_a_failed_wait() {」，锁外 poll `crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1342`：「started.device().poll(wgpu::PollType::Poll)」，队列空了再销 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1349`：「.take_in_that_the_submitted_work_finished();」。销的是那一刻在那一台上记着的每一笔：`crates/singlefs-harness/src/gpu_memory_ledger.rs:1546`：「.return_the_leases_kept_after_a_failed_wait_on(self.device);」。注释给的理由是 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1332`：「/// 不在账的锁里 poll：队列空了是在 poll 那一刻，那几笔丢掉的缓冲早在那之前提交、丢掉，之后什么时候销都不多给。」——它只对 poll 之前已经记着的那几笔成立；poll 之后、销之前才记成没等成的那一笔，它的活可以是 poll 之后才提交的。

历史（harness 档模型 `src/opus_k11_poll_race.rs`，入库 `GpuMemoryLedger` 原样调用，真卡由一张记账表代替）：借据 X 等超时、照旧记着；线程 B 列卡、poll 交回队列空了（X 的缓冲交回驱动）；线程 A 借 8000 MiB、建好、提交了活、丢借据时没等成（照旧记着）；B 销账，A 一起销了。对照臂 `take-in-before-a` 是 B 在 A 之前销完。`out/k11.txt` 原样：

```
ATTACK arm=take-in-between step=x_kept_buffers_in_flight ledger_room=19440 truth_room=19550 under=110 over=0
ATTACK arm=take-in-between step=a_kept_buffers_in_flight ledger_room=11440 truth_room=21550 under=10110 over=0
ATTACK arm=take-in-between step=after_b_takes_in ledger_room=23510 truth_room=21550 under=0 over=1960
ATTACK arm=take-in-between a_on_the_books=false
ATTACK arm=take-in-before-a step=x_kept_buffers_in_flight ledger_room=19440 truth_room=19550 under=110 over=0
ATTACK arm=take-in-before-a step=a_kept_buffers_in_flight ledger_room=21440 truth_room=21550 under=110 over=0
ATTACK arm=take-in-before-a step=after_b_takes_in ledger_room=21440 truth_room=21550 under=110 over=0
ATTACK arm=take-in-before-a a_on_the_books=true
```

四句：分辨（只按 poll 之前那一刻记着的借据销就不中，对照臂那一行 `over=0`）；系统看得到（列卡那一刻可以把记着的借据号一起取走，poll 之后只销这几笔）；满足正文 K1-1「有没有一形借之前那一下 Poll 让账多给」；判决第三节第 1 条只给了「借之前 Poll 一次」这一个形，它在这一格上照中。

限度：A 要在 B 的 poll 返回到销账之间借到、提交、丢掉、而且没等成。没等成要么是 30 秒超时、要么是 wgpu 报错或 panic；超时那一支要 B 在这几步之间停 30 秒以上，报错那一支窗口是几步之间。真卡上撞不撞得上是推的，我没上卡。模型上多给 1960 MiB 是被额度那一项截过的数，换成额度宽的卡最多多给 A 那一整笔，推的。

改法（推的、被攻过零轮）：列卡时把每张卡上记着的借据号一起取走，poll 交回队列空了只销这几笔。`take-in-before-a` 那一臂只说明「只销 A 记上之前那几笔」时不多给，改法本身没实现、没跑。

### K1-2 丢了的设备判放掉（没打中）

账这一层：`release_a_device`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:900`：「fn release_a_device(&mut self, device: DeviceNumberOnTheCard) {」）只在 `DeviceStartedOnTheBooks` 最后一个 Arc 丢掉时调；卡池那一格换上新设备时旧那一份随之丢。我按「丢了 → 重起被拒 → 再起好 → 借据先丢 / 卡先丢 → 第二次丢」几种次序推了一遍，账上判放掉的时刻都不早于这个进程里最后一个 `GpuCard` 克隆丢掉。重起被拒时卡池那一格留着丢了的旧卡、旧设备的驱动那一份一直占着卡、下一次重起可能一直放不下，这一形第八块之前就有（`started_or_start` 起不来不换），不是这一批带进来的，没量。「wgpu 句柄另拿出来活得更久」与「最后一个句柄丢掉时驱动当场收回显存」两条前提今天都没有不要卡的检查，前一条正文第二节已列。

### K1-3 展开中不再抛（没打中）

`crates/singlefs-harness/src/gpu_memory_ledger.rs:1576`：「if !std::thread::panicking() {」——只在已在展开中时不再抛，原来那一下 panic 照样往外走、用例照红。我想到的一形：卡线程的活在 `catch_unwind` 里 panic（Out of Memory），展开途中丢的借据等设备时又 panic，后一下被吞，外层把前一下认成显存不够、报本次未跑。那时前一下本来就是显存不够，结局没有从红变绿；后一下被吞丢的只是第二个错误的文字。不算打中。

## 四、K3：第七块（没打中）

- K3-1：`crates/singlefs-harness/src/gpu_memory_ledger.rs:2294`：「if blocks.any_left_to_take(stop_of_the_cards_now()) {」——后面没有装了状态的卡就不问，有就问一次；起好的卡线程领块那一圈照旧每块看叫停（第八块变异行「卡线程判块那一圈不看叫停（正推改回去 C）」钉着），所以「该看叫停时没看、叫停之后还起新卡领块」造不出：起卡那一刻没叫停、之后叫停，新卡第一次领块时就看到。
- K3-2：三处产品路径（`crash_amplification.rs` 第 2857、6235、6381 行）都调打印的那一半；造那一行的那一半只在库单测里直接调。没打中。

## 五、K4（同 K1-1）

- K4-1：就是 K1-1 那一形。
- K4-2：借之前那一串 `started_now()` 要拿卡池那一格的锁；那一格正在起设备时这一次借（哪怕借的是别的卡）等它起完。要撞上，列卡那一刻那张卡还有没等成的借据、拿锁那一刻它又已在起设备；起设备之前那一步 `prepare_the_driver_share_before_starting_the_device` 就把没等成的那几笔清掉了，窗口只有拿锁到清账之间。我没造出，正文第二节已列。

## 六、K5 判别力

- 第九块：K2-1 那一形（R 与所选那一版之间还有一次发布）没有用例，入库两条丙用例的 R 都直接建在所选那一版上；把丙那一截改回空的两行变异钉的也是那一形。补用例可以直接用 `src/opus10zghi-probe/tests/opus_k21_chain.rs` 的 `record_chain` 加故障组 `cfgq,qroot,qlast,racct`、覆写到写掉 R 那一槽、核根落之前那个状态。
- 第八块 harness 档账：我在冻结副本拷贝上另做了 5 条改坏，跑 `cargo test -p singlefs-harness --lib -- gpu_memory_ledger`。`out/mutants.txt` 原样：

```
MUTANT M1-release-keeps-kept-leases test result: ok. 104 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.20s
MUTANT M2-release-keeps-device-in-use test result: ok. 104 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.20s
MUTANT M3-device-never-released test result: FAILED. 99 passed; 5 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.20s
MUTANT M5-never-resume-unwind test result: FAILED. 103 passed; 1 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.21s
MUTANT M6-poll-exit-returns-every-card test result: ok. 104 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.20s
```

  照绿的三条：M1（放掉一台时不销它上面没等成的）、M2（放掉在用的那一台时不清 `device_in_use`）只在「在用的那一台没丢就被放掉」时有差，那一形今天只在卡池整个丢掉时出现；M6（销没等成的时连上一台的也销）在今天走不到：上一台记成丢了的那一刻它上面没等成的已经清了。三条我判等价或走不到，推的，不算打中。M3、M5 红。
- 第七块：扫源码用例 `the_product_path_prints_the_not_run_line_of_a_lease_it_could_not_get` 只看打印那一半的函数体与调用处数目（`crates/singlefs-checker-tier/src/crash_amplification.rs:3894`：「.filter(|line| line.contains("print_the_not_run_line_or_refuse("))」），不看不打印的那一半里有没有 `println!`。把 `println!` 加回 `not_run_line_or_refusal_of_a_lease` 那两个 `Ok` 臂（第七块之前的形，例如 `crates/singlefs-checker-tier/src/crash_amplification.rs:2764`：「ScratchBookingRefusalOutcome::NotRun { not_run_line } => Ok(not_run_line),」那一臂），产品路径打两遍、库单测的标准输出里又出现本次未跑那一行，这条扫源码用例照绿——推的：它在 checker 档包里，我不跑 checker 档测试，没量；门禁收不收这几行我也没查。

## 没打中的形状

| 形状 | 取样 |
|---|---|
| K2-1 单层 R（建在所选那一版上），丙、h2、exact 三臂 | 3 种 R × 30 段 × 3 臂，`onsWFbu`、3–12 步；丙 0 红（h2 也 0 红：这几段里没有一段把 R 继承的单元回收出去） |
| K2-2 丙多护造成准入拒 | 同上 270 段，被拒的只有管理员回退目标不在候选集那一段 |
| K1-1 并发借、poll 与超时交错的别的次序 | 只推了「poll 之前记着」「销之后才记」两种，都不多给 |
| K1-2 设备丢了、重起、借据与卡的各种丢的次序 | 只推了，没写模型 |
| K3 叫停与起下一张卡 | 只读了代码，没写模型 |

崩溃状态：这一条腿真跑的层 0 全量 0 个状态（每段历史的每一步要么超过约 10⁶ 而没跑，见第一节限度，要么我没选它跑）；别的都是段边界前缀那一个状态与每一步收尾的终态。

## 这条腿自己的限度

- 只有一次抽样；K2-1、K1-1 是打中，没打中的那几格一次不算数。
- 被判对象里第八块与第七块的 checker 档那一半（`gpu_unit_checks.rs`、`crash_amplification.rs`）我只读不跑：要卡或在 checker 档包里。
- K2-1 只在一个几何、一种 Q、一种 R 上造过，全量层 0 没跑。
- 模型都在冻结副本的拷贝上量，副本上打了 `src/opus10zghi-arms.patch`（不设 `OPUS10ZGHI_ARM` 时与冻结副本行为相同；`OPUS_BING_ROOTS`、`OPUS_BYPASS_READ_FAULTS` 两个只在副本里）。

## 没做什么

- 没跑重型测试、没起 GPU、没跑门禁阶段；没跑 rerun.sh 整条（前台时限，分步跑过）。
- K2-3、K2-4 没造历史（见第二节）。
- 草稿目录 `~/.local/share/singlefs/work/m3-batch10z-ghi-code-r1-opus/` 里两份冻结副本拷贝（`copy/`、`copy-m/`）与三个编译目录交回前已删；留着 `runs/`（b1–b3 各件原始日志）与各 `*.log`，给主 agent 核对原始输出用。rerun.sh 自己从冻结副本重建拷贝，不依赖它们。


## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m3-batch10z-ghi-code-r1-opus-output.md --root ~/.local/share/singlefs/work/m3-batch10z-ghi-code-r1/tree --root .` 末行：

```
  ✓ 核了 17 处引文，对上 17 处，没判 0 处（逐处列在上面）
```
