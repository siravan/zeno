import numbers

import numpy as np

from . import engine


class SymbolicaFunc:
    def __init__(self, model, dtype="float64", **args):
        self.model = model
        self.args = args
        self.samples = None
        self.is_complex = dtype == "complex128"

        if model is None:
            self.compiler = None
            self.complex_compiler = None
            self.args = {}
            return

        if dtype == "complex128":
            self.compile_complex()
            self.compiler = None
        else:
            self.compile_real()
            self.complex_compiler = None

    def compile_real(self):
        compiler = engine.RustyCompiler(self.model, dtype="float64", **self.args)
        self.compiler = compiler

    def compile_complex(self):
        compiler = engine.RustyCompiler(self.model, dtype="complex128", **self.args)
        self.complex_compiler = compiler

    def evaluate(self, inputs):
        if self.compiler is None:
            self.compile_real()

        inputs = np.asarray(inputs)
        c = self.compiler

        assert inputs.shape[1] == c.count_params

        outs = np.zeros((inputs.shape[0], c.count_obs), dtype=np.float64)

        args = np.ascontiguousarray(inputs[:, : c.count_params].real, dtype=np.float64)
        c.evaluate_matrix(args, outs)
        return outs

    def evaluate_complex(self, inputs):
        if self.complex_compiler is None:
            self.compile_complex()

        inputs = np.asarray(inputs)
        c = self.complex_compiler

        assert inputs.shape[1] == c.count_params // 2

        outs = np.zeros((inputs.shape[0], c.count_obs // 2), dtype=np.complex128)

        args = np.ascontiguousarray(inputs, dtype=np.complex128)
        c.evaluate_matrix(args, outs, 2)
        return outs

    def dump(self, name, what="scalar", dtype="complex128"):
        if dtype == "complex128" and self.complex_compiler is not None:
            return self.complex_compiler.dump(name, what=what)
        elif self.compiler is not None:
            return self.compiler.dump(name, what=what)

    def dumps(self, what="scalar", dtype="complex128"):
        if dtype == "complex128" and self.complex_compiler is not None:
            return self.complex_compiler.dumps(what=what)
        elif self.compiler is not None:
            return self.compiler.dumps(what=what)

    def save(self, file, dtype="complex128"):
        if dtype == "complex128":
            self.compile_complex()
            self.complex_compiler.save(file)
        else:
            self.compile_real()
            self.compiler.save(file)

    def __call__(self, *args):
        if self.is_complex:
            return self.evaluate_complex(np.asarray([args], dtype=np.complex128))
        else:
            return self.evaluate(np.asarray([args], dtype=np.float64))
