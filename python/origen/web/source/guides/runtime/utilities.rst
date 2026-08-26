Utilities
=========

Runtime utilities provide shared services used across applications and plugins.
The logger is documented below; user, session, LDAP, mailer, revision-control,
and publishing utilities are covered in :doc:`../utilities` and the generated
API reference.

Prefer these services over application-specific global state. They integrate
with Origen configuration, user identity, verbosity, and workspace paths.

.. toctree::

  utilities/logger
