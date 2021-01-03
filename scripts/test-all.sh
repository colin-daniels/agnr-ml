#!/bin/zsh

set -eux
source .venv/bin/activate

# stop the build if there are Python syntax errors or undefined names
flake8 agnr_ml tests --count --select=E9,F63,F7,F82 --show-source --statistics
# exit-zero treats all errors as warnings. The GitHub editor is 127 chars wide
flake8 agnr_ml tests --count --exit-zero --max-complexity=10 --max-line-length=127 --statistics

function checksum_sources() {
    sha512sum src/**/*(.) Cargo.* pyproject.toml agnr_ml/**/*(.) | grep --invert-match __pycache__
}

if [ ! -e scratch/dist.chk ]; then
  mkdir -p scratch
  checksum_sources > scratch/dist.chk
elif ! sha512sum --quiet -c scratch/dist.chk || [ ! -e scratch/dist ]; then
  docker run --rm -v "$PWD:/io" \
    --env CARGO_TARGET_DIR=scratch/maturin-target \
    maturin build \
    --bindings pyo3 \
    --manylinux 2010 \
    --interpreter python3.7 \
    --interpreter python3.8 \
    --interpreter python3.9 \
    --out scratch/dist
  checksum_sources > scratch/dist.chk
fi

for ver in cp37-cp37m cp38-cp38 cp39-cp39; do
  pip_cache="$PWD/scratch/pip-cache-$ver"
  mkdir -p "$pip_cache"

  docker run --rm -v "$PWD:/io" -v "$pip_cache:/root/.cache/pip/" \
    quay.io/pypa/manylinux2010_x86_64 \
    bash -c "
      export PATH=/opt/python/$ver/bin:\$PATH

      set -eux
      cd /io
      chown root:root /root/.cache/pip/
      python -m pip install -q --upgrade pip
      python -m pip install -q pytest
      python -m pip install -q --find-links=scratch/dist agnr_ml
      pytest --color=yes
"
done
