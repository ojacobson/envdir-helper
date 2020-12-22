from setuptools import setup, find_packages

setup(
    name="envdir-helper",
    use_scm_version=True,

    author="Owen Jacobson",
    author_email="owen@grimoire.ca",

    packages=find_packages(),

    setup_requires=[
        "setuptools_scm ~= 4.1",
    ],

    install_requires=[
        "click ~= 7.1.0",
    ],

    entry_points={
        "console_scripts": [
            "envdir-helper=envdir.cli:main",
        ],
    },
)
