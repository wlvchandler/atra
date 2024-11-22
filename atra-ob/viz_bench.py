import json
import matplotlib.pyplot as plt
import seaborn as sns
import pandas as pd
import numpy as np

sns.set_theme()

with open('bench_results.json') as f:
    results = json.load(f)

# depth impact
depths = [m['depth'] for m in results['depth_impact']]
latencies = [m['latency_ns'] for m in results['depth_impact']]

plt.figure(figsize=(10, 6))
sns.lineplot(x=depths, y=latencies, marker='o')
plt.title('Order Book Depth Impact on Latency')
plt.xlabel('Order Book Depth')
plt.ylabel('Latency (ns)')
plt.grid(True)
plt.savefig('img/depth_impact.png')
plt.close()

# latency distribution
plt.figure(figsize=(10, 6))
sns.histplot(data=results['latency_distribution'], kde=True)
plt.title('Order Processing Latency Distribution')
plt.xlabel('Latency (ns)')
plt.ylabel('Frequency')
plt.grid(True)
plt.savefig('img/latency_distribution.png')
plt.close()

# batch performance
batch_sizes = [m['batch_size'] for m in results['batch_performance']]
throughput = [m['orders_per_second'] for m in results['batch_performance']]

plt.figure(figsize=(10, 6))
sns.barplot(x=batch_sizes, y=throughput)
plt.title('Throughput vs Batch Size')
plt.xlabel('Batch Size')
plt.ylabel('Orders per Second')
plt.grid(True)
plt.savefig('img/batch_performance.png')
plt.close()




latencies = pd.Series(results['latency_distribution'])
print("\nLatency Statistics (ns):")
print(f"Mean: {latencies.mean():.2f}")
print(f"Median: {latencies.median():.2f}")
print(f"95th percentile: {latencies.quantile(0.95):.2f}")
print(f"99th percentile: {latencies.quantile(0.99):.2f}")
