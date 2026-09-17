# alloc-basis-r2 Opus 攻方腿报告（Z1、Z2、Z3，Z5 里 Z1 / Z2 那一半）

轮名 `alloc-basis-r2`；腿 `three-way-attack`（Opus）；立场：假设主 agent 的倾向（Z1 乙′、Z2 F-生、Z3 合）是错的。不碰 Z4，不复核第一轮判决。副本上的探针与模型 03:26–04:26 UTC 跑（04:26 起按主 agent 的要求停了重跑批），报告 04:05 UTC 起写。

## 复跑命令与文件指纹

两套装置，都在草稿目录 `/tmp/claude-1000/alloc-basis-r2-opus/` 里：

1. **crates 副本探针**（Rust）：`run-probe.sh` 把仓里的 `Cargo.toml`、`Cargo.lock`、`crates/` 用 `rsync -a --exclude target --exclude .git` 拷两份，先逐个核 6 个文件的 `git hash-object`（与正文第三节相同才往下走，不同退出码 3），仓里已被改过的三个文件从 `crates-at-report/` 换回报告读的那一版（见「读的实现」那一段），一份原样、一份用 `patch-copy.py` 按环境变量打上「切臂 / 造坏镜像」的开关，各编一个 `alloc_basis_r2_probe`，跑全部场景。**复跑核过一次**：04:03–04:26 UTC 用当时的脚本从头跑，完成的 36 份产物与 `outputs/` 里的逐字节相同（`cmp`，其中 `p-s5-hole-yiprime-I52.txt` 与 `outputs/p-s5-hole-yiprime-T1.txt` 是同一组环境变量换了个名字）；第 37 份（甲-T1 + F-生 的合法操作搜索）跑到一半，主 agent 04:26 要机器安静，本腿中止了（`kill` 写死的两个 pid），没跑完；甲-a 的 12 份、`z3-sweep.txt` 与 `newcrates-*.txt` 是复跑开始之后才加的，复跑没罩到。**这些都是副本上的数，不是入库装置上的数。** 打过补丁的那份在环境变量全不设时，S1 与 S3 的产物与原样那份逐字节相同（脚本里 `cmp`，不同退出码 5）。
2. **Z3 条款层计数模型**（Python 3，纯算术、无随机源，跑 N 遍与 1 遍信息一样多）：`z3_gates.py`。

```
cd /home/fy5090/code/singlefs
bash research/prompts/alloc-basis-r2-opus-model/run-probe.sh      # 约 25 分钟；产物写 outputs-rerun/，与 outputs/ 逐个 cmp
cd research/prompts/alloc-basis-r2-opus-model
nice -n 19 python3 z3_gates.py --selftest   # 17 条断言（含 5 条控制矩阵判别力自证），末行 SELFTEST-COMPLETE
nice -n 19 python3 z3_gates.py --verbose    # 六组（臂 × 读法）三段历史逐次发布的读数，末行 RUN-COMPLETE
nice -n 19 python3 z3_gates.py --sweep      # 18 个几何上重判 3.2–3.4，末行 SWEEP-COMPLETE（约 8 秒）
```

| 文件 | sha256 |
|---|---|
| `probe.rs` | `5a6b05793ed34930d47c58ab18b9ad4ec4efbff9eb8480ecf8b5b65efc4a2127` |
| `patch-copy.py` | `df94c8cbfb40d02d5ce45fe649bc79192482591b4bc17498023260ab587f47cd` |
| `z3_gates.py` | `3c138b80218b00748e5743f9b618d6db73d13c5dce57fa8af91c978585f7af35` |
| `run-probe.sh` | `23b5689cd6bfb552f704d63713f92fd9b3a2c37941301d7495a9a90eba97e674` |
| `crates-at-report/allocator.rs`（报告读的那一版，`git hash-object` `6bd429ed…`） | `40791099d69039a3266609454dd912e95032c446677f73150432e748c3a0282d` |
| `crates-at-report/mount.rs`（`34246932…`） | `416bb4427094da1749d21adcc11e1fbe50c3076b230bf80eb5054670f0b4462e` |
| `crates-at-report/recovery.rs`（`af342120…`） | `48c265391ac9f6c4f2e699c12bea287d89fe425fd90e15ff40f6de15d76b5696` |
| `outputs/SHA256SUMS`（57 份产物各自的 sha256） | `9854a78bb332456b7180cfef7ba9edc8bd43fc85fe02939cdce9e5eda25a100e` |
| `outputs/s1-turnover.txt` | `88eb3bbb531f9285083f6c7bcb0bf47d420844a620d91500c8cfe706341df061` |
| `outputs/s1-turnover-remount26.txt` | `1928199adb756159fb032d868910bea88a1cb83569669dd250f9cb877964773e` |
| `outputs/s4l-legal-hit-k1-1-k2-6-f8.txt` | `219fabc7fcfc83fdea13604b8dd46c4ecfbb43ecf832e6afac2373df43ab09d5` |
| `outputs/p-s4l-legal-hit-k1-1-k2-6-f8-checkerF-effective.txt` | `4194afa5a6313109746347a5623f3e9a84819c277bd35f28ebcd9ec0d9d19534` |
| `outputs/s2l-legal-search.txt` | `6f11c94d530ae9ff6e43f367e7a7f664ea0a670cfa6651f516a9a40f4e584acf` |
| `outputs/p-s2l-legal-search-T1-fsheng-8-16.txt` | `2a378fbed42abf92c25ad96434be38ade617e79c614f4dcdb97875038c9fdfe3` |
| `outputs/z3-selftest.txt` | `30cfe3168ed2c57f5b3dae90b601dbc29abd8c04e185bad1cf4bb22649444d47` |
| `outputs/z3-results.txt` | `1dc761cc9ae1aee66ca3a2730dd3b3a3a020d210bf59e67db2a345137dbb7251` |
| `outputs/z3-sweep.txt` | `76af41cc8a5ff3e8ea7413947c44a8a261d2d27543742d5ffd6505e395f7254e` |
| `outputs/newcrates-hashes.txt` | `80c2ba8d865fd937527f2a2fde06fd00322852e5ac6d27c5935bc3fff9566a94` |

读的实现（03:26 UTC 拷副本时逐个 `git hash-object`，与正文第三节记的六个逐字相同；同一份拷在 `/tmp/claude-1000/alloc-basis-r2-opus/crates-snapshot/`）：`allocator.rs` `6bd429ed…`、`mount.rs` `34246932…`、`recovery.rs` `af342120…`、`transaction.rs` `6dd2ef9a…`、`walk.rs` `6f539f57…`、`image.rs` `7e4292e7…`。04:05 UTC 回到仓里再核一次，六个都没变。**04:21 UTC 再查，另一个会话改了三个**：`allocator.rs` → `1eb3f2b0…`、`mount.rs` → `1ac1348e…`、`recovery.rs` → `ef4da610…`（另三个没变；`outputs/newcrates-hashes.txt`）。改动是影子账改成窄读法、抬 F 之前按新 F 补隔离、被抛弃根读不出只计数；`reclaim_released_up_to` 在 `raise_rollback_floor` 里仍在循环之前（新版 `mount.rs:450`），`mount.rs:419-421` 那段注释原样还在。本腿把探针对着新版本另编了一份（`/tmp/claude-1000/alloc-basis-r2-opus/copy-new/`）复跑三格：S1 转环（`outputs/newcrates-s1-turnover.txt`）与 2.1 的合法窗口那一格（`outputs/newcrates-s4l-legal-hit-k1-1-k2-6-f8.txt`）与旧版本的产物**逐字节相同**；里程碑脚本 S3（`outputs/newcrates-s3-crash-between.txt`）只有隔离槽数不同（回退之后 36 → 34，崩溃态上回退之后 92 → 82），各行判定不变。**下文 `crates/` 的行号都是正文第三节那六个哈希里的行号**，新版本里的位置不再逐个核。`run-probe.sh` 编之前从 `crates-at-report/` 把这三个文件换回报告读的那一版（换回之后仍逐个核哈希）。

读的 kb（行号都是这些版本里的）：`16-发布语义.md` `80ce26aa…`、`03-空间分配.md` `d5dfe952…`、`28-挂载期承诺量.md` `6032feab…`、`23-journal的角色与格式.md` `59128ca4…`、`invariants.md` `bd508e70…`、`13-验证路线.md` `46aba507…`、`.claude/rules/fs-design.md` `7e4d1fab…`、`checks-owed.md` `52fb41e5…`。

副本各场景的读数一行的格式（`probe.rs` 的 `state_line`）：`txg` 与 `inst` 是这次发布；`F_root` 是这条根带的 F；`F_eff` 按恢复的定义（`recovery.rs:351` `effective_rollback_floor`）从盘上现算；`acct_*` 是这次发布写进记账行的盘 0 的数（单位 16 KiB 槽）；`mem_*` 是内存分配器此刻的数；`|` 后面是今天的 checker（`walk::check_pool_image`）在盘上最新状态判红的不变量与第一处细节，全绿写 `GREEN`。

## 各格判定一览

