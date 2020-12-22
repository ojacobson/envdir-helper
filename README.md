# Environment Directory Helper

This program loads environment variables from files.

The program was motivated by the pattern of configuring various tokens via
environment variables. I found my shell profile increasingly littered with code
of the form:

    export SOME_TOKEN="$(< ~/.some_token)"

I've replaced all of that with a single line:

    eval "$(envdir-helper)"

This program also supports setting non-exported shell variables, using the
`--no-export` flag. This is useful for prompts and other shell configuration
that should not be propagated through to subshells and other programs. This behaviour is the default if the env directory's name ends in `rc`:

    eval "$(envdir-helper .envdir.rc)"

## Security

As alluded to above, one of the use cases for this is env-specific tokens. These
kinds of tokens deserve special care - not just with this program, but in
general:

* They should be in files readable only by the current user (`-rw-------`) or by
  the current user and group (`-rw-r-----`), as appropriate;
* They should be rotated regularly; and
* They should only be set when in use.

This program does relatively little to manage this directly. One approach that helps is to invoke `envdir-helper` from [`direnv`] or similar, instead of from your shell profile, and to store the actual tokens in a system such as [Vault] or in the [macOS Keychain] to avoid leaving them on disk. Program entries in the environment directory can retrieve data from outside sources.

[`direnv`]: https://direnv.net/
[Vault]: https://www.vaultproject.io/
[macOS Keychain]: https://developer.apple.com/documentation/security/keychain_services/keychain_items/searching_for_keychain_items

## Installation

Some familiarity with Python is assumed, here:

* Make a virtual environment;
* `$VIRTUALENV/bin/pip install git+https://github.com/ojacobson/envdir-helper/#egg=envdir-helper`; and
* Add its `bin` directory to `PATH` by other means, or invoke it by full path.

## Development

I use [pyenv] and [`direnv`] to manage development. The configuration in
`.envrc` will automatically create a virtual Python environment using Pyenv (if
possible) or your current Python version (otherwise), and load it, once the
configuration is allowed. See the `direnv` documentation and the included
`.envrc` script for details.

[pyenv]: https://github.com/pyenv/pyenv
