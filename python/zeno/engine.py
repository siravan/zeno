import ctypes
import json
import os
import platform
import sys

import numpy as np


class Engine:
    def __init__(self):
        dll_name = None

        if sys.platform == "linux" and platform.machine() == "x86_64":
            dll_name = self.find_dll("x86_64-linux")
        if sys.platform == "linux" and platform.machine() == "aarch64":
            dll_name = self.find_dll("aarch64-linux")
        if sys.platform == "linux" and platform.machine() == "riscv64":
            dll_name = self.find_dll("riscv64-linux")
        if sys.platform == "darwin":
            dll_name = self.find_dll("darwin")
        elif sys.platform == "win32":
            dll_name = self.find_dll("win_amd64")

        if dll_name is None:
            self.is_valid = False
            return

        try:
            dll_path = os.path.join(os.path.dirname(__file__), dll_name)
            self.dll = ctypes.CDLL(dll_path)
            self.populate()
            self.is_valid = True
        except AttributeError as e:
            print(e)
            self.is_valid = False

    def populate(self):
        self._info = self.dll.info
        self._info.argtypes = []
        self._info.restype = ctypes.c_char_p

        self._check_status = self.dll.check_status
        self._check_status.argtypes = [ctypes.c_void_p]
        self._check_status.restype = ctypes.c_char_p

        self._translate = self.dll.translate
        self._translate.argtypes = [
            ctypes.c_char_p,
            ctypes.c_char_p,
            ctypes.c_uint32,
            ctypes.c_size_t,
        ]
        self._translate.restype = ctypes.c_void_p

        self._finalize = self.dll.finalize
        self._finalize.argtypes = [ctypes.c_void_p]
        self._finalize.restype = None

        self._evaluate = self.dll.evaluate
        self._evaluate.argtypes = [
            ctypes.c_void_p,  # handle
            ctypes.POINTER(ctypes.c_double),  # args
            ctypes.c_size_t,  # nargs
            ctypes.POINTER(ctypes.c_double),  # outs
            ctypes.c_size_t,  # nouts
        ]
        self._evaluate.restype = ctypes.c_bool

        self._evaluate_matrix = self.dll.evaluate_matrix
        self._evaluate_matrix.argtypes = [
            ctypes.c_void_p,  # handle
            ctypes.POINTER(ctypes.c_double),  # args
            ctypes.c_size_t,  # nargs
            ctypes.POINTER(ctypes.c_double),  # outs
            ctypes.c_size_t,  # nouts
        ]
        self._evaluate_matrix.restype = ctypes.c_bool

        self._count_params = self.dll.count_params
        self._count_params.argtypes = [ctypes.c_void_p]
        self._count_params.restype = ctypes.c_size_t

        self._count_outs = self.dll.count_outs
        self._count_outs.argtypes = [ctypes.c_void_p]
        self._count_outs.restype = ctypes.c_size_t

    def info(self):
        return self._info()

    def find_dll(self, substr):
        files = os.listdir(os.path.dirname(__file__))
        matches = list(filter(lambda s: s.find(substr) >= 0, files))
        if len(matches) == 0:
            return None
        else:
            return matches[0]


#################################################################

lib = Engine()  # interface to the rust codegen engine


def from_raw_parts(ptr, count):
    if count == 0:
        return np.zeros(1)
    else:
        return np.ctypeslib.as_array(ptr, shape=(count,))


class RustyCompiler:
    def __init__(
        self,
        model,
        ty="native",
        use_simd=True,
        use_simd512=False,
        opt_level=2,
        dtype="float64",
        num_params=1,
        simd_branch=False,
    ):
        dtype = str(dtype)
        if dtype not in ["float64", "complex128"]:
            raise ValueError("`dtype` should be `float64` or `complex128`")

        opt = (
            (0x00000001 if use_simd else 0)
            | (0x00000002 if use_simd512 else 0)
            | (0x00000020 if dtype == "complex128" else 0)
            | (0x00000080 if simd_branch else 0)
            | ((opt_level & 0x0F) << 8)
        )

        self.dtype = dtype
        self.ty = ty

        self.p = lib._translate(
            model.encode("utf-8"), ty.encode("utf8"), opt, num_params
        )
        self.symbolica = True

        status = lib._check_status(self.p)
        if status != b"Success":
            raise ValueError(status.decode())

        self.model = model
        self.populate()

    def __del__(self):
        if hasattr(self, "p"):
            lib._finalize(self.p)

    def populate(self):
        self.count_params = lib._count_params(self.p)
        self.count_obs = lib._count_outs(self.p)

    def evaluate(self, args, outs):
        pargs = args.ctypes.data_as(ctypes.POINTER(ctypes.c_double))
        nargs = args.size
        pouts = outs.ctypes.data_as(ctypes.POINTER(ctypes.c_double))
        nouts = outs.size

        if not lib._evaluate(self.p, pargs, nargs, pouts, nouts):
            raise ValueError("cannot evaluate the model")

    def evaluate_matrix(self, args, outs, k=1):
        pargs = args.ctypes.data_as(ctypes.POINTER(ctypes.c_double))
        nargs = args.size
        pouts = outs.ctypes.data_as(ctypes.POINTER(ctypes.c_double))
        nouts = outs.size

        if not lib._evaluate_matrix(self.p, pargs, nargs * k, pouts, nouts * k):
            raise ValueError("cannot evaluate the model")
