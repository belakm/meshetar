add_ta <- function(candles_df){
  suppressMessages(
  con_OHLC_quantmod <- quantmod::OHLCV(candles_df)
  )

  # Assign open_time to rownames, this is how as.xts input works.s
  rownames(con_OHLC_quantmod) <- con_OHLC_quantmod$open_time
  con_OHLC_quantmod$open_time <- NULL
  
  con_OHLC <- xts::as.xts(con_OHLC_quantmod)

  close_price_quantmod <- quantmod::Cl(con_OHLC)
  close_price <- as.numeric(close_price_quantmod)
  
  sma <- TTR::SMA(close_price)
  ema <- TTR::EMA(close_price)
  bb_dn <- TTR::BBands(close_price)[,'dn']
  bb_mavg <- TTR::BBands(close_price)[,'mavg']
  bb_up <- TTR::BBands(close_price)[,'up']
  bb_pct_b <- TTR::BBands(close_price)[,'pctB']
  macd <- setNames(TTR::MACD(close_price)[,'macd'], "macd")
  macd_sig <- setNames(TTR::MACD(close_price)[,'signal'], "macd_sig")
  rsi <-  setNames(TTR::RSI(close_price), "rsi") #Relative Strength Index
  # print("before conOHLC")
  tr <- TTR::ATR(con_OHLC)[,'tr']
  true_high <- TTR::ATR(con_OHLC)[,'trueHigh']
  true_low <- TTR::ATR(con_OHLC)[,'trueLow']
  atr <- TTR::ATR(con_OHLC)[,'atr'] # True Range / Average True Range
  # print("after  ATR")
  smi <- TTR::SMI(quantmod::HLC(con_OHLC))[,'SMI']
  smi_signal <- setNames(TTR::SMI(quantmod::HLC(con_OHLC))[,'signal'], "smi_signal")
  # print("after  SMI")
  adx <- TTR::ADX(quantmod::HLC(con_OHLC))[,'ADX']
  adx_dip <- TTR::ADX(quantmod::HLC(con_OHLC))[,'DIp']
  adx_din <- TTR::ADX(quantmod::HLC(con_OHLC))[,'DIn']
  dx <- TTR::ADX(quantmod::HLC(con_OHLC))[,'DX']
  aroon <- TTR::aroon(con_OHLC[,c('high','low')])[,'oscillator']
  aroon_up <- TTR::aroon(con_OHLC[,c('high','low')])[,'aroonUp']
  aroon_dn <- TTR::aroon(con_OHLC[,c('high','low')])[,'aroonDn']
  # print("after  aroon")
  chaikin_volatility <- setNames(quantmod::Delt(TTR::chaikinVolatility(con_OHLC[,c("high","low")]))[,1], "chaikin_volatility")
  # print("after  chaikin_volatility")
  # emv <- TTR::EMV(
  #   HL = cbind(con_OHLC[,c('high','low')]), 
  #   volume = con_OHLC[,'volume'])[,'emv']
  # print("after  emv")
  # ma_emv <- setNames(as.xts(EMV(cbind(con_OHLC[,c('high','low')]), con_OHLC[,'volume'])[,'maEMV']),nm =  "ma_emv")
  # print("after  ma_emv")
  mfi <- setNames(xts::as.xts(TTR::MFI(con_OHLC[,c("high","low","close")], con_OHLC[,'volume'])), "mfi")
  # print("after  mfi")
  sar <- TTR::SAR(con_OHLC[,c('high','close')]) [,1] # Parabolic Stop-and-Reverse
  # print("after  sar")
  # volat <- setNames(TTR::volatility(con_OHLC), "volat")
  # print("after  volat")

  #bullish indicator
  # print("sma20 before")
  ma20 <- TTR::SMA(close_price, n = 20)
  # print("sma20 after")
  ma50 <- TTR::SMA(close_price, n = 50)
  # print("sma50 after")
  bullish <- ifelse(ma20 > ma50, 1, 0) # Create a bullish dummy variable based on the moving average crossover
  volume <- con_OHLC$volume

  TA <- data.frame(sma,
                   ema,
                   bb_dn,
                   bb_mavg, 
                   bb_up, 
                   bb_pct_b, 
                   macd, 
                   macd_sig, 
                   rsi,
                   tr, 
                   true_high, 
                   true_low, atr, 
                   smi, 
                   smi_signal, 
                   adx, 
                   adx_dip,  
                   adx_din, 
                   dx,  
                   aroon,
                   aroon_up, 
                   aroon_dn, 
                   chaikin_volatility, 
                   # emv, 
                   # ma_emv, 
                   mfi,
                   sar, 
                   # volat, 
                   ma20, 
                   ma50, 
                   bullish,
                   volume)
  # print("dataframe TA constructed")
  TA <- xts::reclass(TA, con_OHLC_quantmod)
  return(TA)
}

