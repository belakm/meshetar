#%%
import tensorflow as tf
import sqlite3
import pandas as pd
from ta import add_all_ta_features
import warnings
import pickle
from sklearn.preprocessing import RobustScaler
import os
while not os.path.basename(os.getcwd()) == 'server':
    os.chdir('..')  # Move up one directory

def backtest(candle_time=None):
    warnings.simplefilter(action='ignore', category=FutureWarning)
    warnings.simplefilter("ignore", category=RuntimeWarning)

    #%%

    # Load the saved model
    loaded_model = tf.keras.models.load_model("./models/neural_net_model")  # Specify the path to your saved model directory or .h5 file

    #%%
    conn = sqlite3.connect('./database.sqlite')
    # cursor = sqliteConnection.cursor()
    query = """
    SELECT datetime(open_time / 1000, 'unixepoch') AS open_time,
    open,
    high, 
    low, 
    close, 
    volume
    FROM candles
    WHERE asset = 'BTCUSDT'
    ORDER BY open_time DESC
    LIMIT 1440;"""

    klines = pd.read_sql_query(query, conn)
    # Make predictions using the loaded model
    klines = add_all_ta_features(klines,
                                    open = "open", 
                                    close = "close",
                                    volume = "volume",
                                    low = "low",
                                    high = "high",
                                    fillna=True).dropna()
    #%%

    columns_not_to_predict = [
        'open_time', 
        'open',
        'close', 
        'low', 
        'high', 
        'volume']
    klines_to_predict = klines.drop(columns=columns_not_to_predict)

    # lags = range(1, 15)
    # klines_to_predict.assign(**{
    #     f'{col} (t-{lag})': klines_to_predict[col].shift(lag)
    #     for lag in lags
    #     for col in klines_to_predict
    # })

    #%%
    scaler = RobustScaler()
    klines_to_predict = scaler.fit_transform(klines_to_predict.astype('float32'))
    predictions = loaded_model.predict(klines_to_predict)
    file_path = './models/cutoffs.pickle'
    with open(file_path, 'rb') as handle:
        cutoffs = pickle.load(handle)
    cut_predictions = pd.DataFrame()
    for index, cutoff in enumerate(cutoffs):  
        cut_predictions[f'model_prediction_V{index+1}']=  list(zip(*predictions))[index] > cutoff  
    def set_model_prediction(row):
        if row["model_prediction_V1"]:
            return "buy"
        elif row["model_prediction_V3"]:
            return "sell"
        else:
            return "hold"
    cut_predictions['model_prediction'] = cut_predictions.apply(set_model_prediction, axis=1).astype(str)
    return(cut_predictions['model_prediction'].tolist())
