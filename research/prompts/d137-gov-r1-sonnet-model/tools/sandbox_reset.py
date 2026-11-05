#!/usr/bin/env python3
"""沙箱复位：删掉样本跑出来的文件，只留 setup_sandbox.sh 建的那一批，免得先跑真 bash 留下的文件让钩子读得到。"""
import os, shutil
LEG = os.environ.get("D137_LEG_DIR") or os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
S = os.path.join(LEG, "sandbox")
KEEP = {"bin", "pathdir", "pd2", "bashenv.sh", "heavy.sh", "g1.sh", "f.txt", "benign.sh", "execd.sh", "execsh.sh", "src_ok.sh", ".bashrc", ".cargo", "fwd.sh", "cmdvar.sh", "cmdfile.sh"}
def reset():
    for name in os.listdir(S):
        if name in KEEP: continue
        p = os.path.join(S, name)
        if os.path.isdir(p) and not os.path.islink(p): shutil.rmtree(p, ignore_errors=True)
        else:
            try: os.remove(p)
            except OSError: pass
