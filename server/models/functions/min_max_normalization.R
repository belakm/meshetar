#' MinMax normalization function
#'
#' @param x 
#'
#' @return matrix
#' @export

min_max_normalize <- function(x) {
  if (any(!is.na(x))) {
    (x - min(x, na.rm = TRUE)) / (max(x, na.rm = TRUE) - min(x, na.rm = TRUE))
  } else {
    x
  }
}