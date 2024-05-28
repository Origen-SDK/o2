import _origen
from contextlib import contextmanager, ContextDecorator
import origen.producer
import origen.helpers
from pathlib import Path
import importlib
from origen.interface import BasicInterface

top_level_flow_open = False


class Producer(_origen.producer.PyProducer):
    def __init__(self):
        _origen.producer.PyProducer.__init__(self)
        self._generate_prepared = False
        self.continue_on_fail = True

    # Defines the methods that are accessible within blocks/<block>/registers.py
    def api(self):
        return {
            "Pattern": self.Pattern,
            "Flow": self.Flow,
        }

    def generate(self, *sources):
        ''' Generate a pattern, either from a file or function source.

            Note: changing the directories, mode, verbosity levels, etc. are not supported here but
            can be changed prior to calling this method.
        '''
        _origen.set_operation("generate")
        # Just do this once, consider a case like the examples command where this is being called
        # multiple times in the same thread of execution
        if not self._generate_prepared:
            origen.tester._prepare_for_generate()
            self._generate_prepared = True
        for (i, src) in enumerate(sources):
            origen.logger.info(
                f"Executing source {i+1} of {len(sources)}: {src}")
            if isinstance(src, Path):
                src = str(src)

            context = origen.producer.api()
            # Starts a new JOB in Origen which provides some long term storage and tracking
            # of files that are referenced on the Rust side
            # The JOB API can be accessed via origen.producer.current_job
            if isinstance(src, str):
                origen.producer.create_job("generate", src)
                origen.load_file(src, locals=context)
            elif callable(src):
                origen.producer.create_job("generate", src.__name__)
                src(context)
            else:
                origen.logger.error(
                    f"Cannot generate source {src} at index {i}. Unrecognized type {type(src)}"
                )
        _origen.prog_gen.render()

    def summarize(self):
        stats = origen.tester.stats()
        changes = stats['changed_pattern_files'] > 0 or stats[
            'changed_program_files'] > 0
        new_files = stats['new_pattern_files'] > 0 or stats[
            'new_program_files'] > 0
        # TODO add this back in when save_ref is re-added
        # if changes or new_files:
        #     print("")
        #     if changes:
        #         print("To save all changed files run:")
        #         print("  origen save_ref --changed")
        #     if new_files:
        #         print("To save all new files run:")
        #         print("  origen save_ref --new")
        #     if changes and new_files:
        #         print("To save both run:")
        #         print("  origen save_ref --new --changed")

    @contextmanager
    def Pattern(self, **kwargs):
        _origen.set_operation("generatepattern")
        # Always freshly load the target when generating a pattern, no matter how much anyone
        # complains about this!
        # It guarantees that produced patterns are always the same regardless of generation
        # order by clearing all DUT state.
        origen.target.load()

        job = origen.producer.current_job
        name = kwargs.pop("name", None) or Path(job.source_file).stem
        pat = PatternClass(name, **kwargs)

        # This initializes a new AST for the pattern we are about to generate
        _origen.start_new_test(pat.name)
        origen.tester.generate_pattern_header(pat.header_comments)

        origen.logger.debug(f"Producing pattern {pat.name} in job {job.id}")
        origen.callbacks.emit("toplevel__startup", kwargs=kwargs)
        origen.callbacks.emit("controller__startup", kwargs=kwargs)
        yield pat
        origen.callbacks.emit("controller__shutdown", kwargs=kwargs)
        origen.callbacks.emit("toplevel__shutdown", kwargs=kwargs)

        origen.tester.end_pattern()
        # True means continue on fail, should make this dynamic in future so that the user can
        # decide whether to blow up upon an error or continue to the next pattern.
        origen.tester.render_pattern(origen.producer.continue_on_fail)

    @contextmanager
    def Flow(self, **kwargs):
        _origen.set_operation("generateflow")
        # Instantiate the app interface
        if origen.interface is None:
            path = f'{_origen.app_config()["name"]}.interface.interface'
            origen.logger.trace(
                f"Looking for application test program interface at {path}")
            try:
                origen.logger.trace(
                    f"Found application interface module, instantiating the Interface class"
                )
                m = importlib.import_module(path)
                origen.interface = m.Interface()
            except ModuleNotFoundError:
                origen.logger.trace(
                    f"Not found, instantiating Origen's basic interface instead"
                )
                origen.interface = BasicInterface()
            except AttributeError:
                origen.logger.trace(
                    f"Not found, instantiating Origen's basic interface instead"
                )
                origen.interface = BasicInterface()

        global top_level_flow_open

        job = origen.producer.current_job
        name = Path(job.current_file).stem
        flow = FlowClass(name, **kwargs)

        if top_level_flow_open:
            top_level = False
            origen.logger.debug(
                f"Producing sub-flow '{flow.name}' in job '{job.id}'")
            flow_refs = _origen.prog_gen.start_new_flow(flow.name,
                                                        sub_flow=True)
        else:
            origen.logger.debug(
                f"Producing flow '{flow.name}' in job '{job.id}'")
            top_level = True
            top_level_flow_open = True
            origen.target.load()
            options = {}
            if kwargs.get(
                    "bypass_sub_flows") or origen.interface.bypass_sub_flows:
                options["bypass_sub_flows"] = True
            if kwargs.get("add_flow_enable"):
                options["add_flow_enable"] = kwargs["add_flow_enable"]
            else:
                options["add_flow_enable"] = origen.interface.add_flow_enable
            flow_refs = _origen.prog_gen.start_new_flow(flow.name, **options)
            origen.interface.top_level_options = kwargs

        #origen.tester.reset()
        #origen.target.reload()
        #origen.tester.clear_dut_dependencies(ast_name=flow.name)
        #origen.tester.generate_pattern_header(flow.header_comments)

        #origen.producer.issue_callback('startup', kwargs)
        yield origen.interface
        #origen.producer.issue_callback('shutdown', kwargs)

        if top_level:
            top_level_flow_open = False

        _origen.prog_gen.end_flow(flow_refs)

        #origen.tester.end_pattern()
        #origen.tester.render()


