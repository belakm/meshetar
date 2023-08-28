source(
  paste0(
    here::here(),
    "/models/functions/find_local_minima.R"
  )
)

optimal_trading_signal <- function(binance_kline, 
                                   buy_threshold = 0.01,
                                   sell_threshold = -0.01,
                                   fee_rate = 0.015, 
                                   max_holding_period) {
  
  roc <- diff(binance_kline$close)
  roc[is.na(roc)] <- 0

  # Calculate the returns based on the holding period
  returns <- c(rep(0, length(roc)))

  # First create a function for i in max_holding period (function to optimize)
  create_signals <- function(holding_period){
    if(holding_period == 1){
      returns <- roc
    } else{
      for (i in holding_period:length(returns)) {
        if (i <= holding_period) {
          returns[i] <- sum(roc[2:i])
        } else {
          returns[i] <- sum(roc[(i-holding_period+1):(i)])
        }
      }
    }

    # Create a vector to store the trading signals
    signals <- rep(0, length(returns))

    ## Calculate the optimal buying, holding, and selling signals
    # sell when the returns were the highest
    tryCatch({
      for (i in (holding_period):(length(returns))) {
        # sell when return was above fee + min_profit
        if (returns[i] > (buy_threshold + fee_rate)) {
          signals[i] <- -1

          # Add buy signals for every sell - holding_period (if there are no other buy signals)
          signals[i - holding_period] <- 1
        }
      }
    }, error = function(e) {
      if (grepl("No history fetched in database", conditionMessage(e))) {
        # Handle the error caused by missing history in the database
        # You can print an error message, set default values, or perform other actions.
        print("Error: No history fetched in database")
        # Additional error handling code can go here
      } else {
        # Handle other errors that might occur during the execution of the code
        stop("An unexpected error occurred: ", conditionMessage(e))
      }
    })


    # Apply the trading fees to the returns
    actual_returns <- rep(0, length(returns))
    for (i in 1:length(signals)) {
      if (signals[i] == 1) {
        actual_returns[i] <- returns[i] - fee_rate*returns[i]
      } else if (signals[i] == -1) {
        actual_returns[i] <- returns[i] - fee_rate*returns[i]
      }
    }

    # Calculate the cumulative returns
    cumulative_returns <- cumsum(actual_returns)

    return(list(signals = signals,
                net_returns = returns,
                cumulative_returns = cumulative_returns))
  }

  # Loop over all holding periods
  find_optimal_hodl <- NULL; for(i in 1:max_holding_period){
    holding_period_i <- create_signals(holding_period = i)
    find_optimal_hodl <- rbind(find_optimal_hodl,
                               tail(holding_period_i$cumulative_returns,1))
  }

  # Find optimal holding_period by position of max cumulative return
  optimal_hodl <- which(find_optimal_hodl == max(find_optimal_hodl))
  if(length(optimal_hodl) > 1){ stop("More than 1 optimal holding_period (probably no signal was created)")}

  # Final output
  # optimal_signals_with_optimal_hodl <- create_signals(holding_period = optimal_hodl)
  # optimal_signals_with_optimal_hodl$signals <- ifelse(
  #   optimal_signals_with_optimal_hodl$signals == 1,
  #   yes = 0,
  #   no = optimal_signals_with_optimal_hodl$signals
  #   )
  optimal_signals_with_optimal_hodl <- list()
  
  # create all hold signals
  optimal_signals_with_optimal_hodl$signals <- rep(0, nrow(binance_kline))
  local_minima <- find_local_minima(x = binance_kline$close, 
                                    threshold = 20)
  optimal_signals_with_optimal_hodl$signals[local_minima$maxima] <- -1 
  optimal_signals_with_optimal_hodl$signals[local_minima$minima] <- 1 
  
  
  # Find the indices of -1 in the time series
  negative_one_indices <- which(optimal_signals_with_optimal_hodl$signals == -1)
  for(i in negative_one_indices){
    negative_one_indices[i] <- i-1
  }
  
  # optimal_signals_with_optimal_hodl$signals[negative_one_indices] <- -1
  # add element of optimal hold to the list
  optimal_signals_with_optimal_hodl$opt_hold_period = optimal_hodl
  
  # Return the trading signals and the cumulative returns
  return(optimal_signals_with_optimal_hodl)  

}