| 格 | 历史 / 镜像（副本上跑的） | 中的臂 | 分不分辨臂 | 满足判据字面哪一句 | 判定 |
|---|---|---|---|---|---|
| Z1-0 主 agent 推的「同一次挂载里转环不回收、txg ≥ 26 之后 I-3.1 红」 | S1：mkfs → A(3) → 同一进程覆盖写到 31 | 甲-T0 | 甲-T1 在同一段历史上 0（分辨） | W1（`_alloc-basis-r2-body.md:86`） | **坐实**：txg 26 +1、27 +11、之后每次 +10（1.1）。**比推的更宽**：重开救不回来——在满环上重开，重建先按「此刻环里最旧的根」回收，写行与暖机随即把环再转 2–3 格，暖机之后的状态 I-3.1 照样红（+20、+30）（1.2） |
| Z1-hole 环上有洞 | S5：txg 26 的根槽写没落下（槽里留 txg 2 的根），覆盖写到 52 | 甲-T0、甲-T1、甲-b（挂载内同 T1） | 分辨：乙′（T0 / T1 两种回收时点）全程绿；甲-a（上界从分配记录重数）副本上全程绿 | W1 同上 | 甲-T0 与甲-T1 逐格同为 +20 → +240，txg 50 盖掉 txg 2 之后甲-T1 回 0（1.3）。三条臂一起中，按反向接受条款不拿来区分这三条，但把它们与乙′、甲-a 分开 |
| Z1-乙′-I52 乙′ 按它自己列的改法 | S6：乙′ 写者（第 1 项 = 仍分配）+ 今天的 I-5.2 | 乙′ | 分辨（甲-* 的 I-5.2 不受影响） | W1 字面；也是 W5「要改的地方不在它自己列的清单里」 | 第一个事务（txg 3）起每个状态 I-5.2 红：空闲 + 仍分配 ≠ 单元区，差的正是 defer（1.4）。倾向列的代价只有「改第一个事务记账第 1 项的值、改 I-3.1 的读法甲」，I-5.2（`invariants.md:167`）不在里面 |
| Z2-窗口 F-今 在第一、二条带新 F 的根之间把回收的槽发出去 | S2′：mkfs → A(3) → B(4) → 重开（写行 5、暖机 6、7）→ 覆盖写 8–13 → 抬 F 到 8 → **崩在 txg 14 持久之后** | F-今 | 分辨：F-生（同一套搜索的子集）0 格 | W2（`_alloc-basis-r2-body.md:87`）；崩溃态上 checker 全绿，也是 W4 | **打中**（2.1）：txg 14 那次空发布（F=8、只落盘 0）把记账树与树表写在 50240、50242——A（txg 3）的 extent 根与 inode 叶。崩溃态上 F_生效 = 0，A 仍是回退候选；今天的 checker 全绿；`mount_rollback` 到 (1, 3) 报 `UnitUnreadable { slot: 50240 }`；普通重开之后 I-2.1、I-5.1 红。只用合法操作的搜索 12989 格里 91 格有这种干净崩溃态 |
| Z2-F生(pre) 记账行按「写这条根时的 F_生效」 | S3：里程碑脚本到 14、F-生（第二条根持久之后才回收） | F-生（最弱读法） | 分辨（F-今 同格绿） | W1 | 第二条带新 F 的根（txg 16）持久之后 I-3.1 红：记账 71、候选并集 50（2.2）。按「这条根持久之后的 F_生效」写记账行（与甲-T1 同一个时点）就绿，崩溃态、重开、回退、E 全绿 |
| Z2-写失败 | 读 `mount.rs:371-399` 推的，没在副本上注入 | F-今 | 分辨 | W2 | 第二条根写失败时 `raise_rollback_floor` 经 `?` 返回错误，回收已经做了、F 在第一条根上，之后的普通发布带着新 F 继续分配，窗口只会更长（2.3，推的） |
| Z3-串 | H1：写满 → 删 16 块 → 每做一次改用户可见状态的发布就试写 16 块 | 串 | 分辨（串-重判、合同一段历史上写得回来） | W3「删掉的空间在界 3 之内回不来」，不在 `03-空间分配.md:369` ⚠️ 承认的格里 | **打中**（3.2）：用户把能删的对象删光（乙′ 11 个、甲 9 个）也写不回，F 一直是 0；扫开 18 个几何，乙′ 11 格要等到第 23 次（环转过去）、7 格删光也写不回，甲 18 格都写不回；机理是第一道闸扣掉全部 defer，抬 F 的逻辑在第二道闸，第一道不过就走不到 |
| Z3-合 | H3（回退隔离 36 块）、H4（上一次准入推过空发布 / 再重开两次） | 合 | 分辨（串、串-重判三段都 0） | W3「`df` 报出 ≥ s 而写 s 失败」 | **打中三截**（3.3），各有一格控制：① 推空放掉的固定点 D16 的 df 算可用、合的第一道闸不算；② 重开的写行与暖机放掉的超过 D16 残留那一截，对齐 ① 之后 D16 自己的第二道闸拒；③ D16 的 df 没有被抛弃根独占量那一项。合 × 甲再多一截：式子里「已分配」含 defer，放得回来的那一截仍被扣一次（H1 里 12 次） |
| Z3-串-重判 | 同 H1 / H3 / H4 | — | — | — | **没打中第 1 条**（扫开 18 个几何都是 0 次）；第 2 条**不稳定**：基准几何上删后第 5 次（乙′）写回、落在 ⚠️ 承认的「保留池扫描里到第 6 窗口」以内，扫开之后乙′ 18 格里 6 格是第 7 次（对象 4 块的那 6 格），越过 6；甲那一族的病根是第一轮 Y1 的双扣（3.4） |
| Z5 坏镜像 ①②③ × Z1 五臂、Z2 两臂 | S6、S7、S2′ 崩溃态，副本上用环境变量造坏镜像 | 见第四节表 | ③ 五条 Z1 臂各有一种形态全绿 ⇒ **不分辨**，另记共用前提的账；② 在窗口崩溃态上只 F-今 全绿 ⇒ 分辨 | W4 | 甲-T0 / T1 / b：③「defer 统计量单独多报」全绿；甲-a 最弱读法 ① ③ 全绿；乙′（加上改写的 I-5.2）三份都红，但「分配器一直不回收」那种 defer 多报（空闲同步少报）全绿——今天代码里真实存在的 Z1-0 这个错在乙′ 下没有任何检查判得出（4.2） |

## 一、Z1 账本形态与回收时点

### 1.1 主 agent 推的那一句：坐实

代码里回收只有两处调用（`grep -rn "reclaim_released_up_to" crates/ --include=*.rs`，去掉 `crates/singlefs-harness/tests` 之后原样）：

```
crates/singlefs-core/src/mount.rs:234:    allocator.reclaim_released_up_to(reclaim_floor(effective_floor, oldest_valid_root));
crates/singlefs-core/src/mount.rs:367:    let reclaimed = allocator.reclaim_released_up_to(reclaim_floor(new_floor, oldest_valid_root));
crates/singlefs-core/src/allocator.rs:22:/// 回收过的落点被再分配时那条记录改写、不追加，槽号在每盘仍唯一；回收要 F_生效 抬到释放代之上，见 `reclaim_released_up_to`）。
crates/singlefs-core/src/allocator.rs:526:    pub fn reclaim_released_up_to(&mut self, floor: CheckpointTxg) -> Vec<Placement> {
```

历史 S1：mkfs → 取号 → 暖机（1、2）→ A（3）→ 同一进程里覆盖写到 txg 31，每次发布之后在副本盘上跑 checker。`outputs/s1-turnover.txt` 第 23–25 行整行：

```
S1 overwrite txg=25 inst=1 F_root=0 F_eff=0 acct_alloc=233 acct_free=211735 acct_defer=221 mem_alloc=233 mem_defer=221 mem_isolated=0 | GREEN
S1 overwrite txg=26 inst=1 F_root=0 F_eff=0 acct_alloc=243 acct_free=211725 acct_defer=231 mem_alloc=243 mem_defer=231 mem_isolated=0 | I-3.1[盘 0：记账的已分配 Some(3981312)，遍历全部有效根得到 3964928]
S1 overwrite txg=27 inst=1 F_root=0 F_eff=0 acct_alloc=253 acct_free=211715 acct_defer=241 mem_alloc=253 mem_defer=241 mem_isolated=0 | I-3.1[盘 0：记账的已分配 Some(4145152)，遍历全部有效根得到 3964928]
```

3981312 / 16384 = 243、3964928 / 16384 = 242：txg 26 差 1 槽（第 0 版树表单元 50178，释放代 3，最后引用它的是 txg 0、1、2 三条根，txg 26 盖掉 txg 2），txg 27 差 11 槽（再加 A 那一版的 10 槽，释放代 4，只被 txg 3 引用，txg 27 盖掉它），之后每次 +10。落点公式 `root_ring.rs` 的 `target_for_publish`：txg t 与 t − 24 同区同槽。

四句：
1. 分不分辨臂：分辨。同一段历史在副本上把回收时点换成甲-T1（`SFPROBE_T1=1`：分配之前按此刻盘上的环与 F_生效 回收，记账行按这条根持久之后的环与 F_生效 算），`outputs/p-s1-turnover-T1.txt` 第 24、25 行整行：
   ```
   S1 overwrite txg=26 inst=1 F_root=0 F_eff=0 acct_alloc=242 acct_free=211726 acct_defer=230 mem_alloc=243 mem_defer=231 mem_isolated=0 | GREEN
   S1 overwrite txg=27 inst=1 F_root=0 F_eff=0 acct_alloc=242 acct_free=211726 acct_defer=230 mem_alloc=252 mem_defer=240 mem_isolated=0 | GREEN
   ```
2. 看不看得到判别它的东西：看得到。写者知道这次盖掉哪一条根（它自己算落点），不读盘也知道环里最旧的根。
3. 满足判据字面哪一句：W1（`_alloc-basis-r2-body.md:86`）。覆盖写、同一进程连续发布都是合法操作。
4. 改法在这几格上还中不中：正文没给改法；甲-T1、甲-a、甲-b、乙′ 都把回收时点改成「同甲-T1」，甲-T1 在这几格 0（上面两行）。

### 1.2 比推的更宽：重开救不回来

主 agent 那句只说「同一次挂载里」。副本上重开之后照样红，而且不需要挂载内转 24 格。`outputs/s1-turnover-remount26.txt` 第 25 行（txg 26 之后重开：写行 27、暖机 28）与 `outputs/s1-turnover.txt` 第 30 行（txg 31 之后重开：写行 32、暖机 33、34）整行：

```
S1 remount+warm txg=28 inst=2 F_root=0 F_eff=0 acct_alloc=252 acct_free=211716 acct_defer=240 mem_alloc=252 mem_defer=240 mem_isolated=0 | I-3.1[盘 0：记账的已分配 Some(4128768)，遍历全部有效根得到 3801088]
S1 final-remount+warm txg=34 inst=2 F_root=0 F_eff=0 acct_alloc=256 acct_free=211712 acct_defer=244 mem_alloc=256 mem_defer=244 mem_isolated=0 | I-3.1[盘 0：记账的已分配 Some(4194304)，遍历全部有效根得到 3702784]
```

第一行差 252 − 232 = 20，第二行差 256 − 226 = 30。机理：`mount.rs:229-234` 在写行之前按「此刻环里最旧的有效根」回收一次，随后 `establish_instance`（`mount.rs:447-503`）的写行与暖机各盖掉一条根，它们换下的单元从候选并集里出去，记账却不再回收。环一旦满（txg ≥ 24），每次重开之后的暖机状态都红，差值 = 写行加暖机的次数 × 那几条被盖掉的根独占的槽数。同一段历史在甲-T1 下（`outputs/p-s1-turnover-remount26-T1.txt` 第 25 行）：`S1 remount+warm txg=28 inst=2 F_root=0 F_eff=0 acct_alloc=232 acct_free=211736 acct_defer=220 mem_alloc=242 mem_defer=230 mem_isolated=0 | GREEN`。

四句同 1.1；第 3 句补一句：可写挂载与暖机是 D16 已定项 8 定的必经路径，这一格不是边角历史。它还说明甲-T0 的回收时点在第一版固定脚本里没被撞到，只因为脚本走不到 24 次发布；走到的一刻，挂载本身就触发。

### 1.3 环上有洞：甲-T0、甲-T1、甲-b 一起中，乙′ 与甲-a 不中

历史 S5：S1 到 txg 25，txg 26 那次的根槽写在副本设备上静默丢掉（槽里留 txg 2 的根；`probe.rs` 的 `drop_next_root_write`），之后覆盖写到 txg 52（分配记录树一个节点再装不下下一次覆盖写就停）。整行（每组两行：txg 27 是洞刚出现之后第一个持久状态，txg 50 盖掉 txg 2）：

```
outputs/p-s5-hole-T0.txt:5:S5 hole@26 txg=27 inst=1 F_root=0 F_eff=0 acct_alloc=253 acct_free=211715 acct_defer=241 mem_alloc=253 mem_defer=241 mem_isolated=0 | I-3.1[盘 0：记账的已分配 Some(4145152)，遍历全部有效根得到 3817472]
outputs/p-s5-hole-T0.txt:28:S5 hole@26 txg=50 inst=1 F_root=0 F_eff=0 acct_alloc=483 acct_free=211485 acct_defer=471 mem_alloc=483 mem_defer=471 mem_isolated=0 | I-3.1[盘 0：记账的已分配 Some(7913472)，遍历全部有效根得到 3964928]
outputs/p-s5-hole-T1.txt:5:S5 hole@26 txg=27 inst=1 F_root=0 F_eff=0 acct_alloc=253 acct_free=211715 acct_defer=241 mem_alloc=253 mem_defer=241 mem_isolated=0 | I-3.1[盘 0：记账的已分配 Some(4145152)，遍历全部有效根得到 3817472]
outputs/p-s5-hole-T1.txt:28:S5 hole@26 txg=50 inst=1 F_root=0 F_eff=0 acct_alloc=242 acct_free=211726 acct_defer=230 mem_alloc=483 mem_defer=471 mem_isolated=0 | GREEN
outputs/p-s5-hole-yiprime-T1.txt:5:S5 hole@26 txg=27 inst=1 F_root=0 F_eff=0 acct_alloc=12 acct_free=211715 acct_defer=241 mem_alloc=253 mem_defer=241 mem_isolated=0 | GREEN
outputs/p-s5-hole-yiprime-T1.txt:28:S5 hole@26 txg=50 inst=1 F_root=0 F_eff=0 acct_alloc=12 acct_free=211726 acct_defer=230 mem_alloc=483 mem_defer=471 mem_isolated=0 | GREEN
```

