# m3-formal-code-b30-r1 云端攻方（Opus）报告（2026-10-05）

被攻的是第 30 批三份补丁与主 agent 两处改动（背景材料 `research/prompts/_m3-formal-code-b30-r1-background.md` 第三节 M1–M4）。代码引文写冻结副本 `/tmp/claude-1000/m3-formal-code-b30-r1/tree/` 里的 `crates/…` 路径与行号；kb 引文写 kb 文件自己的行号。副本上量出的数只在攻方副本上成立，不算入库装置上的数。

## 复跑

模型目录 `research/prompts/m3-formal-code-b30-r1-opus-model/`（tests/ 是探针与接在入库用例副本后面的几段，tools/ 是脚本，out/ 是这一轮的原样输出）。在草稿目录里复跑：

```bash
cp -r research/prompts/m3-formal-code-b30-r1-opus-model <草稿>/model
FROZEN=/tmp/claude-1000/m3-formal-code-b30-r1/tree bash <草稿>/model/tools/rerun.sh <草稿>/work all
# 或逐步：prep 之后任选 m1-* / m2-* / m3-pins（步名见 tools/rerun.sh 的 case），例：
bash <草稿>/model/tools/rerun.sh <草稿>/work prep m2-units m2-alg m3-pins
```

- `all` 合计约 30 分钟；每一步单独跑都在前台 4 分钟以内（最长的是 m2-f1、m2-f2，各约 2.5 分钟，m1-broad-retain / m1-broad-drop 各约 2 分钟）。cargo 一律经 `tools/cargo_capped.sh`（`run-with-memory-cap.sh 8G` 加 `capped.sh 7`），只编、跑 `singlefs-harness` 的测试目标；`m2-legal-*`、`m2-arm`、`m2-f1`、`m2-f2` 用 `--release`。
- 比对：`<草稿>/work/out/` 与模型目录 `out/` 同名文件里的 `SWEEP`、`B30*`、`test result` 行。`out/e1-probe-arm-batch1-summary.txt` 是 `m2-arm` 那一步日志的汇总（原日志 250 KB，只留汇总）。
- 我在草稿目录 `rerun-check/` 里照它跑过 `prep m3-pins m2-alg m2-units m1-b3`，输出与 `out/` 同（`B30PIN`、`B30CRC`、`B30ALG`、`B30UNIT slot=50264`、`B30TIME` 各行逐字同）；其余步没有整套重跑第二遍。

模型目录逐文件 sha256（`cd research/prompts/m3-formal-code-b30-r1-opus-model && find . -type f | sort | xargs sha256sum`）：

