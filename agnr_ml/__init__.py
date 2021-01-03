from .agnr_ml import AGNR as NativeAGNR
from pymatgen import Lattice, Structure
from typing import List, Tuple, Union, Generator


class AGNR:
    def __init__(self, spec: Union[NativeAGNR, List[Tuple[int, int]]]):
        if isinstance(spec, NativeAGNR):
            self._spec = spec
        else:
            self._spec = NativeAGNR(spec)

    def __len__(self):
        return len(self.spec())

    def spec(self) -> List[Tuple[int, int]]:
        return self._spec.spec

    def to_structure(
            self,
            cc_bond: float = 1.42045,
            ch_bond: float = 1.09047,
            vacuum_sep: float = 15.0,
    ) -> Structure:
        structure = self._spec.to_structure(
            cc_bond=cc_bond,
            ch_bond=ch_bond,
            vacuum_sep=vacuum_sep,
        )

        return Structure(
            lattice=Lattice(structure.lattice()),
            species=structure.types(),
            coords=structure.coords(),
            coords_are_cartesian=True,
        )


def generate_all_possible_agnrs(
    min_len: int,
    max_len: int,
    min_width: int,
    max_width: int,
    symmetric_only: bool = False,
) -> Generator[AGNR, None, None]:
    all_agnrs = NativeAGNR.all_possible_agnrs(
        min_len=min_len,
        max_len=max_len,
        min_width=min_width,
        max_width=max_width,
        symmetric_only=symmetric_only,
    )
    for agnr in all_agnrs:
        yield AGNR(agnr)