甲-T0 与甲-T1 在 txg 27–49 逐格相同（+20，每次再 +10，txg 49 是 473 − 233 = 240），甲-T1 到 txg 50 盖掉 txg 2 才一次回 0；乙′ 在 T0、T1 两种回收时点下（`p-s5-hole-yiprime-T0.txt`、`p-s5-hole-yiprime-T1.txt`）全程绿。副本上的造法是「根槽写没落下、写者当成功」；条款里的写失败（`23-journal的角色与格式.md:691` 已定项 14 里那一句逐字「根槽写失败重发时 checkpoint_txg 推进一格再发（根环区域 = `txg mod R`，D22（单元原子性怎么合成） 已定项 2，不推进就是反复重写同一个槽、把 R 个失败域用成一个）」）是写者收到错误、推进一格重发，两者留在环上的形状相同（槽里是 24 代之前的旧根），重发那一次按真实的环算，甲-T1 照样多扣。

- 甲-b（挂载与回退时读环里每条有效根的分配记录树，按「仍被某条有效根引用」重建）：挂载那一刻是准的；挂载之后回收时点「同甲-T1」，洞在挂载内出现时它就是上面甲-T1 那两行。副本上没实现甲-b，这一句是推的。
- 甲-a（I-3.1 拆成「并集 ≤ 记账」与「记账 − 并集 ≤ 谓词在洞上多扣的量」）：副本上把 checker 的 I-3.1 换成这两条（`SFPROBE_ARM=jiaa_exact`：上界从最新根的分配记录重数「已释放 ∧ 释放代 > max(F, 环里最旧有效根) ∧ 没有候选根引用」；`jiaa_weak`：上界取记账第 5 项），写者用甲-T1，两种都全程绿。`outputs/p-jiaa-jiaa_exact-hole.txt` 第 5 行整行：`S5 hole@26 txg=27 inst=1 F_root=0 F_eff=0 acct_alloc=253 acct_free=211715 acct_defer=241 mem_alloc=253 mem_defer=241 mem_isolated=0 | GREEN`。
- 四句：① 分辨：把甲-T0 / 甲-T1 / 甲-b 与乙′、甲-a 分开；三条甲臂之间不分辨，按反向接受条款这一格不拿来判它们三条之间的高低；② 看得到：写者知道那次根槽写失败，但跨一次重开就看不到每个已释放槽「最后引用它的根」是谁（分配记录里分配代被释放代盖掉，第一轮报告 2.4 那一段，这一轮没重核）；③ W1 字面；④ 甲-a、乙′ 就是为这一格设计的改法，它们在这一格上不中。

### 1.4 乙′ 按它自己列的改法：I-5.2 在第一个事务上就红

乙′ 把第 1 项改成仍分配（不含 defer）。第 2 项空闲今天独立维护、不含 defer（`allocator.rs:112-113` 两行注释整行：`/// 空闲槽数，独立维护：分配时减、回收放回时加，不由「容量 − 已分配」现算（D5（快照 / 空间记账机制） 已定项 4 的 ⚠️：` 与 `/// 那样 I-5.2（空闲统计对得上） 是恒真式；三方代码第一轮攻方腿打中）。已释放而还在 defer 窗口里的不算空闲。`）。I-5.2 整行（`invariants.md:167`）：

```
| I-5.2 | 空闲统计对得上 | 空闲空间统计 == 总空间 − 已分配空间 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
```

副本上只把写者的第 1 项改成仍分配、把 checker 的 I-3.1 改成乙′ 的两条（`SFPROBE_ARM=yiprime`），I-5.2 不动。`outputs/p-s6-yiprime-asListed-legal.txt` 第 1 行（第一个事务 A）整行：

```
S6 A txg=3 inst=1 F_root=0 F_eff=0 acct_alloc=12 acct_free=211955 acct_defer=1 mem_alloc=13 mem_defer=1 mem_isolated=0 | I-5.2[盘 0：空闲 Some(3472670720) + 已分配 Some(196608) ≠ 单元区 Some(3472883712)]
```

差 3472883712 − 3472670720 − 196608 = 16384，正是 defer 那 1 槽。之后每个状态都红（同一文件第 2–5 行）。把 I-5.2 改判成「空闲 + 第 1 项 + defer == 单元区」（`SFPROBE_I52=rewritten`）之后同一段历史全绿（`outputs/p-s6-yiprime-I52-legal.txt` 第 1 行 `S6 A txg=3 inst=1 F_root=0 F_eff=0 acct_alloc=12 acct_free=211955 acct_defer=1 mem_alloc=13 mem_defer=1 mem_isolated=0 | GREEN`）。

四句：① 分辨（甲-* 的第 1 项含 defer，I-5.2 原式成立）；② 看得到（记账行里 defer 就在旁边）；③ W1 字面，也是 W5：正文第五节倾向那一句逐字「代价是改第一个事务记账第 1 项的值、改 I-3.1 的读法甲」，I-5.2 不在里面；④ 改法「I-5.2 跟着改成三项之和」在这几格上不中，被攻过零轮，而且它引出 4.2 那一格。

### 1.5 W2（回收的安全）在 Z1 各臂上

- 甲-T1（副本上 `SFPROBE_T1=1`）：分配器每次发布之前按「此刻盘上的环与 F_生效」回收，释放代 ≤ 环里最旧有效根的槽，最后引用它的根早已被盖掉，按构造发不出候选根引用的槽。副本读数：`p-s1-turnover-T1.txt`、`p-s1-turnover-remount26-T1.txt`、`p-s5-hole-T1.txt` 三份里 `grep -c "I-2.1"` 都是 0（走读会在候选根的单元被换内容时报 I-2.1，第 2.1 节那一格就是它报的）；2.1 节的合法操作搜索在甲-T1 + F-生下 1942 格一次都没打中。**没打中**。
- 甲-b：挂载时按「没有任何有效根引用」回收，洞里的槽会比 `16-发布语义.md:371` 那条谓词（`| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) |`）更早回到空闲。这不是 W2 的中（没有候选根还引用它们），是它要改那条用户定案的谓词、或者只改记账不改分配器，两种都得写明（1.6）。副本上没实现。
- 甲-a、乙′：分配器同甲-T1，不改。**没打中**。

### 1.6 W5（连带）：各条 Z1 臂要改的地方

| 臂 | 条款 | 第一个事务的字节 | `crates/` 与门禁 |
|---|---|---|---|
| 甲-T0 | 不改 | 不改 | 不改；W1 在 1.1、1.2 输 |
| 甲-T1 | `16-发布语义.md` 已定项 1 要补「记账行按这条根持久之后的环与 F_生效 判可再分配」（第一轮账 2） | 不改（副本上 A 那一版记账 13 槽与甲-T0 相同，`p-s1-turnover-T1.txt` 第 1 行） | `transaction.rs:1272-1286` 三行记账各扣 / 加「已释放 ∧ 释放代 ≤ 持久之后门槛 ∧ 没回收」；分配之前回收（副本补丁加在 `publish_version` 拷分配器之前，要读 24 个根槽或在内存里维护环）；`allocator.rs:524-525` 注释「第一版根环 24 槽、种子 txg 0，环里最旧有效根恒 0，`floor` 就是 F_生效」与 `mount.rs:354-356` 注释「生效（两块盘都有带新 F 的根）之前这个进程不会再发布，回收的槽在生效之前发不出去」都不成立（1.1、2.1）；`crates/mutations.tsv` 第 23 行锚点是 `            device_map.allocated_slots() * SLOT_BYTES,`，改这一行门禁 59 号要跟着改表 |
| 甲-a | `invariants.md:120` I-3.1 拆成两条，并写死「谓词在洞上多扣的量」怎么算 | 不改 | `walk.rs:840` 拆两条；上界要 checker 自己读分配记录的释放代与环，等于把运行时的回收谓词再写一遍——`.claude/rules/fs-design.md:36` 那一句够得着它（整行抄在 4.2） |
| 甲-b | `23-journal的角色与格式.md:1209` 回退段逐字「defer 队列、分配器游标、全部记账统计量的现行值从 R_old 那棵账重新载入」与「按环里每条有效根的分配记录树重建」冲突，要改；若分配器也按「没有有效根引用」回收，还要改 `16-发布语义.md:371` 的谓词 | 不改 | `mount.rs:196-252` 的重建多读至多 24 棵分配记录树（形态同 `mount.rs:237-249` 的影子账那一圈）；挂载内同甲-T1 |
| 乙′ | `invariants.md:120` I-3.1、**`invariants.md:167` I-5.2**（倾向没列）；`05-快照-空间记账机制.md:485` 第 1 项的值；`28-挂载期承诺量.md:40`「已分配 = I-3.1 的被审计对象」仍成立，式子不改 | **改**：`layout/01-first-txn.md:291` 与 `05-快照-空间记账机制.md:485` 的已分配每盘 212 992 → 196 608（副本 `p-s6-yiprime-I52-legal.txt` 第 1 行 `acct_alloc=12`） | `transaction.rs:1274`；`walk.rs:840`（I-3.1 要多存一份「最新根可达」的引用集）与 `walk.rs:847`（I-5.2）；`allocator.rs:109-111` 注释；测试 `first_transaction_step_five_publish.rs:938`（`212_992`）与 `:940-944`（「已分配 + 空闲 = 单元区」）、`second_transaction_step_three_second_instance.rs:261-264`（`47 * SLOT_BYTES`）、`checker_known_bad_images.rs:349-356`（「记账树里盘 0 的「空闲」多记 16384」那份 I-5.2 坏镜像要按新式子重造）；`crates/mutations.tsv` 第 8 行（「空闲不随分配减（I-5.2 要红）」）与第 23 行锚点；E142 装置 `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:3337` 那一行产物与 `experiments/142-第一个事务的干跑.md:91` 的 `allocated_bytes_per_device=212992` |

## 二、Z2 崩溃、根槽写失败与两个 F

### 2.1 F-今 在「第一条带新 F 的根持久、第二条没持久」那一格把回收的槽发了出去（打中）

`mount.rs:354-356` 注释整行：

```
    // 回收在写第一条带新 F 的根之前：一条根带的 F 与它的记账行要说同一件事——checker 的 I-3.1（已分配统计对得上） 按那条根自己的 F
    // 取候选集的并集，释放代 ≤ F 的落点不在并集里，记账的「已分配」也不能再算它们。生效（两块盘都有带新 F 的根）之前这个进程不会再发布，
    // 回收的槽在生效之前发不出去。
```

最后一句不成立：`mount.rs:371-399` 那个循环自己就在发布，每次空发布经 `publish_version` 重写四个固定点，提交内生块从开放聚簇段 bump（`allocator.rs:495-522`），开放段满了就开「单元区内最低的、used 与 isolated 都为 0 的 64 槽段」（`allocator.rs:297-312`）。刚回收完的一整段就是这样的段。

历史（全部是合法操作；探针 `legal-hit 1 6 8 1 1 3`，抬 F 那一步照 `mount.rs:331-399` 一行一行做、停在第一次发布之后，因为 `raise_rollback_floor` 本身不停在中间；实例表改从盘上最新根读，理由见第七节 1）：

| txg | 操作 | 带的 F | 落哪块盘 |
|---|---|---|---|
| 3 | A（第一个事务，实例 1） | 0 | 盘 0 |
| 4 | B 覆盖写 | 0 | 盘 1 |
| 5 / 6 / 7 | 重开（实例 2）：写行 / 暖机 / 暖机 | 0 | 盘 0 / 盘 0 / 盘 1 |
| 8–13 | 六次覆盖写 | 0 | — |
| — | 抬 F 到 8（上限 10 = 第 4 新的非空根；回收 30 个落点） | — | — |
| 14 | 抬 F 的第一次空发布 | 8 | 盘 0（区域 2） |
| — | **崩**（txg 15 的根没写） | — | — |

