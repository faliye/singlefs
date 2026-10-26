# m4-multi-inode-size-r1 判决：截断与变长从第一个文件推广到任意活 inode（主 agent，2026-10-26，第一轮）

<!-- doc-lint:not-numbers S2 S3 S4 S5 S6 S7 H1 H2 H3 H4 H5 H6 K1 K2 K3 K5 K9 K12 -->

被判的条款：`.claude/kb/decisions/08-核心索引结构.md` 已定项 5（含「截断入口边界」那一句）、已定项 14；截断判决 `research/prompts/m4-trunc-r1-main-verification.md`、`research/prompts/m4-trunc-r2-main-verification.md`。射程由用户 2026-10-26 定：「任意活 inode 都能缩短与变长」，core 别处只认第一个文件的地方仍留里程碑十的一·3（C618）。冻结副本 `~/.local/share/singlefs/work/m4-close-rounds-frozen/`（crates 705 份的 sha256 在 `~/.local/share/singlefs/work/m4-close-rounds-frozen.sha256`）。

两条腿：正推 `research/prompts/m4-multi-inode-size-r1-sonnet-output.md`（sha256 b98ac04720ce243775db308580d5a2dafa4676fbda3c7d55f886f4fdd25866f1），攻方 `research/prompts/m4-multi-inode-size-r1-opus-output.md`（sha256 d3f81c588c1601baf0be8405fbc88c3c7a128c8933f490366aa7434b2cc1e533），模型目录 `research/prompts/m4-multi-inode-size-r1-opus-model/`。

## 一、核查

- 开工快照（目录 `research/prompts/m4-multi-inode-size-r1-snapshot/`，`crates-sha256.txt` 事后补记冻结副本与当前工作区的 `crates/singlefs-core/src/transaction.rs`）：`python3 scripts/agents/snapshot-tree.py m4-multi-inode-size-r1 --out <草稿目录>/mi/tree`：清单 3 份，倒推出 2 份、sha256 对上 2 份（`08-核心索引结构.md`、`04-fourth-txn.md`）；`.claude/kb/checks-owed.md` 倒推不出（快照之后别的会话改过工作区那一份，diff 里没有它的基底）。两份腿报告里引 `checks-owed.md` 的只有正推腿第 346 行一处 C618 的指路（不带行号以外的原文），人工核：C618 那一行在此刻工作区照旧在，它是「core 从只认第一个文件泛化到任意 inode」那一条，与引法相合。
- cite-check：`python3 scripts/agents/cite-check.py research/prompts/m4-multi-inode-size-r1-sonnet-output.md research/prompts/m4-multi-inode-size-r1-opus-output.md --root <上一条的树> --root /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --background research/prompts/_m4-multi-inode-size-r1-background.md`，末行「✓ 核了 98 处引文、114 处裸行号，对上 93 处，没判 119 处（其中裸行号没有引文可比 104 处；逐处列在上面）」，没有对不上的。没判的里主 agent 在冻结副本上逐处抽核了判决要用的几行：`transaction.rs:10494`–`:10498`（别的文件 `data_units_existing` 取 0）、`truncate.rs:297`（换下角色写死 `ExtentLowerNode`）、`recovery.rs:3853`（`TailOfAFile::NoHoleAfterTheLastDataUnit,`）、`walk.rs:13057`（`(None | Some(DataUnitsDeclaredByTheUpperLeafEntry::NoDataUnit), _) => (`）、`crates/mutations.tsv:4389`（「m4 截断截甲 条款没写的那一格：第一个文件之外的 inode 改成别的 size 不拒」，锚在 `crates/singlefs-core/src/transaction.rs`），逐字相同。
- 攻方模型复跑：拷到 `~/.local/share/singlefs/work/m4-multi-inode-size-r1-verify/opus-model/`，四份文件 sha256 与报告第 12–15 行逐份相同；复跑命令与结果见第六节。
- 正推腿的探针没有模型目录，留在它的草稿目录；它量的数（读侧矩阵、计划 11/11、回退 1 条）只当线索，判决里引到的都另由攻方模型或读码撑住。

## 二、逐格判定

