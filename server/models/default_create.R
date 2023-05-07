pacman::p_load(RSQLite, TTR, quantmod, xgboost,ROCR, Information, PerformanceAnalytics, rpart, randomForest, caret, tidyverse, here)

here::i_am("server/models/default_create.R")

# Connect to the SQLite database
conn <- dbConnect(RSQLite::SQLite(), "database.sqlite")

# Query the klines table and retrieve the historical data
query <- "SELECT * FROM klines WHERE symbol = 'BTCUSDT' ORDER BY open_time ASC"
data <- dbGetQuery(conn, query)

# Exclude any rows that contain NA, NaN, or Inf values
data <- data[complete.cases(data), ]

# create an xts object for technical analysis (TTR lib)
candles_df <- as.xts(data) |> suppressWarnings()

source(paste0(here::here(), "/server/models/funs/optimal_trading_signal.R"))
source(paste0(here::here(), "/server/models/funs/add_ta.R"))

# Find the target (optimal signal)
optimal_signal_params <- optimal_trading_signal(candles_df, max_holding_period = 8*60) # 4 hours maximum possible hold
signal <- optimal_signal_params$signals

# Assign technical indicators to the candles
tech_ind <- add_ta(candles_df = candles_df)

# Find the number of omitted cases due to MA calculations due to lags
how_many_ommited <- sum(!complete.cases(cbind(signal, tech_ind)))

# create a modelling dataframe
signal_with_TA <- cbind(signal, tech_ind)

# We are interested in buy signals only (optimal holding time is fixed)
signal_with_TA$signal <- ifelse(signal_with_TA$signal == -1, 0, signal_with_TA$signal)

# create a train set index for model training
train_index <- how_many_ommited:round(0.8*(nrow(signal_with_TA)-how_many_ommited))
train <- signal_with_TA[train_index,]

#####################################################################
####  XGboost model for feature importance and feature selection ####
#####################################################################

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
  subset_dtrain <- xgb.DMatrix(subset_data, label = train$signal)
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
important_train <- train[, c("signal", 
                             importance$Feature[which(cv_results$error_rate > median(cv_results$error_rate))])]

features_to_remove <- caret::findCorrelation(
  cor(important_train)
)

reduced_important_train <- important_train[, -features_to_remove]

#for the model i will be using features above median of cross validation error rate
formula_str <- paste("signal ~",paste(colnames(reduced_important_train[,-1]), collapse = " + "))
model_formula  <- as.formula(formula_str)

model <- glm(formula = model_formula ,
                        data = train,
                        family = binomial(link = "logit"))
# save the optimal holding period to the model object
model$optimal_hold_period <- optimal_signal_params$opt_hold_period


########################### 
### Find optimal cutoff ###
###########################

test <- signal_with_TA[max(train_index):nrow(signal_with_TA),]
predictions <- predict(model, test,
                       type = 'response')

# If there are no buy signals in the test period
if(all(test$signal == 0)){
  # set it to the median  of predicted probability
  model$optimal_cutoff <- median(predictions)
} else {
  # Find optimal cut
  pred <- pROC::roc(as.numeric(test[,"signal"]), predictions)
  model$optimal_cutoff <- optimal_cutoff <- pROC::coords(pred, "best", ret = "threshold", input.sort = FALSE)
}



# Save the trained model to a file
saveRDS(model, "server/models/prediction_model.rds")

# Disconnect from the database
dbDisconnect(conn)
