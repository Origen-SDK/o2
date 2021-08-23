import pathlib
from sphinx.util.logging import getLogger

shorthand_defs = {}
default_output_dir = None

logger = getLogger('shorthand')

nspace_sep = '~'
''' Separator used for namespacing '''


def all_include_rsts():
    ''' Returns the output files for all RSTs, from all namespaces '''
    rsts = []
    for _nspace, defs in shorthand_defs.items():
        rsts += [defs.output_file]
    return rsts


def split_namespace(target):
    ''' Splits a namespaced target into its namespace and target components '''
    if nspace_sep in target:
        return tuple(target.split(nspace_sep, 1))
    else:
        return (0, target)


def get(target):
    ''' Retrieves a target '''
    nspace, t = split_namespace(target)
    if not nspace in shorthand_defs:
        logger.error(f"Unknown namespace '{nspace}'")
    else:
        return shorthand_defs[nspace].get(t)


def generate(app):
    '''
    Updates the ``shorthand_defs`` with definitions from the user's config and
    generates RST files for all defintion namespaces.
  '''

    global shorthand_defs
    if isinstance(app.config.shorthand_defs, list):
        for defs in app.config.shorthand_defs:
            if 'namespace' in defs:
                shorthand_defs[defs['namespace']] = ShorthandDefs(app, defs)
            else:
                # Only allowing one unnamed (anonymous, main, whatever) definition group
                if 0 in shorthand_defs:
                    logger.error(
                        "An un-namespaced definition group has already been defined. Please namespace any additional definitions!"
                    )
                else:
                    shorthand_defs[0] = ShorthandDefs(app, defs)

        # Need a default namespace, even if its empty
        if not 0 in shorthand_defs:
            shorthand_defs[0] = ShorthandDefs(app, {})
    elif isinstance(app.config.shorthand_defs, dict):
        shorthand_defs[0] = ShorthandDefs(app, app.config.shorthand_defs)
    for _nspace, defs in shorthand_defs.items():
        defs.generate()


def all_from_category(category):
    targets = {}
    for nspace, defs in shorthand_defs.items():
        targets[nspace] = defs.all_from_category(category)
    return targets


class Target:
    ''' Represents a single, unresolved target, in any namespace '''
    def __init__(self, name, category, config, namespace):
        self.namespace = namespace
        self.name = name
        self.category = category
        if isinstance(config, str):
            self.target = config
            self.text = None
            self.opts = None
        else:
            self.text = config[0]
            self.target = config[1]
            self.opts = config[2] if len(config) > 2 else None

    @property
    def is_ref(self):
        return self.category == 'refs'

    @property
    def is_static(self):
        return self.category == 'statics'

    @property
    def is_doc(self):
        return self.category == 'docs'

    @property
    def is_substitution(self):
        return self.category == "substitutions"

    @property
    def is_sub(self):
        return self.is_substitution

    @property
    def is_extlink(self):
        return self.category == "extlinks"

    @property
    def is_abslink(self):
        return self.category == "abslinks"

    @property
    def is_api(self):
        return self.category == "api"

    def generate_substitution(self, f):
        ''' Prints the target's substitution representation into the given file '''
        if self.namespace:
            n = f"{self.namespace}~{self.name}"
        else:
            n = self.name
        if self.is_substitution:
            f.write(f'.. |{n}| replace:: {self.target}\n')
        elif self.is_extlink:
            if self.text:
                f.write(
                    f'.. |{n}| replace:: :{self.target}:`{self.text} <>`\n')
            else:
                f.write(
                    f'.. |{n}| replace:: :{self.target}:`{self.name} <>`\n')
        elif self.is_ref:
            if self.text:
                f.write(
                    f'.. |{n}| replace:: :ref:`{self.text} <{self.target}>`\n')
            else:
                f.write(
                    f'.. |{n}| replace:: :ref:`{self.name} <{self.target}>`\n')
        elif self.is_doc or self.is_static:
            if self.text:
                f.write(
                    f'.. |{n}| replace:: :shorthand-link-to:`{self.text} <{n}>`\n'
                )
            else:
                f.write(
                    f'.. |{n}| replace:: :shorthand-link-to:`{self.name} <{n}>`\n'
                )
        elif self.is_api:
            if self.text:
                f.write(
                    f'.. |{n}| replace:: :shorthand-link-to:`{self.text} <{n}>`\n'
                )
            else:
                f.write(
                    f'.. |{n}| replace:: :shorthand-link-to:`{self.name}, <{n}>`\n'
                )
        elif self.is_abslink:
            if self.text:
                f.write(f'.. |{n}| replace:: `{self.text} <{self.target}>`\n')
            else:
                f.write(f'.. |{n}| replace:: `<{self.target}>`\n')
        else:
            logger.error(
                f"Target '{n}' has unknown category '{self.category}'\n")


class ShorthandDefs:
    categories = [
        'extlinks', 'substitutions', 'statics', 'refs', 'docs', 'abslinks',
        'api'
    ]
    ''' Listing of the recognized categories '''
    def __init__(self, app, opts):
        self.app = app
        self.opts = opts
        self.namespace = self.opts.get('namespace', None)
        self.output_name = self.opts.get(
            'output_name', self.namespace if self.namespace else
            (self.namespace or 'shorthand_defs'))
        self.output_dir = pathlib.Path(self.opts['output_dir']).resolve(
        ) if 'output_dir' in self.opts else None
        self.flattened_targets = self._flatten_targets()

    def _flatten_targets(self):
        all_flattened = {}

        def flatten(targets, category, namespace):
            _flattened = {}

            for name, t in targets.items():
                if namespace is None:
                    n = name
                else:
                    n = namespace + f":{name}"
                if isinstance(t, dict):
                    flatten(t, category, n)
                else:
                    if n in all_flattened:
                        logger.warn(
                            f"Clashing target: '{n}' has already been set to '{all_flattened[n]}'"
                        )
                    all_flattened[n] = Target(n, category, t, self.namespace)

        for c in self.categories:
            targets = self.opts.get(c, {})
            flatten(targets, c, None)
        return all_flattened

    def get(self, target):
        return self.flattened_targets.get(target, None)

    @property
    def output_file(self):
        '''
      Unless specifically indicated otherwise, use the default namespace's output dir.
      Fall back to the app's source dir otherwise.
    '''
        return (self.output_dir or
                (shorthand_defs[0].output_dir if 0 in shorthand_defs else None)
                or default_output_dir or pathlib.Path(
                    self.app.srcdir)).joinpath(f"{self.output_name}.rst")

    def _new_defs_file(self):
        self.output_file.parent.mkdir(parents=True, exist_ok=True)
        f = open(self.output_file, 'w')
        f.write('.. Substitution definitions derived from Shorthand\n')
        f.write('\n')
        f.write(':orphan:\n')
        f.write('\n')
        f.write('.. start-content\n')
        f.write('\n')
        return f

    def generate(self):
        '''
      Generates the output RST file for this group's definitions.
    '''
        rst = self._new_defs_file()
        for _name, target in self.flattened_targets.items():
            target.generate_substitution(rst)
        rst.close()

    def all_from_category(self, category):
        return {
            n: t
            for (n, t) in self.flattened_targets.items()
            if t.category == category
        }
