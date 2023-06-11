def try_home_dir(q, options):
    import os
    os.environ.pop("HOME", None)
    os.environ.pop("USERPROFILE", None)

    import origen_metal
    origen_metal.users.lookup_current_id(update_current=True)
    origen_metal.users.current_user.set_home_dir()
