def setenv(config_root, config_name=None, bypass_config_lookup=None, cd=None):
    import os, inspect, pathlib, sys
    if bypass_config_lookup:
        os.environ['origen_bypass_config_lookup'] = "1"
    if config_root is not None:
        if isinstance(config_root, list):
            os.environ['origen_config_paths'] = os.pathsep.join([str(c) for c in config_root])
        else:
            if config_name is None:
                config_name = inspect.stack()[1].function
            if config_name:
                os.environ['origen_config_paths'] = str(config_root.joinpath(f"{config_name}.toml").absolute())
            else:
                if isinstance(config_root, pathlib.Path):
                    os.environ['origen_config_paths'] = str(config_root.absolute())
                else:
                    os.environ['origen_config_paths'] = config_root

    if cd:
        os.chdir(str(cd))