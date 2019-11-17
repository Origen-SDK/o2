use core::python;

pub fn main() {
    python::run("
import _origen;
import code;
code.interact(banner=f\"Origen {_origen.version()}\", local=locals(), exitmsg=\"\")
    ");
}
