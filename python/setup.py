from setuptools import find_packages, setup
from setuptools_rust import RustExtension


with open("./requirements.txt", "rt") as fh:
    install_requires = fh.read().splitlines()

setup(
    name="pytext-vectorize",
    version="0.1.0a1",
    description="Text vectorization and TF-IDF",
    rust_extensions=[
        RustExtension(
            "pytext_vectorize._lib",
            "./Cargo.toml",
            rustc_flags=["--cfg=Py_3"],
            features=["numpy/python3"],
        )
    ],
    install_requires=install_requires,
    packages=find_packages(),
    python_requires="~=3.5",
    zip_safe=False,
)
