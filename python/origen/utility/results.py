import _origen


class GenericResult(_origen.utility.results.GenericResult):
    def __init__(self, **kwargs):
        _origen.utility.results.GenericResult.__init__(self, **kwargs)


class BuildResult(_origen.utility.results.BuildResult):
    def __init__(self, **kwargs):
        _origen.utility.results.BuildResult.__init__(self, **kwargs)


class UploadResult(_origen.utility.results.UploadResult):
    def __init__(self, passed, message=None, metadata=None):
        _origen.utility.results.UploadResult.__init__(self, passed, message,
                                                      metadata)
