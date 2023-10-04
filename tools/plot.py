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



def plot_csv2(file_path, sample_rate=44100):
    df = pd.read_csv(file_path)

    fig, ax = plt.subplots(figsize=(20, 12))

    num_samples = len(df)
    time_ms = np.linspace(0, num_samples / sample_rate * 1000, num_samples)

    ax.axhline(y=0, color='#777777', linestyle='-', linewidth=1)

    ax.plot(time_ms, df['sample'], label='sample', color=string_to_color('sample'))

    for column in sorted([col for col in df.columns if col != 'sample']):
        ax.plot(time_ms, df[column], label=column, color=string_to_color(column))

    ax.set_xlabel('Time (ms)')
    ax.set_ylabel('Value')

    plt.legend()

    fig.canvas.manager.window.setGeometry(2660, 100, 2300, 1210)

    plt.show()

plot_csv2(sys.argv[1])