#' Finding local minima
#'
#' @param time_series 
#' @param window_length 
#'
#' @return 
#' @export
#'
#' @examples
find_local_minima <- function(x = binance_kline, threshold = holding_period){
up   <- sapply(1:threshold, function(n) c(x[-(seq(n))], rep(NA, n)))
down <-  sapply(-1:-threshold, function(n) c(rep(NA,abs(n)), x[-seq(length(x), length(x) - abs(n) + 1)]))
a    <- cbind(x,up,down)
list(minima = which(apply(a, 1, min) == a[,1]), maxima = which(apply(a, 1, max) == a[,1]))
}
#' #' 
#' find_local_minima <- function(time_series = binance_kline, window_length = holding_period) {
#'   num_points <- length(time_series)
#'   local_minima <- numeric(num_points - window_length + 1)
#'   
#'   for (i in 1:(num_points - window_length + 1)) {
#'     window_data <- time_series[i:(i + window_length - 1)]
#'     
#'     # Calculate the first-order differences
#'     diff_data <- diff(window_data)
#'     
#'     # Find the index of the local minimum in the differences
#'     min_index_diff <- which.min(diff_data)
#'     
#'     # Calculate the index in the original window
#'     min_index_original <- min_index_diff + 1
#'     
#'     # Check if the minimum index is at the center
#'     if (min_index_original == ceiling(round(window_length / 2))) {
#'       local_minima[i + min_index_original - 1] <- window_data[min_index_original]
#'     }
#'   }
#'   
#'   # Remove zero values and return
#'   local_minima <- local_minima[local_minima != 0]
#'   
#'   return(local_minima)
#' }


# find_local_minima <- function(time_series = binance_kline, window_length = holding_period) {
#   num_points <- length(time_series)
#   local_minima <- vector("list", num_points - window_length + 1)
#   
#   for (i in 1:(num_points - window_length + 1)) {
#     window_data <- time_series[i:(i + window_length - 1)]
#     
#     min_index <- which.min(window_data)
#     
#     if (min_index == ceiling(round(window_length / 2))) {
#       local_minima[[i + min_index - 1]] <- window_data[min_index] 
#     }
#   }
#   
#   local_minima <- round(
#     unlist(local_minima, use.names = FALSE)
#   )
#   return(local_minima)
# }