```text
a8f771d261940f226181998872e62dfbe3a0e20894b82ab3ad000cf6ae11f84b  ./out/alg-probe.log
5b39c9127e141744058756257504565ce5da27c053d1b2b6bccc678db0008c72  ./out/alg-probe-with-f2.log
3aa753bb8593be7bc29ba4d6601958d4a2df23a21ae5031ed6ee4b76de66d4d6  ./out/baseline-b3.log
5cb881bef421ab9671e57c0d8651b3934ad680bfbb0defd25667a24bc42d815c  ./out/baseline-b3.txt
265a77e900003e0e8786149e69cfb95277afb5f225f10d4aa6bb88d21b26cb33  ./out/baseline.log
160d45d5217559b95c12a702bda31d7a5716619dfd16f8733288fe550b938ae7  ./out/baseline.txt
5062d58fff868baec60f13d15d9cafd810fd91118ca362984e76ebd603b055cf  ./out/drop_only_third.log
5edcbbee5c923719771c7a0f94af4862dcaa9211db23b31722e1eb7854a6018b  ./out/drop_only_third.txt
afe3413d2c2e7dc8bf299f4090b0bf672a4f7b7df5df2a7b6aa8ea0826091a5b  ./out/drop_up_to_fourth-broad.log
64ccdd32d673896ae60792356556b690226047266ee08e1787a79916be6610c1  ./out/drop_up_to_fourth-broad.txt
7ffb2ea65d410a5eba243e8c29291a2b085d2d7ce745195b585ac931a11a811c  ./out/drop_up_to_fourth-lib.log
dd9de46ba2980c38e0d6193cb08ca1d1f4dcd35064d42a8a2f34506f702ceb7f  ./out/drop_up_to_fourth-lib.txt
6a52c78c960aeb8bbd7566f15039cdd8521bc55b97f61c03a6de48d7ae249b4a  ./out/drop_up_to_fourth.log
3ad75b7dc180b6a5ff2acc08f08a097882f92464824efdb37757284d430ae104  ./out/drop_up_to_fourth.txt
938346a0675bb7bb03cbd6b8b1256ce297929eb9ae06d26077f463eca421d98a  ./out/e1-i54-legal-batch1.log
d8e4feb053b34550a9e40bf3e86696ba0b6253afce262746b36edbe67b9aa6a1  ./out/e1-i54-legal-batch1-with-f1.log
e613f7740d9ac5e824d1fec50182b7b49c43232526af384260a8742f4bace625  ./out/e1-i54-legal-batch1-with-f2.log
a58cf2d7433cb676ef306feffe85e0487a92463eee4b41e1fbd9e4790570dbdb  ./out/e1-i54-legal-batch2.log
b1d8967bf15d832dbd293cbf6ad16c46c786f75be4971d9eb480f566877a8fab  ./out/e1-probe-arm-batch1-summary.txt
b6256f8b687ddbe55042e82571cbb7562e4df5a336b294b913a8bf78c2967767  ./out/glue_cap_at_fourth-b3.log
c54da71c546babb122a301f63b262807609c1cfad2c4a09c97caec2620b9543b  ./out/glue_cap_at_fourth-b3.txt
e4e55d348cfa7db178b35d9ba894ec4d6fa3defb0d3eb35ae5d87189e8a26421  ./out/glue_cap_at_fourth-broad.log
47bc385f23f19e458a20b4e14bba0cdb4b4ad544395c87df2f1bc8f9bd6f46b8  ./out/glue_cap_at_fourth-broad.txt
521b09474f5dacbc95cab7d6df02862445aeef92aafc11b6174284898726bcf3  ./out/glue_cap_at_fourth.log
ac5062391c8ddf7f27df0b77b9c28e6cbd7cd1a53589f05c88b4f80c3e57e465  ./out/glue_cap_at_fourth-random.log
eeca5950ef97d319e5635ca60c32287ce69bda51dbb42d830b89352f78a0a30c  ./out/glue_cap_at_fourth-random.txt
4d565f31100353419c657d1e9eb6a57ad8de583275b7ffef3158842c7bd25807  ./out/glue_cap_at_fourth.txt
78e6b455a17bfcd70c0891909415f7d8bc1fc3c75bfe05a953ec2f3834517c2b  ./out/i54-units-and-time.log
035fa942eaaff04178f6f9b14276ca747813debc7864c50d528a4ac6f7bc8c6c  ./out/i54-units-with-f1.log
d825dafbf7bab60ea1056332decaee03f5032b689a32214a05fbca0ebef05935  ./out/image-crc.log
a7c62763c9caf0d394ba51f98bd174e409c3f4ecf5beeaa95deb15b32a8f4eaf  ./out/known-bad-with-f1.log
2a3c8b2ce9da42012784716c2f89fe00df11ad6784f8cdd91655366010eb236b  ./out/kx1_cap_at_third-b3.log
c54da71c546babb122a301f63b262807609c1cfad2c4a09c97caec2620b9543b  ./out/kx1_cap_at_third-b3.txt
6af954228e4ebf6195899becdc9b899547177fb464503ca7d339a5dde4926197  ./out/kx1_cap_at_third.log
2383c8feedaec468eab6a4a38255ed6ea200ac46e2fb09e3423447fbff1e61fd  ./out/kx1_cap_at_third.txt
348e026b9723df77e52e1e2add42b59fce89ff48282037d91ea039500da90a14  ./out/kx2_cap_at_third.log
86c22800f2a549c0496d61f48718894cf374f9d477704b70d7380f13b122734e  ./out/kx2_cap_at_third.txt
95953f1819f44f7b1ede23ace87499622b99fcc05fc1cd459fe4b490f3500585  ./out/model_cap_at_fourth-lib.log
4ccbc968876fa0773cb2168cba1d6c792819a2944cbc9770dbd697a7ab0738b3  ./out/model_cap_at_fourth-lib.txt
a163fbb63d3d2ff679897eb353845525cb6873d110750260c35c4e9915e1542a  ./out/model_cap_at_fourth.log
914a8fa9394662e46d0f2f31454d957b7d8128e593bef18d8bee1c67e7832545  ./out/model_cap_at_fourth.txt
7dd91778ec85f40f917566c31590e1f8b3623bc18ef0aa966cc099c256773b76  ./out/retain_only_second.log
6ce325bf2a5729e7478323506c495b2eb938fe91dbf848b6aafbad7ea9cef164  ./out/retain_only_second.txt
b13728e5056fe9df9734d8c2a01d18678b3bc70bb309436c787ec28c572031dd  ./out/retain_up_to_third-broad.log
0d3e44fd3a821d445a47de550aac6304ddcee43b483c964d840a5625fa00f6aa  ./out/retain_up_to_third-broad.txt
6399fffcb7b27c1239984e0c3d8805ea59b22911136dc8eacc220033dc5a408e  ./out/retain_up_to_third-lib.log
92dc786dc86c27cbd18cdedb8cb9d0c023050b15c2a6858372741c3fb98687ba  ./out/retain_up_to_third-lib.txt
71c96f087bc15fbb69e04216e7e41a4bd22cd91b18e9a5deea7c5beebf0c2f80  ./out/retain_up_to_third.log
98106eb8214775345934f9f10c0d19cc33d1df88876fbb8edd69d4e01700c21a  ./out/retain_up_to_third.txt
dfaa309e2bfaaffa5b491ca60d70dce316021caaf4084e24e7d9cd97146f104f  ./tests/b30_opus_alg_append.rs
c7e53c797150bfa12f228480d7fd19b2bfd590abeddc1f96b4aff3568c5ef586  ./tests/b30_opus_e1_i54_legal_histories.rs
6227f9fc4d7c08e8e43b3608462ddc2b37fc526e872eadfaf0040542312bd555  ./tests/b30_opus_glue_leaf_role_is_injective.rs
0618bd0b334ab2c546fa5a2903625c645bc1e80f60851b7073f1388736f3a6ae  ./tests/b30_opus_i54_append.rs
25b8509015cdf491fc8df58d6a26f69b73a8f250e20cecf2cf6be2161fd857e6  ./tests/b30_opus_image_crc_append.rs
0eedd23385d585755d82bc939699008fcb78518de6d970fc893e18e8d21b1536  ./tests/b30_opus_leaf_container_sweep.rs
fcbdc9cd97f582b3319108fec599f1951493f5ae389f4c23e9fc9e8df5e6eb88  ./tests/b30_opus_time_threshold_slot_digests.rs
d716d03a44be1797ef8ad27646db5780a5775263495808ee806ff6ae1fa2e6da  ./tools/apply_variant.py
7bfc4ddeea108306000630f8b3301ca5bbd81bd027bfe505208055aa3f3c17ab  ./tools/cargo_capped.sh
6856a507b448331144e68b2e7483f1b59d3dddb28b9b8a5c98f769bcae86ae3a  ./tools/e1_probe_variant.py
72214eff773eff2537d70144cf15ee6f802d8699b2c7137e7165c8fbf97649b3  ./tools/i54_tree_table_fix.py
7db1ca02985b7e0e6ab1934710f44808008fba991b3c700fcaa00c45cadd9e60  ./tools/pointer_head_fix.py
e87daf3aed32a68f545f2a6eea13b8893f10d3da519d7d768dd72bef5c4d6049  ./tools/rerun.sh
cafcdfeb3b684a8606f6193846a71f3f3a520c53154a52659588c9ae7e41990a  ./tools/run_variant.sh
61790896a5ddbfe02b5e2eae07128da8edf630d1cd57b5d82c75cce93007d43e  ./tools/sum_legal.py
```

