# m3-w3-r3 攻方腿第二次抽样（Opus，s2）报告：K2、K3（2026-10-09）

轮名 m3-w3-r3（最终轮）；背景材料 `research/prompts/_m3-w3-r3-background.md`（正文 `research/prompts/_m3-w3-r3-body.md`，附录 `research/prompts/_m3-w3-r3-appendix.md`）；前两轮判决 `research/prompts/m3-w3-r1-main-verification.md`、`research/prompts/m3-w3-r2-main-verification.md`。
冻结树 `~/.local/share/singlefs/work/m3-w3-r1/tree/`（只读）；草稿 `~/.local/share/singlefs/work/m3-w3-r3-opus-s2/`；模型 `research/prompts/m3-w3-r3-opus-s2-model/`。
这是这一轮攻方的第二次抽样，没读第一次抽样与正推腿的报告、模型、草稿。数全是我自写的抽象模型或拷进草稿目录的前两轮模型上量的，不是入库装置上的数；我提的改法只在我的模型上量过、被攻过零轮。

（本文件分段追加：先写复跑与 sha256、判定一览，再按攻击面分节。）

## 复跑命令与模型文件

在仓根，`W` 取一个空目录；每段一条前台命令（各在 4 分钟以内），编译与跑都经 `research/scripts/run-with-memory-cap.sh`（编译 6G、跑 4G），线程 5：

```
M=research/prompts/m3-w3-r3-opus-s2-model; W=<空目录>
for p in build run-a run-b run-c run-d traces recheck-r2 compare; do SINGLEFS_REPO=$PWD bash $M/rerun.sh $W $p; done
```

交回前在空目录 `~/.local/share/singlefs/work/m3-w3-r3-opus-s2/rerun-empty/` 从头跑了这八段：`compare` 末行 `compared=19 status=0`；`recheck-r2` 打印「r2 攻方 main 0：与它的 expected/model-main-0.out 逐行相同」「r2 辩方 defense 4：与它的 expected/model-defense-4.out 逐行相同」。`expected/` 是草稿目录里另一次独立跑的输出（同一份源码），不是这一次。`compare` 只比 `W3R3S2` 开头的行、去掉 `elapsed_s`、排序后比。

模型目录每个文件的 sha256（`find . -type f | LC_ALL=C sort | xargs sha256sum`）：

```
947c83e0b2a58acaf8e059a150e5e7672836f278f3dc753504090e3dab6f7c63  ./Cargo.toml
245714b2ce4b4efdcc5b3a1442bf90d840b920ff4adbaa79a3fc9ad5556fd822  ./expected/run-0.out
3d4937a4ee9a4118869b74be0e5bbef9c8352dd84b27bd57528dcade262ce28c  ./expected/run-1.out
6bf37e1db97e748ed0dfb04eddd01fda0651c7da4a83a1fcb33451d78db73745  ./expected/run-10.out
0f0d24b420bd78f483fa6cf5d83674f94523965a4c2dc290152228bdc9fa6005  ./expected/run-11.out
47c387f13d47b2fd60d71574010bbc23ab357362e226677d6dba71efdaf5aeba  ./expected/run-12.out
12028d72cab03e64a96db01189f2761590561bfaaa66c774c4405afb4a25f3a7  ./expected/run-13.out
dc13b36353ac3e8cbd6fa8c0d5f60d757a2df0412bd9a75bf20fc65b18190b5b  ./expected/run-14.out
2f8bcc9420acf889216f9b5f790b9135bfc21bdb722c7c351790eaf7a9dbc5e1  ./expected/run-15.out
50222b48a3cdf48820acdc49a07712775e06762f9ebb7b4a98ba46ce9b380503  ./expected/run-16.out
6254cdb4ad7b08541fde23d6df4b47e92fbf488312dd180560a351d294ac8336  ./expected/run-17.out
38503446250b316112916b9b33f7732a3484c6a11d79b5ed2d15511c71d00e87  ./expected/run-2.out
2dbced0ad49240d77d88f65e1a714a0dae44406b2bfa598b731b2162dbde4cf7  ./expected/run-3.out
b2ee7d0d5a13c46ebb88e44f2837f53565c143ce6cd977290187ff96d66c6c41  ./expected/run-4.out
3056b048d2c31927572b2e6ba203b63ee234d435b33bde10a1c75af7fcad18bd  ./expected/run-5.out
2c1a456d85d360c2ddaf08daf967fdaa54c5e19b3ebf94c95a7c11ccaaf9ec17  ./expected/run-6.out
14eb46ee1104a812913600362f2e8618eea9cf00bc897dda6c3d63850c2eb0e5  ./expected/run-7.out
43d1f9a6c3d0d4f317b5500325d7a4e0df0c5cac41b81bf53f33edf93f1cc771  ./expected/run-8.out
c6f8fe66196e8939be057bbdcf4b5b790ea75f2f0f74140fecc84de4b9f41bb9  ./expected/run-9.out
f75bd1c7f9f490deadf7e62c595a4d56b5de872abedf01da337347fc0f75fd7d  ./expected/traces.out
54fba827f52b3d261a4b980b66b775d020db76b9e64b478db86cf2c7f830e151  ./rerun.sh
ed06b4bc269aea434afc6856ecc6dbc9de20df1995fba100088b7bcc9029144d  ./src/main.rs
eb4f2c28573f489888e7a3aefc240f53049499bc532812b36a84f6eb23303995  ./tools/table.py
```

装置只有一套：`src/main.rs` 是从头写的抽象时间线模型（不依赖 `crates/`，没拷前两轮的模型代码）。条文走法、候选与臂写在文件头；要点：

