# TODO see about removing pytest dependency
import abc, pytest
from abc import abstractproperty
import origen_metal


class DataStoreView(abc.ABC):
    @property
    def frontend(self):
        origen_metal._origen_metal.frontend.initialize() is True
        return origen_metal._origen_metal.frontend.frontend()

    @property
    def ds_test_name(self):
        return f"data_store_view_test__{self.data_store_class.__name__}"

    @property
    def cat_test_name(self):
        return f"data_store_view_test_cat__{self.data_store_class.__name__}"

    @abstractproperty
    def data_store_class(self):
        pass

    @property
    def ds_api_class(self):
        return origen_metal.frontend.DataStoreAPI

    @property
    def ds(self):
        return self.cat[self.ds_test_name]

    @property
    def cat(self):
        return self.frontend.data_stores[self.cat_test_name]

    def add_ds(self):
        self.cat.add(self.ds_test_name, self.data_store_class,
                     self.params.get("init_args", None),
                     self.params.get("init_kwargs", None),
                     **self.params.get("add_ds_opts", {}))

    def add_cat(self):
        self.frontend.data_stores.add_category(self.cat_test_name)

    def boot(self):
        self.params = self.parameterize()
        self.add_cat()
        self.add_ds()

    def setup_test(self):
        print("setting up test")

    def teardown_test(self):
        print("tearing down test")

    @property
    def inst_booted(self):
        return getattr(self.__class__, "_inst_booted_")

    @pytest.fixture(autouse=True)
    def wrap_test(self):
        if not self.inst_booted:
            self.boot()
            setattr(self.__class__, "_inst_booted_", True)
        self.setup_test()
        yield
        self.teardown_test()

    @classmethod
    def setup_class(cls):
        setattr(cls, "_inst_booted_", False)

    def parameterize(self):
        return {}

    def test_class_inherits_from_data_store_api(self):
        assert isinstance(self.ds, self.ds_api_class)

    def test_name_can_be_retrieved(self):
        assert self.ds.name == self.ds_test_name

    def test_category_can_be_retrieved(self):
        assert self.ds.category == self.cat

    # TODO Add more view suppport

    # def test_ds_can_be_marked_as_stale(self):
    #     fail

    # def test_ds_can_be_marked_as_orphaned(self):
    #     fail

    # def test_get_operation(self):

    # def test_store_operation(self):
    #     fail

    # def test_remove_operation(self):
    #     fail

    # class TestDictLike:
    #     ...
