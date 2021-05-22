def setenv(config_root, config_name=None, bypass_config_lookup=None):
    import os, inspect, pathlib, sys
    if bypass_config_lookup:
        os.environ['origen_bypass_config_lookup'] = "1"
    if config_name is None:
        config_name = inspect.stack()[1].function
    os.environ['origen_config_paths'] = str(
        config_root.joinpath(f"{config_name}.toml").absolute())
