'''
Definitions for Shorthand's development environment.

This is part of the effort keep Shorthand decoupled from Origen as much as possible...
even though the extension is in the Origen namespace and the docs are reside in
the in the |guides|.
'''

from . import add_defs, app
import origen.helpers


def add_shorthand_dev_defs():
    add_defs(shorthand_dev_defs)


guides_root = 'guides/documenting/shorthand_ext'
m = origen.helpers.mod_from_file(
    origen.web.source_dir.joinpath('_conf/shorthand.py'))
api_root = m.autoapi_root
api_mod_root = 'origen.web.shorthand'

shorthand_dev_defs = {
    'namespace': 'shorthand',
    'api': {
        'api': ('Shorthand API', api_mod_root),
        'add_defs': f'{api_mod_root}.add_defs',
        'anchor_to': f'{api_mod_root}.anchor_to',
        'href_to': f'{api_mod_root}.href_to',
        'link_to': f'{api_mod_root}.link_to',
        'categories_var': f'{api_mod_root}.shorthand.ShorthandDefs.categories'
    },
    'refs': {
        'templating':
        ('Template Helpers', f'{guides_root}:Usage In Templating'),
        'categories':
        f'{guides_root}:Categories',
        'config_var':
        ('shorthand_defs config variable', f'{guides_root}:Basic Usage'),
        'conf_var':
        ('shorthand_defs config variable', f'{guides_root}:Basic Usage'),
        'config_keys': ('shorthand_defs config keys',
                        f'{guides_root}:Other Configuration Keys'),
        'multidefs': ('Using Multiple Shorthand Definitions',
                      f'{guides_root}:Using Multiple Shorthand Definitions'),
        'namespaces':
        f'{guides_root}:Definition Namespaces',
        'project_namespace':
        f'{guides_root}:Project Namespace',
        'basic_usage': ('Basic Usage', f'{guides_root}:Basic Usage'),
    }
}
