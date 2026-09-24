# defs-gate54-tiering-r1 云端攻方（Opus）报告：K3 与 K2×K3

<!-- doc-lint:not-numbers K1 K2 K3 S1 S2 S3 S4 S5 S6 -->

- 攻击面：K3（收尾跑 54 号 `--full`、整轮门禁只核全绿标记），以及 K2 与 K3 撞在一起那一格。K1、K2 的定义句交叉归本地攻方，这里不判。
- 跑于 2026-09-23T23:26Z 前后（UTC；东京时间 2026-09-24 08:26 前后）。主仓 HEAD `3b60f098e97dc4c4f3ed9c6355422b607db1c34c`。
- 装置：合成仓 + 打合成日志的假 cargo，不编译 Rust、不跑真全量。合成仓都建在 `/tmp/claude-1000/defs54-attack/` 下，各自 `git init`，只读主仓（`cp` 与 `git show HEAD:…`），没写主仓的 git common-dir。
- 这里所有数都是**副本上的数**，不算入库装置上的数。

## 复跑

```
bash research/prompts/defs-gate54-tiering-r1-opus-model/run-all.sh
```

输出进 `research/prompts/defs-gate54-tiering-r1-opus-model/outputs/<场景>.out`，每次覆盖。输出里带开跑时刻、临时目录名，逐字节复跑不相同；判定行与退出码相同。

模型文件的 sha256（`sha256sum`，在模型目录里现算）：

```
dd1909c513f18033f44e84e081c4a4e96178cb620b500ceb2512f9619b603054  lib.sh
86b1c6667a17cdee361fbfd3f39ef9395308038bf412ff438fb0d0556b9e004a  run-all.sh
1fef86a26e50f0f9faa9d1617fbadcbfcf890805a6364099276ee4db29be8707  s1-worktree-vs-staged.sh
22c5ad8b21555197ee6c11b7fb53fb2c0c72a8d86ca098292132098a4e71c673  s2-trigger-gaps.sh
ce26e48dab28ab87fbf1eebfa870ba4c57b56e9bbc211e36f4fdd0dd7a0963ad  s3-marker-slot.sh
cf6f4d0cadc1f7327593f768997005819bd7cc45f1840edc6081c12aa7d4b459  s4-marker-provenance.sh
3a88a08ce6e594bb55cff74f773ecc3e1f2a6607c0028ab2248a3f25336a1b03  s5-transient-edit.sh
96ab8bdb26433319159fdbc771e574d7a08bdc9749bff1ca9b0b4ecd215d6751  s6-fix-arms.sh
82450172504fa8fad879a8fe7fa250cb7c541ec9d54e6edbdb426defe6b01e56  fake-bin/cargo
dc6b70a1989baf29c7d6d7f7aaa883590772ced1d8355406fbeb131814a0c347  outputs/s1-worktree-vs-staged.out
d6beb174daa5f9f59a7c9b4a5b78f657d325a12db070d0fd1705f45fd60482af  outputs/s2-trigger-gaps.out
95c8e559a278912738c6ae73e99a19d115484fe0748e5bf1d8b3e8410fcf1641  outputs/s3-marker-slot.out
1b27b2c49dc03808a90acae07fc2bd8be20d40b9875e57d647a0362e6a85d747  outputs/s4-marker-provenance.out
1b01085b9131f5a349f00ff1c740f2f22f68acb67bd60d22785b22544ef63202  outputs/s5-transient-edit.out
7b597eaec98ed9209c72be2affed3c89991a141295ea87dc719c0b16c3200ec6  outputs/s6-fix-arms.out
```

装置从主仓拷的被判对象（跑前现算，前两份与开工快照 `sha256sums.txt` 里的逐字相同）：

```
86ff561e868cab9ffb138a2e5fbcd668c467caea6939592c32485a162e017ec9  .claude/gate.d/54-layer0-replay.sh
03314d49bf746b584c170722aaef29a1271120894c3dc952473d46509aab5acb  .claude/gate.d/stage-inputs.tsv
5c75fa0eef04bb105330a935032c3f34eb5759c29918caf0f1506a4d1bf323db  research/scripts/stage-must-run.sh
f0c64f9f470c9f478be0ab864d5386e1f40f3e85df92b8a78e5e4a00fcb514f2  research/scripts/change-touches-crates.sh
```

`stage-must-run.sh` 的工作区版本与 HEAD 不同（`git diff --stat HEAD` 报 11 行增、5 行删，不是这一轮的六份文件之一），装置拷的是工作区那一份；`change-touches-crates.sh` 与 HEAD 相同。
「pre 臂」（分档前）用 `git show HEAD:` 取 54 号与 `stage-inputs.tsv`。

`gate.sh --staged` 的仿法：照共享 `gate.sh` 的建法建临时 worktree（`.claude/singlefs-ai-sop/scripts/gate.sh` 第 58 行 `worktree add --detach … HEAD`、第 75 行 `apply --index` 暂存区的 diff），在里面按它调阶段的法 `bash <阶段> <根>` 只跑 54 号。别的阶段不跑；77 在整轮里记「本次未跑」、不记失败，引第 436 行原文：

```
      77) NOT_RUN+=("$sname    本次未跑：阶段报了这一轮无对象可判（退出码 77），原因见上方它的输出") ;;
```

## 各格判定一览

「分辨」一列问的是：同一段历史上，分档前的 54 号（pre 臂，门禁自己在临时 worktree 里跑全量）判得对不对。pre 对、post 错 ⇒ 分辨；两边一起错 ⇒ 不分辨。