- 起点固定在新池新建文件之后（实例 1、txg 3、水位 2、树 [(1, 3)]），之后由用户定的步按字母表**全部穷举**到给定深度，只固定前缀；故障也是字母表里的步（`crash-hide` = 最新那条根与它的单元挂载时读不出、记录读得出，即第二轮攻方原型 B 那一类；`crash-hide-root-and-record` = 根与记录都读不出；`crash-hide-two` = 这条时间线上最新两条根都读不出；`crash-mid-create` = 建 inode 的发布记录写了、单元没写全；`rollback1-cur-hidden` = 回退那一刻 cur 的根槽读不出）。
- 管理员回退的水位按 Y1 两种取法各一臂：Opt1 = max(cur 内存里的, 环里读得出的全部根)，即今天 `mount.rs` 的取法；Opt2 = max(cur 内存里的, 环里读得出、按 cur 的实例表没被抛弃的根)。阳性对照：V1（取目标那一版的）、VR（只取环里读得出的根，C143 收严之前的式子）、V3（恢复时头 1 的水位少算 1）。
- 新实例第一个 txg 三臂：Product = max(读得出的根, 读得出的记录) + 1（冻结树 `mount.rs` 第 1317 行）、RingRootsOnly（丢了记录那一半）、SelectedPlusOne（所选根 + 1）。
- 克隆：`clone` 建头 2（树 ID 写死 2），水位取 origin 的、树里的对象照抄（D8 已定项 6 那一行克隆头的写法，`.claude/kb/decisions/08-核心索引结构.md:175`）；回退只在目标与 cur 头集合相同时做（今天 `mount.rs` 头集合不同就拒）。
- 判红只读最新那条根、它的表与它下面的历史、环里读得出的根与记录；「合法」= 这条时间线上没有同 (树, 号) 两个出生代、也没有发过的号 ≥ 水位；「洞」= I-9.29 与 I-9.6（只看树的口径）都不红。

18 组的规模：合计 118 422 597 张镜像（`expected/run-*.out` 每份第一行 `images=` 相加）；每组的臂与张数见 `expected/run-<组号>.out` 第一行（组号由编出来的二进制 `w3-r3-s2-model list` 列出）。

## 各格判定一览

数都是我的抽象模型上量的（组号指 `expected/run-<组号>.out`），被攻过零轮；「推的」= 按代码与条文推、没在模型或装置上跑。

| 格 | 形态 / 读法 | 判定 |
|---|---|---|
| K2 维持 D23 已定项 14（Opt1）：容许跳升那一句会不会放过真重发 | C2aL（第二轮最强形：回退跳升那次也留台阶行） | **没打中**：V1、VR、V3 三臂（组 1、2、3，各约 1088 万张）漏事件 0、漏持久镜像 0；Product（组 0，10 880 722 张）误报 0 |
| 同上 | 最宽的容许：「谱系里有过跳升就不判 ③」（C2a-skip） | **没打中**：同三臂漏 0，Product 误报 0 |
| 同上 | 照字面核来源：跳升那一格只在环里还读得出一条被抛弃、水位 ≥ 跳升值的根时才算一行（C2a-src） | **打中误报（判别子观测不到）**：组 0 误报 169 958 张，最短 `[create1,crash-hide,rollback1,crash-mid-create]`（`expected/traces.out` 第 9 行）：来源根的槽被新根盖掉之后，镜像上再没有能证明「跳升来自被抛弃的根」的东西 |
| 同上 | C3rL、C4g、C2i | 不需要容许，Y1 两种取法上漏与误报的格局相同：C3rL 0；C4g 只在 V3 漏（第二轮辩方已报的限度）；C2i 在 V1、VR 漏持久镜像，C2ig 0 |
| K2 改成只取没被抛弃的根（Opt2） | 三个形态与字面 C2a | **没打中**：七组 Product（组 0、4、8、10、12、14、15，回退合计 11 059 819 次；这个量两种取法都记）里「环里没被抛弃的根水位高过 cur 内存里的」0 次 ⇒ Opt2 的值恒等于 cur 内存里的水位；字面 C2a（只在建 inode 时留台阶）误报从组 0 的 349 428 降到组 4 的 0 |
| 同上：表读不出、根读不出怎么办 | 今天的回退 | **没打中**：cur 的实例表读不出今天在任何写之前就拒；读不出的根今天就跳过；Opt2 下没有哪一条根的值会被用到（上一行） |
| 同上：C143「回退那次取 max、不重发」退不退 | — | **不退，条件是 max 里留着「cur 内存里的」那一项**；把条款写成「只取环里没被抛弃的根」而丢了 cur（VR 臂）就是 C143 那一洞本身：组 6 重发事件 41 730、洞 17 714（Opt1 的组 2 是 39 769 / 16 348），最短 `[create1,rollback1-cur-hidden,create1]`（`expected/traces.out` 第 14–17 行；同一段在 Opt2 Product 上不重发，第 10–13 行） |
| K3 克隆头 | C2aL（台阶行按 (统计量, 树) 分，谱系 H_cur 取各头最大值或每头一个） | **没打中**：组 12 3 613 164 张误报 0，组 13（V1）漏 0 |
| 同上 | C3rL 照第二轮定义（根下一份只追加的 (号, 出生代)） | **打中误报**：组 12 512 388 张，最短 `[clone,create1,create2]`（`expected/traces.out` 第 20–23 行）；按树分、克隆不抄 origin 的记录：2 075 412 张（`[clone]`）；按树分、克隆照抄：0 |
| 同上 | C4g 照第二轮定义（谱系记录多带一个回退前水位 W，16 → 24 字节） | **打中误报**：组 12 107 460 张，最短 `[clone,create1,rollback1,create2]`（第 24–28 行）；每个头一个 W：0 |
| 同上 | C2i 照第二轮定义（每实例一行） | **打中误报**：组 12 312 057 张（C2ig 326 323），最短 `[clone,create1,create2]`；按 (实例, 头) 分：0 |
| K3 副本坏一份（w = 2） | 三个形态 | **没打中（推的，读代码）**：单份坏在读路径上透明（逐条位置试、任一份过校验就算读到），只在重放（两份都要验过）上把那次发布变成被抛弃的发布，那一形模型里已穷举；回退复活集逐盘验，单份坏就拒 |
| K3 新实例第一个 txg 被改坏、与被抛弃发布的出生代撞上 | 三个形态 | **没打中**：两种改坏（组 16、17，各 4 985 840 张）同一条时间线上的重发 0 次；身份相撞（另一个实例写过逐字相同的 (树, 号, 出生代)）93 207 张，三个形态在这些镜像上红 0、误报 0（不分辨臂）。附带：这一改坏今天没有一条检查红（I-3.12、I-9.29、I-9.6 都 0），`crates/mutations.tsv` 里也没有这一行 |
| 重算前两轮的数 | 第二轮攻方 55 040、163 700；辩方 12 056 / 76 552；闭式 65 543、1 048 583；记账叶 477 / 462；谱系容器 2 039 / 1 359 | **都对得上**：拷来的两份模型复跑逐行相同；算术重算相同；我的模型独立复现「跳升全部来自被抛弃的根」「字面 C2a 的误报全在跳升之后」与同一条最短反例 |

