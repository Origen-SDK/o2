{% set origen_exec = '../rust/origen/target/debug/origen.exe' if origen.running_on_windows else '../rust/origen/target/debug/origen' -%}
{% set run_in_shell = false if origen.running_on_windows else true -%}

Utilities
=========

Origen comes with some utilities to integrate commonly-used tasks that occur in industry or larger-scale use cases with your Origen apps.

Users
-----

Origen provides a representation of users to ease interactions with articles like passwords,
home directories, and email addresses.

Just by booting up Origen, the current user is always available at :meth:`origen.current_user() <_origen.users.current_user>`. The user's ID, which is  :link-to:`discerned from the OS <rust_whoami>`, can be read using the :meth:`_origen.users.User.id` method.

Additional users can be added with :meth:`origen.users.add('\<id\>') <_origen.users.Users.add>`. Given any ID, this creates a new :class:`User <_origen.users.User>` and stores it in the dict-like container, :class:`Users <_origen.users.Users>`, in a way which both the frontend and backend can access and query fields. You can see all added users with :meth:`origen.users.users <_origen.users.users>`.

A user comes with a number of fields that can be set and queried at will:

{% for d in origen.users.data_fields %}
* :meth:`{{d}} <_origen.users.User.{{d}}>`
{% endfor -%}

The :meth:`username <_origen.users.User.username>` and :meth:`display_name <_origen.users.User.display_name>` fields act a bit differently. If they are unset, a backup is returned instead. See the API for details:

* :meth:`username <_origen.users.User.username>`
* :meth:`display_name <_origen.users.User.display_name>`

Passwords work in much the same way with the caveat of an unset for the current user triggers a dialog prompting the user to enter their password. Non-current users will need to explicitly set passwords with the setter method, ``password=``.

``Users`` can be customized by the |origen_config|. The below sections will go through some of these, or see the |example_users_configuration| in the repository.

Datasets
^^^^^^^^

For cases where Origen acts not just as a pattern or program generation tool but as a more full-fledged application or tool distribution system, a single user may be attached to multiple systems.

Consider an application which builds a workspace from two independent systems: an internal Bitbucket system, and the external Github system, with the intention of building a fully-functioning development workspace, requiring proper credentials for both systems simultaneously.

For this, two sets of credentials are needed, as the BitBucket credentials need not match that same user's Github credentials. :class:`Datasets <_origen.users.Dataset>` allow for a single user to set and query fields independent of each other. For example:

.. code:: python

    origen.current_user().datasets['git'].password = "git"
    origen.current_user().datasets['bitbucket'].password = "bb"
    origen.current_user().datasets['git'].password
        #=> "git"
    origen.current_user().datasets['bitbucket'].password
        #=> "bb"

Using the |origen_config|, we can indicate that additional datasets are needed:

.. code:: toml

    [user__datasets]
    [user__datasets.git]
    [user__datasets.bitbucket]

These are then available on the |dict-like| container :meth:`datasets <_origen.users.Datasets` and our example application can interact with these two systems as the same user with two different sets of credentials.

.. code:: python

    from somewhere import access_git, access_bitbucket

    git_dataset = origen.current_user().datasets['git']
    bb_dataset = origen.current_user().datasets['bitbucket']

    access_git(git_dataset.username, git_dataset.password)
    access_git(bb_dataset.username, bb_dataset.password)

Each dataset can be configured independently from the |origen_config|. This will be most applicable when handling :ref:`data integration <guides/utilities:Data Integration>`.

You may also notice the ``default`` dataset present. When a user field is accessed on any ``User``, instead of a ``dataset``, the ``default dataset`` is actually what gets queried. The default dataset can be changed the |origen_config|:

.. code:: python

    # Before configuration
    origen.current_user().password = "p"
    origen.current_user().password
        #=> "p"

    origen.current_user().datasets['git'].password = "git"
    origen.current_user().datasets['git'].password
        #=> "git"

    origen.current_user().password
        #=> "p"
    origen.current_user().datasets['default'].password = "pw"
    origen.current_user().password
        #=> "pw"

.. code:: toml

    user__default_dataset = "test"

    [user__datasets]
    [user__datasets.git]

