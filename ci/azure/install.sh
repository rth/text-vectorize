#!/bin/bash

set -e

UNAMESTR=`uname`


TO_INSTALL="python=$PYTHON_VERSION pip
	numpy>=1.12.0 scipy>=1.0.0 pytest>=4.0.0 wheel>=0.31.1
	nomkl"

conda create -n $VIRTUALENV --yes $TO_INSTALL
source activate $VIRTUALENV

python --version
pip --version
python -c "import numpy; print('numpy %s' % numpy.__version__)"
python -c "import scipy; print('scipy %s' % scipy.__version__)"
pip list

curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2019-02-28
source $HOME/.cargo/env

cd python/
pip install -r requirements.txt
python setup.py bdist_wheel

pip install --pre --no-index --find-links dist\ vtextpy
