use core::python;

pub fn main() {
    python::run(
        "
import origen;
import code;
code.interact(banner=f\"Origen {origen.version}\", local=locals(), exitmsg=\"\")
    ",
    );
}
