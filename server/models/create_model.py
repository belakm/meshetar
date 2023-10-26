#%%
import pandas as pd
import numpy as np
from ta import add_all_ta_features
from ta.utils import dropna
import matplotlib.pyplot as plt
import sqlite3
from scipy.signal import find_peaks
import seaborn as sns
import tensorflow as tf
from tensorflow.keras.callbacks import LearningRateScheduler
from scipy.special import softmax
from sklearn.preprocessing import StandardScaler, LabelEncoder, RobustScaler
from sklearn.metrics import roc_curve, roc_auc_score, RocCurveDisplay, confusion_matrix
from sklearn.model_selection import train_test_split
import pickle

pd.set_option('display.max_rows', 500)
pd.set_option('display.max_columns', 50)

# %%
import os
print(os.path.abspath('./database.sqlite'))
conn = sqlite3.connect('./database.sqlite')
# cursor = sqliteConnection.cursor()

# %%
query = """SELECT datetime(open_time / 1000, 'unixepoch') AS open_time,
                 high, 
                 low, 
                 close, 
                 volume
          FROM candles 
          WHERE asset = 'BTCUSDT';"""

# %%
klines = pd.read_sql_query(query, conn)
klines['open_time'] = pd.to_datetime(klines['open_time'])
klines.loc[:, klines.columns.difference(['open_time'])] = klines.loc[:, klines.columns.difference(['open_time'])].apply(pd.to_numeric, errors='coerce')
#%%
klines = add_all_ta_features(klines,
                             open = "open", 
                             close = "close",
                             volume = "volume",
                             low = "low",
                             high = "high",
                             fillna=True).dropna()
# %%
peaks, _ = find_peaks(klines["close"], distance = 12)
klines["close"] = klines["close"].astype(float)
valleys, _ = find_peaks(-klines["close"], distance  = 12)
# plt.plot( klines["close"])
# plt.plot(peaks, klines["close"][peaks], "x", color = "green")
# plt.plot(valleys, klines["close"][valleys], "+", color = "red")
# plt.show()

# %%
klines["signal"] = "hold"
klines.loc[valleys, 'signal'] = "buy"
klines.loc[peaks, 'signal'] = "sell"
# %%
X_train , X_test, y_train , y_test = train_test_split(klines[klines.columns[~klines.columns.isin(['open_time', 'open','close', 'low', 'high', 'volume', 'signal'])]], klines[['signal']], test_size=0.2, random_state=44)
# %%
# y_train['target'] = y_train['signal'].idxmax(axis = 0)
# y_test['target'] = y_test.idxmax(axis=0)
# %%
label_encoder = LabelEncoder()
y_train['target_encoded'] = label_encoder.fit_transform(y_train)
y_test['target_encoded'] = label_encoder.transform(y_test)
# %%
scaler = RobustScaler()
X_train = scaler.fit_transform(X_train)
X_test = scaler.fit_transform(X_test)
#%%
def schedule(epoch):
    if epoch < 10:
        return 3e-4
    return 1e-4 if epoch < 25 else 3e-5
scheduler = LearningRateScheduler(schedule)
#%%
model = tf.keras.Sequential([
    tf.keras.layers.Input(shape=(X_train.shape[1],)),
    tf.keras.layers.Dense(X_train.shape[1]*2, activation='relu'),
    tf.keras.layers.Dense(X_train.shape[1], activation='relu'),
    tf.keras.layers.Dense(X_train.shape[1]/2, activation='relu'),
    tf.keras.layers.Dense(len(label_encoder.classes_), activation='softmax')
])
model.summary()

#%%
model.compile(
    optimizer='adam',
    loss='sparse_categorical_crossentropy', 
)
# %%
class_weights = len(y_train)/(3*y_train['target_encoded'].value_counts())
normalized_class_weights = {cls: weight / sum(class_weights) for cls, weight in class_weights.items()}
#%%
decay_rate = 0.0005
sample_weights = np.exp(-decay_rate * (len(y_train) - y_train.index))
plt.plot(sample_weights)
#%%
history = model.fit(
    X_train, y_train['target_encoded'],
    epochs=1000, batch_size=86,
    validation_data=(X_test, y_test['target_encoded']),
    callbacks=[scheduler],
    sample_weight=sample_weights,
    # class_weight=normalized_class_weights,
)
#%%
plt.plot(history.history["loss"])
plt.plot(history.history["val_loss"])
plt.show()
#%%
train_proba = model.predict(X_train)
test_proba = model.predict(X_test)
# %%
cutoffs = []  # Create an empty list to store the cutoffs
for class_index in range(len(label_encoder.classes_)):
    fpr_train, tpr_train, thresholds_train = roc_curve(y_train['target_encoded'] == class_index, train_proba[:, class_index])
    roc = RocCurveDisplay.from_predictions(y_train['target_encoded'] == class_index, train_proba[:, class_index])
    roc.plot()
    plt.title(f"Train roc for class: {class_index}")
    plt.show()

    fpr_test, tpr_test, thresholds_test = roc_curve(y_test['target_encoded'] == class_index, test_proba[:, class_index])
    roc = RocCurveDisplay.from_predictions(y_test['target_encoded'] == class_index, test_proba[:, class_index])
    roc.plot()
    plt.title(f"Test roc for class: {class_index}")
    plt.show()

    optimal_cutoff_index = np.argmax(tpr_train - fpr_train)
    optimal_cutoff = thresholds_train[optimal_cutoff_index]

    cutoffs.append(optimal_cutoff)  # Append the class index and optimal cutoff as a tuple    print(f"{class_index, optimal_cutoff =}")

    y_train[f'model_prediction_V{class_index+1}'] = list(zip(*train_proba))[class_index] > optimal_cutoff
    y_test[f'model_prediction_V{class_index+1}'] = list(zip(*test_proba))[class_index] > optimal_cutoff

# %%
with open('neural_net_model/cutoffs.pickle', 'wb') as handle:
    pickle.dump(cutoffs, handle, protocol=-1)

# %%
def set_model_prediction(row):
    if row["model_prediction_V1"]:
        return "buy"
    elif row["model_prediction_V3"]:
        return "sell"
    else:
        return "hold"
y_train['model_prediction'] = y_train.apply(set_model_prediction, axis=1).astype(str)
y_test['model_prediction'] = y_test.apply(set_model_prediction, axis=1).astype(str)

# %%
conf_matrix = confusion_matrix(y_train['signal'], y_train['model_prediction'], labels=label_encoder.classes_)
plt.figure(figsize=(8, 6))
sns.heatmap(conf_matrix, annot=True, fmt='d', cmap='Blues', xticklabels=label_encoder.classes_, yticklabels=label_encoder.classes_)
plt.xlabel('Model Prediction')
plt.ylabel('Actual Target')
plt.title('Confusion Matrix train')
plt.show()
# %%
conf_matrix = confusion_matrix(y_test['signal'], y_test['model_prediction'], labels=label_encoder.classes_)
plt.figure(figsize=(8, 6))
sns.heatmap(conf_matrix, annot=True, fmt='d', cmap='Blues', xticklabels=label_encoder.classes_, yticklabels=label_encoder.classes_)
plt.xlabel('Model Prediction')
plt.ylabel('Actual Target')
plt.title('Confusion Matrix test')
plt.show()

model.save("./neural_net_model")

# %%
