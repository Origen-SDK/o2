Creating a Plugin
=================

An Origen plugin is simply an Origen application like any other, no special setup is required
in order to make an existing application function as a plugin to another parent application.

To distribute an application, it must be packaged up by running the following command:

.. code:: none

  origen app package

This creates a `Python wheel <https://realpython.com/python-wheels/>`_ archive of the application
in the :code:`dist/` directory.

Releasing a Plugin
------------------

To release a plugin (to make it available to your user community) the wheel should be uploaded to your
company's package server by...

.. include:: /_templates/doc_fail.rst