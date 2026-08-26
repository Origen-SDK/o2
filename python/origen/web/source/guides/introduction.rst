Introduction
============

Origen 2 (O2) is a semiconductor development framework for describing devices,
generating tester patterns, and producing tester program resources from a shared
model. Application code is written in Python while performance-sensitive models
and generators are implemented in Rust and exposed through Python.

An O2 application normally contains:

* a device-under-test (DUT) model composed of blocks, registers, pins, and timing;
* targets which select a DUT, tester, and environment;
* pattern and test-program sources;
* plugins for reusable device or test content;
* tests and generated reference files; and
* a Sphinx documentation application.

O2 separates intent from output format. A pattern describes operations such as
driving a pin or verifying a register without embedding a tester file format in
the source. The active tester translates those operations into its native output.
The same principle applies to tester-neutral test-program flows.

Where To Start
--------------

New users should follow :doc:`getting_started` in order:

#. install or build O2 and verify the CLI;
#. understand workspaces, applications, and targets;
#. create an application;
#. configure its DUT and tester; and
#. generate and inspect output.

Use :doc:`device_modeling` for the DUT model, :doc:`testers` for pattern and
program generation, and the generated API reference when implementing code.

Project Status
--------------

O2 is under active development. Some APIs and generated formats are more mature
than others. The supported-tester guide calls out implemented backends, while
individual tester pages document known limitations. Pin application dependencies
and keep generated output under regression test when adopting a development
release.
