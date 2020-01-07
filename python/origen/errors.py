class DuplicateInstanceError(Exception):
    def __init__(self, obj, klass_alias = None):
        klass = str(obj.__class__).split("'")[1] if klass_alias is None else klass_alias
        if isinstance(klass, str):
            Exception.__init__(self, f"Cannot create instance of '{klass}' named '{obj.name}', it already exists!")
        else:
            raise TypeError("Class alias must be of type 'str'!")