| 格 | 两条腿 | 判定 |
|---|---|---|
| S2 别的文件的洞 | 正推推荐「粘住的标签 0」：别的 inode 变长到 size > 0 而零单元时写标签 0 条目，条目留到 unlink 最后一批；另加 checker 新判「别的文件没有上段条目 ⇒ size 0」。攻方打中（不分辨臂）：同一份「size > 0、没有条目」镜像今天挂载态读者拒、`rebuild_version` 与 `mount_writable` 收、CPU 池级 checker 109 条全不红（模型 `out/probe.txt` 第 8–20 行） | **采正推的形并收进攻方那一笔账**：别的 inode 照第一个文件的标签 0、3、4（用户 2026-10-16「加标签声明洞」、2026-10-24「一律写标签 0」推到任意 inode）；「有记录、没有条目、size > 0」在挂载态读者、三处冷读者（`extents_of_the_first_file` 对别的文件那两道门与第 3853 行写死的尾部）、CPU I-4.6 的「没有条目」那一支、GPU 检查内核与恢复内核上给同一个判法：读者拒、checker 红，配必红世界。两条腿在这一格不矛盾：攻方的账就是正推那条新判 |
| S3 截断续做与计划 | 两条腿都现查到同一组只认第一个文件的地方：`publish_size_change` 里别的文件单元数取 0（10494）、续做认领 `names_the_live_first_file`（10604）与单元数取 `previous.data_pointers`（10616）、`TruncationBatch` 不带号（5826）、K3 计划换下的是第一个文件的下段（`truncate.rs:297`）；攻方在只删拒绝那一支的副本上量到 H2–H5 四段「发成」而 checker 红 I-9.38 / I-4.6；正推量到 unlink 的批计划在不带重写单元的批上与截断批逐节点相同、在意图那次漏重写罩着单元 k 的叶 | **采**：抽出下段计划（入参是这个文件自己的下段节点表），截断两种批带 inode 号，单元数、尾单元重写的对象与角色、旧净荷来源、映射 key、续做认领全按条目点名的 inode；意图那次罩着被重写单元的叶到根恒重写（K9 叶边界对任意 inode 成立） |
| S4 K1 留不留 | 两条腿一致：签发点唯一、签发前断言队列空、回退先删现行全部截断条目再放 R_old 的一条，合不出两条。攻方另打中：留 K1 而某个别的号的条目续不下去时，「先走空」让全池每一次用户改动都停住（`out/probe.txt` 第 42、56 行） | **留**，条款文字不改（K1 本来没限 inode）。攻方那一条副作用由 S3 的「续做按号认领」收掉：它是 K1 能留的前提，与 S3 同批落地 |
| S5 对拍对象 | 两条腿一致：RefFS 没有文件身份；推荐只对模型。攻方打中三样今天模型判不了：胶水要求单元接起来恰好等于 size（洞与截断中间态解不开）、截断字段只比第一个文件、崩溃读回的允许集合只装第一个文件；正推另列模型对快照、整理、截断三臂在别的文件带数据时拒答，翻译层截断支不分 inode | **只对模型**，RefFS 多文件归里程碑十的一·3。模型这一侧是前提不是附带：胶水按标签与截断条目读别的文件、截断字段按 inode 比、崩溃读回按 inode 给允许集合、快照与整理与截断三臂对别的文件带数据作答；翻译层对 inode ≠ 1 且写了盘的截断停下 |
| S6 拒绝边界 | 两条腿一致删成员；攻方打中「只断言发成」的正例判别力为零，换上的正例必须在第 4389 行那个变异体上红；正推推荐 β 不管改 size | **删成员**；删完第一个文件之后才建的 inode 改 size 照任意活 inode 办（用户的射程「任意活 inode」把它包括在内；β 管的是写数据，改 size 不经 β 的门）；新正例在第 4389 行的变异体上必须红（断言四个读者收、读回等于期望、checker 零红），第 4456、4462、4469 三行换锚或随成员删、各有新行顶上（正推 6.4 节那张清单） |
| S7 只认第一个文件的清单 | 两条腿一致：`grep FIRST_INODE_NUMBER` 漏掉 K3 计划本体、`TruncationBatch`、`previous.data_pointers`、冷恢复尾部那一类不带常量的耦合；正推给三层判法（行为层 6 条历史、grep 层、范围层）与必须一起改的 9 组清单 | **采正推的 7.2 节清单与 7.1 节三层判法**，再加攻方的判法：以只删拒绝那一支的副本上 H1–H5 五段当会红的世界，清单改完五段都变成合法镜像、四个读者都收 |

