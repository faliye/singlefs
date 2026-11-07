import os, shutil, subprocess
def build(root, destination):
    shutil.copytree(os.path.join(root, "crates"), os.path.join(destination, "crates"))
    shutil.copy2(os.path.join(root, "Cargo.lock"), destination)
    subprocess.run(["rsync", "-a", "--exclude", "target", "prune-server", destination], check=True)
    subprocess.run(["git", "-C", root, "archive", "HEAD", "crates", "Cargo.toml"], check=True)
    shutil.copyfile(os.path.join(destination, "notes.txt"), os.path.join(root, "crates", "notes.txt"))
COPIED_ENTRIES = ("crates", ".cargo", "prune-server", "Cargo.lock")
