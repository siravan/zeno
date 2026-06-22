from cProfile import label

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

df = pd.read_excel("zeno_benchmarks.xlsx")

first = 1

nt = df.iloc[first:, 1]
bytes = 314 * nt
eager = df.iloc[first:, 2]
symjit_no_simd = df.iloc[first:, 3]
symjit_simd = df.iloc[first:, 4]
zeno_no_simd = df.iloc[first:, 5]
zeno_simd = df.iloc[first:, 6]

fig, ax = plt.subplots(1, 1, figsize=(6, 6))

ax.loglog([960000, 960000], [0.01, 10000], "--", color="gray")
ax.loglog([12000000, 12000000], [0.01, 10000], "--", color="gray")
ax.loglog([32000000, 32000000], [0.01, 10000], "--", color="gray")


ax.loglog(bytes, eager, ".-", label="eager")
ax.loglog(bytes, symjit_no_simd, "+-", label="symjit (f64)")
ax.loglog(bytes, symjit_simd, "+-", label="symjit (f64x4)")
ax.loglog(bytes, zeno_no_simd, "o-", label="zeno (f64)")
ax.loglog(bytes, zeno_simd, "o-", label="zeno (f64x4)")


ax.text(650000, 0.5, "L1")
ax.text(8000000, 0.5, "L2")
ax.text(20000000, 0.5, "L3")

ax.set_xlabel("code size (bytes)")
ax.set_ylabel("time (μsec)")

ax.legend()
plt.show()