## 各格判定一览

| 格 | 判定 | 量过 / 推的 | 一句话 |
|---|---|---|---|
| M1 四个变体换取样点 | 没打中 | 量过 | 入库四个变体在 20 段放开用户动作的历史（回退目标第 1、2、3 新，回退后建 0、1、3、8 个）上，凡是能分辨它的取样点都红 |
| M1「第 k 片都对」罩到哪 | **打中**（同 b29 L1 那一形往后挪一片） | 量过 | 胶水封顶第 4 片、回退只取回第 2–3 片、回退只丢第 2–4 片三个变体在 30-1 三条、`--lib`、b27 两条、135 片追加、`unlink_in_histories_judged_by_the_model`、`random_histories`（不含 ignored）上全绿，只有建到第 5 片的取样点红 |
| M1 四行变异锚点互相掩护 | 没打中 | 推的（读表） | 同一原文、不同替换文的几行各自单独改坏、单独跑，不会互相掩护 |
| M2 E1 合法历史误红 | 没打中 | 量过 | 四组比重 168 段历史 7728 张每步镜像 I-2.4 违例 0；探针臂证这批里 1704 / 2604 张走到了墓碑树或谱系树容器子指针 |
| M2 走读跟随而不判 extent 偏移的指针 | 没打中 | 推的（逐调用点读码） | 池级 checker 六处 `read_referenced_unit` 加 `read_index_node`，GPU 走读内核对应几处，每条指针都判 extent 偏移；映射条目没有指针头 |
| M2 走读跟随而不判加密 / 压缩那几段 | **打中**（判据外的一格） | 量过（CPU），推的（GPU 走读） | 池级 checker 只判 MAC / nonce，不判算法类型、压缩算法码、压后长度；core 恢复拒的三种坏盘上池级 checker 一条都不红 |
| M2 GPU 恢复内核种类 ≥ 32、加密 / 压缩 8 处与 core | 没打中 | 推的（读码） | 字段、偏移、判法与 core `PointerHeadAsRead::judged` 逐格同，两边在同一盘面上都拒或都收 |
| M2 I-5.4 按版本在合法历史上多出部分没判 | 没打中 | 量过 | 同一批 7728 张镜像，新式部分没判 0 次 |
| M2 I-5.4 按版本在坏盘上放过 | **打中** | 量过 | 某一版的树表读不出时那一版整版不进评估，别的版本走到了叶就报成立；B 的树表、A 的树表、第 0 代树表各清零都报成立 |
| M3 core 写出、G1 期望、字段表同单位同值 | 没打中 | 量过 | 三处都是 5000 毫秒 |
| M3 全仓按秒解偏移 433 | 没打中 | 量过 + 读码 | crates/ 与 research/ 装置里没有读这 4 字节的解码者；池级 checker 与 core 恢复对这 4 字节取 5 种值判定逐字不变 |
| M3 三个新钉值的出处 | 没打中 | 量过 | 两个槽摘要与整盘 CRC 把 T_time 换回 5、重封之后各等于换钉前的旧值，差只在 [433, 435) 与校验和 4 字节 |
| M3 checker 档读码判不变的钉值 | **打中**（重取法分不开两种变化） | 推的 | 分片 golden 的重取法是整份取 `left`，30-2 若改了 `GOLDEN_TALLY` 的计数，会与 T_time 带来的计划哈希变化一起被吞进新钉值 |
| M4 变异行只改名 | 没打中 | 推的（读脚本） | 按条记录的行键是六段原样的 sha256，改名只让这一行重跑，不会对不上 |
| M4 字段表第 149 行与 D16 已定项 5 | **打中**（低，格式空白） | 推的 | 4 字节毫秒写不出 1 ms 以下与约 49.7 天以上，D16 定的是「不限制、越界只警告」，写不出的值怎么办两处都没写 |

## M1　30-1：第 4 片与回退用例

**做法**：攻方探针 `tests/b30_opus_leaf_container_sweep.rs`，从 `AfterFirstFile` 起 unlink 第一个文件、建 `base` 个 inode，可选回退到根环第 `index` 新那条根、再建 `after` 个。固定的只有前缀（`base`）与故障（变体），用户动作（`index`、`after`）放开扫：A 建到第 5 片（936）；B 第三片被重写过（471）× index {1, 2, 3} × after {0, 1, 3, 8} 取 7 格；C 没有第四片（701）取 6 格；D 第四片被重写过（704）取 3 格；E 没有第五片（934）取 3 格，共 20 段。变体由 `tools/apply_variant.py` 换进副本（原文恰好命中一次）。基线 20 / 20 过（`out/baseline.txt`）。

