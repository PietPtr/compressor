# This is a seperate Python script because matplotlib can produce a nice interactive viewer much more
# easily than the plotters library ever could (056b9854d82a363b49f44ecfddcd65633051df04)

import os
import sys
os.environ['QT_QPA_PLATFORM'] = 'xcb'

import matplotlib.pyplot as plt
import pandas as pd
import numpy as np
import hashlib
import tkinter as tk

sample_rate = 44100

root = tk.Tk()
screen_width = root.winfo_screenwidth()

def string_to_color(input_string):
    hash_object = hashlib.md5(input_string.encode())
    hex_dig = hash_object.hexdigest()
    color = '#' + hex_dig[:6]
    return color

def plot_sample_lines(ax, length, sample_rate=44100):
    if length > 5000:
        return # Too long, takes too much time
    for i in range(length):
        ax.axvline(x=i/(sample_rate / 1000.0), color='#777777', linestyle='-', linewidth=0.1)


def plot_csv2(file_path, sample_rate=44100):
    df = pd.read_csv(file_path)

    fig, ax = plt.subplots(figsize=(13, 7))

    num_samples = len(df)
    time_ms = np.linspace(0, num_samples / sample_rate * 1000, num_samples)

    ax.axhline(y=0, color='#777777', linestyle='-', linewidth=1)
    plot_sample_lines(ax, num_samples)

    ax.plot(time_ms, df['sample'], label='sample', color=string_to_color('sample'))

    for column in sorted([col for col in df.columns if col != 'sample']):
        ax.plot(time_ms, df[column], label=column, color=string_to_color(column))

    ax.set_xlabel('Time (ms)')
    ax.set_ylabel('Value')

    plt.legend()

    if screen_width > 2560:
        fig.canvas.manager.window.setGeometry(2660, 100, 2300, 1210)

    fig.canvas.manager.set_window_title("Float")

    plt.show()

plot_csv2(sys.argv[1])