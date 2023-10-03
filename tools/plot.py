# This is a seperate Python script because matplotlib can produce a nice interactive viewer much more
# easily than the plotters library ever could (056b9854d82a363b49f44ecfddcd65633051df04)

import os
import sys
os.environ['QT_QPA_PLATFORM'] = 'xcb'

# Importing required libraries
import matplotlib.pyplot as plt
import pandas as pd
import numpy as np

sample_rate = 44100

# Function to plot data from a CSV file
def plot_csv(file_path):
    df = pd.read_csv(file_path)

    plt.figure(figsize=(20, 12))

    num_samples = len(df)
    time_ms = np.linspace(0, num_samples / sample_rate * 1000, num_samples)
    
    for column in df.columns:
        plt.plot(time_ms, df[column], label=column)

    plt.xlabel('Time (ms)')
    plt.ylabel('Value')
    
    plt.legend()
    
    plt.show()

plot_csv(sys.argv[1])