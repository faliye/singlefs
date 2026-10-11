# 删除线 checker 九格与两问：设计轮第二轮攻方腿（Opus）报告（2026-10-05）

攻击面：正文第三节 T2、T3、T10、T11、T12（重点），T4、T5、T6、T7、T9、T14 能造的也造了。数都在冻结副本的拷贝上量（`rerun.sh prepare` 现建这份拷贝；我跑时用的拷贝与编译目录交回前已删，walk.rs 只加了一个只读的 `attack_stash` 模块把删除态扫描那份版本清单上每一版的事实留给原型，判定一行没改），不是入库装置上的数。候选判法是原型里写的，不是实现。

## 复跑

```
FROZEN_TREE=/tmp/claude-1000/m3-delete-checker-r2/tree SINGLEFS_REPO=/home/fy5090/code/singlefs \
  bash research/prompts/m3-delete-checker-r2-opus-model/rerun.sh <空目录> <步骤>
```
步骤依次：prepare build sweep-f1 sweep-f2 sweep-f3 sweep-fc-a sweep-fc-b sweep-r-a sweep-r-b press s4a knob t7-a t7-y xleaf tomb crash-count show analyze summary（`all` 串起全部）。编译经 `run-with-memory-cap.sh` 6G，跑每一步 4G，`capped.sh 6`，原型线程 6。在空目录 `/tmp/claude-1000/m3-delete-checker-r2-opus/rr3/` 从 prepare 跑到 summary 一遍（单步最长 press 83.4 秒、sweep-fc-a 78.9 秒；整轮挂钟约 9 分钟，不到 40 分钟，没缩取样）；另一个空目录 `rr2/` 从头跑的那一遍与 rr3 在共有的 1359 行摘要上逐行相同（两个工作目录交回前已删，输出拷进了模型目录 `out/`）。summary 步末行原样：`SUMMARY_MATCHES lines=1471`。

模型目录每个文件的 sha256（`research/prompts/m3-delete-checker-r2-opus-model/`）：
```
a245351d828c7dce834985968e3b5ece8a703da8e8b9c05f144c0708284af3a2  aggregate.py
bc90421b1550c77e86f9b0ffe27bb785ba4303cff7d6dcd669ba880ede748339  expected-summary.txt
eea77cb27ffb54bb14584ae5cba334c26e2029b76a951754378dc2aaefbd60f1  out/analysis-press.out
fe478bbbf308d5859774100d3fbc0a5fae422619460eef86d43e243f56286e46  out/analysis-sweeps.out
9fdd65afef5f8f892b92b93bbeeb4d091dbe958a549d50add3aeb5b8deb2a2fa  out/bad-knob.out
9de2cac41d64acedc5e09f0dba1f160b4ba37ef84dcaed45aeac5b15c52de7fc  out/bad-press.out
f23d3d7d2201caf2504320ce58b1ffa1de6a2f6f93143ef7e0efa1a50c08f58a  out/bad-s4a.out
dc509d5c5b122c78d1bd5561bc8573f6d448887ab71a15670c9f29358371f511  out/bad-t7-a.out
5d1eaf585a1bb796a535f7547f7125c9046c20058fbe5f30422557df80cadf0f  out/bad-t7-y.out
678c655cb36bec94b805760f8fee4157a66a6d2ed5d166d7baa08e793433014e  out/bad-tomb.out
70bc901e1e678cf3337c8a9934929737e74c6bfc91ab80af74fbc5654582ce5c  out/bad-xleaf.out
bf5189fd0e2dc213a4adfbbe4b48aec3efbe0a0a70661a7671a19beba4a7c723  out/build.log
575d70652b34792c44490ea360b5e6f25c1bcd214d9a59ddeeb9cd5e58eb53b7  out/crash-count-c1.out
ccfd566604c6b54dd5fc3b7a5a75d278864e52ce8cb709ae355dfede224eafcb  out/crash-count-c2.out
afd61568fb790a3aefa229fb988e85ec1ee3212b15ad6c747c88b9b659914581  out/show-C2_C1_X_C1.out
0fcca67df116118d04690cf8c126ba8aca7ee9bed99ba4fdb0378b12e0d16061  out/show-C2_U1_R2_U2_U3_C1.out
c298b36e8103ea628bb1db27e6b2abb03ecf57928cb31b2cd0b45fc02c14d33f  out/show-C31800.out
9b12284f72a5bd72aa1835743c62616ccb6387ebafe347e23a6123bdc801e194  out/show-U1_R2_R1.out
314bd398c61539264eb78a9fc22e09659cbc2ff2fa4eada26c6ea43d52cc3563  out/show-U1_R3.out
67f44a7ef64b9d6614460a18225b1ea05445435ba3e5d6c6a3f608e1735470ed  out/show-U1_Y6_R2.out
4a1339f15c21f9b258e3fadc7606e4a491fbda7aecf68816d7b733d771277b66  out/sweep-f1.out
a81f90351a7699fa9589ce6c49102d7f6187b75e4b1fcad5518fcfa07ac2139b  out/sweep-f2.out
15a4f8b6077a63bdcd0eff0cebf42598a8fb349eb6d591e905e8fcdb61ba4714  out/sweep-f3.out
e6d3ed0e9b57d8fb0a502ab4d492244df79e09076137c7c16cc8f9d8f07b8176  out/sweep-fc-a.out
a6bf6b16b15a1bf78cb91276d0e3e7cef555956c655d3bc225b476ab2611b591  out/sweep-fc-b.out
a036f60c99c2d0310df8c390b51d0c5b3eafafe6ec2ab7b22c7d295f3eee2daa  out/sweep-r-a.out
3cbbae744d1217623352c05092701b3e0cdf07f526d754b5e697e1cecfbea8f6  out/sweep-r-b.out
a5c27bfceb8cf05d66e18ee0901025f51d69d519ce9e6ff578333af1643c0e27  press_aggregate.py
187724703c84c626eef0797176dd5910da812db98625bb4c51e35dfc4478232b  proto/Cargo.toml
07728eabb0dc64fa81a48f07fd7de8d98ad6cc8e5742877e535b3de9649a6b91  proto/src/bad.rs
d2be59cd0544c21edf3bd62f8498995f20c7ed9cd637839ed0072372d6369b1e  proto/src/cand.rs
cce8795f36717aa95c015e70de96f22366fa64f3ff6c5a19d021f9e35d991be2  proto/src/crashes.rs
4b6298b8f138da36fe26649d6450caf7a51231c64013a794e14d08bdeae5ccc2  proto/src/driver.rs
3940f8b2b909f06a81a25a5229140a5551c13fa6a9c14cf5644767cc01e3184f  proto/src/main.rs
179f7aaca9a206592acb1b742af417996a708102de6894450e03bcea995c921b  proto/src/plans.rs
ec7ee949b670014a794888fffcc4c4787f7e7bc7c8f466490d81043421d80750  rerun.sh
498d2bb7dd6d0f2ae103a7127449e4d5d3ca050d747b9d2bcb3f8f8c0cb09c49  summarize.py
651800a7a65f703df5747ee0f7a93b863a72232337b4175334617b3bd35630cd  walk-attack-stash.patch
a1fd8ccd01f4e873aeb27284cde906b628a04c661bd6e431b40e62e797bc6c6a  workspace.patch
```