| 变体 | 探针 20 段里红的 | 30-1 三条 | 别的目标 |
|---|---|---|---|
| kx1_cap_at_third（入库，胶水 `.min(2)`） | 13：A，C 全部 6，D 全部 3，E 全部 3 | 2 红 | — |
| kx2_cap_at_third（入库，模型 `.min(2)`） | 同上 13 | 2 红 | — |
| retain_only_second（入库） | 19：除 A 外全部 | 2 红 | — |
| drop_only_third（入库） | 8：C 的 index 2、3 五格，E 三格 | 1 红 | — |
| glue_cap_at_fourth（攻方，胶水 `.min(3)`） | 4：A，E 三格 | **3 过** | `--lib` 84 过；b27 两条、135 片追加、unlink 8 条、random 23 条（19 ignored）全过 |
| model_cap_at_fourth（攻方，模型 `.min(3)`） | 4：A，E 三格 | 3 过 | `--lib` 1 红（模型自己的 135 片单测） |
| retain_up_to_third（攻方，回退只取回第 2–3 片） | 7：C index 1，D 三格，E 三格 | **3 过** | `--lib` 84 过；b27 两条、unlink 8 条、random 23 条全过 |
| drop_up_to_fourth（攻方，回退只丢第 2–4 片） | 3：E 三格 | **3 过** | 同上全过 |

（逐条在 `out/<变体>.txt`、`out/<变体>-broad.txt`、`out/<变体>-lib.txt`、`out/glue_cap_at_fourth-random.txt`。）

**入库四个变体**：换取样点、放开回退目标与回退后建几个都还红，回退后一个都不建（after = 0）也红（回退那一步就对不上）。这一格没打中。

**打中的一格**：30-1 把「第 k 片都对」从 k = 3 推到 k = 4，没有推到一般的 k。往后挪一片的三个变体在仓里全部 harness 档相关目标上活着，只有建到第 5 片的取样点抓得到；再加一条第 5 片的用例，挪两片的又活着。被改的三句：

- `crates/singlefs-harness/src/model_comparison.rs:80`：「ModelUnitRole::inode_leaf_container_at(u64::from(index.0))」
- `crates/singlefs-harness/src/model.rs:1720`：「!matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))」
- `crates/singlefs-harness/src/model.rs:1727`：「matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))」

模型落点那一句（`crates/singlefs-harness/src/model.rs:197`）不在此列：模型自己的单测 `crates/singlefs-harness/src/model.rs:4261`「fn the_inode_leaf_container_position_reaches_the_last_of_135_leaf_containers_and_panics_one_record_beyond」逐片扫到第 135 片，封顶在哪一片都红。

四句：① 分辨臂——分辨「胶水与模型对任意 k 都对」与「只对 k ≤ 4 对」，入库三条分不开；② 判别子看得到——历史建到第 5 片时模型逐片比分配代就看得到；③ 满足的是 M1「三条用例罩不罩得住它们要证的行为」那一句（它们要证的是第 k 片都对），不是「四个变体还红不红」那一句；④ 改法见下表，「加一个取样点」那一类在挪两片的变体上照样不中。

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| B1/B2 的路子再加一条建到第 5 片的用例 | 挪一片的三个 | 量过（探针 A、E 那几格）；挪两片的照样活，推的 |
| B3：胶水映射逐片单测，k = 0..135 映到互不相同、且等于模型第 k 片的角色（`tests/b30_opus_glue_leaf_role_is_injective.rs`） | 胶水封顶在任何一片 | 量过：基线过，glue_cap_at_fourth 与 kx1_cap_at_third 都红（`out/*-b3.txt`）。只在我的模型上量过、被攻过零轮 |
| B4：模型回退那两句的单测，目标与现行各取 1..135 片的几组组合，回退后右侧各片的角色集合等于目标的 | 回退只取回 / 只丢某几片的任意截断 | 推的（没实现：那两句在模型内部，要一个只供测试的入口） |

**锚点互相掩护**：四行与第 28 批 KX1、KX2 等行同一原文、不同替换文（`crates/mutations.tsv` 里 `model_comparison.rs` 那一句三行、`model.rs` 落点那一句三行）。按条记录每行单独改坏、单独跑点名的测试，同原文的几行之间不共享改坏态；全表同原文同替换文的行组有 191 组（`python3` 现数），也是这种用法。没打中。

## M2　30-2：E1、种类 ≥ 32、I-5.4 按版本、GPU 加密 / 压缩

### E1 在合法历史上误红：没打中（量过）

探针 `tests/b30_opus_e1_i54_legal_histories.rs`：四组比重（BROAD、DELETING_FILES_OF_SEVERAL_LOWER_LEAVES、ROLLBACK_AFTER_RAISING_THE_FLOOR、REUSE_AFTER_RAISING_THE_FLOOR）随机生成历史，`PerStepChecker::Skipped`，起点与每一步之后对执行器交出的镜像各跑一次 `check_pool_image`，记 I-2.4 违例与 I-5.4 各结局。取样分批（先跑一小段估时：每组 1 段 30 步 124 张镜像 43 秒，release）：

| 批 | 种子 | 每段步数 | 镜像 | I-2.4 违例 | I-5.4 | 新式部分没判 | unlink / 回退 / 可写挂载做成 |
|---|---|---|---|---|---|---|---|
| 1（`out/e1-i54-legal-batch1.log`） | 1000..1021 每组 21 段 | 30 | 2604 | 0 | Holds 2569、NotApplicable 35 | 0 | 49 / 296 / 264 |
| 2（`out/e1-i54-legal-batch2.log`） | 5000..5021 每组 21 段 | 60 | 5124 | 0 | Holds 4931、NotApplicable 193 | 0 | 53 / 653 / 525 |