`outputs/s4l-legal-hit-k1-1-k2-6-f8.txt` 第 1–6 行整行：

```
S4L before-raise txg=13 inst=2 F_root=0 F_eff=0 acct_alloc=97 acct_free=211871 acct_defer=85 mem_alloc=97 mem_defer=85 mem_isolated=0 | GREEN
S4L ceiling=10 reclaimed_count=30
S4L publish txg=14 F=8 wrote=[(AllocationTree, 50367, 1), (AccountingTree, 50240, 1), (MappingTree, 50241, 1), (TreeTable, 50242, 1)]
S4L crash-state txg=14 inst=2 F_root=8 F_eff=0 acct_alloc=66 acct_free=211902 acct_defer=54 mem_alloc=66 mem_defer=54 mem_isolated=0 | GREEN
S4L crash->rollback(victim) failed: Recovery(UnitUnreadable { slot: SlotNumber(50240) })
S4L crash->remount txg=16 inst=3 F_root=0 F_eff=0 acct_alloc=108 acct_free=211860 acct_defer=96 mem_alloc=108 mem_defer=96 mem_isolated=0 | I-2.1[树 11（种类 1）的根 在盘 0 槽 50240 的那一份与位置条目里的校验和对不上] ; I-5.1[盘 0：树表单元（槽 50242 跨 1）与 inode 树的孩子（槽 50242）重叠]
```

读法：
- 崩溃态盘上只有盘 0 带着 F = 8 的根，`recovery.rs:351` 的 `effective_rollback_floor` 算出 0，A（txg 3，实例 1，实例表里那一行 T = 4，3 ≤ 4）按 `16-发布语义.md:376`（`| 回退候选集 | 按实例表判仍然有效 ∧ txg ≥ F_生效（D23（journal 的角色与格式） 已定项 14 的候选集随之加这一条） |`）是回退候选。
- txg 14 把记账树写在 50240（A 的 extent 根）、树表写在 50242（A 的 inode 叶）。今天的 checker 用最新根自己带的 F（`walk.rs:786-790`）把 A 排出候选集，崩溃态全绿。
- 管理员照 `mount_rollback` 回退到 (1, 3)：候选检查放行，读 A 的单元时报 `UnitUnreadable { slot: 50240 }`。
- 不回退、普通重开：新实例的根写 F_生效 = 0（`mount.rs:460`），A 回到 checker 的候选集，I-2.1 与 I-5.1 红——一段只由合法操作组成的历史，结尾是一个 checker 判红的镜像。
- 同一个崩溃态换成「checker 按恢复的定义现算 F_生效」（`SFPROBE_CHECKER_FLOOR=effective`），`outputs/p-s4l-legal-hit-k1-1-k2-6-f8-checkerF-effective.txt` 第 4 行整行：`S4L crash-state txg=14 inst=2 F_root=8 F_eff=0 acct_alloc=66 acct_free=211902 acct_defer=54 mem_alloc=66 mem_defer=54 mem_isolated=0 | I-2.1[树 11（种类 1）的根 在盘 0 槽 50240 的那一份与位置条目里的校验和对不上] ; I-3.1[盘 0：记账的已分配 Some(1081344)，遍历全部有效根得到 1605632] ; I-5.1[盘 0：树表单元（槽 50242 跨 1）与 inode 树的孩子（槽 50242）重叠]`。

它有多常见：`legal-search 20 25`（k1 = 1..20 次覆盖写 → 重开 → k2 = 0..25 次覆盖写 → 抬 F 到 (旧 F, 上限] 的每个值）12989 格，`outputs/s2l-legal-search.txt` 末两行整行 `S2L SUMMARY cells=12989 clean_hits=258` 与 `PROBE-COMPLETE`；打中的 (k1, k2, f) 格共 138 个；其中 91 格有干净崩溃态（写到受害槽的那次发布不是让 F 生效的最后一次、受害根没被这次发布自己的根盖掉），干净的那 258 行里受害根是 (1, 3) 240 行、(1, 10) 6 行、(1, 17)–(1, 22) 各 2 行；另 47 格只在最后一次发布上打中（要崩在它的单元写完、根没写之前），没逐格核。（数法：`grep "S2L HIT" outputs/s2l-legal-search.txt` 取第 3–5 列去重，带 `last=false` 的再去重。）不重开、只在同一进程里覆盖写的那一族（`s2-window-search.txt`）也打中，但要在抬 F 之前多一次空发布才对得上开放段的边界，那次空发布今天没有合法来源，只当旁证。

四句：
1. 分不分辨臂：分辨。同一搜索的子集（k1 ≤ 8、k2 ≤ 16）换成 F-生（回收推迟到两条根都持久之后，副本上 `SFPROBE_T1=1 SFPROBE_RAISE=fsheng`），`outputs/p-s2l-legal-search-T1-fsheng-8-16.txt` 整份两行：`S2L SUMMARY cells=1942 clean_hits=0`、`PROBE-COMPLETE`（最后一次发布上的也是 0 行）；F-今 在同一子集（k1 ≤ 8、k2 ≤ 16）上 88 格打中、其中 60 格有干净崩溃态。
2. 看得到吗：看得到。写者知道第二条带新 F 的根还没写；checker 从盘上各根带的 F 就能算出 F_生效。
3. 字面：W2（`_alloc-basis-r2-body.md:87`），候选集按 `16-发布语义.md:376`；崩溃态 checker 全绿，同时是 W4。它也违反 `16-发布语义.md:371` 与 `:375`（`| 生效 | 每块幸存盘上都有带新 F 的持久根才生效；恢复后生效值 = 各幸存盘所带 F 最大值的最小值 |`）：回收用的是还没生效的 F。
4. 改法：F-生（回收推迟到生效之后）在这 91 格上不中（子集 0）。「先回收、但开段时绕开回收的槽直到生效」也修得掉，没量。

### 2.2 F-生：最弱读法在第二条带新 F 的根上红，另一种读法全绿

F-生 那一行逐字「记账行 | F_生效」，没写是「写这条根的那一刻的 F_生效」还是「这条根持久之后的 F_生效」。两种都在副本上跑了里程碑脚本到 txg 14 再抬 F = 11（S3：A、B、重开、C、回退到 (1, 3)、D、暖机、覆盖写 11–14）。

- **读法 pre**（写那一刻；副本上 `raise_floor_manual(…, fsheng = true)`、不开 `SFPROBE_T1`）配「checker 按恢复的定义现算 F_生效」（`SFPROBE_CHECKER_FLOOR=effective`），`outputs/p-s3-crash-between-checkerF-effective.txt` 第 12、16 行整行：
  ```
  S3 F-生(pre) crash-after-first-F-root txg=15 inst=3 F_root=11 F_eff=0 acct_alloc=67 acct_free=211901 acct_defer=55 mem_alloc=67 mem_defer=55 mem_isolated=36 | GREEN
  S3 F-生(pre) after-F-effective txg=16 inst=3 F_root=11 F_eff=11 acct_alloc=71 acct_free=211897 acct_defer=59 mem_alloc=50 mem_defer=38 mem_isolated=36 | I-3.1[盘 0：记账的已分配 Some(1163264)，遍历全部有效根得到 819200]
  ```
  txg 16 这条根一持久 F 就生效，checker 的候选集随之去掉 txg < 11 的根，而这条根的记账行是在回收之前写的：71 对 50。
- **读法 post**（这条根持久之后；副本上开 `SFPROBE_T1=1`，写者算「盖掉同槽旧根、加上这条之后」的环与 F_生效，分配器仍在生效之后才回收），同一个 checker，`outputs/p-s3-crash-between-T1-Fsheng-checkerEff.txt` 第 12、16 行整行（标签沿用 `F-生(pre)`，是探针里场景名写死了，这一份开着 T1）：
  ```
  S3 F-生(pre) crash-after-first-F-root txg=15 inst=3 F_root=11 F_eff=0 acct_alloc=67 acct_free=211901 acct_defer=55 mem_alloc=67 mem_defer=55 mem_isolated=36 | GREEN
  S3 F-生(pre) after-F-effective txg=16 inst=3 F_root=11 F_eff=11 acct_alloc=50 acct_free=211918 acct_defer=38 mem_alloc=50 mem_defer=38 mem_isolated=36 | GREEN
  ```
  同一份里崩溃态重开、崩溃态回退到 (3, 9)、生效之后的 E（txg 17，数据落 50178）也都绿；加上乙′ 写者与 checker（`p-s3-crash-between-yiprime-T1-Fsheng-checkerEff.txt`）也全绿。

- 对照：F-今 的记账（回收在第一条根之前）配上 F-生 的 checker，崩溃态就红（同一份 `p-s3-crash-between-checkerF-effective.txt` 第 3 行 `S3 F-今 crash-after-first-F-root txg=15 inst=3 F_root=11 F_eff=0 acct_alloc=46 acct_free=211922 acct_defer=34 mem_alloc=46 mem_defer=34 mem_isolated=36 | I-3.1[盘 0：记账的已分配 Some(753664)，遍历全部有效根得到 1097728]`）；今天的 checker 取最新根自己带的 F，正好把这一格藏住（`s3-crash-between.txt` 第 3 行同一状态 `| GREEN`）。所以 F-今 与「checker 取最新根的 F」是绑在一起的：这对组合在 2.1 那一格上放过了一份真坏的镜像。正文问的「恢复用 F_生效（旧值）重建、而最新根的记账按新 F 说话」那一格，窗口里没有复用时（里程碑脚本抬 F 到 11 的那两次空发布都落在开放段里）今天的代码判绿：同一份 `s3-crash-between.txt` 第 4、5 行整行 `S3 F-今 crash->remount txg=17 inst=4 F_root=0 F_eff=0 acct_alloc=77 acct_free=211891 acct_defer=65 mem_alloc=77 mem_defer=65 mem_isolated=36 | GREEN` 与 `S3 F-今 crash->rollback(3,9) txg=17 inst=4 F_root=0 F_eff=0 acct_alloc=29 acct_free=211939 acct_defer=17 mem_alloc=29 mem_defer=17 mem_isolated=92 | GREEN`——第 4 行是崩溃态上普通重开（写行与暖机按 F_生效 = 0 写根、按旧门槛重建记账，F 退回 0），第 5 行是崩溃态上回退到 (3, 9)，I-3.1、I-5.2 都绿；窗口里有复用时就是 2.1 那几行。

四句（pre 那一格）：① 分辨（F-今 同一状态绿）；② 看得到（写者知道这条根会让 F 生效）；③ W1（`_alloc-basis-r2-body.md:86`）；④ 改法「记账行按这条根持久之后的 F_生效」就是甲-T1 那个时点，在这一格上不中，被攻过零轮。按反向接受条款记 F-生 在 W1 输一次（最弱读法），条款补一句就能收严。正文第五节给 F-生 写的推翻观测逐字「一个合法的崩溃状态在 F-生 下判红」：txg 16 持久之后立刻崩掉就是这样一个崩溃状态，按最弱读法触发，按持久之后读法不触发。

### 2.3 根槽写失败

