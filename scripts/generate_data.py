import sys
import torch
from safetensors.torch import save_file
from pathlib import Path

if (len(sys.argv) != 2):
    print("Usage: python3 ./generate_data.py <N>")
    sys.exit(1)

SCRIPT_DIR = Path(__file__).parent.resolve()
DATA_DIR = SCRIPT_DIR.parent / "data"
DATA_DIR.mkdir(exist_ok=True)
OUTPUT_FILE = DATA_DIR / "matmul_data2.safetensors"


N = int(sys.argv[1])

A = torch.randn(N,N)
B = torch.randn(N,N)
C = torch.matmul(A, B)

save_file({
    "A": A,
    "B": B,
    "C": C
}, str(OUTPUT_FILE))