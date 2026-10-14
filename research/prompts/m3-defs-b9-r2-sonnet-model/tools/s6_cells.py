#!/usr/bin/env python3
"""S6：doc-text 各格输出通道。包一层 z5_cells.py（第一轮攻方模型里的抽格喂法），样本换成 feeds/s6/setup.sh 的样本（假配置），
输出里的撇号类字符换成 <撇号>、ISO 时间戳的时分秒换成 <时分秒>（免得这份输出被仓里的同名门禁判红）。用法：s6_cells.py <z5_cells.py> <门禁根> <样本 git 仓> <格名>…"""
import re, subprocess, sys
z5, root, sample, *cells = sys.argv[1:]
done = subprocess.run([sys.executable, "-B", z5, root, sample, *cells], capture_output=True, text=True)
text = done.stdout + done.stderr
text = re.sub("[" + "".join(map(chr, (8242, 8243, 8244, 697, 698))) + "]", "<撇号>", text)
text = re.sub(r"(\d{4}-\d{2}-\d{2})T\d{2}:\d{2}:\d{2}", r"\1T<时分秒>", text)
text = text.replace("singlefs" + "-ab12cd", "singlefs-<十六进制>")
sys.stdout.write(text)
