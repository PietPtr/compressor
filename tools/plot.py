# This is a seperate Python script because matplotlib can produce a nice interactive viewer much more
# easily than the plotters library ever could (056b9854d82a363b49f44ecfddcd65633051df04)

import os
import sys
os.environ['QT_QPA_PLATFORM'] = 'xcb'

import matplotlib.pyplot as plt
import pandas as pd
import numpy as np
import hashlib

sample_rate = 44100


def string_to_color(input_string):
    hash_object = hashlib.md5(input_string.encode())
    hex_dig = hash_object.hexdigest()
    color = '#' + hex_dig[:6]
    return color


def plot_csv(file_path):
    df = pd.read_csv(file_path)

    plt.figure(figsize=(20, 12))

    num_samples = len(df)
    time_ms = np.linspace(0, num_samples / sample_rate * 1000, num_samples)

    plt.plot(time_ms, df['sample'], label='sample', color=string_to_color('sample'))
    
    for column in sorted([col for col in df.columns if col != 'sample']):
        plt.plot(time_ms, df[column], label=column, color=string_to_color(column))

    plt.xlabel('Time (ms)')
    plt.ylabel('Value')
    
    plt.legend()
    
    plt.show()

plot_csv(sys.argv[1])