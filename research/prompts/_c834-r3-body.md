# C834（择到的系统配置实例代号随盘序变）：第三轮正文（2026-10-15）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 K1 K2 K3 K4 K5 K6 R1 R2 R3 R4 T1 T2 T3 T4 T5 -->

## 一、这一轮要判什么

同一题的第三轮，也是最终轮，不再开第四轮。前两轮判决：`research/prompts/c834-r1-main-verification.md`、`research/prompts/c834-r2-main-verification.md`。第二轮判下：K6 出局；第一轮「指针槽读法在丁上全拒」是攻方原型把 32 KiB 缓冲交给 `parse_index_node` 的缺陷，读盘长度按单元类别取之后两腿都没打中它。

这一轮只判两个留下的变体：

- **指针槽读法**：被判的较新可读根 R，它的根记录里树表与中央映射树根两个指针，各两个落点（每块盘一个），共四个槽；四个槽里有一个读出自证过的单元头、其实例代号比 R 的大，就放过，否则拒。读盘长度按单元类别取（索引节点 16 KiB、数据单元与打包记录单元 32 KiB）。实现见 `research/prompts/c834-r2-opus-model/patch/apply_candidates.py` 的 `pointer_per_kind` 与 `research/prompts/c834-r2-sonnet-model/patch/apply_candidates.py` 的 `pointer_slots_fixed`。
- **映射根变体**：只读中央映射树根那两个槽、只认索引节点，其余同。实现见 `research/prompts/c834-r2-sonnet-model/patch/apply_candidates.py` 的 `mapping_root_only`。

第二轮里两个变体在走到那一判的全部（状态，次序）上逐行同判，没造出过一段分得开它们的历史。

第三轮分工：「已定」项撑不撑得住 / 未定项清单的完整性 / 那一轮报出的数字能不能核。

出口：

- 两个变体各判站得住 / 站不住。
- 都站得住时，给出分得开它们的历史；造不出就写明，并带每个变体的可用性代价（误拒的形与状态数），交用户选一个。
- 都出局时，写明判别子缺什么，交用户；参照臂是「一律拒」，它的代价是入库 `urJ` 用例红。

## 二、实现今天的样子（主 agent 的观测，2026-10-15 现查）

冻结副本沿用第一轮那一份 `~/.local/share/singlefs/work/c834-r1/repo/`：主工作区此刻的 `crates/singlefs-core/src/mount.rs`、`recovery.rs`、`unit.rs` 与第二轮快照的 sha256 相同，冻结副本对快照全对上，两样都是主 agent 现核的。快照照抄在 `research/prompts/c834-r3-snapshot/crates-sha256.txt`，行号按冻结副本，方案按 `crates/` 今天的实现来谈。

- 那一判：`crates/singlefs-core/src/mount.rs` 第 2078 行（`placements_in_the_accounts_of_newer_readable_roots` 里那个 `match`，第 2072 行起）。`crates/` 里没有指针槽读法，也没有映射根变体：`grep -rn 'pointer_slot\|mapping_root_only' crates/` 零命中。
- `parse_index_node` 在 `crates/singlefs-core/src/unit.rs` 第 390 行起，第 391 行按 `NODE_BYTES`（16384）判长度。
- 第二轮两腿的模型：
  - `research/prompts/c834-r2-opus-model/`：384 槽小盘、R 是一次覆盖写；丙、丁、甲、乙、戊、三次挂载、丁只藏记录、丁加 y2，崩溃状态全量枚举，两种盘序。
  - `research/prompts/c834-r2-sonnet-model/`：九段历史 × 两序，加攻方五段。
  - 第二轮攻方报告第三节有一张表，按四个落点读出的东西分了 13 类。
- 第二轮没造出来的形：
  - 多层中央映射树：384 槽小盘只有一层。
  - 指针槽上落着数据单元或打包记录单元：只出现了全零与索引节点两种。
  - R 有多条记录。
  - 读回全零之外的坏字节，例如撕裂写或位翻转落在指针槽上。
  - 树表槽是更新单元而映射根槽不是的形。
- 可用性代价，第二轮量过：
  - `later_mount_records_landed_r_units_intact` 那一形拒。
  - clean 前缀里 M1 建在 R 上、R 的单元一份不少的 3 个状态拒。
  - 丁加 y2 二重故障下，R 已被写掉的 16388 个状态拒。
- 第一轮判决 G5：`.claude/kb/decisions/23-journal的角色与格式.md` 第 392 行「现算不成不隔离、不拒」与实现第 2078 行走拒对不上；用户 2026-10-08 定的 Y2 收严在 kb 里零命中。新条款落 D23（journal 的角色与格式） 已定项 14。

