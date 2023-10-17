#%%
import pandas as pd
import numpy as np
from ta import add_all_ta_features
import matplotlib.pyplot as plt
import sqlite3
from scipy.signal import find_peaks

# %%
conn = sqlite3.connect('../database.sqlite')
# cursor = sqliteConnection.cursor()
query = """SELECT datetime(open_time / 1000, 'unixepoch') AS open_time,
                 high, 
                 low, 
                 close, 
                 volume
          FROM klines
          WHERE symbol = 'BTCUSDT';"""
klines = pd.read_sql_query(query, conn)
# %%

add_all_ta_features(klines)
# %%
peaks, _ = find_peaks(klines["close"], distance = 500)
valleys, _ = find_peaks(-klines["close"], distance  = 500)
plt.plot( klines["close"])
plt.plot(peaks, klines["close"][peaks], "x", color = "green")
plt.plot(valleys, klines["close"][valleys], "+", color = "red")
plt.show()
# %%
