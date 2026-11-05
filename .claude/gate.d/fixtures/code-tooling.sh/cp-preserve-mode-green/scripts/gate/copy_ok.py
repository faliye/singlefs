"""样本：文档串里写 cp -a 不算；shutil 与不保留权限的 argv 不点名。"""
import shutil, subprocess
SAMPLE = "样本命令 cp -p first second"   # 带空白的样本命令串
shutil.copytree("a", "b", symlinks=True)
subprocess.run(["cp", "-dR", "--preserve=timestamps,links", "a", "b"], check=True)
