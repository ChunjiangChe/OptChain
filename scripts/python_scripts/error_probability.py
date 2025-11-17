import numpy as np
import matplotlib.pyplot as plt

shard_sizes = [2, 4, 6, 8, 10, 16]
# shard_sizes = [2, 4, 8, 16]
honest_node_num = 64
highest_bandwidth = 60

optimal_throughputs = []
errors = []

for shard_size in shard_sizes:
    error_1 = (1 - (1 / shard_size)) ** honest_node_num
    errors.append(error_1)
    optimal_throughputs.append(shard_size * highest_bandwidth)

# Print results
for i in range(len(shard_sizes)):
    print(f"shards: {shard_sizes[i]} Error {errors[i]:.2e} Optimal Throughput: {optimal_throughputs[i]}")

# Plot
plt.figure(figsize=(8, 5))
plt.plot(errors, optimal_throughputs, marker='o', linestyle='-', color='b')

plt.xscale('log')  # logarithmic scale for exponential appearance
plt.xlabel("Error (exponential notation)")
plt.ylabel("Optimal Throughput")
plt.title("Optimal Throughput vs Error for Different Shard Sizes")
plt.grid(True, which="both", ls="--", lw=0.5)
plt.tight_layout()
plt.savefig('./optimal_throughput_vs_error.png', dpi=300, bbox_inches='tight')

# error = (((64+1)/(64*4+1))**4) * ((64+1)/(64*4*3))
# print("error: {}", error)