'''
Various helper functions, mostly meant for internal usage.
'''

import subprocess

def run_cmd(cmd, *, check=False):
    '''
        Runs a command, capturing stdout, and with options which
        should work for Windows/Linux and across supported Python versions
    '''
    return subprocess.run(cmd, shell=True, stdout=subprocess.PIPE, universal_newlines=True, check=check)
