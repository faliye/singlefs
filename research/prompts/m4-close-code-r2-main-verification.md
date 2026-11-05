# m4-close-code-r2 判决：里程碑四收口修补二批的代码轮（主 agent，2026-11-03，第二轮）

正文 `research/prompts/_m4-close-code-r2-body.md`，diff `research/prompts/_m4-close-code-r2-diff.md`，背景材料 `research/prompts/_m4-close-code-r2-background.md`。腿：正推（Sonnet）`research/prompts/m4-close-code-r2-sonnet-output.md`，攻方（Opus）`research/prompts/m4-close-code-r2-opus-output.md`（模型 `m4-close-code-r2-opus-model/`），攻方第二次抽样 `research/prompts/m4-close-code-r2-opus-s2-output.md`（模型 `m4-close-code-r2-opus-s2-model/`）。

被判的产品代码（冻结副本 `~/.local/share/singlefs/work/m4-close-code-r2/repo/`，开工快照 `research/prompts/m4-close-code-r2-snapshot/crates-sha256.txt`）：`crates/singlefs-core/src/admission.rs`、`crates/singlefs-core/src/container_tree.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/verified_model.rs`、`crates/singlefs-replay-prefix/src/gc_ring_interval_proofs.rs`；`crates/singlefs-core/src/mounted_session.rs` 只读、这一轮没改。

## 一、核查

