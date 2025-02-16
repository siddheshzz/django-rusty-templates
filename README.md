# Django Rusty Templates

Django Rusty Templates is an experimental reimplementation of Django's templating language in Rust.

## Goals

* 100% compatibility of rendered output.
* Error reporting that is at least as useful as Django's errors.
* Improved performance over Django's pure Python implementation.

## Installation

Django Rusty Templates is not yet ready for full release, so it is not available on PyPI yet. Instead it can be installed from github or from a local clone:

```sh
$ pip install git+https://github.com/LilyFoote/django-rusty-templates.git
```

```sh

$ git clone git@github.com:LilyFoote/django-rusty-templates.git
$ pip install ./django-rusty-templates
```

You will need a rust compiler installed (https://rustup.rs/).

## Usage

Add an entry to your [`TEMPLATES` setting](https://docs.djangoproject.com/en/5.1/ref/settings/#std-setting-TEMPLATES) with `"BACKEND"` set to `"django_rusty_templates.RustyTemplates"`:

```python
TEMPLATES = [
    {
        "BACKEND": "django_rusty_templates.RustyTemplates",
        ... # Other configuration options
    },
]
```

## Contributing

Django Rusty Templates is open to contributions. These can come in many forms:

* Implementing missing features, such as filters and tags built into Django.
* Reporting bugs where Django Rusty Templates gives the wrong result.
* Adding new test cases to ensure Django Rusty Templates behaves the same as Django.
* Adding benchmarks to track performance.
* Refactoring for readability or performance.

### Getting started with development

To develop locally, first create a virtualenv using your preferred method.

```bash
$ python -m venv .venv
```

Activate the virtualenv and install the requirements:

```bash
$ source .venv/bin/activate
$ pip install -r requirements.txt
```

To run the Python tests, build Django Rusty Templates in develop mode with maturin and then run pytest. Each change in rust needs a new execution of maturin develop.

```bash
$ maturin develop
$ pytest
```

You can also run the Rust tests:

```bash
$ cargo test
```

If you get an `ImportError` from python, you may need to set the `PYTHONPATH` environment variable:

```bash
export PYTHONPATH=/path/to/venv/lib/python3.x/site-packages
```
