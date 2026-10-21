# m4-publish-lookup-code-r1：里程碑四七·6 发布路径按身份查，代码轮第一轮判决（2026-10-20）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 -->

被判的产品代码：`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/mounted_session.rs`（七·6 补丁，`research/prompts/_m4-publish-lookup-code-r1-diff.md`）。测试代码 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs` 与 `crates/mutations.tsv` 末尾九行一并判 T4。

## 一、核查

| 核了什么 | 怎么核 | 结果 |
|---|---|---|
| 两条腿的引文 | `python3 scripts/agents/cite-check.py research/prompts/m4-publish-lookup-code-r1-sonnet-output.md research/prompts/m4-publish-lookup-code-r1-opus-output.md --root ~/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo --root /home/fy5090/code/singlefs --background research/prompts/_m4-publish-lookup-code-r1-background.md` | 「✓ 核了 182 处引文、20 处裸行号，对上 182 处，没判 20 处（其中裸行号没有引文可比 17 处；逐处列在上面）」；没判的另 3 处是只写文件名的 `transaction.rs:3717`、`transaction.rs:3227`（两处），主 agent 在冻结副本里读：第 3227 行与第 3717 行都是 `.expect(EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION)`，与攻方报告第 92、210 行说的「两条查法共用一个 panic 文案」相符 |
| 开工快照 | `sha256sum -c research/prompts/m4-publish-lookup-code-r1-snapshot/sha256sums.txt` | `transaction.rs`、`crates/mutations.tsv` 对不上，其余四份 OK。两份都被别的会话在这一轮里改过，倒推回去都与快照相同：`mutations.tsv` 用里程碑四预开发的会话打补丁前拷的原样（sha256 `99dc56e6387ada5b7c96ba36f0cd5d0f734eb0fc7e2b528f719e1241c5b0c63a`，与快照同），`transaction.rs` 把里程碑三的会话系统配置落地那份 `crates.patch` 只对它反向打回，sha256 前缀 `4f27be528003e1bf9abb` 与快照同。证据与改动清单在 `research/prompts/m4-publish-lookup-code-r1-snapshot/reversal/`。两条腿只引冻结副本里的行，不受这两处影响 |
| 攻方的复跑数 | 模型目录拷到 `~/.local/share/singlefs/work/m4-publish-lookup-code-r1-main-check/model/`，`bash scripts/hosts/run-with-memory-cap.sh 8G bash <那份>/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m4-publish-lookup-code-r1/repo --work <草稿>/run --model <那份>`，退 0 | 原样输出：`name=differential kind=random lines=2798 before_vs_after=identical`、`name=differential kind=scripted lines=216 before_vs_after=identical`、`name=t2 M4T2 total keys_compared=14352 both_panicked=942 found=9808`、`name=half_mapping_mutant 4 tests_ok; batches_published=13 images_sha256=763f4477ce2f2d78947048a8f8d37327601b302c81bd8eb0fdaa6596427d89e4 batches_published=39 images_sha256=f065921a3fc7507c1d89d4fba41acc5fe7ef5f6c9a302f48b4244a86e9998a5c`、`name=growth_row_nine growth_in_percent=362 test result: ok. 1 passed`。与攻方 `out/rerun-expected.txt` 逐项对得上 |
| 正推腿的数 | 正推报告 T4 的增长读数（「整理读版本退回逐 key 查映射」359%、357%；调用点整个还原成改前 379%、377%；补丁后 310%、287%）没有单独复跑 | 与攻方复跑的 362% 同一侧（都低于 400%），两条腿各自独立量到同一结论；核不动的不当核过，判决只引攻方复跑的那一个数 |

## 二、判定与去向

| 格 | 正推 | 攻方 | 判定 |
|---|---|---|---|
| T1 索引与线性查找是不是同一项 | 成立 | 没打中（223 段随机历史、23 段固定前缀，改前改后写流与内存版本摘要逐行相同） | 成立。补丁在取样的历史上与改前逐行相同；「没打中」只抽一次样，靠正推的读码（取最先一项、身份类型的 `Ord` 派生、建了索引之后改表编不过）与复跑的差分两条路一起撑 |
| T2 整理读版本一遍走叶与逐 key 查 | 成立 | 没打中（14352 次比对逐个相同，942 次两边一起 panic 文案相同） | 成立 |
| T3 照抄三处查不到时的行为 | 成立（另报 `transaction.rs:5652` 还剩一处 `mapping_key_carried_from` 线性调用，不在点名五处） | 没打中 | 成立。第 5652 行那一处并进下一批 |
| T4 测试与变异判得出 | 部分不成立 | 打中两处 | **打中，两处都坐实**：① 整理读版本只读半张映射（`.step_by(2)`），七·6 的四条用例全过、镜像摘要已变（复跑原样见第一节），用例只打 `images_sha256` 不断言；② 增长慢档用例门槛 400% 在登记的 debug 命令下分不出改前改后，「七·6 增长：整理读版本退回逐 key 查映射」那一行变异不红（复跑 362%）。两处都是测试的判别力不够，不是补丁的行为反例 |
| T5 有没有改到「行为不变」之外 | 成立 | 没打中 | 成立 |

## 三、交给下一批的实现员（按文件聚簇）

都落在测试代码与变异表，外加 `transaction.rs` 一处查法：

1. **T4 ①**：等价用例要能判出整理读版本读漏、读错。做法由实现员在两条里选并写依据：在用例里拿同一版本逐 key 走 `central_mapping_locations_in_the_version` 当参照，与整理实际用到的映射条目逐项比（不改产品可见性最好；要改可见性就写明改了哪一处）；或把两档镜像摘要钉成常量断言（攻方甲，代价是格式一改常量就要重钉）。验收：攻方那两条变异（`.step_by(2)` 与 `.skip(1)`，攻方报告 T4 那一节）加进 `crates/mutations.tsv`，点名的用例红。
2. **T4 ②**：增长判据不再量 CPU 时间。改成数次数（攻方丙：每批整理解了几片叶、按身份查了几次），或另一种不随负载与构建档漂移的判法；要在产品里加计数的，写明加在哪、只在哪个特性或测试构建下开。验收：两行增长变异都红，未变异的用例绿，判据不读挂钟、不读 CPU 时间。
3. **T3 附带**：`transaction.rs:5652`（冻结副本行号）那一处 `mapping_key_carried_from` 线性调用改走同一个索引，带一条会红的变异。

增长慢档用例与「按身份查」七行变异的其余部分不动。

## 四、复跑与核不动的

- 正推腿的增长读数没有单独复跑，见第一节。
- 「没打中」各格只抽一次样（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」）。这一轮把 T1、T2、T3、T5 判成立，靠的是两条腿各自的路子：正推读码，攻方做差分。两条路不共用一段代码、一次采样，不是只凭攻方一次「没打中」。
- 崩溃状态没有逐个枚举（攻方「限度」一节：两边写流逐条相同，崩溃状态集合由写流定，这一步是推的）；崩溃轻型在验证段另跑。

## 五、交用户

没有要用户定的岔路。改法甲到戊都是攻方在自己模型上提的，被攻过零轮；这一轮按第三节交实现员，改完的测试改动按 `.claude/rules/implementation-workflow.md` 只改测试代码不开代码轮，`transaction.rs` 那一处查法改动随下一轮代码轮再攻。

## 回看决策

不涉及决策：这一轮判的是七·6 的查法改成索引、行为不变，没有改动或确立 `.claude/kb/decisions/` 下任何一条的说法；压着的条款是 `.claude/kb/milestone/04-fourth-txn.md` 七·6 那一行与 `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」。
