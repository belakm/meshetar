suppressMessages(
  here::i_am("models/default_create.R")
)

## Connect to the SQLite database
conn <- DBI::dbConnect(RSQLite::SQLite(), "database.sqlite")

# Query the klines table and retrieve the historical data
# query <- "SELECT datetime(open_time / 1000, 'unixepoch') AS open_time,
#                  high, 
#                  low, 
#                  close, 
#                  volume
#           FROM candles 
#           WHERE asset = 'BTCUSDT'
#           ORDER BY open_time ASC;"

query <- "SELECT strftime('%s', open_time) AS open_time,
                 high, 
                 low, 
                 close, 
                 volume
          FROM candles 
          WHERE asset = 'BTCUSDT'
          ORDER BY open_time ASC;"

data <- DBI::dbGetQuery(conn, query)

print(data)

# Disconnect from the database
DBI::dbDisconnect(conn)
data$open_time <- as.POSIXct(as.numeric(data$open_time), origin="1970-01-01", tz="UTC")
rownames(data) <- as.POSIXct(data$open_time)

# Exclude any rows that contain NA, NaN, or Inf values
candles_df <- data[complete.cases(data), ]
# Convert 'open_time' from milliseconds since the epoch to a date-time object
# data$open_time <- as.POSIXct(data$open_time / 1000, origin="1970-01-01", tz="UTC")

# Create an xts object for technical analysis (TTR lib)
# candles_df <- as.xts(data) |> suppressWarnings()

source(paste0(here::here(), "/models/functions/optimal_trading_signal.R"))
source(paste0(here::here(), "/models/functions/add_ta.R"))

half_the_candles <- round(nrow(candles_df)/2)
quarter_the_candles <- round(half_the_candles/2)
one_eight_the_candles <- round(quarter_the_candles/2)

# Find the target (optimal signal)
optimal_signal_params <- optimal_trading_signal(
  candles_df, 
  max_holding_period = quarter_the_candles) 

signal <- optimal_signal_params$signals

# Assign technical indicators to the candles
tech_ind <- add_ta(candles_df = candles_df)
# print("tech_ind_success")

if(length(signal) != nrow(tech_ind)){
  signal <- c(0, signal) # add 0 as the first signal.
}

# Find the number of omitted cases due to MA calculations due to lags
how_many_ommited <- 1:sum(!complete.cases(cbind(signal, tech_ind)))

####  normalization of the tech_ind dataframe  ####

tech_ind_normal <- as.data.frame(
  apply(tech_ind, 2, scale)
) 

signal_str <- ifelse(signal == 1, "buy",
                     ifelse(signal == 0, "hold",
                            ifelse(signal == -1, "sell", "unknown")))

class.ind <- function(cl)
{
  n <- length(cl)
  cl <- as.factor(cl)
  x <- matrix(0, n, length(levels(cl)) )
  x[(1:n) + n*(unclass(cl)-1)] <- 1
  dimnames(x) <- list(names(cl), levels(cl))
  as.data.frame(x)
}

# create a modelling data frame
signal_with_TA <- cbind(class.ind(signal_str), 
                        tech_ind_normal)[-how_many_ommited,]

# create a train set index for model training
train_index <- 1:round(0.7*(nrow(signal_with_TA)))
train <- signal_with_TA[train_index,]

y_labs <- c("buy", "hold", "sell")
x_train <- train[, !colnames(train) %in% y_labs]
y_train <-  train[, colnames(train) %in% y_labs]


test <- signal_with_TA[-train_index,]
x_test <- test[, !colnames(test) %in% y_labs]
y_test <- test[, colnames(test) %in% y_labs]

##### Neural Net ###################################
formula_str <- paste(
  paste(colnames(y_train), collapse = " + "), 
  "~", 
  paste(colnames(x_train), collapse = " + ")
)