探针臂（`tools/e1_probe_variant.py`：把 E1 那一判换成恒判 I-2.4 不成立，只为数走到几次）在批 1 上 1704 / 2604 张镜像红，第一条违例落在墓碑树 1020 张、谱系树 684 张（`out/e1-probe-arm-batch1-summary.txt`）：E1 那一句在这批合法历史里确实被走到（删除进行中、回退之后、多实例都在里面），没有一次误红。

取样的限度：只判每一步之后的干净镜像，没枚举崩溃态（层 0 枚举一个状态也没跑）；墓碑树、谱系树这批里都只有一层容器（根直接管容器），多层容器树没取到。

### 走读跟随而不判 extent 偏移的指针：没打中（推的，逐调用点读码）

池级 checker 读被引单元的入口只有 `read_referenced_unit` 六处与 `read_index_node`：`crates/singlefs-checker/src/walk.rs:1271`「judge_pointer_extent_offset_is_zero(&mut self.judgements, &pointer, what);」罩所有经 `read_index_node` 读的码 2 节点（树表、各树根、多层码 2 树、分配记录树、extent 上下段、容器树根之下的内部节点）；`crates/singlefs-checker/src/walk.rs:2065`「judge_pointer_extent_offset_is_zero(&mut self.judgements, &child, &what);」是 E1；inode 内部条目、实例表链、extent 数据指针各在 2229、2570、2731 行判。剩下的 1542 行读的是映射条目指的单元，映射条目只有两条位置条目、没有指针头（`crates/singlefs-checker/src/walk.rs:1522`「judge_location_order(&mut self.judgements, &view, "映射条目");」）。GPU 走读内核 `w_read_index_node` 与 1582、1767、2593、2927 行几处一一对应，各带 `w_judge_pointer_extent_offset`。E1 之后 I-2.4 那一行「走读跟随的每条指针都判 extent 偏移」在这两份走读里成立。

### 走读跟随而不判加密 / 压缩那几段：打中（判据外的一格；CPU 量过，GPU 走读推的）