- 开工快照：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-close-code-r2-judge -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py m4-close-code-r2 --out /home/fy5090/.local/share/singlefs/work/m4-close-code-r2-judge/m4-close-code-r2-tree --repository /home/fy5090/code/singlefs`（任务 5602）：「快照 m4-close-code-r2：清单点名 9 份，倒推出 9 份，sha256 对上 9 份」，9 份都取工作区此刻那一份（腿跑着时没人改它们）。
- cite-check：`python3 scripts/agents/cite-check.py research/prompts/m4-close-code-r2-sonnet-output.md research/prompts/m4-close-code-r2-opus-output.md research/prompts/m4-close-code-r2-opus-s2-output.md --root /home/fy5090/.local/share/singlefs/work/m4-close-code-r2-judge/m4-close-code-r2-tree --background research/prompts/_m4-close-code-r2-background.md`：「✓ 核了 168 处引文、51 处裸行号，对上 127 处，没判 92 处」（倒推的树只有快照里那 9 份，别的文件的引文判不了）；换成不带 `--root`（仓根）再跑：「✓ 核了 168 处引文、51 处裸行号，对上 167 处，没判 52 处（其中裸行号没有引文可比 51 处）」，没有一处对不上；换成冻结副本：「对上 144 处，没判 75 处」，同样没有对不上的。
- 复跑（主 agent 经服务交，都不是重型）：
  - 攻方探针（任务 5531，在攻方交接的副本里 `cargo test -p singlefs-core --lib inode_record_lookup_by_leaf_order_over_overlapping_leaves_probe`）：「test result: FAILED. 1 passed; 3 failed」，红的三条是 `overlapping_leaves_keep_the_maintained_counts_equal_to_the_recount`、`overlapping_leaves_agree_on_the_lookup`、`overlapping_leaves_find_the_record_the_final_batch_removes`，与攻方手推相同；
  - 攻方模型复跑（任务 5532）末行「name=rerun_attack_m4_close_code_r2 cargo_exit=101 control_ok=1 overlap_failed=3 probe_lines=15」；
  - 第二次抽样的测试（任务 5576，在它交接的副本里 `cargo test -p singlefs-harness --test inode_record_counts_on_a_writable_mount_of_overlapping_inode_leaves`）：「test result: ok. 2 passed; 0 failed」（两条用例按「坏镜像让计数漂、最后一批 panic」「合法两片叶删左片记录计数照对」写，绿即打中站住）；
  - 第二次抽样的模型复跑（任务 5577）末行「name=attack_s2_summary all_three_as_expected=yes」（「把按叶序折半找改坏成恒取最右那片」时，现有那份计数测试照绿「4 passed」、合法两片叶那条红）。
- 核不动的：无。

## 二、逐格判定

| 格 | 判定 | 依据 |
|---|---|---|
| T1 开片原地改与上一版只记节点数 | 成立 | 正推判成立（开片的 `ensures` 还对着没改的规格，`container_tree.rs:2374`、`:431`；上一版只被问层数、某位置有没有节点、根在哪，改前也只问这三样）；攻方两次都没打中 |
| T2 求祖先从最右端往左 | 成立 | 正推判成立（三棵容器小树只增不减，重写清单只在 `container_tree.rs:3661` 造出，前提破坏时 `container_tree.rs:2786` 的 `assert!` 当场 panic）；攻方两次都没打中 |
| T3 孩子区间累加与按位置取指针 | 成立 | 正推判成立（三处造 `TransactionOutput` 的地方各记一次根单元下标，`units` 次序之外没人动，断言合 `.claude/singlefs-ai-sop/rules/code-discipline.md`「不变量被破坏是 bug，用断言」）；攻方两次都没打中 |
| T4 准入不遍历 inode 树 | 合法镜像上成立；坏镜像上打中 | 正推按代码判成立（数与 inode 树吃同一份写入、同一个上一版）。攻方两次都打中同一处：盘上两片 inode 叶交叠（违反 I-9.12）时，可写挂载重建只判 `recovery.rs:6641` 那一句下界、不判交叠，镜像收得下；新加的按叶序折半找（`transaction.rs:4189`）漏找，写者删者逐片找得到，维护的计数偏高、落盘的记账标签 6 偏高，unlink 最后一批在 `transaction.rs:8395` 的 `expect` 上 panic。改前每次逐片重数，不漂、不 panic。任务 5531「1 passed; 3 failed」与任务 5576「2 passed」现量坐实 |
| T4 方法 | 打中 | 正推与第二次抽样都报：`inode_record_counts_kept_with_each_version.rs` 只用两组比重、每段至多 30 步、每步建 1 个 inode，inode 树始终一片叶，跨片查找那一半没有样本；截断、建删快照、整理、部分写、多 inode 几条路径也没有直接取样。任务 5577 现量：把查找改坏成「恒取最右那片」，这份测试照绿 |
| T5 G7 | 成立 | 正推判成立、攻方两次都没打中；G7 在 core 一侧靠攒批会话先排空待发再起整理（API 纪律），core 里没有一行运行期比较——记进第五节 |
| T5 Y5 | 打中（方法） | 三份报告一致：新补进 `root_ring_is_shaped` 的两条只有同批写的见证在用（必红世界第 453、454 行只证见证本身会红），`RootRingModel` 在产品代码里没有使用者，「在 core 的值上成立」没有代码或证明罩着；`roots` 取环里实际的根还是全部历史、要不要先过实例表、丢盘后 F_生效 可高于某块盘上最新有效根（`mount.rs:3407`），模型都没写 |

## 三、交给下一批的实现员（按文件聚簇）

腿提的改法都只在腿自己的模型上量过或推过，被攻过零轮。

- **读回判写者前提那一簇**（`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/mount.rs`，连同 T3.1 计数加搬迁修复同改 `transaction.rs`，并成一个实现员）：T4 坏镜像那一处取攻方改法甲——可写挂载重建（与冷恢复读 inode 树的那一处）照 I-9.12 判全部 inode 叶记录连起来严格升序，交叠就拒，读者照写者的 Verus 前提（`inode_tree.rs:1033`、`:325`）拒；先写会红的用例（第二次抽样那份坏镜像用例改成期望「挂载拒」），每处改法留变异行。改法乙（找法改成逐片找）与丙（只把 `expect` 换成类型化错误）不采：乙修查找不修读回收下坏镜像，丙只修 panic。
- **T4 取样那一簇**（`crates/singlefs-harness/tests/inode_record_counts_kept_with_each_version.rs`，只改测试代码）：比重加上截断、建删快照、整理、部分写、多 inode 那几组，段长或每步建的 inode 数放大到 inode 树长出多片叶（至少两片），改坏成「恒取最右那片」时要红；`:243` 那句写 `BROAD` 抽快照与整理的文档注释照 `crates/singlefs-harness/src/history.rs:1236` 的定义改对。
- 测试注释（`crates/singlefs-core/src/container_tree.rs:3770`，`a_full_root_grows_...` 那条测试注释写错了位置，断言本身对）：并进第一簇顺手改。
- 准入之前另三处随总数涨的步骤（第四节第一行，用户定「现在就改」）：`transaction.rs:6923` 起的 `mapped_units_of_a_container_tree`、`container_tree.rs:2926` 走全部节点那一处、`transaction.rs:4205` 的 `inode_leaf_container_contents`，并进第一簇，改成只随这一次碰到的容器、节点与叶数（加树高）涨的写法；每处一条钉代价的测试（形状做成从最左读起就溢出或越界，与 T2 那条同法）与变异行。

## 四、交用户

| 事 | 出处 | 被攻过几轮 |
|---|---|---|
| 准入之前还有三处随总数涨的步骤，不在这一轮五格、也不在 E199（发布照抄上一版的代价） 的模型里：`transaction.rs:6923` 起 `mapped_units_of_a_container_tree` 每轮走全部容器，`container_tree.rs:2926` 走全部节点，`transaction.rs:4205` `inode_leaf_container_contents` 每轮拷一遍全部 inode 叶 | 正推报告「要主 agent 现查的发现」第 2 条；攻方报告「没人看过的地方」 | 零轮 |
| 第三节两簇的改法（T4 取改法甲） | 腿报告各节 | 零轮 |

## 五、后续

- Y5 桥没接（core 的值没有一条路装进 `RootRingModel`）与 G7 只靠 API 纪律两件，延后：登进 `.claude/kb/checks-owed.md` 一行，随这一批收尾的 kb 写回写（写回之前不进验证段）。
- 背景材料三处更正（正推报告第 4 条），不改冻结的材料，记在这里：正文写的「D16（发布语义） 已定项 10 的统计量那一句」不对，管「准入之前」的是 D8（核心索引结构） 已定项 16 的射程（`.claude/kb/decisions/08-核心索引结构.md:516`）；`run_compaction_in_the_background` 在冻结副本里有两个，T5 说的是攒批会话那一个（`mounted_session.rs:5565`），不是裸会话那一个（`:2401`）；`container_tree.rs:3770` 那条测试注释写错了位置（并进第三节第一簇）。
- 第三节两簇文件互不相交，同一批派；T3.1 照用户定案「计数加搬迁修复」并进第一簇（同改 `transaction.rs`）。
- 这一轮之后代码轮不再开第三轮：T1–T3 与 G7 两次没打中、成立；T4 与 Y5 打中的改法写回之后，照验证段第 4 步重验，产品代码改动随下一批代码轮攻。

## 用户定案

- 第四节第一行（准入之前另三处随总数涨的步骤）：用户在弹窗里选「现在就改」——并进第三节第一簇，与 T4、T3.1 同一个实现员改成不随总数涨的写法；代价没量过，改法被攻过零轮。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 16 | 支撑 | 2026-11-03 不受影响：准入增量维护照射程「只管准入之前」做了；T4 坏镜像那一处是读者没照 I-9.12 判交叠，改读者，不改条款；准入之前另三处随总数涨的步骤交用户 |
| D13（验证路线） 已定项 14 | 不影响 | 2026-11-03 不受影响：Y5 桥没接是证明与实现之间的接法缺口，条款不改，记欠账 |
| D16（发布语义） 已定项 10 | 不影响 | 2026-11-03 不受影响：这一轮正文误引，判的格里没有用到它 |
