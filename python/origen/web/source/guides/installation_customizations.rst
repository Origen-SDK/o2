Installation Customizations
===========================

Package Sources
---------------

O2 development releases are available from public PyPI, but organizations may
mirror them and plugins on an internal index. Configure pip or UV using your
organization's normal credential mechanism; do not commit credentials to
``pyproject.toml`` or ``origen.toml``.

Application dependency declarations belong in ``pyproject.toml``. Pin Origen and
critical plugins to a reviewed range so a new prerelease does not silently alter
generated output.

Source Development
------------------

Applications can use a local O2 checkout during framework development. Set up
the application environment with the local path according to ``origen env
setup --help`` for the installed release. Rebuild native extensions whenever the
Rust implementation or Python ABI changes.

Python Selection
----------------

``python_cmd`` in ``origen.toml`` can select a Python executable when automatic
discovery is unsuitable. The chosen interpreter must satisfy both Origen and all
application/plugin requirements.

Platform Dependencies
---------------------

Published wheels avoid a local Rust build. Source builds require Rust, a C/C++
linker, OpenSSL development support, and platform libraries such as D-Bus on
some Linux distributions. See :doc:`developers/installation` for the current
source-build procedure.

Offline And Controlled Environments
-----------------------------------

For reproducible or disconnected environments, mirror wheels and Rust crates,
retain lockfiles, and record the exact Python/platform combination. Validate the
same artifacts in CI before distributing them to users.
