from agnr_ml import RsAGNR


def test_agnr():
    agnr = RsAGNR([(0, 4), (1, 5)])
    assert agnr.len() == 4


def test_generation():
    agnrs = RsAGNR.all_possible_agnrs(
        min_len=1,
        max_len=4,
        min_width=2,
        max_width=6,
        symmetric_only=True,
    )
    assert len(agnrs) == 447
