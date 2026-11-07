import subprocess


def launch():
    subprocess.run(["bash", "scripts/hosts/capped.sh", "16", "cargo", "build"], check=False)
