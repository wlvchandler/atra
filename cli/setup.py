from setuptools import setup, find_packages
import os

with open("cli/requirements.txt") as f:
    requirements = [line.strip() for line in f if line.strip() and not line.startswith("#")]

# this part is optional
long_description = ""
if os.path.exists("README.md"):
    with open("README.md", "r", encoding="utf-8") as fh:
        long_description = fh.read()

setup(
    name="atra-cli",
    version="0.1.0",
    description="Atra CLI client",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="will chandler",
    packages=find_packages(),
    install_requires=requirements,
    python_requires=">=3.10",
    entry_points={
        'console_scripts': [
            'atra=cli.__main__:main',
            'atra=cli.__main__:main',
        ],
    },
    classifiers=[
        "Development Status :: 4 - Beta",
        "Environment :: Console",
        "Intended Audience :: Developers",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.10",
    ],

    package_data={
        "cli": ["platform/env/*"],
    },
)