正推腿第 28 行「core 里这道门今天没有变异行钉着」不对：`crates/mutations.tsv:4389` 锚在 `crates/singlefs-core/src/transaction.rs` 这道门上（攻方报告 S6 节）。正推只按 `is_the_first_file` 搜，没按门的原文搜。

## 三、打中之后的四句（S2）

1. 分不分辨臂：不分辨。病根是四处读法对同一形各判各的，选哪一臂都要先让它们判同一个结论，这一笔账照第二节 S2 那一格收掉。
2. 被判的系统看不看得到：看得到，记录的 size 与上段有没有条目都在盘上。
3. 满足的分句：正文 S2「写或不写，三处读者……、池级 checker 与 GPU 恢复内核各怎么判」。
4. 改法在打中的格上还中不中：选标签 0 之后，CPU I-4.6 与 GPU 检查内核的「没有条目」那一支要加「size = 0」，不加的话 H1 那份镜像照样绿；这一句进实现的验收。

## 四、交用户

| 项 | 形态 | 被攻过几轮 | 去向 |
|---|---|---|---|
| 别的 inode 写标签 0（粘住），checker 加「没有条目 ⇒ size 0」 | 正推提、攻方的账同形 | 零轮（攻方没读到这一形，只攻了今天的实现） | 写回 D8 已定项 14 不限第一个文件，进实现 |
| 抽出下段计划、批带 inode | 正推提 | 零轮 | 进实现 |
| 只对模型，模型补四样 | 两腿同 | 零轮 | 进实现 |
| 删成员、β 不管改 size | 正推提，主 agent 按用户射程判 | 零轮 | 进实现；用户若认为删完第一个文件之后才建的 inode 改 size 要拒，另起一个成员 |
| S7 清单与三层判法 | 正推提，攻方补判法 | 零轮 | 进实现的验收 |

## 五、写回与后续

- 写回（kb-scribe，等这一批推论轮判完一起写）：D8 已定项 5「截断入口边界」那一句改成任意活 inode；已定项 14 标签 0 定义句与射程不限第一个文件；D8 已定项 16 射程「不带 truncate」那一句（正推第 437 行指出过期）随同改；`.claude/kb/layout/04-fourth-txn.md` 标签那一行；`checks-owed.md` 新立一条「挂载态读者、冷读者、池级 checker 与 GPU 两个内核对『有记录、没有条目、size > 0』判同一个结论」的检查，落地后销；里程碑四待优化表那一行写成「判决 m4-multi-inode-size-r1」。
- 实现（implementation-writer，等 GPU 垃圾回收第 39 批交回、`transaction.rs` 与 `mount.rs` 放出来）：第二节 S2、S3、S5、S6 的改法，验收按 S7 的三层判法；新变异行照正推 6.4 节；GPU 两个内核与 CPU 对拍、别的 inode 的截断崩溃流（照六·3 的三条）归 checker 档，提交时跑。
- 要不要加别的 inode 的截断崩溃流：照「崩溃点测试准确率和正确率优先」，加。

## 六、复跑

`SINGLEFS_RUN_SCOPE=… bash scripts/hosts/run-with-memory-cap.sh 3G bash ~/.local/share/singlefs/work/m4-multi-inode-size-r1-verify/opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --work ~/.local/share/singlefs/work/m4-multi-inode-size-r1-verify/opus-rerun`，退 0，末行「name=rerun_compare same_as_model_out=yes lines=73」：攻方 `out/probe.txt` 73 行在拷贝里逐行复出。复跑脚本跑完自己删了副本与编译目录；模型拷贝留到这一题不再开下一轮时删。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 5 | 支撑 | 2026-10-26 改了：截断入口边界推广到任意活 inode，并进写回批 |
| D8（核心索引结构） 已定项 14 | 支撑 | 2026-10-26 改了：标签 0 不限第一个文件，并进写回批 |
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-10-26 不受影响：射程里「不带 truncate」那一句是过期现状，随写回改文字，条款不变 |
