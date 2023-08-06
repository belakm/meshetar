library("RSQLite")
library("TTR")
library("xts")
library("quantmod")
library("here")
# pacman::p_load(RSQLite, TTR, xts, quantmod, here)
suppressMessages(
  here::i_am("models/default_create.R")
)

# Connect to the SQLite database
conn <- dbConnect(RSQLite::SQLite(), "database.sqlite")

# Load the trained model from the file
model <- readRDS("models/prediction_model.rds")

# Query the klines table and retrieve the latest data for the chosen crypto pair
query <- "SELECT datetime(open_time / 1000, 'unixepoch') AS open_time,
                 high, 
                 low, 
                 close, 
                 volume
          FROM klines
          WHERE symbol = 'BTCUSDT'
          ORDER BY open_time DESC
          LIMIT 50;"
data <- dbGetQuery(conn, query)
# Disconnect from the database
dbDisconnect(conn)


rownames(data) <- as.POSIXct(data$open_time)

# data$open <- as.numeric(as.character(data$open))
# data$high <- as.numeric(as.character(data$high))
# data$low <- as.numeric(as.character(data$low))
# data$close <- as.numeric(as.character(data$close))
# data$volume <- as.numeric(as.character(data$volume))

# Exclude any rows that contain NA, NaN, or Inf values
data <- data[complete.cases(data), ]

# Create an xts object for technical analysis (TTR lib)
# candles_df <- as.xts(data) |> suppressWarnings()
candles_df <- data


# Assign technical indicators to the candles
source(paste0(here::here(), "/models/functions/add_ta.R"))
tech_ind <- add_ta(candles_df = candles_df)

# Use the model to predict whether to buy or sell
suppressWarnings(
  prediction <- predict(model, newdata = last(tech_ind), type = 'response')
)

# Output either 1 (buy) or 0 (do not buy)

output <- "hold"
if (prediction > model$optimal_cutoff) {  
  output <- "buy" 
# } else if (prediction < model$optimal_cutoff) {  
#   output <- -1
}
cat(output)
# cat(paste("Optimal hold period:", model$optimal_hold_period, "candles"))

    