| 格 | 历史（场景） | 结果 | 打中哪一类（正文五「跑前条款」第 2 行的字面） | 分辨 |
|---|---|---|---|---|
| K3 触发条件 | 这一批只改 `Cargo.lock` 或只改根 `Cargo.toml`（S2） | **打中**：K3 那一句不触发，`gate.sh --staged` 下 54 号退 77，post、pre 两臂都是 4 次里 4 次 77 | 该跑全量的没跑 | **不分辨**（pre 一样 77）⇒ 照 evidence-discipline「打中不分辨臂」另立一笔账 |
| K3 触发条件 | 只改测试文件、只追加 `crates/mutations.tsv`、别的会话改了 `crates/` 而这一批没改（S2） | 没打中：前两样 K3 字面触发、快档核标记；第三样 `--staged` 下退 77，这一批里确实没有 `crates/` | — | — |
| K3 工作区 ≠ 暂存区 | 主工作区里别的会话有没暂存的 `crates/` 改动 / 未跟踪文件 / 两样都有；收尾照 `main-agent.md` 第 43 行在暂存之前于主工作区跑 `--full`（S1） | 标记在 `--staged` 下**对不上**：3 种形状 3 次都判红并列出别人的文件。出路句：**暂存之后**建 HEAD + 暂存区的 worktree 跑 `--full <根>`，3 次都绿；**暂存之前**建（第 43 行给的时点），3 次都红 | 判红本身是对的（标记罩的不是这一批）；打中的是「照定义做的那一趟全量必然白跑」 | 不适用（不是误判，是定义给的次序） |
| K3 标记出处 | 主工作区的 54 号脚本有别的会话没暂存的半成品；这一批让枚举退化成非全量；收尾在主工作区跑 `--full`（S4） | **打中**：标记里 `LAYER0 … exhaustive=false`，`--staged` 下快档判绿并把这一行原样打出来 | 该红的没红 | **分辨**（pre 判红「层 0 不是全量」） |
| K3 跑中输入改了又改回 | 收尾 `--full` 在主工作区跑，第二条流编译那一刻别的会话临时改了 `crash.rs`、跑完改回（S5） | **打中两样**：S5a 这一批本身坏、被临时改好 ⇒ 标记照写、`--staged` 判绿；S5b 这一批是好的、被临时改坏 ⇒ `--full` 判红 | S5a 该红的没红；S5b 不该红的红了 | **分辨**（pre：S5a 红、S5b 绿） |
| K2×K3 标记只有一格 | S3a 同一会话：验证员 `--full` 已绿，收尾又后台跑 `--full`，下一行 gate-triage 在它跑的途中起门禁。S3b 两个会话：A 在出路 worktree 里、B 在主工作区里各跑一次 `--full`（S3） | **打中**：S3a 6 格里 1 格误红（验证员绿过、门禁起在收尾途中）；S3b 8 格里 3 格 A 误红（A 先跑完 B 后跑完，B 绿、B 红各一格；A 夹在 B 里、B 绿） | 不该红的红了 | **分辨**（pre 两处都绿） |
| K2×K3 等待闸 | 收尾那一趟的进程命令行（S3c） | 命令行 `bash .claude/gate.d/54-layer0-replay.sh --full`，含 `gate.sh` 0 次：崩溃验证员第 23 行、gate-triage 第 24 行的「别的 `gate.sh` 在跑」看不见它 | 事实，是 S3 能发生的条件 | — |

## 一、K3 的触发条件（S2）

K3 那一句在 `.claude/main-agent.md` 第 43 行，整行原文：

```
| 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 这一批碰了 `crates/` 就后台跑 `bash .claude/gate.d/54-layer0-replay.sh --full`（层 0 全量只在这一步跑，判绿写全绿标记，整轮门禁的 54 号只核这个标记）→ 下一行 |
```

快档先问的两问里，第二问只拿 `crates/` 当前缀（`.claude/gate.d/54-layer0-replay.sh` 第 66 行）：

```
  scope_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/change-touches-crates.sh" "$ROOT" crates/)"
```

而这一道登记的输入是三条（`.claude/gate.d/stage-inputs.tsv` 第 11 行）：

```
54-layer0-replay.sh	crates/ Cargo.toml Cargo.lock	# 跑 crates 的崩溃点重放；两条流的用例编译期与运行期都不读 .claude/kb/layout/ 与 research/results/（段序列与布局表的比对归 52 号）
```

**历史**（`s2-trigger-gaps.sh`，每格一个新合成仓）：base 提交 → 在 base 上跑一次 `--full`（标记与 base 相等）→ 把 `refs/sop/staged-green` 指到 base 那棵树 → 做这一批并暂存 → 两种起门禁的法各起一次：gate-triage 定义第 25 行的 `gate.sh --staged`（没有 `SINGLEFS_STAGED_TREE`）与 `gate-staged.sh`（有）。这一批之后没人再跑 `--full`（K3 字面没触发时正是这样）。
「K3 字面触发」按「暂存的路径里有没有 `crates/` 开头的」算。fix 臂是我提的改法 f2：第 66 行的前缀换成 `crates/ Cargo.toml Cargo.lock`。

