import os
import pandas as pd

IP_TO_NAME = {
    "192.168.42.42:8080": "Accelerated CNF",
    "00.00.00.00:8080": "Private Cloud",
    "00.00.00.00:8080": "Public Cloud",
}

def merge_csv_files(input_folder, output_file):
    # List all CSV files in the input folder
    csv_files = [f for f in os.listdir(input_folder) if f.endswith('.csv')]
    print(csv_files)
    # Initialize an empty list to hold dataframes
    dataframes = []
    
    # Read each CSV file and append to the list
    for file in csv_files:
        file_path = os.path.join(input_folder, file)
        df = pd.read_csv(file_path)
        dataframes.append(df)
    
    # Concatenate all dataframes
    merged_df = pd.concat(dataframes, ignore_index=True)
    
    # Save the merged dataframe to a new CSV file
    merged_df.to_csv(output_file, index=False)

def group_by_column(input_file, output_file, group_column):
    # Read the input CSV file
    df = pd.read_csv(input_file)
    
    # Group by the specified column
    grouped_df = df.groupby(group_column).mean()

    # Rename the "Server Address" column to "Server Name"
    grouped_df = grouped_df.rename(index=IP_TO_NAME)
    
    # Save the grouped dataframe to a new CSV file
    grouped_df.to_csv(output_file)

def set_rtt_time_scale(input_file, output_file, scale):
    # Read the input CSV file
    df = pd.read_csv(input_file)
    
    # Set the RTT time scales for each row and column where "RTT" is in the column name
    for column in df.columns:
        if 'RTT' in column:
            df[column] = df[column] * scale
    
    # Save the scaled dataframe to a new CSV file
    df.to_csv(output_file, index=False)

def transpose_csv(input_file, output_file):
    # Read the input CSV file
    df = pd.read_csv(input_file)
    
    # Transpose the dataframe
    transposed_df = df.T
    
    # Save the transposed dataframe to a new CSV file
    transposed_df.to_csv(output_file, header=False)

def sort_csv(input_file, output_file, sort_column):
    # Read the input CSV file
    df = pd.read_csv(input_file)
    
    # Sort the dataframe by the specified column
    sorted_df = df.sort_values(by=sort_column, ascending=True)
    
    # Save the sorted dataframe to a new CSV file
    sorted_df.to_csv(output_file, index=False)

    

if __name__ == "__main__":
    input_folder = 'results/'
    output_file = 'merged_rtt_times.csv'
    print('Merging CSV files...')
    merge_csv_files(input_folder, output_file)
    print('Grouping by server address')
    group_by_column(output_file, 'grouped_rtt_times.csv', ['Server Address', 'Speedtest-Mode'])
    print('Setting RTT time scale to milliseconds')
    set_rtt_time_scale('grouped_rtt_times.csv', 'scaled_rtt_times.csv', 1/1000)
    print('Sorting the CSV file by server address')
    sort_csv('scaled_rtt_times.csv', 'sorted_rtt_times.csv', 'Server Address')
    print('Transposing the CSV file')
    transpose_csv('sorted_rtt_times.csv', 'transposed_rtt_times.csv')
    print('Done!')