## 原型怎么判（读产物之前要知道的）

- 历史：前缀固定（mkfs → 取号 → 暖机 → 新池新建文件 3000 字节、实例 1 → 正常可写挂载开会话、实例 2），之后每一步是用户决定的动作：`C<n>` 经会话建 n 个空 inode，`U<k>` 经会话 unlink 号 k（走空 + 意图 + 全部批），`W<n>` 顺序写第一个文件 n 个单元，`R<k>` 管理员回退到根环里从新到旧第 k 条（取模），`X` 崩溃恢复抛弃最新那条根（截掉流尾最后一次根槽 FUA 之后的系统配置轮换写，下一次可写挂载时那条根的根槽与它那次发布点名的单元读回全 0，与历史执行器同一造法），`Y<k>[f]` 崩在上一步那段写的第 k 个屏障 / FUA 段界（f：再加第 k 段的第一个写）之后重挂载，`M` 正常重挂载，`P<p>x<n>` 只供测试的水位开关压回 p 再建 n 个（坏）。盘是内存稀疏盘。
- 每一步之后的整池镜像跑一次池级 checker，再从 `attack_stash` 取 checker 删除态扫描那份版本清单（回退候选集里每条根加由记录施加出来的几版）上每一版的 txg、队列、inode 记录（叶序原样，含出生代）、墓碑容器（叶序、已发布与否、记录五段）、谱系记录、记账水位，各候选读法在这份事实上判。候选名与正文格的对应：`T2a` 按号（T2-a）、`T2c-A` 按号且只取谱系上的（T2-c，谱系按 T3-a）、`sel-x-map` 所选根墓碑对 checker 跨版本号→出生代表（T2 的第二种范围）；`I918/A`、`I919/A`、`I920/A` 是 T3-a，`/B` 是 T3-b，`/A2` 是攻方补的「每版自己的 txg 起算、谱系记录取所选根那一版的谱系树」（零轮），`+pub` 是 T4「滤」；`C479yi` 是乙，`C479bing/lineage` 是丙按谱系配父，`/txg-1` 是丙一律 txg − 1；`I96/code` 是今天的读法（跨版本最大 key 对最新根水位），`I96/newest` 只取最新根，`I96/each` 每版自己的水位对自己的最大 key（攻方补的，零轮）；`S4a-jia` 类型 2 容器内记录号严格递增（甲 / 乙同一判法），`S4a-bing` 不限类型。

## 各格判定一览

合法状态 11 527 个（五个家族 1 684 段历史每一步之后的整池镜像，含崩在屏障段界的崩溃镜像与它重挂载之后的镜像）；坏状态另列。

| 格 | 候选 | 判定 | 依据（详见各节） |
|---|---|---|---|
| T3 | b 一律用所选根 | **打中：合法镜像误红**。I-9.18 读法在 1 656 个合法状态上红，I-9.19 读法在 46 个上红（两种形：unlink 后回退到意图之前；回退到意图之前再回退回删完那一版） | 第一节 |
| T3 | a 每版自己的 | 没打中：11 527 个合法状态 I-9.18 / I-9.19 / I-9.20 三条 0 红 | 第一节 |
| T3 | A2（攻方补，零轮）每版 txg 起算、谱系取所选根 | 没打中：与 a 同数 | 第一节 |
| T4 | 滤 / 不滤 | 没打中：合法状态上两种逐格同数（0 红） | 第六节 |
| T2 | a 按号 / c 按号且只取谱系上的；每版 / 所选根对跨版本表 | 合法状态五种读法都 0 红；**c 有漏判**：「unlink → 回退到 unlink 之前 → 压水位重发」那一族，墓碑在谱系外，c 与乙丙（窗口过去之后）全不红、只有 a 红 | 第二节 |
| T10 | 甲 / 乙 / 丙 / 丁 | 乙、丙（按谱系配父）合法 0 红；丙按 txg − 1 配父在 1 579 个合法状态上红（确认调查报告「要按谱系配父」）；压回之后再发 30 次，各候选与今天的 checker 全不红的 93 行（多是活记录被原地换的那几族） | 第二节 |
| T12 | 改 kb 成任一版 / 改代码只取最新根 | 合法 0 红（三种取法）；两种在坏状态上互不包含：今天的跨版本读法比只取最新根多抓 22 行（压回后不再发）；攻方补的「每版各比自己的」（零轮）在压回后再发 30 次时抓 81 行，今天的读法只抓 3 行 | 第三节 |
| T11 | 甲 / 乙 / 丙 / X2 | 甲 / 乙判法合法 0 红；坏镜像 S4a、两种互换只红它自己（今天 checker 0 红）；丙不限类型在 487 个合法状态上红（确认调查报告「要限定类型 2」）；跨叶同号今天 I-9.4、I-9.12 两条一起红；inode 树今天长不过一层，跨子树同号造不出 | 第四节 |
| T9 | 并 I-9.18 | **打中：调查报告第二、五、六节三份坏镜像做不到只红自己**——意图那一版还在候选集里、与删完那一版共用同一片墓碑容器，改那片容器连带 I-9.18 红；根环转过之后再改，I-9.19、I-9.20 那两份只红自己，I-9.10 那份还连带乙（带墓碑时） | 第五节 |
| T14 | 每条记录管谁 | 区间那一句管全部墓碑：合法 0 红；不重号那一句按全部墓碑判在 81 个合法状态上红（就是条款原文写过的「不按谱系过滤」那一形）——调查报告第 174 行「两种读法在合法镜像上同绿」只对区间那一句成立 | 第六节 |
| T5 / T6 | 旋钮信号 / 判哪几版 | 确认调查报告：旋钮压小的合法镜像上 I-9.21 产品读法 42 个状态红，每版与只所选根同数 | 第六节 |
| T7 | a / b | 今天身份不符时红的是 I-4.8、**I-7.2**、I-7.4 三条（调查报告只列 I-4.8、I-7.4）；调查报告推的「I-3.1 连带」在两段历史上都没出现 | 第六节 |
| 新 | 墓碑死亡 txg 改成早于对象出生代 | **打中：漏判**。I-9.18 / I-9.19 / I-9.20 各读法（a、A2、b、全部）全成立；九格里没有一条管死亡 txg 的取值。攻方补的「对象出生代 ≤ 死亡 txg ≤ 这一版的 txg」（零轮）合法 0 红、这几份红 | 第五节 |
| 新 | C616 那一步 | 推的、没造：崩溃恢复后新实例第一条根的 txg − 1 是被抛弃的 txg，按条款算它在链上；合法镜像上没有死于那几个 txg 的墓碑，不误红 | 第一节 |
| GPU | 各候选跟不跟得上 | 推的（读代码，没跑 GPU）：每版各判的读法套得进扫描内核「一版一版重填」的做法；I-9.20 不重号要一张每版墓碑表、乙 / 丙 / T12 跨版本读法要跨版本表，草稿区 298 736 字、inode 记录表 1 024 行，产品镜像墓碑可到 11 万条，超了只能判不了回落 CPU | 第七节 |