- F-今：`raise_rollback_floor` 的循环里 `publish_version(...)?`（`mount.rs:376-392`），第二条带新 F 的根写失败时整个函数返回错误；回收在循环之前已经做完（`mount.rs:367`），`current` 停在第一条带新 F 的根上。之后的普通发布照 `publish_overwrite` 的 `rollback_floor: previous.root.rollback_floor`（`transaction.rs:902`）带着新 F 往下发，直到某一次落到另一块盘；这中间的每一次分配都能拿 2.1 那种已回收的槽，窗口比 2.1 更长。**推的，没在副本上注入写失败。**条款侧的「推进一格再发」（`23-journal的角色与格式.md:691`）今天在这条路上没有实现。
- F-生：分配器在两块盘都有带新 F 的持久根之前不回收，写失败只让窗口变长，不引入可复用的槽。副本上没注入，推的。
- 洞与 Z1 各臂：见 1.3。
- I-5.2 在 Z2 这几格上：`outputs/` 里 `grep -l "I-5.2\["` 只命中 `p-s6-yiprime-asListed-legal.txt`、`p-s6-yiprime-I52-deferplus.txt`、`p-s6-yiprime-asListed-deferplus.txt` 三份（都是乙′ 那一格与坏镜像 ③），2.1、2.2 的崩溃态、崩溃后重开、回退里 I-5.2 一次都没红：甲-* 的空闲与已分配从同一个分配器取，回收时一加一减。

### 2.4 W5：Z2 两臂要改的地方

| 臂 | 条款 | `crates/` |
|---|---|---|
| F-今 | `16-发布语义.md:371` 与 `:375` 按字面不允许在生效之前回收，要么改这两行，要么认 2.1 | 不改；`mount.rs:354-356` 的注释要删掉最后一句 |
| F-生 | `invariants.md:120` I-3.1 的「txg ≥ 最新根带的回退下界 F」改成 F_生效；D16 已定项 1 补「记账行按这条根持久之后的 F_生效」（不补就是 2.2 的 pre 读法） | `mount.rs:354-367`：回收挪到循环之后；`walk.rs:786-790`：checker 按各盘所带 F 最大值的最小值现算（副本补丁 `patch-copy.py` 里那一段）；记账行按持久之后的 F（与甲-T1 同一处改动，`transaction.rs:1272-1286`） |

## 三、Z3 两道闸三种读法

### 3.1 模型、读法与它没建的东西

`z3_gates.py`，每块盘一个数（两块盘互为镜像），单位 16 KiB 块。只读导入的 crates 事实与条款取值写在文件头注释里。三种读法怎么落成代码：

| 读法 | 第一道闸（`可用 ≥ 需求`） | 第二道闸 | 用户看到的 `df` |
|---|---|---|---|
| 串 | `28-挂载期承诺量.md:21` 的式子，defer 全扣 | `16-发布语义.md:377`：`可分配 = min(可再分配 + 活元数据 − 保留池, df_D16)` 不够就推空发布抬 F 到上限，至多 8 次 | `28-挂载期承诺量.md:90` 那一句：可用 |
| 合 | 同一个式子，defer 那一项减去「抬 F 在有界步数内放得回来的」（释放代 ≤ max(上限, 环里最旧有效根)） | 同上 | `16-发布语义.md:378` 的 df |
| 串-重判 | 同串；不过就推一次抬 F 的空发布、回收，再判一次，至多 8 次 | 同上 | 同串 |

建模选择（每一条都是这条腿取的读法）：第 1 项取乙′（不含 defer）与甲（含 defer）两套各跑一遍；需求 = 对象自己的块数（含它的文件元数据），固定点走保留池；「活元数据」取 0（C371 没定义）；回收时点取甲-T1；c_max = 4、切换预留每块盘 56 块、checkpoint 保留池 4 块（`28-挂载期承诺量.md:84`、`:88`、`:100` 的第一个事务几何）；容量每块盘 300 块、对象 16 块。**没建**：在飞合成（D28 已定项 2）、墓碑与「解开自己」、C370 的单位换算、E139 的故障世界、三块盘。

每段历史逐次发布一行（`outputs/z3-results.txt`）：`F` 是这条根带的；`F_eff` 各盘所带 F 最大值的最小值；`oldest` 环里最旧有效根；`ceiling` 抬 F 的上限（`16-发布语义.md:374`）；`still` 仍分配；`defer` 已释放没回收；`restorable` 抬 F 放得回来的；`lag` 滞后量（`16-发布语义.md:378` 的定义）；`df` 这种读法下用户看到的；`gate1` 第一道闸左边；`allocatable` 此刻分配器真发得出去的。负数照原样印，表示已经吃进保留池。

### 3.2 串：删掉的空间回不来（打中）

H1，乙′，串（`outputs/z3-results.txt` 第 25–33 行与第 67–74 行整行；行首的「行号:」是 `awk` 加的）：

```
25:     txg= 12 写 o10(16)                    F=  0 F_eff=  0 oldest=  0 ceiling=  9 still=183 defer= 48 isolated= 0 restorable= 36 lag= 12 df=   9 gate1=   9 allocatable=  69
26:               请求 o10 16 块：df=29 成功（推 0 次）
27:               请求 o11 16 块：df=9 第一道闸 9 < 16 ⇒ ENOSPC
28:     txg= 13 删 o0(16)                     F=  0 F_eff=  0 oldest=  0 ceiling= 10 still=167 defer= 68 isolated= 0 restorable= 40 lag= 28 df=   5 gate1=   5 allocatable=  65
29:       删后第 0 次改用户可见状态的发布之后试写 16 块：ENOSPC（df=5）
30:     txg= 14 写 tiny(1)                    F=  0 F_eff=  0 oldest=  0 ceiling= 11 still=167 defer= 73 isolated= 0 restorable= 44 lag= 29 df=   0 gate1=   0 allocatable=  60
31:               请求 tiny 1 块：df=5 成功（推 0 次）
32:       删后第 1 次改用户可见状态的发布之后试写 16 块：ENOSPC（df=0）
33:               请求 tiny 1 块：df=0 第一道闸 0 < 1 ⇒ ENOSPC
67:     txg= 23 删 o9(16)                     F=  0 F_eff=  0 oldest=  0 ceiling= 20 still= 23 defer=253 isolated= 0 restorable=193 lag= 60 df= -36 gate1= -36 allocatable=  24
68:       删后第 10 次改用户可见状态的发布之后试写 16 块：ENOSPC（df=-36）
69:               请求 tiny 1 块：df=-36 第一道闸 -36 < 1 ⇒ ENOSPC
70:       1 块的覆盖写被拒，改删 o10 当这一次改用户可见状态的发布
71:     txg= 24 删 o10(16)                    F=  0 F_eff=  0 oldest=  1 ceiling= 21 still=  7 defer=269 isolated= 0 restorable=209 lag= 60 df= -36 gate1= -36 allocatable=  24
72:       删后第 11 次改用户可见状态的发布之后试写 16 块：ENOSPC（df=-36）
73:               请求 tiny 1 块：df=-36 第一道闸 -36 < 1 ⇒ ENOSPC
74:       1 块的覆盖写被拒、也没有别的对象可删，用户再也改不了状态
```

用户删光了 o0–o10 这 11 个 16 块的对象（第 34–66 行是中间的删 o1–o8，形状相同），抬 F 放得回来的已经有 209 块，F 一直是 0：第一道闸把 defer 整个扣掉，每次都在第一道闸就拒，`16-发布语义.md:377` 那句「准入不够时先推空发布抬 F」在第二道闸里，走不到。甲那一组（第 399–456 行）同形。机理不靠这个容量（乙′ 的账，每块盘）：串下第一道闸左边 = `df` = 容量 − 仍分配 − defer − 隔离 − 60；第二道闸 `可分配` = min(容量 − 仍分配 − defer − 隔离 − 38, 容量 − 仍分配 − 67 − 滞后量) = 第一道闸左边 + min(22, defer − 滞后量 + 隔离 − 7)，而滞后量 ≤ defer，所以 `可分配` ≥ 第一道闸左边 − 7。抬 F 只会在「第一道闸过了、第二道闸没过」时发生，也就是请求落在第一道闸左边往下 7 块宽的带里；删掉的对象在 defer 里、第一道闸照扣，而写回它要的正是第一道闸过不去的那一截，所以抬 F 不会为这次写回而发生，只能等别的请求恰好落进那条带、顺手抬一次。

几何敏感性（`outputs/z3-sweep.txt`，容量 300 / 600 / 1200 块 × 对象 4 / 16 / 64 块 × c_max 4 / 9，c_max = 9 时切换预留取 `28-挂载期承诺量.md:88` 的 116 块、checkpoint 保留池取 9 块）：乙′ 串 18 格里 11 格第 23 次改用户可见状态的发布之后才写回（删掉的那一版要等环里最旧的根越过它的释放代）、7 格删光也没写回；甲 串 18 格全没写回。没有一格落进 ⚠️ 的 6 以内，判定不翻面。

四句：① 分辨（同一段历史上合第 3 次、串-重判第 5 次写回，`outputs/z3-selftest.txt` 第 8、15 行）；② 看得到（抬 F 的上限、可放回的释放代都在内存里）；③ W3 后半「删掉的空间在界 3 之内回不来」。`03-空间分配.md:369` 那条 ⚠️ 承认的是 E139 上「主格第 1..3 窗口、保留池扫描里到第 6 窗口」越过界 3，这一格在基准几何上删光全部对象也回不来、扫开之后最快也要第 23 次，不在那几格里；④ 改法：串-重判、合都在这一格上写得回来。

### 3.3 合：`df` 报出 ≥ s 而写 s 失败（打中三截）

三截各有一格控制（只改一个量，那一截跟着出现 / 消失，并且停在它该停的那一道闸），`outputs/z3-selftest.txt` 第 16–23 行整行：

```
16:ok   乙′ 合：回退隔离 36 块之后按 D16 的 df 写，第一道闸拒 ⇒ 假性 ENOSPC
17:ok   乙′ 合：两次重开之后按 D16 的 df 写（空发布放掉的不算滞后量）⇒ 假性 ENOSPC
18:ok   甲 合：式子里「已分配」含 defer，抬 F 放得回来的那一截仍被扣一次 ⇒ 假性 ENOSPC
19:ok   ① 不重开、不对齐：上一次准入推空放掉的固定点，D16 的 df 算可用、合的第一道闸不算 ⇒ 第一道闸拒
20:ok   ① 的控制：第一道闸把空发布放掉的也算可用 ⇒ 归 0
21:ok   ② 对齐之后重开两次：写行与暖机放掉的超过 D16 残留那一截 ⇒ D16 自己的第二道闸拒（df 仍报 6）
22:ok   ③ 的控制：对齐、隔离 0 ⇒ 归 0
23:ok   ③ 对齐、隔离 36 块：D16 的 df 没有被抛弃根独占量那一项 ⇒ df 报 38 时写 16 块被第一道闸拒
```

