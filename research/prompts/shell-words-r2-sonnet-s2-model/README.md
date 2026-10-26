# shell-words-r2 正推腿乙（V3）模型目录

- tools/build-skeleton.sh：在草稿目录搭喂钩子用的几棵小树（repo 主工作区、copy 仓副本、stagedrepo / emptyrepo、wr 工作根）。
- tools/cases_*.py：案例表（命令、身份、应当值、判据名）；tools/frag.py：判据名 → 文件头原文片段。
- tools/run.py：把案例喂 heavy-test-guard.sh 与 bash-command-detector.sh；`--all` 跑全部。tools/adhoc.py：单条喂。
- tools/compare.py、tools/rerun.sh：复跑并与 out/ 里登记的逐条比。tools/gen_report.py、tools/expand.py：出报告用的表与引文展开。
- out/*.jsonl：登记的逐行结果（每行：编号、身份、命令、两份钩子的退出码与标准错误首行、应当值、判据名）。
复跑见报告第〇节。
