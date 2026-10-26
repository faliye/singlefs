import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import dep_child_sub as unreferenced

def work():
    return 1

if __name__ == "__main__":
    work()
