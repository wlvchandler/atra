from setuptools import setup, find_packages

setup(
    name="atra-cli",
    version="0.1.0",
    packages=find_packages(),
    install_requires=[
        "grpcio>=1.60.0",
        "grpcio-tools>=1.60.0",
    ],
    entry_points={
        'console_scripts': [
            'atra-cli=atra_cli.cli:main',
        ],
    },
)
