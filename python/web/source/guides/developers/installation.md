```eval_rst
:name: installation
```

## Installation

...

## Development Environment Setup

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

4) Compile the Rust code (you will repeat this step everytime you change it):

To compile the core and CLI:
~~~
cd o2/rust/origen
cargo build --workspace --bins
~~~

To compile the Python extension:
~~~
cd o2/rust/pyapi
cargo build
~~~

To build it all:

~~~
cd o2/rust
cd origen && cargo build --workspace --bins && cd ../pyapi && cargo build
~~~

Or, using powershell:

~~~
cd o2\rust
cd origen ; cargo build --workspace --bins ; cd ..\pyapi  ; cargo build
~~~

4a) Missing Ubuntu Packages

On Ubuntu, the following packages may need to be installed if you get errors:

~~~
sudo apt install libssl-dev
sudo apt install pkg-config
sudo apt install python3-distutils
sudo apt install python3-venv
~~~

4b) Windows Compiled Libary

On Windows, in addition to `4)`, the resulting `_origen.dll` must be moved and renamed to a `.pyd`:

~~~
cp .\target\debug\_origen.dll ..\..\python\_origen.pyd
~~~

5) Add this dir to your $PATH, ahead of any other dir that provides an `origen` command:
~~~
export PATH="</path/to/your>/o2/rust/origen/target/debug:$PATH"
~~~

6) Verify that you now have the new `origen` command available:
~~~
$ origen -v
Origen: 2.0.0-pre0
~~~

7) Make sure your system has at least Python 3.5 available

8) Now that you have the Origen CLI available and Python, you can try booting the example app:

~~~
cd o2/example
origen setup
origen i
~~~

8a) If you are running this on the **Windows Sub-System Linux (WSL)** like I am, you might run into the following issues:
    **I was running Ubuntu 18.04 LTS as my WSL environment.**

A quick solution to this is running the following commands in your WSL environment
- **sudo pip install pyfs**
- **sudo pip install --upgrade keyrings.alt**

9) You should now be able to access the Origen interactive environment without issues


All being well, you now have a booted Origen console and an app instance available. e.g. `origen.app.config` should return a DICT from the values defined in `config/application.toml`.
