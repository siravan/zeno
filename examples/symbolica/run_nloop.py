import os
import sys
import time

import numpy as np
from zeno import compile_evaluator

if len(sys.argv) < 2:
    print("use nloop n; where n=1, 2, or 3")

CONFIG = os.path.join(os.path.dirname(__file__), "symjit.toml")
INSTRUCTIONS = os.path.join(
    os.path.dirname(__file__), f"{sys.argv[1]}loop_instructions_2.txt"
)

with open(INSTRUCTIONS, "rt", encoding="utf-8") as fd:
    evaluator = fd.read()

print("Building zeno evaluator...")
t_start = time.time()
f = compile_evaluator(evaluator, dtype="complex128", ty=CONFIG)
print(f"completed in {time.time() - t_start:.1f} s.")

n = f.complex_compiler.count_params // 2

N_SAMPLES = 1000

rng = np.random.default_rng(1337)
samples_real = rng.random(n)
samples_imag = rng.random(n)
samples = samples_real + 1j * samples_imag

t_start = time.time()
for _ in range(N_SAMPLES):
    f.evaluate_complex(samples[None, :])
print(f"Symjit evaluation: {((time.time() - t_start) * 1000.0 / N_SAMPLES):.3f} ms")
print(f.evaluate_complex(samples[None, :]).sum())


