library("quantmod")
library("xgboost")
library("ROCR")
library("Information")
library("PerformanceAnalytics")
library("rpart")
library("randomForest")
library("dplyr")
library("magrittr")
library("here")
library("ggplot2")
library("svglite")
library("neuralnet")

# pacman::p_load(RSQLite, TTR, quantmod, xgboost, ROCR, Information, PerformanceAnalytics,
#                rpart, randomForest, dplyr, magrittr, here, ggplot2, svglite, neuralnet)

suppressMessages(
  here::i_am("models/default_create.R")
)

# Connect to the SQLite database
conn <- dbConnect(RSQLite::SQLite(), "database.sqlite")

# Query the klines table and retrieve the historical data
query <- "SELECT datetime(open_time / 1000, 'unixepoch') AS open_time,
                 high, 
                 low, 
                 close, 
                 volume
          FROM klines
          WHERE symbol = 'BTCUSDT'
          ORDER BY open_time DESC;"
data <- dbGetQuery(conn, query)

# Disconnect from the database
dbDisconnect(conn)

rownames(data) <- as.POSIXct(data$open_time)

# # Calculate the rate of change (ROC) based on the price data
# data$open <- as.numeric(as.character(data$open))
# data$high <- as.numeric(as.character(data$high))
# data$low <- as.numeric(as.character(data$low))
# data$close <- as.numeric(as.character(data$close))
# data$volume <- as.numeric(as.character(data$volume))

# Exclude any rows that contain NA, NaN, or Inf values
candles_df <- data[complete.cases(data), ]
# Convert 'open_time' from milliseconds since the epoch to a date-time object
# data$open_time <- as.POSIXct(data$open_time / 1000, origin="1970-01-01", tz="UTC")

# Create an xts object for technical analysis (TTR lib)
# candles_df <- as.xts(data) |> suppressWarnings()

source(paste0(here::here(), "/models/functions/optimal_trading_signal.R"))
source(paste0(here::here(), "/models/functions/add_ta.R"))

half_the_candles <- nrow(candles_df)/2

# Find the target (optimal signal)
optimal_signal_params <- optimal_trading_signal(
  candles_df, 
  max_holding_period = half_the_candles) 

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

# create a modelling data frame
signal_with_TA <- cbind(signal = signal, 
                        tech_ind_normal)[-how_many_ommited,]

# create a train set index for model training
train_index <- 1:round(0.7*(nrow(signal_with_TA)))
train <- signal_with_TA[train_index,]

x_train <- train[,-which(names(train) == "signal")]
y_train <- train[,"signal"]


test <- signal_with_TA[-train_index,]
x_test <- test[,-which(names(train) == "signal")]
y_test <- test[,"signal"]

##### Neural Net ###################################
formula_str <- paste("signal ~",paste(colnames(x_train), collapse = " + "))
library(h2o) # parallel computing
h2o.init(nthreads = -1)
train_h2o <- as.h2o(train)

nnet_model <- neuralnet(formula_str,
                        train_h2o, 
                        hidden = length(x_train)*2,
                        err.fct = "sse", #cross-entropy, 
                        linear.output = TRUE,
                        lifesign = 'full', # change this to 'none', for no output
                        rep = 2, #number of repetitions for the neural network’s training
                        algorithm = "rprop+",
                        stepmax = 100000) # logit probability

which_rep <- which(nnet_model$result.matrix[1, ]== min(nnet_model$result.matrix[1, ])) 
output <- compute(nnet_model, 
                  rep = which_rep, 
                  x_train)$net.result


tab1 <- table(ifelse(output > 0.5, 1, 0), train$signal)

tab1

########################### 
### Find optimal cutoff ###
###########################

test <- signal_with_TA[(max(train_index)+1):nrow(signal_with_TA),]

suppressWarnings(
  predictions <- predict(model, test,
                         type = 'response')
)

# If there are no buy signals in the test period
if(all(test$signal == 0)){
  # set it to the median  of predicted probability
  model$optimal_cutoff <- median(predictions)
} else {
  # Find optimal cut
  suppressMessages(
    pred <- pROC::roc(as.numeric(test[,"signal"]), predictions)
  )
  model$optimal_cutoff <- optimal_cutoff <- pROC::coords(pred, "best", ret = "threshold", input.sort = FALSE)
}

# Save the trained model to a file-
saveRDS(model, "models/prediction_model.rds")


# Plot the signals the model was trained on:
plot_trading_signal <- function(ohlc_data, signals, buy = TRUE, sell = TRUE){
  
  test_predictions <-  data.frame(plot_time = as.POSIXct(names(predictions)),
                                  test_predictions = ifelse(predictions > model$optimal_cutoff, 1, 0))
  
  # candle_hour_minute <- format(as.POSIXct(ohlc_data$open_time),"%H:%M")
  df <- data.frame(plot_time = as.POSIXct(ohlc_data$open_time),
                   plot_price = ohlc_data$close, 
                   plot_signal = c(0,signals$signals))
  
  df <- merge(df, test_predictions, by= "plot_time", all = TRUE)
  
  print(tail(df))
  ggplot2::ggplot(df, ggplot2::aes(x = plot_time, y = plot_price)) +
    ggplot2::geom_line() +
    ggplot2::geom_point(data = subset(df, plot_signal == 1),
                        ggplot2::aes(x = plot_time, y = plot_price), color = "green", shape = "+", size = 4, stroke = 4) +
    ggplot2::geom_point(data = subset(df, plot_signal == -1),
                        ggplot2::aes(x = plot_time, y = plot_price),
                        color = "red", shape = "-", size = 10, stroke = 4) +
    ggplot2::geom_point(data = subset(df, test_predictions == 1),
                        ggplot2::aes(x = plot_time, y = plot_price),
                        color = "blue", shape = "+", size = 4, stroke = 4) +
    ggplot2::labs(x = "Time", y = "Price", title = "Price over time with Buy/Sell Signals", subtitle = paste("Holding period:", signals$opt_hold_period)) +
    ggplot2::scale_x_datetime(breaks = scales::date_breaks(paste(nrow(candles_df)/6, "min")), 
                              labels = scales::date_format("%Y-%m-%d %H:%M")) +
    geom_vline(xintercept = df[max(train_index), 'plot_time'], color = "red") +
    theme(axis.text.x = element_text(angle = 45, vjust = 0.1))
}

historical_signal_plot <- plot_trading_signal(
  ohlc_data = candles_df,
  signals =  optimal_signal_params)

# Save the svg plot to the folder /server
suppressMessages(
  ggplot2::ggsave(
    filename = "historical_trading_signals_model_was_trained", 
    plot = historical_signal_plot, 
    device = "svg")
)

cat("Model done")
