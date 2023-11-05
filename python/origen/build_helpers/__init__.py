import platform, subprocess, sys, os

windows = platform.system() == 'Windows'
origen_publish_step_env = os.getenv("ORIGEN_PUBLISH_STEP", None)
if origen_publish_step_env == 0:
    publish_step = False
elif origen_publish_step_env == 1 :
    publish_step = True
else:
    publish_step = \
        os.getenv("GITHUB_WORKFLOW", "") == "publish" \
        or "PEP517_BUILD_BACKEND" not in os.environ

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
        shell=windows,
    )
    if compile_result.returncode != 0:
        print(f"Failed to build target from {dir}. Received return code {compile_result.returncode}")
        exit(1)