#Mutlithreading:
# suppressWarnings(
#   suppressMessages(
#     h2o::h2o.init(nthreads = -1, log_level = 'WARN')
#   )
# )
# train_h2o <- h2o::as.h2o(train)

# Without multithreading (parallel processing)
train_h2o <- train

nnet_model <- neuralnet::neuralnet(
  formula_str,
  train_h2o, 
  hidden = c(length(x_train)*2, length(x_train)), # 2 hidden layers
  err.fct = "sse", #cross-entropy 'ce', 
  linear.output = FALSE,  # Use softmax activation if FALSE                       
  lifesign = 'full', # change this to 'none', for no logging
  rep = 1, #number of repetitions for the neural network’s training
  algorithm = "rprop+",
  stepmax = 100000) # Boost this for more complex nnet

source(paste0(here::here(), "/models/functions/predict_nnet.R"))

nnet_output <- predict_nnet(nnet_model, test)

# for development: confusion matrix - beyond some accuracy, do not save the model

# Save the trained model to a file-
saveRDS(nnet_model, "models/prediction_model.rds")


# if you are predicting test set:
nnet_output$plot_time <- as.POSIXct(candles_df[-train_index, "open_time"][-how_many_ommited])

# Plot the signals the model was trained on:
plot_trading_signal <- function(ohlc_data, signals, buy = TRUE, sell = TRUE, test_predictions = NULL){
  # For logit:
  # test_predictions <-  data.frame(plot_time = as.POSIXct(names(predictions)),
  #                                 test_predictions = ifelse(predictions > model$optimal_cutoff, 1, 0))

  # candle_hour_minute <- format(as.POSIXct(ohlc_data$open_time),"%H:%M")
  df <- data.frame(plot_time = as.POSIXct(ohlc_data$open_time),
                   plot_price = ohlc_data$close, 
                   plot_signal = c(0,signals$signals))
  df$plot_signal[-train_index] <- NA
  
  df <- merge(df, test_predictions, by= "plot_time", all = TRUE)
  
  ggplot2::ggplot(df, ggplot2::aes(x = plot_time, y = plot_price)) +
    ggplot2::geom_line() +
    ggplot2::geom_point(data = subset(df, plot_signal == 1),
                        ggplot2::aes(x = plot_time, y = plot_price), color = "green", shape = "+", size = 4, stroke = 4) +
    ggplot2::geom_point(data = subset(df, plot_signal == -1),
                        ggplot2::aes(x = plot_time, y = plot_price),
                        color = "red", shape = "-", size = 10, stroke = 4) +
    ggplot2::geom_point(data = subset(df, prediction == "buy"),
                        ggplot2::aes(x = plot_time, y = plot_price),
                        color = "lightgreen", shape = "+", size = 4, stroke = 4) +
    ggplot2::geom_point(data = subset(df, prediction == "sell"),
                        ggplot2::aes(x = plot_time, y = plot_price),
                        color = "orange", shape = "-", size = 4, stroke = 4) +
    ggplot2::labs(x = "Time", y = "Price", title = "Price over time with Buy/Sell Signals", subtitle = paste("Holding period:", signals$opt_hold_period)) +
    ggplot2::scale_x_datetime(breaks = scales::date_breaks(paste(nrow(candles_df)/6, "min")),
                              labels = scales::date_format("%Y-%m-%d %H:%M")) +
    ggplot2::geom_vline(xintercept = df[max(train_index), 'plot_time'], color = "red") +
    ggplot2::theme_dark() +
    ggplot2::theme(axis.text.x = ggplot2::element_text(angle = 45, vjust = 0.1))
}

historical_signal_plot <- plot_trading_signal(
  ohlc_data = candles_df,
  signals =  optimal_signal_params, 
  test_predictions = nnet_output)

# Save the svg plot to the folder /server
suppressMessages(
  ggplot2::ggsave(
    filename = "static/historical_trading_signals_model.svg", 
    plot = historical_signal_plot, 
    device = "svg")
)

cat("Model done")