| 这一批 | 暂存的路径 | K3 字面触发 | post（两种起法的退出码） | pre | f2 |
|---|---|---|---|---|---|
| 只改 `Cargo.lock`（依赖 1.0.0 → 1.0.1） | `Cargo.lock` | 否 | 77 / 77 | 77 / 77 | 1 / 1（「内容不同：Cargo.lock」） |
| 只改根 `Cargo.toml`（加 `[profile.release] debug-assertions = true`） | `Cargo.toml` | 否 | 77 / 77 | 77 / 77 | 1 / 1（「内容不同：Cargo.toml」） |
| 只改测试文件 | `crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs` | 是 | 1 / 1 | 0 / 0（门禁自己跑全量） | 1 / 1 |
| 只追加 `crates/mutations.tsv` | `crates/mutations.tsv` | 是 | 1 / 1 | 0 / 0 | 1 / 1 |
| 这一批只改 `NOTES.md`，别的会话改了 `crates/singlefs-core/src/lib.rs` 没暂存 | `NOTES.md` | 否 | 77 / 77 | 77 / 77 | 77 / 77 |

post 臂里「只改测试」「只追加变异表」两格的 1 是因为模型在这一批之后故意没跑 `--full`；K3 字面在这两格触发，照做就会跑，所以这两格不算打中。
原样判定行（`outputs/s2-trigger-gaps.out`，`lock-only｜post` 那一格）：

```
      ! 本阶段跳过（这次改动没碰它判的东西）：没碰：与基 HEAD 之间共 1 个改动路径，没有一个落在 crates/ 底下
    [gate --staged 里 54 号的退出码 77]
```

**结论**：只改 `Cargo.lock` 或根 `Cargo.toml` 的一批，K3 不触发、整轮门禁的 54 号退 77，`staged-green` 照样前移（77 不记失败），此后复用那一问答「可跳过」；直到下一批碰 `crates/` 才有人跑全量。这一段里提交的依赖或编译配置改动没跑过层 0 全量。

**四句**：
1. 分不分辨：不分辨。pre 臂同一段历史同样 77——第 66 行的前缀在分档前就是 `crates/`。按 `evidence-discipline.md`「判据自己也会写错」那一节，另立一笔账，不拿它判 K3 这一格站不站得住。K3 那一句的字面（「碰了 `crates/`」）把同一个漏洞写进了定义，改的时候两处一起改。
2. 被判的系统当时看不看得到：看得到。暂存树里 `Cargo.lock` 变了，f2 的快档就判红了（量过）。
3. 满足判据的哪一个分句：正文五第 2 行「该跑全量的没跑」。
4. 跑前条款给的改法在这几格上中不中：条款三的改法（改出路句与 `main-agent.md` 那一句）碰不到第 66 行。只改 K3 那一句的字面（「碰了 54 号登记的输入」）也只管收尾跑不跑，管不到门禁退 77，所以整轮门禁照样不说话（推的）。f2 在两格上都判红（量过）。

**什么会推翻它**：共享 `gate.sh` 把 77 记成失败，或 `staged-green` 遇到 77 不前移；或真仓里 `change-touches-crates.sh` 对 `Cargo.lock` 另有判法（我读的是 `research/scripts/change-touches-crates.sh` 第 58–117 行的 `judge`，第 105 行起只拿调用方传的前缀去比）。

## 二、K3 在「工作区 ≠ 暂存区」上（S1）

出路句在 54 号里写了三处，文件头第 19 行：

```
# 工作区里有别的会话没提交的输入时，工作区跑的全量罩不到这一批暂存内容，--staged 下判红；那时在只含这一批的 worktree 里跑 `--full <它的根>`。
```

快档判红时第 225 行：

```
    echo "                不同的文件里有别的会话没提交的东西时，工作区跑的全量罩不到这一批：在只含这一批的 worktree 里跑 --full <它的根>。"
```

`main-agent.md` 第 43 行把 `--full` 放在「这一批交回到齐、暂存之前」那一步，命令不带根（默认主工作区）。

**可达性（真仓现查，只读）**：`git status --porcelain -- crates/ Cargo.toml Cargo.lock` 按状态计数，2026-09-23T23:30:44Z：

```
     21 ??
     54 M
```

暂存区此刻为空（`git diff --cached --name-only | wc -l` 得 0）。也就是说此刻任何一批的收尾在主工作区跑 `--full`，标记里都会带着这 75 个不在这一批里的文件。

**历史**（`s1-worktree-vs-staged.sh`）：base 提交 → A 这一批：新建 `crates/singlefs-harness/src/batch_a.rs`、`lib.rs` 加一行 → B 的半成品（扫三种：未跟踪的 `crates/singlefs-core/src/b_wip.rs`；已跟踪、没暂存的 `crates/singlefs-core/src/lib.rs`；两样都有）→ ① 主工作区 `bash .claude/gate.d/54-layer0-replay.sh --full` → 暂存 A 的两个文件 → ③ `gate.sh --staged` → ② 出路：`worktree add --detach HEAD` + `apply --index` 暂存区 diff，跑 `bash <wt>/.claude/gate.d/54-layer0-replay.sh --full <wt>` → ④ `gate.sh --staged`。
出路 worktree 什么时候建是人定的那一步，扫两种：暂存之后、暂存之前（第 43 行给的时点）。

| B 的东西 | ③ 主工作区的标记在 `--staged` 下 | 出路在暂存之后建：④ | 出路在暂存之前建：④ |
|---|---|---|---|
| 未跟踪 | 退 1，「只在标记里：crates/singlefs-core/src/b_wip.rs」 | 退 0 | 退 1，「内容不同：lib.rs」「只在这一次里：batch_a.rs」 |
| 已跟踪、没暂存 | 退 1，「内容不同：crates/singlefs-core/src/lib.rs」 | 退 0 | 退 1，同上 |
| 两样都有 | 退 1，两行都列 | 退 0 | 退 1，同上 |

