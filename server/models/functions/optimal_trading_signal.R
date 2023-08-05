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
          returns[i] <- sum(roc[(i-holding_period+1):(i
                                                    )]) 
        }
      }
    }
    
    # Create a vector to store the trading signals
    signals <- rep(0, length(returns))
    
    ## Calculate the optimal buying, holding, and selling signals
    # sell when the returns were the highest
    for (i in (holding_period):(length(returns))) {
      # sell when return was above fee + min_profit
      if(returns[i] > (buy_threshold + fee_rate)) {
          signals[i] <- -1  
          
          # Add buy signals for every sell - holding_period (if there are no other buy signals)
          signals[i-holding_period] <- 1
      }
    }

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
  optimal_signals_with_optimal_hodl <- create_signals(holding_period = optimal_hodl)
  
  # add element of optimal hold to the list
  optimal_signals_with_optimal_hodl$opt_hold_period = optimal_hodl
  
  # Return the trading signals and the cumulative returns
  return(optimal_signals_with_optimal_hodl)  

}

