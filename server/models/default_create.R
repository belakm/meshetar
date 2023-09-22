print("starting to build the model")
# Clear workspace

# cat("after restarting the session")
suppressMessages(
  here::i_am("models/default_create.R")
)

source("models/functions/igc.R")
igc()

print("1")

## Connect to the SQLite database
conn <- DBI::dbConnect(RSQLite::SQLite(), "database.sqlite")

# Query the klines table and retrieve the historical data
query <- "SELECT datetime(open_time / 1000, 'unixepoch') AS open_time,
                 high, 
                 low, 
                 close, 
                 volume
          FROM klines
          WHERE symbol = 'BTCUSDT';"
data <- DBI::dbGetQuery(conn, query)

# Get fee information
# query_fee <- "SELECT *
#           FROM asset_ticker
#           WHERE symbol = 'BTCUSDT';"
# 
# fee_data <- DBI::dbGetQuery(conn, query_fee)
# Disconnect from the database
DBI::dbDisconnect(conn)

rownames(data) <- as.POSIXct(data$open_time)

# Exclude any rows that contain NA, NaN, or Inf values
candles_df <- data[complete.cases(data), ]
# Convert 'open_time' from milliseconds since the epoch to a date-time object
# data$open_time <- as.POSIXct(data$open_time / 1000, origin="1970-01-01", tz="UTC")

source(paste0(here::here(), "/models/functions/optimal_trading_signal.R"))
source(paste0(here::here(), "/models/functions/add_ta.R"))

half_the_candles <- round(nrow(candles_df)/2)
quarter_the_candles <- round(half_the_candles/2)
one_eight_the_candles <- round(quarter_the_candles/2)
cat("before optimal signal params")
# Find the target (optimal signal)
optimal_signal_params <- optimal_trading_signal(
  candles_df, 
  max_holding_period = one_eight_the_candles) 

igc()

signal <- optimal_signal_params$signals

# Assign technical indicators to the candles
tech_ind <- add_ta(candles_df = candles_df)
# print("tech_ind_success")
igc()

if(length(signal) != nrow(tech_ind)){
  signal <- c(0, signal) # add 0 as the first signal.
}

# Find the number of omitted cases due to MA calculations due to lags
how_many_ommited <- 1:sum(!complete.cases(cbind(signal, tech_ind)))

####  normalization of the tech_ind dataframe  ####

# source("models/functions/min_max_normalization.R")
# Applying MinMax normalization after removing NAs
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


# Convert to zero-based categorical values
signal_zero_based <- signal + 1

# create a modelling data frame
signal_with_TA <- cbind(keras::to_categorical(signal_zero_based), 
                        tech_ind_normal)[-how_many_ommited,]
y_labs <-  c("buy",  "hold", "sell")
colnames(signal_with_TA)[1:3] <- y_labs

# create a train set index for model training
train_index <- 1:round(0.7*(nrow(signal_with_TA)))
train <- signal_with_TA[train_index,]

x_train <- train[, !colnames(train) %in% y_labs & !colnames(train) %in% "signal_str"] |>
  (\(x) keras::array_reshape(x, dim = dim(x)))()

if(any(colnames(train) %in% y_labs)){
  y_train <- train[, colnames(train) %in% y_labs] |>
    (\(x) keras::array_reshape(x, dim = dim(x)))()
  
} else {
  y_train <- data.frame(signal_str = train[,"signal_str"]) |>
    (\(x) keras::array_reshape(x, dim = dim(x)))()
}

test <- signal_with_TA[-train_index,]
x_test <- test[, !colnames(test) %in% y_labs & !colnames(test) %in% "signal_str"] |>
  (\(x) keras::array_reshape(x, dim = dim(x)))()

if(any(colnames(test) %in% y_labs)){
  y_test <- test[, colnames(test) %in% y_labs] |>
    (\(x) keras::array_reshape(x, dim = dim(x)))()
} else {
  y_test <- data.frame(signal_str  = test[, "signal_str"]) |>
    (\(x) keras::array_reshape(x, dim = dim(x)))()
}
igc()

