suppressMessages(
  here::i_am("models/default_run.R")
)

# Connect to the SQLite database
conn <- DBI::dbConnect(RSQLite::SQLite(), "database.sqlite")

# Load the trained model from the file
# model <- readRDS("models/prediction_model.rds")
model <- keras::load_model_hdf5("models/prediction_model.h5")
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
data <- DBI::dbGetQuery(conn, query)
# Disconnect from the database
DBI::dbDisconnect(conn)

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



# KERAS preparation
# Applying MinMax normalization after removing NAs
source(paste0(here::here(), "/models/functions/min_max_normalization.R"))
tech_ind_normal <- as.matrix(
  apply(tech_ind, 2, min_max_normalize)
) |> keras::array_reshape(dim = dim(tech_ind))

# Use the model to predict whether to buy or sell

source(paste0(here::here(), "/models/functions/predict_nnet.R"))

suppressWarnings(
  prediction <- predict_nnet(nn_model = model, data_to_predict = tail(tech_ind_normal, 1))
)
output <- unlist(prediction)

# Output either 1 (buy) or 0 (do not buy)
# output <- ifelse(prediction == 1, "buy",
#                  ifelse(prediction == 0, "hold",
#                         ifelse(prediction == -1, "sell", "unknown")))
cat(output)
# cat(paste("Optimal hold period:", model$optimal_hold_period, "candles"))

    
