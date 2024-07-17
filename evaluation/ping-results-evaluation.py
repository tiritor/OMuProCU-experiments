
import pandas as pd
import matplotlib.pyplot as plt

### Pinger Results Evaluation ###

# df : pd.DataFrame = pd.read_csv('ping-results-table-test.csv', )
df : pd.DataFrame = pd.read_csv('ping-results-11.csv', )

print ("Dataframe: ", df)
print ("Mean: ", df.mean().values)
print ("Standard Deviation: ", df.std().values)
print ("Median: ", df.median().values)
print ("Minimum: ", df.min().values)
print ("Maximum: ", df.max().values)

# plot this data using matplotlib (x-axis: time, y-axis: ping) (Data frame structure: time, ping)

# df['Ping_Diff'] = df.mean(axis=1).diff().abs()

# print("Jitter: ", df['Ping_Diff'].mean(), " µs")
# print(df['Ping_Diff'].mean())

# latencies = df['latency']
# print(latencies)

plt.plot(range(len(df)),df.mean(axis=1))
# plt.plot(range(len(df)),df["experiment_1"], label="Experiment 1")
plt.xlabel('Time')
plt.ylabel('Ping')
plt.title('Ping Results')
plt.savefig('ping-results.pdf')
plt.close()

# print(df.mean(), df.std(), df.median(), df.min(), df.max()

# Show before half and after half time ping results
before_half_stats = df.iloc[0:int(len(df)/2)] #.describe()
after_half_stats = df.iloc[int(len(df)/2):]

# Calculate the aggregated statistics
# print("Before half mean:" , before_half_stats.mean(axis=1).mean())
# print("After half mean:" , after_half_stats.mean(axis=1).mean())
# print("Before half std:" , before_half_stats.mean(axis=1).std())
# print("After half std:" , after_half_stats.mean(axis=1).std())
# print("Before half median:" , before_half_stats.mean(axis=1).median())
# print("After half median:" , after_half_stats.mean(axis=1).median())
# print("Before half min:" , before_half_stats.mean(axis=1).min())
# print("After half min:" , after_half_stats.mean(axis=1).min())
# print("Before half max:" , before_half_stats.mean(axis=1).max())
# print("After half max:" , after_half_stats.mean(axis=1).max())
# Create a new dataframe to store the aggregated statistics
aggregated_stats = pd.DataFrame(columns=['Statistic', 'Before Half', 'After Half', 'Difference in %'])

# Calculate the aggregated statistics
aggregated_stats.loc[0] = ['Mean', before_half_stats.mean(axis=1).mean(), after_half_stats.mean(axis=1).mean(), round((after_half_stats.mean(axis=1).mean() - before_half_stats.mean(axis=1).mean()) / before_half_stats.mean(axis=1).mean() * 100, 3)]
aggregated_stats.loc[1] = ['Standard Deviation', before_half_stats.mean(axis=1).std(), after_half_stats.mean(axis=1).std(), round((after_half_stats.mean(axis=1).std() - before_half_stats.mean(axis=1).std()) / before_half_stats.mean(axis=1).std() * 100, 3)]
aggregated_stats.loc[2] = ['Median', before_half_stats.mean(axis=1).median(), after_half_stats.mean(axis=1).median(), round((after_half_stats.mean(axis=1).median() - before_half_stats.mean(axis=1).median()) / before_half_stats.mean(axis=1).median() * 100, 3)]
aggregated_stats.loc[3] = ['Minimum', before_half_stats.mean(axis=1).min(), after_half_stats.mean(axis=1).min(), round((after_half_stats.mean(axis=1).min() - before_half_stats.mean(axis=1).min()) / before_half_stats.mean(axis=1).min() * 100, 3)]
aggregated_stats.loc[4] = ['Maximum', before_half_stats.mean(axis=1).max(), after_half_stats.mean(axis=1).max(), round((after_half_stats.mean(axis=1).max() - before_half_stats.mean(axis=1).max()) / before_half_stats.mean(axis=1).max() * 100, 3)]

# Print the aggregated statistics
print(aggregated_stats.to_string(index=False))

# Save the aggregated statistics to a CSV and LaTeX file
aggregated_stats.to_csv('aggregated-stats.csv', index=False)

aggregated_stats.to_latex('aggregated-stats.tex', index=False)

plt.bar(["Before Rules Update", "After Rules Update"], [before_half_stats.mean(axis=1).mean(), after_half_stats.mean(axis=1).mean()])
plt.ylabel('Mean Ping (µs)')
plt.title('Ping Results Before and After Rules Update')
plt.savefig('ping-results-before-after.pdf')
plt.close()

############################################################################################################
### MOC Shell Timetracking Evaluation ###
# Read the CSV file
df = pd.read_csv("moc_shell_timetracking-1.csv")

# Calculate the average latency for each command as group
df["average_latency"] = df.groupby("Command")["Processing Time"].transform("mean")

print(df)

# combined_stats = pd.concat([before_half_stats, after_half_stats], axis=1)
# print(combined_stats)