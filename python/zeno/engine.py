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
        use_threads=True,
        cse=True,
        fastmath=True,
        opt_level=1,
        convert=True,
        defuns=None,
        sanitize=True,
        dtype="float64",
        action="compile",
        file="",
        num_params=1,
        order="fortran",
        simd_branch=False,
        fast_complex=True,
        direct=True,
        compress=False,
        huge=False,
        parallel_mul=True,
    ):
        if convert:
            model = json.dumps(model)

        dtype = str(dtype)
        if dtype not in ["float64", "complex128"]:
            raise ValueError("`dtype` should be `float64` or `complex128`")

        if order not in ["c", "fortran"]:
            raise ValueError("`order` should be either `c` or `fortran`")

        opt = (
            (0x01 if use_simd else 0)
            | (0x00000002 if use_threads else 0)
            | (0x00000004 if cse else 0)
            | (0x00000008 if fastmath else 0)
            | (0x00000010 if sanitize else 0)
            | (0x00000020 if dtype == "complex128" else 0)
            | (0x00000040 if order == "c" else 0)
            | (0x00000080 if simd_branch else 0)
            | (0x00002000 if compress else 0)
            | (0x00004000 if direct else 0)
            | (0x00008000 if fast_complex else 0)
            | (0x00100000 if huge else 0)
            | (0x00200000 if parallel_mul else 0)
            | ((opt_level & 0x0F) << 8)
        )

        self.dtype = dtype
        self.ty = ty

        if action == "translate":
            self.p = lib._translate(
                model.encode("utf-8"), ty.encode("utf8"), opt, num_params
            )
            self.symbolica = True
        else:
            raise ValueError(f"action {action} not defined")

        status = lib._check_status(self.p)
        if status != b"Success":
            raise ValueError(status.decode())

        self.model = model
        self.json_model = None
        self.populate()

    def __del__(self):
        if hasattr(self, "p"):
            lib._finalize(self.p)

    def populate(self):
        pass

    def dump(self, name, what="scalar"):
        if not lib._dump(self.p, name.encode("utf-8"), what.encode("utf-8")):
            raise ValueError("cannot dump the requested code")
        with open(name, "rb") as fd:
            buf = fd.read()
            return buf

    def dumps(self, what="scalar"):
        name = "symjit_dump.bin"
        self.dump(name, what=what)
        with open(name, "rb") as fd:
            b = fd.read()
        os.remove(name)

        if b[0] == ord("#") and b[1] == ord("!"):
            return b.decode("utf8")
        else:
            return b.hex()

    def execute(self):
        if not lib._execute(self.p):
            raise ValueError("cannot execute the model")

    def execute_vectorized(self, buf):
        ptr = buf.ctypes.data_as(ctypes.POINTER(ctypes.c_double))
        n = buf.shape[1]
        if not lib._execute_vectorized(self.p, ptr, n):
            raise ValueError("cannot execute the model")

    def execute_matrix(self, states, obs):
        if not lib._execute_matrix(self.p, states.p, obs.p):
            raise ValueError("cannot execute the model")

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

    def fast_func(self):
        if self.ty == "bytecode":
            return None

        f = lib._fast_func(self.p)

        if f is None:
            return None

        sig = [ctypes.c_double for _ in range(self.count_states + 1)]
        fac = ctypes.CFUNCTYPE(*sig)
        return fac(f)

    def callable_quad(self, use_fast=True):
        f = lib._fast_func(self.p)

        try:
            from scipy import LowLevelCallable

            if f is not None and use_fast:
                return LowLevelCallable(
                    lib._callable_quad_fast,
                    user_data=ctypes.c_void_p(f),
                    signature="double (int, double *, void *)",
                )
            else:
                return LowLevelCallable(
                    lib._callable_quad,
                    user_data=ctypes.c_void_p(self.p),
                    signature="double (int, double *, void *)",
                )
        except:
            return None

    def callable_filter(self, use_fast=True):
        try:
            from scipy import LowLevelCallable

            return LowLevelCallable(
                lib._callable_filter,
                user_data=ctypes.c_void_p(self.p),
                signature="int (double *, npy_intp, double *, void *)",
            )

        except:
            return None
