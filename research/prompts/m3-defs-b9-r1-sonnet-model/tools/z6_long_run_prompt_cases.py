#!/usr/bin/env python3
"""⑭ 长活声明的喂法：python3 cases14.py <树> [-v]；每例打 名 / ⑭ 这一项是否在拒绝项里"""
import subprocess, sys, re, os, importlib.util, tempfile
tree = sys.argv[1]
spec = importlib.util.spec_from_file_location("lib_long_run", os.path.join(tree, ".claude/hooks/lib_long_run.py"))
lr = importlib.util.module_from_spec(spec); sys.dont_write_bytecode = True; spec.loader.exec_module(lr)
C = lr.CANONICAL_HOSTS_SENTENCE
HEAD = ("重型测试：不跑\n开工先读 `.claude/agent-common.md`\n草稿目录：/home/fy5090/.local/share/singlefs/work/zz-nonexistent/\n"
        "报告：/home/fy5090/.local/share/singlefs/work/zz-nonexistent/r.md\n线程上限：31\n这一段回答的岔路：岔路 1\n")
OVER = "预计挂钟：超过 2 小时\n分片：按行号 mod 片数切\n续跑：结果表里已有的行跳过，重起同一条命令只补没做完的\n"
cases = [
 ("canon", "experiment-runner", OVER + "多机：" + C + "\n"),
 ("canon-period-zh", "experiment-runner", OVER + "多机：" + C + "。\n"),
 ("canon-period-en", "experiment-runner", OVER + "多机：" + C + ".\n"),
 ("canon-period-trailing-space", "experiment-runner", OVER + "多机：" + C + "。  \n"),
 ("canon-double-period", "experiment-runner", OVER + "多机：" + C + "。。\n"),
 ("canon-semicolon-note", "experiment-runner", OVER + "多机：" + C + "；双机时两台各取这一个数\n"),
 ("canon-comma-note", "experiment-runner", OVER + "多机：" + C + "，机器忙也不降\n"),
 ("canon-then-next-line-live", "experiment-runner", OVER + "多机：" + C + "\n每台线程按此刻空闲现算，机器忙时按负载降\n"),
 ("canon-then-next-line-label-live", "experiment-runner", OVER + "多机：" + C + "\n说明：每台线程按此刻空闲现算\n"),
 ("canon-then-dup-label", "experiment-runner", OVER + "多机：" + C + "\n多机：每台线程按此刻空闲现算\n"),
 ("canon-then-dup-label-bold", "experiment-runner", OVER + "多机：" + C + "\n**多机**：每台线程按此刻空闲现算\n"),
 ("canon-then-dup-label-listmark", "experiment-runner", OVER + "多机：" + C + "\n- 多机：每台线程按此刻空闲现算\n"),
 ("canon-then-indented-label", "experiment-runner", OVER + "多机：" + C + "\n  多机：每台线程按此刻空闲现算\n"),
 ("canon-then-midline-label", "experiment-runner", OVER + "多机：" + C + "\n补充（多机：每台线程按此刻空闲现算）\n"),
 ("canon-then-halfwidth-colon-dup", "experiment-runner", OVER + "多机：" + C + "\n多机: 每台线程按此刻空闲现算\n"),
 ("dup-shards", "experiment-runner", OVER + "分片：再按文件名切一层\n多机：" + C + "\n"),
 ("dup-resume", "experiment-runner", OVER + "续跑：失败的整片重做\n多机：" + C + "\n"),
 ("dup-unshardable-ok", "experiment-runner", OVER + "多机：" + C + "\n分不了：装置只认本机那块盘，第二台上没有这块盘\n分不了：第二行\n"),
 ("within-2h-dup-hosts", "experiment-runner", "预计挂钟：2 小时内\n多机：" + C + "\n多机：每台线程按此刻空闲现算\n"),
 ("within-2h-live-hosts-line", "experiment-runner", "预计挂钟：2 小时内\n多机：每台线程按此刻空闲现算\n"),
 ("fence-hosts-example-block-then-canon", "experiment-runner", OVER + "喂法示例：\n```\n多机：每台线程按此刻空闲现算\n```\n多机：" + C + "\n"),
 ("canon-inside-fence-only", "experiment-runner", "预计挂钟：超过 2 小时\n分片：按行号 mod 片数切\n续跑：结果表里已有的行跳过，重起同一条命令只补没做完的\n```\n多机：" + C + "\n```\n"),
 ("tilde-fence-hosts-example", "experiment-runner", OVER + "~~~\n多机：每台线程按此刻空闲现算\n~~~\n多机：" + C + "\n"),
 ("list-fence-hosts-example", "experiment-runner", OVER + "- ```\n  多机：每台线程按此刻空闲现算\n  ```\n多机：" + C + "\n"),
 ("unclosed-fence-hosts-example", "experiment-runner", OVER + "多机：" + C + "\n```\n多机：每台线程按此刻空闲现算\n"),
 ("wallclock-only-in-fence", "experiment-runner", "```\n预计挂钟：2 小时内\n```\n"),
 ("canon-full-width-colon", "experiment-runner", OVER.replace("分片：", "分片：") + "多机：" + C + "\n"),
 ("canon-tab-between", "experiment-runner", OVER + "多机：\t" + C + "\n"),
 ("canon-nbsp-trailing", "experiment-runner", OVER + "多机：" + C + " \n"),
 ("canon-with-trailing-markdown-bold", "experiment-runner", OVER + "多机：" + C + "**\n"),
 ("canon-in-backticks", "experiment-runner", OVER + "多机：`" + C + "`\n"),
 ("canon-in-quote-marks", "experiment-runner", OVER + "多机：「" + C + "」\n"),
]
for name, typ, body in cases:
    fd, path = tempfile.mkstemp(suffix=".md", dir=os.path.join(tree, ".."))
    os.write(fd, (HEAD + body).encode()); os.close(fd)
    r = subprocess.run(["bash", os.path.join(tree, ".claude/hooks/runner-dispatch-guard.sh"), "--check-prompt", path, "--type", typ, "--root", tree], capture_output=True, text=True)
    os.unlink(path)
    out = r.stdout + r.stderr
    m = re.search(r"会拒 \d+ 处（([^）]*)）", out)
    names = m.group(1).split("、") if m else []
    print(f"{name}\t预计挂钟与长活声明被拒={'是' if '预计挂钟与长活声明' in names else '否'}")
    if "-v" in sys.argv:
        for line in out.splitlines():
            if line.startswith("✗") : print("   ", line[:230])
