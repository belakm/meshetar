library(RSQLite)

# Connect to the SQLite database
conn <- dbConnect(RSQLite::SQLite(), "database.sqlite")

# Load the trained model from the file
model <- readRDS("models/prediction_model.rds")

# Query the klines table and retrieve the latest data for the chosen crypto pair
query <- "SELECT * FROM klines WHERE symbol = 'BTCUSDT' ORDER BY open_time DESC LIMIT 1"
data <- dbGetQuery(conn, query)
# Disconnect from the database
dbDisconnect(conn)

# Preprocess the data in the same way as the first script
data$open_time <- as.POSIXct(data$open_time/1000, origin="1970-01-01")
data$close_time <- as.POSIXct(data$close_time/1000, origin="1970-01-01")
data$range <- data$high - data$low

# Use the model to predict whether to buy or sell
prediction <- predict(model, newdata = data)

# Output either "buy", "sell" or "none"
# All other output will be treated as "none" and ignored

output <- "hold"
if (prediction > 0.7) {  
  output <- "sell" 
} else if (prediction < -0.7) {  
  output <- "buy"
}

cat(output)