## 三、格

| 格 | 判什么 |
|---|---|
| T1 | 第二轮没造出的形上，两个变体各判成什么：多层中央映射树（换几何，让映射根读得出而下层节点读不出，或反过来）、指针槽上落着数据单元或打包记录单元（整个或后半截）、R 有多条记录、撕裂写或位翻转落在指针槽上。结局记写没写掉还在用的单元、该做成的做没做成 |
| T2 | 造一段分得开两个变体的历史：一个放过、一个拒，且其中一个判错；造不出，写明为什么造不出（读码或量的） |
| T3 | 把指针槽读法写成一句可以落进 D23（journal 的角色与格式） 已定项 14 的条款：读哪几个槽、按什么长度读、「比 R 新」比的是哪一个实例代号（G3 里列的四种读法之外，指针槽上单元头里的那一个）、读不出时怎么判；逐句核它与已定项 14 第 392 行那一段、Y2 收严、I-7.2、I-7.4 对不对得上 |
| T4 | 第二轮报的数能不能核：两腿模型在冻结副本上重跑，丙、三次挂载、丁、形二、形三那几格的状态数与结局逐格对；`mapping_root_only` 在乙那一类中间约 16000 个状态上补跑全量 |
| T5 | 未定项清单的完整性：这一判的条款写定之后，还有哪些相邻的格没有条款（例：回卷之后的号、映射根与树表落点不在同一块盘上、可写挂载重读一次的范围），逐条列出，写明是不是挡着 C834 |

## 四、分工

| 腿 | 立场 | 格 |
|---|---|---|
| 云端正推（Sonnet） | 从条款与冻结副本推出指针槽读法与映射根变体各该写成什么样，核数 | T3、T4、T5 |
| 云端攻方（Opus） | 在冻结副本的拷贝上造第二轮没造过的形攻两个变体，并找分得开它们的历史；读码得出的结论尽量换成造出来的 | T1、T2 |

## 五、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一个变体被造出判错的历史（写掉还在用的单元，或该做成的拒且那一形是入库用例认的合法态） ⇒ 那个变体出局 | 一段可复现的历史，两序之一判错 |
| 两个变体都站得住、有分得开它们的历史 ⇒ 按那段历史上谁判对取谁 | T2 造出一段，两个变体判法不同 |
| 两个变体都站得住、分不开 ⇒ 写岔路单交用户，每个变体带可用性代价与读盘开销 | T2 造不出 |
| 两个变体都出局 ⇒ 写明判别子缺什么，交用户（含「一律拒」的可用性代价） | 两个变体各有判错的历史 |

⚠️ 打中之后，先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

⚠️ 第三轮之后不再开轮。这一轮里新冒出来的零轮形态写进报告，标「零轮」。

## 六、交付要求

- 报告：
  - 先写进 `research/prompts/c834-r3-<腿名>-output.md`（正推 `-sonnet-output.md`、攻方 `-opus-output.md`）。
  - 每一次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建。
  - 回复只写短句；早点开写。
- 引文：
  - 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
  - 引代码写冻结副本里的路径与行号。
- 草稿与模型：
  - 草稿放腿自己的目录 `~/.local/share/singlefs/work/c834-r3-<腿名>/`。
  - 量法拷冻结副本，不在冻结副本里改；可以拷第二轮两腿的模型目录来改。
  - 模型与驱动随报告拷进 `research/prompts/c834-r3-<腿名>-model/`，含 `tools/rerun.sh`：从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换 `rerun_steps` 那一段。
  - 复跑比对去掉不确定的字段，在 rerun.sh 里写明；复跑经 `scripts/hosts/run-with-memory-cap.sh`。
  - rerun.sh 里编 rocksdb 的那一步带 `BINDGEN_EXTRA_CLANG_ARGS`，取 `scripts/crash/crash-amplification-lib.sh` 的 `CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS`。
- 资源：
  - 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 24G 的上限（librocksdb-sys 在 16G 下会 oom）。
  - 同一时刻只起一条 cargo；用自己的编译目录，不从别处拷编译产物。
  - 前台命令预计超过 4 分钟的拆成小段。
  - 一段历史的崩溃状态全量超过约 10⁶ 个的，不跑，写进报告「要主 agent 跑的命令」。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
- 新写的文字不用「层 0」「第一段 / 第二段」「第一条流 / 第二条流」这几种旧名（`.claude/kb/term-renames.md`）。
