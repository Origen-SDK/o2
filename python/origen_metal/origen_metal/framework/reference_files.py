"""
This module provides an API to implement Origen's reference file system, that is the system
which records when output files are new or whether they have changed vs. a previously generated
reference file, allowing the user to decide when and if to save the latest version as a new
reference.

Under the hood the API fully supports parallel recording of change/new file references, and is
therefore fully compatible with references being recorded when batch processing on a system
like LSF.
"""


def set_save_ref_dir(dir: str):
    """
    **This must be called before using any other function**.
    
    It defines where you want the temporary files (the so called save_refs) to be stored.

    The given directory should not be under revision control.
    """
    ...


def create_changed_ref(key: str, new_file: str, ref_file: str):
    """
    When a change has been detected between a newly created file and reference file, call this
    to record the change.

    The given key can be later used to apply it (copy the new file over to the reference
    file location).
    """
    ...


def create_new_ref(key: str, new_file: str, ref_file: str):
    """
    When a new file has been detected (one that doesn't have an existing reference file to compare to),
    call this to record the new file.

    The given key can be later used to apply it (copy the new file over to the reference
    file location).
    """
    ...


def apply_ref(key: str):
    """
    Apply's a particular change or new file reference by copying the changed/new file over to the
    previously given reference file location.

    The key given to this function should match a key previously given to either 
    origen_metal.framework.create_changed_ref() or origen_metal.framework.create_new_ref()
    """
    ...


def apply_all_changed_refs():
    """
    Apply's all changed references that have been previously registered via origen_metal.framework.create_changed_ref()
    """
    ...


def apply_all_new_refs():
    """
    Apply's all new file references that have been previously registered via origen_metal.framework.create_new_ref()
    """
    ...


def clear_save_refs():
    """
    Clears all previously registered changed or new file references
    """
    ...