崩溃状态没有按层 0 的枚举域枚举：每段删除历史每一步的闭式状态数都在 10⁶ 以上（一次 unlink 83 886 093），按派发规则不跑，见「这条腿自己的限度」。

## 第一节　T3「在谱系上」从哪条根起算（重点）

### 打中一：T3-b 在「unlink → 回退到意图之前」上误红 I-9.18

历史 `U1,R3`（批大小产品值），每一步与许可它的那一句：

1. 前缀：mkfs、新池新建文件（实例 1），正常可写挂载（实例 2，txg 4、5）。
2. `U1`：经会话 unlink 号 1，D8 已定项 16 第 6 条许可「会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的 `deleted_inodes` 队列走空」（`.claude/kb/decisions/08-核心索引结构.md:469`）；意图那次 txg 6（队列 {1}、墓碑 (1, 3, 死于 6)），最后一批 txg 7。
3. `R3`：管理员回退到 txg 4（候选集里，入口接了），谱系树追加 (8, 4)，D18 已定项 24 第 5 条「管理员回退那次发布追加一条」（`.claude/kb/decisions/18-块里携带什么信息.md:574`）。
4. checker 的删除态扫描照「判走读到的任一版（候选集里每条根，含由记录施加出来的那几版），不只判最新根」（`.claude/kb/invariants.md:292`）判每一版，意图那一版 txg 6 还在里面：「  V 根（实例 2、txg 6） txg=6 newest=false rec=false q=Some([1]) wm=Some(2) recs=[(1, 0, 3)] tombs=[(1, 3, 6, true)] lineage=[] gap=None」（`research/prompts/m3-delete-checker-r2-opus-model/out/show-U1_R3.out:24`）；最新根 txg 8 的谱系是「lineage=[(8, 4)]」（`research/prompts/m3-delete-checker-r2-opus-model/out/show-U1_R3.out:30`）。
5. 按 I-9.18 的定义「从挂载根的 txg 起往回走，有谱系记录 (T_rb, T_old) 的那个 txg（T_rb）的父是 T_old，其余 txg 的父是 txg − 1」（`.claude/kb/invariants.md:305`），T3-b 从 txg 8 走：8 → 4 → 3 …，6 不在链上，txg 6 那一版队列里的 1 找不到谱系上的墓碑。原型原样：「FIRSTRED show	step	I918/B	show #2:R3 [rolled-back-to txg4/inst2 now txg8] :: 根（实例 2、txg 6）：队列号 1（出生代 3）没有谱系上的整对象墓碑」（`research/prompts/m3-delete-checker-r2-opus-model/out/show-U1_R3.out:69`）；同一张镜像今天的 checker 一条不红：「STEPLINE show #2:R3 [rolled-back-to txg4/inst2 now txg8] versions=9 checker_red=[] cand_red=["I918/B", "I918/B+pub", "I919/nofilter", "C479bing/txg-1"]」（`research/prompts/m3-delete-checker-r2-opus-model/out/show-U1_R3.out:74`）。T3-a 从 txg 6 自己走（它自己的谱系树里还没有 (8, 4)），6 在链上，成立。

这是调查报告推的那一形（「按 b 它不在谱系上，I-9.18 在一个合法镜像上红」，`/tmp/claude-1000/m3-delete-checker-r2/prep-report.md:128`），这里坐实了。

### 打中二：T3-b 在「回退到意图之前 → 再回退回删完那一版」上误红 I-9.19

历史 `U1,R2,R1`：`U1` 同上（txg 6 意图、7 最后一批）；`R2` 回退到 txg 5（号 1 带回来、nlink 1），谱系 (8, 5)；`R1` 回退到 txg 7（删完那一版），谱系 (9, 7)。墓碑树随现行那一版，txg 8 那一版里有墓碑 (1, 3, 死于 6) 而记录 1 活着：「  V 根（实例 2、txg 8） txg=8 newest=false rec=false q=Some([]) wm=Some(2) recs=[(1, 1, 3)] tombs=[(1, 3, 6, true)] lineage=[(8, 5)] gap=None」（`research/prompts/m3-delete-checker-r2-opus-model/out/show-U1_R2_R1.out:42`）。T3-b 从最新根 txg 9 走：9 → 7 → 6 → 5，6 在链上，于是 txg 8 那一版「谱系上的墓碑指活对象」：「FIRSTRED show	step	I919/B	show #3:R1 [rolled-back-to txg7/inst2 now txg9] :: 根（实例 2、txg 8）：谱系上的墓碑 (1, 3, 死于 6) 指着 nlink 1 的记录」（`research/prompts/m3-delete-checker-r2-opus-model/out/show-U1_R2_R1.out:83`）；今天的 checker：「STEPLINE show #3:R1 [rolled-back-to txg7/inst2 now txg9] versions=10 checker_red=[] cand_red=["I919/B", "I919/nofilter", "C479bing/txg-1"]」（`research/prompts/m3-delete-checker-r2-opus-model/out/show-U1_R2_R1.out:88`）。T3-a 从 txg 8 自己的谱系 (8, 5) 走：8 → 5，6 不在链上，成立。这一形调查报告没提；它说明 T3-b 的病根不只是「被抛弃分支上的版本还在候选集里」，还有「后一次回退又把被抛弃的那一段接回链上」。

