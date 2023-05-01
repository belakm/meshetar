library(RSQLite)

# Connect to the SQLite database
conn <- dbConnect(RSQLite::SQLite(), "database.sqlite")

# Query the klines table and retrieve the historical data
query <- "SELECT * FROM klines WHERE symbol = 'BTCUSDT' ORDER BY open_time ASC"
data <- dbGetQuery(conn, query)

# Exclude any rows that contain NA, NaN, or Inf values
data <- data[complete.cases(data), ]

# Preprocess the data
data$open_time <- as.POSIXct(data$open_time/1000, origin="1970-01-01")
data$close_time <- as.POSIXct(data$close_time/1000, origin="1970-01-01")
data$range <- data$high - data$low
data$target <- ifelse(data$close > data$open, -1, 
                      ifelse(data$close < data$open, 1, 0))

# Train your analytical model on the preprocessed data
model <- lm(target ~ range + volume + quote_asset_volume + number_of_trades + taker_buy_base_asset_volume + taker_buy_quote_asset_volume, data = data)

# Save the trained model to a file
saveRDS(model, "models/prediction_model.rds")

# Disconnect from the database
dbDisconnect(conn)
