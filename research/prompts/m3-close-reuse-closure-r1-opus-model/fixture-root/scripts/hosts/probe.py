import sys


def probe(seconds):
    return {"seconds": seconds}


if __name__ == "__main__":
    print(probe(float(sys.argv[1])))
