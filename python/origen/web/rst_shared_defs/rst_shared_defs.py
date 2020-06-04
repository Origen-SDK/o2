import pathlib
from sphinx.util.logging import getLogger

logger = getLogger('rst_shared_defs')
_shared_defs = None

def shared_defs(app, *, func=None, func_args=[]):
  global _shared_defs
  if _shared_defs is None:
    _shared_defs = list()
    if isinstance(app.config.rst_shared_defs, list):
      for defs in app.config.rst_shared_defs:
        _shared_defs.append(SharedDefs(app, defs))
    else:
      _shared_defs.append(SharedDefs(app, app.config.rst_shared_defs))
  if func:
    retn = list()
    for d in _shared_defs:
      retn.append(getattr(d, func)(*func_args))
    return retn
  else:
    return _shared_defs

def generate(app, config):
  return shared_defs(app, func='generate')

class SharedDefs:
  def __init__(self, app, opts={}):
    self.app = app
    self.opts = opts
    self.output_name = opts.get('output_name', 'shared')
    self.output_dir = pathlib.Path(opts.get('output_dir', app.srcdir)).resolve()
    self.output_dir.mkdir(parents=True, exist_ok=True)

  @property
  def all_targets(self):
    targets = {
      **self.opts.get('refs', {}),
      **self.opts.get('api', {})
    }
    return targets

  def find(self, target):
    _target = target.split(':')
    scope = self.all_targets
    for t in _target[0:-1]:
      scope = scope[t]
    return scope.get(_target[-1], None)

  @property
  def shared_file(self):
    return self.output_dir.joinpath(f"{self.output_name}.rst")

  def new_defs_file(self, name):
    f = open(self.output_dir.joinpath(f"{name}.rst"), 'w')
    f.write('.. Substitution definitions derived from extlinks and references\n')
    f.write('\n')
    f.write(':orphan:\n')
    f.write('\n')
    f.write('.. start-content\n')
    f.write('\n')
    return f

  def generate(self):
    '''
    '''

    def generate_ref(ref, target, f):
      if isinstance(target, str):
        # Direct pointer to the Sphinx reference. Discern the name as anything
        # after the first '#' following any '/'
        f.write(f'.. |{ref}| replace:: {target}\n')
        f.write(f'.. |ref_{ref}| replace:: :ref:`{target}`\n')
      elif isinstance(target, tuple):
        # Interpret a tuple as (name, ref, options={}), where the last is optional
        f.write(f'.. |{ref}| replace:: {ref}\n')
        f.write(f'.. |ref_{ref}| replace:: :ref:`{target[0]} <{target[1]}>`\n')
      else:
        logger.error(f"Unexpected type in reference for '{ref}'. Got '{type(target)}'\n")

    # generate the raw references to references and the reference themselves
    shared_file = self.new_defs_file(self.output_name)
    other_files = {}
    for ref, target in self.opts.get('refs', {}).items():
      if isinstance(target, dict):
        if ref in other_files:
          f = other_files[ref]
        else:
          f = self.new_defs_file(ref)
          other_files[ref] = f
        for r, t in target.items():
          generate_ref(r, t, f)
      else:
        generate_ref(ref, target, shared_file)

    # for t, targets in self.opts.get('api', {}).items():
    #   for n, target in targets.items():
    #     shared_file.write(f'.. |{n}| replace:: {target[1]}')
    #     shared_file.write(f'.. |ref_{n}| replace:: :{t}:`{target[1]} <{target[0]}>`\n')

    shared_file.write('\n.. sub-defines start\n\n')
    for f in other_files.keys():
      shared_file.write(f".. include:: {self.output_dir.joinpath(f)}.rst\n")
      shared_file.write(f"  :start-after: start-content")

    shared_file.close
    for f in other_files.values():
      f.close()
