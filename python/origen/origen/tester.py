import origen
import _origen
import pickle
from contextlib import contextmanager, ContextDecorator
from origen_metal import _origen_metal


class Tester(_origen.tester.PyTester):
    def __init__(self):
        pass
        #self.db = _origen.tester.PyTester("placeholder")
        #_origen.tester.PyTester.init(self, "placeholder")

    def set_timeset(self, tset):
        # For simplicity, a timeset can be given as a string which is assumed to be a top-level timeset.
        # Due to lazy loading though, its possible that the timesets haven't been instantiated yet, causing
        # a very confusing 'no timeset found' error, yet then using 'dut.timesets' to check shows them as loaded.
        # Load them here, if they haven't already.
        if origen.dut and not origen.dut.timesets_loaded:
            origen.dut.timesets
        return _origen.tester.PyTester.set_timeset(self, tset)

    # Returns stats on the number of patterns generated, etc.
    def stats(self):
        return pickle.loads(bytes(self._stats()))

    @contextmanager
    def eq(self, *names):
        (pat_ref_id, prog_ref_id,
         clean_tester_names) = self._start_eq_block(names)
        testers = []
        for t in clean_tester_names:
            if t == "V93K":
                testers.append(V93K())
            elif t == "V93KSMT7":
                testers.append(V93K(7))
            elif t == "V93KSMT8":
                testers.append(V93K(8))
            elif t == "IGXL":
                testers.append(IGXL())
            elif t == "ULTRAFLEX":
                testers.append(IGXL("ULTRAFLEX"))
            elif t == "J750":
                testers.append(IGXL("J750"))
            else:
                raise Exception(
                    f"The API for tester '{t}' has not been implemented yet!")

        if len(testers) == 1:
            yield testers[0]
        elif len(testers) == 2:
            yield testers[0], testers[1]
        elif len(testers) == 3:
            yield testers[0], testers[1], testers[2]
        elif len(testers) == 4:
            yield testers[0], testers[1], testers[2], testers[3]
        elif len(testers) == 5:
            yield testers[0], testers[1], testers[2], testers[3], testers[4]
        elif len(testers) == 6:
            yield testers[0], testers[1], testers[2], testers[3], testers[
                4], testers[5]
        elif len(testers) == 7:
            yield testers[0], testers[1], testers[2], testers[3], testers[
                4], testers[5], testers[6]
        elif len(testers) == 8:
            yield testers[0], testers[1], testers[2], testers[3], testers[
                4], testers[5], testers[6], testers[7]
        elif len(testers) == 9:
            yield testers[0], testers[1], testers[2], testers[3], testers[
                4], testers[5], testers[6], testers[7], testers[8]
        elif len(testers) == 10:
            yield testers[0], testers[1], testers[2], testers[3], testers[
                4], testers[5], testers[6], testers[7], testers[8], testers[9]
        else:
            raise Exception(
                f"Only up to 10 testers are supported in a with-specific-tester block"
            )
        for t in testers:
            del t
        self._end_eq_block(pat_ref_id, prog_ref_id)

    @contextmanager
    def neq(self, *names):
        (pat_ref_id, prog_ref_id,
         clean_tester_names) = self._start_neq_block(names)
        yield
        self._end_neq_block(pat_ref_id, prog_ref_id)


class DummyTester:
    def __init__(self):
        pass

    def generate(self, ast):
        for i, n in enumerate(ast.nodes):
            print(f"Python Generator: Node: {i}: {n}")


class V93K(_origen_metal.tester_apis.V93K):
    pass


class IGXL(_origen_metal.tester_apis.IGXL):
    def add_dut_pinmap(self, groups=None, pin_type="I/O", comment=""):
        """Populate an IG-XL pin map from the active O2 DUT.

        Physical pins are emitted once. Multi-pin DUT groups are expanded into
        ordered IG-XL group rows. Pass ``groups`` to restrict group generation;
        otherwise all non-trivial DUT groups are included.
        """
        if origen.dut is None:
            raise RuntimeError(
                "Cannot build an IG-XL pin map without an active DUT")

        physical_names = list(origen.dut.physical_pins.keys())
        for name in physical_names:
            self.add_pin(name, pin_type=pin_type, comment=comment)

        requested_groups = set(groups) if groups is not None else None
        for name in origen.dut.pins.keys():
            group = origen.dut.pins[name]
            if group.width <= 1:
                continue
            if requested_groups is not None and name not in requested_groups:
                continue
            for pin_name in group.pin_names:
                self.add_group_pin(
                    name,
                    pin_name,
                    pin_type=pin_type,
                    comment=comment,
                )

    def add_dut_timesets_basic(self, timesets=None, timing_mode="Machine"):
        """Derive UltraFLEX Time Sets (Basic) rows from O2 DUT wavetables.

        A row is generated for every physical pin with applied waves. Conflicting
        event times for the same IG-XL edge are rejected, since silently choosing
        one would change the DUT timing semantics.
        """
        if origen.dut is None:
            raise RuntimeError(
                "Cannot build IG-XL timing without an active DUT")

        selected = set(timesets) if timesets is not None else None
        for timeset_name in origen.dut.timesets.keys():
            if selected is not None and timeset_name not in selected:
                continue
            timeset = origen.dut.timesets[timeset_name]
            for wavetable_name in timeset.wavetables.keys():
                wavetable = timeset.wavetables[wavetable_name]
                period = wavetable.__period__
                if period is None:
                    period = timeset.__period__
                if period is None:
                    period = timeset.default_period
                if period is None:
                    raise RuntimeError(
                        f"No period is defined for {timeset_name}.{wavetable_name}"
                    )

                for pin_name, indicators in wavetable.applied_waves().items():
                    edges = {
                        "drive_on": [],
                        "drive_data": [],
                        "drive_return": [],
                        "drive_off": [],
                        "compare_open": [],
                        "compare_close": [],
                    }
                    for wave in indicators.values():
                        for event in wave.events:
                            at = IGXL._igxl_event_time(event)
                            if event.action in ("DriveHigh", "DriveLow"):
                                edges["drive_data"].append(at)
                            elif event.action == "HighZ":
                                edges["drive_off"].append(at)
                            elif event.action in ("VerifyHigh", "VerifyLow",
                                                  "VerifyZ"):
                                edges["compare_open"].append(at)

                    resolved = {
                        edge: IGXL._single_igxl_edge(
                            values,
                            f"{timeset_name}.{wavetable_name}.{pin_name}.{edge}",
                        )
                        for edge, values in edges.items()
                    }
                    self.add_timeset_basic(
                        timeset_name,
                        str(period),
                        pin_name,
                        setup="i/o",
                        timing_mode=timing_mode,
                        **resolved,
                    )

    @staticmethod
    def _single_igxl_edge(values, description):
        unique = list(dict.fromkeys(values))
        if len(unique) > 1:
            raise RuntimeError(
                f"Conflicting O2 timing events cannot map to one UltraFLEX edge: "
                f"{description} has {unique}")
        return unique[0] if unique else ""

    @staticmethod
    def _igxl_event_time(event):
        at = str(event.__at__)
        unit = event.unit
        if unit and at.replace(".", "", 1).isdigit():
            return f"{at}*{unit}"
        return at