core 读走读跟随的指针时，加密与压缩那几段整组判：`crates/singlefs-core/src/pointer.rs:214`「if self.algorithm_type != 0 {」、`crates/singlefs-core/src/pointer.rs:221`「if self.compression_algorithm_code != 0 {」，第 30 批把 GPU 恢复内核也对齐到这一组（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:512`「return all_zero(base, o, 28u) && rd8(base, o + 28u) == 0u && rd8(base, o + 29u) == 0u && rd16(base, o + 30u) == 0u;」）。两份池级 checker 只判前 28 字节：`crates/singlefs-checker/src/image.rs:1272`「pointer_bytes[..POINTER_MAC_AND_NONCE_BYTES]」，长度取自 `crates/singlefs-checker/src/image.rs:1260`「const POINTER_MAC_AND_NONCE_BYTES: usize = 16 + 12;」；GPU 走读 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1318`「w_judge(INVARIANT_I_2_4, p.mac_nonce_zero);」。

坏盘（`tests/b30_opus_alg_append.rs` 接在 `pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader.rs` 副本后面，`out/alg-probe.log`）：新池新建文件那一版，上段叶内联数据指针或树表 inode 树条目根指针的头部改一处、重封、校验和沿引用链补齐。

| 改的那一段 | 池级 checker 判红的 | I-2.4 | core 冷启动恢复 |
|---|---|---|---|
| 算法类型 0 → 1 | `[]` | Holds | UnitMalformed（两处各报自己那一句） |
| 压缩算法码 0 → 1 | `[]` | Holds | 同上 |
| 压后长度 0 → 1（码仍 0） | `[]` | Holds | 同上 |
| MAC 第 0 字节 0 → 1（对照） | `["I-2.4"]` | Violated | 同上 |

四句：① 不分辨 30-2 的臂：病根在两份池级 checker 共用的「指针头只判 MAC / nonce 与 extent 偏移」，30-2 只把 GPU 恢复内核对齐了 core；② 判别子看得到：这 4 字节就在 checker 读的那条指针里；③ 不是 I-2.4 字面的违例：`.claude/kb/invariants.md:133` 那一行写的是「单元头的这 29 字节与指针头部的 MAC 16 + nonce 12（D19（块指针的结构与宽度预算） 已定项 3）由同一条规则守」，算法类型、压缩算法码、压后长度不在任何一条不变量里，算「坏盘放过」的一格，不算实现违反条款；④ 改法 F2（`tools/pointer_head_fix.py`：判的范围从前 28 字节扩到前 32 字节）在副本上量过：四种坏盘都红 I-2.4，`checker_known_bad_images` 89 过、`checker_judges_reserved_bytes_and_widths_it_used_to_skip` 15 过，合法批 1 的 2604 张镜像 I-2.4 违例 0（`out/alg-probe-with-f2.log`、`out/e1-i54-legal-batch1-with-f2.log`）。违例那一句与 kb 那一行要跟着改。只在我的模型上量过、被攻过零轮；GPU 走读内核同一处要一起改（推的）。

### GPU 恢复内核种类 ≥ 32 与加密 / 压缩 8 处：没打中（推的，读码，GPU 没跑）

- 种类 ≥ 32：`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2994`「if (rd16(tree_table.base, entry_offset(tree_table, earlier) + 10u) == kind) {」比整个 u16，与 `crates/singlefs-core/src/recovery.rs:2335`「if !kinds_seen.insert(entry.kind) {」同一谓词。逐格对过的盘面：同种 ≥ 32 两条（根指针全零或非零）两边都拒；单条 ≥ 32、根非零两边都因种类没登记拒（GPU 码 34）；单条 ≥ 32、根全零两边都跳过；< 32 与 ≥ 32 不会相等。GPU 是逐条解、逐条比，core 是先比再往下，次序不同只改失败码，`walk_failed` 的码只进诊断字，不进红位。
- 8 处：字段偏移（MAC 16、nonce 12、算法类型 1 在 28、压缩码 1 在 29、压后长度 2 在 30、extent 偏移 2 在 32）与 core 字段表同；判法都是「不是 0 就判这条指针所在的结构损坏」；位置逐处对：树表条目根指针、inode 内部条目子指针、分配记录树内部条目子指针、extent 内部条目（整节点先判）、上段叶标签 1 / 2、下段叶数据指针、实例表链指针，加原有的容器树条目（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2893`）、码 2 多层树内部条目与 journal 新根段四条。根记录两边都只判 extent 偏移。没找到一处 GPU 拒而 core 收或反过来。
- 推翻条件：GPU 对拍新加的 9 条在有卡的机器上任一条 GPU 与 CPU 结局不同。

### I-5.4 按版本：合法历史上没多出部分没判（量过）；坏盘上放过一形（打中，量过）

合法历史：上面两批 7728 张镜像里 I-5.4 部分没判 0 次（新式那一句、别的部分没判都是 0）。

坏盘：覆盖写之后（A = txg 3、B = txg 4）逐个把一个单元的两份副本清零（照 b29 正推探针的造法，`tests/b30_opus_i54_append.rs` 接在 `checker_known_bad_images.rs` 副本后面，`out/i54-units-and-time.log`）。26 个单元里：A 的分配记录树五个节点（50245–50249）、B 的两片叶（50257、50258）与 B 的另外三个节点（50259–50261）清零都报部分没判（新式那一句），第 30 批要的就是这几格；**B 的树表 50264、A 的树表 50252、第 0 代树表 50178 清零都报 I-5.4 = Holds**：

```text
B30UNIT slot=50252 len=16384 violated=["I-2.1", "I-3.1", "I-4.8", "I-7.4"] I-5.4=Holds
B30UNIT slot=50264 (B 的树表) len=16384 violated=["I-2.1", "I-4.8", "I-7.2", "I-7.4"] I-5.4=Holds
```

机理：`crates/singlefs-checker/src/walk.rs:4919`「read_index_node_without_judging(reader, &tree_table_pointer, cache)」读不出时，这一版既不进「带树一版根记录那一格」，种类 3 的根也取不到，整版对 I-5.4 没有一次评估、也不留一句没判；文档注释自己写着 `crates/singlefs-checker/src/walk.rs:4885`「/// 树表读不出的根这一格不判（认不出带不带树）」。别的版本的树走到了叶，I-5.4 就报成立——这正是 b29 判决 L2 那一行要堵的「别的版本的树走到了叶替这一版作保」，换了一个单元（树表而不是分配记录树）就照样成立。整体 checker 仍红（I-2.1 等），所以不是整份镜像被放过，是 I-5.4 这一条的结论说多了。

四句：① 不分辨新旧两种判法：第 30 批之前与之后在这三张镜像上都报成立，是两种判法共用的缺口，不是第 30 批引进的；② 判别子看得到：树表指针非零而读不出，checker 当场知道；③ 满足的是 M2「I-5.4 按版本……在坏盘上会不会放过」那一句，「放过」在这里指该报部分没判而报成立，不是漏一条违例；④ 改法 F1（`tools/i54_tree_table_fix.py`：树表指针非零而读不出的候选根记进一份单列的名单，I-5.4 报部分没判、点名那几版）在副本上量过：三张镜像都改报部分没判（`B30UNIT slot=50264 … I-5.4=PartiallyJudged(树表读不出的版本，分配记录树认不出、一格没判（别的版本的树走到了叶不替它作保）：根（实例 1、txg 4）)`），`checker_known_bad_images` 89 过，合法批 1 的 2604 张镜像部分没判 0（`out/i54-units-with-f1.log`、`out/known-bad-with-f1.log`、`out/e1-i54-legal-batch1-with-f1.log`）。只在我的模型上量过、被攻过零轮。F1 在带读故障的崩溃态上会多报部分没判（那时说的是实话），层 0 / 崩溃注入的计数里 I-5.4「不适用」一格会跟着变（`crates/singlefs-checker-tier/src/crash.rs:1492`「// 部分没判（`InvariantVerdict::PartiallyJudged`）不红、也不算评估过（判到的那一部分成立不等于整条成立）：计进不适用这一格，」），推的。

另记一处同形、没造盘面（推的）：树表 0 条、根记录直接持有分配记录树的那一版（写过行的那一版），它的树读不出时也不进按版本那一层（那一层只收带树的版本），别的版本走到了叶照样报成立。

GPU 一侧：池级 checker 的 GPU 判器只带违例位（`crash_judge_checker_scan.wgsl` 的 `judge_allocation_records_disjoint` 只在违例时 `violate`），部分没判与成立在卡上是同一个结局，按版本这一层与 F1 都不需要 GPU 跟改（推的）。

## M3　30-3：T_time 改毫秒

**三处同单位同值：没打中（量过）。** `crates/singlefs-core/src/system_configuration.rs:86`「const TIME_THRESHOLD_MILLISECONDS: u32 = 5000;」由 `crates/singlefs-core/src/system_configuration.rs:523`「writer.put_u32(self.runtime.time_threshold_milliseconds());」写进偏移 433；G1 期望 `crates/singlefs-harness/src/mount_transparency.rs:1359`「const BUILT_IN_TIME_THRESHOLD_MILLISECONDS_OF_THE_CLAUSES: u32 = 5000;」；字段表 `.claude/kb/layout/01-first-txn.md:149`「| 可调值 | T_time | 4 | 5000（单位毫秒，即 5 秒） | D16（发布语义） 已定项 5 |」。探针每槽断言今天写的是 5000（`B30TIME`、`B30PIN`、`B30CRC` 三处都先断言这一句才往下）。

**全仓按秒解这 4 字节：没打中（量过 + 读码）。**
- 读码：`crates/` 里读偏移 433 的只有 G1 那一处（按毫秒）；core 自己不读（运行配置一档没有字段，挂载选项通道不存在）；池级 checker 与两个 GPU 内核没有一处读它。`research/` 的装置：E91、E159 内部常量本来就按毫秒，E108 按秒但只是模型参数不读盘，E142 已写 5000，E170 五个规约执行器按偏移 433 解出 `checkpoint_time_threshold` 却不拿它判任何东西。
- 量过：覆盖写之后的镜像上把两块盘四个系统配置槽的 [433, 437) 一起换成 5、0、200、4999、u32::MAX，重封；`check_pool_image` 的整份判定与 `recover` 两遍（看 / 不看 journal）的结局与没换时逐字相同（`out/i54-units-and-time.log` 的 `B30TIME` 五行 `checker_same=true consulted_same=true ignored_same=true`）。这同时坐实了 30-3 报告「池级 checker 不读这 4 字节」那一句。
- 顺手记一处（不算打中）：E170 五个目录的合成池仍写 5，例 `scripts/experiments/e170_second_run_o3_spec_executor/synthetic_pool_for_tests.py:59`「"checkpoint_time_threshold": 5,」，按今天的单位是 5 毫秒；执行器只解不判，判定不受影响，换不换由实验那边定（30-3 报告第七节已列）。

**三个新钉值的出处：没打中（量过，另一条路）。** 换一条不靠「取断言的 left」的路复核：今天的字节里把 T_time 换回 5、整槽校验和重封，得到的摘要应等于第 30 批之前一直入库判绿的旧钉值。

```text
B30PIN mkfs old_equals=true differing_offsets=[155, 156, 157, 158, 433, 434]
B30PIN after_new_pool_file_creation old_equals=true differing_offsets=[155, 156, 157, 158, 433, 434]
B30CRC today=1329175690 reverted=4048568891 old_pin=4048568891 new_pin=1329175690
```

（两行 `B30PIN` 在 `out/i54-units-and-time.log` 里，由 `tests/b30_opus_time_threshold_slot_digests.rs` 打出（`tools/rerun.sh` 的 m3-pins 一步把它单写进 `slot-digests.log`）；`out/image-crc.log` 由 `tests/b30_opus_image_crc_append.rs` 打出。）`crates/singlefs-harness/tests/system_configuration_mutability_classes.rs:93`、`:95` 两个摘要与 `crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs:314`「const DEFAULT_RING_NEW_POOL_FILE_CREATION_IMAGES_CRC32: u32 = 1_329_175_690;」都与旧值只差这 4 字节加校验和；整盘 CRC 那一份说明 mkfs 到第一个文件两块盘上没有别处带着系统配置槽的副本或它的校验和（journal 记录、根都不带）。

**checker 档读码判不变的钉值：打中重取法这一格（推的）。** 分片枚举那条 golden（`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs:845`「"e1ab3f797f369a2e9f3e45195331d252306b41346bef10922c7e04e1336a8cc4";」）钉的是 58 行打印的整份 sha256，其中两行是 `crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs:785`「println!("GOLDEN_TALLY {}", count_line(&tally));」，计数行里带池级 checker 逐条不变量的计数（`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs:269`「tally.checker_counts_by_invariant(),」）。30-3 报告第六节的重取法是「先跑一次，取打出的行那条断言的 left」。这一趟的 left 同时吃进两样：T_time 改了计划哈希（`LAYER0_RESUME` 里进度文件名），以及 30-2 若改了某条不变量在这 46 个状态上的计数（E1 多了 I-2.4 的判、I-5.4 按版本可能把成立改成部分没判）。整份取 left 分不开这两样，后一样会被当成 T_time 的副作用吞进新钉值。我读码判这 46 个状态上 30-2 不改计数（新池新建文件这条流里带树的只有 A 一版，A 的根落盘时它的分配记录树已落盘；E1 只多判、合法态上不红），但这是推的，没跑 checker 档。

- 四句：① 不分辨 30-2 与 30-3 的对错，管的是重取这一步能不能看出变了什么；② 看得到：断言消息里原样打着 58 行，旧的 58 行在根环线 2026-10-05 那份 checker 档日志里；③ 满足的是 M3「checker 档读码判不变的那些钉值，判得对不对」那一句里「判」的方法；④ 改法（推的）：重取时把新旧 58 行逐行比，只许 `LAYER0_RESUME` 那一行的计划哈希不同，两趟 `GOLDEN_TALLY` 必须逐字相同，不同就停下、不换钉。

## M4　主 agent 两处

**变异行只改名：没打中（推的，读脚本）。** 按条记录的键是 `scripts/crates-mutation-rows.py:46`「行键 = sha256（这一行六段原样用制表符接起来 + 这一趟的每条内存上限 + 每条限时）。」，名字在六段里，改名只让 `crates/mutations.tsv` 第 2636 行那一行下一趟重跑，不会复用到别的记录、也不会对不上；而 core 这一批改了源码，底座指纹本来就变，整表都要重跑。仓里按旧名找这一行的只剩两份改动全文（`_m3-formal-code-b29-r1-diff.md`、`_m3-formal-code-b30-r1-diff.md`）与 b29 正推腿的产物，都是当时的记录，不是在用的键。

**字段表第 149 行与 D16 已定项 5：打中一格格式空白（低，推的）。** D16 已定项 5 的定案句 `.claude/kb/decisions/16-发布语义.md:140`「**`T_time` 用户可设，不限制**：挂载时传的值覆盖盘上的字段并回写；超出 [200 ms, 5 s] 不拒绝，只给警告（用户 2026-09-20）」。4 字节毫秒写得出 1 ms 到 4 294 967 295 ms（约 49.7 天），写不出 1 ms 以下（例 0.5 ms）与约 49.7 天以上的值；「不限制、不拒绝」与「写不出」之间怎么处置（取整、饱和、还是拒），字段表与 D16 都没写。默认值与 [200 ms, 5 s] 区间都写得出，b29 判决 L2 那一格的空白已经补上；剩下的是这两端。今天没有挂载选项通道（C422），走不到，推的。改法（零轮，交用户定）：字段表那一行或 D16 已定项 5 补一句写不出的值怎么办，例「挂载时传的值按毫秒向上取整、超过 u32 上限按上限写并警告」。

## 没打中的形状

- M1：入库四个变体在 20 段历史（取样点 5 种 × 回退目标 1、2、3 × 回退后建 0、1、3、8 的 19 种组合加一段不回退）上，凡是那个取样点分辨得了它的都红；回退后不建也红。挪一片的模型落点变体被模型自己的 135 片单测抓住。没试：回退两次、回退之后再回退、跨实例（关了再可写挂载）之后建 inode、回退目标比第 3 新更旧（index ≥ 4）。
- M2 E1：四组比重 168 段（30 步 84 段、60 步 84 段）共 7728 张每步镜像，I-2.4 违例 0；探针臂证其中 1704 / 2604 张走到容器子指针。没试：崩溃态（层 0 枚举 0 个状态）、多层容器树、带读故障的历史。
- M2 走读跟随指针清单：CPU 六处 `read_referenced_unit` 加 `read_index_node`、GPU 走读对应几处，逐处读码，都判 extent 偏移。
- M2 GPU 恢复内核：种类 ≥ 32 的四种盘面、8 处字段与判法，逐格读码与 core 同；GPU 没跑。
- M2 I-5.4：合法 7728 张镜像新式部分没判 0；26 个单元逐个清零里分配记录树那几格都报部分没判。
- M3：`crates/`、`research/` 装置里找不到按秒解偏移 433 的读者；池级 checker 与 core 恢复对这 4 字节五种取值判定逐字不变；三个新钉值换回秒等于旧钉值。
- M4：变异行键含名字，改名只触发重跑。

## 这条腿自己的限度

- 副本上量出的数只在我的副本上成立；F1、F2、B3 三个改法只在我的模型上量过、被攻过零轮，F2 的违例那一句与 kb 那一行没跟着改，B4 没实现。
- 合法历史只判每一步之后的干净镜像，取样靠随机生成的四组比重；没有枚举崩溃态（定义允许在自己的原型里调 `enumerate_layer0` 跑小流，这一轮没做：要把原型放进一个不属于 `singlefs-checker-tier` 测试二进制的新 crate，时间花在了 M1、M2 上），所以 E1 与 I-5.4 在崩溃态上的结论是推的。层 0 状态数：本腿 0 个，接手的腿从 0 算起。
- GPU 一侧（恢复内核 8 处、种类 ≥ 32、走读内核的加密 / 压缩缺口）全是读码，GPU 对拍没跑。
- M3 分片 golden 那一格的「30-2 不改这 46 个状态上的计数」是读码推的；checker 档没跑。
- 一次抽样的「没打中」只算这一次。
- 我在 M4 搜旧名时，一条 `grep` 顺带打出了同一轮正推腿模型目录里一行数据文件的名字与那一行开头（`research/prompts/m3-formal-code-b30-r1-sonnet-model/data/mutants.tsv` 第 19 行，变异名本身）；没读它的报告与别的产物，这一行没用进任何结论。
- 跑之前看负载：别的会话的 `cargo test --no-run`（`/tmp/claude-1000/m3-formal-31-1/tree`）在跑，没有性能测量；包装每次照理由并行放行，没排队等锁。

## 没做什么

- 没跑任何重型测试：checker 档测试、GPU 对拍、`crates/mutations.tsv` 整表、层 0 枚举都没跑；只跑了 `singlefs-harness` 的测试目标（自己副本里，经 `run-with-memory-cap.sh 8G` 与 `capped.sh 7`）。
- 没改仓内任何文件，只写了这份报告与模型目录。副本与两份编译目录（`/tmp/claude-1000/m3-formal-code-b30-r1-opus/tree`、`/tmp/claude-1000/m3-formal-code-b30-r1-opus/rerun-check`）已删；草稿目录里留 `model/`（与入库模型目录同内容）、`notes/`（编译与试跑日志）、`ct.sh`（早期的 cargo 包装，后来换成 `tools/cargo_capped.sh`），都很小。
- 不判正推那几格，不替主 agent 采纳；不开第二次抽样（「没打中」那几格要支撑结论，照规则另派一条 `-s2`）。
- 要主 agent 跑的命令：没有必须的；要坐实 M3 分片 golden 那一格，提交时那一趟 `checker-tier-layer0-replay` 跑完后把新旧 58 行逐行比（只许 `LAYER0_RESUME` 的计划哈希不同）。

## 引文核对

`python3 scripts/cite-check.py research/prompts/m3-formal-code-b30-r1-opus-output.md --root /tmp/claude-1000/m3-formal-code-b30-r1/tree --root . --background research/prompts/_m3-formal-code-b30-r1-background.md` 末行：

```text
  ✓ 核了 29 处引文，对上 29 处，没判 0 处（逐处列在上面）
```
