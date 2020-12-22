import click
import os
import os.path as op
import pathlib
import shlex
import stat
import subprocess as sp


@click.command()
@click.pass_context
@click.version_option()
@click.option(
    "--export/--no-export",
    default=None,
    help="Export generated environment variables [default: --export]",
)
@click.argument(
    "envdir",
    default=str(pathlib.Path.home() / ".envdir"),
    metavar="DIR",
)
def main(context, export, envdir):
    r"""Load environment variables from DIR (or ~/.envdir).

    For each non-directory entry in DIR, this will output a brief shell script
    to set an environment with the same name. If the entry is an executable
    program, it will be run (with the current environment) and its output will
    be used as the value of the environment variable. Otherwise, it will be
    read, and its content will be used. In either case, a single trailing
    newline will be removed, if present.

    This process skips any file which can't be run or read, as appropriate, and
    outputs a warning on stderr.

    The intended use case for this is in shell profiles, in the form:

        eval "$(envdir-helper)"

    The generated output is compatible with sh, and thus with bash and zsh.
    """

    def warn_on_skipped(path, reason):
        """Log any skipped paths to stderr."""
        stderr = click.get_text_stream("stderr")
        click.echo(f"{context.info_name}: skipping {path}: {reason}", file=stderr)

    if export is None:
        env_script = detect_env_script(
            envdir, rc=no_export_env_script, default=export_env_script
        )
    elif export:
        env_script = export_env_script
    else:
        env_script = no_export_env_script

    for name, content in walk_entries(envdir, on_skipped=warn_on_skipped):
        script = env_script(name, content)
        click.echo(script)


def detect_env_script(path, rc, default):  # pylint: disable=invalid-name
    """Detect which of two values to use based on whether `path` ends with
    `"rc"`. If it does, returns `rc`; otherwise, returns `default`.
    """
    if path.endswith("rc"):
        return rc
    return default


def export_env_script(name, content):
    """Given a name and contents, generate a shell script that will set and
    export the corresponding environment variable, with that content."""
    # use sh-friendly syntax here: don't assume `export` can have assignment
    # side effects, so don't use `export FOO=BAR`. `FOO=BAR; export FOO` is
    # portable to all posix shells.
    qname = shlex.quote(name)
    qcontent = shlex.quote(content)
    return f"{qname}={qcontent}; export {qname}"


def no_export_env_script(name, content):
    """Given a name and contents, generate a shell script that will set and NOT
    export the corresponding environment variable, with that content."""
    qname = shlex.quote(name)
    qcontent = shlex.quote(content)
    return f"{qname}={qcontent}"


def walk_entries(envdir, on_skipped):
    """Yields a name, value pair for each environment file in envdir.

    The path for any skipped items (generally, failing or unrunnable programs,
    or unreadable files) will be passed to the `on_skipped` callback, along with
    the exception that caused it to be skipped.
    """
    for name in sorted(os.listdir(envdir)):
        path = op.join(envdir, name)
        try:
            path_stat = os.stat(path)
            if directory(path_stat):
                continue

            if executable(path_stat):
                content = from_program(path)
            else:
                content = from_file(path)
        except Exception as reason:  # pylint: disable=broad-except
            on_skipped(path, reason)
            continue
        if content.endswith("\n"):
            content = content[:-1]
        yield name, content


def directory(item):
    """True iff item is a stat_result representing a directory."""
    return stat.S_ISDIR(item.st_mode)


def executable(item):
    """True iff item is a stat_result representing something executable.

    This doesn't distinguish between dirs and files; both are considered
    executable if any +x bit is set.
    """
    mode = stat.S_IMODE(item.st_mode)
    mask = stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH
    return mode & mask != 0


def from_program(path):
    """Reads a program's complete stdout into a string."""
    result = sp.run(
        [path],
        check=True,
        stdout=sp.PIPE,
    )
    return result.stdout.decode("UTF-8")


def from_file(path):
    """Reads a file's complete content into a string."""
    with open(path, "r") as file:
        return file.read()
