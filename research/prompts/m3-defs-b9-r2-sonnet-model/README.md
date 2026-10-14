# m3-defs-b9-r2 辩方腿（Sonnet）模型目录

复跑（整条经内存包装，上限 2G；仓根、冻结副本、草稿目录都从参数传，草稿目录要在 `~/.local/share/singlefs/work/` 底下且是新的空目录）：

    bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-defs-b9-r2-sonnet-model/tools/rerun.sh \
      --repo <仓根> --frozen <这一轮冻结副本 m3-defs-b9-r2/repo> --work <新的草稿目录>

- 第一轮冻结副本默认取 `--frozen` 同级的 `m3-defs-b9-r1/repo`，可用环境变量 `B9_FROZEN` 改。
- `RERUN_PARTS` 取 `r1sonnet r1opus s1 s2 s3 s4 s5 s6 s7` 的子集（默认全跑，约 5 分钟；单次前台命令不超过 240 秒时分几次跑）。
- 末尾逐份打 `same` / `DIFF` / `新出`，对的是 `out/` 里同名文件。路径与撇号类字符、时间戳在写进 `out/` 之前换成占位。
- `out/s1-real-prompts.txt` 不在复跑里：语料是 `~/.local/share/singlefs/work/*/prompt.md`，随时间变；命令见 `tools/s1_real_prompts.py` 文件头。

目录：`tools/` 喂法与驱动；`feeds/s3/sample` 变量来源边界的样本（`EXPECT.tsv` 是预期）；`feeds/s6/setup.sh` doc-text 样本（假配置，值都是编的）；
`feeds/shim/python3` 让 `import markdown_it` 真的失败的垫片；`gate-samples/` 门禁格要主 agent 跑的小树与命令；`out/` 原输出。

第一轮两腿模型里的喂法（`research/prompts/m3-defs-b9-r1-sonnet-model/`、`research/prompts/m3-defs-b9-r1-opus-model/`）按 `--repo` 引用，不拷贝。
