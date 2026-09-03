from os import PathLike
from typing import Any, Iterator, Optional, Sequence, Union

Path = Union[str, PathLike[str]]

def load(
    path: Path,
    *,
    top: Optional[str] = ...,
    cache_dir: Optional[Path] = ...,
    threads: Optional[int] = ...,
    preserve_comments: bool = ...,
) -> Model: ...

class Model:
    @property
    def root(self) -> Instance: ...
    @property
    def module_count(self) -> int: ...
    @property
    def specialization_count(self) -> int: ...
    @property
    def instance_count(self) -> int: ...
    @property
    def connection_count(self) -> int: ...
    def resolve_path(self, path: str) -> Instance: ...
    def find_instances(self, pattern: str) -> InstanceResults: ...
    def find_instances_of(self, pattern: str) -> InstanceResults: ...
    def find_ports(self, pattern: str) -> PortResults: ...
    def find_scan_registers(self, pattern: str) -> ScanRegisterResults: ...
    def find_data_registers(self, pattern: str) -> DataRegisterResults: ...
    def find_registers(self, pattern: str) -> RegisterResults: ...
    def find_aliases(self, pattern: str) -> AliasResults: ...

class Instance:
    id: int
    name: str
    path: str
    module_type: str
    qualified_module_type: str
    parent: Optional[Instance]
    children: InstanceResults
    ports: PortResults
    scan_registers: ScanRegisterResults
    data_registers: DataRegisterResults
    registers: RegisterResults
    aliases: AliasResults
    def find_instances(self, pattern: str) -> InstanceResults: ...
    def find_instances_of(self, pattern: str) -> InstanceResults: ...
    def find_ports(self, pattern: str) -> PortResults: ...
    def find_scan_registers(self, pattern: str) -> ScanRegisterResults: ...
    def find_data_registers(self, pattern: str) -> DataRegisterResults: ...
    def find_registers(self, pattern: str) -> RegisterResults: ...
    def find_aliases(self, pattern: str) -> AliasResults: ...

class Port:
    id: int
    name: str
    path: str
    kind: str
    width: int
    first_index: int
    last_index: int
    owner: Instance
    active_polarity: Optional[bool]
    default_load_value: Any
    enum_ref: Optional[str]
    connections: ConnectionResults

class ScanRegister:
    id: int
    name: str
    path: str
    kind: str
    width: int
    first_index: int
    last_index: int
    owner: Instance
    default_load_value: Any
    reset_value: Any
    enum_ref: Optional[str]
    connections: ConnectionResults

class DataRegister(ScanRegister): ...

class Alias:
    id: int
    name: str
    path: str
    width: int
    first_index: int
    last_index: int
    owner: Instance
    segments: AliasSegmentResults
    bits: AliasBitResults

class AliasSegment:
    relative_path: list[str]
    target: Any
    selection: BitSelection
    inverted: bool
    alias_bit_offset: int

class AliasBit:
    relative_path: list[str]
    target: Any
    target_index: int
    inverted: bool
    alias_bit_offset: int

class Connection:
    id: int
    kind: str
    source_span: tuple[int, int]
    owner: Any
    segments: ConnectionSegmentResults

class ConnectionSegment:
    relative_path: list[str]
    target: Any
    selection: BitSelection
    inverted: bool

class BitSelection:
    kind: str
    first: Optional[int]
    last: Optional[int]
    descending: Optional[bool]
    width: int

class BitValue:
    width: int
    value: int
    unknown_mask: int
    is_fully_known: bool
    def __int__(self) -> int: ...

class _Results(Sequence[Any]):
    def __iter__(self) -> Iterator[Any]: ...

class InstanceResults(_Results): ...
class PortResults(_Results): ...
class ScanRegisterResults(_Results): ...
class DataRegisterResults(_Results): ...
class RegisterResults(_Results): ...
class AliasResults(_Results): ...
class ConnectionResults(_Results): ...
class AliasSegmentResults(_Results): ...
class AliasBitResults(_Results): ...
