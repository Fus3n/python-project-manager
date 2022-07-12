# PPM

A Command-line tool to create, manage and deploy your python projects

## Table of Contents

- [PPM](#ppm)
  - [Main Features](#main-features)
    - [Create a Project](#create-a-project)
    - [project.ini file](#projectini-file)
    - [Project](#project)
    - [Install/Uninstall Packages](#install-uninstall-packages)
    - [Run Scripts](#run-scripts)
    - [⚙️ Generate requirements.txt](#⚙️-generate-requirementstxt)
    - [⏬ Install Packages from project.ini](#⏬-install-packages-from-projectini)
  - [Build From Source](#build-from-source)

</br>

| 🔗 | [Try it out](https://github.com/Fus3n/python-project-manager/releases)  |
|---------------|:------------------------|

</br>

## Main Features

- Virtual Environment Manager
- Package Manager (using pip)
- Scripts (run test, build, etc)

### Create a Project

```bash
ppm new <project-name>
cd <project-name>
ppm start
```

### project.ini file

```ini
[Project]
name=example
version=0.1.0
description="an example project"
main=./src/main.py

[Packages]
pyopt_tools=0.7
numpy=1.23.1

[Scripts]
test="python -m unittest src/test.py"
serve="python -m http.server"
sayhello="echo Hello world!"
```

### Project

Get an overview of your project

```bash
$ ppm info
Project: example
Version: 0.1.0
Description: an example project

-- 4 Scripts --
test: python -m unittest src/test.py
serve: python -m http.server
sayhello: echo Hello world
upgrade: python -m pip install --upgrade pip

-- 2 Packages --
pyopt_tools==0.7
numpy==1.23.1
```

### Install/Uninstall Packages

You can add or remove multiple packages at the same time.
This will install it into your venv

```bash
ppm add <package-names>
ppm remove <package-names>
```

### Run Scripts

Create scripts and run them by simply doing

```bash
ppm run <script-name>
```

### ⚙️ Generate requirements.txt

Generates requirements.txt from packges listed in project.ini

```bash
ppm gen
```

### ⏬ install Packages from project.ini

This will try to install all the packages listed in project.ini
It will also create a virtual environment if it doesn't exist

```bash
ppm install
```

## Build From Source

```bash
git clone https://github.com/Fus3n/python-project-manager
cd python-project-manager
cargo build --release
```
