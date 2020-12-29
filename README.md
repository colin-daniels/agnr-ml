# AGNR-ML
Research code for the generation and analysis of armchair graphene nanoribbon
(AGNR) structures using machine learning.

## Requirements
On Ubuntu 20.04, the following packages are required dependencies:
```sh
build-essential
clang
cmake
gfortran
libffi-dev
libopenblas-dev # or alternative
pkg-config
python3.8-dev
```

Python dependencies are listed in the root `requirements.txt`.

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
