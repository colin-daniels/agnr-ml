from agnr_ml.agnr_ml import AGNR


def test_agnr():
    agnr = AGNR([(0, 4), (1, 5)])
    assert agnr.len() == 4


def test_generation():
    agnrs = AGNR.all_possible_agnrs(
        min_len=1,
        max_len=4,
        min_width=2,
        max_width=6,
        symmetric_only=True,
    )
    assert len(agnrs) == 447
