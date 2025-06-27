import sys
import torch
from safetensors.torch import save_file

if (len(sys.argv) != 2):
    print("Usage: python3 ./generate_data.py <N>")
    sys.exit(1)

torch.manual_seed(1)

N = int(sys.argv[1])

A = torch.randn(N,N)
B = torch.randn(N,N)
C = torch.matmul(A, B)

save_file({
    "A": A,
    "B": B,
    "C": C
}, "./data/matmul_data.safetensors")