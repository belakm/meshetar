import tensorflow as tf
import sqlite3
import pandas as pd
from ta import add_all_ta_features


# Load the saved model
loaded_model = tf.keras.models.load_model("server/models/neural_net_model")  

conn = sqlite3.connect('./server/database.sqlite')
# cursor = sqliteConnection.cursor()
query = """SELECT datetime(open_time / 1000, 'unixepoch') AS open_time,
                 high, 
                 low, 
                 close, 
                 volume
          FROM klines
          WHERE symbol = 'BTCUSDT'
          ORDER BY open_time DESC
          LIMIT 50;"""
klines = pd.read_sql_query(query, conn)
klines['open_time'] = pd.to_datetime(klines['open_time'])
klines.loc[:, klines.columns.difference(['open_time'])] = klines.loc[:, klines.columns.difference(['open_time'])].apply(pd.to_numeric, errors='coerce')

# Make predictions using the loaded model

klines = add_all_ta_features(klines,
                             open = "open", 
                             close = "close",
                             volume = "volume",
                             low = "low",
                             high = "high",
                             fillna=True).dropna()
predictions = loaded_model.predict(klines[klines.columns[~klines.columns.isin(['open_time', 'open','close', 'low', 'high', 'volume', 'signal'])]].astype('float32'))


predicted_classes = tf.argmax(predictions, axis=1)  # For a classification model
def set_model_prediction(row):
    if row == 1:
        return "hold"
    elif row == 0:
        return "sell"
    else:
        return "buy"

print(set_model_prediction(predicted_classes[-1]))