- **① 推空放掉的固定点**。`16-发布语义.md:378` 的 df 把「已释放的」都算进来，只扣「滞后量」，而滞后量的定义逐字排除「空发布放掉的已释放块」，由「推空最坏残留 5 + 6 c_max」统一兜着；合的第一道闸只给「抬 F 放得回来的」记可用，空发布的释放代 > 上限，放不回来。两边扣的常数差：D16 扣 38 + 29 = 67，D28 扣 56 + 4 = 60，只差 7 块；上一次准入推两次空发布（c_max = 4 时 8 块）就超过它。H4 不重开那一格：df 报 6、第一道闸 1。
- **② 重开的写行与暖机**。它们也是空发布（事务号 0），D16 的 df 同样不算滞后量；「推空最坏残留」是按一次准入内至多 B 次推空定的，D16 已定项 8 的暖机不在里面。下面抄的 H4 第 247–252 行是没对齐的那一份（① ② 叠在一起，由第一道闸拒）；只留 ② 的那一格是控制矩阵第 21 行（对齐 ① 之后由 D16 自己的第二道闸拒，df 仍报 6），它逐次发布的读数没进 `z3-results.txt`，在模型目录里跑 `python3 -c "import z3_gates as z; h = z.history_remounts_then_write(\"乙′\", \"合\", 300, 16, 2, align_empty=True); print(h[\"false_enospc\"]); print(\"\\n\".join(h[\"log\"]))"` 可看。
- **③ 回退之后的隔离量**。`16-发布语义.md:378` 的 df 没有「被抛弃根独占量」这一项（`28-挂载期承诺量.md:30` 第九项只在 D28 的式子里）；副本上里程碑脚本回退之后隔离 36 块（`outputs/s3-crash-between.txt` 第 1 行 `mem_isolated=36`；04:21 之后的新版本改成窄读法是 34，`outputs/newcrates-s3-crash-between.txt` 第 1 行，仍远大于 7）。H3 第 210–213 行：df 报 38，写 16 块被第一道闸拒。`checks-owed.md:296` C318（影子账隔离的单元没进准入不等式） 那一行的题面逐字已经写过「E139（按盘回退下界的收严形态） 那类没扣进 df 的量会翻成假性 ENOSPC（第四轮探针 > 60 块就出 24 次）」，合把 D16 的 df 交给用户，等于把这一格重新打开。
- **合 × 甲**：式子里的「已分配」含 defer，合只把 defer 那一项减掉放得回来的，已分配里那一份还在，放得回来的块仍被扣一次。H1 里 12 次（`outputs/z3-results.txt` 第 509 行那个列表）。它是第一轮 Y1（双扣）在合上的后果，不单独记输，但它说明「合」只在乙′ 这类第 1 项不含 defer 的账上才站得住。

H4（乙′、合，重开两次）第 244、247、251、252 行与 H3（乙′、合，隔离 36）第 210–213 行整行：

```
244:     txg= 17 写 o12(16)                    F= 10 F_eff= 10 oldest=  0 ceiling= 11 still=215 defer= 28 isolated= 0 restorable=  4 lag= 12 df=   6 gate1=   1 allocatable=  57
247:     txg= 18 重开：写行                        F= 10 F_eff= 10 oldest=  0 ceiling= 11 still=215 defer= 34 isolated= 0 restorable=  4 lag= 12 df=   6 gate1=  -5 allocatable=  51
251:     txg= 22 重开：暖机                        F= 10 F_eff= 10 oldest=  0 ceiling= 11 still=215 defer= 52 isolated= 0 restorable=  4 lag= 12 df=   6 gate1= -23 allocatable=  33
252:               请求 by-df 6 块：df=6 第一道闸 -23 < 6 ⇒ ENOSPC
210:     txg= 15 写 o7(16)                     F=  9 F_eff=  9 oldest=  0 ceiling= 10 still=183 defer= 24 isolated=36 restorable=  4 lag= 12 df=  38 gate1=   1 allocatable=  57
211:               请求 o7 16 块：df=54 成功（推 2 次）
212:               请求 o8 16 块：df=38 第一道闸 1 < 16 ⇒ ENOSPC
213:               请求 by-df 38 块：df=38 第一道闸 1 < 38 ⇒ ENOSPC
```

几何敏感性（同一份 `z3-sweep.txt`）：H3、H4 的假性 ENOSPC 在乙′、甲两套账的 18 个几何上都 ≥ 1；控制格里 ③（隔离）18 格都中、它的控制 18 格都归 0；① 11 格中、它的控制 18 格都归 0；② 9 格中。③ 不随几何翻面；① ② 只在一部分几何上单独出现，叠在一起的 H4 18 格都中。串、串-重判三段历史 18 格都是 0 次。

四句：① 分辨（串、串-重判三段历史假性 ENOSPC 都是 0，`z3-selftest.txt` 第 1、3、4、6 行）；② 看得到（隔离量、空发布放掉的块、上限都在内存里）；③ W3 前半「`df` 报出 ≥ s 而写 s 失败」，与 `03-空间分配.md:363` 第 1 条字面同一句；它不在 `:369` 那条 ⚠️ 里（⚠️ 只管第 2 条的界）；正文第五节给合写的推翻观测逐字「合让 D3（空间分配） 已定项 9 第 1 条在某段历史上不成立」，按字面触发；④ 改法：把 D16 的 df 补上隔离量、把空发布放掉的块按实际能不能放回算，① ③ 两格归 0（控制 20、22 行）；② 那一格要另把暖机算进残留，没量。

### 3.4 串-重判：第 1 条没打中；第 2 条不稳定

`outputs/z3-selftest.txt` 第 3、6 行整行：`Z3 arm=乙′ reading=串-重判 H1_steps_to_reuse=5 H1_false_enospc=0 H3_false_enospc=0 H4_false_enospc=0`、`Z3 arm=甲 reading=串-重判 H1_steps_to_reuse=8 H1_false_enospc=0 H3_false_enospc=0 H4_false_enospc=0`。用户看到的 `df` 就是第一道闸左边，三段历史一次假性 ENOSPC 都没有。第 2 条：乙′ 删后第 5 次、甲第 8 次才写回。多出来的几次是这段历史的形状造成的：池满到连 1 块的覆盖写都被第一道闸拒，用户只能拿「删下一个对象」当改用户可见状态的发布，每删一个、加上为它推的两次空发布，又有一批块进 defer、放不回来。乙′ 那一格落在 `03-空间分配.md:369` ⚠️ 逐字承认的「保留池扫描里到第 6 窗口」以内，按正文的要求先排除；但扫开几何之后（`outputs/z3-sweep.txt` 第 1、4、7、10、13、16 行，全是对象 4 块的几何）乙′ 串-重判是第 7 次，越过 6，另外 6 格 ≤ 3、6 格 4–6。按 `.claude/rules/mutation-sampling.md` 第六类「翻面就记「不稳定」」，这一格记**不稳定**，不记输也不记没打中：删掉的对象小到与每次发布放掉的固定点同一个量级时，写回它要的改用户可见状态的发布数会越过 6。甲那一族（12 格 > 6、4 格删光也没写回）的病根是式子里 defer 被扣两次（第一轮 Y1，辩方复核的那一格），不另记。第 1 条：18 个几何、两套账，假性 ENOSPC 都是 0。

写行那次发布之前不推（`16-发布语义.md:377` 括注）在三种读法上各走到哪一步（推的，模型里没有在飞与崩溃）：写行是新实例的第一次发布，元数据走切换预留；同一次发布里的用户数据是崩溃之前没发布的在飞写的重做，照走准入、不够就 ENOSPC、不推。在飞写的准入在崩溃之前已经过了：若当时为它推过空发布并且 F 已生效，重开时 `mount.rs:209-234` 按生效的 F 回收，重做拿得到同一片空间；若崩在推空中间，那次写的准入还没返回，用户手里没有「df 报过 s」的承诺。三种读法在这一格都**没打中**；它依赖 D28 已定项 2 在飞合成的实现形态，模型没建。

### 3.5 W5：Z3 三种读法要改的地方

| 读法 | 条款 | `crates/` |
|---|---|---|
| 串 | 不改字；但 `03-空间分配.md:367` 第 2 条的界 3 在这种读法下兑现不了（3.2） | 准入式今天没有实现（正文第三节的 grep 零命中） |
| 合 | `28-挂载期承诺量.md:90`「`df`：可用已经扣掉这一项，`df` 报可用」改写；`28-挂载期承诺量.md:21` 式子里 defer 那一项改成「减去抬 F 放得回来的」；`16-发布语义.md:378` 的 df 要补「被抛弃根独占量」、把空发布放掉的块按能不能放回算、把暖机算进残留（3.3 ①②③）；`05-快照-空间记账机制.md:338` 起的式子逐项对照表跟着改；只在第 1 项不含 defer 的账上成立（3.3 合 × 甲） | 同上 |
| 串-重判 | `03-空间分配.md:430` 已定项 12 要补「第一道闸不过时推一次抬 F 的空发布、重判，至多 B 次」；`16-发布语义.md:377` 准入行同步 | 同上 |

## 四、Z5 可验证性：Z1 五臂、Z2 两臂 × 三份坏镜像

副本上造坏镜像的办法（`patch-copy.py`，写者一侧改，checker 一侧不动或按臂改）：① `SFPROBE_LEAK_TXG=5`：txg 5 的覆盖写不释放上一版的 extent 根（槽泄漏，不释放、记账不减）；② 不抬 F 就把释放代 ≤ 11 的落点回收再覆盖写（`probe.rs` 的 `scenario_reuse_candidate`，与步 5 验收那条必红的形态同形），以及 2.1 的窗口崩溃态；③ `SFPROBE_DEFER_PLUS=1`：记账第 5 项在盘 0 上多报 1 槽，其余不动。另加一种 ③′：分配器一直不回收（defer 多报、空闲同步少报），就是 1.1 的 S1。短历史 S6：A → 覆盖写 4–9 → 抬 F 到 5（10、11 两次空发布）→ 覆盖写 12。

### 4.1 判红表（副本读数；「—」= 所有检查全绿）

| 臂 | ① 槽泄漏 | ② 候选根引用的槽被复用 | ③ defer 单独多报 | ③′ 一直不回收 | checker 与运行时是不是两套算法 |
|---|---|---|---|---|---|
| 甲-T0 | I-3.1，泄漏那一槽的根出候选集之后才红（txg 5、9 绿，11 红） | I-2.1、I-3.1、I-5.1 | **—** | I-3.1 | 是：走读取并集 vs 分配时增量维护 |
| 甲-T1 | 同甲-T0 | 同甲-T0 | **—** | I-3.1 | 是 |
| 甲-a | 最弱读法（上界取记账第 5 项）**—**；上界从分配记录重数时 I-3.1 红（F 生效之后） | 同甲-T0（走读没改） | **—**（两种上界都是） | 最弱读法 **—**；重数上界 I-3.1 红（txg 26 起） | 半：重数的上界就是运行时的回收谓词再算一遍 |
| 甲-b | I-3.1（推的） | 同甲-T0 | **—**（推的） | I-3.1（推的） | 挂载那一刻不是：运行时按「有效根的分配记录取并集」、checker 按「有效根的走读取并集」，根集合的定义两边共用 |
| 乙′（加上改写的 I-5.2） | I-3.1，当场红（txg 5） | I-2.1、I-3.1（不等式）、I-5.1 | I-5.2（改写后的） | **—** | 是 |
| 乙′（按它自己列的，I-5.2 不改） | I-3.1 当场红，但每个合法状态 I-5.2 也红 | 同上 | 每个状态都红，分不出 | **—** | 是 |
| F-今（checker 取最新根的 F） | I-3.1，F 生效之后 | 窗口外 I-2.1；**2.1 的窗口崩溃态 —** | **—** | I-3.1 | 候选集的 F 取自被审计的写者自己写的那个字段 |
| F-生（checker 现算 F_生效） | 同上 | 窗口外 I-2.1；窗口崩溃态 I-2.1、I-3.1、I-5.1 | **—** | I-3.1 | 是：checker 与恢复各写一份「各盘所带 F 最大值的最小值」 |