合计（五个家族全部合法状态）：「AGG I918/B holds=4681 red=1656 na=5190」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:9`）、「AGG I919/B holds=4833 red=46 na=6648」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:15`）；同一批状态上「AGG I918/A holds=6337 red=0 na=5190」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:6`）、「AGG I919/A holds=6337 red=0 na=5190」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:12`）、「AGG I920/A holds=6337 red=0 na=5190」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:17`）。I-9.20 的 b 读法 0 红（`analysis-sweeps.out` 第 19 行），因为从最新根走，较早那几版的两条同号墓碑只有一条在链上。

### 打中之后的四句（两处同答）

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分辨：同一批状态上 a、A2 都 0 红，只有 b 红 |
| 被判的系统看不看得到判别它的东西 | 看得到：每一版自己的 txg 与它树表里的谱系树都在盘上，checker 删除态扫描本来就逐版读那一版的树表 |
| 满足判据字面的哪一个分句 | 正文第五节第一条「一个候选在一个从入口走得到的合法状态上报不成立：这个候选出局（误红）」：两段历史只经会话 unlink 与回退入口、无故障，今天的 checker 全绿 |
| 跑前条款给的改法在打中的格上还中不中 | 正文给的另一个候选 a 在这两格上成立（上面两行原样）；A2（零轮）也成立 |

### T3-a 没打中的形状与取样

f1：前缀 4 种（只有第一个文件；再建 2 个；第一个文件写 3 个单元、批大小 1；写 3 个单元再建 1 个）× unlink 对象 1–3 种 × 回退到第 1–8 新的根 × 11 种尾巴（再删同一个、删另一个、建一个、崩溃恢复、重挂载、崩溃恢复后再删、再回退到第 1 / 第 3 新、建一个→崩溃恢复→再建、再删→回退到第 2 新），616 段；f2：崩溃恢复落到旧根之后重发号（14 种 × 3 种前缀），42 段；f3：三个号 6 种删除次序 × 回退 / 崩溃恢复插在 4 个位置，102 段；fc：8 种前缀 × 崩在上一步那段写的每个屏障 / FUA 段界（与段里第一个写之后）× 6 种尾巴，624 段；r：300 段随机（建、删任意号、回退到第 1–8 新、崩溃恢复、崩在段界、重挂载）。T3-a、A2 在这 11 527 个状态上 I-9.18 / I-9.19 / I-9.20 全 0 红。

A2（每版自己的 txg 起算、谱系记录取所选根那一版的）与 a 在全部状态上逐格同数。它依赖「随现行那一版、只增不删」（`.claude/kb/decisions/18-块里携带什么信息.md:574`）让所选根的谱系树包住候选集里每一版自己的谱系记录——推的；好处是 GPU 只读一张谱系表（第七节）。

### C616 那一步（推的，没造坏镜像）

崩溃恢复抛弃 txg 7 之后，新实例的根从 txg 8 起：「STEP #3 X -> dropped_rotation_writes=2 mounted inst3 chosen=txg6 effective=txg6 continuation=0」（`research/prompts/m3-delete-checker-r2-opus-model/out/show-C2_C1_X_C1.out:29`），「  V 根（实例 3、txg 8） txg=8 newest=false rec=false q=Some([]) wm=Some(4) recs=[(1, 1, 3), (2, 1, 6), (3, 1, 6)] tombs=[] lineage=[] gap=None」（`research/prompts/m3-delete-checker-r2-opus-model/out/show-C2_C1_X_C1.out:40`）。按「其余 txg 的父是 txg − 1」，txg 8 的父算成被抛弃的 txg 7，7 在链上。合法镜像上现行墓碑树取自所选根，没有死于被抛弃那几个 txg 的墓碑，所以 a、A2、b 都不因此误红（上面 0 红的数里包含这些历史）；反过来，墓碑死亡 txg 被改成被抛弃的那个 txg 时三种读法都当它在谱系上、判不出——C616「其余 txg 的父是 txg − 1。后一步是 txg 分配规则的推论」（`.claude/kb/checks-owed.md:477`）那一笔欠账在这里多一个后果。没造这份坏镜像。

## 第二节　T2（I-9.10 墓碑那一个等号）与 T10（C479 立什么）（重点）

### 合法镜像：各读法都没误红

同上 11 527 个合法状态：「AGG T2a/each holds=6337 red=0 na=5190」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:30`）、「AGG T2c-A/each holds=6337 red=0 na=5190」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:32`）、「AGG T2a/sel-x-map holds=6337 red=0 na=5190」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:31`）；乙两种、丙按谱系配父 0 红：「AGG C479yi/rec holds=11527 red=0 na=0」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:4`）、「AGG C479yi/rec+tombA holds=11527 red=0 na=0」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:5`）、「AGG C479bing/lineage holds=11527 red=0 na=0」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:2`）。丙一律按 txg − 1 配父：「AGG C479bing/txg-1 holds=9948 red=1579 na=0」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:3`），第一形就是 `U1,R3` 的回退那一版（上一节 `show-U1_R3.out:74` 的 cand_red 里有它），坐实调查报告「要按谱系配父：回退那一版的父是 R_old，不是 txg − 1 那一版」（`/tmp/claude-1000/m3-delete-checker-r2/prep-report.md:262`）。

合法的「崩溃恢复落到旧根之后重发」在取样里真的走到了：`C2,C1,X,C1` 里号 4 先在实例 2 的 txg 7 发（出生代 7）：「  V 根（实例 2、txg 7） txg=7 newest=true rec=false q=Some([]) wm=Some(5) recs=[(1, 1, 3), (2, 1, 6), (3, 1, 6), (4, 1, 7)] tombs=[] lineage=[] gap=None」（`research/prompts/m3-delete-checker-r2-opus-model/out/show-C2_C1_X_C1.out:26`），崩溃恢复抛弃它之后实例 3 重发号 4（出生代 11）：「  V 根（实例 3、txg 11） txg=11 newest=true rec=false q=Some([]) wm=Some(5) recs=[(1, 1, 3), (2, 1, 6), (3, 1, 6), (4, 1, 11)] tombs=[] lineage=[] gap=None」（`research/prompts/m3-delete-checker-r2-opus-model/out/show-C2_C1_X_C1.out:53`）；txg 7 那条根按实例表判抛弃、不在候选集里，乙与 a 都成立：「STEPLINE show #4:C1 [ok versions=1 now txg11] versions=11 checker_red=[] cand_red=[]」（`research/prompts/m3-delete-checker-r2-opus-model/out/show-C2_C1_X_C1.out:95`）。

### 坏状态：水位开关压到哪放开扫

前缀 13 种（C3；C2,U3；C2,U3 之后回退到第 1 / 2 / 3 新；C3 之后回退到第 2 / 3 / 4 新；C2,U2,U3；C1,X,C1；C2,U3,X；C2,U3 再建 20 个；C3 再建 20 个），压回的号 p 取 1 到水位 − 1 的每一个，建 n = 1、2、3 个，压完之后再正常建 0、1、30 次，每行判最后那张镜像。每组 249 行，243 行真把一个发过的号重发了（其余 6 行写路径拒了）。

| 压完之后再发 | 甲 T2-a | 甲 T2-c | 乙（记录） | 乙（记录 + 谱系上的墓碑） | 丙（按谱系配父） | 今天 checker 有红 | 全不红 |
|---|---|---|---|---|---|---|---|
| 0 次 | 38 | 26 | 243 | 243 | 243 | 172 | 0 |
| 1 次 | 49 | 33 | 243 | 243 | 243 | 151 | 0 |
| 30 次 | 54 | 36 | 87 | 117 | 87 | 44 | 93 |

