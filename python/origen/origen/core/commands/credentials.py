import _origen, origen
from . import is_subcmd, unsupported_subcmd

def run(args):
    _origen.set_operation("credentials")
    datasets = args.get("datasets", None)
    all = args.get("all", False)
    if is_subcmd("set"):
        if all:
            origen.logger.display("Setting passwords for all available datasets...")
            for d in origen.current_user.datasets.values():
                d.password = None
                d.password()
            origen.logger.display("Done!")
        elif datasets is None:
            origen.current_user.password = None
            origen.current_user.password
        else:
            for d in datasets:
                d.password = None
                d.password()
    elif is_subcmd("clear"):
        if all:
            origen.logger.display("Clearing all cached passwords...")
            origen.current_user.clear_cached_passwords()
        elif datasets is None:
            origen.logger.display("Clearing cached password for topmost dataset...")
            origen.current_user.clear_cached_password()
        else:
            for d in datasets:
                origen.logger.display(f"Clearing cached password for dataset '{d}'")
                origen.current_user.datasets[d].clear_cached_password()
        origen.logger.display("Done!")
    else:
        unsupported_subcmd()
