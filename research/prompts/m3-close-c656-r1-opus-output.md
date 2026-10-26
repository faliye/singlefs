# m3-close-c656-r1 攻方腿（Opus）报告（2026-10-20）

轮名 m3-close-c656-r1，第一轮。数全是本腿草稿目录里仓副本上量的（副本 `crates/` 起点整树哈希 `3bcd43eafa2159bb44e65fdb853a7643fadfe254fb8eeb038aa9234fc1329d25`，与调查员副本同一棵；副本里只加了 `make_filesystem_in_order` 一个函数与探针 crate），不算入库装置上的数。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-close-c656-r1-opus-model/`。复跑（冻结副本 = 仓里 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo` 的一份拷贝，`crates/` 整树哈希应为上面那个数，脚本开头打出来）：

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m3-close-c656-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录>
```

跑完比 `<草稿目录>/out/run-*.log` 的 `name=done`、`name=count` 行与模型目录 `out/*.txt` 同名行（`name=example` 的序号随线程次序变，不比）。`--selftest` 过了（「✓ 自检通过（查了 5 格）」）；整条复跑没在模型目录这一份上跑过，各件是逐条在草稿目录跑的（命令形见第七节）。

```
3b587158ec40f050d513ffc5b1e007ec63d10a8fe737c398f994cc038b33b0d9  ./out/count-and-image.txt
8a0dd415a18e2da4a8ea484980cc08a877331fa389fbec72b1f47d27011c1a90  ./out/invalidate.txt
e689a9118839155fb6b98691d649edb76d7fcec8953a7016bc270b45e8c8b28c  ./out/new-then-sysconf.txt
297f232172905bf492d48459e49ac383984b889de4be0969790004fa7349d682  ./out/today-partial-examples.txt
fb3a1e7c2fc8120a6ce9e06286e6f277a6539e975ca945faca15a4a5b809d19b  ./patches/make_filesystem-order.patch
1a7aae7bfab678e8d459e25ef55b99ed838ed8f757f07b072ecf55ab6d162e0e  ./patches/workspace.patch
60893ccb15c1de5e72236d853e1155cf18f3415291a1e104274cbae85bd8fb76  ./probe/Cargo.toml
d865c4a4824d3257563956710607d10382f235083740ba284f1ae3ea5dbdeb2a  ./probe/main.rs
4fadb696833363bfae5de42b677a964cda70d64e94018c4db0de04f1c0b58ad3  ./tools/rerun.sh
```

装置：旧池两块盘（journal 环 6 MiB、单元区 4096 槽的稀疏内存盘）在同一个进程里 mkfs → 取号 → 暖机 → 第一个文件 → u−2 次覆盖写（实例 1，最后确认的一版是 (1, u+1)）；新 mkfs 两块盘都重用、新池 io_min 512、fsid 由历史名定。崩溃状态走 `enumerate_crash_states_with_expansion`，每一段都按 `EveryProperSubset` 展开（与 `enumerate_crash_states_versions` 同一个展开；原地覆写取三态），挂一个观察者逐态记：择到哪份系统配置、恢复读回什么、池级 checker 红不红、只读挂载、可写挂载（按择到那一池的参数）。观察者在编译时的判器键表里没有组，库照判、不入库（`CRASH_CASE_BLOCKS_KEY_REFUSED`），所以这一趟不能断点续跑。

写序（`patches/make_filesystem-order.patch`，字节与 `make_filesystem` 逐字节相同，只换次序）：
- today：同 `make_filesystem`。
- invalidate（乙）：每盘清 [0, 根环基址) → 屏障 → 清根环三区域、journal 环、写两单元 → 屏障 → 根 FUA × 3 → 系统配置 × 4 → 屏障。
- sysconf-first（甲，提交点 = 系统配置先写）：系统配置 × 4 → 屏障 → 清 [槽 1 末尾, 根环基址)、根环、journal 环、写两单元 → 屏障 → 根 FUA × 3 → 屏障。
- new-then-sysconf（甲，正文举的例：先在旧池不读的地方写、最后动系统配置槽）：两单元 → 屏障 → 根 FUA × 3（只写槽 0）→ 系统配置 × 4 → 屏障 → 清 [槽 1 末尾, 根环基址)、各区域槽 1..S、journal 环 → 屏障。

历史名 `o<旧 io_min>-u<u>-f<same|diff>`：fsame = 旧池与新池 fsid 相同。

## 各格判定一览

| 格 | 判定 | 量过 / 推的 | 一句话 |
|---|---|---|---|
| V1 甲做不做得到 | **打中** | 量过（甲那两种写序各一份）+ 推的（卡在哪一类字节） | fsid 相同时提交点一过就挂上新旧混合：new-then-sysconf 在 o512-u5-fsame 上 7874 态里 6653 态择到新池系统配置、读回旧池的文件，其中 5911 态可写挂载做成；读回更早一版的 160 态。卡在根环：格式把根环基址、journal 环起点钉成第一版常量，新旧两池的根环逐字节重叠，清旧根（C484）与旧池要读旧根不能同时在提交点同一侧 |
| V1 甲（fsid 不同） | 没打中（一份样本） | 量过 | new-then-sysconf 在 o512-u5-fdiff、o512-u23-fdiff 上提交点之前 18 态旧池读回内容都与最后确认的一版相同；但写到了旧池读得到的字节（单元区起点两单元、根环槽 0），字面「不改动旧池读得到的任何字节」不成立 |
| V2 乙撑不撑得住 | **没打中**（一份样本，要 s2） | 量过 | 五份历史、每份 31 150 到 31 195 态全量：作废屏障之前 8 态旧池读回最后确认的一版、checker 绿；屏障之后到系统配置落盘之前全部整池拒（NoValidSystemConfiguration）；混合 0 态。模型外的撕裂形状 12 种 × 3 份历史也全完好 |
| V2 一盘作废一盘没作废 | 会择到没作废那块盘的旧池系统配置（量过），旧池完好；另：择系统配置取第一块可择盘那一份、不取全池最大世代号（量过 7 → 8 那一格） | 量过 | 见第二节 |
| V3 代价 | 给数 | 状态数量过、改动处数现数 | 乙 mkfs 段形 `2+12+1+1+1+4`、新盘闭式 4117（今天 16402）；用过的盘上全量 31 195（今天 280 019）；钉段形的 crates 用例 2 处 |
| V4 没量的几何 | fsid 相同让甲翻面（量过）；乙在 fsid 相同、旧 io_min 65536、u = 6、u = 23 上都不翻（量过） | 量过 | 今天写序在这几样上没跑（超前台时限，命令交主 agent） |
| 第三节的数能不能在入库代码上核 | 总状态数 280 019 在未打补丁的公开函数上对上；分类的数没核完 | 量过（总数）；分类没跑完 | 今天写序全量一趟超过前台 240 秒、观察者键拒算不能续跑，命令交主 agent |

## 一、V1 甲做不做得到

### 打中：fsid 相同，提交点之后挂上新旧混合（量过）

历史 o512-u5-fsame（旧池用新池同一个 fsid、同 io_min，最后确认 (1, 6)），写序 new-then-sysconf，全量 7874 态（`out/new-then-sysconf.txt`）。按段汇总（段号 = 第一个没落的写所在的段；s0–s3 在提交点之前，s4 是系统配置那一段，s5 是提交点之后的清零段）：

| 段 | 态数 | 混合（择新池系统配置而读回旧池内容） | 其中可写挂载做成 (1, 6) | 其中读回更早一版 | 旧池完好 | 新池空池 |
|---|---|---|---|---|---|---|
| s0–s3 | 18 | 0 | 0 | 0 | 18 | 0 |
| s4 | 80 | 29 | 23 | 0 | 50 | 0 |
| s5 | 7775 | 6624 | 5888 | 160 | 0 | 1151 |
| s6 | 1 | 0 | 0 | 0 | 0 | 1 |

原样两行（模型目录 `out/new-then-sysconf.txt`）：
- 「count=4224 s5__torn__new__oldlast__red__ro_i1t6__rw_ok_i1t6__mixed」：择到的是新池那份系统配置（新 fsid、世代号 1、实例代号 0），恢复读回旧池 (1, 6) 的文件，可写挂载做成、生效根 (1, 6)。
- 「count=64 s5__whole__new__oldolder_t5_at_i1t5__red__ro_i1t5__rw_err_file_version_without_any_journal_record__mixed」：同样是新池系统配置，读回旧池 (1, 5) 那一版。

最小历史（草稿目录 `runs/run-o512-u5-fsame-new-then-sysconf.log` 的例行，原样）：「ordinal=34 key=s4__torn__new__oldlast__red__ro_i1t6__rw_ok_i1t6__mixed withheld=[7,9,10,11,12,13,14,15,16,17,18,19,20] torn_images_landed=[21:system_configuration_slot@d0o0,23:system_configuration_slot@d1o0]」——旧池上用同一个 fsid 重做 mkfs，两单元与三条根落了、两块盘的系统配置槽 1 新写落了、槽 0 撕裂，根环槽 1..7 与 journal 环还没清：择系统配置得新池那份，择根按 (txg, 实例) 取最大择到旧池 (1, 6)，可写挂载照常做成。

每一步许可它的那一句：
- fsid 由调用方给：「pub filesystem_identifier: [u8; 16],」（`crates/singlefs-core/src/make_filesystem.rs:72`）。
- 同 fsid 的旧根照样自证得过、择得中：「拿同一个 fsid 在用过的盘上重做 mkfs，旧池留在槽 1..7 里的根照样自证得过、解得开，」（`crates/singlefs-core/src/make_filesystem.rs:533`）。
- 根环基址与 journal 环起点是第一版常量、读者不等就拒，新旧两池的根环与 journal 环因此落在同一段字节上：D22 已定项 16 第 1 句「读者解槽时读这个字段与 journal 环起点字段」（`.claude/kb/decisions/22-单元原子性怎么合成.md:337`），同一句接着写不等一律整池拒。

为什么甲在 fsid 相同时做不到（推的，由上面量过的那一格撑）：新池提交之后可挂，就要求那一刻根环里不剩旧池的根（C484 的理由），而旧池在提交之前可挂又要求它的根还在；两样落在同一段字节上（上一条），只能一前一后。甲的两种排法各卡一边：
- 先清根环再提交：提交之前旧池的根已没了，旧池读回不了（今天的写序就是这样，调查员量到读回更早一版与恢复失败）。
- 先提交再清根环（new-then-sysconf、sysconf-first）：提交点到清零落盘之间是新池系统配置 + 旧池的根 = 混合（上表）。sysconf-first 在 fsid 相同时同一个机理，**推的**，没跑完（124 499 态，超前台时限，命令在第七节）。

卡在哪一类字节：根环（非重叠不可：区域公式的基址是格式常量；非覆盖不可：新池的第 0 代根只能写在三个区域的槽 0，`(txg div R) mod S` 给 0）。journal 环同理（起点常量，旧池同 fsid 的记录也会被新池扫进来，`crates/singlefs-core/src/recovery.rs:4351` 按 fsid 认记录）；单元区起点按环长现算，新旧环长相同时也重叠。

### fsid 不同：读回没打中，字面打中（量过）

o512-u5-fdiff 与 o512-u23-fdiff（旧池最后确认 (1, 24) 正住区域 0 槽 0），写序 new-then-sysconf：提交点之前 18 态，择到旧池系统配置、恢复读回的内容都与最后确认的一版逐字节相同，混合 0 态。但这条写序在提交点之前写了旧池读得到的字节：
- 单元区起点的两单元（`unit_write@d0o23068672` 等，写表的 kinds 行在草稿目录 `runs/count-o512-u5-fdiff.log`，没抄进模型目录）；为什么旧池没被打坏没查（推的：这一份历史里旧池最新一版不再引用那几槽，或者读法容得下）。
- 根环区域 0 槽 0：u23 那份上新根盖掉了旧池最后确认的 (1, 24) 那条根，恢复择到 (1, 23)、经 journal 补到 (1, 24) 的内容（原样「count=1 s2__whole__old_g26__oldlast_at_i1t23__green__ro_i1t24__rw_ok_i1t24__notmixed」）——读回靠 journal，环里少了一条回退候选。

所以甲的字面「不改动旧池读得到的任何字节」在这条写序上不成立；退一步读成「旧池读回的内容与最后确认的一版逐字节相同」，这两份历史上成立，但只在 fsid 不同时。

## 二、V2 乙撑不撑得住

### 全量枚举：没打中（一份样本，量过）

写序 invalidate，五份历史各一趟全量（`out/invalidate.txt`）：

| 历史 | 全量态数 | 作废屏障之前（s0） | 其中旧池读回最后确认的一版且 checker 绿 | 屏障之后到系统配置之前（s1–s4） | 系统配置那一段（s5） | 混合 |
|---|---|---|---|---|---|---|
| o512-u5-fdiff | 31 195 | 8 | 8 | 31 106，全部 NoValidSystemConfiguration | 80：新池 64、整池拒 16 | 0 |
| o512-u5-fsame | 31 195 | 8 | 8 | 31 106，同上 | 80：新池 64、整池拒 16 | 0 |
| o65536-u5-fsame | 31 150 | 8 | 8 | 31 106，同上 | 35：新池 31、整池拒 4 | 0 |
| o512-u6-fdiff | 31 195 | 8 | 8 | 31 106，同上 | 80：新池 64、整池拒 16 | 0 |
| o512-u23-fdiff | 31 195 | 8 | 8 | 31 106，同上 | 80：新池 64、整池拒 16 | 0 |

（s1–s4 那一列是总数减 s0、s5 与末态 1 个现算的；每份的 s1、s2、s3、s4 原样计数在 `out/invalidate.txt` 的 `name=count` 行里。）

s0 的 8 态里可写挂载：4 态做成 (1, 6)，4 态拒（`rw_err_system_configuration_unreadable_after_the_read_stage`，一块盘一份自证过的系统配置都没有时取号拒那一条）。s5 里择到新池的态有一部分可写挂载做成 (0, 0)、池级 checker 红（一块盘的系统配置槽撕裂或缺），只读挂载报 `ro_err_open`（空池没有文件可开）。

### 作废那一步之前：今天那 248 832 态里哪些落进这一段（推的映射 + 量过的结局）

乙的 s0 只有两次写（两块盘各一次 [0, 根环基址) 清零），每次三态，9 种组合减去两次都整写落了的那一种 = 8 态。今天写序第 0 段里「只有这两次清零落了（整写或撕裂）、别的 12 次写都没落」的组合也是 9 种，盘面与乙的那 9 种逐字节相同（推的：同一组写、同一个撕裂镜像算法 `crates/singlefs-checker-tier/src/crash.rs:2274`）。其中择到旧池系统配置的是除「两块盘都整写清了」之外的 8 态——就是乙 s0 那 8 态，结局量过：恢复读回 (1, 6)、checker 绿、混合 0。248 832 里其余 248 824 态都含根环、journal 环或单元区的写，乙的写序下它们只能出现在作废屏障之后，那时没有自证得过的系统配置、整池拒（上表 s1–s4）。

### 一盘作废一盘没作废（量过）

会择到没作废那块盘上的旧池系统配置：择系统配置时两槽都不可择的盘整块跳过（`crates/singlefs-core/src/recovery.rs:1442` 起那一段）。乙的写序下此刻旧池别的字节一个没动，恢复读回最后确认的一版。

另量到一格（不是打中，记一笔）：择系统配置取第一块可择盘那一份，不比各盘的世代号：「None => chosen = Some(best_on_device),」（`crates/singlefs-core/src/recovery.rs:1442`），后面的盘只核 fsid 与设备数。盘 0 的最新槽被清、盘 1 两槽完好时择到的是盘 0 的世代号 7、不是盘 1 的 8（`out/count-and-image.txt` 原样「h=o512-u5-fdiff generation=8 newest_slot=0 shape=d0_newest_only old_g7__oldlast__green__ro_i1t6__rw_ok_i1t6__notmixed__oldintact」）。这些盘面上读回不受影响（读回按根环择根，不按系统配置的世代号）。

### 模型外的撕裂形状（量过）

崩溃点重放的撕裂镜像只取一种：新旧不同那一截「前一半新、后一半旧」（「torn[first_old_byte..].copy_from_slice(&old[first_old_byte..]);」（`crates/singlefs-checker-tier/src/crash.rs:2274`））。[0, 根环基址) 清零在用过的盘上，这一截从槽 0 开头到槽 1 末尾，撕出来的总是「槽 0 清了、槽 1 还在」；真设备上 1 MiB 的写零按扇区任意次序落，反过来那一种（槽 0 还在、槽 1 清了）模型里没有。改在镜像上直接摆 12 种形状（两块盘各自清哪一槽、清最新还是较旧那一槽），三份历史（最新槽在槽 0 的 u5，在槽 1 的 u6，fsid 相同的 u5）：36 个盘面恢复都读回最后确认的一版、checker 全绿、混合 0（`out/count-and-image.txt` 的 `name=image` 行）。

### 整池拒之后用户看到什么、能不能直接再跑 mkfs（量过 / 推的）

看到的是 `NoValidSystemConfiguration`（恢复与可写挂载；只读挂载报 `Recovery` 包着它），与「两块盘的系统配置都坏了」是同一个错，分不出「mkfs 中途崩溃」。再跑 mkfs：mkfs 不读盘上旧内容（`make_filesystem` 第一步只判几何），可以直接再跑，推的、没跑（本腿没枚举「崩溃后再跑一次 mkfs」那条用户动作）。

## 三、V3 代价

| 项 | 今天 | 乙 | 甲（sysconf-first） | 甲（new-then-sysconf） | 量过 / 推的 |
|---|---|---|---|---|---|
| mkfs 屏障（按池计） | 2 道 | 3 道 | 3 道 | 3 道 | 推的（写序里数） |
| 段形 | `14+1+1+1+4` | `2+12+1+1+1+4` | `4+14+1+1+1` | `4+1+1+1+4+10` | 量过（`out/count-and-image.txt`） |
| 写数 | 21 | 21 | 21（[0, 根环基址) 那一步改成从槽 1 末尾起） | 21（同上；根环清零改成每区槽 1..S） | 量过 |
| 新盘上的闭式状态数 | 16402 | 4117 | 16402 | 1057 | 量过 |
| 用过的盘上全量（o512-u5-fdiff，C656 验收要枚举的量） | 280 019 | 31 195 | 124 499 | 7 874 | 量过 |
| fsid 相同时混合 | 今天没跑 | 0（量过） | 推的：有 | 6653 / 7874（量过） | — |

要改的处数：
- crates 里钉 mkfs 段形的用例 2 处：「"14+1+1+1+4",」（`crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:280`），「(25, "14+1+1+1+4".to_string(), 16402)」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:364`）。另有 E142 装置（`scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs` 里命中 `14+1+1+1+4`、`16402` 或 `[14, 1, 1, 1, 4]` 的 8 行）、`scripts/crash/check-segment-registry-product-sample.txt` 一行、layout 八「mkfs 种根」那一行（`.claude/kb/layout/01-first-txn.md:410`）。checker 档用例：崩溃点重放从 mkfs 之后的池起枚举，没有钉 mkfs 段形的，`grep -rnF '14+1+1+1+4' crates` 只命中上面两处。
- 乙要改 D22 已定项 8 第 3 条那一句「清零几段之间不加屏障」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）：改成 [0, 根环基址) 先清、过一道屏障，再清根环与 journal 环。甲要改的是同一条的次序，外加 fsid 相同那一支的处置（见第六节）。
- 乙在 mkfs 文件头注释的写序（第 3–5 行）与第 540–544 行「不为一个没有条款的语义加屏障」那一段要跟着改（推的）。

## 四、V4 没量的几何会不会翻面

| 几何 | 甲 | 乙 | 今天写序 |
|---|---|---|---|
| fsid 相同而两块盘都重用 | **翻面**：new-then-sysconf 混合 0 → 6653（量过：o512-u5-fdiff 对 o512-u5-fsame） | 不翻（量过：混合 0，s0 8 态完好） | 没跑（命令在第七节） |
| 旧池 io_min 65536、新池 512、两块盘都重用（fsid 相同） | 没跑 | 不翻（量过：o65536-u5-fsame） | 没跑 |
| u 取别的值 | fsid 不同时 u = 23（最后确认的根正住槽 0）读回靠 journal 补（量过） | 不翻（量过：u = 6、u = 23） | 没跑 |

注意旧 io_min 65536 这一格的一个推的机理：新池按自己的槽距清根环区域（S × 4096 = 32 KiB），旧池在 64 KiB 倍数上的根槽 1..7 不在新池清的那一段里、也不在新池读的槽位上（新池读区域起点 + j × 4096）；所以这一格不让新池择到旧根，只是清不干净。

## 五、第三节的数能不能在入库代码上核

- 280 019 这个总数：在未打补丁的公开函数上对上（量过）。`crash_state_count_with_torn_in_place_overwrites` 按每段 `EveryProperSubset` 数，o512-u5-fdiff（即调查员的 `W:x=512:y=512:u=5:reuse=both:fsid=diff`，盘大小不同）得「states_full_on_this_base=280019」，新盘上得 16402（`out/count-and-image.txt` 第一行）。调查员那一份是把 `visit_crash_state_range_of_the_read_back_flow_with_slices` 改成 pub 之后数的，两条路总数相同。
- 分类的数（248 832 / 173 568 / 6 144 / 9 216 / 59 904 / 246 528 / 53 248 / 1 024）：没核完。全量一趟要十几分钟，超过前台 240 秒；观察者的判器键拒算（库照判、不入库），断了不能续跑。跑了 225 秒的前一段，没出计数，只出了例行：同一类结局在未打补丁的代码上都出现了（量过，`out/today-partial-examples.txt`），例如「ordinal=16962 key=s0__whole__old_g8__oldolder_t5_at_i1t5__red__ro_i1t5__rw_skipped__notmixed」——不带撕裂镜像、择旧池系统配置、恢复读回 (1, 5)。这一行的序号与调查员的「序号 11780」不是同一个编号体系能不能对上，没核。
- 调查员「完好」的定义里有「按旧池参数可写挂载做成 ∧ 挂载之后 checker 绿」两项，可写挂载要真写盘，入库的崩溃枚举用例没有这一路观察者；在入库装置上核这几格要另写交回镜像的观察者（`CrashPointReplayObserverHandingBackAnImage`），推的。

## 六、打中之后的四句

V1（fsid 相同，提交点之后混合）：
1. **分不分辨臂**：分辨。同一份历史（o512-u5-fsame）上乙混合 0、甲（new-then-sysconf）混合 6653，量过。用户动作（崩溃后只读挂载、可写挂载）每态都放开跑了，不是装置写死的；旧池用法 u 只取了 5（fsid 相同那一支）。
2. **被判的系统当时看不看得到判别它的东西**：看不到。择系统配置那一刻，新池那份与「旧池用同一个 fsid」时旧池的根都自证得过、fsid 相同；根记录里只有 fsid，没有能把根绑到某一次 mkfs 的字段（推的，按 `RootRecord` 的字段与 `choose_root` 只核自证与 fsid）。这正是「判别子观测不到」那一形：甲要在 fsid 相同时成立，判据或格式要先改。
3. **满足的是判据字面的哪一个分句**：正文 V1「提交点之后与乙的后半段同问」，乙的后半段那一句是「不许挂上旧池或新旧混合」；打中的是「新旧混合」那一分句（新池系统配置 + 旧池根与单元），其中 160 态同时是「读回更早一版」。
4. **跑前条款给的每个改法在打中的那几格上还中不中**：正文没给改法。下表是本腿自己提的。

| 改法 | 修哪一格 | fsid 相同的混合格上 | 标注 |
|---|---|---|---|
| 乙（先作废再写） | 甲的混合格不存在（乙在提交前已清根环） | 混合 0 | 量过（o512-u5-fsame invalidate） |
| 甲 + mkfs 拒绝与盘上旧池同 fsid（要先读盘） | 混合格 | 推的：不中；但 mkfs 要读旧池，旧池坏了读不出时怎么判没有条款 | 推的，被攻过零轮 |
| 甲 + 根记录与系统配置各带一个每次 mkfs 现取的随机数、择根只认相同的 | 混合格 | 推的：不中；改格式（根记录字段表、D22 已定项 7），且破「同参数两次 mkfs 逐字节相同」那条验收 | 推的，只在我的模型上想过、被攻过零轮 |
| 甲 + 提交点之后先清根环槽 1..S 再写系统配置的第二份（两段提交） | 混合格 | 推的：系统配置写了一半时仍是新池系统配置 + 旧根，照样中 | 推的 |

以上改法都只在我的模型上量过或推过、被攻过零轮。

## 七、要主 agent 跑的命令

前台 240 秒跑不完、观察者键拒算不能续跑的三件。工作目录 = 一个放好了副本的草稿目录（复跑脚本会建），环境变量前缀 `CARGO=$(rustup which cargo) SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=<草稿目录>/lib-<件名>`，内存上限 3G，线程 31：

```
bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 3G <草稿目录>/tree/target/release/c656-attack-probe run o512-u5-fdiff today
bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 3G <草稿目录>/tree/target/release/c656-attack-probe run o512-u5-fsame today
bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 3G <草稿目录>/tree/target/release/c656-attack-probe run o512-u5-fsame sysconf-first
```

第一件前加 `PROBE_WRITABLE=1` 才能与调查员的可写挂载那几格比（更慢）。判结果看 `name=done` 那一行的 states（第一件应为 280 019，量过；第二件、第三件推的分别与 o512-u5-fdiff 上量过的 280 019、124 499 同数）与 `name=sum` 行：第一件对调查员第三节的分类数（`seg=s0 choice=old_g8` 加 `old_g7` 应为 248 832，`rec=oldlast` 应为 173 568 一类）；第二件看今天写序在 fsid 相同时有没有混合（`seg=… mixed`）；第三件看 sysconf-first 在 fsid 相同时的混合数（推的：有）。三件状态合计约 68.5 万，各自不超过 10⁶。

## 八、没打中的形状

- 乙：五份历史（fsid 同 / 不同，旧 io_min 512 / 65536，u = 5 / 6 / 23）全量，作废屏障之前 40 态、之后 155 890 态，混合 0、提交前旧池不完好 0；12 种撕裂形状 × 3 份历史的镜像 36 个，全完好。没试：旧池正处在「根已落、系统配置没轮换」的半截发布上；旧池经过第二次可写挂载（实例表重写、单元区起点那几槽被复用）；三块以上盘；真设备上写零被实现成 discard 的读回。
- 甲 fsid 不同：new-then-sysconf 两份历史（u = 5、u = 23）提交点之前 36 态读回都完好；没试旧池单元区起点几槽仍被最新一版引用的历史（推的：那时两单元的写会盖掉旧池在用的单元）。

## 九、这条腿自己的限度

- 一份样本：V2「没打中」只算一次，按规则要再派一条 s2 才能记「没打中」。
- 旧池用法只在同一个进程里做（mkfs 之后一直挂着）；崩溃后再可写挂载的历史、正常卸载的历史没造。
- 用户动作只放开了「崩溃后只读挂载 / 按择到那一池的参数可写挂载」两样；「拿新池参数去可写挂载一个择到旧池的盘面」「崩溃后再跑一次 mkfs」没扫。
- 今天写序的分类数没在本副本上跑完（第五节、第七节）；sysconf-first 在 fsid 相同时的混合是推的。
- 副本上的数，不是入库装置上的数；主工作区 `crates/` 带别的会话暂存未提交的改动（`git diff --cached --stat -- crates` 末行「228 files changed, 137089 insertions(+), 5460 deletions(-)」），副本照工作区现状拷。S3（按 fsid 筛）还没落进主工作区，择系统配置按今天不筛的读法。

## 十、没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没跑重型测试、没调 checker 档测试目标，原型只 `cargo build` 后直接执行。
- 原型状态合计：全量枚举 8 趟（31 195 × 4 + 31 150 + 7 874 × 3 = 179 552 态）加两趟超时中断的前一段（today 约 225 秒、sysconf-first 约 110 秒，计数未出），镜像 36 个；都远在 10⁷ 之内。
- 仓副本 `~/.local/share/singlefs/work/m3-close-c656-r1-opus/tree/`（连同 `target/`）交回前删掉；草稿目录里的 `runs/`、补丁与 `rerun.sh` 留着。


## cite-check 末行

`python3 scripts/agents/cite-check.py research/prompts/m3-close-c656-r1-opus-output.md` 末行（写进这一节之前那一次）：「✓ 核了 8 处引文、4 处裸行号，对上 8 处，没判 4 处（其中裸行号没有引文可比 4 处；逐处列在上面）」