.. code:: python

    origen.current_user().datasets['git'].password = "git"
    origen.current_user().datasets['git'].password
        #=> "git"
    origen.current_user().password
        #=> "git"

    origen.current_user().datasets['git'].password = "git_pw"
    origen.current_user().password
        #=> "git_pw"

    origen.current_user().datasets['default'].password
        #=> Error
        #=> "no dataset default is available"

Notice that when a default dataset is specified, Origen will not create its own ``default`` dataset and attempts to access it without having explicitly added it will raise exceptions.

Data Integration
^^^^^^^^^^^^^^^^

User's can be integrated with supported ``data-sources``. Currently, the only supported source is the |origen_utilities:ldap|, but more may be added in the future.

All data integration in done via the |origen_config|. The ``data_source`` option denotes what other options are available and how they are used.

LDAP Integration
&&&&&&&&&&&&&&&&

These options only pertain to integrating an existing |origen_utilities:ldap| configuration with a user dataset. See the |origen_utilities:ldap| section for setting up the LDAP itself.

Integrating a LDAP is done per-dataset where the ldap's name and various lookup parameters are given. A ``dataset_mappings`` table will map the LDAP's ``attribute`` to the available  ``fields``.

.. code:: toml

    # Add user datasets
    [user__datasets]

    # Add two blank datasets
    [user__datasets.git]
    [user__datasets.bitbucket]

    # Add a dataset with LDAP integration
    [user__datasets.ldap]
    data_source = "ldap"        # Indicates LDAP data source
    data_lookup = "ldap"        # The LDAP name.
                                # The LDAP itself is configured elsewhere
    data_id = "uid"             # When searching the LDAP, indicates what attribute
                                # should be used during lookup.
    data_service_user = "ldap"  # Indicates if a service user should be used to
                                # search the LDAP. The service users are also
                                # configured elsewhere.
    try_password = true         # When a password is retrieved, attempted to validate it
                                # against the LDAP (e.g., attempt to bind with the "data_id"
                                # and given password.
    auto_populate = false       # Indicate if the LDAP should populate
                                # the user field at initialization.

|the_users_tests| contains a setup and some tests against a :link-to:`freely available LDAP<ldap:test_server>` and can be used as an example and a reference.

More On Passwords
^^^^^^^^^^^^^^^^^

Password Caching
&&&&&&&&&&&&&&&&

By default, users who have had their passwords set and validated will have their passwords stored in the |origen_utilities:session_store| for future retrieval. Passwords are stored encrypted using Origen's |default_encryption_key| and |default_encryption_nonce| (so this is really just to avoid plaintext password as an actual security mechanism) but can be overridden by the |origen_config|.

.. code:: python

    # Allows passwords to be stored in the user's session store
    user__cache_passwords = false

    # Allows custom encryption keys used by passwords only
    # These must conform to AES-256 GCM standards
    password_encryption_key = "..."
    password_encryption_nonce = "..."

Passwords are stored in the ``user_session``, so retrieval will persists not just across invocations but across applications and are available in the global space.

Passwords for the current user can be set and cleared on the command line as well using the ``credentials`` command:

{{ insert_cmd_output(origen_exec + " credentials --help", shell=run_in_shell) }}

Password "Reasons"
&&&&&&&&&&&&&&&&&&

Returning :link-to:`to the example from Datasets <origen_utilities:user_datasets>` momentarily, recall that password datasets can be retrieved on a per-dataset basis. There is an alternative though: passwords can be retrieved for the given *reason*, which will attempt to match an arbitrary ``string`` with its corresponding dataset.

These "reasons" are set in the |origen_config| and are retrieved by passing the reason
into the :meth:`password_for <_origen.users.User.password_for>` method. Without other options, this will raise an exception if the password reason is not found. :meth:`dataset_for <_origen.users.User.dataset_for>` can query if a dataset matches the given reason.

.. code:: toml

    [user__password_reasons]
    "just because" = "git"

.. code:: python

    origen.current_user().default_dataset
        #=> "bitbucket"
    origen.current_user().dataset["git"].password = "git_pw"
    origen.current_user().dataset["bitbucket"].password = "bb_pw"

    origen.current_user().password_for("just because")
        #=> "git_pw"

    origen.current_user().password_for("no reason")
        #=> Error

A ``default dataset`` option will return the password for that dataset in the event the reason is unmatched. The special value ``None`` can also be given to return the global default dataset:

.. code:: python

    origen.current_user().password_for("no reason", default: "git")
        #=> "git_pw"

    origen.current_user().password_for("no reason", default: None)
        #=> "bb_pw"

Password Validation
&&&&&&&&&&&&&&&&&&&

If a ``data_source`` is available, passwords can be validated against the given system. By default, passwords will always be validated when the setup allows but this can be disabled on a per-dataset basis with the ``try_password`` key.

Service Users
&&&&&&&&&&&&&

*Service users*, or possibly known as *functional accounts*, are accounts with a dedicated purpose, usually to interact with a system on other's behalf. These users can be added in the |origen_config|:

.. code:: toml

    # Create a service account 'service' with username 'serv' and password 'pass'
    [service_users]
    [service_users.service]
    username = "serv"
    password = "pass"

See Also
^^^^^^^^

* :class:`_origen.users.Users`
* :class:`_origen.users.User`
* |user_api_tests|

LDAP
----

Origen includes a wrapper for the ``Lightweight Directory Access Protocol``, or |ldap:wiki|, an interface common in corporate environments for storing user data.

LDAP instances are added via |origen_config|. A single LDAP only has a few parameters:

.. code-block:: toml

    # Denote that there are LDAPs
    [ldaps]

    # A single LDAP configuration, with name "forumsys"
    [ldaps.forumsys]
    # Required server and port location, combined into one URL
    server = "ldap://ldap.forumsys.com:389"

    # Required base DN for all operations, including binding
    base = "dc=example,dc=com"

    # Optional auth scheme. Currently, only "simple_bind" exists, but others
    # may be added in the future.
    auth = "simple_bind"

    # Optional service user account to use for binding and searching.
    # If none is given, the 'username' and 'password' parameters will be used.
    service_user = "ldap_account"

    # Username and password to use for binding, if the service user is not given.
    # If a service user is given, these are ignored.
    username = "u"
    password = "p"

Note: the above is a configuration for a :link-to:`free LDAP server <ldap:test_server>` and should work for testing or debug. See the |ldap_tests| for example interactions with this system.

Added LDAPs are available as :class:`origen.ldaps <_origen.utilities.ldap.LDAPs>`, a |dict-like| container:

.. code:: python

    origen.ldaps.keys
        #=> ['forumsys']
    
    origen.ldaps['forumsys']
        #=> _origen.utility.ldap.LDAP

    # Bind (connect to, with the service user or username/password, depending on which was given)
    origen.ldaps['forumsys'].bind()
        #=> True # if successful

Common Methods
^^^^^^^^^^^^^^

The LDAP wrapper has two main purposes: general searches and validating user's credentials.

Searching can be done using the :meth:`search <_origen.utilities.ldap.Ldap.search>` method. This takes a |ldap:filter| and an attribute list and spits out the resulting query. For simpler searches, where the |ldap:filter| is expected to return exactly one or zero entries, you can use :meth:`search_single_filter <_origen.utilities.ldap.Ldap.search_single_filter>` to get a friendlier return value. If more than one entry is returned then an error is raised.

The :meth:`validate_credentials <_origen.utilities.ldap.Ldap.validate_credentials>` method will check that the given username and password validates against the LDAP. The state of the LDAP itself is unchanged.

Implementation note: this method looks strictly for ``error code 49``, |ldap:invalid_credentials|. An exception will be raised for other error codes.

Scope Of Origen's LDAP
^^^^^^^^^^^^^^^^^^^^^^

Currently, the LDAP wrapper does not support modification functions and there are no plans to add these by the core team at this team.

For authentication, only "simple_bind", by providing a username and password, is supported. More auth schemes can be added as needed but the core team does not currently have a means to validate them, so they are omitted. If additional auth schemes are needed, please |open_a_ticket| to start the discussion.

LDAP Resources
^^^^^^^^^^^^^^

For more information on Origen's LDAP, see the resources below:

* :class:`origen.ldaps <_origen.utilities.ldap.LDAPs>`
* :class:`ldap API <_origen.utilities.ldap.Ldap>`
* |ldap:filters|
* |ldap:wiki|
* |ldap_regression_tests|
* |ldap:test_server|

Mailer
------

A simple command-line interface is also available:

{{ insert_cmd_output(origen_exec + " mailer --help", shell=run_in_shell) }}

