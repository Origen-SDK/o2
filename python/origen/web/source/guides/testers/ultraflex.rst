UltraFLEX
=========

Origen can generate Teradyne UltraFLEX patterns and IG-XL test-program
worksheets. UltraFLEX program generation uses the same tester-neutral flow AST
as the V93K backends; only test-instance and tester-resource construction is
platform-specific.

Selecting the Tester
--------------------

Create an UltraFLEX target in the application:

.. code-block:: python

   # targets/tester/uflex.py
   origen.tester.target("UltraFLEX")

Load it with the DUT through the normal target command:

.. code-block:: shell

   origen target set dut/eagle tester/uflex

The application can then access the UltraFLEX API from a tester condition:

.. code-block:: python

   with tester().eq("uflex") as uflex:
       # Define UltraFLEX resources here.
       pass

Keeping Flows Tester-Neutral
----------------------------

Application flow source should continue to use the common flow API for IDs,
conditions, groups, test numbers, and binning. Tester-specific construction is
best kept in the application's interface layer.

For example, the interface can return either a V93K Test Suite or an UltraFLEX
Test Instance, while the flow remains unchanged:

.. code-block:: python

   test = interface.functional_test("read_array", pattern="read_array")

   flow.add_test(
       test,
       id="read_array",
       number=1000,
       bin=3,
       softbin=100,
   )

Conditions and groups are also tester-neutral:

.. code-block:: python

   with flow.if_job("FT"):
       flow.add_test(test)

   with flow.unless_enable("quick"):
       flow.add_test(test)

   with flow.group("program", id="program_group"):
       flow.add_test(erase)
       flow.add_test(program)
       flow.add_test(verify)

Test Instances
--------------

The generic Test Instance constructor accepts a template library, template
name, and template attributes:

.. code-block:: python

   with tester().eq("uflex") as uflex:
       instance = uflex.new_test_instance(
           "read_array",
           template="functional",
           pattern="read_array_pset",
       )

Convenience constructors are provided for the standard UltraFLEX templates:

.. code-block:: python

   functional = uflex.functional("read_array", pattern="read_array_pset")
   empty = uflex.empty("empty_test")
   other = uflex.other("existing_procedure", proc_name="ExistingProcedure")
   ppmu = uflex.ppmu("leakage", pinlist="data_pins")
   supply = uflex.dcvi_powersupply("standby_current", power_pins="vdd")

``pin_pmu()`` is also available as an alias-style alternative to ``ppmu()``.

Custom VB DLL instances can be created with:

.. code-block:: python

   custom = uflex.new_custom_test_instance(
       "trim_test",
       proc_name="MyTrimProcedure",
       arg0="TrimCode,Result",
       arg1="trim_code",
       arg2="result",
   )

Measurement mode and CPU wait flags are configured on the instance:

.. code-block:: python

   ppmu.set_measure_mode("current")
   ppmu.set_wait_flags("a", "c")

Accepted measurement modes are ``current``/``fvmi`` and
``voltage``/``fimv``.

Instance Groups and Versions
----------------------------

Use a Test Instance group when multiple configurations share one logical Test
Instance name:

.. code-block:: python

   with uflex.test_instance_group("program_flash"):
       slow = uflex.functional("program_flash_slow", pattern="slow_pset")
       fast = uflex.functional("program_flash_fast", pattern="fast_pset")

Distinct configurations are rendered deterministically as
``program_flash_v1``, ``program_flash_v2``, and so on. Identical definitions
are deduplicated.

Limits
------

Direct low and high limits are supported:

.. code-block:: python

   instance.set_lo_limit(-2)
   instance.set_hi_limit(2)

Multiple named limits generate ``Test-defer-limits`` and ``Use-Limit`` rows:

.. code-block:: python

   instance.add_limit("low_current", number=1001, lo=-2, hi=2)
   instance.add_limit("high_current", number=1002, lo=-1, hi=1)

UltraFLEX provides one Units column shared by the low and high limits. Origen
reports an error if both limits specify different non-empty units.

Pattern Resources
-----------------

Create a Pattern Set with one or more pattern files:

.. code-block:: python

   pset = uflex.new_patset(
       "read_array_pset",
       patterns=["read_array.PAT", "nvm_global_subs.PAT"],
   )

Pattern Subroutine resources are created with ``new_patsubr()``:

.. code-block:: python

   uflex.new_patsubr(
       "global_subroutines",
       patterns=["nvm_global_subs.PAT", "trim_subs.PAT"],
   )

All referenced patterns are normalized, sorted, deduplicated, and written to
``referenced.list``.

Program Resources
-----------------

The UltraFLEX interface provides APIs for the main IG-XL resource worksheets:

.. list-table:: UltraFLEX resource APIs
   :header-rows: 1

   * - Resource
     - API
   * - References
     - ``add_reference()``
   * - Job List
     - ``new_job()``
   * - Global Specs
     - ``add_global_spec()``
   * - AC Specs
     - ``add_ac_spec()``
   * - DC Specs
     - ``add_dc_spec()``
   * - Pin Map
     - ``add_pin()``, ``add_power_pin()``, ``add_utility_pin()``, ``add_group_pin()``
   * - Pin Levels
     - ``add_level()``
   * - Edge Sets
     - ``add_edgeset()``
   * - Time Sets
     - ``add_timeset()``
   * - Time Sets (Basic)
     - ``add_timeset_basic()``

For example:

.. code-block:: python

   with tester().eq("uflex") as uflex:
       uflex.add_reference(r".\inc\utility.xla", comment="Utility library")

       uflex.new_job(
           "FT",
           pinmap="pinmap_test",
           instances=["prb1_instances", "global_instances"],
           flows="prb1_flow",
           ac_specs="SpecsAC_func",
           dc_specs="SpecsDC_func",
           patsets=["prb1_patsets", "global_patsets"],
       )

       uflex.add_ac_spec(
           "cycle",
           "func_100MHz",
           selector="nom",
           typ=10e-9,
           min=9e-9,
           max=11e-9,
       )

       uflex.add_dc_spec(
           "vdd_main",
           "power_up",
           selector="nom",
           typ=0.9,
           min=0.8,
           max=1.0,
       )

Numeric AC/DC values are converted to IG-XL engineering-unit expressions. For
example, ``10e-9`` becomes ``=10*ns`` and an ``ioh`` value of ``0.002`` becomes
``=2*mA``.

Resource Filenames
------------------

Name an individual worksheet family with ``set_resource_filename()``:

.. code-block:: python

   uflex.set_resource_filename("references", "Refs")
   uflex.set_resource_filename("jobs", "Jobs")
   uflex.set_resource_filename("ac_specs", "SpecsAC_func")

These produce ``Refs.txt``, ``Jobs.txt``, and ``SpecsAC_func.txt``.

Assign multiple resource families to one workbook through the generic flow API:

.. code-block:: python

   flow.set_resources_filename("shared")

All applicable worksheet sections are then concatenated into ``shared.txt``.
Resources from multiple source flows are collected before the workbook is
written, so later flows do not overwrite earlier resources.

DUT-Derived Resources
---------------------

The Pin Map can be populated from the active O2 DUT:

.. code-block:: python

   uflex.add_dut_pinmap(groups=["JTAG", "DATA"])

This emits physical pins once and expands the selected multi-pin groups.

Time Sets (Basic) can be derived from O2 timesets and applied wavetable events:

.. code-block:: python

   uflex.add_dut_timesets_basic(timesets=["functional", "scan"])

Origen maps drive and verify events to UltraFLEX drive and compare edges. If
multiple O2 events conflict for one IG-XL edge, generation reports an error
instead of choosing one silently.

O2 does not currently expose a tester-neutral electrical-level model, so Pin
Level rows are defined explicitly with ``add_level()``.

Generated Files
---------------

A typical UltraFLEX program can generate:

.. code-block:: text

   prb1_flow.txt
   prb1_instances.txt
   prb1_patsets.txt
   prb1_patsubrs.txt
   prb1_patgroups.txt
   referenced.list
   shared.txt

Resource families with independent names produce files such as ``Refs.txt``,
``Jobs.txt``, ``SpecsAC_func.txt``, and ``SpecsDC_func.txt``.

Unsupported Operations
----------------------

The UltraFLEX backend supports the generic flow operations that have an IG-XL
equivalent. If a flow contains an operation with no UltraFLEX equivalent,
generation reports an error rather than silently dropping the operation.

Generating the Program
----------------------

After selecting the UltraFLEX target, use the normal generation command:

.. code-block:: shell

   origen generate path/to/flow.py

No UltraFLEX-specific generation command is required.
