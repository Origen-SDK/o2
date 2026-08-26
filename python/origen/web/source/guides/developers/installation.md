(development_setup)=
# Development Environment Setup

These instructions are for how to setup an environment for development of Origen, they should not be followed by
anyone who only wants to use Origen - if that's you, then follow the
{ref}`user installation guide <user-installation-guide>` instead.

## 1st Time Development Environment Setup



1) [Install Rust](https://www.rust-lang.org/tools/install)

2) Install and select the current stable Rust toolchain:

   ~~~
   rustup toolchain install stable
   cd path/to/o2
   rustup override set stable
   ~~~

3) By this point make sure your $PATH contains the following to make the `cargo` command available:

   ~~~
   export PATH="$HOME/.cargo/bin:$PATH"
   ~~~

4) Make sure you have a supported Python version (currently 3.7 through 3.12)
   available via either the `python` or `python3` command:

   ~~~
   $ python3 --version
   Python 3.11.11
   ~~~

   If you need to install a suitable Python version, here is one of many available guides on it: https://realpython.com/installing-python/

5) Install UV, which manages every Python environment in the workspace. Origen
   requires **0.12.5 or newer**; older releases lack flags the CLI depends on and
   `origen env setup` will refuse to run. Installing the pinned version keeps a
   local checkout aligned with CI:

   ~~~
   curl -LsSf https://astral.sh/uv/0.12.5/install.sh | sh
   uv --version
   ~~~

   If an older `uv` is already on your `$PATH` (a common one is 0.6.x from an
   earlier install), make sure the newly installed binary comes first.

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
   Origen: {{origen.version}}
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
This can be done simply by executing the `origen env setup` command within the application directory:

~~~
cd o2/test_apps/python_app
origen env setup
~~~

## Regular Workflow

To build Origen core and its Python bindings and plug it into the example application (the most common build during
development), simply run:

~~~
origen develop_origen build
~~~

To re-build the CLI run:

~~~
origen develop_origen build --cli
~~~

To build either with release optimizations add the `--release` switch:

~~~
origen develop_origen build --release
origen develop_origen build --cli --release
~~~

To use a local version of Origen within an application run the following commands from within the application's workspace:

~~~
origen env setup --origen path/to/your/o2
origen develop_origen build
~~~

`--origen` records the checkout as a path dependency in the application's
`pyproject.toml` and `uv.lock`, so the change outlives the command and should
not be committed. Restoring the application to a released Origen package means
dropping that dependency first; `origen env setup` on its own only
resynchronizes whatever the manifest currently declares:

~~~
uv remove origen
origen env setup
~~~