原样：「PRESSAGG follow=0 caught T2a=38 T2c-A=26 T2c-A2=26 T2a/selmap=0 yi/rec=243 yi/rec+tomb=243 bing=243 I96code=166 I96newest=144 I96each=144 I919A=0」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-press.out:2`），「PRESSAGG follow=30 caught T2a=54 T2c-A=36 T2c-A2=36 T2a/selmap=48 yi/rec=87 yi/rec+tomb=117 bing=87 I96code=3 I96newest=3 I96each=81 I919A=0」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-press.out:17`），「PRESSAGG follow=30 checker_silent rows=93 caught_by=[]」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-press.out:18`）。

读法：乙、丙只在「被换之前那一版还在候选集里」时抓得到（压完立刻判全抓，再发 30 次只剩 87 行）；甲只抓「号被删过、墓碑还在」的那几族，抓到的不随时间丢；全不红的 93 行多是活记录被原地换的那几族（前缀 C3 一族、C2,U3 再建 20 个那一族），单镜像不变量在证据转出候选集之后都看不见，与调查报告第十一节的推断一致，这里给了数。

### 打中三：T2-c（只取谱系上的墓碑）漏掉「回退到 unlink 之前 → 重发」那一族

历史 `C2,U3,R2,P3x1` 再建 30 次：建 2、3 号；unlink 3（意图 txg 7 写墓碑 (3, 6, 死于 7)）；回退到 txg 6（3 号带回来、活着；墓碑树随现行那一版，墓碑留着、死亡 txg 7 落在被抛弃的那一段，不在谱系上）；开关压回 3 再建一个（3 号原地换成出生代 10 的新对象）；再建 30 次。最后那张镜像上：「PRESS prefix=C2,U3,R2 wm_before=4 p=3 n=1 follow=30 reissue=true outcome=[pressed now txg10] marks=R--RHHHHHH- checker=[] detail={"T2a/each": "根（实例 2、txg 24）：号 3 的墓碑出生代 6 而记录 [10]", "T2a/sel-x-map": "所选根墓碑 (3, 6) 对 checker 跨版本表里 10"}」（`research/prompts/m3-delete-checker-r2-opus-model/out/bad-press.out:329`）——只有按号、不滤谱系的读法红，T2-c 没对象（`-`），乙丙与今天的 checker 都不红。同形 12 行：「PRESSAGG follow=30 checker_silent rows=12 caught_by=['T2a', 'T2a/selmap']」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-press.out:21`）。

四句：分辨臂——分辨 T2-a / T2-c（同一张镜像一红一不红）；系统看得到——看得到，墓碑与记录都在同一版的树里；判据分句——正文第五节第二条「在违反条款字面的镜像上报成立：记漏判」只在 T15 选「写」（同一条时间线上号不复用写成明文）之后才成立：按今天 D8 已定项 6 的字面这张镜像与第 19 批 S 那一形同一地位，(3, 6) 与 (3, 10) 是两对、不违反「一对不复用」（调查报告第 248 行），所以这一格是「C479 想立的那条不变量选甲时，甲取 c 会漏」，不是今天字面的漏判；改法——T10 的乙、丙在这几格上不红（窗口已过），甲取 a 红。调查报告第二节说「合法历史上 a 与 c 判的一样」（推的），合法这一半量过成立（上面 0 红），坏状态这一半 a 比 c 多抓这一族。

### T2 的两种范围

「所选根墓碑 × checker 跨版本号→出生代表」（`sel-x-map`）比的那个出生代来自最后走到的那一版：表里同一个号后走到的那一版盖掉先走到的（冻结副本 `/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker/src/walk.rs:2390` 「self.inode_object_birth.insert(inode, object_birth);」），走读次序是最新根先、候选根后，所以它常拿较旧那一版的出生代比——压完立刻判时 243 行一行没抓（上表 follow=0 那一行 `T2a/selmap=0`），每版各比自己的（`T2a/each`）抓 38 行。合法状态上两种都 0 红。这一种范围判的东西随走读次序变，不建议取。

## 第三节　T12（I-9.6「树内最大 key」跨根取）（重点）

今天的代码在主走读走到的每一版的每片叶上取最大（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker/src/walk.rs:2395` 「self.largest_inode_number_in_the_inode_tree = Some(」），再与最新根的水位比（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker/src/walk.rs:9338` 「"记账里的 inode 号水位 {watermark} 不大于 inode 树内最大 key {largest_inode}"」）；条款字面是「> 该树 inode 树内最大 key」（`.claude/kb/invariants.md:293`）。原型量了三种取法（只看树内那一半，墓碑那一半不在这里）：`code` 今天的跨版本最大对最新根水位（= 正文「改 kb 字面成候选集里任一版」），`newest` 只取最新根自己的（= 「改代码只取最新根」），`each` 每一版自己的水位对自己的最大 key（攻方补，零轮，与 I-9.5 共用判法同形）。

