from . import rst_shared_defs

app = None

def setup(sphinx):
  sphinx.add_config_value('rst_shared_defs', {}, '')
  sphinx.connect('config-inited', rst_shared_defs.generate)
  global app
  app = sphinx

def all_shared():
  retn = []
  if isinstance(app.config.rst_shared_defs, list):
    for defs in app.config.rst_shared_defs:
      retn.append(rst_shared_defs.SharedDefs(app, defs).shared_file)
  else:
    retn.append(rst_shared_defs.SharedDefs(app, app.config.rst_shared_defs).shared_file)
  return retn

def find(d, *, group=None):
  if len(rst_shared_defs.shared_defs(app)) > 1 and not group:
    raise NotImplementedError
  else:
    return rst_shared_defs.shared_defs(app, func='find', func_args=[d])[0]
