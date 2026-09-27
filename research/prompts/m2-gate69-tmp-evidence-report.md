# gate 69 第二类红：清零报告

## 任务
门禁 69 号（`.claude/gate.d/69-evidence-in-repo.sh`）判据二红：`research/prompts/` 下这一轮新写的
.md 把 `/tmp` 下的东西当依据引用。把还在盘上的 /tmp 证据拷进仓、改引用指向仓内路径。

## 69 号原样判定：改前 / 改后

改前（`/tmp/claude-1000/gate69-tmp-evidence/g69-before.log`）：判据二命中 44 行，跨 21 个文件。
改后（`/tmp/claude-1000/gate69-tmp-evidence/g69-after2.log`）：判据二命中 1 行——
`research/prompts/e158-r4-prereg.md:472`，见下方「没做什么」。判据一（E142/E158 产物跟不上装置）不归本任务，
改前改后都红，原样保留。

## 拷进仓里的 13 份 /tmp 证据（全部还在，全部拷了）

按来源目录一一对应，保留原相对结构，落在 `research/prompts/<来源目录名>-tmp-evidence/` 下：

| 源（/tmp/claude-1000/…） | sha256（源） | 目的 | sha256（拷贝，逐字节核对通过） | 大小 |
|---|---|---|---|---|
| process-safety/report.md | 6174fa1d227cfb1677ca4aab78b6913d1f0624b972cf33347fd5a68edb0b8f8f | research/prompts/process-safety-tmp-evidence/report.md | 同源（拷贝时） | 12K |
| defs-closeout-draft/report.md | 46bcc439bcf3faa1b9dbfbc91c35faeabef47c94736fc3f9aac52c97fa903b7f | research/prompts/defs-closeout-draft-tmp-evidence/report.md | 同源 | 48K |
| defs-closeout-draft-2/report.md | 651fee1e009d1f5e1beb5e869308612ba1b322de24d1b64f16921fc4a672608d | research/prompts/defs-closeout-draft-2-tmp-evidence/report.md | 同源 | 52K |
| defs-closeout-r1-fixes/report.md | a066e5cea56928e497b50922e77cc0d876a0a7d5a2c4f3e1e1dca3c5993267b9 | research/prompts/defs-closeout-r1-fixes-tmp-evidence/report.md | 同源 | 84K |
| defs-closeout-r1-fixes/probe/f14-cases.json | df2d1ee879012b06618b752c121ae5d713450eb0ab836f42df004e02ed94965a | research/prompts/defs-closeout-r1-fixes-tmp-evidence/probe/f14-cases.json | 同源 | 4.0K |
| defs-closeout-r1-fixes/probe/f14-out.txt | 3109772f30121c10fb56fb8504caa97c76e56a579bd975b35481825cf876dcb4 | research/prompts/defs-closeout-r1-fixes-tmp-evidence/probe/f14-out.txt | 同源 | 8.0K |
| defs-closeout-r2-fixes/report.md | 5f4790150ed73ea220e1ca655cfed7eec6396ee43fb48380cf895ada0000becc | research/prompts/defs-closeout-r2-fixes-tmp-evidence/report.md | 同源 | 84K |
| l0scale-r1-frozen/crates/singlefs-harness/src/crash.rs | de9df1da6a9d28b62e79cba074efc7c583af76cfbe8be3704a0fef59cc8aee8f | research/prompts/l0scale-r1-frozen-tmp-evidence/crates/singlefs-harness/src/crash.rs | 同源 | 104K |
| impl-rbf-4b/report.md | 8d2b47bd0d95fabe4951555eeb537e957a0836d0275533426ca00d109569b1da | research/prompts/impl-rbf-4b-tmp-evidence/report.md | 同源 | 52K |
| impl-rbf-4a/report.md | 7b6488dc73c851ea1a9ae56f21b1530fddf430f8187d99ca69ce3554da1920f6 | research/prompts/impl-rbf-4a-tmp-evidence/report.md | 同源（拷贝之后又改了一行引用，见下） | 36K |
| impl-shard-1/red-batch-1.log | b34d9ceb69ac695ceaa1aae40af09e696a4e8e31348b3827a89d439d5aa84e0c | research/prompts/impl-shard-1-tmp-evidence/red-batch-1.log | 同源 | 4.0K |
| impl-rbf-4c/report.md | f53a6dd85099f133a652c8fef038a21b64b929c5169f51789b27e7d039a4faf9 | research/prompts/impl-rbf-4c-tmp-evidence/report.md | 同源 | 32K |
| impl-rbf-7/diffstat.txt | 421b2f19099ad82e803f41ef48403a6c5fb7d3a5fcf6fa50de354c68188df4d5 | research/prompts/impl-rbf-7-tmp-evidence/diffstat.txt | 同源 | 8.0K |

拷贝后逐份 `sha256sum` 与源逐字节核对，13 份全部 OK（见 `/tmp/claude-1000/gate69-tmp-evidence/source-sha256.txt`）。
没有超过 2 MB 的文件，没有需要跳过的。

## 改了引用的 21 个文件、44 处

按 gate 69 改前判定原样列出的 44 行，逐处把 `/tmp/claude-1000/...` 换成对应的仓内路径（行号后缀
如 `:1-171`、`:9-30` 照留）。全部用 Edit 工具做定点替换（旧串在文件里恰好命中一次才换，等价于
`replace-once.py` 的不变量）：

