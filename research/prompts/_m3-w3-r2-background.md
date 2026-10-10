# W3：盘上留发号历史——设计轮第二轮正文（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 K4 C1 C2 C3 C4 C2a C3r C2ay V1 V2 -->

## 一、题面

第一轮判决 `research/prompts/m3-w3-r1-main-verification.md`（腿报告 `research/prompts/m3-w3-r1-opus-output.md`、`m3-w3-r1-sonnet-output.md`，模型在同名 `-model/`）。第一轮留下：C1（实例表行记水位）、只记峰值或每实例一行的 C2、C4 出局；C2a（记账按每次建 inode 的发布留完整台阶加逐条记录比）与 C3r（根下 (号, 出生代)）K1 两腿都没打中，K3 被攻方「回退那次从 R_old 把历史带回来」与「旁证挤出环」打中，挂起；补法「随现行那一版走的谱系记录再核一遍」（C2ay 一类）零轮；V2 那一形两者全漏。这一轮复核出局、攻挂起的形态。不许重复第一轮攻过的角度（E175 V1 那两次、op92 / op95、K3 的 A、B 两种变异本身）；攻修补本身、产生结论的方法、没被看过的地方。

## 二、实现今天的样子（主 agent 的观测，2026-10-08）

与第一轮正文第二节相同；冻结树同一份 `~/.local/share/singlefs/work/m3-w3-r1/tree/`。方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　第一轮判决站不站得住（辩方）
- C1、只记峰值的 C2、每实例一行的 C2、C4 的出局：逐处判分不分辨臂、判别子看不看得到、打中的是判据字面哪一句；替出局的候选找最强形态量它。
- C2a 与 C3r 被 K3 打中那两形（回退从 R_old 带历史、旁证挤出环），是不是同样打中所有留下的候选。
- V2 那一形在不在 W3 的射程里：它是同一条时间线上的重发，还是第二种水位开关造出的、产品路径走不到的形态。

### K2　谱系核对的完整形态（攻方）
- C2a 与 C3r 各加「随现行那一版走的谱系记录核对」：给一个最强的定义（谱系记录今天在 D18 已定项 24 第 5 条，D8 已定项 6 引用），然后找一形漏报或误报：崩溃恢复抛弃一段建 inode 的发布再建（第一轮 K2 没有一次观测，要造出来）、清扫删实例表行、实例切换、管理员回退两次（A → B → A）。
- 每个形态每次发布多写的字节与单元、碰几刀写。

### K3　新不变量（攻方）
- 第一轮正推给的 I-9.30 草案（水位地板、出生代截断、历史盖住最新根）配上 K2 的形态，找一条它放过的变异。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | K1 | 替出局的候选辩护，复核第一轮判决 |
| 云端攻方（Opus） | K2、K3 | 攻挂起的两个形态加谱系核对 |

## 五、跑前写死的判据

- 辩方给出某个出局候选的一个形态，在第一轮打中它的那几格上不中（主 agent 现查），它回到候选集。
- 攻方坐实一形漏报或误报（主 agent 现查），那个形态收严或出局。
- 「没打中」一次抽样不算数。

交付：
- 腿的报告写进 `research/prompts/m3-w3-r2-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 模型放 `research/prompts/m3-w3-r2-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次，复跑脚本不靠自己在仓里的位置找文件（路径一律从环境变量或参数取）；仓路径从 `SINGLEFS_REPO` 取；编译与跑都经 `research/scripts/run-with-memory-cap.sh`（编译 6G、跑 4G）；同一时刻只起一条 cargo；用自己的编译目录，不沿用别的树编过的编译目录。只跑 harness 档。可以从第一轮的模型目录拷起；E175 装置源码原样在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/device-source.rs`。
- 引 kb 条款写 kb 文件自己的行号；引代码写冻结树里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。

### 小节清单：`.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E175 产品路径上 inode 号水位退回与重发号 —— 已跑（够判） | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E175 标题之下、第一个下级标题之前的正文：第 2-18 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 做了什么 | 抄 | 默认全抄（脚本生成） |
| ### 复跑 | 抄 | 默认全抄（脚本生成） |
| ### 产物清单 | 抄 | 默认全抄（脚本生成） |
| ### 读数（命令与原样输出） | 抄 | 默认全抄（脚本生成） |
| ### 判据逐条（登记第六节，每个量各报各的） | 抄 | 默认全抄（脚本生成） |
| ### 钉值（登记第七节，快档用例，三臂各跑一次） | 抄 | 默认全抄（脚本生成） |
| ### 判决行的点名 | 抄 | 默认全抄（脚本生成） |
| ### 单测与变异 | 抄 | 默认全抄（脚本生成） |
| ### 岔路表 | 抄 | 默认全抄（脚本生成） |
| ### 它答不了的 | 抄 | 默认全抄（脚本生成） |
| ### 路径与结论登记 | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |
| ### 2026-10-08：两段在冻结副本上重跑，W 行够判 | 不抄 | 历史节的子节 |
| ### 2026-10-08：第二段派发，crates/ 与第一段不同，停跑 | 不抄 | 历史节的子节 |
| ### 2026-10-08：建页，第一段已跑 | 不抄 | 历史节的子节 |

### 小节清单：`.claude/kb/decisions/18-块里携带什么信息.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D18 块里携带什么信息 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D18 标题之下、第一个下级标题之前的正文：第 2-4 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：谱系重写序号不做 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：全部码 2 节点无条件带 key 区间 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：数据块带明文自描述头 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：形态取元数据多副本 + 分散放置，链臂否掉，单盘布局线先不开 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：重建是降级路径，按见证者分三级 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：数据块自描述头放在低位，载荷从头之后开始 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：第一版单元头字段表，按单元类分 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：删除要写墓碑单元 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：「块」就是单元，一个单元一份头 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：墓碑取区间记录 + 打包共享单元 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：单元类型标签的值域是一张登记表，打包记录单元是独立的一类 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：码 2 的写序收窄成 4 字节，腾出的 6 字节给载荷 CRC 与预留 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：加密开启时码 1 的身份段与 `声明长度` 都进密文 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：nonce 代号就是完整的 96 位 nonce，住单元明文头 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：已撤回·此编号不再使用 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：nonce / MAC / 算法类型预留位算进头，声明长度的原点是含预留位的头末端 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 17：32 字节头校验和字段放 CRC32C 4 字节 + 28 字节零；自证结构的校验和覆盖整个槽含补齐 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 18：码 2 的声明长度 = 条目数 × 条目宽，补齐区恒 0，码 1 / 码 2 各补一张带偏移字段表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 19：字段值不值得带进块里：三条判据 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 20：已定要带的身份字段 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 21：自描述的射程是错向写，不是丢失写 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 22：要支持扫盘重建的信息必须显式写下，不折进校验和种子 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 23：否决清单 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 24：第一版墓碑：墓碑树、40 字节记录、谱系记录 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/decisions/08-核心索引结构.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D8 核心索引结构 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D8 标题之下、第一个下级标题之前的正文：第 2-4 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：记账取幂等完整值，接受 key 数量上升 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：节点大小 16 KiB，钉成常量 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：key 布局 —— 改名与索引分开处理 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：节点大小 —— 上限由恢复时间定，靠实验取值 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：whiteout 删除 —— 接受，但批量删除必须走 intent log | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：inode 树的叶是码 3 打包记录单元，记录定长 140，已发布的 (inode 号, 对象出生代) 不复用 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：write buffer 条目的 `seq` —— 4 字节，住 value | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：树表条目的字段表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：树的种类码的登记表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：第一版记账直落叶，`seq` 的取值规则 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：码 2 的载荷布局与 key 比较序 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：write buffer 前端与节点消息缓冲区的衔接 —— 一棵树只准一级 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：索引树更新做状态折叠 —— 同一次发布里同一个 key 的中间状态不落叶 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：一套 btree 实现 + 多个独立 keyspace + 两个前端，派生树可以异构 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：write buffer 条目的三条硬要求 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：第一版 unlink：只立 deleted_inodes、从尾部分批、挂载里续做到空 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/invariants.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 不变量清单 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （不变量清单 标题之下、第一个下级标题之前的正文：第 2-8 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ## 状态 | 抄 | 默认全抄（脚本生成） |
| ## I-1 块自描述 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-7 根环 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-8 journal —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-2 校验和 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-3 空间记账 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-4 事务与崩溃一致性 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-5 分配器 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-6 加密不变量 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## 待补 | 抄 | 默认全抄（脚本生成） |
| ## I-9 inode 树结构 —— 类 | 抄 | 默认全抄（脚本生成） |

**出处 `.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:1-18`（整段抄，未转述）**

```markdown
## E175 产品路径上 inode 号水位退回与重发号 —— 已跑（够判）

<!-- doc-lint:not-numbers W1 W2 W3 P1 P2 Q1 Q2 Q3 Q3b Q4 Q5 Q6 Q7 Q9 Q10 Q11 S1 S2 S3 V1 V2 V3 V4 V5 V717 A1 A2 A3 A4 A5 A6 A7 F1 F2 F3 F4 F5 F6 F7 F8 F9 F3a F3b F4 D1 D2 D3 G0 -->

跑前登记 `research/prompts/e175-preregistration.md`（照它跑两段时 sha256 `d68c9d2229bcf3800c5efd36c9194e6af9684b193a172258b58846474afae805`，问法定成读法 B：「重发号」只算同一条时间线上的；第十二节「修订」有两次：修订 1 是执行员写的 8 条读法钉死与装置写法，修订 2 是设计员写的「两段都在一份冻结副本上跑」与几何四点，判据、门槛、臂、量的去留都没动）。岔路单 `research/prompts/m3-3-5-forks.md` 第 11 行 W：C479（inode 号复用已发过的号今天造不出来） 剩下的「活记录被原地换」那一形，翻面观测是「产品路径上（不装只供测试的水位开关），有没有一段历史能让 inode 号水位退回去、造出重发号」，够判条件是在随机历史与带回退的历史上跑够登记的历史数、数到重发号与 I-9.29（同号只许一个出生代） 红的次数，有一次就够判。

**这一页的数全部出自仓副本装置，跑在一份冻结副本上**：装置照登记第五节「装置写在哪」写死，放在冻结副本 `~/.local/share/singlefs/work/e175-frozen-crates/repo/` 的 `crates/singlefs-harness/tests/e175_product_path_inode_watermark_regression.rs`，主工作区 `crates/` 不动（形式化线冻结期）；装置源码原样留在产物目录的 `device-source.rs`。冻结副本 `code_baseline=b000407ab5f4afdffe9083ddaac343fe8470438ea17891c4637b1684d5052b44`，拷于 2026-10-08，根环线第 10z 批第九块合入、验完之后（第八块合入之前）；HEAD `31e47115471ba5502a4531d25dffcc921be36899`（`baseline-frozen-crates.out` 第一行）。kb 别处引这些数要写明「仓副本装置、冻结副本」；装置要不要进 `crates/`、要不要在入库装置上重做一次，由主 agent 定。

**现状**：两段都在冻结副本上跑完：第一段（D 族钉值、F3a 定向穷举、F2 带回退的随机历史）与第二段（F1 四组现有随机历史、第八节几何四点 G-long、G-small、G-draw、G-long-F1），P、V1、V2 照登记各跑各的规模。第三段（F3b、F4）是附带，W 行够判之后没跑（够判后未跑）。

- **旧代码上的数**：产物目录下 `superseded-old-crates/` 是更早那份 `crates/`（清单 `superseded-old-crates/old-crates.sha`，336 行）上第一段的 19 份产物，照登记修订 2 第 4 条原样挪进去、不进 W 行判决、不进任何量的合计。新第一段九份结果文件的 E7RESULT 行（去掉 `elapsed_ns`、排序后）与旧的逐份相同，汇总除 `code_baseline` 外逐行相同（「读数」一节的比对命令）。

- **产品臂 P**：登记的 17 105 段历史（D 8、F3a 16 105、F2 256、F1 四组各 128、几何四点 64 + 64 + 64 + 32）里，同一条时间线上的重发号（Q1）0 次，I-9.29（同号只许一个出生代） 红 0 段（判不了 0 步；F1 四组与 G-long-F1 有不适用的步，F3a、F2、G-long、G-small、G-draw 每一步都判成立），水位退回（Q3）0 步，跨被抛弃时间线的重发号（Q3b）0 次；任何一条不变量在任何一步都没判红。
- **路径跑到了**：跨过已发号的回退之后同一条时间线上又建了 inode 的窗口（Q5）F3a 7 870 个、F2 1 062 个，合计 8 932，过了登记的 50；第二段另有 F1 四组 572 个、几何四点 1 196 个。F1 四组抽到 `PublishFirstFile`（1 号重发唯一的路）：F1-broad 197 次、F1-reuse 68 次、F1-deleting 12 次、F1-newest 197 次，G-long-F1 203 次。
- **阳性对照**：V1（回退取目标那一版的水位）与 V2（发布不加新建个数）在跑到的每一族都造出重发号、I-9.29（同号只许一个出生代） 红，R-V1 在 F3a、F2、F1 四组、G-long 上全过，R-V2 在 F3a、F2、F1 四组上全过（几何点照登记只跑 G-long 的 V1），登记第十节 F1、F2 没触发。
- **停机条款**：S3 两条路判抛弃不一致 0 次、S1 与 S2 对不上（F3 两形）0 次、Q11（发的号不等于上一版水位）0 次、条款锚点 A1–A3 与独立锚点 A4–A7 全对上，都没触发。
- **W 行**：照登记第十一节「第二段跑完、F1 F2 F7 F8 都没触发，W 行够判」：登记的 17 105 段产品臂历史里没造出同一条时间线上的重发号，也没有一段 I-9.29（同号只许一个出生代） 红。这是「登记的历史里没造出」，不是「造不出」。

```

**出处 `.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:19-25`（整段抄，未转述）**

```markdown
### 做了什么

- **三样读法**（登记第六节）：S1 账：每一步之后用 `singlefs_core::recovery::readable_roots` 列可读根，同一根键只用 `rebuild_version` 重建一次，读那一版记账里的水位与 inode 树全部 (号, 出生代)，按号记「见过的出生代 → 持有过它的根」；S2：每一步对镜像调 `singlefs_checker::walk::check_pool_image`，判红不停，I-9.29（同号只许一个出生代）、I-9.6（水位大于两处最大号） 按成立 / 不适用 / 部分没判 / 违例四栏数；S3：被崩溃恢复抛弃的根两条路，路 1 用最新那条根的实例表按 `publish_is_abandoned_by_the_instance_table` 判，路 2 是步账（每次挂载做成，新实例接的是那一步之前模型根环里最新的那条根，同一旧实例里 txg 更大的根记被抛弃）。
- **臂**：P（冻结副本 `crates/` 原样，与拷它那一刻的主工作区逐字节相同：`baseline-frozen-crates.out` 的 `main_before_copy_equal=true main_after_copy_equal=true`）、V1（`crates/singlefs-core/src/mount.rs` 第 7758 行回退那次的水位换成目标那一版的）、V2（`crates/singlefs-core/src/transaction.rs` 第 8035 行 `} + new_inodes_of_this_publish;` 换成 `} + 0;`）；V717（`crates/mutations.tsv` 第 717 行，名字「实三（D23 已定项 14「水位」）：回退那次的 inode 号水位只取环里读得出的根，不与内存里现行那一版的取 max」）登记写明 Q6 > 0 才跑，两段各族 Q6 都是 0，记「与 P 同输出（由 Q6 = 0 推出），未跑」。三处锚点的行号与命中数在 `baseline-frozen-crates.out` 的 `name=mutation_anchor` 三行；补丁原样在 `patch-V1.diff`、`patch-V2.diff`。每臂施加前与还原后在副本根对 `baseline-frozen-crates.sha` 核一次（`name=arm_check` 九行，`listed_mismatch=0 extra_files=0`）；三臂二进制各编一次，两段共用（`name=arm_binary` 三行）。
- **执行**：`HistoryExecution { per_step_checker: Skipped, device_width: FourGibibytes（G-small 换 UnitAreaOf384Slots）, space_admission: JudgedByTheFormula }`，不调 `set_inode_number_watermark_source`、不加 `SharedFaultPlan`；7 个工作线程（派发提示的线程上限）按序号递增领活，合并按序号次序，与单线程逐字相同（快档 `one_thread_and_three_threads_give_the_same_lines`）；每段历史的挂钟在装置进程里自己计（结果行 `elapsed_ns`），被打断再跑同一条命令就续跑，跳过已有结果行的序号。每份结果文件第一行是 `# code_baseline=…` 文件头，续写时文件头不同就拒，汇总要求全部输入同一个 `code_baseline`、每一行汇总行带它（登记修订 2 第 2 条）。
- **试跑**（登记修订 2 第 5 条）：F2 P 前 8 个种子 488 步，平均 223.63 毫秒 / 步 / 线程，外推第一段约 31 分钟；G-long P 前 4 个种子 604 步，248.38 毫秒 / 步，外推第二段约 30 分钟；都不到 2 小时，不分片（`trial-f2-P-first-8-seeds.out`、`trial-g-long-P-first-4-seeds.out`，按结果行 `elapsed_ns` 之和除以 `steps_observed` 之和）。

```

**出处 `.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:26-45`（整段抄，未转述）**

````markdown
### 复跑

```
# 冻结副本（拷 Cargo.toml、Cargo.lock、.cargo、crates 四样；主工作区 crates/ 不动），核清单再放装置
F=~/.local/share/singlefs/work/e175-frozen-crates/repo; R=$PWD/research/results/e175-product-path-inode-watermark-regression-2026-10-08
(cd $F && sha256sum -c --quiet $R/baseline-frozen-crates.sha)
cp $R/device-source.rs $F/crates/singlefs-harness/tests/e175_product_path_inode_watermark_regression.rs
# 每一臂：施加补丁（P 不施加；V1、V2 照 $R/patch-V1.diff、$R/patch-V2.diff）、编、拷出二进制、还原，再核一次清单
ROOT=$PWD; (cd $F && CARGO_TARGET_DIR=<编译目录> bash $ROOT/research/scripts/capped.sh 7 cargo test --release --offline -p singlefs-harness --test e175_product_path_inode_watermark_regression --no-run)
E175_ARM=P bash research/scripts/capped.sh 7 bash research/scripts/run-with-memory-cap.sh 8G <测试二进制> --test-threads 7
E175_ARM=P E175_FAMILY=F2 E175_RESULT_FILE=$R/f2-P.out E175_CODE_BASELINE=b000407ab5f4afdffe9083ddaac343fe8470438ea17891c4637b1684d5052b44 E175_WORKER_THREADS=7 E175_TIME_BUDGET_SECONDS=170 \
  bash research/scripts/capped.sh 7 bash research/scripts/run-with-memory-cap.sh 8G <测试二进制> --ignored --exact run_one_family_on_one_arm_into_the_result_file
# 族名：D、F3a、F2、F1-broad、F1-reuse-after-raising-the-floor、F1-deleting-files-of-several-lower-leaves、
#       F1-newest-system-configuration-slot-reading-zeros、G-long、G-small、G-draw、G-long-F1
# 汇总（两段各一份）
E175_SUMMARY_INPUTS=<逗号分隔的结果文件> E175_SUMMARY_FILE=<新文件> E175_CODE_BASELINE=b000407ab5f4afdffe9083ddaac343fe8470438ea17891c4637b1684d5052b44 <测试二进制> --ignored --exact summarize_the_result_files
# 登记的复跑（research/scripts/replay.sh 一行 E175|@driver_e175：拷今天的 crates/ 进副本、放进 device-source.rs、跑 D 族 P 臂）
bash research/scripts/replay.sh E175
```

````

**出处 `.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:46-71`（整段抄，未转述）**

```markdown
### 产物清单

目录 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/` 顶层 37 份（`ls -1 | grep -v superseded | wc -l` 输出 37），全部是冻结副本 `code_baseline=b000407ab5f4afdffe9083ddaac343fe8470438ea17891c4637b1684d5052b44`（拷于 2026-10-08，根环线第 10z 批第九块合入之后）上的；另有子目录 `superseded-old-crates/` 20 份（旧代码上的数，不进判决）：

| 文件 | 是什么 |
|---|---|
| `e175-product-path-inode-watermark-regression-2026-10-08/device-source.rs` | 装置源码原样（sha256 `310c60a51a7a0c5cc015b070006d7d48ae0eb09d8a581ec8acba9d6bfc185fca`，两段同一份） |
| `e175-product-path-inode-watermark-regression-2026-10-08/baseline-frozen-crates.sha` | 冻结副本放装置之前的清单（340 行，路径相对副本根；sha256 `b000407ab5f4afdffe9083ddaac343fe8470438ea17891c4637b1684d5052b44`，就是 `code_baseline`） |
| `e175-product-path-inode-watermark-regression-2026-10-08/baseline-frozen-crates.out` | `name=frozen_copy`（拷的日期、批次、HEAD、两行 diff / status 的 sha256、文件数、主工作区拷前拷后清单都与副本相同）、`name=mutation_anchor` 三行、`name=arm_check` 九行、`name=arm_binary` 三行、`name=main_drift_after_copy` 一行 |
| `e175-product-path-inode-watermark-regression-2026-10-08/patch-V1.diff`、`e175-product-path-inode-watermark-regression-2026-10-08/patch-V2.diff` | V1、V2 两臂施加的补丁 |
| `e175-product-path-inode-watermark-regression-2026-10-08/quick-tests.out` | 三臂快档与三个弄坏开关的输出 |
| `e175-product-path-inode-watermark-regression-2026-10-08/trial-f2-P-first-8-seeds.out`、`e175-product-path-inode-watermark-regression-2026-10-08/trial-g-long-P-first-4-seeds.out` | 外推挂钟的两次试跑 |
| `e175-product-path-inode-watermark-regression-2026-10-08/d-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/d-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/d-V2.out` | D 族钉值，三臂 |
| `e175-product-path-inode-watermark-regression-2026-10-08-d-P-replay.out` | `driver_e175` 形态的输出（冻结副本上 D 族 P 臂 4 线程再跑一次，E7RESULT 行加收尾行），`replay.sh` 比它；去掉 `elapsed_ns` 与 `d-P.out` 的 E7RESULT 行逐行相同 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f3a-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f3a-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f3a-V2.out` | F3a，三臂 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f2-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f2-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f2-V2.out` | F2，三臂 |
| `e175-product-path-inode-watermark-regression-2026-10-08/summary-segment-one.out` | 第一段汇总：九份族 × 臂结果文件的合计、重复与冲突、事件计数 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f1-broad-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-broad-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-broad-V2.out` | F1 `BROAD`，三臂 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f1-reuse-after-raising-the-floor-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-reuse-after-raising-the-floor-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-reuse-after-raising-the-floor-V2.out` | F1 `REUSE_AFTER_RAISING_THE_FLOOR`，三臂 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f1-deleting-files-of-several-lower-leaves-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-deleting-files-of-several-lower-leaves-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-deleting-files-of-several-lower-leaves-V2.out` | F1 `DELETING_FILES_OF_SEVERAL_LOWER_LEAVES`，三臂 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f1-newest-system-configuration-slot-reading-zeros-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-newest-system-configuration-slot-reading-zeros-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-newest-system-configuration-slot-reading-zeros-V2.out` | F1 `NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS`，三臂 |
| `e175-product-path-inode-watermark-regression-2026-10-08/g-long-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/g-long-V1.out` | 几何点 G-long（F2 比重，150 步），P 与 V1 |
| `e175-product-path-inode-watermark-regression-2026-10-08/g-small-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/g-draw-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/g-long-f1-P.out` | 几何点 G-small（`UnitAreaOf384Slots`）、G-draw（`RecentUniformOrBeyond`）、G-long-F1（`BROAD`，150 步），P |
| `e175-product-path-inode-watermark-regression-2026-10-08/summary-segment-two.out` | 第二段汇总：十七份族 × 臂结果文件 |
| `e175-product-path-inode-watermark-regression-2026-10-08/superseded-old-crates/` | 旧代码上的第一段 19 份（原名原样，与顶层同名的 16 份之外是 `superseded-old-crates/baseline.out`、`superseded-old-crates/baseline-segment-two.out`、`superseded-old-crates/trial-f3a-P-8-long-sequences.out`）与那份 `crates/` 的清单 `superseded-old-crates/old-crates.sha`；挪之前、挪之后 19 份的 sha256 逐行相同 |

```

**出处 `.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:72-125`（整段抄，未转述）**

````markdown
### 读数（命令与原样输出）

在产物目录里跑，从两份汇总取每个族 × 臂的量（`sum_` 是各段相加，`segments_q2_one_birth_violated` 是 I-9.29（同号只许一个出生代） 红过的段数，`sum_i9_29_holds`、`sum_i9_29_not_applicable`、`sum_i9_29_partially_judged` 是 I-9.29 判成立、不适用、判不了的步数）：

```
for s in one two; do echo "== $s"; head -1 summary-segment-$s.out; awk '/name=summary/{o=""; for(i=1;i<=NF;i++){split($i,a,"="); if(a[1]~/^(family|arm|histories|complete|duplicates|conflicts|ending_completed|sum_q1|sum_q3b|segments_q2_one_birth_violated|sum_i9_29_holds|sum_i9_29_not_applicable|sum_i9_29_partially_judged|sum_q9_still|sum_q9_gone|sum_q3_steps|max_q3_peak|sum_q4|sum_q5|sum_q6|sum_q7|q10_segments_issuing_at_least_two|sum_q11|sum_s1_unreadable|sum_s3_disagreements|sum_f3_still_but_holds|sum_f3_violated_without_two_births)$/) o=o" "$i}; print substr(o,2)}' summary-segment-$s.out; done
```

```
== one
# code_baseline=b000407ab5f4afdffe9083ddaac343fe8470438ea17891c4637b1684d5052b44
family=D arm=P histories=8 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=7 ending_completed=8 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=45 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=4 sum_q5=3 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=D arm=V1 histories=8 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=7 ending_completed=8 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=42 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=3 sum_q11=0 segments_q2_one_birth_violated=3 sum_q3_steps=7 sum_q3b=0 sum_q4=4 sum_q5=3 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=3 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=2
family=D arm=V2 histories=8 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=7 ending_completed=8 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=23 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=13 sum_q11=0 segments_q2_one_birth_violated=7 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=13 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F2 arm=P histories=256 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=231 ending_completed=256 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=15616 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=1748 sum_q5=1062 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F2 arm=V1 histories=64 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=53 ending_completed=64 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=1769 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=336 sum_q11=0 segments_q2_one_birth_violated=53 sum_q3_steps=1476 sum_q3b=0 sum_q4=311 sum_q5=195 sum_q6=0 sum_q7=280 sum_q9_gone=5 sum_q9_still=331 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=7
family=F2 arm=V2 histories=64 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=53 ending_completed=64 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=1265 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=696 sum_q11=0 segments_q2_one_birth_violated=53 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=696 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F3a arm=P histories=16105 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=16105 ending_completed=16105 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=159440 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=11641 sum_q5=7870 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F3a arm=V1 histories=1464 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=1464 ending_completed=1464 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=11788 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=975 sum_q11=0 segments_q2_one_birth_violated=567 sum_q3_steps=2243 sum_q3b=0 sum_q4=798 sum_q5=577 sum_q6=0 sum_q7=168 sum_q9_gone=0 sum_q9_still=975 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=4
family=F3a arm=V2 histories=1464 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=1464 ending_completed=1464 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=4392 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=3990 sum_q11=0 segments_q2_one_birth_violated=1464 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=3990 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
== two
# code_baseline=b000407ab5f4afdffe9083ddaac343fe8470438ea17891c4637b1684d5052b44
family=F1-broad arm=P histories=128 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=49 ending_completed=128 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3638 sum_i9_29_not_applicable=330 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=39 sum_q5=13 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-broad arm=V1 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=15 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=821 sum_i9_29_not_applicable=124 sum_i9_29_partially_judged=0 sum_q1=12 sum_q11=0 segments_q2_one_birth_violated=4 sum_q3_steps=43 sum_q3b=0 sum_q4=11 sum_q5=4 sum_q6=0 sum_q7=9 sum_q9_gone=0 sum_q9_still=12 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=5
family=F1-broad arm=V2 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=15 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=637 sum_i9_29_not_applicable=124 sum_i9_29_partially_judged=0 sum_q1=90 sum_q11=0 segments_q2_one_birth_violated=15 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=90 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-deleting-files-of-several-lower-leaves arm=P histories=128 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=110 ending_completed=128 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3910 sum_i9_29_not_applicable=58 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=615 sum_q5=526 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-deleting-files-of-several-lower-leaves arm=V1 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=27 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=556 sum_i9_29_not_applicable=10 sum_i9_29_partially_judged=0 sum_q1=103 sum_q11=0 segments_q2_one_birth_violated=26 sum_q3_steps=327 sum_q3b=0 sum_q4=97 sum_q5=86 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=103 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=12
family=F1-deleting-files-of-several-lower-leaves arm=V2 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=27 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=433 sum_i9_29_not_applicable=10 sum_i9_29_partially_judged=0 sum_q1=244 sum_q11=0 segments_q2_one_birth_violated=27 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=244 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-newest-system-configuration-slot-reading-zeros arm=P histories=128 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=57 ending_completed=128 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3638 sum_i9_29_not_applicable=330 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=52 sum_q5=18 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-newest-system-configuration-slot-reading-zeros arm=V1 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=17 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=806 sum_i9_29_not_applicable=124 sum_i9_29_partially_judged=0 sum_q1=16 sum_q11=0 segments_q2_one_birth_violated=6 sum_q3_steps=52 sum_q3b=0 sum_q4=14 sum_q5=6 sum_q6=0 sum_q7=14 sum_q9_gone=0 sum_q9_still=16 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=5
family=F1-newest-system-configuration-slot-reading-zeros arm=V2 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=17 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=613 sum_i9_29_not_applicable=124 sum_i9_29_partially_judged=0 sum_q1=110 sum_q11=0 segments_q2_one_birth_violated=17 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=110 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-reuse-after-raising-the-floor arm=P histories=128 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=67 ending_completed=128 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3917 sum_i9_29_not_applicable=51 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=32 sum_q5=15 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-reuse-after-raising-the-floor arm=V1 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=15 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=896 sum_i9_29_not_applicable=10 sum_i9_29_partially_judged=0 sum_q1=7 sum_q11=0 segments_q2_one_birth_violated=6 sum_q3_steps=18 sum_q3b=0 sum_q4=7 sum_q5=6 sum_q6=0 sum_q7=3 sum_q9_gone=0 sum_q9_still=7 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=7
family=F1-reuse-after-raising-the-floor arm=V2 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=15 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=806 sum_i9_29_not_applicable=10 sum_i9_29_partially_judged=0 sum_q1=82 sum_q11=0 segments_q2_one_birth_violated=15 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=82 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=G-draw arm=P histories=64 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=56 ending_completed=64 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3904 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=334 sum_q5=191 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=G-long arm=P histories=64 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=53 ending_completed=64 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=9664 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=1045 sum_q5=661 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=G-long arm=V1 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=29 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=1411 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=476 sum_q11=0 segments_q2_one_birth_violated=29 sum_q3_steps=1943 sum_q3b=0 sum_q4=420 sum_q5=275 sum_q6=0 sum_q7=345 sum_q9_gone=6 sum_q9_still=470 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=9
family=G-long-F1 arm=P histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=25 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=4708 sum_i9_29_not_applicable=124 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=188 sum_q5=109 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=G-small arm=P histories=64 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=53 ending_completed=64 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3904 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=397 sum_q5=235 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
```

与旧代码上第一段的比对（新旧九份结果文件的 E7RESULT 行去掉 `elapsed_ns`、排序后取 sha256 前 16 位；汇总去掉 `code_baseline` 后比）：

```
for f in d-P d-V1 d-V2 f3a-P f3a-V1 f3a-V2 f2-P f2-V1 f2-V2; do a=$(grep '^E7RESULT' superseded-old-crates/$f.out | sed 's/ elapsed_ns=[0-9]*//' | sort | sha256sum | cut -c1-16); b=$(grep '^E7RESULT' $f.out | sed 's/ elapsed_ns=[0-9]*//' | sort | sha256sum | cut -c1-16); echo "$f old=$a new=$b $([ $a = $b ] && echo same || echo DIFFERENT)"; done
diff <(grep -v '^#' superseded-old-crates/summary-segment-one.out) <(grep -v '^#' summary-segment-one.out | sed 's/ code_baseline=[0-9a-f]*//') && echo summary-same-apart-from-code-baseline
```

九行都是 `same`（例 `f3a-P old=40278af11b04d80e new=40278af11b04d80e same`），末行 `summary-same-apart-from-code-baseline`。

产物齐不齐（逐份结果文件数结果行、带 `line_end=ok` 的、带臂字段的、事件行、`first_violation` 不是 `none` 的段、完成标记，每份一条 `awk '/^E7RESULT name=history /{n++; if($0 ~ / line_end=ok$/)ok++; if($0 ~ / arm=/)arm++; for(i=1;i<=NF;i++){split($i,a,"="); if(a[1]=="first_violation" && a[2]!="none") fv++}} /^E7RESULT name=event /{ev++} /name=family_complete/{c++} NR==1{head=$0} END{print head, n+0, ok+0, arm+0, ev+0, fv+0, c+0}' <文件>`）：二十六份结果文件第一行都是 `# code_baseline=b000407ab5f4afdffe9083ddaac343fe8470438ea17891c4637b1684d5052b44`，各自的结果行数等于登记的段数（D 8、F3a P 16 105、F3a V 1 464、F2 P 256、F2 V 64、F1 P 每组 128、F1 V 每组 32、G-long P 64、G-long V1 32、G-small P 64、G-draw P 64、G-long-F1 P 32），每一行都带 `arm=` 与 `line_end=ok`，都有 `name=family_complete`；P 臂十二份的事件行 0 条、`first_violation` 不是 `none` 的段 0 段。两份汇总的 `unfinished_lines count=0`。F3a P 的操作步：`awk '/^E7RESULT name=history /{n++; for(i=1;i<=NF;i++){split($i,a,"="); if(a[1]=="steps_observed")s+=a[2]}} END{print "f3a-P histories=" n " operation_steps=" s-n}' f3a-P.out` 输出 `f3a-P histories=16105 operation_steps=143335`。

`PublishFirstFile`（1 号重发唯一的路）抽到几次：`for f in f1-broad-P f1-reuse-after-raising-the-floor-P f1-deleting-files-of-several-lower-leaves-P f1-newest-system-configuration-slot-reading-zeros-P g-long-f1-P f2-P; do echo "$f first=$(grep -h '^E7RESULT name=history' $f.out | grep -o '[[,]first(' | wc -l)"; done` 输出 `f1-broad-P first=197`、`f1-reuse-after-raising-the-floor-P first=68`、`f1-deleting-files-of-several-lower-leaves-P first=12`、`f1-newest-system-configuration-slot-reading-zeros-P first=197`、`g-long-f1-P first=203`、`f2-P first=0`（F2 这一组比重从 `AfterFirstFile` 起、unlink 之后那张表不抽它）。

````

**出处 `.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:126-154`（整段抄，未转述）**

```markdown
### 判据逐条（登记第六节，每个量各报各的）

F1 四组按 `BROAD` / `REUSE_AFTER_RAISING_THE_FLOOR` / `DELETING_FILES_OF_SEVERAL_LOWER_LEAVES` / `NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS` 的次序列，几何四点按 G-long / G-small / G-draw / G-long-F1 的次序列；数都出自「读数」一节两份汇总。

| 量 | F3a | F2 | F1 四组 | 几何四点 | 判定 |
|---|---|---|---|---|---|
| Q1 同一时间线重发号（P） | 0 | 0 | 0 / 0 / 0 / 0 | 0 / 0 / 0 / 0 | 没有 |
| Q2 I-9.29（同号只许一个出生代） 红（P，段数） | 0（成立 159 440 步，不适用、判不了 0 步） | 0（成立 15 616 步） | 0（成立 3 638 / 3 917 / 3 910 / 3 638 步，不适用 330 / 51 / 58 / 330 步，判不了 0 步） | 0（成立 9 664 / 3 904 / 3 904 / 4 708 步，G-long-F1 不适用 124 步，判不了 0 步） | 没有 |
| Q9（P 有 Q1 事件才有） | 无事件 | 无事件 | 无事件 | 无事件 | 不适用 |
| Q3 水位退回（P，步数；峰值与期末） | 0，峰值 0、期末 0 | 0，峰值 0、期末 0 | 0，峰值 0、期末 0 | 0，峰值 0、期末 0 | 没有 |
| Q3b 跨被抛弃时间线重发号（P） | 0 | 0 | 0 | 0 | 没有（附带） |
| Q4 回退跨过发号（P） | 11 641 | 1 748 | 39 / 32 / 615 / 52 | 1 045 / 397 / 334 / 188 | 报数 |
| Q5 跨过发号的回退之后又建 inode 的窗口（P） | 7 870 | 1 062 | 13 / 15 / 526 / 18 | 661 / 235 / 191 / 109 | F2 + F3a 合计 8 932 ≥ 50，够 |
| Q6 回退时环里可读根的水位最大值 < 现行（P） | 0（判了 17 954 次） | 0（判了 2 951 次） | 0（判了 185 / 74 / 733 / 224 次） | 0（判了 1 984 / 714 / 612 / 407 次） | V717 不跑 |
| Q7 挂载 / 恢复落到水位更低的根（P） | 0 | 0 | 0 | 0 | 报数 |
| Q10 一条时间线发过 ≥ 2 个号的段（各臂） | P 16 105、V1 1 464、V2 1 464 | P 231、V1 53、V2 53 | P 49 / 67 / 110 / 57，V1、V2 各 15 / 15 / 27 / 17 | P 53 / 53 / 56 / 25，G-long V1 29 | V2 跑到的每一族都必须红 |
| Q11 发号与上一版水位不等（各臂） | 0 | 0 | 0 | 0 | 一致 |
| R-V1（V1：Q5 ≥ 1 时 Q1 ≥ 1 且 Q2 ≥ 1 段） | Q5 577、Q1 975、Q2 567 段 | Q5 195、Q1 336、Q2 53 段 | Q5 4 / 6 / 86 / 6，Q1 12 / 7 / 103 / 16，Q2 4 / 6 / 26 / 6 段 | G-long：Q5 275、Q1 476、Q2 29 段 | 跑到的每一族都过 |
| R-V2（V2：Q10 ≥ 1 时 Q1 ≥ 1） | Q1 3 990 | Q1 696 | Q1 90 / 82 / 244 / 110 | 登记不跑 | 跑到的每一族都过 |
| R-V717 | 未跑 | 未跑 | 未跑 | 未跑 | 与 P 同输出（由 Q6 = 0 推出），未跑 |

几何敏感性（登记第八节）：G-long、G-small、G-draw、G-long-F1 四点各与 G0（F2，第一段）比，P 臂 Q1、Q2、Q9 的判定都是「没有」，不翻面；两点同值 0，门槛挪不进两点之间，判别力由 R-V1、R-V2 担（G-long 的 V1 臂 Q1 476、Q2 29 段）。

失败与停机条款（登记第十、十一节）：F1、F2 没触发（上表 R-V1、R-V2）；F3 两形 0（`sum_f3_still_but_holds`、`sum_f3_violated_without_two_births` 二十六份全 0）；F4、F5 没触发（下一节）；F6、F9 只在 P 有 Q1 事件时做，没有事件、没做；F7 没触发（F2 + F3a 的 Q5 合计 8 932）；F8 没触发（S1 重建报错 0 次，P 臂看到根的次数 F3a 1 431 682、F2 304 853、F1 四组 54 585 / 67 690 / 57 613 / 60 489、几何四点 213 378 / 75 155 / 75 554 / 104 381，出自两份汇总的 `sum_s1_root_observations`）；S3 两路不一致 0 次、读不出实例表的步 0 步；合并不确定没触发（二十六份 `conflicts=0`、`duplicates=0`，快档多线程与单线程对拍过）。

V1、V2 上全部 Q1 事件按 Q9 分类与那一步 I-9.29（同号只许一个出生代） 的判定交叉数：`grep -h '^E7RESULT name=event ' d-*.out f3a-*.out f2-*.out f1-*.out g-*.out | grep -o 'arm=[A-Z0-9]* .*kind=[a-z0-9_]* .*q9=[A-Za-z]* i9_29_at_step=[a-z_]*' | sed -E 's/index=[0-9]+ //; s/position=[^ ]* inode=[^ ]* new_birth=[^ ]* older_births=[^ ]* //' | awk '{print $1, $2, $3, $4}' | sort | uniq -c` 输出四行：`2 arm=V1 kind=q1_same_timeline q9=Gone i9_29_at_step=holds`、`9 arm=V1 kind=q1_same_timeline q9=Gone i9_29_at_step=violated`、`1917 arm=V1 kind=q1_same_timeline q9=Still i9_29_at_step=violated`、`5225 arm=V2 kind=q1_same_timeline q9=Still i9_29_at_step=violated`。判「还在」的事件那一步 I-9.29 全是违例（F3 第一形 0）。

阳性对照臂上「活记录被原地换」那一形（Q9「已转出」）出现 11 次：`grep -n 'q9=Gone' f2-V1.out g-long-V1.out` 命中 `f2-V1.out` 第 67、284、326、327、386 行与 `g-long-V1.out` 第 40、71、155、358、389、390 行。其中 `g-long-V1.out` 第 389、390 行（G-long V1 序号 24，第 92、95 步发号 7、8）那两步 I-9.29（同号只许一个出生代） 判成立：旧出生代已转出候选集、同一步候选集里也没有别的号有两个出生代，I-9.29 看不见那次重发——C479（inode 号复用已发过的号今天造不出来） 那一行写的「被换之前那一版转出候选集之后看不见」在 V1 臂上出现了；其余 9 次那一步另有号的两个出生代、I-9.29 判违例。P 臂上这一形 0 次。

```

**出处 `.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:155-162`（整段抄，未转述）**

```markdown
### 钉值（登记第七节，快档用例，三臂各跑一次）

- A1、A2（D1，起点）：`first_file_alone_reads_inode_one_born_at_three_and_watermark_two` 三臂都过，现行那一版 inode 树 {(1, 3)}、水位 2，与 D8（核心索引结构） 已定项 6「新池新建文件写」相同。
- A3、A4（D2(k)、D3，P）：用例 `rollback_one_step_then_create_matches_the_arm` 等六条与 `creations_across_mounts_and_a_cold_start_match_the_arm` 在 P 臂上过（`quick-tests.out` 的 `===== q-P exit=0` 那一段，`test result: ok. 26 passed; 0 failed; 2 ignored`），断言 D2(1)–D2(3) 依次发 2、3、4，D3 发 2、3、4。`d-P.out` 的结果行与之相符：D2(1)、D2(2)、D2(3) 各 `issuances=4 watermark_final=5 rollbacks_applied=1`（起点 1 号加三次建）；D2(4) `issuances=3 watermark_final=4 rollbacks_applied=1`（回退到第一个文件还活着的那一版，之后建 inode 不调）；D2(5)、D2(6) `rollbacks_applied=0`，A3 那三格没有样本（修订 1 第 5 条）；D3 `issuances=4 watermark_final=5`。这一次快档没带 `--nocapture`，用例里的打印行没进 `quick-tests.out`。
- A5（D2(1)，V1）：发 3、Q1 恰好 1、Q9「还在」、那一步 I-9.29（同号只许一个出生代） 违例，过；k = 2、3 上发的号 = S1 读到的目标那一版水位 2，过。
- A6（D2(1)，V2）：前两次都发 2、第一次建之后那一步 I-9.6（水位大于两处最大号） 违例、第二次建那一步 Q1 恰好 1，过。
- A7：`f3a_scale_matches_the_registered_counts` 过（16 105、1 464、143 335），产物里数出来的段数与步数也是这三个数；第二段规模 `second_segment_scales_match_the_registered_counts` 过（F1 P 15 360 步、F1 V 3 840 步、几何四点 P 22 080 步、G-long V1 4 800 步），产物里各族段数与登记相同（「读数」一节的 `histories=`）。

```

**出处 `.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:163-166`（整段抄，未转述）**

```markdown
### 判决行的点名

装置不写 `name=verdict` 行：在产物目录顶层 `grep -n 'name=verdict' $(ls -1 | grep -v superseded) | wc -l` 输出 0。每个量各报各的数（「读数」一节），判定按登记第六节逐条写在「判据逐条」一节；不为 0 的「没过」类计数只在阳性对照臂上，都是登记预期内的：V1、V2 两臂 Q1、Q2 不为 0（阳性对照本该红），V1 臂 Q3、Q7 不为 0（回退取目标那一版的水位，水位就会退回去）。P 臂上 Q1、Q2、Q3、Q3b、Q7、Q11、S3 不一致、F3 两形、S1 重建报错都是 0。

```

**出处 `.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:167-172`（整段抄，未转述）**

```markdown
### 单测与变异

单测 28 条：`grep -c '^#\[test\]' device-source.rs` 输出 28，其中 2 条带 `#[ignore]`（大规模入口与汇总）；`quick-tests.out` 里三臂各 `test result: ok. 26 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out`。比第一段那一版多的 11 条：第二段规模与比重 2 条、每个新族序号 0 走一遍 8 条、汇总拒收两份代码混用或缺文件头的结果文件 1 条（`result_files_from_two_code_baselines_or_without_a_header_are_refused_by_the_summary`）。

变异（登记第九节）6 条：V1、V2 是两条阳性对照臂，跑到的每一族都分得出（「判据逐条」一节 R-V1、R-V2 过）；V3、V4、V5 是装置里的弄坏开关，`quick-tests.out` 里各红在点名的那一条（三次都带 `--skip first_history_of`，跳过 8 条每族序号 0 的取样用例，`17 passed; 1 failed; 2 ignored; 0 measured; 8 filtered out`）：V3（账按号去重）与 V4（发号事件分类时每条根都判被抛弃）在 V1 臂上都红在 `rollback_one_step_then_create_matches_the_arm` 的「A5：Q1 恰好 1」（`left: 0`、`right: 1`；V4 那一次同一段打印 `q1=0 q3b=1`），V5（Q9 一律判「还在」）在 P 臂上红在 `q9_classification_says_gone_when_no_holder_is_readable`；V717 照登记有条件，Q6 = 0，没跑。抓到 5、无效 0、没红 0、未跑 1。装置不在 `research/` 下、也不进 `crates/mutations.tsv`，没有变异表，`mutate.sh` 不适用。装置文件头弄坏开关那张表给 V3 写的用例名 `rollback_one_step_then_create_on_v1_reissues_three_on_the_same_timeline` 与实际红的 `rollback_one_step_then_create_matches_the_arm` 不同（文件头的字没跟用例改名一起改），开关红在的是后者。

```

**出处 `.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:173-178`（整段抄，未转述）**

```markdown
### 岔路表

| 岔路行 | 状态 | 依据 | 还差什么，剩下的量能不能让它翻面 |
|---|---|---|---|
| W：C479（inode 号复用已发过的号今天造不出来） 剩下的「活记录被原地换」那一形 | 已够判（冻结副本 `code_baseline=b000407ab5f4afdffe9083ddaac343fe8470438ea17891c4637b1684d5052b44`） | 「读数」一节两份汇总：P 臂带回退的历史（F3a、F2）与随机历史（F1 四组）、几何四点 Q1 全 0、`segments_q2_one_birth_violated` 全 0；F2 + F3a 的 Q5 合计 8 932 ≥ 50；R-V1、R-V2 在跑到的每一族都过；F1、F2、F7、F8 没触发 | 不差。登记的 17 105 段产品臂历史里没造出同一条时间线上的重发号，I-9.29（同号只许一个出生代） 红 0 段，水位退回 0 步。剩下的量是第三段附带的 F3b（崩溃恢复抛弃根之后再建）、F4（写 / 刷盘故障注入之后再建），够判后未跑；它们不在 W 行够判条件点名的两类历史里，但能让「水位退回」那半句翻面：崩溃恢复落到较旧的根之后水位随所选根退回，那是条款允许的跨被抛弃时间线那一形（Q3b），按读法 B 不翻 W 行；若在那两族里量出同一条时间线上的 Q1 ≥ 1，W 行翻面 |

```

**出处 `.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:179-187`（整段抄，未转述）**

```markdown
### 它答不了的

- 「登记的历史里没造出」，不是「造不出」：P 臂 17 105 段是登记的取样（F3a 中段长到 4 的穷举、F2 / F1 / 几何点按种子随机），长度更长、回退目标或操作次序没被抽到的历史没有数。
- 崩溃恢复抛弃根（双故障形，F3b）与写 / 刷盘故障注入后建 inode（F4）是登记第三段附带，够判后未跑；历史里抽到的 `CrashRecoveryAbandoningTheNewestRoot` 在 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙之后挂载被拒、不造被抛弃根，Q3b 的 0 不说明跨被抛弃时间线的重发不存在。装置没有数「判成被抛弃的根」的次数，两段里 S3 有没有判出过一条被抛弃根，页上不下结论。
- Q6 的读法有一处局限：现行那一版取自这一步可读根里最新的那条，回退之前它自己就在环里读得出，Q6 > 0 只可能在现行那一版自己重建不出时出现；无读故障的历史里 Q6 = 0 由这个读法推得出，不是独立的观测。V717 因此在每一族都没跑，「回退取 max」那条护栏在这些历史里有没有被走到，这一页答不了。
- 「活记录被原地换」那一形 I-9.29（同号只许一个出生代） 看不见：阳性对照 V1 臂 G-long 序号 24 的两次重发（`g-long-V1.out` 第 389、390 行）那两步 I-9.29 判成立。产品臂上没造出这一形，所以这一页答不了「产品路径上出现了 I-9.29 能不能抓」，只说 P 臂上它 0 次。
- 「仓副本装置、冻结副本」：数出自 2026-10-08 拷的冻结副本（`baseline-frozen-crates.out`），没在入库装置上重做过。拷完之后主工作区 `crates/` 变了 4 个文件（`baseline-frozen-crates.out` 的 `name=main_drift_after_copy`：`crates/mutations.tsv` 与三个 `crates/singlefs-checker-tier/src/bin/` 下的装置），都不在这个装置编进来的 `singlefs-core`、`singlefs-checker`、`singlefs-harness` 里（推的：按路径判，没在漂移后的代码上重跑）；`research/scripts/replay.sh` 的 `driver_e175` 在漂移后的主工作区上编、跑 D 族 P 臂一次，输出去掉 `elapsed_ns` 与 `e175-product-path-inode-watermark-regression-2026-10-08-d-P-replay.out` 相同。
- 什么现象会推翻页上的读数：同一份冻结副本、同一装置重跑出任何一段 P 臂结果行与产物不同（续跑与多线程合并在快档对拍过）；或第三段 F3b / F4、或登记之外的历史里出现同一条时间线上 Q1 ≥ 1、I-9.29（同号只许一个出生代） 红。

```

**出处 `.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:188-198`（整段抄，未转述）**

```markdown
### 路径与结论登记

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | S3 路 1（实例表） | 最新那条可读根的实例表，按 (实例, txg) 判每条根抛不抛弃；对两段每一步的每一条可读根（扫遍） | `research/results/e175-product-path-inode-watermark-regression-2026-10-08/device-source.rs:713 abandoned_by_the_table`，判法 `crates/singlefs-core/src/recovery.rs:1553 publish_is_abandoned_by_the_instance_table` | `readable_roots` 列出的根、盘上实例表、`crates/` 的抛弃判法 |
| 2 | S3 路 2（步账） | 每次挂载做成，模型根环里那一步之前最新的那条根当所选根，同一旧实例里 txg 更大的根判抛弃 | `research/results/e175-product-path-inode-watermark-regression-2026-10-08/device-source.rs:723 abandoned_by_the_step_ledger`，所选根取自 `crates/singlefs-harness/src/model.rs:2235 answer_mount_writable` 的择法 | `readable_roots` 列出的根、理想模型的根环、执行器交回的挂载结局 |
| 3 | S1 账的重发号（Q1） | 按号记出生代与持有它的根，一次发号碰上同号另一个出生代就记事件 | `research/results/e175-product-path-inode-watermark-regression-2026-10-08/device-source.rs:1028 HistoryAccount::observe`，重建 `crates/singlefs-core/src/recovery.rs:5405 rebuild_version` | `readable_roots`、`rebuild_version`、路 1 的抛弃判法 |
| 4 | S2 checker 的 I-9.29（同号只许一个出生代） | 池级 checker 对每一步的镜像判 | `research/results/e175-product-path-inode-watermark-regression-2026-10-08/device-source.rs:1158 HistoryAccount::judge_with_the_checker`，`crates/singlefs-checker/src/walk.rs:10947 check_pool_image` | 只有镜像本身；与 1–3 不共用解析代码 |

路 1 与路 2 判同一个量，逐根比、不一致计数（`device-source.rs` 第 865 行 `s3_path_disagreements`），两段二十六份结果文件都是 0 次；两路共用 `readable_roots` 列出的那一组根，它列错了两路会一起错。路 3 与路 4 不是同一个量，登记第十节 F3 把两者对着核（第 1188、1199 行），两段都是 0 次。跨路径的比较是计数、不是 `assert_eq!`，欠一条把两路连起来的断言。

```

**出处 `.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:199-205`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 6 | 备料 | 2026-10-08 不受影响：两段在冻结副本上重跑，射程「同一条时间线上 inode 号不复用」在登记的 17 105 段产品臂历史上 0 次重发、0 段 I-9.29（同号只许一个出生代） 红，W 行够判；决策依据段没引 E175（产品路径上 inode 号水位退回与重发号），这一格该不该升成支撑交主 agent 定 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-08 不受影响：两段在冻结副本上重跑，回退那次的水位取 max 这一格，D2(k) 上发 4（A3）、F3a / F2 / F1 / 几何点的 Q4 窗口里 0 次重发；决策依据段没引 E175（产品路径上 inode 号水位退回与重发号），升不升成支撑交主 agent 定 |

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:1-4`（整段抄，未转述）**

```markdown
## D18 块里携带什么信息 —— 已定

D18（块里携带什么信息） 管一个单元（块）里带哪些信息：一个字段值不值得带进块里的判据（已定项 19）、已定要带与已否的字段（已定项 20、已定项 23）、各类单元头的字段表与偏移、单元类型标签的值域与打包记录类型的登记表（含实例表）、头校验和的算法与 nonce / MAC / 算法类型预留位，以及扫描重建凭块头能做到哪一级。不管的：「这个块还活着吗」是**关于集合的问题**，块永远答不了——那不是格式没做好，是 D5（快照 / 空间记账机制） 已定它是全局属性，且那正是 O(1) 的来源；AAD 的字段组成归 D9（加密） 已定项 6。与别的决策的连锁：数据块自描述与 D9（加密） 已定项 5 的泄漏面权衡直接相关（已定项 3、已定项 13）；节点内 parity 与 D2（RAID 条带策略） 的变宽全条带在多盘上部分重叠，但**单盘是 D2（RAID 条带策略） 够不着的地方**（已定项 4）；携带的信息若落在**每条记录**上而非块头上，成本按记录数乘（D8（核心索引结构），已定项 19）；「上次校验时间」内联会让 **scrub 从只读变成写操作**，每扫一遍造新块、把旧块判死灌进 deadlist，**与 D3（空间分配） 常驻整理的目标正好相反**（已定项 19 判据 1）。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:36-48`（整段抄，未转述）**

```markdown
#### 已定项 1：谱系重写序号不做

**定案**：谱系重写序号（沿谱系单调增，与事务号正交）不做。**它要买的那件事已经被别人买走了**：谱系重写序号的用途是「分得清陈旧副本与现行副本」，而 D3（空间分配） 已定项 1 的逐落点谓词正是那个能力的权威记录点 ⇒ 再加一个序号买不到新的东西，只多一个永久的格式字段。

**射程**：定案否掉的是「为分辨陈旧副本而设的序号」。若将来出现别的用途（例如与 D16（发布语义） 已定的 checkpoint 序号怎么共存那一问真的独立成立），要重开就重开，**但重开的人得先说清它买的不是已经买到的那件事**。单元头的写序 (实例代号, 事务号)（已定项 7）不在否掉的范围内：它买的是「这个单元发布了没有、属于哪条时间线」；同 key 已发布版本的定序靠诞生代号 + 每事务每 key 一种净效果就够，写序只在同 txg 跨事务改同 key 时顺带定序。已定项 11 曾拟用诞生代号给同一身份的多版打包容器择新，正是这里否掉的那个用途；已改按 D3（空间分配） 已定项 1 的谓词判，已定项 1 不重开。

**依据**：

- E59（缓冲消息能不能只从单元重算）：只扫单元时同一个 key 的多份可读副本分不出现行值，加上逐落点谓词之后四类消息的分歧全部归零——序号只买到覆写那一格，再加一个买不到新的东西。
- 用户定案，记在变更史；写序那一段射程注随 C113（扫描重建时多版单元的现行版本判定无输入） 定案加。

**欠**：C288（映射 key 定案的连带改动没落到别的决策） 第 ⑥ 条：射程注里补「出生序号只做 key 的一段、不拿来择新」。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:49-63`（整段抄，未转述）**

```markdown
#### 已定项 2：全部码 2 节点无条件带 key 区间

**定案**：**每一个码 2 索引节点都带 `[min_key, max_key]`，不按树分、不设开关。**码 2 的头宽公式 `86 + 2 × key 宽`（已定项 18）因此对全部树无条件成立，扫描期解一个码 2 节点不用先去查「这棵树带不带」。它买到的是：一个节点单独捡起来能自证它该覆盖哪一段 key，与 `.claude/rules/fs-design.md`「不为省空间牺牲自包含」同向。**区间取子树覆盖区间**：`[min_key, max_key]` 是这个节点的子树里到得了的最小 key 与最大 key——叶层码 2 节点上它就是首末两条条目的 key；孩子是码 3 容器的内部节点上，max_key 是最后那片容器里最后一条记录的 key，不是最后一条条目的分隔 key。改新池新建文件的字节：否——六棵码 2 树写出的字节里区间本来就都在，两种读法在新池新建文件那一档同值（inode 树根 [1, 1]）。

**射程**：外推的是「带不带」这个判定，不是 E73（节点 key 区间的扇出代价） 那几个代价数——哪棵树的扇出掉几格仍要各自量。**dirent 树那一格随之关闭**：有 dirent 树时它照样带 key 区间；那时要重新量的是扇出代价，不是这个判定。key 区间住块头（D15（格式冻结政策） 第 1 层），与记账树节点头同一格（C102（索引节点头没有预留区，加字段不是 compat））。checker 的 `check_index_node_keys`（`crates/singlefs-checker/src/lib.rs`）判的是首末两条条目的 key，并行线三的根写出 [2, 2] 而子树覆盖 [2, 234]，两处都要按子树覆盖区间改（C478（码 2 头的 key 区间是条目键区间还是子树覆盖区间））。按位置寻址的树（分配记录树、extent 树，D8（核心索引结构） 已定项 14），节点头的 key 区间写这个节点按位置规定罩的那一段，与「子树覆盖区间」在这类树上是同一段；checker 按位置独立算出每个节点该罩的那一段逐节点核。

**依据**：

- E142（新池新建文件的干跑）：第六次跑写出的六棵码 2 树全部带 key 区间，不带就没有一棵树的头宽算得出来；按树带的那一种要在码 2 头里再加一个「带不带」标志位，而扫描重建路径上没有树表，读到标志位之前先要知道头有多长，是个循环。
- E73（节点 key 区间的扇出代价）：带区间扇出最多掉 1 格、树高不涨，D8（核心索引结构） 已定项 2 的 16 KiB 不用重开；孤立节点判覆盖区间，不带区间的那条臂判不出来。
- 用户定案（弹窗，没有走三方论证，可推翻；主 agent 推荐的也是这一档），原话在变更史。
- 用户定案 2026-09-23：区间取子树覆盖区间，原话在变更史。

**欠**：C478（码 2 头的 key 区间是条目键区间还是子树覆盖区间）：checker 与写路径按子树覆盖区间改，配一份区间写成条目键区间的坏镜像。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:64-90`（整段抄，未转述）**

```markdown
#### 已定项 3：数据块带明文自描述头

**定案**：每个数据单元带一个明文的自描述头，字段取 D9（加密） 已定项 6 已定的那一组 `(单元类型标签, 树 ID, 对象 ID, 对象出生代, 锚点偏移)`。**宁可写大一点，不为省空间裁字段。**五个字段的宽度：**单元类型标签 1 + 树 ID 8 + 对象 ID 8 + 对象出生代 8 + 锚点偏移 8 = 33**（[verification-build.md](../verification-build.md) 第 8 问由此关闭；依据追溯在已定项 11）。

- **与 D1（数据可移动性 / 反向索引） 的边界**：**判据在 D1（数据可移动性 / 反向索引）**（什么可以进自包段、什么进扩展点，见它的已定项 1 / 3 / 4），**字段表在这里**（实际进了哪几个字段、各多宽、按单元类怎么分）。判据变了这张表跟着改；表怎么排不回头改判据。
- **不加第六个字段 `locality_id`**：`locality_id` 是提示（D8（核心索引结构） 已定项 3），重建时对象按 (树 ID, inode) 认、只看本头的 extent 树，给这个对象的全部 extent key（幸存的与找回的）编同一个首段，取值 inode 记录 > 幸存 extent key > 0；取错只丢局部性，重建产物本来只读（已定项 5）。重建规则的正文在 D8（核心索引结构） 已定项 6「重建」那一段。
- **两条必须守住的线**：

| 不许放 | 为什么 |
|---|---|
| **物理落点 / 设备 ID** | D9（加密） 已定项 6 已定「物理位置一律不进 AAD」⇒ 放进块头就是一个**不被认证、可无声篡改**的字段。而物理地址的权威记录点按 D9（加密） 已定项 5（只做 scrub）留在加密结构里，无密钥的一侧本来就读不到它 |
| **文件名 / 分隔 key / 任何路径信息** | D9（加密） 已定项 5 的定案逐字写着，它与 ZFS 那条路的**全部**区别就是范围——「不暴露 btree 内部节点的分隔 key」，而 dirent 树的分隔 key 就是文件名。**守不住就等于走了 ZFS 那条路** |

- **扩展点里放物理指向为什么不撞这两条线，理由是「造不出静默误读」，不是「它被认证了」**：这两条线要挡的是**静默误读**——块头里一个被伪造的物理落点，会让读者读到错块却以为读对了。而扩展点里那条指向**进不了任何判定路径**（D1（数据可移动性 / 反向索引） 已定项 5 逐字：它不构成引用），每次解引用都要过**目标的诞生代号**比对与目标单元自己的头校验和 ⇒ 伪造它最多让一条线索指向空处，**造不出「读错还以为读对」**。这与 D9（加密） 已定项 5 已经知情接受的那一格（明文侧可被重排、代号可被伪造）同类，不新增类别。**不许拿「扩展点在整单元的校验 / 认证覆盖内」当理由**；理由维持「造不出静默误读」，它不依赖 MAC 覆不覆盖扩展点；换回「扩展点被认证」要重新论证，没做。

**射程**：它买的是「索引没了还能重建」，**不是「攻击者改了还能发现」**——后者靠 AAD 失配与 D4（校验和位置） 的 Merkle 树。**块头明文可读不构成认证**：常规读（索引在）明文头不参与，AAD 期望值按 A6（AAD 期望值不来自明文侧） 来自查找路径；重建（索引没了）必须参与，单改字段会 MAC 失配，但整单元搬运（头 + 密文一起搬）验证完全通过，而那正是 A6（AAD 期望值不来自明文侧） 要挡的「块级重排」——这一格按已定项 5 的降级语义处置。两条线管的是自包不可变数据那一段，不管扩展点：**物理落点是可变的**（块会被搬，而 D1（数据可移动性 / 反向索引） 存在的理由正是要能搬），按「随版本变不变」它本来就进不了自包段；扩展点是给可变数据的（D1（数据可移动性 / 反向索引） 已定项 3：要么为空、要么是一个指向）。加密开启时块头这个元组是它在盘上**唯一的**明文表达（D9（加密） 已定项 5 不设明文映射层、不设明文反向索引），锚点偏移直接给出文件大小与文件洞；这个泄漏不接受，处置归已定项 13，已定项 3 本身不变——数据单元仍然带自描述头、字段集合与宽度都不动，已定项 13 改的只是「加密开启时这几个字段在盘上是明文还是密文」；主依据（为快速拿到而存在、不为省空间牺牲自包含）不依赖泄漏面这一格。这一组同时是 D9（加密） 已定项 6 的 AAD 字段集，而 C72（extent 和单元什么关系没人答） 那轮已确立「AAD 的字段组成是 day-1 永久契约」⇒ 动它是一次永久契约变更。由已定项 3 派生出的三格各自是正式分项：已定项 5、已定项 6、已定项 7。

**依据**：

- E28（明文映射层能不能只靠单元自描述重建）：删掉明文映射层、只扫单元头重建时，元数据侧一条不缺一条不错，数据块侧缺口恰等于数据块总数；阳性对照给数据块加上自描述之后缺口归零——它买到的就是这个洞。
- 用户定案：这些字段是为**快速拿到**而存在的，越快越好；通用取向是**不为省空间牺牲自包含**（`.claude/rules/fs-design.md`），原话在变更史。
- 两条线的射程：用户定案，原话在变更史。
- 「块头明文可读不构成认证」：三方论证的反推腿指出，主 agent 逐字核 D9（加密） 已定项 6 的判据 A6（AAD 期望值不来自明文侧） 坐实，经过在变更史。
- 不加 `locality_id`：三轮三方（`research/prompts/d18-locality-r1-main-verification.md`、`d18-locality-r2-main-verification.md`、`d18-locality-r3-main-verification.md`）之后的用户定案，被否形态的论证在三份判决里。

**欠**：C346（后台重新聚簇没有定义）；C68（不变量白名单与已定条款打架）：门禁比对工装。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:91-103`（整段抄，未转述）**

```markdown
#### 已定项 4：形态取元数据多副本 + 分散放置，链臂否掉，单盘布局线先不开

**定案**：单盘自恢复的形态取「元数据多副本 + 分散放置」（ditto），链臂正式否掉；但**先不开单设备布局线**。ditto 取 ZFS 的真实形态：用户数据 1 份、数据集元数据 2 份、全池元数据 3 份，副本分散放置。**取 ditto 时必须带上的一条**：放置偏移**不许取 2 的幂**——与任何 2 的幂周期的故障共振，三份副本同余、一起死。改新池新建文件的字节：**否**（ditto 是副本份数与放置政策，不动任何字段表；单盘布局线没开 ⇒ 新池新建文件写出的字节不变）。

**射程**：**已定项 3 的定案不减轻已定项 4**：数据块自描述防的是「**索引没了**」，已定项 4 防的是「**块本身没了**」——自描述读不出一个已经不在盘上的块，两者射程不同。不开单盘线的理由：单盘线是**另一套布局**，不是「2 盘线拔掉一块盘」——D2（RAID 条带策略） 已定项 6 的 `w ≥ 2` 与单设备互斥（D2（RAID 条带策略） 已定项 9），D12（目标介质） 还要给它一个 incompat 位，那是一条永久的格式分支，按 `.claude/rules/fs-design.md` 的分支判据要单独证成；开线要单独过崩溃点重放，且要有只供测试的开关能强制进入。多盘与单盘不等价：`w ≥ 2` 且零冗余条带不许发出 ⇒ 每条条带在 ≥2 盘池里天然带 parity，丢 1 列可由其余列重建（「一个单元整个落一列（D2（RAID 条带策略） 已定项 10）⇒ 丢一块盘等于丢整个单元 ⇒ 多盘与单盘一样」这个说法**不成立**）；**单盘没有这道机制**。链若取版本链（前后 = 本单元的 COW 前驱与后继）会撞否决清单两条（已定项 23：「子块记录当前父节点物理地址」判「时序上不可能」，「COW 前驱地址（版本链）」判「净值为负」），只剩同批链可比。

**依据**：

- E118（单盘自恢复的三条臂）：跑前写死的失败条款按字面触发——同批链的恢复数在**所有**故障点上都等于无冗余臂，它买到的是**检测**，而检测 D3（空间分配） 已定项 3 的分配记录本来就答得了 ⇒ 净增 ≈ 0；ditto 是三条臂里唯一买到恢复能力的；「至少隔 1/8 盘」那个偏移在模型里是 2 的幂，与 2 的幂周期的故障共振。开线前的六条举证由它一次还清。
- 三方论证落下的六条开线举证与判红条件、用户定案，都在变更史。

**欠**：C201（ditto 的放置偏移不许取 2 的幂）。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:104-141`（整段抄，未转述）**

```markdown
#### 已定项 5：重建是降级路径，按见证者分三级

**定案**：承认重建是降级路径，但形态不是一句声明，是按拿得到的见证者分级：

| 级 | 拿到什么见证 | 处置 |
|---|---|---|
| 2 | 分配记录（D3（空间分配） 已定项 3）+ 校验和 / MAC 双证 | 可写，正常晋升；⚠️ 今天进不去——分配记录是派生态（D21（权威态与派生态的分界） 已定项 5），派生态当不了见证，级 2 的见证者不存在 |
| 1 | 只有校验和 / MAC 自洽，没有分配记录 | **只读挂载** |
| 0 | 连校验和都验不了 | 只报告，不进映射 |

**分级判据可计算，且输入在操作期间不变**，满足 `.claude/rules/fs-design.md` 分支硬要求 1 与 5。**三样落实，缺一样它就只是一句提醒句**（`.claude/singlefs-ai-sop/rules/show-me-test.md` 逐字禁止把踩过的坑写成提醒句）：

| # | 落成什么 | 为什么必须是这个形态 |
|---|---|---|
| 1 | 系统配置里一个 `map_provenance`（重建纪元 + 重建时所依据的权威态摘要），**day-1 留位** | 不 day-1 留，「以后可以加」会在 D15（格式冻结政策） 冻结那天悄悄过期。它必须落在被认证的字段上（D9（加密） 已定项 8 已定系统配置因 nonce 水位必须认证） |
| 2 | `map_provenance = rebuilt` 且未晋升 ⇒ **只读挂载** | `.claude/rules/fs-design.md`「把用错了变成挂不上」的 `compat_ro` 形态：挂得上、写不了，失败模式显式 |
| 3 | 一条纪律：该状态下**任何权威态结构不得由重建结果就地修复** | 它是 I-6.10（已停用·明文映射层不存在了） 逐字「差异一律判红，**不许就地修复**」扩到重建产物上，不是新机制 |

`map_provenance` 的三条格式级要求：

| # | 要求 | 为什么 |
|---|---|---|
| 1 | 它住系统配置，**且不许住在一个能被无痕改写的位置** | 一道能被改一个字节就绕过的闸，与没有这道闸等价。它的保护强度随加密开关变（射程） |
| 2 | **级 1 → 级 2 的晋升必须留痕**，不许原地把标志位改回正常 | 晋升就是「合法地清掉 rebuilt」——**它本身就是绕过路径的形状**。留痕内容：依据哪一批双证、何时晋升；那条记录自己也要能被 checker 复核 |
| 3 | 后续正常写路径**不得覆盖它** | 遗忘形态：声明一次、之后再没人看，此后每次挂载都显示正常。**善意行事也会走到那里** |

**「要人确认」这半句不采纳**：管理员手里没有原始索引或参照数据可比对，**判断不了文件有没有被移动** ⇒ 那个要求在实践中等于逼人在没有可验证数据的前提下信任重建过程，**而那正违背了认证的目的**。晋升的判据只能是级 2 那种可计算的双证，不能是一次人工点头。

**射程**：**第一版防得住什么、防不住什么，照实写**：D9（加密） 已定项 8 让系统配置必须被认证，那条的手段是**主密钥派生的 MAC**，而 D9（加密） 已定项 10 已定加密不进第一个可运行版本 ⇒ **第一版没有主密钥，那份认证不存在** ⇒ `map_provenance` 在第一版只有非密钥校验和护着，**防意外损坏，不防故意篡改**；这与 D9（加密） 已定项 5 的取向一致，但**必须写下来**，不许把「有这个字段」读成「这条降级状态抹不掉」。**A6（AAD 期望值不来自明文侧） 在第一版一个字节都不管辖**：第一版真正改字节的**只有 `map_provenance` 的留位**（系统配置「映射来源（重建纪元 + 权威态摘要）」24 字节、全 0，[layout/01-first-txn.md](../layout/01-first-txn.md)），其余是重建代码写成什么样，而 `crates/` 里没有扫描重建（`grep -rn "扫描重建" crates --include=*.rs` 零命中），第一版的 checker（`crates/singlefs-checker`）不开加密、不判 A6（AAD 期望值不来自明文侧）。**级 2 的见证者没了**（分配记录是派生态，C73（分配记录的态别没定） 逐字「派生态的东西自己就要被重建，当不了见证」）：级 2 要么另找一个住在权威态里的双证，要么这一格重开；**级 1 与级 0 照常可用**，受影响的是晋升那一步。

**依据**：

- 无实验：分级是按拿得到的见证者给重建产物定挂载态，定义性的，没有可量的量。
- 三方论证：两条腿独立确认候选 ①（靠父指针）在「索引丢了才要重建」这个前提下是循环的——它要用的父指针住索引，而索引正是被重建的那棵树，权威侧按 D21（权威态与派生态的分界） = 单元 + 记账 + 根、不含索引；本地腿指出「要人确认」判断不了。用户定案取候选 ②，原话在变更史。
- 级 2 见证者不存在：D21（权威态与派生态的分界） 已定项 5 的用户定案（分配记录是派生态）。

**欠**：C75（降级挂载这道闸自己不会红）。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:142-154`（整段抄，未转述）**

```markdown
#### 已定项 6：数据块自描述头放在低位，载荷从头之后开始

**定案**：每个块的**第 0 字节起是自描述头**，载荷紧随其后。**「头挂块级还是单元（extent）级」这一半定成块级**：定在低位就意味着扫描器按块步进即可命中，而单元级要求扫描器先知道单元边界——**那正是它要重建的东西**（循环）。

**射程**：块 = 单元（已定项 9）且 extent 不跨多单元（D4（校验和位置） 已定项 7）之后，「块级」与「单元级」在这个取值下重合；块级当初要付的「一个跨 N 块的大 extent 要付 N 份头」那笔摊销代价 N 恒为 1，对象没了。

**依据**：

- 无实验：判据是「恢复期的全盘扫描要能在**没有索引**的前提下找到这些头」⇒ 头必须落在扫描器能**预测**的位置上，而块的起始是唯一不需要任何先验知识就能算出的位置。
- 用户定案，记在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:155-201`（整段抄，未转述）**

```markdown
#### 已定项 7：第一版单元头字段表，按单元类分

**定案**：第一版单元头字段表按单元类分——共同明文前缀 + 类身份段；**fsid 与诞生代号放进头里，头校验和取 32 字节**；v1 预留 nonce 代号与 MAC 的字段位；自证单元（journal 记录 / 根槽 / 系统配置槽）不用这张表。

**共同明文前缀（数据单元与索引节点，42 字节初值；宽度沿用 E85（单元头的字段表）的
需求矩阵口径，写代码时再调）**：

| 字段 | 宽 | 出处 |
|---|---|---|
| magic | 4 | 扫描探针（E86（扫描步进与两种单元大小）：16384 步进落在数据载荷内部的探针靠它 + 头校验和拒收） |
| 格式版本 | 2 | D21（权威态与派生态的分界）：扫描重建要能解全部历史单元格式 |
| flags | 2 | **偏移 6 = 单元类型标签（已定项 11 登记表的码），偏移 7 = 8 个 flag 位、第一版恒 0、非 0 一律拒收（前向兼容政策）**——加密开启后元数据身份进密文，扫描器必须在解密前分出类才知道套哪张白名单 |
| 声明长度 | 2 | D4（校验和位置）已定项 6：I-2.3（补齐字节为零且参与校验和）要在扫描期无索引可判。⚠️ **加密开启时它是密文**（已定项 13，臂庚）——位置与宽度不动，只是不再属于明文白名单 |
| 头校验和 | 32 | 扫描重建时没有父指针，头完整性唯一防线（加密开启后对元数据类只罩明文部分，类身份段的完整性转由 MAC 承担）；**覆盖范围由已定项 11 定：偏移 0 到该类明文头末尾**（I-2.4（头校验和覆盖范围））；宽度与 E23（journal 几何）`header_csum`、D22（单元原子性怎么合成）已定项 7 根记录自证校验和同口径。**算法由已定项 17 定**：CRC32C 4 字节 + 28 字节零 |

**类身份段——数据单元（+63 字节 ⇒ 共 105 字节初值，占 32 KiB 单元 0.32%）**：
<!-- format-const: DATA_UNIT_HEADER_BYTES = 105 -->

| 字段 | 宽 | 出处 |
|---|---|---|
| 五元组 | 33 | D18（块里携带什么信息）已定项 3 已定（明文、宁可写大）；其中单元类型标签与 flags 类标签位冗余，接受（`.claude/rules/fs-design.md`「不为省空间牺牲自包含」） |
| 诞生代号（birth txg） | 8 | 三条独立理由，见依据 |
| fsid | 8 | 两条独立理由，见依据 |
| **写序（实例代号 4 + 事务号低 48 位）** | **10** | C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1：写出它的那个事务的 (实例代号, 事务号)，三件事共用一个字段——同 key 已发布版本定序、分辨崩溃前后时间线、在飞 checkpoint 被部分重放时分开已施加与被抛弃；事务号是 D23（journal 的角色与格式） 已定项 7 已定的 8 字节字段的低 48 位，不新开字段 |
| **载荷 CRC** | **4** | C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3：副本归并要求「同类身份段的多份可读单元载荷相同」，码 1 没有别的字段能证明这一点 ⇒ C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据） 的载荷 CRC 是码 1 副本归并 / 级 1 恢复的前置；只加在码 1 自己的类身份段，不进共同前缀 |

- **加密开启时这张类身份段表里只有 fsid 与载荷 CRC 留明文**（已定项 13）：五元组 / 诞生代号 / 写序就地变密文，位置与宽度一个都没动，明文头总宽仍 105。加密关闭时整张表都是明文，新池新建文件写出的字节逐字节不变。
- **载荷 CRC 覆盖的是盘上字节，不是加密前的明文载荷**（已定项 13-c；码 1 / 2 / 3 三类同一口径，别处一律链回这里）。加密关闭时两者同义，开启时算的是**密文**。两条独立理由：① 覆盖明文的话无密钥一侧读不到明文，D9（加密） 已定项 5 承诺的「检测损坏、报告坏块」当场落空；② 覆盖明文而 CRC 值又在明文头里，等于给无密钥的攻击者一个 32 位的**内容确认预言机**——猜一段明文、算 CRC、比对，就能确认猜得对不对。知情接受的代价：同一个 extent 的各副本是同一份密文（D2（RAID 条带策略） 已定「N 个副本是同一份密文写 N 遍、共用一个 nonce」）⇒ CRC 相等即可认出副本 ⇒ 反解 `w`；第一版 `w` 恒 2（D2（RAID 条带策略） 已定项 9）⇒ 这一格第一版没有增量。**它验的是「密文没坏」，不是「用户数据没坏」**：解密路径自己引入的错误由 MAC 抓，不由它抓。
- **类身份段——索引节点**：树 ID + 层级 + **key 区间**（已定项 2：全部码 2 节点无条件带）+ 诞生代号 + fsid + **写序 4（只实例代号）+ 出生序号 4 + 载荷 CRC 4 + 预留 2 + 自描述三段（key 宽 1 + 条目数 2 + 条目宽 2）5**（写序、载荷 CRC、预留三段见已定项 12；出生序号见 C288（映射 key 定案的连带改动没落到别的决策） 第 ① 条；自描述三段见已定项 18 与 D8（核心索引结构） 已定项 11；五段合计 19），类身份段之后是 29 字节 nonce / MAC / 算法类型预留位（已定项 16）。三段各自的语义：**写序 4** 是 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1 那 10 字节里唯一对码 2 有消费者的一段——I-1.2（块头写序已发布） 逐字「码 2 / 3 要 b ≤ T_pub」，谓词只读实例代号；它买的是让一个没发布的 checkpoint 的节点按已发布谓词判成垃圾、能被清扫。**载荷 CRC 4** 还 C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据） 码 2 那一半，口径照码 1 与码 3：CRC32C，覆盖从头末尾到单元末尾（记录区 + 补齐），落在头校验和覆盖内。**预留 2** 按 D8（核心索引结构）「flags / 填充 / 预留三段同一政策」：恒 0，读者遇到非 0 ⇒ 该记录 EIO（对象级），赋义要 incompat。**出生序号 4**：D19（块指针的结构与宽度预算） 已定项 6 给码 2 / 码 3 各加的那 4 字节，**紧跟写序**——码 2 在写序 4 与载荷 CRC 4 之间，码 3 在写序之后（偏移 103，已定项 11）；只做中央映射 key 的一段，不拿来择新（D19（块指针的结构与宽度预算） 已定项 6）。**被去掉的是那 6 字节事务号**（已定项 12）。码 2 的完整偏移表见已定项 18，不在这里存第二份：明文头 86 + 2 × key 宽，含 29 字节预留位后 115 + 2 × key 宽（D8（核心索引结构） 已定项 11）。
- **v1 就预留 nonce 代号与 MAC 的字段位**（D9（加密）已定项 10「只预留格式字段」的 day-1 预留纪律）：不预留则开加密成了布局变更，而 D21（权威态与派生态的分界）要求扫描重建解得了全部历史格式。
- **自证单元（journal 记录 / 根槽 / 系统配置槽）不用这张表**：各自的头已由 D23（journal 的角色与格式）与 D22（单元原子性怎么合成）已定项 7 定；它们没有父引用、没有树身份，I-1.1（块头自述逻辑地址）/ I-1.3（块头树 ID 一致）对它们**显式豁免**，自证纪律在 I-7（根环）/ I-8（journal）两类与 D22（单元原子性怎么合成）已定项 21。它们在已定项 11 的登记表里各占一个码——根记录（根槽）16、journal 记录 17、系统配置槽 18（次序以已定项 11 的登记表为准），只做 AAD 域分隔子。
- **认领规则**：同一个卷的自备份镜像 fsid 挡不住（fsid 相同），那一格靠**认领规则**挡：扫描器解出一个单元头并认领它之后，就按整单元跳（码 1、码 3 两槽，码 2 一槽），载荷内部的探针不再参与认领。这条规则要做成 checker 里能判红的形式。
- **加密开启后的明文白名单随之改写成两张**（权威记录在 [invariants.md](../invariants.md) I-6.2（明文头字段白名单））：两类共享共同前缀 + nonce 代号 + MAC；元数据类身份字段（五元组 / key 区间）一律进密文（无密钥重建本来就不可能，D21（权威态与派生态的分界）已定）；数据单元类另许 {fsid, 载荷 CRC}（已定项 13 收窄之后）。

**射程**：索引节点头的（树 ID + 层级 + key 区间）只回答**扫描重建**的身份，**不回答 AAD 身份**——AAD 期望值不来自明文侧（D9（加密）已定项 6），索引节点进 AAD 用哪些身份字段全仓无落点（C107（索引节点的 AAD 身份无落点）；D9（加密）已定项 9 已撤回）。已定项 7 管块头这一半；另一半（明文映射层的字段表）随 D9（加密） 已定项 5 改取「只做 scrub」一并撤销 ⇒ **不再有「两半的字段口径必须对得上」这个约束**，块头是这个元组在盘上唯一的明文表达。诞生代号今天承重的消费者是已发布谓词（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2）；「撕裂孤儿靠它才升得到级 2」那条理由在级 2 见证者不存在期间分辨不出带不带它（已定项 5），另两条理由不受影响。fsid 那两条理由描述的是级 2 可进时的形态：今天冒名单元到不了晋升那一步、止于级 1。加密开启后 fsid 与 KDF 冗余，**这份冗余接受**，代价是 8 字节。nonce 那 12 字节的归属由已定项 14 收口（完整 96 位 nonce 住单元明文头、指针里那份保留），D9（加密） 已定项 4、[invariants.md](../invariants.md)「nonce 代号必须放在明文头」与单元自包含三处同真。认领规则按整单元跳，与 E88（冒名单元与孤儿单元的判别） 量的跳法（跳到本单元起点 + 单元大小）一致。

**依据**：

- E85（单元头的字段表）：需求矩阵与字节账——已定最小集判不了 I-1.2（块头写序已发布） / I-1.4（块头 fsid 一致） / 头自证，修补集全够，与 I-6.2（明文头字段白名单） 的冲突逐项点名；共同前缀的宽度沿用它的口径。
- E88（冒名单元与孤儿单元的判别）：定案形态一个都没分错，去掉 fsid 与诞生代号的头即使开着认领规则也错分冒名与撕裂孤儿 ⇒ 两条骑手站住；自身镜像那一格只有认领规则挡得住。
- E104（扫描重建的现行版本判定）：写序三件事各有一格离不开它——头里只有实例代号、没有事务号的那条臂恰在「在飞 txg 被部分重放」那一格失守 ⇒ 码 1 的写序要带事务号。
- 三方论证（C68（不变量白名单与已定条款打架） 收口）：三条腿的原样输出在 `research/prompts/c68-unit-header-{sonnet,opus,local}-output.md`，背景材料在 `_c68-unit-header-background.md`；反推腿三条可达破绽把 fsid 与诞生代号翻回头里、头校验和取 32 字节。诞生代号的三条理由（kill 规则的输入在重建后无落点、撕裂孤儿要靠「birth 大于根的 checkpoint_txg」正面认出、扫描方向要能判 I-1.2（块头写序已发布））与 fsid 的两条理由（错向写与嵌套别卷镜像、C74（无加密期的跨卷错向写没防线） 的 scrub 半边）原文在变更史。用户批准原提案、判决按骑手修订后落地。
- 写序与载荷 CRC：C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1 / P3。
- 用户定案（2026-10-03）：认领之后按整单元跳（码 1、码 3 两槽，码 2 一槽），判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第六节 S2-2 行，原话在变更史。

**欠**：C89（单元头收口的遗留重算）：认领规则进 checker；C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据）：检查仍欠；C107（索引节点的 AAD 身份无落点）；C289（码 2 的出生树取哪个字段没写）；C291（出生序号的分配规则只在实验源码里）：检查仍欠；C68（不变量白名单与已定条款打架）：门禁比对工装。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:202-216`（整段抄，未转述）**

```markdown
#### 已定项 8：删除要写墓碑单元

**定案**：**删除不再是「盘上什么都不留」——它在盘上留下墓碑记录。** 第一版在意图那次发布写，载体与字段见已定项 24。三句分开写：容器的单元类型标签取已定项 11 登记表的**码 3（打包记录单元）**；容器内的记录取**打包记录类型 1（墓碑区间记录）**；死者由墓碑区间记录自己的字段指认（对象 ID + 对象出生代 + 区间，已定项 10），**不用容器身份顶替记录身份**。为什么必须有它：删除若在盘上不留痕迹，扫盘重建就**没有任何输入**能知道这个 key 死过，而谱系重写序号救不了这一格（它只买到覆写那一格，且已定项 1 定案不做——盘上没有那条消息的任何痕迹时，序号无从比较）。

**射程**：删除这个操作本身要写盘，而 D3（空间分配） 已定项 17 逐字写着「COW 系统必须保证『释放空间』这个操作本身不需要申请空间」；那笔账已由 D3（空间分配） 已定项 2 收掉：墓碑计入准入控制「解开自己需要多少」那个量，在写入那一刻付。**不在这里记第二份。** 已定项 8 定的是「盘上留一个墓碑单元」，**没有定它什么时候可以被回收**——回收规则、粒度与载体归已定项 10（回收规则只有初值、判据欠实验）。D11（索引节点要不要留消息缓冲区） 已定项 4 的第三格压在这一条上。

**依据**：

- E59（缓冲消息能不能只从单元重算）：只扫单元时已删的 key 全数复活，复活数恰好等于删除次数；序号 + 显式墓碑单元那一臂四类全 0——删除要写墓碑单元。
- E102（单元类登记表与打包记录单元）：五元组对装满的容器只指认得了 1 条，打包类身份段 + 解析全部指认得了——死者由记录自己的字段指认，不用容器身份顶替。
- 第一版在意图那次写：用户定案 K-U2「T1：意图那次写整对象墓碑」，原话在变更史。
- 用户定案，记在变更史。

**欠**：C634（单次 unlink 的墓碑字节没有 trace 检查）（记录字段表与单次删除的墓碑字节在已定项 24）。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:217-230`（整段抄，未转述）**

```markdown
#### 已定项 9：「块」就是单元，一个单元一份头

**定案**：**一个单元付一份自描述头；扫描器按池里最小的那种单元大小步进。** 第一版有三种单元并存——索引节点（码 2）16 KiB，数据单元（码 1）与打包记录单元（码 3）各 32 KiB（D4（校验和位置）已定项 1；三类的单元大小 / 对齐以已定项 11 的登记表为准），所以步进取 16384。**它给分配器加了一条硬要求**：单元起点按类对齐——**数据单元与打包记录单元对齐 32768、索引节点对齐 16384**（三档取自已定项 11 登记表的「单元大小 / 对齐」列），否则扫描器步进会错位。落点是 D3（空间分配） 的分配器实现，**要做成能被测试强制违反的检查**（`.claude/rules/fs-design.md` 硬要求 2）：造一个起点不对齐的单元，扫描重建必须判红。

**射程**：不按 32768 步进——那样正好会漏掉全部落在奇数槽上的 16 KiB 节点。**它靠 D4（校验和位置） 已定项 7 解锁，而不是推翻已定项 6**：D4（校验和位置） 已定项 7 定下 extent 占盘**恒 32768** 之后，**单元边界变成可预测的**（按池内最小单元大小步进即可全部命中，前提是分配器按单元对齐分配）⇒ 「块级」与「单元级」在这个取值下重合，已定项 6 的结论原样成立。分配器侧码 3 按哪一档取落点由 D3（空间分配） 已定项 10 ⑤ 定（按数据单元那一档，起点 32768 对齐），`crates/singlefs-core/src/allocator.rs` 按类对齐分配；checker 侧没有对齐的判红（`grep -rn "对齐\|align" crates/singlefs-checker/src` 零命中）。

**依据**：

- E86（扫描步进与两种单元大小）：按 16384 步进全收，代价是每个数据单元多出 1 次落在载荷内部的探针、靠 magic 加头校验和拒掉；按 32768 步进恰好漏掉全部落在奇数槽上的节点；让节点独占一对 32K 槽也能全收，但元数据占的空间翻倍。
- 不取「块 = 设备物理块」（算术）：若块 = 4096 字节物理块，一个 32 KiB 单元要付 **8 份内容逐字相同的头**；512 字节盘上 **64 份**，头占比 15.6%，五元组的值在同一个单元的各份头里完全相同，是纯冗余。
- 用户定案（块 = 单元；步进口径修正为 16384），记在变更史。

**欠**：C86（两种单元大小与扫描步进对不拢）：黄金镜像上的红绿对照与不对齐单元的判红。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:231-251`（整段抄，未转述）**

```markdown
#### 已定项 10：墓碑取区间记录 + 打包共享单元

**定案**：

- **记录粒度**：对象整个删除写**一条区间记录**（对象 ID + 对象出生代 + 全区间）；truncate / punch 这类部分删除按**每段连续 extent 区间一条**。对象出生代与五元组同口径、扫描时逐字段对照；D8（核心索引结构） 已定项 6 只保证已发布的 (inode 号, 对象出生代) 不复用，崩溃恢复之后号会被重发，同号的两个已发布对象的墓碑靠它分开。
- **载体**：墓碑记录**打包进共享的墓碑单元**（容器是已定项 11 登记表的码 3 打包记录单元、记录是打包记录类型 1），不给每条记录独占一个单元。
- **装载**：**墓碑单元按代际装——一个代际结束就把当时开着的单元关掉，不让不同代际的记录混进同一个单元。** 这跟 D2（RAID 条带策略）「不让两个生命周期不同的对象共享同一个物理映射单元」是同一条纪律，只是作用在墓碑上。
- **回收规则（初值，用户口径：先这么写，再实验）**：**墓碑记录随快照 deadlist 同一套节奏清**——一条墓碑在「它指认的死亡已被全部还引用它的根（含快照与可写头）送走」之后可回收；量词取「根」不取「快照」：克隆头是头不是快照，凡是它里面还活着、而 origin 后来删掉的对象，按「快照」量词它的 kill 记录可以被回收，而克隆引用的那具尸体一直可读。与 D5（快照 / 空间记账机制）销毁快照时过滤 deadlist 是同一个判定点，不另开扫描。第一版不回收墓碑（已定项 24）。

**射程**：打包把回收耦合成 E80（条带的部分释放） pin_stripe 那个钉住形状（回收粒度 = 单元）。已定项 11 之后按容器数读：同时开着的容器数 = 树 × 打包记录类型 × 活代际 × 活头（C106（开放容器数上界）），且开放容器有新记录的那次发布才重写，不删除的发布照抄（已定项 24；写放大的账在 C105（打包容器按发布频率重写的写放大））。两条连锁：① D3（空间分配）已定项 2 的墓碑预留额按已定项 10 的口径重算——预留的是**记录**（摊 1/815 个单元，第一版墓碑记录 40 字节，已定项 24）不是整单元，E82（准入的在飞合成）的「墓碑一格」是定粒度之前的高估；② 墓碑记录的宽度是定值 40 字节（已定项 24）。

**依据**：

- E83（墓碑的粒度）：四种组合 × 三种负载全表——每条记录独占单元是不可用的形态，区间记录 + 打包共享单元最便宜，truncate 大文件上差距更大。
- E84（打包墓碑的钉住量）：混装代际时稀疏删除是最坏格，活着的墓碑单元里大半是早已可回收的记录；**按代际分单元装载（代际边界关掉开放单元）钉住恒为 0**，代价只是每个活代际一个未满的开放单元。
- E104（扫描重建的现行版本判定）：墓碑不按门回收时旧版本复活——回收门承重；量词「根」随 C113（扫描重建时多版单元的现行版本判定无输入） 定案放宽。
- 用户定案（粒度与载体、装载纪律、回收初值），记在变更史。
- 40 字节定值、第一版不回收、号重发那半句：墓碑判决 `research/prompts/m3-tomb-r1-main-verification.md` 第三节 W2 号重发、W3 回收两行，`research/prompts/m3-delete-r3-main-verification.md` 第三节「墓碑第一轮一轮各行」那一行。

**欠**：回收判据欠实验（第一版不回收、记录字段表在已定项 24；回收门「数据单元已被覆盖或清扫」那一维要等整理落地，C621（整理落地那天要动的已写回删除条款没有清单之外的检查））；C105（打包容器按发布频率重写的写放大）；C106（开放容器数上界）。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:252-351`（整段抄，未转述）**

```markdown
#### 已定项 11：单元类型标签的值域是一张登记表，打包记录单元是独立的一类

**定案**：

**值域只有一处登记位——D18（块里携带什么信息） 已定项 11 的这张登记表。** D9（加密） 已定项 6 的值域行、D18（块里携带什么信息） 已定项 7 的字段表、D21（权威态与派生态的分界） 已定项 4、I-6.2（明文头字段白名单）、D20（承重面：单元的原子性与自包含） 的两类表、I-1（块自描述） 的豁免行，六处各自保留自己那一维的判定，**类的名字与码一律取自这张表**。**冻结的是「码 → 类」这一列，冻在两个时刻**：码 1 / 2 / 3 是共同前缀里真实存在的一个字节，第一个镜像写出那一刻冻结；码 16–18 盘上不存在，只出现在 AAD 首字节，加密开机那一刻冻结（之前改它零代价，之后改它是 AAD 组成变更、按 D9（加密） 已定项 6 是格式断裂）。**码一旦用过不许回收，退役码含义永久锁定**——与 D15（格式冻结政策） 对 feature bit 的纪律同形，理由相同（盘上真实存在过的字节）。

| 码 | 类 | 单元大小 / 对齐 | 头走哪张表 | 类身份段 | AAD 变体 | I-6.2（明文头字段白名单） | 扩展点 | 认领之后怎么枚举内容 |
|---|---|---|---|---|---|---|---|---|
| 0 | **无效**（永不分配） | — | — | — | — | — | — | 见到即判损坏 |
| 1 | **数据单元** | 32768 / 32768 | D18（块里携带什么信息） 已定项 7 数据单元类 | 五元组 + 诞生代号 + fsid + 写序 + 载荷 CRC（明文头 105 / 含预留位 134） | D9（加密） 已定项 6 的五项——引用时必须连 D9（加密） 已定项 6 索引行那句一起引：五项对快照维是瞎的且静默，字段表欠一次重开；树 ID 今天取读者树 | 数据单元类 | 带（D21（权威态与派生态的分界） 已定项 4） | 载荷是用户数据，不枚举 |
| 2 | **索引节点** | 16384 / 16384 | D18（块里携带什么信息） 已定项 7 索引节点类 | key 宽 1 + 树 ID + 层级 + key 区间 + 诞生代号 + fsid + 写序 4 + 出生序号 4 + 载荷 CRC 4 + 预留 2 + 条目数 2 + 条目宽 2（86 + 2k，含预留位 115 + 2k；偏移表见已定项 18） | **未定，无落点**（D9（加密） day-1 第 ③ 项）——C107（索引节点的 AAD 身份无落点） | 元数据类 | 条件带（D21（权威态与派生态的分界） 已定项 4） | 按节点格式解条目 |
| 3 | **打包记录单元** | 32768 / 32768 | 共同前缀 + D18（块里携带什么信息） 已定项 11 的类身份段表 | 65 字节 ⇒ 头 **107**（不含 nonce / MAC 预留位，与 105 同口径；含预留位 136） | （标签, 出生树 ID, 打包记录类型, 容器号, 容器出生代）——树维取**出生树**，即 D9（加密） 那次重开后码 1 也要取的语义；E90（树 ID 进 AAD 与跨头共享） 的两笔格式含义（指针带出生 (树, txg)、克隆祖先表住已认证结构）归那次重开，它们落地前码 3 的 AAD 不构成完整契约 | 元数据类 | 带（D21（权威态与派生态的分界） 已定项 8：紧贴单元尾部，第一版 N = 0）；会红的检查欠在 C108（扩展点归属对打包记录单元无判定） | 按记录数 × 记录宽定宽切分 |
| 4–15 | 保留给将来的内容类 | | | | | | | 未登记 ⇒ 拒收；新增一类占一个 incompat 位（D15（格式冻结政策） 判定表） |
| 16 | 根记录（根槽） | 槽宽 | D22（单元原子性怎么合成） 已定项 7 | 自己的 | 归 D22（单元原子性怎么合成） / D9（加密） | 不适用（自证） | 不带 | 只做 AAD 域分隔子 |
| 17 | journal 记录 | 记录 | D23（journal 的角色与格式） 已定项 1 的 A 条 + 已定项 4 的 487 字节头（含已定项 15 的新根段 364、fsid 8 与 MAC 16） | 自己的 | 归 D23（journal 的角色与格式） / D9（加密） | 不适用 | 不带 | 只做 AAD 域分隔子 |
| 18 | 系统配置槽 | 槽宽 | D22（单元原子性怎么合成） 已定项 9 | 自己的 | 占位，未定 | 不适用 | 不带 | 只做 AAD 域分隔子 |
| 19–31 | 保留给将来的自证结构 | | | | | | | 与 16–18 同一冻结时刻 |
| 32–255 | 保留；分配规则已定项 11 不定，任何日后分配都过 D15（格式冻结政策） 判定表 | | | | | | | |

- **D9（加密） 已定项 6 那四个名字怎么落**：数据 extent → 1；btree 节点 → 2；根记录 → 16；**记账单元 → 2**——D9（加密） 已定项 6 给标签的理由是「不同单元类的字段集合不同……可序列化成同一串字节」，而记账树节点与别的索引节点类身份段字段集合相同 ⇒ 域分隔在这一对上买不到东西，「记账」是树 ID 给出的角色。两者态别不同（记账是权威态，C102（索引节点头没有预留区，加字段不是 compat） 记着），态别不是 I-6.2（明文头字段白名单） 与扩展点的判据；若日后按态别把记账另立一类，按 D15（格式冻结政策）「改变已有字段含义」是 incompat 且码 2 不复用。D9（加密） day-1 第 ① 项的射程随值域一起收窄：收窄掉的只是「记账 / 非记账这个区分不许从树 ID 推」这一句——码 2 本身是显式的标签值，① 对其余各码原样成立。D22（单元原子性怎么合成） 已定项 3 依据里的「记账单元」从此是角色名（记账树节点），不是类名。**D18（块里携带什么信息） 已定项 8 的「墓碑」**：容器 = 码 3，记录 = 打包记录类型 1。自证单元也占码，是因为需要一个单射编码，同一命名空间是达成它最省的一种（1 字节对 4 字节 magic）。
- **标签的宽度与位置**：单元类型标签 = 共同前缀**偏移 6**那一个字节（flags 两字节各自独立定义，不依赖字节序）；**偏移 7 是 8 个 flag 位，第一版恒 0，读者遇到非 0 位一律拒收**——这是前向兼容政策，不是损坏检出（头校验和已经罩着它）：不给这 8 位留「忽略陌生位」的通道 ⇒ 将来给任一位赋义按 D15（格式冻结政策）「改变已有字段含义」判 incompat；它不是判定表第一行意义上的预留区（装不下字段），C102（索引节点头没有预留区，加字段不是 compat） 要写明这句。只对单元头的 flags 取这一侧；D8（核心索引结构） 已定项 6 的 inode 属性存在位是另一个位空间，那个两难已定项 11 不解。共同前缀 42 一个字不动。数据单元五元组首字节必须等于偏移 6，不等判损坏。AAD 规范编码里标签码是第一个字节，每一类都是。256 个值登记 7 个，余 249。
- **五元组 33 的拆分：标签 1 + 树 ID 8 + 对象 ID 8 + 对象出生代 8 + 锚点偏移 8**（[verification-build.md](../verification-build.md) 第 8 问关闭）。**备选没有论证过，用户按 33 总数与四个 8 的现有出处直接定案**：树 ID 4、出生代 6（48 位按 E44（序号位宽的本机实测与代价） 的 2785 发布/秒够用三千年）、锚点偏移按单元粒度 6，任一都会改 33，定案后不再是选项。**六元组的两个候选不在这次定案里**：`locality_id`（已定项 3 那条提议，用户定案不收）与出生快照维（D9（加密） 已定项 6 索引行欠的那次重开），都是 AAD 组成的永久变更，各归其分项。
- **打包记录单元（码 3）是独立的一类**：D21（权威态与派生态的分界） 硬约束 6（算不出来的权威值不许住可重建的容器）排除码 2；五元组的对象 ID 只写得下一个 ⇒ 排除码 1——墓碑指认的是死者，而**任何**索引都不再持有它（已定项 3 当时「同一份信息已经躺在明文反向索引里」那个前提随明文侧取消而消失，结论不靠它）。

**码 3 的类身份段（65 字节，头 107）**：

| 偏移 | 字段 | 宽 | 为什么 |
|---|---|---|---|
| 42 | 单元类型标签 | 1 | 偏移 6 的密文侧副本（I-6.2（明文头字段白名单） 补注要求白名单项在密文侧有权威副本）。⚠️ 码 2 今天没有这份副本，C111（类标签在密文侧无副本） |
| 43 | 出生树 ID | 8 | 对象号跨 keyspace 必然碰撞（D9（加密） 已定项 6）；一个容器只装一棵树的记录 |
| 51 | 打包记录类型 | 2 | 第二级登记表的码；一个容器只装一种 |
| 53 | 容器号 | 8 | 分配规则按打包记录类型分：类型 1（墓碑）树内单调分配（C103（容器索引无落点） 那套，回退不破身份）；类型 2（inode 记录）= 触发分裂的那条新记录的 inode 号，不要分配器（D8（核心索引结构） 已定项 6）。两类都是发布无关，容器被 COW 重写时不变；4 字节按 E44（序号位宽的本机实测与代价） 的 2785 发布/秒 17 天回绕 |
| 61 | 容器出生代 | 8 | 分配容器号那次发布的 checkpoint_txg；与容器号合起来是稳定身份，与数据单元的（对象 ID, 对象出生代）同一分工 |
| 69 | 记录数 | 2 | ≤ 2039（头 136 含预留位，已定项 16；最窄的记录是类型 5 谱系记录，宽 16，已定项 24） |
| 71 | 记录宽 | 2 | 登记表给、头里再抄一份：不认识记录类型的读者才判得了头合法性条件（D23（journal 的角色与格式） 已定项 1 的 B 条同形） |
| 73 | 诞生代号 | 8 | 只记这一次写（D18（块里携带什么信息） 已定项 7 翻回头里的三条理由） |
| 81 | fsid | 8 | D18（块里携带什么信息） 已定项 7 同理 |
| 89 | 载荷校验和 | 4 | CRC32C，覆盖偏移 107 到单元末尾（nonce / MAC 预留位 + 记录区 + 补齐）。整单元校验和 / MAC 住父指针（D4（校验和位置）），扫描认领时没有父指针，头校验和只是头完整性防线（E76（载荷校验和的判别力）：头落了载荷半落，只有头校验和判不出）。宽度挂 D23（journal 的角色与格式） 已定项 13 |
| 93 | 写序（实例代号 4 + 事务号低 48 位） | 10 | C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1，与码 1 同一字段；容器只在 checkpoint 的固定点写出，事务号取那次 checkpoint 里已分配的最大事务号；**码 3 上它只做同一 checkpoint 内的平局破除**（I-1.8（归并后版本全序） 对码 3 的全序键是 (诞生代号, 实例代号, 事务号)）；码 2 的写序已按已定项 12 收窄成 4 字节（只实例代号）、不进 P3、不进 I-1.8（归并后版本全序）；容器不单独加实例代号，写序里那 4 字节就是它（D8（核心索引结构） 已定项 6 那句由此重开） |
| 103 | 出生序号 | 4 | D19（块指针的结构与宽度预算） 已定项 6：码 3 的中央映射 key = 类标签 + 出生树（偏移 43）+ 出生 txg（诞生代号，偏移 73）+ 实例代号（写序前 4 字节，偏移 93）+ 出生序号；只做 key 的一段，不拿来择新；位置紧跟写序（C288（映射 key 定案的连带改动没落到别的决策） 第 ① 条取甲） |

- nonce / MAC / 算法类型那 29 字节从偏移 107 起，含预留位的码 3 头是 136（已定项 14 / 已定项 16）。
- **头校验和的覆盖范围（权威条目 I-2.4（头校验和覆盖范围））**：覆盖从偏移 0 到该类明文头最后一个字节，端点随 D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18 的字段表走（按现行初值：码 1 到偏移 104、码 2 到偏移 85 + 2 × key 宽、码 3 到偏移 106），头校验和字段自身按 0 参与；那 29 字节 nonce / MAC / 算法类型预留位不在头校验和覆盖内（I-2.4（头校验和覆盖范围））；加密开启后（系统配置级、全有全无，I-6.4（无未加密块））元数据类身份段进密文由 MAC 罩，头校验和的覆盖范围 = I-6.2（明文头字段白名单） 该类白名单里的全部明文头字段，不另列——**同一格式版本下它随系统配置的加密声明分两种，判别子是 D9（加密） 已定项 10 那格的加密位**。⇒ 第一版里载荷校验和落在头校验和覆盖内（D23（journal 的角色与格式） 已定项 13 的落点要求）。
- **声明长度口径与码 1 相同**：声明长度 = 头之后的净荷字节数，补齐起点 = 头 + 声明长度，声明长度 ≤ 32768 − 136（含 29 字节预留位的头，已定项 16）。
- **记录级不带字段**：未发布时撕裂由 I-1.2（块头写序已发布） 判成垃圾；已发布后载荷坏由载荷校验和（扫描路径）与父指针校验和 / MAC（常规路径）判；回收粒度 = 单元，活位没有对象；一容器一种定宽记录，类型与长度在头一份就够。**I-2.3（补齐字节为零且参与校验和） 射程扩到码 3**。判定顺序：头校验和 → 类身份段 → I-1.2（块头写序已发布） / I-1.3（块头树 ID 一致） / I-1.4（块头 fsid 一致） → 记录数 × 记录宽 ≤ 声明长度 → 载荷校验和 → 逐条记录，写在 I-1.7（打包容器合法与判定顺序）。
- **I-6.2（明文头字段白名单） 归元数据类的理由**：白名单分的是无密钥侧能读到什么，不是态别——记账树节点是权威态、今天就归元数据类；无密钥侧按 D9（加密） 已定项 5 只做检测与搬运，容器身份对它无用。
- **容器的 COW 重写与索引**：开放容器在它所属代际内有新记录的那次发布才被重写（已定项 24），新单元、新诞生代号、身份不变。**容器索引是打包单元指针头部（MAC / nonce / 算法类型）与位置提示的持有者，权威位置在中央映射（类型 4 实例表例外：它的物理指针由根记录直接持有，D22（单元原子性怎么合成） 已定项 7），别的树按身份引用容器**——更新一个容器的代价按打包记录类型分：类型 1（墓碑）= COW 它 + 改容器索引一条（容器索引独立存在，C103（容器索引无落点））；类型 2（inode 记录）= COW 它 + 改全部祖先（容器索引就是 inode 树的内部节点，D8（核心索引结构） 已定项 6），这是 inode 打包（D8（核心索引结构） 已定项 6）与 D14（双轨（大小文件 / 持久临时）） 已否的跨文件尾部打包的区别所在。旧版容器走普通 COW 释放路径，不写墓碑。容器索引独占的物理指针与 MAC 都是派生值（MAC = f(密钥, 密文, AAD)，可重算；重算后的认证力 = 对被扫载荷的信任，正是 D18（块里携带什么信息） 已定项 5 的降级语义，对容器索引与 extent 树里的数据指针一视同仁）⇒ 不撞 D21（权威态与派生态的分界） 硬约束 6。**码 2 不进扫描重建的择版本规则**：inode 树的内部节点与根是派生态，重建从叶重造（叶是码 3、自带载荷校验和，D8（核心索引结构） 已定项 6）；记账树的节点是权威态，但它的条目取幂等完整值（D8（核心索引结构） 已定项 1），多版**在条目级合并**（D5（快照 / 空间记账机制） 已定项 2：单调统计量取最大值、非单调统计量由条目自带的 seq 定「后者」），**不需要在容器级择版本**；单元类码不是态别的判据（D21（权威态与派生态的分界））。**扫描重建同时找到同一身份的多版容器时按 C113（扫描重建时多版单元的现行版本判定无输入） 定案的规则判**：先按实例表的已发布谓词滤掉未发布的版本（定案 P2），再在对该根可见的候选里取**该类的择新键**最大者（定案 P3：码 1 取写序，码 3 取 (诞生代号, 实例代号, 事务号) 三元——诞生代号打头，因为同一实例连着两个 checkpoint 都不分配新事务号时两版写序逐字节相同；事务号留在末位做同一 checkpoint 内的平局破除，固定点会迭代、同一容器在一个 checkpoint 里不保证只写一次），两版可以同时属于不同的头；常规路径仍由 D3（空间分配） 已定项 1 的谓词判。诞生代号不做择新键：写序买的是已发布性与时间线（已定项 1 的射程），同 txg 跨事务改同 key 时它顺带定序。容器每次发布重写一次，这一格的重量比数据 extent 大得多（1000 文件 / 100 次 fsync ⇒ 一个身份 100 个物理版本）。
- **四条装载纪律**：① 一容器一种打包记录类型；② 一容器一个代际——把已定项 10 对墓碑定的纪律推广到按代际回收的打包记录类型（今天只有类型 1；类型 2 的记录逐条死亡、容器由 B 树分裂 / 合并回收、一片叶活很多代，D8（核心索引结构） 已定项 6）（D2（RAID 条带策略） 写的粒度第 2 条）；代际不存字段，= 活快照列表（I-3.7（活快照可枚举））里包含容器出生代的那一段，**只在回收判定时用，扫描重建不用**；快照发布之前必须先关闭并写出当前开放容器（D16（发布语义） 已定项 7 同一条次序；同样只对类型 1——类型 2 的叶不是代际意义上的开放容器，快照只是共享它）；③ 一容器一棵树；④ 记录定宽，记录数 × 记录宽 ≤ 声明长度，在认领时判。
- **打包记录类型是第二级登记表**：0 无效（容器头里的 0 判损坏；同一段字节以身份引用形态出现在 inode 树内部节点条目里时，0 表示「子节点是码 2 索引节点」，登记在 D8（核心索引结构） 已定项 6 与 I-9.2（条目身份与子头相符），与容器头场景不共享判定路径）；**1 墓碑区间记录**（宽 40，字段表见已定项 24；一个 32 KiB 容器 815 条（记录区起点 136）；E83（墓碑的粒度） 与 E102（单元类登记表与打包记录单元） 已在第 2 次按 40 字节重跑，E84（打包墓碑的钉住量） 仍按 56 字节算，重跑欠在 C89（单元头收口的遗留重算） ⑤）；**2 inode 记录**（启用，D8（核心索引结构） 已定项 6：定长 140，一容器 233 条，容器就是 inode 树的叶）；**3 退役记录**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P5 / P6：容器合并时右半退役、重生时旧身份退役，记录 = 被退役的 (出生树, 类型, 容器号, 出生代) 四元组，不带写序；住自己的容器，容器号按类型 1 那套单调分配；字段表与 C84（墓碑单元的粒度没人定） 同一张表定）；**4 实例表**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2，字段表见「打包记录类型 4（实例表）是独立的一类记录格式」那一条）；**5 谱系记录**（宽 16，字段表见已定项 24）；6–65535 保留。类型 3 / 4 按 D15（格式冻结政策） 默认档 incompat，冻结前零代价。
- **类型 1 容器归写者树独占**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P6）：快照发布之前当前开放容器先关闭并写出（已定项 10 的装载纪律），克隆不把 origin 的类型 1 容器收进自己的容器索引，回收（重写）只由写者树做，别的头在扫描重建时靠祖先表读到它们；克隆头的类型 1 容器号从 1 起按树内单调分配，身份由出生树分开 ⇒ 类型 1 从不跨头 COW，跨头首次 COW 的重生只对类型 2。**别的头能靠祖先表读到它们，前提是分叉点快照引用的那一版永不释放**（D6（快照实现模型） 定案连带触发的第 2 处：分叉点快照不许销毁）——而 D6（快照实现模型） 的九条采纳条件第 5 条明写那是用一条功能缺失回避掉的、不是满足，所以「分叉点快照可销毁 / 可写头可销毁」是 C113（扫描重建时多版单元的现行版本判定无输入） 定案的一条前置，与它同时到期。代价：凡是在分叉点还活着、之后在 origin 被删掉的对象，它的 kill 记录永远回收不了（门永远开不了），墓碑容器只涨不缩——C105（打包容器按发布频率重写的写放大） 与 C106（开放容器数上界） 之外的第三笔账。
- **打包记录类型 4（实例表）是独立的一类记录格式，不套用「一容器一种定宽记录」的模型**：一片记录宽恒 88（链指针记录与行记录同宽），按首字节 `kind` 分两种——
  - `kind` = 0 行记录：`kind 1 | 实例代号 4 | 所选根的 checkpoint_txg 8 | 属于该实例的最大已施加事务号 W 8 | flags 1（第一版恒 0、非 0 拒收；bit0 退役、不回收——曾标回退行，以后不给它另赋含义，与单元类型码「用过不许回收」同一条纪律）| 预留 66`；
  <!-- format-const: INSTANCE_ROW_BYTES = 88 -->
  <!-- format-const: INSTANCE_ROW_RESERVED = 66 -->
  <!-- format-const: INSTANCE_TABLE_PAGE_RECORDS = 370 -->
  - `kind` = 1 链指针记录（恒为一片的最后一条）：`kind 1 | 有无下一片 1 | 位置指针 86（无下一片时清零占位；与根记录里的实例表单元指针同型，D19（块指针的结构与宽度预算） 已定项 7 / 8）| 预留 0`。打包记录定长，kind 1 与 kind 0 必须同宽（`INSTANCE_ROW_BYTES`）；预留归零之后这条记录没有余量了：指针再宽一个字节，`INSTANCE_ROW_BYTES` 就要跟着涨、一片的条数跟着变。
  - `kind` 不是 0 或 1 的记录：这一片解不开（表不可读），与行记录 flags 非 0 拒收、单元类型标签不在登记表里同形；链指针记录不在一片的最后一条、或最后一条不是合法的链指针记录（含说有下一片而位置指针头部有一段不是第一版的取值：MAC / nonce、算法类型、压缩算法码与压后长度、extent 偏移，第一版取值见 D19（块指针的结构与宽度预算） 已定项 3），同样解不开。检查是 I-3.14（实例表每片记录形态合法）。
  - **身份与容量**：四元组 (出生树 0, 类型 4, 容器号 = 片序号从 0 起, 出生代 0)；出生树 0 与 D5（快照 / 空间记账机制） 给「全池 / 无归属」行的约定同一个数；不属于任何树、不进容器索引，由根记录直接持有物理指针（D22（单元原子性怎么合成） 已定项 7）。一片 ⌊(32768 − 136) / 88⌋ = 370 条记录（含链指针，数据行 369；行数多于 369 时一片写满 369 行再开下一片，最后一片装剩下的；136 是含 nonce / MAC / 算法类型预留位的码 3 头，已定项 14 / 已定项 16）；mkfs 写出一片空表，记录数 1（唯一一条是「无下一片」的链指针记录）。
  - **行怎么写**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2）：按实例代号唯一、后写覆盖，只在恢复与实例切换时写（管理员回退是挂着时的一次向前发布，不写行，D23（journal 的角色与格式） 已定项 14），与第一个新根同一次发布——**每次可写挂载都写行**（实例 0 不写；写行那次发布是新实例的第一次发布，新实例的每一个根引用的实例表都已含 [max(所选根的实例, 1), 新实例) 的行（清扫发布删去的行除外）；写行之前不推抬 F 的空发布，写行那次的元数据走切换预留，D28（挂载期承诺量） 已定项 3）；每次写行 COW 重写整条链。
  - **恢复写的行**：恢复在所选根指着的那一版表上给 [max(所选根的实例, 1), 新实例) 每个实例写一行——所选根那个实例写 (i, **重放之后那个根的 checkpoint_txg**, **属于实例 i 的、被这次重放施加的最大事务号**)，严格介于所选根的实例与新实例之间的实例写 (i, 0, 0)——jsn = 实例代号 32 位 + 计数器 48 位（D23（journal 的角色与格式） 已定项 9），重放按 jsn 严格连续、断号即止 ⇒ 任何一次重放都跨不过实例边界 ⇒ **同一次恢复写出的那批行里，只有所选根那个实例的 W 可能非 0；同一次切换写出的那批行里，只有被照旧事务的写序主人的 W 可能非 0；其余恒 0**（限定在「同一次写出的那批」上：后来的恢复只给 [新的所选根实例, 新实例) 写行、不回头改旧行，所以表上允许有多行 W ≠ 0，它们来自不同次的恢复或切换）。恢复恒选最新可读且自证通过的根，它可以不是最新发布过的根（槽坏了就退一格，之后的记录接不上就一条都不施加、W = 0）；一条都没施加的写 (i, 所选根 txg, 0)。
  - **切换写的行**：实例切换（D23（journal 的角色与格式） 已定项 14 的注）同崩溃行，只是**所选根按 D23（journal 的角色与格式） 已定项 14 实例切换那一句取**——重新读盘择根，不取调用方内存里记的根（C458（实例切换取内存里的根，不重新读盘））；切换不重放，行只落在 [这个根的实例, 新实例)，每个实例一行、取 ①–④ 里第一条适用的：① 被重发的那次发布自己写的行原样重发（写行那次的恢复行与中间实例行）；② 被照旧事务的写序实例 k 写 (k, k 最后发布的根的 txg（没发布过根时 0）, 被照旧的、写序属于 k 的最大事务号)；③ 所选根那个实例写 (i, 这个根的 txg, 0)；④ 其余写 (i, 0, 0)——被重发的 checkpoint 里没有事务号非 0 的记录时 ② 一条都不适用。
  - **W 是精确的前缀**：事务号在实例内单调、记录按事务号顺序追加（第一版串行提交下恒成立），一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加（D23（journal 的角色与格式） 已定项 7 的注）。
  - **已发布谓词（全局）**：i_now = 挂载根的实例，单元写序 (i, n)、诞生代号 b——i < i_now 且无行 ⇒ 已发布；有行 (i, T_pub, W)：码 1 ⇒ b ≤ T_pub ∨ n ≤ W（在所选根里，或被重放施加），码 2 / 3 ⇒ b ≤ T_pub（重放之后那个根之后的固定点单元一律未发布，恢复实例重写它们；重放施加进来的那几次发布的固定点单元由记录里的新根段引用（D23（journal 的角色与格式） 已定项 15），按 T_pub = 重放之后那个根的 txg 判已发布；码 3 那 6 字节事务号不进已发布谓词——它只做同一 checkpoint 内的平局破除（I-1.8（归并后版本全序） 对码 3 的全序键是 (诞生代号, 实例代号, 事务号) 三元，C113（扫描重建时多版单元的现行版本判定无输入） 定案第九版），码 2 的写序已收窄成只有实例代号（已定项 12））；i == i_now ⇒ b ≤ 挂载根的 txg；i > i_now ⇒ 判损坏。「无行 ⇒ 已发布」只对同一时间线成立。
  - **回收**：行可删当且仅当整轮清扫对**池中每一个落点**都得出了判定（读成功并按行判、已抹头、已被覆盖）且其中没有该实例的未发布单元、那次清扫里没有任何一次读失败（读不到不等于不存在，扇区维；抹头之前那次重读也算：重读报设备错、或读回的不是那个头，这次清扫都不删，用户 2026-10-04 定，零轮），且全部设备在线（设备维），且根环里没有该实例发布的根——根环每个槽都读成功才算「没有」，读不出的槽按「可能有该实例的根」算（一个暂时读不出的根槽能让行被删，槽恢复后被抛弃的根回到回退候选集，而它引用的单元已被清扫抹头；代价是一个根槽持续读不出时行永远删不掉，C335（根槽持续读不出时实例表只增不减））。读成功的全零根槽与读不出同待遇（按「可能有该实例的根」算），这一次不删；其余的槽须自证过，否则重读一次，仍不过就这次不删。**删行落在一次专用的清扫发布上**：抹头全部 FUA 返回、过一道屏障之后才发这次清扫发布；读失败、设备不全不删。**次序是先判、后抹**：先判根环每个槽自证过、环里没有该实例的根，判过了才抹，不许先抹后判。**清扫发布的形状**：事务号 0、不进 W、实例表整链重写、不判挂载准入式子，一次删满足条件的全部行；不单列预留，清扫发布的落点被拒时这次清扫不删行，挂载照样交回可写。**谁跑、什么时候跑**：可写挂载末尾（写行、暖机、抬 F、删除续做之后，交回之前）判一次。开关关着时，实例表行数超过阈值就在这次挂载里立刻判、抹、发清扫发布，没超过不扫；开关开着时，交给挂着时的后台清扫，后台抹之前在同一串行段里重读那一槽的头，仍是候选行的未发布单元才抹。阈值是配置项，默认值另定（C690（清扫触发阈值的默认值没定））。**池中每一个落点怎么读**：先照认领规则（已定项 7）认单元，再跳过根环里任一可读根的账里仍分配的槽（低于现行 F（D16（发布语义） 已定项 1） 的根跳过它的账，用户 2026-10-04 定，零轮）；清扫认「是单元」只看头校验和，不比任何读盘判定更严。量词取「池中每一个」不取「该实例写过的每一个」：已抹头与已被覆盖的落点说不出它原来属于谁，按后者写的话要枚举的集合与要销毁的信息是同一份（被抛弃的根留在环里时，它的行是回退候选判据的输入）；zoned 第一版不回收、链长无上界（C121（zoned 上实例表链长无上界））。
  - **表不可读时**：只读挂载；扫描重建停在级 1——表没了就一条行都没有，全部旧实例的全部单元落进「无行 ⇒ 已发布」，被崩溃恢复抛弃的整段时间线也在内，所以不是记歧义。实例表单元自己不走 I-1.2（块头写序已发布） 的谓词：它的现行版本由根记录持有的物理指针唯一确定，指针指不到或校验不过就是表不可读。
  - **作废一个 checkpoint 必须换实例代号**：当前实例没有行，谓词对它只能问「诞生代号 ≤ 挂载根 txg」，重发同一个号一发布，诞生代号等于它的孤儿全部判已发布——这是失败表能封闭的依据（D23（journal 的角色与格式） 已定项 14 的注）。**实例代号**住系统配置（E100（系统配置的三段几何） 段四那 4 字节）。
  - **可写挂载的顺序**：先判这次能不能可写（可写设备数够 w 的下限 ∧ **独占打开池中过半的设备**——任意两个过半集合相交，每设备的独占打开才成为池级互斥；这条与它的代价（4 盘池只剩 2 块可写时只能只读）是 D2（RAID 条带策略） 已定项 13 定的，不是已定项 11 ∧ 实例表可读 ∧ 实例切换的预留拿得到 ∧ 这次挂载要发的写行与暖机在分配器的副本上预演取得到全部落点——预演里写行那次的释放核验报错，同样判这次不能可写；预演不做释放前读盘核，差别见 C542（取号前预演不读盘核，坏盘加旧根全被盖掉会烧号）），再取新代号 = max(**这次挂载独占打开成功的那个集合**里各系统配置的代号（每块盘两槽里全部自证过的槽——校验和过且 fsid 与本池相同）, 根环里全部根记录的实例代号) + 1 并写进**那个集合**里的每一份系统配置、**全或无**（写系统配置与数据单元写同一个取向：一次 I/O 错要重试到 T_retry 用尽才算失败；用尽后全或无判失败时**先把已经写出的那几份回卷成旧代号**（旧代号 = 取号之前这个集合里全部自证过的槽中最大的实例代号；回卷写发生在任何单元之前），回卷不成才只读挂载——回卷期间盘上出现的「同一集合内代号不等」由 I-7.7（系统配置实例代号不低于根环） 的第 ② 句判成立（较大的号没有任何根、记录、单元带着），不另开例外，下一次挂载按 max + 1 修复；取号之后那道屏障（D23（journal 的角色与格式） 已定项 16）在任一块盘上报错同样判全或无失败，不许只重发屏障就继续；取号的写入集合与 max 的取值集合取同一个——写「全部可见」而只独占一部分时，集合外那些盘写不写得进去不由这个挂载说了算，两个挂载能互相逼成只读。**过半是准入门槛，不是写入范围**：写入集合取独占打开成功的全部设备，只写过半会把集合外的健康盘落下、它们的代号从此是旧的，与真正错过取号的盘分不开。**「可见」= 独占打开成功且系统配置读得通**；I-7.7（系统配置实例代号不低于根环） 的第 ② 句限定到同一个集合），之后才动任何单元；只读挂载不取号；从 1 起、0 无效。
  - **取号之前逐盘核带不带所选那一版**（主 agent 2026-09-25 定，被攻过零轮；实现 `crates/singlefs-core/src/mount.rs` 的 `devices_without_the_selected_version`，实二七）：可写挂载核施加前缀之后那一版；交进来的盘有一块不带就在取号之前、任何写之前拒可写（`MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion`），只读挂载不做这道核、逐条试位置条目读所选那一版。「不带」两种：两个系统配置槽一份自证过的都没有（校验和过、fsid 与本池相同），即这块盘不「可见」；世代号最大那份系统配置的 (实例代号, journal tail) 小于所选那一版那次发布末条记录的 jsn，**并且**所选那一版有单元在这块盘上它的落点读不出或字节不同，或所选那一版就是择到的那条根、它的根槽归这块盘而那一槽读一次、重读一次都不是它（读不出、自证不过，或自证得过而 (实例, txg) 不同；拒成 `SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndItsRootNotInItsRingSlot`）；施加过 journal 记录得来的那一版崩在根槽 FUA 之前、哪块盘的根环里都没有它，不核根槽那一半。放行两种：只落后、单元都在、根槽那一半不缺（根落盘之后、轮换之前崩；取号失败回卷写回取号那一刻的见证值（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续）），这是正常崩溃状态；系统配置跟得上、只缺几份单元（单份坏，由 D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 的读盘核隔离）。挂着之后收盘表的每个入口（正常卸载、管理员回退、会话每次发布、抬 F）在任何写之前核三样：「可见」那一支（一块盘一份本池 fsid 的自证系统配置槽都没有就拒）；「可见」之后，每块盘自证槽里的本盘设备号等于盘表给它的身份；落后支——这块盘最新那份自证系统配置的 (实例代号, journal tail) 小于现行那一版末条记录的 jsn，**并且**（现行那一版有单元在这块盘上它的落点读不出或字节与这个进程验得过的那一份不同，或现行那一版的根槽归这块盘而那一槽读一次、重读一次都不是它），才拒（改法 C，与可写挂载的落后支同一判，「所选那一版」换成调用方持着的现行那一版；缺根那一半的重读照 D16（发布语义） 已定项 1「根槽这一次读坏」读一次、重读一次，单元那一半读不出直接记缺、不重读；只落后、单元都在、根槽里是它的不拒，只剩一槽自证、那一槽停在上一次轮换的盘由此不误拒）。后两样用户定，起因与依据在代码三方第二轮判决 `research/prompts/m2-closeout-code-r2-main-verification.md`「同池旧快照的盘经入口洗白」与「盘体对调的盘表」两格；落后支取「落后且缺单元」、不取「落后就拒」，照攻方原型 `research/prompts/m2-closeout-code-r2-opus-model/fix_prototype_swapped_or_behind.patch`（`research/prompts/m2-impl-r2-fixes-b-implementer-report.md` 第四节第 6 条）。会话「每次发布」按「每次用户改动」读（一次调用只核一次，在循环之前），循环里为空间不够推的每一串抬 F 自己再核一次三样、紧挨着重发之前，所以实际发生的每个写之前都核过——这是推的，没有用例在循环中途把盘换掉。三样都已实现：「可见」那一支见 `research/prompts/m2-impl-z3a-yi-implementer-report.md`；本盘设备号与落后支在 `crates/singlefs-core/src/mount.rs` 的 `caller_inputs_agreeing_with_the_disk`（正常卸载、抬 F、管理员回退；本盘设备号那一判是 `own_device_numbers_on_disk_differing_from_the_identities_handed_in`，落后支是 `devices_behind_the_current_version`，与可写挂载共用 `what_a_device_behind_the_version_lacks`：先 `units_of_the_version_missing_on_a_device_behind_it`，单元都在再 `root_of_the_version_not_in_its_ring_slot_on_the_device`）与 `crates/singlefs-core/src/mounted_session.rs` 的 `publish_user_change`（会话），用例 `crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs`。拒时报的成员：「可见」——正常卸载、抬 F 报 `MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`（`disagreeing_device_table` 带 `NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn`），管理员回退报 `RollbackError::CallerInputsDisagreeWithTheDisk`，会话报 `UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration`；本盘设备号——正常卸载、抬 F 报同一个 `MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`（`disagreeing_device_table` 带 `OwnDeviceNumberDiffersFromTheIdentityHandedIn`），管理员回退报 `RollbackError::CallerInputsDisagreeWithTheDisk`，会话报 `UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable`；落后支——正常卸载、抬 F 报 `MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`，管理员回退报 `RollbackError::CallerInputsDisagreeWithTheDisk`（带 `CallerInputsDisagreeingWithTheDisk::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`），会话报 `UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`。
  - **前提**：同一时刻只有一个可写挂载——由过半独占打开保证；多主机共享存储上两个可写挂载会算出同一个代号、谓词分不出它们，随 D9（加密） 已定项 8 那一类宣布不防。取号那一刻两路见证不完整可见时不额外拦截，与整卷回滚同格；I-7.6（根环区域落在互不相同的盘上） 只在 devs ≥ R 时让根环成为独立见证，第一版 (R = 3, devs = 2) 不成立。错过取号的盘分两类：**打开过又掉了、且缺号期间池里确实发布过**的，回归时整盘作废、只读到重同步完成；**从未被这次挂载独占打开的**，或缺号期间池里一次发布都没有的，不作废——它没有分叉的物理可能；第一版 2 盘、掉一块只能只读挂载 ⇒ 分叉盘回归不可达；D2（RAID 条带策略） 已定项 3 / 4 允许在线加盘 ⇒ 第三块盘一加就可达，C120（分叉盘回归的判定与重同步）在允许加第三块盘之前到期，与过半规则同时。
  - 它与 D23（journal 的角色与格式） 记录头的「类型」是两个命名空间，政策不同。
- **两条政策的权威记录点是 D15（格式冻结政策） 判定表**：新增单元类 ⇒ incompat；**新增打包记录类型默认也是 incompat**——D15（格式冻结政策） 判定表「新增记录类型（新 key 类型）⇒ compat_ro」那一行说的是 btree item 类型，打包记录类型不走它：跳过一个墓碑类记录会让已删 key 复活（E59（缓冲消息能不能只从单元重算）），跳过 inode 类记录让只读挂载空转，今天登记的两个类型都不满足「跳过它不改变任何只读答案」；只有登记某个新类型时能证明这一点才降到 compat_ro，且要定宽、加密后先密钥先 MAC 再整体跳过、不许回收。类型码是 AAD 一个字段的值不是组成。compat_ro 读者不许跑扫描重建。新读者读没有码 3 的旧镜像不需要任何位。
- **对三条下游各定了什么**：墓碑——已定项 8 那两句拆成三句（容器标签取码 3；记录取打包记录类型 1；死者由记录自己的字段指认）；inode 路 ①——打包记录类型 2 预留，更新代价估算写回 D8（核心索引结构） 已定项 6；抗老化——偏移 6 是类标签、偏移 7 已定成恒 0 + 非 0 拒收；D26（后台整理与放置回收） 已定项 4 日后若要用节点头的位做标签，偏移 7 已不可用，要么另开字段按 D15（格式冻结政策） 判档、要么占 incompat，已定项 11 不替 D26（后台整理与放置回收） 判。

<!-- format-const: PACKED_UNIT_HEADER_BYTES = 107 -->
**数字**（E102（单元类登记表与打包记录单元））：cap(107, 56) = 583（区间 [65, 120]）、cap(107, 140) = 233（区间 [9, 148]；头 107 在两个区间内）；nonce / MAC / 算法类型预留位落在头里（已定项 16），107 + 29 = 136 ⇒ 按 56 字节墓碑算容量 582（第一版墓碑记录定成 40 字节，容量 815，已定项 24）；E84（打包墓碑的钉住量） 仍按 91 字节头算（583），重跑欠在 C89（单元头收口的遗留重算） ⑤（E83（墓碑的粒度） 的那一半已在第 2 次按记录区起点 136、记录 40 字节重跑）——**120 不是格式约束，是 E83（墓碑的粒度） 的一个绝对值锚**。

**射程**：值域登记表管的是「码 → 类」与各类的判定落点，AAD 的字段组成仍归 D9（加密） 已定项 6（码 1 的五项对快照维是瞎的，那次重开没做；码 2 的 AAD 身份无落点，C107（索引节点的 AAD 身份无落点）；码 3 的 AAD 在 E90（树 ID 进 AAD 与跨头共享） 的两笔格式含义落地前不构成完整契约）。已定的两个 AAD 身体不等长是巧合，不是域分隔的依据。开放容器的写放大、开放容器数上界、哪些树的 key 死亡要写墓碑、扫描路径上码 1 / 2 的载荷完整性、容器索引的落点都没在已定项 11 里定：C105（打包容器按发布频率重写的写放大）、C106（开放容器数上界）、C109（哪些树的 key 死亡要写墓碑）、C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据）、C103（容器索引无落点）。fsync 交织的删除下开放容器每次发布重写一次——1000 文件每 10 个 fsync 一次 ≈ 6.25 MiB，是 E83（墓碑的粒度） 单次批量口径的约 50 倍（代价账，不定字节）。挂着之后收盘表的入口与可写挂载取号之前逐盘核的落后支都取「落后且缺那一版（入口是现行那一版，可写挂载是所选那一版）的单元」，代价是同一个取舍：那一版是零单元发布、或旧快照上那一版的单元一个不缺（落点读得出、字节相同）时，同池旧快照的盘不被拒；换来的是只剩一槽自证、那一槽停在上一次轮换的盘不误拒（`research/prompts/m2-impl-r2-fixes-b-implementer-report.md` 第四节第 6 条）。这个取舍放进来过的一形（C585（会话发布放进发布途中取下的旧快照））：发布途中（单元与记录已写、根槽 FUA 未写）取下的那块盘的旧快照换进来，它落后而现行那一版的单元一个不缺。改法 C（用户定）——落后且缺那一版的单元，或那一版的根写在这块盘上而那一槽里不是它，才拒——在四个入口与可写挂载取号之前同一判，已实现：这一形在任何写之前拒成落后支；可写挂载上只有所选那一版就是择到的根时才核根槽那一半（用户 2026-10-01 定）。代码三方 `research/prompts/m3-code-r1-main-verification.md`、`research/prompts/m3-code-r2-main-verification.md`、`research/prompts/m3-code-r3-main-verification.md` 三轮判过；三块盘的池今天 mkfs 不收，区域 2 落在第三块盘上的那一格等三块盘开线时补（`research/prompts/m3-code-r3-main-verification.md` 第四节）。实例表删行的清扫排在可写挂载末尾（写行、暖机、抬 F、删除续做之后，交回之前，谁跑、什么时候跑见「回收」一条）：连崩挂载留下的 (j, 0, 0) 行（恢复给严格介于所选根的实例与新实例之间的实例写的行）是这次挂载的写行那次发布写下的，同一次挂载末尾的清扫就判得到它，不用等下一次挂载取号。

**依据**：

- E102（单元类登记表与打包记录单元）：仓里四处清单与墓碑在登记表下全部映得到、映不到与一名多码都是 0，只拿 D9（加密） 已定项 6 那行当登记表则墓碑等名字落在未登记码上（C98（单元类型标签的值域没有登记位） 说的那个洞）；等宽身体去标签会撞、带标签不撞；五元组对装满的打包单元只指认得了 1 条，排除码 1；混装两种记录宽会错分、不混装 0；未登记单元类 incompat 静默漏 0、跳过政策全漏；码 3 按记录区起点 136 算，两个下游容量 cap(136, 40) = 815、cap(136, 140) = 233 与 kb 值对得上（第 2 次，墓碑记录 40 字节）。
- E104（扫描重建的现行版本判定）：实例表与已发布谓词——`recovery_own_txns_global_w` 世界（把一个全局量写给范围里的每一个实例，恢复实例自己的孤儿复活）钉住「只有所选根那个实例的 W 可能非 0」；`lost_root` 世界（T_pub 按字面取在飞 txg − 1）钉住码 1 那条析取；不看根环则被抛弃的根整批回到候选集，钉住回收的根环条件；写序买的是已发布性与时间线，`no_wseq` 臂钉住同 txg 跨事务改同 key 时它顺带定序。
- E85（单元头的字段表）：五元组 33 这个总数来自它的宽度假设（自陈「身份字段取 8 字节」），拆分从没被单独论证过；四个 8 各有独立出处但都不是为五元组定的（逐个出处在变更史）。
- E59（缓冲消息能不能只从单元重算）：跳过一个墓碑类记录会让已删 key 复活——新增打包记录类型默认 incompat。
- 三方论证：三轮九腿的原样输出在 `research/prompts/d18-item11-r{1,2,3}-*-output.md` 与 `research/results/d18-item11-r{1,2,3}-local.out`，提案三版在 `research/prompts/_d18-item11-{unit-class-registry,r2-amended,r3-amended}.md`（第一轮本地腿被字词损坏检查判红作废，第一轮口径是两条云端腿）；实例表的各条规则来自 C113（扫描重建时多版单元的现行版本判定无输入） 定案与之后的用户定案：写行与暖机（C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方）、中间实例行与切换的所选根（C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）、取号读法、回卷与屏障报错（C322（取号那一步的屏障怎么放没有条款） 三轮三方）、行宽 88（C304（实例表链指针记录装不下 83 宽的指针））；错过取号的盘分两类来自 C113（扫描重建时多版单元的现行版本判定无输入） 第八轮反推腿（`c113-r8 那一轮的腿报告（产物在 git 里）`）3.1 / 7.2。经过与原话在变更史。
- 用户指示收口（值域登记表、五元组拆分），原话在变更史。
- 「唯一持有者」改成位置提示的持有者、1019 / 815 条、类型 1 宽 40 与类型 5 宽 16：墓碑判决 `research/prompts/m3-tomb-r1-main-verification.md` 第三节 W1 两句字面、W2 字段表两行，`research/prompts/m3-delete-r3-main-verification.md` 第三节 G4-谱系住处那一行（类型 5 被攻过零轮）。
- 用户定案 2026-09-23：切换的所选根重新读盘择根（`research/prompts/c381-r3-main-verification.md` 第四节第 3 题），原话在变更史。
- 用户定案 2026-09-24：实例表的行一片写满再开下一片（原话在变更史）；无实验：与片数式子 max(1, ⌈行数 ÷ 369⌉) 直接对得上，没有可量的代价差。
- 用户定案 2026-09-24：取号之前按分配器副本上的预演判落点、写行那次的释放核验挪到取号之前、预演不读盘核那一格记欠账（实十八报告 `research/prompts/m2-followups2-implementer-report.md` 第三节 1–3），原话在变更史；无实验：照今天的实现写成条款，预演的代价没量。
- 用户定案 2026-09-26：管理员回退改成挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14），回退行（kind 0 记录 flags bit0）与「回退写的行」删掉，旧镜像靠换 incompat 布局身份位挡在挂载之外（D15（格式冻结政策） 已定项 4），原话在变更史；无实验：三方判决 `research/prompts/m2-rollback-forward-r3-main-verification.md` 第二节 K7 与辩方复核（回退实例永远不给自己写回退行，截断走不到是结构事实）。
- 用户定案 2026-09-27：挂着之后收盘表的入口在「可见」之后再核本盘设备号与落后支（代码三方第二轮判决 `research/prompts/m2-closeout-code-r2-main-verification.md`「同池旧快照的盘经入口洗白」「盘体对调的盘表」两格），原话在变更史；无实验：实现与用例在 `research/prompts/m2-impl-r2-fixes-b-implementer-report.md` 第三节第 1 行——`crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` 10 条：四个入口各一条钉旧快照、各一条钉盘体对调，只剩一槽自证四个入口都不误拒一条，攻方放开扫的 16 格一条；落后支取「落后且缺单元」的取舍在同一份第四节第 6 条。
- 用户定案 2026-09-28：C585（会话发布放进发布途中取下的旧快照） 取改法 C；用户定案 2026-10-01：可写挂载上的缺根一半留着、条款补句。原话都在变更史。无实验：改法 C 的判法与用例由代码三方 m3-code-r1 至 m3-code-r3 三轮判过（判决 `research/prompts/m3-code-r1-main-verification.md`、`research/prompts/m3-code-r2-main-verification.md`、`research/prompts/m3-code-r3-main-verification.md`）。
- 实例表删行与整轮清扫器：三方判决 `research/prompts/m3-ring-r1-main-verification.md` 第二节第 10 行与第四节第 4 条（I-3.8（实例表行唯一且低于挂载根） 草案的条件要整轮清扫，仓里没有清扫器；用户 2026-10-01 定现在做），`research/prompts/m3-ring-r2-main-verification.md` 第二节第 10 行（删行落在专用的清扫发布上、抹头先于它、读失败不删；(j, 0, 0) 行删不掉的一半成立），`research/prompts/m3-ring-r3-main-verification.md` 第二节第 9 行（读成功的全零根槽不当没有根、每槽自证过或全零否则重读一次仍不过就不删，两处收严零轮）。无实验：清扫器是条款，仓里零代码，数都在三方副本上。
- 清扫谁跑与什么时候跑、池中每一个落点怎么读、清扫发布的预留与空间不够、清扫的次序、写行一条加除外：判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第二节 S1 清扫五句各行与第三节 F1、F3、F6 行，用户定案（2026-10-03，同一份判决第六节 Q1、Q3、Q4 行），原话在变更史；开关开着那一支与两半合用都被攻过零轮，次序与认单元口径只收严；无实验：数在三方攻方副本上，清扫器仓里零代码。
- 清扫算账跳过低于现行 F 的根、抹头之前重读读失败这次清扫不删行：代码轮第三轮判决 `research/prompts/m3-root-ring-code-r3-main-verification.md` 第二节 K1-a、K1-b 两行（K1-a：无故障合法历史上正常卸载抬过 F 之后，F 之下旧根的账读不出，每次清扫都停下、行一条删不掉；K1-b：抹头之前那次重读读失败时，头不抹而行照删，删行之后剩一个孤儿；副本量过，主 agent 复跑对上）与第六节 Q1、Q2 两行；用户定案 2026-10-04，原话在变更史。无实验：数在攻方副本上量，没立实验号。
- 链指针记录说有下一片而位置指针头部有一段不是第一版的取值也算解不开：代码轮判决 `research/prompts/m3-batch10o-code-r1-main-verification.md` 第二节 K2-1 那一格（实现与 checker 对这一形口径不同，主 agent 读码坐实）与第五节 K2-1 那一行，岔路单 `research/prompts/m3-batch10o-code-r1-forks.md` K2-1 那一行；用户定案 2026-10-06，原话在变更史。无实验：口径差是读码判的，没立实验号。

**欠**：C98（单元类型标签的值域没有登记位）：门禁比对工装；C107（索引节点的 AAD 身份无落点）；C108（扩展点归属对打包记录单元无判定）：会红的检查；C111（类标签在密文侧无副本）；C103（容器索引无落点）；C105（打包容器按发布频率重写的写放大）；C106（开放容器数上界）；C109（哪些树的 key 死亡要写墓碑）；C113（扫描重建时多版单元的现行版本判定无输入）：检查仍欠；C597（实例表行只增不删，删行的条件没有条款）；C690（清扫触发阈值的默认值没定）；C121（zoned 上实例表链长无上界）；C120（分叉盘回归的判定与重同步）；C304（实例表链指针记录装不下 83 宽的指针）：会红的检查；C329（写行那次发布之前推抬 F 的空发布没有检查）、C330（中间实例那一行的 T_pub 取所选根的 txg）：检查仍欠；C89（单元头收口的遗留重算） ⑤：E83（墓碑的粒度） 按新头宽重跑。；C542（取号前预演不读盘核，坏盘加旧根全被盖掉会烧号）

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:352-370`（整段抄，未转述）**

```markdown
#### 已定项 12：码 2 的写序收窄成 4 字节，腾出的 6 字节给载荷 CRC 与预留

**定案**：码 2 类身份段里写序那 6 字节事务号去掉，改成「写序 4（只实例代号）+ 载荷 CRC 4 + 预留 2」，这三段合计仍是 10，已定项 12 本身不改头宽（码 2 今天的头见已定项 18）。腾出的 6 字节给两笔已登记欠账：载荷 CRC 4 还 C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据） 码 2 那一半，预留 2 按 D8（核心索引结构） 的三段政策恒 0、非 0 判该记录 EIO。**码 3 那 6 字节维持不动**（用户定案取「只改码 2」那一档）。

| 谁用到写序的哪一段 | 码 1 | 码 2 | 码 3 |
|---|---|---|---|
| 已发布谓词（定案 P2） | 实例代号 + 事务号 | **只用实例代号** | 只用实例代号 |
| 择新键（定案 P3） | 写序全段 | **不进规则** | (诞生代号, 实例代号, 事务号) |
| I-1.8（归并后版本全序） | 覆盖 | **不覆盖** | 覆盖 |

**射程**：同一条依据对码 3 逐字成立，但改它要动 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 与 I-1.8（归并后版本全序） 的码 3 全序键，那一步仍开着，记在 C163（固定点各轮写序相同使平局破除落空）。**预留只有 2 字节，C102（索引节点头没有预留区，加字段不是 compat） 只还得了一半**：C102（索引节点头没有预留区，加字段不是 compat） 自陈 flag 位「装不下字段」不算预留区，2 字节同样装不下大多数字段——它买到的是「有一个预留区」这件事本身，不是「以后加什么都 compat」。**空间不是依据**：砍 6 字节在码 2、码 3 两处都买不到空间（码 3 每容器记录数零变化，码 2 扇出至多多 1 格、树高全不变），定案取的是**把买不到东西的字节换成有登记欠账在等的字节**，不是省空间。两条被打回的判据不许再用：「今天零消费者 ⇒ 举证责任在留着那边」（回扫会一并砍掉已定项 7 的 nonce 与 MAC 预留位、D19（块指针的结构与宽度预算） 已定项 3 的 MAC 与 nonce 恒定占位、D8（核心索引结构） 已定项 8 的预留 32 字节；成立的是另一条：预留位的举证是「将来要用」，事务号的举证是「现在就买不到」，两者不同形）；「记账树里有不能由遍历重算的量 ⇒ 码 2 需要择版本」（多版记账行在**条目级**合并，不在容器级择版本——D5（快照 / 空间记账机制） 已定项 2 逐字「非单调统计量「后者胜」（后者由 seq 定），单调统计量取最大值」）。E73（节点 key 区间的扇出代价） / E97（记账与分配记录的条目编码） / E98（inode 记录与 inode 树的几何） / E103（inode 打包路 ① 的更新代价） 按旧三档头宽算的数归 C89（单元头收口的遗留重算） 第 ④ 笔与 C288（映射 key 定案的连带改动没落到别的决策） 第 ⑦ 条重算。

**依据**：

- E104（扫描重建的现行版本判定）：它买的唯一用途在规则的前提下不可能发生——源码自陈（`e104_current_version.rs:1128-1131`）「写序的事务号是 checkpoint 的属性、给不出全序」「真实形态是固定点一次」「那一维是模型的补丁不是规则的一部分」；I-1.2（块头写序已发布） 的谓词对码 2 只读实例代号，I-1.8（归并后版本全序） 逐字「码 2 不进这条」。
- 三方论证：原样输出在 `research/prompts/d18-item12-r1-{sonnet,opus}-output.md`、`d18-item12-r2-{opus-defense,sonnet-attack}.md`、`c163-r1-{opus,sonnet}-output.md` 与 `research/results/d18-item12-r1{,b}-local.out`、`d18-item12-r2{,b}-local.out`、`c163-r1{,b}-local.out`；过程与两轮背景材料自身的错在变更史。用户定案，原话在变更史；头宽加 4 之后按新数读原定案成立（C298（头宽加 4 改动了用户定案里的数））。

**欠**：C163（固定点各轮写序相同使平局破除落空）：会红的检查；C102（索引节点头没有预留区，加字段不是 compat）：剩下那一半（冻结前要不要加宽）。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:371-395`（整段抄，未转述）**

```markdown
#### 已定项 13：加密开启时码 1 的身份段与 `声明长度` 都进密文

**定案**：取臂庚——加密开启时，码 1 的五元组 / 诞生代号 / 写序**就地变密文**，且 `声明长度` 从三类共用的共同明文前缀里去掉、一并变密文。字段的位置与宽度一个都没动，明文头总宽仍 105（I-2.4（头校验和覆盖范围） 逐字，加密开启后覆盖范围是「白名单里的全部明文头字段」这个**字段集合**，不是连续前缀 ⇒ 不必搬字段）。I-6.2（明文头字段白名单） 的码 1 白名单收成 {fsid, 载荷 CRC}。**载荷 CRC 覆盖的是盘上字节（密文）**（13-c，口径写在已定项 7 字段表下方）。改新池新建文件的字节：**否**（D9（加密） 已定项 10 逐字「加密不进第一个可运行版本，只预留格式字段」，加密关闭时这几个字段仍在原位、仍是明文 ⇒ 新池新建文件写出的字节逐字节不变）。

**射程**：**泄漏清不到零，各条臂差的是量级不是有无。** 臂庚 把最重的一项（`声明长度` ⇒ 小文件精确大小）消掉，也顺带消掉码 3 的 `声明长度` ÷ 公开的记录宽（I-1.7（打包容器合法与判定顺序） 逐字「类型 2 是 140」）给出的容器记录条数；剩下的仍在：

| 仍在明文 | 泄漏什么 | 逐字依据 |
|---|---|---|
| **单元类型标签**（flags 偏移 6） | 数据单元与元数据单元各有多少 | D18（块里携带什么信息） 已定项 7 逐字「扫描器必须在解密前分出类才知道套哪张白名单」⇒ 结构上进不了密文 |
| **载荷 CRC 4** | 相等 ⟺ 同一份密文 ⇒ 副本数 `w`（第一版 `w` 恒 2，D2（RAID 条带策略） 已定项 9 ⇒ 这一格第一版无增量） | I-6.2（明文头字段白名单） |

树表表长不在明文里：树表住根记录间接层的树表单元里，而树表单元是密文（D22（单元原子性怎么合成） 已定项 7 / 12）。庚 的代价：它让单元类型标签**多了一个无密钥侧消费者**，「把类标签也藏起来」这条路因此更贵。已定项 7 的认领规则按整单元跳，与 E88（冒名单元与孤儿单元的判别） 量的「跳到本单元起点 + 单元大小」对得上。重建路径要解密才拿得到身份，完整 96 位 nonce 由已定项 14 放进单元明文头。

**依据**：

- E88（冒名单元与孤儿单元的判别）：它的源码里 `声明长度` / `decl` 出现 0 次、认领后按整单元跳 ⇒ 认领规则换掉零代价，不必重跑。
- 码 3 的判定顺序零增量：I-1.7（打包容器合法与判定顺序） 的七步里 `类身份段` 排第 2、`边界`（含声明长度）排第 5，码 3 的类身份段整段本来就是元数据类密文 ⇒ 第 5 步今天就要密钥。
- 无密钥 scrub 能力损失 0：D9（加密） 已定项 5 承诺的「检测损坏、报告坏块」由头校验和 + 载荷 CRC + nonce 代号兑现，三样都不消费 `声明长度`；补齐区损坏也照样抓得到——载荷 CRC 把补齐算在内。
- E28（明文映射层能不能只靠单元自描述重建）：臂丁（加密开启时不带身份段）的缺口 = 数据块总数，而泄漏面并不更小——出局。
- 用户定案「块头的泄漏不接受」（开这一项的依据）与取臂庚，原话在变更史；D18（块里携带什么信息） 已定项 3 让数据单元身份留明文时的辩护是「同一份信息已经躺在明文反向索引里」，D9（加密） 已定项 5 改取「只做 scrub」之后那一层没有了。
- 两轮三方论证：经过在 [records/2026-09-07-D18项3重开第一轮.md](../../../records/2026-09-07-D18项3重开第一轮.md) 与 [records/2026-09-07-D18项13第二轮.md](../../../records/2026-09-07-D18项13第二轮.md)；臂庚 在跑前写死的六条判据上字典序胜出（第 1 关严格胜，2 / 3 / 5 / 6 关全平，第 4 关只多 8 处措辞、没有一处是字节、AAD 五项不动 ⇒ 不碰 day-1 永久契约），各臂的判词在变更史。
- 13-c 载荷 CRC 覆盖盘上字节：正推腿与反推腿各自独立查到定案前是全仓空白，两条独立理由写在已定项 7；用户定案，记在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:396-409`（整段抄，未转述）**

```markdown
#### 已定项 14：nonce 代号就是完整的 96 位 nonce，住单元明文头

**定案**：**取臂甲**：nonce 代号就是完整的 96 位 nonce（12 字节），住单元明文头，指针里那份保留。码 2 / 码 3 的那组预留位（今天 29 字节，已定项 16）**紧接整个类身份段之后，排在出生序号后面**——码 3 从偏移 107 起，码 2 从类身份段末尾起；放在共同前缀之后（42 起）会让类身份段的偏移随加密开关变，那是 `.claude/rules/fs-design.md` 标 ✗ 的参数化布局。码 1 那 29 字节由已定项 16 定成紧接类身份段之后、算进头：码 1 明文头 105、含预留位 134，[layout/01-first-txn.md](../layout/01-first-txn.md) 按这一档写。改新池新建文件的字节：**是**（已定项 7 预留那两个字段位的理由逐字是「不预留则开加密成了布局变更」⇒ 位在第一版就占着盘上的字节）。

**射程**：重建路径上没有指针，而已定项 13 之后身份在密文里 ⇒ 不在单元自己身上放完整 nonce 就解不了密；它补的是这一跳。这一格还开着两处空白：I-6.1（nonce 不重用） 那条注记的前提（「checker 没有密钥」这个场景在 D9（加密） 已定项 5 改取「只做 scrub」之后还成不成立）没人回看过——它不改臂的选择，重建要解密，nonce 无论如何得在明文头里；以及单元头里的 MAC 与 D4（校验和位置） 那边内联在指针的 Merkle 字段是不是同一个 MAC 存两份，全仓空白。

**依据**：

- E117（预留位算进去之后的头宽与几何）：把预留位算进头之后扇出各掉 0–1 格、码 3 的每容器记录数一格都不掉；四个下游实验同日重跑，结论一条都没翻。
- 一轮三方论证：经过在 [records/2026-09-07-D18项14第一轮.md](../../../records/2026-09-07-D18项14第一轮.md)；另外三条臂各自出局——白化（丙）买不到东西（同一份镜像上已经有五处结构性的次序 / 分组泄漏，白化一处都碰不到）且杀死 I-6.5（nonce 水位单调）；删掉指针那份（乙）牵动至少 7 处已登记数字并把 I-6.7（副本共用 nonce） 从结构性自动成立变成要逐位比对的新检查，而它只买空间；重建不解密（丁）撞 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 的择新键全在密文里。各臂判词在变更史。
- 用户定案（「取甲，但先把头宽算清楚再定」），原话在变更史；头宽加 4 之后按新数读原定案成立（C298（头宽加 4 改动了用户定案里的数））。

**欠**：C184（重建路径拿不到完整 nonce）：门禁比对。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:410-422`（整段抄，未转述）**

```markdown
#### 已定项 15：已撤回·此编号不再使用

**定案**：撤回，此编号不再使用。原题「码 2 的择版本规则」：码 2 要不要、以及用什么输入（已有的诞生代号、实例代号、key 区间，或新加的字段）判出「根 R（txg T）引用的是盘上哪一版码 2」。它由用户在 C292（码 2 身份段零消费者的理由已不成立） 的四条岔路里取 ②（给码 2 补择版本规则）新立，三轮三方论证之后由用户撤回，回到 C292（码 2 身份段零消费者的理由已不成立） 的岔路 ① / ③ / ④；用户随后取 ①（「部分可以进权威态 如果需要的话」），D21（权威态与派生态的分界） 码 2 那一行拆成两行。撤回之后码 2 照旧不进择版本规则（已定项 12），码 2 头不加字段。已定项 15 名下保留的是三轮留下的结论，**不是现行设计**。

**射程**：三轮的结论：字面的两条候选（key 区间取栅栏语义、节点号加退役记录）在「父节点与子项的现行版一起丢、旧版还在盘上」这一格都会静默挑回旧版；唯一不给错答案的形态（栅栏加验收门）在被快照引用的 extent 内部节点上几乎永远过不了门，能过的写法要重开 D11（索引节点要不要留消息缓冲区） 已定项 6 与四处用户定案的宽度，而且还没有一条腿攻过它；补上择版本规则也不让池回到可写（级 2 的见证者没了，已定项 5）。**若将来重开**，起点是第三轮那个方向（逻辑校验和门加 extent 内部节点不留缓冲），先补三样：逻辑校验和对规范序列化后的字节算（载荷 CRC、头校验和、加密时的头 MAC 一起置 0）、ε = 0 的实测基线、根层级的载体；然后再走一轮三方论证。

**依据**：

- 无实验：撤回的是一条没被采纳的规则，三轮论证的判决是对条款与腿模型的判断，没有采纳的数。
- 三轮三方：核实与判决在 `research/prompts/d18-item15-r1-main-verification.md`、`research/prompts/d18-item15-r2-main-verification.md`、`research/prompts/d18-item15-r3-main-verification.md`，三条腿的原件在同目录；逐轮结论表在变更史。用户撤回，原话在变更史。

**欠**：C292（码 2 身份段零消费者的理由已不成立）：对账门禁。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:423-437`（整段抄，未转述）**

```markdown
#### 已定项 16：nonce / MAC / 算法类型预留位算进头，声明长度的原点是含预留位的头末端

**定案**：三类单元的 **29 字节**预留位（nonce 12 + MAC 16 + 算法类型 1）都紧接类身份段之后算进头：码 1 头 **134**、码 3 头 **136**、码 2 头 = 86 + 2 × key 宽 + 29 = **115 + 2 × key 宽**（D8（核心索引结构） 已定项 11 加自描述三段 5 之后）；声明长度的原点 = 含预留位的头末端，上界 32768 − 134（码 1）/ 32768 − 136（码 3）；已定项 11 的容量与 I-1.7（打包容器合法与判定顺序） 按 136 算：`⌊(32768 − 136) / 40⌋ = ⌊32632 / 40⌋ = 815`（第一版墓碑记录 40 字节，已定项 24）。**算法类型那 1 字节**：扫描重建路径上没有父指针，单元只能靠自己说清它是用哪种算法加密 / 认证的——与已定项 14 把完整 nonce 放进明文头的理由逐字相同（「重建路径上没有指针」）；不加的话，换过算法的池在扫描重建时只能逐码试解。登记表与系统配置里那份声明是同一张表（D9（加密），登记位 D22（单元原子性怎么合成） 已定项 17），单元头这一份是**每单元**的权威，系统配置那一份只说当前默认。第一版写 0（未加密）。改新池新建文件的字节：**是**——码 3 声明长度的上界与原点、码 2 条目区的起点都随它定，码 1 的载荷起点与码 3 的记录区起点跟着定。

**射程**：预留位里放什么、nonce 为什么是完整的 96 位归已定项 14；码 2 的完整偏移归已定项 18；那 29 字节不在头校验和覆盖内（I-2.4（头校验和覆盖范围））。

**依据**：

- E117（预留位算进去之后的头宽与几何）：已量过含预留位的头宽与扇出代价。
- E102（单元类登记表与打包记录单元）：第 2 次复出 cap(136, 40) = ⌊32632 / 40⌋ = 815。
- 已定项 14 定了 nonce 住单元明文头、C288（映射 key 定案的连带改动没落到别的决策） 第 ① 条定了它紧跟出生序号，缺的只是「算不算进头、声明长度从哪起」这一句。
- 用户按 [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) 第九节的选项逐条定案（弹窗问答，主 agent 记录），算法类型那 1 字节另一次弹窗定案；两次都没有走三方论证，主 agent 的推荐与用户的选择都可推翻。原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:438-451`（整段抄，未转述）**

```markdown
#### 已定项 17：32 字节头校验和字段放 CRC32C 4 字节 + 28 字节零；自证结构的校验和覆盖整个槽含补齐

**定案**：32 字节头校验和字段里放 CRC32C 4 字节 + 28 字节零；根记录自证校验和、系统配置整槽校验和、journal `header_csum` 同口径。自证结构（根记录、系统配置槽、journal 记录头）的校验和覆盖整个槽含补齐，校验和字段自身按零参与；补齐恒为 0。**写死的参数**：CRC32C = CRC-32C，Castagnoli 多项式 0x1EDC6F41，反射实现（输入输出都反转），初值 0xFFFFFFFF，输出取反；结果按小端写进 32 字节字段的**前 4 字节**，其余 28 字节恒 0，读者遇到非 0 一律判该结构损坏。全仓每一处 CRC32C（头校验和、载荷 CRC、位置条目里的 4 字节、自证结构的校验和）同一套参数，别处一律链回这里。**校验和算法标识进系统配置**：自举头段加 1 字节「校验和算法标识」，登记表 0 = 无效、1 = CRC-32C，第一版写 1（D22（单元原子性怎么合成） 已定项 9）；挂载时按这个字节选算法，与单元头里那 4 字节校验和的口径同一处登记。改新池新建文件的字节：**是**——每一个校验和字段的值与那 4 字节在字段里的位置。

**射程**：那 28 字节按 0 参与自己所在的校验和（I-2.4（头校验和覆盖范围）），守它的是「恒 0、非 0 判损坏」这条规则，不是另一个校验和。算法取的是 iSCSI / SCTP / btrfs / ext4 都在用的那一套（`crc32c` 硬件指令直接算出来的就是它），没有第二种通行选法。

**依据**：

- E144（头校验和算法的代价与判别力）：六条候选在头上单比特、双比特、随机改字节都零漏；SHA-256 比 CRC32C 贵得多，但绝对量都只是一次 fsync 的零头；CRC32C 是仓里已定过的函数。用户定向「与加密无关就便宜一点」，原话在变更史。
- D13（验证路线） 已定项 5 要 checker 与实现各写一份独立实现，而 D13（验证路线） 自己逐字写着「CRC32C 各写一份的前提是参数钉死（多项式、初值、输入输出反转、异或输出）：参数不钉死，两份独立实现会在健康镜像上给出不同结果，那是假红而不是独立性」——四个参数留白就买不到那条独立性。
- 用户按 [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) 第九节的选项逐条定案（弹窗问答），参数在总审核第二轮补齐，算法标识进系统配置另一次弹窗定案；都没有走三方论证，可推翻。原话在变更史。

**欠**：C309（32 字节校验和的算法没定）：checker 侧的独立实现与「实现与 checker 用同一个函数」的比对。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:452-476`（整段抄，未转述）**

```markdown
#### 已定项 18：码 2 的声明长度 = 条目数 × 条目宽，补齐区恒 0，码 1 / 码 2 各补一张带偏移字段表

**定案**：码 2 的声明长度 = 条目数 × 条目宽；条目区之后的补齐区恒 0；I-2.3（补齐字节为零且参与校验和） 的射程扩到码 2。码 1 与码 2 各补一张与码 3 同形的带偏移字段表——

| 类 | 带偏移字段表 |
|---|---|
| 码 1 | 42 标签副本 / 43 树 ID / 51 对象 ID / 59 对象出生代 / 67 锚点偏移 / 75 诞生代号 / 83 fsid / 91 写序 / 101 载荷 CRC / 105 预留位 29 ⇒ 明文头 105、含预留位 134 |
| 码 2 | 42 树 ID / 50 层级 / **51 key 宽 1** / 52 key 区间 2k <!-- format-const: INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT = 2 --> / 52 + 2k 诞生代号 / 60 + 2k fsid / 68 + 2k 写序 4 / 72 + 2k 出生序号 / 76 + 2k 载荷 CRC / 80 + 2k 预留 2 / 82 + 2k 条目数 2 / 84 + 2k 条目宽 2 / 86 + 2k 预留位 29 ⇒ 明文头 86 + 2k <!-- format-const: INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE = 86 -->、含预留位 115 + 2k |
| 码 3 | 明文头 107、含预留位 136（预留位 29 与另两类同口径；记录区从 136 起，声明长度上界 32768 − 136） |

- **key 宽字段在固定偏移 51**，排在 key 区间之前：扫描重建路径上没有父指针也没有树表，解一个码 2 节点先要知道 `k` 才知道 key 区间占多少字节；读者按 `[0, 51)` 定长前缀读出 `k`、再按 `86 + 2k` 定出整个头，一次解出。
- **码 2 的载荷 CRC 覆盖 `[86 + 2 × key 宽, 16384)`**，起点是**明文头末尾**，含 29 字节预留位、条目区与其后的补齐区；口径与码 1 的 105、码 3 的 107 同（三类都从明文头末尾起算，不从含预留位的头末尾起算）。条目区的起点是另一个数——含预留位的头末尾 `115 + 2 × key 宽`（D8（核心索引结构） 已定项 11），两者差的正是那 29 字节。
- 已定项 11 登记表的码 2 行补上出生序号 / 载荷 CRC / 预留 / 自描述三段。码 2 的完整偏移只登记在这一处，已定项 7 的索引节点段只写字段集合、不再存第二份数。
- 改新池新建文件的字节：**是**——六个码 2 单元的条目区起点、声明长度与载荷 CRC、key 宽字段的落点与头宽（`115 + 2k`：inode 树根 131、extent 163、分配记录 135、记账 159、映射 169、树表 131）。

**射程**：补齐区恒 0 与 I-2.3（补齐字节为零且参与校验和） 的射程是照码 1 / 码 3 的既有口径推的，没有第二种选法。码 2 头宽公式 `86 + 2k` 对全部树成立靠已定项 2（全部码 2 节点带 key 区间）。

**依据**：

- E145（码 2 自描述头与映射树 key 宽的代价）：自描述三段每节点多 5 字节、扇出与树高一格不掉；不自描述的头逐宽试探解得出登记树、解不出没登记的树。用户定向「码 2 可以自描述」，原话在变更史。
- key 宽挪到偏移 51：C325（码 2 的 key 宽字段定位它要先知道 k） 立的账（key 宽写在 `81 + 2k` 时它自己的偏移就依赖 `k` ⇒ 读者只能对 `k` 穷举试解）。
- 用户按 [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) 第九节的选项逐条定案（弹窗问答，主 agent 记录），偏移表的两处（key 宽挪位、预留位加到 29）另一次弹窗定案，码 2 载荷 CRC 的起点在总审核第二轮补齐；都没有走三方论证，主 agent 的推荐与用户的选择都可推翻。原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:477-502`（整段抄，未转述）**

```markdown
#### 已定项 19：字段值不值得带进块里：三条判据

**定案**：**判据已定，三条叠加，缺一不可。** 一个字段值不值得带进块里：

| # | 问什么 | 不满足的后果 |
|---|---|---|
| **1（必要）** | **它的更新周期与内容变化周期绑定吗？** 绑定 = 内容变它才变 | 解耦的字段每次变都独立触发一次 H 级全链 COW（COW 下地址反正要改、要重写到根，这是免费便车；解耦的字段搭不上）|
| **2（必要）** | **缺了它，哪一个具体的恢复/判定会从「可判」变成「不可判」？** | 答不上来就是永久格式债，样本见 D18（块里携带什么信息） 已定项 19「反面样本」 |
| **3（必要）** | **相对已有机制净增了多少？对哪一侧可恢复？** | 与已有机制重叠的字段边际收益趋近零；「有密钥才可恢复」和「无密钥也可恢复」是两回事 |

- **成本在每条记录上，不在块头上**：块头这一层的开销按「头 / 节点」的比值看是免费的，而记录格式里多 1 字节就可能让每条记录多付一整格（bcachefs 的 bkey 头多 1 字节 ⇒ extent key 从 8 字节变 16 字节 = 2 倍）⇒ **约束不在「头 / 节点」这个比值上，在「每条记录都要再付一遍」这件事上。**
- **反面样本：一个没买到失败模式改善的字段**——btrfs 的 `chunk_tree_uuid`，16 字节/块：一处写、一处 scrub 验、一处挂载时**自指**读取期望值，**常规读路径 `btrfs_validate_extent_buffer` 对它 0 命中** ⇒ **字段的判据不是「多一份身份信息更保险」，而是「缺了它，哪一个具体的判定会从可判变成不可判」。**

**射程**：⚠️ **判据 2 不是 `.claude/rules/fs-design.md` 那条「能不能把用错了变成挂不上」**——那条是为**格式分支**设计的挂载期二元判断；携带信息是运行期对单个对象的判断，两条射程不同，不许互相替代。由「现役修复工具修不了什么」倒推出的五个候选里，谱系重写序号、节点自带 key 区间、数据块自描述已答（已定项 1 / 2 / 3），另两个**未定**、没有立成分项：

| 候选 | 绑着哪个失败 | 状态 |
|---|---|---|
| **提交点身份 =（计数, 提交点 hash）**，不只是计数 | **单调计数在回滚 / fsck 之后会重复**，重复之后上一条时间线的残留块会冒充合法块（f2fs 把 checkpoint 的 CRC 塞进 `cp_ver` 高 32 位；UBI 的 `image_seq` 解决同类问题） | **未定** |
| **「我是搬迁副本」标记 + 副本专属数据 CRC** | UBI 第一方承认：两个 PEB 对应同一个 LEB 时**光看序号不够**，较新的那个可能是搬到一半掉电的——「this is just a guess, which might be wrong」；它的解法是 `copy_flag` + 只有副本才付的 `data_crc` | **未定**，与 D3（空间分配） 的常驻整理、设备替换直接相关 |

**依据**：

- 无实验：三条判据是字段准入的规则，各自的依据是现役实现的事实——块头一层的开销（btrfs `btrfs_header`、bcachefs `btree_node`）按比值免费、bcachefs 的 bkey 头多 1 字节就让 extent key 翻倍、btrfs 的 `chunk_tree_uuid` 常规读路径 0 命中；出处、字节数与原话在变更史，未在本项目验证。

**欠**：C18（解耦字段污染只读维度）：判据 1 的会红形态。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:503-521`（整段抄，未转述）**

```markdown
#### 已定项 20：已定要带的身份字段

**定案**：判据三条全过、已定要带的三样：

| 字段 | 缺了它哪个判定不可判 | 落点 |
|---|---|---|
| **自身逻辑地址 + fsid** | 错向写（ghost write）**无法检测**；只有块号不够，错向写可能落到**另一个文件系统的正确块号上**，所以位置信息**必须含文件系统标识** | I-1.1（块头自述逻辑地址） / I-1.4（块头 fsid 一致） |
| **generation（出生代）** | **丢写留下的旧块是自洽的**：块的代号与父节点指针里记的期望代号比对，才分得出「根本没写成」与「写错了地方」的块 | I-1.2（块头写序已发布） |
| **所属树 ID（出生树）** | 孤儿块**连损坏范围都判不出来** | I-1.3（块头树 ID 一致） |

**射程**：数据单元头里的「自身逻辑地址」由五元组表达（锚点偏移即它的逻辑地址，I-1.1（块头自述逻辑地址）），不放物理落点（已定项 3 的两条线）；各类单元头里的落点见已定项 7 / 11 / 18；自证单元对 I-1.1（块头自述逻辑地址） / I-1.3（块头树 ID 一致） 显式豁免（已定项 7）。它们防的是错向写与拿错块，不防丢失写（已定项 21）。

**依据**：

- E88（冒名单元与孤儿单元的判别）：没有 fsid 与诞生代号的头，冒名单元混进级 2、撕裂孤儿一个都清不掉。
- 现役实现的第一方原话：btrfs「the measures have to be done ahead of time」与 `fs/btrfs/disk-io.c:113-118`「This is how we detect blocks that either didn't get written at all or got written in the wrong place.」；XFS「without an owner field in the metadata we have no idea of the scope of the corruption」；全文在变更史，未在本项目验证。

**欠**：无。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:522-535`（整段抄，未转述）**

```markdown
#### 已定项 21：自描述的射程是错向写，不是丢失写

**定案**：**丢失写（lost write / phantom write）**——设备回报写成功但实际没写——留下的旧块**在块内层面完全自洽**：旧内容、旧校验和、正确的自身逻辑地址、正确的 fsid，逐项匹配 ⇒ **任何「这是上一版」的判定都需要外部见证者。这是定义性的，不是工程限制。** **见证者必须在不同的失败域上**：父节点若在同一次事件里也没写成，就要祖父，递归到根 ⇒ **根 / 系统配置是最终见证者**（也是 D16（发布语义） 的一条独立理由）。⇒ **抓它的信息必须在块外**，唯一的办法是父节点持有子块的校验和/代号，即 D4（校验和位置） 已定的 Merkle 树。**自描述的射程是「错向写 / 拿错块」，不是「丢失写」。两者不许混为一谈。** COW 下丢失写的形态是**新块写好了、父节点的更新丢了** ⇒ 父仍指向旧子块，而旧子块完全合法，**整棵子树静默回退到上一版** ⇒ 抓它的**不是块自己记的地址，是父指针里记的期望代号**——**D4（校验和位置） 的 Merkle 不是「因为块自己抓不到」，而是因为「期望值」按定义只能存在于提问方那一侧。**

**射程**：**设备 ECC 的作用域：不是「不够强」，是「不在讨论范围内」。** 设备 ECC 的作用域是单个 codeword 内部的位是否自洽，失败模式是**报错**，不是静默；而错向写与丢失写的共同结构是**盘上的位完全合法、ECC 完美通过**，错的是「这些位属于谁」和「第几代」。

**依据**：

- 无实验：定义性的——合法的旧版本与合法的新版本**在块内层面同构**；三条反例路径全部失败：带墙钟时间戳（旧块**合法地**带旧时间戳）、递增本地计数器与带版本历史哈希链（「期望计数」「期望链长」都是外部信息）。
- OSTEP §45.6 逐字（本机 PDF 抽取复核）：「Unfortunately, the answer is no: the old block likely has a matching checksum, and the physical ID used above (disk number and block offset) will also be correct.」；NetApp FAST'20 现场数据里 lost write 占 13.54%，靠 WAFL 的**块签名（身份 + 版本号）**发现，不是 SSD 的 ECC；未在本项目验证。
- XFS 的 `xfs-online-fsck-design.rst:945-946` 声称块里记的 ondisk address「enables the filesystem to detect lost writes」，与 OSTEP §45.6 直接矛盾，判 OSTEP 对（XFS 原地更新，地址上原本是同一对象的旧版本时地址校验通过）；两篇闪存可靠性综述（arXiv:1706.08642、arXiv:1711.11427）逐词计数，错向写 / 丢失写相关六个词全部 0 命中。原文与逐格对照在变更史。

**欠**：C7（过期块无检查）。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:536-547`（整段抄，未转述）**

```markdown
#### 已定项 22：要支持扫盘重建的信息必须显式写下，不折进校验和种子

**定案**：**凡是要支持「扫盘重建」的信息必须显式写下来；只有「只需检出误用、不需重建」的才可以折进种子。** 折进种子（ext4 把 `(fs uuid, i_ino, i_generation)` 折进校验和种子，零额外字节；dm-integrity 把扇区号混进 mac）的**代价是只能验证一个猜测，不能反解身份**——拿一个孤儿块问「你属于谁」，折进种子的方案答不出来。

**射程**：它定的是身份类信息写成显式字段还是折进种子；一个显式字段值不值得带，仍过已定项 19 的三条判据。

**依据**：

- 无实验：折进种子能不能反解身份是定义性的（校验和只能验证一个给定的猜测），没有可量的量；ext4（`fs/ext4/ialloc.c:1289-1293`）与 dm-integrity 的做法是现役事实，未在本项目验证。

**欠**：无。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:548-567`（整段抄，未转述）**

```markdown
#### 已定项 23：否决清单

**定案**：下列候选不带进块里：

| 候选 | 否决理由 |
|---|---|
| **子块记录「当前」父节点物理地址** | **时序上不可能**：COW 是子先写、父后写——子块下笔那一刻，它想记的东西还不存在。改成回填就违反「一次写完不回写」。这是逻辑矛盾不是工程麻烦 |
| **checkpoint 内联「本次共 N 块」** | **同类时序死锁**：N 只有 checkpoint 收尾才知道，而字段要嵌进期间写的每一个块 |
| **COW 前驱地址（版本链）** | **净值为负**：① 与 D5（快照 / 空间记账机制） 的 birth txg + deadlist 功能重叠；② 前驱块被物理复用后那个指针变成**误导指针**——沿它看到的是**别的对象的当前内容**；③ 它要求新增一条不变量去防它自己制造的坑 |
| **块内纠错码（ECC）** | **否决，理由是「两头都不划算，中间没有甜点区」**（不是「太贵」）。① 便宜档：Hamming 16 位纠 1 比特，4 KiB 块上开销 **0.05%**，**但它只修设备 ECC 已经修好的那类**；贵档：RS 修整个 512 字节扇区要 12.5%、修撕裂写丢的半块要 50%；而设备 ECC 覆盖不了的两类里，撕裂写要 50% parity、**错向写整块都是别人的数据，没有任何冗余可用**。② **全场唯一实例是 OCFS2 `metaecc`**（`fs/ocfs2/ocfs2_fs.h:408` 32 位 CRC32 + 16 位 Hamming，`blockcheck.c:405` 纠 1 bit 后重验 CRC），**只纠单比特、默认关闭**，其设计文档自陈动机是「Modifying it to this level of safety isn't really possible **without creating an entirely new filesystem**」——**这个动机对从零设计的本工程不成立**。③ 另查 13 个系统（含 IRON、CrookFS、NOVA-Fortis、EnvyFS），**块内 ECC 零命中**；NOVA-Fortis 显式评估过 ECC 然后拒绝 |
| **每块存墙钟时间** | 四家块头逐字段查过，**一个都没有**，全是 `generation` / `txg`——单调、可比、COW 提交有天然定义点。墙钟只在 ZFS 的 uberblock 出现一次，**全池一份** |

**射程**：否掉的是这五种字段形态。同批链（前后 = 同一次写出的那一批单元在分配顺序上的邻居）不撞否决清单（分配先于写，没有时序倒置），它是否值得带由已定项 4 另判（出局）。

**依据**：

- 无实验：前两条是时序矛盾（子先写父后写；N 只有收尾才知道），COW 前驱地址与 D5（快照 / 空间记账机制） 的机制重叠且被复用后成误导指针，块内纠错码与墙钟时间来自现役调研；出处在否决理由里与变更史，未在本项目验证。

**欠**：无。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:568-597`（整段抄，未转述）**

```markdown
#### 已定项 24：第一版墓碑：墓碑树、40 字节记录、谱系记录

**定案**：

1. **墓碑树**：一棵专用树，day-1 零根注册进树表；根恒为码 2，条目照 inode 树内部节点的 120 字节形态（分隔 key 容器号 8 + 身份引用 26 + 子指针 86），孩子是码 3 容器（打包记录类型 1 墓碑）；容器与根进中央映射，条目里的指针是位置提示；根满时长高，照 D8（核心索引结构） 已定项 6，实现要做这一步（用户定案 K-R3-3）；容器号从 1 起，下一个号 = 树里最大号 + 1，零根时 1（与已定项 11 里克隆头的类型 1 容器号从 1 起按树内单调分配同一条规则）。
2. **墓碑记录**：对象 ID 8 + 对象出生代 8 + 区间起 8 + 区间止 8 + 死亡 txg 8 = 40 字节；整对象写 [0, 2⁶⁴ − 1]；一个 32 KiB 容器装 ⌊(32768 − 136) ÷ 40⌋ = 815 条；死亡 txg = 意图那次发布的 checkpoint_txg。
3. **开放容器**：有新记录的那次发布才重写，不删除的发布照抄；满了才关；关闭路径加只供测试的容量旋钮，旋钮不落盘。
4. **回收**：第一版不回收墓碑，统计量 13、14（D5（快照 / 空间记账机制） 已定项 4）第一版不登记；10⁶ 次 unlink 的墓碑约 76.7 MiB（两份）。
5. **谱系记录**：一棵专用谱系树，day-1 零根注册，随现行那一版、只增不删；孩子是码 3 容器（打包记录类型 5，不复用已定项 11 登记过的类型 3）；记录 (T_rb 8, T_old 8) = 16 字节（T_rb 是回退那次发布的 txg，T_old 是 R_old 的 txg，读作「T_rb 的父是 T_old」），一个 32 KiB 容器装 ⌊(32768 − 136) ÷ 16⌋ = 2039 条；管理员回退那次发布追加一条；谱系树根满同样长高，容器号从 1 起，下一个号 = 树里最大号 + 1，零根时 1。
6. **登记**：树表 7 → 11 条（`deleted_inodes`、墓碑树、谱系树、logged_ops 零根，最后一棵见 D8（核心索引结构） 已定项 5），树 ID 水位 65544 → 65548（推的：四棵各占一个树 ID；树 ID 从 65536 起编，D8（核心索引结构） 已定项 11）；不加新 incompat 位，并进第一条布局线。

**射程**：第一版单头、无快照、无清扫。扫盘重建怎么读墓碑与谱系记录不在这里定，记欠账（K-W5b，与 C113（扫描重建时多版单元的现行版本判定无输入） 同组）。谱系树的载体、打包记录类型 5、16 字节记录被攻过零轮。

**依据**：

- E83（墓碑的粒度）：第 2 次（墓碑记录 40 字节、记录区起点 136）复出第 2 条的 815 条一容器与第 4 条的 10⁶ 次 unlink 约 76.7 MiB（两份）。
- E102（单元类登记表与打包记录单元）：第 2 次 cap(136, 40) = 815，与第 2 条的 815 对得上。
- 墓碑树、谱系树的其余数：三方腿的模型与副本原型上量的（墓碑判决、m3-delete-r3 判决），还没在入库装置上量过。
- 墓碑树：墓碑判决 `research/prompts/m3-tomb-r1-main-verification.md` 第三节 W1 载体那一行（两轮；`research/prompts/m3-delete-r3-main-verification.md` 第三节「墓碑第一轮一轮各行」那一行把孩子里的类型 3 退役删掉）；根满长高：用户定案 K-R3-3。
- 墓碑记录：墓碑判决第三节 W2 字段表、W2 死亡 txg 两行；死亡 txg 随用户定案 K-W5，m3-delete-r3 判决第三节「墓碑第一轮一轮各行」那一行。
- 开放容器：墓碑判决第三节 W4 开放容器那一行（两轮）；旋钮不落盘：m3-delete-r3 判决第三节 G6-容量旋钮那一行（零轮）。
- 回收：墓碑判决第三节 W3 回收那一行（两轮）；76.7 MiB 是 m3-delete-r3 判决第三节 G8-其余数那一行（40 字节、815 条一容器）。
- 谱系记录：m3-delete-r3 判决第三节 G4-谱系住处那一行（零轮）。
- 登记：墓碑判决第三节登记那一行（零轮；两棵树时水位 19 → 21，是那一份判决当时的树 ID 编号，树 ID 现从 65536 起编）。
- 用户定案 K-U2「T1：意图那次写整对象墓碑」、K-W5「谱系记录」、K-W5b「写侧随谱系落地，读侧记欠账」、K-R3-3「写根长高（推荐）」，原话在变更史。
- 容器号从 1 起、墓碑树与谱系树根满长高：`research/prompts/m3-blanks-r1-main-verification.md` 第二节容器号、墓碑树根长高两行（一轮）。
- 树 ID 从 65536 起编：用户定案 2026-10-01「丙 树 ID 整体挪开」，原话在变更史。

**欠**：C615（扫盘重建对挂着时的回退没有输入）、C616（谱系父链靠 txg 连号没有断言）、C617（删后回退留下的墓碑没有预付罩着）、C614（回退时墓碑树取错与删完对象无墓碑单镜像判不了）；C84（墓碑单元的粒度没人定） 还清、C103（容器索引无落点） 的类型 1 那半还清、C109（哪些树的 key 死亡要写墓碑） 收窄。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:1-4`（整段抄，未转述）**

```markdown
## D8 核心索引结构 —— 已定

D8（核心索引结构） 管核心索引长什么样：一套 btree 实现配多个独立 keyspace 与两个前端、节点大小与它的上限判据、key 布局、whiteout 删除与无界操作的分批、inode 树的形态与 inode 记录的盘上字段表、write buffer 条目的 `seq`、树表条目的字段表与树的种类码登记表、码 2 索引节点的载荷布局与 key 比较序、前端与节点消息缓冲区的衔接、同一次发布里同一个 key 的状态折叠。**不预设 B-tree**，调查见 [prior-art.md](../prior-art.md) 第五节。不管的：记账统计量的载体、key 与合并规则在 D5（快照 / 空间记账机制）；多可写头下 key 要不要加一维快照、快照的实现模型在 D6（快照实现模型）；消息缓冲区的 ε 布局轴在 D11（索引节点要不要留消息缓冲区）；单元头、单元类型码与打包记录单元的装载纪律在 D18（块里携带什么信息）；指针的字段与宽度在 D19（块指针的结构与宽度预算）；发布语义与 checkpoint 边界在 D16（发布语义）；格式冻结的层次在 D15（格式冻结政策）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:28-40`（整段抄，未转述）**

```markdown
#### 已定项 1：记账取幂等完整值，接受 key 数量上升

**定案**：**记账条目一律是「某个统计量在某一代的完整值」**，写入时整格覆盖；**不用增量 Δ**。合并规则按统计量分（D5（快照 / 空间记账机制） 已定项 3 第 2 条、登记表见 D5（快照 / 空间记账机制） 已定项 10）：非单调统计量「后者胜」（后者由 `seq` 定），单调统计量取最大值。取最大值同样幂等、可交换、可结合，本项的论证（幂等完整值胜过增量 Δ）不受影响。付出的是 key 数量上升（每个统计量 × 每一代各占一条）。

**射程**：它定的是记账条目 value 的语义；载体（记账住一棵独立 keyspace 的 btree、条目是「(统计量, 维度元组, 代) → 完整值」）在 D5（快照 / 空间记账机制）。⚠️ **key 数量上升的量还没测**：「接受」是对方向的定案，量出来之前不许拿它去推「记账占多少空间」。⚠️ **兜底**：若某一类账确实做不到幂等（本项不预判有没有这一类），那一类必须另有一条与运行时**不共享判定代码**的独立重算路径。⚠️ 它同时改变 D11（索引节点要不要留消息缓冲区） 代价 1 的成立条件——该代价的判据是「缓冲里放的是幂等完整值还是增量 Δ」，本项把答案钉在幂等那一侧 ⇒ **代价 1 在本工程不成立**；D11（索引节点要不要留消息缓冲区） 的代价 2（可重建性退化）不受影响。

**依据**：

- E14（I-3.1 的判别力：幂等完整值 vs 增量 Δ）：同一棵树同一负载注入四类语义 bug，幂等完整值形态下 I-3.1（已分配统计对得上） 全抓、零盲区，增量 Δ 形态下一条都没抓到——Δ 形态下账本身就是权威内容，checker 只能拿同一个累加函数重放同一份 Δ 日志，它在和自己比。
- 用户定案 2026-08-29：取幂等完整值，接受 key 数量上升（原话在变更史）。

**欠**：C15（write buffer 混入 Δ）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:41-56`（整段抄，未转述）**

```markdown
#### 已定项 2：节点大小 16 KiB，钉成常量

**定案**：**节点大小 16 KiB。mkfs 把 16 KiB 写进系统配置，实现按常量走，不按设备特性算**——格式里是一个常量，不是一条公式。
<!-- format-const: NODE_BYTES = 16384 -->

**射程**：⚠️ **登记 `format-const` 不等于冻结这个数**（2026-08-31 用户定案：「这个数需要可以调整」）。那条标记只保证 **kb 与实验源码写的是同一个数**，改它照常走 D8（核心索引结构） 的流程；它写进系统配置，仍然是 mkfs 参数。⚠️ **它现在只钉得住一部分源码**：门禁阶段「格式常量在 kb 与实验源码之间同步」（`.claude/gate.d/code-source-discipline.sh` 的 format-constants 格）的形态是 `const 名 = 整数字面量`，`e56_epsilon.rs` 的 `16 * 1024`、`e43_extension_point_budget.rs` 与 `e52_head_mechanisms.rs` 的数组形式都不认。⚠️ **16 KiB 是 CPU 缓存轴上的选择，不是设备 I/O 轴上的**：E7（离线索引 harness） 量设备 I/O 时说小节点两端都赢、方向相反，两者量的是不同资源，不矛盾。上限那一维归已定项 4；E1（节点大小 vs 可恢复性） 的恢复时间那一半一旦给出比 16 KiB 更紧的上限，本项重开。

**依据**：

- E20（指针变宽的 CPU 缓存代价）：唯一超出 L3 的工作集上 16 KiB 是最小点；同一实验里 40 字节的指针头部在 4 KiB 节点付出的相对代价远高于 16 KiB 节点，大节点吸收指针变宽；同一条曲线上 8 KiB 的反常六轮稳定且无人解释 ⇒ 在解释掉它之前不许把节点大小写成「按设备特性算」。
- E6（加密算法选型）：加密 / 校验单元的增益在 16 KiB 之后基本饱和。
- E73（节点 key 区间的扇出代价）：码 2 头带上 key 区间之后树高一格都没涨，16 KiB 不用重开。
- 用户定案 2026-08-30：16 KiB 钉成常量（原话在变更史）。

**欠**：C56（没登记的格式常量仍会漂）、C386（节点大小结论欠一次按最终指针宽度的复跑，没有编号）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:57-87`（整段抄，未转述）**

```markdown
#### 已定项 3：key 布局 —— 改名与索引分开处理

**定案**：**key = `(locality_id, inode, offset)`**，`locality_id` 在创建时从父目录继承，**改名时故意不更新**。offset 段是**文件字节偏移**（这个单元第一个字节在文件里的偏移，第 n 个单元从 0 起是 n × 净荷容量 32634）。

- 命名走 dirent 树（`parent, name` → inode），改名就是删一条加一条，**O(1)**
- 数据按 inode 编 key，与名字彻底解耦
- scan 局部性来自继承的 `locality_id`，改名后**局部性缓慢退化，正确性不受影响**
- 退化由 D3（空间分配） 已承诺的常驻后台整理修复，不是新增负担

**批量改名**：`mv 大目录 别处` 在这个设计下**只改一条 dirent**。三处会把它重新变成无界操作，必须一开始就堵死：

1. **`locality_id` 永远不许被改名更新。** 它是**提示，不是正确性的一部分**——可以是错的、可以过期。
2. **改名覆盖已存在的目标 = 一次删除**，必须走已定项 5（whiteout 删除）的 intent log。
3. **后台重新聚簇必须是增量、可中断、可续做的**，且随时可以停掉不做。

**推论（可当 review 判据用）**：**任何让 rename 触及后代的设计改动，一律否决。**

**inode 记录必须存一份 `locality_id`**：它既是数据 key 的首段、又永不随改名更新，而命名树的 value 只有 inode、数据单元头的五元组里也没有它，所以盘上只有 key 自己带着它——读者打开一个改过名的文件时必须有地方拿到它，否则改名之后数据只能靠扫描才找得到。落点是已定项 6（inode 记录的盘上字段表）；这个值按本项从父目录继承，而第一版没有目录，所以「没有父目录时取什么」也要一起定——第一版取 0，登记在已定项 6 的记录字段表。扫描重建时重编出来的 key 首段按 inode 记录 > 幸存 extent key > 0 给这个对象的全部 key 编同一个首段（2026-09-16 用户定案，D18（块里携带什么信息） 已定项 3 那条加字段的提议不收）。

**射程**：它管数据 key 怎么编与改名怎么不触及后代，不管命名树自己的 key（dirent 树按 `(parent_inode, name)` 编，见已定项 5）。scan 局部性不一定要住在 key 序里：可以住 key 序（全路径索引，改名是 O(子树深度)）、住分配器（把子目录的数据放到父目录附近）、住第二索引（仍然贵，且要多维护一份一致性），本工程取「出生时继承、改名不更新的 locality 前缀」（较好，改名免费）。⚠️ 「它是提示、可以是错的」准许的是**不信它的含义**，不是**不存它的位**：作为地址首段它的取值精确且定义性。⚠️ **E9（key 编码对遍历局部性的影响） 的遍历局部性数只在一个口径下成立，引用必须连口径一起引**：**交错创建**（`interleave`，老化的代理）**加上 `cache_leaves = 8`**；`locality_id` 的收益是**新树的收益**，不是稳态收益，改名强度的现实性未校准——曲线可用，绝对刻度不可用。⚠️ 全路径索引的遍历优势也量出来了，但**不许拿它去替 D9（加密） 泄漏面与 D1（数据可移动性 / 反向索引） 稳定性做判断**——E9（key 编码对遍历局部性的影响） 对那两样一个字没说。⚠️ 「每次改名平均重写约 2700 个 key」这个数**缺口径**（查不到是哪一档改名、哪个种子），标为未核实；同表里 ext4 Orlov 分配器「遍历内核树快 30%」也是未核实——kb 内无出处无口径，要用先补。⚠️ 三段 key 在 D6（快照实现模型） 取多可写头之后曾经不完整（两个可写头各写一次同一个 `(locality_id, inode, offset)` 会落在同一个 key 槽、后写覆盖先写），2026-08-30 D6（快照实现模型） 已定项 1 取「每头一棵自己的树」之后消解——每头一棵树之下三段 key 在树内唯一，**两个头不再落进同一个 key 槽**。⚠️ **nonce 那条义务仍在**：D9（加密） 已定项 4 已定 nonce 每 extent 存，**生成时仍必须保证两个可写头之间不撞**（候选形态是按头划分 nonce 空间），它不再由身份唯一性免费得到，否则 I-6.1（nonce 不重用） 从「由身份唯一性保证」变成「由生成器保证」而没人保证它；该义务是推论，未走三方论证。⚠️ 后台重新聚簇是只搬物理位置还是重编 key、重编时怎么分批、进度从哪续做，全仓没有定义。

**依据**：

- E9（key 编码对遍历局部性的影响）：同一棵目录树同一个 LRU 上四条臂对比，`locality_id` 前缀比按 inode 编 key 少发设备读，值得做；depth-first 创建时三条真实臂逐位相同，缓存装得下整棵树时四条臂全部相等、效应消失；收益随改名次数衰减到几乎为零。
- 机制论证（2026-08-25 定案时写下，原文在变更史）：全路径索引在本工程还要多付三条 BetrFS 不必付的代价——分隔 key 就是路径会把 D9（加密） 已定项 5 认定的敏感信息从一棵树扩到所有按路径编 key 的树、引用方身份是路径会让 D1（数据可移动性 / 反向索引） 的反向索引条目随改名全部失效、lifting 对 key 编码的额外要求与 D9（加密） 的 nonce 载体和 D6（快照实现模型） 的快照 ID 位置抢同一段 key 空间。
- 用户定案 2026-09-16：扫描重建时给一个对象的全部 extent key 编同一个首段（原话在变更史）。
- E157（并行线一两条条款的计数模型） 量出：定位一页的节点读取次数、树高与叶片数、换容量后字节区间还原那一格，两臂在全部取样点上逐点相等、分不开；唯一分得开的是 AAD 锚点那一格，文件字节偏移下 N 个 key 全算对、单元序号下只有 1 个算对。
- D9（加密） 已定项 6 逐字「锚点偏移 = 该 extent 首次写出时的对象内起始偏移，读者现算 `key.offset − ptr.extent_off`」，而同一条决策已定项 12 第 2 项写明 AAD 字段组成是 day-1 永久契约 ⇒ 条款本来就预设了这个单位。

**欠**：C346（后台重新聚簇没有定义）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:88-105`（整段抄，未转述）**

```markdown
#### 已定项 4：节点大小 —— 上限由恢复时间定，靠实验取值

**定案**：大节点省 seek、树更浅，但**重扫代价随节点大小线性放大**，所以**不照抄 128K–256K**。**节点大小换的是「恢复时间」，不是「数据安全」，目标线该定在恢复时间上。取值靠 E1（节点大小 vs 可恢复性） 定：在「重扫代价 ≤ 目标线」的前提下取查找代价最低的那档。** 三条约束：

1. **它是 mkfs 时的格式参数，不是挂载参数。** 自动选取只发生在 mkfs 一次。改它 = **重建索引**，不是重建文件系统（D21（权威态与派生态的分界） 已定索引是派生态、冻结第 2 层降级为本地缓存格式）。
2. **默认策略必须是确定性的、可查的**，且 mkfs 要把「为什么选了这个值」打出来。
3. **测试钉死的是「模拟设备的参数」，不是节点大小本身**——固定一组设备档位让自动规则照常跑，**连选取规则本身也一起被测**。要求只有一条：同一档位必须得到同一取值。

**射程**：⚠️ **判据不是「全盘扫描重建成功率 ≥ 目标线」**——索引坏的永久损失恒 0 ⇒ 成功率这一维与节点大小**无关**，拿它当判据夹不出任何上限。⚠️ **目标线本身仍未定**，它是一个恢复时间预算（多久之内要把被破坏的那部分重扫回来），本项自己算不出来；E1（节点大小 vs 可恢复性） 剩下那一半仍欠 mkfs、checker 与一个能定点破坏镜像的工具。⚠️ 前置：索引可从单元自描述重建——E28（明文映射层能不能只靠单元自描述重建） 实测**元数据侧成立、数据块侧今天不成立**（缺口恰等于数据块总数，欠 D18（块里携带什么信息） 已定项 3）；⚠️ **「改它 = 重建索引」取消的是定性，不是代价**：整棵树按新尺寸重建是一次满量级的全盘 I/O，只是不丢数据；而且这个取消有未满足的前置——数据块自描述补上之前，「等于重建文件系统」在数据块那一侧仍然成立。桥接依据是索引树重建与明文映射层重建**读同一份输入**（每个单元自带「我是谁、我属于谁」），⚠️ 这一步是推理，E28（明文映射层能不能只靠单元自描述重建） 只测了明文映射层那一个实例；D11（索引节点要不要留消息缓冲区） 「不能拖」的全部理由就是这半句。⚠️ **第一期只有字节主导那一端在射程内**：D12（目标介质） 已定项 5 定第一条布局线是纯 SSD ⇒ 结论不变，**支撑从两端变一端**；寻道主导那一端要等 HDD 布局线开出来才重新在射程里。⚠️ **节点大小的自动选取判据里必须含指针宽度这一项**（E20（指针变宽的 CPU 缓存代价）：大节点吸收指针变宽，方向与重扫代价相反），此前没有。⚠️ **「按设备特性取相反方向」这个理由不成立**：两端最优方向相同，都偏小；节点大小仍是 mkfs 参数，要保留按介质调档需另找依据。

**依据**：

- E29（坏一个节点的爆炸半径）：索引坏的永久损失恒为 0（D21（权威态与派生态的分界） 已定索引是派生态、丢了只是慢），数据单元坏的损失与节点大小无关，只有重扫代价随节点大小线性放大（第 1 层节点的重扫量恰等于扇出）——所以上限只能定在恢复时间上，「爆炸半径」这个说法被它纠正。
- E7（离线索引 harness）：**固定内存预算下大节点三个轴全都更差**（量的是 io/op、字节/次、点查读/次），机制是节点越大、同样的内存装得下的节点越少，命中率掉得比「每次读更多」赚回来的更快；寻道主导那一端方向相同，大节点仍差 ⇒ 「不照抄 128K–256K」在字节主导与寻道主导两端都得到支持。
- 用户定案 2026-09-06：口径改写成「上限由恢复时间定」（原话在变更史）。

**欠**：C386（节点大小结论欠一次按最终指针宽度的复跑，没有编号）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:106-151`（整段抄，未转述）**

```markdown
#### 已定项 5：whiteout 删除 —— 接受，但批量删除必须走 intent log

**定案**：删大目录、截断大文件会产生海量墓碑，塞进一个事务必然爆掉。**做法（bcachefs 已验证的形态）**：凡是**可能修改无界数量 extent** 的操作（truncate、collapse、删大目录），**先把意图写进 logged_ops 树，再分批推进**；崩溃后恢复时续做未完成的意图。删除中的 inode 单独进 `deleted_inodes` 树，由后台回收——用户的 `rm` 返回，不等于所有 extent 都已释放。第一版 unlink 不往 logged_ops 树写条目：意图就是 inode 记录 nlink = 0 加 `deleted_inodes` 那一条（已定项 16）。logged_ops 树在第一个文件版本那次与 `deleted_inodes` 一起零根注册，字段表、回退时取哪一版、态别留到第一个写者（整理，D26（后台整理与放置回收） 已定项 2）落地时定（C364（整理意图记录没有字段表））。意图分两类：**承诺类**（unlink；truncate、删大目录）必须续做到底，可写挂载交回之前走到空；**提示类**（整理）可暂停、可在停机谓词下提前收、交给盘那条线上不续做，不挡挂载；续做入口按树分、共用发布路径。本条第 2 条与三态表、C1（意图续做做成常规路径） 的字面今天按承诺类写，整理落地时按类分开写（C621（整理落地那天要动的已写回删除条款没有清单之外的检查））。

**硬要求**：无界规模的操作一律不许做成单个事务；意图必须**幂等**；whiteout 只在节点重写合并 bset 时清除，**不许有别的清除路径**。

意图本身就是一条普通 key，骑在同一个事务层上，断电后只有三种状态：

| 断电时机 | 恢复后看到 | 怎么收场 |
|---|---|---|
| 意图提交之前 | 没有意图 | 操作从未发生，用户那边也没返回成功 |
| 意图已提交、工作做了一半 | 意图还在 | 恢复时**续做**，已做完的部分被幂等地重跑 |
| 工作做完、意图未删 | 意图还在 | 续做一遍发现无事可做，然后删掉意图 |

不可能出现「意图没了但目标状态没达成」——**删除意图必须与最后一批工作在同一个事务里提交**。

1. **意图声明目标状态，绝不声明增量。**「把文件 X 截到 N 字节」可以重复执行，「再删 1000 个 extent」不能。
2. **续做是每次恢复都跑的常规动作，不是修复手段**（否则会出现「平时不跑、真出事时才第一次跑」的代码路径）。
3. **中间状态必须每一个都是合法的文件系统状态**——「删了一半 extent 的文件」本身就得合法。
4. **一批里每个动作施加时先核它要删的东西还在**（随 D5（快照 / 空间记账机制） 已定项 12）——幂等的字面：一批按批开始时的状态为几个意图各算一份计划、施加时不复核，两个区间重叠的级联会把同一块放两次，统计量减两次、I-3.1（已分配统计对得上） 红。

**范围重建 vs 逐项修改：两条都要有，且要有显式判据**（`.claude/rules/fs-design.md`「不追求万能方案」）：

| 走法 | 代价 | 适用 |
|---|---|---|
| 逐项修改 | K × 随机访问 | 受影响的 key 稀疏散布 |
| 范围重建 | N × 顺序访问 | 受影响的 key 在 key 序上连续 |

**真正的开关不是 K/N，是「受影响的 key 连不连续」**：连续 ⇒ 范围重建可整节点丢弃或重写；不连续 ⇒ 只能逐项。**由此得到一条布局规则：让一起死的东西在 key 序上挨在一起。**

- dirent 按 `(parent_inode, name)` 编 key → 删整个目录是范围操作
- extent 按 `(inode, offset)` 编 key → 截断是范围操作
- 同一快照的 key 带同一 snapshot 前缀 → 删快照有机会变成范围操作

**这顺带从结构上解掉了 whiteout 膨胀**：范围删除直接丢弃整个节点，不用插 N 个墓碑。

**射程**：朴素的交叉点是 `K/N > 顺序代价/随机代价`（机械盘约 1%、SSD 约 10%）。⚠️ 这两个数是**推理，不是实测**，说的是**耗时**，要靠 E4（范围重建 vs 逐项修改的交叉点） 量，那一半要 QEMU。⚠️ **第一期只取 SSD 那一档**（D12（目标介质） 已定项 5：第一条布局线是纯 SSD）。⚠️ 写放大那一半量到的交叉点，自变量是「布局」与「节点大小」而不是介质，不能拿它去证伪「机械盘约 1%、SSD 约 10%」——那说的是耗时。按 `.claude/rules/fs-design.md` 的五条要求，这个分支还欠：判据写成可计算的式子、留一个能强制走指定分支的测试开关、两条分支各自跑崩溃点重放、运行时能看出走了哪条。⚠️ **切换判据里不许出现「当前填充率」这类操作中途会变的量**——判据只能用操作期间稳定的输入。**与 D3（空间分配） 的交互（容易漏）**：意图未完成期间那些空间还没真的释放，空间统计必须把「待删除但未完成」计进去，否则会出现「我删了一大堆，`df` 却不动」。

**依据**：

- E30（交叉点的写放大那一半）：**写放大那一半已量**——在写放大口径下量出条目连续与条目散布两种布局在 4 KiB / 16 KiB 节点上的交叉点，两者相差两个数量级以上——开关是布局与节点大小，不是介质。
- 用户定案 2026-09-16：加第 4 条硬要求「一批施加时先核要删的东西还在」（原话在变更史）。
- 三方 `research/prompts/m3-intent-r1-main-verification.md`：意图分承诺类与提示类（两腿独立得出，一轮）；logged_ops 树现在零根注册是用户定案 K-I1（原话在变更史；被攻过零轮）。

**欠**：C1（意图续做做成常规路径）、C2（无界操作做成单事务）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:152-224`（整段抄，未转述）**

```markdown
#### 已定项 6：inode 树的叶是码 3 打包记录单元，记录定长 140，已发布的 (inode 号, 对象出生代) 不复用
<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 P6 P7 P8 P9 -->

**定案**：inode 树 = 码 2 内部节点（派生态，不留消息缓冲区，条目 120 字节、扇出 135）+ 码 3 打包记录单元做叶（打包记录类型 2，权威态单元，一叶 233 条 140 字节记录）；根恒为码 2；容器索引就是这棵树的内部节点；已发布的 (inode 号, 对象出生代) 不复用，下一个号是记账里一条取最大值合并的统计量。码 2 根装满时长一层——新根 1 个码 2、分出的右半 1 个码 2（右半没有容器身份），inode 树与墓碑树（D18（块里携带什么信息） 已定项 24）共用这一条；长高那次发布多 2 个 16 KiB 节点 × 副本数。根长高之前今天的代码在 `crates/singlefs-core/src/inode_tree.rs` 的 `MoreLeafContainersThanOneRootNodeHolds` 拒（推的单元数，零轮）。

| 部件 | 是什么 | 态别 |
|---|---|---|
| 叶 | 一个 32 KiB 码 3 打包记录单元（D18（块里携带什么信息） 已定项 11），打包记录类型 2，装 233 条 140 字节记录；自带类身份段、记录数、记录宽、载荷校验和，扫描路径上的载荷完整性证据（C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据） 对码 2 的洞）对它不存在 | 权威态（单元） |
| 内部节点 | 码 2 索引节点。头带 key 区间（min / max 各 8 字节，D18（块里携带什么信息） 已定项 2 留给 inode 树的那一格由此定「带」：理由与记账树同一条——一个节点单独捡起来能自证覆盖段，key 是 inode 号不是文件名、敏感性论证够不到）。**头是块头，落 D15（格式冻结政策） 第 1 层（冻结），与记账树节点头同一格（C102（索引节点头没有预留区，加字段不是 compat））；载荷布局（条目 120、ε = 0）落第 4 层，不永久** | 派生态，丢了从叶重建 |
| 根 | 恒为码 2 索引节点，哪怕树只有一片叶，哪怕树是空的（零条目，key 区间写 [0, 0]，0 是无效 inode 号）。理由：D19（块指针的结构与宽度预算） 的指针不带被指对象的类与大小，树表指针指向的根若类会随树长大而变，读者按 16 KiB 读 32 KiB 是静默误读 | 派生态 |
| 容器索引 | 就是这棵树的内部节点，不另立树：物理指针只住内部节点，dirent 树引用的是 inode 号。更新一条记录 = COW 叶 + 全部祖先（D23（journal 的角色与格式） 已定项 1 甲） | — |

**内部节点条目 = 分隔 key 8 + 身份引用 26 + 子指针 86 = 120 字节**（扇出按今天的头 131 算 ⌊(16384 − 131) / 120⌋ = 135）。身份引用就是 D18（块里携带什么信息） 已定项 11 给「别的树引用容器」的那 26 字节：出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8。**类型段兼作「子节点是什么」**：0 ⇒ 子节点是码 2 索引节点（容器号、出生代两段恒 0；登记表里的 0 在容器头里仍是无效，两处含义靠所在结构区分），2 ⇒ 子节点是码 3 容器（读两个 16 KiB 槽）；其余值判条目损坏。合法集合按树分：inode 树 {0, 2}（I-9.2（条目身份与子头相符））、墓碑树 {0, 1}（I-9.23（墓碑树条目身份与子头相符））、谱系树 {0, 5}（I-9.17（谱系树条目身份与子头相符）），三棵树的内部条目同一形态。读者由此知道读几个槽；码 3 那条边的 AAD 五段（标签由类型 ≠ 0 推出，其余四段照抄条目）全部来自查找路径，D9（加密） 的 A6（AAD 期望值不来自明文侧）在这条边上不需要树的种类；码 2 那条边（内部节点指内部节点）的 AAD 仍是 C107（索引节点的 AAD 身份无落点）。出生树逐条目携带，是因为克隆头（D6（快照实现模型） 已定项 1）共享 origin 的叶，同一个内部节点里会混着出生树不同的条目；D19（块指针的结构与宽度预算） 已定项 7 让子指针头部也带出生树之后，身份引用里的出生树**照旧保留**——两份并存，加密开不开都占位、不挪作他用；类型段 0 条目身份引用里的出生树恒 0，与容器号、出生代两段同形；码 3 那条边 AAD 的出生树取身份引用那份，不取子指针那份。**分隔 key** = 该孩子创建时的最小 key，条目按分隔 key 单调递增，查找取最后一个分隔 key ≤ 目标 key 的条目。
<!-- format-const: INODE_INTERNAL_ENTRY = 120 -->

**内部节点不留消息缓冲区，inode 树的 ε = 0**——这是对 D11（索引节点要不要留消息缓冲区） 已定项 6 的树轴增补（布局轴「全部布局共用同一组 ε」不动）。分支变量是 D11（索引节点要不要留消息缓冲区） 已定项 4 那条谓词：**这棵树待下刷的消息能不能只从权威态（单元（带写序）+ 实例表 + 祖先表 + 墓碑单元）重算**。能 ⇒ 留缓冲（ε = 0.65）；不能 ⇒ 不留。inode 树不能：属性更新（mtime / nlink / mode 的新值）在到达叶之前没有权威态载体，缓冲进派生态节点就撞 D21（权威态与派生态的分界） 硬约束 6。extent / dirent / 反向索引能（E59（缓冲消息能不能只从单元重算） 的「单元 + 记账」一行三类全 0）；记账树按 D21（权威态与派生态的分界）「离线可由遍历重算」判能，但 E59（缓冲消息能不能只从单元重算） 那一列是神谕、未实测。它落 D15（格式冻结政策） 第 4 层（派生态），不是格式分支。**态别按重算性判，不按「树」这个词判**：一棵树的内部节点可从它的叶重算 ⇒ 派生态；叶若装着算不回来的值 ⇒ 权威态。记账是同一条判据的已定实例，权威记录点在 D21（权威态与派生态的分界）。

**装载纪律按记录类型分**：「一容器一个代际」与「快照发布前先关闭开放容器」只对按代际回收的打包记录类型 1（墓碑）成立；类型 2 的记录逐条死亡，容器由 B 树分裂 / 合并回收，一片叶活很多代、反复重写，快照只是共享它。类型 2 的记录死亡不写墓碑，死亡由现行叶版本体现，择现行版本压在 C113（扫描重建时多版单元的现行版本判定无输入）。

**容器身份不需要分配器**：inode 号在每一条时间线上单调（管理员回退的新根取 R_old 那一版的 inode 树、水位取 max(现行那一版内存里的, 环里读得出的根的)；崩溃恢复回到所选根的树与它的水位；两种之后发的号都大于那棵树里每一个 key）⇒ 插入永远落在最右的叶 ⇒ 分裂只发生在最右叶，**分裂点取末尾**——左半保留全部原有记录与身份，右半从触发分裂的那条新记录起，**右半容器号 = 那条新记录的 inode 号**，出生代 = 本次发布的 checkpoint_txg，**出生树 = 执行这次分裂的那棵树**（分裂是新建，与「COW 重写身份不变」是两件事）；第一个容器的号 = 第一条记录的 inode 号。内部节点同样在右端溢出、末尾分裂，右半是码 2、没有容器身份。合并：只许**左吸收右 / 左从右借**（右吸收左或右从左借会破「记录号 ≥ 容器号」），保留左半身份，右半退役走 COW 释放路径；合并时机是写路径策略（单测 / 模型对拍），不进格式。**记录数为 0 的类型 2 容器不落盘**（发布前必须合并掉；唯一的叶被删空时是不落盘、根变成零条目）。**跨头首次 COW 的重生**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P6）：新身份 = (本头树 ID, 类型 2, 原容器号, 本次发布代号)——保留容器号、只换出生树与出生代，同时给旧身份写退役记录。重生只对类型 2：类型 1 容器归写者树独占、克隆不收进自己的容器索引（D18（块里携带什么信息） 已定项 11），从不跨头 COW。两条不同的断言并存：「容器被 COW 重写时身份不变」（内容同、位置变）与「分裂后左半身份不变」（内容少了、身份不变）。类型 1 的容器号仍按 C103（容器索引无落点） 那套单调分配，两条规则按记录类型分。

**key 与 inode 号**：key = inode 号 8 字节，唯一性范围 (树 ID, inode)，位宽 64，声明生命周期创建总数上界 2^63。**已发布的 (inode 号, 对象出生代) 不复用**（D6（快照实现模型） 判据 9 的戒律）：`deleted_inodes` 后台回收窗口里复用会让新旧对象的 extent key 三段逐字相同。⚠️ **「已发布的」这四个字不是为了对齐措辞，是让现行设计自洽**：水位住记账（D5（快照 / 空间记账机制） 已定项 4 第 12 项），被抛弃 checkpoint 的那笔账随它一起丢 ⇒ **每一次普通崩溃**都会让那一代发过的 inode 号被重发。管理员回退是挂着时的一次向前发布，那次发布的 inode 号水位取 max(现行那一版内存里的, 环里读得出的根的)，已发过的号不重发（D23（journal 的角色与格式） 已定项 14）；崩溃恢复落到较旧的根时水位随所选根那棵账载入，被抛弃时间线里已发过的号会被重发，而恢复之后每一个新对象的出生代都大于环里读得出的每一条根与记录的 txg（D23（journal 的角色与格式） 已定项 14 注 3）——要几个故障才撞得上被抛弃时间线里已发布对象的出生代，崩溃恢复那一形没量过，见 C143（inode 号水位在崩溃恢复落到旧根后退回去重发）；对外暴露的对象身份 = (inode 号, 对象出生代)，凡按 inode 号认对象的结构都要带上出生代或随根回退。`deleted_inodes` 回收窗口里 extent key 三段逐字相同那一句说的是同一条时间线里的复用（旧对象已发布、它的 extent 在树里等后台回收），照旧被禁；崩溃恢复之后重发同一个号，新对象的 extent 落在所选根那棵 extent 树里，旧对象的 extent 随被抛弃的根一起不在树里——**这靠 extent 树与 `deleted_inodes` 都随所选根**；管理员回退那次发布的新根取 R_old 的 inode 树与 extent 树，`deleted_inodes` 也取 R_old 那一版的（否则 R_old 之后删掉、随 R_old 的 inode 树回来的对象会被后台回收再收一次；主 agent 2026-09-26 定，推的，被攻过零轮）；`deleted_inodes` 的形态（已定项 16）守这两条；盘上分开两者的是单元五元组里的对象出生代（D9（加密） 已定项 6）。**inode 记录在回收完成前留在 inode 树里（nlink = 0），`deleted_inodes` 只存号**。

**下一个 inode 号是记账里的一条统计量**（D5（快照 / 空间记账机制） 已定项 4 第 12 项「inode 号水位」）：key = (统计量, 树 ID, 设备段, 代)，不带设备维，value = 下一个可用号，**合并规则取最大值**（幂等、交换、结合）；条目照样带 seq（已定项 15 第 3 条是格式要求，一棵 write buffer 树一套条目格式），只是**合并结果不依赖它**。选它而不是「后者胜」的理由是崩溃后 txg 重发那一格：同一个代段里先后出现大小两个值，取最大值给大的（浪费号、不复用号）；「未发布的等于没发生」只对常规挂载路径成立，扫描重建看的是盘。「标签 → 合并规则」是盘上语义，统计量登记表立成独立冻结组件（D15（格式冻结政策））。水位行只为可写头维护、随每次发布重写（D5（快照 / 空间记账机制） 已定项 2：每条根的记账树只装这条根那一代的行，每次发布把这一代的行全写一遍），只读树没有这一行；读现行值 = 取该维度元组下代号最大的那一行，空 ⇒ 从未维护过、不是 0。克隆头：新头的水位 = origin 那一刻的运行时水位（含还在 write buffer 里没落盘的那条），建克隆那次发布必须为新树写出这条水位行；两个头此后各自分配，号会相交，唯一性靠 (树 ID, inode)。

**记录字段表（定长 140，小端；对齐口径是记录内偏移，容器内第 n 条起点 136 + 140n（码 3 头 107 + nonce / MAC / 算法类型预留位 29，D18（块里携带什么信息） 已定项 16）不按 8 对齐，按非对齐读）**
<!-- format-const: INODE_RECORD_BYTES = 140 -->
<!-- format-const: INODE_LEAF_RECORDS = 233 -->

| 偏移 | 字段 | 宽 |
|---|---|---|
| 0 | inode | 8 |
| 8 | 对象出生代（创建那次发布的 checkpoint_txg；inode 号不复用后它不再承担复用判别，按五元组 / AAD 的 day-1 契约保留，扫描时与该对象每个数据单元头逐字段对照） | 8 |
| 16 | locality_id（第一版无父目录取 0） | 8 |
| 24 | mode / uid / gid / nlink，各 4 | 16 |
| 40 | size / blocks / rdev，各 8 | 24 |
| 64 | atime / mtime / ctime 秒，各 8（unlink 的意图那次三个共写调用方给的一个值，各批照抄） | 24 |
| 88 | 改动计数（最后一次改动所在发布的 checkpoint_txg；提交锚 = 叶容器头的诞生代号；同一发布窗口内多次改动只有最后一版落盘——内部节点不缓冲；unlink 的意图那次与每一批也算改动，写本次发布的 checkpoint_txg，不照抄上一版；创建算一次改动，所以改动计数不低于记录偏移 8 的对象出生代，I-9.26（inode 改动计数落在出生代与容器诞生代号之间）） | 8 |
| 96 | atime / mtime / ctime 纳秒，各 4 | 12 |
| 108 | 填充（恒 0） | 4 |
| 112 | flags（恒 0） | 8 |
| 120 | 预留（恒 0） | 20 |

**写入时间（偏移 64 起三个秒字段、偏移 96 起三个纳秒字段）取写这条记录的那一次发布的参数，不取系统时钟**：每次发布调用带一个写入时间，同一个参数两次发布逐字节相同（可复现）。三个秒字段都写参数的秒数；三个纳秒字段写参数的纳秒部分，今天的参数只带秒数，所以写 0。**新建**（新池新建文件、新建 inode）写创建那次发布的参数；**改写内容**（覆盖写、顺序写）重写这条记录时写这一次发布的参数，不照抄上一版的值；**unlink 的意图那次**写调用方随这次 unlink 给的时间，三个时间秒（atime、mtime、ctime）共一个值，各批照抄意图那次写的值；**不改这条记录内容的发布**——同一片叶里别的记录改动、这条被连带重写，回退——照抄这条记录上一版的值。

**`blocks` 是逻辑长度的 512 字节块数**：`blocks` = ⌈`size` ÷ 512⌉，按文件长度向上取整，**不表示分到的空间**，不按这个对象分到几个数据单元算。新池新建文件那条记录 3000 字节 ⇒ 6；新建、还没写过数据的 inode ⇒ 0。

不放 extent 指针（D14（双轨（大小文件 / 持久临时）） 已定项 3 内联阈值 0），btime 不进记录。**flags / 填充 / 预留三段同一政策：恒 0，读者遇到非 0 ⇒ 该记录 EIO（对象级，不拒整个容器）**——前向兼容政策不是损坏检出（载荷校验和照样过），落成 I-9.7（记录三段恒零），在 I-1.7（打包容器合法与判定顺序） 的中止链之外。将来给任一位 / 任一字节赋义按 D15（格式冻结政策）「改变已有字段含义」判 incompat，退役的字节永久报废。**记录内不存在 compat 扩展路径**：inode 属性都有语义依赖，D15（格式冻结政策） 判定表第一行够不着，且记录被反复重写、旧读者会把不认识的内容丢掉。预留 20 不承重：记录内任何赋义都是 incompat，宽度只决定那次要不要连记录宽一起改。**溢出路线按属性逐个判档**：无语义依赖的属性走新增 keyspace（D15（格式冻结政策） 判定表给 compat），有依赖的走 flags 位 / 预留（incompat）；属性 keyspace 今天没有落点（xattr 被推迟）。**记录宽必须等于登记表给类型 2 的宽**，不等判损坏（I-1.7（打包容器合法与判定顺序） 里在「类身份段」之后、I-1.2（块头写序已发布） 之前插一步，只需头字段）；改宽是 incompat。记录格式、码 3 类身份段对类型 2 的用法、inode 树的 key 编码是永久字节：**立成一个独立的已冻结组件**（C9（冻结后偷改格式） 按组件记），冻结时刻是第一个写出类型 2 容器的镜像；不动 D15（格式冻结政策） 的四层；类型 1 的记录格式（宽 40，D18（块里携带什么信息） 已定项 24）各自独立。

**不变量**：I-9（inode 树结构） 一类，十四条，见 [invariants.md](../invariants.md)；状态列统一带前置「树的种类可识别（已定项 8）」，第一版镜像上可由类型 2 容器头里的出生树反推哪棵树是 inode 树。nlink == dirent 条数不在这里立——它的门（dirent 树存在）今天没有输入，随 dirent 树上线时一起立。第一版的唯一文件取 inode 1（0 无效），nlink = 1，不进孤儿回收。分裂 / 合并的身份传递与合并方向以外的写路径纪律是操作断言，走单测 / 模型对拍与崩溃点重放。

**重建**：内部节点丢了，收集打包记录类型 2 的容器，按 (出生树, 容器号, 容器出生代) 分组，同身份多版的择现行 = C113（扫描重建时多版单元的现行版本判定无输入）；每叶 key 范围由记录算，内部节点的 key 区间由叶算；产物落 D18（块里携带什么信息） 已定项 5 的级 1（只读挂载），级 2 的见证者今天不存在。**extent 树的重建**：对象按 (树 ID, inode) 认、只看本头的 extent 树；这个对象的全部 extent key（幸存的与找回的）编同一个首段，取值 inode 记录 > 幸存 extent key > 0，记录完好时重建视图里的值就是记录里的值；取错只丢局部性。I-9.9（locality 三值对照） 在重建之后按侧判：一侧是从另一侧编出来的就输出「不可判定」。

**新池新建文件写**：一个码 2 根节点（头带 key 区间 [1, 1]；一条条目：key 1、类型 2、出生树、容器号 1、出生代 3、指针）；一个码 3 叶（类型 2、容器号 1、出生代 3、记录数 1、记录宽 140）；一条记录（inode 1、出生代 3、locality 0、nlink 1、改动计数 3、三段 0）；记账里一条「inode 号水位 = 2」。

**射程**：**同一条时间线上 inode 号不复用**——「时间线」指所选根的祖先链：含管理员回退抛弃的那一段（回退那次发布的水位取 max，那一段里发过的号不重发），不含崩溃恢复抛弃的那一段（崩溃恢复落到较旧的根之后，被抛弃那一段里发过的号可以重发）。**唯一性的射程是一条已发布历史内**——每个容器号都是某条记录首次插入时的 inode 号；号在崩溃恢复落到较旧的根之后可以重发（管理员回退那次发布的水位取 max、不重发，D23（journal 的角色与格式） 已定项 14），所以同一棵树里两个容器可以同号，分开它们的是容器出生代（右半的出生代 = 触发分裂那次发布的 txg，而恢复之后新实例的每一个 txg 都大于环里读得出的每一条根与记录的 txg，D23（journal 的角色与格式） 已定项 14 注 3），同一次发布内「分裂 → 合并 → 再分裂」也造不出同身份两版。跨崩溃不成立：崩溃后 checkpoint 号会被重发，被抛弃那版与重试那版四段逐字相同、内容不同、都在盘上——这与数据单元的五元组、码 2 节点的 (树, 层级, 诞生代号) 是同一种暴露（根记录靠 D16（发布语义） 骑手条款 1 的实例代号分辨，别的单元类靠类身份段里的写序 (实例代号, 事务号) 分辨——容器不单独加实例代号，写序里那 4 字节就是它），常规路径由 D3（空间分配） 已定项 1 的谓词（分配记录）判，扫描重建路径按 C113（扫描重建时多版单元的现行版本判定无输入） 定案的 P2 / P3 规则判。⚠️ **判它安全的依据与树 ID 那条同源**：孤儿与活版的实例代号必不相等（D18（块里携带什么信息） 已定项 11「作废一个 checkpoint 必须换实例代号」），扫描重建的两道闸按写序择新。⚠️ **残留的疑点**：讲统计量合并规则那一句「『未发布的等于没发生』只对常规挂载路径成立，**扫描重建看的是盘**」读起来像是反方向的——它管的是「水位在一个代段里取最大值而不是后者胜」，与「未发布过的号可不可以重发」不是同一个命题，但两者挨得很近。⚠️ **重建的射程是单头镜像**：有克隆头时共享叶头里的出生树是 origin（I-1.3（块头树 ID 一致） 的 C110（跨头共享与加密后元数据类的块头一致性读法） 那一格），按出生树认领会把克隆头的树认残——多头镜像的归属划分要克隆祖先表（E90（树 ID 进 AAD 与跨头共享） 第二笔），落地前多头镜像的 inode 树重建没有输入。⚠️ 填充率没有下界：`⌈n/233⌉` 是新建树的值，1e6 顺序创建再删 90% 不合并是 9.98 倍空间放大；加宽记录能同时缩小爆炸半径（160 ⇒ 204 条），这笔取舍归 C100（inode 容器丢失是永久损失而没有任何冗余条款覆盖它）。⚠️ **挡着「新池新建文件写」的九样，① ② 已定案，其余照录**：① 树表条目已由已定项 8 定；② 记账条目的 seq 住 value、宽 4 字节（已定项 7）；③ 统计量标签值（C71（不认识的统计量标签静默忽略））；④ 不带设备维的统计量的设备段取值、新建树的记账行初始化、读现行值的读法（C117（不带设备维的统计量的设备段没有取值））；⑤ 码 3 容器带不带扩展点（C108（扩展点归属对打包记录单元无判定））；⑥ 码 2 节点头的基础字节数（已由已定项 11 收口）；⑦ `deleted_inodes` 的形态（已定项 16）；⑧ ⑨ 记录格式与统计量登记表两个冻结组件的 spec 文件（C119（冻结组件没有 spec 文件））。⚠️ 跨头首次 COW 时容器身份要不要重生的另一半归 E90（树 ID 进 AAD 与跨头共享） 第二笔。⚠️ **`blocks` 不承担对外的 `st_blocks`**：stat 报的 `st_blocks` 怎么给（stat 时从 extent 树现算分到的单元）、SEEK_HOLE / SEEK_DATA，挪到做 POSIX 面的里程碑。`blocks` 按文件长度向上取整改新池新建文件的字节：那条记录偏移 48 的 8 字节从 64 变 6。 ⚠️ **写入时间单镜像判不出**：取值是发布参数，单个镜像里看不出参数是什么——O1（内存模型对拍（HashMap 理想文件系统）） 的模型与对拍不带写入时间，O2（独立解析器 + checker） 只判单个镜像，O3（独立规约执行器） 的投影不进 inode 树；判它的是按写路径钉写的 harness 用例（覆盖写、顺序写、建 inode、unlink 各一条），多单元顺序写、回退、同一片叶里被连带重写的别的记录三格没有用例（C677（写入时间在三格写路径上没有用例））。

**依据**：

- E98（inode 记录与 inode 树的几何）：inode 记录的盘上形态——打包进码 3 不触发 D21（权威态与派生态的分界） 硬约束 6、住索引节点触发（14 个字段里 11 个算不回来）；改动计数单靠 checkpoint_txg 排不了同窗口内两个版本，所以内部节点不缓冲、同一发布窗口只有最后一版落盘。
- E103（inode 打包路 ① 的更新代价）：路 ①（独立容器索引 + inode 树按身份引用）的读代价比容器索引并进内部节点高、还多一次单元读，而单次写不变——容器索引不另立树。
- E104（扫描重建的现行版本判定）：不认退役记录时退役容器里已死的 inode 会复活；模型打出「重生取最小 inode 号当容器号会撞车」，所以重生保留容器号、只换出生树与出生代。
- 四轮三方论证后用户定案 2026-09-05（材料 `_d8-item6-path.md`、`_d8-item6-r2-amended.md`、`_d8-item6-r3-conclusion.md`、`_d8-item6-r4-conclusion.md`、`_d8-item6-r5-landing.md`，四轮三腿输出 `research/prompts/d8-item6-r{1,2,3,4}-{sonnet,opus}-output.md` 与 `research/results/d8-item6-r{1,2,3,4}-local.out`）。
- 用户定案 2026-09-12（C297（inode 内部条目里出生树两份没有定处理），两轮三方）：身份引用里的出生树与子指针里的那份两份并存，原话在变更史。
- 用户定案 2026-09-16（C143（inode 号水位在崩溃恢复落到旧根后退回去重发），三轮三方）：唯一性收窄到 (inode 号, 对象出生代)，原话在变更史。
- 用户定案 2026-09-23：`blocks` 按文件长度向上取整、不表示分到的空间，`st_blocks` 挪到做 POSIX 面的里程碑（主 agent 的提议，C480（inode 记录的 blocks 怎么算全仓没有条款）），原话在变更史。
- 用户定案 2026-10-01：K-R3-3「写根长高」，`research/prompts/m3-delete-r3-main-verification.md` 第三节 G6-墓碑树根满（原话在变更史）。
- 判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K3：写入时间在各写路径上的取值（两腿同，攻方在副本上量过、主 agent 复跑坐实），三条 oracle 都判不到。
- 用户定案 2026-10-03：删除步算改动（「算改动，改实现」）、时间字段取「乙 意图带时间、三格同改」，原话在变更史；代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 N3 那一行（收窄掉「ctime 跟着改」没回问用户，记录偏移 64 起三格写同一个值）。
- 用户定案 2026-10-05：射程加「同一条时间线上 inode 号不复用」（「写吧」），判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T15 那一行，原话在变更史。

**欠**：C100（inode 容器丢失是永久损失而没有任何冗余条款覆盖它）、C102（索引节点头没有预留区，加字段不是 compat）、C103（容器索引无落点）、C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据）、C106（开放容器数上界）、C107（索引节点的 AAD 身份无落点）、C110（跨头共享与加密后元数据类的块头一致性读法）、C113（扫描重建时多版单元的现行版本判定无输入）、C115（inode 属性的溢出路线无落点）、C116（叶容器的分裂 / 合并纪律没有会失败的检查）、C119（冻结组件没有 spec 文件）、C152（收窄后未发布号重发的扫描重建复核）、C347（祖先表读不出时克隆头认领什么没有条款）、C677（写入时间在三格写路径上没有用例）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:225-242`（整段抄，未转述）**

```markdown
#### 已定项 7：write buffer 条目的 `seq` —— 4 字节，住 value

**定案**：**`seq` 宽 4 字节，住 value 不住 key**，射程覆盖记账 / 反向索引 / LRU 三棵 write buffer 树，**一棵树一套条目格式、三棵同宽**。宽度定死之后因果倒过来：**`T_dirty` 的上界从此是挂载时判定**（4 字节 = 32 位窗口，按 34 字节/条对应约 136 GiB）；D16（发布语义） 已定项 5 那句「`T_dirty` 是可调值，不是格式常量」原样成立，改的是它多了一个由格式定的上界。**在 seq 里再放一份 txg 是把 key 里已有的段抄第二遍**，不许。两条骑手：

1. **反向索引与 LRU 定 key 时必须让去重窗口同样被 `T_dirty` 界住**（代进 key，或别的等效办法）。4 字节够用的依据是「同一个 key 的待去重条目必然同属一个发布窗口」，而那一条在记账树上由 D5（快照 / 空间记账机制） 已定项 5 的 key 末段是「代」推出来；那两棵树的 key 布局全仓一个字都没有，不兑现这条骑手，4 字节在它们身上没有依据。
2. **同一个 key 的跨批次插入顺序必须与 `seq` 同向**，不许留成隐含假设——btree 层同 key 是「谁后写谁赢」，不是「谁 `seq` 大谁赢」，而按叶分区的并行 flush 下两个批次的物理落地先后不受 `seq` 约束。

**射程**：它定的是条目里这一段的宽度与落点，不定别的段（记账 key 的 `(统计量类型标签, 树 ID, 设备, 代)` 四段在 D5（快照 / 空间记账机制） 已定项 5）。⚠️ 这一段不是与别段正交的：那四段在**同一个 checkpoint 内不唯一**（幂等完整值会被写很多次），去重规则「后者胜」逐字要求「后者」由条目自己携带 ⇒ 不定它，那四段合起来还不是一个定得下合并结果的 key。它**立在 D8（核心索引结构） 不立在 D5（快照 / 空间记账机制）**，因为 write buffer 覆盖「反向索引 / LRU / 记账」三类，立在记账那边会让另外两棵树各自重问一遍。⚠️ 记账条目因此从 30 字节（key 22 + value 8）变 **34**，E97（记账与分配记录的条目编码） / E71（记账 key 的条目数与宽度） / E54（记账的「代」要保留几代） / E99（write buffer 条目与 seq 的去重） 四页的数要重算。⚠️ 「inode 号水位」那条记账条目照样带 `seq`（已定项 15 第 3 条是格式要求），只是它的合并结果取最大值、不依赖 `seq`。⚠️ 取值规则（从几起、一次发布里怎么数）在已定项 10；前端与节点缓冲怎么衔接在已定项 12。

**依据**：

- E99（write buffer 条目与 seq 的去重）：量的正是同一个 key 的待去重条目在一个发布窗口内能有多少条、去重要几位——窗口需求落在 32 位以内，4 字节够。
- E17（write buffer 合并上限）：第一版漏了条目序号，两条臂与真值当场对不上，逼出「条目必须自带 `seq`」这个可判定形态；按叶分区可并行的实测同时说明并行 flush 下物理落地先后不受 `seq` 约束，骑手 2 因此必须显式写下来。
- 三条已定条款的字面冲突（三方论证，主 agent 逐条现查）：D5（快照 / 空间记账机制） 已定项 2 逐字「不用找 ⇒ 是点删不是扫描；触及的 key 数 = 统计量数」——`seq` 进 key 之后同一前缀下有多行，点删当场不成立；已定项 6 逐字「条目照样带 seq……只是**合并结果不依赖它**」——单调统计量取最大值要求两条相遇，而 `seq` 进 key 之后它们是不同的 key、根本不会相遇；D5（快照 / 空间记账机制） 已定项 5 的 key 定义本身不含 `seq`，而「后者由 `seq` 定」只有在 `seq` 不进 key 时才自洽。主 agent 补的第四条：`seq` 进 key 会让 key 不可构造——折叠后同一代只留一条，读者想查 `(标签, 树, 设备, 代)` 的值时不知道 `seq`，点查退化成前缀扫。
- 用户定案 2026-09-06：宽度取 4 字节并把因果倒过来、射程扩到三棵树、`seq` 住 value（原话在变更史）。

**欠**：C161（缓冲两级并存而衔接没定）、C162（seq 进 value 之后四个实验的数要重算）、C345（反向索引 / LRU 的陈旧检出没有时间线索）、C416（去重窗口被 T_dirty 界住那条骑手没人查）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:243-285`（整段抄，未转述）**

```markdown
#### 已定项 8：树表条目的字段表

**定案**：**树表条目合计 200 字节** = **树 ID 8**（打头）+ 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8 + 诞生 txg 8 + **头 ID 8** + 预留 76。每层 ⌊(16384 − 131) / 200⌋ = ⌊16253 / 200⌋ = **81** 棵，二层封顶 81² = 6561 棵。⚠️ **分母的口径写死在这里**：16384 减**含预留位的码 2 头**（131 是树表单元自己的码 2 头，key 宽 8 ⇒ 115 + 16，D18（块里携带什么信息） 已定项 16 / 已定项 18；条目区起点是含预留位的头末尾，已定项 11），不减别的数。

- **① 快照进树表**，每个快照一条条目；**诞生 txg 8 字节**，口径是**快照完成（发布）那一次 checkpoint 的 txg**，不是根节点的诞生代号（那是它最后一次 COW 的号，这棵树自己算不出来）。它给 I-3.7（活快照可枚举） 一个载体。
- **② 树 ID 水位住根记录，取根环里全部根记录该字段的 max**。逐字形态：`新水位 = max(根环里全部根记录的该字段, 本次 checkpoint 发出的最高树 ID + 1)`，**每次发布随根记录重写**。「根环里全部根记录」读作根环里全部自证过的根（被抛弃时间线上的也算），并上环里全部自证通过的 journal 记录新根段带的该字段（D23（journal 的角色与格式） 已定项 15）：根所在的槽读不出而它那条记录还在时，恢复本来就从记录把那条根重建出来，记录带的水位就是那条根的水位；journal 记录要等它对应的树写好、根落盘之后才会被绕环盖掉。根与它那条记录都读不出（双重介质故障）的那一格盖不住，记在 C342（树 ID 水位在根读不出时退回去重发）。从 mkfs 之外的水位发第一个文件版本时，十二棵树的号从水位起连号发，次序照已定项 11 那一组 65536..65547（extent、inode、分配记录、记账、中央映射、livelist、稀疏旁表、deadlist、`deleted_inodes`、墓碑树、谱系树、logged_ops）。⚠️ **「累计」这两个字是承重的**：读成「本 checkpoint 发出的最高号」（不累计）时，树 ID 会随根轮出环被重发，而**那时被重发的号属于已发布的树**，D6（快照实现模型） 判据 9 正面命中 ⇒ 整条定案不成立。
- **③ `previous_snapshot_txg` 每头一份，住树表条目**（8 字节）。它是**每个头一个标量**不是每个单元一份，答的是「这个头从哪个快照分出来」，与单元自己的内容无关 ⇒ 按 D1（数据可移动性 / 反向索引） 已定项 4 的判据它进不了单元的自包段。一条树表条目上有**两个各自独立的 8 字节 txg 字段**——诞生 txg（这条自己发布的号）与 `previous_snapshot_txg`（它前驱那个快照的号），对活头条目与快照条目都有定义。
- **④ 兼容档取「树表条目自带长度」这一层，不取「索引节点每条目自带长度」那一层**：前者让旧读者**跳过**一棵不认识的树；后者让旧读者**解开**新树的节点，而**今天没有消费者**——它不是被判成本高，是被判**买不到今天要的东西**。
- **㊀ 旧写者对不认识的记账行**（限「树 ID 不认识、统计量标签认识」这一半）：**写者必须把这类行原样带到新代**（值不变、只换代号；有前端的写者把这类行当前端条目写、随发布排空到叶，第一版写者没有前端（已定项 10）、直落叶，盘上布局相同、读不错）。
- **㊁ flags 其余 15 位**：**未知位一律拒收——该条目 EIO**（对象级，不拒整个池）。**不取**「忽略未知位、继续按已知语义解释」，那是静默误读。配两条纪律：**必须被旧读者看见的改动走系统配置 incompat 位，不许塞进每条目的 flags**；**checker 不许走「跳过」**——I-3.1（已分配统计对得上） 逐字「已分配空间统计 == 实际遍历**所有**引用得到的和」无限定，checker 跳过一棵不认识的树，遍历和就短一截，判出来的是假阳性红而不是漏检，它遇到不认识的树的正确行为是**报「无法完整遍历」并停**。
- **排序契约与嵌套层数**：**条目按树 ID 升序排；嵌套层数不设格式上限**。排序取树 ID 的理由：它是五元组、记账 key、容器索引里到处用的那个键，且 ② 让它随创建序严格递增。不设上限的理由：设上限等于多一个失败模式，而那个上限还随条目宽变。
- **平局判别子取 `(诞生 txg, 树 ID)`，零字节**：D16（发布语义） 已定要用 `(txg, 快照 id)` 打破并列，而 ② 改住根记录之后树 ID **严格随创建序递增**（水位累计、只增不减，崩溃只烧号不回退——I-7.8（根记录树 ID 水位不低于全池最大树 ID) 把这一条变成可判的）⇒ `(诞生 txg, 树 ID)` 就是与创建序一致的全序，而树 ID 本来就在条目里。
- **头 ID 8 字节的取值**：inode 树条目写自己的树 ID（65537），extent 树条目写它服务的那个头（65537），`deleted_inodes` 与墓碑树写 inode 树的树 ID（65537），谱系树写 0（全池一份），logged_ops 写 0（字段表未定，C364（整理意图记录没有字段表）），分配记录树 / 记账树 / livelist 共享树 / 稀疏旁表树 / deadlist 树写 0（无归属，与 D5（快照 / 空间记账机制） 给「全池 / 无归属」行的约定同一个数）；中央映射树不进树表。见 D5（快照 / 空间记账机制） 已定项 9。 ⚠️ 括号里的 65537 是树 ID 从 mkfs 水位 65536 起发的那一版（新池新建文件）上的取值，判据是关系、不是字面值：inode 树条目的头 ID 等于它自己的树 ID；extent 树、`deleted_inodes`、墓碑树条目的头 ID 等于同一张树表里 inode 树条目的树 ID；谱系树、logged_ops、分配记录树、记账树、livelist 共享树、稀疏旁表树、deadlist 树写 0。树 ID 不是从 mkfs 水位起发的那一版（水位已被抬过，或一个克隆头的 inode 树）上，头 ID 随这张表里 inode 树的树 ID 走，不再是 65537（D5（快照 / 空间记账机制） 已定项 9「可写头的 ID = 该头 inode 树的树 ID」）。判它的是 I-9.27（树表条目的头 ID 取值）。
- **预留 76 字节**：已知认购者按「一段字节只有一个意思」排，**逐条登记在树表条目预留的认购表里**（门禁 doc-registries 的 tree-table-reserve 格按它算和：认购合计不许超过 76，表里每条要指得到一个还开着的欠账或一条已定分项）。合计 48，余 28。

<!-- gate:tree-table-reserve total=76 -->

| 认购者 | 住哪条条目 | 宽度 | 出处 |
|---|---|---|---|
| livelist 每头计数器 | 头条目 | 8 | C271（livelist 的 FREE 在什么时刻记没有定义） |
| 第二道销毁闸的活快照计数 | 头条目 | 8 | C272（第二道销毁闸没有 O(1) 的检查法） |
| origin 链接 / 克隆时刻 | 头条目 | 8 | C271（livelist 的 FREE 在什么时刻记没有定义） |
| 快照谱系的区间标号 | 快照条目 | 16 | C144（快照谱系的区间标号没有载体） |
| `children`（子快照计数） | 快照条目 | 8 | D5（快照 / 空间记账机制） 已定项 18 |

⚠️ **inode 号水位不在表里**：C143（inode 号水位在崩溃恢复落到旧根后退回去重发） 2026-09-16 定案「不搬水位，水位照旧住记账」（D5（快照 / 空间记账机制） 已定项 4 第 12 项）⇒ 它不认购这里的字节。

**改新池新建文件的字节：是**——树表单元 t12 的条目宽字段（200）、声明长度（11 × 200 = 2200）与 11 条条目的排布。

**射程**：⚠️ **㊀ 的射程只到「树 ID 不认识」那一半，另一半的处方是相反的**：不认识**统计量标签**那一半由 C71（不认识的统计量标签静默忽略） 管，而它逐字要求「挂载时遇到不认识的统计量标签**必须按 compat_ro 拒绝可写挂载**」——㊀ 不覆盖也不推翻它。两半分得开的机制依据：D5（快照 / 空间记账机制） 已定项 5 的 key 定宽 ⇒ 不认识标签**含义**的写者照样能按固定偏移切出各段；成立的机制是已定项 1「记账条目取幂等完整值」，一条行的值对搬运者不透明也搬得动，而写者不认识那棵树就改不了它 ⇒ 值本来就不变。不搬的代价：D5（快照 / 空间记账机制） 已定项 2「只保留最近 K 代」⇒ 没人重写的行过 K 代消失。⚠️ **暴露面今天很窄**：只有一个统计量带树维（D5（快照 / 空间记账机制） 已定项 4 第 12 项，只为可写头维护）⇒ 不是可写头的树一行都没有；再加一个带树维的统计量，不搬的丢失量就按树数长。⚠️ **㊁ 拒收不丢数据依赖 ④ 的条目自带长度**：条目 EIO 但**位置还在** ⇒ 写者知道「那儿有一棵树我读不了」⇒ 不做跨它的 deadlist 合并；没有长度字段就定位不到下一条，这条推导不成立。⚠️ **㊁ 与本工程对另外两个 flags 字段已定的政策是同一条**：D18（块里携带什么信息）「偏移 7 是 8 个 flag 位……读者遇到非 0 位一律拒收……本工程不给这 8 位留「忽略陌生位」的通道」；已定项 6「flags / 填充 / 预留三段同一政策：恒 0，读者遇到非 0 ⇒ **该记录 EIO**」，已落成 I-9.7（记录三段恒零）⇒ 三个 flags 字段并成一条，不用再解释它们为什么不同。⚠️ **「位」和「种类」是两根轴**：D15（格式冻结政策） 已定项 2 那条 compat（新增 keyspace 对旧读者只是看不到新树）管的是**种类**，㊁ 管的是**位**——一棵旧读者自以为认识的树被加了位，它的语义变了，那不是加法。⚠️ **嵌套层数不设上限的代价是性能悬崖**，按 `.claude/rules/fs-design.md` 硬要求 4 必须可观测：「每次发布 COW 的树表单元数 = 层数」要能在运行时读出来——走错层数不报错，没有观测手段就永远发现不了。⚠️ **① 交付的那张表不够用**：D5（快照 / 空间记账机制) 点名的「全部根的 txg 列表」写在 D6（快照实现模型） 已定项 1（每头一棵自己的树）之前，而 D6（快照实现模型） 用一个反例打穿了它的前提（主线 txg=10 建可写快照 X、在 X 里于 txg=30 写一个块、主线 txg=35 建快照 Y，按 D5（快照 / 空间记账机制） 的条件判「Y 引用它」而 Y 根本看不见它）⇒ 要的不是「每个根一个 txg 标量」，是**谱系**；诞生 txg 这 8 字节仍然非有不可（I-3.5（引用区间的精确性） 要按 txg 升序枚举全部快照根与活头、I-3.6（deadlist 紧界） 要 `prev(S).txg`），欠的是**在它之上再加一层标号**（嵌套区间标号是两个数，而且随谱系变化要回头改写，与「诞生 txg 写一次不再动」是完全不同的格式性质）。同一个前提的三个消费者里定案只兑现了 I-3.7（活快照可枚举） 一个：I-3.5（引用区间的精确性） 的可判定形式建在「全部根按 txg 排成单一全序」上，谱系成树之后那一条恒假；D18（块里携带什么信息） 的代际回收走同一个区间测试，方向是过判 ⇒ 少回收 ⇒ 泄漏。⚠️ **③ 只补了活头头部的半格**：D5（快照 / 空间记账机制） 逐字「每个可写头各持一份 `previous_snapshot_txg` **与一份 deadlist**」「建快照时把**活头的 deadlist 整个交接给新快照**，活头换一个空的」⇒ 快照条目也要能指到一份 deadlist，而 deadlist 的盘上落点全仓一处都没有。⚠️ **丙-2 唯一盖不住的那一格判为不是错误**：某个 checkpoint 分配了树 ID 42、写出带 42 的单元头，根没写成就崩了 ⇒ 水位随那个根一起丢 ⇒ 下次挂载重发 42，而盘上留着带 42 的孤儿单元；判它不构成错误的依据是孤儿与活版的实例代号必不相等（D18（块里携带什么信息） 已定项 11「作废一个 checkpoint 必须换实例代号」）、I-1.2（块头写序已发布） 逐字「判不成已发布者必是未发布的撕裂事务垃圾或被抛弃时间线的残留」、扫描重建的两道闸与孤儿回收条件都不看树 ID、孤儿与活版共用五元组本来就是常态（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1 加 10 字节写序正是为此）、D6（快照实现模型） 判据 9 的理由句说的是「树里的 key 变成可见」而孤儿不在任何树里。由此 D6（快照实现模型） 判据 9 的射程按它自己的理由句收窄成「**已发布**的快照标识单调不回收」（2026-09-06 用户定案，同一次）。⚠️ **inode 号水位另给答案**：D5（快照 / 空间记账机制） 已定项 4 第 12 项逐字「**带**（只为可写头维护）」⇒ 每个可写头一个标量、条数随头数长，而根记录也是定长槽 ⇒ 丙-2 同样搬不过去；2026-09-16 定案不搬水位，把唯一性收窄到「已发布的 (inode 号, 对象出生代) 不复用」（已定项 6）。⚠️ **树表容量在两处按不同条目宽算**：E79（根记录的容量） 与 D22（单元原子性怎么合成） 已定项 7 报的每层棵数是另一个模型（它把树表条目当成「树 ID 8 + 根指针 59」= 67 字节），方向不变（间接层躲不开、超过一层再嵌一层），只是门槛比 D22（单元原子性怎么合成） 正文写的低。⚠️ 预留里那两笔已知认购（deadlist 的落点、快照谱系的嵌套区间标号）**没有顺手命名成字段**——命名等于把两个未定项一并定了，那是两个决定。排序契约（① 树 ID 严格升序，② 合发号次序）由 I-9.16（树表条目按树 ID 严格升序且合发号次序） 判：重建与冷走读在读任何一棵树之前拒，池级 checker 另判一份（实审 A3b Q2 与树表排序实现，用户 2026-09-27 定立不变量并同步）。

**依据**：

- E112（旧写者遇到不认识的树）：跑前写死判据与接受条款下六格的静默结局与搬运代价——`Skip + Carry` 在挂得上的那些里零静默结局，`Ignore` 那一格记录被按错编码解释，`Refuse` 全挂不上且与 D15（格式冻结政策） 已定项 2 的 compat 档不相容；搬运代价每代只多写两行，跑前定的作废阈值未触发。
- E113（不认识的树·补齐臂集）：臂集补成 5 flags × 3 记账、选法改成跑前写死的字典序之后，`RejectEntry` 在全部扫描点里一次都没输过，`Skip` 在第 1 关就输（隐形快照使 deadlist 合并认错邻居）、`Ignore` 输在第 2 关、攻击腿提的按位分档 `PerBit` 输在它没有第三种行为——㊁ 由此从「跳过该条目」重定为「该条目 EIO」；记账那一维取 `Carry` 不取 `Exempt`（`Exempt` 要改 D5（快照 / 空间记账机制） 已定项 2，那笔代价一个格子都没占）。
- 用户定案 2026-09-06：四问（快照进树表、树 ID 水位住根记录、`previous_snapshot_txg` 每头一份、兼容档取条目自带长度）、㊀ ㊁、排序契约与嵌套层数、平局判别子、预留取 32 字节（原话在变更史）。
- 用户定案 2026-09-23：② 的 max 也并上环里记录新根段带的水位（C342（树 ID 水位在根读不出时退回去重发） 前置那一问；C511（回退到无文件那一版之后诞生代怎么接） 第 3 步的实现员先选了这一边，用户同日认下，原话「journal 日志就是为了保证树能完整写入的」），原话在变更史；发号次序是主 agent 同日照格式常量次序定的，被攻过零轮。
- 用户定案 2026-09-14：树 ID 挪到条目最前、根指针 83 → 86、预留里切 8 字节做头 ID（原话在变更史；依据是已定项 11 定码 2 条目里 key 一律是条目的前 key 宽 个字节，树 ID 不打头就要给通用解码规则加例外）。
- 用户定案 2026-09-16：条目从 148 加宽到 200、预留 76，代价是每单元从 109 棵降到 81 棵、二层封顶从 11881 降到 6561，用户知情接受（原话在变更史）。
- 判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K1：头 ID 读同表关系、不读字面 65537——字面读法在一段合法的高水位历史上误红 4 条，还放行头 ID 写死 65537 的坏实现（攻方在副本上量过，主 agent 复跑坐实）；判它的 I-9.27（树表条目的头 ID 取值） 按表判，树表有条目时才判。

**欠**：C144（快照谱系的区间标号没有载体）、C145（deadlist 在盘上没有落点）、C147（树 ID 水位的不变量还没有）、C148（树表条目变宽把间接层门槛压到二层）、C149（根槽重发之后 birth 口径松一格）、C156（记录编码误读没有不变量）、C157（树表容量在两处按不同条目宽算）、C158（checker 跳过不认识的树会判假阳性）、C159（E112 的臂集缺三条且天平不含挂不上的代价）、C160（记账臂的真实代价不在天平上）、C342（树 ID 水位在根读不出时退回去重发）、C675（同一种树在树表里至多一条没有不变量）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:286-317`（整段抄，未转述）**

```markdown
#### 已定项 9：树的种类码的登记表

**定案**：树表条目里「树的种类」那 2 字节的取值，唯一登记位是已定项 9 这一节的登记表；**新树只追加不重用码**，0 保留为「无」。

| 码 | 树 | 出处 |
|---|---|---|
| 0 | 无（树表条目不用它） | — |
| 1 | extent 树 | D4（校验和位置） 已定项 7 |
| 2 | inode 树 | 已定项 6 |
| 3 | 分配记录树 | D3（空间分配） 已定项 3 |
| 4 | 记账树 | D5（快照 / 空间记账机制） 已定项 3 |
| 5 | 中央映射树 | D19（块指针的结构与宽度预算） 已定项 5 |
| 6 | livelist 共享树 | D6（快照实现模型） 已定项 2 |
| 7 | 稀疏旁表树 | D5（快照 / 空间记账机制） 已定项 6 |
| 8 | deadlist 树 | D5（快照 / 空间记账机制） 已定项 11 |
| 9 | `deleted_inodes` 树 | 已定项 16 |
| 10 | 墓碑树 | D18（块里携带什么信息） 已定项 24 |
| 11 | 谱系树 | D18（块里携带什么信息） 已定项 24 |
| 12 | logged_ops 树 | 已定项 5 |

新池新建文件写出的 **11** 条树表条目照此表（码 1..4、6..12 各一条；中央映射树的根住根记录、不进树表，D19（块指针的结构与宽度预算） 已定项 11；码 5 照登记）；E142（新池新建文件的干跑） 的 `TREE_KIND_*` 常量是码 1..8 那八个值（码 9..12 它没有）；`crates/singlefs-core/src/records.rs` 与 checker 的 `walk.rs` 用同一组常量（码 5 不进树表，那里没有它）。

**射程**：它只登记「树的种类」这一段的取值，树 ID 是另一个量，两个量的值域不相交（树 ID ≥ 65536，取值在已定项 11）。与 D18（块里携带什么信息） 已定项 11 的单元类登记表是两张表、两个概念（用户不取合并）。码 8（deadlist）与 livelist 共享树、稀疏旁表同一个先例——都是「销毁快照时要读、而销毁不许全盘扫描」的结构，day-1 注册进树表比第一次建快照时惰性创建少一条「首次建树」路径；条目形态另立 D5（快照 / 空间记账机制） 已定项 12。**改新池新建文件的字节：是**——加码 8 那一次让树表从 6 条变 7 条、树 ID 水位从 18 变 19。

**依据**：

- 无实验：码值是取值约定，没有可量的量；登记表由用户 2026-09-13 定案（码 0..7）、2026-09-14 定案加码 8，两次都没有走三方论证、可推翻，原话在变更史。
- 码 9..12：判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节种类码那一行（两腿各自得出）；树 ID 与种类码值域不相交由用户定案 2026-10-01 取（岔路单 `research/prompts/m3-blanks-r1-forks.md` 的「种类码与树 ID 的错开」一行，原话在变更史）。
- 判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K2：新池新建文件写出的树表条目数按实现现值（11 条，`crates/singlefs-format/src/lib.rs` 的 `NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES`）。

**欠**：C315（树的种类码没有登记表）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:318-329`（整段抄，未转述）**

```markdown
#### 已定项 10：第一版记账直落叶，`seq` 的取值规则

**定案**：**第一版记账直落记账树叶，不走 write buffer 前端**；D5（快照 / 空间记账机制） 已定项 3 第 3 条读成「前端是后续优化」。**`seq` = 同一个 `(标签, 树, 设备, 代)` 在一次发布里被写进树的第几次**（不是运行期那个统计量被改的次数），**从 1 起**；第一版禁边插边 COW（D19（块指针的结构与宽度预算） 已定项 12 的固定点：每个码 2 / 码 3 一个实例一个 checkpoint 只写出一次，发布末尾一次性 COW）⇒ 每个 key 每次发布只写进树一次 ⇒ **恒 1**；合并规则取最大值的统计量也照写。

**射程**：它定的是第一版走哪条路与 `seq` 怎么数，不定 `seq` 的宽度与落点（已定项 7）、也不定前端与节点缓冲的衔接（已定项 12，第一版两级都不开）。放开固定点、开前端或节点缓冲之后「恒 1」还成不成立，由已定项 13 的状态折叠接着管。

**依据**：

- 无实验：第一版走不走前端是实现排期的选择，没有可量的量；由用户 2026-09-13 在写代码前的总审核里定案，没有走三方论证、可推翻，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:330-362`（整段抄，未转述）**

```markdown
#### 已定项 11：码 2 的载荷布局与 key 比较序

**定案**：码 2 节点头自描述（预留 2 之后是条目数 2 + 条目宽 2，key 宽 1 排在 key 区间之前，偏移表见 D18（块里携带什么信息） 已定项 18），基础字节数 81 → 86；含 29 字节预留位后头 = `115 + 2 × key 宽`。条目区从含预留位的头末尾（`115 + 2 × key 宽`）起，**定宽条目连续排列**，**声明长度 = 条目数 × 条目宽**；条目之后的空闲字节恒 0 且参与载荷 CRC。载荷 CRC 覆盖 `[86 + 2 × key 宽, 16384)`（起点是明文头末尾，口径与码 1 的 105、码 3 的 107 同）。**条目里 key 一律是条目的前 key 宽 个字节**——「读条目宽 → 按条目宽切条目 → 取前 key 宽 个字节当 key」是一条对全部码 2 树都成立的通用解码规则，扫描重建路径上不用另查一张「这棵树的 key 在条目里排第几」的表。key 的全序按逐字段无符号整数、字段自左向右比较，**盘上小端存储不构成 memcmp 序**。树 ID 从 65536 起编（种类字段 2 字节取不到的最小值，两个量的值域永不相交）：extent 65536、inode 65537、分配记录 65538、记账 65539、中央映射 65540、livelist 65541、稀疏旁表 65542、deadlist 65543、`deleted_inodes` 65544、墓碑树 65545、谱系树 65546、logged_ops 65547；mkfs 时水位 65536，第一个文件版本那次之后 65548。

**改新池新建文件的字节：是**——六个码 2 单元的条目区起点（`115 + 2 × key 宽`：inode 树根 131、extent 163、分配记录 135、记账 159、映射 169、树表 131）、声明长度，以及树表条目的字段次序；树 ID 字段（树表条目里的树 ID 与头 ID、码 2 头里的树 ID、记账 key 的树 ID 段）与树 ID 水位的值也变，随树 ID 从 65536 起编。

**多层码 2 树怎么长、怎么收**（记账树、中央映射树，以及照已定项 14 仍用码 2 btree 的树）：① 一次发布里分裂新建的节点与别的单元落在同一段，整条根到叶的路径照常 COW，不加写序步骤、不加屏障；② 叶装不下时从中间切；③ 收缩只摘空节点，根只剩一个孩子时降高一层，不做「低于一半合并」与借条目；④ 分隔 key 跟着维护：插到最左分隔 key 之下时把它压低，条目在兄弟间挪动之后把右边那个孩子的分隔 key 改成它新的最小 key；逐层可判：分隔 key ≤ 孩子节点头的最小 key，且 > 左邻孩子节点头的最大 key（D18（块里携带什么信息） 已定项 18 的 key 区间）；⑤ 树高 = 根节点码 2 头里的层级 + 1。

- **一次发布里先删后插**：上一版有、这一版没有的 key 先按 key 升序删，再按 key 升序插这一版新有的；只决定形状，不决定写序（写的是最终态，已定项 13）。
- **从盘上重建的树分隔 key 坏了**：重建只核拼得成树的几样（层级、节点不空、内部条目宽、不重复引用），不核分隔 key；下一次发布按分隔 key 删不掉时规划交回，可写挂载在取号之前拒，盘上不变。遇到这种盘之后怎么办（修复、只读挂载）没定。
- **叶条目宽不由读树判**：读树只切 key，叶条目窄于字段表由解条目的一方各报各的；内部条目窄于 key + 86 由读树报。

**内部节点的条目 = 本树 key + 子指针 86**：记账树 22 + 86 = 108、中央映射树 27 + 86 = 113。分配记录树与 extent 树按已定项 14 另起按 key 空间寻址的结构，结构在三方 `m2-keyspace-r1` 判；照码 2 btree 做时它们的内部条目是 96 与 110（extent 110 是用户 2026-09-24 早先定的）。

<!-- format-const: ACCOUNTING_INTERNAL_ENTRY_BYTES = 108 -->
<!-- format-const: CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES = 113 -->

**射程**：它管码 2 节点的载荷怎么排与 key 怎么比，不管头里各字段的偏移（D18（块里携带什么信息） 已定项 18）、也不管每棵树的条目里除 key 之外还有什么（各树自己的分项）。唯一因「key 打头」需要重排的是树表条目，已按这条改成树 ID 打头（已定项 8）。「定宽条目连续排列」与「声明长度 = 条目数 × 条目宽」在中央映射条目回到定宽 55 之后不用动（D19（块指针的结构与宽度预算） 已定项 10）——条目宽字段只有一个 u16，按类两宽时这两句顶不住。两条 2026-09-14 补的定案都是用户弹窗定案，没有走三方论证，可推翻。⚠️ 分裂、收缩、分裂点、写序四样被攻过零轮，三方原型只抽了一次样；「低于一半合并」与「追加时末尾切」在崩溃历史上与选中的那两样零对零，没选的理由是用户的取舍，不是安全性。「分裂单开一次发布」出局：只改结构的发布挤掉 D16（发布语义） 已定项 1 的「最近 4 个可退到的不同状态」。

**依据**：

- E145（码 2 自描述头与映射树 key 宽的代价）：量的正是码 2 头自描述要加的那几段与映射树 key 宽的代价——加到 86 之后扇出、树高、解头那几节都不掉档，自描述值得做。
- 用户定案 2026-09-13：码 2 头自描述、基础 86、条目区起点、载荷 CRC 覆盖范围、key 比较序（原话在变更史）。
- 用户定案 2026-09-14：条目里 key 一律是前 key 宽 个字节；「定宽条目连续排列」与「声明长度 = 条目数 × 条目宽」原样成立（原话在变更史）。
- 用户定案 2026-09-24：extent 树内部条目 110 字节（原话在变更史）；E157（并行线一两条条款的计数模型） 在 110、136 两个取样点上结论都不翻面。
- 无实验：分裂、收缩、分裂点、写序四样是三方原型（`research/prompts/m2-treesplit-r1-opus-model/`）在同一批崩溃历史上零对零之后由用户选的，没立实验号；分隔 key 跟着维护是那一轮主 agent 按打中定（原型 t1.out：沿用 D8（核心索引结构） 已定项 6 的「分隔 key 创建后不变」24 格全红、跟着维护 24 格全 0），判决 `research/prompts/m2-treesplit-r1-main-verification.md`。
- 用户定案 2026-09-24：写序同段照常 COW、只摘空节点且根剩一个孩子时降高、从中间切、内部条目 = 本树 key + 子指针 86（原话在变更史）。
- `research/prompts/m2-treesplit-implementer-report.md` 第五节 B1、B5、B6。
- 用户定案 2026-10-01：树 ID 整体挪到 65536 起，与种类码的值域不相交（岔路单 `research/prompts/m3-blanks-r1-forks.md` 的「种类码与树 ID 的错开」一行，原话在变更史）。
- 判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节种类码那一行：新树的种类码取 9..12（两腿各自得出）。

**欠**：C306（码 2 头不自描述 key 宽）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:363-376`（整段抄，未转述）**

```markdown
#### 已定项 12：write buffer 前端与节点消息缓冲区的衔接 —— 一棵树只准一级

**定案**：走 write buffer 前端的三棵树（记账 / 反向索引 / LRU）**ε 恒 0、内部节点不留消息缓冲区**；这三棵树上的写**一律进前端**——含已定项 8 ㊀ 那类旧写者搬运的记账行：有前端的写者把它当前端条目写，第一版写者没有前端（已定项 10）、直落叶，盘上布局相同、读不错；**发布前把前端排空到叶**，落叶时同 key 比同一窗口的到达序；发布集的前端条目在同一次发布里排空、按事务切，不跨发布（fsync = 一次全量发布）。其余树按 D11（索引节点要不要留消息缓冲区） 已定项 6 的树轴留不留缓冲。**第一版两级都不开**（已定项 10、D11（索引节点要不要留消息缓冲区） 已定项 7），不改新池新建文件的字节。

**检查**：前端树的写路径只暴露前端入口（类型层）；运行时断言——一个 checkpoint 里前端树叶上的每一处逻辑改动（key → 值）都对得上一条这个 checkpoint 进过叶的前端条目，窗口中途 flush 的与发布时排空的都算、实例切换重发时保留；重组、索引重建、损坏恢复重建三个例外各配一条「前后每个 key 读出来的值不变」。它只能是运行时的：盘上分不出经前端排空与直写落叶。按字面写的三种断言（只认发布时排空的、只认这一次尝试的、按物理新叶数）在合法历史上会误报（`c161-r3-model-output.txt` 的 `G3_publish_only` / `G3_attempt` / `G3_physical` 行）。

**射程**：**代价**：每次发布排空到叶的写出是「排空到根缓冲」那类形态的 3.4–10.7 倍（覆盖写负载 4.8–10.6 倍，同一产物 `part` 列的第三段），发布集的条目都得在这次发布里落叶、不能按预算分批；同一份脏页预算下前端装的条目少一个量级——`T_dirty` 按什么算没有口径。**买到的**：前端树上没有同 key 跨级并存，读的次序最短；陈旧检出（C52（陈旧当成合法））在这三棵树上没有对象。**没出局但没选的**：甲-R4（前端直落叶、节点缓冲只收不走前端的写）与乙-D（前端进根缓冲逐层下推、按位置定新旧）各只在塌缩上被打中一次，它们的陈旧检出要每条消息带发布号（与已定项 7「不许再放一份 txg」冲突），节点布局进 D15（格式冻结政策） 第 4 层。**连带**：重组时节点缓冲怎么搬（射程是全部留缓冲的树，这一项只省了前端三棵）；单调统计量的点删遇墓碑（D5（快照 / 空间记账机制） 已定项 3 第 2 条）；反向索引 / LRU 的时间线索；D11（索引节点要不要留消息缓冲区） 已定项 6 树轴的分支变量与已定项 5 的措辞。

**依据**：

- 无实验：这一格量的是设计形态在合法历史上会不会被分辨臂打中，依据是三轮三方（`research/prompts/c161-r1-main-verification.md`、`c161-r2-main-verification.md`、`c161-r3-main-verification.md` 各自第四节）——这一形在三轮试过的全部形状（乱序 flush、K 代点删、分裂与合并、部分发布、排空做到一半崩溃、删除、树高 4、索引重建、实例切换、管理员回退）上没被打中；出局的乙 + seq 全池单调两轮被打中（计数器不落盘；实例切换时计数器从所选根重新载入），还要 8 字节 seq 与落盘计数器。用户 2026-09-16 收尾弹窗定案，原话在变更史。

**欠**：C161（缓冲两级并存而衔接没定）、C344（单调统计量的点删遇墓碑没有会红的检查）、C345（反向索引 / LRU 的陈旧检出没有时间线索）、C348（T_dirty 按什么算没有口径）、C349（重组时节点消息缓冲区怎么搬没有条款）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:377-388`（整段抄，未转述）**

```markdown
#### 已定项 13：索引树更新做状态折叠 —— 同一次发布里同一个 key 的中间状态不落叶

**定案**：一棵索引树上的同一个 key 在同一次发布里被改了多次（A → B → C → D → E）时，落叶只做一次：叶里这个 key 的条目直接写成最终态 E，A–D 不写进任何节点。「同一周期」是一次发布：同一个 `checkpoint_txg`，fsync 触发的也是一次发布（D16（发布语义） 已定项 6）。**E 按同一次发布里的到达序取最后一次改动，不按值取**；E 是墓碑时，折叠的结果就是墓碑，不许把墓碑当数值与 A–D 一起取最大（C344（单调统计量的点删遇墓碑没有会红的检查） 实测这种朴素折叠四条候选 16 / 16 丢点删，`b.out`）。到达序在内存里由条目自己带着（已定项 15 第 3 条），不靠按 key 排序之后的相对位置。

**射程**：**全部索引树的全部落叶路径**。第一版的直落叶（已定项 10）之外，以后开的 write buffer 前端排空（已定项 12：落叶时同 key 比同一窗口的到达序，比出来的最后一条就是 E）、节点消息缓冲区（D11（索引节点要不要留消息缓冲区） 已定项 6 的树轴）、边插边 COW（D3（空间分配） 已定项 5 允许、D19（块指针的结构与宽度预算） 已定项 12 第一版不取的那条实现）都照这个规矩落叶。**第一版已经满足它**：D19（块指针的结构与宽度预算） 已定项 12 的固定点让已定项 10 推出「每个 key 每次发布只写进树一次」、`seq` 恒 1；已定项 13 把这件事从固定点的推论提成独立条款，放开固定点之后照样成立，不改新池新建文件的字节。实现今天没有对象：`crates/singlefs-core/src/transaction.rs` 的 `publish_overwrite` 一次覆盖写就是一次发布（txg、事务号各加一），没有「同一次发布里几个事务改同一个 key」的路径；同一个 checkpoint 里装两个对象的用例改的是不同的 key。⚠️ **没答的两条**：① A–D 在树之外的去向不归它管——A–D 是同一个逻辑块被覆盖写几次得到的数据单元版本时，按 D16（发布语义） 已定项 2 的丢失窗口与切分纪律「一事务一单元」，A–D 各是一个事务，各有一个落了盘的数据单元与一条 journal 记录（D23（journal 的角色与格式） 已定项 7：一条记录一个事务号），这几个单元没有根引用、也不在 I-7.4（近 K 代块未被复用） 的保护里，它们要不要写、写了在同一次发布里怎么释放、记账怎么算，决策正文、欠账表、不变量清单里都没有管它的条款；② 已定项 7 的盘上 `seq` 在折叠之下恒 1（已定项 10 的「恒 1」从第一版扩到所有版本），而认 E 用的是内存里的到达序，盘上那 4 字节 `seq` 还拿不拿来当 `T_dirty` 上界的依据（已定项 7 定案里「`T_dirty` 的上界从此是挂载时判定」那一句），没答。⚠️ 「单调统计量在同一次发布里不减、所以两种取法给同一个数」是主 agent 的推论，没有检查。

**依据**：

- 无实验：用户定「这个是决策 记录 暂时不需要做和验证」，没开三方、没建实验，也没有可量的量；用户 2026-09-19 定案与同日对「同一周期 = 一次发布」的确认，原话在变更史。A–D 可以不落的三条支点是已有条款（主 agent 2026-09-19 现查）：D16（发布语义） 已定项 4 与 D23（journal 的角色与格式） 已定项 14 第五条（施加的单位是一次发布，合法前缀停在一次发布的两个事务之间时那次发布整体不施加，所以 replay 出来的只有发布边界上的状态）、D16（发布语义） 已定项 6（fsync 触发的也是一次发布，被 fsync 确认过的状态都落在发布边界上）、D16（发布语义） 已定项 14（快照发布点对齐 checkpoint 边界、钉在这个 checkpoint 全部数据变更之后，没有快照引用 A–D）。

**欠**：C383（同一次发布里同 key 的中间态折叠没有检查）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:389-442`（整段抄，未转述）**

```markdown
#### 已定项 14：一套 btree 实现 + 多个独立 keyspace + 两个前端，派生树可以异构

**定案**：**一套 btree 实现 + 多个独立 keyspace + 两个前端，管权威态的树与照分裂做的树；派生态的分配记录树与 extent 树（D21（权威态与派生态的分界） 已定项 5、9）可以另起按 key 空间寻址的结构，格式可以与码 2 btree 不同。**

| 层 | 做什么 |
|---|---|
| 独立 keyspace | 每类数据一棵树（extent / inode / dirent / 反向索引 / 空闲空间 / 记账 …），互不干扰 |
| 日志结构节点 | 节点内新 key 追加不插入，内存里定期重排；写顺序、可在只读锁下回写 |
| **write buffer** 前端 | 高频小更新（反向索引、LRU、记账）批量攒起来排序去重后批量插入 |
| **key cache** 前端 | 同一批 key 反复读写的（分配信息）走哈希表，不做全树查找 |

**两棵派生树的结构**：

- **分配记录树**：按绝对槽号按位置寻址。每片叶罩一段固定的槽 `[k × W, (k + 1) × W)`，W 取偶数（两槽数据单元永远不跨叶），且 W ≤ 叶条目容量 812（一条记录至少罩一个槽，叶永远装得下）；记录按起点槽落叶。往上各层按位置寻址，子节点罩哪一段由它在父节点里的位置算出。**没有记录的一段 = 全空闲**：那片叶不写，父节点那一格留空（用户 K1）。checker 逐节点核「这个节点罩的就是它的位置规定的那一段」与「记录的末槽不越过它所在叶的末槽」。
- **extent 树**：两段，都按位置寻址。上段按 inode 号的位置寻址；下段一个文件一棵，按数据单元号的位置寻址（叶罩 144 个单元，内部扇出 147），洞就是缺席。**只有一个数据单元的文件不建下段**：上段叶条目里直接放那个数据指针；条目带一个标签字节区分「没有单元 / 下段根指针 / 内联数据指针」（用户 K2）。
- **挂载怎么读**：分配记录树挂载时整棵读进挂载态（分配器本来就要）；extent 树按需，打开文件时按位置走下去（用户 K4）。
- **实现取值**（实二一取，主 agent 2026-09-25 定，被攻过零轮）：
  - **分配记录树**：
    - 叶宽 W = 812（取叶条目容量 (16384 − 135) ÷ 20 本身），内部条目 96 字节（key 10 + 子指针 86），扇出 169。
    <!-- format-const: ALLOCATION_RECORD_TREE_LEAF_SLOTS = 812 -->
    <!-- format-const: ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES = 96 -->
    <!-- format-const: ALLOCATION_RECORD_TREE_INTERNAL_FANOUT = 169 -->
    - 层级 L 的节点罩 W × 169^L 个槽。
    - 内部节点与根只写有孩子的格：每个孩子一条内部条目（key 10 + 子指针 86），条目的 key 取这个孩子按位置罩的那一段的起点（孩子所在的盘, 那一段的第一个槽号），按 key 排，没有孩子的格不占条目；读者按这个 key 认孩子，槽号不落在孩子那一层的格点上、盘不在池里、或认出的孩子不在这个父节点名下的，这条条目不指任何孩子（`crates/singlefs-core/src/allocation_record_tree.rs` 的 `child_named_by_entry_key`）。根节点头的 key 区间罩整个 key 空间，smallest_key = (0, 0)，largest_key = (0xFFFFFFFF, 2^48 − 1)。
    - 根罩整个 key 空间、按盘分流；根层取最小的 R ≥ 1，使 Σ_盘 ⌈盘上槽数 ÷ (W × 169^(R−1))⌉ ≤ 169。4 GiB × 2 时根在第 2 层、树高 3。
  - **extent 树**：
    - 上段叶条目 113 字节 = key 24（(0, inode, 0)）+ 标签 1 + 载荷 88。标签 0 全零，只有读者认、写者不写；标签 1 是下段根的节点指针 86 字节加 2 字节零；标签 2 是那一个数据单元的数据指针 88 字节。
    <!-- format-const: EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES = 113 -->
    - 上段叶罩 143 个 inode，内部条目 110 字节（key 24 + 子指针 86），扇出 147。
    - 上段叶节点头的 key 区间照 D18（块里携带什么信息） 已定项 2 射程段写这片叶按位置罩的那一段，不写叶里第一条与最后一条条目的 key：闭区间，第 k 片（从 0 起）smallest_key = (0, 143k, 0)，largest_key = (0, 143k + 142, 2^64 − 1)。
    <!-- format-const: EXTENT_TREE_UPPER_LEAF_INODES = 143 -->
    <!-- format-const: EXTENT_TREE_INTERNAL_ENTRY_BYTES = 110 -->
    <!-- format-const: EXTENT_TREE_INTERNAL_FANOUT = 147 -->
    - 下段叶罩 144 个单元，下段 key 的字节偏移 = 单元号 × 32634。上段与下段的内部条目，key 都取这个孩子按位置罩的那一段的起点（`crates/singlefs-core/src/extent_tree.rs` 的 `build_extent_internal_entry`）。
    <!-- format-const: EXTENT_TREE_LOWER_LEAF_DATA_UNITS = 144 -->
  - **发布路径的固定点**：猜重写集 → 在分配器拷贝上走释放（释放弄脏的叶先并进重写集）→ 取落点 → 算脏节点 → 不在集合里就并进去重来。第 64 轮还没收拢，就在任何落盘之前返回 `AllocationRecordTreeRewriteSetDidNotSettle`；64 没有依据，见新立的欠账。
  - **盘上槽数**：取盘上整槽数，盘尾不足一槽的零头不算；写侧与读侧只有一处定义（`crates/singlefs-core/src/allocator.rs` 的 `absolute_slot_count_of_device`），分配记录树根层两侧同一个数（实二六取，主 agent 2026-09-25 定，被攻过零轮；用例 `half_a_slot_past_a_root_level_boundary_leaves_the_writer_and_the_reader_on_the_same_root_level_and_every_tree_written_reads_back`）。
  - **打开文件时自报的读数**：`ExtentTreeReadsAtOpen` 数的是读了几个 extent 树节点，不是块层的设备读次数；提示过期、经中央映射回退时每跳多 2 次设备读，两个数不相等（实二六查出，主 agent 2026-09-25 认这个口径、不另加设备读数，被攻过零轮；用例 `opening_a_file_whose_lower_extent_segment_has_two_levels_reports_as_many_node_reads_as_the_block_layer_counts` 钉相等的那一格，`a_leaf_of_a_two_level_lower_extent_segment_with_a_stale_location_hint_reads_back_every_data_unit_through_the_central_mapping` 钉不等的那一格，都在 `crates/singlefs-harness/tests/tree_nodes_and_the_central_mapping_on_mounted_read.rs`）。
- 两棵树的节点照样带 D18（块里携带什么信息） 已定项 18 的码 2 头（树 ID、层级、key 区间、诞生代号），树高照 D8（核心索引结构） 已定项 11「树高 = 根节点头层级 + 1」读。

**射程**：⚠️ 异构只放给派生态、可重建的两棵树：分配记录树与 extent 树；记账树（权威态）、中央映射树、树表、实例表、inode 树照一套码 2 btree 做（D8（核心索引结构） 已定项 11 的分裂与收缩）。异构结构照样要：单个节点自描述（哪棵树、哪一层、管哪一段 key、第几代，D18（块里携带什么信息））、损坏可检出（D21（权威态与派生态的分界） 已定项 11）、每一种新写序进层 0 崩溃点、checker 另有一套判它的代码。那两棵的结构在三方 `m2-keyspace-r1` 判。⚠️ 「日志结构节点」说的是**内存里的排布**——盘上每次落盘是整单元 COW 重写，按已定项 11 的单一定宽条目数组写（声明长度恰等于条目数 × 条目宽），不是多段追加的 bset。⚠️ **write buffer 前端第一版不实装**（已定项 10：记账直落记账树叶）；前端与节点消息缓冲区怎么衔接在已定项 12，条目要满足的三条硬要求在已定项 15。索引结构可以多样、事务层不许多样那条纪律在 `.claude/rules/fs-design.md`，本项只定索引这一侧。⚠️ 叶宽 W 的具体取值、上段扇出、上段叶条目的完整字段表，条款没写到的由实现员照上面几条取、交回里写明，主 agent 定后补进这一节。⚠️ 三种新写序（长高多层时位置 0 那一串节点同一次发布写、矮下去多层时同一串一起释放、内联与下段互换）要进层 0，checker 判定随实现一起做。⚠️ 这一节的四项都被攻过零轮；「全局按位置＋内联」的崩溃历史没扫（用户定全量崩溃最后统一跑）。⚠️ 被否掉的候选：全局按 key 排（上段要分裂）、每文件一棵根住 inode 记录（要改已冻结的 inode 记录）、全局按位置不内联（一次挂载里连续建到第 14625 个小文件就假性报满，违反 D3（空间分配） 已定项 9 第 1 条）——理由在判决第三节。

**依据**：

- 无实验：一套 btree 实现配多 keyspace 的形态没有可量的量，依据是 2026-08-29 现查 `linux-6.17/fs/bcachefs/bcachefs_format.h` 的 `BCH_BTREE_IDS()` 宏（数 `x(` 行与带 `BTREE_IS_write_buffer` 的行，源码固定在 `~/code/fs-refs/`）：bcachefs 用**一套 btree 实现 + 21 棵树**，其中 **5 棵**走 write buffer（`lru` / `backpointers` / `deleted_inodes` / `rebalance_work` / `accounting`）；比「按访问模式选不同结构」好在**只有一套结构要调对**——异构方案意味着 N 套实现、N 套崩溃一致性 bug、N 套 checker。prior-art.md 的 bcachefs 那一行以这一段为准。
- 用户定案 2026-09-24：派生树可以异构（原话在变更史）；多出来的那套实现、崩溃一致性与 checker 由层 0 流与 checker 各自覆盖，不再拿「只有一套结构要调对」挡。
- 无实验：三方原型（`research/prompts/m2-keyspace-r1-opus-model/`）在同一批历史与代价模型上量过，没立实验号，选哪一个由用户定，判决 `research/prompts/m2-keyspace-r1-main-verification.md`。
- 用户定案 2026-09-24：K1、K2、K4（原话在变更史）。
- 实二一报告 `research/prompts/m2-keyspace-implementer-report.md`：实现员取值，主 agent 认，被攻过零轮。
- 实二六报告 `research/prompts/m2-e26-implementer-report.md` 第六节 Q3、Q4：盘上槽数与打开文件自报读数两条取值，主 agent 认，被攻过零轮。
- E142（新池新建文件的干跑） 第十五次跑第五段（`research/prompts/e142-r15-step5-runner-report.md` 第四节）：分配记录树内部与根节点那两格、extent 上段叶 key 区间那一格，独立装置的变体臂只有这一种读法逐字节对上 crates；这是归因，不是在装置主读法上验过，主读法按这几格改完的重跑在后面。

**欠**：C211（节点落盘形态没有正文）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:443-460`（整段抄，未转述）**

```markdown
#### 已定项 15：write buffer 条目的三条硬要求

**定案**：

1. **排序会摧毁局部性而不是创造它 ⇒ 节点缓存必须是抗扫描的**（2Q / LRU-K 之类）。按叶排序之后每次 flush 就是一次覆盖整个叶子空间的顺序扫描，而 **LRU 对「反复的、大于缓存的顺序扫描」命中率恰好为 0**。**这是硬要求，不是建议。**
2. **条目形态必须是幂等完整值，不许是增量 Δ**。记账那一棵咬得最紧——「释放了 4096 字节」的自然表达就是 Δ，处置见已定项 1 的定案。本工程因此与 bcachefs 分道：其 accounting 树是 delta 形态（`disk_accounting_format.h` 原文「updates are _deltas_」）。
3. **条目必须自带序号（seq），这是格式要求**。flush 时按 key 排序**丢弃时间序**，而去重规则是「后者胜」⇒ **「后者」必须由条目自己携带**；用不稳定排序时同 key 条目的相对次序未定义，去重结果因此不确定。

**射程**：三条管的是 write buffer 这条路上的条目与缓存策略，宽度与落点在已定项 7、取值规则在已定项 10、与节点消息缓冲区的衔接在已定项 12。⚠️ 要求 2 与要求 3 是两条不同的要求：幂等管「重复应用不出错」，序号管「谁覆盖谁是确定的」。⚠️ **已知代价**（bcachefs 原文）：write buffer 的更新无序且最终一致，flush 时按 key 排序丢弃时间序，且这个 sort-merge-sweep **本质单线程**，在多线程负载下它的效率是关键路径。这句有两种读法，实测结果相反——**线程读法**（合并的 CPU 吞吐撑不住多线程负载）**不成立**：单线程合并吞吐按每 4 KiB 用户写 3 条条目算远超盘的带宽，按叶分区还可并行；**I/O 读法**（flush 的树插入 I/O 是关键路径）**成立，且是三项里最大的一项**。⇒ **关键路径上真正卡住的是 D9（加密） 的加密，不是合并**；⚠️ E17（write buffer 合并上限） 只量了合并这一段，不含树插入与 I/O。

**依据**：

- E7（离线索引 harness）：本工程这一侧的 io/op 在缓存占树 2% / 6% / 25% 三档上逐位相同，干预证实保持同叶聚合不变、只打乱组的处理顺序就能让 25% 缓存档的读次数下降，100% 缓存档打乱前后完全相同（阴性对照）——LRU 对「反复的、大于缓存的顺序扫描」命中率恰好为 0，所以要求 1 是硬要求；同一装置量出 flush 的树插入 I/O 折算下来远大于加密与合并，I/O 读法成立。
- E14（I-3.1 的判别力：幂等完整值 vs 增量 Δ）：增量 Δ 形态下 I-3.1（已分配统计对得上） 判别力归零，幂等完整值形态下全抓——要求 2 由它钉住。
- E17（write buffer 合并上限）：第一版漏了 `seq`，两条臂与真值当场对不上，逼出要求 3 的可判定形态；同一实验的单线程合并吞吐与按叶分区的并行倍数把「sort-merge-sweep 本质单线程」的线程读法判为不成立。

**欠**：C15（write buffer 混入 Δ）。

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:461-489`（整段抄，未转述）**

```markdown
#### 已定项 16：第一版 unlink：只立 deleted_inodes、从尾部分批、挂载里续做到空

**定案**：

1. **意图**：意图那次发布把 inode N 的记录写成 nlink = 0（size 不变），`deleted_inodes` 加 key = N（8 字节、无 value、不带对象出生代），往墓碑树开放容器追加一条墓碑（D18（块里携带什么信息） 已定项 24）；不立 logged_ops（K-U1 L4）。
2. **分批**：从尾部按数据单元个数分批，每批释放尾部 K 个单元、size 改成「剩下单元数 × 净荷容量」；K 产品取 `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`（144，自由选取），测试由只供测试的旋钮压小；删除批只重写被截掉那一片下段叶与它的祖先（K-R3-1，extent 写法的新分支，要过 `.claude/rules/fs-design.md` 五条硬要求）；最后一批释放剩下全部单元、删 inode 记录、删 `deleted_inodes` 那条 key，同一次发布；只有这一条路，不设一次做完的快路。意图那次、每一批与续做的每一批各占一个事务号，等于本实例的下一个（与普通发布同一套编号）。
3. **建树**：`deleted_inodes` 在第一个文件版本那次发布零根注册；它是派生态：队列 = inode 树里 nlink = 0 的号集合（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V7）。
4. **读者与写入口**：`InodeRecord` 的 nlink 由 `parse` 读、`to_bytes` 写结构里的值；现造记录的入口拒「nlink = 0」与「没有这条记录」；从意图提交起只读挂载、冷走读、模型读回答「没有文件」；写入口拒 nlink = 0 的 inode（`research/prompts/m3-delete-r2-main-verification.md` 第二节第 1 行、第三节 V6）。
5. **删空**：「有树、文件集合为空」是 `RebuiltVersion` / `PreviousVersion` / `PoolVersion` 的第三个成员，不借 `WithoutFile`；判定 = 树表有条目 ∧ inode 树零条目根（key 区间 `[0, 0]`，照已定项 6）∧ extent 树零条目根（key 区间按位置规定）∧ `deleted_inodes` 根指针置零；不看墓碑树与谱系树。删完之后只能做 `publish_new_inodes`（size 0）、写行与空发布（K-U5 X1）（`research/prompts/m3-delete-r3-main-verification.md` 第三节 G5）。删空的一版上覆盖写、顺序写、unlink 拒成已有的「没有文件版本可改」那一种拒绝（`UserChangeRefused::NoFileVersionToChange`）。
6. **续做**：`mount_writable` 在写行与暖机之后、交回之前把队列走到空；不进取号前预演；会话里的队列每次从现行那一版读、不缓存；崩溃注入加 `WritableMountPhase::Continuation`（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V3）。会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的 `deleted_inodes` 队列走空，再做它自己；挂载续做照旧；unlink 一次调用 = 走空 + 意图 + 全部批（与续做共用同一个走空）。会话走空时删除类发布（意图那次与分批的每一批）被落点那一判以空间拒，推一串抬 F、回收后再试；每一次被拒的删除类发布各算一次准入（一次准入最多 `PUBLISHES_PER_ADMISSION_AT_MOST` = 8 次发布，`crates/singlefs-core/src/mount.rs`）；推满仍放不下交回「抬 F 之后仍没空间」的拒绝（`UserChangeRefused::NoSpaceAfterRaisingTheFloor`），已发成的留在盘上。挂载续做走空时删除类发布被落点那一判以空间拒，同样推抬 F 再试，预算口径与会话走空相同；推满仍放不下时挂载照样交回可写、队列留着，之后会话里每次用户改动先走空、走不动报空间不够，正常卸载照常做成，靠的是 D16（发布语义） 已定项 1「正常卸载」那一行当场可用的那一截回收出来的槽（与 D16（发布语义） 已定项 1「准入」那一行同一定案）。任何一版里 `deleted_inodes` 至多一条 key；多于一个根装得下那一支拒成类型化错误、不是空间不够。
7. **幂等的判据五条**：① 计划相等；①b 同一份崩溃镜像各自重挂载续做同一批，两份盘逐字节相同（意图那次与每一批的改动计数是本次发布的 checkpoint_txg；意图那次的三个时间秒由调用方给、共一个值，各批照抄，续做的几批不另取时间）；② 删完再续做一个写都不发；③ 每个写点截断、重开续做到底，终态逻辑内容与不崩溃的终态相同、每个崩溃态 checker 全绿；④ 每个被判状态的记账标签 4、6 等于从这一版镜像现算的值。「逻辑内容」含 inode 每条（号、对象出生代、size、nlink）、extent 单元序号集与内容校验和、`deleted_inodes` key 集、墓碑记录多重集（含死亡 txg）、谱系记录多重集、标签 12 / 4 / 6、读者的答案；不含实例号、根与单元头里的 txg、改动计数、落点、单元头的诞生代号与写序、逐盘标签 1 / 2 / 5 / 10 / 11。另加 harness 层每种发布的重写角色集合钉绝对值（零轮）（`research/prompts/m3-delete-r2-main-verification.md` 第三节 V1；`research/prompts/m3-delete-r3-main-verification.md` 第三节 G2）。

**射程**：第一版只做单文件 unlink、按 inode 号、无目录、不带 truncate（用户 2026-09-28 定）；删完之后不再建带数据的文件（K-U5，记欠账）；崩溃流接在 `crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs` 那条流后面 unlink 它的 3 单元文件（K-U6 F1），截断成 0 另立一条流。只改尾叶的 extent 分支、写入类发布按发布之后的标签判、重写角色集合钉绝对值被攻过零轮；第 6 条增补的会话里每次用户改动先走空、与任何一版至多一条 key，也被攻零轮。

**依据**：

- 无实验：数都在三方腿的副本原型上量（三份删除判决），入库装置「删除借保留量的环窗口」在量环窗口那一半、还没出量。
- 三方判决 `research/prompts/m3-delete-r1-main-verification.md`、`research/prompts/m3-delete-r2-main-verification.md`、`research/prompts/m3-delete-r3-main-verification.md`。
- 用户定案 2026-10-01：K-U1、K-U5、K-U6、K-R3-1（原话在变更史）。
- 判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节队列上界与接口那一行：回退到意图那一版会把号带回队列，只有「会话里每次用户改动先走空」与「回退之后续做」两个臂在放开用户动作的历史上队列长度与有队列时的写次数两项都守住。
- 用户定案 2026-10-01：会话里每一次用户改动先走空（岔路单 `research/prompts/m3-blanks-r1-forks.md` 的「会话里的续做点」一行，原话在变更史）。
- 用户定案 2026-10-02：会话走空被空间拒推抬 F 再试；2026-10-03：每一次被拒的删除类发布各算一次准入、挂载续做被空间拒同样推抬 F 再试、推满仍放不下照 D16（发布语义） 已定项 1 做成可写、删除类发布各占一个事务号（原话在变更史）。
- 代码轮判决 `research/prompts/m3-unlink-code-r1-main-verification.md` 第二节 K2 那一行（续做、走空与推抬 F）与第四节（挂载续做推满仍放不下的结局交用户）；实现：会话走空推抬 F 是 `crates/singlefs-core/src/mounted_session.rs` 的 `publish_the_queue_until_it_is_empty_raising_the_floor_for_space`，事务号是 `crates/singlefs-core/src/transaction.rs` 的 `publish_deletion_step`。
- 主 agent 2026-10-02 定 Q-L 照收（调度记录 `records/2026-10-01-删除与并发事务.md` 第 320 行）：删空的一版上覆盖写、顺序写、unlink 拒成已有的那一种拒绝。
- 用户定案 2026-10-03：「卸载必须做成」，续做推满仍放不下之后正常卸载靠 D16（发布语义） 已定项 1「正常卸载」那一行放开的那一截回收出来的槽做成，单盘幸存那一问取「甲 GA（推荐）」（原话在变更史）；设计轮判决 `research/prompts/m3-unmount-after-short-continuation-r1-main-verification.md`，代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 N1 各行。
- 用户定案 2026-10-03：删除步算改动（「算改动，改实现」），时间字段取「乙 意图带时间、三格同改」（原话在变更史）；代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 N3 那一行。

**欠**：C618（删完之后不能再建带数据的文件）、C619（墓碑树根长高那次发布没有崩溃流）、C614（回退时墓碑树取错与删完对象无墓碑单镜像判不了）；C118（`deleted_inodes` 树的形态无落点） 还清。

```

**出处 `.claude/kb/invariants.md:1-8`（整段抄，未转述）**

```markdown
# 不变量清单

**checker 是 invariants.md 的可执行形式。** invariants.md 每加一条，checker 就要加一个检查。
两者不同步的 commit 一律不收（见 [format-evolution.md](../rules/format-evolution.md)「硬约束」）。

每条不变量必须写成**可判定**的形式——能对着一个镜像回答"成立/不成立"。池级 checker 对一条不变量的判定有四种：成立、不成立、不适用（必须带理由，不许静默算成立）、部分没判（`InvariantVerdict::PartiallyJudged`，`crates/singlefs-checker/src/image.rs`）。部分没判是判到的那一部分都对得上、射程里另有一部分这张镜像上判不了，说明逐条列出没判到的是什么；它不是红，也不是成立：层 0 与崩溃放量把它算成不红，层 0 计数并进「不适用」那一格，GPU 判器只记违例位、它在那一侧与成立、不适用同是 0。今天报它的不止一条，逐条见各行（例：I-5.4（分配记录罩住的槽互不相交）、I-5.5（从 mkfs 水位建树那一版的 t1 落点）、I-9.28（全部可读根里同一树 ID 的诞生 txg 相同））。
写不成可判定形式的，说明还没想清楚，不许进 invariants.md。

```

**出处 `.claude/kb/invariants.md:9-20`（整段抄，未转述）**

```markdown
## 状态

**新池新建文件阶段（2026-09-14 起）。** 磁盘格式已落成字节：mkfs、新池新建文件的写路径与冷启动恢复在 `crates/singlefs-core`；checker 在 `crates/singlefs-checker`，与实现只共享常量模块（D13（验证路线） 已定项 5）。
池级 checker（`walk::check_pool_image`）判 84 条（`crates/singlefs-checker/src/image.rs` 的 `IMPLEMENTED_INVARIANTS` 的长度；状态列写「已实现」的行用 `grep -c '^| I-.*已实现' .claude/kb/invariants.md` 数，也是 84 条，其中 I-3.5（引用区间的精确性） 是部分实现：出现规则已实现（下标 65），③ ④（deadlist 那两条）未实现，I-4.4（意图只描述目标态） 在清单里（下标 73）、第一版恒报不适用；第一版 23 条，2026-09-16 里程碑「覆盖写、释放、回退与复用」步 3 加 I-3.8（实例表行唯一且低于挂载根），2026-09-17 步 6 加 I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽），2026-09-18 加 I-3.9（释放代落在停止引用它的那一格区间里）、I-9.14（树表条目的诞生 txg 跨根不变） 与 I-5.4（分配记录罩住的槽互不相交），2026-09-21 加 I-1.8（归并后版本全序）、I-7.3（环健康性）、I-8.6（反向链算法） 与 I-8.7（实例内事务号不重号），2026-09-22 加 I-1.10（码 2 条目宽等于字段表宽）、I-9.6（水位大于两处最大号）、I-9.12（分隔 key 落在孩子区间之外）、I-1.2（块头写序已发布） 与 I-4.2（无被引用未提交块），2026-09-23 加 I-8.8（前缀里的事务不被切开） 与 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号），2026-09-24 加 I-7.9（回退下界 F 不高于抬 F 的上限）、I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）、I-3.11（已分配减 defer 等于最新根走读） 与 I-8.9（一次发布的记录序号连续且只有末条带标志），2026-09-25 加过 I-7.10（已停用·回退见证表删了） 与 I-7.11（已停用·见证择根删了）、2026-09-26 随回退见证删掉退役，同日加 I-7.12（系统配置 F 不低于同盘根上的 F），2026-09-27 加 I-1.11（映射 key 与单元头相符），同日加 I-7.13（系统配置池级字段在读者收的范围里） 与 I-9.16（树表条目按树 ID 严格升序且合发号次序），2026-09-28 加 I-7.14（系统配置本盘设备号等于盘在池里的身份），2026-10-02 加里程碑三二·1 那一批 16 条（I-1.12（指针出生树等于被指单元头）、I-3.12（实例表行 T 低于引用它的根的 txg）、I-3.13（记账行代段等于这一版的 txg）、I-4.1（重放后树可遍历）、I-4.9（非最新候选根引用的单元不晚于它）、I-7.5（根槽按判定宽度对齐）、I-7.15（各盘所选系统配置槽设备数相同）、I-7.16（系统配置槽世代号与槽号的关系）、I-7.17（根槽位置等于 txg 算出的位置）、I-8.1（环几何够大）、I-8.5（tail 槽有效集非空）、I-8.10（记录新根段与同代根相同）、I-8.11（点名项与被指单元头相同）、I-8.12（同实例记录的 txg 不减）、I-9.25（树表条目的 previous_snapshot_txg 取值）、I-9.26（inode 改动计数落在出生代与容器诞生代号之间）），2026-10-04 加 I-9.27（树表条目的头 ID 取值）、I-5.5（从 mkfs 水位建树那一版的 t1 落点） 与 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同），2026-10-05 加里程碑三·3 删除线那五条：下标 69 的 I-9.5（删除队列里的号仍在树里）、下标 70 的 I-3.4（可用空间扣待删占用）、下标 71 的 I-4.7（删除中 inode 有账）、下标 72 的 I-4.6（中间态其余不变量成立） 与下标 73 的 I-4.4（意图只描述目标态），2026-10-06 加下标 83 的 I-3.14（实例表每片记录形态合法）），状态列写「已实现」的就是这 74 条（I-3.5（引用区间的精确性） 只算出现规则那一半，I-4.4（意图只描述目标态） 的判定恒是不适用）：除 I-4.4（意图只描述目标态） 外每条配一份改坏了正好触发它的镜像（`crates/singlefs-harness/tests/checker_known_bad_images.rs`，删除线那几份的搭建与改法在 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`），层 0 的每个崩溃状态都跑它。
I-7.7（系统配置实例代号不低于根环） 2026-09-14 按 C322（取号那一步的屏障怎么放没有条款） 定案改成 ① ②，层 0 里 0 个状态判红；I-3.1（已分配统计对得上） 与 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 的 checker 读法 2026-09-14 由用户收尾弹窗定甲，写在各自那一行。
其余条目仍写「未实现」：它们要覆盖写、释放、回退与复用（释放、重用、残留）、journal 一侧的那一类检查、快照或加密，新池新建文件的镜像上判不出东西。

违例位是 128 位（4 个 u32 字，`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 的 `POOL_CHECKER_VIOLATED_BIT_CAPACITY`），位 i 对 `IMPLEMENTED_INVARIANTS` 的第 i 条（`pool_checker_violated_bits` 读成 u128）；今天用了 74 位；`POOL_CHECKER_COVERED_INVARIANT_INDEXES` 列 74 个下标，就是 `IMPLEMENTED_INVARIANTS` 的每一条（下标 65 的 I-3.5（引用区间的精确性） 走读内核在回退候选集的版本都走完之后逐对判；66 是 I-9.27（树表条目的头 ID 取值）、67 是 I-5.5（从 mkfs 水位建树那一版的 t1 落点）、68 是 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）；69 是 I-9.5（删除队列里的号仍在树里）、70 是 I-3.4（可用空间扣待删占用）、71 是 I-4.7（删除中 inode 有账）、72 是 I-4.6（中间态其余不变量成立），这四条由扫描内核的删除态扫描在回退候选集的每一版上判，读不全的版不置位；73 是 I-4.4（意图只描述目标态），第一版池级 checker 恒报不适用、内核不置位），没有「内核还没判」的那张表；I-9.6（水位大于两处最大号） 墓碑那一半也在扫描内核里判、并进下标 41 那一位；编译期断言覆盖表按下标升序、一条不缺一条不重（第 k 项是 k）（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行判定的位宽加宽与 GPU 内核补齐）。

<!-- invariant-count -->
现共 105 条在用（编号至 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同），另有六条退役，编号不回收）。这一句是当前条数的权威登记位；「## 历史版本」里那几句「现共 N 条在用」记的是当时的数，不跟着改。

```

**出处 `.claude/kb/invariants.md:21-47`（整段抄，未转述）**

```markdown
## I-1 块自描述 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-1.1 | 块头自述逻辑地址 | 任一单元，其头记录的**类内身份**与实际引用一致——数据单元的身份是五元组（锚点偏移即它的逻辑地址）；索引节点是树 ID + 层级（+ 该树已定携带的 key 区间，D18（块里携带什么信息）已定项 2 口径）；打包记录单元（码 3）是（出生树 ID, 打包记录类型, 容器号, 容器出生代）四元组，「父级」按打包记录类型分——类型 1 走容器索引的身份引用（C103（容器索引无落点）），类型 2 走 inode 树内部节点条目的身份引用（I-9.2（条目身份与子头相符）），**类型 4（实例表）走根记录的物理指针，不属于任何树、不进容器索引**。**自证单元（journal 记录 / 根槽 / 系统配置槽 / 类型 4 实例表单元）显式豁免**：无父引用无树身份，自证纪律在 I-7（根环）/ I-8（journal）两类与 D22（单元原子性怎么合成）已定项 21；多层码 2 树（记账树、中央映射树）的索引节点，身份里的层级与 key 区间按引用它的父条目核：层级 = 父 − 1、区间落在父条目给的那一段、内部节点区间是子树覆盖区间；按位置寻址的两棵树（分配记录树、extent 树），节点头里的层级与 key 区间等于它的位置规定的那一层、那一段；内部条目的 key 是某个孩子那一段的起点，孩子按位置严格递增；分配记录叶里每条记录的起点与末槽都落在叶罩的那一段里；extent 上段叶条目的 inode 号、下段叶记录的单元号落在叶罩的那一段里，上段叶条目的标签只能是 0、1、2（标签 0 载荷全零、标签 1 载荷末 2 字节为零）；码 2 节点里的条目按 key 严格递增，同 key 两条判红（`deleted_inodes`（D8（核心索引结构） 已定项 16）里同一个号两条就归这一条，不归 I-9.24（删除队列至多一条 key））；根之下没有空节点（这一半 checker 由实二五补）。 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；`walk::Walk::walk_code_two_subtree`；坏镜像用例 `the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root`；`crates/singlefs-checker/src/position_addressed.rs`；码 2 节点 key 严格递增由 `crates/singlefs-checker/src/lib.rs` 的 `check_index_node_keys` 判，`deleted_inodes` 同号两条的坏镜像 `crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs` 的 `a_deletion_queue_holding_one_number_twice_reddens_only_the_key_order_invariant`） |
| I-1.2 | 块头写序已发布 | 任一单元，其头记录的写序 (i, n)（D18（块里携带什么信息）已定项 7 / 11 的字段）与诞生代号 b 按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的全局已发布谓词判成已发布：i < 挂载根实例且无该实例的行 ⇒ 已发布，有行 (i, T_pub, W) ⇒ 码 1 要 b ≤ T_pub ∨ n ≤ W、码 2 / 3 要 b ≤ T_pub（所选根那个实例的 T_pub = 重放之后那个根的 txg，严格介于所选根的实例与新实例之间、又不是切换照旧事务写序主人的实例写 (i, 0, 0)，切换的所选根按 D23（journal 的角色与格式） 已定项 14 取被重发的在飞 checkpoint 所基于的根；W = **属于实例 i 的**、被这次重放施加的最大事务号，切换时是被照旧的、写序属于 i 的最大事务号；同一次恢复写出的那批行里只有所选根那个实例的 W 可能非 0，同一次切换写出的那批里只有被照旧事务的写序主人的 W 可能非 0，表上允许有多行非 0，它们来自不同次的恢复或切换；2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）；i == 挂载根实例 ⇒ b ≤ 挂载根的 checkpoint_txg；i > 挂载根实例 ⇒ 判损坏。**例外**：打包记录类型 4 的实例表单元不走这条谓词，它的现行版本由根记录持有的物理指针唯一确定。**判不成已发布者必是未发布的撕裂事务垃圾或被抛弃时间线的残留**——这是扫描方向独立于遍历方向判它的依据（D13（验证路线） 已定项 12 双方向要求）；2026-09-05 前它只比诞生代号与当前 txg，崩溃后 txg 重发让那条判据对同 txg 两条时间线失效（E104（扫描重建的现行版本判定）：无实例表时崩溃世界复活 6） | 已实现（2026-09-22，池级 checker `walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`，**只判遍历方向那一半**：走读到的每个被引用单元，头里的诞生代号 b、写序的实例代号 i 与尾段（码 1 是写序的事务号 n、码 2 / 码 3 是出生序号）要与**引用它的那条指针**记的相同——不同则「头记录的写序」不是这个单元的真实身份，谓词判的就是假输入；口径同恢复路径今天的前置检查（`crates/singlefs-core/src/recovery.rs` 五处报 I-1.2（块头写序已发布） 的 `InvariantViolated` 里「出生身份与指针不符」那三处；「诞生于根之后」那两处是谓词那一半，归 I-4.2（无被引用未提交块）），checker 按字段表自己解一份。⚠️ **扫描方向那一半（「任一单元」都按谓词判、i > 挂载根实例 ⇒ 判损坏）对着一份崩溃镜像判不了，没写**：① 判不成已发布本身不是违例——本行判据自己写着那是未发布的撕裂事务垃圾或被抛弃时间线的残留；② 「i > 挂载根实例 ⇒ 判损坏」只在恢复刚发布完新根的那一刻成立，而上一次挂载取了号、写出单元、一个根都没发布就崩时，盘上合法地留着 i > 最新根实例的在飞单元，那一支今天由 I-7.7（系统配置实例代号不低于根环） ① 按各盘系统配置的实例代号兜着。谓词本身照旧在用：`PublishedPredicate` 过滤 I-1.8（归并后版本全序） 的射程、判 I-4.2（无被引用未提交块）。打包记录类型 4 的实例表单元按本行的**例外**只免谓词那一半：根记录偏移 170 的实例表指针与链上每一片，指针里的出生 txg、实例代号、尾段与被指单元头里的值相等，仍挂这一条的遍历方向（已实现：`walk::Walk` 的 `judge_birth_identity_against_the_pointer`，被引用单元与实例表链上每一片都判；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 3 行）。坏镜像是 extent 树根头里的出生序号 +1、引用它的指针一个字节不动（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）；层 0 每个崩溃状态都跑它，走读一个被引用单元都读不到的状态报「不适用」） |
| I-1.3 | 块头树 ID 一致 | 任一单元，其头记录的所属树 ID 与实际引用它的树一致。中央映射树不进树表，「实际引用它的树」取根记录（或记录新根段）里中央映射树根指针带的出生树，不写死 65540：回退到无文件那一版之后再发第一个文件版本，十二棵树从水位起重新发号（D8（核心索引结构） 已定项 8 ②），映射树的号随之变（主 agent 2026-09-23 定，C511（回退到无文件那一版之后诞生代怎么接） 第 3 步实现员查出写死 15 的读法在合法的回退历史上误红）。自证单元（含类型 4 实例表单元）与 I-1.1（块头自述逻辑地址）同口径显式豁免。⚠️ 打包记录单元（码 3）与被快照共享的数据单元头里的树 ID 是出生树，跨头共享时与引用树不同，那一情形的判法随 D9（加密） 那次「绑出生快照」的重开（C110（跨头共享与加密后元数据类的块头一致性读法））。这一射程覆盖码 3 全部打包记录类型，含类型 2（inode 树的叶，D8（核心索引结构） 已定项 6）：克隆头共享 origin 的叶时，叶头的出生树是 origin | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.4 | 块头 fsid 一致 | 任一单元，其头记录的 fsid == 系统配置 fsid（D18（块里携带什么信息）已定项 7 的字段；加密开启后与 KDF 绑定冗余，接受）。scrub 半边「本卷全部单元归属指向本卷」靠它才写得出（C74（无加密期的跨卷错向写没防线）补救的另一半） journal 记录头 2026-09-14 起也带 fsid 8（系统配置 fsid 低 8 字节，D23（journal 的角色与格式） 已定项 4）：fsid 不符的记录整条不进重放前缀、按断链处理（E142（新池新建文件的干跑） 第七次跑记的 G25） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.5 | 系统配置记管道身份 | 系统配置记录写出该镜像的管道身份（实现标识 + 版本），且非空。**字段 2026-09-14 用户定案有了**（D22（单元原子性怎么合成） 已定项 9）：自举头段「写入者身份 20」= 实现标识 16 字节 ASCII 零补齐 + 版本 4 字节 u32，第一版写 `singlefs-rs` / 1；实现标识全零判红 | 未实现（条款已写、检查仍欠） |
| I-1.6 | 类标签登记且三处相等 | 任一单元，共同前缀偏移 6 的单元类型标签 ∈ D18（块里携带什么信息） 已定项 11 的登记表（码 0 与未登记码即判损坏）；偏移 7 的 8 个 flag 位第一版恒 0，非 0 一律拒收（前向兼容政策，D15（格式冻结政策） 判定表）；数据单元（码 1）五元组首字节 == 偏移 6；打包记录单元（码 3）类身份段首字节 == 偏移 6 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.7 | 打包容器合法与判定顺序 | 任一打包记录单元（码 3）：记录数 × 记录宽 ≤ 声明长度且声明长度 ≤ 32768 − 136（含 29 字节预留位的头，D18（块里携带什么信息） 已定项 16；2026-09-14 预留位从 28 加到 29，`cap(136, 40) = ⌊32632 / 40⌋ = 815` 与按 135 算同值，40 是第一版墓碑记录宽，D18（块里携带什么信息） 已定项 24），**先于**载荷校验和判（D23（journal 的角色与格式） 已定项 1 的 B 条同一纪律）；判定顺序恒为头校验和 → 类身份段 → 记录宽 == 登记表为该打包记录类型登记的宽（类型 2 是 140，D8（核心索引结构） 已定项 6；类型 1（墓碑）是 40、类型 5（谱系）是 16，D18（块里携带什么信息） 已定项 24；不等判损坏，只需头字段）→ I-1.2（块头写序已发布） / I-1.3（块头树 ID 一致） / I-1.4（块头 fsid 一致） → 边界 → 载荷校验和 → 逐条记录，任一步失败即中止；打包记录类型 4 的两种 kind 记录另按 D18（块里携带什么信息） 已定项 11 的实例表字段表判，不套用「记录宽 == 登记表的宽」那一步（一片记录宽恒 88，条数上限由片容量给——D18（块里携带什么信息） 已定项 11 2026-09-13 从 64 改宽）（对象级的判定——I-9.7（记录三段恒零）、I-9.8（记录 inode 等于 key）——在这条中止链之外，只让那条记录 EIO）；容器头声明一种打包记录类型、一棵出生树；打包记录类型 ∈ 第二级登记表，未登记类型的容器其物理指针不得出现在任何回收 / 复用操作的输入集合 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；类型 1、类型 5 两种记录宽 2026-10-05 删除线第 22 批补坏镜像） |
| I-1.8 | 归并后版本全序 | 码 1 / 3 的可读已发布单元按「类身份段全部字段含写序」归并成组（字段表在 D18（块里携带什么信息） 已定项 7（码 1）与已定项 11（码 3）；归并键取这些字段但去掉载荷校验和那一段——它正是 ①「同一组的成员载荷相同」要比的那个量，进了键 ① 就恒真）之后，同 key 的各组两两全序键不等——码 1 的键是写序，码 3 的键是 (诞生代号, 实例代号, 事务号) 三元——诞生代号打头（同一实例连着两个 checkpoint 都不分配新事务号时两版写序逐字节相同），事务号留在末位做同一 checkpoint 内的平局破除（固定点会迭代，同一容器在一个 checkpoint 里不保证只写一次）；同一组的成员载荷相同（码 3 由载荷校验和判、码 1 由载荷 CRC 判），不同即判损坏、不择（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 副本归并，射程 w = 2；码 2 不进这条） | 已实现（2026-09-21，池级 checker `walk::check_pool_image` 判 ① 同一组的成员载荷相同；② 同 key 的各组两两全序键不等**只判码 3**——码 1 的键停着等 C464（I-1.8 对码 1 的全序键写窄了） 定案，checker 里显式写成 `TotalOrderKey::UndecidedForDataUnit`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.9 | 清扫水位内无可读垃圾 | 清扫水位（D5（快照 / 空间记账机制） 已定项 4 第 13 项）以内不存在可读垃圾；其外的可读垃圾每个 key 都被更大写序的可见版本或可见墓碑压住（管理员回退之后的镜像不在此列，见 C615（扫盘重建对挂着时的回退没有输入），出处 `research/prompts/m3-tomb-r1-main-verification.md` 第二节；C113（扫描重建时多版单元的现行版本判定无输入） 定案 P4；扫描重建的产物未晋升期间清扫禁止，这条在重建之后失效） | 未实现 |
| I-1.10 | 码 2 条目宽等于字段表宽 | 任一码 2 索引节点，头里自述的**条目宽**（D18（块里携带什么信息） 已定项 7 的自描述三段之一：key 宽 1 + 条目数 2 + 条目宽 2，偏移表在 D18（块里携带什么信息） 已定项 18、载荷布局在 D8（核心索引结构） 已定项 11）**等于**它那棵树登记的条目字段表宽度：extent 叶记录 112、inode 内部条目 120、分配记录 20、记账条目 34、记账树内部节点 108（key 22 + 子指针 86）。条目宽是**盘上读来的值**，`index_node_view` 只判了它 ≥ key 宽（extent 24、inode 8、分配记录 10、记账 22），够不着「装得下整条记录」——不判就按字段表的固定偏移切，切片当场越界（增补 3 第 6 件 panic 面普查第二族，C476（盘上内容可控时走得到的 panic 有 21 处） R1 / R3 / R4 / R13）。条目宽为 0 时条目数必须为 0（`声明长度 = 条目数 × 条目宽` 在宽 0 时恒成立，挡不住它），否则判该节点损坏（2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做）。判定顺序与 I-1.7（打包容器合法与判定顺序） 同一纪律：**先于**按字段表解条目。⚠️ **中央映射树还不判**：D19（块指针的结构与宽度预算） 已定项 10 已定映射树 key 一律 27 字节（码 2 / 码 3 末尾补零）、条目一律 55 字节，但读者只拒窄于 55 的条目，`field_table_width_of_tree_kind` 不认种类 5，宽于 55 的映射节点不判损坏，账在 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树）；**树表条目（200 字节）两边都没写**，而走读照样按固定偏移切树表节点的条目——要不要把射程扩到它，账在 C476（盘上内容可控时走得到的 panic 有 21 处） | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::walk_tree_table_entry`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；条目宽为 0 时条目数非 0 那一条 2026-09-27 实审 A3-checker-2 落地：`index_node_view_judging_a_zero_entry_width`（`crates/singlefs-checker/src/walk.rs:747`）在解码失败理由是 `Verdict::EntryWidthZeroWithEntries`、且这棵树登记了条目字段表（`EntryFieldTableRegistration::Registered`）时报违例；树表条目与中央映射树不在射程内，只记走读失败；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`） |
| I-1.11 | 映射 key 与单元头相符 | 任一中央映射条目，key 的类标签属于 D18（块里携带什么信息） 已定项 11 的登记表，且等于它的位置条目指的单元头里的类标签；key 里的出生 txg、实例代号、尾段（码 1 是写序的事务号 6 字节；码 2 / 码 3 是出生序号 4 字节，其后 2 字节补零恒 0）分别等于那个单元头里的诞生代号、写序实例代号、事务号或出生序号；key 里的出生树等于那个单元头里的出生树（码 1 / 码 3 取类身份段偏移 43，码 2 取哪个字段随 C289（码 2 的出生树取哪个字段没写） 定）。不等判损坏。 | 已实现（2026-09-27，池级 checker `walk::check_pool_image`，判定在 `crates/singlefs-checker/src/walk.rs` 的 `walk_central_mapping_entries`；key 各段的偏移按 D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10 的字段表另写一份，不用实现的解析；判到的射程：类标签在登记表里且等于被指单元头的，出生 txg / 实例代号 / 尾段等于单元头的，码 2 / 码 3 尾段后 2 字节补零恒 0，码 1 / 码 3 的出生树等于单元头偏移 43，码 2 的出生树随 C289（码 2 的出生树取哪个字段没写） 不判；映射根全零（无文件的一版）时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（映射里数据单元那条 key 的出生 txg 3 → 4）与 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`。编号已落到代码：`crates/singlefs-checker/src/image.rs:38` 的常量 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER` 现在写的值就是这个编号本身，I-1.11（映射 key 与单元头相符）） |
| I-1.12 | 指针出生树等于被指单元头 | 任一被走读的指针，偏移 34 的出生树等于被指单元头里的出生树：码 1、码 3 取头偏移 43；码 2 的出生树取哪个字段随 C289（码 2 的出生树取哪个字段没写） 定，不判（只写进状态列）。指针各段落点依据 D19（块指针的结构与宽度预算） 已定项 8（指针携带被指单元映射 key 除类标签外的各段）。inode 内部条目到叶容器那一类由 I-9.2（条目身份与子头相符） 罩住，不重判。 | 已实现（2026-10-02，池级 checker `walk::check_pool_image` 走读里的 `judge_pointer_birth_tree_against_the_unit_header`，只在两处调用：实例表链上每一片、extent 数据单元；码 1、码 3 取头偏移 43，码 2 不判，码 2 那一支不登记成「判了一部分」，那会让每份干净镜像整条报不适用；没有走到这两处指针的镜像报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；对应的账是 C294（指针里的 key 各段与被指单元头相等没有不变量）；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 3 行） |

> I-1（块自描述） 类的适用范围是**任一单元**（数据与元数据都在内，
> D20（承重面：单元的原子性与自包含） 口径），判定字段的落点与宽度在
> [decisions.md](decisions.md) D18（块里携带什么信息） 已定项 7 的字段表（码 1 / 2）与已定项 11 的类身份段表（码 3）；
> 自证单元（journal 记录 / 根槽 / 系统配置槽）对 I-1.1（块头自述逻辑地址） 与
> I-1.3（块头树 ID 一致） 显式豁免，豁免的依据是它们的自证纪律另有落点
> （I-7（根环） / I-8（journal） 两类与 D22（单元原子性怎么合成） 已定项 21）。
> ⚠️ **豁免行是穷举的，按 D18（块里携带什么信息） 已定项 11 的登记表**：码 1 / 2 / 3 在字段表里有类身份段，码 16 / 17 / 18 在豁免行里点名，
> 其余码即判红（I-1.6（类标签登记且三处相等））——这是「穷举的显式分支」作用在不变量射程上。

```

**出处 `.claude/kb/invariants.md:48-82`（整段抄，未转述）**

```markdown
## I-7 根环 —— 类

**依赖 D22（单元原子性怎么合成），形态未定。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-7.1 | 根槽有效集非空 | 全部根槽位中，**自证校验和通过**的记录集合 S 非空 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.2 | 最新根可完整遍历 | S 中代号最大者，其 checkpoint 树根指针出发按 D4（校验和位置） 递归校验，能走完全部权威态（[decisions.md](decisions.md) D21（权威态与派生态的分界） 定义：单元 + 记账），无一处失配 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.3 | 环健康性 | **环健康性**：S 中除代号最大者外，至少还存在一条更早代号的记录。不存在即判红——它意味着某次提交把上一代直接覆盖了，轮换逻辑已失效，**下一次撕裂将无路可退**。**例外只有一个**：S 全部是第 0 代（mkfs 把第 0 代种进全部区域，D22（单元原子性怎么合成）已定项 8）时判绿，崩溃后回退到这个态的镜像同样判绿。⚠️ 射程：种子没被覆盖完之前，它们自己就充当「更早代号的记录」，连续两代落进同一槽的轮换 bug 要等 R × S 次发布把种子全部覆盖之后才会被这一条抓到 | 已实现（2026-09-21，池级 checker `walk::check_pool_image`，例外（自证过的根全部是第 0 代）照判据原样实现；坏镜像建在 mkfs 刚写完那份上（三条第 0 代根的 checkpoint_txg 一起改成 1），例外那一支另有一份阳性对照；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.4 | 近 K 代块未被复用 | 回退候选集里每一个根（按实例表判仍然有效 ∧ txg ≥ F_生效，D23（journal 的角色与格式） 已定项 14）所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头。**「K 代」2026-09-13 起就指这个候选集**：平时回退下界 F 不动时是整个根环，盘紧时抬 F 缩到最近 4 个可退到的状态、且每块盘上最新的持久有效根都在内（D16（发布语义） 已定项 1）；原「K 是运行时策略、下限 ≥ 2」作废。[decisions.md](decisions.md) D22（单元原子性怎么合成）：**根环深度是块重用延迟的上界**（D22（单元原子性怎么合成）同时明令块复用的界不能是格式常量），不满足则环里的旧根是假的回退候选。「有效」按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 的回退候选集判：(i, T) 有效 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti——被抛弃时间线的根不在候选集里，这只管回退目标的选择；被抛弃时间线今天只由崩溃恢复造出（管理员回退是挂着时的一次向前发布，不抛弃任何根，D23（journal 的角色与格式） 已定项 14）。它们引用的块在离开根环之前同样不许重新分配、不许抹头（2026-09-13 用户定案，C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 取影子账，D23（journal 的角色与格式） 已定项 14；留着的理由是第一轮判决 H6：崩溃恢复落到旧根之后，被抛弃的根引用的单元被复用、之后恢复落在它上面读不出）。⚠️ F_生效 按 D16（发布语义） 已定项 1「生效」取根上与系统配置里读得出的最大值；checker 今天只读根上带的 F，带 F 的根全坏时会在合法状态上误红这一条与别的按回退候选集判的几条（第三轮判决 Y7，推的，没实现），欠 C556（checker 与层 0 不读系统配置里的 F）。崩溃后 defer 队列的内存态丢失，接手的是清扫准入「释放代 ≤ max(F_生效, 环里最旧有效根)」那条闸（谓词由 D16（发布语义） 已定项 1 给，2026-09-13；有效性按实例表判），保护宽度随之变 | 已实现（2026-09-17，池级 checker `walk::check_pool_image`，按每条回退候选集里更早的根各判一格：走读它引用的单元时校验和对不上或头用不了，就是它指着的块被复用或抹头；坏镜像是 mkfs 树表单元（只有第 0 代根还引用）改内容重封；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 被抛弃、还在根环里的根也判（2026-09-27，用户定「checker 现在就补被抛弃根那一半」，代码审阅第 7 条）：最新根的实例表有行时，根环里每条被判抛弃的根（有行 (i, Ti, ·) 且 T > Ti）各用一份自己的走读走一遍，只看读回来对不对——走出 I-2.1（校验和与内容匹配） 违例或走读失败就按这条根判红，判定丢掉、引用不进 I-3.1（已分配统计对得上） 的并集（`walk::judge_blocks_referenced_by_abandoned_roots`，用例在 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`）；根环有读不出的槽且最新根的实例表有行时，没有违例就整条报不适用、不报成立；C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 那一形在这一半上照实红，等 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 修好转绿。 |
| I-7.5 | 根槽按判定宽度对齐 | 每个根槽的**起始偏移**是判定宽度的整数倍，判定宽度 = max(声明的 `physical_block_size`, 声明的 `io_min`)（`io_min` 为 0 视为没声明）；mkfs 在任何写之前先拒不按这个宽度对齐的几何，非常规几何由 mkfs 拒、不再是合法镜像。**逐槽判，不是只判第 0 个**——偏移半个槽的槽照样跨 2 个原子单元（E41（根环槽几何 2×4×256） 实测：对齐时风险槽数 1，偏移 256 时是 2），撕裂会连带打掉相邻的槽而没有任何东西报警 | 已实现（2026-10-02，池级 checker `judge_root_slots_aligned_to_the_decision_width`（`crates/singlefs-checker/src/image.rs`）逐槽判：判定宽度取各盘所选系统配置声明的 max(`physical_block_size`, `io_min`) 里最大的（异构池取最大，`research/prompts/m3-inv-a-r1-main-verification.md` 第四节用户定案第 1 条），是声明值、注明「声明值，未探测」，探测比对仍欠 C388（checker 的判定宽度不自己探测）；mkfs 在任何写之前拒不按这个宽度对齐的几何（`crates/singlefs-core/src/make_filesystem.rs` 的 `first_slot_not_aligned_to_the_decision_width`）；异构池一份与逐槽判的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 5 行） ⚠️ **射程注（2026-09-13，D2（RAID 条带策略） 已定项 15）**：`io_min` 大于 `physical_block_size` 时按 `io_min` 判——已定项 17 收窄后根槽与系统配置槽在它的射程内。 |
| I-7.6 | 根环区域落在互不相同的盘上 | R 个区域**在 mkfs 时逐区域写下的设备身份**（D2（RAID 条带策略） 已定项 7 用户定案逐字「归属不再按 `(r × P) mod devs` 算，mkfs 时逐区域把设备身份写下来」）取满 min(R, devs) 个不同值，且没有一块盘背超过 ⌈R / devs⌉ 个区域（鸽笼下界，D22（单元原子性怎么合成） 已定项 14，2026-09-13 用户定案）：devs ≥ R 时就是两两不同；devs < R 时不拒绝挂载，容错承诺按盘数降级（E87（固定结构的放置） 量过的 2 盘 R = 3：掉盘 1 最坏 1 代、掉盘 0 最坏 2 代）。取值不满或某盘超过下界即**拒绝挂载**——名义容错 F 个失败域而实际区域挤在同一块盘上，是 E47（根环失败域的损失） 实测过的形态（`even_span` 在 18 个几何格里只有 3/18 真的落到 4 块不同盘） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.7 | 系统配置实例代号不低于根环 | 可写挂载独占打开成功的那个集合里（它至少是过半，过半是准入门槛不是写入范围），每块盘的系统配置实例代号按它两槽里全部自证过（校验和过且 fsid 与本池相同）的槽读，取号用同一个读法（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）：① fsid 与本池相同的每个根记录、journal 记录、单元写序的实例代号 ≤ 各盘系统配置实例代号的最大者，任一根环区域、journal 记录槽、单元头读不出时这一半报「不适用」；② 各盘的实例代号（每盘取它全部自证过的槽里最大的那个）不等时，较大者不出现在任何根记录、journal 记录、单元写序里，镜像记不下这次挂载独占打开的集合时报「不适用」，第一版两盘就是全部盘。取号没做完、全或无失败回卷期间的代号不等由 ② 判成立，不另开例外（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的取号规则）。只在 devs ≥ R 时根环才是系统配置之外的独立见证，第一版 R = 3 > devs = 2 时两路见证的失败域是同一对盘 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） ⚠️ **2026-09-14 按 C322（取号那一步的屏障怎么放没有条款） 用户定案改成 ① ②**：checker 读每盘两槽里全部自证过的系统配置，扫描方向加读 journal 环与三类单元头的写序；层 0 全量 262165 个状态 I-7.7（系统配置实例代号不低于根环） 违例 0（旧定义「各盘择到的相等且 ≥ 根环」下取号只落一块盘的 2 个状态判红）；坏镜像三份：没人引用的实例 2 孤儿（①）、journal 环里实例 2 的记录（①）、盘 0 最大号 2 而孤儿带 2（②） |
| I-7.8 | 根记录树 ID 水位不低于全池最大树 ID | **根环里全部根记录的「树 ID 水位」字段取 max，必须 > 该池中出现过的最大树 ID。**「出现过的」由 checker 全盘扫描算出：全部单元头的索引节点类身份段里那个树 ID（D18（块里携带什么信息） 已定项 7），并上树表条目里的树 ID；**不许与运行时共用同一段代码**（D13（验证路线） 已定项 5）。取 max 的范围是「根环里**全部**根记录」不是「回退候选集」——被抛弃时间线的根仍然承载它那一代发过的号，形态与 D23（journal 的角色与格式） 已定项 14 给 checkpoint_txg 取值那句同源。**判别力**：抓两类——① 某次发布没更新水位；② 水位被实现成「本 checkpoint 发出的最高号」而不是**累计高水位**（后者的镜像是连发 R + 1 个 checkpoint 各建一棵树，第一棵的号会随根轮出环而被重发）。⚠️ **只许写成不等式**：水位是「下一个可用号」的上界，写成等式会把正常镜像判红 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 崩溃孤儿两条流（第一个文件版本那次发布的每个前缀 → 可写挂载 → 挂载之后六种动作）在 `crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs`；中间实例行（T = 0）不排除的坏镜像是 `checker_known_bad_images.rs` 的 `published_nodes_behind_an_intermediate_row_still_count_against_the_tree_identifier_watermark`（实例 2 的根槽全清零之后实例 3 写中间实例行 (2, 0, 0)，根环水位压低之后只红 I-7.8（根记录树 ID 水位不低于全池最大树 ID）；回退行那一道随回退行删掉，D18（块里携带什么信息） 已定项 11）；实现在 `walk::written_after_its_instance_was_last_applied` ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲；2026-09-24 主 agent 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 1 条补第二道，被攻过零轮）**：崩在发布之前的那个事务写出的孤儿节点不算「出现过」。扫描方向不数两类码 2 节点：① 诞生代号超过根环里最大 checkpoint_txg 的；② 写序实例 i 在最新根指着的实例表里有行 (i, Ti, ·)、Ti > 0、而诞生代号 > Ti 的——行说实例 i 只被施加到 Ti，之后写的没发布过；下一个实例第一次发布取的 txg 可以等于孤儿的诞生代号（D23（journal 的角色与格式） 已定项 14 注 3 取等号），只靠 ① 挡不住（第一个文件版本崩在记录落盘之前、再可写挂载的两条流上 130 格在合法状态上判红）。Ti = 0 的行（恢复写的中间实例行）不排除：中间实例发布过的号仍要算（D8（核心索引结构） 已定项 8 ②）。节点头 fsid 不是本池的照旧数。按字面数孤儿，层 0 全量 262165 个状态里 261891 个判红（水位 11 而盘上已有树 ID 11–15 的孤儿节点），见 records 2026-09-13-总审核 十一·七 ⚠️ **判法补一句（已实现：`walk::check_pool_image` 末尾在取 max 那一格之后再判最新那条根自己的水位）**：另判最新那条根的水位不低于全池最大树 ID。只取根环里全部根的 max 判不到把最新那条根的水位压低的镜像：攻方量到 checker 49 条 0 违例，而 core 的 `recover` 报 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 失败；core 重发树 11..18 的状态 checker 也 0 违例。用攻方那两份坏镜像钉住（只压低最新那条根的水位；建一次树之后经会话入口覆盖写 24 次把发树那条根轮出去、再把环里每条根的水位压到 11，只红它），被攻过零轮（`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 9、10 行） |
| I-7.9 | 回退下界 F 不高于抬 F 的上限 | **抬 F 的那一条根**（它带的 F 比同一实例里 txg 比它小的最新那条根带的高；新实例的第一条根没有同实例的前一条，不算抬——它带的是恢复算出的 F_生效；管理员回退是现行实例里的一次向前发布，带的 F 与前一条相同，不算抬；一次抬 F 推的几次空发布只有第一条算）根记录里那个回退下界 F（D22（单元原子性怎么合成） 已定项 7 的字段），按这条根 flags 里的**卸载记号**（D22（单元原子性怎么合成） 已定项 7 flags 位 0）分两支判：**带卸载记号的**，F 不高于根环里 txg 比它小、又落在锚在镜像最新根上的那一个一圈窗口里的最新那条根的 txg（只取这一个窗口，`crates/singlefs-checker/src/walk.rs` 5782 行一带；窗口里一条都没有时不判，同不带记号那一支的「之前的根」；盲带：F 越过洞里一条有效根时这一支看不见，欠账见 C717（回退下界上限的单窗口盲带）；正常卸载把 F 抬到现行那一版的 txg、不另判上限，D16（发布语义） 已定项 1「抬 F 的上限」）；**不带的**（准入抬 F），F 不高于**用它之前的根算出的** D16（发布语义） 已定项 1 的准入抬 F 上限：min(每块盘上最新的持久有效根的 txg, 第 4 新的非空持久有效根的 txg)，非空有效根不足 4 个时取最旧有效根的 txg。有效 = 按那条根自己指着的实例表判仍然有效 ∧ txg ≥ 抬之前的 F；「之前的根」（两支都取）取锚在镜像最新有效根的 txg 上的那一个一圈窗口里 txg 比它小的那些（窗口 = txg > 锚 − 3S，3S 是环长；用户 2026-10-04 定「乙，两支都取两个窗口」，第 10 批起锚在被判那条根上的那个窗口被这一个罩住、不再另取，主 agent 认的读法、被攻过零轮，`records/2026-09-30-里程碑三开工第一轮.md` 第 10 批 core 簇交回那一条），窗口里没有比它小的根时那条根不判；只取锚在被判那条根上的窗口时，洞里留着的上一圈的旧根会顶上来、在合法镜像上判红（随机历史 230 段红 21、150 段红 9，并上锚在镜像最新有效根上的窗口之后各剩 1 段，红在带卸载记号那一支，`records/2026-09-30-里程碑三开工第一轮.md` 第 9 批 9-6 交回那一条）；取全环时攻方五批随机历史里每批 27–46 段红（判决 `research/prompts/m3-root-ring-code-r2-main-verification.md` 第五节）；上限照 D16（发布语义） 已定项 1「抬 F 的上限」那一行，窗口里挂载那一刻读不出、之后这个进程没写过的槽当作可能是一条非空有效根、F 不越过它，这些槽 checker 看不见，checker 只按镜像上读得出的根判（推的：读故障不留在盘上）。只判抬 F 的那一条，不在后来每张镜像上重算：F 抬过之后再回退，后来的根会把当初撑着 F 的非空根判成无效，现算的上限会低于 F，而 F 不回落（用户定案 2026-09-24，理由见实二报告：按「有效根不要求 txg ≥ F」的读法，固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与两个随机种子在合法状态上判红）；非空 = 它的树表里 inode 树、extent 树的根指针与它前一条有效根（按 txg 排）树表里的不同——比的是树表里这两棵树的条目，不比树表单元自己的落点（D16（发布语义） 已定项 1 表格后那段「「非空」从盘上怎么认」）；数「第 4 新的非空」时去重：从新到旧数，一条非空根只有在它的 (inode 树根指针, extent 树根指针) 与每一条比它新、已计入的根都不同时才计入（同一段）。⚠️ 它判的是一张镜像上 F 有没有抬过头，不判 F 该不该抬（那是准入的事，D28（挂载期承诺量） 已定项 1）。⚠️ **卸载记号盘上没有别的东西核它**：准入入口误打记号、F 又抬过头，按带记号那一支放过（第三轮判决第二节 K6：(b) 丢的正是「准入抬过头却带了卸载记号」那一格）——这一格归层 0 录制流另核「卸载记号只由卸载入口打」（C557（卸载记号只由卸载入口打没有检查）），不归这一条。⚠️ F 写进了系统配置而带新 F 的根没落盘或坏了、恢复后生效值怎么取的那一类，归层 0 与 C556（checker 与层 0 不读系统配置里的 F），不归这一条。算上限要比的有效根树表读不出时，它空不空判不了、不猜：F 不高于把这些根全算空得出的上限就成立，高于全算非空得出的上限就违例，落在两者之间这条根不判 | 已实现，两支（不带记号那一支 2026-09-24、带卸载记号那一支 2026-09-26；两支的「之前的根」2026-10-04 起取两个一圈窗口的交，第 10 批起只取锚在镜像最新有效根上的那一个）：池级 checker `walk::check_pool_image` 的 `judge_rollback_floor_raises_against_their_ceilings`，根环里每条抬 F 的根各判一格，按根记录读出的卸载记号分支（checker 读成 `UnmountMarkerView`）。**不带记号那一支**：坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`（步 5 那段历史上抬到上限 11 与之后再发一版都成立、抬到 12 只红它）；固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与随机历史各档不红；层 0 `crash_enumeration_fixed_script_stream.rs` 抬过 F 的三条流要求真被评估过；`crates/mutations.tsv` 7 条变异（判法拿掉、上限取大、第 4 新改第 3 新、有效按新 F 取、不限同一实例、一条都不认、「4 个不同状态」不去重）；用户 2026-09-24 定义，里程碑「覆盖写、释放、回退与复用」收口表第 26 行。**带卸载记号那一支**：坏镜像同一文件的 `marked_root_raising_the_floor_above_the_previous_root_reddens_only_the_rollback_floor_invariant`（正常卸载之后的镜像按带记号那一支判 6 ≤ 6 成立、按准入那一支会红；把那条带记号的根的 F 改成 7 只红它）；变异「checker 把每条根都读成不带卸载记号」「带记号那一支把不高于前一条根的 txg 判成低于」；记号的写读在 `crates/singlefs-core/src/root_record.rs`（写者照 `unmount_marker` 写 flags 位 0、读者拒记号以外的位），写者、读者各有一条变异（实二）。**一圈窗口**（用户 2026-10-04 定两个窗口，被攻过零轮；第 10 批起只留锚在镜像最新有效根上的那一个）：harness 用例在 `crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs`（带卸载记号的根在洞上再挂之后不许红，用例 `an_unmount_sequence_root_after_a_hole_one_revolution_later_is_not_judged_against_the_old_root_left_in_the_hole`；两个窗口里仍照红，用例 `an_unmount_sequence_root_raising_the_floor_above_the_previous_root_inside_both_windows_still_reddens_the_rollback_floor_invariant`；准入那一支的几形同文件），`crates/mutations.tsv` 里以「第9批改钉」打头、点名带卸载记号那一支的 2 行，与以「第9批」打头、点名「一圈窗口」的 4 行；改钉之后随机历史 230 段、150 段两批 0 段红（改钉簇实现员量的，输出在 `research/prompts/m3-r1-batch9-repin-evidence/window-fuzz/`）；GPU 判器的扫描内核两支同取锚在镜像最新有效根上的那一个窗口（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `in_the_one_revolution_window`），对拍用例与 8 行变异在 checker 档、没跑 |
| I-7.10 | 已停用·回退见证表删了 | **此编号不再使用**——2026-09-26 管理员回退改成挂着时的一次向前发布，系统配置里的回退见证表删掉（D23（journal 的角色与格式） 已定项 14、D22（单元原子性怎么合成） 已定项 9）⇒ 这一格**无对象**。曾要求：每块盘两槽里自证过的系统配置槽，槽内偏移 481 起的回退见证表解得开（条数不超过 R × S − 1、条目严格升序、目标实例代号小于新实例代号、条数之后全 0），同一个新实例代号在各盘各槽里记的回退目标相同 | 不适用 |
| I-7.11 | 已停用·见证择根删了 | **此编号不再使用**——2026-09-26 回退见证表删掉（D23（journal 的角色与格式） 已定项 14），择根不再跳过被见证抛弃的根 ⇒ 这一格**无对象**。曾要求：所选根是根环里自证过、不被回退见证表抛弃的根里 (txg, 实例代号) 最大的那一条；根环里至少一条自证过的根不被见证表抛弃；见证表里新实例代号不大于所选根实例代号的每一条，所选根指着的实例表罩得住它 | 不适用 |
| I-7.12 | 系统配置 F 不低于同盘根上的 F | 每块盘分开判：这块盘两槽里自证过（校验和过且 fsid 与本池相同，与取号同一读法，D18（块里携带什么信息） 已定项 11）的系统配置槽中回退下界 F（D22（单元原子性怎么合成） 已定项 9 的字段）的最大值，不低于这块盘根槽里每一条自证过的根带的 F（D22（单元原子性怎么合成） 已定项 7 的字段）。抬 F 那一串先把新 F 写进每块盘的系统配置、过一道屏障，才发第一条带新 F 的根（D16（发布语义） 已定项 1「抬 F 那一串」），所以任何合法状态上一条根带的 F 都已经在它所在那块盘的系统配置里。某块盘两槽都自证不过时，那块盘不判；一块盘都判不了时整条报「不适用」。判别力：把「先写系统配置」挪到第一条根之后，崩在两者之间的状态必须红 | 已实现（2026-09-26，池级 checker `walk::check_pool_image` 的 `judge_system_configuration_floor_against_the_roots_on_each_device`：每块有自证过的系统配置槽的盘判一格，那块盘上一条根都没有时那一格成立；两槽都自证不过的盘不判，一块盘都判不了时整条报「不适用」。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `one_device_whose_system_configuration_floor_is_below_a_root_on_it_reddens_only_the_system_configuration_floor_invariant`：步 5 那段历史上抬到上限 11 的镜像一条都不红、I-7.12（系统配置 F 不低于同盘根上的 F） 真被评估过且成立，盘 1 两槽的 F 改回 0 只红它、红在盘 1 txg 16 那条带 11 的根上；变异「不比系统配置里的 F（每块盘恒成立）」「checker 不读系统配置里的 F（读成 0）」（实二）。条款仍是主 agent 2026-09-26 按 SysPre 的写序推的，没三方；层 0 每个崩溃状态都跑池级 checker，按判法每个状态都评估得到它（推的，层 0 没跑） |
| I-7.13 | 系统配置池级字段在读者收的范围里 | 任一自证过（magic 与整槽校验和对，`check_system_configuration_slot`）、incompat 位认得的系统配置槽（字段住 D22（单元原子性怎么合成） 已定项 9 的字段表，读者收的界在 D22（单元原子性怎么合成） 已定项 2 与已定项 16）：格式版本 = 1（`SYSTEM_CONFIGURATION_FORMAT_VERSION_THIS_CHECKER_READS`）、加密类型 = 0（第一版恒关，`SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF`）、固定结构槽距 ≥ 4096 字节（`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`）且槽 1 整槽落在根环基址之前、`physical_block_size` ∈ [457（`ROOT_RECORD_BYTES`）, 槽距]、journal 环长 ÷ 4096 ÷ F（`JOURNAL_SAFETY_FACTOR`）≥ 1 且环末端不越过单元区起始槽号（`geometry_of` 的 `fixed_structure_slot_spacing_lies_in_the_format_range` / `physical_block_size_fits_a_root_slot` / `journal_ring_bytes_lie_in_the_supported_range`）。任一盘任一槽不满足其中一项即判红：checker 报违例、不作保，且该池其余不变量一律报「不适用」，与实现整池拒绝挂载一致（用户 2026-09-27 定系统配置越界整池拒）。**判的字段与 core 收同一张表**：格式版本、加密类型、槽距（上界用同一槽自述的根环起点）、根槽宽、环长、单元区起始槽号（偏移 417）= 环长现算的起点且在 64 槽段边界上、journal 环起点（偏移 325）与根环起点（偏移 371）等于第一版常量，共八项（D22（单元原子性怎么合成） 已定项 16 第 1 条）。R（区域数）与 S（每区槽数）越界不在这一条里，仍归「这一槽不可择」 | 已实现（2026-09-27，池级 checker `crates/singlefs-checker/src/image.rs` 的常量 `SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS` 登记这个编号、列进 `IMPLEMENTED_INVARIANTS`；判定 `judge_system_configuration_values_the_reader_accepts` 在 `walk::check_pool_image` 里调用，任一盘任一槽带越界值时该池其余不变量整批报「不适用」并提前返回。**八项 checker 与 core 两边都判**：checker 解槽时判格式版本与加密类型（`crates/singlefs-checker/src/lib.rs`），其余六项在 `geometry_of` 判——先判 journal 环起点（偏移 325）= `JOURNAL_RING_START_SLOT`（1024）、根环起点（偏移 371）= `ROOT_RING_BASE_SLOT`（64）（两个起点先于槽距判：槽距上界按同一槽自述的根环起点算），再判槽距、根槽宽、环长，最后判单元区起始槽号（偏移 417）= 同一槽自述的环起点 + ⌈环长 ÷ 16384⌉（`unit_area_start_slot_is_the_slot_after_the_journal_ring`）且是 `CLUSTER_SEGMENT_SLOTS`（64）的整数倍；新加的四判各报一个 `Verdict` 成员（`JournalRingStartSlotNotTheFirstVersionConstant`、`RootRingBaseSlotNotTheFirstVersionConstant`、`UnitAreaStartNotTheSlotAfterTheJournalRing`、`UnitAreaStartOffTheClusterSegmentBoundary`），都归 `value_refused_by_this_reader`（整池不作保）；常量取自 `singlefs-format`，算式 checker 自己写。core 一侧 `crates/singlefs-core/src/system_configuration.rs` 的 `parse_slot` 读出偏移 325、371 两个字段，`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_values_this_reader_accepts` 按格式版本、加密类型、槽距（上界用同一槽自述的根环起点，`largest_fixed_structure_slot_spacing_under_the_root_ring_base`）、根槽宽、根环起点、journal 环起点、环长、单元区起点的次序判，不收的值报 `SystemConfigurationValueOutsideWhatThisReaderAccepts` 的一个成员（两个起点是 `RootRingBaseNotTheFirstVersionSlot`、`JournalRingStartNotTheFirstVersionSlot`），经 `RecoveryFailure::SystemConfigurationValueRefused` 整池拒。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：`a_system_configuration_slot_whose_encryption_type_is_on_reddens_only_its_own_invariant_and_every_other_is_not_applicable`（盘 1 槽 0 加密类型改成 1、重算整槽校验和，只红 I-7.13（系统配置池级字段在读者收的范围里）、其余不变量全报不适用），`each_system_configuration_geometry_field_the_reader_refuses_reddens_only_its_own_invariant_naming_the_refused_field`（几何字段各一格，各只红 I-7.13（系统配置池级字段在读者收的范围里）、违例说明点名被拒的那个成员）。两边收同一张表由 `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 钉住：新池新建文件之后的镜像上两块盘四个系统配置槽改同一个几何字段、整槽校验和重封，五格各一条用例，每格 core 的只读与可写挂载都拒成同一个 `SystemConfigurationValueRefused` 值、两块盘逐字节不变，checker 报 I-7.13（系统配置池级字段在读者收的范围里） 违例、其余不变量报不适用；根环起点写 0 那一格两边点名的成员不同（core 先判槽距上界、报槽距，checker 先判根环起点、报根环起点），照两边各自的判定次序钉（用户 2026-09-27 定「读字段，不等于常量就整池拒」，`records/2026-09-24-里程碑二收尾调度.md` 第 293 行；代码三方第二轮判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节「core 与 checker 对系统配置几何字段的读法」那一格）|
| I-7.14 | 系统配置本盘设备号等于盘在池里的身份 | 每块盘两个系统配置槽里，每个自证过（magic 与整槽校验和对）、fsid 与本池相同的槽，自举头里的本盘设备号等于这块盘在池里的身份。字段住 D22（单元原子性怎么合成） 已定项 9 的系统配置字段表（自举头「本盘设备号 4」）；实现一侧是 D18（块里携带什么信息） 已定项 11 里挂载核盘表那一句：可写挂载比盘表（`crates/singlefs-core/src/mount.rs` 的 `device_table_disagreeing_with`，`OwnDeviceNumberDiffersFromTheIdentityHandedIn`）。别的池的槽归 I-1.4（块头 fsid 一致） 判，不在这一条里 | 已实现（2026-09-28，池级 checker `crates/singlefs-checker/src/image.rs` 的常量 `SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY` 登记这个编号、列进 `IMPLEMENTED_INVARIANTS`；判定 `judge_own_device_number_is_the_device_identity` 在 `walk::check_pool_image` 里调用，每块盘每个自证过、fsid 属于本池的系统配置槽判一次，本盘设备号与 `ImageReader::devices` 给的这块盘的身份比。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：两块盘四个系统配置槽的本盘设备号互换，`devices_whose_own_device_numbers_are_swapped_redden_only_the_own_device_number_invariant` 钉只红 I-7.14（系统配置本盘设备号等于盘在池里的身份），这份坏镜像也在 `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 的清单里；变异表 `crates/mutations.tsv` 里对着这一判的三行（判定恒真、池级走读不调这一判、坏镜像写回原值）改法 A 的 `prove-red.sh` 各抓到（`research/prompts/m2-impl-r2-fixes-a-implementer-report.md` 第 77–78 行）。合法历史上不判违例：门禁 harness-model-differential-and-scenarios 随机历史那一趟 I-7.14（系统配置本盘设备号等于盘在池里的身份） 判绿 4155 次、不适用 0 次（同一报告第 189 行）；层 0 快档与全量没跑）|
| I-7.15 | 各盘所选系统配置槽设备数相同 | 各盘所选的系统配置槽（自证过、世代号最大者）里的设备数相同（D22（单元原子性怎么合成） 已定项 9 的字段）。不要求等于盘表：降级镜像（掉了一块盘）里 checker 手里的盘数小于设备数是合法的。形态抄 I-1.4（块头 fsid 一致） 的逐盘循环；core 的 `choose_system_configuration` 在几块盘的设备数对不上时报 `SystemConfigurationsDisagree`、整池挂不上，两边同表。 | 已实现（2026-10-02，池级 checker `walk::check_pool_image` 逐盘把各盘所选槽的设备数同第一块盘所选槽的比；坏镜像是盘 1 两个槽的设备数改成 3 并重封，只红它，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；对应的账是 C466（几块盘的系统配置设备数对不上时 checker 判不出）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 7 行） |
| I-7.16 | 系统配置槽世代号与槽号的关系 | 每块盘上，任一可择的（自证过、池级字段在读者收的范围里）系统配置槽 i：世代号 ≥ 2 时世代号 mod 2 == i；两个槽都可择时世代号相差不超过 1，相等时只许都是 1（mkfs 写下的两槽世代号都是 1，D22（单元原子性怎么合成） 已定项 16）。带越界值的槽归 I-7.13（系统配置池级字段在读者收的范围里） 整池拒，不进这一条。 | 已实现（2026-10-02，池级 checker `judge_system_configuration_slot_generations_follow_the_rotation`（`crates/singlefs-checker/src/image.rs`）：奇偶一格、差值一格，差值那一格取「两槽都是 mkfs 写的 1，或相差恰为 1」；差值与奇偶各一份只红它的坏镜像，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 7 行与第四节用户定案第 1 条） |
| I-7.17 | 根槽位置等于 txg 算出的位置 | 根环里任一自证过的根所在的槽位 (区域 r, 槽 s)：`checkpoint_txg` ≥ 1 时 r == txg mod R 且 s == (txg div R) mod S；`checkpoint_txg` == 0（mkfs 把第 0 代种进全部区域，D22（单元原子性怎么合成） 已定项 8）时只判 s == 0，不判 r。公式是 D22（单元原子性怎么合成） 已定项 16 的定案。根环补洞判决要的「槽位与 txg 对得上」就是这一条（`research/prompts/m3-ring-r1-main-verification.md` 第二节第 6 行）。 | 已实现（2026-10-02，池级 checker `judge_root_records_live_in_the_slot_their_txg_names`（`crates/singlefs-checker/src/image.rs`）：根环里每条自证过的根逐条判，第 0 代只判槽 0；公式同 core 的 `target_for_publish`（`crates/singlefs-core/src/root_ring.rs`）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 8 行） |

> ⚠️ **I-7.3（环健康性） 的价值在于它不需要真的崩溃一次才能发现 bug。**
> 轮换逻辑写错（例如连续两代落到同一槽位）在**下一次正常提交后的 checker 扫描**里就会被抓到，
> 而不是等到某次撕裂真的发生、发现无处可退时才暴露。
> 这与「校验和会不会红」是两类判别力：前者查机制活着没有，后者查数据对不对。

> ⚠️ **I-7（根环） 类验不了「失败域独立」这个假设。** 根环的多区域布局依赖
> 「盘首与盘尾的物理故障 / 掉电撕裂事件统计独立」，而对着镜像判不出这一条——
> 它只能验证「机制按设计跑了」，不能验证「设计的假设是真的」。
> 该假设**本机无证据**，见 [decisions.md](decisions.md) D22（单元原子性怎么合成） 已定项 2。

```

**出处 `.claude/kb/invariants.md:83-124`（整段抄，未转述）**

```markdown
## I-8 journal —— 类

**依赖 D23（journal 的角色与格式），形态已定（定长环 + tail 住系统配置槽），checker 未实现。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-8.1 | 环几何够大 | 每块盘每个自证过的系统配置槽里三个字段各占一格：journal 在飞记录数上限 = ⌊环长 ÷ 4096 ÷ F⌋，journal 最坏占用 = 在飞上限 × 4096，安全系数 F = `JOURNAL_SAFETY_FACTOR`（格式常量，checker 与 core 都判等于，不是下界），任一不等判红；另判一格：环里同一实例、同一 `checkpoint_txg` 的自证记录（同一次发布）不多于在飞记录数上限条。`环大小 ≥ F × 任一事务的最坏 journal 占用` 是这几格的推论，不另判。mkfs 时算、mount 时复算比对，core 挂载同判。三格任一不等时与 I-7.13（系统配置池级字段在读者收的范围里） 同口径整池不作保：checker 报这一条违例，其余不变量一律报「不适用」，与实现整池拒绝挂载一致；I-7.13（系统配置池级字段在读者收的范围里） 与这一条同时不成立时走 I-7.13（系统配置池级字段在读者收的范围里） 那一支（只红 I-7.13（系统配置池级字段在读者收的范围里），这一条报不适用）；同一次发布记录数那一格不触发整池不作保。 | 已实现（2026-10-02，池级 checker：三个字段等式在 `judge_journal_ring_geometry_fields_agree`（`crates/singlefs-checker/src/image.rs`），每块盘每个可择的系统配置槽各判三格：F 等于 `JOURNAL_SAFETY_FACTOR`，在飞上限等于环长 ÷ 4096 ÷ 该槽里的 F 取整（F 为 0 判红、不除），最坏占用等于在飞上限 × 4096（乘法溢出判红）；带越界值的槽归 I-7.13（系统配置池级字段在读者收的范围里）、不可择的槽不读几何；同一次发布的记录数不多于在飞上限那一格在 `walk::judge_records_of_one_publish_within_the_in_flight_record_limit`，取所选槽记的上限。F 那一格 2026-10-03 从「F ≥ 2」改成等于格式常量，三格不等的整池不作保那一支加在 `walk::check_pool_image` 里 I-7.13（系统配置池级字段在读者收的范围里） 那一支之后、I-8.5（tail 槽有效集非空） 之前（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 3、4 行）。坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（`each_ring_geometry_and_root_slot_alignment_bad_image_reddens_only_its_own_invariant`：F 取 1、2、4 与只改没被择的那一槽 F 各一份；`journal_ring_geometry_fields_that_disagree_redden_only_the_ring_geometry_invariant_and_every_other_is_not_applicable`：三格不等时其余不变量报不适用）。core 挂载同判：`crates/singlefs-core/src/recovery.rs` 的 `journal_ring_geometry_fields_recompute_from_the_ring_length`，放在 `system_configuration_values_this_reader_accepts` 的最后。在飞上限那一格复算取槽里的 F、core 取格式常量，F 不等于格式常量时两边都已因 F 那一格拒，不影响判红与否；对应的账是 C28（环几何不变量未实现）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 4 行） ⚠️ **口径注（D28（挂载期承诺量） 已定项 4）**：ckpt_cost 按当时各记录树的高现算、挂载期内会长；实现上最坏占用 = 在飞上限 × 4096，由环长与 F 倒算，取的是上界、不随树高变（`research/prompts/m3-inv-a-r1-main-verification.md` 回看决策那一行）。 |
| I-8.2 | 记录头不跨原子单元 | 任一 journal 记录的头完整落在一个 `physical_block_size` 单元内。该宽度**运行时探测**，不许硬编码 | 未实现 |
| I-8.3 | 重放前缀严格连续 | 被重放的记录集合，其 `jsn` 严格连续（`jsn == expected`，断号即止；链从所选根覆盖的最后一条记录之后接，跨实例边界即停，记录水位按实例代号为主比——D23（journal 的角色与格式） 已定项 14 的注，2026-09-13），且整段落在在飞记录数上限之内。⚠️ **这一条不是前缀纪律的全部**：D23（journal 的角色与格式） 已定项 14 的前缀口径共五条，这里只落了其中三条半——**「提交标记齐全的事务才施加」与「施加前逐项验证点名单元」两条不在这条判据里**，而两者的处境不同：前者 2026-09-23 已落成 I-8.8（前缀里的事务不被切开），checker 判定与崩溃点重放那一半仍欠，记在 C37（事务与记录的对应无定义）；后者**已经有了**，只是不住这一条——它住 D16（发布语义） 已定项 7，两条故障注入自证 2026-09-14 在层 0 做出（C76（发布时序没有权威落点））。⇒ 照这一条实现恢复施加逻辑的人会漏掉那两道防线，而它们正是挡住「半个事务」与「点名单元已损坏的记录」的 | 未实现 ⚠️ **判据补一句（2026-09-13，D16（发布语义） 已定项 4）**：前缀不得停在一次发布的中间——一次发布整体施加或整体不施加。 ⚠️ **再补一句（2026-09-14，D23（journal 的角色与格式） 已定项 18 用户定案）**：「在飞记录数上限」的语义就是**这个前缀最多 N 条**（N = 环槽数 ÷ F），不是 XFS 那种「最后 N 条内算撕裂、之外算损坏」；**任一校验和不过的记录一律断链即止**——停在它之前那一条，第一版不区分撕裂与损坏、不因此拒绝挂载。 |
| I-8.4 | 重放幂等 | 任一 journal 记录被重放两次与重放一次，产生的读出内容相同（等价关系取「读出的内容相同」，不比扇区、实例代号与行） | 未实现（实现在途：取 R-换，在崩溃注入的二次崩溃上判恢复加写行那一整段；被攻过零轮，实现时先在崩溃注入上量误红；`research/prompts/m3-inv-a-r1-main-verification.md` 第四节用户定案第 2 条） |
| I-8.5 | tail 槽有效集非空 | 池里至少有一块盘的两个系统配置槽里有一个槽自证过（magic 与整槽校验和对）：tail 住这些槽（D23（journal 的角色与格式） 已定项 3），这个集合就是全部 tail 见证者。全池皆废判红（mkfs 没做完、写到系统配置之前失败的盘面也是全池皆废，照判红，故障注入的已知红清单认这一类，用户 2026-10-03 定）；单盘全废不判违例，走旁路报告：另报哪块盘少于两槽自证过、可写挂载会拒。对着只读恢复判。自证过而不可择的槽（R / S 越界、incompat 位不认得）与带越界值的槽都算自证过；I-7.13（系统配置池级字段在读者收的范围里） 带越界值整池拒、或 I-8.1（环几何够大） 三格不等整池不作保的那一支里，这一条不判、报「不适用」。有系统配置槽自证过、而全池择不出一块可择的系统配置时同样报不适用（理由：这不是一个挂得上的镜像；代码 `crates/singlefs-checker/src/walk.rs` 10022 行一带），一个槽都没自证过照旧判红。 | 已实现（2026-10-02，池级 checker `judge_some_device_carries_a_self_verified_system_configuration`（`crates/singlefs-checker/src/image.rs`）：把「不自证」（magic 或整槽校验和不对）与「自证过而不可择」分成两类，后一类计入自证过（`slot_verifies_itself`），别的不变量在全池皆废时照旧报不适用；单盘全废走旁路报告 `system_configuration_redundancy_report`，不进违例通道；自证过而不可择的四槽一条都不红的一份坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（`self_verified_system_configuration_slots_no_reader_can_choose_still_witness_the_tail`）；对应的账是 C461（checker 判「挂不上的镜像」整片报不适用）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 6 行与第四节用户定案第 3 条） |
| I-8.6 | 反向链算法 | 任一 journal 记录的反向链 = CRC32C(**本实例内逻辑前一条**记录的 487 字节头，`header_csum` 那 32 字节按零参与)；**本实例写出的第一条记录反向链恒 0**（D23（journal 的角色与格式） 已定项 10 / 已定项 19，2026-09-13 用户定案，链按实例算是 2026-09-14 用户定案，头宽随 D23（journal 的角色与格式） 已定项 4 现行 487）。不等的记录不进重放前缀。jsn 的计数器为 0 的记录判违例：它没有本实例内的逻辑前一条，反向链无从定义（D23（journal 的角色与格式） 已定项 18 的落点式 `(计数器 − 1) mod 槽数 × 4096` 从计数器 1 起；用户 2026-09-27 定）。⚠️ 链按实例算，所以跨实例边界的两条记录之间**不判**这一条——实例边界由实例代号挡着（I-8.3（重放前缀严格连续)） | 已实现（2026-09-21，池级 checker `walk::check_pool_image`，逐盘判：计数器 1 与跨实例边界的第一条判反向链恒 0，同实例前一条在盘上时判链值，前一条不在盘上不判；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 里环还空着的 4 个状态报不适用，其余每个崩溃状态都判）；计数器 0 判违例（2026-09-27，`walk.rs` 的 `ExpectedBackChain::CounterZeroHasNoLogicalPredecessor`，用例 `a_journal_record_whose_counter_is_zero_reddens_the_back_chain_invariant` 在 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`） |
| I-8.7 | 实例内事务号不重号 | 判环里全部自证过的记录，同一实例按 `jsn` 的计数器看：① 按计数器相邻的两条非 0 记录，事务号不减（事务号 0 是不承载事务的空发布记录，不进这个比较）；② 一个事务号的记录在计数器上连续，被别的记录（含事务号 0）隔开之后不再出现。D23（journal 的角色与格式） 已定项 7 逐字：「事务号按实例计数、从 1 起」「记录按事务号顺序追加……这是实例表行的 W 能当精确前缀的依据」；同一项定一个事务可以跨多条记录，所以相邻两条可以同号。 | 已实现（2026-09-21 立时判「非 0 事务号严格递增」；2026-09-23 按 D23（journal 的角色与格式） 已定项 7 改成不减加同号连续。池级 checker `walk::check_pool_image` 的 `judge_transaction_numbers_per_instance`：逐盘按实例判，同一实例按计数器相邻的两条非 0 记录，号不等时后一条严格大于前一条（①），号相等时两条之间不许坐着同一实例的别的记录（②，事务号 0 的也算；中间只有读不出的槽或别的实例的记录不判红）；「被别的非 0 号隔开之后又出现」一定伴着一处 ① 红，不另判；同一实例非 0 的记录不足两条时报不适用。坏镜像三份（交错两份、事务号倒退）在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，每份只红这一条，违例说明标出红的是 ① 还是 ②） |
| I-8.8 | 前缀里的事务不被切开 | 判环里全部自证过的记录：③ 同一实例同一事务号的一组记录里，**带提交标记的至多一条，且它是这一组计数器最大的那一条**；提交标记那一字节只许 0 或 1；④ **一组里一条提交标记都没有时，这个实例在它后面不许再有任何记录**（事务号 0 的空发布也算；别的实例的记录不算）：D23（journal 的角色与格式） 已定项 7 逐字写「一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加」，实例切换之后这个实例不再写任何记录。它与 I-8.7（实例内事务号不重号） 的 ① ② 合起来，保证 D23（journal 的角色与格式） 已定项 7 原话那个恢复算法「丢掉提交标记还没出现的那个事务的全部记录」**恰好丢掉、且只丢掉**被崩溃切开的那个尾巴；`crates/singlefs-core/src/recovery.rs` 今天那一步写成「按 jsn 扫到第一条不带提交标记的记录就停」，只对一事务一条成立，一事务跨多条记录时要改，归并行线一。四条里任一不成立，恢复要么丢掉一个已经提交的事务、要么施加半个事务，而盘上没有任何痕迹。⚠️ **今天可达镜像上它恒真**：第一版一条记录一个事务、`is_commit` 全仓没有一处写 false，所以 ③ ④ 今天都不会被可达历史违反。**判别力只能靠坏镜像**，坏镜像不做这条就是恒真的检查——恒真的检查比没有检查更糟。⚠️ **它管的是条款允许的形态，不是第一版的形态**：D23（journal 的角色与格式） 已定项 7 定的是「可以跨多条，记录头带事务边界字段」，而第一版按 C310（事务切分纪律与记录数口径打架） 一条记录一个事务。I-8.8（前缀里的事务不被切开） 挡的是**一事务跨多条记录落地那一天**。⚠️ **它只判一份镜像上的记录流自洽，不判「施加的结果等于整数个事务」**：后者要拿「这次事务该有的全部效果」当 oracle，一份镜像上判不出来（形态同 I-4.3（提交原子） 那一行），归层 0 崩溃点重放。 | 已实现（2026-09-23，池级 checker `walk::check_pool_image` 的 `judge_commit_markers_per_transaction`：逐盘，一组 = 同一块盘上实例代号与事务号都相同的非 0 记录；③ 两条以上的一组里带提交标记的至多一条、且是计数器最大那条；提交标记字节每条记录都判（事务号 0 的也算），不是 0 或 1 即红；④ 一条提交标记都没有的组，后面同实例还有任何一条记录（事务号 0 的空发布也算）即红。没有 ③ 的对象也没有 ④ 的对象 ⇒ 报不适用，提交标记字节单独成立不算成立；第一版一事务一条、每条都带提交标记，层 0 每个崩溃状态都判、全部报不适用。阳性对照（一事务两条、提交标记在 jsn 最大那条，I-8.7（实例内事务号不重号） 与 I-8.8（前缀里的事务不被切开） 都成立）与四份坏镜像（提交标记放错位置、没提交的事务后面还有号更大的记录、没提交的事务后面跟一条空发布、提交标记字节写成 2，每份只红这一条）在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，违例说明逐条标出红的是哪条判据） |
| I-8.9 | 一次发布的记录序号连续且只有末条带标志 | 同一实例、同一 `checkpoint_txg` 的记录（一次发布写出的全部记录）：「本次发布内序号」依次是 1..N、与 jsn 同步（jsn 连号时序号也连号），记录标志位 0 恰好只在序号 N 那一条上为 1，记录标志其余位为 0；序号 0、一次发布之内跳号、没有末条或多于一条末条，都是违例。只判读得出的记录，读不出的那条不判（D23（journal 的角色与格式） 已定项 4、17，用户 2026-09-24 定，`records/2026-09-24-里程碑二收尾调度.md` 第三节「序号与末条标志的不变量」） | 已实现（`crates/singlefs-checker/src/walk.rs` 的 `judge_publish_ordinals_and_last_record_flags`；阳性对照与七份坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`） |
| I-8.10 | 记录新根段与同代根相同 | 某块盘 journal 环里自证过的记录 J，带末条标志（记录标志位 0 = 1），且这块盘上同（实例代号, `checkpoint_txg`）带末条标志的记录只有这一条；根环里任一自证过、实例代号与 `checkpoint_txg` 都与 J 相同的根 R，J 的新根段四段与 R 的同名四段逐字节相同：新根段 [0..86) = R[36..122)、[86..172) = R[256..342)、[172..180) = R[122..130)、[180..188) = R[130..138)（树表单元指针、中央映射树根指针、树 ID 水位、回退下界 F，D23（journal 的角色与格式） 已定项 15）。只认带末条标志那一条；同（实例, txg）有两条以上不同内容的根不判，归 C88（根环的时间线判别未实现）；末条标志不唯一归 I-8.9（一次发布的记录序号连续且只有末条带标志）。 | 已实现（2026-10-02，池级 checker `walk::judge_new_root_segment_of_the_last_record_against_the_roots`：逐盘按 (实例, txg) 分组，带末条标志恰一条才判，与根环里同 (实例, txg) 的每条自证过的根比四段，一对都没比到报不适用；坏镜像取记录一侧，把 F 改一位并重封头校验和，另有树表指针、映射根指针、树 ID 水位各一份，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 4 行） |
| I-8.11 | 点名项与被指单元头相同 | 任一自证过的 journal 记录里任一点名项的每条位置条目：若它指着的那一份单元的整单元校验和与位置条目里的相符，则单元头里的类标签、诞生代号、key 尾段与点名项偏移 28、37、45 的同名字段逐字节相同（码 1 尾段 = 写序实例代号 4 + 事务号 6，码 2 / 码 3 尾段 = 实例代号 4 + 出生序号 4 + 补零 2，补零恒 0）；出生树（点名项偏移 29）只判码 1、码 3，等于头偏移 43，码 2 随 C289（码 2 的出生树取哪个字段没写） 不判。校验和不相符的位置条目不判（落点被合法复用之后，单元头就不是它点名的那个单元了）。 | 已实现（2026-10-02，池级 checker `walk::judge_named_entries_against_the_unit_headers`：点名项每条位置条目，所指那一份整单元校验和相符才比，不符不判；出生树只判码 1、码 3；一条都没比到报不适用；类标签、出生树、写序实例代号、key 尾段各一份只红它的坏镜像，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 2 行） |
| I-8.12 | 同实例记录的 txg 不减 | 同一块盘 journal 环里自证过的记录，同一实例代号的记录按 jsn 计数器从小到大，`checkpoint_txg` 不减。换实例后严格大于不判（设备失而复得下没核）。 | 已实现（2026-10-02，池级 checker `walk::judge_journal_record_txg_per_instance`：逐盘、逐实例按计数器看 txg 不减，换实例不判，没有可比的一对时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 9 行） |

> ⚠️ **D23（journal 的角色与格式）的死锁 3 全靠 I-8.1（环几何够大），而 I-8.1（环几何够大） 的 checker 未实现。**
> 「写进本清单」不等于「有东西在拦」——取定长环那条论证依赖的是**拦得住**，不是**写下来**。
> 欠检查见 [checks-owed.md](checks-owed.md) C28（环几何不变量未实现）。

> ⚠️ **I-8.4（重放幂等） 是被实验逼出来的，不是设计时想到的。**
> [experiments.md](experiments.md) E24（恢复算法：先信 tail 会不会丢记录）测出：
> 全环扫描式恢复虽然不丢数据，却会把**陈旧 tail 以来的记录全部重放一遍**
> （窗口为空时仍重放满环 64 条）。没有幂等，「陈旧 tail 只是多做功」
> 就变成「重复施加已生效的记录」。

> ⚠️ **I-8.3（重放前缀严格连续） 与 I-8.1（环几何够大） 不许互相顶替。**
> E24（恢复算法：先信 tail 会不会丢记录）测到窗口撑爆环时（100 条 > 64 槽）
> **两种恢复算法一起丢**——恢复算法救不了几何。

> ⚠️ **I-8.4（重放幂等） 的射程有一条已实测的边界（E78（重放的起点），2026-09-02）**：
> 幂等管「同一条记录重复施加」，**管不到「旧记录的点名块已被合法复用」**——
> 陈旧 tail 的重放窗口远长于 I-7.4（近 K 代块未被复用）扣块的窗口，
> 窗口内记录的点名块换过内容后，施加时验证（E77（发布的持久顺序）证明它是承重步骤）会失配。
> 「陈旧 tail 只是多做功」因此只在不验证或有重放下界时成立；
> 重放下界的两条出路（根带 jsn 水位 / 盲放加尾删）见
> [checks-owed.md](checks-owed.md) C77（重放起点未定义）。

```

**出处 `.claude/kb/invariants.md:125-135`（整段抄，未转述）**

```markdown
## I-2 校验和 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-2.1 | 校验和与内容匹配 | 任一被引用的块，其校验和与内容匹配。按多长去比（父条目的类型段说的那一类与孩子头里的类标签不一致时）：走读往下走孩子的那一支，按父条目类型段说的那一类的长度读回比；不往下走孩子的几支（墓碑、谱系、inode 三棵树的类型段不认得、孩子自证成另一类，墓碑、谱系两棵树层级 ≤ 1 节点里类型段 0 的），每条位置条目按那一份头里类标签说的那一类的长度比，类标签不在登记表里或头读不出的，16 KiB 与 32 KiB 两种长度都对不上才算不匹配。同一块、同一对校验和，两支可以判得不同（用户 2026-10-06 定保持，`research/prompts/m3-batch29-code-r1-main-verification.md` 第四节） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；2026-09-17 起「被引用」按 I-3.1（已分配统计对得上） 那个候选集里的根算：F 之下的根引用的单元可以已被回收复用，不在它们上判——不抬 F 就回收复用时 A 的根还在候选集里、这一条红，是 C22（刚释放的块立即重分配） 的必红） |
| I-2.2 | 校验和算法全盘一致 | 校验和算法标识在整个文件系统内一致。**字段 2026-09-14 用户定案有了**（D22（单元原子性怎么合成） 已定项 9）：系统配置自举头段「校验和算法标识 1」，登记表 0 = 无效、1 = CRC-32C，第一版写 1；读到 0 或未登记的码一律拒绝挂载。算法的四个参数写死在 D18（块里携带什么信息） 已定项 17 | 未实现（条款已写、检查仍欠） |
| I-2.3 | 补齐字节为零且参与校验和 | 任一数据单元（码 1）或打包记录单元（码 3，D18（块里携带什么信息） 已定项 11），其声明长度（净荷字节数；补齐起点 = 头 + 声明长度）之后到单元末尾的补齐字节**恒为 0**，且校验和 / MAC 覆盖**整个单元含补齐**（[decisions.md](decisions.md) D4（校验和位置） 已定项 2：不足一个单元的 extent 补齐到 32 KiB；码 3 由头校验和（到头末尾）与载荷校验和（头之后到单元末尾）两段合起来覆盖整个单元）。补齐字节非零、或校验和只覆盖声明长度，一律判红。索引节点（码 2）从 2026-09-13 起同样在射程内（D18（块里携带什么信息） 已定项 18 用户定案）：声明长度 = 条目区字节数，之后到节点末尾的补齐字节恒为 0，载荷 CRC 覆盖 [86 + 2 × key 宽, 16384)——起点是明文头末尾，含 29 字节预留位、条目区与其后的补齐区，口径与码 1 的 105、码 3 的 107 同（预留位 2026-09-14 从 28 加到 29，含预留位的头是 115 + 2 × key 宽）。⚠️ **加密开启时判的是解密后的明文，不是盘上字节**（2026-09-07 用户定案，还清 C189（I-2.3 在加密开启时判明文还是盘上字节没定））：补齐在盘上是 `E(0)` 而不是 0，按盘上字节判必然全红；checker 本来就要解整单元，取值同一步就有。**与 D18（块里携带什么信息） 已定项 7 的载荷 CRC 方向相反是对的**——载荷 CRC 要给无密钥一侧用，所以算盘上字节；这一条只有 checker 用，所以判明文 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-2.4 | 头校验和覆盖范围 | 任一单元的头校验和覆盖从偏移 0 到该类明文头的最后一个字节，端点写死：码 1 到偏移 104、码 2 到偏移 85 + 2 × key 宽、码 3 到偏移 106（D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18 的字段表）；头校验和字段自身按 0 参与。**29 字节 nonce / MAC / 算法类型预留位虽然算进头（已定项 16）、也是明文，但不在头校验和覆盖内**——MAC 在头校验和之后才算得出来，罩不住它；单元头的这 29 字节、指针头部的 MAC 16 + nonce 12 与算法类型 1、压缩算法码 1、压后长度 2（D19（块指针的结构与宽度预算） 已定项 3；第一版不加密、压缩算法码登记表只有码 0、码 0 时压后长度恒 0，D9（加密） 已定项 10 / 已定项 14）由同一条规则守：加密关着时恒 0、读者遇到非 0 一律判该结构损坏（已定项 17；指针头部那一半 2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做），加密开启后由 MAC 自己罩（2026-09-14 用户定案把预留位从 28 加到 29，多的那 1 字节是算法类型）。根槽偏移 428..457 的算法类型 1 + nonce 12 + MAC 16（D22（单元原子性怎么合成） 已定项 7 的字段表）是同一条规则的第三处：加密关着时恒 0，池级 checker 对根环里每一条读得出、自证过的根判这 29 字节全 0（不按回退候选集滤），core 的根记录读者不读它们、挂载透明也不比它们（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y1「算法、nonce、MAC」那一行：单镜像性质，归池级 checker）。指针头部的 extent 偏移 2 字节第一版恒 0（第一版指针指整个单元），读者遇到非 0 判该指针所在的结构损坏，与 MAC / nonce 同一处置、与加密开不开无关（D19（块指针的结构与宽度预算） 已定项 3；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行）。根记录里四条指针（树表、实例表、中央映射树、分配记录树）头部有一段不是第一版的取值（MAC / nonce 非 0、算法类型非 0、压缩算法码非 0、压后长度非 0、extent 偏移非 0，五样任一）按这条根自证不过处置：core 的根记录读者（`RootRecord::parse_slot`，与 journal 新根段那四条指针同一判法）把这条根算不可择、退回上一条根（靠 journal 重放可以回到同一版，挂载照样成功），checker 红这一条而 core 挂载成功，两边各判各的、不矛盾（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 6 行）。加密开启（系统配置级、全有全无，I-6.4（无未加密块））后覆盖范围 = I-6.2（明文头字段白名单） 该类白名单里的全部明文头字段，密文段由 MAC 罩。改动覆盖区内任一字节必须失配、改动净荷字节不许失配 | 已实现（`crates/singlefs-checker` 的 `check_unit`：三类的端点各自写死，里程碑步 2 / 步 3 验收里改坏头任一字节判红，2026-09-14；池级走读在 `walk::judge_unit_header`（`crates/singlefs-checker/src/walk.rs:1174` 起）另判单元头偏移 4 的格式版本 = 1（walk.rs:1220，不认得就判该结构损坏、不按今天的字段表往下解）与 29 字节预留位全 0（walk.rs:1258）；走读跟随的每条指针头部按三个判法各判一遍（`crates/singlefs-checker/src/image.rs`：`judge_pointer_mac_and_nonce_are_zero`（:1381）判 MAC 16 + nonce 12 全 0，`judge_pointer_encryption_and_compression_fields_take_the_first_version_values`（:1396）判算法类型、压缩算法码、压后长度三段各一格全 0，`judge_pointer_extent_offset_is_zero`（:1456）判 extent 偏移 0），五个入口是码 2 节点指针（`Walk::read_index_node`，walk.rs:1301 起：树表单元、树表条目的树根、中央映射树、inode 树、extent 树、分配记录树的节点都经它，根记录里树表、中央映射树、分配记录树那三条根指针也经它）、墓碑树与谱系树内部条目的子指针（`walk_the_entries_of_a_container_tree_node`，walk.rs:2150 起）、inode 内部条目子指针（`walk_inode_internal_node`，walk.rs:2443 起）、实例表链指针（`walk_instance_table_chain`，walk.rs:2836 起，根记录里的实例表指针是链的第 0 片；链指针记录有下一片而位置指针头部有一段不是第一版的取值时，这一片按 I-3.14（实例表每片记录形态合法） 第五形解不开、链不往下走，那条指针不跟随、这一处 I-2.4 不判）、extent 数据指针（`walk_extent_data_pointer`，walk.rs:3009 起），只判不断、指针指向的单元照样走下去，2026-09-27 实审 A3-checker-2；根槽偏移 428..457 那一处也已实现（`image::judge_root_record_algorithm_nonce_and_mac_are_zero`，image.rs:1431，`walk::check_pool_image` 经 `check_pool_image_given_the_tombstone_container_capacity_knob` 在 `valid_roots` 的结果上调它，walk.rs:10413）；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`）。**extent 偏移那一半与算法类型、压缩算法码、压后长度那几段已实现**（checker 走读跟随的每条指针都判这三格；GPU 判器走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `w_judge_pointer_head` 与 `w_judge_pointer_extent_offset` 在同样那几个入口判，根槽那一处是同文件的 `judge_root_record_algorithm_nonce_and_mac_are_zero`；core 每个读盘上指针头部的读者都判这五段（MAC / nonce、算法类型、压缩算法码、压后长度、extent 偏移）、判该结构损坏：树表条目的树根指针、inode 内部条目的子指针、extent 叶记录的数据指针、extent 上段叶条目的下段根指针与内联数据指针、extent 树内部条目的子指针、分配记录树内部条目的子指针、实例表链指针各一处，加 journal 新根段、码 2 内部条目、根记录三处（`crates/singlefs-core/src/pointer.rs` 的 `read_judging_the_encryption_compression_and_extent_offset_fields_from`，`PointerHead`、`NodePointer`、`DataPointer` 各一个；`grep -rn 'Pointer::read_from' crates/singlefs-core/src/` 只剩 `pointer.rs` 自己的单测一处）；影子账认单元的两个读者（`allocation_record_tree::node_pointers_as_far_as_readable`、`recovery::instance_table_page_pointers_as_far_as_readable`）经 `NodePointer::read_with_head_fields_judgement_from` 的 `NotJudgedByTheShadowLedger` 不判（影子账要多隔离、不漏隔离）；走读不跟随的指针 I-2.4 不判：根环里走读没走到的旧根，它们的树表单元只被 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 读条目取诞生 txg、不跟随条目里的树根指针，这些指针的头部没人判（测过的镜像上 12 条：三个旧树表单元各四条树根指针，`research/prompts/m3-formal-code-b30-r1-sonnet-output.md`；core 把那样一条根当恢复目标时经 `TreeTableEntry::parse` 判，推的、没造盘面验证）；坏镜像遍历偏移 32、33，core 每类读者各一条用例在 `crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader.rs`，加密、压缩那几段 core 每类读者各一条在 `crates/singlefs-harness/tests/pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader.rs`，根记录四条指针各改一类字段、两种挂载都退到上一条根的在 `crates/singlefs-harness/tests/root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts.rs`；池级 checker 的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：extent 偏移落在墓碑树、谱系树的容器子指针上（`an_extent_offset_on_the_container_pointer_of_the_tombstone_tree_root_reddens_only_the_pointer_head_invariant` 与谱系树那一份），算法类型、压缩算法码、压后长度各一份（`an_inline_data_pointer_naming_an_encryption_algorithm_reddens_only_the_header_coverage_invariant`、`a_tree_table_root_pointer_naming_an_unregistered_compression_algorithm_reddens_only_the_header_coverage_invariant`、`an_inline_data_pointer_carrying_a_compressed_length_without_compression_reddens_only_the_header_coverage_invariant`），根槽算法类型、nonce、MAC 各一份（`a_root_slot_whose_algorithm_type_is_one_reddens_only_the_header_checksum_coverage_invariant` 等三条）；对应的账是 C581（extent 偏移非 0 没有判据）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行、`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 5 行、`research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L4 的容器子指针那一行、`research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2 的算法类型等三段与旧树表根指针两行、`research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2 的根记录四条指针那一行） |
| I-2.5 | 位置条目按设备身份升序 | 任一块指针、中央映射条目的 value、journal 点名项里的位置条目数组按设备身份严格升序（D19（块指针的结构与宽度预算） 已定项 11、D23（journal 的角色与格式） 已定项 17，2026-09-13 用户定案）；两条位置条目设备身份相同或逆序，一律判损坏。⚠️ **射程（2026-09-14 补）：整条 86 字节全零的指针表示「注册了但还没有根」，整条豁免升序判定**——它内部那两条位置条目都是 (设备 0, 槽 0, 校验和 0)，按字面正是「设备身份相同」那一档。今天写出全零指针的有四处：树表里 livelist 树、稀疏旁表树与 deadlist 树那三条条目（D6（快照实现模型） 已定项 2、D5（快照 / 空间记账机制） 已定项 6 与 D5（快照 / 空间记账机制） 已定项 11 各自定案的「day-1 注册进树表，根指针为零」），以及 mkfs 与暖机的根记录里那个还没发布过的中央映射树根指针。非全零的指针不享这条豁免。 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |

```

**出处 `.claude/kb/invariants.md:136-171`（整段抄，未转述）**

```markdown
## I-3 空间记账 —— 类

**依赖 D5（快照 / 空间记账机制）。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-3.1 | 已分配统计对得上 | 已分配空间统计 == 实际遍历所有引用得到的和 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲）**：「实际遍历所有引用」按根环里全部有效根的引用取并集——新池新建文件之后第 0 版树表单元只被更早的根引用、仍占着空间（已释放、在 defer 队列里），只按最新根算会少 1 槽。**2026-09-17 起「有效根」= 回退候选集里的根**：按最新根指着的实例表有效（无那个实例的行，或有行 (i, Ti, Wi) 且 T ≤ Ti）且 txg ≥ F_生效（D16（发布语义） 已定项 1「生效」）。遍历的是最新根与不被抛弃、不低于 F 的有效根（RGall：候选集不再滤一圈窗口，环上有洞时洞那一槽里留着的上一圈的根只要不被抛弃、不低于 F 也走；代码 `crates/singlefs-checker/src/walk.rs` 10322 行一带）。F_生效 由 checker 自己算：取各块盘上最新的有效根所带 F 的最大值，再与池里自证过、fsid 与本池相同的系统配置槽带的 F 取大（`walk::check_pool_image` 里的 `effective_rollback_floor`，C556（checker 与层 0 不读系统配置里的 F） 已还清）——被抛弃时间线的根引用的单元由影子账隔离、F 之下的根引用的已释放单元已可再分配，都不在当前账里（里程碑「覆盖写、释放、回退与复用」步 4 / 步 5，`crates/singlefs-checker/src/walk.rs`）。⚠️ **最新根带的 F 低于 F_生效（两者不等）时这一格不判、报「不适用」（实二的读法，主 agent 2026-09-26 暂认，最终代码三方专攻这一格）**：先把新 F 写进系统配置之后、带新 F 的根落盘之前，或带新 F 的根全读不出，最新根的「已分配」是按它自己那个较低的 F 记的，还算着 F_生效 之下那些根引用的槽，与按 F_生效 取的候选集并集不可比，照判会在合法状态上红；另一种读法（并集按最新根自己的 F 取、别的几条按 F_生效 取）条款没选。坏镜像 `checker_known_bad_images.rs` 的 `every_root_carrying_the_raised_floor_unreadable_keeps_the_candidate_invariants_green_because_the_checker_reads_the_system_configuration_floor` 钉住这一格报不适用，`crates/mutations.tsv` 里把这道例外拿掉的那一行（实二）让它红。F_生效 之下仍在 defer 的槽：F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的账里罩住它的记录是已释放、释放代高于回收门槛的（还在 defer 里，分配器还不许回收；回收门槛 = max(F_生效, 环里最旧有效根)，D16（发布语义） 已定项 1「可再分配」那一行），并进遍历得到的和（收口表第 43 行取丁-defer，用户 2026-09-28 定，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M1；`walk::deferred_slots_referenced_only_below_the_floor_per_device`，用例 `raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs`）；⚠️ 它会不会盖住「该回收而没回收」那一类错，没拿变异试过。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 中央映射条目指的物理范围算进遍历得到的引用，与树里指向同一单元的引用同 (设备, 起点槽, 跨度) 时算一个（2026-09-27，`research/prompts/m2-rev-b1-implementer-report.md` 第六节第 2 条）。由记录施加出来的那一版照 D23（journal 的角色与格式） 已定项 15，走那条记录新根段里的实例表指针与分配记录树根（`walk::walk_version_applied_only_by_records`；用户 2026-10-02 定新根段回到三字段，实例表指针与分配记录树根随新根段换成那次发布的）。⚠️ **没量的前提**：「隔一条也是同一张实例表」是按「一个实例写行之后每一版照抄同一张实例表」推的，没量（`research/prompts/m2-rev-b2-implementer-report.md` 第七节第 2 条）；量它是 C699（取号跳过改认出生代号） 的第一步。 ⚠️ **钉最旧 txg 的代价（用户认的）**：读坏槽与读回全零的槽把回收门槛钉在它可能住过的最旧 txg（D16（发布语义） 已定项 1「根槽这一次读坏」），这样的合法盘面上 I-3.1（已分配统计对得上） 红在记账多的一边，到钉解开为止（这个进程写掉那一槽，或进程内记根时重读读出有效根、读出被抛弃的根，`crates/singlefs-core/src/allocator.rs` 的 `RootRingOccupant` 文档注释）；`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 三条慢档用例在第 10h 批改成只许红在记账多的一边（用户 2026-10-04 定钉最旧 txg、2026-10-05 定全零槽钉门槛而不并进上限，`records/2026-09-30-里程碑三开工第一轮.md` 第 10b、10c 批）。 |
| I-3.2 | 已停用·方向反了 | **此编号不再使用**——它写成「≤ 引用它的**最新**快照的代号」，方向反了：紧的约束是最**旧**的那个。旧写法被 I-3.5（引用区间的精确性） 的子判据 ② 严格蕴含，且检不出任何一类真实错误 | 不适用 |
| I-3.3 | 已停用·归属未定义 | **此编号不再使用**——「所属快照」未定义，而两种归属约定给出不同的比较符。由 I-3.6（deadlist 紧界） 取代 | 不适用 |
| I-3.5 | 引用区间的精确性 | **引用区间的精确性**。**内容**：对任一块 b 与任一根 R（快照根或活头根），**「b 从 R 可达」这个可达性谓词，必须等于记账所用的那个谓词**——两者是同一个定义，不是两份。**当前形态下该谓词的表达式**是 `birth(b) ≤ R.txg < death(b)`（[decisions.md](decisions.md) D5（快照 / 空间记账机制） 形态）；**换快照模型时表达式要重写，内容不变**。可判定形式（与 I-3.1（已分配统计对得上） 共用同一趟全盘遍历）：按 txg 升序枚举全部快照根与活头得 `Reach(b)`，则 ①② 合成一条**出现规则**：取回退候选集（I-7.4（近 K 代块未被复用） 用的那一份，加由记录施加出来、根槽从没落盘的那几版）按 txg 升序，同 txg 的几条并成一条（R⁻ 侧引用集合取并，R 侧逐条各判一次），相邻两条 (R⁻, R) 为一对，逐对判。R 引用而 R⁻ 不引用的每个落点 b（第一次出现与再次出现都判，`Reach(b)` 在 txg 序上是否连续不再单判）满足 (a) `birth(b) > R⁻.txg`；或 (b) b 落在 R 树表里某棵用户可见树（今天 extent 树、inode 树，`deleted_inodes` 树根非全零之后加它）的子树里，那棵树的根指针非全零、出生 txg 严格小于 R⁻.txg，且那棵子树里每个读得出的单元头的诞生代号都不大于那个出生 txg（读不出头的单元不计入）。`birth(b)` 取 b 的单元头里的诞生代号；b 自己的头读不出时这一个落点不判，读不出归 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配）。**缺口**：R.txg ≠ R⁻.txg + 1 时这一对之间有洞，分两种处置——最新根的实例表里有一行 (R⁻ 的实例, T = R⁻.txg)、R 的实例比 R⁻ 的大、两者之间每个实例的行都是 (j, 0, 0)，即实例表行证明 R 所在的实例直接接在 R⁻ 那一版上的，这一对照常判（gap-row）；不是的，用户可见树子树里的落点不判，别的落点（树表、记账树、分配记录树、中央映射树的单元与实例表单元）仍照 (a) 判 `birth(b) > R⁻.txg`。R 或 R⁻ 的引用集合数不全（走读读不出，或走到没有条款的形状）时整对报不适用；没有一对判到任何落点时整条报不适用并带理由（同 I-4.9（非最新候选根引用的单元不晚于它） 候选集只剩一条根的写法）；③ 若 b 不被活头引用，则 b **恰好**出现在一个 deadlist 中，且该 deadlist 的归属区间 = `(Reach(b) 中最晚者.txg, 其下一个根.txg]`；④ 若 b 被活头引用，则 b 不出现在任何 deadlist 中。⚠️ **射程**：缺口的来源是新实例第一条根的 txg 取根环与 journal 里见过的最大 txg 加一（`crates/singlefs-core/src/mount.rs` 取号那一步），所以写行那次发布的记录已落而根槽没落、多记录发布只落一部分之后再可写挂载，必出现缺口；另有恢复丢掉尾巴记录、根槽写失败重发跳一格（今天的代码走不到，随失败处置在后面的里程碑做）、根槽读不出、根槽被环盖掉而记录也被盖掉（短 journal 环）。**盲区**：缺口右边那一对里，落在用户可见树子树里的新写单元（新叶）的诞生代号被改小到 R⁻.txg 以下，判不出；整条用户可见链（树根指针出生 txg 与树里单元的诞生代号）一起改小到 R⁻.txg 以下的坏镜像同样判不出：盘上与一次合法回退逐项同形，判别子是回退那次发布写的谱系记录（D18（块里携带什么信息） 已定项 24），`crates/` 今天没有。**谱系记录落地之后**，(b) 收成「R 的谱系树里有一条记录 (T_rb, T_old)，R⁻.txg < T_rb ≤ R.txg，且那棵用户可见树的根指针出生 txg ≤ T_old；谱系树读不全时该对不适用」，这一形与缺口条款并存（`research/prompts/m3-birth-floor-r2-main-verification.md` 第二节第 9 行）。**判别力**：诞生代号改小到恰好等于 R⁻.txg 的新写单元必须红；最新根的 inode 树指回已丢的旧叶、分配记录与映射条目一起改平必须红；回退根槽被环盖掉、记录也盖掉之后的合法镜像不许红。**推翻条件**：任一合法镜像（池级 checker 与层 0 其余判定都不红）上红。 | 部分实现：出现规则（TF、β、gap、gap-row、N3）已实现，池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_the_appearance_rule_of_adjacent_candidate_versions`（`IMPLEMENTED_INVARIANTS` 下标 65；坏镜像在 `crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs`；变异在 `crates/mutations.tsv` 里「第8批 I-3.5」那几行）；③ ④（deadlist 那两条）未实现（实现进下一批）。出现规则取按树继承下界加 β 出现规则（用户 2026-10-02 定 ① 取 β）加缺口处置；缺口处置用户 2026-10-03 定 gap 加 N3（判决里的局部标号，只放过用户可见树里的单元）加 gap-row 两样都加。**被攻过几轮**：按树继承下界加 β 加整对不适用的缺口，在合法镜像上两轮没红（各一次抽样）；gap 加 N3 加 gap-row 的合成形、同 txg 几条并成一条、`deleted_inodes` 非全零才算继承、读不出头的单元不计入这三处措辞、谱系记录落地之后的区间形，都是零轮，终轮不再开轮，实现时再攻。实现挂在池级 checker 的主走读上（每版落点集合快照、用户可见树标签、压得住的判、新判定函数、位置键折叠、`IMPLEMENTED_INVARIANTS` 加一条、用例与变异），I-3.1（已分配统计对得上） 在短环上洞里的误红归下一批调查，与这一条共用同一个「候选集有洞」判法（`research/prompts/m3-birth-floor-r3-main-verification.md` 第二节第 4、5 行与第四节）。② 的上界半（`birth(b) ≤ R_first.txg`）另立 I-4.9（非最新候选根引用的单元不晚于它）（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行）。⚠️ **射程注（2026-09-13）**：跨头共享的块（稀疏旁表计数 > 1，D5（快照 / 空间记账机制） 已定项 6）不由 deadlist 单独管，判据 ③ / ④ 对它们不适用；它们的生死由旁表计数与各头的可达性合起来判，那条不变量还没立。 |
| I-3.6 | deadlist 紧界 | 任一 deadlist(S) 中的块满足 `birth ≤ prev(S).txg`（约定 B 下的紧界）。判别力：抓「kill 时该走直接释放却进了 deadlist」这一类（泄漏方向） | 未实现 |
| I-3.4 | 可用空间扣待删占用 | 待删中 inode 没释放的单元计在记账「已分配」里（释放只进 defer），记账标签 4（待删除但意图未完成占用）恒 0，可用空间统计因此已扣除待删占用；准入式子另扣环窗口常驻项（D28（挂载期承诺量） 已定项 1）。来源 `research/prompts/m3-delete-r3-main-verification.md` 第三节判标签 4 那一格，被攻过零轮 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，函数 `judge_pending_deletion_occupancy_of_a_version`；坏镜像两份：标签 4 那一行的值非 0、整行缺失，登记在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `.chain`，改法在 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`；GPU 判器下标 70，扫描内核判）。池级 checker 只判「标签 4 恒 0」：判的每一版里，走全了的记账树都要有标签 4 那一行且值为 0，缺行判红；另两句不在这一行判：计在「已分配」里归 I-3.1（已分配统计对得上）、I-3.11（已分配减 defer 等于最新根走读），可用空间已扣待删占用归 I-5.2（空闲统计对得上），即空闲加已分配等于容量；判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法；出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节） |
| I-3.7 | 活快照可枚举 | 盘上存在一个可枚举的持久结构，其元素恰为当前全部活快照（既不多也不少） | 未实现 |
| I-3.8 | 实例表行唯一且低于挂载根 | 实例表 kind 0 行按实例代号唯一；行只在回收条件成立后删（整轮清扫对池中每一个落点都得出判定、其中没有该实例的未发布单元、那次清扫里没有读失败 ∧ 全部设备在线 ∧ 根环里没有该实例发布的根、且根环每个槽都读成功）；行的实例代号 < 挂载根实例（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2） | 已实现（2026-09-16，`crates/singlefs-checker/src/walk.rs` 的 `judge_instance_table_rows`：行按实例代号唯一、每行实例代号低于最新根的实例（记录形态归 I-3.14（实例表每片记录形态合法））；最新根的实例表第 0 片解不开时报不适用，理由点名这一片解不开、行没收全；回收那一半没有输入，第一版行只增不删，删行的条件没人判） |
| I-3.9 | 释放代落在停止引用它的那一格区间里 | 任一带已释放标志的分配记录，它的释放代必须落在区间 `(最后一条仍引用这个落点的有效根的 checkpoint_txg, 最早不再引用它的有效根的 checkpoint_txg]` 内；根环没有洞时这个区间只有一个值，等同于「等于最早不再引用它的那条根的 txg」。若每一条有效根都还引用它，它不许带已释放标志（D3（空间分配） 已定项 7（释放时条目不删、改写 value：跨度段最高位置 1、后 8 字节从分配代换成释放代）；C374（释放代与树表诞生 txg 只有验收断言盯着））。环上有洞时（被施加的 journal 记录做的释放写着一个根槽从没写过的 txg，实测 2026-09-18 残留记录那条层 0 流的 12 个状态）按区间判，按「等于」判会在合法镜像上判红；已经回收、还没被复用的已释放记录跳过不判（见证它释放的根已不在候选集里，定位不了那个区间），全是这一类时整条报不适用并带理由。判别力：把发布 B 那次的释放代从 4 写成 3 必须红（最早不再引用那十个落点的有效根是 B、txg 4）。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_release_generation_and_tree_table_birth`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`；层 0 每个崩溃状态都判）。**判法收窄在实现途中**：今天任一候选根走不全就整条报不适用；收窄成只在比最后引用它的那条根晚的候选根都走得全时才判这一条已释放记录，攻方造出的对照格镜像（抬 F 的空发布之后再覆盖写，把最后一次空发布那条根的树表翻坏）钉进用例（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 10 行） |
| I-3.10 | 已分配记录的分配代等于它罩住的单元的诞生代号 | 任一**未带已释放标志**的分配记录，它的分配代（D3（空间分配） 已定项 3：`value = 分配代`；盘上编码见 D3（空间分配） 已定项 7）等于它罩住的那个单元头里的**诞生代号**（D18（块里携带什么信息） 已定项 7 的类身份段）——记录与单元是同一次发布写出来的，两个数因此同源。⚠️ **射程**：① 只管未释放的记录，已释放的那一半归 I-3.9（释放代落在停止引用它的那一格区间里）；② 记录罩住多个槽时（数据单元跨两槽）取起点槽那个单元头；③ **那个槽上读不出可用的单元头时这一条没有对象**（报不适用，不报违例）——读不出本身由 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配） 管；④ **只读回退候选集里那几版的分配记录**（候选根的分配记录树、树表 0 条那一版根记录直接持有的那一片、由记录施加出来的那几版），**再加下一次挂载会先施加的那一版**：下一次挂载的恢复照 D23（journal 的角色与格式） 已定项 14 的重放规则施加的**第一次**发布（与 I-4.1（重放后树可遍历） 同一份重放，所选根取最新根），取它末条那一份新根段里树表指着的分配记录树；一次都不施加、锚点认不定、所选根的 txg 没有后继时没有这一版，前缀停在后面某次发布的点名单元上时仍取第一次（`walk::version_the_next_mount_applies_first` 与 `walk::replay_the_next_mount_would_do`；认法依据 `research/prompts/m3-formal-code-b36-r1-main-verification.md` 第二节 P1 与第三节第 1 条，取末条那一份依据 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P1；主 agent 2026-09-24 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 2 条定接这一步，被攻过零轮；只接这一步——恢复连着施加几次发布时第二次及以后那几版读不到，欠在欠账表）；不读被实例表判抛弃的根与低于 F 的根：低于 F 的根那棵账里未释放的记录罩住的槽，在 F 抬过之后可以被合法回收、换成诞生代号不同的单元，照「任一」去判会在合法镜像上红（主 agent 2026-09-23 定，被攻过零轮）。⚠️ 「再加下一次挂载会先施加的那一版」那一段的同源前提，靠 D16（发布语义） 已定项 7 持久顺序里单元与记录之间那道屏障撑着：拿掉那道屏障、又复用回收槽时，记录已持久而复用槽上仍是旧单元，恢复按点名验证拒施加、结局正确，这一条却在那个槽上红（2026-09-28 主 agent 复跑 floorreuse4：删屏障 271 个状态红 48、带屏障 73 个状态 0，`research/prompts/k3-9-barrier-r1-main-verification.md` 第一节）；用户 2026-09-28 定屏障维持现状、checker 不改，欠在 C592（单独拿掉单元写与记录间那道屏障今天没判据会红）。**它拦的是什么**：复用一个已回收的落点时改写了那条分配记录、却把上一次的代留着（2026-09-22 增补 3 第 1 件的代码三方第一轮攻方腿造出这条变异，快档与大档都判不出，因为全仓没有一条不变量管已分配记录的分配代） | 已实现（2026-09-23，待代码三方与崩溃验证；池级 checker `walk::judge_allocation_generations_against_unit_births`，读法照射程 ④，同一条记录（盘、槽、代）只判一次，射程 ③ 按记录逐条跳过、一条都没判到才整条报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 下一次挂载会先施加的那一版由 `walk::tree_table_pointer_of_the_version_the_next_mount_applies_first` 并进读集；它的坏镜像（发布 B 的记录已落、根槽没落，B 那棵分配记录树里一条未释放记录的分配代改成诞生代号 + 1）是 `checker_known_bad_images.rs` 的 `an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount` |
| I-3.11 | 已分配减 defer 等于最新根走读 | 逐盘：被判镜像上记账树里「已分配」那一行（记账第 1 项）减「defer 待释放」那一行（第 5 项），等于从最新有效根走读到的、这块盘上被引用的槽数。统计量的标签与行形态是 D5（快照 / 空间记账机制） 已定项 4 与已定项 10 的那几行，读的是镜像里的记账行、不读内存里的分配器；走读与 I-3.1（已分配统计对得上） 同一个走法，只取最新有效根。⚠️ 它补的是 I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 都够不着的那一种：写者把活单元记成已释放、放进 defer，已分配与空闲两个和照样对得上（alloc-basis 第三轮判决 `research/prompts/alloc-basis-r3-main-verification.md` 的 Z10）。⚠️ **判别力自证在造出来的基底上做**：E156（alloc-basis 四条岔路的代价数） 第 4 次重跑在造出来的基底 β_syn（第 5 项为 0）上再记 1 槽 defer，I-3.11（已分配减 defer 等于最新根走读） 判红、去掉「减 defer」判绿（`research/results/e156-alloc-basis-counts-2026-09-26-r2.out` 的 `E7RESULT name=q7c1_not_subtracting_defer basis=beta_syn k=1 real_check_is_red=true without_subtracting_defer_is_red=false flips_red_to_green=true`）。可达状态里第 5 项能不能为 0 没量到：挂载时回退（回退写行那次发布）今天没有了，同一份产物里挂着回退那一版第 5 项是 356 槽（`E7RESULT name=u13a_rollback_to_oldest_candidate overwrites=72 target_txg=52 txg_before=75 txg_after=76 instance_before=1 instance_after=1 instance_table_rows_before=0 instance_table_rows_after=0 item1_slots=374 item5_slots=356 i311=holds structure_holds=true`）。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 | 已实现（池级 checker `walk::check_pool_image`：逐盘判「defer 待释放 + 最新根走读 == 已分配」，最新根走读取 `walk_root(最新根)` 刚走完那一刻、与 I-3.1（已分配统计对得上） 同一个走法；最新根下面没有记账树时报不适用。坏镜像 `live_unit_recorded_as_deferred_reddens_only_the_allocated_minus_deferred_invariant`（盘 0 defer 行 +1 / −1 槽，只红它，I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 照旧真被评估过且成立）；判别力自证在造出来的基底上 `on_a_synthetic_base_with_an_empty_defer_queue_one_deferred_slot_reddens_the_allocated_minus_deferred_invariant`（两盘 defer 挪回 0 之后再记一槽）；`crates/mutations.tsv` 5 条变异（判定恒真、不减第 5 项、`==` 放宽成 `≥` / `≤`、拿全部候选版本那一份走读比）；用户 2026-09-23 定采纳：alloc-basis 岔路单第 12 行（岔路 7）） |
| I-3.12 | 实例表行 T 低于引用它的根的 txg | 根环里任一自证过的根 R，它指着的实例表的每一行 (i, T, W) 满足 T < R 的 `checkpoint_txg`；链上某一片读不出的根不判。依据 D18（块里携带什么信息） 已定项 11（恢复写的行，所选根那个实例的 T 是重放之后那个根的 txg，中间实例写 0）与 D23（journal 的角色与格式） 已定项 14（新实例第一次发布的 `checkpoint_txg` 大于环里全部根与记录的号）。射程是每条根各自指着的那张表，不是只有挂载根那一张（那一张归 I-3.8（实例表行唯一且低于挂载根））；同一实例的行在两张表里是否逐字节相同不判，条款没有这一句。 | 已实现（2026-10-02，池级 checker `walk::judge_instance_table_rows_against_the_roots_referencing_them`：每条自证过的根各沿自己的实例表链判，链上有一片读不出的根跳过，一行都没判到时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 6 行） |
| I-3.13 | 记账行代段等于这一版的 txg | 走读到的任一版（候选根，以及由记录施加出来的那几版）的记账树里，每一行的代段（记账 key 偏移 14，8 字节）等于这一版的 `checkpoint_txg`，只判走全了的树。依据 D5（快照 / 空间记账机制） 与 D16（发布语义） 已定项 6 的「记账代 = checkpoint 号」（每条根的记账树只装这条根那一代的行）。 | 已实现（2026-10-02，池级 checker 走读里的 `walk_tree_table_entry` 对每一版的记账树判，版本含由记录施加出来的，只判走全了的树；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 5 行） |
| I-3.14 | 实例表每片记录形态合法 | 走读到的实例表每一片里：每条记录的 `kind` 只有 0（行）或 1（链指针记录）；链指针记录恰好是这一片的最后一条、片中间不出现，且自身合法（有无下一片 0 或 1、无下一片时位置指针全零、有下一片时不全零，且位置指针头部 MAC / nonce、算法类型、压缩算法码与压后长度、extent 偏移都是第一版的取值：有一段不是第一版的取值，这一片也解不开）；`kind` 0 行的 flags 字节为 0。依据 D18（块里携带什么信息） 已定项 11（`kind` 不是 0 或 1 ⇒ 这一片解不开；flags「第一版恒 0、非 0 拒收」；链指针记录「恒为一片的最后一条」，最后一条不是合法的链指针记录同样解不开，位置指针头部的第一版取值见 D19（块指针的结构与宽度预算） 已定项 3），用户 2026-10-06 定另立；头部那一形用户 2026-10-06 定算第五种形态（岔路单 `research/prompts/m3-batch10o-code-r1-forks.md` K2-1 那一行） | 已实现（2026-10-06，`crates/singlefs-checker/src/walk.rs` 的 `judge_instance_table_page_record_forms` 判前三件、`walk_instance_table_chain` 里链指针记录那一格判最后一件，走到的每一片都判；不成立的一片解不开：行不收、链不往下走、候选集滤不全（I-9.29 的门关）；只读不判的读法与引用集合那一遍同一个读法；GPU 走读内核同判，扫描内核的引用集合那一段与只读读法、恢复内核读实例表同判 flags；有下一片而位置指针头部有一段不是第一版取值的那一形：CPU 三处读法（主走读、只读不判的读法、引用集合那一遍）经 `crates/singlefs-checker/src/walk.rs` 的 `chain_record_view` 判，GPU 走读与扫描经 `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl` 的 `w_chain_record_kind` 判，恢复内核读实例表同判；坏镜像两张（位置指针 MAC 不全零、extent 偏移非 0），在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`） |

> ⚠️ **I-3.5（引用区间的精确性）四条子判据双侧夹死 与 I-3.1（已分配统计对得上） 正交，两条都必须有。** I-3.1（已分配统计对得上） 是标量总账（已分配 == 遍历和），
> 而**归属错误对它不可见**——块被挂到错的 deadlist 上时，它仍然被恰好一处引用，总账依然平。
> 要检出归属错，必须有**按快照分箱的向量等式**。
> 这与 [decisions.md](decisions.md) D14（双轨（大小文件 / 持久临时）） 已定项 6 推导 D-3（多轨兼容可判据） 里「标量等式 → 按轨道分箱的向量等式，
> 判别力净增」是同一个形状。
>
> ⚠️ **数据丢失方向只在空间被重新分配之前可检出。** 该进 deadlist 却被释放的块，
> 一旦其空间被复用，症状就退化成 I-2.1（校验和与内容匹配） 校验和不匹配，即「损坏，原因不明」。
> **所以这类检查必须在崩溃点重放 harness 里紧跟着删除动作跑，不能只在全量里跑。**
>
> ⚠️ I-3.1（已分配统计对得上） 是**唯一允许全盘遍历的记账检查**——它是 checker，不是运行时路径。
> 运行时记账必须增量维护（D5（快照 / 空间记账机制）），checker 才用遍历来验证增量维护有没有出错。
> 这条对照关系是本工程验证记账正确性的核心手段。

```

**出处 `.claude/kb/invariants.md:172-190`（整段抄，未转述）**

```markdown
## I-4 事务与崩溃一致性 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-4.1 | 重放后树可遍历 | 任一崩溃点重放后，所选根（重放之后那一版）指向的树全部可完整遍历。起点含介质损坏的镜像（前缀停在一条点名单元两份没都验过的记录上）报不适用。另三处报不适用：所选根那次发布读得出的记录里带末条标志的多于一条（锚点认哪一条条款没写，实现在任何写之前拒挂，重放不出一版）；所选根的 checkpoint_txg 是 u64 的最大值（接在它后面的发布取不到 txg）；所选根之上一次发布都不施加（重放之后那一版就是所选根，可不可遍历归 I-7.2（最新根可完整遍历））——没有记录压在最新根之上的镜像上这一条因此恒报不适用、不报成立。重放之后那一版的树表指针全零时红：core 的根记录读者不把全零当特例，照它那两条位置条目读盘 0 槽 0、读不出（两种挂载都拒），另起的走读跟过去、记一次走读失败（`walk.rs` 的 `Walk::walk_version_applied_only_by_records` 里的 `follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core`；用户 2026-10-06 弹窗定，选项原文「甲 两类都判损坏」，`records/2026-10-01-里程碑三形式化接入.md`「用户 2026-10-06 弹窗定 b34 四问」那一行）；重放施加了几次发布时判的是最后那一版，前面某次发布全零而最后一版完好时不红，core 也照常挂上。重放用的读环按 (实例代号, 计数器) 建键，两块盘同一计数器坐着不同实例的记录时两条都留，与实现恢复同键（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 2、8 行）。 | 已实现（2026-10-02，池级 checker `walk::judge_trees_of_the_replayed_version_are_traversable`：R-独，checker 自己照 D23（journal 的角色与格式） 已定项 14 的前缀规则复刻只读重放（`replay_the_next_mount_would_do`），施加出来的那一版（所选根换上末条记录新根段的七项：树表指针、中央映射树根指针、树 ID 水位、F、实例表指针、分配记录树根、卸载记号，只有 fsid 照所选根；D23（journal 的角色与格式） 已定项 15）另起一份走读，走读断了判红；在飞上限按环长 ÷ 4096 ÷ `JOURNAL_SAFETY_FACTOR` 现算。读环键 2026-10-03 改成 (实例代号, 计数器)（`records_the_replay_reads`），反向链那一关改按 (实例代号, 计数器 − 1) 找前一条（`record_back_chain_lets_it_into_the_replay_prefix`），消掉两块盘同一计数器坐着不同实例的记录时与实现分开的形态（推为第一版两盘池上不可达）；`records_as_the_recovery_reads_them` 与 `record_can_enter_a_replay_prefix` 两处同改按 (实例代号, 计数器) 建键，已做（主 agent 2026-10-03 定）。坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（重放六条规则各一份、两块盘同计数器不同实例两份）；层 0 里与 core 的 `effective_root` 逐状态差分今天没有能跑的入口，要先开口子，进下一批清单（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 8、9 行）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 2 行与第四节用户定案） |
| I-4.2 | 无被引用未提交块 | 任一崩溃点重放后，不存在被引用但未提交的块 | 已实现（2026-09-22，池级 checker `walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`：走读到的每个被引用单元，头里的诞生代号与写序（D18（块里携带什么信息） 已定项 7 单元头字段表里的字段）按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的已发布谓词判成已发布——**「未提交」取的就是「按那个谓词判不成已发布」**，谓词与 I-1.8（归并后版本全序） 扫描方向共用同一份 `PublishedPredicate`。打包记录类型 4 的实例表单元按 I-1.2（块头写序已发布） 那一行的例外不走谓词，不进这一条。坏镜像是把数据单元头里的诞生代号与引用它的 extent 记录里那条数据指针的诞生代号一起抬到最新根的 txg 之上——指针仍与头一致 ⇒ I-1.2（块头写序已发布） 不跟着红，分得开两条的就是这一份（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）；层 0 每个崩溃状态都跑它。⚠️ **「任一崩溃点重放后」那半句没实现**：池级 checker 判的是手上这一份镜像、不跑恢复，判的是「被引用 ⇒ 已发布」这个态上的性质；重放之后再判一遍归层 0） |
| I-4.3 | 提交原子 | 提交是原子的：不存在"部分生效"的事务 | 未实现（它要判的是 D23（journal 的角色与格式） 已定项 7 那两个事务边界字段——事务号与提交标记——施加之后的效果。**对着一份镜像判不出来，没硬写**：「部分生效」要拿「这次事务该有的全部效果」当 oracle 才分得出来，而一份镜像上任何状态都自洽——同一份字节既可能是「事务整体没生效的旧态」，也可能是「生效了一半」，池级 checker 手上没有第二个输入。判它的是层 0 崩溃点重放加 oracle：`.claude/kb/milestone/02-second-txn.md` 第 486 / 487 行那两条（崩在第二条记录之前 ⇒ 整个事务不施加、文件是旧长度；把提交标记提前到第一条记录 ⇒ 恢复施加了半个事务、I-4.3（提交原子） 红）说的就是这条路。形态同 C116（叶容器的分裂 / 合并纪律没有会失败的检查）：操作断言，不进池级 checker，恒真的检查比没有检查更糟） |
| I-4.4 | 意图只描述目标态 | 任一未完成的意图 key，其字段只描述目标状态，不含任何增量字段（"再删 N 个"一类） | 已实现（2026-10-05，在清单里的第一版恒报不适用：池级 checker 在 `walk::check_pool_image` 里对它调 `not_applicable`，理由逐字是 `crates/singlefs-checker/src/image.rs` 的 `INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION`；不要求坏镜像，`crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `INVARIANTS_WITHOUT_A_BAD_IMAGE` 点名；GPU 判器下标 73，内核不置位）。第一版不适用：意图载体（nlink = 0 的记录、`deleted_inodes` 的 key、墓碑）没有可装增量的字段，checker 报不适用并写理由；logged_ops 有字段表时实现（用户 2026-10-05 定，岔路单 `research/prompts/m3-delete-checker-r1-forks.md`） |
| I-4.5 | 已停用·需两镜像 | **此编号不再使用**——幂等要比对两个镜像，单镜像判不了；判据落在 [experiments.md](experiments.md) E3（意图日志的断电语义） 判据③ | 不适用 |
| I-4.8 | 近 K 代根校验和自洽 | 任一崩溃点重放后，从回退候选集（D23（journal 的角色与格式） 已定项 14：按实例表判仍然有效 ∧ txg ≥ F_生效）里**任一**根出发遍历，所有块的校验和均与其父指针记录的一致（候选集的深度由 D16（发布语义） 已定项 1 的回退下界 F 定：平时整个根环，盘紧时缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内）。**判别力**：抓「本事务内释放的块被重新分配并写入」这一类——那会让旧根指向的块被写入新内容，**使旧根成为假的回退候选**，而只验最新那一代根是发现不了的 | 已实现（2026-09-17，池级 checker `walk::check_pool_image`，最新根与候选集里每条更早的根各判一格：从它出发的遍历有单元对不上或读不出就红；坏镜像是 mkfs 树表单元抹成零；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-4.6 | 中间态其余不变量成立 | 存在未完成意图的镜像上，其余全部不变量仍然成立（分批之间的中间态是合法状态）。「其余全部不变量仍然成立」由各条在每个被判的版本上各自判，中间态的版本也在内；I-4.6（中间态其余不变量成立） 自己判的是中间态特有的一格：每条 nlink = 0 的记录，数据单元数与 size 对得上——size 为 0 时 0 或 1 个单元，size > 0 时 ⌈size ÷ 数据单元净荷容量⌉（实现 `crates/singlefs-core/src/write_request_split.rs` 第 61 行 `div_ceil(...).max(1)`）。判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，函数 `judge_data_unit_counts_of_the_inodes_being_deleted_against_their_size`；净荷容量 checker 自己算一份（`data_unit_payload_capacity_in_bytes`）；每条 nlink = 0 的记录判一次，一条都没有时报不适用并带理由；坏镜像：第一个文件的 size 加一个净荷（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `raise_the_size_of_the_first_file_by_one_data_unit_payload`）；GPU 判器下标 72，扫描内核判；用户 2026-10-05 弹窗定「只判单元数对 size」，出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第四节与岔路单 `research/prompts/m3-delete-checker-r1-forks.md`；射程只对 nlink = 0 的记录，每条记录都判的那一形不挂在这一行名下，另记欠账 C711（单元数对 size 只判了 nlink = 0 的记录）） |
| I-4.7 | 删除中 inode 有账 | 任一处于删除中的 inode，要么在删除中队列里有条目，要么其 extent 已全部释放——两者皆非的中间态不存在。「处于删除中的 inode」指两样：inode 树里 nlink = 0 的记录；extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 的记录的号。判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，函数 `judge_inodes_being_deleted_against_the_queue_and_their_data_units`；每个处于删除中的号判一次，一个都没有时报不适用并带理由；坏镜像：extent 树里有数据单元、inode 树里没有它的记录、号也不在队列里（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `drop_the_record_of_the_first_file_and_keep_its_data_units`）；GPU 判器下标 71，扫描内核判；对象取这两样，出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节逐格判决表里 I-4.7（删除中 inode 有账） 对象那一格） |
| I-4.9 | 非最新候选根引用的单元不晚于它 | 回退候选集里不是最新的每一条根 R，它引用的单元（按落点认，读落点上单元头里的诞生代号，头校验和过）的诞生代号 ≤ R 的 `checkpoint_txg`。最新那条根归 I-4.2（无被引用未提交块），不在这一条里。依据 D16（发布语义） 已定项 7（单元先于根落盘）。 | 已实现（2026-10-02，池级 checker `walk::judge_unit_birth_against_an_older_candidate_root` 判走读里每个被引用的单元，共享子树里别的版本先走过的单元由 `merge_placements_below_a_unit_walked_by_an_earlier_version` 补判；候选集只剩一条根时报不适用，同 I-3.9（释放代落在停止引用它的那一格区间里）；坏镜像沿 I-4.2（无被引用未提交块） 那一份的造法，映射 key 也要跟着改，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行） |

> ⚠️ **编号只往上加，不回收、不重编。** 回收一个编号会让别处的旧引用悄悄指向另一条陈述，
> 而检索只端出其中一条，不会告诉你它换了意思（`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条「矛盾比空白更糟」）。
> 退役的编号留一行说明去向，比留一个无解释的空档安全。

```

**出处 `.claude/kb/invariants.md:191-213`（整段抄，未转述）**

```markdown
## I-5 分配器 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-5.1 | 物理范围不重叠 | 不存在被两个不同引用同时占用的物理范围（除非是显式的多副本） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 引用按版判（2026-09-27，`research/prompts/m2-rev-b1-implementer-report.md` 第六节第 1 条、`research/prompts/m2-rev-b2-implementer-report.md` 第八节）：同一版（根环里的一条根，或由记录施加出来的一版）里同一个 (设备, 槽) 被两条指针引用判红，(设备, 槽) 取自位置条目的设备 ID 与物理偏移（D19（块指针的结构与宽度预算） 已定项 4），一条指针自己的两条位置条目同落点算一个引用（那一格归 I-2.5（位置条目按设备身份升序））（`walk::Walk::note_references_of_a_pointer_followed_by_the_walk`）；中央映射条目指的物理范围算进引用集合；别的版本走过的共享子树不重走，按缓存把它下面的落点并进这一版再比、不读盘（`walk::Walk::merge_placements_below_a_unit_walked_by_an_earlier_version`）。坏镜像在 `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs`。 |
| I-5.2 | 空闲统计对得上 | 空闲空间统计 == 总空间 − 已分配空间 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-5.3 | 报出的空闲都兑现得了 | `df` 报给用户的空闲 ≤ 当下可分配 + 能在有界步数内放回的扣住量（defer 窗口里、回退下界以下的释放）。运行时的等价形态：`df` 报出 s 字节空闲时写 s 字节必须成功；删掉 s 字节之后同样大小的写在有界步数内成功（[decisions.md](decisions.md) D3（空间分配） 已定项 9，用户定案）。界随 D16（发布语义） 已定项 1 的形态定。判别力：把「放回扣住量」那一步摘掉，近满盘上删一个对象再写同样大小必须由绿转红 | 未实现 ⚠️ **口径注（2026-09-13，D28（挂载期承诺量） 已定项 4）**：`df` 要扣的 checkpoint 保留池是每次发布现算的量，发布即重算，近满盘上报出的空闲会随树高变小。 |
| I-5.4 | 分配记录罩住的槽互不相交 | 判的版本是回退候选集里每条根，加由记录施加出来、根槽从没落盘的那几版（已被一次恢复施加过、实例表里有行，树表指针与「树表 0 条那一版的分配记录树根」取自记录的新根段；认法 `walk::versions_applied_only_by_records`，另有三条都要成立：txg 不低于回退下界、它换下的单元还没到回收的时候（根环里有一条没被实例表判抛弃的根，txg 比它小）、记录进得了重放前缀；一次发布切成几条记录时新根段取带末条标志那条记录的，读不到带末条标志的那一条时退回先到的那一条）与下一次挂载先施加的那一版（下一次挂载的恢复照 D23（journal 的角色与格式） 已定项 14 的重放规则施加的**第一次**发布，取它末条那一份新根段，与 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 认同一个函数 `walk::version_the_next_mount_applies_first`；一次都不施加、锚点认不定、所选根的 txg 没有后继时没有这一版，前缀停在后面某次发布的点名单元上时仍取第一次；根环里没有它的根）；这些版本的分配记录树里，同一块盘上任意两条分配记录（不论已分配、已释放、释放代是否已低于回退下界）罩住的槽区间 [槽号, 槽号 + 跨度) 互不相交。复用一个已回收的落点时，新记录罩住的槽上别的已回收记录要随这次分配一起删掉；留着它，下一次可写挂载重建分配器时两条记录罩住同一个槽。同一条下另判一格：树表非空（带树）的那一版，根记录里「树表 0 条那一版的分配记录树根」那 86 字节必须全零（D22（单元原子性怎么合成） 已定项 7 字段表「分配记录树根指针」那一行），非零就是这一版有两棵分配记录树，判违例。这一格成立的那一次，只在这一版自己的分配记录树（树表种类 3 那一条指着的那一棵）走到叶时计进 I-5.4 的评估，没走到叶的版本不计。部分没判（逐版列出，不判红；判出违例照报违例）：树表指针非零而树表读不出的版本、树表指针全零的版本（core 的根记录读者照它读盘 0 槽 0、读不出），都认不出带不带树，根记录里分配记录树根全零那一格与它自己的分配记录树都没判；树表读得出、0 条而分配记录树由根记录直接持有（C512（树表 0 条的一版上被换下的实例表记在哪没有条款））、那棵树读不出的版本；带树一版自己的分配记录树没走到叶的版本。这两类由记录造出来的版本里树表指针全零的，这一条自己报部分没判（与根环里的候选根同一处置）；它们另外判损坏：由记录施加出来的那几版与根环里的根同一读法，主走读照 core 的根记录读者跟过去、记一次走读失败（用户 2026-10-06 弹窗定，选项原文「甲 两类都判损坏」，`records/2026-10-01-里程碑三形式化接入.md`「用户 2026-10-06 弹窗定 b34 四问」那一行）；下一次挂载先施加的那一版全零，在重放只施加这一次发布时就是重放之后那一版，红 I-4.1（重放后树可遍历）。候选集里没有一棵分配记录树走得到叶（第 0 代树表是空的，或树表 / 分配记录树读不出、位置对不上）报不适用。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_allocation_records_disjoint`，吃一张版本表（`VersionToJudgeTheAllocationRecordsOf`：称呼、树表指针、树表 0 条那一版的分配记录树根），表里是回退候选集里每条根、由记录施加出来的那几版与下一次挂载先施加的那一版（`check_pool_image` 拼表）；GPU 判器扫描内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `judge_allocation_records_disjoint_of_a_version`，候选根、由记录施加出来的版本、下一次挂载先施加的那一版各调一次，只有违例位、部分没判不置位；这两类版本上的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `two_allocation_records_covering_one_slot_in_a_version_applied_only_by_its_journal_record_redden_only_the_disjointness_invariant` 与 `two_allocation_records_covering_one_slot_in_the_version_the_next_mount_applies_first_redden_only_the_disjointness_invariant`，读不出的几格各有用例（`the_tree_table_of_a_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged`、`the_allocation_record_tree_held_by_the_root_record_of_a_version_without_trees_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged`、`an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged` 等）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `an_allocation_record_whose_span_covers_the_next_record_reddens_only_the_disjointness_invariant`；层 0 每个崩溃状态都判，同日在新代码上全量：第二条流 2,104,413 个状态里评估 1,842,252 个、违例 0，其余 262,161 个报不适用，第一条流 262,165 个状态里评估 4 个、违例 0（`m2-wave2-crash-verifier-report.md`「主 agent 要的两个数」）。同日立，依据 `research/prompts/m2-wave2-code-r1-main-verification.md` 第二节第 1 行：复用时跨度变了留下重叠记录，下一次可写挂载在重建分配器时 panic，而 I-5.1（物理范围不重叠） 只管树里的引用、判不出记录之间的重叠；带树一版分配记录树根全零那一格的挂法见判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-5.4 挂法」那一行与 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2「成立要至少一棵树走到叶」那一行，树表读不出与根记录直接持有的树读不出两形见 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2「I-5.4 按版本：坏盘」那一行与 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2 第一行，射程含两类由记录造出来的版本见同一节 P2「由记录施加出来的版本与下一次挂载先施加的那一版」那一行，树表指针全零的版本见同一节 P2「树表指针全零的根」那一行与第五节第 1 条；下一次挂载先施加的那一版改取重放施加的第一次发布见 `research/prompts/m3-formal-code-b36-r1-main-verification.md` 第二节 P1 与第三节第 1 条，由记录施加出来的那一版取带末条标志那条记录的新根段见 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P1-b，两类由记录造出来的版本树表指针全零判损坏见 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P2 与第五节第 1 条） |
| I-5.5 | 从 mkfs 水位建树那一版的 t1 落点 | 走读到的任一版 V（候选集里每条根，含由记录施加出来的那几版）若是从 mkfs 水位第一次建树的那一版——① V 的树表非空，且种类 1（extent 树）条目的树 ID 等于 65536；② V 的树表里每条条目的诞生 txg（条目偏移 108）都等于 V 的 `checkpoint_txg`——则 V 引用的数据单元里，每块盘上落点槽号最小的那一个，槽号等于这一版分配记录里 D3（空间分配） 已定项 8 第 1 条那条政策算出的那一对的起点槽号（起点最小、32768 对齐、两槽都空、不在开放段里）。新池新建文件那一次是 U + 4，U 是所选系统配置偏移 417 的单元区起始槽号（默认 768 MiB 环 U = 50176，所以是 50180）；回收之后（例如 mkfs 的实例表 U、U + 1 那两槽被回收）不再是 U + 4。V 没有数据单元时不判；不满足 ①② 的版本不判。这一类含新池新建文件，也含 mkfs 之后重挂、实例 1、2、3 从 mkfs 水位写的第一个文件（用户 2026-10-04 定案，原话在决策变更史 D3（空间分配） 节）。只判 t1，t2–t12 不判（D3（空间分配） 已定项 10 射程没有「偏离即不合法」）。已知限度：这一版盘面上复算不出的几样只能近似——已释放的记录一律当占着（回收门槛复算不出：写死从 U + 4 起扫的坏实现与回收门槛放宽的坏实现两个方向都判不出，C689（建树落点判据抓不到写死从 U + 4 起扫的坏实现））；这一版自己分配的记录当空；开放段拿这一版树表单元所在的段当它（这次挂载更早开过、已经用尽的段认不出；提交内生块回落时会误排：这要求这一版发布时每块盘上都没有全空的 64 槽段，今天的写侧在「只暖机 / 只写行的挂载接发第一个文件」这一族里量到 0 次回落（单元区 240 / 256 / 384 槽，连续挂载 k ≤ 60，一次抽样、不是证明）；分配器上留了回落计数，`crates/singlefs-harness/tests/allocator_fallback_count_on_small_devices_after_repeated_mounts_and_the_first_file.rs` 断言它是 0，哪天走进回落、用例红，回来重看开放段认法）；隔离位、抬 F 扣住位与保护期内的 R 盘上都没有，没有分配记录罩着的槽一律当空。依据 D3（空间分配） 已定项 8 射程与已定项 10 ③ ④，以及判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第二节 K2 与用户 2026-10-04 对判据改法的定案（原话在决策变更史 D3（空间分配） 节）。 | 已实现（判据是 f2（按这一版分配记录现算），代码轮判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md`；池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark`：候选集里每条根与由记录施加出来的那几版各自重读树表与 extent 树判，U 取所选系统配置偏移 417；GPU 判器扫描内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `judge_first_data_unit_of_a_version`，`IMPLEMENTED_INVARIANTS` 下标 67；判不了的几格报部分没判（`InvariantVerdict::PartiallyJudged`，没有违例时整条报它，逐版列出实例、txg 与为什么，形态同 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同））：这一版的树表读不出（含树表指针全零：core 的根记录读者照它读盘 0 槽 0、读不出）或条目宽不到 200 字节，认不出这一版是不是这一类；认出来的那一版 extent 树读不全（数据单元数不全）、分配记录树读不全（落点算不出）；种类 1 条目不止一条、种类 3（分配记录树）条目不是恰好一条，这两格的树表本身由 I-9.16（树表条目按树 ID 严格升序且合发号次序） ③ ④ 判红、这里只记没判；有数据单元而池里某块盘上一个都没有，那块盘没有对象，跳过、不记（CPU 与 GPU 一致）；GPU 判器只有违例位，这几格不置位；走读到的版本里一版都不是从 mkfs 水位第一次建树的那一版时，整条报专门的不适用理由（`walk::NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK`，`crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 里那条用例的 I-5.5 一支只认它）；认出了那一版、而池里的盘上都没有它的数据单元（extent 树根全零、一条非零的数据指针都没有，或数据指针的落点全在池里没有的盘上），又没有别的版本判到时，整条报另一句专门理由（`walk::VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT`，用户 2026-10-06 弹窗定，选项原文「甲 另写一句」，`records/2026-10-01-里程碑三形式化接入.md`「用户 2026-10-06 弹窗定 b34 四问」那一行）；三格的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `a_tree_table_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged`、`an_extent_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged`、`an_allocation_record_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（数据单元从 50180 搬到 50182），合法镜像在 `crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`） |

> ⚠️ **I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 的等式在 zoned 布局上有一个已知缺口（[decisions.md](decisions.md) D12（目标介质） 定了多布局之后才出现）。**
> zoned 设备上一个 zone 里只要还有一块活数据就不能 reset ⇒
> **一个活快照钉住的物理空间是 zone 粒度，而它引用的块数是块粒度**。
> 于是「已分配 == 遍历所有引用得到的和」在那套布局上不成立，
> 除非把「已分配」明确定义为「被引用」，**并另立一个「不可回收」量**。
>
> 连带一条对用户可见的要求：**报给用户的「这个快照占了多少」不许被定义成引用块数之和**——
> 那个数在 zoned 上会**系统性地低于**删掉该快照真正能拿回的空间，
> 而 [pitfalls.md](pitfalls.md) 第 12 条骂的正是这类「`df` 输出对用户没有意义」。
>
> **本缺口尚未有对应条目**，补它需要先定义「不可回收空间」这个量。

```

**出处 `.claude/kb/invariants.md:214-263`（整段抄，未转述）**

```markdown
## I-6 加密不变量 —— 类

**依赖 D9（加密），17 个分项全部已定。**
**这些检查在加密未开启时应当整类跳过并显式报告为「不适用」，不许静默通过。**

<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-6.1 | nonce 不重用 | 加密开启时，全盘任意两个**已写出**的块（不论当前是否仍被引用），其 (密钥代号, nonce) 二元组不相同 | 未实现 |
| I-6.2 | 明文头字段白名单 | 加密开启时，任一单元的明文头字段集合 ⊆ 它那一类的白名单，**两张，按 D18（块里携带什么信息）已定项 11 登记表的码分**（码 1 数据单元类；码 2 / 3 元数据类；D18（块里携带什么信息）已定项 7 收口）——共同前缀两类都许：{magic / 格式版本 / flags（含单元类标签位）/ 头校验和 / **nonce 代号** / MAC}；元数据类**仅此而已**（五元组 / key 区间等身份字段一律进密文）；数据单元类另许 {fsid / 载荷 CRC}。⚠️ **2026-09-07 用户定案（D18（块里携带什么信息） 已定项 13，臂庚）按两处收窄**：① 码 1 的 {五元组 / 诞生代号 / 写序} 从「另许」里去掉，就地变密文；② **`声明长度` 从共同前缀里去掉**，三类一起变密文——它给出一个装得下的小文件的精确字节数，是收窄前泄漏面里最重的一项。字段的**位置与宽度一个都没动**，变的只是「盘上这几个字节是明文还是密文」 | 未实现 |
| I-6.3 | MAC 与密文匹配 | 加密开启时，任一被引用的块，其 MAC 与密文及关联数据匹配；**关联数据的每个字段由 checker 从权威树的 key 现算，不许从指针或明文侧读回**（需要密钥在场） | 未实现 |
| I-6.4 | 无未加密块 | 系统配置声明加密时，不存在算法类型字段标为「未加密」的数据块或元数据块 | 未实现 |
| I-6.5 | nonce 水位单调 | 记录在案的 nonce 水位 > 全盘出现过的最大 nonce（崩溃后不重用的可判定形式） | 未实现 |
| I-6.6 | MAC 长度一致 | 任一指针记录的 MAC 长度与系统配置声明的一致 | 未实现 |
| I-6.7 | 副本共用 nonce | 同一份数据的多个副本，其指针记录的 nonce 相同（nonce 未绑物理位置的可判定形式） | 未实现 |
| I-6.8 | 密文校验和匹配 | 加密开启时，任一被引用的块，其指针记录的密文校验和与盘上密文匹配（**不需要密钥**） | 未实现 |
| I-6.9 | 密文过而 MAC 败判身份错配 | 加密开启且密钥在场时，任一被引用的块，若**密文校验和通过而 MAC 失配**，判红并归类为「身份错配」——**不许归类为介质损坏，不许触发副本修复或投毒** | 未实现 |
| I-6.10 | 已停用·明文映射层不存在了 | **此编号不再使用**——2026-09-07 D9（加密） 已定项 5 改取「只做 scrub」，不设明文映射层、不设明文反向索引 ⇒ 这一格**无对象**。曾要求：加密开启且密钥在场时，checker 从权威侧重建出的明文映射层与反向索引与盘上明文副本逐条相同、差异判红不许就地修复 | 不适用 |

> ⚠️ **I-6.1（nonce 不重用） 只在「不论是否被引用」这个范围下才有判别力。** nonce 重用最危险的一批数据
> 恰恰是崩溃遗留的孤儿块——已写盘、未进树、因而不被引用。
> 把检查范围写成「被引用的块」，等于在设计上排除掉最该查的那一批。
> 因此 checker 必须两阶段：① 遍历树可达块；② **额外全盘扫描已分配物理区间**，
> 靠块的明文头识别「这是一个已写出的 singlefs 块」并取出其 nonce 纳入去重集合。
>
> ⚠️ **阶段 ② 对 D9（加密） 已定项 4（nonce 载体）提出一条硬约束：nonce 的代号部分必须放在明文头，
> 无密钥可读，不能整体埋进密文。** 埋进密文的话 checker 没有密钥就连「这块用过哪个 nonce」
> 都判断不出来，I-6.1（nonce 不重用） 的阶段 ② 与 [experiments.md](experiments.md) E5（nonce 唯一性 × 崩溃点重放） 的判据③一起失效。
> I-6.2（明文头字段白名单） 白名单里那一项因此写作「nonce 代号」而不是含糊的「序号」。
>
> ⚠️ **I-6.1（nonce 不重用） 是抗 bug 检查，不是抗攻击检查。** 阶段 ② 靠明文头 magic 枚举已写出的块，
> 而一个能写裸设备的攻击者抹掉 magic 就能让块从扫描里消失。
> 对抗场景下它给不出任何保证——[decisions.md](decisions.md) D9（加密） 已定项 6 记了适用范围。
>
> I-6.2（明文头字段白名单） 与 D18（块里携带什么信息） 已定项 3 的打架已按
> 「两张白名单、按单元类分」收口（2026-09-02，三方论证判决）；扫描器解密前先读
> 共同前缀里的类标签位分出类，再套对应那张表。**头校验和罩仍在明文的部分，MAC 罩密文**（覆盖范围见 I-2.4（头校验和覆盖范围）；加密是系统配置级全有全无），
> 加密元数据的头完整性由两者分层。仍挂在
> [checks-owed.md](checks-owed.md) C68（不变量白名单与已定条款打架） 的那一半是
> nonce 12 字节的归属三处打架与门禁比对工装。
>
> ⚠️ **I-6.2（明文头字段白名单） 的白名单是「允许出现在明文头」的清单，不是「可以拿来判定」的清单。**
> 里面每一项都只是提示：**必须在密文侧有权威副本，读者取密文侧那一份。**
> 从明文侧取期望值等于让攻击者控制判定的输入。依据见 [decisions.md](decisions.md) D9（加密） 已定项 6 的「期望值来源」一节。
>
> ⚠️ I-6.1（nonce 不重用） 与 I-6.5（nonce 水位单调） 是**唯一能把「nonce 重用」这类灾难变成可检出事实的检查**。
> nonce 重用在 ChaCha20 这类流密码下直接泄漏明文，而它不会表现为任何读写错误——
> 没有这两条检查，它永远不会被发现。验证手段见 [experiments.md](experiments.md) E5（nonce 唯一性 × 崩溃点重放）。

```

**出处 `.claude/kb/invariants.md:264-278`（整段抄，未转述）**

```markdown
## 待补

以下类别在对应决策定案前无法写出：

- 反向索引一致性（D1（数据可移动性 / 反向索引） 定案后：正向引用与反向索引必须双向可达且一致）
- 条带 / 多副本布局（D2（RAID 条带策略） 细化后）
- 快照可见性（D6（快照实现模型） 定案后）
- 索引结构自身的结构不变量，如树的有序性、平衡性（D8（核心索引结构） 定案后）
- 密钥层级与换钥（主密钥槽 80 与 KDF 标识 4 的宽度与落点 2026-09-13 由 D22（单元原子性怎么合成） 已定项 9 定死；还剩主密钥槽的内部构成与换钥不重写数据）

---


---

```

**出处 `.claude/kb/invariants.md:279-321`（整段抄，未转述）**

```markdown
## I-9 inode 树结构 —— 类

**依赖 D8（核心索引结构） 已定项 6，形态已定（码 2 内部节点 + 码 3 叶容器、记录定长 140、inode 号单调不复用），checker 未实现。**
与 I-1（块自描述） 类的分工：I-1.6（类标签登记且三处相等） 管单元自己的类标签合不合法，I-1.7（打包容器合法与判定顺序） 管一个容器自己合不合法并且任一步失败即容器级中止；
I-9（inode 树结构） 管跨单元的结构约束（谁该指向什么、号与 key 的关系）与对象级（一条记录）的判定，对象级判红只让那条记录 EIO，不进 I-1.7（打包容器合法与判定顺序） 的中止链。
**共同前置**：checker 要知道一棵树是 inode 树才能对它判这一类，而树表条目里有没有「树的种类」一段无落点（D8（核心索引结构） 已定项 8）；第一版镜像上可由打包记录类型 2 容器头里的出生树反推哪棵树是 inode 树，所以这一类今天不是恒不评估。
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-9.1 | inode 树根恒码 2 | 树表指向的 inode 树根，共同前缀偏移 6 恒为 2（索引节点），哪怕树只有一片叶或是空的（零条目、key 区间 [0, 0]）——D19（块指针的结构与宽度预算） 的指针不带被指对象的类与大小，根的类若会变，读者按 16 KiB 读 32 KiB 是静默误读 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.2 | 条目身份与子头相符 | inode 树内部节点的每条条目：类型段 ∈ {0, 2}；类型段 0 ⇒ 子单元偏移 6 = 2 且身份引用的出生树、容器号、出生代三段为 0；类型段 2 ⇒ 子单元偏移 6 = 3，子头的 (出生树, 打包记录类型, 容器号, 容器出生代) 与条目的身份引用四段逐字相等，子指针头部的出生树 == 身份引用的出生树，且子容器的打包记录类型 == 2（出生树那两处 2026-09-12 随 C297（inode 内部条目里出生树两份没有定处理） 用户定案加）；其余判该条目损坏。孩子头里的类与类型段说的那一类不一致时（条目照判损坏之后）怎么往下认：两份副本里有一份头里类标签是类型段说的那一类（类型段 0 是码 2 节点，类型段 2 是码 3 容器）、按那一类长度的整单元校验和与它的位置条目对得上，就照类型段那一支往下走那一份，引用登记的槽数按那一类；另一份按同一长度读回比校验和，对不上照红 I-2.1（校验和与内容匹配），I-4.8（近 K 代根校验和自洽）、I-7.4（近 K 代块未被复用） 跟着；一份都没有的不往下走，每份副本照 I-2.1（校验和与内容匹配） 那一行的长度读法判。两份各自证成两种不同的别类时取哪一类还没有条款，记在 C739（认孩子的类时两份副本各自证成不同类，取哪一类、按哪一类登记槽数没有条款）（`research/prompts/m3-batch28-code-r1-main-verification.md` 第三节第 1、4 条，`research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M1 那一行）。跨头共享时出生树的读法随 C110（跨头共享与加密后元数据类的块头一致性读法） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；GPU 判器走读内核 inode 树只接类型段 2 的层级 1 条目：类型段 0 而有一份按码 2 自证过的条目落判不了（`UNDECIDABLE_TOO_DEEP`），交回 CPU 判（删除线第 30 批，第 31 批补对拍流）） |
| I-9.3 | 内部节点消息数恒零 | inode 树内部节点的消息计数恒 0（ε = 0，D11（索引节点要不要留消息缓冲区） 已定项 6 的树轴）；计数字段住节点内部布局（D15（格式冻结政策） 第 4 层，未冻结），那一层定了这条才可判 | 未实现，前置：树的种类 + 节点内部布局 |
| I-9.4 | 容器号不超最小 key | 打包记录类型 2 的容器内每条记录的 inode 号 ≥ 容器号；沿叶序容器号严格递增，且左容器的最大 key < 右容器号（这条逼出合并只许左吸收右 / 左从右借）；同一片打包记录类型 2 容器里记录号沿记录序严格递增 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；同一片容器里记录号严格递增那一句 2026-10-05 删除线第 22 批实现） |
| I-9.5 | 删除队列里的号仍在树里 | 两个方向都要成立。① `deleted_inodes` 里的每个 inode 号在同一棵树的 inode 树里存在且记录的 nlink = 0（记录在回收完成前留在 inode 树里，`deleted_inodes` 只存号）；② inode 树里 nlink = 0 的每条记录，号都在同一棵树的 `deleted_inodes` 里（队列是派生态，队列 = inode 树里 nlink = 0 的号集合，D8（核心索引结构） 已定项 16 第 3 条）。**共用判法**（I-4.6（中间态其余不变量成立）、I-4.7（删除中 inode 有账）、I-3.4（可用空间扣待删占用） 都照这一段，不另写）：判走读到的任一版（候选集里每条根，含由记录施加出来的那几版），不只判最新根；`deleted_inodes` 与 inode 树经树表条目偏移 116 的头 ID 配对（头 ID 的取值见 I-9.27（树表条目的头 ID 取值）），配不上的版记判不了；树表、`deleted_inodes` 或 inode 树读不全的版报 `part_of_the_range_not_judged`（射程里判不了，不判红），这是今天除 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 外各条用的报法，是否改报第四种判定「部分没判」随 C701（别的判法静默跳过读不出的部分） 那一问由用户答，答了这几条随它一起改。树表 0 条的版没有这几棵树，不判；树表指针全零的版本照 core 的根记录读者跟过去判损坏（候选集里自证过的根照 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 那一行；由记录施加出来的那几版与根环里的根同一读法，用户 2026-10-06 弹窗定，选项原文「甲 两类都判损坏」，`records/2026-10-01-里程碑三形式化接入.md`「用户 2026-10-06 弹窗定 b34 四问」那一行），这一条的删除态那一遍（`walk::judge_deletion_state_of_the_versions`）对这两种版本都报射程里判不了；这一遍不扫下一次挂载先施加的那一版；树表单元相同的几版内容相同，只扫一次；没有删除的版本上 I-9.5（删除队列里的号仍在树里） 报成立（两个方向比的是队列与整棵 inode 树，干净的一版也真比过），I-4.7（删除中 inode 有账） 与 I-4.6（中间态其余不变量成立） 没有对象时报不适用并带理由 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，删除态扫描是 `judge_deletion_state_of_the_versions`；坏镜像登记在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `.chain`，搭建与改法在 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`，逐形的用例在 `crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs`；GPU 判器下标 69，扫描内核判）。两个方向各有坏镜像，「队列里的号在树里 nlink ≠ 0」那一形也有（C708（树里 nlink = 0 的 inode 不在删除队列里，池级 checker 判不出） 已还清）；`deleted_inodes` 的形态已定在 D8（核心索引结构） 已定项 16；判法与读法的出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 |
| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key（回退候选集里任一版的 inode 树都算），> 该树全部已发布的（按实例表判）墓碑记录的对象 ID（墓碑记录住墓碑树，D18（块里携带什么信息） 已定项 24；被崩溃恢复抛弃的时间线上写的墓碑不在所选根的墓碑树里）——2026-09-16 随 D8（核心索引结构） 已定项 6 收窄：被抛弃时间线的墓碑不算——崩溃恢复落到较旧的根之后水位随所选根那棵账载入，被抛弃时间线里发过的号会被重发，算上那些墓碑 checker 按扫描读就判红（管理员回退是挂着时的一次向前发布，水位取 max、不重发，D23（journal 的角色与格式） 已定项 14）；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）。**两半都判**：水位 > 树内最大 key；墓碑那一半（2026-10-05 起）水位 > 所选根（最新的那条根）墓碑树里已发布的（按所选根指着的实例表判）墓碑容器里全部记录的对象 ID 最大值，函数 `largest_published_tombstone_object_identifier`，一条已发布的墓碑都没有时没有对象，墓碑树读不全报射程里判不了；坏镜像：墓碑记录的对象 ID 抬到水位（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `raise_the_tombstone_object_identifier_to_the_inode_number_watermark`）；GPU 判器内核一并判，并进下标 41 那一位 |
| I-9.7 | 记录三段恒零 | 打包记录类型 2 的记录偏移 108 / 112 / 120 三段（填充 4 / flags 8 / 预留 20）恒 0；非 0 判该记录 EIO，容器其余记录照读（前向兼容政策，D15（格式冻结政策） 判定表；不是损坏检出，载荷校验和照样过） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.8 | 记录 inode 等于 key | 打包记录类型 2 的记录偏移 0 的 inode 号 == 它在 inode 树里的 key；不等判红、该记录 EIO，不许静默修复 | 未实现，前置：树的种类 |
| I-9.9 | locality 三值对照 | 记录的 `locality_id` == 该对象每条 extent key 的首段；extent 树未经重建时输出红 / 绿；系统配置 `map_provenance` 标 rebuilt 时两侧同源，输出「不可判定」，不许输出「通过」；按侧判（2026-09-16）：记录那一侧是从 key 编出来的、或 key 那一侧是从记录编出来的，同样输出「不可判定」；分批重新聚簇的中间态怎么判随 C346（后台重新聚簇没有定义） 定 | 未实现 |
| I-9.10 | 对象出生代三处一致 | 记录偏移 8 的对象出生代 == 该对象每个数据单元头五元组里的对象出生代 == 该对象的墓碑记录里的对象出生代（若有；该对象的墓碑记录按 inode 号认，走读到的每一版各比这一版自己的；墓碑记录住墓碑树，D18（块里携带什么信息） 已定项 24；只对 inode 记录还在的对象判，删完之后记录不在，这一个等号无对象）；重建后两侧同源，输出「不可判定」，与 I-9.9（locality 三值对照） 同形 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；墓碑那一个等号 2026-10-05 删除线第 22 批实现，GPU 判器第 23 批补） |
| I-9.11 | 可写克隆头有水位行 | 每个可写克隆头的树 ID 在记账里都有「inode 号水位」行（树表可枚举、记账可查）；「初值 ≥ origin 那一刻的水位」是建克隆那一刻的操作断言，转模型对拍——它的右操作数（建克隆那一刻的水位行）只住写出它的那条根自己的记账树里（D5（快照 / 空间记账机制） 已定项 2：每条根的记账树只装这条根那一代的行），对着镜像读不到 | 未实现，前置：树的种类 |
| I-9.12 | 分隔 key 落在孩子区间之外 | inode 树内部节点的条目按分隔 key 严格递增；第 i 条的分隔 key ≤ 第 i 个孩子的 key 区间下界，且 > 第 i−1 个孩子的 key 区间上界。判的字段是内部节点条目头 8 字节那个分隔 key（D8（核心索引结构） 已定项 6：条目 120 字节 = 分隔 key 8 + 身份引用 26 + 子指针 86，「分隔 key = 该孩子创建时的最小 key，条目按分隔 key 单调递增」）；孩子的 key 区间对码 3 容器由它装的记录现算，容器头不带 key 区间 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.13 | 类型 2 容器不空 | 任一打包记录类型 2 的容器记录数 ≥ 1（空叶在重建时没有 key 范围；唯一的叶被删空时不落盘、根变成零条目） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.14 | 树表条目的诞生 txg 跨根不变 | 同一棵树（按树 ID）的树表条目，在**同一条时间线上**的任意两条有效根各自的树表里，诞生 txg 必须相同；只有这棵树第一次出现在树表里时才设这个值（D8（核心索引结构） 已定项 8（树表条目的字段）；C374（释放代与树表诞生 txg 只有验收断言盯着））。「同一条时间线上的有效根」就是回退候选集（D23（journal 的角色与格式） 已定项 14 + D16（发布语义） 已定项 1）：最新那条根本身；以及根环里 (i, T) 满足两条的根——按**最新那条根指着的实例表**，没有实例 i 的行、或有行 (i, Ti, Wi) 且 T ≤ Ti；并且 T ≥ 最新根带的回退下界 F。有行 (i, Ti, Wi) 且 T > Ti 的根在被崩溃恢复切掉的旧线上，**不与现行线上的根比**（C511（回退到无文件那一版之后诞生代怎么接） 2026-09-23 用户定案收窄射程）。候选集不足两条根，或同一条时间线上没有一棵树的条目出现在两个不同的树表单元里时，报「不适用」，不报成立——几条根指着同一个树表单元、拿同一份字节比自己是恒真判定。判别力：① 把树表条目的诞生 txg 改成跟着这次发布的 txg 走，必须红；② 管理员回退前后同一条线上记得不同，必须红（回退是向前发布，前后是同一条线）。跨线（含被崩溃恢复抛弃的旧线）的诞生 txg 一致由 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 判，这一条不再有「只有被切掉的那条线记得不同，不许红」那一格（用户 2026-10-04 定）。推翻条件：找到一条历史，最新根的实例表把某条根判成有效，而它不是最新根的祖先（不在同一条线上）。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_release_generation_and_tree_table_birth`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`；层 0 每个崩溃状态都判。2026-09-23 收窄之后同文件两条：判别力 ② 的坏镜像 `birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_birth_invariant`（挂着时回退到 A 之后，回退那次发布 D 的树表把 extent 树（树 65536）的诞生 txg 记成 D 的 txg 9、A 那一版记 3：A 与 D 在同一条线上，只有 I-9.14（树表条目的诞生 txg 跨根不变） 红），原判别力 ③ 的正向用例 `birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_is_not_compared_and_every_invariant_holds`（被切掉的线由崩溃恢复造：C 被抛弃之后只改 C 那片树表里的诞生 txg）要改：这张镜像上 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 会红，不能再断言每一条都成立；欠账在里程碑三开工第一轮的第 9 批（`records/2026-09-30-里程碑三开工第一轮.md`，池级 checker 簇），由实现员改它，`judge_tree_table_birth_txg` 的注释里说明判别力 ③ 的那一段跟着改；判定在 `walk::judge_tree_table_birth_txg`） |
| I-9.15 | inode 记录的 blocks 等于 ⌈size ÷ 512⌉ | 打包记录类型 2 的每条记录，偏移 48 的 blocks == ⌈偏移 40 的 size ÷ 512⌉（D8（核心索引结构） 已定项 6：blocks 是逻辑长度的 512 字节块数，不表示分到的空间；C480（inode 记录的 blocks 怎么算全仓没有条款） 用户 2026-09-23 定）；不等判红。读者要不要也判、判了报什么没有条款 | 已实现（2026-09-24，池级 checker `walk::check_pool_image` 走 inode 叶容器时逐条判，与 I-9.7（记录三段恒零） 同一处；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：blocks 写成 64 / 5 / 7 各只红它；层 0 每个走得到 inode 叶的崩溃状态都判；变异：判定恒真、按向下取整比、写路径写回 64；里程碑「覆盖写、释放、回退与复用」收口表第 46 行） |
| I-9.16 | 树表条目按树 ID 严格升序且合发号次序 | 同一张树表（D8（核心索引结构） 已定项 8 的字段表）里，① 盘上相邻两条的树 ID 严格升序；② 树表里有的种类按发号次序（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist、`deleted_inodes`、墓碑树、谱系树、logged_ops；中央映射树不进树表）排，相邻两棵的树 ID 严格升序；③ 每一种种类码（登记过的与没登记的都算）至多一条；④ 树表非空时进树表的十一种（② 那一串）各恰好一条，读者重建一版时十二棵树的号逐棵取，缺哪一种都重建不出这一版（报「树表里没有那棵树」）。四道任一道不成立即违例；0 条的树表四道都成立，1 条的前三道成立。依据 D8（核心索引结构） 已定项 8「排序契约」与它的 ②「十二棵树的号从水位起连号发，次序照已定项 11 那一组 65536..65547」。判别力落在 ②：① 与 checker 在树表上判的 I-1.1（块头自述逻辑地址） 那组 key 严格递增是同一个谓词，只违反 ① 的镜像两条一起红；③ ④ 各有只违反它的坏镜像（一种出现两次、缺一种） | 已实现（2026-09-27，core `crates/singlefs-core/src/recovery.rs` 常量 `TREE_TABLE_ENTRIES_ORDERING_CONTRACT`，① ② 合成一个入口，`rebuild_version` 与 `walk_to_file` 在读任何一棵树之前拒；③ 是 `tree_table_entries_each_kind_at_most_once`，④ 由 `rebuild_version` 逐种取号时报「树表里没有那棵树」、`walk_to_file` 报「树表里缺一棵有根的树」；池级 checker `crates/singlefs-checker/src/walk.rs` 另写一份：`judge_tree_table_entries_ordering` 判 ① ②，`judge_tree_table_entry_kind_counts` 判 ③ ④（④ 的十一种是 `TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES`），违例写明哪一版根与哪两条或哪一种，列进 `image.rs` 的 `IMPLEMENTED_INVARIANTS`；GPU 判器走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `w_judge_tree_table_entries_ordering` 同判四道，② ④ 的循环十一种；坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`「互换两条的种类」那份只红 I-9.16（树表条目按树 ID 严格升序且合发号次序）；只违反 ① 的钉在 `tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs`；③ ④ 的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，用例名以 `a_tree_table_whose_` 与 `a_tree_table_without_its_` 打头（livelist 或 logged_ops 那一条改成第二条 extent、deadlist 那一条改成第二条分配记录、分配记录条目种类改成没登记的码、整条去掉分配记录 / deadlist / logged_ops 那一条），各只红 I-9.16（树表条目按树 ID 严格升序且合发号次序），去掉 logged_ops 那一条的一份走读也红；实现员报告 `research/prompts/m2-rev-tree-table-ordering-implementer-report.md`） |
| I-9.17 | 谱系树条目身份与子头相符 | 谱系树（D18（块里携带什么信息） 已定项 24）的每条条目：类型段 ∈ {0, 5}；类型段 0 ⇒ 子单元是码 2（谱系树根满长高之后，D8（核心索引结构） 已定项 6），身份引用的出生树、容器号、出生代三段为 0；类型段 5 ⇒ 子单元偏移 6 = 3，子头的 (出生树, 打包记录类型, 容器号, 容器出生代) 与条目的身份引用四段逐字相等，子指针头部的出生树 == 身份引用的出生树，且子容器的打包记录类型 == 5；其余判该条目损坏。孩子头里的类与类型段说的那一类不一致时（条目照判损坏之后）怎么往下认：两份副本里有一份头里类标签是类型段说的那一类（类型段 0 是码 2 节点，类型段 5 是码 3 容器）、按那一类长度的整单元校验和与它的位置条目对得上，就照类型段那一支往下走那一份，引用登记的槽数按那一类；类型段那一支自己也不往下走的（层级 ≤ 1 节点里类型段 0 的），两份都照 I-2.1（校验和与内容匹配） 那一行不往下走的读法判（`research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K1 N1 那一行），「另一份按同一长度」那一句不管它；另一份按同一长度读回比校验和，对不上照红 I-2.1（校验和与内容匹配），I-4.8（近 K 代根校验和自洽）、I-7.4（近 K 代块未被复用） 跟着；一份都没有的不往下走，每份副本照 I-2.1（校验和与内容匹配） 那一行的长度读法判。两份各自证成两种不同的别类时取哪一类还没有条款，记在 C739（认孩子的类时两份副本各自证成不同类，取哪一类、按哪一类登记槽数没有条款）（`research/prompts/m3-batch28-code-r1-main-verification.md` 第三节第 1、4 条，`research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M1 那一行）。类型段 0 那一支认「孩子是码 2」时只认按自己头里那一类的长度整份自证过的副本；两份都没自证过时当读不出，「孩子是码 2」这一小句不判（用户 2026-10-07 定，原话在 `records/2026-10-01-删除与并发事务.md` 2026-10-07 第 32 批 P 交回那一条；`research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 3 条）。身份不符单独判定，不再记走读失败；与 I-1.6（类标签登记且三处相等）、I-1.7（打包容器合法与判定顺序） 重合的小句仍留作合取。与 I-9.2（条目身份与子头相符） 同形 | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补） |
| I-9.18 | 删除队列里的号有谱系上的墓碑 | `deleted_inodes`（D8（核心索引结构） 已定项 16）里的每个号 N，墓碑树（D18（块里携带什么信息） 已定项 24）里都有一条在谱系上的墓碑记录 (N, N 的 inode 记录偏移 8 的对象出生代, [0, 2⁶⁴ − 1])。「在谱系上」：墓碑记录的死亡 txg 落在这一版的父链上——每一版从这一版自己的 txg 起、按这一版树表里的谱系树往回走，有谱系记录 (T_rb, T_old) 的那个 txg（T_rb）的父是 T_old，其余 txg 的父是 txg − 1（谱系记录住谱系树，D18（块里携带什么信息） 已定项 24；txg − 1 那一步核心层没有显式断言，C616（谱系父链靠 txg 连号没有断言））。不按谱系过滤的写法放过改坏（`research/prompts/m3-delete-r3-main-verification.md` 第三节判新不变量写法那一格）；墓碑记录不按已发布与否滤 | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补；对「队列里的号没有记录」那一形报判不了而不是红，I-9.5（删除队列里的号仍在树里） 的这一形坏镜像才只红自己，出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节最后一行） |
| I-9.19 | 谱系上的墓碑不指活对象 | 墓碑树里每条在谱系上的墓碑记录（「在谱系上」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义）指认的 (号, 对象出生代)，inode 树里有号与对象出生代都相同的记录时，那条记录的 nlink = 0 | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补） |
| I-9.20 | 谱系上的墓碑不重号且记录合法 | 墓碑树里在谱系上的墓碑记录（「在谱系上」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义）按 (号, 对象出生代) 不重复，这一句只判在谱系上的；墓碑树里全部墓碑记录（不论在不在谱系上）每条区间起 < 区间止，第一版只许整对象 [0, 2⁶⁴ − 1]，每条记录的对象出生代 ≤ 死亡 txg ≤ 这一版的 txg；每个打包记录类型 1 容器的记录数 ≤ 815（⌊(32768 − 136) ÷ 40⌋），这一句仍与前几句合取。不按谱系过滤的写法在「删 → 回退 → 再删」的合法历史上判红（`research/prompts/m3-delete-r3-main-verification.md` 第三节判新不变量写法那一格） | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补；「对象出生代 ≤ 死亡 txg ≤ 这一版的 txg」这一句被攻过一轮（`research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K2 N5 那一行，没打中，一次抽样）） |
| I-9.21 | 产品镜像里非最右墓碑容器装满 | 没有用只供测试的容量旋钮的镜像上，墓碑树里容器号沿叶序严格递增（根长高之后根的条目是类型段 0、容器号恒 0，见 I-9.23（墓碑树条目身份与子头相符），所以不沿根的条目读），除最右那一个之外的每个打包记录类型 1 容器记录数 == 815（开放容器满了才关，D18（块里携带什么信息） 已定项 24）。旋钮值由池级 checker 的新入口从镜像外给，旧入口当没装旋钮；判回退候选集里的每一版，墓碑树零根或只一片容器（没有非最右的容器）报默认不适用 | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补） |
| I-9.22 | 测试镜像里墓碑容器不超旋钮值 | 只供测试的容量旋钮压小了一个墓碑容器装的记录数的镜像上，墓碑树里容器号沿叶序严格递增，每个打包记录类型 1 容器记录数 ≤ 旋钮值；旋钮不落盘，checker 从镜像外拿旋钮值：旋钮值由池级 checker 的新入口给，旧入口当没装旋钮；判回退候选集里的每一版，墓碑树零根报默认不适用（只一片容器照判 ≤ 旋钮值）。按产品容量判这类镜像会在合法状态上红（`research/prompts/m3-delete-r3-main-verification.md` 第三节判容量旋钮那一格） | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补，GPU 恒不置位、层 0 恒不适用：层 0 与放量只走产品入口，拿不到旋钮值，照 I-4.4 先例） |
| I-9.23 | 墓碑树条目身份与子头相符 | 墓碑树（D18（块里携带什么信息） 已定项 24）每个内部条目的类型段 ∈ {0, 1}：0 ⇒ 子单元是码 2（根长高之后，D8（核心索引结构） 已定项 6），身份引用的出生树、容器号、出生代三段为 0；1 ⇒ 子单元是码 3，子头 (出生树, 打包记录类型, 容器号, 容器出生代) 与身份引用四段逐字相等、子指针头部出生树 == 身份引用出生树、子容器打包记录类型 == 1。孩子头里的类与类型段说的那一类不一致时（条目照判损坏之后）怎么往下认：两份副本里有一份头里类标签是类型段说的那一类（类型段 0 是码 2 节点，类型段 1 是码 3 容器）、按那一类长度的整单元校验和与它的位置条目对得上，就照类型段那一支往下走那一份，引用登记的槽数按那一类；类型段那一支自己也不往下走的（层级 ≤ 1 节点里类型段 0 的），两份都照 I-2.1（校验和与内容匹配） 那一行不往下走的读法判（`research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K1 N1 那一行），「另一份按同一长度」那一句不管它；另一份按同一长度读回比校验和，对不上照红 I-2.1（校验和与内容匹配），I-4.8（近 K 代根校验和自洽）、I-7.4（近 K 代块未被复用） 跟着；一份都没有的不往下走，每份副本照 I-2.1（校验和与内容匹配） 那一行的长度读法判。两份各自证成两种不同的别类时取哪一类还没有条款，记在 C739（认孩子的类时两份副本各自证成不同类，取哪一类、按哪一类登记槽数没有条款）（`research/prompts/m3-batch28-code-r1-main-verification.md` 第三节第 1、4 条，`research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M1 那一行）。类型段 0 那一支认「孩子是码 2」时只认按自己头里那一类的长度整份自证过的副本；两份都没自证过时当读不出，「孩子是码 2」这一小句不判（用户 2026-10-07 定，原话在 `records/2026-10-01-删除与并发事务.md` 2026-10-07 第 32 批 P 交回那一条；`research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 3 条）。身份不符单独判定，不再记走读失败；与 I-1.6（类标签登记且三处相等）、I-1.7（打包容器合法与判定顺序） 重合的小句仍留作合取。与 I-9.2（条目身份与子头相符） 同形（墓碑正推报告 `research/prompts/m3-tomb-r1-sonnet-output.md` 第 241 行 N1；根长高之后类型段 0 那一支是主 agent 按 K-R3-3 补的，被攻过零轮） | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补） |
| I-9.24 | 删除队列至多一条 key | 任何一版里 `deleted_inodes`（D8（核心索引结构） 已定项 16）至多一条 key；每个被判的版本各判一次，不遍历版本之间的历史；每条 `deleted_inodes` 条目都判，不管树表条目的头 ID 配不配得上，干净的一版报成立。依据：会话里每一次用户改动先把现行那一版的队列走空再做自己（D8（核心索引结构） 已定项 16 第 6 条的增补，用户定案），判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节「B4 + B8 队列上界与接口」一行 | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补） |
| I-9.25 | 树表条目的 previous_snapshot_txg 取值 | 走读到的任一根（候选集里每条，含由记录施加出来的那几版）的树表里：树的种类不是 inode 树（种类码 ≠ 2）的条目，偏移 100 的 `previous_snapshot_txg` 恒 0；inode 树的条目为 0，或 1 ≤ p ≤ 这条根的 `checkpoint_txg`。依据 D5（快照 / 空间记账机制） 已定项 9（`previous_snapshot_txg` 只写在该头 inode 树那条树表条目里，其余条目写 0 且 0 = 不适用）；inode 树条目「≤ 根 txg」那一半的条款是用户补定的（`research/prompts/m3-inv-b-r1-main-verification.md` 第四节用户定案第 2 条）。 | 已实现（2026-10-02，池级 checker `walk_tree_table_entry` 每走到一条树表条目判一次，每个走到的版本（含记录施加出来的）都走它；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，inode 树条目的 previous_snapshot_txg 压过根 txg 那一份由 2026-10-03 补；⚠️ 树表单元被几版共享时只在第一个走到它的那一版判（`visited_units`），共享它的别的候选根漏判，与条款字面「候选集里每条」不符，代码注释自认判得更松，代码改回 kb 字面，实现进下一批（主 agent 2026-10-03 定；`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 2 行，推的，没造镜像）；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 5 行） |
| I-9.26 | inode 改动计数落在出生代与容器诞生代号之间 | 走读到的任一 inode 叶容器（码 3，打包记录类型 2）里每条记录：对象出生代（记录偏移 8）≤ 改动计数（记录偏移 88）≤ 这片容器头的诞生代号（偏移 73）。上界不是等于：容器因别的记录改动而重写时，没改的那条记录的改动计数小于容器诞生代号是合法的；下界要「创建算一次改动」，这一句条款是用户补定的。依据 D8（核心索引结构） 已定项 6 偏移 88 那一行（改动计数 = 最后一次改动所在发布的 `checkpoint_txg`）与 D18（块里携带什么信息） 已定项 9 之后的容器重写规则。 | 已实现（2026-10-02，池级 checker `walk::walk_inode_leaf_container`：每条记录判偏移 8、偏移 88 与容器头里偏移 73 的诞生代号（不是偏移 61 的容器出生代）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，下界一份（改动计数 = 对象出生代 − 1）由 2026-10-03 补；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 5 行与第四节用户定案第 2 条） |
| I-9.27 | 树表条目的头 ID 取值 | 走读到的任一版（候选集里每条根，含由记录施加出来的那几版）的树表有条目时，逐条判条目偏移 116 的头 ID：种类码 2（inode 树）的条目等于它自己的树 ID（条目偏移 0）；种类 1（extent 树）、9（`deleted_inodes`）、10（墓碑树）的条目等于这张树表里种类 2 条目的树 ID，表里有多于一条种类 2 条目时放宽成「等于其中某一条的树 ID」（收严留给多头那一批），表里没有种类 2 条目时这三种判红；种类 3、4、6、7、8、11、12 的条目等于 0；别的种类码不判。树表 0 条的版本一条也不判；条目宽不到 124 字节（头 ID 那 8 字节读不全）的树表整张不判。不拿字面 65537 比：它只是树 ID 从 mkfs 水位 65536 起发的那一版上的取值。依据 D8（核心索引结构） 已定项 8「头 ID 8 字节的取值」与 D5（快照 / 空间记账机制） 已定项 9。 | 已实现（2026-10-04，池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_tree_table_entry_head_identifiers`，挨着 I-9.16（树表条目按树 ID 严格升序且合发号次序） 每张树表判一次、按条目计数，共享这张树表的别的版本不重判；条目宽不到 124 字节时这张树表报射程里判不了；GPU 判器走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `w_judge_tree_table_entry_head_identifiers`，`IMPLEMENTED_INVARIANTS` 下标 66；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（extent 树条目头 ID 写 0，只红它），从根环水位发树的合法历史在 `crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`） |
| I-9.28 | 全部可读根里同一树 ID 的诞生 txg 相同 | 根环里全部读得出且自证合法的根（不按实例表判有效与否，含被崩溃恢复抛弃的旧线上的根），任两条根各自的树表里同一树 ID（条目偏移 0）的条目，诞生 txg 必须相同；只有这棵树第一次出现在树表里时才设这个值（D8（核心索引结构） 已定项 8（树表条目的字段））。判定四种：有一处诞生 txg 对不上 → 不成立；各处都对得上，但有一条读得出、自证过的根树表读不出（含树表指针全零）、或条目比字段表短 → 部分没判，列出这些根（盘号、实例、txg、为什么没判）；没有一棵树出现在两个不同的树表单元里（几条根共用一个树表单元时拿同一份字节比自己，是恒真判定）→ 不适用；其余 → 成立。树表指针全零的根不是「没有树表」：core 的根记录读者不把全零当特例，照它那两条位置条目读盘 0 槽 0、读不出（可写挂载拒成 `Recovery(UnitUnreadable { slot: SlotNumber(0) })`），池级 checker 对根环里自证过的这样的根照 core 跟过去、判这条根损坏（红 I-2.1（校验和与内容匹配），最新根再红 I-4.8（近 K 代根校验和自洽）、I-7.2（最新根可完整遍历）、I-7.4（近 K 代块未被复用），别的候选根再红 I-4.8（近 K 代根校验和自洽）、I-7.4（近 K 代块未被复用）），这一条把它与树表读不出同一处置、报部分没判并点名「树表指针全零」（用户 2026-10-05 弹窗定，选项原文「甲 checker 判它损坏」，`records/2026-10-01-里程碑三形式化接入.md` 第 33 批「第 32–33 批代码轮第一轮判决」那一条，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第五节第 1 条）。只判根环里全部读得出、自证过的根（含被抛弃的、F 之下的），不含由记录造出来的版本：由记录施加出来的那几版与下一次挂载先施加的那一版不进这一条的比较，也不为它们报部分没判。这两类版本里树表指针全零，与根环里的根同一读法、照 core 的根记录读者跟过去判损坏（用户 2026-10-06 弹窗定 b34 四问，选项原文「甲 两类都判损坏」「甲 同改」，另一问选「乙 照今天」，出处 `records/2026-10-01-里程碑三形式化接入.md`「用户 2026-10-06 弹窗定 b34 四问」那一行）：由记录施加出来的那几版红 I-2.1（校验和与内容匹配）、I-3.1（已分配统计对得上）、I-4.8（近 K 代根校验和自洽）、I-7.4（近 K 代块未被复用）（判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P2）；下一次挂载先施加的那一版全零，在重放只施加这一次发布时就是重放之后那一版，红 I-4.1（重放后树可遍历），core 的两种挂载都拒。不判 I-2.5（位置条目按设备身份升序）：那一行把整条全零的指针整条豁免，不收窄。GPU 判器的走读内核对全零的树表指针与 CPU 同判：两条位置条目照 I-2.1（校验和与内容匹配） 判、记一次走读失败，映射根不走，不判 I-2.5（位置条目按设备身份升序）（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `w_walk_tree_table_and_mapping`；GPU 一侧是读码推的，没在卡上跑，判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P3）。与 I-9.14（树表条目的诞生 txg 跨根不变） 的分工：I-9.14（树表条目的诞生 txg 跨根不变） 只比同一条时间线上的根，这一条不分时间线、比全部可读根。依据：树 ID 被重发（C342（树 ID 水位在根读不出时退回去重发））时，新线上的树与老线上同一个 ID 的树诞生 txg 不同，I-9.14（树表条目的诞生 txg 跨根不变） 的同线收窄看不见这一形。推翻条件：任一合法历史（池级 checker 与层 0 其余判定都不红）上红。 | 已实现（2026-10-04，欠账来源 C342（树 ID 水位在根读不出时退回去重发）；池级 checker `walk::check_pool_image` 的 `judge_tree_identifier_birth_txg_across_every_readable_root`，四种结局；GPU 判器的扫描内核挨着 I-9.14（树表条目的诞生 txg 跨根不变） 判、只置违例位；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，合法历史、部分没判、不适用、对不上照报违例的用例在 `crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs`，树表指针全零的根报部分没判并点名的用例在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged`；`crates/mutations.tsv` 里以「第9批」「第9批改钉」打头、点名 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）或第四种判定的各行，与以「第 9 批 9-9 GPU 判器」打头、点名 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）的 3 行）。用户 2026-10-04 定新立，同时删 I-9.14（树表条目的诞生 txg 跨根不变） 判别力第 ③ 条与它的正向用例，判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第六节 S2-1 行。被攻过零轮：攻方三段合法历史上 0 红（一次抽样），被切掉旧线的镜像上红。部分没判：用户 2026-10-04 定「树表读不出算部分没判」、加第四种判定；四种结局的细则主 agent 定，零轮，`records/2026-09-30-里程碑三开工第一轮.md` 第 9 批「用户 2026-10-04 弹窗定（9-6 交回的两问）」那一条 |
| I-9.29 | 同号只许一个出生代 | 回退候选集里各版合起来（含由记录施加出来的那几版）同一个 inode 号只许对应一个对象出生代：每一版的 inode 记录的出生代与那一版谱系上墓碑的出生代都算；最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了（「在谱系上」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义）。依据 D8（核心索引结构） 已定项 6「同一条时间线上 inode 号不复用」；判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T10、T15 两格与第四节用户定案（T15 定「写」）。 | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补；判不了的门第 25 批改成看主走读有没有沿最新根的实例表链走到末片，与候选集滤法同一遍走读（`Walk::abandoned_roots_filtering`，判决 `research/prompts/m3-batch23-code-r1-main-verification.md` 第三节第 1 条），GPU 链走满上限还没完记判不了第 27 批补） |

**不进 I-9（inode 树结构） 的**：分裂 / 合并的身份传递（左半身份不变、右半号 = 触发分裂的新记录号、出生树 = 执行分裂的树）与合并时机是操作断言，对着一个镜像判不了，走单测 / 模型对拍与崩溃点重放（C116（叶容器的分裂 / 合并纪律没有会失败的检查））；
nlink == dirent 条数的门（dirent 树存在）今天没有输入，写不成可判定形式，随 dirent 树上线时一起立。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:6-35`（整段抄，未转述）**

```markdown

**D18（块里携带什么信息）的分项只有一套编号**，与「未定项」一节连号，不重排。

| # | 分项 | 定案 |
|---|---|---|
| 1 | **谱系重写序号** | 不做：它要买的「分得清陈旧副本与现行副本」已由 D3（空间分配） 已定项 1 的逐落点谓词买到 **状态：已定。** |
| 2 | **节点自带 key 区间** | 全部码 2 节点无条件带 key 区间，不按树分、不设开关 **状态：已定。** |
| 3 | **数据块自描述** | 明文，且宁可写大一点：每个数据单元带明文自描述头，五元组共 33 字节；不加 `locality_id` **状态：已定。** |
| 4 | **单盘自恢复的形态** | 取元数据多副本 + 分散放置（ditto），链臂否掉；先不开单设备布局线；放置偏移不许取 2 的幂 **状态：已定。** |
| 5 | **重建路径怎么与 A6（AAD 期望值不来自明文侧） 相容** | 重建是降级路径，按拿得到的见证者分三级；系统配置 `map_provenance` 留位 + 只读挂载 + 不许就地修复 **状态：已定。** |
| 6 | **数据块自描述头的位置** | 头放在低位，载荷从头之后开始；头挂块级不挂单元级 **状态：已定。** |
| 7 | **块头还要不要第六个明文字段** | 单元头字段表按单元类分：共同明文前缀 42 + 类身份段，预留 nonce / MAC 位；自证单元不用这张表 **状态：已定。** |
| 8 | **删除要不要写墓碑单元** | 要：删除在盘上留墓碑记录，容器取码 3，记录取打包记录类型 1，死者由记录自己的字段指认 **状态：已定。** |
| 9 | **「块级」的「块」是什么** | 块 = 单元，一个单元一份头；扫描器按池内最小单元大小步进（第一版 16384）；单元起点按类对齐 **状态：已定。** |
| 10 | **墓碑的粒度与载体** | 区间记录 + 打包共享单元；按代际装载、不跨代际混装；第一版 40 字节、不回收（已定项 24） **状态：已定。** |
| 11 | **单元类型标签的值域，以及打包记录单元算哪一类** | 值域是一张登记表；打包记录单元是独立的一类（码 3，头 107）；打包记录类型是第二级登记表：1 墓碑、2 inode、4 实例表、5 谱系 **状态：已定。** |
| 12 | **码 2 的写序要不要收窄成 4 字节** | 收窄：码 2 写序只留实例代号 4，腾出的 6 字节给载荷 CRC 4 + 预留 2；码 3 那 6 字节不动 **状态：已定。** |
| 13 | **加密开启时码 1 的身份段与 `声明长度` 放明文还是密文** | 取臂庚：码 1 身份段与声明长度就地变密文，位置宽度不动；码 1 明文白名单收成 {fsid, 载荷 CRC} **状态：已定。** |
| 14 | **重建路径怎么拿到完整的 96 位 nonce** | 取臂甲：nonce 代号就是完整的 96 位 nonce（12 字节），住单元明文头，指针里那份保留 **状态：已定。** |
| 15 | **已撤回·此编号不再使用** | 撤回（原题：码 2 的择版本规则）：三轮三方没找到一条没被打过的写法；码 2 照旧不进择版本规则 **状态：已定。** |
| 16 | **nonce / MAC / 算法类型预留位的开关贯通：码 1 的落点、码 2 / 码 3 含预留位的头宽、声明长度的原点** | 29 字节预留位紧接类身份段算进头：码 1 头 134、码 3 头 136、码 2 头 115 + 2 × key 宽；声明长度原点在含预留位的头末端 **状态：已定。** |
| 17 | **32 字节头校验和的算法；自证结构的覆盖范围** | 32 字节字段放 CRC32C 4 字节 + 28 字节零；自证结构的校验和覆盖整个槽含补齐，补齐恒为 0 **状态：已定。** |
| 18 | **码 2 的声明长度语义、补齐区恒 0、码 1 / 码 2 的偏移表** | 码 2 声明长度 = 条目数 × 条目宽；条目区之后的补齐区恒 0；码 1 与码 2 各一张带偏移字段表 **状态：已定。** |
| 19 | **字段值不值得带进块里：三条判据** | 三条叠加，缺一不可：更新周期与内容绑定；缺了它有判定从可判变不可判；相对已有机制有净增 **状态：已定。** |
| 20 | **已定要带的身份字段** | 自身逻辑地址 + fsid、generation（出生代）、所属树 ID（出生树）三样必带 **状态：已定。** |
| 21 | **自描述的射程：错向写，不是丢失写** | 丢失写块里带什么都抓不到，要块外的见证者：父指针里的期望代号与校验和，递归到根 **状态：已定。** |
| 22 | **显式字段 vs 折进校验和种子** | 要支持扫盘重建的信息必须显式写下来；只需检出误用、不需重建的才可以折进种子 **状态：已定。** |
| 23 | **否决清单** | 不带：子块记当前父节点地址、checkpoint 内联块数、COW 前驱地址、块内纠错码、每块墙钟时间 **状态：已定。** |
| 24 | **第一版墓碑：墓碑树、40 字节记录、谱系记录** | 第一版墓碑：专用墓碑树，40 字节整对象记录带死亡 txg，回退写谱系记录 **状态：已定。** |

```

**出处 `.claude/kb/decisions/08-核心索引结构.md:6-27`（整段抄，未转述）**

```markdown

**D8（核心索引结构）的分项只有一套编号**，不重排。

| # | 分项 | 定案 |
|---|---|---|
| 1 | **记账怎么表达成幂等完整值** | 记账条目一律是某个统计量在某一代的完整值、整格覆盖，不用增量 Δ，接受 key 数量上升 **状态：已定。** |
| 2 | **节点大小取多少** | 16 KiB：mkfs 写进系统配置，实现按常量走，不按设备特性算 **状态：已定。** |
| 3 | **key 布局** | key = `(locality_id, inode, offset)`，`locality_id` 创建时从父目录继承、改名时故意不更新 **状态：已定。** |
| 4 | **节点大小的上限由什么定** | 上限由恢复时间定，不由可恢复性定；取值在「重扫代价 ≤ 目标线」下取查找代价最低的那档，靠实验 **状态：已定。** |
| 5 | **whiteout 删除** | 接受墓碑；可能修改无界数量 extent 的操作先写意图再分批推进；第一版 unlink 的意图住 `deleted_inodes`（已定项 16） **状态：已定。** |
| 6 | **inode 记录的盘上字段表** | inode 树的叶是码 3 打包记录单元（类型 2，一叶 233 条定长 140 字节记录），内部节点是码 2、条目 120 字节、不留缓冲；已发布的 (inode 号, 对象出生代) 不复用 **状态：已定。** |
| 7 | **write buffer 条目自带的 `seq`** | 宽 4 字节、住 value 不住 key，射程覆盖记账 / 反向索引 / LRU 三棵树，`T_dirty` 的上界因此是挂载时判定 **状态：已定。** |
| 8 | **树表条目：树 ID 的分配与编码、树的种类、根指针以外还有什么** | 条目合计 200 字节、树 ID 8 打头；快照进树表；树 ID 水位取根环全部根记录该字段的 max；未知 flags 位拒收该条目 **状态：已定。** |
| 9 | **树的种类码的登记表** | 码 0 无、1..8 是八棵既有的树、9..12 是 deleted_inodes、墓碑树、谱系树、logged_ops；新树只追加不重用码 **状态：已定。** |
| 10 | **第一版记账走不走 write buffer 前端；`seq` 的取值规则** | 第一版记账直落记账树叶、不走前端；`seq` = 同一个 key 在一次发布里被写进树的第几次，从 1 起，第一版恒 1 **状态：已定。** |
| 11 | **码 2 索引节点的载荷布局、种类到条目宽的表、key 比较序** | 码 2 头自描述、基础 86 字节，条目区从含预留位的头末尾起定宽连续排列，key 是条目的前 key 宽 个字节 **状态：已定。** |
| 12 | **write buffer 前端与节点消息缓冲区的衔接** | 一棵树只准一级：走前端的三棵树 ε 恒 0、写一律进前端、发布前排空到叶；第一版两级都不开 **状态：已定。** |
| 13 | **索引树更新做不做状态折叠** | 同一次发布里同一个 key 改了多次时落叶只做一次、直接写最终态 E，中间态不写进任何节点，E 按到达序认 **状态：已定。** |
| 14 | **索引的总体结构** | 一套 btree 实现 + 多个独立 keyspace + 两个前端（write buffer、key cache）；派生态的分配记录树与 extent 树可以另起按 key 空间寻址的结构 **状态：已定。** |
| 15 | **write buffer 条目的三条硬要求** | 节点缓存必须抗扫描；条目必须是幂等完整值、不许是增量 Δ；条目必须自带序号 `seq`，这是格式要求 **状态：已定。** |
| 16 | **第一版 unlink 怎么删** | 第一版 unlink：只立 `deleted_inodes`，从尾部按 144 单元分批，挂载里续做到空 **状态：已定。** |

```
