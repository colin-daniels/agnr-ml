from agnr_ml import AGNR, generate_all_possible_agnrs
from unittest import TestCase


class TestAGNR(TestCase):
    def test_construction(self):
        agnr = AGNR([(0, 4), (1, 5)])
        self.assertEqual(len(agnr), 2)

    def test_to_structure(self):
        agnr = AGNR([(0, 4), (1, 5)])
        structure = agnr.to_structure()
        structure = structure.as_dict(verbosity=0)

        self.assertDictEqual(structure, {
            '@module': 'pymatgen.core.structure',
            '@class': 'Structure',
            'charge': None,
            'lattice': {
                'matrix': [
                    [4.26135, 0.0, 0.0],
                    [0.0, 21.15072892402803, 0.0],
                    [0.0, 0.0, 15.0]
                ]
            },
            'sites': [
                {'species': [{'element': 'C', 'occu': 1}], 'abc': [-0.16666666666666666, 0.35459770804776924, 0.5]},
                {'species': [{'element': 'C', 'occu': 1}], 'abc': [0.16666666666666666, 0.35459770804776924, 0.5]},
                {'species': [{'element': 'C', 'occu': 1}], 'abc': [-0.16666666666666666, 0.47091954160955385, 0.5]},
                {'species': [{'element': 'C', 'occu': 1}], 'abc': [0.16666666666666666, 0.47091954160955385, 0.5]},
                {'species': [{'element': 'C', 'occu': 1}], 'abc': [0.33333333333333337, 0.41275862482866155, 0.5]},
                {'species': [{'element': 'C', 'occu': 1}], 'abc': [0.6666666666666667, 0.41275862482866155, 0.5]},
                {'species': [{'element': 'C', 'occu': 1}], 'abc': [0.33333333333333337, 0.5290804583904462, 0.5]},
                {'species': [{'element': 'C', 'occu': 1}], 'abc': [0.6666666666666667, 0.5290804583904462, 0.5]},
                {'species': [{'element': 'H', 'occu': 1}], 'abc': [-0.2946155561031129, 0.30994795978344486, 0.5]},
                {'species': [{'element': 'H', 'occu': 1}], 'abc': [0.29461555610311285, 0.30994795978344486, 0.5]},
                {'species': [{'element': 'H', 'occu': 1}], 'abc': [0.20538444389688715, 0.5737302066547706, 0.5]},
                {'species': [{'element': 'H', 'occu': 1}], 'abc': [0.794615556103113, 0.5737302066547706, 0.5]},
            ],
        })


class TestGeneration(TestCase):
    def test_that_it_works(self):
        agnrs = generate_all_possible_agnrs(
            min_len=2,
            max_len=2,
            min_width=2,
            max_width=2,
            symmetric_only=True,
        )
        self.assertEqual(len(list(agnrs)), 2)
