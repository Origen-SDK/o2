class DuplicateInstanceError(Exception):
    def __init__(self, obj, klass_alias=None):
        klass = str(obj.__class__).split(
            "'")[1] if klass_alias is None else klass_alias
        if isinstance(klass, str):
            if 'name' in dir(obj):
                Exception.__init__(
                    self,
                    f"Cannot create instance of '{klass}' named '{obj.name}', it already exists!"
                )
            elif 'id' in dir(obj):
                Exception.__init__(
                    self,
                    f"Cannot create instance of '{klass}' with ID '{obj.id}', it already exists!"
                )
            else:
                Exception.__init__(
                    self,
                    f"Cannot create instance of '{klass}', it already exists!")
        else:
            raise TypeError("Class alias must be of type 'str'!")


class UndefinedDataError(Exception):
    def __init__(self, msg):
        Exception.__init__(self, msg)


class FileExtensionError(Exception):
    def __init__(self, ext):
        Exception.__init__(self, f"File extension incorrect, must be '{ext}'!")
