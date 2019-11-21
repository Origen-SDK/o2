use core::python;

pub fn main() {
    python::run(
        "
import origen;
import code;
from origen import dut, tester;
code.interact(banner=f\"Origen {origen.version}\", local=locals(), exitmsg=\"\")
    ",
    );
}
