import os
import shutil
import sys
sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "hosts"))
import lib_repository_copy as repository_copy
REQUIRED_ITEMS = ("crates", "Cargo.toml")

def build(root, destination, source):
    """rsync -a crates 进副本（文档串里的不算）"""
    closure = repository_copy.copy_closure(root, [("scripts/tool.py", "副本里起它")])
    repository_copy.copy_entries(root, destination, closure.entries)
    shutil.copytree(source, os.path.join(destination, "kept"))
    shutil.copyfile(os.path.join(destination, "notes.txt"), os.path.join(root, "crates", "notes.txt"))
