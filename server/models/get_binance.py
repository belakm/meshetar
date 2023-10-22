# %%
import ccxt
import time
import toml

#%%
# Read and parse the TOML configuration file
config = toml.load('../config.toml')
exchange = ccxt.binance({
    'apiKey': config["binance_api_key"],
    'secret': config["binance_api_secret"],
})
exchange.fetch_balance()
# %%
symbol = 'BTC/USDT'  # Replace with the pair you want
timeframe = '5m'    # 
limit = 10000  # Adjust to the number of candles you need
candles = exchange.fetch_ohlcv(symbol, timeframe, limit=limit)
# %%