「两样都有、暂存之后」那一格的原样（`outputs/s1-worktree-vs-staged.out`）：

```
[退出码 0]
[标记：input_hash=a75d8368e7e8fca8… judged_root=/tmp/claude-1000/defs54-attack/s1/repo-both-after-staging；exhaustive=true]
         内容不同：crates/singlefs-core/src/lib.rs
         只在标记里：crates/singlefs-core/src/b_wip.rs
[gate --staged 里 54 号的退出码 1]
[退出码 0]
[标记：input_hash=6b52c81a754dde09… judged_root=/tmp/claude-1000/defs54-attack/s1/outlet-both-after-staging；exhaustive=true]
[gate --staged 里 54 号的退出码 0]
```

**结论**：
- 条款三前半「收尾跑出的标记在 `--staged` 下对不上」：**量到了**，三种形状三次都对不上，判红时列出的正是别人的文件。
- 后半「出路句那条路走不通」：**看次序**。暂存之后用 HEAD + 暂存区建 worktree，走得通（3/3 绿）；按第 43 行的时点（暂存之前）建，暂存区是空的，建出来的只有 HEAD，走不通（3/3 红）。出路句自己没写时点，也没写怎么建那棵 worktree。
- 所以照定义的次序做，只要主工作区里有别人没提交的 `crates/` 改动，第 43 行那一趟全量（真仓两条流合计约 4 小时：背景材料 `research/prompts/_defs-gate54-tiering-r1-background.md` 第 731 行抄的 54 号分档交回报告的说法，我没现量）**一定白跑**：门禁必红，得在暂存之后再跑一趟。判红本身是对的，不是误判。
- 崩溃验证员走不了这条出路：共用约束第 41 行不许子 agent 做 `worktree`：

```
- 不做 git 写操作（`add`、`commit`、`stash`、`checkout`、`restore`、`reset`、`worktree`……），不提交，不推送。门禁自带的 git 写只有 `gate-triage` 例外，见它的定义。
```

  它只能在主工作区跑 `--full`，写出的标记在上述历史里同样对不上（与 ① 同一条路，推的，没单独跑）。

**四句**：
1. 分不分辨：这一格不是误判，打中的是「定义给的次序让全量必然多跑一趟」。pre 臂没有这一步（门禁自己在临时 worktree 里跑全量），代价不同、结论相同。
2. 系统当时看不看得到：看得到。快档列出了「只在标记里」「内容不同」的文件。
3. 满足判据的哪一个分句：条款三两半里只有前半无条件满足；后半只在「照第 43 行的时点」这个读法下满足。条款二的三类误判都不满足。
4. 条款三的改法（出路句与第 43 行一起改）碰得到：把第 43 行改成「暂存之后，在 HEAD + 暂存区的 worktree 里跑 `--full <它的根>`」，S1 的 3 种形状在「暂存之后」那一列全绿（量过，就是上表第三列）；这个改法在下文叫 f5。

**什么会推翻它**：真仓的 `git worktree add` + `apply --index` 建出来的树与 `gate.sh --staged` 的临时 worktree 在 `crates/` 下的文件集或内容不同（比如 `.gitattributes` 的换行过滤只作用于其中一边），那么「暂存之后」那一列也会红。

## 三、K3「整轮门禁只核全绿标记」：标记是谁写的、跑的是哪一版（S4、S5）

快档核标记只核三样（54 号第 213、218、230 行，原文）：

```
  if [[ ! -f "$full_green_marker_path" ]]; then
  marker_input_hash="$(sed -n 's/^input_hash=//p' "$full_green_marker_path" | head -1)"
  if [[ "$marker_count_line_total" != 3 ]]; then
```

`--full` 里「不是全量判红」在第 274 行（第二条流在第 300 行），快档不再核标记里那三行说了什么；标记里也不记是哪一版 54 号写的，而输入哈希只算 `stage-inputs.tsv` 第 11 行登记的三条路径，54 号脚本自己不在里面。
`--full` 只在开跑与跑完各算一次输入（第 246、310、311 行）：

```
write_layer0_input_manifest "$manifest_at_start" || fail_without_input_manifest
write_layer0_input_manifest "$manifest_at_finish" || fail_without_input_manifest
if [[ "$layer0_input_hash" != "$input_hash_at_start" ]]; then
```

### S4　标记出处：主工作区里的 54 号不是暂存区里那一份

**历史**（`s4-marker-provenance.sh`）：base 提交（54 号是今天分档后的版本）→ A 这一批把 `crash.rs` 改坏成「某一段没展开子集」（假 cargo 见到 `NONEXHAUSTIVE` 就打 `exhaustive=false`，用例本身不红），暂存 → B 会话在主工作区里改 54 号改到一半、没暂存，恰好让第 274 行那一判不生效（模型里把那一行换成 `if false; then`）→ A 收尾照第 43 行在主工作区跑 `bash .claude/gate.d/54-layer0-replay.sh --full`，跑的是工作区那一份脚本 → `gate.sh --staged`（临时 worktree 里是 HEAD 那一份 54 号）。

post 臂原样（`outputs/s4-marker-provenance.out`）：

