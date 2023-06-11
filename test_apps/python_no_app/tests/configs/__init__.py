from tests import python_app_shared
with python_app_shared():
    from python_app_tests._shared.for_proc import setenv

def test_config_dir_from_env_is_added(q, options):
    setenv(options['config_dir'], config_name=False)

    import origen
    q.put(('files', origen.__config_metadata__['files']))

def test_direct_config_from_env_is_added(q, options):
    setenv(options['config_toml'], config_name=False)

    import origen
    q.put(('files', origen.__config_metadata__['files']))

def test_multiple_configs_from_env_are_added(q, options):
    setenv(options['configs'], config_name=False)

    import origen
    q.put(('files', origen.__config_metadata__['files']))

def test_relative_config_from_env_is_added(q, options):
    setenv(options['configs'], config_name=False)

    import origen
    q.put(('files', origen.__config_metadata__['files']))

def test_error_on_non_toml_config_in_env(q, options):
    setenv(options['configs'], config_name=False)

    import origen
    q.put(('files', origen.__config_metadata__['files']))

def test_plugins_are_collected_by_default(q, options):
    setenv(None, bypass_config_lookup=True)

    import origen
    q.put(('configs', origen.__config_metadata__['files']))
    q.put(('plugins', origen.plugins.names))

def test_suppressing_plugin_collection(q, options):
    setenv(options['configs'], config_name=False, bypass_config_lookup=True)

    import origen
    q.put(('configs', origen.__config_metadata__['files']))
    q.put(('plugins', origen.plugins.names))

def test_enumerating_plugins_to_load(q, options):
    setenv(options['configs'], config_name=False, bypass_config_lookup=True)

    import origen
    q.put(('configs', origen.__config_metadata__['files']))
    q.put(('plugins', origen.plugins.names))
