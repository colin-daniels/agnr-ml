# AGNR-ML
[![rust-build-status](https://img.shields.io/github/workflow/status/colin-daniels/agnr-ml/Rust)](https://github.com/colin-daniels/agnr-ml/actions?query=workflow%3ARust)

Research code for the generation and analysis of armchair graphene nanoribbon
(AGNR) structures using machine learning.

## Requirements
On Ubuntu 20.04, the following packages are required build dependencies:
```sh
clang
cmake
g++
gfortran
libffi-dev
libopenblas-dev # or lapack/blas alternative
pkg-config
make
pkg-config
python3.8-dev
```

## Usage
TODO

## Development
This project uses [git subtree](https://www.atlassian.com/git/tutorials/git-subtree) to
manage the `rsp2` dependency. Examples:
```sh
# add remote
git remote add -f rsp2-origin git@github.com:ExpHP/rsp2.git
# push changes to remote
git subtree push --prefix=rsp2 rsp2-origin <branch>
# pull from remote
git subtree pull --prefix=rsp2 rsp2-origin master --squash
```

## License
In general, this code is available under the MIT license (see `LICENSE`) but
the `rsp2` component is distributed (in part) under the GNU GPLv3, see
`rsp2/README.md` and the accompanying license files for more information.
