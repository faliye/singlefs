# 写回员报告：阶段同步欠账表、决策与 prior-art 七件

## 规格
- 草稿目录：`/home/fy5090/.local/share/singlefs/work/rot-2026-11-02-scribe-owed`
- 规格：`spec.json`、`spec.md`（同内容，36 条）；`replace-batch.py` 用的 `replace-spec.json`（去掉 basis）；检查输出 `spec-check.txt`
- 规格检查末行（`python3 scripts/kb/kb-spec-check.py spec.json`，退 0；spec.md 同绿）：
  `✓ 规格 36 条（1 份）：旧串各恰好 1 次，编号引用、指代、依据配对、依据没有别的会话还没提交的改动、变更史形态都过（查了 36 条）`
- 起草时检查第一次红 18 处，全部是 basis 字段点名了暂存区里的新文件或有未提交改动的文件（`records/2026-11-01-服务描述与历史腐烂.md`、`task-server/task_server_lib.py`、`records/2026-10-29-subagent拆分提案.md`、`crates/singlefs-harness/tests/warm_up_journal_counter.rs`）。改法是 basis 不再写这些路径，改指两份回扫报告与派发提示；kb 正文与变更史新串里的指针照给定写，没动。

## replace-batch.py 原样输出
- dry-run：`✓ 36 处都命中得对（15 个文件），--dry-run 没写`
- 实写：`✓ 36 处替换，写了 15 个文件（改名换上新 inode），回读一致`（`replace-batch-output.txt`）
- 另用脚本回读 36 条：新串在目标里各恰好 1 次、旧串已不在（历史块锚点除外），0 条不符。
- 写范围核对：`grep -P '^kb-scribe\t' .claude/hooks/agent-write-scope.tsv` 四行，其中 `.claude/kb/**` 盖住全部 15 个目标文件；找不到的目标：0 条。

## 逐条改动（文件均在 `.claude/kb/` 下；三语同改）
1. C922（`checks-owed.md`/`.en.md`/`.ja.md` 第 576 行）：旧「文件头第 74 行」，新「文件头讲 `GPU_QUIESCE_ORIGIN_READ_BACK=1` 的那一段」（en：the header paragraph on …；ja：ファイル冒頭の … を述べた一段）。现查：`scripts/hosts/multi-host-run.sh` 第 74 行是 `MULTI_HOST_RUN_SELFTEST_PEAK_FILE` 的自证说明，`GPU_QUIESCE_ORIGIN_READ_BACK=1` 在第 84 行。去掉行号，没引新行号。
2. C907（三语第 562 行，「怎么拦」一格）：旧「或在用例里写明串行的理由并声明串行段；判别力：看门狗对它不再报在用核数过少」，新「或在用例里写明串行的理由；判别力：`task-server/task status` 记的这条用例每次尝试的平均在用核数不再过少」（en、ja 对应）。按丙报告 F8：「现象」一格里 2026-10-24 的看门狗读数与「没有声明串行段」是当天量的读数，没动。en 原写 `Discrimination:`，新串改写 `Discriminative power:`，对上术语表「判别力」。
3. C436（三语第 301 行）：标题改「反引号里的路径，归档够不着」；现象格删「三层各自够不着」，保留 2026-09-21 链接指向阶段只判 `[文字](路径)` 且 `mask_code_spans()` 涂空的现查，加「反引号里的仓内路径指不指得到，现由门禁 doc-registries 的 governance-refs 格判（治理文档与 kb 中文页，`.claude/gate.d/lib-governance-refs.py`）；还欠归档这一半：`INPUT_TREES`、`SCAN` 都不含 `.claude/agents/`」；「怎么拦」只留「把 `.claude/agents/` 加进归档的 `SCAN`（它会把引用改成只留文件名）」。没关账。
4. C855（三语）：标题改「过期候选脚本的载体模式不含 skill 非 `SKILL.md` 的文件与 `scripts/**/README.md`」；现象格去掉「写死、不读 `.claude/kb-directories`」，改写成这两类进不了候选表，括注 `kb_directories()` 读登记位、不变量目录跟着认、记录另有登记表 `scripts/kb/stale-candidates-record-carriers.tsv`；「怎么拦」只留两类另加模式，判别力自证改成在 skill 目录放一个带旧说法的非 `SKILL.md` 文件。现查：`scripts/kb/stale-candidates.py` 第 247 行 `kb_directories()`、第 289 行 `record_carrier_patterns()`、`.claude/kb-directories` 登了 `.claude/kb` 与 `crates/singlefs-checker/invariants`。
5. D13（验证路线）已定项 15（`decisions/13-验证路线.md` 第 405 行、`13-verification-route.en.md` 第 402 行、`13-検証ルート.ja.md` 第 403 行）：「能单独跑（命令带 `SINGLEFS_HEAVY_TESTS=user-request`）」→「能单独跑（用户要求时由主 agent 经 `task-server/task submit` 交，命令不带 `SINGLEFS_HEAVY_TESTS` 前缀）」（en、ja 对应）。现查：`task-server/task_server_lib.py` 的 `RETIRED_VARIABLES` 含 `SINGLEFS_HEAVY_TESTS`（第 1016 行一带，派发写 1018）。
   - 变更史：`decisions-history.md`/`.en.md`/`.ja.md` 「## D13（验证路线）」节已有的「### 2026-11-02」块末尾各追加 1 个 `- ` 项（摘要、依据、改前照派发给定），没开新块。`git diff` 三份各新增 3 行，全是这一项，没有这一轮之外的行。分项状态与进度没动，`decisions.md` 哈希前后不变。
