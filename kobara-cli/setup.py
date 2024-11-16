from setuptools import setup, find_packages

setup(
    name="kobara-cli",
    version="0.1.0",
    packages=find_packages(),
    install_requires=[
        "grpcio>=1.60.0",
        "grpcio-tools>=1.60.0",
    ],
    entry_points={
        'console_scripts': [
            'kobara-cli=kobara_cli.cli:main',
        ],
    },
)
