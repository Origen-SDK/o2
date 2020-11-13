import copy
import enum
import re
import pdb


# All procesor handler methods must return one of these
class Return(enum.Enum):
    # Deletes the node from the output AST.
    #   return Return.none
    none = 1
    # Clones the node (and all of its children) into the output AST. Note that
    # the child nodes are not processed in this case (though they will appear in
    # the output unmodified).
    #   return Return.unmodified
    unmodified = 2
    # Clones the node but replaces it's current children with their
    # processed counterparts in the output AST.
    #   return Return.process_children
    process_children = 3
    # Replace the node in the output AST with the given node.
    #   return Return.replace, new_node
    replace = 4
    # Removes the node and leaves its children in its place.
    #   return Return.unwrap
    unwrap = 5
    # Replace the node in the output AST with the given nodes, the vector wrapper
    # will be removed and the nodes will be placed inline with where the current
    # node is/was.
    #   return Return.inline, [new_node1, new_node2,...]
    inline = 6


class Processor:
    def __init__(self):
        self.snakecaser = re.compile(r'(?<!^)(?=[A-Z])')

    def process(self, node):
        orig_node = node
        node = clone(node)
        on_handler = "on_" + self.snakecaser.sub('_', node["attrs"][0]).lower()
        result = None
        if hasattr(self, on_handler):
            result = getattr(self, on_handler)(node)
            if result is None:
                raise RuntimeError(
                    f"Node handler '{on_handler}' returned None, must return a valid Return code"
                )
        elif hasattr(self, "on_all"):
            result = self.on_all(node)
            if result is None:
                raise RuntimeError(
                    f"Node handler 'on_all' returned None, must return a valid Return code"
                )

        if result is None:
            return self.process_children(orig_node)
        else:
            return self._process_return_code(result, orig_node)

    # Returns a new node which is a copy of the given node with its children replaced
    # by their processed counterparts.
    def process_children(self, node):
        orig_node = node
        node = clone(node)

        if len(node["children"]) == 0:
            return node

        nodes = []

        for child in node["children"]:
            pchild = self.process(child)
            if pchild is not None:
                if pchild["attrs"][0] == "_Inline":
                    for c in pchild["children"]:
                        nodes.append(c)
                else:
                    nodes.append(pchild)

        # Call the end of block handler, giving the processor a chance to do any
        # internal clean up or inject some more nodes at the end
        if hasattr(self, "on_end_of_block"):
            result = self.on_end_of_block(node)
            if result is None:
                raise RuntimeError(
                    f"Node handler 'on_end_of_block' returned None, must return a valid Return code"
                )

            new_node = self._process_return_code(result, orig_node)
            if new_node is not None:
                if new_node["attrs"][0] == "_Inline":
                    for c in new_node["children"]:
                        nodes.append(c)
                else:
                    nodes.append(new_node)

        return replace_children(node, nodes)

    def _process_return_code(self, code, node):
        if type(code) is not tuple:
            code = (code, None)
        if code[0].name == "none":
            return None
        elif code[0].name == "process_children":
            return self.process_children(node)
        elif code[0].name == "unmodified":
            return node
        elif code[0].name == "replace":
            return code[1]
        elif code[0].name == "unwrap":
            # We can't return multiple nodes from this function, so we return them
            # wrapped in a meta-node and the process_children method will identify
            # this and remove the wrapper to inline the contained nodes.
            return inline(copy.copy(node["children"]))
        elif code[0].name == "inline":
            return inline(code[1])
        else:
            raise RuntimeError(f"Unhanded return code: '{code[0].name}'")


def clone(node):
    new_node = copy.copy(node)
    new_node["attrs"] = copy.deepcopy(node["attrs"])
    return new_node


# Update the attribute at the given index with the given value
def update_attribute(node, index, new_value):
    attrs = list(node["attrs"])
    attrs[1][index] = new_value
    node["attrs"] = tuple(attrs)


# Returns a new node which is a copy of the given node with its children replaced
# by the given collection of nodes.
def replace_children(node, nodes):
    new_node = clone(node)
    new_node["children"] = nodes
    return new_node


# Returns a new node which is a copy of the given node with its attrs replaced
# by the given attrs.
def replace_attrs(attrs):
    new_node = clone(node)
    new_node["attrs"] = attrs
    return new_node


def node(*args):
    if len(args) == 1:
        return {'attrs': (args[0], ), 'meta': None, 'children': []}
    elif len(args) == 2:
        return {'attrs': (args[0], args[1]), 'meta': None, 'children': []}
    else:
        return {
            'attrs': (args[0], list(args[1:])),
            'meta': None,
            'children': []
        }


def inline(nodes):
    return {'attrs': ('_Inline'), 'meta': None, 'children': nodes}
