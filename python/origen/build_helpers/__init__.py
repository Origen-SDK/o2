import platform, subprocess, sys

windows = platform.system() == 'Windows'

def compile_rust(dir, target=None, workspace=False):
    cmd = ['cargo', 'build']
    if workspace:
        cmd.append("--workspace")
    if target:
        cmd.append(f"--{target}")
    print(f"Building Src At: {dir}: {(' ').join(cmd)}")
    compile_result = subprocess.run(
        cmd,
        stderr=sys.stderr,
        stdout=sys.stdout,
        cwd=dir,
        shell=True,
    )
    if compile_result.returncode != 0:
        print(f"Failed to build target from {dir}. Received return code {compile_result.returncode}")
        exit(1)
