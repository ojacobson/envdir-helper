from importlib.metadata import version, PackageNotFoundError

try:
    __version__ = version("envdir-helper")
except PackageNotFoundError:
    # package is not installed
    pass
