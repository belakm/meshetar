pacman::p_load(RSQLite, TTR, xts, here)
here::i_am("models/default_create.R")

# Connect to the SQLite database
conn <- dbConnect(RSQLite::SQLite(), "database.sqlite")

# Load the trained model from the file
model <- readRDS("models/prediction_model.rds")

# Query the klines table and retrieve the latest data for the chosen crypto pair
query <- "SELECT * FROM klines WHERE symbol = 'BTCUSDT' ORDER BY open_time DESC LIMIT 1"
data <- dbGetQuery(conn, query)
# Disconnect from the database
dbDisconnect(conn)

# Calculate the rate of change (ROC) based on the price data
data$open <- as.numeric(as.character(data$open))
data$high <- as.numeric(as.character(data$high))
data$low <- as.numeric(as.character(data$low))
data$close <- as.numeric(as.character(data$close))
data$volume <- as.numeric(as.character(data$volume))

# Exclude any rows that contain NA, NaN, or Inf values
data <- data[complete.cases(data), ]

# Convert 'open_time' from milliseconds since the epoch to a date-time object
data$open_time <- as.POSIXct(data$open_time / 1000, origin="1970-01-01", tz="UTC")

# Create an xts object for technical analysis (TTR lib)
# candles_df <- as.xts(data) |> suppressWarnings()
candles_df <- data


# Assign technical indicators to the candles
source(paste0(here::here(), "/models/functions/add_ta.R"))
tech_ind <- add_ta(candles_df = candles_df)

# Use the model to predict whether to buy or sell
prediction <- predict(model, newdata = last(tech_ind), type = 'response')

# Output either 1 (buy) or 0 (do not buy)

output <- "hold"
if (prediction > model$optimal_cutoff) {  
  output <- "buy" 
# } else if (prediction < model$optimal_cutoff) {  
#   output <- -1
}

cat(output)

# cat(paste("Optimal hold period:", model$optimal_hold_period, "candles"))

    
