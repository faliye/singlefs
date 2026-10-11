# m3-batch52-54-code-r1 判决：删除线第 52–54 批代码轮第一轮（2026-10-10）

被判的改动（按路径点名）：`crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/bad_disk_input.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs`、`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`、`crates/singlefs-core/src/inode_tree.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/transaction.rs`；测试十三份见 `research/prompts/_m3-batch52-54-code-r1-diff.md`。正文 `research/prompts/_m3-batch52-54-code-r1-body.md`。

## 一、证据核对（主 agent）

| 证据 | 核法 | 结论 |
|---|---|---|
| 开工快照 | 冻结副本 `~/.local/share/singlefs/work/m3-fixes-code-r1/repo/` 上 `sha256sum -c research/prompts/m3-batch52-54-code-r1-snapshot/crates-sha256.txt`；仓根 `kb-sha256.txt` | crates 25 份全对；kb 只有 `.claude/kb/checks-owed.md` 对不上（别的会话在腿跑着时加了 1 行，`git diff --stat` 1 insertion）。两条腿报告里 `grep -n checks-owed` 零命中，腿引的行不落在那一处 |
| 正推腿 `research/prompts/m3-batch52-54-code-r1-sonnet-output.md`（sha256 723754f2e759…）、攻方 `research/prompts/m3-batch52-54-code-r1-opus-output.md`（371 行，sha256 7b9788268330…） | `cite-check.py` 两份合判，`--root` 冻结副本，`--background` 背景材料 | 「✓ 核了 77 处引文，对上 77 处，没判 0 处（逐处列在上面）」 |
| 攻方模型 | 拷到 `~/.local/share/singlefs/work/m3-batch52-54-code-r1-verify/model/`（七份脚本与探针 sha256 与报告逐个相同），`rerun.sh … all` | 见第六节 |
| 主 agent 现查 | 读冻结副本 | `crates/singlefs-checker/src/walk.rs` 第 11992–11999 行只判 `left_container < right_container`，第 12008–12011 行最右那片 `continue`，第 12019 行旋钮下只判 `<=`：起点、连号、每片至少一条都没有判法；`crates/singlefs-core/src/recovery.rs` 第 2916 行条目宽、第 2919 行条目数、第 2927 行逐条判代与 seq，次序如攻方所说；`crates/singlefs-core/src/container_tree.rs` 只有第 476、511 行两个追加函数，`grep -n 'remove\|retain\|drain\|truncate'` 在 `container_tree.rs`、`tombstone_tree.rs`、`lineage_tree.rs`、`retirement_tree.rs` 零命中：三棵容器树只增不删；`crates/singlefs-checker-tier/src/crash_amplification.rs` 第 2316–2325 行：GPU 判不了的状态整块拒跑、「配置开了 GPU，不退回 CPU 判器」 |

## 二、逐格判

