from origen_metal import _origen_metal
_Mailer = _origen_metal.utils.mailer.Mailer

Maillist = _origen_metal.utils.mailer.Maillist
Maillists = _origen_metal.utils.mailer.Maillists

class Mailer(_Mailer):
    def __init__(self, *args, **kwargs):
        _Mailer.__init__(self)