逐格的出处（整行都在 `outputs/`）：
- 甲-T0 ①：`p-s6-T0-Fjin-leak5.txt` 第 9 行 `S6 after-raise txg=11 inst=1 F_root=5 F_eff=5 acct_alloc=61 acct_free=211907 acct_defer=48 mem_alloc=61 mem_defer=48 mem_isolated=0 | I-3.1[盘 0：记账的已分配 Some(999424)，遍历全部有效根得到 983040]`；同一文件第 3、7 行 txg 5、9 `| GREEN`。
- 甲-T1 ①：`p-s6-T1-leak5.txt` 第 9 行与上一行逐字段相同。
- 甲-T0 ③：`p-s6-T0-Fjin-deferplus.txt` 第 10 行 `S6 overwrite-after-raise txg=12 inst=1 F_root=5 F_eff=5 acct_alloc=70 acct_free=211898 acct_defer=59 mem_alloc=70 mem_defer=58 mem_isolated=0 | GREEN`（记账 59、分配器 58）；十行全绿。甲-T1 ③：`p-s6-T1-deferplus.txt` 第 10 行 `S6 overwrite-after-raise txg=12 inst=1 F_root=5 F_eff=5 acct_alloc=70 acct_free=211898 acct_defer=59 mem_alloc=70 mem_defer=58 mem_isolated=0 | GREEN`。仓里的已知坏镜像表也没有这一份：`checker_known_bad_images.rs` 里 以 `adjust_accounting(bytes, ` 开头的调用只出现在第 329 行（统计量 1）与第 356 行（统计量 2）。
- ②：`p-s7-reuse-candidate-T0.txt` 第 2 行 `S7 reuse-without-raising-F txg=15 inst=3 F_root=0 F_eff=0 acct_alloc=52 acct_free=211916 acct_defer=40 mem_alloc=52 mem_defer=40 mem_isolated=36 | I-2.1[树表单元 在盘 0 槽 50178 的那一份与位置条目里的校验和对不上] ; I-3.1[盘 0：记账的已分配 Some(851968)，遍历全部有效根得到 1196032] ; I-5.1[盘 0：树表单元（槽 50178 跨 1）与 数据单元（槽 50178）重叠]`；乙′ 同一格 `p-s7-reuse-candidate-yiprime.txt` 第 2 行 `S7 reuse-without-raising-F txg=15 inst=3 F_root=0 F_eff=0 acct_alloc=12 acct_free=211916 acct_defer=40 mem_alloc=52 mem_defer=40 mem_isolated=36 | I-2.1[树表单元 在盘 0 槽 50178 的那一份与位置条目里的校验和对不上] ; I-3.1[乙′ 盘 0：记账第 1 项 Some(196608)，最新根可达 196608；候选并集 1196032，defer 655360] ; I-5.1[盘 0：树表单元（槽 50178 跨 1）与 数据单元（槽 50178）重叠]`。窗口崩溃态：`s4l-legal-hit-k1-1-k2-6-f8.txt` 第 4 行（F-今，全绿）与 `p-s4l-legal-hit-k1-1-k2-6-f8-checkerF-effective.txt` 第 4 行（F-生 的 checker，三条红），两行都抄在 2.1。
- 乙′ ①：`p-s6-yiprime-I52-leak5.txt` 第 3 行 `S6 overwrite txg=5 inst=1 F_root=0 F_eff=0 acct_alloc=13 acct_free=211935 acct_defer=20 mem_alloc=33 mem_defer=20 mem_isolated=0 | I-3.1[乙′ 盘 0：记账第 1 项 Some(212992)，最新根可达 196608；候选并集 540672，defer 327680]`。
- 乙′ ③：`p-s6-yiprime-I52-deferplus.txt` 第 1 行 `S6 A txg=3 inst=1 F_root=0 F_eff=0 acct_alloc=12 acct_free=211955 acct_defer=2 mem_alloc=13 mem_defer=1 mem_isolated=0 | I-5.2[盘 0：空闲 Some(3472670720) + 已分配 Some(196608) ≠ 单元区 Some(3472883712)]`。消息文字是今天 `walk.rs:848` 的模板，没把 defer 印出来；判定按改写后的三项之和：3472670720 + 196608 + 32768 = 3472900096 ≠ 3472883712，差的 16384 就是多报的那一槽。
- 乙′ ③′：`p-s1-turnover-yiprime-noT1.txt` 第 25 行 `S1 overwrite txg=27 inst=1 F_root=0 F_eff=0 acct_alloc=12 acct_free=211715 acct_defer=241 mem_alloc=253 mem_defer=241 mem_isolated=0 | GREEN` 与第 30 行 `S1 final-remount+warm txg=34 inst=2 F_root=0 F_eff=0 acct_alloc=12 acct_free=211712 acct_defer=244 mem_alloc=256 mem_defer=244 mem_isolated=0 | GREEN`；同一段历史甲-T0 是 1.1、1.2 那几行红。`p-s5-hole-yiprime-T0.txt` 到 txg 52 全绿（第 28 行 txg 50 `acct_defer=471`，而甲-T1 在同一格回收到 230）。
- 甲-a ①：`p-jiaa-jiaa_weak-short-leak5.txt` 第 9 行 `S6 after-raise txg=11 inst=1 F_root=5 F_eff=5 acct_alloc=61 acct_free=211907 acct_defer=48 mem_alloc=61 mem_defer=48 mem_isolated=0 | GREEN` 与 `p-jiaa-jiaa_exact-short-leak5.txt` 第 9 行 `S6 after-raise txg=11 inst=1 F_root=5 F_eff=5 acct_alloc=61 acct_free=211907 acct_defer=48 mem_alloc=61 mem_defer=48 mem_isolated=0 | I-3.1[盘 0：记账的已分配 Some(999424)，遍历全部有效根得到 983040]`；③：`p-jiaa-jiaa_exact-short-deferplus.txt` 第 10 行 `S6 overwrite-after-raise txg=12 inst=1 F_root=5 F_eff=5 acct_alloc=70 acct_free=211898 acct_defer=59 mem_alloc=70 mem_defer=58 mem_isolated=0 | GREEN`（`jiaa_weak` 那一份同一行也是 `GREEN`）；③′（写者用甲-T0、不回收）：`p-jiaa-jiaa_weak-turnover-noT1.txt` 第 25 行 `S1 overwrite txg=27 inst=1 F_root=0 F_eff=0 acct_alloc=253 acct_free=211715 acct_defer=241 mem_alloc=253 mem_defer=241 mem_isolated=0 | GREEN` 与 `p-jiaa-jiaa_exact-turnover-noT1.txt` 第 25 行 `S1 overwrite txg=27 inst=1 F_root=0 F_eff=0 acct_alloc=253 acct_free=211715 acct_defer=241 mem_alloc=253 mem_defer=241 mem_isolated=0 | I-3.1[盘 0：记账的已分配 Some(4145152)，遍历全部有效根得到 3964928]`。
- 甲-b：副本上没实现；它的 checker 就是甲-T1 那一条等式，① ② ③ 三份坏镜像在挂载内的读数与甲-T1 那几行相同，表里写「推的」的是挂载那一刻运行时怎么算。

### 4.2 按反向接受条款判 W4

- **③ 这一族五条 Z1 臂一起中，不分辨**：甲-T0、甲-T1、甲-a、甲-b 在「defer 单独多报」上全绿；乙′ 加上改写的 I-5.2 抓得住它，却在「分配器一直不回收」（③′）上全绿；乙′ 按它自己列的改法，I-5.2 在每个合法状态都红，分不出坏镜像。每条臂都有一种 defer 多报一个检查都判不出，按正文第六节反向接受条款，这一格不拿来判它们，另记一笔共用前提的账：**第 5 项 defer 没有任何一条检查按它自己的定义独立核**——`walk.rs` 只读统计量 1、2（`walk.rs:36-37` 的 `STATISTIC_ALLOCATED_BYTES`、`STATISTIC_FREE_BYTES` 两个常量，`grep -rn "STATISTIC_\|defer" crates/singlefs-checker/src/` 只命中这两个常量与它们在 `:838`、`:846` 的两次使用）。

  四句（③′ 在乙′ 上）：① 这一格单看是分辨的（甲-* 红、乙′ 绿），但与 ③ 合起来五条臂各输一种形态，所以整族记不分辨；② 看得到：checker 读得到最新根的分配记录（每条带释放代）与环，算得出「已释放 ∧ 释放代 > 这条根持久之后的门槛」应有多少；③ W4「某条臂下某一份坏镜像全部检查判绿」；④ 正文的倾向理由「它的等式不受环上的洞影响」与这一格是同一件事的两面：乙′ 的不等式放过洞上多扣，也就放过一直不回收，因为两者在「并集 − 可达 ≤ defer」里长得一样。**今天代码里真实存在的 1.1 那个错，在乙′ 下没有任何检查判得出**（`p-s1-turnover-yiprime-noT1.txt` 全绿）。正文第五节给乙′ 写的推翻观测逐字是「Z5 的坏镜像 ③（defer 多报）在乙′ 下没有任何检查判得出」，按字面在 ③′ 这一形态上触发；按反向接受条款，③ 族五条臂各输一种形态，这一格不拿来单判乙′，交主 agent 定。
- **① 分辨甲-a（最弱读法）**：上界没写怎么算，最弱读法取记账第 5 项，1 槽泄漏被几十槽的 defer 盖住（副本 `p-jiaa-jiaa_weak-short-leak5.txt` 九行全绿），③′ 也一样全绿。记甲-a 在 W4 输一次（最弱读法）。精确读法下它抓得住，但要 checker 把运行时的回收谓词再算一遍（`.claude/rules/fs-design.md:36` 那一句逐字「第二条不依赖任何关于性能的假设——它要求的恰恰是**运行时与 checker 必须用不同的算法**。」）。
- **② 在窗口崩溃态上分辨 F-今**：2.1 那个镜像今天的 checker 全绿、F-生 的 checker 三条红。记 F-今 在 W4 输一次，与 2.1 的 W2 是同一段历史。四句：① 分辨；② 看得到（各盘所带 F 都在盘上）；③ W4 字面；④ 改法「checker 按恢复的定义现算 F_生效」在这一格上不中，但只配 F-生 的记账才不在崩溃态上误红（2.2 最后一段）。
- **甲-b 的「两套算法」**：挂载那一刻运行时的量与 checker 的并集都是「对有效根的集合取并」，差别只剩「分配记录 vs 走读」；根集合的定义（按实例表有效 ∧ txg ≥ 哪个 F）一旦写错，两边一起错，I-3.1 看不见。这正是 `.claude/rules/fs-design.md:24` 那一行（整行：`| **checker / 审计** | **必须**遍历 | 若运行时也用遍历算，checker 的遍历与运行时就是**同一次计算**，对照关系当场归零 |`）说的形态（推的，没实现甲-b）。记甲-b 在「两套算法」那一问上输一次（最弱读法）。

### 4.3 层 0 的崩溃状态数

五条 Z1 臂与两条 Z2 臂都不改一次发布写几个块、几道屏障：甲-T1 / 乙′ / F-生改的是记账行的值与回收发生在哪一刻，甲-a 只改 checker，甲-b 在挂载时多读至多 24 棵分配记录树（读不进录制流）。按 `crates/singlefs-harness/src/crash.rs:335` `closed_form_state_count` 的算法（按段数算），状态数不变。**推的，没在副本上跑层 0。** 另记一句：CLAUDE.md 写的层 0 负载是「第一个事务，以及覆盖写 + 释放 + 重开写行 + 暖机 + 发布 C」，没有抬 F，所以 2.1 那一格今天的层 0 走不到。

## 五、按反向接受条款记账（主 agent 判，本腿只列）

| 臂 | 在哪一格输一次（最弱读法） | 一起中、不拿来判它的格 |
|---|---|---|
| 甲-T0 | W1：1.1（同一挂载转环）、1.2（满环上重开） | W1 洞（1.3，与甲-T1、甲-b 一起）；W4 ③ 族 |
| 甲-T1 | — | W1 洞（1.3）；W4 ③ 族 |
| 甲-a | W4 ①（上界取第 5 项） | W4 ③ 族 |
| 甲-b | W4「两套算法」（4.2） | W1 洞（挂载内同甲-T1）；W4 ③ 族 |
| 乙′ | W1 / W5：I-5.2 不在它列的改法里、照列的改就每个状态红（1.4） | W4 ③ 族（③′ 那一格） |
| F-今 | W2 + W4：2.1 的窗口崩溃态 | — |
| F-生 | W1：记账行按写那一刻的 F_生效时第二条带新 F 的根上红（2.2） | — |
| 串 | W3 第 2 条（3.2） | — |
| 合 | W3 第 1 条，三截（3.3） | 合 × 甲 那一截（第一轮 Y1 的后果） |
| 串-重判 | —（第 2 条记不稳定：乙′ 18 个几何里 6 格越过 ⚠️ 的 6，3.4） | 甲那一族越过 6（第一轮 Y1） |