合法：三种都 0 红——「AGG I96/code holds=11527 red=0 na=0」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:25`）、「AGG I96/each holds=11527 red=0 na=0」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:26`）、「AGG I96/newest holds=10260 red=0 na=1267」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:27`）。取样含回退到第 1–8 新、崩溃恢复落到旧根后重发、崩在段界（见第一节取样）。没造出「候选集里某一版的 key ≥ 最新根水位」的合法状态。

坏（第二节那 243 × 3 行）：三种两两互不包含。命令与原样输出（在模型目录 `out/` 里跑）：
```
$ python3 -c "import re; rows=[re.search(r'follow=(\d+) reissue=(\w+) outcome=\[[^\]]*\] marks=(\S+)',l).groups() for l in open('bad-press.out') if l.startswith('PRESS ')]; ..."
follow 0 code-only-vs-newest 22 newest-only-vs-code 0 code-not-each 22 each-not-code 0
follow 1 code-only-vs-newest 11 newest-only-vs-code 0 code-not-each 8 each-not-code 17
follow 30 code-only-vs-newest 0 newest-only-vs-code 0 code-not-each 0 each-not-code 78
```
（脚本全文：对每个 follow 取 reissue=true 的行，marks 第 8、9、10 位分别是 code、newest、each，算集合差。）

- 改代码只取最新根：比今天少抓 22 行（压完立刻判）、11 行（再发一次），没有一行是它独有的。少抓的是「回退到号更少的那一版之后压回」「删完之后压回」一类：最新根自己的树里没有那个大号，较旧那一版有。
- 每版各比自己的（零轮）：压完再发几次之后它抓得多（再发 30 次 81 行，今天的读法 3 行），因为压回那一版的水位不大于它自己树里的最大号、那一版还在候选集里，而之后的版本水位又被正常发号推高了；压完立刻判时它比今天少抓 22 行。三种取并集最全；合法状态上三种都 0 红，并集也 0 红（推的：各自 0 红，并集是逐版逐条的析取）。
- C479 变体 c（删完 3、压回 2）：今天 I-9.6 红（`bad-press.out` 第 57 行 `checker=["I-9.6"]`），树内那一半在 `code` 下红、在 `newest` 下不红；但墓碑那一半（水位 3 不大于墓碑号 3）照样红，所以取哪一种都不改变 I-9.6 整条红不红，只改变是哪一半红。

打中？没有：T12 两个候选在合法状态上都站得住；坏状态上今天的读法（改 kb 字面）严格比只取最新根强。攻方补的「每版各比」是第三个候选，零轮。

## 第四节　T11（S4a：容器内记录次序）（重点）

- 合法：甲 / 乙的判法（类型 2 容器内记录号严格递增）0 红：「AGG S4a-jia/type2-sorted holds=11527 red=0 na=0」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:29`）。不限类型的读法红：「AGG S4a-bing/all-types-sorted holds=11040 red=487 na=0」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:28`），红的是墓碑容器（按死亡先后追加，删 3 再删 2 就是 [3, 2]；删 → 回退 → 再删还有 [1, 1] 两条同号），坐实调查报告「丙要限定类型 2」。
- 坏镜像（`C2` 之后那片叶 [1, 2, 3] 上改）：S4a [1 nlink 0, 1 nlink 1, 3]（第二条整条换成第一条、第一条 nlink 0）今天 0 红、新判法红：「checker_red=[] candidates=["S4a-jia/type2-sorted=R", "S4a-bing/all-types-sorted=R"]」（`research/prompts/m3-delete-checker-r2-opus-model/out/bad-s4a.out:3`）；两种互换 [2, 1, 3]、[1, 3, 2] 同样今天 0 红、只红新判法（`bad-s4a.out` 第 5、6 行）；S4b 今天红 I-9.5、I-4.7（第 4 行）。所以甲 / 乙都做得到「只红自己」（新条不进 checker，这里只量了今天的 checker 不连带红）。
- 跨容器同号：240 个号两片叶，把第一片最后一条改成第二片第一条的号，今天 I-9.4 与 I-9.12 两条一起红（`bad-xleaf.out` 第 4、5 行）；调查报告「跨容器的同号今天已经被 I-9.4 罩住」成立，只是 I-9.12 也连带。
- 跨子树：inode 树今天长不过一层——建 31 800 个号被写路径拒：「STEP #1 C31800 -> REFUSED Publish { cause: InodeTreeWriteRefused(MoreLeafContainersThanOneRootNodeHolds { containers: 137, capacity: 135 })」（`research/prompts/m3-delete-checker-r2-opus-model/out/show-C31800.out:10`），所以跨子树同号的合法前缀造不出来，洞今天只在同一片叶里。
- X2（扫描对同号第二条记判不了）没进原型，代价沿用第 20 批攻方的量数（调查报告第 12.4 节），这里没重量。

## 第五节　坏镜像只红不红自己（T9 连带）与墓碑死亡 txg 的漏判

在最新根第一片墓碑容器的第一条墓碑上改一处，重封、补校验和；今天的 checker 与各候选读法各红什么（`bad-tomb.out`）：

| 行 | 历史 | 改法 | 今天 checker | 候选红的 |
|---|---|---|---|---|
| 2 | C1,C1,U3 | 对象 ID 3 → 2（调查报告第二节 I-9.10 那份） | 0 | T2a、T2c、**I918 三种**、乙（带墓碑） |
| 3 | C1,C1,U3 | (3, b3) → (2, b2)（第五节 I-9.19 那份） | 0 | **I918 三种**、I919 |
| 4 | C1,C1,U3 | 区间止 → 5（第六节 I-9.20 那份） | 0 | **I918 三种**、I920 |
| 6 | 同上再建 30 次 | 对象 ID 3 → 2 | 0 | T2a、T2c、乙（带墓碑） |
| 7 | 同上再建 30 次 | (3, b3) → (2, b2) | 0 | 只 I919 |
| 8 | 同上再建 30 次 | 区间止 → 5 | 0 | 只 I920 |
| 10 | C2,U3,Y4（崩在第一批之前的崩溃镜像） | 死亡 txg → 最新根 txg + 1（第四节 I-9.18 那份） | 0 | 只 I918（与下面零轮那条） |
| 5 / 9 / 11 / 12 | 删完 / 意图之后 | 死亡 txg → 0，或 → 对象出生代 − 1 | 0 | **I-9.18 / 19 / 20 各读法一个都不红**，只有零轮那条 |

原样（各行「」之后的尾段）：「newest_txg=9 clean_checker=[] clean_cand=[] checker_red=[] cand_red=["I918/A", "I918/B", "I918/A2", "I919/A", "I919/A2"]」（`research/prompts/m3-delete-checker-r2-opus-model/out/bad-tomb.out:3`）；「newest_txg=39 clean_checker=[] clean_cand=[] checker_red=[] cand_red=["I919/A", "I919/A2"]」（`research/prompts/m3-delete-checker-r2-opus-model/out/bad-tomb.out:7`）；「newest_txg=7 clean_checker=[] clean_cand=[] checker_red=[] cand_red=["TombDeath/birth-le-death-le-txg"]」（`research/prompts/m3-delete-checker-r2-opus-model/out/bad-tomb.out:11`）。

### 打中四：调查报告三份坏镜像在 T9 选「并」时做不到只红自己

调查报告写的是「I-9.18：队列空；只红 I-9.10」（`/tmp/claude-1000/m3-delete-checker-r2/prep-report.md:82`）、「I-9.18 队列空；I-9.20 不重；I-9.6 2 < 水位。只红 I-9.19」（`/tmp/claude-1000/m3-delete-checker-r2/prep-report.md:162`）、「I-1.7：2 × 40 ≤ 80。只红 I-9.20」（`/tmp/claude-1000/m3-delete-checker-r2/prep-report.md:179`）。删完那一版队列确实空，但意图那一版（txg 8，队列 {3}）还在候选集里，而且与删完那一版共用同一片墓碑容器（开放容器只在有新记录的那次发布重写），改那片容器就同时改了意图那一版的墓碑，I-9.18 在意图那一版上红（行 2、3、4）。

四句：分辨臂——这一格判的是坏镜像的造法，不判 T2 / T3 的臂（I918 的 a、A2、b 三种一起红）；看得到——是；判据分句——正文第三节「坏镜像能不能只红自己」；改法——再建 30 次让意图那一版转出候选集之后再改，I-9.19、I-9.20 两份只红自己（行 7、8，量过）；I-9.10 那份仍连带乙（若乙带谱系上的墓碑，行 6），乙只比记录时不连带。T9 选「不并」时这三份照调查报告那样只红自己（推的：行 2–4 里多出来的只有 I918）。

### 打中五：墓碑死亡 txg 改成早于对象出生代，九格里没有一条红（漏判）

历史 `C2,U3,Y4`：建 2、3 号（txg 6），unlink 3 的意图那次 txg 7 落盘后崩在第一批之前（`Y4`：前 4 段全持久，就是意图那次发布整次落盘、第一批一个写都没落）。这张崩溃镜像上队列 {3}，墓碑 (3, 6, 死于 7)。把死亡 txg 改成 0 或 5（早于对象出生代 6）：I-9.18 的「在谱系上」只问死亡 txg 落不落在链上，[0, 7] 整段都在链上，所以 a、A2、b 都成立；I-9.19、I-9.20 也都不红（行 11、12）。删完那一版上改成 0 同样（行 5、9）。

四句：分辨臂——不分辨（T3 三种读法、T14 两种读法一起不红），病根在各读法共用的「在谱系上 = 死亡 txg 落在链上」，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 那张表的第一行办，另立一笔账；看得到——看得到，墓碑记录自带对象出生代，这一版的 txg 在根里；判据分句——正文第五节第二条「在违反条款字面的镜像上报成立：记漏判」，违反的字面是 D18 已定项 24 第 2 条「死亡 txg = 意图那次发布的 checkpoint_txg」（`.claude/kb/decisions/18-块里携带什么信息.md:571`）；改法——正文 T2–T15 的候选在这几格上都还不红（上表），攻方补「对象出生代 ≤ 死亡 txg ≤ 这一版的 txg」（零轮，只在我的模型上量过、被攻过零轮）：合法 0 红「AGG TombDeath/birth-le-death-le-txg holds=6337 red=0 na=5190」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:36`），这几份全红。它只缩小窗口、不等于判「等于意图那次的 txg」：改成 [对象出生代, 意图 txg) 里另一个链上的 txg 仍判不出（推的）。连带：它与调查报告 I-9.18 那份坏镜像（死亡 txg → 最新根 txg + 1）同红（行 10），采纳它的话 I-9.18 的坏镜像要改成「死亡 txg → 被回退抛弃那一段里的一个 txg」（在 [出生代, 版本 txg] 里、不在链上），推的、没造。