```
════ S4 [post] 工作区 vs 暂存区： M .claude/gate.d/54-layer0-replay.sh M  crates/singlefs-harness/src/crash.rs ；B 的改动命中 1 行
  ── 收尾：主工作区 bash .claude/gate.d/54-layer0-replay.sh --full
    [标记：input_hash=ad967f0e6db52cb2… judged_root=/tmp/claude-1000/defs54-attack/s4/repo-post；exhaustive=false]
  ── gate.sh --staged
      ✓ 全绿标记与这批输入的内容哈希相同（ad967f0e6db52cb2…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock）：层 0 全量跑完于 2026-09-23T23:28:09Z，标记里的计数行原样：
          LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=false
    [gate --staged 里 54 号的退出码 0]
```

pre 臂（分档前，门禁在临时 worktree 里用暂存区那份脚本自己跑全量）原样：

```
      ✗ 层 0 不是全量：LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=false
    [gate --staged 里 54 号的退出码 1]
```

**四句**：
1. 分不分辨：分辨。pre 判红，post 判绿。
2. 系统当时看不看得到：看得到。标记里那一行 `exhaustive=false` 就在快档手里，快档还把它原样打了出来，只是不判。
3. 满足判据的哪一个分句：正文五第 2 行「该红的没红」。
4. 改法在这一格上中不中：条款三的改法（f5，收尾在 HEAD + 暂存区的 worktree 里、用那棵树里的 54 号跑）**修得到**：`--full` 当场判红「层 0 不是全量」、不写标记，门禁判红（量过，`outputs/s6-fix-arms.out`「S4 × f5」）。另两个我提的改法：f4a（快档另核标记里两行都有 `exhaustive=true`）与 f4b（标记记下写它的 54 号的 sha256、快档与自己比）也都判红（量过，`outputs/s4-marker-provenance.out` 的 f4a、f4b 两格）。

**可达性要打的折扣**：要有别的会话同时改 54 号、改坏的又恰好是 `--full` 里的判定。这一轮就是例子：54 号的分档改动此刻还在工作区里、没提交。

**什么会推翻它**：主 agent 实际跑的不是 `bash .claude/gate.d/54-layer0-replay.sh`（工作区那一份）而是暂存区里那一份；或快档在别处核标记里的计数行。

### S5　跑的中途别的会话改了又改回（A → B → A）

**历史**（`s5-transient-edit.sh`）：base 提交 → A 这一批改 `crates/singlefs-harness/src/crash.rs`、暂存 → 收尾在主工作区跑 `--full` → 第二条流 `cargo test` 起来那一刻（假 cargo 的钩子）B 会话改主工作区里的 `crash.rs`，第二条流跑完后改回（`cp` 回原文）→ `--full` 跑完，开跑与跑完的输入哈希相等 → `gate.sh --staged`。
假 cargo 在每条流「编译」时记下 `crates/` 的内容指纹，证明两条流编的是不是同一版。
- S5a：A 这一批本身坏（`// STREAM2_BUG`，只有第二条流的全量抓得到），B 临时删掉了这一行。
- S5b：A 这一批是好的，B 临时加了 `// STREAM2_BUG`。

S5a post 原样（`outputs/s5-transient-edit.out`）：

```
      ✓ 全绿标记写进 /tmp/claude-1000/defs54-attack/s5/repo-a-post/.git/singlefs-layer0-full-green（输入哈希 e54134bdac4527be…，8 个文件；开跑 2026-09-23T23:28:09Z，跑完 2026-09-23T23:28:09Z）
    [标记：input_hash=e54134bdac4527be… judged_root=/tmp/claude-1000/defs54-attack/s5/repo-a-post；exhaustive=true]
      ✓ 全绿标记与这批输入的内容哈希相同（e54134bdac4527be…，8 个文件，登记路径 crates/ Cargo.toml Cargo.lock）：层 0 全量跑完于 2026-09-23T23:28:09Z，标记里的计数行原样：
    [gate --staged 里 54 号的退出码 0]
    compiled test=first_transaction_step_seven_layer0 include_ignored=1 root=/tmp/claude-1000/defs54-attack/s5/repo-a-post crates_content=0380147310f385b4
    compiled test=second_transaction_step_zero_layer0 include_ignored=1 root=/tmp/claude-1000/defs54-attack/s5/repo-a-post crates_content=93f8d9b332633f60
    主工作区此刻 crash.rs 与开跑前逐字相同：是
```

两条流编译时的内容指纹不同（`0380147310f385b4` 与 `93f8d9b332633f60`），第二条流跑的不是标记里记的那一版；标记照写，门禁判绿。

| 臂 | S5a（该红） | S5b（该绿） |
|---|---|---|
| post（主工作区跑 `--full`） | 标记写上，`--staged` 退 0 | `--full` 红「两次发布那条流的层 0 用例判红」，`--staged` 退 1 |
| pre（门禁在临时 worktree 里自己跑全量） | 退 1 | 退 0 |
| f5（收尾在 HEAD + 暂存区的 worktree 里跑） | `--full` 红、不写标记，`--staged` 退 1 | 标记写上，`--staged` 退 0 |

