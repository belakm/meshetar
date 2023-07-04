library(TTR)

add_ta <- function(candles_df){
  con_OHLC <- xts::as.xts(OHLCV(candles_df))
  
  close_price <- quantmod::Cl(con_OHLC)
  
  sma <- TTR::SMA(close_price)
  ema <- TTR::EMA(close_price)
  bb_dn <- TTR::BBands(close_price)[,'dn']
  bb_mavg <- TTR::BBands(close_price)[,'mavg']
  bb_up <- TTR::BBands(close_price)[,'up']
  bb_pct_b <- TTR::BBands(close_price)[,'pctB']
  macd <- setNames(TTR::MACD(close_price)[,'macd'], "macd")
  macd_sig <- setNames(TTR::MACD(close_price)[,'signal'], "macd_sig")
  rsi <-  setNames(TTR::RSI(close_price), "rsi") #Relative Strength Index
  # tr <- TTR::ATR(con_OHLC)[,'tr']
  true_high <- TTR::ATR(con_OHLC)[,'trueHigh']
  true_low <- TTR::ATR(con_OHLC)[,'trueLow']
  atr <- TTR::ATR(con_OHLC)[,'atr'] # True Range / Average True Range
  smi <- TTR::SMI(quantmod::HLC(con_OHLC))[,'SMI']
  smi_signal <- setNames(TTR::SMI(quantmod::HLC(con_OHLC))[,'signal'], "smi_signal")
  adx <- TTR::ADX(quantmod::HLC(con_OHLC))[,'ADX']
  adx_dip <- TTR::ADX(quantmod::HLC(con_OHLC))[,'DIp']
  adx_din <- TTR::ADX(quantmod::HLC(con_OHLC))[,'DIn']
  dx <- TTR::ADX(HLC(con_OHLC))[,'DX']
  aroon <- TTR::aroon(con_OHLC[,c('high','low')])[,'oscillator']
  aroon_up <- TTR::aroon(con_OHLC[,c('high','low')])[,'aroonUp']
  aroon_dn <- TTR::aroon(con_OHLC[,c('high','low')])[,'aroonDn']
  chaikin_volatility <- setNames(quantmod::Delt(TTR::chaikinVolatility(con_OHLC[,c("high","low")]))[,1], "chaikin_volatility")
  # emv <- TTR::EMV(cbind(con_OHLC[,c('high','low')]), candles_df[,'volume'])[,'emv']
  # ma_emv <- setNames(as.xts(EMV(cbind(con_OHLC[,c('high','low')]), con[,'volume'])[,'maEMV']),nm =  "ma_emv")
  mfi <- setNames(xts::as.xts(TTR::MFI(con_OHLC[,c("high","low","close")], candles_df[,'volume'])), "mfi")
  sar <- TTR::SAR(con_OHLC[,c('high','close')]) [,1] # Parabolic Stop-and-Reverse
  volat <- setNames(TTR::volatility(con_OHLC,calc="close"), "volat")
  #bullish indicator
  ma20 <- TTR::SMA(close_price, n = 20)
  ma50 <- TTR::SMA(close_price, n = 50)
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
                 #  tr, 
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
                   volat, 
                   ma20, 
                   ma50, 
                   bullish,
                   volume)
  
  TA <- xts::reclass(TA, suppressWarnings(xts::as.xts(candles_df)))
  return(TA)
}