## 第六节　其余能造的几格

- **T4（谱系上的墓碑再按已发布谓词滤不滤）**：合法状态上滤与不滤逐格同数，「AGG I918/A+pub holds=6337 red=0 na=5190」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:7`）与 I918/A 同；I-9.19 同（第 13 行）。取样里含崩在段界后由记录施加出来的那几版与崩溃恢复抛弃的根，没造出候选集里有「未发布」墓碑容器的合法版本。没打中。
- **T14（I-9.20 的每条记录管谁）**：区间那一句按全部墓碑判 0 红「AGG I920/interval-all-T14 holds=6337 red=0 na=5190」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:21`）；不重号那一句按全部墓碑判 81 红「AGG I920/all-dedup holds=6256 red=81 na=5190」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:20`），就是「删 → 回退 → 再删」两条同号墓碑，条款原文已写这一形必须滤谱系。调查报告第 174 行「写法相同，两种读法在合法镜像上同绿（推的）」（`/tmp/claude-1000/m3-delete-checker-r2/prep-report.md:174`）只对区间那一句成立；把 T14 的「全部」理解成整条 I-9.20 都管全部就误红。这是读法澄清，不算新打中。
- **T5 / T6（旋钮）**：墓碑容器压到 1 或 2 条、码 2 节点压到 2 或不压，5 段历史 × 3 种旋钮（含回退、崩溃恢复），I-9.21 产品读法在合法状态上红 42 次：「COUNT knob	knob	I921/each	holds=0	red=42	na=57」（`research/prompts/m3-delete-checker-r2-opus-model/out/bad-knob.out:23`），只判所选根同数（第 24 行）。坐实调查报告「不定旋钮信号就红」；T6 两种读法在这些状态上没分开。产品容量下的 I-9.21 要 816 次 unlink 才有第二片容器，没造。
- **T7（身份不符怎么报）**：墓碑树 / 谱系树根第一条条目的容器号 +1、类型段改成 7 或 0，今天 checker 红的都是 I-4.8、I-7.2、I-7.4 三条，例：「"I-7.2: 最新的根走不完：墓碑树内部条目的身份 (65545, 1, 2, 7) 与容器头 (65545, 1, 1, 7) 不符"」（`research/prompts/m3-delete-checker-r2-opus-model/out/bad-t7-a.out:4`）。调查报告「T7-b 下坏镜像必连带 I-4.8、I-7.4，做不到只红自己」（`/tmp/claude-1000/m3-delete-checker-r2/prep-report.md:98`）漏了 I-7.2：T7-a 要「不再记走读失败」，最新根那一遍的 I-7.2 才不连带。调查报告推的「推的连带：I-3.1 红」（`/tmp/claude-1000/m3-delete-checker-r2/prep-report.md:91`）两段历史上都没出现，包括那片墓碑容器只被最新根引用的崩溃镜像（`C2,U3,Y4`，类型段 → 7：「"I-7.2: 最新的根走不完：墓碑树内部条目的类型段 7 不在 {0, 1} 里"」（`research/prompts/m3-delete-checker-r2-opus-model/out/bad-t7-y.out:5`））。
- **T8 / T13 / T15**：没造。I-9.24 在全部合法状态上 0 红（`analysis-sweeps.out` 第 24 行），用户动作只走会话，队列多于一条造不出来；T15 是条款要不要写，原型能给的只是「乙、丙在合法取样里 0 红」这一条旁证（第二节）。

## 第七节　GPU 判器跟不跟得上（读代码，推的，没跑 GPU）

- 扫描内核的删除态扫描是「一版一版重填」：回退候选集里每条根各调一次 `d_judge_deletion_state_of_a_version`（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2431` 「d_judge_deletion_state_of_a_version(root.base, 36u);」），每版的 inode 记录表上限「const D_MAX_INODE_RECORDS: u32 = 1024u;」（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:764`），表满记判不了（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2105` 「undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (49u << 8u);」），判不了的状态回落 CPU（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:1658` 「判不了的状态交回 `None`（回落到 CPU）」）。每个状态的草稿区「const SCRATCH_END: u32 = 298736u;」（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:776`）。
- 每版各判、只要这一版自己数据的读法（T3-a、A2、b，T2-a / c 每版，I-9.18、I-9.19、I-9.24、I-9.21 / I-9.22，攻方补的死亡 txg 界、I-9.6 每版各比）套得进这个做法：I-9.18 的队列至多一条 key（I-9.24），只要扫一遍墓碑找那一个号；I-9.19、T2 是逐条墓碑去查这一版的 inode 记录表，墓碑可以流式扫、不存表。谱系记录 T3-a 每版读一次，A2、b 只读所选根那一张。
- 要表的读法：I-9.20「按 (号, 对象出生代) 不重复」要这一版谱系上墓碑的一个集合；产品镜像一生最多约 11 万条墓碑（调查报告第 344 行引的数），每条至少 4 个字（号与出生代各 64 位），一版就要四十多万字，超过整个草稿区；只能设表上限、超了判不了回落 CPU——层 0 的流里墓碑只有几条，对拍看不出，产品规模上 GPU 那一路对这一条等于不判。乙要跨版本的号→出生代表（版本数 × 记录数），丙要同时拿着一版与它父版本两张记录表，T12 今天的跨版本最大 key 走读内核已有一个跨版本量、不加表。这三样都超出「一版一版重填」，要新表或两遍扫描。
- 覆盖表要求每条新下标都进内核（调查报告第十六节第 3 条），所以上面要表的几条要么在 GPU 上设上限回落、要么不进这一批；这是代价项，不是正确性项。

