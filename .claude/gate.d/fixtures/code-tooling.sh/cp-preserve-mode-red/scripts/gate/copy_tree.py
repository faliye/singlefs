"""样本：python 的 argv 列表里带保留权限的 cp（两处都该点名）。"""
import subprocess
source, target = "a", "b"
subprocess.run(["cp", "-a", source, target], check=True)
subprocess.run(['cp', '--preserve=all', 'first', 'second'], check=True)
