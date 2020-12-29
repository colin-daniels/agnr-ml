import sys
import time

__doc__ = """
Entry points to python scripts used internally by the rust side of rsp2.
"""

# (you can add calls to this if you want to do some "println profiling".
#  rsp2 will timestamp the lines as they are received)
def info(*args):
    print(*args, file=sys.stderr); sys.stderr.flush(); time.sleep(0)
