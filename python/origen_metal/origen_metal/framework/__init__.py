"""
This module provides APIs that are closely related to creating features for an application framework
like Origen, e.g. a logging system.

They are generic enough to allow you to use them to create a similar feature in your own
application framework, but they are not quite as generic as the APIs found in the origen_metal.utils module.
"""

from origen_metal import _origen_metal

file_permissions = _origen_metal.framework.file_permissions

Outcome = _origen_metal.framework.outcomes.Outcome
FilePermissions = _origen_metal.framework.file_permissions.FilePermissions