6. D23 三语第 422 行：测试名去掉 `c366_` 前缀，其余逐字不变；现查 `crates/singlefs-harness/tests/warm_up_journal_counter.rs` 第 149 行函数名为去前缀的全名。三份里 `c366_when` 现 0 处。
7. prior-art 三语（`prior-art.md` 第 14 行、`.en.md`/`.ja.md` 第 9 行）：「11」→「20」。现数：`grep -nE '^\s+(ck|ckn|ckdoc|ckdocn) ' scripts/kb/verify-citations.sh | grep -c 合成` = 20（量的，源文件）。

## 要主 agent 判的点
- D23 的测试名依据的是 `warm_up_journal_counter.rs` 暂存区里的改名（HEAD 里还是 `c366_` 前缀）：kb 这一句要和那处代码改名进同一次提交，否则提交内容之间对不上。
- 变更史依据里引的 `records/2026-10-29-subagent拆分提案.md` 现行管理方案 M07，在暂存区里是新文件（HEAD 里没有）：按派发给定的写，随那份记录同批提交。
- C855 三语「不变量目录跟着认」是按 `.claude/kb-directories` 内容与 `kb_directories()` 实现现查的，没跑候选脚本。

## 要主 agent 跑的命令
- 本轮没有翻分项状态，没有改分项计数，`decisions.md` 的分项清单生成块不用刷新，没有 `--write` 要交。
- 提交前整轮门禁照 `.claude/skills/gate/staged-gate.md`，由主 agent 判；重点看 doc-text 的 trilingual-glossary、doc-decisions 的 entry-added/shape、doc-experiments 的 decision-links（本轮没动实验页与依据段，应不受影响）。

