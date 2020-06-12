import pytest

from vtext.stem import SnowballStemmer


def test_snowball_stemmer():
    stemmer = SnowballStemmer(lang="english")
    assert stemmer.stem("fruitlessly") == "fruitless"
    assert stemmer.stem("continuité") == "continuité"

    stemmer = SnowballStemmer(lang="french")
    assert stemmer.stem("continuité") == "continu"


def test_snowball_stemmer_get_params():
    est = SnowballStemmer()
    assert est.get_params() == {'lang': 'english'}


def test_snowball_stemmer_input_validation():
    with pytest.raises(ValueError, match="lang=catalan is unsupported"):
        SnowballStemmer(lang="catalan")
