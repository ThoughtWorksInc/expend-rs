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
# or YAML files, you can extract them directly from the confirmation dialog shown
# when posting per-diems.
expend post from-file ./payload-file.yaml
```

#### Prerequesites

Before you can run any `post` command successfully, you will need to **authenticate** and to **create a context**. The former identifies _you_, the latter
identifies _your project_.

 1. Run `expend authenticate` and follow the on-screen instructions. Please note that you should login
    using SAML, other login options won't work.
    * Even though you shouldn't need this, new credentials can be forcefully generated [here](https://www.expensify.com/tools/integrations/?action=create).
 2. Run `expend context set -e your_email@domain.com -p 'Project Name'`
    * Note that the project name has to be copied directly from the respective Expensify field
      of an existing Expense in the web-frontend.
    * _Did you know_ that you can have multiple contexts and switch between them on a per-invocation
      basis with the `--context` flag? That way creating expenses for multiple projects is easy.

### Roadmap

#### v1.2.0

* [ ] Mileage sub-command

#### v1.1.0

* [x] The 'authenticate' sub-command and a better workflow for getting started.

#### v1.0.1

* [x] Properly cleanup username after entering it. Otherwise an 'n' will be appended, which formerly
  was a newline.

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
