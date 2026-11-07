import subprocess


def copy_entries(root, destination, entries):
    # 样本自己的共用库：拷法只在它里面定义，这一份不扫
    subprocess.run(["rsync", "-a", "--relative", *["./" + entry for entry in entries], "./crates", destination + "/"], cwd=root, check=True)
