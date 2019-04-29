from vtext.metrics.string import dice_similarity, jaro_similarity
from vtext.metrics.string import jaro_winkler_similarity

import pytest


def test_dice_similarity():
    assert dice_similarity("healed", "sealed") == 0.8


def test_jaro_similarity():
    assert jaro_similarity("SHACKLEFORD", "SHACKELFORD") == pytest.approx(
        0.970, rel=1e-2
    )


def test_jaro_winkler_similarity():
    assert jaro_winkler_similarity(
        "SHACKLEFORD", "SHACKELFORD", 0.1, 4
    ) == pytest.approx(0.982, rel=1e-2)
