import origen
import pdb
import pytest
import copy

from origen.generator.processor import *


class MyProcessor(Processor):
    def __init__(self):
        super().__init__()
        self.count = 0

    def on_all(self, node):
        self.count += 1
        return Return.process_children


class CommentUpcaser(Processor):
    def on_comment(self, node):
        update_attribute(node, 1, node["attrs"][1][1].upper())
        return Return.replace, node


class CycleCombiner(Processor):
    def __init__(self):
        super().__init__()
        self.cycle_count = 0

    def consume_cycles(self):
        cyc = node("Cycle", self.cycle_count, True)
        self.cycle_count = 0
        return cyc

    def on_cycle(self, node):
        # if compressable
        if node["attrs"][1][1]:
            self.cycle_count += node["attrs"][1][0]
            return Return.none
        else:
            if self.cycle_count > 0:
                cyc = self.consume_cycles()
                return Return.inline, [cyc, node]
            else:
                return Return.unmodified

    # Don't let it leave an open block with cycles pending
    def on_end_of_block(self, node):
        if self.cycle_count > 0:
            return Return.replace, self.consume_cycles()
        else:
            return Return.none

    # This will be called for all nodes except for cycles
    def on_all(self, node):
        if self.cycle_count == 0:
            return Return.process_children
        else:
            cyc = self.consume_cycles()
            new_node = self.process_children(node)
            return Return.inline, [cyc, new_node]


def test_is_alive():
    p = MyProcessor()

    test = node("Test", "trim_vbgap")
    c = node("Comment", 1, "Hello")
    test["children"].append(c)

    p.process(test)
    assert p.count == 2


def test_comment_upcaser_processor():
    ast = node("Test", "trim_vbgap")
    ast["children"].append(node("Comment", 1, "Hello"))
    reg_trans = node("RegWrite", 10, 0x12345678, None, None)
    reg_trans["children"].append(
        node("Comment", 1, "Should be inside reg transaction"))
    for i in range(5):
        reg_trans["children"].append(node("Cycle", 1, True))
    ast["children"].append(reg_trans)

    orig = copy.deepcopy(ast)

    assert orig == ast

    ast2 = node("Test", "trim_vbgap")
    ast2["children"].append(node("Comment", 1, "HELLO"))
    reg_trans = node("RegWrite", 10, 0x12345678, None, None)
    reg_trans["children"].append(
        node("Comment", 1, "SHOULD BE INSIDE REG TRANSACTION"))
    for i in range(5):
        reg_trans["children"].append(node("Cycle", 1, True))
    ast2["children"].append(reg_trans)

    assert orig != ast2
    assert ast2 == CommentUpcaser().process(ast)
    assert orig == ast


def test_cycle_combiner_processor():
    ast = node("Test", "trim_vbgap")
    ast["children"].append(node("Comment", 1, "Hello"))
    reg_trans = node("RegWrite", 10, 0x12345678, None, None)
    reg_trans["children"].append(
        node("Comment", 1, "Should be inside reg transaction"))
    for i in range(5):
        reg_trans["children"].append(node("Cycle", 1, True))
    ast["children"].append(reg_trans)

    orig = copy.deepcopy(ast)

    assert orig == ast

    ast2 = node("Test", "trim_vbgap")
    ast2["children"].append(node("Comment", 1, "Hello"))
    reg_trans = node("RegWrite", 10, 0x12345678, None, None)
    reg_trans["children"].append(
        node("Comment", 1, "Should be inside reg transaction"))
    reg_trans["children"].append(node("Cycle", 5, True))
    ast2["children"].append(reg_trans)

    assert orig != ast2
    assert ast2 == CycleCombiner().process(ast)
    assert orig == ast
