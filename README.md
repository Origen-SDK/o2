## Development Environment Setup

1) [Install Rust](https://www.rust-lang.org/tools/install) 
2) Enable Rust nightly version (this must be done for every o2 workspace):

~~~
rustup install nightly
cd path/to/o2
rustup override set nightly
~~~

3) By this point make sure your $PATH contains the following to make the `cargo` command available:

~~~
export PATH="$HOME/.cargo/bin:$PATH"
~~~

4) Compile the Rust code (you will repeat this step everytime you change it):
~~~
cd o2/rust/origen
cargo build --workspace
~~~

On Ubuntu, the following packages may need to be installed if you get errors:

~~~
sudo apt install libssl-dev
sudo apt install pkg-config
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

All being well, you now have a booted Origen console and an app instance available. e.g. `origen.app.config` should return a DICT from the values defined in `config/application.toml`.



