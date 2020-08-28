```eval_rst
:name: development_setup
```

# Development Environment Setup

```eval_rst
These instructions are for how to setup an environment for development of Origen, they should not be followed by
anyone who only wants to use Origen - if that's you, then follow the :ref:`user installation guide <user-installation-guide>` instead.
```

## 1st Time Development Environment Setup



1) [Install Rust](https://www.rust-lang.org/tools/install)

2) Enable Rust nightly version (this must be done for every o2 workspace):

   ~~~
   rustup install nightly or rustup default nightly (this will make rust nightly the default version being run)
   cd path/to/o2
   rustup override set nightly
   ~~~

3) By this point make sure your $PATH contains the following to make the `cargo` command available:

   ~~~
   export PATH="$HOME/.cargo/bin:$PATH"
   ~~~

4) Make sure you have Python >= 3.6 available via either the `python` or `python3` commands, 

   ~~~
   $ python3 --version
   Python 3.6.9
   ~~~

   If you need to install a suitable Python version, here is one of many available guides on it: https://realpython.com/installing-python/

5) Install the following Python packages which are required to build Origen:

   ~~~
   pip3 install poetry
   pip3 install maturin
   pip3 install twine
   pip3 install pyfs
   pip3 install yapf
   pip3 install -U keyrings.alt
   ~~~

6) Add this dir to your $PATH, ahead of any other dir that provides an `origen` command so that you will be using
   the version of Origen Command Line Interface (CLI) built from this workspace:

   ~~~
   export PATH="</path/to/your>/o2/rust/origen/target/debug:$PATH"
   ~~~

7) Build the CLI:

   ~~~
   cd o2/rust/origen
   cargo build --workspace --bins
   ~~~

8) Verify that you now have the `origen` command available:

   ~~~
   $ origen -v
   Origen: 2.0.0-pre2
   ~~~

9) Missing Ubuntu Packages:

   On Ubuntu, the following packages may need to be installed if you get errors:
   
   ~~~
   sudo apt install libssl-dev
   sudo apt install pkg-config
   sudo apt install python3-distutils
   sudo apt install python3-venv
   ~~~

## 1st Time Python App Setup

Whenever a new workspace is created for an Origen Python application its local environment needs to be setup and the test
application embedded within the Origen 2 environment is no exception.
This can be done simply by executing the `origen setup` command within the application directory:

~~~
cd o2/test_apps/python_app
origen setup
~~~

## Regular Workflow

To build Origen core and its Python bindings and plug it into the example application (the most common build during
development), simply run:

~~~
origen build
~~~

To re-build the CLI run:

~~~
origen build --cli
~~~