共用前提的账（不回落到任何一条臂）：
1. **defer（第 5 项）没有独立的检查**（4.2）。
2. **记账行的回收时点没有条款**（第一轮账 2 还在）：甲-T0 在 1.1、1.2 输；F-生 的两种读法（2.2）就是这一问在 F 上的形态。
3. **环上有洞时，谓词多扣**（第一轮账 3 还在）：甲-T0 / T1 / b 一起中；乙′、甲-a 按构造绕开，但绕开的代价在 4.2。
4. **`16-发布语义.md:378` 的 df 与 `28-挂载期承诺量.md:21` 的式子扣的东西不同**（被抛弃根独占量、空发布放掉的块、暖机、常数差 7 块）：在串与串-重判下它藏在第二道闸里不露面，在合下交到用户手里就是 3.3 的三截。

## 六、本腿提的改法（每一条都只在本腿的模型上量过、被攻过零轮）

| 编号 | 改法 | 写成对实现的改动 | 本腿量到的 | 在打中的哪几格上不起作用 |
|---|---|---|---|---|
| G7 | F-生，且记账行按「这条根持久之后的 F_生效」写（与甲-T1 同一个时点），checker 按恢复的定义现算 F_生效 | `mount.rs:354-367` 回收挪到循环之后；`transaction.rs:1272-1286` 三行记账按持久之后的门槛扣 / 加；`walk.rs:786-790` 现算 | 副本 S3 里程碑脚本（崩溃态、崩溃后重开、崩溃后回退、生效、E）全绿；合法操作搜索子集 1942 格 0 次打中；加乙′ 写者也全绿 | 1.3 的洞；4.2 的 ③ 族 |
| G8 | 给第 5 项立一条独立检查：checker 从最新根的分配记录数「已释放 ∧ 释放代 > 这条根持久之后的回收门槛」，逐盘与第 5 项相等 | `walk.rs` 读分配记录树（今天只走、不读条目的释放代）加一条判定；`invariants.md` 立一条 | **没量** | 它用的门槛就是运行时的回收谓词，`.claude/rules/fs-design.md:36` 够得着它；洞上两边一起多扣，等式照样成立，所以它不打算管 1.3 |
| G9 | 乙′ 的 I-5.2 跟着改成「空闲 + 第 1 项 + defer == 单元区」 | `walk.rs:847`；`invariants.md:167` | 副本：合法状态全绿、③ 红（1.4、4.1） | ③′（4.2） |
| G10 | 合的 df 补上被抛弃根独占量、空发布放掉的块按「抬 F 放不放得回来」算 | `16-发布语义.md:378` | Z3 模型控制矩阵：① ③ 两截归 0（`z3-selftest.txt` 第 20、22 行） | ②（暖机超出残留）没量；合 × 甲 那一截 |
| G11 | 抬 F 的那几次空发布开新聚簇段时绕开「已回收、F 还没生效」的槽（F-今 的另一种修法） | `allocator.rs:297-312` 的 `lowest_empty_segment` 多一个合取 | **没量** | 用户数据走 `lowest_user_data_slot`，同一窗口里若有用户写照样拿得到回收的偶数槽对；今天的循环里没有用户写 |

## 七、旁：读代码时看到、不归这一轮判的

1. `raise_rollback_floor` 在一个从没做过可写挂载的进程里会 panic：`mount.rs:333` 整行 `    let table = InstanceTableRecords::parse(&current.unit(TransactionUnit::InstanceTable).bytes)`，而 `transaction.rs:535` 整行 `            .expect("八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在")`——实例表单元要等可写挂载写行时重写过才在 `units` 里。副本探针第一版照这一行写，单实例的搜索当场 panic（那次输出整行 `thread 'main' (1556105) panicked at crates/singlefs-core/src/transaction.rs:535:14:`，下一行是上面那句 expect 的消息；那份输出已被重跑覆盖，没留档），之后改成从盘上最新根读实例表。今天步 5 的验收都在回退之后调它，走不到。归步 4 / 步 5 那一轮。
2. `allocator.rs:524-525` 的注释「环里最旧有效根恒 0，`floor` 就是 F_生效」只到 txg 23 为止（1.1）；`mount.rs:354-356` 注释最后一句不成立（2.1）。
3. `checker_known_bad_images.rs` 的已知坏镜像表里没有 defer 那一份（4.1 ③ 那一条的出处）。

## 八、没打中的形状

| 攻的 | 试过的形状 | 取样范围 | 结果 |
|---|---|---|---|
| 甲-T1 在 W1 上 | 同一挂载转环到 31；txg 26 之后重开；到 31 之后重开 | S1 两段，副本 | 全绿（有洞那一格另算，1.3） |
| 甲-a 在 W1 上 | 写者甲-T1，checker 换成甲-a 的两条（两种上界）：同一挂载转环到 31 再重开、txg 26 的根槽写丢掉后覆盖写到 52、S6 短历史 | 副本 `p-jiaa-*-turnover.txt`、`p-jiaa-*-hole.txt`、`p-jiaa-*-short-legal.txt` 六份 | 全绿 |
| 甲-T1 / F-生 在 W2 上 | k1 次覆盖写 → 重开 → k2 次覆盖写 → 抬 F 到 (旧 F, 上限] 每个值，每次发布写的槽是否落在 [旧 F, 新 F) 的有效根引用的槽上 | k1 ≤ 8、k2 ≤ 16，1942 格 | 0 行 |
| 乙′ 在 W1 上 | 转环、满环重开、洞（T0 / T1 两种回收时点）、里程碑脚本含回退与抬 F 的崩溃态 | S1、S5、S3，副本 | 按「I-5.2 跟着改」全绿；按它自己列的（I-5.2 不改）每个状态红（1.4） |
| F-生（持久之后读法）在 W1 上 | 里程碑脚本崩在两条带新 F 的根之间、崩溃后重开、崩溃后回退到 (3, 9)、生效、E | S3，副本 | 全绿 |
| F-生 的 P1（F_生效 数哪些根） | 被抛弃根带着更高的 F；两个实例各抬一半、各崩一次，第三次挂载 F_生效 = 两个半截 F 的较小值 | 推的，没在副本上造 | 没找到让已回收的槽仍被候选根引用的状态：每个实例在它自己看来 F 没生效就不回收，第三次挂载按生效值回收时那些根已经不是候选 |
| 串-重判 在 W3 第 1 条上 | 删后写回、回退隔离 36 块后按 df 写、推空之后重开两次再按 df 写 | Z3 模型，乙′ 与甲各三段，基准几何加扫开的 18 个几何 | 假性 ENOSPC 0 次 |
| 三种读法在「写行那次发布之前不推」上 | 崩在推空之前 / 之间 / 之后，重开时重做在飞写 | 推的（3.4） | 没找到「df 报过 s、准入返回过、重做 ENOSPC」的历史 |

「没打中」都是确定性计算（副本探针没有随机源，Z3 模型纯算术），同一份输入跑 N 遍一样；强度来自控制矩阵与判别力格（坏镜像在对应的臂上由绿转红），不来自次数。

## 九、这条腿自己的限度

- 副本装置上的数不是入库装置上的数；要引，主 agent 得在入库装置上重做（`.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」）。
- 抬 F 在副本上是照 `mount.rs:331-399` 一行一行重写的 `raise_floor_manual`，不是直接调 `raise_rollback_floor`（它停不在两次发布之间，且单实例进程里会 panic，第七节 1）；两处差别：实例表从盘上最新根读，循环上限写成参数。F-生 的回收位置、甲-T1 的记账口径、乙′ 的写者与 checker、F-生 的 checker，都是本腿在副本上写的实现（`patch-copy.py` 18 处替换；其中甲-a 的 5 处是在 `run-probe.sh` 第一次复跑开始之后加的，见第十节），不是任何人审过的实现。
- 洞是用「根槽写静默丢掉」造的，不是条款里「收到写错误、推进一格重发」那条路（1.3 写了两者的异同）。
- 甲-a 只在副本上实现了 checker 那一半（两种上界，写者用甲-T1 或甲-T0）；甲-b 没有实现，它的格子是按正文 Z1 表的定义推的。
- 合法操作搜索只有一个族（覆盖写、一次重开、覆盖写、抬 F），没有回退在抬 F 之前、没有多次抬 F、没有用户写夹在推空中间；最后一次发布上的 47 格没逐格造「单元写完、根没写」的崩溃态。
- Z3 模型的基准几何是每块盘 300 块、对象 16 块、c_max = 4；`--sweep` 扫了容量 300 / 600 / 1200、对象 4 / 16 / 64、c_max 4 / 9 共 18 格（3.2、3.3、3.4 各写了翻不翻面）。没扫的：「活元数据」（恒取 0）、需求含不含固定点（恒不含）、隔离量（恒 36）、重开次数（恒 2）、三块盘；没有在飞合成与崩溃。判决是跑出来之后才写的，不是跑前写死的：这一轮正文第六节的 W3 是跑前判据，本腿的三段历史与控制格是看了第一版读数之后补的（第一版的「判别力」三格没过，改成控制矩阵；`outputs/z3-selftest.txt` 是改过之后的）。
- 层 0 没跑；4.3 是推的。
- Z3 模型的隔离量 36 取自副本上里程碑脚本的读数，模型本身不建回退。

## 十、没做什么

- 没碰 Z4，没复核第一轮判决（Y1、Y4-a / b / c、旁 1），没判正推、辩方那几格；没替主 agent 采纳任何改法。
- 没改 `crates/`、kb、门禁；编译只在 `/tmp/claude-1000/alloc-basis-r2-opus/` 的两份副本里（`CARGO_TARGET_DIR` 也在那里）。`.claude/gate.d/stage-owners.tsv` 里没有登记给 `three-way-attack` 的阶段（`awk` 查询输出为空），没跑门禁。
- 跑的时候 `ps` 看到两个 `second_transaction_step_zero_layer0` 测试进程各占一个核、两个 `ray::RayWorkerProc`，机器 32 核；探针与 cargo 都加了 `nice -n 19`。
- 没读禁读清单里的文件（`alloc-basis-r2-sonnet*`、`alloc-basis-r2-local-*`、`alloc-basis-r2-main-verification.md`、`_m2-step45-code-r2-*`、`m2-step45-code-r2-*`）。
- 04:26 UTC 收到主 agent「暂停一切重的跑批」：当时在跑的只有本腿的复跑（`run-probe.sh`，pid 1560868；它起的探针 `legal-search 8 16`，pid 1889108），`ps` 列出之后按写死的 pid `kill`，再 `ps` 确认没有残留；复跑日志末行 `exit=143`。那之后本腿只读文件、写报告、算 sha256，没有再编译、没有再跑探针或模型。**没重跑的**：`bash research/prompts/alloc-basis-r2-opus-model/run-probe.sh` 整条（甲-T1 + F-生 那一份搜索、甲-a 的 12 份、`z3_gates.py --sweep` 都没被复跑罩到），等主 agent 说可以继续再跑。
- 改自己的模型文件与报告时，除了 `research/scripts/replace-once.py`，也用过几段内联 python 做同样的事（每处旧串先断言命中次数、写完回读比对）；没有整份重写过已有文件，`outputs/` 里的 `SHA256SUMS` 与两份 z3 产物是删掉旧的再生成的。
