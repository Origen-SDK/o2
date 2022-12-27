# FOR_PR not sure if needed
import origen

def run(**kwargs):
    cc = origen.current_command
    print(f"Class: {cc.__class__.__name__}")
    print(f"Command: {cc.command}")
    print(f"SubCommands: {cc.subcommands}")
    print(f"Args: {cc.args}")