- `_defs-m2-closeout-r1-appendix.md`（1 处）、`_defs-m2-closeout-r1-background.md`（2 处）、
  `_defs-m2-closeout-r1-body.md`（2 处）、`_defs-m2-closeout-r1-diff.md`（5 处）
- `_defs-m2-closeout-r2-appendix.md`（6 处）、`_defs-m2-closeout-r2-background.md`（7 处）、
  `_defs-m2-closeout-r2-body.md`（1 处）
- `_defs-m2-closeout-r3-appendix.md`（4 处）、`_defs-m2-closeout-r3-background.md`（5 处）、
  `_defs-m2-closeout-r3-body.md`（1 处）
- `_m2-layer0-scale-r2-appendix.md`（2 处）、`_m2-layer0-scale-r2-background.md`（2 处）
- `_m2-safety-r3-background.md`（1 处）、`_m2-safety-r3-body.md`（1 处）
- `defs-m2-closeout-r3-local-attack-translation-audit.md`（1 处）
- `e156-r4-prereg.md`（1 处）、`e156-r4-questions.md`（1 处）、`e158-r2-questions.md`（1 处）
- `m2-impl-shard1-implementer-report.md`（1 处）、`m2-impl5-implementer-report.md`（1 处）、
  `m2-impl7-implementer-report.md`（1 处）

只改了被 gate 69 标红的那一行/那一处引用；同一文件里没被标红的 /tmp 提法（描述做法、不是引依据，
例如「草稿放哪」「探针脚本在」）原样未动——这类本来就不该判红（gate 69 脚本自己的注释也这么写）。

## 拷进来的证据文件本身触发了一处新红，已处理

拷 `process-safety/report.md` 进仓之后，它自己第 37 行原样写着「草稿与证据：`/tmp/claude-1000/process-safety/`
（改前副本 `before/`、补丁 `patches/`、各日志）」——这句本来在 /tmp 时不会被 gate 69 扫到，拷进
`research/prompts/` 之后（这一轮新写、未跟踪）就落进判据二的扫描范围，被同样的规则命中。

处理：这句是在说这份旧报告当时的草稿放在哪（方法论），不是这一轮要核的依据；`before/`、`patches/`、
日志本身仍在 `/tmp/claude-1000/process-safety/` 下（1.6M，含 __pycache__、storm-records 等一批调试
副产物），没有被本轮任何三方材料点名引用，所以没有整批拷进仓（拷整棵树超出了本任务「拷已被引用的
证据」的范围）。按 gate 69 脚本自己给的出路「只是在说做法，把句子里的依据词去掉」，把「草稿与证据」
改成「草稿」，并加一句说明这不是这一轮要核的依据；路径与其余描述原样不动，没有丢信息。改后重跑
69 号，这一行不再判红。

## 没做什么

- **判据一（E142/E158 产物跟不上装置）**：改前改后都红，不归本任务（派发提示已说明），原样未动。
- **`research/prompts/e158-r4-prereg.md:472`**：改后第二次重跑 69 号时新出现的一处判据二红。
  这份文件在我开工之后才由**另一个会话**写出（`git status` 是 `??`，mtime 2026-09-26 23:20:38 UTC，
  比我重跑 69 号的时刻 23:24:34 UTC 早 4 分钟；本任务开工前它并不存在，不在 gate69-before.log 的
  44 行里）。按「只动这一轮自己的文件；看到别人没提交的改动不碰、不修」，没有碰它，留给主 agent
  或那个会话自己处理。
- **process-safety 目录里 before/、patches/、日志等其余材料**：仍在 `/tmp/claude-1000/process-safety/`
  下，没有拷进仓（见上一节的理由）。要不要拷、拷多少，留给主 agent 判断。
- doc-lint 复跑（`bash .claude/scripts/doc-lint.sh`）仍有 11 处编号定义/引用不合规
  （M32、M34、U11、U13 缺登记位；`experiments-history.md`、`experiments/158-...md` 里裸引用），
  与本任务的改动无关（都在 `.claude/kb/` 下，我没碰过这些文件），改前大概率已经存在，未处理。
- 没跑重型测试、没碰 `.claude/`、`crates/`、`records/`；只动了 `research/prompts/` 下上述文件与新增目录。
- 没有编译目录、没有仓副本、没有 worktree，没有需要在交回前删除的东西（草稿目录只有几份日志与本报告）。

## 关键文件路径

- 门禁脚本：`/home/fy5090/code/singlefs/.claude/gate.d/69-evidence-in-repo.sh`
- 改前/改后判定：`/tmp/claude-1000/gate69-tmp-evidence/g69-before.log`、
  `/tmp/claude-1000/gate69-tmp-evidence/g69-after2.log`
- 源文件 sha256：`/tmp/claude-1000/gate69-tmp-evidence/source-sha256.txt`
- doc-lint 复跑：`/tmp/claude-1000/gate69-tmp-evidence/doc-lint-after.log`
- 13 份新证据文件在 `research/prompts/*-tmp-evidence/` 下（见上表）
- 21 个改了引用的文件在 `research/prompts/` 下（见上面清单）
