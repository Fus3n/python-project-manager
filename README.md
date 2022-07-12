# PPM

A Command-line tool to create, manage and deploy your python projects

## Main Features

* Virtual Environment
* Package Manager (pip)
* Scripts (run test, build, etc)

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
-- 2 Packages --
pyopt_tools==0.7
numpy==1.23.1
```

### Install/Uninstall Packages/Modules

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
