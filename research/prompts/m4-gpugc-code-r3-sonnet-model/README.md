# m4-gpugc-code-r3 正推腿（Sonnet）的只读事实探针

只读：全是对冻结副本与 kb 文件的 grep / sed / awk，不起 cargo、不起 GPU、不改文件。`out/facts.expected` 是腿在草稿目录里跑出的一份，主 agent 复跑后逐行比。

复跑（整条交 task-server）：

    task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-code-r3-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-code-r3/repo --work <新的草稿目录> --model /home/fy5090/code/singlefs/research/prompts/m4-gpugc-code-r3-sonnet-model

比：`diff <(grep '^FACT' <新的草稿目录>/facts.out) /home/fy5090/code/singlefs/research/prompts/m4-gpugc-code-r3-sonnet-model/out/facts.expected`

`tools/frags.tsv` 是报告 T1 引的 D26 已定项 13 条款片段（行号 TAB 片段），F15 逐条在 kb 文件那一行里 `grep -F`。