#### KERAS NEURAL NET #####

#clear keras session
keras::k_clear_session()
keras_nn_model <- keras::keras_model_sequential(input_shape = dim(x_train)[2]) |>
  keras::layer_dense(dim(x_train)[2]*2, activation = "relu") |>
  keras::layer_dropout(0.2) |>
  keras::layer_dense(dim(x_train)[2]*2, activation = "relu") |>
  keras::layer_dropout(0.2) |>
  keras::layer_dense(dim(x_train)[2], activation = "relu") |>
    keras::layer_dense(round(dim(x_train)[2]/2), activation = "relu") |>

  keras::layer_dense(ncol(y_train), activation = "sigmoid")

summary(keras_nn_model)

# Calculate class weights manually based on class distribution
class_weights_named <-  1 - as.numeric(prop.table(table(signal_str)))

# Compile the model
keras_nn_model |> keras::compile(
  loss = 'categorical_crossentropy',
  optimizer = keras::optimizer_rmsprop(learning_rate = 0.01),
  metrics = c('accuracy'),
  # loss_weights = class_weights_named  # Specify class weights
)

# Define early stopping callback
early_stopping <- keras::callback_early_stopping(
  monitor = "loss",    # Metric to monitor (usually validation loss)
  patience = 10,            # Number of epochs with no improvement before stopping
  restore_best_weights = TRUE  # Restore model weights from the epoch with the best value
)

# Fit the model
history <- keras_nn_model |> 
  keras::fit(
  x = x_train,
  y = y_train,
  epochs = 30,              # Adjust the number of epochs
  batch_size = 32,          # Adjust the batch size
  validation_split = 0.1,    # Optional: Validation split if you want to monitor validation loss
  callbacks = list(early_stopping), # Include the early stopping callback
  # class_weights = class_weights_named
)

keras_nn_model |>
  keras::evaluate(as.matrix(x_test), as.matrix(y_test), batch_size = 32)

predicted_probs <- keras_nn_model |> 
  predict(as.matrix(x_test))

adjust_conservativeness <- function(predicted_probs, conservative_factor){
  buy_col <- which(y_labs == "buy")
  sell_col <- which(y_labs == "sell")
  
  predicted_probs[,c(buy_col, sell_col)] <- predicted_probs[,c(buy_col, sell_col)] + conservative_factor
  predicted_classes <- apply(predicted_probs, 1, which.max)
  
  return(predicted_classes)
}

predicted_classes <- adjust_conservativeness(predicted_probs, 
                                             conservative_factor = 0.2)
nnet_output <- data.frame(
  prediction = y_labs[predicted_classes])

table(nnet_output)

n_matched_sell = sum(y_test[, 3] & nnet_output$prediction == "sell")
n_missed_sell = sum(y_test[,3] & !nnet_output$prediction == "sell")
n_matched_buy = sum(y_test[,1] & nnet_output$prediction == "buy")
n_missed_buy = sum(y_test[,1] & !nnet_output$prediction == "buy")

print(data.frame(
  n_matched_sell, 
  n_missed_sell, 
  n_matched_buy ,
  n_missed_buy )
)
# Save the trained model to a file-
keras::save_model_hdf5(keras_nn_model, "models/prediction_model.keras")
saveRDS(history, "models/prediction_model.rds")

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
                   plot_signal = signals$signals) # c(0,signals$signals))
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
    ggplot2::theme(axis.text.x = ggplot2::element_text(angle = 45, vjust = 0.1, color = "white"),
                   axis.text.y = ggplot2::element_text(color = "white"),
                   plot.background =  ggplot2::element_rect(fill =  rgb(20/255, 30/255, 38/255)),
                   plot.title = ggplot2::element_text(color = "white"),
                   plot.subtitle = ggplot2::element_text(color = "white"), 
                   axis.title = ggplot2::element_text(color = "white"))
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
    width = 10.68,  # Specify the desired width in inches
    height = 5, 
    limitsize = FALSE,
    device = "svg")
)
cat('Model done')