| 格 | 正推 | 攻方 | 判 |
|---|---|---|---|
| K1-1 条目宽提前之后别的坏法 | 一致：被拒的镜像集合不变，只有「窄条目 + 另一处坏」改报条目宽；补一句恢复只拒窄于 34、I-1.10 写「等于」 | **打中**（不分辨批次）：台阶行代写到根之后，两个读者都报「记账条目数…不是 3 + 6 × 盘数」，同一坏法落在别的行上报「代晚于根或 seq 为 0」；拿掉第 52 批照样报条目数 | 坐实。病根与第 52 批自己写的理由同形（`recovery.rs` 第 4770 行注释）：数条目按「统计量 12 且代早于根」认台阶行，代写坏就不认。两个成员同一变体、只差 `detail`；第 52 批两条用例钉的就是 `detail`，按 `detail` 读，算打中。攻方的改法（逐条判提到数条目之前）在它副本上第 52 批两条与 `writable_mount_of_a_formatted_pool` 全过，被攻过零轮。归第 56 批（第三节 G1） |
| K1-1 合法镜像被拒、K1-2 两读者次序 | 一致 | 没打中（六坏法 × 两读者成员两两相同） | 两条腿同向；攻方这一格是单次没打中，不拿它支撑「没有别的次序问题」，第 56 批代码轮复攻 |
| K2-1 合法镜像误报 | 一致（谱系「满了才关」条款没逐字写，靠类比） | 没打中（旋钮下 1800 次判 0 违例；产品容量退役 [959, 1]、谱系 [1359, 1] 成立） | 单次没打中，不作结论。写侧只有一个追加函数（主 agent 现查），误报要写侧另有开新片的路，读码没有。谱系「满了才关」写回 kb 时随 I-9.36 定义写明 |
| K2-2 坏镜像漏报 | 部分一致：容器号不连 / 不从 1 起不判；产品容量下中间多一条由 I-1.7 同时拦 | **打中**：(1, 3)、(2, 3)、最右多一片 0 条，三棵树带旋钮交新入口一条不红；墓碑 I-9.21 / I-9.22 同形漏 | 两条腿一致坐实。三棵树只增不删（第一节现查），所以「第一片是 1、相邻 +1、每片至少一条」在写侧恒成立，D18 已定项 24 第 1、5 条写了起点与「最大号 + 1」。补判法（G2），墓碑那两条一起补 |
| K2-3 CPU 与 GPU | 读码一致；说「超出 256 片交 CPU」 | 推的：GPU 谱系记录表上限 256 条，产品容量下谱系树有 I-9.36 对象（≥ 1360 条）的状态 GPU 都判不了、放量整块拒跑 | 两条腿在「判不了之后怎么办」上分歧，主 agent 现查 `crash_amplification.rs` 第 2316–2325 行：放量拒跑、不退回 CPU，攻方对。这一上限是旧的、这三批没碰，现有放量流没有 1360 次回退；不挡本轮，记欠账（第四节）。旋钮不传 GPU、I-9.37 在 GPU 恒不置位，两条腿一致，与 CPU 产品入口同判 |
| K3 抽函数前后行为 | 一致（读码，core 244 过） | 没打中（四段随机回退历史 52 次回退逐步摘要前后相同） | 两条腿同向；抽出的函数是纯重排，过 |
| K3 三条单测钉的事 | 一致于 inode 号水位三件；树 ID 水位那件没钉 | 同；改成只取没被抛弃的根，回退三份 harness 50 条全绿，推是等价变异 | 两条腿一致：「树 ID 水位照旧取全部」（D23 已定项 14，`.claude/kb/decisions/23-journal的角色与格式.md` 第 385 行）没有会红的测试。等不等价没核到，照 `.claude/rules/mutation-sampling.md`「没红的三类，判据不同」先当取样点不敏感：照第 53 批 F3 的办法造构造态输入（G5） |
| K4-1 参照独立性 | 不调 `inode_tree` 一致；共用前提三样，没比分隔 key | 同；另指读侧只读内存版本、不读盘 | 一致。共用 E176 那一份规则是本来的设计（参照照条款与 E176 写），不改；分隔 key 补比（G4） |
| K4-2 参照覆盖 | 部分：只走到先右先左分歧 1 次、借 1 次；借后吸收、左吸收、删空摘片 0 次；左吸收关掉照绿 | **打中**：13 个种子形状计数逐项相同，同上 | 两条腿一致坐实。补历史段与「每形至少一次」断言（G4） |
| K5 钉值 | 一致（重算对上；打印行、进度文件只多两项） | 没打中（八个钉值从原理或证据重算对上） | 两条腿各自独立重算、同向；过 |
| K6 判别力 | 第 52–54 批变异行能跑的点名用例都红；照绿的写法 12 种（见报告 K6.2 表） | **打中**：「严格递增」整句恒真，`checker_known_bad_images` 230 条、容器树长高 6 条全绿 | 一致坐实。照绿各条逐条归第三节 G2–G6，等价两条（M3 连锁只并一轮、删空摘片后不再并）两条腿都推等价，按条款「相邻两片之和 ≥ 234」前提，写成等价测试留档（G4） |

## 三、采纳的改法（第 56 批）

