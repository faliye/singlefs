import os, runpy, subprocess, sys, importlib
HERE = os.path.dirname(os.path.abspath(__file__))

def via_runpy():
    runpy.run_path(os.path.join(HERE, "child_run.py"), run_name="__main__")

def via_subprocess():
    subprocess.run([sys.executable, os.path.join(HERE, "child_sub.py")], check=True)

def via_import_module():
    return importlib.import_module("child_mod").work()
