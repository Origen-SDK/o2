from pathlib import Path

import pytest

from origen_metal.ijtag import icl


DUMMY_ICL = r'''
Module Leaf {
    Parameter WIDTH = 4;
    ScanInPort data[$WIDTH-1:0];
    DataOutPort result[$WIDTH-1:0] { Source inverted; }
    LogicSignal inverted { ~data[0]; }
    ScanRegister status[$WIDTH-1:0] {
        ScanInSource data[0];
        ResetValue 4'b0011;
    }
    DataRegister config[$WIDTH-1:0] { ResetValue 4'b0000; }
    Alias sparse[2:0] = status[3], status[1], config[0];
    Alias nested[1:0] = sparse[2], sparse[0];
}
Module Top {
    ScanInPort top_port;
    Instance left Of Leaf { Parameter WIDTH = 4; InputPort data = top_port; }
    Instance right Of Leaf { Parameter WIDTH = 4; InputPort data = top_port; }
    Alias cross[1:0] = left.nested[1], right.config[0];
}
'''


@pytest.fixture
def model(tmp_path):
    source = tmp_path / "dummy.icl"
    source.write_text(DUMMY_ICL)
    return icl.load(source)


def test_load_and_navigate(model):
    assert model.module_count == 2
    assert model.specialization_count == 2
    assert model.instance_count == 3
    assert model.root.path == "Top"
    assert model.resolve_path("Top.left").module_type == "Leaf"
    assert model.resolve_path("left").path == "Top.left"


def test_lazy_results_and_globs(model):
    instances = model.find_instances("*")
    assert len(instances) == 3
    assert instances[-1].path == "Top.right"
    assert [instance.name for instance in instances[1:]] == ["left", "right"]
    assert len(model.find_instances_of("L*")) == 2
    assert len(model.find_ports("[dr]*")) == 4
    assert len(model.find_scan_registers("status")) == 2
    assert len(model.find_data_registers("config")) == 2
    assert len(model.find_registers("*")) == 4
    assert len(model.find_aliases("*")) == 5
    with pytest.raises(RuntimeError, match="Invalid name pattern"):
        model.find_ports("[")


def test_scope_registers_ports_and_connections(model):
    leaf = model.resolve_path("left")
    assert len(leaf.children) == 0
    assert len(leaf.ports) == 2
    assert len(leaf.scan_registers) == 1
    assert len(leaf.data_registers) == 1
    assert len(leaf.registers) == 2
    assert len(leaf.find_ports("d*")) == 1
    scan_register = leaf.scan_registers[0]
    assert isinstance(scan_register, icl.ScanRegister)
    assert scan_register.width == 4
    assert int(scan_register.reset_value) == 3
    assert len(scan_register.connections) == 1


def test_internal_signal_and_connection_segment_details(model):
    result = model.resolve_path("left").find_ports("result")[0]
    connection = result.connections[0]
    assert connection.kind == "source"
    assert isinstance(connection.source_span, tuple)
    segment = connection.segments[0]
    assert segment.relative_path == []
    assert isinstance(segment.target, icl.InternalSignal)
    assert segment.target.name == "inverted"
    assert segment.target.owner.path == "Top.left"
    assert segment.selection.kind == "whole"
    assert not segment.inverted


def test_non_contiguous_and_nested_aliases(model):
    sparse = model.resolve_path("left").find_aliases("sparse")[0]
    assert sparse.width == 3
    assert [bit.target_index for bit in sparse.bits] == [3, 1, 0]
    cross = model.find_aliases("cross")[0]
    assert cross.width == 2
    assert [bit.target_index for bit in cross.bits] == [3, 0]
    assert [bit.relative_path for bit in cross.bits] == [["left"], ["right"]]


def test_cache_directory_is_optional_and_opaque(tmp_path):
    source = tmp_path / "inputs" / "dummy.icl"
    source.parent.mkdir()
    source.write_text(DUMMY_ICL)
    cache_dir = tmp_path / "generated" / "icl_cache"

    uncached = icl.load(source)
    assert not cache_dir.exists()
    assert uncached.instance_count == 3

    cached = icl.load(source, cache_dir=cache_dir)
    artifacts = list(cache_dir.iterdir())
    assert cached.instance_count == 3
    assert len(artifacts) == 1
    assert artifacts[0].name.startswith("dummy-")
    assert artifacts[0].suffix == ".o2-icl-cache"

    source.write_text("Module Changed { ScanInPort first; ScanOutPort second; }")
    refreshed = icl.load(source, cache_dir=cache_dir)
    assert refreshed.root.name == "Changed"
    assert len(refreshed.find_ports("*")) == 2


def test_load_errors_include_file_context(tmp_path):
    with pytest.raises(RuntimeError, match="File does not exist"):
        icl.load(tmp_path / "missing.icl")

    malformed = tmp_path / "malformed.icl"
    malformed.write_text("Module Broken { ScanInPort missing_terminator }")
    with pytest.raises(RuntimeError, match="malformed.icl"):
        icl.load(malformed)


def test_relative_includes_and_transitive_cache_invalidation(tmp_path):
    child = tmp_path / "blocks" / "child.icl"
    child.parent.mkdir()
    child.write_text("Module Child { ScanInPort first; }\n")
    top = tmp_path / "top.icl"
    top.write_text(
        '#include "blocks/child.icl"\n'
        "Module Top { Instance child Of Child; }\n"
    )
    cache = tmp_path / "cache"

    first = icl.load(top, cache_dir=cache)
    assert len(first.resolve_path("child").ports) == 1

    child.write_text("Module Child { ScanInPort first; ScanInPort second; }\n")
    refreshed = icl.load(top, cache_dir=cache)
    assert len(refreshed.resolve_path("child").ports) == 2


def test_qualified_namespace_resolution(tmp_path):
    source = tmp_path / "namespaces.icl"
    source.write_text(
        "NameSpace Vendor;\n"
        "Module Leaf { ScanInPort vendor; }\n"
        "NameSpace;\n"
        "Module Leaf { ScanInPort root; }\n"
        "Module Top {\n"
        "  Instance vendor Of Vendor::Leaf;\n"
        "  Instance root Of Leaf;\n"
        "}\n"
    )
    model = icl.load(source, top="Top")
    assert model.resolve_path("vendor").qualified_module_type == "Vendor::Leaf"
    assert model.resolve_path("root").qualified_module_type == "Leaf"
    assert len(model.find_instances_of("Vendor::Leaf")) == 1
