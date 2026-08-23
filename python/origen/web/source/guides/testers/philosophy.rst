Philosophy
==========

Model Intent Once
-----------------

Origen source should state what the test needs to do, not reproduce a tester
file format. Drive and compare actions, register transactions, test invocations,
limits, bins, and conditions are represented as structured nodes. Renderers own
syntax, escaping, ordering constraints, and native resource layout.

Prefer Shared Flows
-------------------

Start with one flow and one application interface for all testers. Isolate
differences with tester conditions at the smallest useful scope. Copying entire
flows per tester makes validation and product changes drift apart.

Fail On Unsupported Intent
--------------------------

A backend should reject an operation it cannot represent rather than silently
drop it. Applications should treat such failures as capability feedback: change
the shared intent, add a guarded tester-specific implementation, or implement
the missing backend feature.

Generated Output Is A Build Artifact
------------------------------------

Tester files are generated artifacts. Review them through approved references or
semantic regression tests, but keep the model and generation source authoritative.
Avoid hand-editing output that will be overwritten by the next generation run.

Reproducibility
---------------

Pin Origen, plugin, template-library, and tester-tool versions. Commit targets
and configuration required to reproduce a build. A generation change should be
explainable by a source, model, configuration, or dependency change.

Escape Hatches
--------------

Tester-specific APIs are necessary for native test methods, instances, levels,
or flow features. Keep them inside the application interface and expose a
tester-neutral method to flows. This prevents native details from spreading
through product flow source.
