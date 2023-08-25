#' Auto distributed lag
#'
#' @param data A data.frame used for model building
#' @param n_lags Number of lags used on all of the variables
#'
#' @return data.frame
#' @import dplyr
#' @importFrom purrr map
#' @importFrom purrr map2
#' @importFrom purrr imap_dfc
#' @importFrom purrr set_names
#' @importFrom tidyr everything
#' @importFrom tidyr nest
#' @importFrom tidyr unnest
#' @export
#'
#' @examples
#'
#'adl(mtcars, n_lags = 2)

adl <- function(data, n_lags = 3){
  lags <- ALL_DATA <- NULL #Define global variables as NULL for later use

  if(n_lags == 0){
    return(data)
  }
  #Creating all combination lags for variables in dataset
  lags_dataset <- data %>%
    tidyr::nest(data = tidyr::everything()) %>%
    dplyr::mutate(lags = purrr::map(data, function(dat) {suppressWarnings(
      purrr::imap_dfc(dat[-1], ~purrr::set_names(purrr::map(1:n_lags, dplyr::lag, x = .x),
                                          paste0(.y, '_lag', 1:n_lags))))
    })) %>%
    dplyr::mutate(ALL_DATA = purrr::map2(.x = data, .y = lags, dplyr::bind_cols)) %>%
    dplyr::select(ALL_DATA) %>%
    tidyr::unnest(ALL_DATA)
  return(lags_dataset)
}
