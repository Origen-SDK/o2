# -*- coding: utf-8 -*-
from setuptools import setup

packages = \
['origen_metal',
 'origen_metal._helpers',
 'origen_metal.framework',
 'origen_metal.frontend',
 'origen_metal.utils',
 'origen_metal.utils.revision_control',
 'origen_metal.utils.revision_control.supported']

package_data = \
{'': ['*'], 'origen_metal': ['.pytest_cache/*', '.pytest_cache/v/cache/*']}

install_requires = \
['colorama>=0.4.4', 'importlib-metadata>=6.7.0', 'termcolor>=1.1.0']

extras_require = \
{':sys_platform == "win32"': ['pyreadline3>=3.3,<4.0']}

setup_kwargs = {
    'name': 'origen-metal',
    'version': '0.4.1.dev2',
    'description': 'Bare metal APIs for the Origen SDK',
    'long_description': '',
    'author': 'Origen-SDK',
    'author_email': 'None',
    'maintainer': 'None',
    'maintainer_email': 'None',
    'url': 'https://origen-sdk.org/o2',
    'packages': packages,
    'package_data': package_data,
    'install_requires': install_requires,
    'extras_require': extras_require,
    'python_requires': '>=3.7.0,<3.13',
}
from poetry_build import *
build(setup_kwargs)

setup(**setup_kwargs)
