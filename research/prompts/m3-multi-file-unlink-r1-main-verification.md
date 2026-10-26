# m3-multi-file-unlink-r1 判决：第一个文件之外还有文件带数据单元时的删除各步（主 agent，2026-10-19）

<!-- doc-lint:not-numbers U1 U2 U3 U4 U5 M1 M2 M3 M4 M5 M6 M7 -->

正文 `research/prompts/_m3-multi-file-unlink-r1-body.md`，背景材料 `_m3-multi-file-unlink-r1-background.md`（sha256 b64a225e…），开工快照 `m3-multi-file-unlink-r1-snapshot/`（`kb-sha256.txt` 与 `tree-sha256.txt`，后者是打上按 inode 号写数据补丁的树 `~/.local/share/singlefs/work/m3-multi-file-unlink-r1-tree` 里 core 与池级 checker 的 43 份源文件）。正推腿（Sonnet）报告 `research/prompts/m3-multi-file-unlink-r1-sonnet-output.md`（363 行，sha256 `58471a81a1396ba6f20782a5669b62a96617b68e35eedd8cdc6dc06040e83ac3`），模型 `m3-multi-file-unlink-r1-sonnet-model/`；攻方腿（Opus）报告 `research/prompts/m3-multi-file-unlink-r1-opus-output.md`（237 行，sha256 `56559f449f8e614b7bda147f58dc892d0e38f862c7b316d72273dc30ff964817`），模型 `m3-multi-file-unlink-r1-opus-model/`。

开工快照：`research/prompts/m3-multi-file-unlink-r1-snapshot/`

## 一、核查

