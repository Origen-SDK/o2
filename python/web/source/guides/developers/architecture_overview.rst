Architecture Overview
=====================

The Origen framework exists in two parts within a single package: a Python-based `frontend`, which
users will build their applications upon, and a compiled `backend`, which maintains the device,
tester, and other various models. This is reminiscent of the :mvc_dp_wiki:`Model-View-Controller Design Pattern <>` where
the Rust-based backend functions as the ``model``, the user-facing |origen_api| API as the ``view``,
and the hidden |_origen_api| module as the ``controller``, bridging the two.
