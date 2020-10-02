Introduction
============

Origen enables easy and wide reaching code re-use via its plugin system.

Any Origen application can be plugged into another application to share its DUT and block models/controllers,
templates, patterns, test flow source files, etc.

A very common use case is to distribute drivers for in-house protocols and test modules (patterns and test programs for an IP block) 
within a company's private ecosystem.

Origen uses the `Python Package Index (PyPI) <https://pypi.org/>`_ system for distribution and any PyPI compliant package
server can be used for distribution of Origen plugins within a company - e.g. 

* `Sonatype Nexus <https://www.sonatype.com/nexus/repository-oss>`_
* `PyPI Server <https://github.com/pypiserver/pypiserver>`_
* `JFrog Artifactory <https://www.jfrog.com/confluence/display/JFROG/JFrog+Artifactory>`_
