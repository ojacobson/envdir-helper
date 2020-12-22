from setuptools import setup, find_packages

setup(
    name="envdir-helper",
    version="0.0.0",

    author="Owen Jacobson",
    author_email="owen@grimoire.ca",
    
    packages=find_packages(),

    install_requires=[
        "click ~= 7.1.0",
    ],

    entry_points={
        "console_scripts": [
            "envdir-helper=envdir.cli:main",
        ],
    },
)
