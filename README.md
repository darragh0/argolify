# argolify

[![WIP](https://img.shields.io/badge/status-WIP-orange?labelColor=%23f5f5f5)](https://github.com/darragh0/argolify)
[![MIT-License](https://img.shields.io/badge/license-MIT-gray?labelColor=%23f5f5f5)](https://opensource.org/license/MIT)

**argolify** is a Rust tool that parses argol DSL files ([see this repo](https://github.com/darragh0/argol)) to automatically generate command-line argument parsers for your projects. argolify will generate a standalone executable that handles all the argument parsing based on your argol file. This executable runs separately from your main program, so you can keep your argument logic out of your core code. You can invoke this separate executable within your main program to handle the argument parsing before passing control to your actual app.

Think of it like an “intermediary” that takes care of the CLI stuff, then hands over a clean and validated set of arguments to your main program. It keeps things modular and makes your app leaner by not cramming the parsing logic directly into it.

The **argol language**  separates your CLI argument logic from the core application code, enabling you to manage and modify your argument structure independently. argol is language-agnostic, providing a standard way to define CLI interfaces.

## License
- MIT license ([LICENSE](./LICENSE) or <https://opensource.org/licenses/MIT>)

> [!WARNING]
> This project and the [argol project](https://github.com/darragh0/argol) is still under active development.