## 几个改法各修哪一格

攻方自己提的改法都只在我的模型上量过、被攻过零轮。「量过」= 模型副本上的原样输出（行号见各节）；「推的」= 按代码推、没实现没跑。

| 改法 | 打中一（T3-b I-9.18） | 打中二（T3-b I-9.19） | 打中三（T2-c 漏一族） | 打中四（坏镜像连带 I-9.18） | 打中五（死亡 txg） |
|---|---|---|---|---|---|
| T3 取 a | 量过：不红（analysis-sweeps 第 6 行） | 量过：不红（第 12 行） | 不相干 | 不修（行 2–4 里 a 照红） | 不修（行 11、12） |
| T3 取 A2（零轮） | 量过：不红（第 8 行） | 量过：不红（第 14 行） | 不相干 | 不修 | 不修 |
| T2 / C479-甲 取 a（按号、不滤谱系） | 不相干 | 不相干 | 量过：12 行只有它红（analysis-press 第 21 行） | 不修 | 不修 |
| 坏镜像改在意图那一版转出候选集之后 | 不相干 | 不相干 | 不相干 | 量过：I-9.19、I-9.20 只红自己（bad-tomb 行 7、8）；I-9.10 那份连带乙（带墓碑时，行 6） | 不相干 |
| 加「对象出生代 ≤ 死亡 txg ≤ 版本 txg」（零轮） | 不相干 | 不相干 | 不相干 | 新连带：I-9.18 那份要换造法（推的） | 量过：这几份红、合法 0 红（analysis-sweeps 第 36 行）；改成链上另一个合法区间里的 txg 仍判不出（推的） |
| I-9.6 加「每版各比自己的」（零轮） | 不相干 | 不相干 | 不相干 | 不相干 | 不相干；它修的是第三节「压完再发」那一族（量过：再发 30 次 81 行对 3 行） |

## 没打中的形状

- T3-a / A2 误红：上面五个家族 1 684 段历史、11 527 个状态，含回退到第 1–8 新、崩溃恢复后重发、三个号的 6 种删除次序、崩在每个屏障段界后重挂载、随机 300 段；0 红。
- T2 各读法、乙、丙（按谱系配父）、T12 三种取法、S4a 甲 / 乙判法、I-9.24：同一批合法状态上 0 红。
- T4 滤与不滤分不开：没造出候选集里有未发布墓碑容器的合法版本。
- 管理员回退到被崩溃恢复抛弃的根：入口按现行那一版的实例表判候选（`RollbackError::TargetNotACandidate`）；草稿跑（`/tmp/claude-1000/m3-delete-checker-r2-opus/sweep2-*.txt`，带逐步结局；草稿、没入模型，只作线索不作依据，入库的模型输出不带逐步结局所以没收它）里回退被拒的 269 次都是这一种，没有一次接了被抛弃的根，所以「回退把被抛弃时间线的号带回来」那一形造不出。
- 实例表清扫删掉被抛弃实例那一行之后让被抛弃的根回到候选集：历史里再建 40 次、重挂载，行没被删（产品清扫配置下），没走到（草稿 `/tmp/claude-1000/m3-delete-checker-r2-opus/show-sweep.txt`，只作线索；没入模型是因为这一形没走到、不支撑任何一格的判定）。
- 跨子树同号：inode 树长不过一层（第四节），造不出。

## 这条腿自己的限度

- **崩溃状态没有按层 0 的枚举域枚举**。每一步单独的闭式状态数都在 10⁶ 以上：「CRASHCOUNT b144:U1 total=83886093 cap=1000000 run=false」（`research/prompts/m3-delete-checker-r2-opus-model/out/crash-count-c1.out:2`），建一个 inode 那一步「  step #1 C1 [ok versions=1 now txg6] closed_form=1048583 segments=[20, 2, 1, 2]」（`research/prompts/m3-delete-checker-r2-opus-model/out/crash-count-c1.out:8`）。按派发规则「一段历史自己就超过约 10⁶ 的，那段不跑」，c1、c2 两组 12 段历史全部没跑；全量合计 0 个状态（离 10⁷ 的总额一个没用）。代替它的是历史里的 `Y<k>[f]` 一步：崩在上一步那段写的每个屏障 / FUA 段界（与段里第一个写之后）再重挂载，崩溃镜像本身与重挂载之后的镜像都判——这是挑出来的崩溃点，不是枚举，段内整写子集只取了「全无 / 第一个 / 全有」三种。
- 我造的「崩溃恢复抛弃最新那条根」之后，今天的 checker 在 250 个状态上红 I-7.4：「AGG checker_red I-7.4 states=250」（`research/prompts/m3-delete-checker-r2-opus-model/out/analysis-sweeps.out:38`），全在带 `X` 的历史里，是已知的 C554「新实例复用了它们的槽」（`.claude/kb/checks-owed.md:427`）那一形。这些状态照样计入候选的分母；候选读法在它们上面也 0 红。
- 候选判法是我在原型里写的（`proto/src/cand.rs`），读的事实来自副本 walk.rs 的 `attack_stash`（同一份版本清单、同一套只读读法），不是实现；实现时的细节（判不了怎么报、配对配不上怎么办）没模拟，配对按「每版一棵 inode 树」取。
- 只抽了一次样；「没打中」的几格照 `.claude/rules/three-way-inference.md` 要再抽一次才算。
- 跑原型时有一次经内存包装撞了 4G 上限（命令 `bash r.sh bad-press2.txt bad press …`，`M3DCR2_FOLLOWS=0,1,30`，退出码 250，6 个线程每段历史每步各留两份整池镜像），那一次的输出不算；改成压回模式不留每步镜像之后在包装里重跑（模型里的 `press` 步就是改过的那一份）。

## 没做什么

- 没跑任何 checker 档测试、GPU、层 0 重放；没编 `gpu` 特性。
- 没改仓内 crates；拷贝上只加了只读的 `attack_stash` 与原型 crate（两份补丁在模型目录）。草稿目录 `/tmp/claude-1000/m3-delete-checker-r2-opus/` 里剩下的是草稿输出与原型源码的草稿（与模型目录同一份），编译目录与仓拷贝都已删。
- 没量 X2；没造产品容量下第二片墓碑容器（816 次 unlink）；没造 C616 那一步的坏镜像；没判 I-9.17 / I-9.23 的新判定（只量了今天 checker 红什么）。
- T8、T13、T15 没攻；正推、辩方那几格不判。

## 引文核对

`python3 scripts/cite-check.py research/prompts/m3-delete-checker-r2-opus-output.md` 末行原样：`✓ 核了 73 处引文，对上 73 处，没判 0 处（逐处列在上面）`