**可达性**：`--full` 一跑约 4 小时（出处同第二节），第二条流的编译在它中途。窗口里别的会话在主工作区改 `crates/`、改完又撤回（试一个改法再撤回，或实现员改到一半被主 agent 退回），就落进这一格。
只改不撤回的那一种（净改动）会被第 311 行判红「全量跑的过程中这一道的输入变了」，这是对的，不算打中；那时同样得在 worktree 里重跑。
我查过仓里几个会在 `crates/` 上「改了再还原」的工具，都不在主工作区里改：门禁 59 号拷到临时目录再改（`.claude/gate.d/59-crates-mutation-replay.sh` 第 6 行「把仓拷到临时目录、改坏那一处、跑点名的测试」），实现员证明测试会红是在草稿副本里（`implementation-writer.md` 第 27 行），`stage-mine.py` 用 `hash-object` + `update-index`、不写工作区（第 165–166 行）。所以 A → B → A 要靠人或主 agent 手动撤回才会出现，不是哪个工具定时造出来的。

**四句**：
1. 分不分辨：分辨。pre 在 S5a 红、S5b 绿，post 两格都反了。
2. 系统当时看不看得到：`--full` 在判定那一刻只看得到两端的清单，两端逐字相同，看不到中途那一版——照今天的设计，这是「判别子观测不到」。但换个在哪跑（f5：隔离的 worktree）中途那一版就进不来，所以不是判据自相矛盾，是跑的地方选错了。
3. 满足判据的哪一个分句：S5a「该红的没红」，S5b「不该红的红了」。
4. 改法在这一格上中不中：f5 两格都对（量过，上表）。条款三的改法如果只改出路句、不改第 43 行的默认次序，两格照样中（推的：默认路还是主工作区）。

**什么会推翻它**：真 cargo 在第二条流里不重新编 `singlefs-harness` 的库（在同一个 target 下，源码的 mtime 变了就会重编，这一点是推的，没编 Rust 验）；或 54 号改成在清单之外另核编译产物的指纹。

## 四、K2 与 K3 撞在一起：标记只有一格（S3）

54 号文件头第 8 行：

```
#     两条流的全量枚举（「全量」「多线程」两段说的就是它），不问复用、不问改动范围，照跑。开跑先删旧的全绿标记，判红、被打断都不留标记；
```

第 243 行就是开跑先删：`rm -f -- "${full_green_marker_path:?}"`。标记路径只有一个（common-dir 下 `singlefs-layer0-full-green`），不分输入哈希。

会让两趟 `--full` 撞上的几句定义，原文：
- `main-agent.md` 第 40 行（改 `crates/` 那一行，派发次序的末尾是验证员）：

```
| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，主 agent 审 diff）→ 上一行的三方（代码轮）→ `crash-verifier`（层 0、QEMU、herd7、crates 变异表） |
```

- `crash-verifier.md` 第 23 行与第 24 行开头（它跑的 54 号带 `--full`）：

```
1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号时，不起 54 号，等它结束。
2. 按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个：`nice -n 19 bash .claude/gate.d/<文件>`，54 号带 `--full`（不带只跑快档）；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
```

- `main-agent.md` 第 43 行（上文第一节整行抄过）：收尾「后台跑」`--full` 之后是「→ 下一行」，即第 44 行：

```
| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → `gate-triage` |
```

- `gate-triage.md` 第 24 行：

```
1. 开跑前照共用约束「不做」一节看负载；另有别的 `gate.sh` 在跑就等它结束。
```

两份定义里挡并发的都是「别的 `gate.sh` 在跑」。S3c 量了收尾那一趟的进程命令行（`outputs/s3-marker-slot.out`）：

```
    收尾 --full 的进程（pid 2015562）的命令行：bash .claude/gate.d/54-layer0-replay.sh --full
    这条命令行里含 gate.sh 的次数：0
```

验证员自己那一趟的命令行照第 24 行也是 `nice -n 19 bash .claude/gate.d/54-layer0-replay.sh --full` 的形状，同样不含 `gate.sh`（推的，按定义字面，没另起进程量）。所以两道等待都看不见这两趟 `--full`，下面的撞车不会被它们拦住。

### S3a　同一个会话，照定义字面走

**历史**：base 提交 → A 这一批改 `lib.rs`、暂存 → （扫）验证员先照第 24 行跑 `--full`（跑过 / 没跑）→ 收尾照第 43 行后台跑 `--full`（模型里让第一条流睡 3 秒撑出窗口）→ 第 44 行 gate-triage 起 `gate.sh --staged`（扫：起在收尾那趟开跑前 / 途中 / 跑完后）。

| 验证员先跑过 | 开跑前 | 途中 | 跑完后 |
|---|---|---|---|
| 跑过（标记与这批输入相等） | 0 | **1（误红）** | 0 |
| 没跑 | 1（该红：还没人跑全量） | 1（该红） | 0 |
| pre 臂（门禁自己跑全量） | 0（只量了一次；分档前没有收尾那一趟，三个时点是同一段历史，推的） | 同左 | 同左 |

「跑过、途中」那一格原样：

```
── [验证员先跑过 --full：yes｜门禁起在收尾 --full 的：during] 起门禁前 [标记：input_hash=0a906e4b33b85a9b… judged_root=/tmp/claude-1000/defs54-attack/s3/a-yes-during；exhaustive=true]
    (收尾 --full 正在跑：[标记不在])
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但没有层 0 全量的全绿标记（/tmp/claude-1000/defs54-attack/s3/a-yes-during/.git/singlefs-layer0-full-green）
    [gate --staged 里 54 号的退出码 1]
```

真仓里「途中」这个窗口约 4 小时，而第 43 行写「后台跑」、没写等它跑完再进第 44 行。验证员那一趟已经证过同一份输入，收尾那一趟开跑就把它的标记删了。