Session Storage
---------------

Some features or :link-to:`plugins <origen_plugins>` cache simple data pieces regarding the current user, workspace configuration, environment, or other aspects - |origen_utilities:password_caching| being one - where the data should persists across invocations. Origen's :class:`session store <_origen.utility.session_store.Session>` provides an interface for such a task.

The current session is accessed through ``origen.app.session`` and has two key functions: :meth:`store <_origen.utility.session_store.Session.store>` and :meth:`get <_origen.utility.session_store.Session.get>`. As their names suggest, ``store`` will put data into the session, storing it for future retrievals while ``get`` will retrieve previously stored data.

.. code:: python

    origen.app.session.get("val")
        #=> None
    origen.app.session.store("val", 1)
    origen.app.session.get("val")
        #=> 1
    origen.app.session.store("val 2", 2)
    origen.app.session.get("val 2")
        #=> 2

:meth:`delete <<_origen.utility.session_store.Session.delete>` will remove an item from the session entirely, returning the deleted value. However, this can also be achieved by storing a ``None`` value, but without getting the value back:

.. code:: python

    origen.app.session.get("val")
        #=> 1
    origen.app.session.get("val 2")
        #=> 2
    
    origen.app.session.delete("val")
        #=> 1
    origen.app.session.get("val")
        #=> None

    origen.app.session.store("val 2", None)
    origen.app.session.get("val 2")
        #=> None

Session Scopes
^^^^^^^^^^^^^^

Previously, we used ``origen.app.session()`` to get handle on the application's session. As
the name would suggest, this session is application specific. Navigating to a different Origen
application's workspace will not carry any of the previous application's session data over.

For session data that should exists for a given user across all of their applications, or even outside of an application, the ``user session`` can be used. :link-to:`Password caching <origen_utilities:password_caching>` is one such item stored in the user's session, as opposed to the application's. Other than scope, this session store behaves identically to the application session.

This session is accessed as :meth:`origen.session_store.user_session() <_origen.utilities.session_store.Sessions.user_session>`.

Session Namespaces
^^^^^^^^^^^^^^^^^^

Using ``session()`` without any arguments yields generic session storage for the application. For organizational purposes, and to ensure that different features or plugins do not inadvertently step on each other, an optional ``str`` argument will grab an entirely disjoint session under that name.

.. code:: python

    origen.session_store.app_session().store("test", 1)
    origen.session_store.app_session("alt").store("test", 2)

    origen.session_store.app_session().get("test")
        #=> 1
    origen.session_store.app_session("alt").get("test")
        #=> 2

This same feature is available for ``user sessions`` as well.

For plugins, the instance itself can be passed to yield its dedicated session. Note however that
this is only a syntactic difference and yields the same session as if plugin's name was used
instead.

.. code:: python

    pl = origen.plugins("python_plugin")

    # Retrieve a plugins app session
    # These two are equivalent
    origen.session_store.app_session(pl)
    origen.session_store.app_session(pl.name)

    # Same is true for user sessions
    origen.session_store.user_session(pl)
    origen.session_store.user_session(pl.name)

As a syntactic shortcut, a plugin's session can also be retrieved from the plugin itself:

.. code:: python

    pl.session
        #=> origen.session_store.app_session(pl)
    
    pl.user_session
        #=> origen.session_store.user_session(pl)

Data Serialization
^^^^^^^^^^^^^^^^^^

Almost any Python object can be stored in the session. Standard objects which could also be
used by the Rust backend, such as strings, numbers, booleans, or lists of those types, are
stored directly. Any other objects, such as custom classes, are serialized using |pickle|.

You can opt to store and get data through your own serialization mechanism. The method :meth:`store_serialized <_origen.utility.session_store.Session.store_serialized>` will bypass any serialization or data type inference occurring in the backend and simply store the given |bytes| directly. When it is retrieved, via the standard :meth:`get <_origen.utility.session_store.Session.get>` method, the |bytes| are retrieved. See the |test case| for an example
of storing via |marshal|.

.. Session File Data
.. ^^^^^^^^^^^^^^^^^

Session Store Resources
^^^^^^^^^^^^^^^^^^^^^^^

* :class:`SessionStore API <_origen.utility.session_store.SessionStore>`
* |session_store_tests|