## 一、K2：Y1 两种取法

### 1. 今天的取法与「跳升」从哪来

回退那次的水位读环里全部读得出的根，不按实例表滤：「let ring_roots = readable_roots(」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7738`），再与 cur 内存里的取 max：「ring.max(current.inode_number_watermark())」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7755`）。条文同：「| 水位 | inode 号水位与树 ID 水位 = max(cur 那一版内存里的水位, 环里读得出的根的水位)，不跟着 R_old 退回去 |」（`.claude/kb/decisions/23-journal的角色与格式.md:385`）。

模型里 Opt1 就是这一取法；Opt2 把「环里读得出的根」换成「环里读得出、按 cur 的实例表没被抛弃的根」，cur 内存那一项不动。七组 Product（组 0、4、8、10、12、14、15）每一次回退都记了两个量：环里只有被抛弃的根水位高过 cur（「跳升只来自被抛弃的根」）的次数，与环里有没被抛弃的根水位高过 cur 的次数。后者七组全是 0（各组第一行 `jumps_from_a_root_not_abandoned=0`），前者组 0 是 87 885 次。这是第二轮攻方在它的模型上报的「跳升全部来自被抛弃的根」，在我这套独立写的模型上重得出来。

### 2. 维持（Opt1）：容许那一句会不会顺带放过一形真重发

容许跳升的判词我按从宽到严写了三种，与第二轮的最强形放在同一组历史上：

| 读法 | 怎么容许 | Product 误报（组 0） | 漏（组 1 V1 / 组 2 VR / 组 3 V3：事件、持久镜像） |
|---|---|---|---|
| C2aL（第二轮最强形） | 回退跳升那次留一行台阶 (T_rb, H_new)，不核来源 | 0 | 0、0 / 0、0 / 0、0 |
| C2a-skip（最宽） | 谱系里出现过 H_new > H_cur 之后 ③ 不再判 | 0 | 0、0 / 0、0 / 0、0 |
| C2a-src（照字面） | 只有环里还读得出一条按最新根的表被抛弃、该树水位 ≥ H_new 的根时，跳升那一格才算一行 | **169 958** | 0、0 / 0、0 / 0、0 |
| 字面 C2a（不容许） | 只在建 inode 时留台阶 | 349 428 | 0、0 / 0、0 / 0、0 |

三臂的规模：V1 事件 218 507（洞 105 633）、持久镜像 416 550；VR 事件 39 769（洞 16 348）；V3 事件 378 708（洞 87 356）。其中发生在「之前有过一次跳升」的历史里的事件 V1 6 122、VR 625、V3 4 114 个，C2aL、C2a-skip、C2a-src 在这些事件上也都漏 0（各组 `event_missed_after_jump` 列）。

读法：

1. **没打中「放过真重发」**。容许只放行回退那一步的**上跳**；同一条时间线上的重发要求水位落到一个发过的号以下，抓它的是 ①（水位 ≥ 每一行 H）、C4g 的「水位 < W」、C3rL 的「同号两个出生代」，三处都不经过 ③，也不读跳升那一行的来源。跳升那一行只会把 ① 的地板抬高，不会让哪一处比较变松。最宽的 C2a-skip 在三臂上都漏 0，就是这一点的观测（只在我的模型上，推翻条件：找到一段历史，C2a-skip 不红而 C2a-create 红、且那一张镜像上有同一条时间线上的重发）。
2. **打中：字面的那一句判不了**。「跳升只来自被抛弃的根」要单镜像看得到那条被抛弃的根；根环 24 槽转一圈（模型里 4 槽）之后它被盖掉，镜像上与「水位被乱抬」逐字段相同。最短反例：建 2@4 → 崩溃时最新根读不出、恢复落到 (1, 3)，新实例 2 → 回退，水位从被抛弃的 (1, 4) 抬到 3 → 再一次挂载，新根 3:8 落进 1:4 的槽（`expected/traces.out` 第 5–9 行；第 9 行 `ring=[3:8:wm3,2:5:wm2,2:6:wm3,1:3:wm2]`、`red=[C2a-create,C2a-src]`）。所以维持 Opt1 时容许那一句只能写成「回退那次的上跳不核来源」（C2aL 的台阶行或 C2a-skip），不能写成「核到来源是被抛弃的根才放行」。
3. C3rL、C4g、C2i 不读水位盖不盖得住，不需要这一句；两种取法下它们的漏与误报格局一样（C4g 在 V3 漏事件 233 956 / Opt2 244 873，是第二轮辩方报过的「只管回退引起的退回」；C2i 在 V1 漏持久镜像 48 190、C2ig 0）。

### 3. 改成只取没被抛弃的根（Opt2）

**值**：上面那条观测（没被抛弃的根从没高过 cur）说明 Opt2 在产品路径上恒等于「cur 内存里的水位」。推的理由：发号只从上一版记账里的水位起、只增（冻结树 `crates/singlefs-core/src/transaction.rs:210` 那一段注释），回退取 max 不会让 cur 低于它的任何祖先，恢复从所选那一版接；按 cur 的表没被抛弃的根就是 cur 的祖先（恢复给所选根那个实例写 (i, 所选根 txg)、中间实例写 (i, 0)，`.claude/kb/decisions/18-块里携带什么信息.md:311`）。所以 Opt2 落到代码上可以就是把 `highest_inode_number_watermark_in_the_ring` 那一段（每条环里的根整版重建一次）拿掉，只留 `current.inode_number_watermark()`（推的，没改代码）。

**表读不出**：今天回退在任何写之前先读 cur 的表，读不出就拒：「let current_table = instance_table_chain_of_root(&*devices, &current.root)」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7633`）、「.map_err(|_unreadable_or_malformed| RollbackError::CurrentInstanceTableMalformed)?」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7634`）。候选集本来就要这张表，Opt2 不多出一个要拒的情形。

**根读不出**：根槽读不出或那一版重建不出的根，今天就不算（同文件第 7431 行那段注释）；Opt2 下按上一条它的值本来就用不到，不需要新条款。

**C143 那一格退不退**：不退，前提是 max 里那一项「cur 那一版内存里的水位」原样留着。C143 第三轮打中的正是只取环的式子：「Y1 打中式子的一个分句（一次瞬时读错，分辨读法）：inode 号水位只取环里读得出的根。现行那条根在回退那一刻读不出时，号 2、3、4 被重发，checker 红 I-9.6。」（`research/prompts/m2-rollback-forward-r3-main-verification.md:31`）。模型 VR 臂就是这一式子配「回退那一刻 cur 的根槽读不出」：Opt2 组 6 重发事件 41 730（洞 17 714），Opt1 组 2 是 39 769（洞 16 348）——Opt1 少的那几千个是环里被抛弃的高水位根碰巧把水位抬过去了，不是保证。最短 `[create1,rollback1-cur-hidden,create1]`：Opt2 Product 上回退水位 3、再建发 3@6（`expected/traces.out` 第 12–13 行），VR 上回退水位 2、再建重发 2@6（第 16–17 行，I-9.29 红）。所以改条款时那一格要写成「max(cur 那一版内存里的水位, 环里读得出、按 cur 的实例表没被抛弃的根的水位)」，不能缩成「只取没被抛弃的根」。

**树 ID 水位（推的，模型没量）**：同一格写的是「inode 号水位与树 ID 水位」，代码同一处也取环：「let tree_identifier_watermark = ring_roots」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7744`）。树的身份就是树 ID、没有出生代，被抛弃时间线里发过的树 ID 不许重发（`.claude/kb/checks-owed.md:277` 那一行 C342）。cur 内存里的树 ID 水位来自挂载时对环里全部根与记录取的 max；被抛弃的根与它的记录在挂载那一刻都读不出、回退时又读得出的那一形，今天回退取环能碰巧补上，Opt2 若连树 ID 一起滤就补不上了（那一形本来就是 C342 记下的「根与它那条记录都读不出那一格盖不住」，回退补上只是碰巧）。所以 Opt2 只该改 inode 号水位那一半；`crates/` 今天没有建树的路径，树 ID 水位实际不动。

**Opt2 顺带去掉的东西（量过一部分）**：没有跳升，字面 C2a 的误报 349 428 → 0（组 0 → 组 4），第二轮攻方那几种「跳升之后误报」的读法（③、C3r 的「记录盖住水位」、谱系带 H_new）都没有了对象；谱系那一对从「上一版」现算会读到 H_new 那一条陷阱也消失（H_new = H_cur）。但有克隆时「只在建 inode 时留台阶」仍不够：建克隆那次要为新树写一行（组 14 字面 C2a 误报 619 493，最短 `[create1,clone]`，`expected/traces.out` 第 31 行）。

## 二、K3：克隆头

`crates/` 今天没有建克隆的路径，回退遇到两版的树 ID 集合不同就拒：「RollbackError::TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7682`）。这一格只能按条文走：克隆头的水位取 origin 那一刻的、建克隆那次为新树写水位行、两个头此后各自发号、号会相交、唯一性靠 (树 ID, inode)（D8 已定项 6 那一行，`.claude/kb/decisions/08-核心索引结构.md:175`）。模型里树 ID 写死 2、回退只在两版头集合相同时做（与今天的拒一致）、水位按头取 max（Opt1 或 Opt2）。字母表 `create1,create2,clone,delete1,rollback1,rollback2,mount,crash-hide,empty`，环 4、深 8，组 12（Opt1 Product）、13（Opt1 V1）、14（Opt2 Product），各 3 613 164 张。

| 形态与读法 | 组 12 误报 | 组 14 误报 | 组 13（V1）漏事件 / 漏持久镜像 | 最短反例 |
|---|---|---|---|---|
| C2aL：台阶行按 (统计量, 树) 分（记账 key 本来带树维），谱系 H_cur 每头一个 | 0 | 0 | 0 / 0 | — |
| C2aL-pool：同上，谱系 H_cur 只带各头最大值、比各树台阶的最大值 | 0 | 0 | 0 / 0 | — |
| C3rL 照第二轮定义：一份 (号, 出生代)，count 一个数 | **512 388** | 516 744 | 0 / 0 | `[clone,create1,create2]` |
| C3rL 按树分 (树, 号, 出生代)，克隆不抄 origin 的记录 | **2 075 412** | 2 075 412 | 0 / 0 | `[clone]` |
| C3rL 按树分，克隆把 origin 的整份记录抄给新树 | 0 | 0 | 0 / 0 | — |
| C4g 照第二轮定义：谱系记录多带一个 W（取各头最大值） | **107 460** | 108 581 | 0 / 0 | `[clone,create1,rollback1,create2]` |
| C4g 每个头一个 W | 0 | 0 | 0 / 0 | — |
| C2i 照第二轮定义：每实例一行 | **312 057** | 309 667 | 0 / 28 510 | `[clone,create1,create2]` |
| C2i 按 (实例, 头) 分 | 0 | 0 | 0 / 26 135 | — |
| C2ig 照定义 / 按头分 | 326 323 / 0 | 323 923 / 0 | 0 / 0 | `[clone,create1,create2]` |

（C2i 漏持久镜像是第二轮辩方已报的限度，按头分不改它；C2ig 两种都 0。）

反例逐步（`expected/traces.out` 第 20–23、24–28 行，Opt1 Product）：

- `[clone,create1,create2]`：克隆在 txg 4 建头 2（水位 2，带 1@3）；头 1 在 txg 5 发 2@5；头 2 在 txg 6 发 2@6。第 23 行 `issued=[(1, 1, 3), (1, 2, 5), (2, 1, 3), (2, 2, 6)]`，没有同 (树, 号) 两个出生代，合法；`red=[C3rL-1log,C3rL-nocopy,C2i-pool,C2ig-pool]`。一份 (号, 出生代) 的记录里 2 号有 5、6 两个出生代，判成重发；每实例一行的 C2i 记的是头 1 的峰值 3（第一次到它在 txg 5），头 2 在 txg 6 发的 2 号「号 < 3 且出生代 > 5」，判成重发。
- `[clone,create1,rollback1,create2]`：回退时 cur 两头水位 3 与 2，谱系只带一个 W 就是 3；之后头 2 在 txg 7 发 2@7，「号 2 < W 3 且出生代 7 > T_rb 6」红（第 28 行 `red=[C3rL-1log,C3rL-nocopy,C4g-pool,C2i-pool,C2ig-pool]`）。

按头分之后的代价（推的，没量）：C4g 的谱系记录从「16 → 24 字节」变成 16 + 8k（k 是回退那一刻的可写头数），而谱系记录今天是一条 16 字节（D18 已定项 24 第 5 条，`.claude/kb/decisions/18-块里携带什么信息.md:576`；格式常量那一行「/// 谱系记录定长 16：T_rb 8 + T_old 8（D18（块里携带什么信息） 已定项 24 第 5 条，打包记录类型 5）。」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:118`）），要么改成变长、要么每头一条记录；一个 32 KiB 容器装 ⌊(32768 − 136) ÷ (16 + 8k)⌋ 条（k = 1、2、4：1 359、1 019、679，`python3` 算的）。C3rL 每条记录多 8 字节树 ID（16 → 24，或按树分容器），建克隆要么把 origin 至今发过的全部记录抄一份（与 origin 发过的号数成正比，模型里 230 505 次克隆抄了 535 074 条），要么让新树共享 origin 那几片记录、C3rs 改成「树里每条都在本树记录或 origin 建克隆之前的记录里」（没实现、没量）。C2i 每 (实例, 头) 一行。C2aL 不用改：记账 key 本来带树维，谱系 H_cur 取各头最大值在这组历史上也 0 误报 0 漏；但 462 行的「0 单元」余量要被 k 个头的台阶行分着用（推的）。

模型的一处已知失真：组 12–14 第一行的 `identity_collision_images=98740` 来自树 ID 写死 2（被抛弃时间线上建过头 2、新实例再建一次头 2，`[clone,crash-hide,clone]`），真实的树 ID 水位取环里全部根与记录的 max（D8 已定项 8 ②），这一数不读。

## 三、K3：副本坏一份（w = 2）

没在模型里单列一步，按代码把它对到已有的步上（推的，没在副本装置上跑）：

- **读路径上透明**：读单元按指针里的两条位置条目逐条试，「/// 按指针里的位置条目读一个单元：逐条试，整单元 CRC-32C 等于条目里的校验和才算读到。」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/recovery.rs:378`）。三个形态的历史（C2aL 的台阶行在记账树里、C3rL 的记录容器、C4g 与 C2i 随谱系或根下的结构）都是普通单元，挂载择根、重建所选那一版、回退重建 R_old、`highest_inode_number_watermark_in_the_ring` 重建环里的根都走这一条：坏一份时读另一份，历史与水位都不变。同一槽两份都过校验而内容不同的情形不存在：校验和在父指针里，读到的那一份要等于条目里的值。
- **重放上等于「那次发布被抛弃」**：施加前点名单元两份都要验过（同文件第 3535 行那段注释，条文是 D23 已定项 14 第 365 行那一段）。根还没落的那次发布有一个点名单元坏一份，整次不施加、恢复落到上一版——就是模型里 `crash-mid-create`（记录在、单元不全）与 `crash-hide`（根读不出、记录在）那两步；它们在组 0–17 每一组的字母表里都穷举了，三个形态的 Product 误报都是 0。C3rL 每次建 inode 的发布多点名 2 个单元，坏一份就把这种发布推成被抛弃的点多 2 个（代价，不是漏报；推的）。
- **回退上是拒**：复活集逐盘验，「    every_copy_of_the_resurrected_units_reads_back(&*devices, &units_to_check_on_every_device)?;」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7704`），坏一份就在任何写之前拒，不留历史。三个形态的历史都「从 cur 带」，不读 R_old 的历史单元，R_old 那边的副本坏不进它们。
- 一块盘整块掉了：第一版两盘池只能只读挂载，不取号、不发号，三个形态没有对象。

判定：**没打中**。扫的是这四条读写路径，没有一条让单份坏改变三个形态读到的历史；推翻条件：找到一个读历史单元的地方只读某一块盘上的那一份（不经逐条位置试），或池级 checker 判这几样历史时只看一份。

## 四、K3：新实例第一个 txg 被改坏、与被抛弃发布的出生代撞上

今天的取法：「    checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:1317`）。两种改坏：RingRootsOnly（丢了记录那一半，组 16）、SelectedPlusOne（所选根 + 1，组 17；我的模型里所选根恒是环里读得出的最新根，两组数相同）。字母表 `create1,empty,rollback1,mount,crash-hide,crash-hide-root-and-record,crash-hide-two,crash-mid-create`，环 4、深 8。

| 组 | 镜像 | 同一条时间线上的重发 | 身份相撞的镜像（另一份读得出） | 三个形态在相撞镜像上红 | 三个形态误报 | I-9.29 / I-9.6 / I-3.12 在相撞镜像上红 | X-txg（我提的） |
|---|---|---|---|---|---|---|---|
| 15 Product | 5 326 115 | 0 | 0 | — | 0 | — | 误报 1 037 260 |
| 16 RingRootsOnly | 4 985 840 | 0 | 93 207（93 207） | 0 | 0 | 0 / 0 / 0 | 相撞镜像上红 58 056 |
| 17 SelectedPlusOne | 4 985 840 | 0 | 93 207（93 207） | 0 | 0 | 0 / 0 / 0 | 相撞镜像上红 58 056 |

（「三个形态」= C2aL、C3rL、C4g、C2i、C2ig 五列都是这个数，`expected/run-16.out` 的 `red_on_identity_collision` 列。X-txg = 最新根的实例在环里最早那条根的 txg，不大于任何一条读得出、按最新根的表被抛弃的根或记录的 txg 就红。）

最短相撞（`expected/traces.out` 第 32–36 行）：空发布 txg 4 → 建 2@5 → 这条时间线上最新两条根挂载时都读不出、记录读得出（`crash-hide-two`），改坏的取法给新实例第一个 txg 4（没改坏是 6）→ 新实例建 2@5。第 36 行 `issued=[(1, 1, 3), (1, 2, 5)]`，与被抛弃的实例 1 那次发布写的 (1, 2, 5) 逐字相同，记录 1:5 还读得出；`red=[X-txg]`，三个形态、I-9.29、I-9.6、I-3.12 都不红。第 35 行还有一形：改坏之后被抛弃的 1:5 按择根的序排在新实例的 2:4 前面（`newest=1:5 cur=2:4`），组 16 这样的镜像 280 800 张——这时再崩一次，恢复会选回被抛弃的那条线（推的后果，模型没往下走）。

读法：

1. **三个形态没打中**：相撞的那一份在另一条（被抛弃的）时间线上，不是同一条时间线上的重发，三个形态按构造只读所选那一版的历史，既不该红也没红；误报 0。这一形不分辨臂。
2. **附带一处空格（零轮，交主 agent）**：这一改坏今天没有检查会红：I-3.12 只判行 T 低于引用它的根（组 16 相撞镜像上 0），冻结树 `crates/mutations.tsv` 里点名第 1317 行那句原文的只有第 1044 行一条（`grep -c` 命中 1），改的是溢出时 panic，不是取值的集合。我写的 X-txg 能抓到 58 056 / 93 207，但在 Product 上红 1 037 260 张：最短 `[create1,crash-hide-root-and-record]`（第 37–39 行）——被抛弃的根与它的记录挂载那一刻都读不出，产品取法给出的第一个 txg 本来就不大于它；条文要求的也只是新实例的 txg 大于环里读得出的根与记录（指路，不抄：D8 已定项 6 射程那一段，`.claude/kb/decisions/08-核心索引结构.md:207`；D23 已定项 14 注 3）。所以 X-txg 进不了不变量，除非条文先把那一形划出去；这一格不在 W3 的射程里。
3. 在 `crates/` 上走到相撞要几个故障（推的）：要这条时间线上连着两条根在挂载时读不出而记录读得出；较早的那次发布返回之前系统配置已轮换并持久（D23 已定项 3 射程那一句，`.claude/kb/decisions/23-journal的角色与格式.md:136`），c_见证 看得到它，重读一次仍读不出就拒可写，所以还要那两块盘上轮换的系统配置槽也读不出——改坏之外至少三类读故障。我的模型里每次挂载只发一次写行的发布；`crates/` 挂载若还推暖机的空发布，要被抛弃的链更长才撞得上。

## 五、前两轮报出的数能不能核

挑了四个：第二轮攻方的跳升次数与字面 C2a 的误报、第二轮辩方的 C4g 在 V3 上的漏、两处闭式与容量算术。

| 数（出处） | 怎么核 | 结果 |
|---|---|---|
| 攻方 `rollback_jumps_from_abandoned_roots_only=55040`、`I930/C2a-create false_positive=163700`（`research/prompts/m3-w3-r2-opus-model/expected/model-main-0.out` 第 1、3 行） | 把它的 `timeline-model/` 拷进我的工作目录、编、跑 `main 0`（`rerun.sh recheck-r2`） | 对上：输出与它的 expected 逐行相同；第 3 行原样见下 |
| 同上两句结论「跳升全部来自被抛弃的根」「每一个误报都在跳升之后」与最短反例 `[create,crash-hide,rollback1]` | 在我从头写的模型上独立量（字母表、深度不同，数不比） | 对上：七组 Product `jumps_from_a_root_not_abandoned=0`；组 0 字面 C2a 误报 349 428、其中跳升之后 349 428；最短反例 `[create1,crash-hide,rollback1]`（`expected/traces.out` 第 1–4 行） |
| 辩方 C4g 在 V3 上洞事件漏 12 056 / 76 552、C2ig 漏 0（`research/prompts/m3-w3-r2-sonnet-output.md:207` 那一行表） | 拷它的 `timeline-model/`、编、跑 `defense 4` | 对上：与它的 `expected/model-defense-4.out` 逐行相同，`table2.py` 打出 `C4g     10218    508048           12056              12056` |
| 同上，独立量 | 我的模型组 3（V3、Opt1） | 方向对上、数不比：C4g 洞事件漏 28 654 / 87 356，C2i、C2ig 漏 0；比例不同来自字母表不同（我没有抬 F、切换、墓碑口径，多了回退时 cur 读不出与建到一半崩溃） |
| 闭式 65 543（段 [16, 2, 1, 2]）、多 4 写 1 048 583（`research/prompts/m3-w3-r2-opus-output.md:184`） | 照冻结树 `crates/singlefs-harness/src/segments.rs` 的 `closed_form_state_count`（1 + Σ(2^写数 − 1)）用 `python3` 重算 | 对上：65543、1048583（[18, 2, 1, 2] 是 262151） |
| 记账叶 477 条、余 462；谱系容器 16 字节 2 039 条、24 字节 1 359 条（`research/prompts/m3-w3-r2-opus-output.md:182`、`:183`） | 用冻结树格式常量 159、34、136 重算 | 对上：477、462、2039、1359 |

攻方第 3 行（我的复跑输出，与它的 expected 第 3 行逐字节相同，`cmp` 退 0）：

```
W3R2COUNT arm=Product mutation=None ring=4 sweep_threshold=never longest=7 candidate=I930/C2a-create false_positive=163700 event_missed=0 residual_missed=0 mutated_history_undetected=0 false_positive_after_a_jump_from_abandoned_roots=163700 false_positive_after_a_sweep_or_switch_or_aba=138121 mutated_images=0
```

我的模型组 0 同一读法那一行（`expected/run-0.out`）：

```
W3R3S2COUNT group=k2 rule=Opt1 arm=Product first_txg=Product ring=4 depth=8 form=C2a-create false_positive=349428 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0 false_positive_after_jump=349428 event_missed_after_jump=0 red_on_identity_collision=0 event_missed_without_jump=0
```

对不上的：没有。要提的一处口径：第二轮判决把 C4g 的谱系记录写成「16 → 24 字节是推的」，这一数只在一个可写头上成立；有克隆头时是 16 + 8k（第二节）。

## 六、每个打中先答四句

| 打中 | 分不分辨臂 | 被判的系统当时看不看得到判别它的东西 | 满足的是判据字面的哪一个分句 | 跑前条款给的改法在这几格上 |
|---|---|---|---|---|
| C2a-src 误报（照字面核「跳升只来自被抛弃的根」） | 分辨读法：同一张镜像 C2aL、C2a-skip 不红（`expected/traces.out` 第 9 行） | **看不到**：来源根的槽被新根盖掉之后，镜像上没有任何东西能证明那次上跳来自被抛弃的根——「判别子观测不到」那一型 | 正文第五节「攻方坐实一形漏报或误报」的「误报」：合法镜像（没有同一条时间线上的重发）判红 | 正文第五节没有给改法，只要求交用户表写明那一形与代价；我的改法：容许写成「上跳不核来源」或取 Opt2，两者在这几格上都不中（组 0 C2aL、C2a-skip 误报 0；组 4 全部 0），量过 |
| 克隆头上 C3rL（一份记录）、C4g（一个 W）、C2i（每实例一行）误报 | 分辨读法：同一张镜像按头分的三种都不红（第 23、28 行） | 看得到：树 ID 在每个头、每条记录的身份里（唯一性本来就是 (树 ID, inode)） | 同上「误报」 | 同上没有给改法；我的改法：按头分（C3rL 还要建克隆时抄或共享 origin 的记录），在这几格上不中（组 12、14 误报 0，组 13 漏 0），量过；代价推的（第二节末段） |

不算打中、但交主 agent 的两处：Opt2 若写成「只取没被抛弃的根」而丢了 cur 内存那一项，就退回 C143 那一洞（组 6，第一节第 3 条）——这是改条款时的写法风险，不是 Opt2 本身的问题；新实例第一个 txg 改坏之后今天没有检查会红（第四节读法 2），不在 W3 射程里。

## 七、改法各修哪一格（都只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 标 | 依据 |
|---|---|---|---|
| Opt1 下容许写成「回退那次的上跳不核来源」（C2aL 留台阶行，或谱系里有过上跳就不判 ③） | C2a-src 误报 | 量过 | 组 0 C2aL、C2a-skip 误报 0；组 1–3 漏 0 |
| 取 Opt2：max(cur 内存里的, 环里读得出、按 cur 的表没被抛弃的根)，cur 那一项照留 | 跳升本身与它带来的全部误报（字面 C2a、C3r 的「记录盖住水位」、谱系带 H_new） | 量过（字面 C2a 那一格）/ 推的（另两种读法这一模型没实现） | 组 4、10 字面 C2a 误报 0；没被抛弃的根高过 cur 0 次 |
| Opt2 只改 inode 号水位，树 ID 水位照 D8 已定项 8 ② 取环里全部根 | Opt2 连树 ID 一起滤时补不上的那一形 | 推的 | 第一节第 3 条末段；模型没有树 ID 水位 |
| Opt2 落到代码上就是只取 `current.inode_number_watermark()`，去掉回退时整环重建 | 回退时重建环里每条根的读与它的失败面 | 推的 | 第一节第 3 条「值」 |
| C4g 每个可写头一个 W（谱系记录 16 + 8k 或每头一条） | 克隆头上 C4g 误报 | 量过（判词）/ 推的（字节） | 组 12、14 误报 0，组 13 漏 0 |
| C3rL 记录按 (树, 号, 出生代)，建克隆时抄（或共享）origin 的记录 | 克隆头上 C3rL 误报 | 量过（抄的那一种）/ 推的（共享那一种） | 组 12、14 误报 0；不抄 2 075 412 |
| C2i 按 (实例, 头) 分 | 克隆头上 C2i 误报 | 量过 | 组 12、14 误报 0 |
| 有克隆时 C2a 的台阶在建克隆那次也为新树写一行（条文本来要求写水位行） | 组 14 字面 C2a 误报 619 493 | 推的（C2aL 那一读法量过 0） | 组 12、14 C2aL 误报 0 |

## 没打中的形状

- **容许那一句放过真重发**：三种容许读法 × V1、VR、V3 三臂 × Opt1、Opt2，环 4 深 8（每组约 1088 万张）与环 6 深 7（约 132 万张），漏事件、漏持久镜像都是 0（C4g 的 V3 漏、C2i 的持久镜像漏两种取法相同，是第二轮已报的限度）。字母表 `create1,delete1,rollback1,rollback2,rollback1-cur-hidden,empty,mount,crash-hide,crash-mid-create` 全部穷举。
- **Opt2 退回 C143**：Product 七组（Opt2 是组 4、10、14）回退合计 11 059 819 次，同一条时间线上的重发 0；组 0、4、8、10 的字母表含「回退那一刻 cur 的根读不出」那一步。
- **克隆头上 C2aL**：组 12–14 共 10 839 492 张，误报 0、V1 漏 0。
- **副本坏一份**：没跑，按代码对到已穷举的步上（第三节）。
- **新实例第一个 txg 改坏**：三个形态误报 0、在相撞镜像上红 0；同一条时间线上的重发 0。
- 没试的形状：清扫删行、实例切换、A → B → A（第二轮攻过）；抬 F；墓碑与 `deleted_inodes`（I-9.29、I-9.6 只用树的口径）；两头以上的克隆、克隆之间的回退（今天拒）；多块盘上的副本分布。

## 这条腿自己的限度

- 只有抽象模型，没有在冻结树副本上建原型、没调层 0 枚举（`enumerate_layer0` 一类一次都没跑，层 0 状态量 0）。三个形态都没写进 `crates/`，它们自己的写在崩溃点上怎么落没进任何枚举。
- 模型的条文走法是我按条款读的：恢复恒选环里读得出的最新根、不按表滤（与第二轮攻方模型同），每次挂载只发一次写行的发布，删只删号最大的那个，克隆的树 ID 写死 2（克隆组的身份相撞数因此不读）。
- 「副本坏一份」只读了代码，没造故障。
- 「Opt2 恒等于 cur」是模型观测加推理，不是证明；推翻条件：`crates/` 上一段历史里，回退那一刻环里有一条按 cur 的表没被抛弃的根，水位高过 cur 内存里的。
- 每格只抽了这一次（这是攻方的第二次抽样，第一次的结论我没读）。
- 副本与模型上的数不是入库装置上的数。

## 没做什么

- 没跑任何重型测试：没编、没跑 `singlefs-checker-tier` 的测试与层 0 枚举，没跑门禁；编译与运行只有我的模型和拷来的两份第二轮模型，全部经 `research/scripts/run-with-memory-cap.sh`（编译 6G、跑 2G–4G）、`nice -n 19`、线程 5。
- 没判 K1（代价并排、Q1–Q7 归正推腿）；没替主 agent 采纳；没读第一次抽样与正推腿的报告、模型、草稿。
- 开工时 `ps` 看到别的会话在跑 `cargo test --offline --no-fail-fast -p singlefs-format -p singlefs-core -p singlefs-harness -p singlefs-checker`（pid 800319），没有性能测量在跑；没等锁。
- 草稿目录 `~/.local/share/singlefs/work/m3-w3-r3-opus-s2/` 里的编译目录（`model/target/`、`recheck/target-r2-opus/`、`recheck/target-r2-sonnet/`、`rerun-empty/target/`、`rerun-empty/recheck/target-*`）与拷来的两份第二轮模型源码交回前删掉；留下原始输出 `out/`（深 7 那一次试跑）、`out2/`（`expected/` 的来源）、`rerun-empty/out/`（交回前那一次从头复跑）与模型源码，约几 MB，主 agent 不要了整个目录可删。这些输出没进 `research/results/`：本轮产物按交付写进模型目录 `expected/`。
