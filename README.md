[![Build Status](https://travis-ci.org/Byron-TW/expend-rs.svg?branch=master)](https://travis-ci.org/Byron-TW/expend-rs)

An application tailored to help automating the creation of repetitive expenses which don't require a receipt, namely *per diems*, *mileage* and *free-form expenses* for everything else.

### Installation on OSX

Using Brew it's certainly easiest:
```
brew tap Byron-TW/expend-rs https://github.com/Byron-TW/expend-rs
brew install expend
```

Without brew, you can download the latest release binary from [the releases page](https://github.com/Byron-TW/expend-rs/releases), unzip the archive and
place the binary in your `PATH`.

### Usage

_Please note that all following commands require some pre-requisites - they are documented further below_.

#### Post Per-Diems
You can create per-diems like this:

```
expend post perdiem weekdays fullday
```

To learn more, just use `--help`
```
expend post perdiem --help
```

#### Post Anything 

In case there is no dedicated sub-command for your kind of expense, you can also post any JSON file content directly. It must be the object expected in the [`inputSettings` field](https://integrations.expensify.com/Integration-Server/doc/#expense-creator) of the typical payload - all other values are provided by
`expend`.

```
expend post from-file ./payload-file.json
```

#### Prerequesites

First of all, you should head over to the [expensify integration
documentation](https://integrations.expensify.com/Integration-Server/doc/#authentication)
to generate your set of **credentials**. New credentials can be generated [here](https://www.expensify.com/tools/integrations/?action=create).
With these equipped, the first time you
run any `expend post` sub-commands you will be prompted for said credentials -
by default they are stored in your systems keychain for safe-keeping.

Before you run a command, you have to **create a default context**, which
identifies your e-mail address and project name. Said project name can be copied
directly from the respective project field from expensify.com. This can be done like this:

```
expend context set -e your_email@domain.com -p 'Project Name'
```

### Roadmap

#### v1.1.0

* [ ] Mileage sub-command

#### v1.0.0

* [x] add 'travel-billable' flag to the user context
* [x] add 'country' to the user context, but default to germany
* [x] add various per-diem types, like 'lunch/dinner'
* [x] implement subtraction for per-diems 
* [x] support for custom comments in per-diems
* [x] travic CI
* [x] brew
* [x] some docs

### Limitiations

#### Support for Linux and Windows

* On _Linux_, compilation currently fails as GMP and DBUS libraries are required for keychain support.
* On Windows compilation fails due to our dependency to Termion.

### Maintenance Notes

#### On creating a new release

* Update the Roadmap so it's clear which new features are included.
* Update the `version` in `Cargo.toml` to match the new release and push the commit.
* Create a tag with the new `version`, ideally without any prefix, and push it with `git push --tags`.
* Travis will be busy generating the binary's archive, and you can run `make update-homebrew` right after pushing
  to update the respective brew file to match the new version.
* Once the brew formula was updated, commit and push, ideally with the comment `[skip CI]` to avoid travis to run unnecessarily.