- 快照：`sha256sum -c m3-multi-file-unlink-r1-snapshot/kb-sha256.txt`（仓根）与 `tree-sha256.txt`（那棵树的根）全部 OK；腿跑着的时候主 agent 没改快照里的文件。那棵树之外，同一份补丁 2026-10-19 打进了主工作区（`apply-writer-patch.py` 带「快照冲突已判」），主工作区 `crates/` 由此与那棵树的 core 一致，腿只读那棵树。
- cite-check（`--root` 仓根、`--background` 背景材料，两份报告一起）：`✓ 核了 190 处引文、21 处裸行号，对上 190 处，没判 21 处（其中裸行号没有引文可比 21 处；逐处列在上面）`。
- 复跑：两腿模型目录拷到 `~/.local/share/singlefs/work/m3-multi-file-unlink-r1-main/`，各经 8G 包装起 `tools/rerun.sh --frozen <那棵树>`：攻方 `name=rerun_compare variant=today same_as_model_out=yes lines=18`、`variant=lift … lines=42`、`variant=lift_count … lines=26`，exit 0；正推 `name=rerun_compare result=identical lines=31 sha256=7783d076f0cc9fa21d06296d6af1c221550df0d86c5ee7095ec3f29bbea7d454`，exit 0。
- 主 agent 现查（那棵树与此刻主工作区同一份）：
  - `crates/singlefs-core/src/transaction.rs:4986`「DeletionStepOfThePublish::FinalBatch { inode } if inode.0 == FIRST_INODE_NUMBER => 0,」在；`:6965`「let data_units_before = previous.map_or(0, |version| version.data_pointers.len());」在；`:6990`–`6992` 批那一支调 `plan_the_extent_tree_after_a_deletion_batch` 传 `FIRST_INODE_NUMBER`。批截第一个文件（攻方 U1-乙）的病根坐实。
  - `transaction.rs:6445`「upper_root_level_for(largest_inode_with_an_entry).max(previous_upper_root_level);」在：写路径层级只升不降。
  - `crates/singlefs-checker/src/rebuild.rs:769`–`771`：被墓碑压着的 (号, 对象出生代) 还有记录就整张交回 `DeletionInProgressNotWrittenForTheScanRebuild`。攻方 U4-甲坐实。
  - `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3171`「if (upper_count != 1u) {」与 `:3174` 要唯一那一条是 inode 1，失败点 `FAILURE_POINT_UNIT_MALFORMED`。正推第 6.1 节「GPU 恢复内核不通用」坐实（推的结局，没跑：CPU 恢复照补丁收多文件，两边对多文件镜像判得不同）。
  - 攻方 U2-甲说「核心这一侧 `read_upper_node_and_its_subtree`（`extent_tree.rs:1130` 起）对空叶、没有孩子的内部节点不判」：**不成立**。`extent_tree.rs:1144` 那一处起的读节点函数在 `:816`「if header.entries.is_empty() && !(is_the_root_of_the_tree && expected_level == 0) {」拒零条目的非根节点（正推第 4.2 节引的同一行）。核心读者与池级 checker（`crates/singlefs-checker/src/lib.rs:701`）对「非根空节点」判得一致；U2-甲 剩下的只有「层级 ≥ 1 的零条目根」与删空判定那一句没写层级，规范形规则（M2）下这一形不出现。

## 二、判定与去向

| 格 | 判定 | 依据 | 去向 |
|---|---|---|---|
| U1 删别的带数据文件 | 两腿一致：意图条款对任一号成立；分批与最后一批条款没写剩余单元数取谁、上段那一串与最后一批对上段条目做什么。攻方量了两种「原样套用」都错：只撤拒绝，最后一批删了记录、条目与单元留着（I-4.7、I-1.13 红，下次可写挂载恢复拒）；只改剩余单元数，批截的是第一个文件、丢的单元被尾部标签洗成洞，删除走完前没有判法报 | 两腿；攻方 `out/opus-lift.txt`、`opus-lift_count.txt`（复跑相同）；主 agent 现查 transaction.rs 三处 | M1 |
| U1 最后一批对上段 | 两腿一致：去掉那一条条目，不写标签 0（`.claude/kb/decisions/08-核心索引结构.md:431`「标签 0 全零，只有读者认、写者不写」） | 两腿 | M1 |
| U2 上段形状 | 两腿各自推出同一条规则：形状只由有条目 inode 集合 S 定（节点 = S 各号那一串的并，根层级 = 罩得住 max(S) 的最低一层，S 空是层级 0 零条目根），写删共用。正推模型：只升不降那条规则下同一个 S 出现两种形状（产品几何 25 / 4441 个 S），规范形 0 个，重写节点数不多于前者；攻方量了只升不降加第一个文件那一支断言：上段长到层 1 再删第一个文件，`unlink.rs:239` 断言 panic，意图已落盘、之后每次可写挂载续做都 panic | 正推 `out/run.out` 第 1–8、15–17 行；攻方 `opus-lift.txt:31`、`:35` | M2 |
| U2 ①b 与逻辑内容 | 两腿一致：两种规则下 ①b 与「逻辑内容」都不破；只升不降破的是第 7 条「重写角色集合钉绝对值」的可钉性（绝对值随历史走） | 两腿 | M2 |
| U3 删第一个文件 | 两腿一致：最后一批对 inode 1 也走 M2 的统一上段计划，触发按 `FinalBatch` 的号分，不再按 `data_units_before > 0` 猜；删完第一个文件而别的文件还在，这一版归 `WithFile`，不加成员 | 两腿；攻方 `opus-lift.txt:24`（同在第 0 片叶时撤拒绝就做成） | M3 |
| U3 K-U5「拦」 | 两腿一致：今天的补丁不拦（攻方量了：删空 → 建 inode → 按号写做成、checker 全绿，写进去的文件之后删不掉，`opus-today.txt:1`–`9`）。攻方另打中：「先建 inode 再删第一个文件再写」与「删空再建 inode 再写」走到逐字相同的版本摘要，按成员判分不开（判别子观测不到）；正推把「拦」的读法摆成 α 版本级、γ 目标级、β 时序级三个候选 | 两腿；`opus-today.txt:12` | 交用户（第五节） |
| U3 删序 | 两腿一致：不同删序终态逐字节与「逻辑内容」都不同（墓碑死亡 txg），不要求相同；上段位置集合在规范形下相同 | 正推 `out/run.out:17`；攻方推的 | M3 |
| U4 队列、续做、墓碑、快照 | 两腿一致：`deleted_inodes` 至多一条 key、续做、墓碑每对象一条、快照 deadlist 都按号通用，条款不改；理想模型对拍那一侧的理由句「只有 inode 1」（`crates/singlefs-harness/src/model_comparison.rs:123`）不再成立 | 两腿 | M4 |
| U4 扫描重建 | 攻方打中（量过、主 agent 现查坐实）：一个对象删除中，扫描重建对整份镜像报不适用，别的文件在整段删除期间失去 I-1.13；正推没看到这一格 | 攻方 `opus-lift.txt:39`；`rebuild.rs:771` | M5 |
| U4 GPU | 正推打中（主 agent 现查坐实）：GPU 恢复内核只收「上段一条、是 inode 1」，多文件镜像判 `UnitMalformed`，与 CPU 恢复不同；这是按 inode 号写数据那份补丁就带来的缺口，不是删除新增的 | 正推第 6.1 节；`crash_judge_common.wgsl:3171`–`3174` | M6 |
| U5 前提清单 | 攻方列 14 处把「只有第一个文件写数据」当前提而正文没点的地方，5 处决定删除各步走哪条分支；随机历史与崩溃流里没有一处调按 inode 号写的入口；正推估改动行数（核心非测试净 +130、测试 +670、checker +210、模型与历史 +380、崩溃流 +1200，推的） | 攻方 U5 节；正推第 7 节 | M7 |

## 三、改法（都被攻过零轮；这一段实现之后的代码轮攻它们）

- **M1 删任一号**：D8 已定项 16 第 2 条对任一 inode N 同一条——剩余单元数取 N 在 extent 树里的单元数（第一个文件取它那一棵，别的文件取 `files_other_than_the_first` 那一项，没有的取 0、只有最后一批）；批只重写 N 被截那一片下段叶与它的祖先，加上段 N 那一串；最后一批释放 N 的全部单元、删记录、删队列 key、把 N 的条目从上段去掉（不写标签 0）。`DeletionStepOfThePublish::Batch` 带号，`data_units_after_this_publish` 不把别的号的批当第一个文件；extent 计划按号分派；被截、被摘的别的文件的数据单元与下段节点进「没有同名后继的上一版角色」。删掉 `refuse_an_unlink_the_deletion_steps_do_not_cover` 与那个拒绝成员，拒绝用例改成成功用例。
- **M2 上段规范形**：写进 D8 已定项 14 extent 那一条与 D8 已定项 16 新增第 8 条，同一份文字——上段形状只由有条目 inode 集合 S 定：节点 = S 里每个号从它那片叶到根那一串的并；根层级 = 罩得住 max(S) 的最低一层；S 空时是层级 0 第 0 格一个零条目根兼叶；非根节点不许零条目；一次发布里进出 S 或条目变了的号那一串上留在新形状里的节点与新出现的节点重写，别的照抄，上一版有而新形状没有的换下；长高那次旧根照抄成根之下的节点，降低那次上一版里降后层级的第 0 个节点照抄成根、旧根与脊线上方摘掉的节点换下。写第一个文件、写别的文件两处去掉 `.max(上一版)` 与「并上一版全部节点」，与删除共用一个纯函数计划。落地层：计划函数、单测、理想模型，加池级 checker 一条新不变量（上段位置集合等于 S 定的形状、根层级最低；必红世界：多一个脊线节点、根层级高一层、少一个祖先），GPU 判器同判同置位——两路逐位相同照用户 2026-10-19 定的双机导入冲突岔路（「补 GPU 扫描重建内核」，`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 163 行）的同一口径，不再开「进覆盖表、内核不置位」的欠账。核心读者拒非规范形状不做（回退与整理搬迁两种发布没过模型，记欠账）。降层方向角色名在 core 里走不走得通由实现员编出来定（正推第 4.6 节末：只读到对称方向）。
- **M3 删第一个文件**：最后一批对 inode 1 也调 M2 的计划，按 `FinalBatch { inode }` 的号分支；删掉 `plan_the_extent_tree_after_the_final_batch_of_the_first_file` 的「上段只有根兼叶」断言那一支；删完第一个文件、别的文件还在的一版归 `WithFile`，D8 已定项 16 第 5 条末尾补一句（inode 树还有任一记录或 extent 树还有条目的一版是带文件的一版，不管 inode 1 在不在；第三个成员只指文件集合真的空）。第 7 条末尾补：不同操作序列的终态不要求逻辑内容相同，要求上段形状只由 S 定、每个崩溃态 checker 全绿。
- **M4 取样点与理由句**：幂等判据 ①–④ 的取样点照正推第 6.2 节那张表扩到每个号（崩在 N 的意图之后、批之间、最后一批之前；删第一个文件那一批前后降层；删到 S 空），另加攻方第 U4-乙 节「删 N 的每一批之后读回第一个文件逐字节相同」；`model_comparison.rs:123` 的理由换成「会话先走空」。
- **M5 扫描重建按对象不判**：I-1.13 报不适用那一列里「删除中（墓碑压着的对象还有记录）」改成只对那一个 (号, 对象出生代) 不比——两侧都去掉它的映射、单元与记录，别的对象照比；必红世界：别的文件删除中途改坏一条映射，I-1.13 红。GPU 扫描重建内核（C821 那份）同步改。
- **M6 GPU 恢复内核收多文件**：`crash_judge_common.wgsl` 恢复那一段按上段条目逐条收（与 CPU 恢复同一套：至少一个单元、单元连号、尾部没有洞），加两文件镜像上 GPU 与 CPU 恢复逐位相同的对拍；崩溃放量判多文件流之前必须做完（多机配置开了 GPU 就由 GPU 判，不降级）。
- **M7 前提清单**：攻方 U5 节 14 处逐处处置（决定分支的 5 处并进 M1、M3；误导的理由句改掉；理想模型、崩溃注入、坏盘输入、场景脚手架里只罩第一个文件的地方，按「罩不罩多文件」逐处写进实现员的验收）。崩溃流：多文件删除流挂在哪（并进一·1 第二条流后面加 unlink 段，或另开一条）等草稿流写出、数过崩溃点再定。

## 四、交给实现员

两份按文件聚簇（core 与 harness 一份；checker 档 GPU 一份），等 K-U5「拦」的读法用户定了、按 inode 号写数据的第二位实现员交回之后再派——第二位实现员此刻在改 `model.rs`、`history.rs`、`model_comparison.rs` 与 `transaction.rs`，与 M1–M4 撞文件；GPU 那一份等 b6 的验证段与代码轮快照过了、C821 补丁打进之后再派（M5、M6 改的是 C821 补丁新加的扫描重建内核与 b6 也在动的 `crash_judge_common.wgsl`）。

## 五、交用户

| # | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| 1 | K-U5「拦」在第一个文件已删的一版上管到哪：α 版本级（inode 树里没有 inode 1 的记录，就拒一切按 inode 号写数据）；γ 目标级（同一前提，目标 inode 此刻没有数据单元才拒）；β 时序级（目标的对象出生代晚于 inode 1 墓碑的死亡 txg 才拒） | 产品语义，没有可量的量；β 依赖墓碑第一版不回收（D18（块里携带什么信息） 已定项 24），且要读墓碑树，与删空判定「不看墓碑树」那一句另立 | 用户答「删完第一个文件之后，原有的带数据的别的文件还能不能改写；删之前建的 size 0 的 inode 还能不能第一次写数据」 | 用户 2026-10-19 弹窗定「β 时序级」：只拒对象出生代晚于 inode 1 墓碑死亡 txg 的 inode 写数据；原有的别的文件照常能写；依赖墓碑第一版不回收，墓碑回收落地时要换读法（记进 C618（删完之后不能再建带数据的文件）） |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 14 | 支撑 | 2026-10-19 改了：extent 上段加规范形一句（M2），归阶段同步，实现之后的代码轮攻它 |
| D8（核心索引结构） 已定项 16 | 支撑 | 2026-10-19 改了：第 2 条对任一号（M1），第 5、7 条补句与新增第 8 条（M2、M3），射程句改「按 inode 号删任一 inode、无目录」，K-U5 那一句随用户定的读法改写；归阶段同步 |
| D18（块里携带什么信息） 已定项 24 | 不影响 | 2026-10-19 不受影响：墓碑每对象一条、死亡 txg 取意图那次，对任一号照用 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-19 不受影响：删除类发布的准入与推抬 F 与号无关 |
| D16（发布语义） 已定项 5 | 不影响 | 2026-10-19 不受影响：切分纪律只管写请求 |
| D5（快照 / 空间记账机制） 已定项 19 | 不影响 | 2026-10-19 不受影响：deadlist 的生死判按角色走，`DataOfAFileOtherThanTheFirst` 已在问生死那一支 |