| 编号 | 改法 | 验收 | 来源 |
|---|---|---|---|
| G1 | 恢复两个读者（重建 `recovery.rs` 第 2916–2930 行、冷走读第 5390–5412 行）：逐条解开并判代与 seq 排到数条目之前（条目宽那一判照旧在最前） | 台阶行代晚于根的镜像两个读者都报「记账条目的代晚于根或 seq 为 0」（攻方 `probes/corrupt_on_disk_k1_append.rs` 那一形做成正式用例，只读恢复与可写挂载各一条）；第 52 批两条用例照过；变异行：两个读者各把次序换回 | 攻方 K1 |
| G2 | 池级 checker：I-9.21 / I-9.22 / I-9.36 / I-9.37 的容器号加判「沿叶序第一片是 1、相邻 +1」与「每片至少一条」（三棵树都只增不删）；CPU 与 GPU 同判 | 攻方 `mut-w-gap2`、`mut-w-start2`、`mut-w-eager` 三形做成坏镜像，三棵树各红自己那一条；合法镜像（旋钮下随机历史、产品容量 [959, 1]、[1359, 1]）不红；变异行各一 | 两条腿 K2-2 |
| G3 | I-9.36 / I-9.37 补会红的用例：「两片同号」坏镜像谱系、退役各一张（CPU 与 GPU）；产品容量的合法多容器镜像（退役 960 次、回退 1360 次）钉 959 / 1359 不对调、最右那片不要求满；旋钮下退役树「多于旋钮值」坏镜像；谱系树、退役树、整版树表读不全时报「射程里判不了」；旧墓碑旋钮入口谱系、退役给 `None` | 正推 K6.2 表 w1、w3、w3b、w4、w5、w6、w7、w10、w12 与攻方 `c-incr-true` 各在新用例上红；产品容量那条慢就标 harness 慢档 | 两条腿 K6 |
| G4 | M3 独立参照：历史加「删到某片叶空」「只有左邻那一对装得下」「删左片到借、再删右片」三段，参照比分隔 key，断言删空摘片、左吸收、借、借后吸收各至少走到一次；连锁只并一轮、删空摘片后不再并两种变异写成等价测试留档（前提：相邻两片之和 ≥ 234） | 正推 m2、m5、m9 与攻方 `k4-no-absorb-after-borrow` 在参照用例上红 | 两条腿 K4-2、K6 |
| G5 | 回退的树 ID 水位：照第 53 批 F3 抽出私有函数加构造态单测，交一条被抛弃的根树 ID 水位高于 cur 那一线，断言取它 | 攻方 `k3-treeid-not-abandoned`、正推 mt1 红 | 两条腿 K3 |
| G6 | GPU 把 I-9.37 的旋钮传进去（或写明 GPU 只判产品入口并让旋钮镜像交 GPU 时报判不了） | 带旋钮的镜像 GPU 与 CPU 新入口逐位相同 | 上一轮遗留、正推 K2.3 |

第 55 批（贴边 unlink warning 用例）照用户定案合入，不再单验（`research/prompts/m3-batch50-code-r1-main-verification.md` 第七节）。

## 四、零轮与欠账

- G1–G6 的改法都被攻过零轮，第 56 批合入后开它的代码轮。
- 单次没打中、本判决不拿来作结论的格：K1-1 合法镜像、K1-2、K2-1（攻方只抽一次；第 56 批代码轮复攻，崩溃恢复抛弃段里写的容器那一形点名要造）。
- 记欠账（`.claude/kb/checks-owed.md`，随 kb 写回一批）：① GPU 谱系记录表上限 256 条，产品容量下谱系树有第二片容器的状态 GPU 判不了、放量拒跑（`crash_judge_checker_common.wgsl` 第 853 行），现有流走不到；② 恢复的条目宽只拒窄于（记账 `records.rs` 第 329 行、码 2 内部节点 `code_two_tree.rs` 第 1005 行），I-1.10 写「等于」，宽条目由池级 checker 拦。
- kb 写回（随删除线写回一批）：`.claude/kb/invariants.md` 第 32 行 I-1.7 谱系记录宽 16 → 24、补类型 3 退役 34；I-9.36、I-9.37 两行（连同 G2 的起点、连号、每片至少一条，谱系「满了才关」写明）；I-9.21 / I-9.22 补 G2 那一半；第 12 行条数。

## 五、交用户表

无。K1 的「成员按变体还是按 `detail` 读」按第 52 批自己钉的读法定（`detail`），不另交。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D18（块里携带什么信息） 已定项 24 | 不影响 | 2026-10-10 不受影响：容器号「从 1 起、最大号 + 1」照写，缺的是池级 checker 的判法（G2），不改条款 |
| D23（journal的角色与格式） 已定项 14 | 不影响 | 2026-10-10 不受影响：Opt2 抽函数前后行为相同；树 ID 水位那半句缺测试（G5），不改条款 |
| D8（核心索引结构） 已定项 6 | 不影响 | 2026-10-10 不受影响：M3 判定照实现；缺的是参照覆盖（G4），不改条款 |

## 六、复跑核对（攻方模型）

攻方模型拷到 `~/.local/share/singlefs/work/m3-batch52-54-code-r1-verify/model/`（七份脚本与探针 sha256 与报告开头那张表逐个相同），在空工作目录 `run/` 从头跑 `SINGLEFS_SCRIPTS=scripts SINGLEFS_REPO=<冻结副本> OPUS_THREADS=15 OPUS_MEMORY_CAP=12G bash model/rerun.sh run all`（第一次没设 `SINGLEFS_SCRIPTS`，模型目录拷出仓之后找不到脚本目录，rc=1、什么都没跑；设上重跑），rc=0，compare 原样末行「compare：22 份，不一致 0 份」，22 份逐份「一致」。第二节引的攻方数（K1 六坏法的成员、K2 旋钮下 1800 次判 0 违例、产品容量两条边、三种容器号坏法不红、K3 四段摘要、K4 13 个种子形状计数、K5 八个钉值、K6 恒真照绿）都在这 22 份里，主 agent 复跑与腿交的一致。