# (_origen.producer.PyPattern)
class PatternClass:
    def __init__(self, name, **kwargs):
        if name in kwargs:
            # User overwrote the pattern name, or provided one for a sourceless generation
            processed_name = kwargs['name']
        else:
            processed_name = name

        if "prefix" in kwargs:
            processed_name = f"{kwargs['prefix']}_{processed_name}"

        if "postfix" in kwargs:
            processed_name = f"{processed_name}_{kwargs['postfix']}"

        self.name = processed_name

        # Collect the header comments from:
        #  * The application
        #  * <To-do> Current plugin
        #  * <To-do> Other plugins (if necessary)
        #  * Pattern specifics given in the header
        self.header_comments = {}
        if origen.helpers.has_method(origen.app, "pattern_header"):
            self.header_comments["app"] = origen.app.pattern_header(self)

        if "header_comments" in kwargs:
            self.header_comments["pattern"] = kwargs["header_comments"]


# (_origen.producer.PyFlow)
class FlowClass:
    def __init__(self, name, **kwargs):
        if name in kwargs:
            # User overwrote the pattern name, or provided one for a sourceless generation
            processed_name = kwargs['name']
        else:
            processed_name = name

        if "prefix" in kwargs:
            processed_name = f"{kwargs['prefix']}_{processed_name}"

        if "postfix" in kwargs:
            processed_name = f"{processed_name}_{kwargs['postfix']}"

        self.name = processed_name

        # Collect the header comments from:
        #  * The application
        #  * <To-do> Current plugin
        #  * <To-do> Other plugins (if necessary)
        #  * Pattern specifics given in the header
        self.header_comments = {}
        if origen.helpers.has_method(origen.app, "flow_header"):
            self.header_comments["app"] = origen.app.flow_header(self)

        if "header_comments" in kwargs:
            self.header_comments["flow"] = kwargs["header_comments"]