## 改过的全部文件与前后 sha256（开工时 → 收尾；`sha-before.txt`、`sha-after.txt`）
- .claude/kb/checks-owed.en.md: a81ded57a5f3f2a4defd3b48b893dd08081108ea1901a8c59f74f6671eb67a64 → 13ad2b999f0e1d27f430c9e764c7b1036bf9e4cb5111f63102332e0b3bffc707
- .claude/kb/checks-owed.ja.md: c85e280a4a416035297c808e3f3c4f2aa1a2478c4e54d1ea96986a48c08f2e4a → 81ca1cc207123dd30e88e5478ebeed798ea3634b75b4fc99a9ee5a2b461f8fcb
- .claude/kb/checks-owed.md: d6f601d867960b4eba31b7335780015289437a5d79f74bd36316775ea39d6d84 → 491855588c1cc48b11bda8b6969bc4ecd0d31786ef35db6fc6061e5cc4fd37de
- .claude/kb/decisions/13-verification-route.en.md: 5021b391fae6b372b67489a54a05e3f0586cf096f0913d01f2da345683fee2b6 → 2fbc1e50e5011e90cce726f30a862e4f4f4a35de135ef8b9b26ad208165bac5f
- .claude/kb/decisions/13-検証ルート.ja.md: 24cfcee104997b921ad3e4ab18255b59c4c6e7c010dfb732d26371d263bff0a3 → daa00305b4a2d1c343d6e4bd3da3ad5f358e1e190725ea40617aea9a844ca4ff
- .claude/kb/decisions/13-验证路线.md: e550c0387257179cdf04f34d8f2e9f0971fe94a3606bc417a10ceacbcc3d89d7 → b9d316a3831e7255009d3b8889659d49ce17ff71bcd6cc1acb67ff7ca585ca7e
- .claude/kb/decisions/23-journal-role-format.en.md: b2725e31e004d48754ce244eebaf8bf7c7ba154e4c5018eb72fd0bb4dcebf52d → a1b3263bb943a08b5c0ec465c88b6b17810ec982bca49c7b313580d4b00a046c
- .claude/kb/decisions/23-journal的角色与格式.md: 7bc3b74c8f7847d7156fb10261c4b58ee8c48a098d00d4963f0b94b1c39cbe4b → 76151b90b643610fe4a1365035b768dd217b592622ccaa109aaa088548718712
- .claude/kb/decisions/23-ジャーナルの役割と形式.ja.md: c68df07657a63fe4d015ef239695426b73567c347606cc47f9d6c8dc749686c6 → dec490728be0669e1e14c0fceb5ec9758b97fbdb3c4c6f5279c45c98e2191254
- .claude/kb/decisions-history.en.md: 80d08d7b976b8eecbdbd4c6216f7cee5a5e2ea3114d46f7cb6947a3ac7d0b44e → c595e27170da52d2a554e2546c56f9666048119b694ba128dc0c31285c37a4ba
- .claude/kb/decisions-history.ja.md: f0a07edb8a88b3644a8fc7b0078d020c468fa2a50bdafadf4742c7e8ea4d4db1 → 4828a43c30cbf99551b4483d42e0c416b08df79ce04129e109dd1bf60d5b18f3
- .claude/kb/decisions-history.md: 410f89e7311a6ec3c86a637c2669555986c99a103ce1bcad74d1bb436ce1f17d → 9115c197f568b238608feb92ed7a82bd659b58d8e824ec8da500aadbc12734a0
- .claude/kb/decisions.md: aa5f896296fc4beeee8fc21e0488e7a16aac29fb523121f26b3162f17920520e → aa5f896296fc4beeee8fc21e0488e7a16aac29fb523121f26b3162f17920520e
- .claude/kb/prior-art.en.md: 4dc94da96915f37cfa207006138345e5c6ccf451857e194a994b7895787ca3dd → ee426103f79f592a3ca086f94d8992e20607258006afda47569fa074e28eba32
- .claude/kb/prior-art.ja.md: 58bc8e22fef27f6c90af1451de66d650e32f414c8754899080ee16becdb6a387 → 326bf303111e4d356a170154a0d8db886f483a37a1e4ed6d3f4eac31d5dceaf1
- .claude/kb/prior-art.md: 38362ba7ff4dbbdb25a53a19789fc8bbfb9044136875cda58887e64cc14065a8 → d823223c95d0addb76e0557db36661bd55fefa673f7ffc9a0088143e1441c733
- `.claude/kb/decisions.md` 前后相同（aa5f8962…），没改。

## 没做什么
- 没判定案对不对；没写规格外的句子；没碰实验页、E172、E173 页；没跑门禁、自证；没往任务服务交任务；没提交。
- C907「现象」格里 2026-10-24 的读数、C436 出处格未动。