### S3b　两个会话各跑一次

**历史**：A 这一批暂存，按第二节的出路在 HEAD + 暂存区的 worktree 里跑 `--full`（标记 H_A）；B 在主工作区里跑 `--full`（B 的崩溃验证员或 B 的收尾；主工作区里有 B 的 `b_wip.rs`，扫它让 B 绿 / 让 B 红）。两趟的先后扫四种（用假 cargo 的 sleep 排开）。最后 A 起 `gate.sh --staged`。

| B 的结果 | A 跑完 B 才开跑 | B 跑完 A 才开跑 | B 整个夹在 A 里 | A 整个夹在 B 里 |
|---|---|---|---|---|
| 绿 | **1（误红：标记被 B 的哈希盖掉）** | 0 | 0 | **1（误红：同左）** |
| 红 | **1（误红：B 开跑删了 H_A，判红不写）** | 0 | 0 | 0 |
| pre 臂（B 红，A 的门禁自己跑全量） | 0 | — | — | — |

「B 红、A 跑完 B 才开跑」那一格原样（`outputs/s3-marker-slot.out`）：

```
    A 的 --full 退出码 0（出路 worktree）
    B 的 --full 退出码 1（主工作区）
── [B 的结果：red｜先后：A-then-B] A 起门禁前 [标记不在]
      ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但没有层 0 全量的全绿标记（/tmp/claude-1000/defs54-attack/s3/b-red-A-then-B/.git/singlefs-layer0-full-green）
    [gate --staged 里 54 号的退出码 1]
```

A 的 `--full` 在 A 的输入上绿过（上面第一行），证据被一个输入不同的会话删掉或盖掉，A 的门禁判红；判红句给的出路是「跑 `--full`」，再跑 4 小时，而那 4 小时里同样的事还会再发生一次。

**四句**（S3a、S3b 一起）：
1. 分不分辨：分辨。pre 臂两段历史都判绿。
2. 系统当时看不看得到：快档那一刻只看得到「没有标记」或「标记的哈希不是我的」，看不到 H_A 曾经绿过——证据已经没了。照今天的一格设计，这是「判别子观测不到」；但标记按输入哈希分格之后（f3）就看得到，所以不是判据自相矛盾，是存法选错了。
3. 满足判据的哪一个分句：正文五第 2 行「不该红的红了」。
4. 改法在这一格上中不中：条款三的改法（f5：只改在哪跑、什么时候跑）**碰不到**：S3b 里 A 本来就在 worktree 里跑，照样 3 格误红；S3a 里收尾那趟换到 worktree 里跑，开跑照样删同一格（推的）。f3（标记按输入哈希分格，`--full` 开跑不删）在 S3a 6 格、S3b 8 格上重跑，只剩两格判红：「验证员没跑、门禁起在收尾那趟开跑前 / 途中」，这两格本来就该红（量过，`outputs/s6-fix-arms.out`「S3a × f3」「S3b × f3」）。
   另一个更小的改法：第 43 行加一句「这批输入的标记已在（验证员跑过）就不再跑」，再加一句「等 `--full` 跑完再进下一行」。它修得了 S3a，修不了 S3b（推的）。

**什么会推翻它**：真仓里 A、B 两个会话不会在同一个 4 小时窗口里各跑一次 `--full`（比如用户规定同一时刻只有一个会话碰 `crates/`）；或主 agent 在第 43 行之后一直等 `--full` 跑完才进第 44 行，而且确认 common-dir 里没有别人的 `--full` 在跑。

## 五、改法各修哪一格

改法全部是我提的，**只在我的模型上量过、被攻过零轮**。「量过」的格贴的是副本上的原样输出（`outputs/` 下对应文件），「推的」是按代码推、没实现或没跑。

| 改法 | S2 只改 `Cargo.lock` / `Cargo.toml` | S1 工作区 ≠ 暂存区（全量白跑） | S4 标记出处 | S5a 改了又改回（该红） | S5b 改了又改回（该绿） | S3a 同会话撞车 | S3b 两会话撞车 |
|---|---|---|---|---|---|---|---|
| f5＝条款三的改法：第 43 行与出路句一起改成「暂存之后，在 HEAD + 暂存区的 worktree 里、用那棵树里的 54 号跑 `--full <它的根>`」 | 不修（推的：碰不到第 66 行） | 修（量过：S1「暂存之后」一列 3/3 绿） | 修（量过：s6「S4 × f5」） | 修（量过） | 修（量过） | 不修（推的：开跑照样删同一格） | 不修（量过：S3b 里 A 本来就照 f5 跑，3 格照样误红；B 也换进 worktree 时推的同样） |
| f2：54 号第 66 行的前缀换成它登记的三条；K3 那一句的「碰了 `crates/`」换成「碰了 54 号在 `stage-inputs.tsv` 登记的路径」 | 修（量过：门禁那一半两格都判红；K3 字面那一半推的） | 不相干 | 不相干 | 不相干 | 不相干 | 不相干 | 不相干 |
| f3：标记按输入哈希分格（`singlefs-layer0-full-green.<哈希>`），`--full` 开跑不删 | 不修（推的） | 不修（推的：哈希照样对不上） | 不修（推的：坏脚本照样写进那一格） | 不修（推的） | 不修（推的） | 修（量过：只剩两格该红的红） | 修（量过：8/8 绿） |
| f4a：快档另核标记里 `LAYER0`、`LAYER0B` 两行都有 `exhaustive=true` | 不相干 | 不相干 | 修（量过） | 不修（推的） | 不修（推的） | 不相干 | 不相干 |
| f4b：标记记下写它的 54 号的 sha256，快档与自己比 | 不相干 | 不相干 | 修（量过） | 不修（推的） | 不修（推的） | 不相干 | 不相干 |

