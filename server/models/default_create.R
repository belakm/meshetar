# library("RSQLite")
# library("TTR")
# library("quantmod")
# library("xgboost")
# library("ROCR")
# library("Information")
# library("PerformanceAnalytics")
# library("rpart")
# library("randomForest")
# library("dplyr")
# library("magrittr")
# library("here")
# library("ggplot2")
# library("svglite")
pacman::p_load(RSQLite, TTR, quantmod, xgboost, ROCR, Information, PerformanceAnalytics,
               rpart, randomForest, dplyr, magrittr, here, ggplot2, svglite)
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
data <- data[complete.cases(data), ]
# Convert 'open_time' from milliseconds since the epoch to a date-time object
# data$open_time <- as.POSIXct(data$open_time / 1000, origin="1970-01-01", tz="UTC")

# Create an xts object for technical analysis (TTR lib)
# candles_df <- as.xts(data) |> suppressWarnings()
candles_df <- data

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

# Find the number of omitted cases due to MA calculations due to lags
how_many_ommited <- sum(!complete.cases(cbind(signal, tech_ind[-1,])))

if(length(signal) != nrow(tech_ind)){
  signal <- c(0, signal) # add 0 as the first signal.
}

# create a modelling data frame
signal_with_TA <- cbind(signal, tech_ind)

# We are interested in buy signals only (optimal holding time is fixed)
signal_with_TA$signal <- ifelse(signal_with_TA$signal == -1, 0, signal_with_TA$signal)

# create a train set index for model training
train_index <- how_many_ommited:round(0.8*(nrow(signal_with_TA)-how_many_ommited))
train <- signal_with_TA[train_index,]

#####################################################################
####  XGboost model for feature importance and feature selection ####
#####################################################################
xgb_train <- xgb.DMatrix(data = as.matrix(train[,-1]), 
                         label = train$signal) 

# Do we really need xgb test?
# xgb_test <- xgb.DMatrix(data = as.matrix(test[,-1]),
#                     label = test$signal) #100 observations

# Set a seed for reproducibility
set.seed(123)
cv <- xgb.cv(data = xgb_train, nfold = 5,
             objective = "binary:logistic", 
             nrounds = 100, 
             early_stopping_rounds = 10,
             maximize = FALSE, verbose = FALSE)

# Get optimal number of rounds
nrounds <- which.min(cv$evaluation_log$test_logloss_mean)

# Train XGBoost model with optimal hyperparameters
bst_select <- xgboost(data = xgb_train, 
                      nrounds = nrounds,
                      max_depth = 3, 
                      eta = 0.1, 
                      objective = "binary:logistic", 
                      verbose = FALSE)
importance <- xgb.importance(feature_names = colnames(xgb_train), model = bst_select)

# Find optimal number of model features
# Define the range of k values to test (all potential features)
k_values <- 1:nrow(importance)

# Create a data frame to store the cross-validation results
cv_results <- data.frame(k = k_values, error_rate = rep(NA, length(k_values)))

params <- list(
  objective = "binary:logistic",
  eval_metric = "logloss",
  max_depth = 5,
  eta = 0.3,
  subsample = 0.7,
  colsample_bytree = 0.7,
  min_child_weight = 3
)
# Perform cross-validation for each value of k
for (i in seq_along(k_values)) {
  k <- k_values[i]
  # Get the top k features based on XGBoost importance
  top_features <- unlist(importance[1:k, "Feature"])
  # Subset the data to include only the top k features
  subset_data <- train[, top_features, drop = FALSE]
  # Train an XGBoost model using the subset data
  subset_dtrain <- xgb.DMatrix(as.matrix(subset_data), label = train$signal)
  cv_error <- xgb.cv(params, subset_dtrain, nfold = 5, nrounds = 10, 
                     metrics = "error", verbose = FALSE)
  # Store the cross-validation error rate for this value of k
  cv_results[i, "error_rate"] <- min(cv_error$evaluation_log$test_error_mean)
}
cv_results <- cv_results[order(cv_results$error_rate, decreasing = TRUE) ,]

options(scipen = 3)
# order by gain metric
importance <- importance[order(importance$Gain, decreasing = TRUE), ]

# Feature selection
# important_train <- train[, c("signal", 
#                              importance$Feature[which(cv_results$error_rate > median(cv_results$error_rate))])]

# features_to_remove <- caret::findCorrelation(
#   cor(important_train)
# )
# 
# reduced_important_train <- important_train[, -features_to_remove]

#for the model i will be using features above median of cross validation error rate
formula_str <- paste("signal ~",paste(colnames(train[,-1]), collapse = " + "))
model_formula  <- as.formula(formula_str)
suppressWarnings(
  model <- glm(formula = model_formula ,
                          data = train,
                          family = binomial(link = "logit")))

# save the optimal holding period to the model object
model$optimal_hold_period <- optimal_signal_params$opt_hold_period

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
