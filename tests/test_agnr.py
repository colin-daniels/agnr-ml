from agnr_ml import AGNR, generate_all_possible_agnrs
from unittest import TestCase
from pymatgen import Lattice, Structure


class TestAGNR(TestCase):
    def test_construction(self):
        agnr = AGNR([(0, 4), (1, 5)])
        self.assertEqual(len(agnr), 2)

    def test_to_structure(self):
        agnr = AGNR([(0, 4), (1, 5)])
        structure = agnr.to_structure()
        print(structure.cart_coords)
        self.assertEqual(structure, Structure(
            lattice=Lattice([
                [4.26135, 0.0, 0.0],
                [0.0, 18.69043735441682, 0.0],
                [0.0, 0.0, 15.0]
            ]),
            species=["C", "C", "C", "C", "C", "C", "C", "C", "H", "H", "H", "H"],
            coords_are_cartesian=True,
            coords=[
                [-0.710225, 7.5, 7.5],
                [0.710225, 7.5, 7.5],
                [-0.710225, 9.96029157, 7.5],
                [0.710225, 9.96029157, 7.5],
                [1.42045, 8.73014578, 7.5],
                [2.8409, 8.73014578, 7.5],
                [1.42045, 11.19043735, 7.5],
                [2.8409, 11.19043735, 7.5],
                [-1.25546, 6.55562528, 7.5],
                [1.25546, 6.55562528, 7.5],
                [0.875215, 12.13481208, 7.5],
                [3.386135, 12.13481208, 7.5],
            ],
        ))


def test_generation_doesnt_crash():
    generate_all_possible_agnrs(2, 2, 2, 2)