每个改法自己带的代价（都是推的）：
- f5：出路 worktree 的 target 是空的，每次多一趟 release 全编；第 43 行要从「暂存之前」挪到暂存之后，与第 44 行的次序要重写；建完 worktree 之后别人往暂存区放东西，照样对不上（与 `gate-staged.sh` 同一个老问题）。
- f3：common-dir 里标记越积越多，要有人删旧的；「同一输入上一趟红了要不要删那一格」得另定（我没实现，那一支是推的）。
- f4a 只挡 `exhaustive` 这一项，挡不住别的判定（线程数、`CHECKER` 行）被改坏。
- f4b 让 54 号每改一个字都作废已有标记，每次都得再跑一趟约 4 小时的全量。
- f2 让只改依赖的一批也要跑全量。

没有哪一个改法单独修得了全部格：f5 修 S1、S4、S5；f3 修 S3；f2 修 S2。

## 没打中的形状

| 试的形状 | 取样范围 | 结果 |
|---|---|---|
| 只改测试文件 / 只追加 `crates/mutations.tsv` | S2 各一格 × 三臂 × 两种起门禁的法 | K3 字面触发，快档核标记；没漏。代价：变异表 54 号不读，却让标记作废、逼出一趟全量（推的） |
| 这一批没碰 `crates/`、别的会话碰了 | S2 一格 × 三臂 × 两种起法 | `--staged` 下 77，这一批里确实没有，判对了 |
| 两个会话输入相同、各跑一次 `--full` | 不另跑：与 S3a「验证员跑过」同形 | 最后写的那一份哈希相同，门禁绿；只有「途中」那一格误红，已记在 S3a |
| 读到写了一半的标记 | 读代码 | 构造不出：第 332 行先写 `.partial.$$` 再 `mv -f`，读的一方要么读到旧的整份、要么读到新的整份。快档分几次读标记（第 213、218、228 行各读一次），中间被换掉时，打出来的计数行可能是另一趟的，判定仍按第 218 行那次读到的哈希；只影响打出来的字，不影响红绿（推的，没跑） |
| 用例在 `crates/` 下写未跟踪文件，让开跑与跑完的哈希对不上 | grep 了 `singlefs-harness` 的 `src/` 与 `tests/` 里的写文件 | 构造不出：镜像写在 `std::env::temp_dir()` 下（`crash_injection.rs` 第 1100 行、`tests/common/mod.rs` 第 40 行），`*.img` 被 `.gitignore` 挡着 |
| 54 号之前的门禁阶段在临时 worktree 的 `crates/` 下写文件，让快档的清单与标记对不上 | 对 54 号之前的 `.claude/gate.d/*.sh` 粗 grep「写进 `crates/`」 | 没找到；grep 很粗，不算证明 |
| `.cargo/config.toml`、`rust-toolchain.toml`、环境变量改了编译结果而不在输入里 | 读代码与 `ls -A .cargo`（只有 `mutants.toml`） | 没跑。推：加这类文件的一批，post 与 pre 都退 77，不分辨，与 S2 同账 |
| 54 号脚本自己改了、旧标记照样认 | 读代码 | 没跑。推：只改 54 号的一批，post 与 pre 在第 66 行都退 77，不分辨（分档交回报告自己列过这一条） |
| 用保留旧 mtime 的办法还原源文件，让 cargo 用旧产物 | 查仓里在 `crates/` 上「改了再还原」的工具 | 构造不出可达的触发：59 号与实现员都在副本里改；没编 Rust 验 cargo 这一侧 |
| gate-triage 把「没有标记」的红归错 | 读 `gate-triage.md` 第 26 行 | 没跑。推：红句点名的是 `.git/` 下的标记，不在暂存区 diff 里，照第 26 行会归「不是这一轮」，而这一轮自己的收尾还没跑完 |

## 这条腿自己的限度

- 假 cargo 把「编译」简化成「读调用那一刻的内容」，真 cargo 的增量编译、文件锁、mtime 判新旧都没模拟；时间窗用几秒的 sleep 代替几小时。
- 只仿了 `gate.sh --staged` 里 54 号那一段，没跑整轮；`staged-green` 遇到 77 照样前移是按共享 `gate.sh` 第 436 行与 `gate-staged.sh` 的代码推的，没真跑 `gate-staged.sh`。
- 合成仓的 `crates/` 只有 8 个文件；「B 的半成品」的形状是我造的。S4 要有人同时改坏 54 号，可达性打了折。
- 场景脚本是确定性的，判定在草稿目录与模型目录各跑过一次，两次相同；没有更多抽样。
- 派发没给禁读清单，按「无」办；读过的东西都在本仓与派发点名的 `/tmp/claude-1000/gate54-tiering/` 里。

## 没做什么

- 没判 K1、K2 的定义句交叉（归本地攻方），没判正推那几格，没替主 agent 采纳改法。
- 没编译 Rust，没跑真全量，没跑 `gate.sh`，没碰主仓的 git common-dir 与暂存区；主仓里只写了本报告与模型目录。
- 副本上的数不算入库装置上的数；要引，得在入库装置上重做。
