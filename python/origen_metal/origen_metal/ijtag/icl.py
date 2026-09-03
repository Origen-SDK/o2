"""High-performance IEEE 1687-2014 ICL loading and search APIs."""

from origen_metal import _origen_metal

_icl = _origen_metal.ijtag.icl

load = _icl.load
Model = _icl.Model
Instance = _icl.Instance
Port = _icl.Port
ScanRegister = _icl.ScanRegister
DataRegister = _icl.DataRegister
Alias = _icl.Alias
AliasSegment = _icl.AliasSegment
AliasBit = _icl.AliasBit
Connection = _icl.Connection
ConnectionSegment = _icl.ConnectionSegment
BitSelection = _icl.BitSelection
BitValue = _icl.BitValue
InstanceResults = _icl.InstanceResults
PortResults = _icl.PortResults
ScanRegisterResults = _icl.ScanRegisterResults
DataRegisterResults = _icl.DataRegisterResults
RegisterResults = _icl.RegisterResults
AliasResults = _icl.AliasResults
ConnectionResults = _icl.ConnectionResults
AliasSegmentResults = _icl.AliasSegmentResults
AliasBitResults = _icl.AliasBitResults

__all__ = [
    "load",
    "Model",
    "Instance",
    "Port",
    "ScanRegister",
    "DataRegister",
    "Alias",
    "AliasSegment",
    "AliasBit",
    "Connection",
    "ConnectionSegment",
    "BitSelection",
    "BitValue",
    "InstanceResults",
    "PortResults",
    "ScanRegisterResults",
    "DataRegisterResults",
    "RegisterResults",
    "AliasResults",
    "ConnectionResults",
    "AliasSegmentResults",
    "AliasBitResults",
]
